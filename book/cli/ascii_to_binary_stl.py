r"""This module, ascii_to_binary_stl.py, converts an ASCII STL file into a
binary STL file.  `automesh remesh` requires binary STL for both input and
output; ASCII STL is not accepted.

Example
-------
source ~/autotwin/automesh/.venv/bin/activate
cd ~/autotwin/automesh/book/cli
python ascii_to_binary_stl.py input_ascii.stl output_binary.stl
"""

import struct
import sys
from pathlib import Path


def ascii_to_binary_stl(source: Path, target: Path) -> None:
    """Reads an ASCII STL and writes an equivalent binary STL."""
    normal = (0.0, 0.0, 0.0)
    verts: list[tuple[float, float, float]] = []
    facets: list[tuple] = []
    for line in source.read_text().splitlines():
        tokens = line.split()
        if not tokens:
            continue
        if tokens[0] == "facet" and tokens[1] == "normal":
            normal = tuple(float(x) for x in tokens[2:5])
            verts = []
        elif tokens[0] == "vertex":
            verts.append(tuple(float(x) for x in tokens[1:4]))
        elif tokens[0] == "endfacet":
            facets.append((normal, verts[0], verts[1], verts[2]))

    with target.open("wb") as out:
        out.write(b"\0" * 80)  # 80-byte header (ignored)
        out.write(struct.pack("<I", len(facets)))  # facet count
        for n, a, b, c in facets:
            # 12 floats (normal + 3 vertices) then a 2-byte attribute count.
            out.write(struct.pack("<12fH", *n, *a, *b, *c, 0))
    print(f"wrote {len(facets)} facets to {target}")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit("usage: python ascii_to_binary_stl.py <ascii.stl> <binary.stl>")
    ascii_to_binary_stl(Path(sys.argv[1]), Path(sys.argv[2]))
