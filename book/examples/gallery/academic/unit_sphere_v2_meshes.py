#!/usr/bin/env python3
"""The n = 10 marching cubes surface and its two `automesh` hex meshes.

The script

1. builds the segmentation of a sphere of radius `n = 10` voxels,
2. runs marching cubes on it and scales the surface to the unit sphere,
3. meshes the surface with `automesh mesh hex`, on a uniform lattice and
   with the adaptive default, and
4. draws the surface (left), the uniform mesh (center), and the adaptive
   mesh (right) in `unit_sphere_v2_meshes.png`, and
5. draws the same three, cut at z = 0, in `unit_sphere_v2_meshes_cut.png`.

A cut clips every triangle, quad, and hex at the plane z = 0 and keeps the lower
half.  Each hex that meets the plane contributes its cross-section, painted by
the hex's Minimum Scaled Jacobian, so the cut face is one flat plane.

The surface and the meshes live in a scratch directory.  Only the figure
stays.  The script prints the `automesh` version and a table of each mesh.

Example
-------
cd ~/autotwin/automesh/book/examples/gallery/academic
uv run --with numpy --with scikit-image --with matplotlib \
  unit_sphere_v2_meshes.py

Output
------
unit_sphere_v2_meshes.png, unit_sphere_v2_meshes_cut.png, and a table on the
terminal.
"""

import subprocess
import tempfile
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np
from matplotlib.cm import ScalarMappable
from matplotlib.colors import LightSource, Normalize
from mpl_toolkits.mplot3d.art3d import Poly3DCollection
from scipy.io import netcdf_file
from skimage import measure

N = 10
CELL = 2 * 1.240409 / 26
HEX_FACES = (
    (0, 1, 5, 4),
    (1, 2, 6, 5),
    (2, 3, 7, 6),
    (3, 0, 4, 7),
    (0, 3, 2, 1),
    (4, 5, 6, 7),
)
QUALITY = (
    ("minimum_scaled_jacobian", "Minimum Scaled Jacobian"),
    ("maximum_edge_ratio", "Maximum Aspect Ratio"),
    ("maximum_skew", "Maximum Skew"),
    ("element_volume", "Element Volume"),
)
HISTOGRAM = (
    {"color": "#eb6834", "linewidth": 2.0, "linestyle": "-"},
    {"color": "#2a78d6", "linewidth": 1.5, "linestyle": ":"},
    {"color": "#2f9e5b", "linewidth": 1.5, "linestyle": "-."},
)
INK = "#3d3d3a"
GRID = "#e6e5e0"
ELEVATION, AZIMUTH = 63, -110
LIGHT = LightSource(azdeg=325, altdeg=45)
SURFACE = (0.72, 0.72, 0.72)
VOXEL = (0.56, 0.37, 0.76)
EPS = 1e-6
HEX_EDGES = (
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0),
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 4),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
)
RADII = (10, 40, 160)
EDGES_MAX = 40
DPI = 200


def sphere(*, radius: int) -> np.ndarray:
    """Returns the segmentation of a sphere of `radius` voxels, axes (x, y, z)."""
    k = np.arange(-radius, radius + 1)
    x, y, z = np.meshgrid(k, k, k, indexing="ij", sparse=True)
    return (x * x + y * y + z * z <= radius * radius).astype(np.uint8)


def surface_make(*, radius: int) -> np.ndarray:
    """Returns the outward-wound triangles of the unit sphere, shape (faces, 3, 3)."""
    padded = np.pad(sphere(radius=radius), 1)
    vertices, faces, _, _ = measure.marching_cubes(padded, level=0.5)
    vertices = (vertices - (radius + 1)) / radius
    return vertices[faces[:, ::-1]]


def stl_write(*, path: Path, triangles: np.ndarray) -> None:
    """Writes a binary STL."""
    normals = np.cross(
        triangles[:, 1] - triangles[:, 0], triangles[:, 2] - triangles[:, 0]
    )
    normals /= np.linalg.norm(normals, axis=1, keepdims=True)
    record = np.dtype(
        [("normal", "<f4", 3), ("vertices", "<f4", (3, 3)), ("attribute", "<u2")]
    )
    data = np.zeros(len(triangles), dtype=record)
    data["normal"] = normals
    data["vertices"] = triangles
    with path.open("wb") as file:
        file.write(b"unit sphere, marching cubes".ljust(80, b" "))
        file.write(np.uint32(len(triangles)).tobytes())
        file.write(data.tobytes())


