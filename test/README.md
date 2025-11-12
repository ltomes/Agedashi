# Testing Agedashi

This directory contains test files for Agedashi.

## Test Files

- `sample-graph.dot` - Simple sample Terraform graph output in DOT format
- `all-aws-resources.dot` - Comprehensive test covering all supported AWS resource types
- `complex-modules-test.dot` - Complex real-world scenario with modules, data sources, and nested modules
- `expected-output.py` - The Python code that Agedashi should generate
- `test.sh` - Automated test script
- `output/` - Generated visual regression test artifacts (gitignored)

## Prerequisites for Testing

Before testing, ensure you have:

1. **Rust and Cargo** installed
2. **Python 3.9+** installed
3. **Graphviz** installed:
   ```bash
   # macOS
   brew install graphviz

   # Ubuntu/Debian
   sudo apt-get install graphviz

   # Fedora
   sudo dnf install graphviz
   ```
4. **Python diagrams library**:
   ```bash
   pip install diagrams
   ```

## Building Agedashi

```bash
cd /home/user/Agedashi
cargo build --release
```

The binary will be at `target/release/agedashi`

## Running Tests

### Test 1: Using Sample DOT File

Test Agedashi with the pre-generated terraform graph output:

```bash
cat test/sample-graph.dot | ./target/release/agedashi --output png --name test-output
```

Expected result: A `test-output.png` file should be created showing:
- VPC
- Public Subnet
- Security Group
- EC2 Instance
- Load Balancer
- RDS Database
- S3 Bucket
- Relationships between them

### Test 2: Using Real Terraform

If you have Terraform installed:

```bash
cd examples
terraform init
terraform graph | ../target/release/agedashi --output svg --name simple-aws
```

Expected result: A `simple-aws.svg` file with a diagram of the infrastructure.

### Test 3: Different Output Formats

Test all supported output formats:

```bash
cat test/sample-graph.dot | ./target/release/agedashi --output svg --name test-svg
cat test/sample-graph.dot | ./target/release/agedashi --output png --name test-png
cat test/sample-graph.dot | ./target/release/agedashi --output pdf --name test-pdf
cat test/sample-graph.dot | ./target/release/agedashi --output jpg --name test-jpg
```

### Test 4: Different Directions

Test layout directions:

```bash
# Top to Bottom (default)
cat test/sample-graph.dot | ./target/release/agedashi --direction TB --name test-tb

# Left to Right
cat test/sample-graph.dot | ./target/release/agedashi --direction LR --name test-lr
```

### Test 5: Verify Python Code Generation

To see the generated Python code, check the stderr output:

```bash
cat test/sample-graph.dot | ./target/release/agedashi 2>&1 | grep -A 50 "Generated Python code:"
```

Compare this with `expected-output.py` to verify correctness.

### Test 6: Test Python Code Directly

You can test the expected output directly:

```bash
python3 test/expected-output.py
```

This should create an `infrastructure.png` file.

## Automated Test Script

Run all tests automatically:

```bash
cd /home/user/Agedashi
bash test/test.sh
```

## What to Verify

When testing, verify:

1. ✅ Binary builds successfully
2. ✅ Can read from stdin
3. ✅ Correctly parses DOT format
4. ✅ Maps AWS resources to correct diagram icons
5. ✅ Generates valid Python code
6. ✅ Python code executes without errors
7. ✅ Output files are created in the correct format
8. ✅ Diagrams show correct resources and relationships
9. ✅ All output formats work (svg, png, pdf, jpg)
10. ✅ Direction parameter works (TB, LR)

## Expected Output

The generated diagrams should show:
- AWS service icons (not generic boxes)
- Correct resource labels
- Arrows showing dependencies/relationships
- Professional cloud architecture diagram appearance

## Troubleshooting

### Error: "Failed to execute Python code"

**Solution**: Install diagrams library
```bash
pip install diagrams
```

### Error: Python code fails with "No module named 'graphviz'"

**Solution**: Install Graphviz
```bash
# macOS
brew install graphviz

# Linux
sudo apt-get install graphviz
```

### Error: "No AWS resources found"

**Solution**: Check that your DOT file contains AWS resource nodes. The parser looks for lines like:
```
"[root] aws_instance.web (expand)" [label = "aws_instance.web", shape = "box"]
```

### Build Error: "failed to get dependency from crates.io"

**Solution**: Check internet connection and try again. Cargo needs to download dependencies.

## Manual Verification

To manually verify the tool works correctly:

1. Open the generated diagram file
2. Check that it contains AWS service icons (not generic boxes)
3. Verify all resources from your Terraform are present
4. Confirm relationships/arrows make sense
5. Compare with your actual infrastructure

## Performance Testing

For large infrastructures:

```bash
# Time the execution
time cat large-graph.dot | ./target/release/agedashi

# Check memory usage
/usr/bin/time -v cat large-graph.dot | ./target/release/agedashi
```

## Integration Testing

Test with real Terraform projects:

```bash
cd /path/to/your/terraform/project
terraform graph | agedashi --output svg --name production-infra
```

Then manually verify the diagram matches your infrastructure.

## Unit Tests

Agedashi includes comprehensive unit tests for core functionality:

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_parse_module_prefixed_resources

# Run with output visible
cargo test -- --nocapture
```

### Test Coverage

The test suite includes:

1. **Module Resource Parsing** - Validates that `module.vpc.aws_vpc.main` is correctly extracted as `aws_vpc`
2. **Data Source Parsing** - Validates that `data.aws_ami.latest` is correctly extracted as `aws_ami`
3. **Nested Module Parsing** - Handles deeply nested modules like `module.app.module.network.aws_vpc.main`
4. **Resource Filtering** - Ensures module and data resources are NOT filtered out
5. **Edge Preservation** - Verifies connections between resources are maintained
6. **Mixed Scenarios** - Tests combinations of modules, data sources, and regular resources
7. **Icon Coverage** - Documents which AWS resources have icon support
8. **Visual Regression** - Generates a complex diagram for manual inspection

## Visual Regression Testing

The `test_visual_regression_complex_modules` test generates a comprehensive diagram that developers can visually inspect:

```bash
# Run the visual regression test
cargo test test_visual_regression_complex_modules -- --nocapture

# Output is generated in test/output/
ls -la test/output/
```

### What to Look For

When reviewing the visual regression output:

✅ **All module resources are visible** - Resources like `module.networking.aws_vpc.main` should appear in the diagram

✅ **Data sources included** - Data sources like `data.aws_ami.amazon_linux_2` should be shown

✅ **Nested modules work** - Deeply nested like `module.app.module.asg.aws_autoscaling_group.web` should render

✅ **Edges connect properly** - Arrows should connect related resources

✅ **Icons or styled boxes** - Either AWS icons (if available) or colored fallback boxes

### Expected Metrics

The complex modules test should show:
- **30+ total resources** (if fewer, resources are being filtered incorrectly)
- **22+ module resources** (validates module prefix handling)
- **4+ data sources** (validates data source handling)
- **30+ edges** (validates relationship preservation)

If these metrics are significantly lower, it indicates a regression in the module/data source handling logic.

### Visual Test Files Generated

After running the visual regression test, check:
- `test/output/complex-modules-test.dot` - Generated DOT file (human-readable)
- `test/output/complex-modules-test-visual-regression.png` - PNG diagram (requires GraphViz)
- `test/output/complex-modules-test-visual-regression.svg` - SVG diagram (requires GraphViz)

> **Note**: These files are gitignored and can be regenerated anytime by running the test.
