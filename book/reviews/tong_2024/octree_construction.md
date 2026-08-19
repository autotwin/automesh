# Octree Construction

*Part of the [Tong 2024](../tong_2024.md) review.  Compares against `automesh`'s own [Octree Construction](../../theory/hex_from_surface/octree_construction.md).*

`automesh` sizes every cell from a single number: the shape diameter function. [HybridOctree_Hex](../tong_2024.md) is Tong, Halilaj & Zhang's independent implementation of the same dual-octree idea. It sizes cells from **two** separate per-vertex measurements instead. Each measurement has its own five-value threshold ladder. Both methods solve the same problem: where should the octree be finer? Comparing the two designs is useful because each answers that question with different information.

This page describes what the published `v1.0` source actually computes. Every claim below was verified by reading the source directly, not by paraphrasing the paper. See the [review of Tong 2024](../tong_2024.md) for the paper's own presentation of the method, including the buffer-zone and quality-improvement stages this page does not cover.

## Curvature

The paper defines a curvature measure and calls it **Gaussian curvature**, $G$. At vertex $P_i$, the paper states this formula:

$$
G(P_i) \;=\; \frac{\left\lVert \displaystyle\sum_{j \in \mathcal{N}(i)} (\cot\alpha_{ij} + \cot\beta_{ij})(P_j - P_i) \right\rVert_2}{4 A_i} \tag{1}
$$

$\mathcal{N}(i)$ is the set of vertices adjacent to $P_i$. $\alpha_{ij}$ and $\beta_{ij}$ are the two angles opposite edge $P_i P_j$. $A_i$ is the Voronoi cell area around $P_i$.

Equation 1 is not Gaussian curvature. Gaussian curvature at a vertex is normally computed from the *angle defect* there: $2\pi$ minus the sum of the incident triangles' interior angles, divided by the Voronoi area. Equation 1 does something else. It sums cotangent-weighted position differences over adjacent vertices, then normalizes by Voronoi area. That construction is the discrete Laplace-Beltrami operator applied to vertex position. Its magnitude estimates *mean* curvature, not Gaussian curvature.[^meyer2003] So the paper's own formula for $G$ does not match the paper's own name for it, independent of anything the source code does.

Equation 1 also does not appear in the source code, at any point in its history. This study searched every commit reachable from every branch, tag, and remote in its fork of the repository, `v1.0` through `v1.3` and both the `hovey` and `CMU-CBML` remotes, across every `.c`, `.cpp`, `.h`, and `.py` file ever tracked. The search covered the operator's defining ingredients directly: the literal terms `cot`, `Voronoi`, `Laplace`, and `Beltrami`; a cotangent computed via `tan` instead of a named cotangent function; and any per-vertex mixed-area or barycentric-area weighting, the Voronoi-area role in Equation 1. None of these appear anywhere, in any version, outside of this study's own added commentary.[^cot_search] Equation 1 is not just mislabeled. It was never implemented.

The published `v1.0` source computes a third, different quantity instead. This page calls it the **dihedral-defect sum**, $r[i]$. For every vertex $i$, the source accumulates:

$$
r[i] \;=\; \sum_{\text{edges } e \ni i} \bigl(\theta_e - \pi\bigr)^2 \tag{2}
$$

The sum runs over every edge incident to $i$ that is shared by exactly two triangles. $\theta_e$ is the dihedral angle between those two triangles' faces at edge $e$.[^r_accum] A flat neighborhood contributes nothing to $r[i]$, because $\theta_e = \pi$ everywhere flat. Any bend contributes its squared angular deviation from flat. Convex and concave bends contribute equally, since the deviation is squared.

Equation 2 does not match Equation 1. Equation 1 sums position vectors, weighted by cotangents, over neighboring vertices. Equation 2 sums squared angles, over incident edges. The two formulas take different inputs and measure different things. Neither is Gaussian curvature. Equation 2 is the quantity the shipped source actually tests against the paper's stated curvature thresholds, in function [`hexGen::ReadRawData()`](https://github.com/hovey/HybridOctree_Hex/blob/c3069751423f15eb972836c326d1e71f0ae9e7ff/HybridOctree_Hex_v1.0/HexGen.cpp#L882), `HexGen.cpp`, line 882, in this study's fork of the paper's repository. Anyone relating the paper's stated thresholds back to a specific differential-geometry quantity should use Equation 2, not Equation 1.

A worked case makes Equation 2 concrete. Take two exactly coincident, overlapping triangles. They share all three edges. Their shared dihedral angle is $\theta_e = 0$. Each of their three shared vertices then picks up $(0 - \pi)^2 = \pi^2 \approx 9.8696$ from that pair alone. This produces a large, easily recognized spike in $r[i]$.

This case is not hypothetical. It happened on Bottle1, one of Table 2's twelve test models. Bottle1's input file declares one more triangle than it actually contains. `ReadRawData()` reads the declared count, runs past the end of the real data, and silently re-parses the last triangle a second time as a result. That duplicate triangle is exactly the coincident-triangle case above, and its phantom spike in $r[i]$ was traced directly to this term.[^dup_tri]

## Thickness

