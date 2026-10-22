#!/usr/bin/env python3
"""Remesh: the smoothed marching cubes surface against Octa-Loop level 4.

The n = 10 marching cubes surface has 3,800 triangles with edge lengths from
0.071 to 0.141 and a 30 degree worst angle.  Octa-Loop level 4 has 2,048
near-equilateral triangles.  The script asks how closely `automesh remesh`
turns the marching cubes surface into the Octa-Loop surface.  It

1. builds the n = 10 surface and takes the target edge length from the mean edge
   of `octa_loop04.stl` (`SIZE`),
2. remeshes the surface with `automesh remesh uniform`,
3. smooths the surface with `automesh smooth` for `SMOOTHING` iterations,
   remeshes it the same way, and smooths it again for `SMOOTHING` iterations,
4. prints a table of the four surfaces and a table of their edge lengths,
5. draws the surfaces side by side in `unit_sphere_remesh.png`.

Example
-------
cd ~/autotwin/automesh/book/examples/gallery/academic
uv run --with numpy --with matplotlib unit_sphere_remesh.py

Output
------
unit_sphere_remesh.png and two tables on the terminal.
"""

import subprocess
import tempfile
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np

from unit_sphere_control import surface_smooth
from unit_sphere_meshes import (
    DPI,
    N,
    axes_set,
    polygons_draw,
    stl_read,
    surface_write,
)

SIZE = 0.1178
ITERATIONS = 20
SMOOTHING = 20
DECIMALS = 5


def surface_remesh(*, stl: Path, name: str) -> Path:
    """Runs `automesh remesh uniform` on `stl` and returns the remeshed STL."""
    out = stl.parent / f"{name}.stl"
    command = ["automesh", "remesh", "-i", str(stl), "-o", str(out), "-q"]
    command += ["uniform", "--iterations", str(ITERATIONS), "--size", str(SIZE)]
    subprocess.run(command, check=True)
    return out


def surface_binary(*, stl: Path) -> Path:
    """Converts `stl` to binary STL, which `stl_read` needs, and returns the copy."""
    out = stl.parent / f"{stl.stem}_binary.stl"
    command = ["automesh", "convert", "mesh", "-i", str(stl), "-o", str(out), "-q"]
    subprocess.run(command, check=True)
    return out


def surface_index(*, triangles: np.ndarray) -> tuple:
    """Returns the points and the faces of a triangle surface of shape (faces, 3, 3).

    Vertices that agree to `DECIMALS` decimals are one point.
    """
    points, index = np.unique(
        np.round(triangles.reshape(-1, 3), DECIMALS), axis=0, return_inverse=True
    )
    return points, index.reshape(-1, 3)


def edge_lengths(*, triangles: np.ndarray) -> np.ndarray:
    """Returns the length of every distinct edge of a triangle surface."""
    points, faces = surface_index(triangles=triangles)
    pairs = np.unique(
        np.sort(
            np.concatenate([faces[:, [0, 1]], faces[:, [1, 2]], faces[:, [2, 0]]]),
            axis=1,
        ),
        axis=0,
    )
    return np.linalg.norm(points[pairs[:, 0]] - points[pairs[:, 1]], axis=1)


def surface_stats(*, triangles: np.ndarray) -> dict:
    """Returns the measures of a triangle surface of shape (faces, 3, 3).

    The surface is closed when every edge belongs to exactly two triangles.  The
    radius deviation of a vertex is its distance from the origin minus 1.  The
    dihedral angle of an edge is the angle between the normals of its two
    triangles, 0 degrees when they lie in one plane.
    """
    a, b, c = triangles[:, 0], triangles[:, 1], triangles[:, 2]
    edges = np.stack([b - a, c - b, a - c], axis=1)
    lengths = np.linalg.norm(edges, axis=2)
    angles = []
    for i in range(3):
        u, w = edges[:, i], -edges[:, (i + 2) % 3]
        cosine = np.einsum("ij,ij->i", u, w) / (
            np.linalg.norm(u, axis=1) * np.linalg.norm(w, axis=1)
        )
        angles.append(np.degrees(np.arccos(np.clip(cosine, -1.0, 1.0))))
    points, faces = surface_index(triangles=triangles)
    pairs = np.sort(
        np.concatenate([faces[:, [0, 1]], faces[:, [1, 2]], faces[:, [2, 0]]]), axis=1
    )
    _, inverse, counts = np.unique(
        pairs, axis=0, return_inverse=True, return_counts=True
    )
    closed = bool((counts == 2).all())
    normals = np.cross(b - a, c - a)
    normals /= np.linalg.norm(normals, axis=1)[:, None]
    owners = np.tile(np.arange(len(faces)), 3)[
        np.argsort(inverse.ravel(), kind="stable")
    ]
    owners = owners.reshape(-1, 2)
    dihedral = np.degrees(
        np.arccos(
            np.clip(
                np.einsum("ij,ij->i", normals[owners[:, 0]], normals[owners[:, 1]]),
                -1.0,
                1.0,
            )
        )
    )
    radii = np.linalg.norm(points, axis=1)
    return {
        "triangles": len(triangles),
        "closed": closed,
        "radius_rms": np.sqrt(np.mean((radii - 1.0) ** 2)),
        "radius_dev": np.abs(radii - 1.0).max(),
        "dihedral_max": dihedral.max(),
        "dihedral_rms": np.sqrt(np.mean(dihedral**2)),
        "edge_mean": lengths.mean(),
        "edge_cv": lengths.std() / lengths.mean(),
        "angle_min": np.min(angles),
        "angle_max": np.max(angles),
        "radius_min": radii.min(),
        "radius_max": radii.max(),
        "volume": np.einsum("ij,ij->i", a, np.cross(b, c)).sum() / 6.0,
        "area": 0.5 * np.linalg.norm(np.cross(b - a, c - a), axis=1).sum(),
    }


