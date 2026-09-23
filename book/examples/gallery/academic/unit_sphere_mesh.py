#!/usr/bin/env python3
"""All-hex meshes of the unit sphere, from Sculpt and from automesh.

The script

1. converts each Sculpt mesh, `unit_sphere_sculpt_*.e.1.0`, to an Abaqus
   `.inp` file, since automesh 0.4.7 cannot read Sculpt's Exodus files,
2. meshes each marching-cubes surface with `automesh mesh hex`, on a uniform
   lattice at the Sculpt cell size and on the default octree, and
3. writes `automesh metrics` for every mesh and prints a summary.

Example
-------
cd ~/autotwin/automesh/book/examples/gallery/academic
# first the Sculpt commands on unit_sphere.md
uv run --with numpy --with scipy unit_sphere_mesh.py

Output
------
unit_sphere_sculpt_*.inp, unit_sphere_uniform_nNNN.inp,
unit_sphere_octree_nNNN.inp, a metrics CSV beside each mesh, and a table on
the terminal.
"""

import subprocess
from pathlib import Path

import numpy as np
from scipy.io import netcdf_file

RADII = (10, 20, 40, 80, 160)
CELL = 2 * 1.240409 / 26


def exodus_read(*, path: Path) -> tuple:
    """Returns the node coordinates and hex connectivity of a Sculpt Exodus file."""
    with netcdf_file(path, "r", mmap=False) as exodus:
        points = np.stack(
            [exodus.variables[f"coord{axis}"][:] for axis in "xyz"], axis=1
        ).astype(float)
        hexes = exodus.variables["connect1"][:].astype(int)
    return points, hexes


def inp_write(*, path: Path, points: np.ndarray, hexes: np.ndarray) -> None:
    """Writes a hex mesh as an Abaqus input file, with 1-based numbering."""
    lines = ["*NODE"]
    lines += [
        f"{i}, {x:.9e}, {y:.9e}, {z:.9e}" for i, (x, y, z) in enumerate(points, 1)
    ]
    lines.append("*ELEMENT, TYPE=C3D8R, ELSET=EB1")
    lines += [f"{i}, " + ", ".join(map(str, h)) for i, h in enumerate(hexes, 1)]
    path.write_text("\n".join(lines) + "\n")


def metrics_write(*, mesh: Path) -> Path:
    """Runs `automesh metrics` on a mesh and returns the path of its CSV."""
    csv = mesh.with_suffix(".csv")
    subprocess.run(
        ["automesh", "metrics", "-i", str(mesh), "-o", str(csv), "-q"], check=True
    )
    return csv


def hex_write(*, stl: Path, mesh: Path, options: list) -> Path:
    """Meshes a surface with `automesh mesh hex` and returns its metrics CSV."""
    csv = mesh.with_suffix(".csv")
    subprocess.run(
        ["automesh", "mesh", "hex", "-i", str(stl), "-o", str(mesh)]
        + options
        + ["--metrics", str(csv), "-q"],
        check=True,
    )
    return csv


def summary_print(*, label: str, csv: Path) -> None:
    """Prints the element count, volume, and quality extremes of one mesh."""
    metrics = np.genfromtxt(csv, delimiter=",", names=True)
    msj = metrics["minimum_scaled_jacobian"]
    print(
        f"{label:>18}  {len(msj):>8,}  {metrics['element_volume'].sum():7.4f}  "
        f"{msj.min():8.3f}  {msj.mean():6.3f}  "
        f"{metrics['maximum_edge_ratio'].max():7.2f}  "
        f"{metrics['maximum_skew'].max():6.3f}"
    )


def main() -> None:
    here = Path(__file__).parent
    print(
        f"{'mesh':>18}  {'elements':>8}  {'volume':>7}  {'MSJ min':>8}  "
        f"{'mean':>6}  {'MAR max':>7}  {'MS max':>6}"
    )
    for exodus in sorted(here.glob("unit_sphere_sculpt_*.e.1.0")):
        inp = here / exodus.name.replace(".e.1.0", ".inp")
        points, hexes = exodus_read(path=exodus)
        inp_write(path=inp, points=points, hexes=hexes)
        summary_print(
            label=inp.stem.removeprefix("unit_sphere_"), csv=metrics_write(mesh=inp)
        )
    for n in RADII:
        stl = here / f"unit_sphere_mc_n{n:03d}.stl"
        for name, options in (
            ("uniform", ["-u", f"{CELL:.6f}"]),
            ("octree", []),
        ):
            mesh = here / f"unit_sphere_{name}_n{n:03d}.inp"
            summary_print(
                label=f"{name}_n{n:03d}",
                csv=hex_write(stl=stl, mesh=mesh, options=options),
            )


if __name__ == "__main__":
    main()
