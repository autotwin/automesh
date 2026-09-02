#!/usr/bin/env python3
"""Renders the RVE geometry: the solid cube, and the same cube cut open.

This replaces a pair of Cubit screenshots.  Everything here is drawn from the
model's own definition, so the two panels share one camera, one light, and one
palette.

The cut is the plane x = y.  The three pore centers are collinear and all lie
on that plane, so the cut passes through every pore center and each pore
appears as an exact circle on the cut face.  The kept half of the cube is a
triangular prism, so the cut solid is built analytically: two triangular caps,
two outer walls, the cut face (a rectangle less three circles), and the pore
surfaces read from `comparison.stl`.

Faces are shaded by a Lambertian term on the facet normal, which is what gives
the flat-shaded solid its form; matplotlib's 3D axes have no lighting model of
their own.

Example
-------
cd ~/autotwin/automesh/book/examples/gallery/academic
# comparison.stl is not committed; see the Downloads section of rve.md
python rve_geometry_figure.py ~/Downloads/comparison.stl

Output
------
The `rve_geometry.png` visualization file, written next to this script.
"""
import sys
from pathlib import Path

import matplotlib
matplotlib.use("agg")
import matplotlib.pyplot as plt
from matplotlib.tri import Triangulation
from mpl_toolkits.mplot3d.art3d import Poly3DCollection
import numpy as np

# The three pores, from the Cubit journal (R = 1): radii R/10, R/20, R/30.
# Every center satisfies x = y, which is what makes one cut plane enough.
PORES = [
    (np.array([-0.25, -0.25, 0.25]), 1 / 10),
    (np.array([0.00, 0.00, 0.00]), 1 / 20),
    (np.array([0.20, 0.20, -0.20]), 1 / 30),
]

CENTER = np.zeros(3)  # the model is centered on the origin
ELEV = 20.0
AZIM = 30.0
AZIM_CUT = -70.0
LIGHT = np.array([-0.3, -0.7, 0.65])  # direction the light comes from
AMBIENT = 0.45
BODY = np.array([0.36, 0.51, 0.71])   # steel blue, for the solid
INTERIOR = np.array([0.55, 0.36, 0.36])  # warmer, for cut and pore surfaces
SURFACE = "#fcfcfb"
INK = "#0b0b0b"
TOLERANCE = 1.0e-6


def _pore_center(facet):
    """Returns the center of the pore a facet belongs to."""
    middle = facet.mean(axis=0)
    return min(PORES, key=lambda p: np.linalg.norm(middle - p[0]))[0]


def stl_read(*, path):
    """Reads triangular facets from an ASCII STL, shape (n_facets, 3, 3)."""
    vertices = []
    for line in path.read_text(errors="replace").splitlines():
        tokens = line.split()
        if tokens and tokens[0] == "vertex":
            vertices.append([float(v) for v in tokens[1:4]])
    return np.array(vertices, dtype=np.float64).reshape(-1, 3, 3)


def pore_facets_select(*, facets):
    """Selects facets not lying on any bounding plane, i.e. the pore surfaces."""
    points = facets.reshape(-1, 3)
    lower, upper = points.min(axis=0), points.max(axis=0)
    on_boundary = np.zeros(len(facets), dtype=bool)
    for axis in range(3):
        for plane in (lower[axis], upper[axis]):
            flush = np.abs(facets[:, :, axis] - plane) < TOLERANCE
            on_boundary |= flush.all(axis=1)
    return ~on_boundary


def normals_compute(*, polygons):
    """Returns a unit normal per polygon, from its first three vertices."""
    normals = np.cross(polygons[:, 1] - polygons[:, 0],
                       polygons[:, 2] - polygons[:, 0])
    lengths = np.linalg.norm(normals, axis=1, keepdims=True)
    return normals / np.where(lengths < TOLERANCE, 1.0, lengths)


def eye_direction(*, azim, elev=ELEV):
    """Returns the unit vector pointing from the model toward the camera."""
    a, e = np.radians(azim), np.radians(elev)
    return np.array([np.cos(e) * np.cos(a), np.cos(e) * np.sin(a), np.sin(e)])


