"""Visualize nested segmented cubes with spheres at their centers."""

from typing import Final, NamedTuple

import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d import Axes3D
from mpl_toolkits.mplot3d.art3d import Poly3DCollection
import numpy as np
import numpy.typing as npt


# --- Configuration using NamedTuple (Idiomatic and Robust) ---
class ViewParams(NamedTuple):
    """Container for 3D view angles."""

    elev: float = 63  # elevation
    azim: float = -110  # azimuth
    roll: float = 0


DPI: Final[int] = 300  # resolution, dots per inch
OUTER_SIZE: Final[float] = 10.0  # mm, Common size for the bounding box
SHOW: Final[bool] = True  # Post-processing visuals, show on screen
SAVE: Final[bool] = True  # Save the .png file
VIEW: Final[ViewParams] = ViewParams()

# -------------------- Helper Functions --------------------


def draw_cube(
    ax: Axes3D,
    origin: npt.ArrayLike,
    size: float,
    alpha: float = 0.1,
    edge_color: str = "b",
    face_color: str = "cyan",
) -> None:
    """
    Draw a transparent cube with edges using a simplified vertex definition.

    Args:
        origin: The [x, y, z] coordinate of the cube's minimum corner.
    """
    x, y, z = origin

    # Generate all 8 vertices efficiently using broadcasting
    _coords = [np.array([c, c + size]) for c in origin]
    X, Y, Z = np.meshgrid(*_coords, indexing="ij")

    # Vertices (1D array of 8 x 3 points)
    vertices = np.stack([X.ravel(), Y.ravel(), Z.ravel()], axis=1)

    # Define the indices of the 4 vertices for each of the 6 faces
    # Indices correspond to the order in which they appear in the ravelled meshgrid
    face_indices = [
        [0, 1, 3, 2],  # Bottom (Z=z)
        [4, 5, 7, 6],  # Top (Z=z+size)
        [0, 1, 5, 4],  # Front (Y=y)
        [2, 3, 7, 6],  # Back (Y=y+size)
        [0, 2, 6, 4],  # Left (X=x)
        [1, 3, 7, 5],  # Right (X=x+size)
    ]

    # Get the coordinates for each face
    faces = vertices[face_indices]

    # Create and add the 3D polygon collection
    cube = Poly3DCollection(
        faces,
        alpha=alpha,
        facecolor=face_color,
        edgecolor=edge_color,
        linewidth=0.5,  # Adjusted linewidth
    )
    ax.add_collection3d(cube)


def draw_sphere(
    ax: Axes3D,
    center: npt.ArrayLike,
    radius: float,
    color: str = "red",
    alpha: float = 0.8,
    n_points: int = 30,
) -> None:
    """Draw a sphere at the given center."""
    u = np.linspace(0, 2 * np.pi, n_points)
    v = np.linspace(0, np.pi, n_points)

    x = center[0] + radius * np.outer(np.cos(u), np.sin(v))
    y = center[1] + radius * np.outer(np.sin(u), np.sin(v))
    z = center[2] + radius * np.outer(np.ones(np.size(u)), np.cos(v))

    # Use shade=False for cleaner appearance with uniform color
    ax.plot_surface(x, y, z, color=color, alpha=alpha, shade=False)


def setup_isometric_view(ax: Axes3D, size: float, view_params: ViewParams) -> None:
    """Set up isometric view and clean axes."""
    ax.view_init(elev=view_params.elev, azim=view_params.azim, roll=view_params.roll)

    # Use LaTeX math mode for cleaner labels (Idiomatic)
    ax.set_xlabel(r"$x$ (length scale)", fontsize=10)
    ax.set_ylabel(r"$y$ (length scale)", fontsize=10)
    ax.set_zlabel(r"$z$ (length scale)", fontsize=10)

    ax.set_xlim([0, size])
    ax.set_ylim([0, size])
    ax.set_zlim([0, size])
    ax.set_box_aspect([1, 1, 1])

    # Set ticks and labels once
    tick_locations = [0, size]
    tick_labels = ["0", r"$s$"]  # Use $s$ for the size label

    ax.set_xticks(tick_locations)
    ax.set_yticks(tick_locations)
    ax.set_zticks(tick_locations)

    ax.set_xticklabels(tick_labels, fontsize=8)
    ax.set_yticklabels(tick_labels, fontsize=8)
    ax.set_zticklabels(tick_labels, fontsize=8)


# -------------------- Core Logic (Generalized and Robust) --------------------


def draw_segmented_cube(ax: Axes3D, grid_n: int, outer_size: float) -> None:
    """
    Draws the outer cube and an N x N x N grid of inner cubes with spheres.
    This function replaces the repetitive logic of subplots 1, 2, and 3.
    """
    # Draw outer cube (same for all subplots)
    draw_cube(
        ax, [0, 0, 0], outer_size, alpha=0.15, edge_color="blue", face_color="cyan"
    )

    # Define parameters for the inner grid
    inner_size = outer_size / grid_n

    # Scale sphere radius dynamically to fit better inside the smallest segment
    # e.g., radius is 0.3 * the segment size
    # sphere_radius = 0.1 * inner_size

    sphere_radius = 0.03 * OUTER_SIZE

    # Loop through the grid
    for i in range(grid_n):
        for j in range(grid_n):
            for k in range(grid_n):
                origin = np.array([i * inner_size, j * inner_size, k * inner_size])

                # Draw the inner segment cube
                draw_cube(
                    ax,
                    origin,
                    inner_size,
                    alpha=0.2,
                    edge_color="darkblue",
                    face_color="lightblue",
                )

                # Draw sphere at center of the segment cube
                center = origin + inner_size / 2.0
                draw_sphere(ax, center, sphere_radius, color="red")

    # Set up the view
    ax.set_title(f"--grid {grid_n} --size $s$", fontsize=12, pad=20)
    setup_isometric_view(ax, outer_size, VIEW)


# -------------------- Main Execution --------------------

if __name__ == "__main__":
    # Define the grid sizes to visualize
    grid_sizes = [1, 2, 3]

    # Create figure and subplots
    fig = plt.figure(figsize=(5 * len(grid_sizes), 5))  # Dynamic figure size

    for index, N in enumerate(grid_sizes):
        # Use a list of axes for cleaner indexing than manual ax1, ax2, ax3
        ax = fig.add_subplot(1, len(grid_sizes), index + 1, projection="3d")

        # Use the generalized function
        draw_segmented_cube(ax, N, OUTER_SIZE)

    # Post-processing (using the original logic for SAVE/SHOW but simplified)
    plt.tight_layout()

    if SAVE:
        plt.savefig("nested_cubes_visualization.png", dpi=DPI, bbox_inches="tight")

    if SHOW:
        plt.show()
