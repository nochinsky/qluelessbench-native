use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Helper: run benchmark with minimal config and return output JSON.
/// Uses --quick for fast CI testing (3 categories, 1 iteration).
fn run_benchmark_and_get_json(output_file: &std::path::Path) -> serde_json::Value {
    let _ = Command::new(env!("CARGO_BIN_EXE_qluelessbench"))
        .args(["--quick", "--timeout", "30", "--output"])
        .arg(output_file)
        .output()
        .expect("Failed to execute benchmark");

    let content = fs::read_to_string(output_file).expect("Failed to read results file");
    serde_json::from_str(&content).expect("Invalid JSON")
}

/// Helper: run full benchmark (all 14 categories) and return output JSON.
/// Uses longer timeout for CI since it runs all categories.
fn run_full_benchmark_and_get_json(output_file: &std::path::Path) -> serde_json::Value {
    let _ = Command::new(env!("CARGO_BIN_EXE_qluelessbench"))
        .args([
            "--iterations",
            "1",
            "--warmup",
            "0",
            "--timeout",
            "120", // 2 minutes per test to allow for slower CI hardware
            "--output",
        ])
        .arg(output_file)
        .output()
        .expect("Failed to execute benchmark");

    let content = fs::read_to_string(output_file).expect("Failed to read results file");
    serde_json::from_str(&content).expect("Invalid JSON")
}

#[test]
fn test_cli_help_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_qluelessbench"))
        .arg("--help")
        .output()
        .expect("Failed to execute benchmark");

    assert!(output.status.success(), "Help command failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("benchmark"), "Should mention benchmark");
    assert!(stdout.contains("--help"), "Should show help option");
}

#[test]
fn test_benchmark_runs_with_minimal_config() {
    let output = Command::new(env!("CARGO_BIN_EXE_qluelessbench"))
        .args(["--iterations", "1", "--warmup", "0", "--timeout", "10"])
        .output()
        .expect("Failed to execute benchmark");

    assert!(
        output.status.success(),
        "Benchmark failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Single-Core") || stdout.contains("single-core"));
}

#[test]
fn test_benchmark_creates_results_json() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = temp_dir.path().join("test_results.json");

    let output = Command::new(env!("CARGO_BIN_EXE_qluelessbench"))
        .args([
            "--iterations",
            "1",
            "--warmup",
            "0",
            "--timeout",
            "10",
            "--output",
        ])
        .arg(&output_file)
        .output()
        .expect("Failed to execute benchmark");

    assert!(output.status.success(), "Benchmark failed");
    assert!(output_file.exists(), "Results file was not created");

    let content = fs::read_to_string(&output_file).expect("Failed to read results file");

    let json: serde_json::Value =
        serde_json::from_str(&content).expect("Results file is not valid JSON");

    assert!(json.get("system_info").is_some(), "Missing system_info");
    assert!(json.get("metadata").is_some(), "Missing metadata");
    assert!(
        json.get("single_core_score").is_some(),
        "Missing single_core_score"
    );
    assert!(
        json.get("multi_core_score").is_some(),
        "Missing multi_core_score"
    );
}

#[test]
fn test_results_contain_required_fields() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = temp_dir.path().join("test_results.json");

    let _ = Command::new(env!("CARGO_BIN_EXE_qluelessbench"))
        .args([
            "--iterations",
            "1",
            "--warmup",
            "0",
            "--timeout",
            "10",
            "--output",
        ])
        .arg(&output_file)
        .output()
        .expect("Failed to execute benchmark");

    let content = fs::read_to_string(&output_file).expect("Failed to read results file");
    let json: serde_json::Value = serde_json::from_str(&content).expect("Invalid JSON");

    let system_info = json.get("system_info").expect("Missing system_info");
    assert!(system_info.get("platform").is_some());
    assert!(system_info.get("cpu_count_logical").is_some());
    assert!(system_info.get("memory_total_gb").is_some());

    let metadata = json.get("metadata").expect("Missing metadata");
    assert!(metadata.get("version").is_some());
    assert!(metadata.get("duration").is_some());
    assert!(metadata.get("completed").is_some());
}

