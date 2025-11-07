#!/bin/bash

# Terrok Test Suite
# This script tests the Terrok CLI tool

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

echo "======================================"
echo "  Terrok Test Suite"
echo "======================================"
echo ""

# Function to print test results
print_result() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓ PASS${NC}: $2"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}✗ FAIL${NC}: $2"
        ((TESTS_FAILED++))
    fi
    ((TESTS_RUN++))
}

# Check prerequisites
echo "Checking prerequisites..."

# Check if binary exists
if [ ! -f "target/release/terrok" ]; then
    echo -e "${RED}Error: Binary not found. Please build first with 'cargo build --release'${NC}"
    exit 1
fi

# Check Python
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}Error: Python 3 not found${NC}"
    exit 1
fi
echo -e "${GREEN}✓${NC} Python 3 found"

# Check diagrams library
if python3 -c "import diagrams" 2>/dev/null; then
    echo -e "${GREEN}✓${NC} diagrams library installed"
    DIAGRAMS_INSTALLED=1
else
    echo -e "${YELLOW}⚠${NC} diagrams library not installed (some tests will be skipped)"
    DIAGRAMS_INSTALLED=0
fi

# Check graphviz
if command -v dot &> /dev/null; then
    echo -e "${GREEN}✓${NC} Graphviz installed"
    GRAPHVIZ_INSTALLED=1
else
    echo -e "${YELLOW}⚠${NC} Graphviz not installed (diagram generation will fail)"
    GRAPHVIZ_INSTALLED=0
fi

echo ""
echo "Running tests..."
echo ""

# Test 1: Help command
echo "Test 1: Help command"
if ./target/release/terrok --help > /dev/null 2>&1; then
    print_result 0 "Help command works"
else
    print_result 1 "Help command failed"
fi

# Test 2: Version command
echo "Test 2: Version command"
if ./target/release/terrok --version > /dev/null 2>&1; then
    print_result 0 "Version command works"
else
    print_result 1 "Version command failed"
fi

# Test 3: Read from stdin with sample DOT file
echo "Test 3: Parse sample DOT file"
if cat test/sample-graph.dot | ./target/release/terrok --output png --name test-basic 2>&1 | grep -q "Generated Python code:"; then
    print_result 0 "Successfully parsed DOT file and generated Python code"
else
    print_result 1 "Failed to parse DOT file"
fi

# Test 4: Test with empty input
echo "Test 4: Handle empty input"
if echo "" | ./target/release/terrok 2>&1 | grep -q "No input provided"; then
    print_result 0 "Correctly handles empty input"
else
    print_result 1 "Failed to handle empty input"
fi

# Test 5-8: Test different output formats (only if diagrams and graphviz are installed)
if [ $DIAGRAMS_INSTALLED -eq 1 ] && [ $GRAPHVIZ_INSTALLED -eq 1 ]; then
    echo "Test 5: Generate PNG output"
    if cat test/sample-graph.dot | ./target/release/terrok --output png --name test-png 2>/dev/null && [ -f "test-png.png" ]; then
        print_result 0 "PNG generation successful"
        rm -f test-png.png
    else
        print_result 1 "PNG generation failed"
    fi

    echo "Test 6: Generate SVG output"
    if cat test/sample-graph.dot | ./target/release/terrok --output svg --name test-svg 2>/dev/null && [ -f "test-svg.svg" ]; then
        print_result 0 "SVG generation successful"
        rm -f test-svg.svg
    else
        print_result 1 "SVG generation failed"
    fi

    echo "Test 7: Generate PDF output"
    if cat test/sample-graph.dot | ./target/release/terrok --output pdf --name test-pdf 2>/dev/null && [ -f "test-pdf.pdf" ]; then
        print_result 0 "PDF generation successful"
        rm -f test-pdf.pdf
    else
        print_result 1 "PDF generation failed"
    fi

    echo "Test 8: Generate JPG output"
    if cat test/sample-graph.dot | ./target/release/terrok --output jpg --name test-jpg 2>/dev/null && [ -f "test-jpg.jpg" ]; then
        print_result 0 "JPG generation successful"
        rm -f test-jpg.jpg
    else
        print_result 1 "JPG generation failed"
    fi
else
    echo -e "${YELLOW}⚠ Skipping output format tests (diagrams or graphviz not installed)${NC}"
fi

# Test 9: Test direction parameter
echo "Test 9: Test direction parameter (TB)"
if cat test/sample-graph.dot | ./target/release/terrok --direction TB --name test-tb 2>&1 | grep -q "direction=\"TB\""; then
    print_result 0 "TB direction parameter works"
else
    print_result 1 "TB direction parameter failed"
fi

# Test 10: Test direction parameter (LR)
echo "Test 10: Test direction parameter (LR)"
if cat test/sample-graph.dot | ./target/release/terrok --direction LR --name test-lr 2>&1 | grep -q "direction=\"LR\""; then
    print_result 0 "LR direction parameter works"
else
    print_result 1 "LR direction parameter failed"
fi

# Test 11: Verify AWS resource detection
echo "Test 11: AWS resource detection"
OUTPUT=$(cat test/sample-graph.dot | ./target/release/terrok 2>&1)
if echo "$OUTPUT" | grep -q "aws.compute" && echo "$OUTPUT" | grep -q "aws.database"; then
    print_result 0 "AWS resources correctly detected and mapped"
else
    print_result 1 "AWS resource mapping failed"
fi

# Test 12: Test expected Python output
if [ $DIAGRAMS_INSTALLED -eq 1 ] && [ $GRAPHVIZ_INSTALLED -eq 1 ]; then
    echo "Test 12: Test expected Python code directly"
    if python3 test/expected-output.py 2>/dev/null && [ -f "infrastructure.png" ]; then
        print_result 0 "Expected Python code generates diagram"
        rm -f infrastructure.png
    else
        print_result 1 "Expected Python code failed"
    fi
else
    echo -e "${YELLOW}⚠ Skipping Python code test (diagrams or graphviz not installed)${NC}"
fi

# Clean up any generated files
rm -f test-*.png test-*.svg test-*.pdf test-*.jpg

echo ""
echo "======================================"
echo "  Test Results"
echo "======================================"
echo "Tests run: $TESTS_RUN"
echo -e "Tests passed: ${GREEN}$TESTS_PASSED${NC}"
if [ $TESTS_FAILED -gt 0 ]; then
    echo -e "Tests failed: ${RED}$TESTS_FAILED${NC}"
else
    echo -e "Tests failed: $TESTS_FAILED"
fi
echo "======================================"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi
