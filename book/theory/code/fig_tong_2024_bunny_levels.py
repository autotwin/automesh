"""
fig_tong_2024_bunny_levels.py

Grouped bar chart comparing octree-level populations between Bunny's
published reference mesh (Tong 2024, our results/bunny.vtk) and this
study's best reproduction (runs/bunny-v1.0-rl4-halfshift-c145/finalMesh.vtk).

Counts are read via the twelve-edge-mean level metric described in
"Octree Construction" and "Reproducing Tong 2024".  Levels 9-10 are
omitted (6 cells total in the reproduction, 0 in the reference) as
negligible noise below the plotted range.

Text renders via matplotlib's built-in mathtext with a Computer-Modern
-style serif font (`mathtext.fontset="cm"`), not real LaTeX
(`text.usetex`) -- visually close to a real-LaTeX-rendered figure, but
with no external LaTeX toolchain dependency. Matches the convention used
in https://github.com/hovey/dictk's own figures (see e.g. `dictk`'s
`src/dictk/image.py`).

Run: python fig_tong_2024_bunny_levels.py
Output: ../../fig/tong_2024_bunny_levels.svg
"""

import os

import matplotlib.pyplot as plt
import numpy as np

TAB_BLUE = "#1F77B4"    # reference
TAB_ORANGE = "#FF7F0E"  # reproduction

LEVELS = [4, 5, 6, 7, 8]
REFERENCE = [204, 5423, 9000, 6960, 108]
REPRODUCTION = [259, 5297, 8344, 8127, 492]


def main():
    x = np.arange(len(LEVELS))
    width = 0.38

    with plt.rc_context({"font.family": "serif", "mathtext.fontset": "cm"}):
        fig, ax = plt.subplots(figsize=(6.5, 4.2))

        ax.bar(x - width / 2, REFERENCE, width, label="Reference (Tong 2024)",
               color=TAB_BLUE, alpha=0.85, edgecolor=TAB_BLUE, linewidth=0.7)
        ax.bar(x + width / 2, REPRODUCTION, width, label="Reproduction (this study)",
               color=TAB_ORANGE, alpha=0.85, edgecolor=TAB_ORANGE, linewidth=0.7)

        ax.set_xticks(x)
        ax.set_xticklabels([str(l) for l in LEVELS])
        ax.set_xlabel("Octree level")
        ax.set_ylabel("Hexahedron count")
        ax.set_title("Bunny: Octree-Level Population, Reference vs. Reproduction")
        ax.legend(frameon=True, fontsize=9)
        ax.spines["top"].set_visible(False)
        ax.spines["right"].set_visible(False)

        for xi, (r, p) in zip(x, zip(REFERENCE, REPRODUCTION)):
            ax.text(xi - width / 2, r + 120, f"{r:,}", ha="center", va="bottom", fontsize=7.5)
            ax.text(xi + width / 2, p + 120, f"{p:,}", ha="center", va="bottom", fontsize=7.5)

        fig.tight_layout()

        here = os.path.dirname(os.path.abspath(__file__))
        out = os.path.join(here, "..", "..", "fig", "tong_2024_bunny_levels.svg")
        fig.savefig(out, format="svg", bbox_inches="tight")
        print(f"Saved: {out}")
        plt.close(fig)


if __name__ == "__main__":
    main()
