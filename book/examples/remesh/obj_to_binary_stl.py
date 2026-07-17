r"""This module, obj_to_binary_stl.py, converts a triangular OBJ mesh into a
binary STL file.  `automesh remesh` reads binary STL (not OBJ), so the Stanford
bunny OBJ from Alec Jacobson's repository must be converted before use.

Example
-------
source ~/autotwin/automesh/.venv/bin/activate
cd ~/autotwin/automesh/book/examples/remesh
python obj_to_binary_stl.py stanford-bunny.obj stanford_bunny.stl
"""

import struct
import sys
from pathlib import Path

import numpy as np


def obj_to_binary_stl(source: Path, target: Path) -> None:
    """Reads a triangular OBJ and writes an equivalent binary STL."""
    verts: list[tuple[float, float, float]] = []
    faces: list[tuple[int, int, int]] = []
    for line in source.read_text().splitlines():
        tokens = line.split()
        if not tokens:
            continue
        if tokens[0] == "v":
            verts.append(tuple(float(x) for x in tokens[1:4]))
        elif tokens[0] == "f":
            # OBJ is 1-indexed; entries may be v, v/vt, or v/vt/vn. Fan-triangulate.
            idx = [int(p.split("/")[0]) - 1 for p in tokens[1:]]
            for k in range(1, len(idx) - 1):
                faces.append((idx[0], idx[k], idx[k + 1]))

    coords = np.array(verts, dtype=np.float64)
    with target.open("wb") as out:
        out.write(b"\0" * 80)  # 80-byte header (ignored)
        out.write(struct.pack("<I", len(faces)))  # facet count
        for a, b, c in faces:
            p, q, r = coords[a], coords[b], coords[c]
            normal = np.cross(q - p, r - p)
            length = np.linalg.norm(normal)
            normal = normal / length if length > 0 else normal
            out.write(struct.pack("<12fH", *normal, *p, *q, *r, 0))
    print(f"wrote {len(faces)} facets to {target}")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit("usage: python obj_to_binary_stl.py <mesh.obj> <mesh.stl>")
    obj_to_binary_stl(Path(sys.argv[1]), Path(sys.argv[2]))
