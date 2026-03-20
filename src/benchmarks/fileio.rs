//! File I/O benchmark tests.
//!
//! Tests sequential write/read, random access, copy, and delete operations.

use anyhow::Result;
use rand::Rng;
use rayon::prelude::*;
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::time::Instant;
use tempfile::TempDir;

use crate::benchmarks::base::{
    calculate_category_score, get_parallel_workers, run_with_iterations, BaseBenchmark,
};
use crate::references::ReferenceValues;
use crate::results::CategoryResult;

/// File I/O benchmark.
pub struct FileIOBenchmark {
    /// If true, run tests in parallel (multi-core mode)
    multi_core: bool,
}

impl FileIOBenchmark {
    /// Create a new FileIOBenchmark.
    pub fn new() -> Self {
        FileIOBenchmark { multi_core: false }
    }

    /// Create a new FileIOBenchmark for multi-core testing.
    pub fn new_multi_core() -> Self {
        FileIOBenchmark { multi_core: true }
    }

    /// Test sequential write performance.
    fn test_sequential_write(size_mb: usize) -> Result<f64> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("sequential_write_test.bin");
        let data = vec![0u8; size_mb * 1024 * 1024];

        let start = Instant::now();
        let mut file = fs::File::create(&file_path)?;
        file.write_all(&data)?;
        file.sync_all()?;
        let duration = start.elapsed().as_secs_f64();

