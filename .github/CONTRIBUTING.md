# Contributing to QlueLessBench Native

Thank you for your interest in contributing!

## How to Contribute

### Reporting Bugs

- Search existing issues before opening a new one
- Use a clear, descriptive title
- Include your OS, Rust version, and hardware info if relevant
- Provide steps to reproduce the issue

### Suggesting Features

- Open an issue with the label `enhancement`
- Describe the use case and why it would be useful
- Keep scope focused — one feature per issue

### Pull Requests

1. **Fork** the repository
2. **Clone** your fork: `git clone https://github.com/<you>/qluelessbench-native.git`
3. **Create a branch**: `git checkout -b feature/my-feature`
4. **Make your changes** — ensure tests pass: `cargo test`
5. **Run clippy**: `cargo clippy -- -D warnings` and `cargo fmt`
6. **Commit** with a clear message: `git commit -m "Add feature X"`
7. **Push**: `git push origin feature/my-feature`
8. Open a **Pull Request** against `main`

### Code Style

- Run `cargo fmt` before committing
- All clippy warnings must be resolved (`cargo clippy -- -D warnings`)
- New features should include unit tests
- Keep functions focused and well-documented

### Running Tests

```bash
# Run all tests
cargo test

# Run with clippy
cargo clippy -- -D warnings

# Check formatting
cargo fmt -- --check
```
