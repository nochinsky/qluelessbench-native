//! Main benchmark runner orchestrator.
//!
//! This module coordinates the execution of all benchmark categories
//! and aggregates the results.

use anyhow::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Instant;

use crate::benchmark_entry;
use crate::benchmarks::BenchmarkEntry;
use crate::config::BenchmarkConfig;
use crate::hardware::get_system_info;
use crate::results::{BenchmarkMetadata, BenchmarkResults, CategoryResult};
use crate::shutdown::is_interrupted;
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

        if self.config.quick {
            println!("{}", style("⚠ QUICK SMOKE TEST MODE ⚠").yellow().bold());
            println!("Running 3 categories with 1 iteration each for fast validation\n");
        }

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

        for (i, entry) in benchmarks.iter().enumerate() {
            if is_interrupted() {
                pb.finish_and_clear();
                println!(
                    "\n{}",
                    style("Interrupted! Saving partial results...").yellow()
                );
                results.calculate_scores();
                results.metadata.duration = start_time.elapsed().as_secs_f64();
                self.save_results(&results);
                return Ok(results);
            }

            let msg = format!("[{}/{}] {}", i + 1, total, entry.name);
            pb.set_message(msg.clone());

            let category_result = entry.single.run_all(
                self.config.iterations,
                self.config.warmup_iterations,
                self.config.timeout_seconds,
            );

            match category_result {
                Ok(result) => {
                    self.print_category_result(entry.name, &result);
                    results.single_core_categories.push(result);
                }
                Err(e) => {
                    self.print_category_error(entry.name, &e.to_string());
                    results.single_core_categories.push(CategoryResult {
                        category: entry.name.to_string(),
                        score: 0.0,
                        duration: 0.0,
                        weight: entry.single.weight(),
                        tests: Vec::new(),
                    });
                }
            }
            pb.inc(1);
        }

        println!("\n{}", style("═══ Multi-Core Tests ═══").bright());

        for (i, entry) in benchmarks.iter().enumerate() {
            if is_interrupted() {
                pb.finish_and_clear();
                println!(
                    "\n{}",
                    style("Interrupted! Saving partial results...").yellow()
                );
                results.calculate_scores();
                results.metadata.duration = start_time.elapsed().as_secs_f64();
                self.save_results(&results);
                return Ok(results);
            }

            let msg = format!("[{}/{}] {}", i + 1, total, entry.name);
            pb.set_message(msg.clone());

            let category_result = entry.multi.run_all(
                self.config.iterations,
                self.config.warmup_iterations,
                self.config.timeout_seconds,
            );

            match category_result {
                Ok(result) => {
                    self.print_category_result(entry.name, &result);
                    results.multi_core_categories.push(result);
                }
                Err(e) => {
                    self.print_category_error(entry.name, &e.to_string());
                    results.multi_core_categories.push(CategoryResult {
                        category: entry.name.to_string(),
                        score: 0.0,
                        duration: 0.0,
                        weight: entry.multi.weight(),
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
        if self.config.summary {
            self.print_summary(&results, comparison.as_ref());
        } else {
            self.print_final_results(&results, comparison.as_ref());
        }

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
    fn get_all_benchmarks(&self) -> Vec<BenchmarkEntry> {
        use crate::benchmarks::{
            ArchiveBenchmark, CompressionBenchmark, ConcurrentBenchmark, CryptoBenchmark,
            DatabaseBenchmark, FileIOBenchmark, ImageFiltersBenchmark, ImageProcessingBenchmark,
            MLInferenceBenchmark, MathematicalBenchmark, MemoryBenchmark, NavigationBenchmark,
            RayTracingBenchmark, TextProcessingBenchmark,
        };

        if self.config.quick {
            return vec![
                benchmark_entry!("FileIO", FileIOBenchmark),
                benchmark_entry!("Compression", CompressionBenchmark),
                benchmark_entry!("Mathematical", MathematicalBenchmark),
            ];
        }

        vec![
            benchmark_entry!("FileIO", FileIOBenchmark),
            benchmark_entry!("Compression", CompressionBenchmark),
            benchmark_entry!("ImageProcessing", ImageProcessingBenchmark),
            benchmark_entry!("TextProcessing", TextProcessingBenchmark),
            benchmark_entry!("Database", DatabaseBenchmark),
            benchmark_entry!("Mathematical", MathematicalBenchmark),
            benchmark_entry!("Archive", ArchiveBenchmark),
            benchmark_entry!("Memory", MemoryBenchmark),
            benchmark_entry!("Concurrent", ConcurrentBenchmark),
            benchmark_entry!("Cryptography", CryptoBenchmark),
            benchmark_entry!("RayTracing", RayTracingBenchmark),
            benchmark_entry!("MLInference", MLInferenceBenchmark),
            benchmark_entry!("Navigation", NavigationBenchmark),
            benchmark_entry!("ImageFilters", ImageFiltersBenchmark),
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

    /// Print compact summary table.
    fn print_summary(&self, results: &BenchmarkResults, comparison: Option<&BenchmarkResults>) {
        println!();
        println!(
            "{}",
            style(format!(
                "  QlueLessBench v{} | {} | {} cores",
                results.metadata.version,
                results.system_info.platform_release,
                results.system_info.cpu_count_logical,
            ))
            .dim()
        );
        println!();
        println!(
            "  {:<22} {:>10}  {:<22} {:>10}",
            style("Category").bold(),
            style("Score").bold(),
            style("Category").bold(),
            style("Score").bold(),
        );
        println!("  {}", "─".repeat(60));

        // Print categories in two columns: single-core left, multi-core right
        let max_len = results
            .single_core_categories
            .len()
            .max(results.multi_core_categories.len());

        for i in 0..max_len {
            let left = results.single_core_categories.get(i);
            let right = results.multi_core_categories.get(i);

            let left_str = left
                .map(|c| format!("{:<22} {:>10.0}", c.category, c.score))
                .unwrap_or_default();
            let right_str = right
                .map(|c| format!("{:<22} {:>10.0}", c.category, c.score))
                .unwrap_or_default();

            println!("  {}  {}", left_str, right_str);
        }

        println!("  {}", "─".repeat(60));

        if let Some(prev) = comparison {
            let sc_delta = results.single_core_score - prev.single_core_score;
            let mc_delta = results.multi_core_score - prev.multi_core_score;
            let sc_pct = if prev.single_core_score != 0.0 {
                (sc_delta / prev.single_core_score) * 100.0
            } else {
                0.0
            };
            let mc_pct = if prev.multi_core_score != 0.0 {
                (mc_delta / prev.multi_core_score) * 100.0
            } else {
                0.0
            };
            let sc_arrow = if sc_delta > 0.0 {
                "↑"
            } else if sc_delta < 0.0 {
                "↓"
            } else {
                "→"
            };
            let mc_arrow = if mc_delta > 0.0 {
                "↑"
            } else if mc_delta < 0.0 {
                "↓"
            } else {
                "→"
            };

            println!(
                "  {:<22} {:>10.0}  {:<22} {:>10.0}",
                style("Single-Core").bold(),
                style(format!("{:.0}", results.single_core_score))
                    .bright()
                    .green(),
                style("Multi-Core").bold(),
                style(format!("{:.0}", results.multi_core_score))
                    .bright()
                    .green(),
            );
            println!(
                "  {:<22} {:>10}  {:<22} {:>10}",
                "",
                format!("{} {:+.1}%", sc_arrow, sc_pct),
                "",
                format!("{} {:+.1}%", mc_arrow, mc_pct),
            );
        } else {
            println!(
                "  {:<22} {:>10}  {:<22} {:>10}",
                style("Single-Core").bold(),
                style(format!("{:.0}", results.single_core_score))
                    .bright()
                    .green(),
                style("Multi-Core").bold(),
                style(format!("{:.0}", results.multi_core_score))
                    .bright()
                    .green(),
            );
        }
        println!();
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
