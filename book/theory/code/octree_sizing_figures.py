r"""Generate the octree sizing illustration for book/theory/hex_from_surface.md.

Draws a 2D quadtree analogue of the 3D octree sizing rule, showing how the
`--scale` argument s, the finest cell size l, and the tree depth L relate to the
shape diameter function of the geometry.

The dumbbell shape has a thick lobe and a thin bar, so the *local* nature of the
refinement is visible: the bar is meshed finely because it is thin, while the
lobes stay coarse.  Both regions receive about s cells across their own local
thickness.

Run manually to regenerate; the PNG is committed as a static asset:

    python3 book/theory/code/octree_sizing_figures.py
"""

import matplotlib.pyplot as plt
import numpy as np
from matplotlib.patches import Rectangle

SCALE = 4  # the `--scale` argument, s
OUT = "book/fig/octree_sizing.png"


def thickness(x, y):
    """Local thickness of a dumbbell: two lobes joined by a thin bar."""
    lobes = [(-0.45, 0.0, 0.30), (0.45, 0.0, 0.30)]
    for cx, cy, r in lobes:
        if (x - cx) ** 2 + (y - cy) ** 2 <= r**2:
            return 2.0 * r
    if abs(x) <= 0.5 and abs(y) <= 0.06:
        return 0.12
    return None


def inside(x, y):
    return thickness(x, y) is not None


def occupied(x0, y0, h):
    """Does this cell straddle the boundary of the solid?"""
    pts = [
        (x0 + fx * h, y0 + fy * h)
        for fx in np.linspace(0, 1, 5)
        for fy in np.linspace(0, 1, 5)
    ]
    flags = [inside(px, py) for px, py in pts]
    return any(flags) and not all(flags)


def local_target(x0, y0, h):
    """Minimum local thickness sampled over the cell."""
    vals = [
        thickness(x0 + fx * h, y0 + fy * h)
        for fx in np.linspace(0, 1, 5)
        for fy in np.linspace(0, 1, 5)
    ]
    vals = [v for v in vals if v is not None]
    return min(vals) if vals else None


def subdivide(x0, y0, h, depth, max_depth, out):
    """Refine while the cell is coarser than (local thickness) / s."""
    target = local_target(x0, y0, h)
    refine = (
        depth < max_depth
        and occupied(x0, y0, h)
        and target is not None
        and h * SCALE > target
    )
    if not refine:
        out.append((x0, y0, h, depth))
        return
    half = h / 2
    for dx in (0, half):
        for dy in (0, half):
            subdivide(x0 + dx, y0 + dy, half, depth + 1, max_depth, out)


def main():
    extent = 2.0  # E, the largest bounding box extent
    d_min = 0.12  # thinnest feature of the geometry
    cell = d_min / SCALE  # l, the finest cell size
    depth = int(np.ceil(np.log2(extent / cell)))  # L, the tree depth

    cells = []
    subdivide(-1.0, -1.0, extent, 0, depth, cells)

    fig, ax = plt.subplots(figsize=(7.2, 6.0))

    for x0, y0, h, d in cells:
        ax.add_patch(
            Rectangle(
                (x0, y0), h, h, facecolor="none", edgecolor="0.75", linewidth=0.4
            )
        )

    # The solid itself.
    grid = np.linspace(-1.0, 1.0, 500)
    xx, yy = np.meshgrid(grid, grid)
    mask = np.vectorize(inside)(xx, yy)
    ax.contour(xx, yy, mask.astype(float), levels=[0.5], colors="#1f77b4", linewidths=2)

    # The finest cell, sitting on the thin bar.
    finest = min(cells, key=lambda c: c[2])
    ax.add_patch(
        Rectangle(
            (finest[0], finest[1]),
            finest[2],
            finest[2],
            facecolor="#d62728",
            edgecolor="#d62728",
            alpha=0.55,
        )
    )

    # d_min annotation across the thin bar.
    ax.annotate(
        "",
        xy=(0.0, -0.06),
        xytext=(0.0, 0.06),
        arrowprops=dict(arrowstyle="<->", color="#d62728", lw=1.6),
    )
    ax.text(
        0.04,
        0.14,
        r"$d_{\min}$" + f"\n({SCALE} cells across)",
        color="#d62728",
        fontsize=11,
        va="bottom",
    )

    # E annotation across the root.
    ax.annotate(
        "",
        xy=(-1.0, -1.12),
        xytext=(1.0, -1.12),
        arrowprops=dict(arrowstyle="<->", color="0.3", lw=1.4),
    )
    ax.text(0.0, -1.20, r"$E$ (root cell)", color="0.3", fontsize=11, ha="center")

    ax.text(
        -1.0,
        1.06,
        rf"$s = {SCALE}$,   $\ell = d_{{\min}}/s$,   "
        rf"$L = \lceil \log_2 (E/\ell) \rceil = {depth}$",
        fontsize=12,
    )
    ax.text(
        finest[0] + finest[2] + 0.03,
        finest[1] - 0.16,
        r"finest cell, $\ell$",
        color="#d62728",
        fontsize=11,
    )
    ax.text(
        -0.72,
        0.34,
        "thick region:\ncoarse cells",
        fontsize=10,
        color="0.35",
        ha="center",
    )

    ax.set_xlim(-1.1, 1.1)
    ax.set_ylim(-1.3, 1.2)
    ax.set_aspect("equal")
    ax.axis("off")
    fig.tight_layout()
    fig.savefig(OUT, dpi=180, bbox_inches="tight")
    print(f"wrote {OUT}  [{len(cells)} cells, depth {depth}]")


if __name__ == "__main__":
    main()
