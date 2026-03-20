//! Compression benchmark tests.
//!
//! Tests ZIP (levels 1-9) and GZIP compression/decompression.

use anyhow::Result;
use flate2::Compression;
use rayon::prelude::*;
use std::io::{Cursor, Write};
use std::time::Instant;

use crate::benchmarks::base::{
    calculate_category_score, get_parallel_workers, run_with_iterations, BaseBenchmark,
};
use crate::references::ReferenceValues;
use crate::results::CategoryResult;

/// Compression benchmark.
pub struct CompressionBenchmark {
    multi_core: bool,
}

impl CompressionBenchmark {
    /// Create a new CompressionBenchmark.
    pub fn new() -> Self {
        CompressionBenchmark { multi_core: false }
    }

    /// Create a new CompressionBenchmark for multi-core testing.
    pub fn new_multi_core() -> Self {
        CompressionBenchmark { multi_core: true }
    }

    /// Test ZIP compression at level 1 (fastest).
    fn test_zip_level_1(data: &[u8]) -> Result<f64> {
        let start = Instant::now();
        let mut encoder = zip::write::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .compression_level(Some(1));
        encoder.start_file("test.bin", options)?;
        encoder.write_all(data)?;
        encoder.finish()?;
        let duration = start.elapsed().as_secs_f64();
        Ok(data.len() as f64 / 1024.0 / 1024.0 / duration)
    }

    /// Test ZIP compression at level 6 (balanced).
    fn test_zip_level_6(data: &[u8]) -> Result<f64> {
        let start = Instant::now();
        let mut encoder = zip::write::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .compression_level(Some(6));
        encoder.start_file("test.bin", options)?;
        encoder.write_all(data)?;
        encoder.finish()?;
        let duration = start.elapsed().as_secs_f64();
        Ok(data.len() as f64 / 1024.0 / 1024.0 / duration)
    }

    /// Test ZIP compression at level 9 (maximum).
    fn test_zip_level_9(data: &[u8]) -> Result<f64> {
        let start = Instant::now();
        let mut encoder = zip::write::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .compression_level(Some(9));
        encoder.start_file("test.bin", options)?;
        encoder.write_all(data)?;
        encoder.finish()?;
        let duration = start.elapsed().as_secs_f64();
        Ok(data.len() as f64 / 1024.0 / 1024.0 / duration)
    }

    /// Test GZIP compression.
    fn test_gzip(data: &[u8]) -> Result<f64> {
        use flate2::write::GzEncoder;
        let start = Instant::now();
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        encoder.finish()?;
        let duration = start.elapsed().as_secs_f64();
        Ok(data.len() as f64 / 1024.0 / 1024.0 / duration)
    }

    /// Generate test data for compression.
    fn generate_test_data(size_mb: usize) -> Vec<u8> {
        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut data = vec![0u8; size_mb * 1024 * 1024];
        rng.fill(&mut data[..]);
        data
    }

