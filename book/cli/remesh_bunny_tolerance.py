r"""This module, remesh_bunny_tolerance.py, studies how the adaptive
`--tolerance` affects the facet count of the remeshed Stanford bunny.

The adaptive edge length follows the Dunyach formula
``L = sqrt(6 * tolerance / curvature - 3 * tolerance**2)`` clamped to
``[minimum, maximum]``.  Because of the ``- 3 * tolerance**2`` term, the facet
count is *non-monotonic* in the tolerance: very small and very large tolerances
both refine the mesh, with the coarsest result in between.  This script sweeps
the tolerance at fixed ``--minimum``/``--maximum``/``--iterations`` and plots the
resulting facet count.

Example
-------
source ~/autotwin/automesh/.venv/bin/activate
cd ~/autotwin/automesh/book/cli
# `automesh` must be on the PATH (e.g. target/release)
python remesh_bunny_tolerance.py

Output
------
The `bunny_tolerance.png` plot, written next to this script, and a summary table
printed to the terminal.
"""

import os
import shutil
import struct
import subprocess
import tempfile
from pathlib import Path

import matplotlib.pyplot as plt

TOLERANCES = [0.0002, 0.0005, 0.001, 0.002, 0.004, 0.008, 0.02, 0.05]
MINIMUM, MAXIMUM, ITERATIONS = 0.002, 0.040, 25
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


def facet_count(stl: Path) -> int:
    """Reads the facet count from a binary STL header (bytes 80-84)."""
    with stl.open("rb") as file:
        file.seek(80)
        return struct.unpack("<I", file.read(4))[0]


def main() -> None:
    here = Path(__file__).resolve().parent
    source = here / "stanford_bunny.stl"
    automesh = automesh_binary()
    counts = []
    print(f"{'tolerance':>10} {'facets':>8}")
    with tempfile.TemporaryDirectory() as tmp:
        for tol in TOLERANCES:
            out = Path(tmp) / "t.stl"
            subprocess.run(
                [automesh, "remesh", "-i", str(source), "-o", str(out),
                 "adaptive", "--minimum", str(MINIMUM), "--maximum", str(MAXIMUM),
                 "-n", str(ITERATIONS), "-t", str(tol)],
                check=True, capture_output=True,
            )
            n = facet_count(out)
            counts.append(n)
            print(f"{tol:>10} {n:>8}")

    coarsest = TOLERANCES[counts.index(min(counts))]
    fig, ax = plt.subplots(figsize=(6, 4))
    ax.plot(TOLERANCES, counts, "o-", color=EDGECOLOR, markerfacecolor=FACECOLOR)
    ax.set_xscale("log")
    ax.set_yscale("log")
    ax.set_xlabel("--tolerance")
    ax.set_ylabel("facets")
    ax.set_title("Bunny facet count vs. adaptive tolerance")
    ax.axvline(
        coarsest, color="crimson", linestyle="--", linewidth=1.0,
        label=f"coarsest near {coarsest}",
    )
    ax.legend()
    ax.grid(True, which="both", alpha=0.3)
    png = here / "bunny_tolerance.png"
    fig.savefig(png, dpi=150, bbox_inches="tight")
    plt.close(fig)
    print(f"wrote {png.name}")


if __name__ == "__main__":
    main()
