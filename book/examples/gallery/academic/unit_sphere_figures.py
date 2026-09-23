#!/usr/bin/env python3
"""Figures for the Unit Sphere page.

    unit_sphere_voxels.png       the segmentations for n = 10, 40, and 160
    unit_sphere_isosurfaces.png  the marching-cubes surfaces for n = 10, 40, and 160
    unit_sphere_convergence.png  the volume and its error, for every n from 4 to 160
    unit_sphere_sculpt.png       the Sculpt meshes, painted by Minimum Scaled Jacobian
    unit_sphere_sculpt_cut.png   the same meshes, cut at z = 0
    unit_sphere_meshers.png      Sculpt and automesh meshes of the n = 160 surface
    unit_sphere_meshers_cut.png  the same meshes, cut at z = 0
    unit_sphere_quality.png      quality histograms of the n = 160 meshes

Example
-------
cd ~/autotwin/automesh/book/examples/gallery/academic
uv run --with numpy unit_sphere_segmentation.py
uv run --with numpy --with scikit-image unit_sphere_isosurface.py
uv run --with numpy --with scipy unit_sphere_mesh.py
uv run --with numpy --with scikit-image --with matplotlib unit_sphere_figures.py

Output
------
unit_sphere_voxels.png, unit_sphere_isosurfaces.png,
unit_sphere_convergence.png, unit_sphere_sculpt.png,
unit_sphere_sculpt_cut.png, unit_sphere_meshers.png,
unit_sphere_meshers_cut.png, and unit_sphere_quality.png.
"""

from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np
from matplotlib.cm import ScalarMappable
from matplotlib.colors import LightSource, Normalize
from mpl_toolkits.mplot3d.art3d import Poly3DCollection

from unit_sphere_isosurface import isosurface, volume_signed
from unit_sphere_segmentation import sphere

RADII = (10, 40, 160)
HEX_FACES = (
    (0, 1, 5, 4),
    (1, 2, 6, 5),
    (2, 3, 7, 6),
    (3, 0, 4, 7),
    (0, 3, 2, 1),
    (4, 5, 6, 7),
)
SWEEP = range(4, 161)
TABLE = (10, 20, 40, 80, 160)
SCULPT_PANELS = (
    ("Octa-Loop level 3", "unit_sphere_sculpt_octa03"),
    ("n=10", "unit_sphere_sculpt_n010"),
    ("n=160", "unit_sphere_sculpt_n160"),
)
MESHER_PANELS = (
    ("Sculpt", "unit_sphere_sculpt_n160"),
    ("automesh, uniform lattice", "unit_sphere_uniform_n160"),
    ("automesh, adaptive octree", "unit_sphere_octree_n160"),
)
DPI = 200
EDGES_MAX = 40
ELEVATION, AZIMUTH = 63, -110
LIGHT = LightSource(azdeg=325, altdeg=45)
COLOR = plt.get_cmap("tab10")(0)
LAYOUT = (0.0, 0.0, 1.0, 0.95)
SERIES = ("#2a78d6", "#eb6834")
INK = "#3d3d3a"
MUTED = "#8a8a85"
GRID = "#e6e5e0"
HISTOGRAM = (
    {"color": "#eb6834", "linewidth": 2.0, "linestyle": "-"},
    {"color": "#2a78d6", "linewidth": 1.5, "linestyle": "--"},
    {"color": "#2f9e5b", "linewidth": 1.5, "linestyle": ":"},
)
QUALITY = (
    ("minimum_scaled_jacobian", "Minimum Scaled Jacobian"),
    ("maximum_edge_ratio", "Maximum Aspect Ratio"),
    ("maximum_skew", "Maximum Skew"),
    ("element_volume", "Element Volume"),
)


def stl_read(*, path: Path) -> np.ndarray:
    """Returns the triangles of a binary STL, with shape (faces, 3, 3)."""
    record = np.dtype(
        [("normal", "<f4", 3), ("vertices", "<f4", (3, 3)), ("attribute", "<u2")]
    )
    with path.open("rb") as file:
        file.seek(80)
        count = int(np.frombuffer(file.read(4), dtype="<u4")[0])
        return np.frombuffer(file.read(), dtype=record, count=count)["vertices"]


