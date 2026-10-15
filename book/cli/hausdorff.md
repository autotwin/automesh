# Hausdorff

`hausdorff` measures how far the surface of a hexahedral or tetrahedral mesh lies from a tessellation (STL).
It writes a signed distance at every boundary node of the mesh and at every vertex of the tessellation.
The [Hausdorff Distance](../theory/hausdorff.md) theory page defines each quantity.

```sh
automesh hausdorff --help
<!-- cmdrun automesh hausdorff --help -->
```

## Example

<!-- TODO: replace with a cmdrun example on a committed fixture once Part 1 lands. -->

```sh
automesh hausdorff --mesh sphere.exo --tessellation sphere.stl --output sphere
```

This command writes `sphere_A_to_B.csv` and `sphere_B_to_A.csv`.

## Output Files

`--output` takes a prefix, not a file name.
`automesh` drops any extension from the prefix, then appends `_A_to_B.csv` and `_B_to_A.csv`.
For example, `-o part`, `-o part.csv`, and `-o part.foo` all write `part_A_to_B.csv` and `part_B_to_A.csv`.
A directory in the prefix stays, so `-o out/part.csv` writes into `out/`.

The rule drops everything after the last dot in the file name.
A prefix such as `results.v2` therefore writes `results_A_to_B.csv`.

### Mesh to Tessellation

`PREFIX_A_to_B.csv` has one row per boundary node of the mesh.

`node,reference node,x,y,z,signed distance`

| Column | Meaning |
| --- | --- |
| `node` | 1-based row id |
| `reference node` | the node's number in the source mesh file |
| `x,y,z` | node coordinates |
| `signed distance` | $s_i$, the signed distance to the closest point on the tessellation |

An Exodus file supplies `reference node` from its `node_num_map`, and an Abaqus file supplies it from its node labels.
A file with no node numbering gets the 1-based node index instead.
The coordinates let you plot the file without the mesh.

<!-- TODO: confirm the reference node source once mrbuche/conspire.rs#255 is released. -->

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
See [Sign Convention](../theory/hausdorff.md#sign-convention) for the definition and its known limit near sharp edges.

## Console Summary

For each direction, `automesh` prints the maximum, mean, and root mean square of $\lvert s \rvert$.
It also prints the sampled Hausdorff distance $H_h$, the larger of the two maxima.

<!-- TODO: paste captured output once implemented. -->

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
