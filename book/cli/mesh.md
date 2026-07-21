# Mesh

`mesh` creates a finite element mesh from a segmentation.

`mesh hex` produces an all-hexahedral (voxel) mesh.  Its input is either a
segmentation, meshed directly into hexahedra, or a tessellation, converted
into hexahedra by octree dualization.  An optional `smooth` subcommand can
be chained directly onto `mesh hex`.  A further `remesh` subcommand can also
be chained after `smooth` — `automesh mesh hex smooth remesh --help`
succeeds, so the command line accepts it — but running it always fails.
`remesh` requires triangular connectivity, and a hex mesh has none, so the
run-time error is always `connectivity contains a non-triangular block`.

`mesh tri` produces an all-triangular isosurface mesh of the material
boundaries from a segmentation.  An optional `smooth` subcommand can be
chained directly onto it, and a further `remesh` subcommand can be chained
after that — `mesh tri smooth remesh` works fully, since a triangular mesh
satisfies `remesh`'s connectivity requirement.

```sh
automesh mesh --help
<!-- cmdrun automesh mesh --help -->
```

## Mesh Hex

```sh
automesh mesh hex --help
<!-- cmdrun automesh mesh hex --help -->
```

## Mesh Tri

```sh
automesh mesh tri --help
<!-- cmdrun automesh mesh tri --help -->
```

## Mesh Hex Smooth

```sh
automesh mesh hex smooth --help
<!-- cmdrun automesh mesh hex smooth --help -->
```

`mesh hex smooth` accepts a further `remesh` subcommand at the command line
(`automesh mesh hex smooth remesh --help` succeeds), but running it always
fails — `remesh` requires triangular connectivity, and a hex mesh has none.
Remeshing after smoothing is only meaningful for `mesh tri`, below.

## Mesh Tri Smooth

```sh
automesh mesh tri smooth --help
<!-- cmdrun automesh mesh tri smooth --help -->
```

## Mesh Tri Smooth Remesh

```sh
automesh mesh tri smooth remesh --help
<!-- cmdrun automesh mesh tri smooth remesh --help -->
```

## Examples

* [Torus](../examples/mesh/torus.md) — a genus-1 solid meshed and smoothed in
  a single chained `mesh hex smooth` command, comparing raw vs. smoothed
  element quality, and reproducing the `mesh hex smooth remesh` failure
  documented above.
* [Remeshed unit sphere](../examples/mesh/remeshed_sphere.md) — `mesh hex`
  dualizing a triangular **surface** into a solid all-hexahedral **volume**,
  and how the `--scale` octree depth affects element quality.
* [Unit sphere](../examples/remesh/sphere.md) and the
  [Stanford bunny](../examples/remesh/bunny.md) — `mesh tri smooth remesh`
  worked in full, as part of the [Remesh](remesh.md) examples.
