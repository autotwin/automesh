#!/usr/bin/env python3
"""Taubin smoothing of the n = 10 marching cubes surface, and its effect on the
minimum scaled Jacobian of the `automesh` hex meshes.

The script

1. builds the n = 10 marching cubes surface of the unit sphere,
2. smooths it with `automesh smooth` (Taubin, default parameters) for each
   iteration count in `ITERATIONS`, where 0 leaves the surface as is,
3. meshes each smoothed surface with `automesh mesh hex`, on a uniform lattice
   and with the adaptive default,
4. prints a table of each mesh,
5. draws the minimum scaled Jacobian of both meshes against the iteration
   count in `unit_sphere_smooth_sweep.png`, and
6. for each mesh, draws the smoothed surface, the mesh, and the mesh cut at
   z = 0, at the iteration count where its minimum scaled Jacobian peaks up to
   `LIMIT` iterations, in `unit_sphere_smooth_uniform.png` and
   `unit_sphere_smooth_adaptive.png`, and
7. draws the four quality histograms of those two meshes together in
   `unit_sphere_smooth_quality.png`.

Taubin smoothing with the default parameters inflates this surface slowly.  At
`LIMIT` = 300 iterations the enclosed volume is 6.8% above the sphere's.  The
peak search stops there.  The sweep plot still shows every iteration count.

The script imports its drawing functions from `unit_sphere_meshes.py`, in the
same directory.  It writes the surfaces and meshes to a scratch directory and
keeps only the figures.

Example
-------
cd ~/autotwin/automesh/book/examples/gallery/academic
uv run --with numpy --with matplotlib unit_sphere_smooth.py

Output
------
unit_sphere_smooth_sweep.png, unit_sphere_smooth_uniform.png,
unit_sphere_smooth_adaptive.png, unit_sphere_smooth_quality.png, and a table
on the terminal.
"""

import argparse
import json
import subprocess
import tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np

from unit_sphere_meshes import (
    CELL,
    DPI,
    GRID,
    HISTOGRAM,
    INK,
    N,
    automesh_run,
    axes_set,
    colorbar_add,
    mesh_draw,
    polygons_draw,
    quality_plot,
    sculpt_metrics,
    surface_write,
)

ITERATIONS = (
    tuple(range(101)) + tuple(range(110, 301, 10)) + tuple(range(350, 1001, 50))
)
CASES = (("uniform", CELL), ("adaptive", None))
EXACT = 4.0 * np.pi / 3.0
WORKERS = 3
LIMIT = 300
MUTED = "#8a8a85"


def count(*, stem: Path) -> int:
    """Returns the number of hexes in the `.csv` file of a mesh."""
    return len(np.genfromtxt(stem.with_suffix(".csv"), delimiter=",", names=True))


def stl_read(*, path: Path) -> np.ndarray:
    """Returns the triangles of a binary STL, with shape (faces, 3, 3)."""
    record = np.dtype(
        [("normal", "<f4", 3), ("vertices", "<f4", (3, 3)), ("attribute", "<u2")]
    )
    count = int(np.frombuffer(path.read_bytes(), dtype="<u4", count=1, offset=80)[0])
    data = np.frombuffer(path.read_bytes(), dtype=record, count=count, offset=84)
    return data["vertices"].astype(float)


def volume_signed(*, triangles: np.ndarray) -> float:
    """Returns the signed volume, positive when the faces wind outward."""
    a, b, c = triangles[:, 0], triangles[:, 1], triangles[:, 2]
    return float(np.einsum("ij,ij->i", a, np.cross(b, c)).sum() / 6.0)


def surface_smooth(*, stl: Path, scratch: Path, iterations: int) -> Path:
    """Smooths a surface with `automesh smooth`, returns the smoothed STL."""
    out = scratch / f"smooth_iter{iterations:03d}.stl"
    if iterations == 0:
        out.write_bytes(stl.read_bytes())
    else:
        subprocess.run(
            ["automesh", "smooth", "-i", str(stl), "-o", str(out)]
            + ["--iterations", str(iterations), "-q"],
            check=True,
        )
    return out


