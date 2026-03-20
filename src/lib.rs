//! QlueLessBench Native - Cross-platform system benchmark tool.
//!
//! A comprehensive benchmark suite written in Rust for testing real-world
//! everyday system performance with statistical rigor.
//!
//! # Features
//!
//! - **Cross-platform**: Windows, Linux, macOS
//! - **No admin privileges required**: Runs entirely in user space
//! - **Statistical rigor**: Median, percentile, CV analysis
//! - **Clean output**: Visual progress bars and formatted scores
//! - **Automatic cleanup**: All test files are automatically removed
//! - **Progressive difficulty**: Quick (~30s) and Full (~5min) test modes
//!
//! # Example
//!
//! ```rust,no_run
//! use qluelessbench_native::runner::BenchmarkRunner;
//! use qluelessbench_native::config::BenchmarkConfig;
//!
//! let config = BenchmarkConfig::default();
//! let runner = BenchmarkRunner::new(config);
//! let results = runner.run().expect("Benchmark failed");
//! println!("Overall score: {}", results.overall_score);
//! ```

pub mod benchmarks;
pub mod config;
pub mod hardware;
pub mod references;
pub mod results;
pub mod runner;
pub mod shutdown;
pub mod stats;

pub use config::BenchmarkConfig;
pub use results::{BenchmarkResults, CategoryResult, TestResult};
pub use runner::BenchmarkRunner;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name
pub const NAME: &str = "QlueLessBench Native";
