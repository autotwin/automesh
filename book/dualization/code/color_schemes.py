"""This module plots discrete grayscale, plasma and viridis color schemes."""

import matplotlib.pyplot as plt
from matplotlib import patches

from quadtree_colors import QuadColors, ColorScheme


def plot_color_schemes(n_colors: int, alpha: float):
    """Shows the different color schemes."""

    fig, axes = plt.subplots(2, 4, figsize=(10, 5))

    # Create sample rectangles to show the colors
    levels = range(n_colors)
    # n_levels = n_colors

    # Row 1: Original color schemes
    # Grayscale
    grayscale = QuadColors(
        n_levels=n_colors,
        edgecolor="black",
        alpha=alpha,
        color_scheme=ColorScheme.GRAYSCALE,
        reversed=False,
    )
    print("Grayscale colors:", grayscale.facecolors)
    ax0 = axes[0, 0]
    for i, level in enumerate(levels):
        rect = patches.Rectangle(
            (0, i),
            1,
            0.8,
            facecolor=grayscale.facecolors[level],
            edgecolor=grayscale.edgecolor,
            alpha=grayscale.alpha,
        )
        ax0.add_patch(rect)
        ax0.text(0.5, i + 0.4, f"Level {level}", ha="center", va="center")

    ax0.set_xlim(-0.1, 1.1)
    ax0.set_ylim(-0.1, n_colors)
    ax0.set_title("Grayscale")
    ax0.set_xticks([])
    ax0.set_yticks([])

    # Plasma
    plasma = QuadColors(
        n_levels=n_colors,
        edgecolor="black",
        alpha=alpha,
        color_scheme=ColorScheme.PLASMA,
        reversed=False,
    )
    print("Plasma colors:", plasma.facecolors)
    ax1 = axes[0, 1]
    for i, level in enumerate(levels):
        rect = patches.Rectangle(
            (0, i),
            1,
            0.8,
            facecolor=plasma.facecolors[level],
            edgecolor=plasma.edgecolor,
            alpha=plasma.alpha,
        )
        ax1.add_patch(rect)
        ax1.text(0.5, i + 0.4, f"Level {level}", ha="center", va="center")
    ax1.set_xlim(-0.1, 1.1)
    ax1.set_ylim(-0.1, n_colors)
    ax1.set_title("Plasma")
    ax1.set_xticks([])
    ax1.set_yticks([])

    # Tab10 (default matplotlib color scheme)
    tab10 = QuadColors(
        n_levels=n_colors,
        edgecolor="black",
        alpha=alpha,
        color_scheme=ColorScheme.TAB10,
        reversed=False,
    )
    print("Tab10 colors:", tab10.facecolors)
    ax2 = axes[0, 2]
    for i, level in enumerate(levels):
        rect = patches.Rectangle(
            (0, i),
            1,
            0.8,
            facecolor=tab10.facecolors[level],
            edgecolor=tab10.edgecolor,
            alpha=tab10.alpha,
        )
        ax2.add_patch(rect)
        ax2.text(0.5, i + 0.4, f"Level {level}", ha="center", va="center")
    ax2.set_xlim(-0.1, 1.1)
    ax2.set_ylim(-0.1, n_colors)
    ax2.set_title("Tab10")
    ax2.set_xticks([])
    ax2.set_yticks([])

    # Viridis
    viridis = QuadColors(
        n_levels=n_colors,
        edgecolor="black",
        alpha=alpha,
        color_scheme=ColorScheme.VIRIDIS,
        reversed=False,
    )
    print("Viridis colors:", viridis.facecolors)
    ax3 = axes[0, 3]
    for i, level in enumerate(levels):
        rect = patches.Rectangle(
            (0, i),
            1,
            0.8,
            facecolor=viridis.facecolors[level],
            edgecolor=viridis.edgecolor,
            alpha=viridis.alpha,
        )
        ax3.add_patch(rect)
        ax3.text(0.5, i + 0.4, f"Level {level}", ha="center", va="center")
    ax3.set_xlim(-0.1, 1.1)
    ax3.set_ylim(-0.1, n_colors)
    ax3.set_title("Viridis")
    ax3.set_xticks([])
    ax3.set_yticks([])

    # Row 2: Reversed color schemes
    # Grayscale reversed
    grayscale_reversed = QuadColors(
        n_levels=n_colors,
        edgecolor="black",
        alpha=alpha,
        color_scheme=ColorScheme.GRAYSCALE,
        reversed=True,
    )
    print("Grayscale colors reversed:", grayscale_reversed.facecolors)
    ax4 = axes[1, 0]
    for i, level in enumerate(levels):
        rect = patches.Rectangle(
            (0, i),
            1,
            0.8,
            facecolor=grayscale_reversed.facecolors[level],
            edgecolor=grayscale_reversed.edgecolor,
            alpha=grayscale_reversed.alpha,
        )
        ax4.add_patch(rect)
        ax4.text(0.5, i + 0.4, f"Level {level}", ha="center", va="center")
    ax4.set_xlim(-0.1, 1.1)
    ax4.set_ylim(-0.1, n_colors)
    ax4.set_title("Grayscale Reversed")
    ax4.set_xticks([])
    ax4.set_yticks([])

    # Plasma reversed
    plasma_reversed = QuadColors(
        n_levels=n_colors,
        edgecolor="black",
        alpha=alpha,
        color_scheme=ColorScheme.PLASMA,
        reversed=True,
    )
    print("Plasma colors reversed:", plasma_reversed.facecolors)
    ax5 = axes[1, 1]
    for i, level in enumerate(levels):
        rect = patches.Rectangle(
            (0, i),
            1,
            0.8,
            facecolor=plasma_reversed.facecolors[level],
            edgecolor=plasma_reversed.edgecolor,
            alpha=plasma_reversed.alpha,
        )
        ax5.add_patch(rect)
        ax5.text(0.5, i + 0.4, f"Level {level}", ha="center", va="center")
    ax5.set_xlim(-0.1, 1.1)
    ax5.set_ylim(-0.1, n_colors)
    ax5.set_title("Plasma Reversed")
    ax5.set_xticks([])
    ax5.set_yticks([])

    # Tab10 reversed
    tab10_reversed = QuadColors(
        n_levels=n_colors,
        edgecolor="black",
        alpha=alpha,
        color_scheme=ColorScheme.TAB10,
        reversed=True,
    )
    print("Tab10 colors reversed:", tab10_reversed.facecolors)
    ax6 = axes[1, 2]
    for i, level in enumerate(levels):
        rect = patches.Rectangle(
            (0, i),
            1,
            0.8,
            facecolor=tab10_reversed.facecolors[level],
            edgecolor=tab10_reversed.edgecolor,
            alpha=tab10_reversed.alpha,
        )
        ax6.add_patch(rect)
        ax6.text(0.5, i + 0.4, f"Level {level}", ha="center", va="center")
    ax6.set_xlim(-0.1, 1.1)
    ax6.set_ylim(-0.1, n_colors)
    ax6.set_title("Tab10 Reversed")
    ax6.set_xticks([])
    ax6.set_yticks([])

    # Viridis reversed
    viridis_reversed = QuadColors(
        n_levels=n_colors,
        edgecolor="black",
        alpha=alpha,
        color_scheme=ColorScheme.VIRIDIS,
        reversed=True,
    )
    print("Viridis colors reversed:", viridis_reversed.facecolors)
    ax7 = axes[1, 3]
    for i, level in enumerate(levels):
        rect = patches.Rectangle(
            (0, i),
            1,
            0.8,
            facecolor=viridis_reversed.facecolors[level],
            edgecolor=viridis_reversed.edgecolor,
            alpha=viridis_reversed.alpha,
        )
        ax7.add_patch(rect)
        ax7.text(0.5, i + 0.4, f"Level {level}", ha="center", va="center")
    ax7.set_xlim(-0.1, 1.1)
    ax7.set_ylim(-0.1, n_colors)
    ax7.set_title("Viridis Reversed")
    ax7.set_xticks([])
    ax7.set_yticks([])

    plt.tight_layout()
    plt.show()


# Demonstrate the color schemes
if __name__ == "__main__":
    n_colors = 5  # Number of discrete colors to extract
    alpha = 0.5  # Opacity for the rectangles
    plot_color_schemes(n_colors=n_colors, alpha=alpha)

    # Show the extracted colors
    # print("Plasma colors:", PLASMA_COLORS)
    # print("Plasma_r colors:", PLASMA_R_COLORS)
