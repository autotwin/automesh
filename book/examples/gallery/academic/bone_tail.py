#!/usr/bin/env python3
"""Reports where the low-quality elements of a hex mesh sit.

A hex face that only one element uses lies on the boundary of the mesh.  The
script groups the elements by how many boundary faces they have.  It reports
the Minimum Scaled Jacobian of each group, and how many of the elements below
a threshold touch the boundary.

The mesh can be any file that `automesh convert mesh` reads.  The script
converts it to an Abaqus deck first, and reads the elements from that.  The
CSV is an `automesh metrics` CSV, with one row per element in the order of
the mesh.  The `--metrics` option of `automesh mesh hex` writes one.

Example
-------
cd ~/autotwin/automesh/book/examples/gallery/academic
python3 bone_tail.py bone.inp bone_reference_metrics.csv
python3 bone_tail.py bone_uniform.vtu bone_uniform_metrics.csv

Output
------
A short report on the terminal.
"""
import csv
import statistics
import subprocess
import sys
import tempfile
from collections import Counter
from pathlib import Path

# The six faces of a hex, in the node order of an Abaqus C3D8 element.
FACES = [(0, 3, 2, 1), (4, 5, 6, 7), (0, 1, 5, 4), (1, 2, 6, 5), (2, 3, 7, 6), (3, 0, 4, 7)]
THRESHOLDS = (0.6, 0.5, 0.4)


def elements_read(*, mesh):
    """Returns the node lists of the elements of `mesh`, in file order."""
    path = Path(mesh)
    with tempfile.TemporaryDirectory() as directory:
        if path.suffix != ".inp":
            deck = Path(directory) / "mesh.inp"
            subprocess.run(["automesh", "convert", "mesh", "-q", "-i", str(path), "-o", str(deck)],
                           check=True)
            path = deck
        lines = path.read_text().split("\n")
    start = next(i for i, line in enumerate(lines) if line.upper().startswith("*ELEMENT"))
    return [tuple(int(token) for token in line.split(",")[1:])
            for line in lines[start + 1:] if line.strip() and not line.startswith("*")]


def boundary_faces_count(*, elements):
    """Returns the number of boundary faces of each element."""
    uses = Counter(tuple(sorted(element[i] for i in face))
                   for element in elements for face in FACES)
    return [sum(1 for face in FACES if uses[tuple(sorted(element[i] for i in face))] == 1)
            for element in elements]


def group_print(*, name, msj, members):
    if not members:
        print(f"  {name:28} {0:6} elements")
        return
    values = [msj[i] for i in members]
    print(f"  {name:28} {len(values):6} elements ({len(values) / len(msj):5.1%})  "
          f"min {min(values):.3f}  median {statistics.median(values):.3f}")


def main():
    if len(sys.argv) != 3:
        print(f"usage: {sys.argv[0]} <mesh> <metrics.csv>", file=sys.stderr)
        sys.exit(1)
    elements = elements_read(mesh=sys.argv[1])
    with open(sys.argv[2]) as f:
        rows = list(csv.reader(f))
    column = rows[0].index("minimum scaled jacobian")
    msj = [float(row[column]) for row in rows[1:]]
    if len(msj) != len(elements):
        sys.exit(f"the mesh has {len(elements)} elements, and the CSV has {len(msj)} rows")
    faces = boundary_faces_count(elements=elements)
    n = len(elements)
    touching = sum(1 for count in faces if count)
    print(f"{sys.argv[1]}: {n} elements, {touching} touch the boundary ({touching / n:.1%})")
    group_print(name="interior", msj=msj, members=[i for i in range(n) if faces[i] == 0])
    group_print(name="one boundary face", msj=msj, members=[i for i in range(n) if faces[i] == 1])
    group_print(name="two or more boundary faces", msj=msj, members=[i for i in range(n) if faces[i] >= 2])
    for threshold in THRESHOLDS:
        low = [i for i in range(n) if msj[i] < threshold]
        if low:
            on = sum(1 for i in low if faces[i])
            print(f"  below {threshold}: {len(low)} elements, {on} on the boundary "
                  f"({on / len(low):.0%}), {len(low) - on} interior")
        else:
            print(f"  below {threshold}: none")


if __name__ == "__main__":
    main()
