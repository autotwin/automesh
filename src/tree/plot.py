import matplotlib.pyplot as plt
import matplotlib.patches as patches


# Function to draw a rectangle
def draw_cell(ax, x, y, width, height):
    """Draw a cell as a patch."""
    ax.add_patch(
        patches.Rectangle(
            (x, y),
            width,
            height,
            edgecolor="red",
            facecolor="green",
            alpha=0.5,
            fill=False,
        )
    )


# Example usage
def main():
    """Creates and plots cells."""

    # Create a new figure and axis
    fig, ax = plt.subplots()

    # # Set limits for the plot
    # ax.set_xlim(0, 800)
    # ax.set_ylim(0, 800)

    # # Draw some rectangles (representing quadtree nodes)
    # draw_cell(ax, 100, 100, 200, 200)  # Rectangle 1
    # draw_cell(ax, 300, 300, 150, 150)  # Rectangle 2
    # draw_cell(ax, 500, 100, 100, 300)  # Rectangle 3

    ax.set_xlim(0.9, 3.1)
    ax.set_ylim(-1.1, 1.1)

    # ax.add_patch(patches.Rectangle((1, -1), 2, 2, edgecolor='blue', facecolor='blue', alpha=0.5, fill=False))
    # ax.add_patch(patches.Rectangle((1, -1), 1, 1, edgecolor='blue', facecolor='blue', alpha=0.5, fill=False))
    # ax.add_patch(patches.Rectangle((2, -1), 1, 1, edgecolor='blue', facecolor='blue', alpha=0.5, fill=False))
    # ax.add_patch(patches.Rectangle((1, 0), 1, 1, edgecolor='blue', facecolor='blue', alpha=0.5, fill=False))
    # ax.add_patch(patches.Rectangle((2, 0), 1, 1, edgecolor='blue', facecolor='blue', alpha=0.5, fill=False)/

    ax.add_patch(patches.Rectangle((1, -1), 2, 2, edgecolor='blue', facecolor='blue', alpha=0.5, fill=False))
    ax.add_patch(patches.Rectangle((1, -1), 1, 1, edgecolor='blue', facecolor='blue', alpha=0.5, fill=False))
    ax.add_patch(patches.Rectangle((2, -1), 1, 1, edgecolor='blue', facecolor='blue', alpha=0.5, fill=False))
    ax.add_patch(patches.Rectangle((1, 0), 1, 1, edgecolor='blue', facecolor='blue', alpha=0.5, fill=False))
    ax.add_patch(patches.Rectangle((2, 0), 1, 1, edgecolor='blue', facecolor='blue', alpha=0.5, fill=False))
    ax.add_patch(patches.Rectangle((2, 0), 0.5, 0.5, edgecolor='blue', facecolor='blue', alpha=0.5, fill=False))
    ax.add_patch(patches.Rectangle((2.5, 0), 0.5, 0.5, edgecolor='blue', facecolor='blue', alpha=0.5, fill=False))
    ax.add_patch(patches.Rectangle((2, 0.5), 0.5, 0.5, edgecolor='blue', facecolor='blue', alpha=0.5, fill=False))
    ax.add_patch(patches.Rectangle((2.5, 0.5), 0.5, 0.5, edgecolor='blue', facecolor='blue', alpha=0.5, fill=False))


    # Set aspect of the plot to be equal
    ax.set_aspect("equal", adjustable="box")

    # Show the plot
    plt.title("Quadtree Visualization")
    plt.xlabel("x-axis")
    plt.ylabel("y-axis")
    # plt.grid()
    plt.show()


if __name__ == "__main__":
    main()