def camera() -> np.ndarray:
    """Returns the unit vector from the origin toward the viewer."""
    elevation, azimuth = np.radians(ELEVATION), np.radians(AZIMUTH)
    return np.array(
        [
            np.cos(elevation) * np.cos(azimuth),
            np.cos(elevation) * np.sin(azimuth),
            np.sin(elevation),
        ]
    )


def voxel_faces(*, voxels: np.ndarray) -> tuple:
    """Returns the exposed voxel faces as quads, with one normal per quad."""
    padded = np.pad(voxels, 1)
    quads, normals = [], []
    for axis in range(3):
        u, w = np.eye(3, dtype=int)[[(axis + 1) % 3, (axis + 2) % 3]]
        for sign in (-1, 1):
            neighbor = np.roll(padded, -sign, axis=axis)
            base = np.argwhere(padded & ~neighbor) - 1
            if sign == 1:
                base[:, axis] += 1
            corners = np.stack([base, base + u, base + u + w, base + w], axis=1)
            quads.append(corners)
            normal = np.zeros(3)
            normal[axis] = sign
            normals.append(np.tile(normal, (len(base), 1)))
    return np.concatenate(quads).astype(float), np.concatenate(normals)


def quads_draw(
    *,
    ax,
    quads: np.ndarray,
    normals: np.ndarray,
    linewidth: float,
    values: np.ndarray | None = None,
) -> None:
    """Draws the quads that face the camera, shaded by their outward normals.

    With `values`, each quad takes its color from Viridis on a 0 to 1 scale.
    """
    # matplotlib has no depth buffer, so back faces must be culled.
    front = normals @ camera() > 0.0
    quads, normals = quads[front], normals[front]
    shade = LIGHT.shade_normals(normals, fraction=1.0)
    if values is None:
        base = np.tile(np.array(COLOR[:3]), (len(quads), 1))
        light = 0.4 + 0.6 * shade
    else:
        base = plt.get_cmap("viridis")(np.clip(values[front], 0.0, 1.0))[:, :3]
        light = 0.7 + 0.3 * shade
    colors = np.clip(base * light[:, None], 0.0, 1.0)
    ax.add_collection3d(
        Poly3DCollection(quads, facecolors=colors, edgecolor="k", linewidth=linewidth)
    )


def voxels_draw(*, ax, voxels: np.ndarray, linewidth: float, ticks: list) -> None:
    """Draws the outer faces of a segmentation on a 3D axis, in voxel units."""
    quads, normals = voxel_faces(voxels=voxels)
    quads_draw(ax=ax, quads=quads, normals=normals, linewidth=linewidth)
    side = voxels.shape[0]
    for limit in (ax.set_xlim, ax.set_ylim, ax.set_zlim):
        limit(0, side)
    ax.set_xlabel("x (voxels)")
    ax.set_ylabel("y (voxels)")
    ax.set_zlabel("z (voxels)")
    ax.set_aspect("equal")
    ax.view_init(elev=ELEVATION, azim=AZIMUTH)
    for axis in (ax.set_xticks, ax.set_yticks, ax.set_zticks):
        axis(ticks)


def voxels_plot(*, here: Path) -> None:
    """Draws the segmentations, in voxel units."""
    fig = plt.figure(figsize=(5 * len(RADII), 5))
    for index, n in enumerate(RADII):
        voxels = np.load(here / f"unit_sphere_n{n:03d}.npy").astype(bool)
        ax = fig.add_subplot(1, len(RADII), index + 1, projection="3d")
        voxels_draw(ax=ax, voxels=voxels, linewidth=2 / n, ticks=[0, n, 2 * n])
        ax.set_title(f"n={n} ({int(voxels.sum()):,} voxels)")
    fig.tight_layout(rect=LAYOUT)
    fig.savefig(here / "unit_sphere_voxels.png", dpi=DPI)
    plt.close(fig)


