# Hausdorff

`hausdorff` measures how far the surface of a hexahedral or tetrahedral mesh lies from a tessellation (STL).
It writes a signed distance at every boundary node of the mesh and at every vertex of the tessellation.
The [Hausdorff Distance](../theory/hausdorff.md) theory page defines each quantity.

```sh
automesh hausdorff --help
<!-- cmdrun automesh hausdorff --help -->
```

## Example

The example uses the unit cube from the [Unit Cube Example](../theory/hausdorff.md#unit-cube-example).
The mesh [`hausdorff_cube_hex8_spike.inp`](hausdorff/hausdorff_cube_hex8_spike.inp) is a 2 × 2 × 2 grid of hexahedra.
Its top-center node sits at $z = 1.3$, which puts a spike of height 0.3 on the top face.
The tessellation [`hausdorff_cube.stl`](hausdorff/hausdorff_cube.stl) is the unit cube, with 12 triangles and no spike.

```sh
automesh hausdorff --mesh hausdorff/hausdorff_cube_hex8_spike.inp --tessellation hausdorff/hausdorff_cube.stl --output hausdorff/spike
<!-- cmdrun automesh hausdorff --mesh hausdorff/hausdorff_cube_hex8_spike.inp --tessellation hausdorff/hausdorff_cube.stl --output hausdorff/spike | ansifilter -->
```

The command writes `spike_A_to_B.csv` and `spike_B_to_A.csv`.
The mesh has 26 boundary nodes.
The spike tip is node 23 in the file and row 22 in `spike_A_to_B.csv`, because interior node 14 has no row.
It is the only node off the cube, at $s = +0.3$:

```sh
grep "1.3," hausdorff/spike_A_to_B.csv
<!-- cmdrun grep "1.3," hausdorff/spike_A_to_B.csv -->
```

Every vertex of the tessellation lies on the mesh surface, so each one has $s = 0$:

```sh
cat hausdorff/spike_B_to_A.csv
<!-- cmdrun cat hausdorff/spike_B_to_A.csv -->
```

## Output Files

`--output` takes a prefix, not a file name.
`automesh` drops any extension from the prefix, then appends `_A_to_B.csv` and `_B_to_A.csv`.
For example, `-o part`, `-o part.csv`, and `-o part.foo` all write `part_A_to_B.csv` and `part_B_to_A.csv`.
A directory in the prefix stays, so `-o out/part.csv` writes into `out/`.

The rule drops everything after the last dot in the file name.
A prefix such as `results.v2` therefore writes `results_A_to_B.csv`.

### Summary Lines

Each file starts with six comment lines, before the column header.
They repeat the [Console Summary](#console-summary) for that file's direction, so each file records its own result.

| Key | Value |
| --- | --- |
| `direction` | `A to B (mesh to tessellation)` or `B to A (tessellation to mesh)` |
| `samples` | the number of rows |
| `max \|s\|` | the largest $\lvert s \rvert$, which is $h_h$ for this direction |
| `mean \|s\|` | the mean of $\lvert s \rvert$ |
| `rms \|s\|` | the root mean square of $\lvert s \rvert$ |
| `H_h` | the sampled Hausdorff distance, the larger of the two `max \|s\|` values |

Each line has the form `# key = value`, and most lines add a description in parentheses after the value.
The value is the first word after the `=`.
For example, the line `# max |s| = 0.3 (max of the absolute value of signed distances)` has the value `0.3`.
Both files carry the same `H_h`.
$H_h$ is a lower bound on the continuous Hausdorff distance $H$, as [Discrete Setting](../theory/hausdorff.md#discrete-setting) explains.
`automesh` does not report $H$.

NumPy's `loadtxt` and `genfromtxt` skip lines that start with `#` by default.
pandas skips them only when told to:

```python
import pandas as pd

rows = pd.read_csv("part_A_to_B.csv", comment="#")
```

### Mesh to Tessellation

`PREFIX_A_to_B.csv` has one row per boundary node of the mesh.

`node,reference node,x,y,z,signed distance`

| Column | Meaning |
| --- | --- |
| `node` | 1-based row id |
| `reference node` | the node's number in the source mesh file |
| `x,y,z` | node coordinates |
| `signed distance` | $s_i$, the signed distance to the closest point on the tessellation |

`automesh` takes `reference node` from `conspire`'s `Mesh::node_labels()`.
An Exodus file supplies those labels from its `node_num_map`, and an Abaqus file supplies them from its `*Node` labels.
A file with no node labels, such as Medit or VTU, gets the 1-based node index instead.
The coordinates let you plot the file without the mesh.

### Tessellation to Mesh

`PREFIX_B_to_A.csv` has one row per vertex of the tessellation.

`vertex,x,y,z,signed distance`

| Column | Meaning |
| --- | --- |
| `vertex` | 1-based row id |
| `x,y,z` | vertex coordinates |
| `signed distance` | $s_j$, the signed distance to the closest point on the mesh surface |

The STL reader merges vertices with identical coordinates, so each row is one distinct point.
For a hexahedral mesh, `automesh` splits each boundary quadrilateral along its shorter diagonal before it measures.
[Surface Triangulation](../theory/hausdorff.md#surface-triangulation) gives the rule.

## Sign Convention

A positive signed distance means the sample lies outside the other surface.
A negative one means it lies inside.
The sign comes from the angle-weighted pseudonormal at the closest point, which stays correct at sharp edges and vertices.
See [Sign Convention](../theory/hausdorff.md#sign-convention) for the definition.

## Console Summary

For each direction, `automesh` prints the maximum, mean, and root mean square of $\lvert s \rvert$.
It also prints the sampled Hausdorff distance $H_h$, the larger of the two maxima.
The [Summary Lines](#summary-lines) at the top of each output file hold the same values.
The [Example](#example) shows the summary for the spike case.
There, $h_h(\mathcal{A} \rightarrow \mathcal{B}) = 0.3$, $h_h(\mathcal{B} \rightarrow \mathcal{A}) = 0$, and $H_h = 0.3$.

If any boundary element has $\hat{J}_{\min} < 0$, `automesh` also prints a warning with the count of those elements.

## Input Requirements

* `--mesh` accepts `exo`, `inp`, `mesh`, and `vtu` files that hold hexahedra or tetrahedra. It rejects `off` and `stl`, because those hold triangles only.
* `--tessellation` accepts `stl` files only.
* Each surface must be closed, consistently wound, and outward. The command checks the tessellation and the triangulated boundary of the mesh, and stops with an error if either fails. [Surface Checks](../theory/hausdorff.md#surface-checks) defines each check.

An inverted element on the boundary of the mesh leads to one of three outcomes:

| Case | Outcome |
| --- | --- |
| Mirrored mesh: every element in left-handed order | rejected, because the boundary points inward |
| Mirrored boundary elements: some elements in left-handed order | rejected, because the boundary is wound inconsistently |
| Tangled boundary elements: $\hat{J}_{\min} < 0$ at some corners | accepted, with a count of the tangled boundary elements; signs near a fold are suspect |

[Inverted Elements](../theory/hausdorff.md#inverted-elements) explains each outcome.
An inverted element in the interior of the mesh does not affect either file.

## Frame of Reference

The tessellation must be in the same frame as the mesh.
`automesh hausdorff` does not transform either input and does not check that the frames agree.
It reports distances in the units of the two files.

The `--xscale`, `--xtranslate`, and other transform options of [`automesh mesh`](mesh.md) change the output mesh only.
Compare a mesh made with them against an STL transformed the same way.