def facing_select(*, normals, azim):
    """Selects the polygons whose outward normal faces the camera.

    matplotlib sorts whole polygons by centroid depth, so a large back face can
    be painted over the small front-facing triangles it sits behind.  Culling
    back faces removes that failure mode, and for a closed solid it is what a
    renderer should do anyway.
    """
    return normals @ eye_direction(azim=azim) > 0.0


def shades_compute(*, normals, color):
    """Returns one Lambertian-shaded color per polygon normal."""
    light = LIGHT / np.linalg.norm(LIGHT)
    lit = np.abs(normals @ light)
    return np.clip(color * (AMBIENT + (1 - AMBIENT) * lit)[:, None], 0, 1)


BOX_NORMALS = np.array([
    [0.0, 0.0, -1.0], [0.0, 0.0, 1.0],
    [0.0, -1.0, 0.0], [0.0, 1.0, 0.0],
    [-1.0, 0.0, 0.0], [1.0, 0.0, 0.0],
])


def box_quads(*, lower, upper):
    """Returns the six faces of an axis-aligned box, in BOX_NORMALS order."""
    (x0, y0, z0), (x1, y1, z1) = lower, upper
    return [
        np.array([[x0, y0, z0], [x1, y0, z0], [x1, y1, z0], [x0, y1, z0]]),
        np.array([[x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1]]),
        np.array([[x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1]]),
        np.array([[x0, y1, z0], [x1, y1, z0], [x1, y1, z1], [x0, y1, z1]]),
        np.array([[x0, y0, z0], [x0, y1, z0], [x0, y1, z1], [x0, y0, z1]]),
        np.array([[x1, y0, z0], [x1, y1, z0], [x1, y1, z1], [x1, y0, z1]]),
    ]


def cut_face_triangles(*, lower, upper, divisions=90):
    """Triangulates the plane x = y inside the box, less the three pore disks.

    In-plane coordinates are (u, z) with u = sqrt(2) * x, which makes each
    pore's intersection an exact circle rather than an ellipse.
    """
    root = np.sqrt(2.0)
    u_limit = upper[0] * root
    disks = [(center[0] * root, center[2], radius) for center, radius in PORES]

    # A regular grid, plus dense points around each disk so its rim stays round.
    us = np.linspace(-u_limit, u_limit, divisions)
    zs = np.linspace(lower[2], upper[2], divisions)
    grid_u, grid_z = np.meshgrid(us, zs)
    points = [np.column_stack([grid_u.ravel(), grid_z.ravel()])]
    for u_c, z_c, radius in disks:
        angles = np.linspace(0, 2 * np.pi, 180, endpoint=False)
        for scale in (1.0, 1.06):
            points.append(np.column_stack([
                u_c + scale * radius * np.cos(angles),
                z_c + scale * radius * np.sin(angles),
            ]))
    cloud = np.vstack(points)
    inside = np.zeros(len(cloud), dtype=bool)
    for u_c, z_c, radius in disks:
        inside |= np.hypot(cloud[:, 0] - u_c, cloud[:, 1] - z_c) < radius * 0.999
    cloud = cloud[~inside]

    triangulation = Triangulation(cloud[:, 0], cloud[:, 1])
    centers = cloud[triangulation.triangles].mean(axis=1)
    drop = np.zeros(len(centers), dtype=bool)
    for u_c, z_c, radius in disks:
        drop |= np.hypot(centers[:, 0] - u_c, centers[:, 1] - z_c) < radius
    kept = triangulation.triangles[~drop]

    corners = cloud[kept]
    x = corners[:, :, 0] / root
    return np.stack([x, x, corners[:, :, 1]], axis=2)


def panel_render(*, axes, polygons, colors, limits, azim, title):
    """Draws one shaded solid."""
    # Adjacent coplanar triangles otherwise show a hairline seam where their
    # antialiased edges meet, which reads as a mesh drawn on a smooth face.
    collection = Poly3DCollection(polygons, linewidths=0.0, antialiased=False)
    collection.set_facecolor(colors)
    collection.set_edgecolor(colors)
    axes.add_collection3d(collection)
    lower, upper = limits
    axes.set_xlim(lower[0], upper[0])
    axes.set_ylim(lower[1], upper[1])
    axes.set_zlim(lower[2], upper[2])
    axes.set_box_aspect(tuple(upper - lower))
    axes.view_init(elev=ELEV, azim=azim)
    axes.set_axis_off()
    axes.set_title(title, color=INK, fontsize=11)


