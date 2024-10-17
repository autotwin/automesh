"""This module illustrates test cases for smoothing algorithms.
Example:
--------
source ~/autotwin/automesh/.venv/bin/activate
cd ~/autotwin/automesh/book/examples/smooothing
python smoothing_figures.py
"""

import datetime
from pathlib import Path
from typing import Final

from matplotlib.colors import LightSource
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d import Axes3D
import numpy as np

import smoothing as sm
import smoothing_types as ty

# Type alias for functional style methods
# https://docs.python.org/3/library/typing.html#type-aliases
DofSet = ty.DofSet
Hexes = ty.Hexes
Neighborhoods = ty.Neighborhoods
Vertex = ty.Vertex
Vertices = ty.Vertices
SmoothingAlgorithm = ty.SmoothingAlgorithm

# Double X test case

vertices: Vertices = (
    Vertex(0.0, 0.0, 0.0),
    Vertex(1.0, 0.0, 0.0),
    Vertex(2.0, 0.0, 0.0),
    Vertex(0.0, 1.0, 0.0),
    Vertex(1.0, 1.0, 0.0),
    Vertex(2.0, 1.0, 0.0),
    Vertex(0.0, 0.0, 1.0),
    Vertex(1.0, 0.0, 1.0),
    Vertex(2.0, 0.0, 1.0),
    Vertex(0.0, 1.0, 1.0),
    Vertex(1.0, 1.0, 1.0),
    Vertex(2.0, 1.0, 1.0),
)

# all free degrees of freedom
dofset_ex1: DofSet = (
    (4, 4, 4),
    (4, 4, 4),
    (4, 4, 4),
    (4, 4, 4),
    (4, 4, 4),
    (4, 4, 4),
    (4, 4, 4),
    (4, 4, 4),
    (4, 4, 4),
    (4, 4, 4),
    (4, 4, 4),
    (4, 4, 4),
)

# hierarchical degrees of freedom
dofset_ex2: DofSet = (
    (2, 2, 2),
    (3, 3, 3),
    (4, 4, 4),
    (2, 2, 2),
    (3, 3, 3),
    (4, 4, 4),
    (2, 2, 2),
    (3, 3, 3),
    (4, 4, 4),
    (2, 2, 2),
    (3, 3, 3),
    (4, 4, 4),
)

# dofset = dofset_ex1
dofset = dofset_ex2

elements: Hexes = (
    (1, 2, 5, 4, 7, 8, 11, 10),
    (2, 3, 6, 5, 8, 9, 12, 11),
)

neighborhoods: Neighborhoods = (
    (2, 4, 7),
    (1, 3, 5, 8),
    (2, 6, 9),
    (1, 5, 10),
    (2, 4, 6, 11),
    (3, 5, 12),
    (1, 8, 10),
    (2, 7, 9, 11),
    (3, 8, 12),
    (4, 7, 11),
    (5, 8, 10, 12),
    (6, 9, 11),
)

SCALE_LAMBDA: Final[float] = 0.3  # lambda parameter for Laplace smoothing
SCALE_MU: Final[float] = -0.33  # mu parameter for Taubin smoothing
NUM_ITERS: Final[int] = 1  # number of smoothing iterations
ALGO: Final = SmoothingAlgorithm.LAPLACE

# Visualization
width, height = 10, 5
# width, height = 8, 4
# width, height = 6, 3
fig = plt.figure(figsize=(width, height))
# fig = plt.figure(figsize=(8, 8))
ax = fig.add_subplot(1, 2, 1, projection="3d")  # r1, c2, 1st subplot
ax2 = fig.add_subplot(1, 2, 2, projection="3d")  # r1, c2, 2nd subplot

el, az, roll = 63, -110, 0
cmap = plt.get_cmap(name="tab10")
NUM_COLORS = 10
VOXEL_ALPHA: Final[float] = 0.9
LINE_ALPHA: Final[float] = 0.5

colors = cmap(np.linspace(0, 1, NUM_COLORS))
lightsource = LightSource(azdeg=325, altdeg=45)  # azimuth, elevation
# lightsource = LightSource(azdeg=325, altdeg=90)  # azimuth, elevation
DPI: Final[int] = 300  # resolution, dots per inch
SHOW: Final[bool] = True  # turn to True to show the figure on screen
SAVE: Final[bool] = False  # turn to True to save .png and .npy files

nx, ny, nz = 2, 1, 1
nzp, nyp, nxp = nz + 1, ny + 1, nx + 1

