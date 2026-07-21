r"""This module, sphere_hex_figures.py, renders the remeshed triangular
sphere surface and the all-hexahedral mesh produced from it by octree
dualization, used in the remeshed unit sphere mesh example.

Example
-------
source ~/autotwin/automesh/.venv/bin/activate
cd ~/autotwin/automesh/book/examples/mesh
automesh remesh -i ../remesh/sphere_radius_1.stl -o sphere_n10.stl uniform -n 10
automesh mesh hex -i sphere_n10.stl -o sphere_hex.inp --scale 8
python sphere_hex_figures.py

Output
------
The `sphere_tri.png` and `sphere_hex.png` visualization files, written next
to this script.
"""

import struct
from collections import Counter
from pathlib import Path
from typing import Final

import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d.art3d import Poly3DCollection
import numpy as np
from numpy.typing import NDArray
from PIL import Image

# Shared "hero" view so only the mesh changes between figures.
ELEV: Final[float] = 22.0
AZIM: Final[float] = -50.0
FACECOLOR: Final[str] = "lightblue"
EDGECOLOR: Final[str] = "navy"

# Local (0-indexed) face node order for an Abaqus C3D8 hexahedron.
C3D8_FACES: Final[tuple[tuple[int, int, int, int], ...]] = (
    (0, 1, 2, 3),
    (4, 5, 6, 7),
    (0, 1, 5, 4),
    (1, 2, 6, 5),
    (2, 3, 7, 6),
    (3, 0, 4, 7),
)


def read_stl(path: Path) -> NDArray[np.float64]:
    """Reads triangular facets from a binary STL file, shape (n_facets, 3, 3)."""
    data = path.read_bytes()
    (n_facets,) = struct.unpack_from("<I", data, 80)
    facets = np.empty((n_facets, 3, 3), dtype=np.float64)
    offset = 84
    for i in range(n_facets):
        values = struct.unpack_from("<12f", data, offset)
        facets[i] = np.array(values[3:12]).reshape(3, 3)
        offset += 50
    return facets


def read_inp_hex(
    path: Path,
) -> tuple[dict[int, tuple[float, float, float]], list[tuple[int, ...]]]:
    """Reads nodes and C3D8 element connectivity from an Abaqus .inp file."""
    nodes: dict[int, tuple[float, float, float]] = {}
    elements: list[tuple[int, ...]] = []
    section = None
    for line in path.read_text().splitlines():
        stripped = line.strip()
        if not stripped:
            continue
        if stripped.startswith("*"):
            lower = stripped.lower()
            section = (
                "node"
                if lower.startswith("*node")
                else "element"
                if lower.startswith("*element")
                else None
            )
            continue
        parts = [p.strip() for p in stripped.split(",")]
        if section == "node":
            nodes[int(parts[0])] = tuple(float(v) for v in parts[1:4])
        elif section == "element":
            ids = [int(p) for p in parts]
            elements.append(tuple(ids[1:9]))
    return nodes, elements


def exterior_quads(
    nodes: dict[int, tuple[float, float, float]],
    elements: list[tuple[int, ...]],
) -> list[NDArray[np.float64]]:
    """Returns the mesh's exterior quad faces, each as a (4, 3) coordinate
    array.  A face is exterior if it belongs to exactly one hex element."""
    face_count: Counter[frozenset[int]] = Counter()
    for elem in elements:
        for face in C3D8_FACES:
            face_count[frozenset(elem[i] for i in face)] += 1

    quads = []
    seen: set[frozenset[int]] = set()
    for elem in elements:
        for face in C3D8_FACES:
            node_ids = tuple(elem[i] for i in face)
            key = frozenset(node_ids)
            if face_count[key] == 1 and key not in seen:
                seen.add(key)
                quads.append(np.array([nodes[n] for n in node_ids]))
    return quads


def crop_to_content(path: Path, margin: int = 10) -> None:
    """Crops a saved PNG to its non-white content, since matplotlib's
    `bbox_inches="tight"` does not shrink the whitespace 3D axes leave
    around a plot even with the axes turned off."""
    image = Image.open(path).convert("RGB")
    array = np.asarray(image)
    non_white = np.any(array != 255, axis=-1)
    rows = np.where(non_white.any(axis=1))[0]
    cols = np.where(non_white.any(axis=0))[0]
    top = max(rows.min() - margin, 0)
    bottom = min(rows.max() + margin + 1, array.shape[0])
    left = max(cols.min() - margin, 0)
    right = min(cols.max() + margin + 1, array.shape[1])
    image.crop((left, top, right, bottom)).save(path)


def render_faces(
    faces: list[NDArray[np.float64]],
    bounds: tuple[NDArray[np.float64], NDArray[np.float64]],
    out_path: Path,
) -> None:
    """Renders a list of polygon faces (triangles or quads) as a shaded 3D
    surface, framed to the given (mins, maxs) bounds so companion figures
    share an identical scale."""
    mins, maxs = bounds
    fig = plt.figure(figsize=(6, 6))
    ax = fig.add_subplot(projection="3d")
    ax.add_collection3d(
        Poly3DCollection(faces, facecolor=FACECOLOR, edgecolor=EDGECOLOR, linewidths=0.4)
    )
    ax.set_xlim(mins[0], maxs[0])
    ax.set_ylim(mins[1], maxs[1])
    ax.set_zlim(mins[2], maxs[2])
    ax.set_box_aspect(tuple(maxs - mins))
    ax.view_init(elev=ELEV, azim=AZIM)
    ax.set_axis_off()
    fig.subplots_adjust(left=0, right=1, bottom=0, top=1)
    fig.savefig(out_path, dpi=150, bbox_inches="tight", pad_inches=0.02)
    plt.close(fig)
    crop_to_content(out_path)
    print(f"wrote {out_path} ({len(faces)} faces)")


if __name__ == "__main__":
    triangles = [f for f in read_stl(Path("sphere_n10.stl"))]

    nodes, elements = read_inp_hex(Path("sphere_hex.inp"))
    quads = exterior_quads(nodes, elements)

    # Share one reference bounding box (the union of both meshes' extents) so
    # the triangular surface and the hex mesh render at an identical scale.
    all_points = np.concatenate([np.concatenate(triangles), np.concatenate(quads)])
    bounds = (all_points.min(axis=0), all_points.max(axis=0))

    render_faces(triangles, bounds, Path("sphere_tri.png"))
    render_faces(quads, bounds, Path("sphere_hex.png"))