#[test]
fn test_compare_functionality_with_previous_results() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let previous_file = temp_dir.path().join("previous.json");
    let current_file = temp_dir.path().join("current.json");

    let previous_results = r#"{
        "system_info": {
            "platform": "test",
            "platform_release": "test",
            "cpu_count_logical": 8,
            "memory_total_gb": 16.0
        },
        "overall_score": 150000.0,
        "single_core_score": 145000.0,
        "multi_core_score": 155000.0,
        "single_core_categories": [],
        "multi_core_categories": [],
        "metadata": {
            "duration": 30.0,
            "completed": "2024-01-01T00:00:00Z",
            "version": "0.1.0",
            "rust_version": "1.75.0"
        },
        "valid_test_count": 26,
        "total_test_count": 26
    }"#;

    fs::write(&previous_file, previous_results).expect("Failed to write previous results");

    let output = Command::new(env!("CARGO_BIN_EXE_qluelessbench"))
        .args([
            "--iterations",
            "1",
            "--warmup",
            "0",
            "--timeout",
            "10",
            "--output",
        ])
        .arg(&current_file)
        .arg("--compare")
        .arg(&previous_file)
        .output()
        .expect("Failed to execute benchmark");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("Single-Core") || stdout.contains("single-core"),
        "Missing Single-Core in output"
    );
}

#[test]
fn test_compare_with_invalid_file_shows_warning() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let invalid_file = temp_dir.path().join("nonexistent.json");

    let output = Command::new(env!("CARGO_BIN_EXE_qluelessbench"))
        .args([
            "--iterations",
            "1",
            "--warmup",
            "0",
            "--timeout",
            "10",
            "--compare",
        ])
        .arg(&invalid_file)
        .output()
        .expect("Failed to execute benchmark");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Warning") || stderr.contains("Could not load"),
        "Should show warning for missing compare file"
    );
}

#[test]
fn test_verbose_flag_works() {
    let output = Command::new(env!("CARGO_BIN_EXE_qluelessbench"))
        .args([
            "--iterations",
            "1",
            "--warmup",
            "0",
            "--timeout",
            "10",
            "--verbose",
        ])
        .output()
        .expect("Failed to execute benchmark");

    assert!(output.status.success());
}

#[test]
fn test_benchmark_scores_are_reasonable() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = temp_dir.path().join("test_results.json");

    let _ = Command::new(env!("CARGO_BIN_EXE_qluelessbench"))
        .args([
            "--iterations",
            "1",
            "--warmup",
            "0",
            "--timeout",
            "10",
            "--output",
        ])
        .arg(&output_file)
        .output();

    let content = fs::read_to_string(&output_file).expect("Failed to read results file");
    let json: serde_json::Value = serde_json::from_str(&content).expect("Invalid JSON");

    let single_core = json
        .get("single_core_score")
        .and_then(|v| v.as_f64())
        .expect("Missing single_core_score");
    let multi_core = json
        .get("multi_core_score")
        .and_then(|v| v.as_f64())
        .expect("Missing multi_core_score");

    assert!(
        single_core >= 0.0,
        "Single-core score should be non-negative"
    );
    assert!(multi_core >= 0.0, "Multi-core score should be non-negative");
}

#[test]
fn test_json_output_is_valid_utf8() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = temp_dir.path().join("test_results.json");

    let _ = Command::new(env!("CARGO_BIN_EXE_qluelessbench"))
        .args([
            "--iterations",
            "1",
            "--warmup",
            "0",
            "--timeout",
            "10",
            "--output",
        ])
        .arg(&output_file)
        .output();

    let content = fs::read_to_string(&output_file).expect("Results file should be valid UTF-8");
    let _: serde_json::Value =
        serde_json::from_str(&content).expect("Results should be valid JSON");
}

