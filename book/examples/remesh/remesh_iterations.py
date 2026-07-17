r"""This module, remesh_iterations.py, studies how the number of remeshing
iterations affects triangle uniformity.  It remeshes the example sphere at the
default target edge length (mean edge length) for a range of `--iterations`
values, measures the edge-length coefficient of variation (CoV = std / mean) of
each result, and plots CoV versus the number of iterations.

Example
-------
source ~/autotwin/automesh/.venv/bin/activate
cd ~/autotwin/automesh/book/examples/remesh
# `automesh` must be on the PATH (e.g. target/release)
python remesh_iterations.py

Output
------
The `sphere_iterations.png` plot, written next to this script, and a summary
table printed to the terminal.
"""

import os
import shutil
import struct
import subprocess
import tempfile
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np
from numpy.typing import NDArray

ITERATIONS = [1, 5, 10, 20, 50, 100]
FACECOLOR = "lightblue"
EDGECOLOR = "navy"


def automesh_binary() -> str:
    """Locates the `automesh` executable: the AUTOMESH environment variable, then
    the PATH, then the repository's target/release build."""
    candidate = os.environ.get("AUTOMESH") or shutil.which("automesh")
    if candidate:
        return candidate
    fallback = Path(__file__).resolve().parents[2] / "target" / "release" / "automesh"
    if fallback.exists():
        return str(fallback)
    raise FileNotFoundError(
        "could not find `automesh`; set AUTOMESH or add it to the PATH"
    )


def read_stl(path: Path) -> NDArray[np.float64]:
    """Reads triangular facets from a binary STL file."""
    data = path.read_bytes()
    (n_facets,) = struct.unpack_from("<I", data, 80)
    facets = np.empty((n_facets, 3, 3), dtype=np.float64)
    offset = 84
    for i in range(n_facets):
        values = struct.unpack_from("<12f", data, offset)
        facets[i] = np.array(values[3:12]).reshape(3, 3)
        offset += 50
    return facets


def edge_lengths(facets: NDArray[np.float64]) -> NDArray[np.float64]:
    """Returns the length of every unique undirected edge in the mesh."""
    keyed = np.round(facets.reshape(-1, 3), 6)
    _, inverse = np.unique(keyed, axis=0, return_inverse=True)
    ids = inverse.reshape(len(facets), 3)
    seen = set()
    lengths = []
    for tri, (a, b, c) in zip(facets, ids):
        for (u, v), (p, q) in (((a, b), (0, 1)), ((b, c), (1, 2)), ((c, a), (2, 0))):
            key = (int(min(u, v)), int(max(u, v)))
            if key not in seen:
                seen.add(key)
                lengths.append(float(np.linalg.norm(tri[p] - tri[q])))
    return np.array(lengths)


def main() -> None:
    here = Path(__file__).resolve().parent
    source = here / "sphere_radius_1.stl"
    automesh = automesh_binary()
    covs, facet_counts = [], []
    print(f"{'n':>4} {'facets':>7} {'mean':>7} {'CoV%':>6}")
    with tempfile.TemporaryDirectory() as tmp:
        for n in ITERATIONS:
            out = Path(tmp) / f"n{n}.stl"
            subprocess.run(
                [automesh, "remesh", "-i", str(source), "-o", str(out),
                 "uniform", "-n", str(n)],
                check=True, capture_output=True,
            )
            facets = read_stl(out)
            lengths = edge_lengths(facets)
            cov = 100.0 * lengths.std() / lengths.mean()
            covs.append(cov)
            facet_counts.append(len(facets))
            print(f"{n:>4} {len(facets):>7} {lengths.mean():>7.4f} {cov:>6.1f}")

    fig, ax = plt.subplots(figsize=(6, 4))
    ax.plot(ITERATIONS, covs, "o-", color=EDGECOLOR, markerfacecolor=FACECOLOR)
    ax.set_xscale("log")
    ax.set_xticks(ITERATIONS)
    ax.get_xaxis().set_major_formatter(plt.ScalarFormatter())
    ax.set_xlabel("number of iterations (--iterations)")
    ax.set_ylabel("edge-length CoV (%)")
    ax.set_title("Triangle uniformity vs. remeshing iterations")
    ax.axvline(5, color="crimson", linestyle="--", linewidth=1.0, label="default (5)")
    ax.legend()
    ax.grid(True, which="both", alpha=0.3)
    png = here / "sphere_iterations.png"
    fig.savefig(png, dpi=150, bbox_inches="tight")
    plt.close(fig)
    print(f"wrote {png.name}")


if __name__ == "__main__":
    main()
