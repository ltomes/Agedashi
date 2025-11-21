# Implementation Plan: Comprehensive Diagram Quality Improvement

## Date: 2025-11-16

## Goal
Satisfy ALL requirements in DIAGRAM_REQUIREMENTS.md simultaneously without breaking any functionality.

## Strategy Overview

**Key Insight from Research**:
GraphViz has fundamental limitations with HTML table labels and port specifications. Rather than fighting these limitations, we should:
1. Let GraphViz handle basic routing and spacing
2. Use SVG post-processing for advanced features (line jumps, connection point distribution)

**Two-Phase Approach**:
- **Phase 1**: Fix critical issues (R1-R3) - Lines through resources, spacing
- **Phase 2**: Implement advanced features (R4, R6) - Connection distribution, line jumps

## Phase 1: Critical Fixes (Satisfy R1, R2, R3)

### Step 1.1: Revert Port Attributes
**Action**: Remove `headport` and `tailport` attributes from edge generation

**Rationale**:
- Port attributes appear to cause routing conflicts with HTML labels
- Automatic routing is safer and proven to work
- Can achieve spacing through other means

**Implementation**:
```rust
// REVERT TO:
dot.push_str(&format!("    {} -> {};\n", from_node, to_node));

// REMOVE:
// dot.push_str(&format!("    {} -> {} [headport=\"{}\", tailport=\"{}\"];\n", ...));
```

**Testing**: Verify lines no longer pass through resources

### Step 1.2: Optimize Spacing Parameters
**Action**: Scientifically determine optimal spacing values through experimentation

**Current Values** (likely insufficient):
```
trace: nodesep=3.5, ranksep=4.0, sep="+40,40", esep="+35,35", margin=0.45
```

**Research-Based Targets**:
- PCB design rule: 0.007-0.010 inches clearance
- Network diagram best practice: "clear space between all elements"
- GraphViz recommendation: Combine overlap, sep, esep, nodesep, ranksep

**Testing Methodology**:
1. Create test graph with known worst-case scenarios:
   - Multiple parallel edges between same nodes
   - Dense clustering of nodes
   - Mixed node sizes (different label lengths)

2. Binary search for minimum spacing values that prevent overlaps:
   - Start with current values
   - Increase by 0.5 increments until overlaps disappear
   - Test across all routing modes

3. Document optimal values per routing mode

**Expected Range** (hypothesis):
```
Minimum viable spacing (prevent overlaps):
- nodesep: 4.0-6.0
- ranksep: 5.0-7.0
- margin: 0.8-1.2

Optimal spacing (professional appearance):
- nodesep: 5.0-7.0
- ranksep: 6.0-8.0
- margin: 1.0-1.5
```

### Step 1.3: Implement Overlap Prevention Best Practices
**Action**: Apply all recommended GraphViz overlap prevention techniques

**Checklist**:
- [x] `concentrate=true` - Merge parallel edges (already implemented)
- [x] `overlap` attribute - Use "scalexy" or "false" (already implemented)
- [x] `sep` and `esep` - Edge spacing (already implemented)
- [ ] Validate `splines` settings don't conflict with overlap prevention
- [ ] Test `packmode` for better overall layout

**Additional Technique**: Invisible Nodes for Complex Routing
- If simple spacing fails, introduce invisible nodes to guide routing
- Place invisible nodes around resources to create "keep-out" zones

**Implementation** (if needed):
```rust
// Add invisible spacer nodes
for node_id in node_map.values() {
    dot.push_str(&format!(
        "    {}_spacer_n [shape=point, style=invis, height=0.5, width=0.5];\n",
        node_id
    ));
    dot.push_str(&format!("    {} -> {}_spacer_n [style=invis];\n", node_id, node_id));
}
```

### Step 1.4: Validate Against All Routing Modes
**Action**: Systematic testing of all combinations

**Test Matrix**:
| Routing Mode | Direction | Validation Criteria |
|--------------|-----------|---------------------|
| curved | TB | R1, R2, R3 satisfied |
| curved | LR | R1, R2, R3 satisfied |
| ortho | TB | R1, R2, R3 satisfied |
| ortho | LR | R1, R2, R3 satisfied |
| trace | TB | R1, R2, R3 satisfied |
| trace | LR | R1, R2, R3 satisfied |
| polyline | TB | R1, R2, R3 satisfied |
| polyline | LR | R1, R2, R3 satisfied |

**Validation Process**:
1. Generate all 8 samples
2. Visual inspection for:
   - Lines through resources (R1)
   - Overlapping endpoints (R2)
   - Adequate spacing (R3)
3. Measure spacing in problematic areas
4. Document any remaining issues

**Phase 1 Success Criteria**:
✅ All 8 sample combinations pass visual validation
✅ No lines pass through resources
✅ No obvious overlapping endpoints
✅ Professional appearance maintained

