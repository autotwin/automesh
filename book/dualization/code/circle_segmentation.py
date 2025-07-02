"""This module creates a circular segmentation plot."""

from pathlib import Path
from typing import NamedTuple

import numpy as np
import matplotlib.pyplot as plt


def circle(diameter: int, dtype=np.uint8) -> np.ndarray:
    """Create a circular segmentation mask from a diameter that is
    a positive, odd integer with minimum value of 3."""
    assert isinstance(diameter, int), "Diameter must be an integer."
    assert diameter >= 3, "Diameter must be a positive number, minimum of 3."
    # assert diameter % 2 != 0, "Diameter must be an odd number."

    # Create a grid of coordinates
    radius = diameter // 2  # floor division
    if diameter % 2 == 0:
        # Diameter is even
        x = np.arange(-radius, radius + 1, 1)  # Include the center
        y = np.arange(-radius, radius + 1, 1)  # Include the center
        x = x[x != 0]  # Exclude the center column
        y = y[y != 0]  # Exclude the center row
        xx, yy = np.meshgrid(x, y, indexing="ij")
        mask = xx**2 + yy**2 <= radius**2 + 1  # +1 to include the edge
    else:
        # Diameter is odd
        x = np.arange(-radius, radius + 1, 1)  # Include the center
        y = np.arange(-radius, radius + 1, 1)  # Include the center
        xx, yy = np.meshgrid(x, y, indexing="ij")
        mask = xx**2 + yy**2 <= radius**2

    # Convert to the specified dtype
    return mask.astype(dtype)


class Configuration(NamedTuple):
    """User input configuation."""

    diameter: int
    save: bool
    ext: str = ".svg"  # ".pdf" | ".png" | ".svg"


def plot_segmentation(segmentation: np.ndarray, cc: Configuration) -> None:
    """Plot the circular segmentation mask."""

    diameter = segmentation.shape[0]

    fig_width, fig_height = 6, 6  # inches, inches
    # plt.figure(figsize=(fig_width, fig_height))
    fig, ax = plt.subplots(figsize=(fig_width, fig_height))
    # plt.imshow(segmentation, cmap="gray", extent=(-radius, radius, -radius, radius))
    # breakpoint()
    plt.imshow(segmentation, cmap="gray")
    # plt.title(f"Circle Segmentation with Radius {radius}")
    plt.title(f"Circle Segmentation with Diameter {diameter}")
    plt.xlabel("x")
    plt.ylabel("y")
    plt.grid(False)
    plt.axis("equal")

    # In Matplotlib, the default behavior of imshow is to have the origin (0, 0)
    # at the top-left corner of the image, which means that the y-axis increases
    # as you move down the page. To change this behavior so that the y-axis
    # increases as you move up the page, you can use the
    # plt.gca().invert_yaxis() function after calling imshow.
    plt.gca().invert_yaxis()

    # Set the ticks to be at the center of each pixel
    # ticks = np.arange(0, diameter + 1, 1)  # Create ticks from 0 to diameter
    ticks = np.arange(0, diameter, 1)  # Create ticks from 0 to diameter
    if diameter > 20:
        ticks = np.arange(0, diameter, 5)  # Coarsen the ticks for larger diameters
    plt.xticks(ticks - 0.5, labels=ticks, fontsize=8)  # Shift ticks to left
    plt.yticks(ticks - 0.5, labels=ticks, fontsize=8)  # Shift ticks to bottom

    # Calculate font size as 80% of the segmentation size
    # fontsize = 0.8 * (segmentation.shape[0] / 10)  # Scale down for better visibility
    inches_to_points = 72  # 1 inch = 72 points
    fontsize = 0.3 * ((fig_height / diameter) * inches_to_points)
    print(f"Font size: {fontsize}")

    # Annotate the values in the segmentation mask
    # for i in range(segmentation.shape[0]):
    #     for k in range(segmentation.shape[1]):
    for i in range(diameter):
        for k in range(diameter):
            # print(f"segmentation[{i}, {k}] = {segmentation[i, k]}")
            if segmentation[i, k]:
                color = "black"  # Complementary color for white (1)
            else:  # If the value is 0
                color = "white"  # Complementary color for black (0)
            plt.text(
                k,
                i,
                str(segmentation[i, k]),
                ha="center",
                va="center",
                color=color,
                fontsize=fontsize,
            )
    plt.show()

    if cc.save:
        parent = Path(__file__).parent
        stem = Path(__file__).stem + "_diam_" + str(diameter)
        fn = parent.joinpath(stem + cc.ext)
        # plt.savefig(fn, dpi=DPI, bbox_inches='tight')
        # fig.savefig(fn, dpi=DPI)
        fig.savefig(fn)
        print(f"Saved {fn}")


if __name__ == "__main__":
    # User input begin
    cc = Configuration(
        diameter=100,
        save=False,
    )
    # User input end

    # Example usage
    mask = circle(diameter=cc.diameter)
    if cc.diameter < 20:
        print(mask)
    plot_segmentation(segmentation=mask, cc=cc)
