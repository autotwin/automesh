r"""This module, smoothing_figures.py, illustrates test cases for
smoothing algorithms.

Example
-------
source ~/autotwin/automesh/.venv/bin/activate
cd ~/autotwin/automesh/book/smoothing
python smoothing_figures.py
"""

import datetime
from pathlib import Path
from typing import Final

from matplotlib.colors import LightSource
import matplotlib.pyplot as plt
import numpy as np

import smoothing as sm
import smoothing_examples as se
import smoothing_types as ty

# Type alias for functional style methods
# https://docs.python.org/3/library/typing.html#type-aliases
# DofSet = ty.DofSet
Hexes = ty.Hexes
Neighbors = ty.Neighbors
NodeHierarchy = ty.NodeHierarchy
Vertex = ty.Vertex
Vertices = ty.Vertices
SmoothingAlgorithm = ty.SmoothingAlgorithm

# Examples
# ex = se.double_x
ex = se.bracket  # overwrite

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
# NUM_COLORS = len(spheres)
NUM_COLORS = 10
VOXEL_ALPHA: Final[float] = 0.9
LINE_ALPHA: Final[float] = 0.5

colors = cmap(np.linspace(0, 1, NUM_COLORS))
lightsource = LightSource(azdeg=325, altdeg=45)  # azimuth, elevation
# lightsource = LightSource(azdeg=325, altdeg=90)  # azimuth, elevation
# OUTPUT_DIR: Final[Path] = Path(__file__).parent
DPI: Final[int] = 300  # resolution, dots per inch
SHOW: Final[bool] = True  # Shows the figure on screen
SAVE: Final[bool] = True  # Saves the .png and .npy files

# output_png_short = ex.file_stem + ".png"
# output_png: Path = (
#     Path(output_dir).expanduser().joinpath(output_png_short)
# )

nx, ny, nz = ex.nelx, ex.nely, ex.nelz
nzp, nyp, nxp = nz + 1, ny + 1, nx + 1
# breakpoint()

vertices_laplace = sm.smooth(
    vv=ex.vertices,
    hexes=ex.elements,
    node_hierarchy=ex.node_hierarchy,
    prescribed_nodes=ex.prescribed_nodes,
    scale_lambda=ex.scale_lambda,
    num_iters=ex.num_iters,
    algorithm=ex.algorithm,
)
# original vertices
xs = [v.x for v in ex.vertices]
ys = [v.y for v in ex.vertices]
zs = [v.z for v in ex.vertices]

# laplace smoothed vertices
xs_l = [v.x for v in vertices_laplace]
ys_l = [v.y for v in vertices_laplace]
zs_l = [v.z for v in vertices_laplace]
# breakpoint()

# draw edge lines
ep = sm.edge_pairs(ex.elements)  # edge pairs
line_segments = [
    (sm.xyz(ex.vertices[p1 - 1]), sm.xyz(ex.vertices[p2 - 1])) for (p1, p2) in ep
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
# fig_stem = Path(__file__).stem
fig_stem = ex.file_stem
# breakpoint()
FIG_EXT: Final[str] = ".png"
bb = fig_path.joinpath(fig_stem + "_iter_" + str(ex.num_iters) + FIG_EXT)
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
