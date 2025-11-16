# CI and Testing Infrastructure

This document describes the comprehensive CI/CD and testing infrastructure for Agedashi.

## Overview

Agedashi uses a multi-layered testing strategy with automated CI/CD pipelines to ensure code quality, reliability, and performance across all supported platforms.

## Test Organization

### 1. Unit Tests (`src/main.rs`)

**Location**: `#[cfg(test)]` module in `src/main.rs`

**Purpose**: Test individual functions and core logic

**Coverage**:
- DOT graph parsing
- AWS resource type extraction
- Module prefix handling (`module.vpc.aws_vpc.main`)
- Data source handling (`data.aws_ami.latest`)
- Nested module parsing
- Icon resolution and mapping
- Edge preservation

**Run with**:
```bash
cargo test --bin agedashi
```

**Key Tests**:
- `test_all_aws_resource_icons`: Validates icon extraction and conversion
- `test_parse_module_prefixed_resources`: Module resource parsing
- `test_parse_data_sources`: Data source parsing
- `test_visual_regression_complex_modules`: Visual regression testing

### 2. Integration Tests (`tests/`)

**Location**: `tests/integration_test.rs`

**Purpose**: Test the CLI as a complete system

**Coverage**:
- CLI argument parsing
- All output formats (PNG, SVG, PDF, JPG)
- Direction parameters (TB, LR)
- Custom color support
- Debug mode
- Error handling
- SVG base64 embedding
- Large graph performance

**Run with**:
```bash
cargo test --test integration_test
```

**Key Tests**:
- `test_all_output_formats`: Validates all 4 output formats
- `test_svg_base64_embedding`: SVG image embedding
- `test_large_graph_performance`: Performance under load
- `test_debug_mode`: Debug .dot file generation

### 3. Benchmarks (`benches/`)

**Location**: `benches/parsing_benchmark.rs`

**Purpose**: Track performance and detect regressions

**Coverage**:
- Graph parsing speed
- Scaling with infrastructure size
- Complex module handling

**Run with**:
```bash
cargo bench
```

## CI/CD Workflows

### Main CI Pipeline (`.github/workflows/ci.yml`)

The primary CI workflow runs on every PR and push to `develop`/`main`.

#### Jobs

##### 1. Lint
- Runs `cargo fmt --check`
- Runs `cargo clippy` with warnings as errors
- **Fast fail**: Fails quickly on style issues

##### 2. Test Matrix
Tests across:
- **Operating Systems**: Linux, macOS, Windows
- **Rust Versions**: stable, beta, MSRV (1.80.0)
- **Total**: 7 test configurations

**Matrix**:
```yaml
os: [ubuntu-latest, macos-latest, windows-latest]
rust: [stable, beta]
include:
  - os: ubuntu-latest
    rust: 1.80.0  # MSRV
```

**Tests Run**:
- All tests (`cargo test --verbose`)
- Release build verification

##### 3. Test Output Formats
- **Matrix**: 4 formats × 2 directions = 8 combinations
- Validates: PNG, SVG, PDF, JPG
- Directions: TB (top-to-bottom), LR (left-to-right)
- Uploads artifacts for visual inspection

##### 4. Test AWS Resources
- Tests all supported AWS resource types
- Validates complex module structures
- Ensures no regressions in resource mapping

##### 5. Code Coverage
- Uses `cargo-tarpaulin`
- Uploads to Codecov
- Generates HTML coverage report
- Minimum target: 80%

##### 6. Security Audit
- Runs `cargo audit`
- Checks for vulnerable dependencies
- Fails on security issues

##### 7. Benchmarks
- Runs performance benchmarks
- Tracks regression
- Only on `push` events (not PRs)

##### 8. Documentation
- Builds rustdoc
- Validates doc comments
- Uploads as artifact

##### 9. Test Summary
- Aggregates all test results
- Posts summary to GitHub
- Fails if any critical test fails

### Documentation Generation (`.github/workflows/docs.yml`)

**Triggers**: Push to `develop`/`main`, PRs, manual dispatch

**Outputs**:
1. **Rust API Documentation**
   - Full rustdoc with private items
   - Deployed to GitHub Pages (`/api`)

2. **AWS Resource Coverage Report**
   - HTML report of supported resources
   - Example diagrams for all resource types
   - Deployed to GitHub Pages

3. **Example Diagrams**
   - Sample infrastructure (TB & LR)
   - All AWS resources diagram
   - Complex modules diagram
   - Available in PNG and SVG

**Deployment**:
- **main branch**: Deploys to GitHub Pages
- **PRs**: Uploads as artifacts

### Sample Diagrams (`.github/workflows/generate-sample-diagrams.yml`)

