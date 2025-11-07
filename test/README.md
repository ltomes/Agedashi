# Testing Agedashi

This directory contains test files for Agedashi.

## Test Files

- `sample-graph.dot` - Sample Terraform graph output in DOT format
- `expected-output.py` - The Python code that Agedashi should generate
- `test.sh` - Automated test script

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
