"""The module defines types for a tetrahedron and its vertices, and the
subcomponents the compose a tetrahedron."""

from dataclasses import dataclass
from typing import NamedTuple

import numpy as np


class Vertex(NamedTuple):
    """A vertex located in Cartesian coordinates."""

    x: float
    y: float
    z: float


@dataclass
class VertexSpherical:
    """A vertex located in spherical coordinates.

    r: The radius (distance from the origin to the point on the sphere surface.
    theta: The polar angle (angle from the positive z-axis) in degrees.
        theta should be in the range [0, 180] degrees, where
        0 is the north pole (positive z-axis),
        90 is the equator (x-y plane), and
        180 is the south pole (negative z-axis).
    phi: The azimuthal angle (angle in the x-y plane from the positive x-axis)
        in degrees.
        phi should be in the range [0, 360) degrees.
    """

    r: float  # Radial distance
    theta: float  # Polar angle in degrees
    phi: float  # Azimuthal angle in degrees

    def __post_init__(self):
        """Post-initialization processing."""
        # Validate inputs
        if self.r < 0.0:
            raise ValueError("Radial distance r must be non-negative.")
        if not 0 <= self.theta <= 180.0:
            raise ValueError("Polar angle theta must be in the range [0, 180].")
        if not 0 <= self.phi < 360.0:
            raise ValueError("Azimuthal angle phi must be in the range [0, 360).")


@dataclass
class Triangle:
    """A triangle in 3D defined by three non-colinear vertices."""

    a: Vertex
    b: Vertex
    c: Vertex

    def __post_init__(self):
        """Post-initialization processing."""

        self._ray_b = np.array(
            [self.b.x - self.a.x, self.b.y - self.a.y, self.b.z - self.a.z]
        )

        self._ray_c = np.array(
            [self.c.x - self.a.x, self.c.y - self.a.y, self.c.z - self.a.z]
        )

        _b_cross_c = np.cross(self._ray_b, self._ray_c)

        if np.allclose(_b_cross_c, [0, 0, 0]):
            raise ValueError("The three vertices must be non-colinear.")

    @property
    def normal(self):
        """Return the normal vector to the triangle face."""
        _b_cross_c = np.cross(self._ray_b, self._ray_c)

        return _b_cross_c / np.linalg.norm(_b_cross_c)


@dataclass
class Tetrahedron:
    """A tetrahedron defined by four non-coplanar vertices."""

    a: Vertex
    b: Vertex
    c: Vertex
    d: Vertex

    def faces(self):
        """Return the faces of the tetrahedron, following the right-hand rule,
        with the normal pointing outwards."""
        return [
            [self.a, self.c, self.b],
            [self.a, self.b, self.d],
            [self.a, self.d, self.c],
            [self.b, self.c, self.d],
        ]
