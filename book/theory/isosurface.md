# Isosurfaces

Isosurfacing extracts a surface from a three-dimensional scalar field — the
step that turns a segmented volume into a triangulated boundary.
[Introduction](isosurface/introduction.md) defines the problem and surveys
the three methods used in practice, including the one `automesh` currently
implements. [Implementation Plan](isosurface/implementation_plan.md) scopes
the work to add the other two. [Marching Cubes](isosurface/marching_cubes.md)
and [Dual Contouring](isosurface/dual_contouring.md) cover those two
widely used general-purpose algorithms in detail.
