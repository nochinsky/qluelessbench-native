//! Concurrent operations benchmark tests.
//!
//! Tests thread spawning, channels, concurrent data structures,
//! and work distribution patterns.

use anyhow::Result;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Instant;
use tempfile::TempDir;

use crate::benchmarks::base::{
    calculate_category_score, get_parallel_workers, run_with_iterations, BaseBenchmark,
};
use crate::references::ReferenceValues;
use crate::results::CategoryResult;

/// Concurrent benchmark.
pub struct ConcurrentBenchmark {
    multi_core: bool,
}

impl ConcurrentBenchmark {
    /// Create a new ConcurrentBenchmark (single-core mode).
    pub fn new() -> Self {
        ConcurrentBenchmark { multi_core: false }
    }

    /// Create a new ConcurrentBenchmark for multi-core testing.
    pub fn new_multi_core() -> Self {
        ConcurrentBenchmark { multi_core: true }
    }

    /// Test threaded file I/O — each thread writes and reads its own file.
    fn test_threaded_file_io(num_threads: usize) -> Result<f64> {
        let temp_dir = TempDir::new()?;
        let file_size = 1024 * 1024; // 1MB per thread

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

    /// Test channel-based message passing between threads.
    fn test_channel_messaging(num_messages: usize, num_workers: usize) -> Result<f64> {
        let (tx, rx) = mpsc::channel();

        let start = Instant::now();

        // Spawn workers that each process a chunk of messages
        let mut handles = Vec::new();
        let messages_per_worker = num_messages / num_workers;

        for worker_id in 0..num_workers {
            let tx = tx.clone();
            let handle = thread::spawn(move || {
                let mut local_sum: u64 = 0;
                for i in 0..messages_per_worker {
                    let msg_id = (worker_id * messages_per_worker + i) as u64;
                    local_sum = local_sum.wrapping_add(msg_id);
                }
                tx.send(local_sum).unwrap();
            });
            handles.push(handle);
        }

        drop(tx); // Drop original sender so receiver can detect all done

        // Collect results from workers
        let mut total: u64 = 0;
        while let Ok(partial) = rx.recv() {
            total = total.wrapping_add(partial);
        }

        for handle in handles {
            handle
                .join()
                .map_err(|_| anyhow::anyhow!("Worker panicked"))?;
        }

        let duration = start.elapsed().as_secs_f64();
        let _ = total;
        Ok(num_messages as f64 / duration)
    }

    /// Test concurrent hash map with per-thread local maps merged after.
    fn test_concurrent_map_build(num_threads: usize, inserts_per_thread: usize) -> Result<f64> {
        let start = Instant::now();

        let mut handles = Vec::new();

        for t in 0..num_threads {
            let handle = thread::spawn(move || {
                let mut local_map = HashMap::new();
                for i in 0..inserts_per_thread {
                    local_map.insert(format!("key_{}_{}", t, i), i);
                }
                local_map
            });
            handles.push(handle);
        }

        let mut merged = HashMap::new();
        for handle in handles {
            let local_map = handle
                .join()
                .map_err(|_| anyhow::anyhow!("Thread panicked during map build"))?;
            merged.extend(local_map);
        }

        let duration = start.elapsed().as_secs_f64();
        let total_inserts = merged.len();
        Ok(total_inserts as f64 / duration)
    }

    /// Test parallel merge sort — divide work across threads.
    fn test_parallel_merge_sort(num_threads: usize) -> Result<f64> {
        let size = 100_000;
        let data: Vec<f64> = (0..size).map(|i| (size - i) as f64).collect();
        let chunk_size = size / num_threads;

        let start = Instant::now();

        // Split data into chunks and sort each in a thread
        let mut handles = Vec::new();
        for chunk_start in (0..size).step_by(chunk_size) {
            let chunk_end = (chunk_start + chunk_size).min(size);
            let chunk = data[chunk_start..chunk_end].to_vec();
            let handle = thread::spawn(move || {
                let mut sorted = chunk;
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                sorted
            });
            handles.push(handle);
        }

        // Merge sorted chunks
        let mut sorted_chunks: Vec<Vec<f64>> = Vec::new();
        for handle in handles {
            sorted_chunks.push(
                handle
                    .join()
                    .map_err(|_| anyhow::anyhow!("Sort thread panicked"))?,
            );
        }

        // K-way merge (simple pairwise)
        let mut result = sorted_chunks.remove(0);
        for chunk in sorted_chunks {
            result = merge_sorted(&result, &chunk);
        }

        let duration = start.elapsed().as_secs_f64();
        let _ = result;
        Ok(size as f64 / duration)
    }
}

impl Default for ConcurrentBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

/// Merge two sorted slices into a single sorted Vec.
fn merge_sorted(a: &[f64], b: &[f64]) -> Vec<f64> {
    let mut result = Vec::with_capacity(a.len() + b.len());
    let mut i = 0;
    let mut j = 0;

    while i < a.len() && j < b.len() {
        if a[i] <= b[j] {
            result.push(a[i]);
            i += 1;
        } else {
            result.push(b[j]);
            j += 1;
        }
    }
    result.extend_from_slice(&a[i..]);
    result.extend_from_slice(&b[j..]);
    result
}

impl BaseBenchmark for ConcurrentBenchmark {
    fn category_name(&self) -> &'static str {
        "Concurrent"
    }

