# Contributing to Agedashi

Thank you for your interest in contributing to Agedashi! This document provides guidelines and information for contributors.

## Development Setup

### Prerequisites

1. **Rust** (latest stable version)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **GraphViz** (required for testing)
   ```bash
   # macOS
   brew install graphviz

   # Ubuntu/Debian
   sudo apt-get install graphviz

   # Fedora
   sudo dnf install graphviz
   ```

### Building

```bash
git clone https://github.com/ltomes/Agedashi.git
cd Agedashi
cargo build --release
```

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
```

### 2. Make Changes

Edit the code following our [coding standards](#coding-standards).

### 3. Run Tests

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --lib          # Unit tests
cargo test --test '*'     # Integration tests

# Run with output
cargo test -- --nocapture
```

### 4. Format and Lint

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings
```

### 5. Commit Changes

We use [Conventional Commits](https://www.conventionalcommits.org/):

```bash
# Format: <type>(<scope>): <description>

# Examples:
git commit -m "feat: add support for GCP resources"
git commit -m "fix: resolve icon loading issue on Windows"
git commit -m "docs: update installation instructions"
git commit -m "test: add integration test for SVG generation"
git commit -m "perf: optimize graph parsing for large files"
git commit -m "refactor: extract icon loading into separate module"
```

**Commit types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Adding or updating tests
- `perf`: Performance improvements
- `refactor`: Code refactoring
- `chore`: Maintenance tasks
- `ci`: CI/CD changes

### 6. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub.

## Testing Guidelines

### Writing Tests

1. **Unit Tests**: Add to `src/main.rs` in the `#[cfg(test)]` module
   - Test individual functions
   - Test parsing logic
   - Test resource mapping

2. **Integration Tests**: Add to `tests/integration_test.rs`
   - Test CLI behavior
   - Test file I/O
   - Test error handling

3. **Benchmarks**: Add to `benches/parsing_benchmark.rs`
   - Test performance-critical paths
   - Measure scaling behavior

### Test Coverage

Aim for:
- >80% code coverage for new features
- 100% coverage for critical parsing logic
- All public APIs should have tests

### Running Tests Locally

```bash
# All tests
cargo test

# With coverage (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out html --output-dir coverage

# Benchmarks
cargo bench
```

## Coding Standards

### Code Style

- Follow Rust standard style (enforced by `rustfmt`)
- Maximum line length: 100 characters
- Use meaningful variable names
- Add comments for complex logic

### Documentation

- Add doc comments (`///`) for public functions
- Include examples in doc comments
- Update README.md for user-facing changes
- Update CHANGELOG.md

Example:
```rust
/// Parses a Terraform DOT graph into our internal representation.
///
/// # Arguments
///
/// * `dot_content` - The DOT format graph as a string
///
/// # Returns
///
/// A `Result` containing the parsed `TerraformGraph` or an error.
///
/// # Examples
///
/// ```
/// let dot = r#"digraph { ... }"#;
/// let graph = parse_dot_graph(dot)?;
/// ```
pub fn parse_dot_graph(dot_content: &str) -> Result<TerraformGraph> {
    // ...
}
```

### Error Handling

- Use `anyhow::Result` for functions that can fail
- Provide context with `.context()`
- Use descriptive error messages
- Don't unwrap() in production code (tests are OK)

```rust
// Good
fs::read_to_string(path)
    .context(format!("Failed to read file: {}", path.display()))?

// Bad
fs::read_to_string(path).unwrap()
```

### Performance

- Avoid unnecessary allocations
- Use iterators instead of collecting when possible
- Profile before optimizing
- Add benchmarks for performance-critical code

## Adding New AWS Resources

To add support for a new AWS resource:

1. **Add icon mapping** in `get_icon_search_pattern()`:
   ```rust
   "your_service" => Some("AWS-Service-Name_64"),
   ```

2. **Add service info** in `get_service_info()`:
   ```rust
   t if t.contains("aws_your_service") => ("your_service", "ServiceName", "#COLOR", "📦"),
   ```

3. **Add to icon extraction** in `download_icons_as_needed()`:
   ```rust
   let icon_names = vec![
       // ... existing icons ...
       "your_service",
   ];
   ```

4. **Add test case** in the unit tests:
   ```rust
   #[test]
   fn test_your_service_parsing() {
       // Test parsing logic
   }
   ```

5. **Update documentation**:
   - Add to README.md supported resources list
   - Update docs/coverage report

## CI/CD Pipeline

Our CI runs automatically on:
- All pull requests
- Pushes to `develop` and `main`

### CI Checks

1. **Lint**: Format and clippy checks
2. **Test Matrix**: Tests on Linux, macOS, Windows with multiple Rust versions
3. **Format Tests**: All output formats (PNG, SVG, PDF, JPG)
4. **Resource Tests**: All AWS resource types
5. **Coverage**: Code coverage reporting
6. **Security**: Dependency audit
7. **Benchmarks**: Performance regression detection
8. **Documentation**: API docs and examples generation

### Making CI Pass

Before submitting a PR, ensure locally:

```bash
# Format
cargo fmt

# Clippy (no warnings)
cargo clippy --all-targets --all-features -- -D warnings

# All tests pass
cargo test

# Build succeeds
cargo build --release
```

## Project Structure

```
Agedashi/
├── .github/
│   └── workflows/        # CI/CD workflows
│       ├── ci.yml        # Main CI pipeline
│       ├── docs.yml      # Documentation generation
│       ├── generate-sample-diagrams.yml
│       └── release.yml
├── benches/              # Benchmark tests
│   └── parsing_benchmark.rs
├── icons/                # Bundled AWS icons
│   └── Asset-Package.7z
├── samples/              # Example diagrams
├── scripts/              # Build and utility scripts
├── src/
│   └── main.rs          # Main application code
├── test/                 # Test fixtures and data
│   ├── sample-graph.dot
│   ├── all-aws-resources.dot
│   ├── complex-modules-test.dot
│   └── examples/        # Terraform examples
├── tests/                # Integration tests
│   ├── integration_test.rs
│   └── README.md
├── build.rs             # Build script
├── Cargo.toml
├── README.md
└── CONTRIBUTING.md      # This file
```

## Getting Help

- **Issues**: Check existing issues or create a new one
- **Discussions**: Use GitHub Discussions for questions
- **Documentation**: See README.md and inline docs

## Code Review Process

1. All PRs require at least one approval
2. CI must pass (all checks green)
3. Code should follow style guidelines
4. Tests should be included for new features
5. Documentation should be updated

## Release Process

Releases are automated via CI when pushed to `main` or `develop`:

- **main**: Full releases (v1.0.0)
- **develop**: Beta releases (v1.0.0-beta.123)

Version bumps follow [Semantic Versioning](https://semver.org/):
- `MAJOR`: Breaking changes
- `MINOR`: New features (backward compatible)
- `PATCH`: Bug fixes

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Questions?

Feel free to open an issue for any questions or concerns!
