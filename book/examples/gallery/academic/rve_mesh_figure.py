#!/usr/bin/env python3
"""Renders the RVE tet4 and hex meshes, painted by Minimum Scaled Jacobian.

Six panels: each mesh is shown whole (top row), cut (middle row), and
zoomed on the largest pore (bottom row).  Colors
follow the autotwin/quality convention (scripts/mesh_render.py,
figures/bone_baseline_mesh.png) -- viridis over a fixed [0, 1] range, so the
two meshes are directly comparable by color, with black element edges.

The three pores are entirely interior, so a whole view shows only the cube's
surface.  Their centers are collinear and all lie on the plane x = y, so the
bottom row keeps the elements whose centroid satisfies x <= y.  That single cut
passes through the center of all three pores at once.

The tet10 mesh is not shown.  Its corner nodes reproduce the tet4 mesh exactly
-- same 35,457 elements in the same order, identical coordinates -- so a tet10
panel would be indistinguishable from the tet4 panel.

Example
-------
cd ~/autotwin/automesh/book/examples/gallery/academic
# none of the inputs are committed; see the Downloads section of rve.md
automesh convert mesh -i hexahedra.exo -o hexahedra.inp
automesh metrics -i tetrahedra_4.inp -o tet4_metrics.csv
automesh metrics -i hexahedra.exo    -o hex_metrics.csv
python rve_mesh_figure.py tetrahedra_4.inp tet4_metrics.csv \
    hexahedra.inp hex_metrics.csv

Output
------
The `rve_mesh_msj.png` visualization file, written next to this script.
"""
import sys
from pathlib import Path

import matplotlib
matplotlib.use("agg")
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.cm import ScalarMappable
from matplotlib.colors import Normalize
from mpl_toolkits.mplot3d.art3d import Poly3DCollection

TET_FACES = [(0, 1, 2), (0, 1, 3), (1, 2, 3), (0, 2, 3)]
HEX_FACES = [(0, 1, 2, 3), (4, 5, 6, 7), (0, 1, 5, 4),
             (1, 2, 6, 5), (2, 3, 7, 6), (3, 0, 4, 7)]

ELEV = 20.0
AZIM = 30.0
# The largest pore, from the Cubit journal: radius R/10 at (-1/4, -1/4, 1/4).
PORE_CENTER = np.array([-0.25, -0.25, 0.25])
ZOOM_HALF = 0.18  # half-width of the zoom box around that pore
# The cut plane x = y has normal (1, -1, 0), which points at azimuth -45.  The
# whole views look along that plane and would show the cut face edge-on, so the
# cut views swing the camera around to the removed side to face it.
AZIM_CUT = -55.0
SURFACE = "#fcfcfb"
INK = "#0b0b0b"
COLORMAP = "viridis"


def inp_load(*, path, per_element):
    """Reads nodes and element connectivity from an Abaqus .inp file."""
    coordinates, labels, elements, section = [], [], [], None
    for line in open(path):
        text = line.strip()
        if text.startswith("*"):
            section = text.lower()
            continue
        if not text or section is None:
            continue
        fields = text.split(",")
        if section.startswith("*node"):
            labels.append(int(fields[0]))
            coordinates.append([float(v) for v in fields[1:4]])
        elif section.startswith("*element"):
            elements.append([int(v) for v in fields[1:1 + per_element]])
    index = {label: i for i, label in enumerate(labels)}
    cells = np.array([[index[n] for n in e] for e in elements])
    return np.array(coordinates), cells


def msj_load(*, path):
    """Reads the minimum scaled jacobian column of an `automesh metrics` CSV."""
    with open(path) as f:
        header = f.readline().strip().split(",")
    column = header.index("minimum scaled jacobian")
    return np.genfromtxt(path, delimiter=",", skip_header=1)[:, column]


def skin_extract(*, cells, faces, subset=None):
    """Returns the boundary faces of a mesh, or of a subset of its elements.

    A face lies on the boundary when it belongs to exactly one element of the
    set considered.  Passing a subset therefore exposes the cut surface as
    well as the original outer surface.
    """
    chosen = range(len(cells)) if subset is None else np.flatnonzero(subset)
    count, owner, corners = {}, {}, {}
    for element in chosen:
        cell = cells[element]
        for face in faces:
            nodes = [cell[i] for i in face]
            key = frozenset(nodes)
            count[key] = count.get(key, 0) + 1
            owner[key] = element
            corners[key] = nodes
    boundary = [k for k, c in count.items() if c == 1]
    return (np.array([corners[k] for k in boundary]),
            np.array([owner[k] for k in boundary]))


