# Unit Tests

The following illustates a subset of the unit tests used to validate
the code implementation.
For a complete listing of the unit sets, see
[voxels.rs](https://github.com/autotwin/automesh/blob/main/tests/voxel.rs)
and [voxel.py](https://github.com/autotwin/automesh/blob/main/tests/voxel.py).

The Python code used to generate the figures is included [below](#source).

**Remark:** We use the convention `np` when importing `numpy` as follows:

```python
import numpy as np
```

### Single

The minimum working example (MWE) is a single voxel, used to create a single
mesh consisting of one block consisting of a single element.  The NumPy input
[single.npy](https://github.com/autotwin/automesh/raw/main/tests/input/single.npy)
contains the following segmentation:

```python
segmentation = np.array(
    [
        [
            [ 11, ],
        ],
    ],
    dtype=np.uint8,
)
```

where the segmentation `11` denotes block `11` in the finite element mesh.

**Remark:** Serialization (write and read)

| Write | Read |
|----------|----------|
| Use the [np.save](https://numpy.org/doc/stable/reference/generated/numpy.save.html) command to serialize the segmentation a `.npy` file  | Use the [np.load](https://numpy.org/doc/stable/reference/generated/numpy.load.html) command to deserialize the segmentation from a `.npy` file  |
*Example: Write the data in `segmentation` to a file called `seg.npy`*</br></br>`np.save("seg.npy", segmentation)` | *Example: Read the data from the file `seg.npy` to a variable called `loaded_array`*</br></br>`loaded_array = np.load("seg.npy")`

Equivalently, the [single.spn](https://raw.githubusercontent.com/autotwin/automesh/main/tests/input/single.spn) contains a
single integer:

```sh
11   #  x:1  y:1  z:1
```

The resulting finite element mesh is visualized is shown in the following
figure:

![single.png](single.png)

Figure: The `single.png` visualization, (left) lattice node numbers, (right)
mesh node numbers.  Lattice node numbers appear in gray, with `(x, y, z)`
indices in parenthesis.  The right-hand rule is used.  Lattice coordinates
start at `(0, 0, 0)`, and proceed along the `x-axis`, then
the `y-axis`, and then the `z-axis`.

The finite element mesh local node numbering map to the following global node
numbers identically,
$S_{\rm{local}} = \{1, 2, 4, 3, 5, 6, 8, 7\}$ and $S_{\rm{local}} = S_{\rm{global}}$:

```bash
[1, 2, 4, 3, 5, 6, 8, 7]
->
[1, 2, 4, 3, 5, 6, 8, 7]
```

which is a special case not typically observed, as shown in more complex
examples below.

**Remark:** Input `.npy` and `.spn` files for the examples below can be found on the repository at [automesh/tests/input](https://github.com/autotwin/automesh/tree/main/tests/input).

## Double

The next level of complexity example is a two-voxel domain, used to create
a single block composed of two finite elements.  We test propagation in
both the `x` and `y` directions.  The figures below show these two
meshes.

### Double X

```sh
11   #  x:1  y:1  z:1
11   #    2    1    1
```

where the segmentation `11` denotes block `11` in the finite element mesh.

![double_x.png](double_x.png)

Figure: Mesh composed of a single block with two elements, propagating along
the `x-axis`, (left) lattice node numbers, (right) mesh node numbers.

### Double Y

```sh
11   #  x:1  y:1  z:1
11   #    1    2    1
```

where the segmentation `11` denotes block `11` in the finite element mesh.

![double_y.png](double_y.png)

Figure: Mesh composed of a single block with two elements, propagating along
the `y-axis`, (left) lattice node numbers, (right) mesh node numbers.

## Triple

```sh
11   #  x:1  y:1  z:1
11   #    2    1    1
11   #    3    1    1
```

where the segmentation `11` denotes block `11` in the finite element mesh.

![triple_x.png](triple_x.png)

Figure: Mesh composed of a single block with three elements, propagating along
the `x-axis`, (left) lattice node numbers, (right) mesh node numbers.

## Quadruple

```sh
11   #  x:1  y:1  z:1
11   #    2    1    1
11   #    3    1    1
11   #    4    1    1
```

where the segmentation `11` denotes block `11` in the finite element mesh.

![quadruple_x.png](quadruple_x.png)

Figure: Mesh composed of a single block with four elements, propagating along
the `x-axis`, (left) lattice node numbers, (right) mesh node numbers.

## Quadruple with Voids

```sh
99   #  x:1  y:1  z:1
0    #    2    1    1
0    #    3    1    1
99   #    4    1    1
```

where the segmentation `99` denotes block `99` in the finite element mesh, and segmentation `0` is excluded from the mesh.

![quadruple_2_voids_x.png](quadruple_2_voids_x.png)

Figure: Mesh composed of a single block with two elements, propagating along
the `x-axis` and two voids, (left) lattice node numbers, (right) mesh node
numbers.

## Quadruple with Two Blocks

```sh
100  #  x:1  y:1  z:1
101  #    2    1    1
101  #    3    1    1
100  #    4    1    1
```

where the segmentation `100` and `101` denotes block `100` and `101`,
respectively in the finite element mesh.

![quadruple_2_blocks.png](quadruple_2_blocks.png)

Figure: Mesh composed of two blocks with two elements elements each,
propagating along the `x-axis`, (left) lattice node numbers, (right) mesh
node numbers.

## Quadruple with Two Blocks and Void

```sh
102  #  x:1  y:1  z:1
103  #    2    1    1
0    #    3    1    1
102  #    4    1    1
```

where the segmentation `102` and `103` denotes block `102` and `103`,
respectively, in the finite element mesh, and segmentation `0` will be included from the finite element mesh.

![quadruple_2_blocks_void.png](quadruple_2_blocks_void.png)

Figure: Mesh composed of one block with two elements, a second block with one
element, and a void, propagating along the `x-axis`, (left) lattice node
numbers, (right) mesh node numbers.

## Cube

```sh
11   #  x:1  y:1  z:1
11   #  _ 2  _ 1    1
11   #    1    2    1
11   #  _ 2  _ 2  _ 1
11   #    1    1    2
11   #  _ 2  _ 1    2
11   #    1    2    2
11   #  _ 2  _ 2  _ 2
```

where the segmentation `11` denotes block `11` in the finite element mesh.

![cube.png](cube.png)

Figure: Mesh composed of one block with eight elements, (left) lattice node
numbers, (right) mesh node numbers.

## Cube with Multi Blocks and Void

```sh
82   #  x:1  y:1  z:1
2    #  _ 2  _ 1    1
2    #    1    2    1
2    #  _ 2  _ 2  _ 1
0    #    1    1    2
31   #  _ 2  _ 1    2
0    #    1    2    2
44   #  _ 2  _ 2  _ 2
```

where the segmentation `82`, `2`, `31` and `44` denotes block `82`, `2`, `31`
and `44`, respectively, in the finite element mesh, and segmentation `0` will
be included from the finite element mesh.

![cube_multi.png](cube_multi.png)

Figure: Mesh composed of four blocks (block `82` has one element, block `2`
has three elements, block `31` has one element, and block `44` has one
element), (left) lattice node numbers, (right) mesh node numbers.

## Cube with Inclusion

```sh
11   #  x:1  y:1  z:1
11   #    2    1    1
11   #  _ 3  _ 1    1
11   #    1    2    1
11   #    2    2    1
11   #  _ 3  _ 2    1
11   #    1    3    1
11   #    2    3    1
11   #  _ 3  _ 3  _ 1
11   #    1    1    2
11   #    2    1    2
11   #  _ 3  _ 1    2
11   #    1    2    2
88   #    2    2    2
11   #  _ 3  _ 2    2
11   #    1    3    2
11   #    2    3    2
11   #  _ 3  _ 3  _ 2
11   #    1    1    3
11   #    2    1    3
11   #  _ 3  _ 1    3
11   #    1    2    3
11   #    2    2    3
11   #  _ 3  _ 2    3
11   #    1    3    3
11   #    2    3    3
11   #  _ 3  _ 3  _ 3
````

![cube_with_inclusion.png](cube_with_inclusion.png)

Figure: Mesh composed of 26 voxels of (block `11`) and one voxel inslusion
(block `88`), (left) lattice node numbers, (right) mesh node numbers.

## Bracket

```sh
1   #  x:1  y:1  z:1
1   #    2    1    1
1   #    3    1    1
1   #  _ 4  _ 1    1
1   #  x:1  y:2  z:1
1   #    2    2    1
1   #    3    2    1
1   #  _ 4  _ 2    1
1   #  x:1  y:3  z:1
1   #    2    3    1
0   #    3    3    1
0   #  _ 4  _ 3    1
1   #  x:1  y:4  z:1
1   #    2    4    1
0   #    3    4    1
0   #  _ 4  _ 4    1
```

where the segmentation `1` denotes block `1` in the finite element mesh,
and segmentation `0` is excluded from the mesh.

![bracket.png](bracket.png)

Figure: Mesh composed of a L-shaped bracket in the `xy` plane.

## Letter F

```sh
11   #  x:1  y:1  z:1
0    #    2    1    1
0    #  _ 3  _ 1    1
11   #    1    2    1
0    #    2    2    1
0    #  _ 3  _ 2    1
11   #    1    3    1
11   #    2    3    1
0    #  _ 3  _ 3    1
11   #    1    4    1
0    #    2    4    1
0    #  _ 3  _ 4    1
11   #    1    5    1
11   #    2    5    1
11   #  _ 3  _ 5  _ 1
```

where the segmentation `11` denotes block `11` in the finite element mesh.

![letter_f.png](letter_f.png)

Figure: Mesh composed of a single block with eight elements, (left) lattice
node numbers, (right) mesh node numbers.

## Letter F in 3D

```sh
1    #  x:1  y:1  z:1
1    #    2    1    1
1    #    3    1    1
1    #  _ 4  _ 1    1
1    #    1    2    1
1    #    2    2    1
1    #    3    2    1
1    #  _ 4  _ 2    1
1    #    1    3    1
1    #    2    3    1
1    #    3    3    1
1    #  _ 4  _ 3    1
1    #    1    4    1
1    #    2    4    1
1    #    3    4    1
1    #  _ 4  _ 4    1
1    #    1    5    1
1    #    2    5    1
1    #    3    5    1
1    #  _ 4  _ 5  _ 1
1    #  x:1  y:1  z:2
0    #    2    1    2
0    #    3    1    2
0    #  _ 4  _ 1    2
1    #    1    2    2
0    #    2    2    2
0    #    3    2    2
0    #  _ 4  _ 2    2
1    #    1    3    2
1    #    2    3    2
1    #    3    3    2
1    #  _ 4  _ 3    2
1    #    1    4    2
0    #    2    4    2
0    #    3    4    2
0    #  _ 4  _ 4    2
1    #    1    5    2
1    #    2    5    2
1    #    3    5    2
1    #  _ 4  _ 5  _ 2
1    #  x:1  y:1  z:3
0    #    2    1    j
0    #    3    1    2
0    #  _ 4  _ 1    2
1    #    1    2    3
0    #    2    2    3
0    #    3    2    3
0    #  _ 4  _ 2    3
1    #    1    3    3
0    #    2    3    3
0    #    3    3    3
0    #  _ 4  _ 3    3
1    #    1    4    3
0    #    2    4    3
0    #    3    4    3
0    #  _ 4  _ 4    3
1    #    1    5    3
1    #    2    5    3
1    #    3    5    3
1    #  _ 4  _ 5  _ 3
```

which corresponds to `--nelx 4`, `--nely 5`, and `--nelz  3` in the
[command line interface](cli.md).

![letter_f_3d.png](letter_f_3d.png)

Figure: Mesh composed of a single block with thirty-nine elements, (left)
lattice node numbers, (right) mesh node numbers.

The shape of the solid segmentation is more easily seen without the
lattice and element nodes, and with decreased opacity, as shown below:

![letter_f_3d_alt.png](letter_f_3d_alt.png)

Figure: Mesh composed of a single block with thirty-nine elements, shown
with decreased opacity and without lattice and element node numbers.

## Sparse

```sh
0    #  x:1  y:1  z:1
0    #    2    1    1
0    #    3    1    1
0    #    4    1    1
2    #  _ 5  _ 1    1
0    #    1    2    1
1    #    2    2    1
0    #    3    2    1
0    #    4    2    1
2    #  _ 5  _ 2    1
1    #    1    3    1
2    #    2    3    1
0    #    3    3    1
2    #    4    3    1
0    #  _ 5  _ 3    1
0    #    1    4    1
1    #    2    4    1
0    #    3    4    1
2    #    4    4    1
0    #  _ 5  _ 4    1
1    #    1    5    1
0    #    2    5    1
0    #    3    5    1
0    #    4    5    1
1    #  _ 5  _ 5  _ 1
2    #  x:1  y:1  z:2
0    #    2    1    2
2    #    3    1    2
0    #    4    1    2
0    #  _ 5  _ 1    2
1    #    1    2    2
1    #    2    2    2
0    #    3    2    2
2    #    4    2    2
2    #  _ 5  _ 2    2
2    #    1    3    2
0    #    2    3    2
0    #    3    3    2
0    #    4    3    2
0    #  _ 5  _ 3    2
1    #    1    4    2
0    #    2    4    2
0    #    3    4    2
2    #    4    4    2
0    #  _ 5  _ 4    2
2    #    1    5    2
0    #    2    5    2
2    #    3    5    2
0    #    4    5    2
2    #  _ 5  _ 5  _ 2
0    #  x:1  y:1  z:3
0    #    2    1    3
1    #    3    1    3
0    #    4    1    3
2    #  _ 5  _ 1    3
0    #    1    2    3
0    #    2    2    3
0    #    3    2    3
1    #    4    2    3
2    #  _ 5  _ 2    3
0    #    1    3    3
0    #    2    3    3
2    #    3    3    3
2    #    4    3    3
2    #  _ 5  _ 3    3
0    #    1    4    3
0    #    2    4    3
1    #    3    4    3
0    #    4    4    3
1    #  _ 5  _ 4    3
0    #    1    5    3
1    #    2    5    3
0    #    3    5    3
1    #    4    5    3
0    #  _ 5  _ 5  _ 3
0    #  x:1  y:1  z:4
1    #    2    1    4
2    #    3    1    4
1    #    4    1    4
2    #  _ 5  _ 1    4
2    #    1    2    4
0    #    2    2    4
2    #    3    2    4
0    #    4    2    4
1    #  _ 5  _ 2    4
1    #    1    3    4
2    #    2    3    4
2    #    3    3    4
0    #    4    3    4
0    #  _ 5  _ 3    4
2    #    1    4    4
1    #    2    4    4
1    #    3    4    4
1    #    4    4    4
1    #  _ 5  _ 4    4
0    #    1    5    4
0    #    2    5    4
1    #    3    5    4
0    #    4    5    4
0    #  _ 5  _ 5  _ 4
0    #  x:1  y:1  z:5
1    #    2    1    5
0    #    3    1    5
2    #    4    1    5
0    #  _ 5  _ 1    5
1    #    1    2    5
0    #    2    2    5
0    #    3    2    5
0    #    4    2    5
2    #  _ 5  _ 2    5
0    #    1    3    5
1    #    2    3    5
0    #    3    3    5
0    #    4    3    5
0    #  _ 5  _ 3    5
1    #    1    4    5
0    #    2    4    5
0    #    3    4    5
0    #    4    4    5
0    #  _ 5  _ 4    5
0    #    1    5    5
0    #    2    5    5
1    #    3    5    5
2    #    4    5    5
1    #  _ 5  _ 5  _ 5
```

where the segmentation `1` denotes block `1` and segmentation `2` denotes block `2` in the finite eelement mesh (with segmentation `0` excluded).

![sparse.png](sparse.png)

Figure: Sparse mesh composed of two materials at random voxel locations.

![sparse_alt.png](sparse_alt.png)

Figure: Sparse mesh composed of two materials at random voxel locations, shown with decreased opactity and without lattice and element node numbers.

## Source

The figures were created with the following Python files:

* [examples_data.py](examples_data.py)
* [examples_figures.py](examples_figures.py)
* [examples_test.py](examples_test.py)
* [examples_types.py](examples_types.py)

### `examples_data.py`

```python
<!-- cmdrun cat examples_data.py -->
```

### `examples_figures.py`

```python
<!-- cmdrun cat examples_figures.py -->
```

### `examples_test.py`

```python
<!-- cmdrun cat examples_test.py -->
```

### `examples_types.py`

```python
<!-- cmdrun cat examples_types.py -->
```
