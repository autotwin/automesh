"""This module tests the tet_to_hex module.

To run the tests, use the command: pytest <filename>.py
"""

import pytest
import numpy as np

from tet_to_hex_types import (
    Triangle,
    Vertex,
    VertexSpherical,
)
from tet_to_hex import spherical_to_cartesian


# Constants
DEG_TO_RAD: float = np.pi / 180.0


def test_conversion_45_60():
    """Test conversion for a known case."""
    spherical_coords = VertexSpherical(r=1.0, theta=45.0, phi=60.0)
    cartesian_coords = spherical_to_cartesian(spherical_coords)

    pi_by_4 = np.pi / 4.0
    pi_by_3 = np.pi / 3.0

    expected_x = 1.0 * np.sin(pi_by_4) * np.cos(pi_by_3)
    expected_y = 1.0 * np.sin(pi_by_4) * np.sin(pi_by_3)
    expected_z = 1.0 * np.cos(pi_by_4)

    assert np.isclose(cartesian_coords.x, expected_x)
    assert np.isclose(cartesian_coords.y, expected_y)
    assert np.isclose(cartesian_coords.z, expected_z)


def test_zero_radius():
    """Test conversion with zero radius."""
    spherical_coords = VertexSpherical(r=0.0, theta=90.0, phi=0.0)
    cartesian_coords = spherical_to_cartesian(spherical_coords)
    assert cartesian_coords == Vertex(0.0, 0.0, 0.0)


def test_negative_radius():
    """Test conversion with negative radius."""
    with pytest.raises(ValueError, match="Radial distance r must be non-negative."):
        spherical_to_cartesian(VertexSpherical(r=-1.0, theta=45.0, phi=60.0))


def test_theta_out_of_bounds():
    """Test conversion with theta out of bounds."""
    with pytest.raises(
        ValueError, match=r"Polar angle theta must be in the range \[0, 180\]."
    ):
        spherical_to_cartesian(
            VertexSpherical(r=1.0, theta=190.0, phi=60.0)
        )  # theta > 180

    with pytest.raises(
        ValueError, match=r"Polar angle theta must be in the range \[0, 180\]."
    ):
        spherical_to_cartesian(
            VertexSpherical(r=1.0, theta=-10.0, phi=60.0)
        )  # theta < 0


def test_phi_out_of_bounds():
    """Test conversion with phi out of bounds."""
    with pytest.raises(
        ValueError,
        match=r"Azimuthal angle phi must be in the range \[0, 360\).",
    ):
        spherical_to_cartesian(
            VertexSpherical(r=1.0, theta=90.0, phi=360.0)
        )  # phi >= 360

    with pytest.raises(
        ValueError,
        match=r"Azimuthal angle phi must be in the range \[0, 360\).",
    ):
        spherical_to_cartesian(VertexSpherical(r=1.0, theta=90.0, phi=-10.0))  # phi < 0


def test_edge_case_theta_0():
    """Test conversion with theta = 0."""
    spherical_coords = VertexSpherical(r=1.0, theta=0.0, phi=0.0)
    cartesian_coords = spherical_to_cartesian(spherical_coords)
    assert cartesian_coords == Vertex(0.0, 0.0, 1.0)  # Should point straight up


def test_edge_case_theta_180():
    """Test conversion with theta = 180."""
    spherical_coords = VertexSpherical(r=1.0, theta=180.0, phi=0.0)
    cartesian_coords = spherical_to_cartesian(spherical_coords)

    # Check if x is close to 0, and y and z are exactly as expected
    assert np.isclose(
        cartesian_coords.x, 0.0, atol=1e-10
    )  # Allow for a small tolerance
    assert cartesian_coords.y == 0.0
    assert cartesian_coords.z == -1.0  # Should point straight down


def test_triangle_normal():
    """Test the normal calculation for a simple triangle."""
    va = Vertex(0.0, 0.0, 0.0)
    vb = Vertex(1.0, 0.0, 0.0)
    vc = Vertex(0.0, 1.0, 0.0)

    triangle = Triangle(va, vb, vc)
    normal = triangle.normal

    # The expected normal for this triangle is (0, 0, 1)
    expected_normal = np.array([0.0, 0.0, 1.0])

    assert np.allclose(normal, expected_normal), "Normal vector is incorrect."


def test_triangle_collinear():
    """Test that a ValueError is raised for collinear points."""
    va = Vertex(0.0, 0.0, 0.0)
    vb = Vertex(1.0, 1.0, 0.0)
    vc = Vertex(2.0, 2.0, 0.0)  # Collinear to vb

    with pytest.raises(ValueError, match="The three vertices must be non-colinear."):
        Triangle(va, vb, vc)


def test_triangle_normal_negative_coordinates():
    """Test the normal calculation for a triangle with negative coordinates."""
    va = Vertex(-1.0, -1.0, 0.0)
    vb = Vertex(0.0, -1.0, 0.0)
    vc = Vertex(-1.0, 0.0, 0.0)

    triangle = Triangle(va, vb, vc)
    normal = triangle.normal

    # The expected normal for this triangle is (0, 0, 1)
    expected_normal = np.array([0.0, 0.0, 1.0])

    assert np.allclose(normal, expected_normal), (
        "Normal vector is incorrect for negative coordinates."
    )


def test_triangle_normal_non_unit_normal():
    """Test that the normal is normalized correctly."""
    va = Vertex(0.0, 0.0, 0.0)
    vb = Vertex(2.0, 0.0, 0.0)
    vc = Vertex(0.0, 2.0, 0.0)

    triangle = Triangle(va, vb, vc)
    normal = triangle.normal

    # The expected normal for this triangle is (0, 0, 1)
    expected_normal = np.array([0.0, 0.0, 1.0])

    assert np.allclose(normal, expected_normal), (
        "Normal vector is not normalized correctly."
    )
