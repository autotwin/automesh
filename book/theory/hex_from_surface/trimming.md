# Trimming

*Stage 4 of five.  Operates on **hexahedra**.  See [Hexahedral Meshing from a Surface](../hex_from_surface.md) for the pipeline overview and terminology.*

The octree is built over the bounding box of the surface, not the surface itself, so dualization produces hexahedra throughout that box — including regions outside the solid.  Trimming discards them.

From this stage onward the octree is no longer consulted; trimming and buffering operate purely on the dual hexahedra.

Each dual **node** is classified inside or outside by casting rays against a bounding volume hierarchy of the surface and inspecting the orientation of the first facet hit.  Three separate ray directions are used, and rays that graze a facet nearly tangentially are rejected in favor of the next direction — a single ray is not robust against grazing hits and coincident facets.

A **hexahedron** is then retained only if **every** one of its eight nodes is inside *and* every node clears the surface by at least half the length of the element's shortest edge.  This clearance margin reserves room for the buffer layer that follows.

Trimming only removes elements; it never moves a node.  Elements that survive trimming retain exactly their template geometry, and therefore their guaranteed quality.

---

Previous: [Dualization](dualization.md).  Next: [Buffering](buffering.md), which fits the trimmed boundary to the surface.
