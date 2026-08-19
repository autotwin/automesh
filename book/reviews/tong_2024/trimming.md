# Trimming

*Part of the [Tong 2024](../tong_2024.md) review.  Compares against `automesh`'s own [Trimming](../../theory/hex_from_surface/trimming.md).*

`automesh`'s trimming stage clears a gap around the trimmed boundary, so the fitting stage has room to work. [HybridOctree_Hex](../tong_2024.md) reaches the same goal with a different unit of removal. `automesh` evaluates each **hexahedron**. It applies a fixed clearance margin, half the element's own shortest edge, and keeps the element only if every node clears the surface. HybridOctree_Hex's primary rule evaluates each **vertex** instead. If a vertex's distance to the surface falls below half the size of the largest element touching it, the rule deletes *every* element sharing that vertex, not just the one nearest the boundary.[^vertex_clearing]

That vertex-driven rule has a documented failure mode. The paper describes it directly: a single flagged vertex near a size transition can force deletion of one large element. That deletion can open a hole a neighborhood of small elements cannot refill. The published source addresses this with a second, per-**element** criterion layered on top. It evaluates a signed distance function $f$ at all eight corners of each hexahedron, using the material convention (positive inside, negative outside). The source removes the element if $f_{\min} + 0.1\,f_{\max} < 0$.[^sdf_removal] This is a soft rule. It keeps an element even if one corner pokes slightly outside, as long as the rest of the element stays safely interior. A further restriction step then removes any remaining boundary elements whose surrounding face normals are too sharply angled to fit well once connected to the surface.[^restriction]

Both methods solve the same problem with the same intuition: clear a margin proportional to local element size. `automesh` applies its rule once, per hexahedron, using only that element's own geometry. HybridOctree_Hex applies two rules in sequence. The second rule explicitly compensates for a brittleness in the first. The full derivation, including the sign-reversal subtlety in the SDF criterion, is in the [review of Tong 2024](../tong_2024.md#31-clearing).

[^vertex_clearing]: [Review of Tong 2024, §3.1](../tong_2024.md#31-clearing), "Vertex Clearing Function."
[^sdf_removal]: [Review of Tong 2024, §3.1](../tong_2024.md#signed-distance-function-sdf). The paper's own wording on the brittleness this addresses, and the full SDF derivation with worked examples, are given there in full.
[^restriction]: [Review of Tong 2024, §3.2](../tong_2024.md#32-restriction).

---

Previous: [Dualization](dualization.md).  Next: [Buffering](buffering.md).
