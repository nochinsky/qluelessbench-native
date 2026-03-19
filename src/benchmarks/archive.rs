//! Archive benchmark tests.
//!
//! Tests ZIP and TAR creation and extraction.

use anyhow::Result;
use rayon::prelude::*;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::time::Instant;
use tempfile::TempDir;
use zip::write::ZipWriter;
use zip::CompressionMethod;

use crate::benchmarks::base::{
    calculate_category_score, get_parallel_workers, run_with_iterations, BaseBenchmark,
};
use crate::results::CategoryResult;

/// Archive benchmark.
pub struct ArchiveBenchmark {
    /// If true, run tests in parallel (multi-core mode)
    multi_core: bool,
}

impl ArchiveBenchmark {
    /// Create a new ArchiveBenchmark.
    pub fn new() -> Self {
        ArchiveBenchmark { multi_core: false }
    }

    /// Create a new ArchiveBenchmark for multi-core testing.
    pub fn new_multi_core() -> Self {
        ArchiveBenchmark { multi_core: true }
    }

    /// Generate test files.
    fn generate_test_files(
        dir: &TempDir,
        count: usize,
        size_kb: usize,
    ) -> Result<Vec<std::path::PathBuf>> {
        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut files = Vec::new();

        for i in 0..count {
            let file_path = dir.path().join(format!("test_file_{}.bin", i));
            let mut file = File::create(&file_path)?;
            let mut data = vec![0u8; size_kb * 1024];
            rng.fill(&mut data[..]);
            file.write_all(&data)?;
            files.push(file_path);
        }

        Ok(files)
    }

    /// Test ZIP creation.
    fn test_zip_create(files: &[std::path::PathBuf], output_path: &std::path::Path) -> Result<f64> {
        let start = Instant::now();

        let zip_file = File::create(output_path)?;
        let mut zip = ZipWriter::new(zip_file);

        for file_path in files {
            let file_name = file_path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("unknown");
            zip.start_file(
                file_name,
                zip::write::FileOptions::default().compression_method(CompressionMethod::Deflated),
            )?;

            let mut file = File::open(file_path)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
        }

        zip.finish()?;
        let duration = start.elapsed().as_secs_f64();

        let total_size: u64 = files
            .iter()
            .filter_map(|p| fs::metadata(p).ok())
            .map(|m| m.len())
            .sum();

        Ok(total_size as f64 / 1024.0 / 1024.0 / duration)
    }

    /// Test ZIP extraction.
    fn test_zip_extract(zip_path: &std::path::Path, extract_dir: &TempDir) -> Result<f64> {
        let start = Instant::now();

        let zip_file = File::open(zip_path)?;
        let mut archive = zip::ZipArchive::new(zip_file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let out_path = extract_dir.path().join(file.name());
            let mut outfile = File::create(&out_path)?;
            std::io::copy(&mut file, &mut outfile)?;
        }

        let duration = start.elapsed().as_secs_f64();
        Ok(1.0 / duration)
    }

    /// Test TAR creation.
    fn test_tar_create(files: &[std::path::PathBuf], output_path: &std::path::Path) -> Result<f64> {
        let start = Instant::now();

        let tar_file = File::create(output_path)?;
        let mut tar = tar::Builder::new(tar_file);

        for file_path in files {
            let file_name = file_path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("unknown");
            tar.append_path_with_name(file_path, file_name)?;
        }

        tar.finish()?;
        let duration = start.elapsed().as_secs_f64();

        let total_size: u64 = files
            .iter()
            .filter_map(|p| fs::metadata(p).ok())
            .map(|m| m.len())
            .sum();

        Ok(total_size as f64 / 1024.0 / 1024.0 / duration)
    }

    /// Test TAR extraction.
    fn test_tar_extract(tar_path: &std::path::Path, extract_dir: &TempDir) -> Result<f64> {
        let start = Instant::now();

        let tar_file = File::open(tar_path)?;
        let mut archive = tar::Archive::new(tar_file);
        archive.unpack(extract_dir.path())?;

        let duration = start.elapsed().as_secs_f64();
        Ok(1.0 / duration)
    }

