"""
fig_tong_2024_tier_coupling.py

Two-panel node-link diagram of the tier-coupling mechanism described in
"Threshold Fitting as a Mechanical Procedure": a cell's refinement flag
("intersect") is set if any child is already flagged, and only falls
through to the cell's own threshold test when none are.

Each panel shows a small before/after pair on the same three-level branch
(root -> one expanded child -> two grandchildren; sibling children shown
collapsed).  One grandchild ("test") is the cell whose own threshold
changes between "before" and "after"; the other grandchild ("other") is
held fixed within each panel, and is set differently between the two
panels specifically to make the asymmetry visible:

Left panel (tighten): "other" stays flagged throughout.  Flipping "test"
off therefore changes nothing above it -- the parent and root were, and
remain, flagged because of "other", not because of "test".  Only "test"
carries a changed-flag ring.

Right panel (loosen): "other" stays unflagged throughout.  Flipping
"test" on is therefore the *only* reason its parent's child-check can
find, so the parent flips too -- and the root's child-check then finds
its own newly-flagged child in turn.  All three carry changed-flag rings.

Text renders via matplotlib's built-in mathtext with a Computer-Modern
-style serif font (`mathtext.fontset="cm"`), not real LaTeX
(`text.usetex`) -- visually close to a real-LaTeX-rendered figure, but
with no external LaTeX toolchain dependency. Matches the convention used
in https://github.com/hovey/dictk's own figures (see e.g. `dictk`'s
`src/dictk/image.py`). Code-styled labels use `$\mathtt{...}$`, mathtext's
supported monospace command, rather than plain-text `\texttt{...}`, which
mathtext does not parse outside of a real LaTeX run.

Run: python fig_tong_2024_tier_coupling.py
Output: ../../fig/tong_2024_tier_coupling.svg
"""

import os

import matplotlib.pyplot as plt
from matplotlib.patches import Circle, FancyArrowPatch

FLAGGED = "#2CA02C"       # intersect = true (green)
UNFLAGGED = "#BFBFBF"     # intersect = false (gray)
CHANGED_EDGE = "#D62728"  # red outline: a node whose flag changed
NODE_R = 0.15

LAYOUT = {
    "root": (1.5, 2.5),
    "sibs": [(0.3, 1.4), (2.7, 1.4)],
    "child0": (1.5, 1.4),
    "leaf_test": (0.9, 0.3),
    "leaf_other": (2.1, 0.3),
}


def draw_edge(ax, p, q):
    ax.add_patch(FancyArrowPatch(p, q, arrowstyle="-", color="#999999",
                                  linewidth=1.0, zorder=1))


def draw_node(ax, xy, flagged, changed=False):
    color = FLAGGED if flagged else UNFLAGGED
    ax.add_patch(Circle(xy, NODE_R, facecolor=color, edgecolor="#333333",
                         linewidth=0.8, zorder=3))
    if changed:
        ax.add_patch(Circle(xy, NODE_R, facecolor="none",
                             edgecolor=CHANGED_EDGE, linewidth=2.4, zorder=4))


def draw_tree(ax, xy0, scale, test_flagged, other_flagged, mark_changed):
    """Draw one small tree at origin xy0, uniformly scaled by `scale`.

    mark_changed: set of node keys ("root", "child0", "leaf_test") to ring.
    """
    def T(p):
        return (xy0[0] + p[0] * scale, xy0[1] + p[1] * scale)

    root, sibs = T(LAYOUT["root"]), [T(p) for p in LAYOUT["sibs"]]
    child0 = T(LAYOUT["child0"])
    leaf_test, leaf_other = T(LAYOUT["leaf_test"]), T(LAYOUT["leaf_other"])

    for s in sibs:
        draw_edge(ax, root, s)
    draw_edge(ax, root, child0)
    draw_edge(ax, child0, leaf_test)
    draw_edge(ax, child0, leaf_other)

    child0_flagged = test_flagged or other_flagged  # child-check
    root_flagged = child0_flagged                    # child-check

    for s in sibs:
        draw_node(ax, s, flagged=False)
    draw_node(ax, leaf_other, flagged=other_flagged)
    draw_node(ax, leaf_test, flagged=test_flagged, changed="leaf_test" in mark_changed)
    draw_node(ax, child0, flagged=child0_flagged, changed="child0" in mark_changed)
    draw_node(ax, root, flagged=root_flagged, changed="root" in mark_changed)


def panel(ax, title, other_flagged, test_before, test_after, mark_changed):
    ax.set_xlim(-0.3, 6.3)
    ax.set_ylim(-0.6, 3.0)
    ax.set_aspect("equal")
    ax.axis("off")
    ax.set_title(title, fontsize=9.5, pad=8, wrap=True)

    draw_tree(ax, (0.0, 0.0), 0.62, test_before, other_flagged, mark_changed=set())
    ax.text(1.0, -0.5, "before", ha="center", fontsize=9, style="italic")

    ax.annotate("", xy=(3.75, 1.2), xytext=(2.65, 1.2),
                arrowprops=dict(arrowstyle="->", color="#333333", lw=1.6))

    draw_tree(ax, (4.05, 0.0), 0.62, test_after, other_flagged, mark_changed=mark_changed)
    ax.text(5.05, -0.5, "after", ha="center", fontsize=9, style="italic")


def main():
    with plt.rc_context({"font.family": "serif", "mathtext.fontset": "cm"}):
        fig, (ax_l, ax_r) = plt.subplots(1, 2, figsize=(11.5, 4.6),
                                          gridspec_kw={"wspace": 0.25})

        # Tighten: the sibling grandchild ("other") stays flagged throughout,
        # so the parent and root were never flagged *because of* "test" --
        # only "test" itself changes.
        panel(ax_l, "Tightening: only the leaf's flag changes",
              other_flagged=True, test_before=True, test_after=False,
              mark_changed={"leaf_test"})

        # Loosen: "other" stays unflagged throughout, so "test" becoming
        # flagged is the only reason the parent and root have to change --
        # and they do.
        panel(ax_r, "Loosening: every ancestor's flag changes too",
              other_flagged=False, test_before=False, test_after=True,
              mark_changed={"leaf_test", "child0", "root"})

        legend_elements = [
            plt.Line2D([0], [0], marker="o", color="w", markerfacecolor=FLAGGED,
                       markersize=13, label=r"$\mathtt{intersect}$ = true"),
            plt.Line2D([0], [0], marker="o", color="w", markerfacecolor=UNFLAGGED,
                       markersize=13, label=r"$\mathtt{intersect}$ = false"),
            plt.Line2D([0], [0], marker="o", color="w", markeredgecolor=CHANGED_EDGE,
                       markerfacecolor="white", markeredgewidth=2.4, markersize=13,
                       label=r"flag changed"),
        ]
        fig.legend(handles=legend_elements, loc="lower center", ncol=3,
                   fontsize=9.5, frameon=True, bbox_to_anchor=(0.5, -0.05))

        fig.suptitle(r"Tier Coupling in $\mathtt{ComputeCellValue()}$", fontsize=13, y=1.03)

        here = os.path.dirname(os.path.abspath(__file__))
        out = os.path.join(here, "..", "..", "fig", "tong_2024_tier_coupling.svg")
        fig.savefig(out, format="svg", bbox_inches="tight")
        print(f"Saved: {out}")
        plt.close(fig)


if __name__ == "__main__":
    main()
