r"""This module, remesh_figures.py, renders the surface triangulations used in
the Remesh section of the documentation.  It reads the input sphere and each
remeshed output (produced by `automesh remesh`) and saves a matched-camera PNG
of every mesh, so the effect of uniform and adaptive sizing can be compared
side by side.

Example
-------
source ~/autotwin/automesh/.venv/bin/activate
cd ~/autotwin/automesh/book/cli
# regenerate the remeshed STL files (binary STL) if needed:
#   automesh remesh -i sphere_radius_1.stl -o sphere_uniform_coarse.stl uniform -s 0.35
#   automesh remesh -i sphere_radius_1.stl -o sphere_uniform_fine.stl   uniform -s 0.08
#   automesh remesh -i sphere_radius_1.stl -o sphere_uniform.stl        uniform -s 0.18
#   automesh remesh -i sphere_radius_1.stl -o sphere_adaptive.stl       adaptive --minimum 0.05 --maximum 0.30
python remesh_figures.py

Output
------
The `*.png` visualization files, one per mesh, written next to this script.
"""

# standard library
import struct
from pathlib import Path
from typing import Final

# third-party library
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d.art3d import Poly3DCollection
import numpy as np
from numpy.typing import NDArray

# Shared view so triangle-size differences (not camera changes) are what the
# reader sees between figures.
ELEV: Final[float] = 20.0
AZIM: Final[float] = -60.0
LIMIT: Final[float] = 1.05  # sphere radius ~1, small margin
FACECOLOR: Final[str] = "lightblue"
EDGECOLOR: Final[str] = "navy"


def read_stl(path: Path) -> NDArray[np.float64]:
    """Reads triangular facets from an STL file, returning an array of shape
    (n_facets, 3, 3).  Binary and ASCII STL are both supported; the format is
    detected by sniffing the leading bytes."""
    data = path.read_bytes()
    is_ascii = data[:6].lower().startswith(b"solid") and b"facet" in data[:512]
    if is_ascii:
        return _read_ascii(data.decode("ascii", errors="replace"))
    return _read_binary(data)


def _read_binary(data: bytes) -> NDArray[np.float64]:
    """Reads a binary STL (80-byte header, uint32 count, 50 bytes per facet)."""
    (n_facets,) = struct.unpack_from("<I", data, 80)
    facets = np.empty((n_facets, 3, 3), dtype=np.float64)
    offset = 84
    for i in range(n_facets):
        # 12 floats: normal (3) + three vertices (9); trailing uint16 attribute.
        values = struct.unpack_from("<12f", data, offset)
        facets[i] = np.array(values[3:12]).reshape(3, 3)
        offset += 50
    return facets


def _read_ascii(text: str) -> NDArray[np.float64]:
    """Reads an ASCII STL, collecting every `vertex` triple into facets."""
    verts = []
    for line in text.splitlines():
        tokens = line.split()
        if tokens and tokens[0] == "vertex":
            verts.append([float(v) for v in tokens[1:4]])
    return np.array(verts, dtype=np.float64).reshape(-1, 3, 3)


def topology(facets: NDArray[np.float64]) -> tuple[int, int, int]:
    """Returns (faces, edges, vertices) for a triangular surface mesh, where
    coincident vertices are merged and each undirected edge is counted once."""
    faces = len(facets)
    keyed = np.round(facets.reshape(-1, 3), 6)
    _, inverse = np.unique(keyed, axis=0, return_inverse=True)
    ids = inverse.reshape(faces, 3)
    edges = set()
    for a, b, c in ids:
        for u, v in ((a, b), (b, c), (c, a)):
            edges.add((int(min(u, v)), int(max(u, v))))
    vertices = int(ids.max()) + 1
    return faces, len(edges), vertices


def edge_lengths(facets: NDArray[np.float64]) -> NDArray[np.float64]:
    """Returns the length of every unique undirected edge in the mesh."""
    keyed = np.round(facets.reshape(-1, 3), 6)
    coords, inverse = np.unique(keyed, axis=0, return_inverse=True)
    ids = inverse.reshape(len(facets), 3)
    seen = set()
    lengths = []
    for tri, (a, b, c) in zip(facets, ids):
        for (u, v), (p, q) in (((a, b), (0, 1)), ((b, c), (1, 2)), ((c, a), (2, 0))):
            key = (int(min(u, v)), int(max(u, v)))
            if key not in seen:
                seen.add(key)
                lengths.append(float(np.linalg.norm(tri[p] - tri[q])))
    return np.array(lengths)


def render_histogram(stl: Path, out_name: str) -> None:
    """Saves a histogram of the triangle edge lengths of the given mesh."""
    lengths = edge_lengths(read_stl(stl))
    fig, ax = plt.subplots(figsize=(6, 4))
    ax.hist(lengths, bins=30, color=FACECOLOR, edgecolor=EDGECOLOR)
    ax.axvline(
        lengths.mean(),
        color="crimson",
        linestyle="--",
        linewidth=1.5,
        label=f"mean = {lengths.mean():.3f}",
    )
    ax.set_xlabel("triangle edge length")
    ax.set_ylabel("count")
    ax.set_title(f"{stl.stem}: edge length distribution")
    ax.legend()
    png = stl.with_name(out_name)
    fig.savefig(png, dpi=150, bbox_inches="tight")
    plt.close(fig)
    print(f"wrote {png.name} ({len(lengths)} edges)")


def render(stl: Path, title: str) -> None:
    """Renders a single STL surface triangulation to a PNG next to it."""
    facets = read_stl(stl)
    fig = plt.figure(figsize=(6, 6))
    ax = fig.add_subplot(111, projection="3d")
    surface = Poly3DCollection(
        facets, facecolor=FACECOLOR, edgecolor=EDGECOLOR, linewidths=0.3, alpha=1.0
    )
    ax.add_collection3d(surface)
    ax.set_xlim(-LIMIT, LIMIT)
    ax.set_ylim(-LIMIT, LIMIT)
    ax.set_zlim(-LIMIT, LIMIT)
    ax.set_box_aspect((1, 1, 1))
    ax.view_init(elev=ELEV, azim=AZIM)
    ax.set_axis_off()
    ax.set_title(f"{title}\n{len(facets)} facets", fontsize=12)
    png = stl.with_suffix(".png")
    fig.savefig(png, dpi=150, bbox_inches="tight")
    plt.close(fig)
    print(f"wrote {png.name} ({len(facets)} facets)")


def main() -> None:
    here = Path(__file__).resolve().parent
    figures = {
        "sphere_radius_1": "input sphere",
        "sphere_uniform_coarse": "uniform, target 0.35",
        "sphere_uniform_fine": "uniform, target 0.08",
        "sphere_uniform": "uniform, target 0.18",
        "sphere_adaptive": "adaptive, 0.05-0.30",
    }
    for stem, title in figures.items():
        stl = here / f"{stem}.stl"
        if stl.exists():
            render(stl, title)
        else:
            print(f"skipping {stl.name} (not found)")

    # Edge-length histogram and topology summary for the example model.
    base = here / "sphere_radius_1.stl"
    if base.exists():
        render_histogram(base, "sphere_edge_histogram.png")
        faces, edges, vertices = topology(read_stl(base))
        print(
            f"sphere_radius_1.stl: F={faces} E={edges} V={vertices} "
            f"(V - E + F = {vertices - edges + faces})"
        )


if __name__ == "__main__":
    main()
