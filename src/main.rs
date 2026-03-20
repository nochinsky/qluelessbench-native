//! QlueLessBench Native - Main entry point.
//!
//! A comprehensive cross-platform system benchmark tool written in Rust.

use anyhow::Result;
use std::process::ExitCode;

use qluelessbench_native::config::parse_args;
use qluelessbench_native::{BenchmarkConfig, BenchmarkRunner};

fn main() -> ExitCode {
    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    // Parse CLI arguments
    let cli = parse_args();

    // Create configuration from CLI
    let config = BenchmarkConfig::from_cli(cli);

    // Initialize logging if verbose
    if config.verbose {
        if let Err(e) = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init()
        {
            eprintln!("Warning: Failed to initialize logging: {}", e);
        }
    }

    // Create runner and execute benchmarks
    let runner = BenchmarkRunner::new(config);
    let _results = runner.run()?;

    Ok(())
}
