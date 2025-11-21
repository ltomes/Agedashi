#!/bin/bash
# Test script to find optimal spacing parameters
# Tests various nodesep/ranksep combinations to prevent overlaps

set -e

INPUT_FILE="test/sample-graph.dot"

echo "Testing spacing parameters for overlap prevention..."
echo ""

# Test configuration: mode, nodesep, ranksep, sep, esep, margin
# We'll modify main.rs temporarily to test these values

# For now, let's generate samples with current values and document observations
echo "Current spacing values (from code):"
echo "  curved:   nodesep=2.0, ranksep=2.5, sep=+25,25, esep=+20,20, margin=0.25"
echo "  ortho:    nodesep=2.5, ranksep=3.0, sep=+30,30, esep=+25,25, margin=0.35"
echo "  trace:    nodesep=3.5, ranksep=4.0, sep=+40,40, esep=+35,35, margin=0.45"
echo "  polyline: nodesep=2.0, ranksep=2.5, sep=+25,25, esep=+20,20, margin=0.25"
echo ""

# Generate samples for visual inspection
MODES=("curved" "ortho" "trace" "polyline")
DIRECTIONS=("TB" "LR")

mkdir -p spacing-tests

for mode in "${MODES[@]}"; do
    for direction in "${DIRECTIONS[@]}"; do
        output="spacing-tests/test-${mode}-${direction}"
        echo "Generating: ${mode}-${direction}"
        cat "$INPUT_FILE" | ./target/release/agedashi \
            --output png \
            --name "$output" \
            --direction "$direction" \
            --edge-routing "$mode" \
            2>&1 | grep -E "(Diagram generated|Warning|Error)" || true
    done
done

echo ""
echo "✅ Test samples generated in spacing-tests/"
echo ""
echo "Visual inspection needed for:"
echo "  [ ] Lines passing through resource icons"
echo "  [ ] Lines passing through resource labels"
echo "  [ ] Overlapping line endpoints"
echo "  [ ] Adequate spacing between lines"
echo "  [ ] Overall professional appearance"
echo ""
echo "If issues found, increase parameters in src/main.rs and re-test"
