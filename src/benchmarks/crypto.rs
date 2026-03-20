//! Cryptography benchmark tests.
//!
//! Tests AES encryption/decryption and SHA256 hashing.

use aes::Aes256;
use anyhow::Result;
use rand::Rng;
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::time::Instant;

use crate::benchmarks::base::{
    calculate_category_score, get_parallel_workers, run_with_iterations, BaseBenchmark,
    WorkloadScale,
};
use crate::results::CategoryResult;

/// Cryptography benchmark.
pub struct CryptoBenchmark {
    multi_core: bool,
}

impl CryptoBenchmark {
    /// Create a new CryptoBenchmark.
    pub fn new() -> Self {
        CryptoBenchmark { multi_core: false }
    }

    /// Create a new CryptoBenchmark for multi-core testing.
    pub fn new_multi_core() -> Self {
        CryptoBenchmark { multi_core: true }
    }

    /// Test AES-256 encryption.
    fn test_aes_encrypt(data_size_mb: usize) -> Result<f64> {
        use aes::cipher::{KeyIvInit, StreamCipher};
        type Aes256Ctr = ctr::Ctr128BE<Aes256>;

        let mut rng = rand::thread_rng();
        let key: [u8; 32] = rng.gen();
        let iv: [u8; 16] = rng.gen();
        let mut cipher = Aes256Ctr::new_from_slices(&key, &iv)
            .map_err(|e| anyhow::anyhow!("AES init failed: {}", e))?;

        let mut data = vec![0u8; data_size_mb * 1024 * 1024];
        for (i, byte) in data.iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }

        let start = Instant::now();
        cipher.apply_keystream(&mut data);
        let duration = start.elapsed().as_secs_f64();

        Ok(data_size_mb as f64 / duration)
    }

    /// Test SHA256 hashing.
    fn test_sha256(data_size_mb: usize) -> Result<f64> {
        let data = vec![0u8; data_size_mb * 1024 * 1024];

        let start = Instant::now();
        let _hash = Sha256::digest(&data);
        let duration = start.elapsed().as_secs_f64();

        Ok(data_size_mb as f64 / duration)
    }

    /// Test parallel AES encryption.
    fn test_parallel_aes_encrypt(num_chunks: usize, chunk_size_mb: usize) -> Result<f64> {
        let start = Instant::now();

        (0..num_chunks)
            .into_par_iter()
            .try_for_each(|_i| -> Result<()> {
                let _ = Self::test_aes_encrypt(chunk_size_mb)?;
                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        Ok((num_chunks * chunk_size_mb) as f64 / duration)
    }

    /// Test parallel SHA256 hashing.
    fn test_parallel_sha256(num_chunks: usize, chunk_size_mb: usize) -> Result<f64> {
        let start = Instant::now();

        (0..num_chunks)
            .into_par_iter()
            .try_for_each(|_i| -> Result<()> {
                let _ = Self::test_sha256(chunk_size_mb)?;
                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        Ok((num_chunks * chunk_size_mb) as f64 / duration)
    }
}

impl Default for CryptoBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseBenchmark for CryptoBenchmark {
    fn category_name(&self) -> &'static str {
        "Cryptography"
    }

    fn weight(&self) -> f64 {
        1.2
    }

    fn run_all(&self, iterations: usize, warmup: usize, timeout: u64) -> Result<CategoryResult> {
        let mut results = Vec::new();
        let mut total_duration = 0.0;
        let scale = WorkloadScale::detect();

        // Reference values (MB/s - calibrated for typical modern CPU)
        let aes_ref = 500.0;
        let sha256_ref = 800.0;

        if self.multi_core {
            let num_workers = get_parallel_workers();
            let chunk_size = scale.scale_capped(10, 50); // 10-50 MB per worker based on cores

            // Multi-core: Parallel AES encryption
            let test_fn = || Self::test_parallel_aes_encrypt(num_workers, chunk_size);
            let result = run_with_iterations(
                test_fn,
                "AES-256 Encrypt (parallel)",
                aes_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Multi-core: Parallel SHA256
            let test_fn = || Self::test_parallel_sha256(num_workers, chunk_size);
            let result = run_with_iterations(
                test_fn,
                "SHA256 Hash (parallel)",
                sha256_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);
        } else {
            let data_size_mb = scale.scale_capped(50, 200); // 50-200 MB based on cores

            // Single-core: AES encryption
            let test_fn = || Self::test_aes_encrypt(data_size_mb);
            let result = run_with_iterations(
                test_fn,
                "AES-256 Encrypt",
                aes_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Single-core: SHA256
            let test_fn = || Self::test_sha256(data_size_mb);
            let result = run_with_iterations(
                test_fn,
                "SHA256 Hash",
                sha256_ref,
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
    fn test_crypto_category_name() {
        let benchmark = CryptoBenchmark::new();
        assert_eq!(benchmark.category_name(), "Cryptography");
    }

    #[test]
    fn test_crypto_weight() {
        let benchmark = CryptoBenchmark::new();
        assert_eq!(benchmark.weight(), 1.2);
    }

    #[test]
    fn test_aes_encrypt_returns_positive() {
        let result = CryptoBenchmark::test_aes_encrypt(1);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_sha256_returns_positive() {
        let result = CryptoBenchmark::test_sha256(1);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_aes_encrypt_larger_data() {
        let result = CryptoBenchmark::test_aes_encrypt(10);
        assert!(result.is_ok());
        let throughput = result.unwrap();
        assert!(throughput > 0.0, "Throughput should be positive");
    }

    #[test]
    fn test_sha256_larger_data() {
        let result = CryptoBenchmark::test_sha256(10);
        assert!(result.is_ok());
        let throughput = result.unwrap();
        assert!(throughput > 0.0, "Throughput should be positive");
    }

    #[test]
    fn test_multi_core_benchmark_creation() {
        let single = CryptoBenchmark::new();
        let multi = CryptoBenchmark::new_multi_core();
        assert_eq!(single.category_name(), multi.category_name());
    }

    #[test]
    fn test_parallel_aes_encrypt() {
        let result = CryptoBenchmark::test_parallel_aes_encrypt(4, 5);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_parallel_sha256() {
        let result = CryptoBenchmark::test_parallel_sha256(4, 5);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }
}
