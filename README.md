# QlueLessBench Native

A comprehensive cross-platform system benchmark tool written in Rust, inspired by Geekbench 6.

## Goal

**QlueLessBench Native** is designed to be a **Geekbench-like CPU benchmark** that measures both single-core and multi-core performance on Windows, Linux, and macOS systems. Unlike synthetic benchmarks that use theoretical workloads, QlueLessBench tests real-world tasks:

- **File I/O** - Storage read/write performance
- **Compression** - ZIP/GZIP compression speed
- **Image Processing** - Resize, blur, sharpen operations
- **Text Processing** - Search, regex, string manipulation
- **Database** - SQLite CRUD operations
- **Mathematical** - Matrix operations, statistics, prime generation
- **Cryptography** - AES-256 encryption, SHA256 hashing
- **Ray Tracing** - 3D path tracing rendering
- **ML Inference** - Neural network forward pass
- **Navigation** - GPS route calculation (Dijkstra's algorithm)
- **Image Filters** - Edge detection, Gaussian blur

## Features

- **Single benchmark mode** - Just run it, no configuration needed
- **Single-core + Multi-core scores** - Properly differentiated performance metrics
- **Cross-platform** - Windows, Linux, and macOS support
- **No admin privileges required** - Runs entirely in user space
- **Clean CLI output** - Simple, readable results
- **~1-2 minute run time** - Comprehensive but not excessive (varies by hardware)
- **Automatic cleanup** - All test files are automatically removed

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/nochinsky/qluelessbench-native.git
cd qluelessbench-native

# Build in release mode
cargo build --release

# The binary will be at target/release/qluelessbench
```

## Usage

### Run Benchmark

```bash
./target/release/qluelessbench
```

### Verbose Output

```bash
./target/release/qluelessbench --verbose
```

### Custom Settings

```bash
./target/release/qluelessbench --iterations 10 --warmup 3 --timeout 60
```

### Save Results

```bash
./target/release/qluelessbench --output my_results.json
```

### Compare Against Previous Run

```bash
./target/release/qluelessbench --compare previous_results.json
```

### All Options

```
QlueLessBench Native - Cross-platform system benchmark tool

Usage: qluelessbench [OPTIONS]

Options:
  -n, --iterations <ITERATIONS>  Number of iterations per test (default: 5)
      --warmup <WARMUP>          Number of warmup iterations (default: 2)
  -t, --timeout <TIMEOUT>        Timeout per test in seconds (default: 30)
  -o, --output <OUTPUT>          Output file path for results JSON
      --compare <COMPARE>        Compare results against a previous run
  -v, --verbose                 Enable verbose logging output
  -h, --help                    Print help
  -V, --version                 Print version
```

## Output Example

```
╔════════════════════════════════════════════════╗
║   QlueLessBench Native v0.1.0                  ║
╚════════════════════════════════════════════════╝

═══ Single-Core Tests ═══
[1/13] Running FileIO tests...
    ✓ FileIO: 11553 (2.89s)

[2/13] Running Compression tests...
    ✓ Compression: 3854 (3.89s)

...

═══ Multi-Core Tests ═══
[1/13] Running FileIO tests...
    ✓ FileIO: 31341 (38.76s)

...

════════════════════════════════════════════════
              QlueLessBench Native Results
════════════════════════════════════════════════

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Single-Core                154463
  Multi-Core                 174398
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

System Information:
────────────────────────────────────────
  Platform:     windows 10.0.22631
  CPU:          13th Gen Intel(R) Core(TM) i7-13700K
  CPU Cores:    24
  Memory:       63.8 GB
```

## Scoring

- **Higher scores are better**
- Each test category produces a score based on performance relative to a reference baseline
- **Single-Core Score**: Weighted average of all single-threaded test results
- **Multi-Core Score**: Weighted average of all multi-threaded (parallelized) test results
- Scores can exceed 1000 — a score of 2000 means the machine is 2x faster than the reference

### What the Scores Mean

- **~1,000 points** = Reference/baseline CPU performance
- **~2,000 points** = 2x faster than baseline
- **~5,000+ points** = High-performance modern CPU

The multi-core score reflects how well your CPU can parallelize workloads. On a 24-core CPU like the i7-13700K, expect multi-core to be 10-50% higher than single-core for mixed workloads.

## Test Categories

| Category | Tests | Description |
|----------|-------|-------------|
| **File I/O** | Sequential R/W, Random Access, Copy, Delete | Storage and filesystem performance |
| **Compression** | ZIP (levels 1-9), GZIP | CPU compression algorithm speed |
| **Image Processing** | Resize, Blur, Sharpen, Format Conversion | Image manipulation workloads |
| **Text Processing** | Search/Replace, Regex, String Ops | Text manipulation performance |
| **Database** | SQLite CRUD, Indexed Queries | Database operation speed |
| **Mathematical** | Array Ops, Matrix Mult, Statistics, Primes | Numerical computation |
| **Archive** | ZIP/TAR Create/Extract | Archive creation and extraction |
| **Memory** | Alloc/Dealloc, Vec/HashMap Ops | Memory allocation and data structures |
| **Cryptography** | AES-256, SHA256 | Encryption and hashing speed |
| **Ray Tracing** | Path Tracer | 3D rendering performance |
| **ML Inference** | Neural Network Forward Pass | AI/machine learning workloads |
| **Navigation** | Dijkstra Pathfinding | GPS/route calculation |
| **Image Filters** | Blur, Edge Detect, Sharpen | Image filter operations |

## Building from Source

### Requirements

- Rust 1.75 or later
- A C compiler (for some dependencies)

### Build Commands

```bash
# Debug build (faster compilation, slower execution)
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test
```

## Comparison with Geekbench 6

| Feature | Geekbench 6 | QlueLessBench Native |
|---------|-------------|---------------------|
| Single-Core Score | ✓ | ✓ |
| Multi-Core Score | ✓ | ✓ |
| Cross-Platform | ✓ | ✓ |
| Image Processing | ✓ | ✓ |
| ML/AI Workloads | ✓ | ✓ |
| Ray Tracing | ✓ | ✓ |
| Compression | ✓ | ✓ |
| Database | ✗ | ✓ (SQLite) |
| Navigation/GPS | ✗ | ✓ (Dijkstra) |
| Cryptography | ✓ (AES, SHA256) | ✓ (AES, SHA256) |
| Calibration | Official baseline | Community-driven |
| Cost | Free (limited) / Paid | Free & Open Source |

## Current Status

**Version: 0.1.0** (Active Development)

### Completed
- ✅ 13 comprehensive benchmark categories
- ✅ Single-core and multi-core scoring
- ✅ Cross-platform support (Windows, Linux)
- ✅ Clean CLI interface
- ✅ Statistical analysis (median, percentiles, CV)
- ✅ Automatic test file cleanup

### Future Enhancements
- PDF rendering benchmark
- HTML5/JavaScript benchmark (requires additional dependencies)
- Code compilation benchmark
- Online results browser for score comparison
- Calibration against reference systems

## Contributing

Contributions are welcome! Areas of interest:
- Additional benchmark workloads
- Performance optimizations
- Better reference value calibration
- Documentation improvements

## License

MIT License - See [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by **Geekbench 6** from Primate Labs
- Original Python version: QlueLessBench V3

## Version

**QlueLessBench Native v0.1.0**

---

*Built with ❤️ using Rust*
