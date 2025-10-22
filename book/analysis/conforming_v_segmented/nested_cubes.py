"""Visualize nested segmented cubes with spheres at their centers."""

from typing import Final, List, NamedTuple

# from matplotlib.colors import LightSource
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d import Axes3D
from mpl_toolkits.mplot3d.art3d import Poly3DCollection
import numpy as np
import numpy.typing as npt


class ViewParams(NamedTuple):
    """Parameters for 3D view setup."""

    elev: float = 63  # elevation angle
    azim: float = -110  # azimuth angle
    roll: float = 0  # roll angle


DPI: Final[int] = 300  # resolution, dots per inch
OUTER_SIZE = 10.0  # mm
SHOW: Final[bool] = True  # Post-processing visuals, show on screen
SAVE: Final[bool] = True  # Save the .png file
SPHERE_RADIUS = 0.3
VIEW: Final[ViewParams] = ViewParams()
# el, az, roll = 63, -110, 0  # used for most visuals
# lightsource = LightSource(azdeg=325, altdeg=45)  # azimuth, elevation


def draw_cube(
    ax: Axes3D,
    origin: List[float],
    size: float,
    alpha: float = 0.1,
    edge_color: str = "b",
    face_color: str = "cyan",
) -> npt.NDArray[np.float64]:
    """
    Draw a transparent cube with edges using a simplified vertex definition.

    Args:
        origin: The [x, y, z] coordinate of the cube's minimum corner.
    """

    x, y, z = origin

    # Define the vertices of the cube
    vertices = np.array(
        [
            [x, y, z],
            [x + size, y, z],
            [x + size, y + size, z],
            [x, y + size, z],  # bottom
            [x, y, z + size],
            [x + size, y, z + size],
            [x + size, y + size, z + size],
            [x, y + size, z + size],  # top
        ]
    )

    # Define the 6 faces of the cube
    faces = [
        [vertices[0], vertices[1], vertices[5], vertices[4]],  # front
        [vertices[2], vertices[3], vertices[7], vertices[6]],  # back
        [vertices[0], vertices[3], vertices[7], vertices[4]],  # left
        [vertices[1], vertices[2], vertices[6], vertices[5]],  # right
        [vertices[0], vertices[1], vertices[2], vertices[3]],  # bottom
        [vertices[4], vertices[5], vertices[6], vertices[7]],  # top
    ]

    # Create the 3D polygon collection
    cube = Poly3DCollection(
        faces, alpha=alpha, facecolor=face_color, edgecolor=edge_color, linewidth=1.5
    )
    ax.add_collection3d(cube)

    return vertices


def draw_sphere(
    ax: Axes3D,
    center: List[float],
    radius: float,
    color: str = "red",
    alpha: float = 0.8,
) -> None:
    """Draw a sphere at the given center."""
    u = np.linspace(0, 2 * np.pi, 20)
    v = np.linspace(0, np.pi, 20)
    x = center[0] + radius * np.outer(np.cos(u), np.sin(v))
    y = center[1] + radius * np.outer(np.sin(u), np.sin(v))
    z = center[2] + radius * np.outer(np.ones(np.size(u)), np.cos(v))
    ax.plot_surface(x, y, z, color=color, alpha=alpha)


def setup_isometric_view(ax: Axes3D, size: float, view_params: ViewParams) -> None:
    """Set up isometric view for the 3D plot."""
    ax.view_init(elev=view_params.elev, azim=view_params.azim, roll=view_params.roll)
    ax.set_xlabel("$x$ (length scale)", fontsize=10)
    ax.set_ylabel("$y$ (length scale)", fontsize=10)
    ax.set_zlabel("$z$ (length scale)", fontsize=10)
    ax.set_xlim([0, size])
    ax.set_ylim([0, size])
    ax.set_zlim([0, size])
    ax.set_box_aspect([1, 1, 1])
    # define tick locaions
    tick_locations = [0, size]
    ax.set_xticks(tick_locations)
    ax.set_yticks(tick_locations)
    ax.set_zticks(tick_locations)
    tick_labels = [0, "s"]
    ax.set_xticklabels(tick_labels, fontsize=8)
    ax.set_yticklabels(tick_labels, fontsize=8)
    ax.set_zticklabels(tick_labels, fontsize=8)


