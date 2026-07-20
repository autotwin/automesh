# Convert

`convert` translates between file formats without changing the underlying
data: `convert mesh` translates between mesh formats (`.exo`, `.inp`, `.mesh`,
`.stl`, `.vtu`), and `convert segmentation` translates between segmentation
formats (`.npy`, `.spn`, `.vti`).

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

**`.stl` is the exception:** `.stl` is a triangulated-surface format only,
so every element it reads or writes is a 3D triangle.

- An `.stl` input can be converted to any of the other mesh formats
  (`.exo`, `.inp`, `.mesh`, `.vtu`); the resulting mesh is composed
  exclusively of triangular elements.
- Any of the other mesh formats can be converted to `.stl`, provided the
  input mesh is itself composed exclusively of triangular elements; the
  resulting `.stl` is then, likewise, composed solely of triangles.
- A volumetric mesh (containing hexahedral, tetrahedral, wedge, or
  pyramidal elements) cannot be converted to `.stl`.

```sh
automesh convert mesh --help
<!-- cmdrun automesh convert mesh --help -->
```

## Convert Segmentation

```sh
automesh convert segmentation --help
<!-- cmdrun automesh convert segmentation --help -->
```