## Phase 2: Advanced Features (Satisfy R4, R6)

**Note**: Only proceed to Phase 2 after Phase 1 is 100% complete and validated

### Step 2.1: Implement SVG Post-Processing Infrastructure

**Action**: Build robust SVG parsing and modification system

**Implementation**:
```rust
struct EdgePath {
    id: String,
    source_node: String,
    target_node: String,
    path_data: String,        // SVG path 'd' attribute
    segments: Vec<LineSegment>,
    color: String,
    width: f32,
}

struct LineSegment {
    x1: f32, y1: f32,
    x2: f32, y2: f32,
}

struct NodeBounds {
    id: String,
    x: f32, y: f32,
    width: f32, height: f32,
}

fn parse_svg_graph(svg_content: &str) -> Result<(Vec<EdgePath>, Vec<NodeBounds>)> {
    // Parse SVG to extract:
    // 1. All edge paths with their geometries
    // 2. All node bounding boxes
    // 3. Edge-to-node associations
}

fn modify_svg(
    svg_content: &str,
    edges: Vec<EdgePath>,
    nodes: Vec<NodeBounds>,
) -> Result<String> {
    // Apply modifications:
    // 1. Distribute connection points (R4)
    // 2. Add line jumps at intersections (R6)
    // 3. Regenerate SVG with modifications
}
```

**Dependencies**:
- Add `roxmltree` or `quick-xml` for SVG parsing
- Add 2D geometry library (`geo`, `geo-types`) for intersection detection
- Consider `svg` crate for manipulation

### Step 2.2: Implement Connection Point Distribution (R4)

**Action**: Post-process SVG to adjust edge endpoints

**Algorithm**:
```
For each node:
1. Identify all edges connected to this node
2. Get actual node bounding box from SVG
3. Calculate available connection points on 3 sides (avoid label side)
4. Distribute edges across these points
5. Modify edge path endpoints to target distributed points
6. Ensure endpoints remain outside node bounds (margin respect)
```

**Connection Point Calculation**:
```rust
fn calculate_connection_points(
    node_bounds: &NodeBounds,
    edge_count: usize,
    avoid_side: Side, // South for labels
) -> Vec<(f32, f32)> {
    // For rectangular node, calculate points on 3 sides
    // Evenly distribute edge_count across available perimeter

    let available_sides = match avoid_side {
        Side::South => vec![Side::North, Side::East, Side::West],
        _ => vec![Side::North, Side::South, Side::East, Side::West],
    };

    // Distribute points with minimum spacing between them
    // Return (x, y) coordinates for each connection point
}
```

**Endpoint Modification**:
```rust
fn adjust_edge_endpoints(
    edge: &mut EdgePath,
    source_point: (f32, f32),
    target_point: (f32, f32),
) -> Result<()> {
    // Modify SVG path to start at source_point and end at target_point
    // Preserve internal routing, only adjust endpoints
}
```

### Step 2.3: Implement Line Jumps (R6)

**Action**: Detect edge intersections and insert gaps/arcs

**Algorithm**:
```
1. For each pair of edges:
   a. Check if paths intersect (use 2D geometry library)
   b. Record intersection points
2. For each edge with intersections:
   a. Sort intersection points along path
   b. Determine which edge should "jump" (z-order or consistent rule)
   c. Insert gap or arc at intersection point
3. Regenerate SVG with modified paths
```

**Intersection Detection**:
```rust
use geo::{Line, LineString};

fn find_edge_intersections(edges: &[EdgePath]) -> Vec<Intersection> {
    let mut intersections = Vec::new();

    for (i, edge1) in edges.iter().enumerate() {
        for edge2 in edges.iter().skip(i + 1) {
            // Convert edge paths to line segments
            let segments1 = edge1.segments.iter();
            let segments2 = edge2.segments.iter();

            for (s1_idx, seg1) in segments1.enumerate() {
                for (s2_idx, seg2) in segments2.enumerate() {
                    if let Some(point) = line_intersection(seg1, seg2) {
                        intersections.push(Intersection {
                            edge1_id: edge1.id.clone(),
                            edge2_id: edge2.id.clone(),
                            point,
                            edge1_segment: s1_idx,
                            edge2_segment: s2_idx,
                        });
                    }
                }
            }
        }
    }

    intersections
}
```