# Create figure with three subplots
fig = plt.figure(figsize=(15, 5))


# ===== Subplot 1: Single cube inset =====
ax1 = fig.add_subplot(131, projection="3d")
ax1.set_title("--grid 1 --size s", fontsize=12, pad=20)

# Draw outer cube
draw_cube(ax1, [0, 0, 0], OUTER_SIZE, alpha=0.15, edge_color="blue", face_color="cyan")

# Draw single inner cube (same size as outer, coincident)
INNER_SIZE_1 = OUTER_SIZE
inner_origin_1 = [0.0, 0.0, 0.0]
draw_cube(
    ax1,
    inner_origin_1,
    INNER_SIZE_1,
    alpha=0.2,
    edge_color="darkblue",
    face_color="lightblue",
)

# Draw sphere at center of inner cube
center_1 = [
    inner_origin_1[0] + INNER_SIZE_1 / 2,
    inner_origin_1[1] + INNER_SIZE_1 / 2,
    inner_origin_1[2] + INNER_SIZE_1 / 2,
]
draw_sphere(ax1, center_1, SPHERE_RADIUS, color="red")

setup_isometric_view(ax1, OUTER_SIZE, VIEW)

# ===== Subplot 2: 2x2x2 cubes inset =====
ax2 = fig.add_subplot(132, projection="3d")
ax2.set_title("--grid 2 --size s", fontsize=12, pad=20)

# Draw outer cube
draw_cube(ax2, [0, 0, 0], OUTER_SIZE, alpha=0.15, edge_color="blue", face_color="cyan")

# Draw 2x2x2 inner cubes
INNER_SIZE_2 = OUTER_SIZE / 2
for i in range(2):
    for j in range(2):
        for k in range(2):
            ORIGIN = [i * INNER_SIZE_2, j * INNER_SIZE_2, k * INNER_SIZE_2]
            draw_cube(
                ax2,
                ORIGIN,
                INNER_SIZE_2,
                alpha=0.2,
                edge_color="darkblue",
                face_color="lightblue",
            )

            # Draw sphere at center of each inner cube
            CENTER = [
                ORIGIN[0] + INNER_SIZE_2 / 2,
                ORIGIN[1] + INNER_SIZE_2 / 2,
                ORIGIN[2] + INNER_SIZE_2 / 2,
            ]
            draw_sphere(ax2, CENTER, SPHERE_RADIUS, color="red")

setup_isometric_view(ax2, OUTER_SIZE, VIEW)

# ===== Subplot 3: 3x3x3 cubes inset =====
ax3 = fig.add_subplot(133, projection="3d")
ax3.set_title("--grid 3 --size s", fontsize=12, pad=20)

# Draw outer cube
draw_cube(ax3, [0, 0, 0], OUTER_SIZE, alpha=0.15, edge_color="blue", face_color="cyan")

# Draw 3x3x3 inner cubes
INNER_SIZE_3 = OUTER_SIZE / 3
SPHERE_RADIUS_3 = 0.2  # Smaller spheres for 3x3x3
for i in range(3):
    for j in range(3):
        for k in range(3):
            ORIGIN = [i * INNER_SIZE_3, j * INNER_SIZE_3, k * INNER_SIZE_3]
            draw_cube(
                ax3,
                ORIGIN,
                INNER_SIZE_3,
                alpha=0.2,
                edge_color="darkblue",
                face_color="lightblue",
            )

            # Draw sphere at center of each inner cube
            center = [
                ORIGIN[0] + INNER_SIZE_3 / 2,
                ORIGIN[1] + INNER_SIZE_3 / 2,
                ORIGIN[2] + INNER_SIZE_3 / 2,
            ]
            draw_sphere(ax3, center, SPHERE_RADIUS_3, color="red")

setup_isometric_view(ax3, OUTER_SIZE, VIEW)

plt.tight_layout()
plt.savefig("nested_cubes_visualization.png", dpi=DPI, bbox_inches="tight")
plt.show()
