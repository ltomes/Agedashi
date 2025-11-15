# Agedashi Test Suite

This directory contains integration tests for the Agedashi CLI tool.

## Test Organization

The test suite is organized into several layers:

### Unit Tests (`src/main.rs`)
- Located in the `#[cfg(test)]` module in `src/main.rs`
- Test individual functions and parsing logic
- Test AWS resource type extraction and mapping
- Test module and data source handling
- Run with: `cargo test --lib`

### Integration Tests (`tests/`)
- Test the CLI as a whole, including subprocess execution
- Test all output formats (PNG, SVG, PDF, JPG)
- Test direction parameters (TB, LR)
- Test error handling and edge cases
- Run with: `cargo test --test '*'`

### Benchmarks (`benches/`)
- Performance benchmarks for parsing and generation
- Measure scaling with graph size
- Run with: `cargo bench`

## Running Tests

### Run all tests
```bash
cargo test
```

### Run only unit tests
```bash
cargo test --lib
```

### Run only integration tests
```bash
cargo test --test integration_test
```

### Run specific test
```bash
cargo test test_sample_graph_png_generation
```

### Run with output
```bash
cargo test -- --nocapture
```

### Run benchmarks
```bash
cargo bench
```

## Prerequisites

Integration tests require:
- **GraphViz** installed and `dot` command available
- **Rust** toolchain
- Test fixtures in `test/` directory

## Test Coverage

The test suite covers:

✅ **CLI Interface**
- Help and version commands
- Argument parsing
- Error handling

✅ **Input Processing**
- Empty input handling
- Sample graph parsing
- Complex module structures
- Data source handling

✅ **Output Generation**
- PNG format
- SVG format (with base64 embedding)
- PDF format
- JPG format

✅ **Configuration**
- Direction parameters (TB, LR)
- Custom colors
- Custom output names
- Debug mode

✅ **Performance**
- Large graph processing
- Scaling with infrastructure size

✅ **Edge Cases**
- Module-prefixed resources (`module.vpc.aws_vpc.main`)
- Data sources (`data.aws_ami.latest`)
- Nested modules
- Duplicate resource names
- Missing icons (fallback to colored boxes)

## Writing New Tests

### Integration Test Example

```rust
#[test]
fn test_my_feature() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let sample_graph = fs::read_to_string("test/sample-graph.dot")
        .expect("Failed to read sample-graph.dot");

    std::env::set_current_dir(&temp_dir).expect("Failed to change directory");

    let output = run_agedashi(
        &sample_graph,
        &["--output", "png", "--name", "test-output"]
    );

    assert!(output.status.success());
    assert!(temp_dir.path().join("test-output.png").exists());
}
```

### Unit Test Example

Add to `src/main.rs` in the `#[cfg(test)] mod tests` section:

```rust
#[test]
fn test_my_parsing_logic() {
    let input = "...";
    let result = parse_dot_graph(input).expect("Parse failed");
    assert_eq!(result.resources.len(), 5);
}
```

## Continuous Integration

All tests run automatically in CI via GitHub Actions:

- **Lint Job**: Formatting and clippy checks
- **Test Matrix**: Tests across Linux, macOS, Windows and multiple Rust versions
- **Format Tests**: Tests all output formats (PNG, SVG, PDF, JPG)
- **Coverage**: Code coverage with tarpaulin
- **Benchmarks**: Performance regression detection

See `.github/workflows/ci.yml` for details.

## Test Artifacts

Integration tests create temporary files that are automatically cleaned up. In CI, some artifacts are uploaded:

- Generated diagrams (for visual inspection)
- Coverage reports
- Benchmark results

## Debugging Failed Tests

### View test output
```bash
cargo test -- --nocapture
```

### Run specific failing test
```bash
cargo test test_name -- --nocapture
```

### Check GraphViz installation
```bash
dot -V
```

### Verify test fixtures exist
```bash
ls -la test/*.dot
```

### Run in debug mode
```bash
AGEDASHI_DEBUG=1 cargo test test_name -- --nocapture
```

This will save intermediate `.dot` files for inspection.

## Contributing

When adding new features:

1. Add unit tests for parsing/mapping logic
2. Add integration tests for CLI behavior
3. Update benchmarks if performance-critical
4. Ensure all tests pass: `cargo test`
5. Check formatting: `cargo fmt --check`
6. Run clippy: `cargo clippy`

## Performance Guidelines

Integration tests should:
- Complete in < 5 seconds each
- Use temporary directories (auto-cleanup)
- Not depend on external network
- Be deterministic (no flaky tests)

Benchmarks should:
- Use realistic workload sizes
- Measure only the relevant code path
- Report consistent results
