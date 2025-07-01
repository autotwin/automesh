"""This module creates a quadtree and plots it."""

from pathlib import Path
from typing import Final, NamedTuple


import matplotlib.pyplot as plt
from matplotlib import patches

# from book.dualization.code.color_schemes import QuadColors
from color_complement import ColorComplement
from color_schemes import QuadColors


class Point(NamedTuple):
    """A point in 2D space."""

    x: float  # x-coordinate
    y: float  # y-coordinate


class Seeds(NamedTuple):
    """A collection of points (seeds) in 2D space."""

    points: list[Point]  # List of Point objects


class Boundary(NamedTuple):
    """A boundary defined by its minimum and maximum
    x and y coordinates."""

    xmin: float  # Minimum x-coordinate
    xmax: float  # Maximum x-coordinate

    ymin: float  # Minimum y-coordinate
    ymax: float  # Maximum y-coordinate


class QuadTree:
    """Defines a quadtree composed of a single parent quad and recursive
    children quads.
    """

    def __init__(
        self,
        *,
        x: float,
        y: float,
        width: float,
        height: float,
        level: int,
        max_level: int,
        seeds: list[Point],
        verbose: bool,
    ):
        # (x, y, width, height)
        self.boundary = Boundary(xmin=x, xmax=x + width, ymin=y, ymax=y + height)
        self.level = level
        self.max_level = max_level
        self.has_children = False
        self.children = []
        assert level <= max_level, (
            f"QuadTree level {level} exceeds max_level {max_level}."
        )
        self.verbose = verbose

        if self.contains_any_point(seeds):
            # If the quad contains any of the seed points, subdivide it
            self.subdivide(seeds=seeds)

    def subdivide(self, seeds: list[Point]):
        """Divides the parent quad into four quad children."""
        if self.level < self.max_level:
            if self.verbose:
                print(
                    f"Subdividing quad at level {self.level} with boundary {self.boundary}"
                )
            x = self.boundary.xmin
            y = self.boundary.ymin
            width = self.boundary.xmax - self.boundary.xmin
            height = self.boundary.ymax - self.boundary.ymin
            half_width = width / 2.0
            half_height = height / 2.0

            self.has_children = True  # overwrite

            # Create four children
            self.children.append(
                QuadTree(
                    x=x,
                    y=y,
                    width=half_width,
                    height=half_height,
                    level=self.level + 1,
                    max_level=self.max_level,
                    seeds=seeds,
                    verbose=self.verbose,
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
                    seeds=seeds,
                    verbose=self.verbose,
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
                    seeds=seeds,
                    verbose=self.verbose,
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
                    seeds=seeds,
                    verbose=self.verbose,
                )
            )  # Bottom-right

    def contains(self, point: Point) -> bool:
        """Check if the quadtree contains a point."""
        # TODO: determine if we want this to be consistent with
        # winding number conventions
        return (
            point.x >= self.boundary.xmin
            and point.x <= self.boundary.xmax
            and point.y >= self.boundary.ymin
            and point.y <= self.boundary.ymax
        )

    def contains_any_point(self, points: list[Point]) -> bool:
        """Check if the quadtree contains any of the given points.
        Python's built-in any() short-circuits: it returns True as
        soon as it finds the first truthy value and stops evaluating the rest.

        """
        # result = any(self.contains(point) for point in points)
        # return result
        return any(self.contains(point) for point in points)

    def draw(self, ax, quadcolors: QuadColors, seeds: list[Point] | None):
        """Draw the quadtree."""
        x = self.boundary.xmin
        y = self.boundary.ymin
        width = self.boundary.xmax - self.boundary.xmin
        height = self.boundary.ymax - self.boundary.ymin
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
            facecolor=ColorComplement.hex_complement(
                quadcolors.facecolors[self.level], "hsv"
            ),
            alpha=quadcolors.alpha,
            zorder=2,
        )
        ax.add_patch(rect)

        # Draw children
        if self.has_children:
            if self.verbose:
                print(f"Quad at level {self.level} has children, drawing them.")
            for child in self.children:
                child.draw(ax, quadcolors, seeds)

        # Draw the seed points, only draw them after we have reached
        # the top level of the quadtree to avoid cluttering the plot
        # with too many points at lower levels.
        if seeds is not None and self.level == self.max_level:
            xs = [seed.x for seed in seeds]
            ys = [seed.y for seed in seeds]
            ax.scatter(
                xs,
                ys,
                marker="o",
                edgecolor=quadcolors.edgecolor,
                color=quadcolors.facecolors[self.level],
                alpha=quadcolors.alpha,
                s=2,  # Adjust size as needed
                zorder=3,
            )


def main():
    # User input begin
    level_min: Final[int] = 0
    level_max: Final[int] = 6
    SAVE: Final[bool] = False
    SHOW: Final[bool] = True
    EXT: Final[str] = ".png"  # ".pdf" | ".png" | ".svg"
    DPI: Final[int] = 300

    xmin = 1
    xmax = 3
    ymin = -1
    ymax = 1
    width = xmax - xmin
    height = ymax - ymin
    verbose = False  # Set to True to see debug output
    seeds = [
        Point(x=2.6, y=0.6),
        Point(x=2.9, y=0.2),
    ]
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
        seeds=seeds,
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
    qt.draw(ax=ax, quadcolors=qc, seeds=seeds)

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