def iteration_run(*, stl: Path, scratch: Path, iterations: int) -> dict:
    """Smooths the surface and meshes it both ways, returns the measurements."""
    smooth = surface_smooth(stl=stl, scratch=scratch, iterations=iterations)
    triangles = stl_read(path=smooth)
    row = {
        "iterations": iterations,
        "radius_mean": float(np.linalg.norm(triangles.reshape(-1, 3), axis=1).mean()),
        "volume": volume_signed(triangles=triangles),
    }
    for name, uniform in CASES:
        stem = automesh_run(
            stl=smooth, name=f"{name}_iter{iterations:03d}", uniform=uniform
        )
        msj = np.genfromtxt(stem.with_suffix(".csv"), delimiter=",", names=True)[
            "minimum_scaled_jacobian"
        ]
        row[name] = {
            "hexes": len(msj),
            "min": float(msj.min()),
            "mean": float(msj.mean()),
            "inverted": int((msj <= 0.0).sum()),
        }
    return row


def peak_find(*, rows: list, name: str, limit: int | None = None) -> dict:
    """Returns the row where the minimum scaled Jacobian of `name` is largest.

    With `limit`, only rows up to that many iterations compete.  A tie goes to
    the smaller iteration count.
    """
    pool = [row for row in rows if limit is None or row["iterations"] <= limit]
    return max(pool, key=lambda row: (row[name]["min"], -row["iterations"]))


def table_print(*, rows: list) -> None:
    """Prints one line per iteration count."""
    print(
        f"{'iterations':>10}  {'radius':>7}  {'volume':>7}  "
        f"{'uniform hexes':>13}  {'min':>6}  {'mean':>6}  "
        f"{'adaptive hexes':>14}  {'min':>6}  {'mean':>6}"
    )
    for row in rows:
        u, a = row["uniform"], row["adaptive"]
        print(
            f"{row['iterations']:>10}  {row['radius_mean']:7.4f}  "
            f"{row['volume']:7.4f}  {u['hexes']:>13,}  {u['min']:6.3f}  "
            f"{u['mean']:6.3f}  {a['hexes']:>14,}  {a['min']:6.3f}  {a['mean']:6.3f}"
        )


def sweep_plot(*, rows: list, output: Path) -> None:
    """Draws the minimum scaled Jacobian and the hex count against iterations.

    A filled star marks the peak up to `LIMIT` iterations.  A hollow star marks
    the peak of the whole sweep, when it lies beyond `LIMIT`.
    """
    fig, (ax, counts) = plt.subplots(
        2, 1, figsize=(9, 7), sharex=True, gridspec_kw={"height_ratios": (3, 1)}
    )
    iterations = [row["iterations"] for row in rows]
    for (name, _), style in zip(CASES, HISTOGRAM[1:], strict=True):
        ax.plot(
            iterations,
            [row[name]["min"] for row in rows],
            marker="o",
            markersize=3,
            label=f"automesh {name}",
            **style,
        )
        counts.plot(
            iterations,
            [row[name]["hexes"] for row in rows],
            marker="o",
            markersize=3,
            **style,
        )
        for limit, filled in ((LIMIT, True), (None, False)):
            peak = peak_find(rows=rows, name=name, limit=limit)
            if not filled and peak["iterations"] <= LIMIT:
                continue
            ax.plot(
                [peak["iterations"]],
                [peak[name]["min"]],
                marker="*",
                markersize=15,
                color=style["color"],
                markerfacecolor=style["color"] if filled else "white",
                linestyle="none",
            )
            ax.annotate(
                f"{peak[name]['min']:.3f} at {peak['iterations']}",
                xy=(peak["iterations"], peak[name]["min"]),
                xytext=(0, 12),
                textcoords="offset points",
                color=style["color"],
                fontsize=9,
                fontweight="bold",
                ha="center",
            )
    ratio = next(row["volume"] for row in rows if row["iterations"] == LIMIT)
    for axis in (ax, counts):
        axis.axvline(LIMIT, color=MUTED, linewidth=1.0, linestyle="-")
        axis.set_xscale("symlog", linthresh=10)
        axis.grid(color=GRID, linewidth=0.6)
        axis.tick_params(colors=INK)
        for spine in axis.spines.values():
            spine.set_color(GRID)
    ax.text(
        LIMIT * 0.93,
        0.03,
        f"volume {ratio / EXACT - 1:+.1%}",
        transform=ax.get_xaxis_transform(),
        color=MUTED,
        fontsize=8,
        rotation=90,
        ha="right",
        va="bottom",
    )
    ax.set_xlim(0, 1300)
    ax.set_ylim(top=0.7)
    ax.set_ylabel("Minimum Scaled Jacobian", color=INK)
    ax.legend(frameon=False, loc="upper left")
    counts.set_yscale("log")
    counts.set_ylabel("Hex Count (int)", color=INK)
    counts.set_xlabel("Taubin smoothing iterations", color=INK)
    counts.set_xticks([0, 5, 10, 20, 50, 100, 300, 1000])
    counts.get_xaxis().set_major_formatter(plt.ScalarFormatter())
    fig.tight_layout()
    fig.savefig(output, dpi=DPI)
    plt.close(fig)


