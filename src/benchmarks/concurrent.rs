//! Concurrent operations benchmark tests.
//!
//! Tests multi-threaded file ops, parallel computation, and synchronization.

use anyhow::Result;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::Instant;
use tempfile::TempDir;

use crate::benchmarks::base::{calculate_category_score, run_with_iterations, BaseBenchmark};
use crate::results::CategoryResult;

/// Concurrent benchmark.
pub struct ConcurrentBenchmark;

impl ConcurrentBenchmark {
    /// Create a new ConcurrentBenchmark.
    pub fn new() -> Self {
        ConcurrentBenchmark
    }

    /// Test threaded file I/O.
    fn test_threaded_file_io() -> Result<f64> {
        let temp_dir = TempDir::new()?;
        let num_threads = Self::get_parallel_workers();
        let file_size = 1024 * 1024; // 1MB

        let start = Instant::now();

        let mut handles = Vec::new();

        for i in 0..num_threads {
            let dir = temp_dir.path().to_path_buf();
            let handle = thread::spawn(move || {
                let file_path = dir.join(format!("thread_{}.bin", i));
                let data = vec![i as u8; file_size];

                let mut file = File::create(&file_path)?;
                file.write_all(&data)?;
                file.sync_all()?;

                let mut file = File::open(&file_path)?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;

                Ok::<_, std::io::Error>(buffer.len())
            });
            handles.push(handle);
        }

        for handle in handles {
            handle
                .join()
                .map_err(|_| anyhow::anyhow!("Thread panicked during file I/O"))??;
        }

        let duration = start.elapsed().as_secs_f64();
        Ok((num_threads * file_size) as f64 / 1024.0 / 1024.0 / duration)
    }

    /// Test parallel computation.
    fn test_parallel_computation() -> Result<f64> {
        use rayon::prelude::*;

        let size = 1000000;
        let data: Vec<f64> = (0..size).map(|i| i as f64).collect();

        let start = Instant::now();

        // Parallel map and reduce
        let sum: f64 = data.par_iter().map(|&x| x * 2.0 + 1.0).sum();

        let _ = sum;

        let duration = start.elapsed().as_secs_f64();
        Ok(size as f64 / duration)
    }

    /// Test thread synchronization.
    fn test_thread_synchronization() -> Result<f64> {
        let num_threads = Self::get_parallel_workers();
        let iterations = 1000;
        let counter = Arc::new(Mutex::new(0));
        let barrier = Arc::new(Barrier::new(num_threads));

        let start = Instant::now();

        let mut handles = Vec::new();

        for _ in 0..num_threads {
            let counter = Arc::clone(&counter);
            let barrier = Arc::clone(&barrier);

            let handle = thread::spawn(move || {
                for _ in 0..iterations {
                    let mut num = counter.lock().unwrap();
                    *num += 1;
                    drop(num);
                }
                barrier.wait();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle
                .join()
                .map_err(|_| anyhow::anyhow!("Thread panicked during synchronization"))?;
        }

        let duration = start.elapsed().as_secs_f64();
        let final_count = *counter
            .lock()
            .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;

        Ok(final_count as f64 / duration)
    }

    /// Test concurrent database-like operations.
    fn test_concurrent_database() -> Result<f64> {
        use std::collections::HashMap;

        let num_threads = Self::get_parallel_workers();
        let operations_per_thread = 1000;
        let map = Arc::new(Mutex::new(HashMap::new()));

        let start = Instant::now();

        let mut handles = Vec::new();

        for t in 0..num_threads {
            let map = Arc::clone(&map);

            let handle = thread::spawn(move || {
                for i in 0..operations_per_thread {
                    let mut data = map.lock().unwrap();
                    data.insert(format!("key_{}_{}", t, i), i);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle
                .join()
                .map_err(|_| anyhow::anyhow!("Thread panicked during concurrent DB ops"))?;
        }

        let duration = start.elapsed().as_secs_f64();
        let final_size = map
            .lock()
            .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?
            .len();

        Ok(final_size as f64 / duration)
    }

    /// Get the number of parallel workers based on available cores.
    fn get_parallel_workers() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(8)
            * 2
    }
}

impl Default for ConcurrentBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseBenchmark for ConcurrentBenchmark {
    fn category_name(&self) -> &'static str {
        "Concurrent"
    }

    fn run_all(&self, iterations: usize, warmup: usize, timeout: u64) -> Result<CategoryResult> {
        let mut results = Vec::new();
        let mut total_duration = 0.0;

        // Reference values (MB/s for I/O, ops/s for computation)
        let threaded_io_ref = 200.0;
        let parallel_ref = 10000000.0;
        let sync_ref = 100000.0;
        let concurrent_db_ref = 50000.0;

        // Test 1: Threaded File I/O
        let test_fn = || Self::test_threaded_file_io();
        let result = run_with_iterations(
            test_fn,
            "Threaded File I/O",
            threaded_io_ref,
            iterations,
            warmup,
            timeout,
        );
        total_duration += result.duration;
        results.push(result);

        // Test 2: Parallel Computation
        let test_fn = || Self::test_parallel_computation();
        let result = run_with_iterations(
            test_fn,
            "Parallel Computation",
            parallel_ref,
            iterations,
            warmup,
            timeout,
        );
        total_duration += result.duration;
        results.push(result);

        // Test 3: Thread Synchronization
        let test_fn = || Self::test_thread_synchronization();
        let result = run_with_iterations(
            test_fn,
            "Thread Synchronization",
            sync_ref,
            iterations,
            warmup,
            timeout,
        );
        total_duration += result.duration;
        results.push(result);

        // Test 4: Concurrent Database
        let test_fn = || Self::test_concurrent_database();
        let result = run_with_iterations(
            test_fn,
            "Concurrent Database",
            concurrent_db_ref,
            iterations,
            warmup,
            timeout,
        );
        total_duration += result.duration;
        results.push(result);

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
    fn test_concurrent_category_name() {
        let benchmark = ConcurrentBenchmark::new();
        assert_eq!(benchmark.category_name(), "Concurrent");
    }

    #[test]
    fn test_threaded_file_io() {
        let result = ConcurrentBenchmark::test_threaded_file_io();
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_parallel_computation() {
        let result = ConcurrentBenchmark::test_parallel_computation();
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_thread_synchronization() {
        let result = ConcurrentBenchmark::test_thread_synchronization();
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }
}