def automesh_run(*, stl: Path, name: str, uniform: float | None) -> Path:
    """Runs `automesh mesh hex` on `stl`, writes `name.inp` and `name.csv`."""
    command = ["automesh", "mesh", "hex", "-i", str(stl)]
    command += ["-o", str(stl.parent / f"{name}.inp")]
    command += ["--metrics", str(stl.parent / f"{name}.csv"), "-q"]
    if uniform is not None:
        command += ["-u", str(uniform)]
    subprocess.run(command, check=True)
    return stl.parent / name


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


def sculpt_metrics(*, exodus: Path, scratch: Path) -> Path:
    """Rewrites a Sculpt mesh as `.inp`, runs `automesh metrics`, returns the stem."""
    points, hexes = exodus_read(path=exodus)
    stem = scratch / exodus.name.removesuffix(".e.1.0")
    inp_write(path=stem.with_suffix(".inp"), points=points, hexes=hexes)
    subprocess.run(
        [
            "automesh",
            "metrics",
            "-i",
            str(stem.with_suffix(".inp")),
            "-o",
            str(stem.with_suffix(".csv")),
            "-q",
        ],
        check=True,
    )
    return stem


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


def polygon_clip(*, polygon: np.ndarray) -> np.ndarray | None:
    """Returns `polygon` clipped to z <= 0, or `None` when nothing is left."""
    kept = []
    for a, b in zip(polygon, np.roll(polygon, -1, axis=0)):
        a_in, b_in = a[2] <= 0.0, b[2] <= 0.0
        if a_in:
            kept.append(a)
        if a_in != b_in:
            kept.append(a + a[2] / (a[2] - b[2]) * (b - a))
    return np.array(kept) if len(kept) >= 3 else None


def hex_section(*, corners: np.ndarray) -> np.ndarray | None:
    """Returns the polygon where the plane z = 0 closes the lower part of a hex.

    A hex that lies wholly above the plane, or touches it only from above,
    returns `None`.  A hex whose top face lies on the plane returns that face.
    """
    z = corners[:, 2]
    if z.min() >= -EPS or z.max() <= -EPS:
        return None
    points = [np.r_[c[:2], 0.0] for c, height in zip(corners, z) if abs(height) <= EPS]
    for i, j in HEX_EDGES:
        if min(z[i], z[j]) < -EPS and max(z[i], z[j]) > EPS:
            t = z[i] / (z[i] - z[j])
            points.append(np.r_[(corners[i] + t * (corners[j] - corners[i]))[:2], 0.0])
    if len(points) < 3:
        return None
    points = np.array(points)
    center = points.mean(axis=0)
    order = np.argsort(np.arctan2(points[:, 1] - center[1], points[:, 0] - center[0]))
    return points[order]


def polygons_draw(
    *,
    ax,
    polygons,
    normals: np.ndarray,
    values: np.ndarray | None,
    width: float | None = 0.2,
    color: tuple = SURFACE,
) -> None:
    """Draws the polygons that face the camera, shaded by their outward normals.

    `polygons` is an array of equal-size polygons, or a list of polygons that
    differ in size.

    With `values`, each polygon takes its color from Viridis on a 0 to 1 scale.
    Otherwise every polygon takes `color`, neutral gray by default.  A `width` of `None`
    draws no edges, for triangles smaller than a pixel.
    """
    # matplotlib has no depth buffer, so back faces must be culled.
    front = normals @ camera() > 0.0
    if isinstance(polygons, np.ndarray):
        polygons = polygons[front]
    else:
        polygons = [p for p, keep in zip(polygons, front) if keep]
    normals = normals[front]
    shade = LIGHT.shade_normals(normals, fraction=1.0)
    if values is None:
        base = np.tile(np.array(color), (len(polygons), 1))
        light = 0.4 + 0.6 * shade
    else:
        base = plt.get_cmap("viridis")(np.clip(values[front], 0.0, 1.0))[:, :3]
        light = 0.7 + 0.3 * shade
    colors = np.clip(base * light[:, None], 0.0, 1.0)
    # Sub-pixel triangles leave anti-aliasing gaps unless edges match faces.
    edges, width = ("k", width) if width else (colors, 0.3)
    ax.add_collection3d(
        Poly3DCollection(polygons, facecolors=colors, edgecolor=edges, linewidth=width)
    )


