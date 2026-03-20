//! Result types and serialization for benchmark results.
//!
//! This module defines the data structures for storing and serializing
//! benchmark results, including individual test results, category results,
//! and the overall benchmark results.

use crate::stats::Reliability;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Result of a single benchmark test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Name of the test
    pub name: String,
    /// Whether the test passed (completed without error)
    pub passed: bool,
    /// Duration of the test in seconds
    pub duration: f64,
    /// Score (0-1000)
    pub score: f64,
    /// Median value of iterations
    pub median: f64,
    /// 95th percentile value
    pub p95: f64,
    /// 99th percentile value
    pub p99: f64,
    /// Coefficient of variation (percentage)
    pub cv: f64,
    /// Number of iterations
    pub iterations: usize,
    /// Reliability classification
    pub reliability: Reliability,
    /// Additional metrics specific to this test
    pub metrics: serde_json::Value,
    /// Error message if the test failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl TestResult {
    /// Create a new TestResult with default values.
    pub fn new(name: String) -> Self {
        TestResult {
            name,
            passed: true,
            duration: 0.0,
            score: 0.0,
            median: 0.0,
            p95: 0.0,
            p99: 0.0,
            cv: 0.0,
            iterations: 0,
            reliability: Reliability::Unknown,
            metrics: serde_json::Value::Object(serde_json::Map::new()),
            error: None,
        }
    }

    /// Create a failed TestResult with an error message.
    pub fn failed(name: String, error: String) -> Self {
        TestResult {
            name: name.clone(),
            passed: false,
            error: Some(error),
            ..Self::new(name)
        }
    }

    /// Get the reliability icon.
    pub fn reliability_icon(&self) -> &'static str {
        self.reliability.icon()
    }

    /// Check if the test results are statistically valid.
    pub fn is_statistically_valid(&self) -> bool {
        self.iterations >= 3 && self.cv <= 10.0
    }
}

/// Result of a benchmark category (group of tests).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryResult {
    /// Category name
    pub category: String,
    /// Category score (0-1000)
    pub score: f64,
    /// Total duration of all tests in seconds
    pub duration: f64,
    /// Weight for this category in overall score calculation
    pub weight: f64,
    /// Individual test results
    pub tests: Vec<TestResult>,
}

impl CategoryResult {
    /// Create a new CategoryResult.
    pub fn new(category: String) -> Self {
        CategoryResult {
            category,
            score: 0.0,
            duration: 0.0,
            weight: 1.0,
            tests: Vec::new(),
        }
    }

    /// Get the number of tests that passed statistical validation.
    pub fn valid_test_count(&self) -> usize {
        self.tests
            .iter()
            .filter(|t| t.is_statistically_valid())
            .count()
    }

    /// Get the total number of tests.
    pub fn total_test_count(&self) -> usize {
        self.tests.len()
    }
}

/// System information collected during benchmark.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemInfo {
    /// Platform name (Windows, Linux, macOS)
    pub platform: String,
    /// Platform release/version
    pub platform_release: String,
    /// CPU model/name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<String>,
    /// CPU frequency in MHz
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_frequency_mhz: Option<u64>,
    /// Number of logical CPU cores
    pub cpu_count_logical: usize,
    /// Number of physical CPU cores
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_count_physical: Option<usize>,
    /// Total memory in GB
    pub memory_total_gb: f64,
    /// GPU information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu: Option<String>,
    /// Storage information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<Vec<StorageInfo>>,
}

/// Storage device information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    /// Storage type (SSD, HDD, NVMe)
    pub r#type: String,
    /// Total capacity in GB
    pub total_gb: f64,
    /// Whether this is the primary drive
    pub is_primary: bool,
}

/// Metadata about the benchmark run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetadata {
    /// Total duration in seconds
    pub duration: f64,
    /// Completion timestamp
    pub completed: DateTime<Utc>,
    /// QlueLessBench version
    pub version: String,
    /// Rust version used
    pub rust_version: String,
}

/// Complete benchmark results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResults {
    /// System information
    pub system_info: SystemInfo,
    /// Overall score (0-1000)
    pub overall_score: f64,
    /// Single-core score (0-1000) - from single-threaded tests
    pub single_core_score: f64,
    /// Multi-core score (0-1000) - from multi-threaded tests
    pub multi_core_score: f64,
    /// Category results for single-core tests
    pub single_core_categories: Vec<CategoryResult>,
    /// Category results for multi-core tests
    pub multi_core_categories: Vec<CategoryResult>,
    /// Metadata about the run
    pub metadata: BenchmarkMetadata,
    /// Number of tests that passed statistical validation
    pub valid_test_count: usize,
    /// Total number of tests
    pub total_test_count: usize,
}

impl BenchmarkResults {
    /// Create new BenchmarkResults.
    pub fn new(system_info: SystemInfo, metadata: BenchmarkMetadata) -> Self {
        BenchmarkResults {
            system_info,
            overall_score: 0.0,
            single_core_score: 0.0,
            multi_core_score: 0.0,
            single_core_categories: Vec::new(),
            multi_core_categories: Vec::new(),
            metadata,
            valid_test_count: 0,
            total_test_count: 0,
        }
    }