    /// Test parallel ZIP creation.
    /// Throughput model: Each worker creates a FULL archive (10 files * 100KB).
    /// N workers create N× the archives in roughly the same time = N× speedup.
    fn test_parallel_zip_create(
        num_workers: usize,
        num_files: usize,
        file_size_kb: usize,
    ) -> Result<f64> {
        let start = Instant::now();
        let mut temp_dirs = Vec::new();
        let mut output_paths = Vec::new();

        // Each worker creates their own files and archive
        for i in 0..num_workers {
            let temp_dir = TempDir::new()?;
            let files = Self::generate_test_files(&temp_dir, num_files, file_size_kb)?;
            let output_path = temp_dir.path().join(format!("test_{}.zip", i));
            temp_dirs.push(temp_dir);
            output_paths.push(output_path);

            // Create ZIP with this worker's files
            let zip_file = File::create(&output_paths[i])?;
            let mut zip = ZipWriter::new(zip_file);

            for file_path in &files {
                let file_name = file_path.file_name().unwrap().to_str().unwrap();
                zip.start_file(
                    file_name,
                    zip::write::FileOptions::default()
                        .compression_method(CompressionMethod::Deflated),
                )?;

                let mut file = File::open(file_path)?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                zip.write_all(&buffer)?;
            }

            zip.finish()?;
        }

        let duration = start.elapsed().as_secs_f64();
        let total_size = (num_workers * num_files * file_size_kb * 1024) as f64;
        Ok(total_size / 1024.0 / 1024.0 / duration)
    }

    /// Test parallel ZIP extraction.
    /// Same total work as single-core (10 files), split across workers.
    fn test_parallel_zip_extract(
        num_workers: usize,
        total_files: usize,
        file_size_kb: usize,
    ) -> Result<f64> {
        use zip::ZipArchive;

        let files_per_worker = total_files / num_workers;
        let start = Instant::now();
        let mut temp_dirs = Vec::new();

        // Create archives (one per worker) with subset of files
        for i in 0..num_workers {
            let temp_dir = TempDir::new()?;
            let files = Self::generate_test_files(&temp_dir, files_per_worker, file_size_kb)?;
            let zip_path = temp_dir.path().join(format!("test_{}.zip", i));

            // Create ZIP
            let zip_file = File::create(&zip_path)?;
            let mut zip = ZipWriter::new(zip_file);
            for file_path in &files {
                let file_name = file_path.file_name().unwrap().to_str().unwrap();
                zip.start_file(
                    file_name,
                    zip::write::FileOptions::default()
                        .compression_method(CompressionMethod::Deflated),
                )?;
                let mut file = File::open(file_path)?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                zip.write_all(&buffer)?;
            }
            zip.finish()?;

            temp_dirs.push(temp_dir);
        }

        // Now extract in parallel
        let extract_results: Result<Vec<_>> = (0..num_workers)
            .into_par_iter()
            .map(|i| -> Result<()> {
                let zip_path = temp_dirs[i].path().join(format!("test_{}.zip", i));
                let extract_dir = TempDir::new()?;
                let zip_file = File::open(zip_path)?;
                let mut archive = ZipArchive::new(zip_file)?;
                for j in 0..archive.len() {
                    let mut file = archive.by_index(j)?;
                    let out_path = extract_dir.path().join(file.name());
                    let mut outfile = File::create(&out_path)?;
                    std::io::copy(&mut file, &mut outfile)?;
                }
                Ok(())
            })
            .collect();

        extract_results?;
        let duration = start.elapsed().as_secs_f64();
        Ok(num_workers as f64 / duration)
    }

    /// Test parallel TAR creation.
    /// Same total work as single-core (10 files * 100KB = 1MB), split across workers.
    fn test_parallel_tar_create(
        num_workers: usize,
        total_files: usize,
        file_size_kb: usize,
    ) -> Result<f64> {
        let files_per_worker = total_files / num_workers;
        let start = Instant::now();
        let mut temp_dirs = Vec::new();

        // Create archives (one per worker) with subset of files
        for i in 0..num_workers {
            let temp_dir = TempDir::new()?;
            let files = Self::generate_test_files(&temp_dir, files_per_worker, file_size_kb)?;
            let tar_path = temp_dir.path().join(format!("test_{}.tar", i));

            let tar_file = File::create(&tar_path)?;
            let mut tar = tar::Builder::new(tar_file);

            for file_path in &files {
                let file_name = file_path.file_name().unwrap().to_str().unwrap();
                tar.append_path_with_name(file_path, file_name)?;
            }

            tar.finish()?;
            temp_dirs.push(temp_dir);
        }

        let duration = start.elapsed().as_secs_f64();
        let total_size = (total_files * file_size_kb * 1024) as f64;
        Ok(total_size / 1024.0 / 1024.0 / duration)
    }