**Gap Insertion** (user's preferred style):
```rust
fn insert_gap_at_intersection(
    path: &EdgePath,
    intersection: &Intersection,
    gap_size: f32, // e.g., 8.0 pixels
) -> String {
    // Split path at intersection point
    // Remove gap_size/2 before and after intersection
    // Create two separate path segments with gap between them
}
```

### Step 2.4: Integration and Testing

**Action**: Integrate post-processing into generation pipeline

**Modified Flow**:
```
1. Generate DOT graph (no port attributes)
2. Execute GraphViz (dot command)
3. Get initial SVG output
4. Post-process SVG:
   a. Parse edges and nodes
   b. Distribute connection points (if enabled)
   c. Add line jumps (if enabled)
   d. Regenerate SVG
5. Embed base64 or save final SVG
6. Convert to PNG if needed
```

**CLI Flags**:
```rust
#[arg(long, default_value = "true")]
distribute_connections: bool,

#[arg(long, default_value = "gap")]
line_jump_style: String,  // none, gap, arc, sharp

#[arg(long, default_value = "8.0")]
line_jump_size: f32,
```

**Testing**:
- Test with/without connection distribution
- Test each line jump style
- Validate performance impact
- Ensure deterministic output

## Phase 3: Optimization and Polish

### Step 3.1: Performance Optimization
- Profile SVG post-processing overhead
- Optimize intersection detection (spatial indexing if needed)
- Cache results where possible

### Step 3.2: Documentation
- Update README with new features
- Document all CLI parameters
- Add examples showing different modes

### Step 3.3: Additional Enhancements
- Consider dynamic spacing based on graph complexity
- Add option for manual spacing override
- Implement layout hints for complex graphs

## Implementation Timeline

**Phase 1**: Critical Fixes
- Step 1.1: 30 minutes (revert ports)
- Step 1.2: 2-3 hours (spacing optimization)
- Step 1.3: 1-2 hours (overlap prevention)
- Step 1.4: 1 hour (validation)
- **Total: ~5-7 hours**

**Phase 2**: Advanced Features
- Step 2.1: 3-4 hours (infrastructure)
- Step 2.2: 2-3 hours (connection distribution)
- Step 2.3: 3-4 hours (line jumps)
- Step 2.4: 2 hours (integration)
- **Total: ~10-13 hours**

**Phase 3**: Polish
- ~2-3 hours

**Grand Total: ~17-23 hours of focused development**

## Risk Mitigation

### Risk 1: SVG Post-Processing Complexity
**Mitigation**: Start with simple cases, build incrementally
**Fallback**: Make post-processing optional (CLI flag)

### Risk 2: Performance Impact
**Mitigation**: Profile early, optimize critical paths
**Fallback**: Disable post-processing for very large graphs

### Risk 3: Edge Cases in Path Modification
**Mitigation**: Extensive testing with varied graph structures
**Fallback**: Graceful degradation (skip problematic modifications)

## Success Criteria

### Must Have (Phase 1)
- ✅ R1: No lines through resources
- ✅ R2: No overlapping endpoints (basic - via spacing)
- ✅ R3: Appropriate spacing
- ✅ All 8 sample combinations validated

### Should Have (Phase 2)
- ✅ R4: Distributed connection points
- ✅ R6: Line jumps (gap style minimum)
- ✅ Backward compatibility maintained
- ✅ Performance acceptable (<2x slowdown)

### Nice to Have (Phase 3)
- ✅ All line jump styles (gap, arc, sharp)
- ✅ Dynamic spacing
- ✅ Comprehensive documentation

## Decision Points

### Decision 1: After Step 1.2
**Question**: Can we achieve acceptable results with spacing alone?
- **If YES**: Maybe Phase 2 is optional, make it enhancement
- **If NO**: Proceed with Phase 2 as planned

### Decision 2: After Step 2.1
**Question**: Is SVG post-processing feasible and performant?
- **If YES**: Continue with Phase 2
- **If NO**: Explore alternative approaches (e.g., custom layout engine)

### Decision 3: After Phase 1 Complete
**Question**: Does user want to proceed to Phase 2?
- **If YES**: Begin Phase 2
- **If NO**: Stop at Phase 1, document future enhancements

## Next Steps

1. **Get user feedback on this plan**
2. **Begin Phase 1, Step 1.1** (revert port attributes)
3. **Validate results** before proceeding
4. **Iterate based on findings**

## Alternative Approaches Considered

### Alternative 1: Custom Layout Engine
**Description**: Build custom graph layout algorithm instead of using GraphViz
**Pros**: Full control over all aspects
**Cons**: Massive effort, reinventing wheel, likely inferior to GraphViz
**Decision**: Rejected

### Alternative 2: Switch to Different Graph Tool
**Description**: Use yFiles, D3.js, or other graph library
**Pros**: Might have better API for our needs
**Cons**: Learning curve, may lose existing features, licensing issues
**Decision**: Rejected for now, keep as backup

### Alternative 3: Embrace GraphViz Limitations
**Description**: Accept automatic routing, focus only on spacing
**Pros**: Simplest, most reliable
**Cons**: Doesn't fully satisfy R4 and R6
**Decision**: This is Phase 1 approach, with Phase 2 as enhancement