def geometry_render(*, stl, png):
    """Saves the two-panel geometry figure."""
    facets = stl_read(path=stl)
    pores = pore_facets_select(facets=facets)
    points = facets.reshape(-1, 3)
    lower, upper = points.min(axis=0), points.max(axis=0)

    figure = plt.figure(figsize=(11, 5.5))
    figure.patch.set_facecolor(SURFACE)

    # Left: the solid cube, drawn as six clean quads rather than the STL's
    # twelve triangles, so no diagonal seams cross the faces.
    solid = np.array(box_quads(lower=lower, upper=upper))
    normals = BOX_NORMALS
    seen = facing_select(normals=normals, azim=AZIM)
    axes = figure.add_subplot(121, projection="3d")
    panel_render(axes=axes, polygons=solid[seen],
                 colors=shades_compute(normals=normals[seen], color=BODY),
                 limits=(lower, upper), azim=AZIM, title="solid")

    # Right: the half with x <= y, a triangular prism, opened on the cut plane.
    (x0, y0, z0), (x1, y1, z1) = lower, upper
    walls = [
        np.array([[x0, y0, z0], [x0, y1, z0], [x1, y1, z0]]),          # bottom cap
        np.array([[x0, y0, z1], [x0, y1, z1], [x1, y1, z1]]),          # top cap
        np.array([[x0, y0, z0], [x0, y1, z0], [x0, y1, z1], [x0, y0, z1]]),
        np.array([[x0, y1, z0], [x1, y1, z0], [x1, y1, z1], [x0, y1, z1]]),
    ]
    cut = cut_face_triangles(lower=lower, upper=upper)
    centroids = facets.mean(axis=1)
    kept = facets[pores & (centroids[:, 0] <= centroids[:, 1])]

    # Outward normals are stated, not inferred.  The cut plane passes through
    # the origin, so "away from the model center" is degenerate for it.
    root = np.sqrt(2.0)
    wall_normals = np.array([
        [0.0, 0.0, -1.0],            # bottom cap
        [0.0, 0.0, 1.0],             # top cap
        [-1.0, 0.0, 0.0],            # x = x0 wall
        [0.0, 1.0, 0.0],             # y = y1 wall
    ])
    cut_normal = np.array([1.0, -1.0, 0.0]) / root
    # A pore's outward-from-solid normal points into the cavity.
    pore_normals = normals_compute(polygons=kept)
    toward = np.array([_pore_center(f) - f.mean(axis=0) for f in kept])
    pore_normals *= np.sign((pore_normals * toward).sum(axis=1))[:, None]

    groups = []
    for polygons, normals, color in (
        (walls, wall_normals, BODY),
        (cut, np.repeat(cut_normal[None, :], len(cut), axis=0), INTERIOR),
        (kept, pore_normals, INTERIOR),
    ):
        seen = facing_select(normals=normals, azim=AZIM_CUT)
        groups.append(([p for p, s in zip(polygons, seen) if s],
                       shades_compute(normals=normals[seen], color=color)))

    polygons = [p for group, _ in groups for p in group]
    colors = np.vstack([shade for _, shade in groups if len(shade)])
    axes = figure.add_subplot(122, projection="3d")
    panel_render(axes=axes, polygons=polygons, colors=colors,
                 limits=(lower, upper), azim=AZIM_CUT,
                 title="cut at $x = y$, through all three pores")

    figure.tight_layout()
    figure.savefig(png, dpi=200, facecolor=SURFACE, bbox_inches="tight")
    plt.close(figure)
    print(f"wrote {png.name} ({len(cut)} cut-face triangles, "
          f"{len(kept)} pore facets, {len(polygons)} drawn after culling)")


def main():
    if len(sys.argv) != 2:
        print(f"usage: {sys.argv[0]} <comparison.stl>", file=sys.stderr)
        sys.exit(1)
    here = Path(__file__).resolve().parent
    geometry_render(stl=Path(sys.argv[1]).expanduser(),
                    png=here / "rve_geometry.png")


if __name__ == "__main__":
    main()
