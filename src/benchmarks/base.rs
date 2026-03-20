//! Base benchmark trait and utilities.
//!
//! This module defines the BaseBenchmark trait that all benchmark categories
//! must implement, along with common utility functions.

use crate::results::TestResult;
use crate::stats::descriptive::{
    calculate_coefficient_of_variation, calculate_median, calculate_percentile,
};
use crate::stats::{get_reliability, Reliability};
use anyhow::Result;
use std::time::{Duration, Instant};

/// Get the number of parallel workers based on available cores.
pub fn get_parallel_workers() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(8)
}

/// Calculate category score as average of test scores.
pub fn calculate_category_score(results: &[TestResult]) -> f64 {
    if results.is_empty() {
        return 0.0;
    }
    results.iter().map(|r| r.score).sum::<f64>() / results.len() as f64
}

/// Base trait for all benchmarks.
pub trait BaseBenchmark {
    /// Get the category name.
    fn category_name(&self) -> &'static str;

    /// Get the weight for this category in overall score calculation.
    fn weight(&self) -> f64 {
        1.0
    }

    /// Run all tests in this category.
    fn run_all(
        &self,
        iterations: usize,
        warmup: usize,
        timeout: u64,
    ) -> Result<crate::results::CategoryResult>;
}

/// Scoring direction for benchmark tests.
pub enum ScoringMode {
    /// Higher metric values mean better performance (e.g., throughput in MB/s).
    HigherIsBetter,
    /// Lower metric values mean better performance (e.g., duration in seconds).
    LowerIsBetter,
}

/// Core iteration runner shared by all scoring modes.
fn run_iterations_core<F>(
    test_fn: F,
    name: &str,
    reference_value: f64,
    scoring_mode: ScoringMode,
    iterations: usize,
    warmup: usize,
    timeout: u64,
) -> TestResult
where
    F: Fn() -> Result<f64>,
{
    let mut result = TestResult::new(name.to_string());
    let mut values: Vec<f64> = Vec::with_capacity(iterations);
    let start_time = Instant::now();
    let deadline = start_time + Duration::from_secs(timeout);

    // Warmup iterations
    for _ in 0..warmup {
        if let Err(e) = test_fn() {
            eprintln!("Warning: warmup iteration failed: {}", e);
        }
    }

    // Measured iterations
    for i in 0..iterations {
        // Check timeout against overall deadline
        if Instant::now() > deadline {
            result.passed = false;
            result.error = Some(format!("Timeout on iteration {}", i));
            break;
        }

        match test_fn() {
            Ok(value) => values.push(value),
            Err(e) => {
                result.passed = false;
                result.error = Some(e.to_string());
                break;
            }
        }
    }

    result.duration = start_time.elapsed().as_secs_f64();
    result.iterations = values.len();

    if values.is_empty() {
        result.score = 0.0;
        result.median = 0.0;
        result.p95 = 0.0;
        result.p99 = 0.0;
        result.cv = 0.0;
        result.reliability = Reliability::Unknown;
        return result;
    }

    // Calculate statistics
    result.median = calculate_median(&values);
    result.p95 = calculate_percentile(&values, 95.0);
    result.p99 = calculate_percentile(&values, 99.0);
    result.cv = calculate_coefficient_of_variation(&values);
    result.reliability = get_reliability(result.cv);

    // Calculate score based on median value and scoring mode
    match scoring_mode {
        ScoringMode::HigherIsBetter => {
            result.score = (result.median / reference_value) * 1000.0;
        }
        ScoringMode::LowerIsBetter => {
            result.score = if result.median > 0.0 {
                (reference_value / result.median) * 1000.0
            } else {
                1000.0
            };
        }
    }

    result
}

/// Run a benchmark test with iterations and warmup.
///
/// # Arguments
///
/// * `test_fn` - The test function to run (returns the metric value)
/// * `name` - Name of the test
/// * `reference_value` - Reference value for scoring (higher = better performance)
/// * `iterations` - Number of measured iterations
/// * `warmup` - Number of warmup iterations (discarded)
/// * `timeout` - Timeout in seconds
///
/// # Returns
///
/// A TestResult with statistics and score.
pub fn run_with_iterations<F>(
    test_fn: F,
    name: &str,
    reference_value: f64,
    iterations: usize,
    warmup: usize,
    timeout: u64,
) -> TestResult
where
    F: Fn() -> Result<f64>,
{
    run_iterations_core(
        test_fn,
        name,
        reference_value,
        ScoringMode::HigherIsBetter,
        iterations,
        warmup,
        timeout,
    )
}

/// Run a benchmark test where lower values are better (e.g., duration).
///
/// Similar to run_with_iterations but inverts the scoring.
pub fn run_with_iterations_lower_is_better<F>(
    test_fn: F,
    name: &str,
    reference_value: f64,
    iterations: usize,
    warmup: usize,
    timeout: u64,
) -> TestResult
where
    F: Fn() -> Result<f64>,
{
    run_iterations_core(
        test_fn,
        name,
        reference_value,
        ScoringMode::LowerIsBetter,
        iterations,
        warmup,
        timeout,
    )
}

/// Format bytes to human-readable string.
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format a number with K/M/B suffixes.
pub fn format_number(value: f64) -> String {
    const K: f64 = 1_000.0;
    const M: f64 = 1_000_000.0;
    const B: f64 = 1_000_000_000.0;

    if value >= B {
        format!("{:.1}B", value / B)
    } else if value >= M {
        format!("{:.1}M", value / M)
    } else if value >= K {
        format!("{:.1}K", value / K)
    } else {
        format!("{:.2}", value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(1073741824), "1.00 GB");
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(500.0), "500.00");
        assert_eq!(format_number(1500.0), "1.5K");
        assert_eq!(format_number(1_500_000.0), "1.5M");
        assert_eq!(format_number(1_500_000_000.0), "1.5B");
    }

    #[test]
    fn test_run_with_iterations() {
        let test_fn = || Ok(100.0);
        let result = run_with_iterations(test_fn, "Test", 100.0, 3, 1, 30);

        assert!(result.passed);
        assert_eq!(result.iterations, 3);
        assert_eq!(result.median, 100.0);
        assert!((result.score - 1000.0).abs() < 0.01);
    }

    #[test]
    fn test_run_with_iterations_error() {
        let test_fn = || -> Result<f64> { anyhow::bail!("Test error") };
        let result = run_with_iterations(test_fn, "Test", 100.0, 3, 1, 30);

        assert!(!result.passed);
        assert!(result.error.is_some());
    }
}
