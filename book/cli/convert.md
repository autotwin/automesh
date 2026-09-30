# Convert

`convert` translates between file formats without changing the underlying
data: `convert mesh` translates between mesh formats (`.exo`, `.inp`, `.mesh`,
`.off`, `.stl`, `.vtu`), and `convert segmentation` translates between
segmentation formats (`.npy`, `.spn`, `.vti`).

```sh
automesh convert --help
<!-- cmdrun automesh convert --help -->
```

## Convert Mesh

`convert mesh` automatically detects the element type(s) present in the
input file — hexahedral, tetrahedral, triangular, quadrilateral, wedge,
pyramidal, or a mix of these within the same mesh — and writes them to the
output format unchanged; there is no separate hex/tet/tri subcommand to
choose.

**`.stl` and `.off` are the exceptions:** both are surface-only formats.
`.stl` holds only 3D triangles. `.off` holds 3D triangles and
quadrilaterals.

- An `.stl` or `.off` input can be converted to any of the other mesh
  formats (`.exo`, `.inp`, `.mesh`, `.vtu`); the resulting mesh is
  composed of whatever surface elements the input held (triangles only,
  for `.stl`; triangles and/or quadrilaterals, for `.off`).
- Any of the other mesh formats can be converted to `.stl`, provided the
  input mesh is itself composed exclusively of triangular elements, or to
  `.off`, provided the input mesh is composed exclusively of triangular
  and/or quadrilateral elements.
- A volumetric mesh (containing hexahedral, tetrahedral, wedge, or
  pyramidal elements) cannot be converted to `.stl` or `.off`.

```sh
automesh convert mesh --help
<!-- cmdrun automesh convert mesh --help -->
```

## Convert Segmentation

```sh
automesh convert segmentation --help
<!-- cmdrun automesh convert segmentation --help -->
```