vertices_laplace = sm.smooth(
    vv=vertices,
    nn=neighborhoods,
    ds=dofset,
    sf=SCALE_LAMBDA,
    num_iters=NUM_ITERS,
    algo=ALGO,
)
# original vertices
xs = [v.x for v in vertices]
ys = [v.y for v in vertices]
zs = [v.z for v in vertices]
# laplace smoothed vertices
xs_l = [v.x for v in vertices_laplace]
ys_l = [v.y for v in vertices_laplace]
zs_l = [v.z for v in vertices_laplace]
# draw edge lines
ep = sm.edge_pairs(elements)  # edge pairs
line_segments = [
    (sm.xyz(vertices[p1 - 1]), sm.xyz(vertices[p2 - 1])) for (p1, p2) in ep
]
line_segments_laplace = [
    (sm.xyz(vertices_laplace[p1 - 1]), sm.xyz(vertices_laplace[p2 - 1]))
    for (p1, p2) in ep
]
for ls in line_segments:
    x0x1 = [pt[0] for pt in ls]
    y0y1 = [pt[1] for pt in ls]
    z0z1 = [pt[2] for pt in ls]
    ax.plot3D(
        x0x1,
        y0y1,
        z0z1,
        linestyle="solid",
        linewidth=0.5,
        color="blue",
    )
# draw nodes
ax.scatter(
    xs,
    ys,
    zs,
    s=20,
    facecolors="blue",
    edgecolors="none",
)

# repeat with lighter color on second axis
for ls in line_segments:
    x0x1 = [pt[0] for pt in ls]
    y0y1 = [pt[1] for pt in ls]
    z0z1 = [pt[2] for pt in ls]
    ax2.plot3D(
        x0x1,
        y0y1,
        z0z1,
        linestyle="dashed",
        linewidth=0.5,
        color="blue",
        alpha=LINE_ALPHA,
    )
for ls in line_segments_laplace:
    x0x1 = [pt[0] for pt in ls]
    y0y1 = [pt[1] for pt in ls]
    z0z1 = [pt[2] for pt in ls]
    ax2.plot3D(
        x0x1,
        y0y1,
        z0z1,
        linestyle="solid",
        linewidth=0.5,
        color="red",
    )
ax2.scatter(
    xs,
    ys,
    zs,
    s=20,
    facecolors="blue",
    edgecolors="none",
    alpha=0.5,
)

ax2.scatter(
    xs_l,
    ys_l,
    zs_l,
    s=20,
    facecolors="red",
    edgecolors="none",
)

# Set labels for the axes
ax.set_xlabel("x")
ax.set_ylabel("y")
ax.set_zlabel("z")
# repeat for the 2nd axis
ax2.set_xlabel("x")
ax2.set_ylabel("y")
ax2.set_zlabel("z")

x_ticks = list(range(nxp))
y_ticks = list(range(nyp))
z_ticks = list(range(nzp))

ax.set_xticks(x_ticks)
ax.set_yticks(y_ticks)
ax.set_zticks(z_ticks)
# repeat for the 2nd axis
ax2.set_xticks(x_ticks)
ax2.set_yticks(y_ticks)
ax2.set_zticks(z_ticks)

ax.set_xlim(float(x_ticks[0]), float(x_ticks[-1]))
ax.set_ylim(float(y_ticks[0]), float(y_ticks[-1]))
ax.set_zlim(float(z_ticks[0]), float(z_ticks[-1]))
# repeat for the 2nd axis
ax2.set_xlim(float(x_ticks[0]), float(x_ticks[-1]))
ax2.set_ylim(float(y_ticks[0]), float(y_ticks[-1]))
ax2.set_zlim(float(z_ticks[0]), float(z_ticks[-1]))


# Set the camera view
ax.set_aspect("equal")
ax.view_init(elev=el, azim=az, roll=roll)
# # Set the projection to orthographic
# # ax.view_init(elev=0, azim=-90)  # Adjust the view angle if needed
# repeat for the 2nd axis
ax2.set_aspect("equal")
ax2.view_init(elev=el, azim=az, roll=roll)

# File name
aa = Path(__file__)
fig_path = Path(__file__).parent
fig_stem = Path(__file__).stem
FIG_EXT: Final[str] = ".png"
bb = fig_path.joinpath(fig_stem + "_iter_" + str(NUM_ITERS) + FIG_EXT)
# Add a footnote
# Get the current date and time in UTC
now_utc = datetime.datetime.now(datetime.UTC)
# Format the date and time as a string
timestamp_utc = now_utc.strftime("%Y-%m-%d %H:%M:%S UTC")
fn = f"Figure: {bb.name} "
fn += f"created with {__file__}\non {timestamp_utc}."
fig.text(0.5, 0.01, fn, ha="center", fontsize=8)

# fig.tight_layout()  # don't use as it clips the x-axis label
if SHOW:
    plt.show()

    if SAVE:
        fig.savefig(bb, dpi=DPI)
        print(f"Saved: {bb}")

print("End of script.")