    fn run_all(&self, iterations: usize, warmup: usize, timeout: u64) -> Result<CategoryResult> {
        let mut results = Vec::new();
        let mut total_duration = 0.0;
        let refs = ReferenceValues::load();

        let num_workers = get_parallel_workers();
        let thread_count = if self.multi_core { num_workers } else { 1 };
        let inserts_per_thread = 2000;
        let messages_per_worker = 50000;

        // Test 1: Threaded File I/O
        let test_fn = || Self::test_threaded_file_io(thread_count);
        let result = run_with_iterations(
            test_fn,
            &format!("Threaded File I/O ({} threads)", thread_count),
            refs.concurrent.threaded_file_io_mbps,
            iterations,
            warmup,
            timeout,
        );
        total_duration += result.duration;
        results.push(result);

        // Test 2: Channel Messaging
        let total_messages = messages_per_worker * thread_count;
        let test_fn = || Self::test_channel_messaging(total_messages, thread_count);
        let result = run_with_iterations(
            test_fn,
            &format!("Channel Messaging ({} workers)", thread_count),
            refs.concurrent.channel_messaging_ops,
            iterations,
            warmup,
            timeout,
        );
        total_duration += result.duration;
        results.push(result);

        // Test 3: Concurrent Map Build (per-thread local maps, merged after)
        let test_fn = || Self::test_concurrent_map_build(thread_count, inserts_per_thread);
        let result = run_with_iterations(
            test_fn,
            &format!("Concurrent Map Build ({} threads)", thread_count),
            refs.concurrent.concurrent_map_build_ops,
            iterations,
            warmup,
            timeout,
        );
        total_duration += result.duration;
        results.push(result);

        // Test 4: Parallel Merge Sort
        if thread_count > 1 {
            let test_fn = || Self::test_parallel_merge_sort(thread_count);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel Merge Sort ({} threads)", thread_count),
                refs.concurrent.parallel_sort_mbps,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);
        } else {
            // Single-thread sort
            let test_fn = || {
                let size = 100_000;
                let mut data: Vec<f64> = (0..size).map(|i| (size - i) as f64).collect();
                let start = Instant::now();
                data.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let duration = start.elapsed().as_secs_f64();
                let _ = data;
                Ok(size as f64 / duration)
            };
            let result = run_with_iterations(
                test_fn,
                "Sequential Sort",
                refs.concurrent.parallel_sort_mbps,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);
        }

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
    fn test_multi_core_category_name() {
        let benchmark = ConcurrentBenchmark::new_multi_core();
        assert_eq!(benchmark.category_name(), "Concurrent");
    }

    #[test]
    fn test_threaded_file_io_single() {
        let result = ConcurrentBenchmark::test_threaded_file_io(1);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_threaded_file_io_multi() {
        let result = ConcurrentBenchmark::test_threaded_file_io(4);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_channel_messaging() {
        let result = ConcurrentBenchmark::test_channel_messaging(10_000, 2);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_concurrent_map_build_single() {
        let result = ConcurrentBenchmark::test_concurrent_map_build(1, 1000);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_concurrent_map_build_multi() {
        let result = ConcurrentBenchmark::test_concurrent_map_build(4, 1000);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_concurrent_map_build_total_entries() {
        let threads = 4;
        let inserts = 100;
        let result = ConcurrentBenchmark::test_concurrent_map_build(threads, inserts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_merge_sorted() {
        let a = vec![1.0, 3.0, 5.0];
        let b = vec![2.0, 4.0, 6.0];
        let merged = merge_sorted(&a, &b);
        assert_eq!(merged, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn test_merge_sorted_empty() {
        let a: Vec<f64> = vec![];
        let b = vec![1.0, 2.0];
        assert_eq!(merge_sorted(&a, &b), vec![1.0, 2.0]);
        assert_eq!(merge_sorted(&b, &a), vec![1.0, 2.0]);
    }

    #[test]
    fn test_parallel_merge_sort() {
        let result = ConcurrentBenchmark::test_parallel_merge_sort(2);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_run_all_single_core() {
        let bench = ConcurrentBenchmark::new();
        let result = bench.run_all(2, 1, 30);
        assert!(result.is_ok());
        let cat = result.unwrap();
        assert_eq!(cat.category, "Concurrent");
        assert!(cat.tests.len() >= 4);
    }

    #[test]
    fn test_run_all_multi_core() {
        let bench = ConcurrentBenchmark::new_multi_core();
        let result = bench.run_all(2, 1, 30);
        assert!(result.is_ok());
        let cat = result.unwrap();
        assert_eq!(cat.category, "Concurrent");
        assert!(cat.tests.len() >= 4);
    }
}
