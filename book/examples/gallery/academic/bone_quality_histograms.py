#!/usr/bin/env python3
"""Quality-metric histograms for the bone mesh, one to three meshes.

Reads `automesh metrics` CSVs (header "maximum edge ratio,minimum scaled
jacobian,maximum skew,element volume") and plots a 2x2 grid.  Each panel is
a log-y step histogram of one metric.  Minimum Scaled Jacobian sits top left,
Maximum Aspect Ratio top right, Maximum Skew bottom left, and Element Volume
bottom right.

The first mesh is drawn solid and slightly heavier, in orange.  The second
mesh is drawn dashed and thinner, in blue.  The third is drawn dotted, in
green.  The lines stay tellable apart where they overlap.

The script also prints a summary of each mesh: the extreme of each metric, the
5th percentile and median of the Minimum Scaled Jacobian, the number of
elements below 0.6, and the number of elements above an aspect ratio of 10.

Example
-------
cd ~/autotwin/automesh/book/examples/gallery/academic
automesh metrics -i bone.inp -o bone_reference_metrics.csv
uv run --with numpy --with matplotlib bone_quality_histograms.py \
    bone_quality_histograms.svg \
    --mesh bone_reference_metrics.csv "Tong 2024"

uv run --with numpy --with matplotlib bone_quality_histograms.py \
    bone_quality_comparison.svg \
    --mesh bone_reference_metrics.csv "Tong 2024" \
    --mesh bone_uniform_metrics.csv "automesh, uniform" \
    --mesh bone_octree_metrics.csv "automesh, octree"

Output
------
The `bone_quality_histograms.svg` visualization file.
"""
import argparse

import matplotlib
matplotlib.use("svg")
import matplotlib.pyplot as plt
import numpy as np

# The first series stays orange, the second blue, and the third green.
SERIES = [
    {"color": "#eb6834", "linewidth": 2.0, "linestyle": "-"},
    {"color": "#2a78d6", "linewidth": 1.5, "linestyle": "--"},
    {"color": "#2f9e5b", "linewidth": 1.5, "linestyle": ":"},
]
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


def panel_plot(*, ax, series, title, log_x=False):
    """Draws one metric panel as overlapping step histograms."""
    lo = min(values.min() for _, values in series)
    hi = max(values.max() for _, values in series)
    if lo == hi:
        lo, hi = lo - 0.5, hi + 0.5
    if log_x:
        lo = max(lo, 1e-3)
        bins = np.logspace(np.log10(lo), np.log10(hi), 41)
        ax.set_xscale("log")
    else:
        bins = np.linspace(lo, hi, 41)
    # Unfilled outlines with alpha on the line, so crossing outlines blend.
    for (label, values), style in zip(series, SERIES):
        ax.hist(values, bins=bins, histtype="step", alpha=0.75, label=label,
                **style)
    ax.set_yscale("log")
    ax.set_title(title, color=INK, fontsize=11)
    ax.set_xlabel(title, color=INK, fontsize=9)
    ax.set_ylabel("Element Count (int)", color=INK, fontsize=9)
    ax.tick_params(colors=INK, labelsize=8)
    ax.grid(color=GRID, linewidth=0.6)
    for spine in ax.spines.values():
        spine.set_color(GRID)
    ax.set_facecolor(SURFACE)


def summary_print(*, label, metrics):
    """Prints the extremes of each metric for one mesh."""
    msj = metrics["minimum scaled jacobian"]
    n = len(msj)
    below = int((msj < 0.6).sum())
    print(f"{label}: {n} elements")
    print(f"  min Minimum Scaled Jacobian {msj.min():.7f}")
    print(f"  5th percentile / median     {np.percentile(msj, 5):.3f} / {np.median(msj):.3f}")
    print(f"  elements below 0.6          {below} ({100 * below / n:.1f}%)")
    print(f"  max Maximum Aspect Ratio    {metrics['maximum edge ratio'].max():.2f}")
    print(f"  elements above ratio 10     {int((metrics['maximum edge ratio'] > 10).sum())}")
    print(f"  max Maximum Skew            {metrics['maximum skew'].max():.4f}")
    print(f"  max Element Volume          {metrics['element volume'].max():.2f}")
    print(f"  total volume                {metrics['element volume'].sum():.1f}")


def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("out_svg")
    parser.add_argument("--mesh", nargs=2, action="append", required=True,
                        metavar=("CSV", "LABEL"),
                        help="an `automesh metrics` CSV and its legend label; "
                             "give it one to three times")
    args = parser.parse_args()
    if len(args.mesh) > len(SERIES):
        parser.error(f"at most {len(SERIES)} meshes")

    meshes = [(label, csv_load(path=path)) for path, label in args.mesh]

    fig, axes = plt.subplots(2, 2, figsize=(9, 7))
    fig.patch.set_facecolor(SURFACE)
    for ax, (key, title) in zip(axes.flat, COLUMNS):
        panel_plot(ax=ax, series=[(label, m[key]) for label, m in meshes],
                   title=title, log_x=(key == "maximum edge ratio"))

    handles, labels = axes.flat[0].get_legend_handles_labels()
    fig.legend(handles, labels, loc="upper center", ncol=len(labels),
               frameon=False, fontsize=9, bbox_to_anchor=(0.5, 1.0))
    fig.tight_layout(rect=[0, 0, 1, 0.93])
    fig.savefig(args.out_svg, facecolor=SURFACE)
    for label, metrics in meshes:
        summary_print(label=label, metrics=metrics)
    print(f"wrote {args.out_svg}")


if __name__ == "__main__":
    main()
