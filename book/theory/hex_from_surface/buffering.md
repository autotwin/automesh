# Buffering

*Stage 5 of five.  Operates on **hexahedra**.  See [Hexahedral Meshing from a Surface](../hex_from_surface.md) for the pipeline overview and terminology.*

[Trimming](trimming.md) leaves a blocky, stair-stepped boundary that does not follow the surface.  Buffering resolves this by adding one conforming layer:

1. The exterior faces of the trimmed mesh are extracted.
2. Each boundary node is projected to its closest point on the surface.
3. A hexahedron is extruded from each boundary face out to the projected nodes.

Before projecting, the extracted boundary is checked for manifoldness: every edge must be shared by exactly two faces.  If not, meshing fails with

```
Error: non-manifold boundary.
```

This check is on the **output** boundary, not the input surface, and it is the only manifoldness test in the pipeline.  It is a common failure — see [Failure Modes](../hex_from_surface.md#failure-modes).

The buffer elements are the only elements whose geometry is fitted to the surface, and consequently the only elements whose quality is unbounded.  A poorly resolved or highly curved surface region will show up here and nowhere else.

---

Previous: [Trimming](trimming.md).  This is the final stage; the mesh is written out from here.  For what to do with the result, see [Quality Assessment and Improvement](../hex_from_surface.md#quality-assessment-and-improvement).
