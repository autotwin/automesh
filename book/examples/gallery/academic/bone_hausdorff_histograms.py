#!/usr/bin/env python3
"""Signed-distance histograms of the bone meshes against the bone surface.

Reads the CSV files that `automesh hausdorff` writes, one pair per mesh, and
plots two panels.  The left panel holds the distances from the boundary nodes
of each mesh to the surface (A to B).  The right panel holds the distances from
the vertices of the surface to the boundary of each mesh (B to A).  Both panels
share one x axis, so a distance reads the same on either side.  The y axis is
the share of samples, in percent, on a log scale, because the meshes have
different numbers of boundary nodes.

A negative distance lies inside the other surface, and a positive one outside.
Each panel labels its two sides: a node inside or outside the tessellation on
the left, and a tessellation vertex inside or outside the mesh on the right.
The legend gives the sampled Hausdorff distance of each mesh.  The lines match
the quality histograms: the first mesh solid in orange, the second dashed in
blue, the third dotted in green.

Example
-------
uv run --with numpy --with pandas --with matplotlib bone_hausdorff_histograms.py \
    bone_hausdorff_histograms.svg \
    --mesh hausdorff_bone "Tong 2024" \
    --mesh hausdorff_bone_uniform "automesh, uniform" \
    --mesh hausdorff_bone_adaptive "automesh, adaptive"

Output
------
The `bone_hausdorff_histograms.svg` visualization file.
"""
import argparse

import matplotlib
matplotlib.use("svg")
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd

STYLES = [
    {"color": "#eb6834", "linewidth": 2.0, "linestyle": "-"},
    {"color": "#2a78d6", "linewidth": 1.5, "linestyle": "--"},
    {"color": "#2f9e5b", "linewidth": 1.5, "linestyle": ":"},
]
INK = "#0b0b0b"
MUTED = "#5f5e5a"
GRID = "#e4e3df"
SURFACE = "#fcfcfb"
BINS = np.arange(-0.305, 0.6 + 1e-9, 0.01)
PANELS = [
    ("A_to_B", "Mesh to tessellation (A → B)", "boundary nodes: where the nodes sit",
     "node inside\nthe tessellation", "node outside\nthe tessellation"),
    ("B_to_A", "Tessellation to mesh (B → A)", "tessellation vertices: where the faces sit",
     "vertex inside\nthe mesh", "vertex outside\nthe mesh"),
]


def distances(prefix, direction):
    return pd.read_csv(f"{prefix}_{direction}.csv", comment="#")["signed distance"].values


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("out_svg")
    parser.add_argument("--mesh", nargs=2, action="append", metavar=("PREFIX", "LABEL"),
                        required=True)
    args = parser.parse_args()
    fig, axes = plt.subplots(1, 2, figsize=(10, 3.8), sharex=True, sharey=True)
    for (direction, title, subtitle, negative, positive), ax in zip(PANELS, axes):
        for (prefix, label), style in zip(args.mesh, STYLES):
            s = distances(prefix, direction)
            share = 100 * np.histogram(np.clip(s, BINS[0], BINS[-1]), BINS)[0] / len(s)
            hausdorff = max(np.abs(distances(prefix, "A_to_B")).max(),
                            np.abs(distances(prefix, "B_to_A")).max())
            ax.stairs(np.where(share > 0, share, np.nan), BINS,
                      label=f"{label}, $H_h$ = {hausdorff:.3f}", **style)
        ax.axvline(0, color=MUTED, linewidth=0.8)
        ax.set_yscale("log")
        ax.set_ylim(0.01, 150)
        ax.set_title(f"{title}\n", color=INK, fontsize=11)
        ax.text(0.5, 1.02, subtitle, transform=ax.transAxes, ha="center",
                color=MUTED, fontsize=8.5)
        ax.text(0.02, 0.97, negative, transform=ax.transAxes, ha="left", va="top",
                color=MUTED, fontsize=8)
        ax.text(0.98, 0.97, positive, transform=ax.transAxes, ha="right", va="top",
                color=MUTED, fontsize=8)
        ax.set_xlabel("signed distance (frame units)", color=INK, fontsize=9)
        ax.tick_params(colors=INK, labelsize=8)
        ax.grid(color=GRID, linewidth=0.6)
        for spine in ax.spines.values():
            spine.set_color(GRID)
        ax.set_facecolor(SURFACE)
    axes[0].set_ylabel("share of samples (%)", color=INK, fontsize=9)
    handles, labels = axes[0].get_legend_handles_labels()
    fig.legend(handles, labels, loc="lower center", ncol=len(labels), frameon=False,
               fontsize=8.5, labelcolor=INK)
    fig.patch.set_facecolor(SURFACE)
    fig.tight_layout(rect=(0, 0.08, 1, 1))
    fig.savefig(args.out_svg, facecolor=SURFACE)
    print(f"wrote {args.out_svg}")


if __name__ == "__main__":
    main()
