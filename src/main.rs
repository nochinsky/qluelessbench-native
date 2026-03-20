//! QlueLessBench Native - Main entry point.
//!
//! A comprehensive cross-platform system benchmark tool written in Rust.

use anyhow::Result;
use std::process::ExitCode;

use qluelessbench_native::config::parse_args;
use qluelessbench_native::{BenchmarkConfig, BenchmarkRunner};

fn main() -> ExitCode {
    qluelessbench_native::shutdown::register_handlers();

    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let cli = parse_args();

    let config = BenchmarkConfig::from_cli(cli);

    if config.generate_refs {
        match qluelessbench_native::references::ReferenceValues::save_to_default_location() {
            Ok(path) => {
                println!("Default reference values config saved to:");
                println!("{}", path.display());
                println!("\nEdit this file to customize reference values for calibration.");
                return Ok(());
            }
            Err(e) => {
                eprintln!("Error saving reference values config: {}", e);
                return Err(anyhow::anyhow!("Failed to save config: {}", e));
            }
        }
    }

    if config.verbose {
        if let Err(e) = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init()
        {
            eprintln!("Warning: Failed to initialize logging: {}", e);
        }
        eprintln!("Press Ctrl+C to interrupt benchmark...");
    }

    let runner = BenchmarkRunner::new(config);
    let result = runner.run();

    if qluelessbench_native::shutdown::is_interrupted() {
        eprintln!("\nBenchmark interrupted by user. Partial results saved if available.");
        std::process::exit(0);
    }

    let _results = result?;

    Ok(())
}