        // Return throughput in MB/s
        Ok(size_mb as f64 / duration)
    }

    /// Test sequential read performance.
    fn test_sequential_read(size_mb: usize) -> Result<f64> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("sequential_read_test.bin");

        // Create test file
        let data = vec![0u8; size_mb * 1024 * 1024];
        let mut file = fs::File::create(&file_path)?;
        file.write_all(&data)?;
        file.sync_all()?;
        drop(file);

        // Read test
        let start = Instant::now();
        let mut file = fs::File::open(&file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        let duration = start.elapsed().as_secs_f64();

        // Return throughput in MB/s
        Ok(size_mb as f64 / duration)
    }

    /// Test random access performance.
    fn test_random_access(file_size_mb: usize, num_accesses: usize) -> Result<f64> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("random_access_test.bin");

        // Create test file
        let data = vec![0u8; file_size_mb * 1024 * 1024];
        let mut file = fs::File::create(&file_path)?;
        file.write_all(&data)?;
        file.sync_all()?;
        drop(file);

        // Random access test
        let mut rng = rand::thread_rng();
        let start = Instant::now();
        let mut file = fs::File::open(&file_path)?;

        for _ in 0..num_accesses {
            let pos = rng.gen_range(0..(file_size_mb * 1024 * 1024) as u64);
            file.seek(SeekFrom::Start(pos))?;
            let mut buffer = [0u8; 4096];
            file.read_exact(&mut buffer).ok();
        }

        let duration = start.elapsed().as_secs_f64();

        // Return accesses per second
        Ok(num_accesses as f64 / duration)
    }

    /// Test file copy performance.
    fn test_copy(size_mb: usize) -> Result<f64> {
        let temp_dir = TempDir::new()?;
        let src_path = temp_dir.path().join("copy_source.bin");
        let dst_path = temp_dir.path().join("copy_dest.bin");

        // Create source file
        let data = vec![0u8; size_mb * 1024 * 1024];
        let mut file = fs::File::create(&src_path)?;
        file.write_all(&data)?;
        file.sync_all()?;
        drop(file);

        // Copy test
        let start = Instant::now();
        fs::copy(&src_path, &dst_path)?;
        let duration = start.elapsed().as_secs_f64();

        // Return throughput in MB/s
        Ok(size_mb as f64 / duration)
    }

    /// Test file delete performance.
    fn test_delete(num_files: usize, file_size_kb: usize) -> Result<f64> {
        let temp_dir = TempDir::new()?;

        // Create test files
        for i in 0..num_files {
            let file_path = temp_dir.path().join(format!("delete_test_{}.bin", i));
            let data = vec![0u8; file_size_kb * 1024];
            let mut file = fs::File::create(&file_path)?;
            file.write_all(&data)?;
            drop(file);
        }

        // Delete test
        let start = Instant::now();
        for i in 0..num_files {
            let file_path = temp_dir.path().join(format!("delete_test_{}.bin", i));
            fs::remove_file(&file_path).ok();
        }
        let duration = start.elapsed().as_secs_f64();

        // Return files deleted per second
        Ok(num_files as f64 / duration)
    }

    /// Test parallel sequential write performance.
    /// Throughput model: Each worker writes the FULL data (100MB).
    /// N workers write N× the data in roughly the same time = N× speedup.
    fn test_parallel_sequential_write(num_workers: usize, size_mb: usize) -> Result<f64> {
        let temp_dir = TempDir::new()?;
        let start = Instant::now();

        (0..num_workers)
            .into_par_iter()
            .try_for_each(|i| -> Result<()> {
                let file_path = temp_dir.path().join(format!("parallel_write_{}.bin", i));
                let data = vec![0u8; size_mb * 1024 * 1024];
                let mut file = fs::File::create(&file_path)?;
                file.write_all(&data)?;
                file.sync_all()?;
                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        // Throughput: num_workers * size_mb completed
        Ok((num_workers * size_mb) as f64 / duration)
    }

    /// Test parallel sequential read performance.
    /// Throughput model: Each worker reads the FULL data (100MB).
    /// N workers read N× the data in roughly the same time = N× speedup.
    fn test_parallel_sequential_read(num_workers: usize, size_mb: usize) -> Result<f64> {
        let temp_dir = TempDir::new()?;

        // Create test files (one per worker, each with full size)
        for i in 0..num_workers {
            let file_path = temp_dir.path().join(format!("parallel_read_{}.bin", i));
            let data = vec![0u8; size_mb * 1024 * 1024];
            let mut file = fs::File::create(&file_path)?;
            file.write_all(&data)?;
            file.sync_all()?;
        }

        let start = Instant::now();

        (0..num_workers)
            .into_par_iter()
            .try_for_each(|i| -> Result<()> {
                let file_path = temp_dir.path().join(format!("parallel_read_{}.bin", i));
                let mut file = fs::File::open(&file_path)?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        // Throughput: num_workers * size_mb completed
        Ok((num_workers * size_mb) as f64 / duration)
    }

    /// Test parallel random access performance.
    /// Throughput model: Each worker does random reads on their own copy of the file.
    /// N workers do N× the accesses in roughly the same time = N× speedup.
    fn test_parallel_random_access(
        num_workers: usize,
        file_size_mb: usize,
        num_accesses: usize,
    ) -> Result<f64> {
        let temp_dir = TempDir::new()?;

        // Create a test file per worker
        for i in 0..num_workers {
            let file_path = temp_dir.path().join(format!("random_src_{}.bin", i));
            let data = vec![0u8; file_size_mb * 1024 * 1024];
            let mut file = fs::File::create(&file_path)?;
            file.write_all(&data)?;
            file.sync_all()?;
        }

        let start = Instant::now();

        (0..num_workers)
            .into_par_iter()
            .try_for_each(|i| -> Result<()> {
                let file_path = temp_dir.path().join(format!("random_src_{}.bin", i));
                let mut rng = rand::thread_rng();
                let mut file = fs::File::open(&file_path)?;

                for _ in 0..num_accesses {
                    let pos = rng.gen_range(0..(file_size_mb * 1024 * 1024) as u64);
                    file.seek(SeekFrom::Start(pos))?;
                    let mut buffer = [0u8; 4096];
                    file.read_exact(&mut buffer).ok();
                }
                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        // Throughput: num_workers * num_accesses completed
        Ok((num_workers * num_accesses) as f64 / duration)
    }

    /// Test parallel file copy performance.
    /// Throughput model: Each worker copies the FULL data (50MB).
    /// N workers copy N× the data in roughly the same time = N× speedup.
    fn test_parallel_copy(num_workers: usize, size_mb: usize) -> Result<f64> {
        let temp_dir = TempDir::new()?;

        // Create source files (one per worker, each with full size)
        for i in 0..num_workers {
            let src_path = temp_dir.path().join(format!("copy_src_{}.bin", i));
            let data = vec![0u8; size_mb * 1024 * 1024];
            let mut file = fs::File::create(&src_path)?;
            file.write_all(&data)?;
            file.sync_all()?;
        }

        let start = Instant::now();

        (0..num_workers)
            .into_par_iter()
            .try_for_each(|i| -> Result<()> {
                let src_path = temp_dir.path().join(format!("copy_src_{}.bin", i));
                let dst_path = temp_dir.path().join(format!("copy_dst_{}.bin", i));
                fs::copy(&src_path, &dst_path)?;
                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        // Throughput: num_workers * size_mb completed
        Ok((num_workers * size_mb) as f64 / duration)
    }

    /// Test parallel file delete performance.
    /// Throughput model: Each worker deletes the FULL set of files (100 files).
    /// N workers delete N× the files in roughly the same time = N× speedup.
    fn test_parallel_delete(
        num_workers: usize,
        num_files: usize,
        file_size_kb: usize,
    ) -> Result<f64> {
        let start = Instant::now();
        let mut temp_dirs = Vec::new();

        // Create test files for each worker (each worker gets full set of files)
        for _ in 0..num_workers {
            let temp_dir = TempDir::new()?;
            for i in 0..num_files {
                let file_path = temp_dir.path().join(format!("delete_test_{}.bin", i));
                let data = vec![0u8; file_size_kb * 1024];
                let mut file = fs::File::create(&file_path)?;
                file.write_all(&data)?;
                drop(file);
            }
            temp_dirs.push(temp_dir);
        }

        // Delete in parallel (each worker deletes their own set of files)
        (0..num_workers)
            .into_par_iter()
            .try_for_each(|w| -> Result<()> {
                for i in 0..num_files {
                    let file_path = temp_dirs[w].path().join(format!("delete_test_{}.bin", i));
                    fs::remove_file(&file_path)?;
                }
                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        // Throughput: num_workers * num_files completed
        Ok((num_workers * num_files) as f64 / duration)
    }
}

impl Default for FileIOBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseBenchmark for FileIOBenchmark {
    fn category_name(&self) -> &'static str {
        "File I/O"
    }

    fn weight(&self) -> f64 {
        1.5 // File I/O is weighted higher as it's a fundamental operation
    }

    fn run_all(&self, iterations: usize, warmup: usize, timeout: u64) -> Result<CategoryResult> {
        let mut results = Vec::new();
        let mut total_duration = 0.0;
        let refs = ReferenceValues::load();

        if self.multi_core {
            // Multi-core tests: parallel file operations with throughput model
            // Each worker does the FULL work, N workers do N× the work
            let num_workers = get_parallel_workers();

            // Test 1: Parallel Sequential Write (each worker writes 100MB)
            let test_fn = || Self::test_parallel_sequential_write(num_workers, 100);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel Write (100MB x {} workers)", num_workers),
                refs.fileio.sequential_write_mbps,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Test 2: Parallel Sequential Read (each worker reads 100MB)
            let test_fn = || Self::test_parallel_sequential_read(num_workers, 100);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel Read (100MB x {} workers)", num_workers),
                refs.fileio.sequential_read_mbps,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Test 3: Parallel Random Access (each worker does 1000 accesses on 100MB file)
            let test_fn = || Self::test_parallel_random_access(num_workers, 100, 1000);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel Random Access ({} workers)", num_workers),
                refs.fileio.random_access_ops,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Test 4: Parallel Copy (each worker copies 50MB)
            let test_fn = || Self::test_parallel_copy(num_workers, 50);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel Copy (50MB x {} workers)", num_workers),
                refs.fileio.copy_mbps,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Test 5: Parallel Delete (each worker deletes 100 files)
            let test_fn = || Self::test_parallel_delete(num_workers, 100, 100);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel Delete (100 files x {} workers)", num_workers),
                refs.fileio.delete_files_per_sec,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);
        } else {
            // Single-core tests: sequential file operations
            // Test 1: Sequential Write (100MB)
            let test_fn = || Self::test_sequential_write(100);
            let result = run_with_iterations(
                test_fn,
                "Sequential Write (100MB)",
                refs.fileio.sequential_write_mbps,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Test 2: Sequential Read (100MB)
            let test_fn = || Self::test_sequential_read(100);
            let result = run_with_iterations(
                test_fn,
                "Sequential Read (100MB)",
                refs.fileio.sequential_read_mbps,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Test 3: Random Access (100MB file, 1000 accesses)
            let test_fn = || Self::test_random_access(100, 1000);
            let result = run_with_iterations(
                test_fn,
                "Random Access",
                refs.fileio.random_access_ops,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Test 4: Copy Files (50MB)
            let test_fn = || Self::test_copy(50);
            let result = run_with_iterations(
                test_fn,
                "Copy Files",
                refs.fileio.copy_mbps,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Test 5: Delete Files (100 files x 100KB)
            let test_fn = || Self::test_delete(100, 100);
            let result = run_with_iterations(
                test_fn,
                "Delete Files",
                refs.fileio.delete_files_per_sec,
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
    fn test_fileio_category_name() {
        let benchmark = FileIOBenchmark::new();
        assert_eq!(benchmark.category_name(), "File I/O");
    }

    #[test]
    fn test_fileio_weight() {
        let benchmark = FileIOBenchmark::new();
        assert_eq!(benchmark.weight(), 1.5);
    }

    #[test]
    fn test_sequential_write() {
        let result = FileIOBenchmark::test_sequential_write(10);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_sequential_read() {
        let result = FileIOBenchmark::test_sequential_read(10);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_copy() {
        let result = FileIOBenchmark::test_copy(10);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_delete() {
        let result = FileIOBenchmark::test_delete(10, 100);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }
}
