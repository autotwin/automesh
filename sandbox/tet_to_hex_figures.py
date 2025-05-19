"""This module visualizes a tetrahedon."""

import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d.art3d import Poly3DCollection


from tet_to_hex_types import (
    Tetrahedron,
    VertexSpherical,
)

from tet_to_hex import spherical_to_cartesian


def plot_tetrahedron(tet: Tetrahedron):
    """
    Plot a tetrahedron given its vertices.

    """

    # vertices = np.array([tet.va, tet.vb, tet.vc, tet.vd])

    # A, B, C, D = vertices

    # Define the faces of the tetrahedron, right-hand rule,
    # with normal pointing outwards
    # faces = [[A, C, B], [A, B, D], [A, D, C], [B, C, D]]
    faces = tet.faces()

    # Create a 3D plot
    fig = plt.figure()
    ax = fig.add_subplot(111, projection="3d")

    # Create a Poly3DCollection for the faces
    tetrahedron = Poly3DCollection(faces, alpha=0.5, linewidths=1, edgecolors="r")
    ax.add_collection3d(tetrahedron)

    # Set limits and labels
    ax.set_xlabel("x")
    ax.set_ylabel("y")
    ax.set_zlabel("z")
    ax.set_title("Tetrahedron Visualization")

    # Set the limits of the axes
    # ax.set_xlim([0, 1])
    # ax.set_ylim([0, 1])
    # ax.set_zlim([0, 1])
    # Set the limits of the axes based on the vertices
    xs = [foo.x for foo in [tet.a, tet.b, tet.c, tet.d]]
    ys = [foo.y for foo in [tet.a, tet.b, tet.c, tet.d]]
    zs = [foo.z for foo in [tet.a, tet.b, tet.c, tet.d]]

    ax.set_xlim([min(xs), max(xs)])
    ax.set_ylim([min(ys), max(ys)])
    ax.set_zlim([min(zs), max(zs)])
    # ax.set_ylim([min(vertices[:, 1]), max(vertices[:, 1])])
    # ax.set_zlim([min(vertices[:, 2]), max(vertices[:, 2])])

    # Set equal aspect ratio
    ax.set_box_aspect([1, 1, 1])  # Aspect ratio is 1:1:1

    # Label the vertices A, B, C, D
    labels = ["a", "b", "c", "d"]
    for label, vertex in zip(labels, [tet.a, tet.b, tet.c, tet.d]):
        ax.text(
            vertex.x,
            vertex.y,
            vertex.z,
            f"{label} ({vertex.x:.2f}, {vertex.y:.2f}, {vertex.z:.2f})",
            size=10,
            zorder=1,
            color="k",
        )

    # Show the plot
    plt.show()


def main():
    # Define the vertices of the tetrahedron
    va = spherical_to_cartesian(VertexSpherical(r=0.0, theta=0.0, phi=0.0))
    vb = spherical_to_cartesian(VertexSpherical(r=1.0, theta=90.0, phi=0.0))
    vc = spherical_to_cartesian(VertexSpherical(r=1.0, theta=90.0, phi=90.0))
    vd = spherical_to_cartesian(VertexSpherical(r=1.0, theta=0.0, phi=0.0))

    # Create the tetrahedron
    tet = Tetrahedron(va, vb, vc, vd)
    # Plot the tetrahedron
    plot_tetrahedron(tet=tet)


if __name__ == "__main__":
    main()
