# Equilibration

*Part of the [Tong 2024](../tong_2024.md) review.  Compares against `automesh`'s own [Equilibration](../../theory/hex_from_surface/equilibration.md).*

`automesh` offers weak and strong balancing as a user choice. [HybridOctree_Hex](../tong_2024.md) has only one balancing rule. That rule is unconditionally the stronger of the two. Every leaf cell is checked against the full group of eight cells sharing its vertex. That check is exactly the face-*and*-edge-*and*-vertex adjacency `automesh`'s `--strong` flag constrains.[^strong_balance] HybridOctree_Hex has no weaker option to opt into.

The mechanism also differs structurally from a template-driven pairing rule. Suppose a cell in that eight-cell group is more than one level coarser than the group's deepest member. The source does not refine that one cell alone. It refines the cell **and its seven siblings together**, as a family.[^refine_brothers] It then re-checks the whole octree, and repeats, until no cell fails the test. This "refine as a family" behavior is what the paper calls its pairing rule. A coarse cell is never split alone.

This family-refinement logic reappears later, unmodified, inside the refinement-criteria stage. There, the routine that decides whether to refine a cell checks its children first. It falls through to the cell's own threshold test only if none of the children already need refining. That one design choice makes a refinement threshold behave asymmetrically. Tightening a threshold and loosening a threshold have different effects. See [Threshold Fitting as a Mechanical Procedure](threshold_fitting.md) for the full consequence.

[^strong_balance]: `HexGen.cpp`, `StrongBalancedOctree()`. Each leaf's group is found via `IsSharedByEightCells()`. A level gap greater than one, against the group's deepest cell, triggers refinement.
[^refine_brothers]: `HexGen.cpp`, `StrongBalancedOctree()` calling `RefineBrothers()`. `RefineBrothers()` refines all eight children of the coarse cell's parent together. `StrongBalancedOctree()` then re-enters itself recursively, until it reports zero unbalanced nodes on a pass.

---

Previous: [Octree Construction](octree_construction.md).  Next: [Dualization](dualization.md).
