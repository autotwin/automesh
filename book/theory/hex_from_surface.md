# Hexahedral Meshing from a Surface

Given a **closed, manifold, triangular surface** — an `stl` tessellation, for example — `automesh` produces an **all-hexahedral** volume mesh of the region the surface encloses.  This page describes the algorithm end to end.

```sh
automesh mesh hex --input surface.stl --output mesh.exo --scale 8
```

The method is a *dual* method: Rather than fitting hexahedra to the surface directly, it builds an octree over the enclosed volume and constructs the **dual** of that octree.  The dual of a balanced octree is all-hexahedral by construction, which guarantees the output mesh to consist of only hexahedral elements.

### Two Meshes, Two Vocabularies

Because a dual method involves two distinct meshes at once, it is worth fixing terminology before describing the algorithm.  Throughout this page:

* A **cell** is a box of the *octree* — the primal structure.  A **leaf cell** is one that has not been subdivided further.  Cells are what the octree refines, balances, and pairs.
* A **hexahedron**, or **element**, belongs to the *dual mesh* — the output.  Hexahedra are what the finished mesh is made of, and what quality metrics are computed on.

The two are related by the dual correspondence, which inverts dimension:

| Octree (primal) | Dual mesh (output) |
| --- | --- |
| leaf cell | node, at the cell center |
| vertex where cells meet | hexahedron, joining those cells' center nodes |

So a dual **node** sits at the center of each octree leaf cell, and a dual **hexahedron** is formed around each octree **vertex**, joining the centers of the eight leaf cells meeting there.  Where the octree is uniform, this is exactly eight equal cells meeting at a corner and the resulting hexahedron is a perfect cube.  Where the octree changes level, fewer or unequal cells meet, and a **template** supplies the connectivity instead — the subject of [Dualization](hex_from_surface/dualization.md).

The pipeline has five stages, and **stage 3 is the pivot between the two vocabularies**: everything upstream of it operates on octree *cells*, and everything downstream operates on dual *hexahedra*.

| Stage | | Operates on |
| --- | --- | --- |
| 1 | [**Octree construction**](hex_from_surface/octree_construction.md) from the shape diameter function | cells |
| 2 | [**Equilibration**](hex_from_surface/equilibration.md), which balances and pairs the octree | cells |
| 3 | [**Dualization**](hex_from_surface/dualization.md), which converts cells into hexahedra via templates | cells → hexahedra |
| 4 | [**Trimming**](hex_from_surface/trimming.md), which discards hexahedra lying outside the surface | hexahedra |
| 5 | [**Buffering**](hex_from_surface/buffering.md), which projects the boundary onto the surface | hexahedra |

Stage 3 consumes the octree and emits the dual mesh.  **The octree plays no further role once dualization is complete** — trimming and buffering never consult it, and never subdivide, coarsen, or otherwise revisit a cell.  Whatever the templates produced is what the remaining stages must work with, which is why the interior quality bound established at stage 3 survives to the finished mesh.

