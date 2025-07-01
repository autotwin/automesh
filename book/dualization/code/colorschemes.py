from typing import NamedTuple

import matplotlib.colors as mcolors
import matplotlib.pyplot as plt
from matplotlib import patches

N_COLORS = 8  # Number of discrete colors to extract


# Pre-compute the colors for better performance
def plasma_color_palette(n_colors: int, reversed: bool):
    """Create a palette of discrete plasma colors."""
    colormap = plt.cm.plasma_r if reversed else plt.cm.plasma
    color_indices = [i / (n_colors - 1) for i in range(n_colors)]
    return [mcolors.to_hex(colormap(idx)) for idx in color_indices]


def grayscale_color_palette(n_colors: int, reversed: bool):
    """Create a palette of discrete grayscale colors between 0.05 and 0.95."""
    # Define the range for the grayscale values
    min_gray = 0.05
    max_gray = 0.95

    # Calculate the step size based on the number of colors
    step = (max_gray - min_gray) / (n_colors - 1)

    # Generate color indices within the specified range
    color_indices = [min_gray + i * step for i in range(n_colors)]

    # Select the colormap and reverse if needed
    colormap = plt.cm.gray_r if reversed else plt.cm.gray

    # Convert the grayscale values to hex colors
    return [mcolors.to_hex(colormap(idx)) for idx in color_indices]


# Pre-computed color palettes (more efficient if called many times)
# The plasma colorscheme goes from dark purple to bright yellow.
# The plasma_r (reversed) goes from bright yellow to dark purple.
PLASMA_COLORS = plasma_color_palette(N_COLORS, reversed=False)
PLASMA_R_COLORS = plasma_color_palette(N_COLORS, reversed=True)
GRAYSCALE_COLORS = grayscale_color_palette(N_COLORS, reversed=False)
GRAYSCALE_R_COLORS = grayscale_color_palette(N_COLORS, reversed=True)


class Color(NamedTuple):
    """A color for a particular level of a quadtree."""

    edgecolor: str
    facecolor: str
    zorder: int
    alpha: float = 0.8


def level_to_colorscheme(
    level: int, n_levels: int, plasma: bool, reversed: bool
) -> Color:
    """Returns a color based on the level of the quadtree.

    Args:
        level: The level of the quadtree node
        n_levels: Total number of levels in the quadtree
        plasma: If True, use plasma colors; if False, use original grayscale
        reversed: If True, use reversed plasma or reversed grayscale
    """
    match (plasma, reversed):
        case (True, True):
            facecolor = PLASMA_R_COLORS[level]
        case (True, False):
            facecolor = PLASMA_COLORS[level]
        case (False, True):
            facecolor = GRAYSCALE_R_COLORS[level]
        case (False, False):
            facecolor = GRAYSCALE_COLORS[level]

    edgecolor = "black"
    zorder = 2 + level
    return Color(edgecolor, facecolor, zorder)


def show_color_schemes():
    """Shows the different color schemes."""

    fig, axes = plt.subplots(1, 4, figsize=(15, 5))

    # Create sample rectangles to show the colors
    levels = range(N_COLORS)
    n_levels = N_COLORS

    # Grayscale
    ax1 = axes[0]
    for i, level in enumerate(levels):
        scheme = level_to_colorscheme(level, n_levels, plasma=False, reversed=False)
        rect = patches.Rectangle(
            (0, i),
            1,
            0.8,
            facecolor=scheme.facecolor,
            edgecolor=scheme.edgecolor,
            alpha=scheme.alpha,
        )
        ax1.add_patch(rect)
        ax1.text(0.5, i + 0.4, f"Level {level}", ha="center", va="center")

    ax1.set_xlim(-0.1, 1.1)
    ax1.set_ylim(-0.1, len(levels))
    ax1.set_title("Grayscale")
    ax1.set_xticks([])
    ax1.set_yticks([])

    # Grayscale reversed
    ax2 = axes[1]
    for i, level in enumerate(levels):
        scheme = level_to_colorscheme(level, n_levels, plasma=False, reversed=True)
        rect = patches.Rectangle(
            (0, i),
            1,
            0.8,
            facecolor=scheme.facecolor,
            edgecolor=scheme.edgecolor,
            alpha=scheme.alpha,
        )
        ax2.add_patch(rect)
        ax2.text(0.5, i + 0.4, f"Level {level}", ha="center", va="center")

    ax2.set_xlim(-0.1, 1.1)
    ax2.set_ylim(-0.1, N_COLORS)
    ax2.set_title("Inverted Grayscale")
    ax2.set_xticks([])
    ax2.set_yticks([])

    # Plasma colors
    ax3 = axes[2]
    for i, level in enumerate(levels):
        scheme = level_to_colorscheme(level, n_levels, plasma=True, reversed=False)
        rect = patches.Rectangle(
            (0, i),
            1,
            0.8,
            facecolor=scheme.facecolor,
            edgecolor=scheme.edgecolor,
            alpha=scheme.alpha,
        )
        ax3.add_patch(rect)
        ax3.text(0.5, i + 0.4, f"Level {level}", ha="center", va="center")

    ax3.set_xlim(-0.1, 1.1)
    ax3.set_ylim(-0.1, N_COLORS)
    ax3.set_title("Plasma Colors")
    ax3.set_xticks([])
    ax3.set_yticks([])

    # Plasma_r colors (reversed)
    ax4 = axes[3]
    for i, level in enumerate(levels):
        scheme = level_to_colorscheme(level, n_levels, plasma=True, reversed=True)
        rect = patches.Rectangle(
            (0, i),
            1,
            0.8,
            facecolor=scheme.facecolor,
            edgecolor=scheme.edgecolor,
            alpha=scheme.alpha,
        )
        ax4.add_patch(rect)
        ax4.text(0.5, i + 0.4, f"Level {level}", ha="center", va="center")

    ax4.set_xlim(-0.1, 1.1)
    ax4.set_ylim(-0.1, len(levels))
    ax4.set_title("Plasma_r Colors (Reversed)")
    ax4.set_xticks([])
    ax4.set_yticks([])

    plt.tight_layout()
    plt.show()


def level_to_colorscheme_optimized(level: int, color_scheme: str = "plasma") -> Color:
    """Optimized version using pre-computed colors.

    Args:
        level: The level of the quadtree node
        color_scheme: "grayscale", "plasma", or "plasma_r"
    """
    if color_scheme == "grayscale":
        colors = ["dimgray", "lightgray", "silver", "gainsboro", "whitesmoke", "white"]
    elif color_scheme == "plasma":
        colors = PLASMA_COLORS
    elif color_scheme == "plasma_r":
        colors = PLASMA_R_COLORS
    else:
        raise ValueError(f"Unknown color scheme: {color_scheme}")

    edgecolor = "black"
    facecolor = colors[level % len(colors)]
    zorder = 2 + level
    return Color(edgecolor, facecolor, zorder)


# Demonstrate the color schemes
if __name__ == "__main__":
    show_color_schemes()

    # Show the extracted colors
    print("Plasma colors:", PLASMA_COLORS)
    print("Plasma_r colors:", PLASMA_R_COLORS)
