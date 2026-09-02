r"""This module, rve_figures.py, renders the surface triangulation of the
Representative Volume Element (RVE) used in the Gallery section of the
documentation.  It reads `comparison.stl`, the Cubit export of a unit cube
containing three spherical pores, and saves a two-panel PNG: the full surface
triangulation, and the interior pore surfaces on their own.

The pores are entirely interior, so an exterior view alone shows only a cube.
The second panel isolates the facets that do not lie on a bounding plane, which
are exactly the three pore surfaces, and draws the cube as a wireframe for
context.

Example
-------
source ~/autotwin/automesh/.venv/bin/activate
cd ~/autotwin/automesh/book/examples/gallery/academic
# comparison.stl is not committed; download it first (see the Downloads
# section of rve.md), then either place it next to this script or pass a path:
python rve_figures.py ~/Downloads/comparison.stl

Output
------
The `comparison.png` visualization file, written next to this script.
"""

# standard library
import sys
from pathlib import Path
from typing import Final

# third-party library
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d.art3d import Poly3DCollection
import numpy as np
from numpy.typing import NDArray

# Shared view so the two panels are read as the same object from the same
# camera, with only the surface subset changing between them.
ELEV: Final[float] = 22.0
AZIM: Final[float] = -55.0
FACECOLOR: Final[str] = "lightblue"
EDGECOLOR: Final[str] = "navy"
PORE_FACECOLOR: Final[str] = "lightcoral"
PORE_EDGECOLOR: Final[str] = "darkred"
TOLERANCE: Final[float] = 1.0e-6
# The pores occupy a small fraction of the cube, so their triangulation needs
# more pixels than the twelve-triangle cube to stay legible.
DPI: Final[int] = 300


def stl_read(*, path: Path) -> NDArray[np.float64]:
    """Reads triangular facets from an ASCII STL file, returning an array of
    shape (n_facets, 3, 3)."""
    verts = []
    for line in path.read_text(errors="replace").splitlines():
        tokens = line.split()
        if tokens and tokens[0] == "vertex":
            verts.append([float(v) for v in tokens[1:4]])
    return np.array(verts, dtype=np.float64).reshape(-1, 3, 3)


def pore_facets_select(*, facets: NDArray[np.float64]) -> NDArray[np.bool_]:
    """Returns a mask selecting the facets that do not lie on any bounding
    plane of the model.  A cube-face triangle has all three vertices on one
    bounding plane; a pore triangle has none, so the mask isolates the pores."""
    points = facets.reshape(-1, 3)
    lower, upper = points.min(axis=0), points.max(axis=0)
    on_boundary = np.zeros(len(facets), dtype=bool)
    for axis in range(3):
        for plane in (lower[axis], upper[axis]):
            flush = np.abs(facets[:, :, axis] - plane) < TOLERANCE
            on_boundary |= flush.all(axis=1)
    return ~on_boundary


def wireframe_draw(*, axes, lower: NDArray[np.float64], upper: NDArray[np.float64]) -> None:
    """Draws the twelve edges of the bounding box, so the isolated pores are
    seen in place rather than floating free."""
    corners = np.array(
        [
            [x, y, z]
            for x in (lower[0], upper[0])
            for y in (lower[1], upper[1])
            for z in (lower[2], upper[2])
        ]
    )
    for i, first in enumerate(corners):
        for second in corners[i + 1 :]:
            # An edge joins two corners differing in exactly one coordinate.
            if np.count_nonzero(np.abs(first - second) > TOLERANCE) == 1:
                axes.plot(*zip(first, second), color="gray", linewidth=0.6)


def panel_render(*, axes, facets, facecolor, edgecolor, linewidth, limits, title) -> None:
    """Renders one triangulated surface into a prepared 3D axes."""
    axes.add_collection3d(
        Poly3DCollection(
            facets,
            facecolor=facecolor,
            edgecolor=edgecolor,
            linewidths=linewidth,
            alpha=1.0,
        )
    )
    lower, upper = limits
    axes.set_xlim(lower[0], upper[0])
    axes.set_ylim(lower[1], upper[1])
    axes.set_zlim(lower[2], upper[2])
    axes.set_box_aspect(tuple(upper - lower))
    axes.view_init(elev=ELEV, azim=AZIM)
    axes.set_axis_off()
    axes.set_title(title, fontsize=11)


def tessellation_render(*, stl: Path, png: Path) -> None:
    """Saves the two-panel figure for the given RVE surface mesh."""
    facets = stl_read(path=stl)
    pores = pore_facets_select(facets=facets)
    points = facets.reshape(-1, 3)
    limits = (points.min(axis=0), points.max(axis=0))

    figure = plt.figure(figsize=(11, 5.5))

    axes = figure.add_subplot(121, projection="3d")
    panel_render(
        axes=axes,
        facets=facets,
        facecolor=FACECOLOR,
        edgecolor=EDGECOLOR,
        linewidth=0.2,
        limits=limits,
        title=f"surface triangulation\n{len(facets)} facets",
    )

    axes = figure.add_subplot(122, projection="3d")
    panel_render(
        axes=axes,
        facets=facets[pores],
        facecolor=PORE_FACECOLOR,
        edgecolor=PORE_EDGECOLOR,
        linewidth=0.2,
        limits=limits,
        title=f"interior pores only\n{int(pores.sum())} facets",
    )
    wireframe_draw(axes=axes, lower=limits[0], upper=limits[1])

    figure.savefig(png, dpi=DPI, bbox_inches="tight")
    plt.close(figure)
    print(f"wrote {png.name} ({len(facets)} facets, {int(pores.sum())} on pores)")


def main() -> None:
    here = Path(__file__).resolve().parent
    stl = Path(sys.argv[1]).expanduser() if len(sys.argv) > 1 else here / "comparison.stl"
    if not stl.exists():
        print(f"skipping {stl} (not found); see the Downloads section of rve.md")
        return
    tessellation_render(stl=stl, png=here / "comparison.png")


if __name__ == "__main__":
    main()