    /// Calculate scores from single-core and multi-core category results.
    pub fn calculate_scores(&mut self) {
        // Calculate single-core score (weighted average of single-core categories)
        if !self.single_core_categories.is_empty() {
            let total_weight: f64 = self.single_core_categories.iter().map(|c| c.weight).sum();
            self.single_core_score = self
                .single_core_categories
                .iter()
                .map(|c| c.score * c.weight)
                .sum::<f64>()
                / total_weight;
        }

        // Calculate multi-core score (weighted average of multi-core categories)
        if !self.multi_core_categories.is_empty() {
            let total_weight: f64 = self.multi_core_categories.iter().map(|c| c.weight).sum();
            self.multi_core_score = self
                .multi_core_categories
                .iter()
                .map(|c| c.score * c.weight)
                .sum::<f64>()
                / total_weight;
        }

        // Overall score is the average of single and multi-core scores
        if self.single_core_score > 0.0 && self.multi_core_score > 0.0 {
            self.overall_score = (self.single_core_score + self.multi_core_score) / 2.0;
        } else {
            self.overall_score = self.single_core_score.max(self.multi_core_score);
        }

        // Update valid test counts
        self.valid_test_count = self
            .single_core_categories
            .iter()
            .map(|c| c.valid_test_count())
            .sum::<usize>()
            + self
                .multi_core_categories
                .iter()
                .map(|c| c.valid_test_count())
                .sum::<usize>();
        self.total_test_count = self
            .single_core_categories
            .iter()
            .map(|c| c.total_test_count())
            .sum::<usize>()
            + self
                .multi_core_categories
                .iter()
                .map(|c| c.total_test_count())
                .sum::<usize>();
    }

    /// Export results to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Save results to a JSON file.
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), anyhow::Error> {
        let json = self.to_json()?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load results from a JSON file.
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, anyhow::Error> {
        let json = std::fs::read_to_string(path)?;
        let results: BenchmarkResults = serde_json::from_str(&json)?;
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_result_new() {
        let result = TestResult::new("Test".to_string());
        assert_eq!(result.name, "Test");
        assert!(result.passed);
        assert_eq!(result.score, 0.0);
        assert_eq!(result.reliability, Reliability::Unknown);
    }

    #[test]
    fn test_test_result_failed() {
        let result = TestResult::failed("Test".to_string(), "Error".to_string());
        assert!(!result.passed);
        assert_eq!(result.error, Some("Error".to_string()));
    }

    #[test]
    fn test_category_result_new() {
        let result = CategoryResult::new("File I/O".to_string());
        assert_eq!(result.category, "File I/O");
        assert_eq!(result.score, 0.0);
        assert!(result.tests.is_empty());
    }

    #[test]
    fn test_benchmark_results_new() {
        let system_info = SystemInfo::default();
        let metadata = BenchmarkMetadata {
            duration: 10.0,
            completed: Utc::now(),
            version: "0.1.0".to_string(),
            rust_version: "1.75.0".to_string(),
        };

        let results = BenchmarkResults::new(system_info, metadata);
        assert_eq!(results.overall_score, 0.0);
        assert!(results.single_core_categories.is_empty());
        assert!(results.multi_core_categories.is_empty());
    }

    #[test]
    fn test_benchmark_results_calculate_scores() {
        let system_info = SystemInfo::default();
        let metadata = BenchmarkMetadata {
            duration: 10.0,
            completed: Utc::now(),
            version: "0.1.0".to_string(),
            rust_version: "1.75.0".to_string(),
        };

        let mut results = BenchmarkResults::new(system_info, metadata);

        let mut cat1 = CategoryResult::new("Category 1".to_string());
        cat1.score = 800.0;
        cat1.weight = 1.0;

        let mut cat2 = CategoryResult::new("Category 2".to_string());
        cat2.score = 900.0;
        cat2.weight = 2.0;

        results.single_core_categories.push(cat1);
        results.multi_core_categories.push(cat2);

        results.calculate_scores();

        // Weighted: single = 800*1/1 = 800, multi = 900*2/2 = 900
        assert!((results.single_core_score - 800.0).abs() < 0.01);
        assert!((results.multi_core_score - 900.0).abs() < 0.01);
        assert!((results.overall_score - 850.0).abs() < 0.01);
    }

    #[test]
    fn test_benchmark_results_save_and_load() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let system_info = SystemInfo::default();
        let metadata = BenchmarkMetadata {
            duration: 10.0,
            completed: Utc::now(),
            version: "0.1.0".to_string(),
            rust_version: "1.75.0".to_string(),
        };

        let results = BenchmarkResults::new(system_info, metadata);

        let mut file = NamedTempFile::new().unwrap();
        let json = results.to_json().unwrap();
        file.write_all(json.as_bytes()).unwrap();

        let loaded = BenchmarkResults::load_from_file(file.path()).unwrap();
        assert_eq!(loaded.metadata.version, "0.1.0");
        assert_eq!(loaded.overall_score, 0.0);
    }
}
