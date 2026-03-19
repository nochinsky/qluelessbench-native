//! Memory benchmark tests.
//!
//! Tests allocation/deallocation cycles, Vec/HashMap operations, and large structure manipulation.

use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;
use std::time::Instant;

use crate::benchmarks::base::{
    calculate_category_score, get_parallel_workers, run_with_iterations, BaseBenchmark,
};
use crate::results::CategoryResult;

/// Memory benchmark.
pub struct MemoryBenchmark {
    /// If true, run tests in parallel (multi-core mode)
    multi_core: bool,
}

impl MemoryBenchmark {
    /// Create a new MemoryBenchmark.
    pub fn new() -> Self {
        MemoryBenchmark { multi_core: false }
    }

    /// Create a new MemoryBenchmark for multi-core testing.
    pub fn new_multi_core() -> Self {
        MemoryBenchmark { multi_core: true }
    }

    /// Test allocation/deallocation cycles.
    fn test_alloc_dealloc() -> Result<f64> {
        use std::hint::black_box;
        let size = 1024 * 1024; // 1MB
        let iterations = 100;

        let start = Instant::now();

        for _ in 0..iterations {
            let _data: Vec<u8> = vec![0u8; size];
            black_box(&_data);
            // Data is dropped here
        }

        let duration = start.elapsed().as_secs_f64();
        Ok(iterations as f64 / duration)
    }

    /// Test Vec operations.
    fn test_vec_operations() -> Result<f64> {
        let size = 10000;

        let start = Instant::now();

        let mut vec: Vec<i32> = Vec::with_capacity(size);

        // Push operations
        for i in 0..size {
            vec.push(i as i32);
        }

        // Pop operations
        for _ in 0..(size / 2) {
            let _ = vec.pop();
        }

        // Insert operations
        for i in 0..100 {
            vec.insert(i, i as i32);
        }

        // Remove operations
        for i in (0..100).rev() {
            let _ = vec.remove(i as usize);
        }

        // Sort
        vec.sort();

        let duration = start.elapsed().as_secs_f64();
        Ok(1.0 / duration)
    }

    /// Test HashMap operations.
    fn test_hashmap_operations() -> Result<f64> {
        let size = 10000;

        let start = Instant::now();

        let mut map: HashMap<i32, String> = HashMap::with_capacity(size);

        // Insert operations
        for i in 0..size {
            map.insert(i as i32, format!("value_{}", i));
        }

        // Get operations
        for i in 0..size {
            let _ = map.get(&(i as i32));
        }

        // Remove operations
        for i in 0..(size / 2) {
            let _ = map.remove(&(i as i32));
        }

        // Iterate
        for (k, v) in &map {
            let _ = (k, v);
        }

        let duration = start.elapsed().as_secs_f64();
        Ok(1.0 / duration)
    }

    /// Test large structure manipulation.
    fn test_large_structure() -> Result<f64> {
        let size = 100000;

        let start = Instant::now();

        // Create large structure
        let mut data: Vec<(i32, String)> = Vec::with_capacity(size);
        for i in 0..size {
            data.push((i as i32, format!("item_{}", i)));
        }

        // Transform
        let transformed: Vec<(i32, String)> = data
            .into_iter()
            .map(|(k, v)| (k * 2, v.to_uppercase()))
            .collect();

        // Filter
        let filtered: Vec<_> = transformed
            .into_iter()
            .filter(|(k, _)| k % 2 == 0)
            .collect();

        std::hint::black_box(&filtered);

        let duration = start.elapsed().as_secs_f64();
        Ok(1.0 / duration)
    }