**Purpose**: Generate and commit sample diagrams

**Triggers**: Push to `develop`/`main`, PRs

**Behavior**:
- **On PRs**: Uploads as artifacts, posts to PR comments
- **On develop/main**: Commits diagrams to repository

**Generates**:
- `samples/sample-infrastructure-tb.{png,svg,pdf}`
- `samples/sample-infrastructure-lr.{png,svg,pdf}`

### Release (`.github/workflows/release.yml`)

**Purpose**: Automated releases

**Triggers**: Push to `main` or `develop`

**Process**:
1. Determines version from conventional commits
2. Updates `Cargo.toml`
3. Builds release binary
4. Generates sample diagrams
5. Publishes to crates.io
6. Creates GitHub release
7. Uploads artifacts

**Versioning**:
- **main**: Full releases (v1.0.0)
- **develop**: Beta releases (v1.0.0-beta.123)

## Test Fixtures

### Sample Files

Located in `test/`:

1. **`sample-graph.dot`**
   - Simple Terraform graph
   - ~10 AWS resources
   - Used for basic testing

2. **`all-aws-resources.dot`**
   - Comprehensive resource coverage
   - Tests all supported AWS services
   - Validates icon mapping

3. **`complex-modules-test.dot`**
   - Real-world complex infrastructure
   - Nested modules
   - Data sources
   - 30+ resources

4. **`examples/`**
   - Real Terraform configurations
   - `simple-aws.tf`
   - `complex-aws.tf`

## Running Tests Locally

### Quick Test
```bash
cargo test
```

### Full CI Simulation
```bash
# Format check
cargo fmt --check

# Clippy
cargo clippy --all-targets --all-features -- -D warnings

# All tests
cargo test --all-targets

# Benchmarks
cargo bench

# Coverage (requires tarpaulin)
cargo tarpaulin --out html
```

### Test Specific Formats
```bash
cargo build --release

# Test PNG
cat test/sample-graph.dot | ./target/release/agedashi --output png --name test

# Test SVG
cat test/sample-graph.dot | ./target/release/agedashi --output svg --name test

# Test PDF
cat test/sample-graph.dot | ./target/release/agedashi --output pdf --name test

# Test JPG
cat test/sample-graph.dot | ./target/release/agedashi --output jpg --name test
```

## Coverage Goals

| Component | Target | Current |
|-----------|--------|---------|
| Overall | 80% | TBD |
| Parsing | 90% | TBD |
| Icon Handling | 85% | TBD |
| CLI | 75% | TBD |

## Performance Benchmarks

| Benchmark | Target | Notes |
|-----------|--------|-------|
| Small graph (10 resources) | < 500ms | Basic infrastructure |
| Medium graph (50 resources) | < 2s | Typical project |
| Large graph (200 resources) | < 10s | Enterprise scale |

## Best Practices

### For Contributors

1. **Always run tests before pushing**
   ```bash
   cargo test && cargo clippy
   ```

2. **Write tests for new features**
   - Unit test for logic
   - Integration test for CLI behavior

3. **Update fixtures when needed**
   - Add to `all-aws-resources.dot` for new services
   - Create specific fixtures for edge cases

4. **Check coverage**
   ```bash
   cargo tarpaulin --out html
   open tarpaulin-report.html
   ```

### For Reviewers

1. **Verify CI passes**
   - All matrix tests green
   - Coverage maintained or improved
   - No new clippy warnings

2. **Check test quality**
   - Tests actually test the feature
   - Edge cases covered
   - No flaky tests

3. **Performance**
   - Benchmarks don't regress
   - Large graphs still performant

## Troubleshooting

### Tests Failing Locally but Pass in CI
- Check Rust version: `rustc --version`
- Check GraphViz: `dot -V`
- Clean and rebuild: `cargo clean && cargo build`

### CI Failing on Specific Platform
- Check platform-specific code
- Test locally with Docker/VM
- Review platform-specific dependencies

### Benchmarks Show Regression
- Profile with `cargo flamegraph`
- Compare against baseline
- Check for O(n²) algorithms

### Coverage Drop
- Run `cargo tarpaulin --out html`
- Identify uncovered lines
- Add targeted tests

## Future Improvements

- [ ] Mutation testing with `cargo mutants`
- [ ] Fuzz testing for parser
- [ ] Property-based testing with `proptest`
- [ ] Cross-compilation testing
- [ ] Docker-based integration tests
- [ ] Performance regression tracking
- [ ] Visual regression testing automation
- [ ] Parallel test execution optimization

## Resources

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Criterion Benchmarking](https://github.com/bheisler/criterion.rs)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Codecov Documentation](https://docs.codecov.com/)
