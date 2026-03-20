use std::fs;
use std::process::Command;
use tempfile::TempDir;

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
