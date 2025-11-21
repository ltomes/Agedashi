# Diagram Generation Requirements

## Purpose
This document defines the comprehensive requirements for diagram generation in Agedashi to ensure high-quality, clear, and professional infrastructure visualizations.

## Core Principles

### 1. Visual Clarity
- **Primary Goal**: Create unambiguous diagrams that are easy to interpret
- **Avoid "Hairball" Effect**: Even complex graphs should maintain visual clarity
- **Consistent Style**: All elements should follow a coherent visual language

### 2. Professional Aesthetics
- **Clean Design**: Minimize visual clutter and maximize information density
- **Intuitive Flow**: Natural reading order (typically top-to-bottom or left-to-right)
- **Appropriate Spacing**: Balance between compact layout and readability

## Critical Requirements (Non-Negotiable)

### R1: No Line-Resource Intersection
**Requirement**: Lines (edges) MUST NEVER pass through or overlap with resource nodes (icons/labels)

**Rationale**:
- From research: "To produce high quality drawings it is important to find routes for edges which do not intersect node boundaries"
- Critical for clarity and professionalism

**Current Status**: ❌ BROKEN (lines passing through resources in ortho-lr mode)

**Implementation Requirements**:
- Node margins must create adequate clearance zones
- Edge routing algorithms must respect node boundaries
- Port positioning must account for icon size + label height

### R2: No Line-Line Intersection at Connection Points
**Requirement**: Lines and their endpoints (dots/arrows) MUST NOT touch or overlap with other lines' endpoints

**Rationale**:
- Network diagram best practice: "draw the diagram in such a way that none of the lines cross each other"
- Prevents confusion about which connection belongs to which resource

**Current Status**: ❌ BROKEN (overlapping starting points)

**Implementation Requirements**:
- Distributed connection points across multiple sides of each node
- Minimum spacing between connection points on same node
- Intelligent port assignment to prevent clustering

### R3: Appropriate Line Spacing
**Requirement**: Lines should maintain adequate spacing from each other throughout their route

**Rationale**:
- Prevents visual clutter
- Makes individual connections traceable
- Professional appearance

**Implementation Requirements**:
- `sep` and `esep` attributes for edge-to-edge spacing
- `nodesep` and `ranksep` for node distribution
- Routing-mode-specific spacing parameters

### R4: Organized Connection Point Distribution
**Requirement**: Connection points should be intelligently distributed around resources

**Rationale**:
- Prevents all lines converging at single point
- Reduces visual clutter
- Allows for clearer tracing of connections

**Implementation Requirements**:
- Use 3 sides of each node (avoid label side)
- Distribute based on edge count and direction
- Consider incoming vs outgoing edge separation

## Routing Mode Requirements

### R5: Circuit/Trace Mode (Inspired by PCB Design)
**Requirement**: When in "trace" mode, follow hardware design patterns

**PCB Design Principles to Apply**:
- **Avoid 90-degree angles**: Use 45-degree or curved transitions
- **Consistent trace width**: Maintain uniform line width
- **Adequate clearance**: 0.007-0.010 inches equivalent spacing
- **Orthogonal preference**: Horizontal/vertical routing when possible
- **No sharp corners**: Prevents signal integrity issues (aesthetic parallel)

**Implementation**:
- Use polyline or ortho splines with generous spacing
- Wider nodesep/ranksep than other modes
- Consider splines=ortho with rounded corners

### R6: Line Jump Support
**Requirement**: When lines must cross, use line jumps (gaps or arcs) to show which line is "on top"

**Draw.io Style Options**:
- **Gap**: Line stops before crossing (preferred by user)
- **Arc**: Small arc over crossing line
- **Sharp**: Sharp angle around crossing
- **None**: Simple overlap (default, least preferred)

**Current Status**: ⚠️ FRAMEWORK ONLY (not fully implemented)

**Implementation Requirements**:
- SVG post-processing to detect edge path intersections
- 2D geometry library for line segment intersection detection
- Path modification to insert gaps/arcs at crossings

## Label and Text Requirements

### R7: Label Positioning
**Requirement**: Resource labels must not be obscured by lines

