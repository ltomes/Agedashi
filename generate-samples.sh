#!/bin/bash
# Generate sample diagrams for all edge routing modes
# This creates a matrix of samples to demonstrate different routing algorithms

set -e

echo "🎨 Generating Sample Infrastructure Diagrams"
echo "=============================================="
echo ""

# Build the project first
echo "📦 Building agedashi..."
cargo build --release
echo ""

# Routing modes to test
ROUTING_MODES=("curved" "ortho" "trace" "polyline")
DIRECTIONS=("TB" "LR")

# Input test file
INPUT_FILE="test/sample-graph.dot"

if [ ! -f "$INPUT_FILE" ]; then
    echo "❌ Error: $INPUT_FILE not found"
    exit 1
fi

echo "📊 Generating samples for each routing mode..."
echo ""

for routing in "${ROUTING_MODES[@]}"; do
    for direction in "${DIRECTIONS[@]}"; do
        output_name="sample-${routing}-${direction,,}"

        echo "  🔹 Generating: $output_name (routing=$routing, direction=$direction)"

        # Generate PNG
        cat "$INPUT_FILE" | ./target/release/agedashi \
            --output png \
            --name "samples/$output_name" \
            --direction "$direction" \
            --edge-routing "$routing" \
            2>&1 | grep -v "^Found\|^Visualizing\|^Using"

        # Generate SVG
        cat "$INPUT_FILE" | ./target/release/agedashi \
            --output svg \
            --name "samples/$output_name" \
            --direction "$direction" \
            --edge-routing "$routing" \
            2>&1 | grep -v "^Found\|^Visualizing\|^Using"
    done
    echo ""
done

echo "✅ Sample generation complete!"
echo ""
echo "📁 Generated files in samples/:"
ls -1 samples/sample-*.png | while read file; do
    echo "   - $(basename $file)"
done

echo ""
echo "🔍 Routing Mode Descriptions:"
echo "   • curved   - Smooth curved edges with node avoidance"
echo "   • ortho    - Orthogonal (horizontal/vertical) routing"
echo "   • trace    - Circuit board PCB style with strict spacing"
echo "   • polyline - Straight segments with angles"