    /// Test parallel TAR extraction.
    /// Same total work as single-core (10 files), split across workers.
    fn test_parallel_tar_extract(
        num_workers: usize,
        total_files: usize,
        file_size_kb: usize,
    ) -> Result<f64> {
        let files_per_worker = total_files / num_workers;
        let start = Instant::now();
        let mut temp_dirs = Vec::new();

        // Create archives (one per worker) with subset of files
        for i in 0..num_workers {
            let temp_dir = TempDir::new()?;
            let files = Self::generate_test_files(&temp_dir, files_per_worker, file_size_kb)?;
            let tar_path = temp_dir.path().join(format!("test_{}.tar", i));

            let tar_file = File::create(&tar_path)?;
            let mut tar = tar::Builder::new(tar_file);
            for file_path in &files {
                let file_name = file_path.file_name().unwrap().to_str().unwrap();
                tar.append_path_with_name(file_path, file_name)?;
            }
            tar.finish()?;

            temp_dirs.push(temp_dir);
        }

        // Extract in parallel
        let extract_results: Result<Vec<_>> = (0..num_workers)
            .into_par_iter()
            .map(|i| -> Result<()> {
                let tar_path = temp_dirs[i].path().join(format!("test_{}.tar", i));
                let extract_dir = TempDir::new()?;
                let tar_file = File::open(tar_path)?;
                let mut archive = tar::Archive::new(tar_file);
                archive.unpack(extract_dir.path())?;
                Ok(())
            })
            .collect();

        extract_results?;
        let duration = start.elapsed().as_secs_f64();
        Ok(num_workers as f64 / duration)
    }
}

impl Default for ArchiveBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseBenchmark for ArchiveBenchmark {
    fn category_name(&self) -> &'static str {
        "Archive"
    }

    fn weight(&self) -> f64 {
        0.8
    }

    fn run_all(&self, iterations: usize, warmup: usize, timeout: u64) -> Result<CategoryResult> {
        let mut results = Vec::new();
        let mut total_duration = 0.0;

        // Reference values (MB/s for create, ops/s for extract - calibrated)
        // Same reference values used for both single-core and multi-core modes
        let (zip_create_ref, zip_extract_ref, tar_create_ref, tar_extract_ref) =
            (30.0, 50.0, 80.0, 150.0);

        if self.multi_core {
            // Multi-core: parallel archive operations with SAME total work as single-core
            let num_workers = get_parallel_workers();
            let total_files = 10;
            let file_size_kb = 100;

            let test_fn = || Self::test_parallel_zip_create(num_workers, total_files, file_size_kb);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel ZIP Create ({} archives)", num_workers),
                zip_create_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let test_fn =
                || Self::test_parallel_zip_extract(num_workers, total_files, file_size_kb);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel ZIP Extract ({} archives)", num_workers),
                zip_extract_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let test_fn = || Self::test_parallel_tar_create(num_workers, total_files, file_size_kb);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel TAR Create ({} archives)", num_workers),
                tar_create_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let test_fn =
                || Self::test_parallel_tar_extract(num_workers, total_files, file_size_kb);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel TAR Extract ({} archives)", num_workers),
                tar_extract_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);
        } else {
            // Single-core tests
            // Generate test files
            let temp_dir = TempDir::new()?;
            let files = Self::generate_test_files(&temp_dir, 10, 100)?;

            // Test 1: ZIP Create
            let output_zip = temp_dir.path().join("test.zip");
            let files_clone = files.clone();
            let output_clone = output_zip.clone();
            let test_fn = move || Self::test_zip_create(&files_clone, &output_clone);
            let result = run_with_iterations(
                test_fn,
                "ZIP Create",
                zip_create_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Test 2: ZIP Extract
            let output_clone = output_zip.clone();
            let test_fn = move || {
                let extract_dir = TempDir::new()?;
                Self::test_zip_extract(&output_clone, &extract_dir)
            };
            let result = run_with_iterations(
                test_fn,
                "ZIP Extract",
                zip_extract_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Test 3: TAR Create
            let output_tar = temp_dir.path().join("test.tar");
            let files_clone = files.clone();
            let output_clone = output_tar.clone();
            let test_fn = move || Self::test_tar_create(&files_clone, &output_clone);
            let result = run_with_iterations(
                test_fn,
                "TAR Create",
                tar_create_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Test 4: TAR Extract
            let output_clone = output_tar.clone();
            let test_fn = move || {
                let extract_dir = TempDir::new()?;
                Self::test_tar_extract(&output_clone, &extract_dir)
            };
            let result = run_with_iterations(
                test_fn,
                "TAR Extract",
                tar_extract_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);
        }

        // Calculate category score
        let category_score = calculate_category_score(&results);

        Ok(CategoryResult {
            category: self.category_name().to_string(),
            score: category_score,
            duration: total_duration,
            weight: self.weight(),
            tests: results,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_category_name() {
        let benchmark = ArchiveBenchmark::new();
        assert_eq!(benchmark.category_name(), "Archive");
    }

    #[test]
    fn test_archive_weight() {
        let benchmark = ArchiveBenchmark::new();
        assert_eq!(benchmark.weight(), 0.8);
    }
}
