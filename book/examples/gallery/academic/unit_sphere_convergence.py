#!/usr/bin/env python3
"""The volume of the voxel sphere and of its marching cubes surface, against n.

For every n from 4 to 160, the script

1. builds the segmentation of a sphere of radius `n` voxels,
2. runs `automesh mesh tri --cubes marching` on it, which scales the surface to
   the unit sphere,
3. adds up the signed volume that the surface encloses, and
4. counts the inside voxels and multiplies by 1/n^3.

It prints the sign changes, the number of volumes above 4 pi / 3, and the
log-log slope of each error, and draws both volumes and both errors in
`unit_sphere_convergence.png`.  The surfaces live in a scratch directory.
Only the figure stays.

Example
-------
cd ~/autotwin/automesh/book/examples/gallery/academic
uv run --with numpy --with matplotlib unit_sphere_convergence.py

Output
------
unit_sphere_convergence.png and a table on the terminal.
"""

import subprocess
import tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np

SWEEP = range(4, 161)
TABLE = (10, 20, 40, 80, 160)
SERIES = ("#2a78d6", "#eb6834")
INK = "#3d3d3a"
MUTED = "#8a8a85"
GRID = "#e6e5e0"
DPI = 200


def sphere(*, radius: int) -> np.ndarray:
    """Returns the segmentation of a sphere of `radius` voxels, axes (x, y, z)."""
    k = np.arange(-radius, radius + 1)
    x, y, z = np.meshgrid(k, k, k, indexing="ij", sparse=True)
    return (x * x + y * y + z * z <= radius * radius).astype(np.uint8)


def stl_read(*, path: Path) -> np.ndarray:
    """Reads a binary STL, returns the triangles, shape (faces, 3, 3)."""
    data = np.fromfile(path, dtype=np.uint8)
    record = np.dtype(
        [("normal", "<f4", 3), ("vertices", "<f4", (3, 3)), ("attribute", "<u2")]
    )
    return np.frombuffer(data[84:].tobytes(), dtype=record)["vertices"].astype(float)


def volume_signed(*, triangles: np.ndarray) -> float:
    """Returns the signed volume, positive when the faces wind outward."""
    a, b, c = triangles[:, 0], triangles[:, 1], triangles[:, 2]
    return float(np.einsum("ij,ij->i", a, np.cross(b, c)).sum() / 6.0)


def volumes_measure(*, radius: int) -> tuple:
    """Returns the voxel volume and the marching cubes volume for `radius`."""
    voxels = sphere(radius=radius)
    scale = 1.0 / radius
    translate = -(radius + 0.5) / radius
    with tempfile.TemporaryDirectory() as scratch:
        npy = Path(scratch) / "sphere.npy"
        stl = Path(scratch) / "sphere.stl"
        np.save(npy, voxels)
        command = ["automesh", "mesh", "tri", "-i", str(npy), "-o", str(stl)]
        command += ["--cubes", "marching", "-q"]
        for axis in "xyz":
            command += [f"--{axis}scale", repr(scale)]
            command += [f"--{axis}translate", repr(translate)]
        subprocess.run(command, check=True)
        surface = volume_signed(triangles=stl_read(path=stl))
    return voxels.sum() / radius**3, surface


def volumes_sweep() -> tuple:
    """Returns the voxel and marching cubes volumes for every n in SWEEP."""
    with ThreadPoolExecutor() as pool:
        rows = list(pool.map(lambda n: volumes_measure(radius=n), SWEEP))
    return np.array([row[0] for row in rows]), np.array([row[1] for row in rows])


def convergence_plot(*, output: Path) -> None:
    """Draws the volume and its error against n, for voxels and marching cubes."""
    exact = 4.0 * np.pi / 3.0
    ns = np.array(SWEEP)
    table = np.isin(ns, TABLE)
    series = dict(zip(("voxels", "marching cubes"), volumes_sweep(), strict=True))
    print(f"n from {ns[0]} to {ns[-1]}, {len(ns)} values")
    for name, volume in series.items():
        signed = volume - exact
        slope = np.polyfit(np.log(ns), np.log(np.abs(signed)), 1)[0]
        print(
            f"{name:>14}: sign changes {np.sum(np.diff(np.sign(signed)) != 0)}, "
            f"above 4π/3 {np.sum(signed > 0)}, log-log slope {slope:.2f}"
        )
    below = np.sum(series["marching cubes"] < series["voxels"])
    print(f"marching cubes below voxels at {below} of {len(ns)} values")
    styles = {
        "voxels": {"color": SERIES[0], "marker": "s", "linestyle": "-"},
        "marching cubes": {"color": SERIES[1], "marker": "o", "linestyle": "--"},
    }
    fig, (left, right) = plt.subplots(1, 2, figsize=(10, 4.2))
    left.axhline(exact, color=INK, linewidth=1.0, linestyle=":")
    left.text(4.2, exact + 0.004, "exact 4π/3", color=INK, fontsize=9)
    for name, volume in series.items():
        style = styles[name]
        error = 100.0 * np.abs(volume - exact) / exact
        for ax, values in ((left, volume), (right, error)):
            ax.plot(
                ns,
                values,
                color=style["color"],
                linestyle=style["linestyle"],
                linewidth=1.0,
                alpha=0.45,
            )
            ax.plot(
                ns[table],
                values[table],
                color=style["color"],
                marker=style["marker"],
                markersize=6,
                alpha=0.6 if ax is left else 1.0,
                linestyle="none",
                label=name,
            )
    for slope, anchor in ((-1, 3.0), (-2, 3.0)):
        guide = anchor * (ns / ns[0]) ** slope
        right.plot(ns, guide, color=MUTED, linewidth=0.8)
        right.text(ns[-1] * 1.03, guide[-1], f"slope {slope}", color=MUTED, fontsize=8)
    left.set_xscale("log")
    left.set_ylim(3.95, 4.25)
    left.set_ylabel("volume")
    left.set_title("Volume approaches 4π/3")
    right.set_xscale("log")
    right.set_yscale("log")
    right.set_ylim(1e-4, 10.0)
    right.set_ylabel("|error| (%)")
    right.set_title("|Error| falls toward zero")
    for ax in (left, right):
        ax.set_xlabel("n (voxels per unit radius)")
        ax.set_xticks(
            [4, 10, 20, 40, 80, 160], labels=["4", "10", "20", "40", "80", "160"]
        )
        ax.minorticks_off()
        ax.grid(color=GRID, linewidth=0.6)
        ax.set_axisbelow(True)
        for spine in ("top", "right"):
            ax.spines[spine].set_visible(False)
    right.set_xlim(right=ns[-1] * 1.6)
    left.legend(frameon=False, loc="lower right")
    fig.tight_layout()
    fig.savefig(output, dpi=DPI)
    plt.close(fig)


if __name__ == "__main__":
    convergence_plot(output=Path(__file__).resolve().parent / "unit_sphere_convergence.png")
