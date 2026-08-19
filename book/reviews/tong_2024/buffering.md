# Buffering

*Part of the [Tong 2024](../tong_2024.md) review.  Compares against `automesh`'s own [Buffering](../../theory/hex_from_surface/buffering.md).*

`automesh` fits the buffer layer with one projection. It optionally follows that with a separate, post-hoc smoothing pass: [Taubin or Laplace](../../theory/smoothing.md), applied after meshing completes. [HybridOctree_Hex](../tong_2024.md) folds fitting and quality improvement into a single iterative loop instead. Every boundary node is projected toward its closest surface point. Every element's Jacobian is pushed toward higher quality at the same time. Both happen together, by gradient descent on one energy functional, minimized at a fixed learning rate $\alpha = 0.8\times10^{-3}$.[^tong_energy] The full derivation of that energy is in the [review of Tong 2024, §4](../tong_2024.md#4-quality-improvement-with-jacobian-control): its piecewise split between untangling (negative-Jacobian elements) and shape improvement (positive-Jacobian elements), and the gradients and Hessians of both terms. This page covers only what governs when the loop stops. That turns out not to match how the paper's own results are ordinarily read.

## The stopping rule is a ratchet, not a convergence test

The published `v1.0` source accepts a checkpoint as a "success" once two conditions hold: every element has a positive Jacobian, and the worst boundary-node projection distance falls under a fixed tolerance. On the *first* success, an internal quality threshold jumps from its starting value to a substantially higher one. On every success after that, the threshold increases by a small fixed step. The loop's only requirement at each checkpoint is meeting whatever the threshold currently is. The threshold is therefore a **ratchet**. It climbs one rung at a time, as long as the mesh keeps clearing it. The loop has no notion of a global "best" quality to converge to.

This matters for reading Table 2's reported worst-scaled-Jacobian column. A single reproduction run was watched climbing through successive checkpoints: 0.53, 0.55, 0.56, 0.57, and on. The mesh stayed the same throughout, unchanged in size or topology. Only the ratchet rung changed. Table 2's worst-SJ value is best read as **the rung the loop had reached at the moment a human operator stopped it**. It is not a value the optimization converged to and could not exceed. The mesh's *size* is fixed early, at the dualization and trimming stages upstream. Only the quality label attached to it keeps climbing, for as long as the run is left going.

## Stalls and a crash, not yet explained

Two failure behaviors turned up repeatedly while reproducing Table 2's models with the published source. Both are distinct from ordinary slow convergence.

* **Convergence stalls.** The worst-quality point gets stuck, oscillating between the same one or two locations for thousands of checkpoints. It never again satisfies the distance tolerance needed to advance the ratchet. This happened on three separate models: [Bottle1](https://github.com/hovey/HybridOctree_Hex/blob/main/analysis.md#2026-08-17-continued--reading-the-octree-level-straight-off-the-meshes-v10-reproduces-bone-exactly), where the cause traced to the duplicated-triangle input defect described above; [Bunny](https://github.com/hovey/HybridOctree_Hex/blob/main/analysis.md#bunny-genus-0), on three of seven threshold configurations tried, none with a known input defect; and Dragon Stand2, on every one of five fitting runs.[^tier_finding] No single common cause explains the Bunny and Dragon Stand2 occurrences.
* **A reproducible crash.** One Dragon Stand2 configuration, `H_THRES[2] = 2.5`, crashed with a segmentation fault inside this stage.[^tier_finding] The crash was a wild-pointer access, distinct from the stalls above, and reproducible for that exact configuration and mesh.

Neither failure mode prevents the *upstream* stages' output from being usable. A stalled or crashed run's mesh size and topology are fixed before this stage runs, and remain valid regardless. But neither guarantees a `finalMesh.vtk` will arrive from this stage, for any given configuration. An operator watching the ratchet climb has no way to know in advance which rung, if any, will be the last one reached.

[^tong_energy]: [Review of Tong 2024, §4](../tong_2024.md#4-quality-improvement-with-jacobian-control), including the full gradient descent update $\mathbf{p}^{(k+1)} := \mathbf{p}^{(k)} - \alpha \nabla E$ and the paper's reported learning rate.
[^tier_finding]: [Finding 15, `analysis.md`](https://github.com/hovey/HybridOctree_Hex/blob/main/analysis.md#finding-15--the-five-refinement-tiers-are-not-independent-knobs-and-the-asymmetry-is-exact-not-approximate), which documents both the Dragon Stand2 stalls and the segfault, in this study's fork of the paper's repository.

---

Previous: [Trimming](trimming.md).  Next: [Reproducing Tong 2024](reproduction.md).
