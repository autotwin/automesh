#!/usr/bin/env python3
"""Each marching-cubes surface, compared with the exact unit sphere.

The script prints three tables.

    Hausdorff  the farthest any point of the surface lies outside and inside
               the unit sphere.  Over a triangle, |p| is convex, so its largest
               value sits at a vertex, and its smallest at the point of the
               triangle closest to the origin.  The larger of the two is the
               Hausdorff distance, since each surface is closed and encloses
               the origin (see unit_sphere.md).
    area       the area of the marching-cubes triangles, against the exact
               area 4 pi.
    shapes     the fraction of triangles of each shape, grouped by the
               maximum edge ratio and minimum scaled Jacobian that
               `automesh metrics` reports.  It writes unit_sphere_mc_nNNN.csv.

Example
-------
cd ~/autotwin/automesh/book/examples/gallery/academic
uv run --with numpy unit_sphere_segmentation.py
uv run --with numpy --with scikit-image unit_sphere_isosurface.py
uv run --with numpy unit_sphere_comparison.py

Output
------
Three tables on the terminal, and unit_sphere_mc_nNNN.csv.
"""

import subprocess
from collections import Counter
from pathlib import Path

import numpy as np

RADII = (10, 20, 40, 80, 160)


def stl_read(*, path: Path) -> np.ndarray:
    """Returns the triangles of a binary STL, with shape (faces, 3, 3)."""
    record = np.dtype(
        [("normal", "<f4", 3), ("vertices", "<f4", (3, 3)), ("attribute", "<u2")]
    )
    with path.open("rb") as file:
        file.seek(80)
        count = int(np.frombuffer(file.read(4), dtype="<u4")[0])
        return np.frombuffer(file.read(), dtype=record, count=count)["vertices"]


def origin_distances(*, triangles: np.ndarray) -> np.ndarray:
    """Returns each triangle's distance to the origin (Ericson 2005, sec. 5.1.5)."""
    a, b, c = (triangles[:, i].astype(float) for i in range(3))
    ab, ac = b - a, c - a

    def dot(x: np.ndarray, y: np.ndarray) -> np.ndarray:
        return np.einsum("ij,ij->i", x, y)

    d1, d2 = -dot(ab, a), -dot(ac, a)
    d3, d4 = -dot(ab, b), -dot(ac, b)
    d5, d6 = -dot(ab, c), -dot(ac, c)
    va, vb, vc = d3 * d6 - d5 * d4, d5 * d2 - d1 * d6, d1 * d4 - d3 * d2
    with np.errstate(divide="ignore", invalid="ignore"):
        scale = 1.0 / (va + vb + vc)
        point = a + ab * (vb * scale)[:, None] + ac * (vc * scale)[:, None]
        regions = (
            (
                (va <= 0) & (d4 - d3 >= 0) & (d5 - d6 >= 0),
                b + (c - b) * ((d4 - d3) / ((d4 - d3) + (d5 - d6)))[:, None],
            ),
            ((vb <= 0) & (d2 >= 0) & (d6 <= 0), a + ac * (d2 / (d2 - d6))[:, None]),
            ((d6 >= 0) & (d5 <= d6), c),
            ((vc <= 0) & (d1 >= 0) & (d3 <= 0), a + ab * (d1 / (d1 - d3))[:, None]),
            ((d3 >= 0) & (d4 <= d3), b),
            ((d1 <= 0) & (d2 <= 0), a),
        )
        for mask, corner in regions:
            point = np.where(mask[:, None], corner, point)
    return np.linalg.norm(point, axis=1)


def triangles_area(*, triangles: np.ndarray) -> float:
    """Returns the total area of the triangles."""
    t = triangles.astype(float)
    return float(
        0.5
        * np.linalg.norm(np.cross(t[:, 1] - t[:, 0], t[:, 2] - t[:, 0]), axis=1).sum()
    )


def shape_fractions(*, path: Path) -> dict:
    """Returns the fraction of triangles per (edge ratio, scaled Jacobian) pair."""
    csv = path.with_suffix(".csv")
    subprocess.run(
        ["automesh", "metrics", "-i", str(path), "-o", str(csv), "-q"], check=True
    )
    metrics = np.genfromtxt(csv, delimiter=",", names=True)
    pairs = zip(
        np.round(metrics["maximum_edge_ratio"], 3),
        np.round(metrics["minimum_scaled_jacobian"], 3),
        strict=True,
    )
    counts = Counter((float(ratio), float(jacobian)) for ratio, jacobian in pairs)
    return {key: count / len(metrics) for key, count in counts.items()}


def main() -> None:
    here = Path(__file__).parent
    exact = 4.0 * np.pi
    print(
        f"{'n':>3}  {'f':>7}  {'outward':>8}  {'inward':>8}  {'Hausdorff':>9}  "
        f"{'n x Hausdorff':>13}"
    )
    for n in RADII:
        triangles = stl_read(path=here / f"unit_sphere_mc_n{n:03d}.stl")
        outward = float(np.linalg.norm(triangles, axis=2).max()) - 1.0
        inward = 1.0 - float(origin_distances(triangles=triangles).min())
        hausdorff = max(outward, inward)
        print(
            f"{n:>3}  {len(triangles):>7,}  {outward:8.5f}  {inward:8.5f}  "
            f"{hausdorff:9.5f}  {n * hausdorff:13.3f}"
        )

    print()
    print(f"{'n':>3}  {'area':>8}  {'error':>7}")
    for n in RADII:
        triangles = stl_read(path=here / f"unit_sphere_mc_n{n:03d}.stl")
        area = triangles_area(triangles=triangles)
        print(f"{n:>3}  {area:8.4f}  {(area - exact) / exact:+7.2%}")
    print(f"exact area 4π = {exact:.4f}")

    print()
    shapes = {
        n: shape_fractions(path=here / f"unit_sphere_mc_n{n:03d}.stl") for n in RADII
    }
    keys = sorted({key for counts in shapes.values() for key in counts}, reverse=True)
    columns = "  ".join(f"{f'n={n}':>7}" for n in RADII)
    print(f"{'edge ratio':>10}  {'scaled Jacobian':>15}  {columns}")
    for key in keys:
        fractions = "  ".join(f"{shapes[n].get(key, 0.0):7.2%}" for n in RADII)
        print(f"{key[0]:10.3f}  {key[1]:15.3f}  {fractions}")


if __name__ == "__main__":
    main()
