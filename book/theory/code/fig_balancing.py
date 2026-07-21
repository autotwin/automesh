"""
fig_balancing.py

Generate a side-by-side figure showing a 2x2 quadtree block before and after
the weak-balancing algorithm is applied.

Left (unbalanced): SW, SE, NW at L_n; NE at L_{n+2} — a 2-level edge disparity.
Right (weakly balanced, key_0112): SW at L_n; SE, NW at L_{n+1}; NE at L_{n+2}.

This matches the sibl reference primal_quad_0112.png and the key_0112 row of
the template table.  The unbalanced left panel looks like key_0111 except the
three "fine" cells are at L_n and the NE cell jumps two levels to L_{n+2}.

Run: python quadtree/fig_balancing.py
Output: quadtree/fig_balancing.png
"""

import os

import matplotlib.patches as patches
import matplotlib.pyplot as plt

plt.rcParams.update({
    "text.usetex": True,
    "font.family": "serif",
    "font.serif": ["Computer Modern Roman"],
})

TAB_ORANGE = "#FF7F0E"   # L_n      (coarse)
TAB_GREEN  = "#2CA02C"   # L_{n+1}  (buffer)
TAB_RED    = "#D62728"   # L_{n+2}  (fine)
ALPHA = 0.5


def draw_rect(ax, x, y, size, color, zorder=1):
    ax.add_patch(patches.Rectangle(
        (x, y), size, size,
        linewidth=0.7,
        edgecolor=color,
        facecolor=color,
        alpha=ALPHA,
        zorder=zorder,
    ))


def draw_unbalanced(ax):
    """SW, SE, NW at L_n; NE subdivided 4x4 at L_{n+2}."""
    ax.set_xlim(-0.05, 2.05)
    ax.set_ylim(-0.05, 2.05)
    ax.set_aspect("equal")
    ax.axis("off")
    ax.set_title("Unbalanced", fontsize=11, pad=4)

    draw_rect(ax, 0, 0, 1.0, TAB_ORANGE, zorder=1)   # SW
    draw_rect(ax, 1, 0, 1.0, TAB_ORANGE, zorder=1)   # SE
    draw_rect(ax, 0, 1, 1.0, TAB_ORANGE, zorder=1)   # NW

    # NE: 4x4 sub-cells at L_{n+2} (size 0.25 each)
    for dr in range(4):
        for dc in range(4):
            draw_rect(ax, 1 + dc * 0.25, 1 + dr * 0.25, 0.25, TAB_RED, zorder=3)

    # Labels: SW sub-cell of each level region
    ax.text(0.5,   0.5,   r"$L_n$",     ha="center", va="center",
            fontsize=11, color=TAB_ORANGE)
    ax.text(1.125, 1.125, r"$L_{n+2}$", ha="center", va="center",
            fontsize=8, color=TAB_RED)

    # Annotation: text in SE cell, arrow points to the SE-NE shared edge midpoint
    ax.annotate(
        "2-level\ndisparity",
        xy=(1.5, 1.0), xytext=(1.5, 0.5),
        fontsize=8, color="red",
        arrowprops=dict(arrowstyle="->", color="red", lw=1.2),
        ha="center", va="center",
    )


def draw_balanced(ax):
    """key_0112: SW=L_n, SE=L_{n+1} (2x2), NW=L_{n+1} (2x2), NE=L_{n+2} (4x4)."""
    ax.set_xlim(-0.05, 2.05)
    ax.set_ylim(-0.05, 2.05)
    ax.set_aspect("equal")
    ax.axis("off")
    ax.set_title(r"Weakly Balanced (\texttt{key\_0112})", fontsize=11, pad=4)

    # SW: single cell at L_n
    draw_rect(ax, 0, 0, 1.0, TAB_ORANGE, zorder=1)

    # SE: 2x2 sub-cells at L_{n+1} (size 0.5 each)
    for dr in range(2):
        for dc in range(2):
            draw_rect(ax, 1 + dc * 0.5, 0 + dr * 0.5, 0.5, TAB_GREEN, zorder=2)

    # NW: 2x2 sub-cells at L_{n+1} (size 0.5 each)
    for dr in range(2):
        for dc in range(2):
            draw_rect(ax, 0 + dc * 0.5, 1 + dr * 0.5, 0.5, TAB_GREEN, zorder=2)

    # NE: 4x4 sub-cells at L_{n+2} (size 0.25 each)
    for dr in range(4):
        for dc in range(4):
            draw_rect(ax, 1 + dc * 0.25, 1 + dr * 0.25, 0.25, TAB_RED, zorder=3)

    # Labels: SW sub-cell of each level region
    ax.text(0.5,   0.5,   r"$L_n$",     ha="center", va="center",
            fontsize=11, color=TAB_ORANGE)
    ax.text(1.25,  0.25,  r"$L_{n+1}$", ha="center", va="center",
            fontsize=9, color=TAB_GREEN)
    ax.text(1.125, 1.125, r"$L_{n+2}$", ha="center", va="center",
            fontsize=8, color=TAB_RED)


def main():
    fig, (ax_left, ax_right) = plt.subplots(
        1, 2, figsize=(9, 4.5),
        gridspec_kw={"wspace": 0.35},
    )

    draw_unbalanced(ax_left)
    draw_balanced(ax_right)

    legend_elements = [
        patches.Patch(facecolor=TAB_ORANGE, edgecolor=TAB_ORANGE, alpha=ALPHA,
                      label=r"$L_n$ (coarse)"),
        patches.Patch(facecolor=TAB_GREEN,  edgecolor=TAB_GREEN,  alpha=ALPHA,
                      label=r"$L_{n+1}$ (buffer)"),
        patches.Patch(facecolor=TAB_RED,    edgecolor=TAB_RED,    alpha=ALPHA,
                      label=r"$L_{n+2}$ (fine)"),
    ]
    fig.legend(
        handles=legend_elements,
        loc="lower center",
        ncol=3,
        fontsize=10,
        frameon=True,
        bbox_to_anchor=(0.5, 0.01),
    )

    fig.suptitle("Quadtree Balancing: Before and After", fontsize=12, y=1.01)

    here = os.path.dirname(os.path.abspath(__file__))
    out = os.path.join(here, "..", "..", "fig", "quadtree_balancing.svg")
    fig.savefig(out, format="svg", bbox_inches="tight")
    print(f"Saved: {out}")
    plt.close(fig)


if __name__ == "__main__":
    main()