Cutting the pipeline the other way: stages 1–3 produce the interior of the mesh and are purely combinatorial, so their quality is bounded in advance (see [Template Quality](#template-quality)).  Stages 4–5 fit that interior to the actual geometry, and are the only stages that can degrade quality.

> **The input surface is not validated.**  `automesh` performs no up-front check for holes, non-manifold edges, or inconsistent orientation.  A defective surface will either produce a poor mesh or fail late, during buffering, with a `non-manifold boundary` error.  Verify the surface before meshing.

> **Consider `--strong` for quality-critical work.**  The balancing rule chosen during equilibration determines the guaranteed interior mesh quality.  The default is *weak* balancing; passing `--strong` raises the guaranteed minimum scaled Jacobian of every interior element from $\approx 0.246$ to $\approx 0.258$, at the cost of a larger mesh.  See [Balancing](hex_from_surface/equilibration.md#balancing).

## Template Quality

Because the interior is assembled exclusively from a finite catalog of templates, its quality is **bounded below by construction**, before any smoothing is applied.  The catalogs below were measured directly from the dualization stage.

Under **strong balancing**, the catalog admits ten distinct values of the minimum scaled Jacobian:

| Minimum scaled Jacobian | Closed form |
| --- | --- |
| 0.258199 | $1/\sqrt{15}$ |
| 0.402015 | $4/\sqrt{99}$ |
| 0.438562 | |
| 0.447214 | $1/\sqrt{5}$ |
| 0.548202 | |
| 0.577350 | $1/\sqrt{3}$ |
| 0.727273 | $8/11$ |
| 0.894427 | $2/\sqrt{5}$ |
| 0.904534 | $3/\sqrt{11}$ |
| 1.000000 | cube |

The interior quality lower bound under strong balancing is therefore

$$\min \text{MSJ} \;\geq\; \frac{1}{\sqrt{15}} \approx 0.258.$$

Under **weak balancing** — the default in `automesh` — two additional configurations become reachable, both below $1/\sqrt{15}$:

| Minimum scaled Jacobian | Closed form |
| --- | --- |
| 0.246183 | $\sqrt{2/33}$ |
| 0.257130 | $2\sqrt{2}/11$ |

relaxing the lower bound to

$$\min \text{MSJ} \;\geq\; \sqrt{\tfrac{2}{33}} \approx 0.246.$$

These two configurations arise only on topologically complex inputs.  Simple convex shapes such as a sphere attain $1/\sqrt{15}$ under either balancing rule, so the difference between the two bounds becomes visible only on demanding geometry.

The difference has a structural explanation.  Weak balancing admits a [fifth edge template](hex_from_surface/dualization.md) that strong balancing rules out; the four-template set under `--strong` is exactly the set illustrated in [Dualization](hex_from_surface/dualization.md).  The measured catalogs and the template inventory agree: the extra template, and the two extra quality values, appear together.

Two practical consequences follow.

**Any element below the lower bound came from the boundary.**  Trimming and buffering are the only stages that deform geometry, so sub-bound quality in a finished mesh is always attributable to them.  The interior is never the culprit, and no amount of octree refinement will fix a boundary problem.

**The bound is a choice, not a constant.**  Passing [`--strong`](hex_from_surface/equilibration.md#balancing) removes the two sub-$1/\sqrt{15}$ configurations from the catalog outright.  This is the only control `automesh` currently offers over interior quality — nothing downstream optimizes it — so it is worth setting deliberately rather than accepting the default.

## Quality Assessment and Improvement

Element quality is reported with `--metrics`, which writes maximum edge ratio, minimum scaled Jacobian, maximum skew, and element volume per element:

```sh
automesh mesh hex --input surface.stl --output mesh.exo --scale 8 --metrics quality.csv
```

Quality can be improved after meshing with Taubin or Laplace smoothing:

```sh
automesh mesh hex --input surface.stl --output mesh.exo --scale 8 smooth --method Taubin
```

Taubin smoothing is preferred over Laplace, which shrinks the mesh; see [Smoothing](smoothing.md).

Optimization-based quality improvement — in particular BFGS-driven untangling and quality maximization, as described by Protais et al.[^Protais_2026] — is the intended direction for `automesh` but is **not yet implemented**.  At present, no stage of the pipeline optimizes element quality; the interior relies on the template bound, and the boundary on smoothing.

## Choosing a Scale

Quality does **not** improve monotonically with `--scale`.  Raising it refines the octree, which improves the resolution of the surface, but also produces more transition regions and more buffer elements — and past a point the latter dominates.

For the remeshed unit sphere [`sphere_n10.stl`](../examples/mesh/sphere_n10.stl), sweeping `--scale` gives:

| `--scale` | Elements | Minimum scaled Jacobian |
| --- | --- | --- |
| 3 *(default)* | 7 | 0.531 |
| 4 | 37 | 0.215 |
| 5 | 73 | 0.171 |
| 6 | 183 | 0.058 |
| 7 | 235 | 0.058 |
| 8 | 393 | **0.280** |
| 9 | 551 | 0.009 |
| 10 | 804 | −0.165 |

Scale 8 is the sweet spot here; scale 10 produces **inverted** elements.  The practical guidance is to **sweep `--scale` and inspect `--metrics`** rather than assuming that a larger value is better.  The optimum is model-specific.

Note what the default produces: at `--scale 3`, this sphere yields **7 elements** — enough to confirm the pipeline runs, but far too coarse to be a usable mesh.  The default is a conservative starting point, not a recommendation.  Expect to raise it for any real model.

`--scale` is a floating-point argument, so intermediate values such as `--scale 7.5` are permitted; the sweep above uses integers only for legibility.

## Failure Modes

**`non-manifold boundary`.**  Raised during buffering.  Frequent causes:

* **Smoothed input tessellations.**  Dualizing a Taubin-smoothed surface reliably triggers this error.  Smooth the *hexahedral mesh* after meshing, not the surface before it.
* **Surfaces with holes, or self-intersections.**  The Stanford bunny [`stanford_bunny.stl`](../examples/remesh/stanford_bunny.stl) fails at every scale for this reason, even though its octree dualizes without complaint.

**Degenerate or inverted elements.**  A minimum scaled Jacobian at or below zero indicates elements produced by the buffer layer that could not be fitted.  Causes:

* **Thin shells.**  The method fills an enclosed *volume*.  A one-element-thick shell has almost no interior, so nearly every element is a buffer element and the template bound does not apply.
* **Excessive `--scale`**, as shown above.

**A far denser mesh than expected.**  Refinement is driven by *local* thickness, so an unintentionally thin feature — a sliver, a near-degenerate facet, a self-intersection that reads as thin — is resolved at $s$ cells across its own small thickness, and the balancing rules then propagate some of that refinement into its neighborhood.  Inspect the model for unintended thin features, or coarsen `--scale`.

## References

[^Protais_2026]: Protais F, Cherchi G, Livesu M. Versatile Volume Fitting with Automatic Feature Preservation. 2026.  HAL open archive, inria.hal.science.  [paper](https://inria.hal.science/hal-05574616/)

* [SIBL — quadtree](https://github.com/sandialabs/sibl/blob/master/geo/doc/quadtree.md)
* [SIBL — dual quad transitions](https://github.com/sandialabs/sibl/blob/master/geo/doc/dual_quad_transitions.md)
* [SIBL — dual lesson 11](https://github.com/sandialabs/sibl/blob/master/geo/doc/dual/lesson_11.md)

See also [Hexahedral Metrics](metrics_hexahedral.md) for the definition of the minimum scaled Jacobian.