#[test]
fn test_results_persistence() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = temp_dir.path().join("persistence_test.json");

    let _ = Command::new(env!("CARGO_BIN_EXE_qluelessbench"))
        .args([
            "--iterations",
            "1",
            "--warmup",
            "0",
            "--timeout",
            "10",
            "--output",
        ])
        .arg(&output_file)
        .output();

    assert!(output_file.exists());

    let content1 = fs::read_to_string(&output_file).expect("First read should succeed");
    let content2 = fs::read_to_string(&output_file).expect("Second read should succeed");

    assert_eq!(content1, content2, "File content should be consistent");
}

#[test]
fn test_all_benchmark_categories_ran() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = temp_dir.path().join("categories.json");
    let json = run_full_benchmark_and_get_json(&output_file);

    let single = json
        .get("single_core_categories")
        .and_then(|v| v.as_array())
        .expect("Missing single_core_categories array");
    let multi = json
        .get("multi_core_categories")
        .and_then(|v| v.as_array())
        .expect("Missing multi_core_categories array");

    // 14 benchmark categories
    assert_eq!(single.len(), 14, "Expected 14 single-core categories");
    assert_eq!(multi.len(), 14, "Expected 14 multi-core categories");

    // Verify expected category names
    let expected = [
        "FileIO",
        "Compression",
        "ImageProcessing",
        "TextProcessing",
        "Database",
        "Mathematical",
        "Archive",
        "Memory",
        "Concurrent",
        "Cryptography",
        "RayTracing",
        "MLInference",
        "Navigation",
        "ImageFilters",
    ];
    for name in &expected {
        let found = single
            .iter()
            .any(|c| c.get("category").and_then(|v| v.as_str()) == Some(name));
        assert!(found, "Missing single-core category: {}", name);
    }
}

#[test]
fn test_scores_are_nonzero() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = temp_dir.path().join("scores.json");
    let json = run_benchmark_and_get_json(&output_file);

    let single_score = json
        .get("single_core_score")
        .and_then(|v| v.as_f64())
        .expect("Missing single_core_score");
    let multi_score = json
        .get("multi_core_score")
        .and_then(|v| v.as_f64())
        .expect("Missing multi_core_score");
    let overall = json
        .get("overall_score")
        .and_then(|v| v.as_f64())
        .expect("Missing overall_score");

    assert!(single_score > 0.0, "Single-core score should be > 0");
    assert!(multi_score > 0.0, "Multi-core score should be > 0");
    assert!(overall > 0.0, "Overall score should be > 0");
}

#[test]
fn test_system_info_complete() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = temp_dir.path().join("sysinfo.json");
    let json = run_benchmark_and_get_json(&output_file);

    let sys = json.get("system_info").expect("Missing system_info");
    assert!(
        sys.get("platform")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .len()
            > 0
    );
    assert!(
        sys.get("cpu_count_logical")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            > 0
    );
    assert!(
        sys.get("memory_total_gb")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
            > 0.0
    );
}

#[test]
fn test_category_tests_nonempty() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = temp_dir.path().join("cat_tests.json");
    let json = run_benchmark_and_get_json(&output_file);

    let single = json
        .get("single_core_categories")
        .and_then(|v| v.as_array())
        .expect("Missing single_core_categories");

    // At least most categories should have tests
    let categories_with_tests: usize = single
        .iter()
        .filter(|c| {
            c.get("tests")
                .and_then(|t| t.as_array())
                .map(|a| !a.is_empty())
                .unwrap_or(false)
        })
        .count();

    assert!(
        categories_with_tests >= 12,
        "Expected at least 12 categories with tests, got {}",
        categories_with_tests
    );
}

#[test]
fn test_metadata_version_matches() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = temp_dir.path().join("version.json");
    let json = run_benchmark_and_get_json(&output_file);

    let version = json
        .get("metadata")
        .and_then(|m| m.get("version"))
        .and_then(|v| v.as_str())
        .expect("Missing metadata.version");

    assert_eq!(version, env!("CARGO_PKG_VERSION"));
}