def isosurfaces_plot(*, here: Path) -> None:
    """Draws the marching-cubes surfaces, on the unit sphere."""
    fig = plt.figure(figsize=(5 * len(RADII), 5))
    for index, n in enumerate(RADII):
        triangles = stl_read(path=here / f"unit_sphere_mc_n{n:03d}.stl")
        count = len(triangles)
        normals = np.cross(
            triangles[:, 1] - triangles[:, 0], triangles[:, 2] - triangles[:, 0]
        )
        # matplotlib has no depth buffer, so back faces must be culled.
        triangles = triangles[normals @ camera() > 0.0]
        normals = normals[normals @ camera() > 0.0]
        shade = 0.4 + 0.6 * LIGHT.shade_normals(normals, fraction=1.0)
        colors = np.clip(np.outer(shade, np.array(COLOR[:3])), 0.0, 1.0)
        ax = fig.add_subplot(1, len(RADII), index + 1, projection="3d")
        # Sub-pixel triangles leave anti-aliasing gaps unless edges match faces.
        edges = "k" if n <= EDGES_MAX else colors
        width = 1 / n if n <= EDGES_MAX else 0.3
        ax.add_collection3d(
            Poly3DCollection(
                triangles, facecolors=colors, edgecolors=edges, linewidth=width
            )
        )
        ax.set_title(f"n={n} ({count:,} triangles)")
        for axis in (ax.set_xlim, ax.set_ylim, ax.set_zlim):
            axis(-1, 1)
        for ticks in (ax.set_xticks, ax.set_yticks, ax.set_zticks):
            ticks([-1, 0, 1])
        ax.set_xlabel("x")
        ax.set_ylabel("y")
        ax.set_zlabel("z")
        ax.set_aspect("equal")
        ax.view_init(elev=ELEVATION, azim=AZIMUTH)
    fig.tight_layout(rect=LAYOUT)
    fig.savefig(here / "unit_sphere_isosurfaces.png", dpi=DPI)
    plt.close(fig)


def volumes_sweep() -> tuple:
    """Returns the voxel and marching-cubes volumes for every n in SWEEP."""
    voxel, surface = [], []
    for n in SWEEP:
        voxels = sphere(radius=n)
        vertices, faces = isosurface(voxels=voxels)
        voxel.append(voxels.sum() / n**3)
        surface.append(volume_signed(vertices=(vertices - (n + 1)) / n, faces=faces))
    return np.array(voxel), np.array(surface)


def convergence_plot(*, here: Path) -> None:
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
    fig.savefig(here / "unit_sphere_convergence.png", dpi=DPI)
    plt.close(fig)


def hex_boundary(*, points: np.ndarray, hexes: np.ndarray) -> tuple:
    """Returns the boundary quads of a hex mesh, their outward normals, and owners."""
    faces = hexes[:, HEX_FACES].reshape(-1, 4)
    owners = np.repeat(np.arange(len(hexes)), len(HEX_FACES))
    _, first, counts = np.unique(
        np.sort(faces, axis=1), axis=0, return_index=True, return_counts=True
    )
    boundary = first[counts == 1]
    quads = points[faces[boundary]]
    normals = np.cross(quads[:, 2] - quads[:, 0], quads[:, 3] - quads[:, 1])
    outward = quads.mean(axis=1) - points[hexes[owners[boundary]]].mean(axis=1)
    normals *= np.sign(np.einsum("ij,ij->i", normals, outward))[:, None]
    return quads, normals, owners[boundary]


def inp_read(*, path: Path) -> tuple:
    """Returns the node coordinates and 0-based hex connectivity of an .inp file."""
    points, hexes, section = [], [], None
    for line in path.read_text().splitlines():
        if line.startswith("*"):
            section = line.split(",")[0].strip().lower()
            continue
        if not line.strip():
            continue
        values = line.split(",")
        if section == "*node":
            points.append([float(v) for v in values[1:4]])
        elif section == "*element":
            hexes.append([int(v) for v in values[1:9]])
    return np.array(points), np.array(hexes) - 1