**Implementation**:
- Labels positioned on dedicated side (typically bottom/south)
- No connection points on label side
- Adequate margin between label and lines

### R8: Label Readability
**Requirement**: All text must be clearly readable

**Implementation**:
- Minimum font size (current: 10pt)
- High contrast colors
- No text overlap with other elements

## Spacing and Layout Requirements

### R9: Routing-Specific Spacing
**Requirement**: Each routing mode should have optimized spacing parameters

**Current Parameters** (from research and existing code):

| Mode | nodesep | ranksep | sep | esep | margin | Use Case |
|------|---------|---------|-----|------|--------|----------|
| curved | 2.0 | 2.5 | +25,25 | +20,20 | 0.25 | General purpose, smooth aesthetics |
| ortho | 2.5 | 3.0 | +30,30 | +25,25 | 0.35 | System diagrams, ER diagrams |
| trace | 3.5 | 4.0 | +40,40 | +35,35 | 0.45 | Circuit board style, strict spacing |
| polyline | 2.0 | 2.5 | +25,25 | +20,20 | 0.25 | Straight segments with angles |

**Validation**: Parameters should be validated against actual output to prevent overlaps

### R10: Dynamic Scaling
**Requirement**: Spacing should adapt to graph complexity

**Future Enhancement**:
- Detect edge density and increase spacing accordingly
- Larger graphs may need proportionally more spacing

## Connection Style Requirements

### R11: Visual Distinction
**Requirement**: Connection endpoints should clearly indicate direction

**Current Options**:
- `dot`: Dot at source only
- `arrow`: Arrow at target only
- `hybrid`: Dot at source, arrow at target (default)
- `odot`: Open dot at source, arrow at target
- `none`: Plain line

**Best Practice**: Use `hybrid` for maximum clarity (shows both origin and direction)

### R12: Appropriate Sizing
**Requirement**: Connection markers should be visible but not overwhelming

**Current Defaults**:
- `connection_size`: 1.2
- `edge_width`: 2.5
- `arrowsize`: 1.2

## Color and Contrast Requirements

### R13: Consistent Color Scheme
**Requirement**: Use service-specific colors for resources, consistent edge colors

**Implementation**:
- AWS service colors for resource nodes
- Single edge color (default: #2D3436) for simplicity
- High contrast against transparent background

## Technical Implementation Constraints

### GraphViz Limitations (from research)
1. **Port Syntax**: Compass ports (node:n, node:s) don't work with HTML table labels
2. **Port Attributes**: Must use `headport/tailport` attributes instead
3. **Overlap Prevention**: No perfect solution; requires combination of techniques
4. **Line Jumps**: Not natively supported; requires SVG post-processing

### Required Approach
- Use `headport` and `tailport` attributes (not node:port syntax)
- Combine `overlap`, `sep`, `esep`, `nodesep`, `ranksep` for spacing
- Use `concentrate=true` to merge parallel edges
- Post-process SVG for line jumps

## Validation Criteria

### Visual Validation Checklist
Before any commit, verify ALL of the following:

- [ ] No lines pass through resource icons
- [ ] No lines pass through resource labels
- [ ] No line endpoints overlap with other endpoints
- [ ] Lines maintain consistent spacing from each other
- [ ] Connection points are distributed around resources (not stacked)
- [ ] All resource labels are clearly readable
- [ ] Diagram maintains professional appearance
- [ ] All routing modes (curved, ortho, trace, polyline) work correctly
- [ ] Both directions (TB, LR) render properly

### Automated Testing
- Integration tests should verify no GraphViz errors
- Performance tests for large graphs
- Visual regression tests comparing sample outputs

## Research Sources

This document is based on research from:
- IEEE publications on edge routing algorithms
- Tom Sawyer Software graph layout principles
- Riccardo Mazza's information visualization principles
- PCB design best practices from Altium, Cadence, Sierra Circuits
- Draw.io connector and line jump documentation
- GraphViz official documentation on attributes and routing
- Stack Overflow GraphViz community best practices

## Revision History

- 2025-11-16: Initial comprehensive requirements document created after identifying issues with incremental implementation approach
