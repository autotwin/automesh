"""Edge valence of the elements of a hexahedral mesh, by minimum scaled Jacobian.

The valence of an edge is the number of hexahedra that share it.  An interior
edge of a regular grid has valence 4.  The script prints how many interior
elements in each quality band touch an interior edge of another valence, the
valences of the worst interior elements, and the quality range of the interior
elements that carry three or more edges of valence 7.

Usage
-----
    uv run --with numpy --with pandas --with meshio python bone_valence.py \\
        adaptive_improved.vtu adaptive_improved.csv
"""

import sys
from collections import Counter

import meshio
import numpy as np
import pandas as pd

EDGES = [(0, 1), (1, 2), (2, 3), (3, 0), (4, 5), (5, 6),
         (6, 7), (7, 4), (0, 4), (1, 5), (2, 6), (3, 7)]
FACES = [(0, 3, 2, 1), (4, 5, 6, 7), (0, 1, 5, 4),
         (1, 2, 6, 5), (2, 3, 7, 6), (3, 0, 4, 7)]


def boundary(cells, count):
    faces = Counter(tuple(sorted(h[list(f)])) for h in cells for f in FACES)
    on = np.zeros(count, bool)
    on[[n for face, k in faces.items() if k == 1 for n in face]] = True
    return on


def main(mesh, metrics):
    quality = pd.read_csv(metrics)["minimum scaled jacobian"].values
    cells = meshio.read(mesh).cells_dict["hexahedron"]
    on = boundary(cells, cells.max() + 1)
    valence = Counter(tuple(sorted((h[a], h[b]))) for h in cells for a, b in EDGES)

    def edges(h):
        return [valence[tuple(sorted((h[a], h[b])))] for a, b in EDGES]

    interior = np.array([not on[h].any() for h in cells])
    for low, high in [(0.0, 0.5), (0.5, 0.6), (0.6, 0.8), (0.8, 1.01)]:
        band = np.where(interior & (quality >= low) & (quality < high))[0]
        irregular = sum(any(v != 4 for v in edges(cells[i])) for i in band)
        print(f"MSJ {low:.1f} to {min(high, 1.0):.1f}: {len(band):5d} interior, "
              f"{irregular:5d} with an edge of valence other than 4")
    print("worst interior elements:")
    for i in [i for i in np.argsort(quality) if interior[i]][:10]:
        print(f"  {i:6d}  MSJ {quality[i]:.3f}  edge valences {sorted(edges(cells[i]))}")
    sevens = [i for i in np.where(interior)[0]
              if sum(v == 7 for v in edges(cells[i])) >= 3]
    print(f"interior elements with three or more edges of valence 7: {len(sevens)}, "
          f"MSJ {quality[sevens].min():.3f} to {quality[sevens].max():.3f}")


if __name__ == "__main__":
    main(*sys.argv[1:3])