def axes_set(*, ax, title: str) -> None:
    """Sets the title, limits, ticks, labels, and view of a panel."""
    ax.set_title(title)
    for limit in (ax.set_xlim, ax.set_ylim, ax.set_zlim):
        limit(-1, 1)
    for ticks in (ax.set_xticks, ax.set_yticks, ax.set_zticks):
        ticks([-1, 0, 1])
    ax.set_xlabel("x")
    ax.set_ylabel("y")
    ax.set_zlabel("z")
    ax.set_aspect("equal")
    ax.view_init(elev=ELEVATION, azim=AZIMUTH)


def mesh_draw(
    *, fig, position: int, title: str, stem: Path, cut: bool, columns: int = 3
) -> None:
    """Draws one hex mesh as panel `position`, painted by Minimum Scaled Jacobian."""
    points, hexes = inp_read(path=stem.with_suffix(".inp"))
    metrics = np.genfromtxt(stem.with_suffix(".csv"), delimiter=",", names=True)
    msj = metrics["minimum_scaled_jacobian"]
    count = len(hexes)
    quads, normals, owners = hex_boundary(points=points, hexes=hexes)
    ax = fig.add_subplot(1, columns, position, projection="3d")
    if cut:
        polygons, directions, values = [], [], []
        for quad, normal, owner in zip(quads, normals, owners):
            clipped = polygon_clip(polygon=quad)
            if clipped is not None:
                polygons.append(clipped)
                directions.append(normal)
                values.append(msj[owner])
        for hex_, value in zip(hexes, msj):
            section = hex_section(corners=points[hex_])
            if section is not None:
                polygons.append(section)
                directions.append([0.0, 0.0, 1.0])
                values.append(value)
        quads, normals, values = polygons, np.array(directions), np.array(values)
    else:
        values = msj[owners]
    polygons_draw(ax=ax, polygons=quads, normals=normals, values=values)
    axes_set(ax=ax, title=f"{title} ({count:,} hexes)")


def colorbar_add(*, fig) -> None:
    """Fits the panels and adds the Minimum Scaled Jacobian color bar."""
    fig.tight_layout(rect=(0.0, 0.0, 0.93, 0.95))
    bar = fig.add_axes((0.94, 0.2, 0.012, 0.6))
    fig.colorbar(
        ScalarMappable(norm=Normalize(0.0, 1.0), cmap="viridis"),
        cax=bar,
        label="Minimum Scaled Jacobian",
    )


def voxel_faces(*, voxels: np.ndarray) -> tuple:
    """Returns the exposed voxel faces as quads, with one outward normal per quad."""
    padded = np.pad(voxels, 1)
    quads, normals = [], []
    for axis in range(3):
        u, w = np.eye(3, dtype=int)[[(axis + 1) % 3, (axis + 2) % 3]]
        for sign in (-1, 1):
            neighbor = np.roll(padded, -sign, axis=axis)
            base = np.argwhere(padded & ~neighbor) - 1
            if sign == 1:
                base[:, axis] += 1
            quads.append(np.stack([base, base + u, base + u + w, base + w], axis=1))
            normal = np.zeros(3)
            normal[axis] = sign
            normals.append(np.tile(normal, (len(base), 1)))
    return np.concatenate(quads).astype(float), np.concatenate(normals)


