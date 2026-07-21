"""
fig_octree_templates.py

Generate a 2x3 panel figure showing the primal octree cells and the dual
hexahedra for the six unique face and edge transition templates:
  FT0, FT1  (face templates)
  ET1--ET4  (edge templates)

Styling mirrors the quadtree template figures of Section 2.2 (primal cells as
wireframe cubes, coarse gray and fine light yellow; dual elements shaded blue;
dual vertices as red dots), rendered in 3D.

The geometry in octree_templates_data.json is the authoritative dual produced
by the automesh dualization (autotwin/automesh, dev branch, src/tree/hex).
For each template, one representative instance was extracted from a minimal
balanced octree: the primal leaf cells, the dual hexahedra belonging to that
template, and their node coordinates (local origin, integer grid units).

Adapted for the automesh book from the companion manuscript figure of the
same name.  Self-contained: the geometry lives in octree_templates_data.json
alongside this script.  Emits SVG so the dense 3D panels stay sharp at any zoom.

Run (from anywhere): python3 book/theory/code/fig_octree_templates.py
Output: book/fig/octree_templates.svg
"""

import json
import os

import matplotlib.patches as patches
import matplotlib.pyplot as plt
import numpy as np
from mpl_toolkits.mplot3d.art3d import Line3DCollection, Poly3DCollection

plt.rcParams.update({
    "text.usetex": True,
    "font.family": "serif",
    "font.serif": ["Computer Modern Roman"],
})

COLOR_COARSE      = "#C8C8C8"   # light gray   — primal L_n cells
COLOR_FINE        = "#FCF3CF"   # light yellow — primal L_{n+1} cells
COLOR_DUAL_FACE   = "#2980B9"   # blue         — dual hexahedron faces
COLOR_DUAL_EDGE   = "#1A5276"   # dark blue    — dual hexahedron edges
COLOR_CENTER      = "#E74C3C"   # red          — dual vertices

# The six faces of a hexahedron in standard (bottom 0123 / top 4567) ordering.
HEX_FACES = [[0, 1, 2, 3], [4, 5, 6, 7], [0, 1, 5, 4],
             [1, 2, 6, 5], [2, 3, 7, 6], [3, 0, 4, 7]]

# The twelve edges of an axis-aligned cube of edge length l at (x, y, z).
CUBE_EDGES = [(0, 1), (1, 2), (2, 3), (3, 0), (4, 5), (5, 6),
              (6, 7), (7, 4), (0, 4), (1, 5), (2, 6), (3, 7)]


def cube_segments(x, y, z, l):
    p = [(x, y, z), (x + l, y, z), (x + l, y + l, z), (x, y + l, z),
         (x, y, z + l), (x + l, y, z + l), (x + l, y + l, z + l), (x, y + l, z + l)]
    return [(p[a], p[b]) for a, b in CUBE_EDGES]


def draw_panel(ax, panel):
    coords = np.array(panel["coords"])

    # Primal cells: wireframe cubes, coarse gray / fine light yellow.
    for (x, y, z, l) in panel["primal"]:
        color = COLOR_FINE if l == 1 else COLOR_COARSE
        ax.add_collection3d(Line3DCollection(
            cube_segments(x, y, z, l), colors=color, linewidths=0.6, alpha=0.55))

    # Dual hexahedra: shaded blue faces with dark blue edges.
    polys = [[coords[h[k]] for k in face] for h in panel["hexes"] for face in HEX_FACES]
    ax.add_collection3d(Poly3DCollection(
        polys, facecolor=COLOR_DUAL_FACE, edgecolor=COLOR_DUAL_EDGE,
        linewidths=0.8, alpha=0.30))

    # Dual vertices used by the shown hexahedra.
    used = sorted({i for h in panel["hexes"] for i in h})
    s = coords[used]
    ax.scatter(s[:, 0], s[:, 1], s[:, 2], color=COLOR_CENTER, s=14, depthshade=False)

    # Equal cube aspect from the drawn extent.
    allpts = coords[used]
    rng = allpts.max(0) - allpts.min(0)
    ax.set_box_aspect(tuple(np.maximum(rng, 1e-3)))
    ax.view_init(elev=18, azim=-60)
    ax.set_axis_off()
    ax.set_title(panel["title"], fontsize=10, pad=-2)


def main():
    here = os.path.dirname(__file__)
    data = json.load(open(os.path.join(here, "octree_templates_data.json")))
    order = ["ft0", "ft1", "et1", "et2", "et3", "et4"]

    fig = plt.figure(figsize=(10, 7))
    for idx, key in enumerate(order):
        ax = fig.add_subplot(2, 3, idx + 1, projection="3d")
        draw_panel(ax, data[key])

    legend_elements = [
        patches.Patch(facecolor=COLOR_COARSE, edgecolor="#717D7E",
                      label=r"Primal coarse ($L_n$)"),
        patches.Patch(facecolor=COLOR_FINE, edgecolor="#717D7E",
                      label=r"Primal fine ($L_{n+1}$)"),
        patches.Patch(facecolor=COLOR_DUAL_FACE, edgecolor=COLOR_DUAL_EDGE,
                      alpha=0.5, label="Dual hexahedron"),
        plt.Line2D([0], [0], marker="o", color="w",
                   markerfacecolor=COLOR_CENTER, markersize=7,
                   label="Dual vertex"),
    ]
    fig.legend(handles=legend_elements, loc="lower center", ncol=4,
               fontsize=9, frameon=True, bbox_to_anchor=(0.5, 0.0))

    fig.suptitle(
        r"Octree Dual Transition Templates: Two Face (FT) and Four Edge (ET)",
        fontsize=11, y=0.98)

    out = os.path.join(here, "..", "..", "fig", "octree_templates.svg")
    fig.subplots_adjust(left=0.02, right=0.98, top=0.93, bottom=0.08,
                        wspace=0.05, hspace=0.12)
    fig.savefig(out, format="svg")
    print(f"Saved: {out}")
    plt.close(fig)


if __name__ == "__main__":
    main()
