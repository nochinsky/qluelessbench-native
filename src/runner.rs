//! Main benchmark runner orchestrator.
//!
//! This module coordinates the execution of all benchmark categories
//! and aggregates the results.

use anyhow::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Instant;

use crate::benchmarks::{
    ArchiveBenchmark, BaseBenchmark, CompressionBenchmark, CryptoBenchmark, DatabaseBenchmark,
    FileIOBenchmark, ImageFiltersBenchmark, ImageProcessingBenchmark, MLInferenceBenchmark,
    MathematicalBenchmark, MemoryBenchmark, NavigationBenchmark, RayTracingBenchmark,
    TextProcessingBenchmark,
};
use crate::config::BenchmarkConfig;
use crate::hardware::get_system_info;
use crate::results::{BenchmarkMetadata, BenchmarkResults, CategoryResult};

type BenchmarkPair = (&'static str, Box<dyn BaseBenchmark>, Box<dyn BaseBenchmark>);
use crate::VERSION;

/// Main benchmark runner.
pub struct BenchmarkRunner {
    config: BenchmarkConfig,
}

impl BenchmarkRunner {
    /// Create a new BenchmarkRunner with the given configuration.
    pub fn new(config: BenchmarkConfig) -> Self {
        BenchmarkRunner { config }
    }

    /// Run all benchmarks and return results.
    pub fn run(&self) -> Result<BenchmarkResults> {
        let start_time = Instant::now();

        // Print header
        self.print_header();

        // Get system info
        let system_info = get_system_info();

        // Create metadata
        let metadata = BenchmarkMetadata {
            duration: 0.0,
            completed: chrono::Utc::now(),
            version: VERSION.to_string(),
            rust_version: get_rust_version(),
        };

        let mut results = BenchmarkResults::new(system_info, metadata);

        // Run all benchmarks
        let benchmarks = self.get_all_benchmarks();
        let total = benchmarks.len();
        let total_tests = total * 2; // single + multi

        // Create progress bar
        let pb = ProgressBar::new(total_tests as u64);
        pb.set_style(
            ProgressStyle::with_template(
                "  [{elapsed_precise}] [{bar:30.cyan/blue}] {pos}/{len} {msg}",
            )
            .unwrap()
            .progress_chars("=> "),
        );

        println!("\n{}", style("═══ Single-Core Tests ═══").bright());

        for (i, (name, single_core_bench, _multi_core_bench)) in benchmarks.iter().enumerate() {
            let msg = format!("[{}/{}] {}", i + 1, total, name);
            pb.set_message(msg.clone());

            let category_result = single_core_bench.run_all(
                self.config.iterations,
                self.config.warmup_iterations,
                self.config.timeout_seconds,
            );

            match category_result {
                Ok(result) => {
                    self.print_category_result(name, &result);
                    results.single_core_categories.push(result);
                }
                Err(e) => {
                    self.print_category_error(name, &e.to_string());
                    results.single_core_categories.push(CategoryResult {
                        category: name.to_string(),
                        score: 0.0,
                        duration: 0.0,
                        weight: single_core_bench.weight(),
                        tests: Vec::new(),
                    });
                }
            }
            pb.inc(1);
        }

        println!("\n{}", style("═══ Multi-Core Tests ═══").bright());

        for (i, (name, _single_core_bench, multi_core_bench)) in benchmarks.iter().enumerate() {
            let msg = format!("[{}/{}] {}", i + 1, total, name);
            pb.set_message(msg.clone());

            let category_result = multi_core_bench.run_all(
                self.config.iterations,
                self.config.warmup_iterations,
                self.config.timeout_seconds,
            );

            match category_result {
                Ok(result) => {
                    self.print_category_result(name, &result);
                    results.multi_core_categories.push(result);
                }
                Err(e) => {
                    self.print_category_error(name, &e.to_string());
                    results.multi_core_categories.push(CategoryResult {
                        category: name.to_string(),
                        score: 0.0,
                        duration: 0.0,
                        weight: multi_core_bench.weight(),
                        tests: Vec::new(),
                    });
                }
            }
            pb.inc(1);
        }

        pb.finish_and_clear();

        // Calculate scores
        results.calculate_scores();
        results.metadata.duration = start_time.elapsed().as_secs_f64();

        // Load comparison results if requested
        let comparison = self.load_comparison();

        // Print final results
        self.print_final_results(&results, comparison.as_ref());

        // Save results
        self.save_results(&results);

        Ok(results)
    }

    /// Load comparison results if --compare was specified.
    fn load_comparison(&self) -> Option<BenchmarkResults> {
        let compare_path = self.config.compare_path.as_ref()?;
        match BenchmarkResults::load_from_file(compare_path) {
            Ok(prev) => Some(prev),
            Err(e) => {
                eprintln!("Warning: Could not load comparison file: {}", e);
                None
            }
        }
    }

    /// Save results to file.
    fn save_results(&self, results: &BenchmarkResults) {
        let output_path = self.config.output_path.clone().unwrap_or_else(|| {
            let timestamp = results.metadata.completed.format("%Y%m%d_%H%M%S");
            std::path::PathBuf::from(format!("qluelessbench_{}.json", timestamp))
        });

        match results.save_to_file(&output_path) {
            Ok(()) => {
                println!(
                    "  Results saved to: {}",
                    style(output_path.display()).cyan()
                );
            }
            Err(e) => {
                eprintln!("Warning: Could not save results: {}", e);
            }
        }
    }

    /// Get all benchmark categories.
    fn get_all_benchmarks(&self) -> Vec<BenchmarkPair> {
        vec![
            (
                "FileIO",
                Box::new(FileIOBenchmark::new()),
                Box::new(FileIOBenchmark::new_multi_core()),
            ),
            (
                "Compression",
                Box::new(CompressionBenchmark::new()),
                Box::new(CompressionBenchmark::new_multi_core()),
            ),
            (
                "ImageProcessing",
                Box::new(ImageProcessingBenchmark::new()),
                Box::new(ImageProcessingBenchmark::new_multi_core()),
            ),
            (
                "TextProcessing",
                Box::new(TextProcessingBenchmark::new()),
                Box::new(TextProcessingBenchmark::new_multi_core()),
            ),
            (
                "Database",
                Box::new(DatabaseBenchmark::new()),
                Box::new(DatabaseBenchmark::new_multi_core()),
            ),
            (
                "Mathematical",
                Box::new(MathematicalBenchmark::new()),
                Box::new(MathematicalBenchmark::new_multi_core()),
            ),
            (
                "Archive",
                Box::new(ArchiveBenchmark::new()),
                Box::new(ArchiveBenchmark::new_multi_core()),
            ),
            (
                "Memory",
                Box::new(MemoryBenchmark::new()),
                Box::new(MemoryBenchmark::new_multi_core()),
            ),
            (
                "Cryptography",
                Box::new(CryptoBenchmark::new()),
                Box::new(CryptoBenchmark::new_multi_core()),
            ),
            (
                "RayTracing",
                Box::new(RayTracingBenchmark::new()),
                Box::new(RayTracingBenchmark::new_multi_core()),
            ),
            (
                "MLInference",
                Box::new(MLInferenceBenchmark::new()),
                Box::new(MLInferenceBenchmark::new_multi_core()),
            ),
            (
                "Navigation",
                Box::new(NavigationBenchmark::new()),
                Box::new(NavigationBenchmark::new_multi_core()),
            ),
            (
                "ImageFilters",
                Box::new(ImageFiltersBenchmark::new()),
                Box::new(ImageFiltersBenchmark::new_multi_core()),
            ),
        ]
    }

    /// Print the benchmark header.
    fn print_header(&self) {
        println!();
        println!(
            "{}",
            style("╔════════════════════════════════════════════════╗").bright()
        );
        println!(
            "{}",
            style("║   QlueLessBench Native v").bright().to_string() + &format!("{:<29}║", VERSION)
        );
        println!(
            "{}",
            style("╚════════════════════════════════════════════════╝").bright()
        );
        println!();
    }

    /// Print category result.
    fn print_category_result(&self, name: &str, result: &CategoryResult) {
        let score = result.score;
        let duration = result.duration;
        let check = if score >= 500.0 {
            style("✓").green()
        } else {
            style("⚠").yellow()
        };
        println!(
            "    {} {}: {:.0} ({:.2}s)",
            check,
            style(name).bold(),
            score,
            duration
        );
        println!();
    }

    /// Print category error.
    fn print_category_error(&self, name: &str, error: &str) {
        println!(
            "    {} {}: {}",
            style("✗").red(),
            style(name).bold(),
            style(error).red()
        );
        println!();
    }

    /// Print final results summary with optional comparison.
    fn print_final_results(
        &self,
        results: &BenchmarkResults,
        comparison: Option<&BenchmarkResults>,
    ) {
        println!();
        println!(
            "{}",
            style("════════════════════════════════════════════════").bright()
        );
        println!(
            "{}",
            style("              QlueLessBench Native Results").bold()
        );
        println!(
            "{}",
            style("════════════════════════════════════════════════").bright()
        );
        println!();

        println!(
            "{}",
            style("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━").bright()
        );
        if let Some(prev) = comparison {
            self.print_score_with_comparison(
                "Single-Core",
                results.single_core_score,
                prev.single_core_score,
            );
            self.print_score_with_comparison(
                "Multi-Core",
                results.multi_core_score,
                prev.multi_core_score,
            );
        } else {
            println!(
                "  {:<22} {:>10}",
                style("Single-Core").bold(),
                style(format!("{:.0}", results.single_core_score))
                    .bright()
                    .green()
            );
            println!(
                "  {:<22} {:>10}",
                style("Multi-Core").bold(),
                style(format!("{:.0}", results.multi_core_score))
                    .bright()
                    .green()
            );
        }
        println!(
            "{}",
            style("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━").bright()
        );
        println!();

        // System info summary
        println!("{}", style("System Information:").bold());
        println!(
            "{}",
            style("────────────────────────────────────────").bright()
        );
        println!("  Platform:     {}", results.system_info.platform_release);
        if let Some(ref cpu) = results.system_info.cpu {
            println!("  CPU:          {}", cpu);
        }
        println!("  CPU Cores:    {}", results.system_info.cpu_count_logical);
        println!(
            "  Memory:       {:.1} GB",
            results.system_info.memory_total_gb
        );
        if let Some(ref gpu) = results.system_info.gpu {
            println!("  GPU:          {}", gpu);
        }
        println!();

        // Summary
        println!(
            "{}",
            style("════════════════════════════════════════════════").bright()
        );
        println!("  Total Duration: {:.2} seconds", results.metadata.duration);
        println!(
            "  Completed:    {}",
            results.metadata.completed.format("%Y-%m-%d %H:%M:%S")
        );
        println!(
            "{}",
            style("════════════════════════════════════════════════").bright()
        );
        println!();
    }

    /// Print a score with comparison delta.
    fn print_score_with_comparison(&self, label: &str, current: f64, previous: f64) {
        let delta = current - previous;
        let pct = if previous != 0.0 {
            (delta / previous) * 100.0
        } else {
            0.0
        };

        let arrow = if delta > 0.0 {
            style("↑").green()
        } else if delta < 0.0 {
            style("↓").red()
        } else {
            style("→").yellow()
        };

        let sign = if delta >= 0.0 { "+" } else { "" };

        println!(
            "  {:<16} {:>8}  (was {:>7}) {} {}{:.1}%",
            style(label).bold(),
            style(format!("{:.0}", current)).bright().green(),
            format!("{:.0}", previous),
            arrow,
            sign,
            pct,
        );
    }
}

/// Get the Rust compiler version.
fn get_rust_version() -> String {
    std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string())
}