def triple_plot(*, smooth: Path, stem: Path, name: str, output: Path) -> None:
    """Draws the smoothed surface, the mesh, and the mesh cut at z = 0."""
    triangles = stl_read(path=smooth)
    normals = np.cross(
        triangles[:, 1] - triangles[:, 0], triangles[:, 2] - triangles[:, 0]
    )
    fig = plt.figure(figsize=(15, 5))
    ax = fig.add_subplot(1, 3, 1, projection="3d")
    polygons_draw(ax=ax, polygons=triangles, normals=normals, values=None)
    axes_set(ax=ax, title=f"smoothed surface ({len(triangles):,} triangles)")
    mesh_draw(fig=fig, position=2, title=name, stem=stem, cut=False)
    mesh_draw(fig=fig, position=3, title=f"{name}, cut at z = 0", stem=stem, cut=True)
    colorbar_add(fig=fig)
    fig.savefig(output, dpi=DPI)
    plt.close(fig)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--scratch", type=Path, help="keep working files here")
    parser.add_argument("--iterations", type=int, nargs="+", help="override the sweep")
    args = parser.parse_args()
    here = Path(__file__).parent
    iterations = tuple(args.iterations) if args.iterations else ITERATIONS
    version = subprocess.run(
        ["automesh", "--version"], check=True, capture_output=True, text=True
    ).stdout.strip()
    print(version)
    with tempfile.TemporaryDirectory() as temporary:
        scratch = args.scratch or Path(temporary)
        scratch.mkdir(parents=True, exist_ok=True)
        stl = scratch / f"unit_sphere_mc_n{N:03d}.stl"
        surface_write(radius=N, path=stl)
        cache = scratch / "rows.json"
        done = (
            {r["iterations"]: r for r in json.loads(cache.read_text())}
            if cache.exists()
            else {}
        )
        todo = [i for i in iterations if i not in done]
        with ThreadPoolExecutor(max_workers=WORKERS) as pool:
            for row in pool.map(
                lambda i: iteration_run(stl=stl, scratch=scratch, iterations=i), todo
            ):
                done[row["iterations"]] = row
                cache.write_text(json.dumps(list(done.values())))
        rows = [done[i] for i in sorted(iterations)]
        table_print(rows=rows)
        sweep_plot(rows=rows, output=here / "unit_sphere_smooth_sweep.png")
        best_meshes = []
        for name, _ in CASES:
            peak = peak_find(rows=rows, name=name, limit=LIMIT)
            best = peak_find(rows=rows, name=name)
            print(
                f"{name}: peak up to {LIMIT} iterations {peak[name]['min']:.3f} at "
                f"{peak['iterations']}, whole sweep {best[name]['min']:.3f} at "
                f"{best['iterations']}, against {rows[0][name]['min']:.3f} at 0"
            )
            triple_plot(
                smooth=scratch / f"smooth_iter{peak['iterations']:03d}.stl",
                stem=scratch / f"{name}_iter{peak['iterations']:03d}",
                name=name,
                output=here / f"unit_sphere_smooth_{name}.png",
            )
            best_meshes.append(
                (
                    f"automesh {name}, {peak['iterations']} iterations",
                    scratch / f"{name}_iter{peak['iterations']:03d}",
                )
            )
        sculpt = sculpt_metrics(
            exodus=here / f"unit_sphere_sculpt_n{N:03d}.e.1.0", scratch=scratch
        )
        before = {"alpha": 0.4, "linewidth": 1.0}
        series = [(f"Sculpt ({count(stem=sculpt):,} hexes)", sculpt, HISTOGRAM[0])]
        for style, (name, _), (label, best) in zip(
            HISTOGRAM[1:], CASES, best_meshes, strict=True
        ):
            zero = scratch / f"{name}_iter000"
            series += [
                (
                    f"automesh {name}, no smoothing ({count(stem=zero):,} hexes)",
                    zero,
                    {**style, **before},
                ),
                (f"{label} ({count(stem=best):,} hexes)", best, style),
            ]
        quality_plot(
            meshes=tuple((label, stem) for label, stem, _ in series),
            output=here / "unit_sphere_smooth_quality.png",
            styles=tuple(style for _, _, style in series),
        )


if __name__ == "__main__":
    main()
