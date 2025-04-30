"""This module, quadtree.py, creates a simple quadtree and plots it."""

from pathlib import Path
from typing import NamedTuple

import matplotlib.pyplot as plt
from matplotlib import patches


# class Seed(NamedTuple):
#     """The (x, y) point used to trigger refinement."""
#     x: float
#     y: float


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
            half_width = width / 2
            half_height = height / 2

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

    def draw(self, ax):
        """Draw the quadtree."""
        x, y, width, height = self.boundary
        # Draw the boundary rectangle
        rect = patches.Rectangle(
            (x, y),
            width,
            height,
            linewidth=1,
            edgecolor="blue",
            facecolor="dimgray",
            alpha=0.3,
            zorder=2,
        )
        ax.add_patch(rect)

        # Draw children
        for child in self.children:
            child.draw(ax)


# User input begin


N_LEVELS = 2
SAVE = True
DPI = 300

XMIN = -12
XMAX = 12
YMIN = -12
YMAX = 12
WIDTH = XMAX - XMIN
HEIGHT = YMAX - YMIN

# User input end


# Create a figure and axis
figwidth, figheight = 8, 8
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
ax0.set_title(f"Quadtree with {N_LEVELS} Levels of Refinement")
plt.grid()
plt.show()

if SAVE:
    stem = Path(__file__).stem + "_level_" + str(N_LEVELS)
    EXT = ".png"
    fn = stem + EXT
    # plt.savefig(fn, dpi=DPI, bbox_inches='tight')
    fig.savefig(fn, dpi=DPI)
    print(f"Saved {fn}")
