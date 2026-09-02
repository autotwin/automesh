#!/usr/bin/env python3
"""Overlapping quality-metric histograms for the RVE: tet4 vs. automesh hex.

Reads two `automesh metrics` CSVs (header "maximum edge ratio,minimum scaled
jacobian,maximum skew,element volume") and plots a 2x2 grid, one panel per
metric, each an overlapping log-y step histogram of the two meshes'
per-element values.

This replicates the figure style used in the autotwin/quality repository
(scripts/quality_histograms.py, figures/bone_baseline_quality_histograms.svg),
so the two studies read the same way: Minimum Scaled Jacobian top left,
Maximum Aspect Ratio top right, Maximum Skew bottom left, Element Volume
bottom right.

The tet4 mesh is drawn first, solid and slightly heavier.  The `automesh` hex
mesh is drawn second, dashed and thinner, so the two stay tellable apart where
they overlap.

Example
-------
cd ~/autotwin/automesh/book/examples/gallery/academic
# comparison.stl and tetrahedra_4.exo are not committed; download them first
# (see the Downloads section of rve.md), then:
automesh convert mesh -i tetrahedra_4.exo -o tetrahedra_4.inp
automesh mesh hex --input comparison.stl --output hexahedra.exo \
    --scale 10 --tolerance 1e-3
automesh metrics -i tetrahedra_4.inp -o tet4_metrics.csv
automesh metrics -i hexahedra.exo -o hex_metrics.csv
python rve_quality_histograms.py tet4_metrics.csv hex_metrics.csv \
    rve_quality_histograms.svg

Output
------
The `rve_quality_histograms.svg` visualization file.
"""
import sys

import matplotlib
matplotlib.use("svg")
import matplotlib.pyplot as plt
import numpy as np

# Same palette as the autotwin/quality figures: the first series stays orange,
# the second stays blue.
SERIES_R = "#eb6834"
SERIES_G = "#2a78d6"
INK = "#0b0b0b"
GRID = "#e4e3df"
SURFACE = "#fcfcfb"

COLUMNS = [
    ("minimum scaled jacobian", "Minimum Scaled Jacobian"),
    ("maximum edge ratio", "Maximum Aspect Ratio"),
    ("maximum skew", "Maximum Skew"),
    ("element volume", "Element Volume"),
]


def csv_load(*, path):
    """Reads an `automesh metrics` CSV into a dict keyed by column name."""
    with open(path) as f:
        header = f.readline().strip().split(",")
    data = np.genfromtxt(path, delimiter=",", skip_header=1)
    return {name: data[:, i] for i, name in enumerate(header)}


def panel_plot(*, ax, first, second, title, log_x=False,
               first_label="tet4 (Cubit)", second_label="automesh hex"):
    """Draws one metric panel as two overlapping step histograms."""
    lo = min(first.min(), second.min())
    hi = max(first.max(), second.max())
    if lo == hi:
        lo, hi = lo - 0.5, hi + 0.5
    if log_x:
        lo = max(lo, 1e-3)
        bins = np.logspace(np.log10(lo), np.log10(hi), 41)
        ax.set_xscale("log")
    else:
        bins = np.linspace(lo, hi, 41)
    # Unfilled step outlines with alpha on the line itself, so crossing and
    # overlapping outlines blend rather than hide one another.
    ax.hist(first, bins=bins, histtype="step", linewidth=2.0, color=SERIES_R,
            alpha=0.75, linestyle="-", label=first_label)
    ax.hist(second, bins=bins, histtype="step", linewidth=1.5, color=SERIES_G,
            alpha=0.75, linestyle="--", label=second_label)
    ax.set_yscale("log")
    ax.set_title(title, color=INK, fontsize=11)
    ax.set_xlabel(title, color=INK, fontsize=9)
    ax.set_ylabel("Element Count (int)", color=INK, fontsize=9)
    ax.tick_params(colors=INK, labelsize=8)
    ax.grid(color=GRID, linewidth=0.6)
    for spine in ax.spines.values():
        spine.set_color(GRID)
    ax.set_facecolor(SURFACE)


def main():
    if not 4 <= len(sys.argv) <= 6:
        print(f"usage: {sys.argv[0]} <tet4.csv> <hex.csv> <out.svg> "
              f"[<first label> <second label>]", file=sys.stderr)
        sys.exit(1)
    first_path, second_path, out_path = sys.argv[1], sys.argv[2], sys.argv[3]
    first_label = sys.argv[4] if len(sys.argv) > 4 else "tet4 (Cubit)"
    second_label = sys.argv[5] if len(sys.argv) > 5 else "automesh hex"

    first = csv_load(path=first_path)
    second = csv_load(path=second_path)

    fig, axes = plt.subplots(2, 2, figsize=(9, 7))
    fig.patch.set_facecolor(SURFACE)
    for ax, (key, title) in zip(axes.flat, COLUMNS):
        panel_plot(ax=ax, first=first[key], second=second[key], title=title,
                   log_x=(key == "maximum edge ratio"),
                   first_label=first_label, second_label=second_label)

    handles, labels = axes.flat[0].get_legend_handles_labels()
    fig.legend(handles, labels, loc="upper center", ncol=2, frameon=False,
               fontsize=9, bbox_to_anchor=(0.5, 1.0))
    fig.tight_layout(rect=[0, 0, 1, 0.93])
    fig.savefig(out_path, facecolor=SURFACE)
    print(f"{out_path}: {len(first[COLUMNS[0][0]])} tet4 elements, "
          f"{len(second[COLUMNS[0][0]])} hex elements")


if __name__ == "__main__":
    main()
