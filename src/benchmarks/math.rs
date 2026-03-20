//! Mathematical benchmark tests.
//!
//! Tests NumPy-like array operations, matrix multiplication, statistics, and prime generation.

use anyhow::Result;
use ndarray::{Array1, Array2};
use rayon::prelude::*;
use std::time::Instant;

use crate::benchmarks::base::{
    calculate_category_score, get_parallel_workers, run_with_iterations, BaseBenchmark,
    WorkloadScale,
};
use crate::results::CategoryResult;

/// Mathematical benchmark.
pub struct MathematicalBenchmark {
    /// If true, run tests in parallel (multi-core mode)
    multi_core: bool,
}

impl MathematicalBenchmark {
    /// Create a new MathematicalBenchmark.
    pub fn new() -> Self {
        MathematicalBenchmark { multi_core: false }
    }

    /// Create a new MathematicalBenchmark for multi-core testing.
    pub fn new_multi_core() -> Self {
        MathematicalBenchmark { multi_core: true }
    }

    /// Test NumPy-like array operations.
    fn test_array_ops(size: usize) -> Result<f64> {
        let mut arr = Array1::from_elem(size, 1.0);

        let start = Instant::now();

        // Element-wise operations
        arr += 1.0;
        arr *= 2.0;
        arr /= 2.0;
        arr -= 1.0;

        // Reductions
        let _sum = arr.sum();
        let _mean = arr.mean().unwrap();
        let _min = arr.iter().cloned().fold(f64::INFINITY, f64::min);
        let _max = arr.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        let duration = start.elapsed().as_secs_f64();
        Ok(1.0 / duration)
    }

    /// Test matrix multiplication.
    fn test_matrix_multiplication(size: usize) -> Result<f64> {
        let a = Array2::from_elem((size, size), 2.0);
        let b = Array2::from_elem((size, size), 3.0);

        let start = Instant::now();
        let _c = a.dot(&b);
        let duration = start.elapsed().as_secs_f64();

        Ok(1.0 / duration)
    }

