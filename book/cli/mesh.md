# Mesh

`mesh` creates a finite element mesh from a segmentation (or, for `hex`,
from a tessellation): `mesh hex` produces an all-hexahedral (voxel) mesh, and
`mesh tri` produces an all-triangular isosurface mesh of the material
boundaries.  Optional `smooth` (and, for triangular meshes, `remesh`)
subcommands can be chained directly onto the meshing step.

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
