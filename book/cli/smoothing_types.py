r"""This module, smoothing_types.py, defines types used for smoothing
hexahedral meshes.
"""

from enum import Enum
from typing import NamedTuple


class Vertex(NamedTuple):
    """A general 3D vertex with x, y, and z coordinates."""

    x: float
    y: float
    z: float


class Hierarchy(Enum):
    """All nodes must be categorized as beloning to one, and only one,
    of the following hierarchical categories.
    """

    INTERIOR = 0
    BOUNDARY = 1
    PRESCRIBED = 2


Vertices = tuple[Vertex, ...]
Hex = tuple[int, int, int, int, int, int, int, int]  # only hex elements
Hexes = tuple[Hex, ...]
Neighbor = tuple[int, ...]
Neighbors = tuple[Neighbor, ...]
NodeHierarchy = tuple[Hierarchy, ...]
PrescribedNodes = tuple[tuple[int, Vertex], ...] | None


class SmoothingAlgorithm(Enum):
    """The type of smoothing algorithm."""

    LAPLACE = "Laplace"
    TAUBIN = "Taubin"


class SmoothingExample(NamedTuple):
    """The prototype smoothing example."""

    vertices: Vertices
    elements: Hexes
    nelx: int
    nely: int
    nelz: int
    # neighbors: Neighbors
    node_hierarchy: NodeHierarchy
    prescribed_nodes: PrescribedNodes
    scale_lambda: float
    scale_mu: float
    num_iters: int
    algorithm: SmoothingAlgorithm
    file_stem: str
