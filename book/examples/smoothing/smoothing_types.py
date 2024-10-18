"""This module defines types used for smoothing hexahedral meshes."""

from enum import Enum
from typing import NamedTuple


class Vertex(NamedTuple):
    """A general 3D vertex with x, y, and z coordinates."""

    x: float
    y: float
    z: float


Vertices = tuple[Vertex, ...]
Hex = tuple[int, int, int, int, int, int, int, int]  # only hex elements
Hexes = tuple[Hex, ...]
# Dof = tuple[int, int, int]
# DofSet = tuple[Dof, ...]  # analong to a SideSet or a NodeSet
Neighbor = tuple[int, ...]
Neighbors = tuple[Neighbor, ...]
NodeHierarchy = tuple[int, ...]


# class DofType(Enum):
#     """All degrees of freedom must belong to one, and only one, of the
#     following smoothing categories.
#     """
# 
#     PRESCRIBED_HOMOGENEOUS = 0
#     PRESCRIBED_INHOMOGENEOUS = 1
#     FREE_EXTERIOR = 2
#     FREE_INTERFACE = 3
#     FREE_INTERIOR = 4


class Hierarchy(Enum):
    """All nodes must be categorized as beloning to one, and only one,
    of the following hierarchical categories.
    """

    INTERIOR = 0
    BOUNDARY = 1
    PRESCRIBED = 2


class SmoothingAlgorithm(Enum):
    """The type of smoothing algorithm."""

    LAPLACE = "Laplace"
    TAUBIN = "Taubin"
