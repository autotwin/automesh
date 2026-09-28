#!/usr/bin/env python3
"""Taubin smoothing of the marching-cubes surfaces, then the default octree.

For each marching-cubes surface, the script

1. smooths the surface with `automesh smooth` (Taubin, default parameters)
   for each iteration count in ITERATIONS, where 0 leaves the surface as is,
2. measures the mean and standard deviation of the smoothed vertices'
   distance from the origin, which is 1 on the unit sphere,
3. meshes the smoothed surface with `automesh mesh hex` and its default
   octree, and
4. prints a table of the mesh size and quality.

The smoothed surfaces are large, so the script writes them to a temporary
directory and deletes them.  It keeps the metrics CSV of each mesh.  It also
keeps the mesh itself for the (n, iterations) pairs in KEPT, which
unit_sphere_figures.py draws.

Example
-------
cd ~/autotwin/automesh/book/examples/gallery/academic
uv run --with numpy unit_sphere_isosurface.py   # the input surfaces
uv run --with numpy unit_sphere_smooth.py

Output
------
unit_sphere_smooth_nNNN_itIII.csv, a metrics file for each mesh,
unit_sphere_smooth_nNNN_itIII.inp for each pair in KEPT, and a table on the
terminal.
"""

import subprocess
import tempfile
from pathlib import Path

import numpy as np

RADII = (10, 20, 40, 80, 160)
ITERATIONS = (0, 5, 10, 20, 50, 100, 200)
KEPT = ((160, 50), (160, 200))


def stl_vertices(*, path: Path) -> np.ndarray:
    """Returns the distinct vertices of a binary STL file."""
    records = np.fromfile(path, dtype=np.uint8)[84:].reshape(-1, 50)
    corners = records[:, 12:48].copy().view(np.float32).reshape(-1, 3)
    return np.unique(corners.astype(float), axis=0)


def surface_smooth(*, stl: Path, out: Path, iterations: int) -> Path:
    """Smooths a surface with `automesh smooth` and returns the path to use."""
    if iterations == 0:
        return stl
    subprocess.run(
        ["automesh", "smooth", "-i", str(stl), "-o", str(out)]
        + ["--iterations", str(iterations), "-q"],
        check=True,
    )
    return out


def octree_mesh(*, stl: Path, mesh: Path, csv: Path) -> None:
    """Meshes a surface with the default octree and writes its metrics."""
    subprocess.run(
        ["automesh", "mesh", "hex", "-i", str(stl), "-o", str(mesh)]
        + ["--metrics", str(csv), "-q"],
        check=True,
    )


def main() -> None:
    here = Path(__file__).parent
    print(
        f"{'n':>4}  {'iterations':>10}  {'radius mean':>11}  {'radius std':>10}  "
        f"{'elements':>8}  "
        f"{'MSJ min':>8}  {'mean':>6}  {'inverted':>8}"
    )
    with tempfile.TemporaryDirectory() as scratch:
        scratch = Path(scratch)
        for n in RADII:
            stl = here / f"unit_sphere_mc_n{n:03d}.stl"
            for iterations in ITERATIONS:
                smoothed = surface_smooth(
                    stl=stl, out=scratch / "smoothed.stl", iterations=iterations
                )
                radius = np.linalg.norm(stl_vertices(path=smoothed), axis=1)
                csv = here / f"unit_sphere_smooth_n{n:03d}_it{iterations:03d}.csv"
                mesh = scratch / "mesh.inp"
                if (n, iterations) in KEPT:
                    mesh = csv.with_suffix(".inp")
                octree_mesh(stl=smoothed, mesh=mesh, csv=csv)
                msj = np.genfromtxt(csv, delimiter=",", names=True)[
                    "minimum_scaled_jacobian"
                ]
                print(
                    f"{n:>4}  {iterations:>10}  {radius.mean():>11.5f}  "
                    f"{radius.std():>10.5f}  "
                    f"{len(msj):>8,}  {msj.min():>8.3f}  {msj.mean():>6.3f}  "
                    f"{(msj < 0).sum():>8}"
                )


if __name__ == "__main__":
    main()
