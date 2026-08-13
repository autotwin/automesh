# Introduction

> **DRAFT.** Written to scope work on the `surfacing` branch. Not yet
> reviewed.

Isosurfacing is a method to extract the surface from a three-dimensional
scalar field. A scalar field $\phi = \phi(x, y, z): \mathbb{R}^3 \mapsto
\mathbb{R}$ assigns a scalar value to every point in three-dimensional
space. When every point in the domain sits on a regular (i.e., uniform)
three-dimensional grid, the scalar field is a *voxel* field.

The simplest non-trivial voxel field takes only two integer values, `0`
and `1`. A `0` marks a point outside the field. A `1` marks a point inside
it, or on its boundary. The interface between `0` and `1` everywhere in the
grid is the field's **isosurface**.

Real segmentations are rarely binary. A material segmentation typically
assigns a distinct positive integer label to each material, with `0`
reserved for void. The isosurface generalizes directly: it is the
interface wherever *any* two labels differ, not only where a label meets
void. `automesh` already works this way — [`mesh tri`](../../cli/mesh.md)
produces isosurfaces of the **material boundaries**, plural, and the
cuberille implementation below emits a face wherever two face-adjacent
voxels' labels differ, whichever two labels they are. Any isosurfacing
method `automesh` adds needs to preserve that: a method that only finds
the outer material/void boundary would be a regression, not an upgrade.
See [Implementation Plan](implementation_plan.md#phase-2--marching-cubes)
for how Marching Cubes and Dual Contouring each need to handle this.

## Three approaches, not two

Marching Cubes and Dual Contouring are the two isosurfacing algorithms
most often cited in the literature, covered in detail two pages from here.
But a third, older approach exists, and it is the one `automesh` uses
today.

### Cuberille: what `mesh tri` actually does

`automesh mesh tri` builds its output surface through conspire's
`Tessellation::from<Voxels>` (`geometry/mesh/tessellation/from/grid/mod.rs`
in `conspire.rs`). The algorithm walks the grid once and, for every pair of
face-adjacent voxels whose labels differ, emits a quad face exactly on the
voxel boundary between them — no interpolation, no gradient information.
Each quad splits into two triangles, coincident vertices weld together, and
any resulting non-manifold "pinch" vertex is duplicated to restore a clean
2-manifold. The result is the exact voxel-boundary surface: axis-aligned,
stair-stepped, and topologically faithful to the segmentation it came from.

This is the **cuberille** method, first described by Herman and
Liu[^Herman_1979] in 1979 for reconstructing 3D organ surfaces from CT
slices. It predates both Marching Cubes (1987) and Dual Contouring (2002).

### Why it matters for `mesh tri`

Cuberille's stair-stepping is why `mesh tri` output is typically smoothed
afterward — `mesh tri smooth` chains Laplace or Taubin smoothing directly
onto it, and `mesh tri smooth remesh` can follow that with remeshing. The
voxel-boundary surface is a faithful but blocky starting point; smoothing,
not the isosurfacing step itself, is what currently produces a
visually smooth result.

Marching Cubes and Dual Contouring take the opposite approach: both
interpolate a smoother surface *during* extraction, at the cost of losing
the guarantee that every output vertex sits exactly on a voxel boundary.
[Marching Cubes](marching_cubes.md) interpolates along voxel edges. [Dual
Contouring](dual_contouring.md) places a vertex inside each voxel, using
gradient information to better preserve sharp features. Neither is
implemented in `automesh` or `conspire` today; they are documented here as
the standard alternatives against which the `surfacing` branch's work is
evaluated. See the [Implementation Plan](implementation_plan.md) for how
that work is scoped.

## References

[^Herman_1979]: Herman GT, Liu HK. Three-dimensional display of human organs from computed tomograms. Computer Graphics and Image Processing. 1979;9(1):1-21. [link](https://doi.org/10.1016/0146-664X(79)90079-0)
