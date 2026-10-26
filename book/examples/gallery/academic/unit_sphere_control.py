#!/usr/bin/env python3
"""Control study: an Octa-Loop surface against the smoothed marching cubes surface.

Taubin smoothing improves the Minimum Scaled Jacobian (MSJ) of the `automesh`
hex meshes of the n = 10 marching cubes surface.  The control study asks whether
a surface that is already smooth does better.  The script

1. meshes `octa_loop04.stl`, the Octa-Loop level 4 sphere of 2,048 triangles,
   with `automesh mesh hex`, on a uniform lattice and with the adaptive default,
2. builds the n = 10 marching cubes surface, smooths it for the iteration count
   that gave the best uniform mesh and for the count that gave the best adaptive
   mesh (`UNIFORM_ITERATIONS` and `ADAPTIVE_ITERATIONS`, from
   `unit_sphere_smooth.py`), and meshes each,
3. prints a table of the four meshes,
4. draws the Octa-Loop surface and its two meshes in
   `unit_sphere_control_meshes.png`, and the same three cut at z = 0 in
   `unit_sphere_control_meshes_cut.png`, and
5. draws the four quality histograms in `unit_sphere_control_quality.png`.

Octa-Loop level 4 comes from the Refinement table of the Subdivision page.  The
1,026 vertices of the level 4 surface lie within 0.3% of the unit sphere.

Example
-------
cd ~/autotwin/automesh/book/examples/gallery/academic
uv run --with numpy --with matplotlib unit_sphere_control.py

Output
------
unit_sphere_control_meshes.png, unit_sphere_control_meshes_cut.png,
unit_sphere_control_quality.png, and a table on the terminal.
"""

import subprocess
import tempfile
from pathlib import Path

import numpy as np

from unit_sphere_meshes import (
    CELL,
    HISTOGRAM,
    N,
    automesh_run,
    figure_plot,
    quality_plot,
    stl_read,
    surface_write,
)

UNIFORM_ITERATIONS = 260
ADAPTIVE_ITERATIONS = 11
BEFORE = {"alpha": 0.4, "linewidth": 1.0}


def nodes_count(*, stem: Path) -> int:
    """Returns the number of nodes in the `.inp` file of a mesh."""
    count, inside = 0, False
    for line in stem.with_suffix(".inp").read_text().splitlines():
        if line.startswith("*"):
            inside = line.lower().startswith("*node")
        elif inside and line.strip():
            count += 1
    return count


def msj_read(*, stem: Path) -> np.ndarray:
    """Returns the Minimum Scaled Jacobian of every hex of a mesh."""
    return np.genfromtxt(stem.with_suffix(".csv"), delimiter=",", names=True)[
        "minimum_scaled_jacobian"
    ]


def surface_smooth(*, stl: Path, iterations: int) -> Path:
    """Runs `automesh smooth` on `stl` and returns the smoothed STL."""
    out = stl.parent / f"smooth_iter{iterations:03d}.stl"
    command = ["automesh", "smooth", "-i", str(stl), "-o", str(out)]
    command += ["--iterations", str(iterations), "-q"]
    subprocess.run(command, check=True)
    return out


def main() -> None:
    here = Path(__file__).parent
    version = subprocess.run(
        ["automesh", "--version"], check=True, capture_output=True, text=True
    ).stdout.strip()
    print(version)
    with tempfile.TemporaryDirectory() as temporary:
        scratch = Path(temporary)
        octa = scratch / "octa_loop04.stl"
        octa.write_bytes((here / "octa_loop04.stl").read_bytes())
        marching = scratch / f"unit_sphere_mc_n{N:03d}.stl"
        surface_write(radius=N, path=marching)
        uniform = surface_smooth(stl=marching, iterations=UNIFORM_ITERATIONS)
        adaptive = surface_smooth(stl=marching, iterations=ADAPTIVE_ITERATIONS)
        rows = (
            (
                "Octa-Loop level 4, uniform",
                automesh_run(stl=octa, name="octa_uniform", uniform=CELL),
                HISTOGRAM[1],
            ),
            (
                "Octa-Loop level 4, adaptive",
                automesh_run(stl=octa, name="octa_adaptive", uniform=None),
                HISTOGRAM[2],
            ),
            (
                f"marching cubes, uniform, {UNIFORM_ITERATIONS} iterations",
                automesh_run(stl=uniform, name="mc_uniform", uniform=CELL),
                {**HISTOGRAM[1], **BEFORE},
            ),
            (
                f"marching cubes, adaptive, {ADAPTIVE_ITERATIONS} iterations",
                automesh_run(stl=adaptive, name="mc_adaptive", uniform=None),
                {**HISTOGRAM[2], **BEFORE},
            ),
        )
        print(
            f"{'mesh':<40}{'nodes':>8}{'hexes':>8}{'MSJ min':>9}{'MSJ mean':>10}"
            f"{'inverted':>10}"
        )
        for label, stem, _ in rows:
            msj = msj_read(stem=stem)
            print(
                f"{label:<40}{nodes_count(stem=stem):>8,}{len(msj):>8,}"
                f"{msj.min():>9.3f}{msj.mean():>10.3f}{int((msj <= 0.0).sum()):>10}"
            )
        binary = scratch / "octa_loop04_binary.stl"
        subprocess.run(
            ["automesh", "convert", "mesh", "-i", str(octa), "-o", str(binary), "-q"],
            check=True,
        )
        triangles = stl_read(path=binary)
        for output, cut in (
            ("unit_sphere_control_meshes.png", False),
            ("unit_sphere_control_meshes_cut.png", True),
        ):
            figure_plot(
                triangles=triangles,
                meshes=(
                    ("automesh uniform", rows[0][1]),
                    ("automesh adaptive", rows[1][1]),
                ),
                output=here / output,
                cut=cut,
                surface_title=f"Octa-Loop level 4 ({len(triangles):,} triangles)",
            )
        quality_plot(
            meshes=tuple(
                (f"{label} ({len(msj_read(stem=stem)):,} hexes)", stem)
                for label, stem, _ in rows
            ),
            output=here / "unit_sphere_control_quality.png",
            styles=tuple(style for _, _, style in rows),
        )


if __name__ == "__main__":
    main()
