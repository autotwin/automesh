# Mesh

`mesh` creates a finite element mesh from a segmentation.  Its subcommands
create all-hexahedral (`hex`), hex-dominant (`hexdom`), polyhedral (`poly`),
all-tetrahedral (`tet`), and all-triangular (`tri`) meshes.

`mesh hex` produces an all-hexahedral (voxel) mesh.  Its input is either a
segmentation, meshed directly into hexahedra, or a tessellation, converted
into hexahedra by octree dualization.  For a tessellation, `--uniform
<SPACING>` replaces the octree with a uniform lattice of cubes of the given
edge length, trimmed to the surface and buffered onto it exactly as the dual
is; the background is then ungraded, so `--scale`, `--tolerance`, `--strong`
and `--levels` no longer apply.  An optional `smooth` subcommand can
be chained directly onto `mesh hex`.  A further `remesh` subcommand can also
be chained after `smooth` — `automesh mesh hex smooth remesh --help`
succeeds, so the command line accepts it — but running it always fails.
`remesh` requires triangular connectivity, and a hex mesh has none, so the
run-time error is always `connectivity contains a non-triangular block`.

`mesh tet` produces an all-tetrahedral mesh from a tessellation.  Every
element is a linear, four-noded tetrahedron.  Ten-noded quadratic
tetrahedra are not supported.  The input
must be an `stl` file.  A segmentation input is rejected.  The command builds
a tetrahedral background, six tetrahedra per cell of an octree fitted to the
surface.  `--uniform <SPACING>` replaces the octree with a uniform lattice.
The command then trims the background to the surface and buffers it onto the
surface with a prism layer and a fit.  An optional `smooth` subcommand can be
chained directly onto `mesh tet`.  A further `remesh` subcommand parses but
always fails, with `connectivity contains a non-triangular block`, for the
same reason as `mesh hex smooth remesh`.

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

## Mesh Tet

```sh
automesh mesh tet --help
<!-- cmdrun automesh mesh tet --help -->
```

`mesh tet` shares its options with `mesh hex`, so `--help` lists options that
apply to other subcommands.  Passing `--marching` without `--uniform` fails at
the command line.  Passing `--levels` fails with
`Dualization requires 2:1 balancing, so levels applies to mesh poly only.`

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

## Mesh Tet Smooth

```sh
automesh mesh tet smooth --help
<!-- cmdrun automesh mesh tet smooth --help -->
```

`mesh tet smooth` accepts a further `remesh` subcommand at the command line,
but running it always fails.  `remesh` requires triangular connectivity.

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