def voxels_plot(*, output: Path) -> None:
    """Draws the segmentation for each radius in `RADII`, in voxel units."""
    fig = plt.figure(figsize=(5 * len(RADII), 5))
    for index, radius in enumerate(RADII, start=1):
        voxels = sphere(radius=radius).astype(bool)
        quads, normals = voxel_faces(voxels=voxels)
        ax = fig.add_subplot(1, len(RADII), index, projection="3d")
        polygons_draw(
            ax=ax,
            polygons=quads,
            normals=normals,
            values=None,
            width=2 / radius,
            color=VOXEL,
        )
        for limit in (ax.set_xlim, ax.set_ylim, ax.set_zlim):
            limit(0, voxels.shape[0])
        for ticks in (ax.set_xticks, ax.set_yticks, ax.set_zticks):
            ticks([0, radius, 2 * radius])
        ax.set_xlabel("x (voxels)")
        ax.set_ylabel("y (voxels)")
        ax.set_zlabel("z (voxels)")
        ax.set_aspect("equal")
        ax.view_init(elev=ELEVATION, azim=AZIMUTH)
        ax.set_title(f"n = {radius} ({int(voxels.sum()):,} voxels)")
    fig.tight_layout(rect=(0.0, 0.0, 1.0, 0.95))
    fig.savefig(output, dpi=DPI)
    plt.close(fig)


def surfaces_plot(*, output: Path) -> None:
    """Draws the marching cubes surfaces for each radius in `RADII`, side by side."""
    fig = plt.figure(figsize=(5 * len(RADII), 5))
    for index, radius in enumerate(RADII, start=1):
        triangles = surface_make(radius=radius)
        normals = np.cross(
            triangles[:, 1] - triangles[:, 0], triangles[:, 2] - triangles[:, 0]
        )
        ax = fig.add_subplot(1, len(RADII), index, projection="3d")
        polygons_draw(
            ax=ax,
            polygons=triangles,
            normals=normals,
            values=None,
            width=1 / radius if radius <= EDGES_MAX else None,
        )
        axes_set(ax=ax, title=f"n = {radius} ({len(triangles):,} triangles)")
    fig.tight_layout(rect=(0.0, 0.0, 1.0, 0.95))
    fig.savefig(output, dpi=DPI)
    plt.close(fig)


def figure_plot(
    *, triangles: np.ndarray, meshes: tuple, output: Path, cut: bool
) -> None:
    """Draws the surface and the meshes side by side.

    Each mesh is a title and the path of its `.inp` and `.csv` files, without
    the extension.  Each element takes its color from its Minimum Scaled
    Jacobian.  With `cut`, every triangle and hex is clipped at z = 0, and the
    lower half stays.  The surface's normals flip, so the camera sees the inside
    of its lower half.
    """
    fig = plt.figure(figsize=(15, 5))
    ax = fig.add_subplot(1, 3, 1, projection="3d")
    normals = np.cross(
        triangles[:, 1] - triangles[:, 0], triangles[:, 2] - triangles[:, 0]
    )
    shown = triangles
    if cut:
        clipped = [polygon_clip(polygon=t) for t in triangles]
        keep = [i for i, c in enumerate(clipped) if c is not None]
        shown, normals = [clipped[i] for i in keep], -normals[keep]
    polygons_draw(ax=ax, polygons=shown, normals=normals, values=None)
    axes_set(ax=ax, title=f"marching cubes, n = {N} ({len(triangles):,} triangles)")
    for index, (title, stem) in enumerate(meshes, start=2):
        mesh_draw(fig=fig, position=index, title=title, stem=stem, cut=cut)
    colorbar_add(fig=fig)
    fig.savefig(output, dpi=DPI)
    plt.close(fig)


def hexes_plot(*, meshes: tuple, output: Path, cut: bool) -> None:
    """Draws hex meshes side by side, each hex painted by Minimum Scaled Jacobian.

    Each mesh is a title and the path of its `.inp` and `.csv` files, without
    the extension.  With `cut`, every hex is clipped at z = 0, and the lower
    half stays.
    """
    fig = plt.figure(figsize=(5 * len(meshes), 5))
    for index, (title, stem) in enumerate(meshes, start=1):
        mesh_draw(
            fig=fig,
            position=index,
            title=title,
            stem=stem,
            cut=cut,
            columns=len(meshes),
        )
    colorbar_add(fig=fig)
    fig.savefig(output, dpi=DPI)
    plt.close(fig)


