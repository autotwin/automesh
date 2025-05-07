# Getting Started

In this section, we use a simple segmentation to create a finite element mesh, a
smoothed finite element mesh, and an isosurface.

## Segmentation Input

We start with a segmentation of a regular octahedron composed of three materials.
The segmentation encodes

* `0` for void (or background), shown in gray,
* `1` for the inner domain, shown in green,
* `2` for the intermediate layer, shown in yellow, and
* `3` for the outer layer, shown in magenta.

The (`7 x 7 x 7`) segmentation, at the midline cut plane,
appears as follows:

<style>
    .container {
        display: flex; /* Use flexbox layout */
    }
    .grid {
        display: grid;
        grid-template-columns: repeat(7, 50px);
        grid-template-rows: repeat(7, 50px);
        gap: 1px;
    }
    .gridito {
        display: grid;
        grid-template-columns: repeat(7, 20px);
        grid-template-rows: repeat(7, 20px);
        gap: 1px;
    }
    .cell {
        display: flex;
        justify-content: center;
        align-items: center;
        font-size: 24px;
        color: white;
    }
    .zero {
        /* background-color: gray; */
        background-color: rgb(128, 128, 128);
    }
    .one {
        /* background-color: green; */
        background-color: rgb(0, 255, 0); /* RGB value for green */
        color: black;  /* text color */
    }
    .two {
        /* background-color: yellow; */
        background-color: rgb(255, 255, 0);
        color: black;  /* text color */
    }
    .three {
        /* background-color: magenta; */
        background-color: rgb(255, 0, 255);
    }
</style>

<div class="grid">
    <!--row 1-->
    <div class="cell zero">0</div>
    <div class="cell zero">0</div>
    <div class="cell zero">0</div>
    <div class="cell three">3</div>
    <div class="cell zero">0</div>
    <div class="cell zero">0</div>
    <div class="cell zero">0</div>
    <!--row 2-->
    <div class="cell zero">0</div>
    <div class="cell zero">0</div>
    <div class="cell three">3</div>
    <div class="cell two">2</div>
    <div class="cell three">3</div>
    <div class="cell zero">0</div>
    <div class="cell zero">0</div>
    <!--row 3-->
    <div class="cell zero">0</div>
    <div class="cell three">3</div>
    <div class="cell two">2</div>
    <div class="cell one">1</div>
    <div class="cell two">2</div>
    <div class="cell three">3</div>
    <div class="cell zero">0</div>
    <!--row 4-->
    <div class="cell three">3</div>
    <div class="cell two">2</div>
    <div class="cell one">1</div>
    <div class="cell one">1</div>
    <div class="cell one">1</div>
    <div class="cell two">2</div>
    <div class="cell three">3</div>
    <!--row 5-->
    <div class="cell zero">0</div>
    <div class="cell three">3</div>
    <div class="cell two">2</div>
    <div class="cell one">1</div>
    <div class="cell two">2</div>
    <div class="cell three">3</div>
    <div class="cell zero">0</div>
    <!--row 6-->
    <div class="cell zero">0</div>
    <div class="cell zero">0</div>
    <div class="cell three">3</div>
    <div class="cell two">2</div>
    <div class="cell three">3</div>
    <div class="cell zero">0</div>
    <div class="cell zero">0</div>
    <!--row 7-->
    <div class="cell zero">0</div>
    <div class="cell zero">0</div>
    <div class="cell zero">0</div>
    <div class="cell three">3</div>
    <div class="cell zero">0</div>
    <div class="cell zero">0</div>
    <div class="cell zero">0</div>
</div>

Consider each slice, `1` to `7`, in succession:

