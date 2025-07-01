"""This module provides color schemes for visualizing quadtree levels."""

from typing import NamedTuple

import matplotlib.colors as mcolors
import matplotlib.pyplot as plt


class ColorSchemes(NamedTuple):
    """Named tuple for color scheme options."""

    GRAYSCALE: str = "grayscale"
    PLASMA: str = "plasma"
    TAB10: str = "tab10"
    VIRIDIS: str = "viridis"


class DiscreteColors:
    """A collection of color schemes for quadtree levels."""

    def __init__(
        self,
        n_levels: int,
        edgecolor: str,
        alpha: float,
        color_scheme: str,
        reversed: bool = False,
    ):
        self.n_levels = n_levels
        self.edgecolor = edgecolor
        self.alpha = alpha
        self.color_scheme = color_scheme
        self.reversed = reversed

        # Validate color scheme
        valid_schemes = [
            ColorSchemes.GRAYSCALE,
            ColorSchemes.PLASMA,
            ColorSchemes.TAB10,
            ColorSchemes.VIRIDIS,
        ]
        if color_scheme not in valid_schemes:
            raise ValueError(
                f"color_scheme must be one of {valid_schemes}, got '{color_scheme}'"
            )

        # Generate colors based on scheme
        # TODO: match was added in Python 3.10, but we continued to support
        # Python 3.8 and newer, so we use if-elif-else instead.jJ:w
        # match color_scheme:
        #     case ColorSchemes.GRAYSCALE:
        #         self.facecolors = grayscale_color_palette(n_levels, reversed=reversed)
        #     case ColorSchemes.PLASMA:
        #         self.facecolors = plasma_color_palette(n_levels, reversed=reversed)
        #     case ColorSchemes.TAB10:
        #         self.facecolors = tab10_color_palette(n_levels, reversed=reversed)
        #     case ColorSchemes.VIRIDIS:
        #         self.facecolors = viridis_color_palette(n_levels, reversed=reversed)
        #     case _:
        #         # Catch-all fallback, shouldn't happen with validation above
        #         self.facecolors = ["#FF00FF"] * n_levels  # magenta for debugging
        if color_scheme == ColorSchemes.GRAYSCALE:
            self.facecolors = grayscale_color_palette(n_levels, reversed=reversed)
        elif color_scheme == ColorSchemes.PLASMA:
            self.facecolors = plasma_color_palette(n_levels, reversed=reversed)
        elif color_scheme == ColorSchemes.TAB10:
            self.facecolors = tab10_color_palette(n_levels, reversed=reversed)
        elif color_scheme == ColorSchemes.VIRIDIS:
            self.facecolors = viridis_color_palette(n_levels, reversed=reversed)
        else:
            self.facecolors = ["#FF00FF"] * n_levels  # magenta for debugging


def n_colors_valid(n_colors: int) -> bool:
    """Check if the number of colors is valid.  Ensure n_colors is
    at least 2 for a valid palette.
    """
    if n_colors < 2:
        raise ValueError(f"n_colors {n_colors} must be at least 2 for a color palette.")
    return True


def plasma_color_palette(n_colors: int, reversed: bool) -> list[str]:
    """Create a palette of discrete plasma colors."""
    assert n_colors_valid(n_colors)
    colormap = plt.cm.plasma_r if reversed else plt.cm.plasma
    color_indices = [i / (n_colors - 1) for i in range(n_colors)]
    return [mcolors.to_hex(colormap(idx)) for idx in color_indices]


def grayscale_color_palette(n_colors: int, reversed: bool) -> list[str]:
    """Create a palette of discrete grayscale colors between 0.05 and 0.95."""
    # Define the range for the grayscale values
    assert n_colors_valid(n_colors)

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


def tab10_color_palette(n_colors: int, reversed: bool) -> list[str]:
    """Create a palette using matplotlib's tab10 (Tableau 10) colors."""
    assert n_colors_valid(n_colors)

    # Define the tab10 colors in order
    tab10_colors = [
        "tab:gray",
        "tab:blue",
        "tab:orange",
        "tab:green",
        "tab:red",
        "tab:purple",
        "tab:brown",
        "tab:pink",
        # "tab:gray",
        "tab:olive",
        "tab:cyan",
    ]

    # Convert to hex colors
    hex_colors = [mcolors.to_hex(color) for color in tab10_colors]

    # Cycle through colors if we need more than 10
    selected_colors = []
    for i in range(n_colors):
        selected_colors.append(hex_colors[i % len(hex_colors)])

    # Reverse if requested
    if reversed:
        selected_colors = selected_colors[::-1]

    return selected_colors


def viridis_color_palette(n_colors: int, reversed: bool) -> list[str]:
    """Create a palette of discrete viridis colors."""
    assert n_colors_valid(n_colors)
    colormap = plt.cm.viridis_r if reversed else plt.cm.viridis
    color_indices = [i / (n_colors - 1) for i in range(n_colors)]
    return [mcolors.to_hex(colormap(idx)) for idx in color_indices]