    /// Test parallel allocation/deallocation cycles.
    /// Throughput model: Each worker does the FULL work (100 iterations of 1MB).
    /// N workers do N× the work in roughly the same time = N× speedup.
    fn test_parallel_alloc_dealloc(num_workers: usize) -> Result<f64> {
        use std::hint::black_box;
        let size = 1024 * 1024; // 1MB
        let iterations = 100; // Each worker does full iterations
        let start = Instant::now();

        (0..num_workers)
            .into_par_iter()
            .try_for_each(|_| -> Result<()> {
                for _ in 0..iterations {
                    let _data: Vec<u8> = vec![0u8; size];
                    black_box(&_data);
                    // Data is dropped here
                }
                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        // Throughput: num_workers * iterations tasks completed
        Ok((num_workers * iterations) as f64 / duration)
    }

    /// Test parallel Vec operations.
    /// Throughput model: Each worker does the FULL work (vec of 10000 elements).
    /// N workers do N× the work in roughly the same time = N× speedup.
    fn test_parallel_vec_operations(num_workers: usize) -> Result<f64> {
        let size = 10000; // Each worker processes full size
        let start = Instant::now();

        (0..num_workers)
            .into_par_iter()
            .try_for_each(|_| -> Result<()> {
                let mut vec: Vec<i32> = Vec::with_capacity(size);

                // Push operations
                for i in 0..size {
                    vec.push(i as i32);
                }

                // Pop operations
                for _ in 0..(size / 2) {
                    let _ = vec.pop();
                }

                // Insert operations
                for i in 0..100 {
                    vec.insert(i, i as i32);
                }

                // Remove operations
                for i in (0..100).rev() {
                    let _ = vec.remove(i as usize);
                }

                // Sort
                vec.sort();

                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        // Throughput: num_workers tasks completed
        Ok(num_workers as f64 / duration)
    }

    /// Test parallel HashMap operations.
    /// Throughput model: Each worker does the FULL work (map of 10000 elements).
    /// N workers do N× the work in roughly the same time = N× speedup.
    fn test_parallel_hashmap_operations(num_workers: usize) -> Result<f64> {
        let size = 10000; // Each worker processes full size
        let start = Instant::now();

        (0..num_workers)
            .into_par_iter()
            .try_for_each(|_| -> Result<()> {
                let mut map: HashMap<i32, String> = HashMap::with_capacity(size);

                // Insert operations
                for i in 0..size {
                    map.insert(i as i32, format!("value_{}", i));
                }

                // Get operations
                for i in 0..size {
                    let _ = map.get(&(i as i32));
                }

                // Remove operations
                for i in 0..(size / 2) {
                    let _ = map.remove(&(i as i32));
                }

                // Iterate
                for (k, v) in &map {
                    let _ = (k, v);
                }

                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        // Throughput: num_workers tasks completed
        Ok(num_workers as f64 / duration)
    }

    /// Test parallel large structure manipulation.
    /// Throughput model: Each worker does the FULL work (structure of 100000 elements).
    /// N workers do N× the work in roughly the same time = N× speedup.
    fn test_parallel_large_structure(num_workers: usize) -> Result<f64> {
        let size = 100000; // Each worker processes full size
        let start = Instant::now();

        (0..num_workers)
            .into_par_iter()
            .try_for_each(|_| -> Result<()> {
                // Create large structure
                let mut data: Vec<(i32, String)> = Vec::with_capacity(size);
                for i in 0..size {
                    data.push((i as i32, format!("item_{}", i)));
                }

                // Transform
                let transformed: Vec<(i32, String)> = data
                    .into_iter()
                    .map(|(k, v)| (k * 2, v.to_uppercase()))
                    .collect();

                // Filter
                let filtered: Vec<_> = transformed
                    .into_iter()
                    .filter(|(k, _)| k % 2 == 0)
                    .collect();

                std::hint::black_box(&filtered);
                Ok(())
            })?;

        let duration = start.elapsed().as_secs_f64();
        // Throughput: num_workers tasks completed
        Ok(num_workers as f64 / duration)
    }
}

impl Default for MemoryBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseBenchmark for MemoryBenchmark {
    fn category_name(&self) -> &'static str {
        "Memory"
    }

    fn weight(&self) -> f64 {
        1.2
    }

    fn run_all(&self, iterations: usize, warmup: usize, timeout: u64) -> Result<CategoryResult> {
        let mut results = Vec::new();
        let mut total_duration = 0.0;

        // Reference values (operations per second - calibrated)
        // Same reference values used for both single-core and multi-core modes
        let (alloc_ref, vec_ref, hashmap_ref, structure_ref) = (3000.0, 3000.0, 3000.0, 300.0);

        if self.multi_core {
            // Multi-core: parallel memory operations with SAME total work as single-core
            let num_workers = get_parallel_workers();

            let test_fn = || Self::test_parallel_alloc_dealloc(num_workers);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel Alloc/Dealloc Cycles ({} workers)", num_workers),
                alloc_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let test_fn = || Self::test_parallel_vec_operations(num_workers);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel Vec Operations ({} workers)", num_workers),
                vec_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let test_fn = || Self::test_parallel_hashmap_operations(num_workers);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel HashMap Operations ({} workers)", num_workers),
                hashmap_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let test_fn = || Self::test_parallel_large_structure(num_workers);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel Large Structure ({} workers)", num_workers),
                structure_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);
        } else {
            // Single-core tests
            // Test 1: Alloc/Dealloc Cycles
            let test_fn = || Self::test_alloc_dealloc();
            let result = run_with_iterations(
                test_fn,
                "Alloc/Dealloc Cycles",
                alloc_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Test 2: Vec Operations
            let test_fn = || Self::test_vec_operations();
            let result = run_with_iterations(
                test_fn,
                "Vec Operations",
                vec_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Test 3: HashMap Operations
            let test_fn = || Self::test_hashmap_operations();
            let result = run_with_iterations(
                test_fn,
                "HashMap Operations",
                hashmap_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Test 4: Large Structure
            let test_fn = || Self::test_large_structure();
            let result = run_with_iterations(
                test_fn,
                "Large Structure",
                structure_ref,
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
    fn test_memory_category_name() {
        let benchmark = MemoryBenchmark::new();
        assert_eq!(benchmark.category_name(), "Memory");
    }

    #[test]
    fn test_memory_weight() {
        let benchmark = MemoryBenchmark::new();
        assert_eq!(benchmark.weight(), 1.2);
    }

    #[test]
    fn test_alloc_dealloc() {
        let result = MemoryBenchmark::test_alloc_dealloc();
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_vec_operations() {
        let result = MemoryBenchmark::test_vec_operations();
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_hashmap_operations() {
        let result = MemoryBenchmark::test_hashmap_operations();
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }
}
