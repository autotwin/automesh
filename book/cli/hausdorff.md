# Hausdorff

`hausdorff` measures how far the surface of a hexahedral or tetrahedral mesh lies from a tessellation (STL).
It writes a signed distance at every boundary node of the mesh and at every vertex of the tessellation.
The [Hausdorff Distance](../theory/hausdorff.md) theory page defines each quantity, and its [Unit Cube Example](../theory/hausdorff.md#unit-cube-example) shows a run with both output files.

```sh
automesh hausdorff --help
<!-- cmdrun automesh hausdorff --help -->
```

`--mesh` must hold only hexahedra or only tetrahedra, and `--tessellation` must hold triangles only.

## Output Files

`--output` takes a prefix.
`automesh` drops any extension from it and appends `_A_to_B.csv` and `_B_to_A.csv`, so `-o part.csv` writes `part_A_to_B.csv` and `part_B_to_A.csv`.

`PREFIX_A_to_B.csv` has one row per boundary node of the mesh:

| Column | Meaning |
| --- | --- |
| `node` | 1-based row id |
| `reference node` | the node's label in the mesh file, from `Mesh::node_labels()`, or the 1-based node index if the file has none |
| `x,y,z` | node coordinates |
| `signed distance` | the signed distance to the tessellation |

`PREFIX_B_to_A.csv` has one row per vertex that a triangle of the tessellation uses:

| Column | Meaning |
| --- | --- |
| `vertex` | 1-based row id |
| `x,y,z` | vertex coordinates |
| `signed distance` | the signed distance to the mesh surface |

Each file starts with six `# key = value` lines that repeat the console summary for its direction: `direction`, `samples`, `max |s|`, `mean |s|`, `rms |s|`, and `H_h`.
The value is the first word after the `=`.
NumPy's `loadtxt` skips these lines by default, and pandas skips them with `pd.read_csv(file, comment="#")`.
