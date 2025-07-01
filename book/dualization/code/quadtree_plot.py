"""This module, quadtree.py, creates a simple quadtree and plots it."""

from pathlib import Path
from typing import Final
from typing import NamedTuple


import matplotlib.pyplot as plt
from matplotlib import patches


# class Seed(NamedTuple):
#     """The (x, y) point used to trigger refinement."""
#     x: float
#     y: float


class ColorScheme(NamedTuple):
    """A color scheme for the quadtree.
    The plasma colorscheme goes from dark purple to bright yellow.  The plasma_r
    reverses the plasma colorscheme, thus it goes from bright yellow to dark purple.
    """

    edgecolor: str
    facecolor: str
    zorder: int
    alpha: float = 0.3


def level_to_colorscheme(level: int) -> ColorScheme:
    """Returns a color scheme based on the level of the quadtree."""
    colors = ["dimgray", "lightgray", "silver", "gainsboro", "whitesmoke", "white"]
    edgecolor = "black"
    facecolor = colors[level % len(colors)]
    zorder = 2 + level
    return ColorScheme(edgecolor, facecolor, zorder)


class QuadTree:
    """Defines a quadtree composed of a single parent quad and recursive
    children quads.
    """

    def __init__(self, x, y, width, height, level=0, max_level=2):
        # (x, y, width, height)
        self.boundary = (x, y, width, height)
        self.level = level
        self.max_level = max_level
        self.children = []
        self.subdivide()

    def subdivide(self):
        """Divides the parent quad into four quad children."""
        if self.level < self.max_level:
            x, y, width, height = self.boundary
            half_width = width / 2.0
            half_height = height / 2.0

            # Create four children
            self.children.append(
                QuadTree(
                    x,
                    y,
                    half_width,
                    half_height,
                    self.level + 1,
                    self.max_level,
                )
            )  # Top-left
            self.children.append(
                QuadTree(
                    x + half_width,
                    y,
                    half_width,
                    half_height,
                    self.level + 1,
                    self.max_level,
                )
            )  # Top-right
            self.children.append(
                QuadTree(
                    x,
                    y + half_height,
                    half_width,
                    half_height,
                    self.level + 1,
                    self.max_level,
                )
            )  # Bottom-left
            self.children.append(
                QuadTree(
                    x + half_width,
                    y + half_height,
                    half_width,
                    half_height,
                    self.level + 1,
                    self.max_level,
                )
            )  # Bottom-right

    def draw(self, ax, edgecolor="black", color="dimgray"):
        """Draw the quadtree."""
        x, y, width, height = self.boundary
        # Draw the boundary rectangle
        rect = patches.Rectangle(
            (x, y),
            width,
            height,
            # linewidth=1,
            linestyle="solid",
            edgecolor="black",
            # facecolor="dimgray",
            facecolor=color,
            alpha=0.3,
            zorder=2,
        )
        ax.add_patch(rect)

        # Draw children
        for child in self.children:
            child.draw(ax)


def main():
    # User input begin
    N_LEVELS: Final[int] = 0
    SAVE: Final[bool] = True
    SHOW: Final[bool] = True
    EXT: Final[str] = ".png"  # ".pdf" | ".png" | ".svg"
    DPI: Final[int] = 300

    XMIN = 1
    XMAX = 3
    YMIN = -1
    YMAX = 1
    WIDTH = XMAX - XMIN
    HEIGHT = YMAX - YMIN
    # User input end

    colors = (
        "tab:blue",
        "tab:orange",
        "tab:green",
        "tab:red",
        "tab:purple",
        "tab:brown",
        "tab:pink",
        "tab:gray",
        "tab:olive",
    )

    # Create a figure and axis
    figwidth, figheight = 6, 6  # inches, inches
    fig, ax0 = plt.subplots(figsize=(figwidth, figheight))

    # Create the quadtree with a boundary of (-12, -12, 24, 24)
    quadtree = QuadTree(XMIN, YMIN, WIDTH, HEIGHT, level=0, max_level=N_LEVELS)

    # Draw the quadtree
    quadtree.draw(ax0)

    # Set limits and aspect
    MARGIN = 0.1 * (XMAX - XMIN)
    ax0.set_xlim(XMIN - MARGIN, XMAX + MARGIN)
    ax0.set_ylim(YMIN - MARGIN, YMAX + MARGIN)
    ax0.set_aspect("equal")
    GRAMMAR_LEVELS = f"{N_LEVELS} Level" if N_LEVELS == 1 else f"{N_LEVELS} Levels"
    ax0.set_title(f"Quadtree with {GRAMMAR_LEVELS} of Refinement")
    plt.grid()
    plt.show()

    if SHOW:
        plt.show()

    if SAVE:
        parent = Path(__file__).parent
        stem = Path(__file__).stem + "_level_" + str(N_LEVELS)
        fn = parent.joinpath(stem + EXT)
        # plt.savefig(fn, dpi=DPI, bbox_inches='tight')
        fig.savefig(fn, dpi=DPI)
        print(f"Saved {fn}")


if __name__ == "__main__":
    main()