    /// Test parallel ZIP compression at level 1.
    /// Throughput model: Each worker compresses the FULL data (10MB).
    /// N workers compress N× the data in roughly the same time = N× speedup.
    fn test_parallel_zip_level_1(data: &[u8], num_workers: usize) -> Result<f64> {
        let start = Instant::now();

        (0..num_workers)
            .into_par_iter()
            .try_for_each(|_| -> Result<()> {
                let mut encoder = zip::write::ZipWriter::new(Cursor::new(Vec::new()));
                let options = zip::write::FileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated)
                    .compression_level(Some(1));
                encoder.start_file("test.bin", options)?;
                encoder.write_all(data)?;
                encoder.finish()?;
                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        // Throughput: num_workers tasks completed
        Ok(num_workers as f64 * data.len() as f64 / 1024.0 / 1024.0 / duration)
    }

    /// Test parallel ZIP compression at level 6.
    /// Throughput model: Each worker compresses the FULL data (10MB).
    /// N workers compress N× the data in roughly the same time = N× speedup.
    fn test_parallel_zip_level_6(data: &[u8], num_workers: usize) -> Result<f64> {
        let start = Instant::now();

        (0..num_workers)
            .into_par_iter()
            .try_for_each(|_| -> Result<()> {
                let mut encoder = zip::write::ZipWriter::new(Cursor::new(Vec::new()));
                let options = zip::write::FileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated)
                    .compression_level(Some(6));
                encoder.start_file("test.bin", options)?;
                encoder.write_all(data)?;
                encoder.finish()?;
                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        // Throughput: num_workers tasks completed
        Ok(num_workers as f64 * data.len() as f64 / 1024.0 / 1024.0 / duration)
    }

    /// Test parallel GZIP compression.
    /// Throughput model: Each worker compresses the FULL data (10MB).
    /// N workers compress N× the data in roughly the same time = N× speedup.
    fn test_parallel_gzip(data: &[u8], num_workers: usize) -> Result<f64> {
        use flate2::write::GzEncoder;
        let start = Instant::now();

        (0..num_workers)
            .into_par_iter()
            .try_for_each(|_| -> Result<()> {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(data)?;
                encoder.finish()?;
                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        // Throughput: num_workers tasks completed
        Ok(num_workers as f64 * data.len() as f64 / 1024.0 / 1024.0 / duration)
    }

    /// Test parallel ZIP compression at level 9 (maximum).
    /// Throughput model: Each worker compresses the FULL data (10MB).
    /// N workers compress N× the data in roughly the same time = N× speedup.
    fn test_parallel_zip_level_9(data: &[u8], num_workers: usize) -> Result<f64> {
        let start = Instant::now();

        (0..num_workers)
            .into_par_iter()
            .try_for_each(|_| -> Result<()> {
                let mut encoder = zip::write::ZipWriter::new(Cursor::new(Vec::new()));
                let options = zip::write::FileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated)
                    .compression_level(Some(9));
                encoder.start_file("test.bin", options)?;
                encoder.write_all(data)?;
                encoder.finish()?;
                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        // Throughput: num_workers tasks completed
        Ok(num_workers as f64 * data.len() as f64 / 1024.0 / 1024.0 / duration)
    }
}

impl Default for CompressionBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseBenchmark for CompressionBenchmark {
    fn category_name(&self) -> &'static str {
        "Compression"
    }

    fn run_all(&self, iterations: usize, warmup: usize, timeout: u64) -> Result<CategoryResult> {
        let mut results = Vec::new();
        let mut total_duration = 0.0;
        let refs = ReferenceValues::load();

        // Generate test data once
        let test_data = Self::generate_test_data(10);

        if self.multi_core {
            // Multi-core: compress same total data in parallel, split across workers
            let num_workers = get_parallel_workers();
            let data_clone = test_data.clone();
            let test_fn = move || Self::test_parallel_zip_level_1(&data_clone, num_workers);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel ZIP Fast (10MB / {} workers)", num_workers),
                refs.compression.zip_level_1_mbps,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let data_clone = test_data.clone();
            let test_fn = move || Self::test_parallel_zip_level_6(&data_clone, num_workers);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel ZIP Balanced (10MB / {} workers)", num_workers),
                refs.compression.zip_level_6_mbps,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let data_clone = test_data.clone();
            let test_fn = move || Self::test_parallel_gzip(&data_clone, num_workers);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel GZIP (10MB / {} workers)", num_workers),
                refs.compression.gzip_mbps,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let data_clone = test_data.clone();
            let test_fn = move || Self::test_parallel_zip_level_9(&data_clone, num_workers);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel ZIP Max (10MB / {} workers)", num_workers),
                refs.compression.zip_level_9_mbps,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);
        } else {
            // Single-core: compress single file
            let data_clone = test_data.clone();
            let test_fn = move || Self::test_zip_level_1(&data_clone);
            let result = run_with_iterations(
                test_fn,
                "ZIP Fast (Level 1)",
                refs.compression.zip_level_1_mbps,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let data_clone = test_data.clone();
            let test_fn = move || Self::test_zip_level_6(&data_clone);
            let result = run_with_iterations(
                test_fn,
                "ZIP Balanced (Level 6)",
                refs.compression.zip_level_6_mbps,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let data_clone = test_data.clone();
            let test_fn = move || Self::test_zip_level_9(&data_clone);
            let result = run_with_iterations(
                test_fn,
                "ZIP Max (Level 9)",
                refs.compression.zip_level_9_mbps,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let data_clone = test_data.clone();
            let test_fn = move || Self::test_gzip(&data_clone);
            let result = run_with_iterations(
                test_fn,
                "GZIP",
                refs.compression.gzip_mbps,
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
    fn test_compression_category_name() {
        let benchmark = CompressionBenchmark::new();
        assert_eq!(benchmark.category_name(), "Compression");
    }

    #[test]
    fn test_generate_test_data() {
        let data = CompressionBenchmark::generate_test_data(1);
        assert_eq!(data.len(), 1024 * 1024);
    }

    #[test]
    fn test_zip_level_1() {
        let data = CompressionBenchmark::generate_test_data(1);
        let result = CompressionBenchmark::test_zip_level_1(&data);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_zip_level_6() {
        let data = CompressionBenchmark::generate_test_data(1);
        let result = CompressionBenchmark::test_zip_level_6(&data);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_zip_level_9() {
        let data = CompressionBenchmark::generate_test_data(1);
        let result = CompressionBenchmark::test_zip_level_9(&data);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_gzip() {
        let data = CompressionBenchmark::generate_test_data(1);
        let result = CompressionBenchmark::test_gzip(&data);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_multi_core_benchmark_creation() {
        let single = CompressionBenchmark::new();
        let multi = CompressionBenchmark::new_multi_core();
        assert_eq!(single.category_name(), multi.category_name());
    }

    #[test]
    fn test_parallel_zip_level_1() {
        let data = CompressionBenchmark::generate_test_data(1);
        let result = CompressionBenchmark::test_parallel_zip_level_1(&data, 2);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_parallel_zip_level_6() {
        let data = CompressionBenchmark::generate_test_data(1);
        let result = CompressionBenchmark::test_parallel_zip_level_6(&data, 2);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_parallel_zip_level_9() {
        let data = CompressionBenchmark::generate_test_data(1);
        let result = CompressionBenchmark::test_parallel_zip_level_9(&data, 2);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_parallel_gzip() {
        let data = CompressionBenchmark::generate_test_data(1);
        let result = CompressionBenchmark::test_parallel_gzip(&data, 2);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }
}
