"""Hand an automesh mesh to the optimizer of the reference, and take it back.

The reference, `HybridOctree_Hex` v1.0, can resume at its projection step from
a file `dualHex.vtk`: an interior hexahedral mesh without a buffer layer.  The
step adds its own buffer layer and runs its ratchet.

`core` strips the buffer layer of an `automesh` mesh, the elements with a face
on the boundary, and writes the rest in the format of `dualHex.vtk`.

`project` moves every boundary node of a mesh the reference wrote onto the
tessellation, to its closest point, and writes a `.vtu` file for
`automesh improve`.

Usage
-----
    uv run --with numpy --with meshio python bone_tong_swap.py core \\
        adaptive_snap.vtu dualHex.vtk
    uv run --with numpy --with meshio --with trimesh --with rtree --with scipy \\
        python bone_tong_swap.py project bestQualityMesh.vtk bone_frame.stl \\
        adaptive_swapped.vtu
"""

import sys
from collections import Counter

import meshio
import numpy as np

FACES = [(0, 3, 2, 1), (4, 5, 6, 7), (0, 1, 5, 4),
         (1, 2, 6, 5), (2, 3, 7, 6), (3, 0, 4, 7)]


def boundary_faces(cells):
    count = Counter(tuple(sorted(h[list(f)])) for h in cells for f in FACES)
    return count


def core(mesh, out):
    m = meshio.read(mesh)
    cells = m.cells_dict["hexahedron"]
    count = boundary_faces(cells)
    outer = np.array([any(count[tuple(sorted(h[list(f)]))] == 1 for f in FACES)
                      for h in cells])
    kept = cells[~outer]
    used = np.unique(kept)
    renumber = -np.ones(len(m.points), int)
    renumber[used] = np.arange(len(used))
    kept = renumber[kept]
    points = m.points[used]
    with open(out, "w") as f:
        f.write("# vtk DataFile Version 2.0\nHybrid Octree Mesh\nASCII\n"
                "DATASET UNSTRUCTURED_GRID\n")
        f.write(f"POINTS {len(points)} double\n")
        for x in points:
            f.write(f"{x[0]:.6f} {x[1]:.6f} {x[2]:.6f}\n")
        f.write(f"CELLS {len(kept)} {9 * len(kept)}\n")
        for h in kept:
            f.write("8 " + " ".join(map(str, h)) + "\n")
        f.write(f"CELL_TYPES {len(kept)}\n" + "12\n" * len(kept))
    print(f"{len(cells)} elements, {outer.sum()} in the buffer layer, "
          f"{len(kept)} in the core, written to {out}")


def project(mesh, tessellation, out):
    import trimesh

    m = meshio.read(mesh)
    cells = m.cells_dict["hexahedron"]
    points = m.points.copy()
    count = boundary_faces(cells)
    nodes = np.unique([n for face, k in count.items() if k == 1 for n in face])
    closest, distance, _ = trimesh.proximity.closest_point(
        trimesh.load(tessellation), points[nodes])
    points[nodes] = closest
    meshio.write(out, meshio.Mesh(points, [("hexahedron", cells)]))
    print(f"{len(nodes)} boundary nodes moved, by at most {distance.max():.3f}, "
          f"written to {out}")


if __name__ == "__main__":
    if sys.argv[1] == "core":
        core(*sys.argv[2:4])
    else:
        project(*sys.argv[2:5])