<div class="container">
    <!--slice 1-->
    1&nbsp;<div class="gridito">
        <!--row 1-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 2-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 3-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 4-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell three"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 5-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 6-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 7-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
    </div>
    &nbsp;
    &nbsp;
    &nbsp;
    <!--slice 2-->
    2&nbsp;<div class="gridito">
        <!--row 1-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 2-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 3-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell three"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 4-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell three"></div>
        <div class="cell two"></div>
        <div class="cell three"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 5-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell three"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 6-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 7-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
    </div>
    &nbsp;
    &nbsp;
    &nbsp;
    <!--slice 3-->
    3&nbsp;<div class="gridito">
        <!--row 1-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 2-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell three"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 3-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell three"></div>
        <div class="cell two"></div>
        <div class="cell three"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 4-->
        <div class="cell zero"></div>
        <div class="cell three"></div>
        <div class="cell two"></div>
        <div class="cell one"></div>
        <div class="cell two"></div>
        <div class="cell three"></div>
        <div class="cell zero"></div>
        <!--row 5-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell three"></div>
        <div class="cell two"></div>
        <div class="cell three"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 6-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell three"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 7-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
    </div>
    &nbsp;
    &nbsp;
    &nbsp;
    <!--slice 4-->
    4&nbsp;<div class="gridito">
        <!--row 1-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell three"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 2-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell three"></div>
        <div class="cell two"></div>
        <div class="cell three"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 3-->
        <div class="cell zero"></div>
        <div class="cell three"></div>
        <div class="cell two"></div>
        <div class="cell one"></div>
        <div class="cell two"></div>
        <div class="cell three"></div>
        <div class="cell zero"></div>
        <!--row 4-->
        <div class="cell three"></div>
        <div class="cell two"></div>
        <div class="cell one"></div>
        <div class="cell one"></div>
        <div class="cell one"></div>
        <div class="cell two"></div>
        <div class="cell three"></div>
        <!--row 5-->
        <div class="cell zero"></div>
        <div class="cell three"></div>
        <div class="cell two"></div>
        <div class="cell one"></div>
        <div class="cell two"></div>
        <div class="cell three"></div>
        <div class="cell zero"></div>
        <!--row 6-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell three"></div>
        <div class="cell two"></div>
        <div class="cell three"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 7-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell three"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
    </div>
</div>
&nbsp;
<div class="container">
    <!--slice 5-->
    5&nbsp;<div class="gridito">
        <!--row 1-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 2-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell three"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 3-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell three"></div>
        <div class="cell two"></div>
        <div class="cell three"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 4-->
        <div class="cell zero"></div>
        <div class="cell three"></div>
        <div class="cell two"></div>
        <div class="cell one"></div>
        <div class="cell two"></div>
        <div class="cell three"></div>
        <div class="cell zero"></div>
        <!--row 5-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell three"></div>
        <div class="cell two"></div>
        <div class="cell three"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 6-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell three"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 7-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
    </div>
    &nbsp;
    &nbsp;
    &nbsp;
    <!--slice 6-->
    6&nbsp;<div class="gridito">
        <!--row 1-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 2-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 3-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell three"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 4-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell three"></div>
        <div class="cell two"></div>
        <div class="cell three"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 5-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell three"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 6-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 7-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
    </div>
    &nbsp;
    &nbsp;
    &nbsp;
    <!--slice 7-->
    7&nbsp;<div class="gridito">
        <!--row 1-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 2-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 3-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 4-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell three"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 5-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 6-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <!--row 7-->
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
        <div class="cell zero"></div>
    </div>
    &nbsp;
    &nbsp;
    &nbsp;
</div>