def minima_mark(*, ax, values: list) -> None:
    """Marks the minimum of each series on the x-axis, with its value above.

    The labels stagger in height, so minima that sit close together do not overlap.
    """
    for index, (v, style) in enumerate(zip(values, HISTOGRAM, strict=True)):
        ax.plot(
            [v.min()],
            [0.0],
            marker="^",
            markersize=9,
            color=style["color"],
            transform=ax.get_xaxis_transform(),
            clip_on=False,
            zorder=5,
        )
        ax.text(
            v.min(),
            0.05 + 0.09 * index,
            f"min {v.min():.3f}",
            transform=ax.get_xaxis_transform(),
            color=style["color"],
            fontsize=8,
            fontweight="bold",
            ha="center",
            va="bottom",
            bbox={"facecolor": "white", "edgecolor": "none", "alpha": 0.8, "pad": 1.0},
            zorder=6,
        )


def quality_plot(*, meshes: tuple, output: Path) -> None:
    """Draws the four quality histograms.

    Each mesh is a label and the path of its `.csv` file, without the extension.
    """
    data = [
        (label, np.genfromtxt(stem.with_suffix(".csv"), delimiter=",", names=True))
        for label, stem in meshes
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
        if key == "minimum_scaled_jacobian":
            minima_mark(ax=ax, values=values)
        ax.set_yscale("log")
        ax.set_title(title, color=INK, fontsize=11)
        ax.set_xlabel(title, color=INK, fontsize=9)
        ax.set_ylabel("Hex Count (int)", color=INK, fontsize=9)
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
    fig.savefig(output, dpi=DPI)
    plt.close(fig)


def main() -> None:
    here = Path(__file__).parent
    version = subprocess.run(
        ["automesh", "--version"], check=True, capture_output=True, text=True
    ).stdout.strip()
    print(version)
    triangles = surface_make(radius=N)
    with tempfile.TemporaryDirectory() as scratch:
        stl = Path(scratch) / f"unit_sphere_v2_n{N:03d}.stl"
        stl_write(path=stl, triangles=triangles)
        meshes = (
            ("uniform", automesh_run(stl=stl, name="uniform", uniform=CELL)),
            ("adaptive", automesh_run(stl=stl, name="adaptive", uniform=None)),
        )
        sculpts = {
            name: sculpt_metrics(
                exodus=here / f"unit_sphere_sculpt_{name}.e.1.0",
                scratch=Path(scratch),
            )
            for name in ("octa03", f"n{N:03d}", "n160")
        }
        sculpt = sculpts[f"n{N:03d}"]
        rows = (("Sculpt", sculpt),) + meshes
        print(
            f"{'mesh':>9}  {'hexes':>8}  {'MSJ min':>8}  {'MSJ mean':>8}  {'inverted':>8}"
        )
        for title, stem in rows:
            msj = np.genfromtxt(stem.with_suffix(".csv"), delimiter=",", names=True)[
                "minimum_scaled_jacobian"
            ]
            print(
                f"{title:>9}  {len(msj):>8,}  {msj.min():>8.3f}  {msj.mean():>8.3f}  "
                f"{int((msj <= 0.0).sum()):>8}"
            )
        quality_plot(
            meshes=tuple(
                (
                    f"{label} ({len(np.genfromtxt(stem.with_suffix('.csv'), delimiter=',', names=True)):,} hexes)",
                    stem,
                )
                for label, stem in (
                    ("Sculpt", sculpt),
                    ("automesh uniform", meshes[0][1]),
                    ("automesh adaptive", meshes[1][1]),
                )
            ),
            output=here / "unit_sphere_v2_quality.png",
        )
        voxels_plot(output=here / "unit_sphere_v2_voxels.png")
        surfaces_plot(output=here / "unit_sphere_v2_surfaces.png")
        panels = (
            ("Octa-Loop level 3", sculpts["octa03"]),
            (f"n = {N}", sculpt),
            ("n = 160", sculpts["n160"]),
        )
        for output, cut in (
            ("unit_sphere_v2_sculpt.png", False),
            ("unit_sphere_v2_sculpt_cut.png", True),
        ):
            hexes_plot(meshes=panels, output=here / output, cut=cut)
        for output, cut in (
            ("unit_sphere_v2_meshes.png", False),
            ("unit_sphere_v2_meshes_cut.png", True),
        ):
            figure_plot(
                triangles=triangles, meshes=meshes, output=here / output, cut=cut
            )


if __name__ == "__main__":
    main()
