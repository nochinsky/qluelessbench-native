//! Configuration and CLI argument parsing.
//!
//! This module defines the command-line interface and benchmark configuration
//! using the clap library.

use clap::Parser;
use std::path::PathBuf;

/// QlueLessBench Native - Cross-platform system benchmark tool.
///
/// A comprehensive benchmark suite for testing real-world everyday system performance.
#[derive(Parser, Debug, Clone, Default)]
#[command(name = "qluelessbench")]
#[command(author = "QlueLess Team")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Cross-platform system benchmark tool", long_about = None)]
pub struct Cli {
    /// Enable verbose logging output
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Number of iterations per test
    #[arg(short = 'n', long, default_value_t = 5)]
    pub iterations: usize,

    /// Number of warmup iterations (discarded)
    #[arg(long, default_value_t = 2)]
    pub warmup: usize,

    /// Timeout per test in seconds
    #[arg(short, long, default_value_t = 30)]
    pub timeout: u64,

    /// Output file path for results JSON
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Compare results against a previous run (path to JSON file)
    #[arg(long)]
    pub compare: Option<PathBuf>,

    /// Print compact summary table instead of verbose output
    #[arg(short, long, default_value_t = false)]
    pub summary: bool,

    /// Run quick smoke test (3 categories, 1 iteration) for fast validation
    #[arg(long, default_value_t = false)]
    pub quick: bool,

    /// Generate default reference values config file and exit
    #[arg(long, default_value_t = false)]
    pub generate_refs: bool,

    /// Custom reference values config file path
    #[arg(long)]
    pub refs_file: Option<PathBuf>,

    /// Filter to run only specific benchmark categories (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub category: Option<Vec<String>>,
}

/// Benchmark configuration.
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Number of iterations per test
    pub iterations: usize,
    /// Number of warmup iterations (discarded)
    pub warmup_iterations: usize,
    /// Timeout per test in seconds
    pub timeout_seconds: u64,
    /// Enable verbose output
    pub verbose: bool,
    /// Output file path for results JSON
    pub output_path: Option<PathBuf>,
    /// Path to previous results for comparison
    pub compare_path: Option<PathBuf>,
    /// Print compact summary table
    pub summary: bool,
    /// Run quick smoke test
    pub quick: bool,
    /// Generate reference values config file
    pub generate_refs: bool,
    /// Custom reference values config file path
    pub refs_file: Option<PathBuf>,
    /// Filter to run only specific benchmark categories
    pub category_filter: Option<Vec<String>>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        BenchmarkConfig {
            iterations: 5,
            warmup_iterations: 2,
            timeout_seconds: 30,
            verbose: false,
            output_path: None,
            compare_path: None,
            summary: false,
            quick: false,
            generate_refs: false,
            refs_file: None,
            category_filter: None,
        }
    }
}

impl BenchmarkConfig {
    pub fn from_cli(cli: Cli) -> Self {
        let mut config = BenchmarkConfig {
            iterations: cli.iterations,
            warmup_iterations: cli.warmup,
            timeout_seconds: cli.timeout,
            verbose: cli.verbose,
            output_path: cli.output,
            compare_path: cli.compare,
            summary: cli.summary,
            quick: cli.quick,
            generate_refs: cli.generate_refs,
            refs_file: cli.refs_file,
            category_filter: cli.category,
        };

        if cli.quick {
            config.iterations = 1;
            config.warmup_iterations = 0;
            config.timeout_seconds = 10;
        }

        config
    }
}

/// Parse CLI arguments and return configuration.
pub fn parse_args() -> Cli {
    Cli::parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.iterations, 5);
        assert_eq!(config.warmup_iterations, 2);
        assert_eq!(config.timeout_seconds, 30);
        assert!(!config.verbose);
        assert!(config.output_path.is_none());
        assert!(config.compare_path.is_none());
    }

    #[test]
    fn test_config_from_cli() {
        let cli = Cli {
            verbose: true,
            iterations: 5,
            warmup: 2,
            timeout: 30,
            output: None,
            compare: None,
            summary: false,
            quick: false,
            generate_refs: false,
            refs_file: None,
            category: None,
        };
        let config = BenchmarkConfig::from_cli(cli);
        assert_eq!(config.iterations, 5);
        assert!(config.verbose);
    }
}
