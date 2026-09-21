#!/usr/bin/env python3
"""Compares a reproduced bone mesh with the published one.

Both files are legacy ASCII VTK hex meshes.  The script reports four things.

    counts        the number of nodes and elements
    connectivity  whether the two meshes have the same elements, in the same order
    positions     how far the nodes of one mesh sit from those of the other,
                  split into boundary nodes and interior nodes
    quality       the extremes of the four Verdict measures, from `automesh metrics`

A node is a boundary node when it belongs to a face that only one element
uses.

Example
-------
cd ~/autotwin/automesh/book/examples/gallery/academic
# reproduced_bone.vtk is the finalMesh.vtk of a HybridOctree_Hex run;
# see the Reproducing the Baseline section of bone.md
python3 bone_baseline_compare.py reproduced_bone.vtk bone.vtk

Output
------
A short report on the terminal.
"""
import csv
import math
import statistics
import subprocess
import sys
import tempfile
from collections import Counter
from pathlib import Path

# The six faces of a hex, in the node order of an Abaqus C3D8 element.
FACES = [(0, 3, 2, 1), (4, 5, 6, 7), (0, 1, 5, 4), (1, 2, 6, 5), (2, 3, 7, 6), (3, 0, 4, 7)]
TOLERANCE = 1e-6


def vtk_load(*, path):
    """Returns the points and the hex cells of a legacy ASCII VTK file."""
    lines = Path(path).read_text().split("\n")
    start = next(i for i, line in enumerate(lines) if line.startswith("POINTS"))
    count = int(lines[start].split()[1])
    points = [tuple(float(v) for v in lines[start + 1 + i].split()) for i in range(count)]
    start = next(i for i, line in enumerate(lines) if line.startswith("CELLS"))
    count = int(lines[start].split()[1])
    cells = [tuple(int(v) for v in lines[start + 1 + i].split()[1:]) for i in range(count)]
    return points, cells


def boundary_nodes(*, cells):
    """Returns the set of nodes that belong to a face only one element uses."""
    uses = Counter(tuple(sorted(cell[i] for i in face)) for cell in cells for face in FACES)
    return {node for key, count in uses.items() if count == 1 for node in key}


def metrics_compute(*, points, cells, directory):
    """Returns the columns of the `automesh metrics` CSV of a hex mesh."""
    deck = Path(directory) / "mesh.inp"
    with open(deck, "w") as f:
        f.write("*Heading\n*Node\n")
        f.writelines(f"{i + 1}, {x}, {y}, {z}\n" for i, (x, y, z) in enumerate(points))
        f.write("*Element, type=C3D8, elset=BLOCK1\n")
        f.writelines(f"{i + 1}, " + ", ".join(str(n + 1) for n in cell) + "\n"
                     for i, cell in enumerate(cells))
    out = Path(directory) / "mesh.csv"
    subprocess.run(["automesh", "metrics", "-q", "-i", str(deck), "-o", str(out)], check=True)
    rows = list(csv.reader(open(out)))[1:]
    return [[float(row[i]) for row in rows] for i in range(4)]


def summary(*, columns):
    ratio, msj, skew, volume = columns
    ordered = sorted(msj)
    n = len(ordered)
    return {
        "elements": n,
        "min Minimum Scaled Jacobian": ordered[0],
        "5th percentile": ordered[int(0.05 * (n - 1))],
        "median": statistics.median(msj),
        "max Maximum Aspect Ratio": max(ratio),
        "elements above ratio 10": sum(r > 10 for r in ratio),
        "max Maximum Skew": max(skew),
        "max Element Volume": max(volume),
    }


def main():
    if len(sys.argv) != 3:
        print(f"usage: {sys.argv[0]} <reproduced.vtk> <published.vtk>", file=sys.stderr)
        sys.exit(1)
    ours_points, ours_cells = vtk_load(path=sys.argv[1])
    theirs_points, theirs_cells = vtk_load(path=sys.argv[2])
    print(f"reproduced: {len(ours_points)} nodes, {len(ours_cells)} elements")
    print(f"published:  {len(theirs_points)} nodes, {len(theirs_cells)} elements")
    same = ours_cells == theirs_cells
    print(f"same elements, in the same order: {same}")
    if same:
        boundary = boundary_nodes(cells=theirs_cells)
        distance = [math.dist(a, b) for a, b in zip(ours_points, theirs_points)]
        for name, members in (("all nodes", range(len(distance))),
                              ("boundary nodes", sorted(boundary)),
                              ("interior nodes", sorted(set(range(len(distance))) - boundary))):
            values = [distance[i] for i in members]
            identical = sum(1 for d in values if d < TOLERANCE)
            print(f"  {name:15} {len(values):6}  identical to 6 digits {identical:6}  "
                  f"mean distance {statistics.mean(values):.5f}  max {max(values):.4f}")
    with tempfile.TemporaryDirectory() as directory:
        ours = summary(columns=metrics_compute(points=ours_points, cells=ours_cells, directory=directory))
        theirs = summary(columns=metrics_compute(points=theirs_points, cells=theirs_cells, directory=directory))
    print(f"{'quality':30} {'reproduced':>12} {'published':>12}")
    for key in ours:
        print(f"  {key:28} {ours[key]:>12.7g} {theirs[key]:>12.7g}")


if __name__ == "__main__":
    main()
