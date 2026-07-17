r"""This module, remesh_bunny_figures.py, renders the Stanford bunny surface
triangulations used in the Stanford bunny remeshing example.  It reads the input
bunny and each remeshed output (produced by `automesh remesh`) and saves a
matched-camera PNG of every mesh.

The bunny's up-axis is +y; the renderer remaps model coordinates (x, y, z) to
plot coordinates (x, z, y) so the bunny stands upright.

Example
-------
source ~/autotwin/automesh/.venv/bin/activate
cd ~/autotwin/automesh/book/examples/remesh
python remesh_bunny_figures.py

Output
------
The `bunny_*.png` visualization files, written next to this script.
"""

import struct
from pathlib import Path
from typing import Final

import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d.art3d import Poly3DCollection
import numpy as np
from numpy.typing import NDArray

# Shared "hero" view so only the triangulation changes between figures.
ELEV: Final[float] = 18.0
AZIM: Final[float] = 55.0
FACECOLOR: Final[str] = "lightblue"
EDGECOLOR: Final[str] = "navy"


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
    # Remap (x, y, z) -> (x, z, y) so the bunny's +y up-axis points up in the plot.
    return facets[:, :, [0, 2, 1]]


def edge_lengths(facets: NDArray[np.float64]) -> NDArray[np.float64]:
    """Returns the length of every unique undirected edge in the mesh."""
    keyed = np.round(facets.reshape(-1, 3), 8)
    _, inverse = np.unique(keyed, axis=0, return_inverse=True)
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
    ax.hist(lengths, bins=40, color=FACECOLOR, edgecolor=EDGECOLOR)
    ax.axvline(
        lengths.mean(),
        color="crimson",
        linestyle="--",
        linewidth=1.5,
        label=f"mean = {lengths.mean():.4f}",
    )
    ax.set_xlabel("triangle edge length")
    ax.set_ylabel("count")
    ax.set_title(f"{stl.stem}: edge length distribution")
    ax.legend()
    png = stl.with_name(out_name)
    fig.savefig(png, dpi=150, bbox_inches="tight")
    plt.close(fig)
    print(f"wrote {png.name} ({len(lengths):,} edges)")


def render(stl: Path, title: str) -> None:
    """Renders a single bunny STL to a PNG next to it.  Dense meshes are drawn
    without edges (a shaded surface); coarse meshes show their triangle edges."""
    facets = read_stl(stl)
    n = len(facets)
    # Show triangle edges for all but the very dense input scan, which is drawn
    # as a shaded surface.  Thin the lines as the facet count grows.
    show_edges = n <= 40000
    linewidth = 0.25 if n <= 10000 else 0.12
    fig = plt.figure(figsize=(5, 5))
    ax = fig.add_subplot(111, projection="3d")
    surface = Poly3DCollection(
        facets,
        facecolor=FACECOLOR,
        edgecolor=EDGECOLOR if show_edges else "none",
        linewidths=linewidth if show_edges else 0.0,
        rasterized=True,
    )
    surface.set_alpha(1.0)
    ax.add_collection3d(surface)

    pts = facets.reshape(-1, 3)
    lo, hi = pts.min(0), pts.max(0)
    center = (lo + hi) / 2
    radius = (hi - lo).max() / 2
    for setter, c in zip((ax.set_xlim, ax.set_ylim, ax.set_zlim), center):
        setter(c - radius, c + radius)
    ax.set_box_aspect((1, 1, 1))
    ax.view_init(elev=ELEV, azim=AZIM)
    ax.set_axis_off()
    ax.set_title(f"{title}\n{n:,} facets", fontsize=11)
    png = stl.with_suffix(".png")
    fig.savefig(png, dpi=150, bbox_inches="tight")
    plt.close(fig)
    print(f"wrote {png.name} ({n:,} facets)")


def main() -> None:
    here = Path(__file__).resolve().parent
    figures = {
        "stanford_bunny": "input scan",
        "bunny_uniform_fine": "uniform, size 0.004",
        "bunny_uniform_coarse": "uniform, size 0.006",
        "bunny_iter_n5": "uniform 0.006, 5 iterations",
        "bunny_compare_uniform": "uniform, size 0.0036",
        "bunny_adaptive": "adaptive, 0.002-0.040",
        "bunny_tol_tight": "tolerance 0.0002",
        "bunny_tol_mid": "tolerance 0.002",
        "bunny_tol_loose": "tolerance 0.02",
        "bunny_adapt_grad_lo": "adaptive, gradation 0.1",
        "bunny_adapt_grad_hi": "adaptive, gradation 0.9",
    }
    for stem, title in figures.items():
        stl = here / f"{stem}.stl"
        if stl.exists():
            render(stl, title)
        else:
            print(f"skipping {stl.name} (not found)")

    # Edge-length histogram of the input scan.
    base = here / "stanford_bunny.stl"
    if base.exists():
        render_histogram(base, "bunny_edge_histogram.png")


if __name__ == "__main__":
    main()
