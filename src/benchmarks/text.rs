//! Text processing benchmark tests.
//!
//! Tests search/replace, regex, string manipulation, and encoding.

use anyhow::Result;
use rayon::prelude::*;
use regex::Regex;
use std::time::Instant;

use crate::benchmarks::base::{
    calculate_category_score, get_parallel_workers, run_with_iterations, BaseBenchmark,
};
use crate::results::CategoryResult;

/// Text processing benchmark.
pub struct TextProcessingBenchmark {
    /// If true, run tests in parallel (multi-core mode)
    multi_core: bool,
}

impl TextProcessingBenchmark {
    /// Create a new TextProcessingBenchmark.
    pub fn new() -> Self {
        TextProcessingBenchmark { multi_core: false }
    }

    /// Create a new TextProcessingBenchmark for multi-core testing.
    pub fn new_multi_core() -> Self {
        TextProcessingBenchmark { multi_core: true }
    }

    /// Generate test text.
    fn generate_test_text(size_kb: usize) -> String {
        let mut text = String::with_capacity(size_kb * 1024);
        let sample = "The quick brown fox jumps over the lazy dog. ";
        while text.len() < size_kb * 1024 {
            text.push_str(sample);
        }
        text
    }

    /// Test search and replace.
    fn test_search_replace(text: &str) -> Result<f64> {
        let start = Instant::now();
        let _result = text.replace("fox", "cat");
        let duration = start.elapsed().as_secs_f64();
        Ok(text.len() as f64 / 1024.0 / duration)
    }

    /// Test regex operations.
    fn test_regex(text: &str) -> Result<f64> {
        let re = Regex::new(r"\b\w{5}\b")?; // Match 5-letter words
        let start = Instant::now();
        let _matches: Vec<_> = re.find_iter(text).collect();
        let duration = start.elapsed().as_secs_f64();
        Ok(text.len() as f64 / 1024.0 / duration)
    }

    /// Test string manipulation.
    fn test_string_manipulation(text: &str) -> Result<f64> {
        let start = Instant::now();
        let _upper = text.to_uppercase();
        let _lower = text.to_lowercase();
        let _trimmed = text.trim();
        let _split: Vec<_> = text.split(' ').collect();
        let _joined = text.split(' ').collect::<Vec<_>>().join("-");
        let duration = start.elapsed().as_secs_f64();
        Ok(text.len() as f64 / 1024.0 / duration)
    }

    /// Test base64 encoding/decoding.
    fn test_encoding(text: &str) -> Result<f64> {
        use base64::{engine::general_purpose, Engine as _};
        let start = Instant::now();
        let encoded = general_purpose::STANDARD.encode(text.as_bytes());
        let _decoded = general_purpose::STANDARD.decode(&encoded)?;
        let duration = start.elapsed().as_secs_f64();
        Ok(text.len() as f64 / 1024.0 / duration)
    }

    /// Test parallel search and replace.
    fn test_parallel_search_replace(texts: &[String]) -> Result<f64> {
        let start = Instant::now();

        texts.par_iter().try_for_each(|text| -> Result<()> {
            let _result = text.replace("fox", "cat");
            Ok(())
        })?;

        let duration = start.elapsed().as_secs_f64();
        Ok(texts.len() as f64 * texts[0].len() as f64 / 1024.0 / duration)
    }

    /// Test parallel regex operations.
    fn test_parallel_regex(texts: &[String]) -> Result<f64> {
        let re = Regex::new(r"\b\w{5}\b")?; // Match 5-letter words
        let start = Instant::now();

        texts.par_iter().try_for_each(|text| -> Result<()> {
            let _matches: Vec<_> = re.find_iter(text).collect();
            Ok(())
        })?;

        let duration = start.elapsed().as_secs_f64();
        Ok(texts.len() as f64 * texts[0].len() as f64 / 1024.0 / duration)
    }

