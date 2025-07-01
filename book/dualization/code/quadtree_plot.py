"""This module, quadtree.py, creates a simple quadtree and plots it."""

from pathlib import Path
from typing import Final
# from typing import NamedTuple


import matplotlib.pyplot as plt
from matplotlib import patches

from colorschemes import QuadColors


class QuadTree:
    """Defines a quadtree composed of a single parent quad and recursive
    children quads.
    """

    def __init__(self, x, y, width, height, level=0, max_level=2):
        # (x, y, width, height)
        self.boundary = (x, y, width, height)
        self.level = level
        self.max_level = max_level
        self.has_children = False
        self.children = []
        self.subdivide()

    def subdivide(self):
        """Divides the parent quad into four quad children."""
        if self.level <= self.max_level:
            print(
                f"Subdividing quad at level {self.level} with boundary {self.boundary}"
            )
            x, y, width, height = self.boundary
            half_width = width / 2.0
            half_height = height / 2.0

            # Create four children
            self.has_children = True  # overwrite
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

    def draw(self, ax, quadcolors: QuadColors):
        """Draw the quadtree."""
        x, y, width, height = self.boundary
        # Draw the boundary rectangle
        print(
            f"Drawing level {self.level} quad at ({x}, {y}) with width {width} and height {height}"
        )
        rect = patches.Rectangle(
            (x, y),
            width,
            height,
            # linewidth=1,
            linestyle="solid",
            edgecolor=quadcolors.edgecolor,
            facecolor=quadcolors.facecolors[self.level],
            alpha=quadcolors.alpha,
            zorder=2,
        )
        ax.add_patch(rect)

        # Draw children
        # if self.has_children:
        #     print(f"Quad at level {self.level} has children, drawing them.")
        #     for child in self.children:
        #         child.draw(ax, quadcolors)


def main():
    # User input begin
    MAX_LEVEL: Final[int] = 2
    SAVE: Final[bool] = True
    SHOW: Final[bool] = True
    EXT: Final[str] = ".png"  # ".pdf" | ".png" | ".svg"
    DPI: Final[int] = 300

    XMIN = -2  # 1
    XMAX = 2  # 3
    YMIN = -2  # -1
    YMAX = 2  # 1
    WIDTH = XMAX - XMIN
    HEIGHT = YMAX - YMIN
    # User input end

    # Create a figure and axis
    width, height = 6, 6  # inches, inches
    fig, ax = plt.subplots(figsize=(width, height))

    # Create the quadtree with a boundary of (-12, -12, 24, 24)
    quadtree = QuadTree(XMIN, YMIN, WIDTH, HEIGHT, level=0, max_level=MAX_LEVEL)

    quadcolors = QuadColors(
        n_levels=MAX_LEVEL + 1,  # +1 because we start at level 0
        edgecolor="black",
        alpha=0.3,
        plasma=True,
        reversed=False,
    )
    print(f"quadcolors.facecolors: {quadcolors.facecolors}")
    breakpoint()
    # Draw the quadtree
    quadtree.draw(ax, quadcolors)

    # Set limits and aspect
    MARGIN = 0.1 * (XMAX - XMIN)
    ax.set_xlim(XMIN - MARGIN, XMAX + MARGIN)
    ax.set_ylim(YMIN - MARGIN, YMAX + MARGIN)
    ax.set_aspect("equal")
    ax.set_xlabel("x")
    ax.set_ylabel("y")
    ax.set_xticks([])
    ax.set_yticks([])
    GRAMMAR_LEVELS = f"{MAX_LEVEL} Level" if MAX_LEVEL == 1 else f"{MAX_LEVEL} Levels"
    ax.set_title(f"Quadtree with {GRAMMAR_LEVELS} of Refinement")
    plt.grid()
    plt.show()

    if SHOW:
        plt.show()

    if SAVE:
        parent = Path(__file__).parent
        stem = Path(__file__).stem + "_level_" + str(MAX_LEVEL)
        fn = parent.joinpath(stem + EXT)
        # plt.savefig(fn, dpi=DPI, bbox_inches='tight')
        fig.savefig(fn, dpi=DPI)
        print(f"Saved {fn}")


if __name__ == "__main__":
    main()