> Remark: The (`7 x 7 x 7`) segmentation can be thought of as a conceptual start point
for a process called
[Loop subdivision](https://en.wikipedia.org/wiki/Loop_subdivision_surface),
used to produce spherical shapes at higher resolutions.
See [Octa Loop](https://github.com/autotwin/mesh/blob/main/doc/octa_loop.md) for additional information.
A sphere in resolutions of (`24 x 24 x 24`) and (`48 x 48 x 48`), used
in the [Sphere with Shells](https://autotwin.github.io/automesh/analysis/sphere_with_shells/index.html) section,
is shown below: ![spheres_cont_cut](analysis/sphere_with_shells/img/spheres_cont_cut.png)

## Segmentation File Types

Two types of segmentation files types are supported: `.spn` and `.npy`.

The `.spn` file can be thought of as the most elementary segmentation file type because it is
saved as an ASCII text file and is therefore readily human-readable.
Below is an abbreviated and commented `.spn` segmentation of the (`7 x 7 x 7`) octahedron
discussed above.

```sh
0 # slice 1, row 1
0
0
0
0
0
0
0 # slice 1, row 2
0
0
0
0
0
0
0 # slice 1, row 3
0
0
0
0
0
0
0 # slice 1, row 4
0
0
3
0
0
0
0 # slice 1, row 5
0
0
0
0
0
0
0 # slice 1, row 6
0
0
0
0
0
0
0 # slice 1, row 7
0
0
0
0
0
0
# ... and so on for the remaining six slices
```

A disadvantage of `.spn` is that it can become difficult to keep track of data
slice-by-slice.  Because it is not a compressed binary file, the `.spn` has a
larger file size than the equivalent `.npy`.

The `.npy` segmentation file format is an alternative to the `.spn`
format.  The `.npy` format can be advantageous because is can be generated easily
from Python.  This approach can be useful because Python can be used to
algorithmically create a segmentation and serialized the segmentation to a compressed
binary file in `.npy` format.

We illustrate creating the octahedron segmentation in Python:

```python
<!-- cmdrun cat octahedron.py -->
```

## The `convert` Command

`automesh` allows for interoperability between `.spn`. and `.npy` file types.
Use the `automesh` help to discover the command syntax:

```sh
automesh convert --help
<!-- cmdrun automesh convert --help -->
```

For example, to convert the `octahedron.npy` to `octahedron2.spn`:

```sh
<!-- cmdrun python octahedron.py > /dev/null -->
automesh convert segmentation -i octahedron.npy -o octahedron2.spn
<!-- cmdrun automesh convert segmentation -i octahedron.npy -o octahedron2.spn | ansifilter -->
```

To convert from `octahedron2.spn` to `octahedron3.npy`:

```sh
automesh convert segmentation -i octahedron2.spn -x 7 -y 7 -z 7 -o octahedron3.npy
<!-- cmdrun automesh convert segmentation -i octahedron2.spn -x 7 -y 7 -z 7 -o octahedron3.npy | ansifilter -->
```

> Remark: Notice that the `.spn` requires number of voxels in each of the x, y, and z dimensions to be specified using `--nelx`, `--nely`, `--nelz` (or, equivalently `-x`, `-y`, `-z`) flags.

We can verify the two `.npy` files encode the same segmentation:

```python
<!-- cmdrun cat octahedron_roundtrip.py -->
```

## Mesh Generation

`automesh` creates several finite element mesh file types from
a segmentation.

Use the `automesh` help to discover the command syntax:

```sh
automesh mesh --help
<!-- cmdrun automesh mesh --help -->
```

To convert the `octahedron.npy` into an ABAQUS finite element mesh, while removing
segmentation `0` from the mesh:

```sh
automesh mesh hex -r 0 -i octahedron.npy -o octahedron.inp
<!-- cmdrun automesh mesh hex -r 0 -i octahedron.npy -o octahedron.inp | ansifilter -->
```

## Smoothing

Use the `automesh` help to discover the command syntax:

```sh
automesh smooth --help
<!-- cmdrun automesh smooth --help -->
```

To smooth the `octahedron.inp` mesh with Taubin smoothing parameters for five
iterations:

```sh
automesh smooth hex -n 5 -i octahedron.inp -o octahedron_s05.inp
<!-- cmdrun automesh smooth hex -n 5 -i octahedron.inp -o octahedron_s05.inp | ansifilter -->
```

The original voxel mesh and the smoothed voxel mesh are shown below:

`octahedron.inp` | `octahedron_s05.inp`
:---: | :---:
![octahedron_voxels](fig/octahedron_voxels.png) | ![octahedron_voxels_s05](fig/octahedron_voxels_s05.png)

See the [Smoothing](smoothing/README.md) section for more information.

## Isosurface

An isosurface can be generated from a segmentation using the `tri` command.

To create a mesh of the outer isosurfaces contained in the `octahedron` example:

```sh
automesh mesh tri -r 0 1 2 -i octahedron.npy -o octahedron.stl
<!-- cmdrun automesh mesh tri -r 0 1 2 -i octahedron.npy -o octahedron.stl | ansifilter -->
```

The surfaces are visualized below:

`octahedron.stl` in MeshLab | `octahedron.stl` in Cubit with cut plane
:---: | :---:
![isosurface_mesh_lab](fig/isosurface_mesh_lab.png) | ![isosurface_cubit_cut_plane](fig/isosurface_cubit_cut_plane.png)

`automesh` creates an isosurface from the boundary faces of voxels.  The
quadrilateral faces are divided into two triangles.  The [Isosurface](https://autotwin.github.io/automesh/isosurface/index.html) section contains more details about alternative methods used to create an isosurface.

The [Sphere with Shells](https://autotwin.github.io/automesh/analysis/sphere_with_shells/index.html) section contains more examples of the command line interface.
