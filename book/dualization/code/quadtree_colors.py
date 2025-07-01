"""This module provides color schemes for visualizing quadtree levels."""

import matplotlib.colors as mcolors
import matplotlib.pyplot as plt


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


def n_colors_valid(n_colors: int) -> bool:
    """Check if the number of colors is valid.  Ensure n_colors is
    at least 2 for a valid palette.
    """
    if n_colors < 2:
        raise ValueError(f"n_colors {n_colors} must be at least 2 for a color palette.")
    return True


# Pre-compute the colors for better performance
def plasma_color_palette(n_colors: int, reversed: bool):
    """Create a palette of discrete plasma colors."""
    if n_colors_valid(n_colors):
        colormap = plt.cm.plasma_r if reversed else plt.cm.plasma
        color_indices = [i / (n_colors - 1) for i in range(n_colors)]
        return [mcolors.to_hex(colormap(idx)) for idx in color_indices]


# Pre-compute the grays for better performance
def grayscale_color_palette(n_colors: int, reversed: bool):
    """Create a palette of discrete grayscale colors between 0.05 and 0.95."""
    # Define the range for the grayscale values
    if n_colors_valid(n_colors):
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
