from typing import NamedTuple

import matplotlib.colors as mcolors
import matplotlib.pyplot as plt
from matplotlib import patches


class QuadColors:
    """A collection of color schemes for quadtree levels."""

    def __init__(
        self,
        n_levels: int,
        edgecolor: str,
        alpha: float,
        plasma: bool = True,
        reversed: bool = False,
    ):
        self.n_levels = n_levels
        self.edgecolor = edgecolor
        self.alpha = alpha
        self.plasma = plasma
        self.reversed = reversed

        match (plasma, reversed):
            case (True, True):
                self.facecolors = plasma_color_palette(n_levels, reversed=True)
            case (True, False):
                self.facecolors = plasma_color_palette(n_levels, reversed=False)
            case (False, True):
                self.facecolors = grayscale_color_palette(n_levels, reversed=True)
            case (False, False):
                self.facecolors = grayscale_color_palette(n_levels, reversed=False)


# Pre-compute the colors for better performance
def plasma_color_palette(n_colors: int, reversed: bool):
    """Create a palette of discrete plasma colors."""
    colormap = plt.cm.plasma_r if reversed else plt.cm.plasma
    color_indices = [i / (n_colors - 1) for i in range(n_colors)]
    return [mcolors.to_hex(colormap(idx)) for idx in color_indices]


# Pre-compute the grays for better performance
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


def show_color_schemes(n_colors: int):
    """Shows the different color schemes."""

    fig, axes = plt.subplots(1, 4, figsize=(15, 5))

    # Create sample rectangles to show the colors
    levels = range(n_colors)
    n_levels = n_colors

    # Grayscale
    grayscale = QuadColors(
        n_levels=n_levels,
        edgecolor="black",
        alpha=0.8,
        plasma=False,
        reversed=False,
    )
    print("Grayscale colors:", grayscale.facecolors)
    ax1 = axes[0]
    for i, level in enumerate(levels):
        rect = patches.Rectangle(
            (0, i),
            1,
            0.8,
            facecolor=grayscale.facecolors[level],
            edgecolor=grayscale.edgecolor,
            alpha=grayscale.alpha,
        )
        ax1.add_patch(rect)
        ax1.text(0.5, i + 0.4, f"Level {level}", ha="center", va="center")

    ax1.set_xlim(-0.1, 1.1)
    ax1.set_ylim(-0.1, n_levels)
    ax1.set_title("Grayscale")
    ax1.set_xticks([])
    ax1.set_yticks([])

    # Grayscale reversed
    grayscale_reversed = QuadColors(
        n_levels=n_levels,
        edgecolor="black",
        alpha=0.8,
        plasma=False,
        reversed=True,
    )
    print("Grayscale colors reversed:", grayscale_reversed.facecolors)
    ax2 = axes[1]
    for i, level in enumerate(levels):
        rect = patches.Rectangle(
            (0, i),
            1,
            0.8,
            facecolor=grayscale_reversed.facecolors[level],
            edgecolor=grayscale_reversed.edgecolor,
            alpha=grayscale_reversed.alpha,
        )
        ax2.add_patch(rect)
        ax2.text(0.5, i + 0.4, f"Level {level}", ha="center", va="center")

    ax2.set_xlim(-0.1, 1.1)
    ax2.set_ylim(-0.1, n_levels)
    ax2.set_title("Grayscale Reversed")
    ax2.set_xticks([])
    ax2.set_yticks([])

    # Plasma colors
    plasma = QuadColors(
        n_levels=n_levels,
        edgecolor="black",
        alpha=0.8,
        plasma=True,
        reversed=False,
    )
    print("Plasma colors:", plasma.facecolors)
    ax3 = axes[2]
    for i, level in enumerate(levels):
        rect = patches.Rectangle(
            (0, i),
            1,
            0.8,
            facecolor=plasma.facecolors[level],
            edgecolor=plasma.edgecolor,
            alpha=plasma.alpha,
        )
        ax3.add_patch(rect)
        ax3.text(0.5, i + 0.4, f"Level {level}", ha="center", va="center")

    ax3.set_xlim(-0.1, 1.1)
    ax3.set_ylim(-0.1, n_levels)
    ax3.set_title("Plasma Colors")
    ax3.set_xticks([])
    ax3.set_yticks([])

    # Plasma_r colors (reversed)
    plasma_reversed = QuadColors(
        n_levels=n_levels,
        edgecolor="black",
        alpha=0.8,
        plasma=True,
        reversed=True,
    )
    print("Plasma colors reversed:", plasma_reversed.facecolors)
    ax4 = axes[3]
    for i, level in enumerate(levels):
        rect = patches.Rectangle(
            (0, i),
            1,
            0.8,
            facecolor=plasma_reversed.facecolors[level],
            edgecolor=plasma_reversed.edgecolor,
            alpha=plasma_reversed.alpha,
        )
        ax4.add_patch(rect)
        ax4.text(0.5, i + 0.4, f"Level {level}", ha="center", va="center")

    ax4.set_xlim(-0.1, 1.1)
    ax4.set_ylim(-0.1, n_levels)
    ax4.set_title("Plasma Colors Reversed")
    ax4.set_xticks([])
    ax4.set_yticks([])

    plt.tight_layout()
    plt.show()


# Demonstrate the color schemes
if __name__ == "__main__":
    n_colors = 8  # Number of discrete colors to extract
    show_color_schemes(n_colors=n_colors)

    # Show the extracted colors
    # print("Plasma colors:", PLASMA_COLORS)
    # print("Plasma_r colors:", PLASMA_R_COLORS)
