"""This module illustrates test cases for smoothing algorithms.

Example:
--------
source ~/autotwin/automesh/.venv/bin/activate
python smooth.py

"""

from enum import Enum
from typing import NamedTuple
# from typing import Final, NamedTuple
# import numpy as np


class Vertex(NamedTuple):
    """A general 3D vertex with x, y, and z coordinates."""
    x: float
    y: float
    z: float


Vertices = tuple[Vertex, ...]
Element = tuple[int, int, int, int, int, int, int, int]
Elements = tuple[Element, ...]
Dof = tuple[int, int, int]
Dofs = tuple[Dof, ...]
Neighbor = tuple[int, ...]
Neighbors = tuple[Neighbor, ...]


class DofType(Enum):
    """All degrees of freedom must belong to one, and only one, of the
    following smoothing categories.
    """
    PRESCRIBED_HOMOGENEOUS = 0
    PRESCRIBED_INHOMOGENEOUS = 1
    FREE_EXTERIOR = 2
    FREE_INTERFACE = 3
    FREE_INTERIOR = 4


class SmoothingAlgorithm(Enum):
    """The type of smoothing algorithm."""
    LAPLACE = 0
    TAUBIN = 1


def smooth(
        vv: Vertices,
        nn: Neighbors,
        dd: Dofs,
        dt: DofType,
        lambda: float
    ):
    """Given an initial position of vertices, the vertex neighbors, and the
    dof classification of each vertex, perform Laplace smoothing, and return
    the updated coordinates."""






vertices: Vertices = (
    Vertex(0.0, 0.0, 0.0),
    Vertex(1.0, 0.0, 0.0),
    Vertex(2.0, 0.0, 0.0),
    Vertex(0.0, 1.0, 0.0),
    Vertex(1.0, 1.0, 0.0),
    Vertex(2.0, 1.0, 0.0),
    Vertex(0.0, 0.0, 1.0),
    Vertex(1.0, 0.0, 1.0),
    Vertex(2.0, 0.0, 1.0),
    Vertex(0.0, 1.0, 1.0),
    Vertex(1.0, 1.0, 1.0),
    Vertex(2.0, 1.0, 1.0),
)

dofs: Dofs = (
    (4, 4, 4),
    (4, 4, 4),
    (4, 4, 4),
    (4, 4, 4),
    (4, 4, 4),
    (4, 4, 4),
    (4, 4, 4),
    (4, 4, 4),
    (4, 4, 4),
    (4, 4, 4),
    (4, 4, 4),
    (4, 4, 4),
)

elements: Elements = (
    (1, 2, 5, 4, 7, 8, 11, 12),
    (2, 3, 6, 5, 8, 9, 12, 11),
)

neighbors: Neighbors = (
    (2, 4, 7),
    (1, 3, 5, 8),
    (2, 6, 9),
    (1, 5, 10),
    (2, 4, 6, 11),
    (3, 5, 12),
    (1, 8, 10),
    (2, 7, 9, 11),
    (3, 8, 12),
    (4, 7, 11),
    (5, 8, 10, 12),
    (6, 9, 11),
)