def meshes_plot(*, here: Path, panels: tuple, output: str, cut: bool) -> None:
    """Draws hex meshes side by side, each element painted by Minimum Scaled Jacobian.

    With `cut`, only the elements whose centers lie below z = 0 are drawn, so
    the cut exposes the interior.
    """
    fig = plt.figure(figsize=(5 * len(panels), 5))
    for index, (title, name) in enumerate(panels):
        points, hexes = inp_read(path=here / f"{name}.inp")
        metrics = np.genfromtxt(here / f"{name}.csv", delimiter=",", names=True)
        msj = metrics["minimum_scaled_jacobian"]
        count = len(hexes)
        if cut:
            below = points[hexes].mean(axis=1)[:, 2] < 0.0
            hexes, msj = hexes[below], msj[below]
        quads, normals, owners = hex_boundary(points=points, hexes=hexes)
        ax = fig.add_subplot(1, len(panels), index + 1, projection="3d")
        quads_draw(
            ax=ax, quads=quads, normals=normals, linewidth=0.2, values=msj[owners]
        )
        ax.set_title(f"{title} ({count:,} elements)")
        for limit in (ax.set_xlim, ax.set_ylim, ax.set_zlim):
            limit(-1, 1)
        for ticks in (ax.set_xticks, ax.set_yticks, ax.set_zticks):
            ticks([-1, 0, 1])
        ax.set_xlabel("x")
        ax.set_ylabel("y")
        ax.set_zlabel("z")
        ax.set_aspect("equal")
        ax.view_init(elev=ELEVATION, azim=AZIMUTH)
    fig.tight_layout(rect=(0.0, 0.0, 0.93, 0.95))
    bar = fig.add_axes((0.94, 0.2, 0.012, 0.6))
    fig.colorbar(
        ScalarMappable(norm=Normalize(0.0, 1.0), cmap="viridis"),
        cax=bar,
        label="Minimum Scaled Jacobian",
    )
    fig.savefig(here / output, dpi=DPI)
    plt.close(fig)


def quality_plot(*, here: Path) -> None:
    """Draws the four quality histograms for the n = 160 meshes."""
    meshes = (
        ("Sculpt", "unit_sphere_sculpt_n160.csv"),
        ("automesh, uniform lattice", "unit_sphere_uniform_n160.csv"),
        ("automesh, adaptive octree", "unit_sphere_octree_n160.csv"),
    )
    data = [
        (label, np.genfromtxt(here / name, delimiter=",", names=True))
        for label, name in meshes
    ]
    fig, axes = plt.subplots(2, 2, figsize=(9, 7))
    for ax, (key, title) in zip(axes.flat, QUALITY, strict=True):
        values = [metrics[key] for _, metrics in data]
        lo = min(v.min() for v in values)
        hi = max(v.max() for v in values)
        if key == "maximum_edge_ratio":
            bins = np.logspace(np.log10(lo), np.log10(hi), 41)
            ax.set_xscale("log")
        else:
            bins = np.linspace(lo, hi, 41)
        for (label, _), v, style in zip(data, values, HISTOGRAM, strict=True):
            ax.hist(v, bins=bins, histtype="step", alpha=0.75, label=label, **style)
        ax.set_yscale("log")
        ax.set_title(title, color=INK, fontsize=11)
        ax.set_xlabel(title, color=INK, fontsize=9)
        ax.set_ylabel("Element Count (int)", color=INK, fontsize=9)
        ax.tick_params(colors=INK, labelsize=8)
        ax.grid(color=GRID, linewidth=0.6)
        for spine in ax.spines.values():
            spine.set_color(GRID)
    handles, labels = axes.flat[0].get_legend_handles_labels()
    fig.legend(
        handles,
        labels,
        loc="upper center",
        ncol=len(labels),
        frameon=False,
        fontsize=9,
        bbox_to_anchor=(0.5, 1.0),
    )
    fig.tight_layout(rect=(0.0, 0.0, 1.0, 0.93))
    fig.savefig(here / "unit_sphere_quality.png", dpi=DPI)
    plt.close(fig)


def main() -> None:
    here = Path(__file__).parent
    voxels_plot(here=here)
    isosurfaces_plot(here=here)
    convergence_plot(here=here)
    for cut in (False, True):
        meshes_plot(
            here=here,
            panels=SCULPT_PANELS,
            output="unit_sphere_sculpt_cut.png" if cut else "unit_sphere_sculpt.png",
            cut=cut,
        )
        meshes_plot(
            here=here,
            panels=MESHER_PANELS,
            output="unit_sphere_meshers_cut.png" if cut else "unit_sphere_meshers.png",
            cut=cut,
        )
    quality_plot(here=here)


if __name__ == "__main__":
    main()
