"""The local optimum of the minimum scaled Jacobian around the worst interior elements.

For each seed element, the patch is every element within 1 to 4 node rings of
it.  A node is free when it is interior to the mesh and every element around
it lies in the patch.  Every other node is fixed.  SLSQP maximizes t subject
to every corner scaled Jacobian that depends on a free node being at least t.
Each ring starts from the best positions of the ring before.  The seeds are the
worst interior elements that share no node with a worse seed.

The optimizer is local and uses finite-difference gradients, so the result is
the best found, not a bound.  Ten seeds take about 30 minutes.

Usage
-----
    uv run --with numpy --with scipy --with pandas --with meshio \\
        python bone_local_optimum.py adaptive_improved.vtu adaptive_improved.csv 10
"""

import sys
from collections import Counter

import meshio
import numpy as np
import pandas as pd
from scipy.optimize import minimize

ADJACENT = np.array([(1, 3, 4), (2, 0, 5), (3, 1, 6), (0, 2, 7),
                     (7, 5, 0), (4, 6, 1), (5, 7, 2), (6, 4, 3)])
FACES = [(0, 3, 2, 1), (4, 5, 6, 7), (0, 1, 5, 4),
         (1, 2, 6, 5), (2, 3, 7, 6), (3, 0, 4, 7)]


def boundary(cells, count):
    faces = Counter(tuple(sorted(h[list(f)])) for h in cells for f in FACES)
    on = np.zeros(count, bool)
    on[[n for face, k in faces.items() if k == 1 for n in face]] = True
    return on


def corners(x, cells, mask):
    p = x[cells]
    e = p[:, ADJACENT, :] - p[:, :, None, :]
    e = e / np.linalg.norm(e, axis=3, keepdims=True)
    value = np.einsum("mci,mci->mc", np.cross(e[:, :, 0], e[:, :, 1]), e[:, :, 2])
    return value.ravel()[mask]


def main(mesh, metrics, count):
    quality = pd.read_csv(metrics)["minimum scaled jacobian"].values
    m = meshio.read(mesh)
    cells = m.cells_dict["hexahedron"]
    points = m.points
    on = boundary(cells, len(points))
    around = [[] for _ in points]
    for i, h in enumerate(cells):
        for n in h:
            around[n].append(i)
    seeds, taken = [], set()
    for i in np.argsort(quality):
        if not on[cells[i]].any() and not taken & set(cells[i]):
            seeds.append(i)
            taken |= set(cells[i])
        if len(seeds) == count:
            break
    print(f"MSJ floor of the mesh {quality.min():.3f}")
    for seed in seeds:
        row = [f"seed {seed} (MSJ {quality[seed]:.3f})"]
        best = points.copy()
        patch = {seed}
        for rings in (1, 2, 3, 4):
            patch |= {e for el in list(patch) for n in cells[el] for e in around[n]}
            free = sorted({n for el in patch for n in cells[el]
                           if not on[n] and all(e in patch for e in around[n])})
            touched = cells[sorted({e for n in free for e in around[n]})]
            is_free = np.zeros(len(points), bool)
            is_free[free] = True
            nodes = np.concatenate([touched[:, :, None], touched[:, ADJACENT]], axis=2)
            mask = is_free[nodes].any(axis=2).ravel()
            start = corners(best, touched, mask).min()

            def unpack(z):
                x = best.copy()
                x[free] = z[:-1].reshape(-1, 3)
                return x

            result = minimize(
                lambda z: -z[-1],
                np.append(best[free].ravel(), start),
                jac=lambda z: np.append(np.zeros(len(z) - 1), -1.0),
                constraints=[{"type": "ineq",
                              "fun": lambda z: corners(unpack(z), touched, mask) - z[-1]}],
                method="SLSQP",
                options={"maxiter": 500, "ftol": 1e-9},
            )
            x = unpack(result.x)
            reached = corners(x, touched, mask).min()
            if reached > start:
                best = x
            row.append(f"ring {rings}: {len(free)} free, {max(reached, start):.3f}")
        print("  " + " | ".join(row), flush=True)


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2], int(sys.argv[3]))
