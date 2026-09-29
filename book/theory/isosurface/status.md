# Status

> **DRAFT.** Status as of 2026-09-29. Not yet reviewed.

`mesh tri` still builds its surface with cuberille.
`conspire` has merged Marching Cubes.
`conspire` has no Dual Contouring.
`automesh` exposes neither method.

## Marching Cubes

`conspire` PR #202 merged Marching Cubes on 2026-09-29.
The code ports the `marching_cubes` function of scikit-image, which implements the algorithm of Lewiner *et al.*[^Lewiner_2003]
That algorithm resolves the ambiguous cases of the classic Lorensen and Cline tables.
The result is a manifold, crack-free surface.

`MarchingCubes::extract` takes a `Voxels<f64>` field and returns an `Isosurface`.
`Tessellation::from(Isosurface)` converts that surface for STL output.

The port matches scikit-image on 429 recorded cases.
Face counts and vertex indices agree exactly.
Vertices, normals, and values agree to between 1e-6 and 1e-5, because scikit-image stores them as `float32`.

`conspire` `main` is version 0.7.8, which is unreleased.
`automesh` pins `conspire` at `=0.7.7`, so it cannot call `MarchingCubes` yet.

## Open items

| Item | State |
|---|---|
| Dual Contouring | No code in `conspire` `main`. |
| Segmentation to scalar field | `extract` takes a field. Nothing builds a field from a segmentation. |
| Multi-material interfaces | Out of scope for PR #202. A separate PR will address them. |
| `automesh` CLI | `mesh tri` has no `--method` flag. |
| Book pages | The [Marching Cubes](marching_cubes.md) and [Dual Contouring](dual_contouring.md) pages do not yet describe the `conspire` code. |

## Multi-material interfaces

Cuberille emits a face wherever two face-adjacent voxels differ in label.
Adjacent materials therefore share one conforming boundary.
A method that only finds the outer material and void boundary would regress this behavior.

Marching Cubes run once per material extracts each boundary independently.
Adjacent regions then get non-conforming surfaces at their shared interface.
PR #202 states that this conforms where two labels meet.
It also states that junctions where three or more labels meet are untested.

Wu and Sullivan[^Wu_Sullivan_2003] classify each cube by the set of labels that touch it.
Frisken[^Frisken_2022] extends the Dual Contouring family the same way.
Either would be the reference for a multi-label method.

## References

[^Lewiner_2003]: Lewiner T, Lopes H, Vieira AW, Tavares G. Efficient implementation of Marching Cubes' cases with topological guarantees. Journal of Graphics Tools. 2003;8(2):1-15. [link](http://thomas.lewiner.org/pdfs/marching_cubes_jgt.pdf)

[^Wu_Sullivan_2003]: Wu Z, Sullivan JM Jr. Multiple material marching cubes algorithm. International Journal for Numerical Methods in Engineering. 2003;58(2):189-207. [link](https://doi.org/10.1002/nme.775)

[^Frisken_2022]: Frisken SF. SurfaceNets for multi-label segmentations with preservation of sharp boundaries. Journal of Computer Graphics Techniques. 2022;11(1):34-54. [link](https://jcgt.org/published/0011/01/03/paper.pdf)
