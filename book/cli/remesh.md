# Remesh

`remesh` applies *isotropic surface remeshing* to an existing triangular
surface mesh.  Starting from the input triangulation, it iteratively splits,
collapses, flips, and smooths edges to drive every edge toward a target edge
length.  The result is a surface mesh with more uniform, better-quality
triangles, either coarsened or refined relative to the input.

```sh
automesh remesh --help
<!-- cmdrun automesh remesh --help -->
```

Remeshing reads and writes surface (triangular) mesh formats; see the `--input`
and `--output` formats listed in the help above.  STL files **must be binary
STL** for both input and output — ASCII STL is not accepted.  The worked examples
below include short scripts to convert ASCII STL or OBJ meshes to binary STL.

`remesh` runs in one of two sizing modes, `uniform` and `adaptive`.

## Remesh Uniform

A single target edge length is applied over the whole mesh; use it to coarsen or
refine a surface to a chosen resolution.

```sh
automesh remesh uniform --help
<!-- cmdrun automesh remesh uniform --help -->
```

- `--iterations <NUM>` — number of remeshing passes (default: 5).  More passes
  bring the mesh closer to the target edge length.
- `--size <SIZE>` — the target edge length.  When omitted, the mean edge length
  of the input mesh is used, which regularizes the mesh without significantly
  changing its resolution.

## Remesh Adaptive

The target edge length varies with local surface curvature, between a `--minimum`
and `--maximum`, so curved regions are refined and flat regions are coarsened.

```sh
automesh remesh adaptive --help
<!-- cmdrun automesh remesh adaptive --help -->
```

- `--iterations <NUM>` — number of remeshing passes (default: 5).
- `--minimum <MIN>` — minimum edge length, used in high-curvature regions
  (required).
- `--maximum <MAX>` — maximum edge length, used in flat regions (required).
- `--tolerance <TOL>` — curvature tolerance controlling how strongly curvature
  drives the local edge length (default: 0.1).
- `--gradation <GRAD>` — size gradation factor controlling how smoothly the edge
  length transitions between the minimum and maximum (default: 0.5).

## Examples

Two worked examples apply these options and illustrate the results:

* [Unit sphere](../examples/remesh/sphere.md) — an analytic unit sphere: mesh
  statistics, closed-surface relationships, uniform vs. adaptive sizing, and
  the effect of the number of iterations.
* [Stanford bunny](../examples/remesh/bunny.md) — a real scanned surface with
  varying curvature, where uniform and adaptive sizing differ visibly, with a
  walkthrough of every `remesh` parameter.
