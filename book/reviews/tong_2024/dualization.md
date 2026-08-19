# Dualization

*Part of the [Tong 2024](../tong_2024.md) review.  Compares against `automesh`'s own [Dualization](../../theory/hex_from_surface/dualization.md).*

[HybridOctree_Hex](../tong_2024.md) balances its octree unconditionally at the vertex-adjacency level (see [Equilibration](equilibration.md)). It dualizes that octree with a catalog of **five** templates: **one** face-transition template and **four** edge-transition templates.[^tong_templates] `automesh`'s catalog under `--strong` has **six**: two face templates (FT0, FT1) and the same **four** edge templates (ET1–ET4). `--strong` is the balancing mode this comparison uses, since Tong 2024 has no weaker option. The edge count matches exactly. The two methods differ only in how many face-transition cases each treats as a distinct configuration.

The paper reports a resulting minimum scaled Jacobian of **0.258** for its template mesh, before any quality-improvement stage runs.[^tong_msj] `automesh`'s own strong-balancing floor is $1/\sqrt{15} \approx 0.258199$, the same number to three decimals. See [Template Quality](../../theory/hex_from_surface.md#template-quality). This study has not checked HybridOctree_Hex's dualization source directly, so it cannot yet say whether the two independently-derived template sets happen to share the same worst-case configuration, or whether the two catalogs are structurally equivalent at a deeper level. This page records the agreement as a fact worth noting, not as a proven equivalence.

[^tong_templates]: Per the paper's own description, as summarized in the [review of Tong 2024, §2](../tong_2024.md#2-all-hex-dual-mesh). Not independently re-derived from the `v1.0` source's dualization code in this study.
[^tong_msj]: The paper states this as the quality floor "before Section 4 optimization," meaning before the buffer-zone quality-improvement stage described in [Buffering](buffering.md).

---

Previous: [Equilibration](equilibration.md).  Next: [Trimming](trimming.md).
