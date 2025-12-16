"""This module demonstrates ficticious springs placed on nodes to allow nodes
to consolidate.

    nnp = 20 (number of nodal points)
    top of the element number is shown in parenthesis
    zero-index
    nel = 4 (number of elements)

   z=1                          z=2                           z=3
    + --- > y                    + ---- > y                    + ---- > y
    |                            |                             |
    |     0-------3-------6      |      8------11------14      |     16------18
    |     |       |       |      |      |       |       |      |      |       |
    v     |       |       |      v      |  (0)  |  (2)  |      v      |  (3)  |
    x     1-------4-------7      x      9------12------15      x     17------19
          |       |                     |       |
          |       |                     |  (1)  |
          2-------5                    10------13
"""

from typing import Final, NamedTuple

import numpy as np


SPRING: Final[float] = 100.0  # N/m


class Position(NamedTuple):
    """A position in R3."""

    x: float  # m
    y: float  # m
    z: float  # m


class Mesh(NamedTuple):
    """A collection of coordinates and elements (connectivity)."""

    coordinates: list[Position]
    elements: list[tuple[int, int, int, int, int, int, int, int]]


def create_mesh() -> Mesh:
    """Creates a four-element start mesh."""
    elements = [
        (1, 4, 3, 0, 9, 12, 11, 8),  # element 0
        (2, 5, 4, 1, 10, 13, 12, 9),  # element 1
        (4, 7, 6, 3, 12, 15, 14, 11),  # element 2
        (9, 12, 11, 8, 17, 19, 18, 16),  # element 3
    ]

    coordinates = [
        # layer z=1
        Position(x=1, y=1, z=1),  # node 0
        Position(x=2, y=1, z=1),  # node 1
        Position(x=3, y=1, z=1),  # node 2
        Position(x=1, y=2, z=1),  # node 3
        Position(x=2, y=2, z=1),  # node 4
        Position(x=3, y=2, z=1),  # node 5
        Position(x=1, y=3, z=1),  # node 6
        Position(x=2, y=3, z=1),  # node 7
        # layer z=2
        Position(x=1, y=1, z=2),  # node 8
        Position(x=2, y=1, z=2),  # node 9
        Position(x=3, y=1, z=2),  # node 10
        Position(x=1, y=2, z=2),  # node 11
        Position(x=2, y=2, z=2),  # node 12
        Position(x=3, y=2, z=2),  # node 13
        Position(x=1, y=3, z=2),  # node 14
        Position(x=2, y=3, z=2),  # node 15
        # layer z=3
        Position(x=1, y=1, z=3),  # node 16
        Position(x=2, y=1, z=3),  # node 17
        Position(x=1, y=2, z=3),  # node 18
        Position(x=2, y=2, z=3),  # node 19
    ]

    result = Mesh(coordinates=coordinates, elements=elements)

    return result


def distance(p1: Position, p2: Position) -> float:
    """Calculates the distance between two points."""
    dd = distance_squared(p1=p1, p2=p2)
    result = np.sqrt(dd)
    return result


def distance_squared(p1: Position, p2: Position) -> float:
    """Returns the squared distance between two points."""
    a = p2.x - p1.x
    b = p2.y - p1.y
    c = p2.z - p1.z
    result = a**2 + b**2 + c**2
    return result


def gap_energy(
    pairs: list[tuple[int, int]], mesh: Mesh, spring: float = SPRING
) -> float:
    """Given a list of node pairs, returns the total internal energy assocated
    with the current nodal positions."""
    # result = 0.0
    # for n1, n2 in pairs:
    #     breakpoint()
    #     ee_i = spring_ie_nodes(n1, n2, mesh, spring)
    #     result += ee_i

    result = sum(spring_ie_nodes(n1, n2, mesh, spring) for n1, n2 in pairs)
    return result


def spring_ie_nodes(
    node1: int, node2: int, mesh: Mesh, spring: float = SPRING
) -> float:
    """Returns the linear spring internal energy."""
    # Given the node number, get the node's position
    p1: Position = mesh.coordinates[node1]
    p2: Position = mesh.coordinates[node2]

    result = spring_ie(p1=p1, p2=p2, spring=spring)
    return result


def spring_ie(p1: Position, p2: Position, spring: float = SPRING) -> float:
    """Retuns the linear spring internal energy."""
    result = 0.5 * spring * distance_squared(p1=p1, p2=p2)
    return result


def main():
    """The main working cycle of the application."""
    print("Hello world!")


if __name__ == "__main__":
    main()

    aa = Position(1, 2, 3)
    bb = Position(4, 6, 8)
    DD = 50.0
    assert distance_squared(p1=aa, p2=bb) == DD
    assert distance(p1=aa, p2=bb) == np.sqrt(DD)
    rr = spring_ie(p1=aa, p2=bb)
    assert spring_ie(p1=aa, p2=bb) == 2500.0

    mm = create_mesh()

    # Define pairs
    ps = [(10, 17), (13, 19), (15, 19), (14, 18), (13, 15)]
    # Calculate gap energy

    ee = gap_energy(pairs=ps, mesh=mm, spring=SPRING)
    assert ee == 500.0

    print("Done.")