    /// Test parallel string manipulation.
    fn test_parallel_string_manipulation(texts: &[String]) -> Result<f64> {
        let start = Instant::now();

        texts.par_iter().try_for_each(|text| -> Result<()> {
            let _upper = text.to_uppercase();
            let _lower = text.to_lowercase();
            let _trimmed = text.trim();
            let _split: Vec<_> = text.split(' ').collect();
            let _joined = text.split(' ').collect::<Vec<_>>().join("-");
            Ok(())
        })?;

        let duration = start.elapsed().as_secs_f64();
        Ok(texts.len() as f64 * texts[0].len() as f64 / 1024.0 / duration)
    }

    /// Test parallel base64 encoding/decoding.
    fn test_parallel_encoding(texts: &[String]) -> Result<f64> {
        use base64::{engine::general_purpose, Engine as _};
        let start = Instant::now();

        texts.par_iter().try_for_each(|text| -> Result<()> {
            let encoded = general_purpose::STANDARD.encode(text.as_bytes());
            let _decoded = general_purpose::STANDARD.decode(&encoded)?;
            Ok(())
        })?;

        let duration = start.elapsed().as_secs_f64();
        Ok(texts.len() as f64 * texts[0].len() as f64 / 1024.0 / duration)
    }

    /// Generate multiple texts for parallel processing.
    /// Throughput model: Each worker processes a full-size text.
    /// N workers process N texts = N× the work.
    fn generate_test_texts(size_kb: usize, num_texts: usize) -> Vec<String> {
        (0..num_texts)
            .map(|_| Self::generate_test_text(size_kb))
            .collect()
    }
}

impl Default for TextProcessingBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseBenchmark for TextProcessingBenchmark {
    fn category_name(&self) -> &'static str {
        "Text Processing"
    }

    fn run_all(&self, iterations: usize, warmup: usize, timeout: u64) -> Result<CategoryResult> {
        let mut results = Vec::new();
        let mut total_duration = 0.0;

        // Generate test text once (1MB)
        let test_text = Self::generate_test_text(1024);

        // Reference values (KB/s)
        // Same reference values used for both single-core and multi-core modes
        let (search_ref, regex_ref, string_ref, encoding_ref) = (10000.0, 5000.0, 8000.0, 5000.0);

        if self.multi_core {
            // Multi-core: process multiple full-size texts in parallel (throughput model)
            // N workers process N texts of the same size = N× the work
            let num_workers = get_parallel_workers();
            let texts = Self::generate_test_texts(1024, num_workers);

            let test_fn = || Self::test_parallel_search_replace(&texts);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel Search/Replace ({} texts)", num_workers),
                search_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let test_fn = || Self::test_parallel_regex(&texts);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel Regex ({} texts)", num_workers),
                regex_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let test_fn = || Self::test_parallel_string_manipulation(&texts);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel String Manipulation ({} texts)", num_workers),
                string_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            let test_fn = || Self::test_parallel_encoding(&texts);
            let result = run_with_iterations(
                test_fn,
                &format!("Parallel Encoding/Decoding ({} texts)", num_workers),
                encoding_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);
        } else {
            // Single-core tests
            // Test 1: Search/Replace
            let test_fn = || Self::test_search_replace(&test_text);
            let result = run_with_iterations(
                test_fn,
                "Search/Replace",
                search_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Test 2: Regex
            let test_fn = || Self::test_regex(&test_text);
            let result = run_with_iterations(
                test_fn,
                "Regex Operations",
                regex_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Test 3: String Manipulation
            let test_fn = || Self::test_string_manipulation(&test_text);
            let result = run_with_iterations(
                test_fn,
                "String Manipulation",
                string_ref,
                iterations,
                warmup,
                timeout,
            );
            total_duration += result.duration;
            results.push(result);

            // Test 4: Encoding/Decoding
            let test_fn = || Self::test_encoding(&test_text);
            let result = run_with_iterations(
                test_fn,
                "Encoding/Decoding",
                encoding_ref,
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
    fn test_text_category_name() {
        let benchmark = TextProcessingBenchmark::new();
        assert_eq!(benchmark.category_name(), "Text Processing");
    }

    #[test]
    fn test_generate_test_text() {
        let text = TextProcessingBenchmark::generate_test_text(10);
        assert!(text.len() >= 10 * 1024);
    }
}