    /// Test statistics calculations.
    fn test_statistics(size: usize) -> Result<f64> {
        let data: Vec<f64> = (0..size).map(|i| i as f64).collect();

        let start = Instant::now();

        // Mean
        let mean = data.iter().sum::<f64>() / data.len() as f64;

        // Standard deviation
        let variance: f64 =
            data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
        let _std_dev = variance.sqrt();

        // Median (requires sorting)
        let mut sorted = data.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let _median = if sorted.len() % 2 == 0 {
            (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };

        let duration = start.elapsed().as_secs_f64();
        Ok(1.0 / duration)
    }

    /// Test prime number generation (Sieve of Eratosthenes).
    fn test_prime_generation(limit: usize) -> Result<f64> {
        let start = Instant::now();

        let mut is_prime = vec![true; limit + 1];
        is_prime[0] = false;
        is_prime[1] = false;

        let mut p = 2;
        while p * p <= limit {
            if is_prime[p] {
                let mut i = p * p;
                while i <= limit {
                    is_prime[i] = false;
                    i += p;
                }
            }
            p += 1;
        }

        let _prime_count = is_prime.iter().filter(|&&x| x).count();

        let duration = start.elapsed().as_secs_f64();
        Ok(1.0 / duration)
    }

    /// Test parallel array operations.
    /// Throughput model: Each worker does the FULL work (1000 elements).
    /// N workers do N× the work in roughly the same time = N× speedup.
    fn test_parallel_array_ops(num_workers: usize, size: usize) -> Result<f64> {
        let start = Instant::now();

        (0..num_workers)
            .into_par_iter()
            .try_for_each(|_i| -> Result<()> {
                let mut arr = Array1::from_elem(size, 1.0);

                // Element-wise operations
                arr += 1.0;
                arr *= 2.0;
                arr /= 2.0;
                arr -= 1.0;

                // Reductions
                let _sum = arr.sum();
                let _mean = arr.mean().unwrap();

                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        // Throughput: num_workers tasks completed
        Ok(num_workers as f64 / duration)
    }

    /// Test parallel matrix multiplication.
    /// Throughput model: Each worker does the FULL work.
    /// N workers do N× the work in roughly the same time = N× speedup.
    fn test_parallel_matrix_multiplication(num_workers: usize, size: usize) -> Result<f64> {
        let start = Instant::now();

        (0..num_workers)
            .into_par_iter()
            .try_for_each(|_| -> Result<()> {
                let a = Array2::from_elem((size, size), 2.0);
                let b = Array2::from_elem((size, size), 3.0);
                let _c = a.dot(&b);
                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        // Throughput: num_workers tasks completed
        Ok(num_workers as f64 / duration)
    }

    /// Test parallel statistics calculations.
    /// Throughput model: Each worker does the FULL work.
    /// N workers do N× the work in roughly the same time = N× speedup.
    fn test_parallel_statistics(num_workers: usize, size: usize) -> Result<f64> {
        let start = Instant::now();

        (0..num_workers)
            .into_par_iter()
            .try_for_each(|_| -> Result<()> {
                let data: Vec<f64> = (0..size).map(|i| i as f64).collect();

                // Mean
                let mean = data.iter().sum::<f64>() / data.len() as f64;

                // Standard deviation
                let variance: f64 =
                    data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
                let _std_dev = variance.sqrt();

                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        // Throughput: num_workers tasks completed
        Ok(num_workers as f64 / duration)
    }

    /// Test parallel prime number generation.
    /// Throughput model: Each worker does the FULL work.
    /// N workers do N× the work in roughly the same time = N× speedup.
    fn test_parallel_prime_generation(num_workers: usize, limit: usize) -> Result<f64> {
        let start = Instant::now();

        (0..num_workers)
            .into_par_iter()
            .try_for_each(|_| -> Result<()> {
                let mut is_prime = vec![true; limit + 1];
                is_prime[0] = false;
                is_prime[1] = false;

                let mut p = 2;
                while p * p <= limit {
                    if is_prime[p] {
                        let mut i = p * p;
                        while i <= limit {
                            is_prime[i] = false;
                            i += p;
                        }
                    }
                    p += 1;
                }

                let _prime_count = is_prime.iter().filter(|&&x| x).count();
                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        // Throughput: num_workers tasks completed
        Ok(num_workers as f64 / duration)
    }
}

impl Default for MathematicalBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseBenchmark for MathematicalBenchmark {
    fn category_name(&self) -> &'static str {
        "Mathematical"
    }

    fn run_all(&self, iterations: usize, warmup: usize, timeout: u64) -> Result<CategoryResult> {
        let mut results = Vec::new();
        let mut total_duration = 0.0;

        // Reference values (operations per second)
        // Same reference values used for both single-core and multi-core modes
        let (array_ref, matrix_ref, stats_ref, prime_ref) = (50000.0, 50.0, 5000.0, 500.0);
        let scale = WorkloadScale::detect();

        // Base sizes scaled by system capabilities
        let array_size = scale.scale_capped(1000, 8000);
        let matrix_size = scale.scale_capped(200, 400);
        let stats_size = scale.scale_capped(10_000, 80_000);
        let prime_limit = scale.scale_capped(100_000, 800_000);

        if self.multi_core {
            let num_workers = get_parallel_workers();

            let test_fn = || Self::test_parallel_array_ops(num_workers, array_size);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel NumPy Array Ops ({} workers)", num_workers),
                array_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let test_fn = || Self::test_parallel_matrix_multiplication(num_workers, matrix_size);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel Matrix Multiplication ({} workers)", num_workers),
                matrix_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let test_fn = || Self::test_parallel_statistics(num_workers, stats_size);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel Statistics ({} workers)", num_workers),
                stats_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let test_fn = || Self::test_parallel_prime_generation(num_workers, prime_limit);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel Prime Generation ({} workers)", num_workers),
                prime_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);
        } else {
            // Single-core tests
            let test_fn = || Self::test_array_ops(array_size);
            let result = run_with_iterations(
                test_fn,
                "NumPy Array Ops",
                array_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let test_fn = || Self::test_matrix_multiplication(matrix_size);
            let result = run_with_iterations(
                test_fn,
                "Matrix Multiplication",
                matrix_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let test_fn = || Self::test_statistics(stats_size);
            let result = run_with_iterations(
                test_fn,
                "Statistics",
                stats_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let test_fn = || Self::test_prime_generation(prime_limit);
            let result = run_with_iterations(
                test_fn,
                "Prime Generation",
                prime_ref,
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
    fn test_math_category_name() {
        let benchmark = MathematicalBenchmark::new();
        assert_eq!(benchmark.category_name(), "Mathematical");
    }

    #[test]
    fn test_array_ops() {
        let result = MathematicalBenchmark::test_array_ops(1000);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_matrix_multiplication() {
        let result = MathematicalBenchmark::test_matrix_multiplication(200);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_prime_generation() {
        let result = MathematicalBenchmark::test_prime_generation(100_000);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_statistics() {
        let result = MathematicalBenchmark::test_statistics(10_000);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_multi_core_benchmark_creation() {
        let single = MathematicalBenchmark::new();
        let multi = MathematicalBenchmark::new_multi_core();
        assert_eq!(single.category_name(), multi.category_name());
    }

    #[test]
    fn test_parallel_array_ops() {
        let result = MathematicalBenchmark::test_parallel_array_ops(2, 1000);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_parallel_matrix_multiplication() {
        let result = MathematicalBenchmark::test_parallel_matrix_multiplication(2, 200);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_parallel_statistics() {
        let result = MathematicalBenchmark::test_parallel_statistics(2, 10_000);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_parallel_prime_generation() {
        let result = MathematicalBenchmark::test_parallel_prime_generation(2, 100_000);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }
}
