import matplotlib.pyplot as plt
import matplotlib.colors as mcolors
from typing import NamedTuple

import numpy as np


N_COLORS = 6  # Number of discrete colors to extract


class ColorScheme(NamedTuple):
    """A color scheme for the quadtree.
    The plasma colorscheme goes from dark purple to bright yellow.  The plasma_r
    reverses the plasma colorscheme, thus it goes from bright yellow to dark purple.
    """

    edgecolor: str
    facecolor: str
    zorder: int
    alpha: float = 0.8


def grayscale_colors(n_colors: int):
    """Creates n_colors grayscale colors from dark to light.

    Args:
        n_colors: Number of grayscale colors to generate

    Returns:
        List of grayscale color names/hex codes
    """
    if n_colors <= 0:
        return []
    elif n_colors == 1:
        return ["gray"]

    # Generate grayscale values from dark to light
    # Using a range that goes from darker grays to lighter grays/white
    gray_values = np.linspace(0.2, 0.95, n_colors)

    # Convert to hex colors
    grayscale_colors = [mcolors.to_hex((val, val, val)) for val in gray_values]

    return grayscale_colors


def level_to_colorscheme(
    level: int, n_levels, use_plasma: bool = True, reverse: bool = False
) -> ColorScheme:
    """Returns a color scheme based on the level of the quadtree.

    Args:
        level: The level of the quadtree node
        n_levels: Total number of levels in the quadtree
        use_plasma: If True, use plasma colors; if False, use original grayscale
        reverse: If True, use plasma_r (reversed plasma)
    """
    n_colors = n_levels[-1] + 1  # Number of discrete colors to extract
    if not use_plasma:
        # Original grayscale colors
        # colors = ["dimgray", "lightgray", "silver", "gainsboro", "whitesmoke", "white"]
        colors = grayscale_colors(n_colors)
        facecolor = colors[level % len(colors)]
    else:
        # Generate discrete colors from plasma colormap
        # n_colors = 6  # Number of discrete colors to extract
        colormap = plt.cm.plasma_r if reverse else plt.cm.plasma

        # Extract evenly spaced colors from the colormap
        color_indices = [i / (n_colors - 1) for i in range(n_colors)]
        plasma_colors = [colormap(idx) for idx in color_indices]

        # Convert RGBA to hex for easier handling
        plasma_hex_colors = [mcolors.to_hex(color) for color in plasma_colors]

        facecolor = plasma_hex_colors[level % len(plasma_hex_colors)]

    edgecolor = "black"
    zorder = 2 + level
    return ColorScheme(edgecolor, facecolor, zorder)


# Example usage and demonstration
def demonstrate_color_schemes():
    """Demonstrate the different color schemes."""

    fig, axes = plt.subplots(1, 3, figsize=(15, 5))

    # Create sample rectangles to show the colors
    # levels = range(6)
    # levels = range(6)
    levels = range(N_COLORS)

    # Original grayscale
    ax1 = axes[0]
    for i, level in enumerate(levels):
        scheme = level_to_colorscheme(level, levels, use_plasma=False)
        rect = plt.Rectangle(
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
    ax1.set_title("Original Grayscale")
    ax1.set_xticks([])
    ax1.set_yticks([])

    # Plasma colors
    ax2 = axes[1]
    for i, level in enumerate(levels):
        scheme = level_to_colorscheme(level, levels, use_plasma=True, reverse=False)
        rect = plt.Rectangle(
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
    ax2.set_ylim(-0.1, len(levels))
    ax2.set_title("Plasma Colors")
    ax2.set_xticks([])
    ax2.set_yticks([])

    # Plasma_r colors (reversed)
    ax3 = axes[2]
    for i, level in enumerate(levels):
        scheme = level_to_colorscheme(level, levels, use_plasma=True, reverse=True)
        rect = plt.Rectangle(
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
    ax3.set_ylim(-0.1, len(levels))
    ax3.set_title("Plasma_r Colors (Reversed)")
    ax3.set_xticks([])
    ax3.set_yticks([])

    plt.tight_layout()
    plt.show()


# Alternative approach: Pre-compute the colors for better performance
def create_plasma_color_palette(n_colors: int = 6, reverse: bool = False):
    """Create a palette of discrete plasma colors."""
    colormap = plt.cm.plasma_r if reverse else plt.cm.plasma
    color_indices = [i / (n_colors - 1) for i in range(n_colors)]
    return [mcolors.to_hex(colormap(idx)) for idx in color_indices]


# Pre-computed color palettes (more efficient if called many times)
PLASMA_COLORS = create_plasma_color_palette(N_COLORS, reverse=False)
PLASMA_R_COLORS = create_plasma_color_palette(N_COLORS, reverse=True)


def level_to_colorscheme_optimized(
    level: int, color_scheme: str = "plasma"
) -> ColorScheme:
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
    return ColorScheme(edgecolor, facecolor, zorder)


# Demonstrate the color schemes
if __name__ == "__main__":
    demonstrate_color_schemes()

    # Show the extracted colors
    print("Plasma colors:", PLASMA_COLORS)
    print("Plasma_r colors:", PLASMA_R_COLORS)
