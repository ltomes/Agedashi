# Current Implementation Issues - Analysis

## Date: 2025-11-16

## Critical Issue: Lines Passing Through Resources

### Affected Sample
- **File**: `samples/sample-ortho-lr.png`
- **Status**: Lines are passing through resource icons/labels
- **Severity**: CRITICAL - Violates R1 (No Line-Resource Intersection)

### Root Cause Analysis

#### What Changed
In commit `a595061`, I added smart port distribution:
- Added `headport` and `tailport` attributes to edges
- Intended to distribute connection points across multiple sides
- Used compass points: n, ne, e, w, nw (avoiding s, se, sw)

#### Why It Broke

**Theory 1: Port Attributes with HTML Labels**
- Earlier we discovered that port SYNTAX (`node:n -> node:s`) doesn't work with HTML table labels
- Port ATTRIBUTES (`headport="n", tailport="s"`) might have similar issues
- GraphViz may be forcing edge routing to specific compass points, overriding margin/spacing settings

**Theory 2: Insufficient Margin/Spacing**
- Port attributes might work, but our margin values are insufficient when edges target specific compass points
- Edges routed to "e" or "w" might bypass the margin zone
- HTML table dimensions (128px icon + label) may not be accurately represented in graph units

**Theory 3: Conflict with Routing Algorithms**
- Combining `splines=ortho` with specific port assignments might create routing conflicts
- Orthogonal routing tries to create horizontal/vertical paths, but specific ports might force edges through nodes
- The `overlap=scalexy` setting might not work well with explicit port assignments

#### Evidence from GraphViz Documentation

From research:
> "Port specification: Edges can have their origin and endpoint on a specific port (n, ne, e, se, s, sw, w, nw, w, c, _)"

But also:
> "Compass point ports (node:n, node:s) don't work with HTML table labels (shape=plaintext)"

**Key Question**: Do port ATTRIBUTES (`headport="n"`) work differently than port SYNTAX (`node:n`) with HTML labels?

## Other Identified Issues

### Issue 2: Overlapping Endpoints
- **Status**: Likely still present
- **Cause**: Even with distributed ports, multiple edges to same compass point will stack
- **Example**: If 3 edges target `node:n`, all 3 endpoints stack at north position

### Issue 3: Spacing Parameter Validation
- **Status**: Spacing values are theoretical, not validated against actual output
- **Problem**: We increased spacing values based on guesswork, not measurement
- **Result**: Some modes may have insufficient spacing despite large parameter values

### Issue 4: Port Distribution Algorithm
- **Current Logic**: Simple round-robin distribution across 5 compass points
- **Problem**: Doesn't account for actual spatial positioning
- **Example**: In LR layout, "n" and "s" ports are vertically distributed, but algorithm treats them equally

## What Was Working Before

### Previous Approach (Before Port Distribution)
```rust
// Simple edge generation without ports
dot.push_str(&format!("    {} -> {};\n", from_node, to_node));
```

**Pros**:
- GraphViz handled routing automatically
- No port conflicts with HTML labels
- Margins provided basic clearance

**Cons**:
- All edges converged at node center
- No distribution of connection points
- Visual clutter with multiple edges

## Comparison: What We Need vs What We Have

| Requirement | Current Implementation | Status |
|-------------|------------------------|--------|
| R1: No line through resources | Port attributes + margins | ❌ BROKEN |
| R2: No overlapping endpoints | Port distribution (5 points) | ❌ BROKEN |
| R3: Appropriate spacing | sep, esep, nodesep, ranksep | ⚠️ PARTIALLY |
| R4: Organized connections | Compass port distribution | ❌ BROKEN |
| R5: Circuit/trace mode | Polyline routing + spacing | ✅ WORKING |
| R6: Line jumps | Framework only | ❌ NOT IMPLEMENTED |

## Failed Approaches History

### Attempt 1: Compass Port Syntax
- **Code**: `node_0:s -> node_1:n`
- **Result**: GraphViz error "lost node_X node_Y edge"
- **Reason**: Incompatible with HTML table labels

### Attempt 2: Port Attributes
- **Code**: `headport="n", tailport="s"`
- **Result**: Lines passing through resources
- **Reason**: Unknown (need to investigate)

## Hypotheses for Solution

### Hypothesis 1: Remove Port Attributes Entirely
- Go back to automatic routing
- Rely solely on spacing parameters
- Accept that edges converge at center

**Pros**: Simple, won't break existing functionality
**Cons**: Doesn't solve endpoint overlap issue (R2, R4)

### Hypothesis 2: Use Invisible Spacing Nodes
- Add invisible helper nodes to guide edge routing
- Place them around actual nodes to create routing paths
- Let edges route through invisible nodes

**Pros**: Doesn't rely on port attributes
**Cons**: Complex, may create other layout issues

### Hypothesis 3: Larger Margins + No Ports
- Dramatically increase node margins (0.45 → 1.0+)
- Remove all port attributes
- Rely on automatic routing with large clearance zones

**Pros**: Simple, proven to work with HTML labels
**Cons**: May create excessive whitespace, doesn't distribute endpoints

### Hypothesis 4: SVG Post-Processing for Connection Points
- Generate graph without port attributes (automatic routing)
- Post-process SVG to adjust edge endpoints
- Move endpoints to distributed positions around nodes

**Pros**: Full control over visual output
**Cons**: Complex, requires SVG parsing and modification

### Hypothesis 5: Switch to Record-Based Nodes
- Use `shape=record` instead of HTML table labels
- Port syntax works with record shapes
- Lose ability to embed PNG icons

**Pros**: Port syntax would work natively
**Cons**: Lose icon embedding (critical feature)

## Recommended Investigation Steps

1. **Test Port Attributes in Isolation**
   - Create minimal example with HTML label + headport/tailport
   - Verify if attributes work or cause routing issues

2. **Measure Actual Node Dimensions**
   - Determine how GraphViz interprets 128px icon in graph units
   - Calculate required margin for specific port routing

3. **Test Alternative Approaches**
   - Try invisible nodes technique
   - Try SVG post-processing approach
   - Compare results

4. **Validate Spacing Parameters**
   - Generate sample with debug markers showing node boundaries
   - Measure actual spacing achieved vs. parameter values

## Immediate Action Required

**Before any new implementation**:
1. Understand WHY port attributes caused line-through-resource issue
2. Validate which approach can satisfy ALL requirements simultaneously
3. Create proof-of-concept for chosen approach
4. Test across all routing modes and directions

**Do NOT**:
- Make incremental changes without understanding root cause
- Fix one requirement while breaking others
- Commit without validating all routing modes