def remesh_plot(*, surfaces: tuple, output: Path) -> None:
    """Draws each (title, triangles) of `surfaces` as one panel, left to right."""
    fig = plt.figure(figsize=(5 * len(surfaces), 5))
    for index, (title, triangles) in enumerate(surfaces, start=1):
        normals = np.cross(
            triangles[:, 1] - triangles[:, 0], triangles[:, 2] - triangles[:, 0]
        )
        ax = fig.add_subplot(1, len(surfaces), index, projection="3d")
        polygons_draw(ax=ax, polygons=triangles, normals=normals, values=None)
        axes_set(ax=ax, title=f"{title} ({len(triangles):,} triangles)")
    fig.tight_layout(rect=(0.0, 0.0, 1.0, 0.95))
    fig.savefig(output, dpi=DPI)
    plt.close(fig)


def main() -> None:
    here = Path(__file__).parent
    version = subprocess.run(
        ["automesh", "--version"], check=True, capture_output=True, text=True
    ).stdout.strip()
    print(version)
    with tempfile.TemporaryDirectory() as temporary:
        scratch = Path(temporary)
        marching = scratch / f"unit_sphere_mc_n{N:03d}.stl"
        surface_write(radius=N, path=marching)
        octa = scratch / "octa_loop04.stl"
        octa.write_bytes((here / "octa_loop04.stl").read_bytes())
        smoothed = surface_smooth(stl=marching, iterations=SMOOTHING)
        remeshed = surface_remesh(stl=smoothed, name="smooth_remesh")
        surfaces = (
            (f"marching cubes, n = {N}", marching),
            ("remesh", surface_remesh(stl=marching, name="remesh")),
            (
                f"Taubin {SMOOTHING}, remesh, Taubin {SMOOTHING}",
                surface_smooth(stl=remeshed, iterations=SMOOTHING),
            ),
            ("Octa-Loop level 4", surface_binary(stl=octa)),
        )
        surfaces = tuple((title, stl_read(path=stl)) for title, stl in surfaces)
        octa_edge = surface_stats(triangles=surfaces[-1][1])["edge_mean"]
        print(f"target edge length {SIZE}, Octa-Loop level 4 mean edge {octa_edge:.4f}")
        print(
            f"{'surface':<36}{'tris':>6}{'closed':>8}{'edge cv':>9}{'angle min':>11}"
            f"{'angle max':>11}{'radius min':>12}{'radius max':>12}{'radius rms':>12}"
            f"{'max |r-1|':>11}{'dihedral max':>14}{'dihedral rms':>14}{'volume':>9}"
            f"{'area':>9}"
        )
        for title, triangles in surfaces:
            s = surface_stats(triangles=triangles)
            print(
                f"{title:<36}{s['triangles']:>6,}{s['closed']!s:>8}{s['edge_cv']:>9.3f}"
                f"{s['angle_min']:>11.1f}{s['angle_max']:>11.1f}"
                f"{s['radius_min']:>12.4f}{s['radius_max']:>12.4f}"
                f"{s['radius_rms']:>12.5f}{s['radius_dev']:>11.4f}"
                f"{s['dihedral_max']:>14.1f}{s['dihedral_rms']:>14.1f}"
                f"{s['volume']:>9.4f}{s['area']:>9.4f}"
            )
        print(f"exact: volume {4 * np.pi / 3:.4f}, area {4 * np.pi:.4f}")
        print(
            f"{'surface':<32}{'edges':>7}{'min':>8}{'5%':>8}{'median':>8}{'mean':>8}"
            f"{'95%':>8}{'max':>8}{'within 10%':>12}"
        )
        for title, triangles in surfaces:
            v = edge_lengths(triangles=triangles)
            low, median, high = np.percentile(v, [5, 50, 95])
            near = np.abs(v - SIZE) <= 0.1 * SIZE
            print(
                f"{title:<32}{len(v):>7,}{v.min():>8.3f}{low:>8.3f}{median:>8.3f}"
                f"{v.mean():>8.3f}{high:>8.3f}{v.max():>8.3f}{near.mean():>12.1%}"
            )
        remesh_plot(surfaces=surfaces, output=here / "unit_sphere_remesh.png")


if __name__ == "__main__":
    main()