Thickness is measured per triangle by ray-casting. A ray is cast from triangle $i$'s centroid, along $i$'s own face normal. The first triangle $j$ the ray hits gives a raw distance. That distance is then scaled by the *largest axis-aligned component* of the unit normal, a factor always in $[1/\sqrt{3}, 1]$. The scaled distance is compared against the threshold ladder. This scaling factor looked like an unexplained implementation choice, so this study tested it directly: building the source with the scaling factor disabled, and using the raw Euclidean distance instead, moves reproduced mesh sizes in the wrong direction. So the scaling is a real part of the criterion, even without a stated reason for it in the paper.[^raw_thickness]

## The five-tier ladder

The source compares $r[i]$ (curvature) and the thickness measurement against five-value threshold arrays, `C_THRES` and `H_THRES`. A vertex exceeding curvature threshold $t$ is flagged for **tier** $t$: $r[i] > \texttt{C\_THRES}[t]$. A triangle whose thickness falls below `H_THRES`$[t]$ is flagged the same way. Tier $t$'s flagged cells are refined one level deeper than tier $t-1$'s.

> **The tier ladder's position on the octree is itself a parameter, not a fixed offset.** A literal reading of the paper suggests tier $t$ always sits at octree level $t+4$. This study checked that offset empirically, across every model attempted, and found it varies. Two source constants fix it: `VOXEL_SIZE` (the octree's overall depth) and an internal constant this study calls `LADDER_TOP` (the level of the deepest tier). The published `v1.0` source hardcodes both for one specific model. Neither is exposed as a documented per-model setting.

## Refinement Level is measurable, not fixed

Table 2 of the paper reports a "Refinement Level" for every test model. Some models get 4, others 5 or 6. This is not a qualitative label. It is the literal count of active tiers in that model's ladder, and it can be read directly off any published reference mesh, without needing the source that generated it.

Every model is normalized into a fixed-size bounding cube before meshing. A leaf hexahedron's edge length therefore *is* its octree cell size, and its level follows from

$$
\text{level} = \operatorname{round}\!\left(\log_2\frac{100}{\bar{e}}\right), \qquad \bar{e} = \tfrac{1}{12}\sum_{k=1}^{12} \ell_k, \tag{3}
$$

the mean of a hexahedron's twelve edge lengths $\ell_k$. A single-edge proxy distorts badly once the quality-improvement stage has smoothed the mesh. The twelve-edge mean is the one metric found to reproduce every model's own reported level population exactly.

Histogramming every element's level by Equation 3 shows a clean population at a small number of levels, for almost every model. One wrinkle remains: the deepest level present is often not real refinement. It is instead a projection-stage smoothing artifact, where a shallower, near-degenerate hexahedron gets squashed until one of its edges reads as one level deeper. A small, sharply-dropping-off population at the deepest level is the signature of this artifact, not a genuine tier. This study confirmed the signature by rebuilding the octree with that level structurally unreachable, and finding that the same small population persists anyway. Reproducing Bunny's own reference mesh this way gives four real tiers, at levels 3 through 6 (maximum level 7). That matches Table 2's reported "Refinement Level 4" for that model exactly.

The full worked derivation is in [Reproducing Tong 2024](reproduction.md): the level-histogram method applied to every model checked, the artifact test, and the resulting per-model ladder settings.

[^meyer2003]: Meyer M, Desbrun M, Schröder P, Barr AH. Discrete Differential-Geometry Operators for Triangulated 2-Manifolds. In: Visualization and Mathematics III. Springer, 2003. A formula of this exact form (cotangent-weighted, Voronoi-area-normalized) is the standard discrete estimator for the mean curvature normal's magnitude in that paper and the literature descending from it.
[^cot_search]: `git log --all -p -- '*.c' '*.cpp' '*.h' '*.py'`, in this study's fork, searched against `cot|voronoi|laplace|beltrami` (case-insensitive) and separately against `\btan\(|atan|1\.0\s*/\s*tan|1/tan` and `mixed.?area|barycentric|dual.?area|vertex.?area`, run after fetching both the `hovey` fork and the `CMU-CBML` upstream remote to guarantee every reachable commit was included. Every match traces to a comment this study itself added to `HexGen.cpp`'s `Initialization.h`, documenting this same finding on 2026-08-05, not to any line the paper's authors wrote.
[^r_accum]: [`hexGen::ReadRawData()`, `HexGen.cpp`, line 882](https://github.com/hovey/HybridOctree_Hex/blob/c3069751423f15eb972836c326d1e71f0ae9e7ff/HybridOctree_Hex_v1.0/HexGen.cpp#L882). The accumulation happens at the line computing `triMesh.r[i] += (angle - PI) * (angle - PI);`, reached once per shared edge per vertex.
[^dup_tri]: Bottle1's input, `bottle1_tri.raw`, committed at `c291245` (2024-01-11), declares 29,665 triangles but contains only 29,664. A later commit, `f3247d8` (2024-01-16), corrects the header and parses cleanly. Full trace: [Finding 4, `analysis.md`](https://github.com/hovey/HybridOctree_Hex/blob/main/analysis.md#2026-08-17--git-archaeology-locating-the-exact-paper-era-reference-mesh-and-source-v10-reproduction-build-apple-m1), in this study's fork of the paper's repository.
[^raw_thickness]: Tested by building the source with the scaling factor disabled and comparing the resulting mesh size against the reference, on the same model. The unscaled version undershoots the target further than the scaled (default) version does.

---

Next: [Equilibration](equilibration.md).
