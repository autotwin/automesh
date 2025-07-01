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

    def __init__(self, *, x, y, width, height, level=0, max_level=2, verbose=False):
        # (x, y, width, height)
        self.boundary = (x, y, width, height)
        self.level = level
        self.max_level = max_level
        self.has_children = False
        self.children = []
        assert level <= max_level, (
            f"QuadTree level {level} exceeds max_level {max_level}."
        )
        self.verbose = verbose
        self.subdivide()

    def subdivide(self):
        """Divides the parent quad into four quad children."""
        if self.level < self.max_level:
            if self.verbose:
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
                    x=x,
                    y=y,
                    width=half_width,
                    height=half_height,
                    level=self.level + 1,
                    max_level=self.max_level,
                )
            )  # Top-left
            self.children.append(
                QuadTree(
                    x=x + half_width,
                    y=y,
                    width=half_width,
                    height=half_height,
                    level=self.level + 1,
                    max_level=self.max_level,
                )
            )  # Top-right
            self.children.append(
                QuadTree(
                    x=x,
                    y=y + half_height,
                    width=half_width,
                    height=half_height,
                    level=self.level + 1,
                    max_level=self.max_level,
                )
            )  # Bottom-left
            self.children.append(
                QuadTree(
                    x=x + half_width,
                    y=y + half_height,
                    width=half_width,
                    height=half_height,
                    level=self.level + 1,
                    max_level=self.max_level,
                )
            )  # Bottom-right

    def draw(self, ax, quadcolors: QuadColors):
        """Draw the quadtree."""
        x, y, width, height = self.boundary
        # Draw the boundary rectangle
        if self.verbose:
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
        if self.has_children:
            if self.verbose:
                print(f"Quad at level {self.level} has children, drawing them.")
            for child in self.children:
                child.draw(ax, quadcolors)


def main():
    # User input begin
    level_min: Final[int] = 0
    level_max: Final[int] = 1
    SAVE: Final[bool] = True
    SHOW: Final[bool] = True
    EXT: Final[str] = ".png"  # ".pdf" | ".png" | ".svg"
    DPI: Final[int] = 300

    xmin = -2  # 1
    xmax = 2  # 3
    ymin = -2  # -1
    ymax = 2  # 1
    width = xmax - xmin
    height = ymax - ymin
    verbose = True  # Set to True to see debug output
    # User input end

    # Create a figure and axis
    fig_width, fig_height = 6, 6  # inches, inches
    fig, ax = plt.subplots(figsize=(fig_width, fig_height))

    # Create the quadtree with a boundary of (-12, -12, 24, 24)
    qt = QuadTree(
        x=xmin,
        y=ymin,
        width=width,
        height=height,
        level=level_min,
        max_level=level_max,
        verbose=verbose,
    )

    # The number of colors will be the number of levels + 1 because
    # the root level is 0 and we want to include it in the color palette
    n_colors = level_max - level_min + 2
    qc = QuadColors(
        n_levels=n_colors,
        edgecolor="black",
        alpha=0.8,
        plasma=True,
        reversed=False,
    )
    if verbose:
        print(f"quadcolors.facecolors: {qc.facecolors}")
    # Draw the quadtree
    qt.draw(ax, qc)

    # Set limits and aspect
    margin = 0.1 * (xmax - xmin)
    ax.set_xlim(xmin - margin, xmax + margin)
    ax.set_ylim(ymin - margin, ymax + margin)
    ax.set_aspect("equal")
    ax.set_xlabel("x")
    ax.set_ylabel("y")
    # ax.set_xticks([])
    # ax.set_yticks([])
    GRAMMAR_LEVELS = f"{level_max} Level" if level_max == 1 else f"{level_max} Levels"
    ax.set_title(f"Quadtree with {GRAMMAR_LEVELS} of Refinement")
    plt.grid()
    plt.show()

    if SHOW:
        plt.show()

    if SAVE:
        parent = Path(__file__).parent
        stem = Path(__file__).stem + "_level_" + str(level_max)
        fn = parent.joinpath(stem + EXT)
        # plt.savefig(fn, dpi=DPI, bbox_inches='tight')
        fig.savefig(fn, dpi=DPI)
        print(f"Saved {fn}")


if __name__ == "__main__":
    main()