def cut_select(*, coordinates, cells):
    """Selects the elements whose centroid satisfies x <= y.

    The three pore centers are collinear and all lie on the plane x = y, so
    this one cut passes through every pore center.
    """
    centroids = coordinates[cells].mean(axis=1)
    return centroids[:, 0] <= centroids[:, 1]


def zoom_select(*, coordinates, cells, cut):
    """Selects the cut elements lying in a box around the largest pore."""
    centroids = coordinates[cells].mean(axis=1)
    inside = (np.abs(centroids - PORE_CENTER) <= ZOOM_HALF).all(axis=1)
    return cut & inside


def panel_render(*, axes, coordinates, skin, owner, msj, title, azim=AZIM,
                 limits=None):
    """Draws one mesh surface, colored per element by scaled jacobian."""
    polygons = Poly3DCollection(coordinates[skin], linewidths=0.15,
                                edgecolor="black")
    polygons.set_facecolor(plt.get_cmap(COLORMAP)(msj[owner]))
    axes.add_collection3d(polygons)
    if limits is None:
        lower, upper = coordinates.min(axis=0), coordinates.max(axis=0)
    else:
        lower, upper = limits
    axes.set_xlim(lower[0], upper[0])
    axes.set_ylim(lower[1], upper[1])
    axes.set_zlim(lower[2], upper[2])
    axes.set_box_aspect(tuple(upper - lower))
    axes.view_init(elev=ELEV, azim=azim)
    axes.set_axis_off()
    axes.set_title(title, color=INK, fontsize=11)


def mesh_figure_render(*, meshes, png):
    """Saves the four-panel figure: each mesh whole, then cut."""
    figure = plt.figure(figsize=(10, 14))
    figure.patch.set_facecolor(SURFACE)
    for column, (label, path, per_element, faces, metrics) in enumerate(meshes):
        coordinates, cells = inp_load(path=path, per_element=per_element)
        msj = msj_load(path=metrics)
        keep = cut_select(coordinates=coordinates, cells=cells)
        zoom = zoom_select(coordinates=coordinates, cells=cells, cut=keep)
        zoom_limits = (PORE_CENTER - ZOOM_HALF, PORE_CENTER + ZOOM_HALF)
        rows = (
            (None, f"{len(cells)} elements", AZIM, None),
            (keep, "cut at $x = y$, through all three pores", AZIM_CUT, None),
            (zoom, "largest pore, radius $R/10$", AZIM_CUT, zoom_limits),
        )
        for row, (subset, suffix, azim, limits) in enumerate(rows):
            skin, owner = skin_extract(cells=cells, faces=faces, subset=subset)
            axes = figure.add_subplot(3, 2, row * 2 + column + 1,
                                      projection="3d")
            panel_render(axes=axes, coordinates=coordinates, skin=skin,
                         owner=owner, msj=msj, title=f"{label}\n{suffix}",
                         azim=azim, limits=limits)
        print(f"{label}: {len(cells)} elements, "
              f"msj [{msj.min():.3f}, {msj.max():.3f}]")

    figure.tight_layout(rect=[0, 0.06, 1, 1])
    bar = figure.colorbar(
        ScalarMappable(norm=Normalize(0.0, 1.0), cmap=COLORMAP),
        ax=figure.axes, orientation="horizontal", fraction=0.04, pad=0.04,
    )
    bar.set_label("Minimum Scaled Jacobian", color=INK, fontsize=10)
    bar.ax.tick_params(colors=INK, labelsize=9)
    figure.savefig(png, dpi=200, facecolor=SURFACE, bbox_inches="tight")
    plt.close(figure)
    print(f"wrote {png.name}")


def main():
    if len(sys.argv) != 5:
        print(f"usage: {sys.argv[0]} <tet4.inp> <tet4_metrics.csv> "
              f"<hex.inp> <hex_metrics.csv>", file=sys.stderr)
        sys.exit(1)
    here = Path(__file__).resolve().parent
    meshes = [
        ("tet4", Path(sys.argv[1]).expanduser(), 4, TET_FACES,
         Path(sys.argv[2]).expanduser()),
        ("automesh hex", Path(sys.argv[3]).expanduser(), 8, HEX_FACES,
         Path(sys.argv[4]).expanduser()),
    ]
    mesh_figure_render(meshes=meshes, png=here / "rve_mesh_msj.png")


if __name__ == "__main__":
    main()
