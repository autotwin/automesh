r"""This module, smoothing_examples.py contains data for the
smoothing examples.
"""

import math
from typing import Final

import smoothing_types as ty

# Type alias for functional style methods
# https://docs.python.org/3/library/typing.html#type-aliases
Hierarchy = ty.Hierarchy
SmoothingAlgorithm = ty.SmoothingAlgorithm
Example = ty.SmoothingExample
Vertex = ty.Vertex

DEG2RAD: Final[float] = math.pi / 180.0  # rad/deg

# L-bracket example
bracket = Example(
    vertices=(
        Vertex(0, 0, 0),
        Vertex(1, 0, 0),
        Vertex(2, 0, 0),
        Vertex(3, 0, 0),
        Vertex(4, 0, 0),
        Vertex(0, 1, 0),
        Vertex(1, 1, 0),
        Vertex(2, 1, 0),
        Vertex(3, 1, 0),
        Vertex(4, 1, 0),
        Vertex(0, 2, 0),
        Vertex(1, 2, 0),
        Vertex(2, 2, 0),
        Vertex(3, 2, 0),
        Vertex(4, 2, 0),
        Vertex(0, 3, 0),
        Vertex(1, 3, 0),
        Vertex(2, 3, 0),
        Vertex(0, 4, 0),
        Vertex(1, 4, 0),
        Vertex(2, 4, 0),
        Vertex(0, 0, 1),
        Vertex(1, 0, 1),
        Vertex(2, 0, 1),
        Vertex(3, 0, 1),
        Vertex(4, 0, 1),
        Vertex(0, 1, 1),
        Vertex(1, 1, 1),
        Vertex(2, 1, 1),
        Vertex(3, 1, 1),
        Vertex(4, 1, 1),
        Vertex(0, 2, 1),
        Vertex(1, 2, 1),
        Vertex(2, 2, 1),
        Vertex(3, 2, 1),
        Vertex(4, 2, 1),
        Vertex(0, 3, 1),
        Vertex(1, 3, 1),
        Vertex(2, 3, 1),
        Vertex(0, 4, 1),
        Vertex(1, 4, 1),
        Vertex(2, 4, 1),
    ),
    elements=(
        (1, 2, 7, 6, 22, 23, 28, 27),
        (2, 3, 8, 7, 23, 24, 29, 28),
        (3, 4, 9, 8, 24, 25, 30, 29),
        (4, 5, 10, 9, 25, 26, 31, 30),
        (6, 7, 12, 11, 27, 28, 33, 32),
        (7, 8, 13, 12, 28, 29, 34, 33),
        (8, 9, 14, 13, 29, 30, 35, 34),
        (9, 10, 15, 14, 30, 31, 36, 35),
        (11, 12, 17, 16, 32, 33, 38, 37),
        (12, 13, 18, 17, 33, 34, 39, 38),
        (16, 17, 20, 19, 37, 38, 41, 40),
        (17, 18, 21, 20, 38, 39, 42, 41),
    ),
    nelx=4,
    nely=4,
    nelz=1,
    # neighbors=(
    #     (2, 6, 22),
    #     (1, 3, 7, 23),
    #     (2, 4, 8, 24),
    #     (3, 5, 9, 25),
    #     (4, 10, 26),
    #     #
    #     (1, 7, 11, 27),
    #     (2, 6, 8, 12, 28),
    #     (3, 7, 9, 13, 29),
    #     (4, 8, 10, 14, 30),
    #     (5, 9, 15, 31),
    #     #
    #     (6, 12, 16, 32),
    #     (7, 11, 13, 17, 33),
    #     (8, 12, 14, 18, 34),
    #     (9, 13, 15, 35),
    #     (10, 14, 36),
    #     #
    #     (11, 17, 19, 37),
    #     (12, 16, 18, 20, 38),
    #     (13, 17, 21, 39),
    #     #
    #     (16, 20, 40),
    #     (17, 19, 21, 41),
    #     (18, 20, 42),
    #     # top layer
    #     (1, 23, 27),
    #     (2, 22, 24, 28),
    #     (3, 23, 25, 29),
    #     (4, 24, 26, 30),
    #     (5, 25, 31),
    #     #
    #     (6, 22, 28, 32),
    #     (7, 23, 27, 29, 33),
    #     (8, 24, 28, 30, 34),
    #     (9, 25, 29, 31, 35),
    #     (10, 26, 30, 36),
    #     #
    #     (11, 27, 33, 37),
    #     (12, 28, 32, 34, 38),
    #     (13, 29, 33, 35, 39),
    #     (14, 30, 34, 36),
    #     (15, 31, 35),
    #     #
    #     (16, 32, 38, 40),
    #     (17, 33, 37, 39, 41),
    #     (18, 34, 38, 42),
    #     #
    #     (19, 37, 41),
    #     (20, 38, 40, 42),
    #     (21, 39, 41),
    # ),
    node_hierarchy=(
        # hierarchy enum, node number, prescribed (x, y, z)
        Hierarchy.PRESCRIBED,  # 1 -> (0, 0, 0)
        Hierarchy.PRESCRIBED,  # 2 -> (1, 0, 0)
        Hierarchy.PRESCRIBED,  # 3 -> (2, 0, 0)
        Hierarchy.PRESCRIBED,  # 4 -> (3, 0, 0)
        Hierarchy.PRESCRIBED,  # 5 -> (4, 0, 0)
        Hierarchy.PRESCRIBED,  # 6 -> (0, 1, 0)
        Hierarchy.BOUNDARY,  # 7
        Hierarchy.BOUNDARY,  # 8
        Hierarchy.BOUNDARY,  # 9
        Hierarchy.PRESCRIBED,  # 10 -> (4.5*cos(15 deg), 4.5*sin(15 deg), 0)
        Hierarchy.PRESCRIBED,  # 11 -> *(0, 2, 0)
        Hierarchy.BOUNDARY,  # 12
        Hierarchy.BOUNDARY,  # 13
        Hierarchy.BOUNDARY,  # 14
        Hierarchy.PRESCRIBED,  # 15 -> (4.5*cos(30 deg), 4.5*sin(30 deg), 0)
        Hierarchy.PRESCRIBED,  # 16 -> (0, 3, 0)
        Hierarchy.BOUNDARY,  # 17
        Hierarchy.BOUNDARY,  # 18
        Hierarchy.PRESCRIBED,  # 19 -> (0, 4, 0)
        Hierarchy.PRESCRIBED,  # 20 -> (1.5, 4, 0)
        Hierarchy.PRESCRIBED,  # 21 -> (3.5, 4, 0)
        #
        Hierarchy.PRESCRIBED,  # 22 -> (0, 0, 1)
        Hierarchy.PRESCRIBED,  # 23 -> (1, 0, 1)
        Hierarchy.PRESCRIBED,  # 24 -> (2, 0, 1)
        Hierarchy.PRESCRIBED,  # 25 -> (3, 0, 1)
        Hierarchy.PRESCRIBED,  # 26 -> (4, 0, 1)
        Hierarchy.PRESCRIBED,  # 27 -> (0, 1, 1)
        Hierarchy.BOUNDARY,  # 28
        Hierarchy.BOUNDARY,  # 29
        Hierarchy.BOUNDARY,  # 30
        Hierarchy.PRESCRIBED,  # 31 -> (4.5*cos(15 deg), 4.5*sin(15 deg), 1)
        Hierarchy.PRESCRIBED,  # 32 -> *(0, 2, 1)
        Hierarchy.BOUNDARY,  # 33
        Hierarchy.BOUNDARY,  # 34
        Hierarchy.BOUNDARY,  # 35
        Hierarchy.PRESCRIBED,  # 36 -> (4.5*cos(30 deg), 4.5*sin(30 deg), 1)
        Hierarchy.PRESCRIBED,  # 37 -> (0, 3, 1)
        Hierarchy.BOUNDARY,  # 38
        Hierarchy.BOUNDARY,  # 39
        Hierarchy.PRESCRIBED,  # 40 -> (0, 4, 1)
        Hierarchy.PRESCRIBED,  # 41 -> (1.5, 4, 1)
        Hierarchy.PRESCRIBED,  # 42 -> (3.5, 4, 1)
    ),
    prescribed_nodes=(
        (1, Vertex(0, 0, 0)),
        (2, Vertex(1, 0, 0)),
        (3, Vertex(2, 0, 0)),
        (4, Vertex(3, 0, 0)),
        (5, Vertex(4, 0, 0)),
        (6, Vertex(0, 1, 0)),
        (
            10,
            Vertex(4.5 * math.cos(15 * DEG2RAD), 4.5 * math.sin(15 * DEG2RAD), 0),
        ),
        (11, Vertex(0, 2, 0)),
        (
            15,
            Vertex(4.5 * math.cos(30 * DEG2RAD), 4.5 * math.sin(30 * DEG2RAD), 0),
        ),
        (16, Vertex(0, 3, 0)),
        (19, Vertex(0, 4, 0)),
        (20, Vertex(1.5, 4, 0)),
        (21, Vertex(3.5, 4, 0)),
        (22, Vertex(0, 0, 1)),
        (23, Vertex(1, 0, 1)),
        (24, Vertex(2, 0, 1)),
        (25, Vertex(3, 0, 1)),
        (26, Vertex(4, 0, 1)),
        (27, Vertex(0, 1, 1)),
        (
            31,
            Vertex(4.5 * math.cos(15 * DEG2RAD), 4.5 * math.sin(15 * DEG2RAD), 1),
        ),
        (32, Vertex(0, 2, 1)),
        (
            36,
            Vertex(4.5 * math.cos(30 * DEG2RAD), 4.5 * math.sin(30 * DEG2RAD), 1),
        ),
        (37, Vertex(0, 3, 1)),
        (40, Vertex(0, 4, 1)),
        (41, Vertex(1.5, 4, 1)),
        (42, Vertex(3.5, 4, 1)),
    ),
    scale_lambda=0.3,
    scale_mu=-0.33,
    num_iters=10,
    algorithm=SmoothingAlgorithm.LAPLACE,
    file_stem="bracket",
)

# Double X two-element example
double_x = Example(
    vertices=(
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
    ),
    elements=(
        (1, 2, 5, 4, 7, 8, 11, 10),
        (2, 3, 6, 5, 8, 9, 12, 11),
    ),
    nelx=2,
    nely=1,
    nelz=1,
    # neighbors=(
    #     (2, 4, 7),
    #     (1, 3, 5, 8),
    #     (2, 6, 9),
    #     (1, 5, 10),
    #     (2, 4, 6, 11),
    #     (3, 5, 12),
    #     (1, 8, 10),
    #     (2, 7, 9, 11),
    #     (3, 8, 12),
    #     (4, 7, 11),
    #     (5, 8, 10, 12),
    #     (6, 9, 11),
    # ),
    node_hierarchy=(
        Hierarchy.BOUNDARY,
        Hierarchy.BOUNDARY,
        Hierarchy.BOUNDARY,
        Hierarchy.BOUNDARY,
        Hierarchy.BOUNDARY,
        Hierarchy.BOUNDARY,
        Hierarchy.BOUNDARY,
        Hierarchy.BOUNDARY,
        Hierarchy.BOUNDARY,
        Hierarchy.BOUNDARY,
        Hierarchy.BOUNDARY,
        Hierarchy.BOUNDARY,
    ),
    prescribed_nodes=None,
    scale_lambda=0.3,
    scale_mu=-0.33,
    num_iters=2,
    algorithm=SmoothingAlgorithm.LAPLACE,
    file_stem="double_x",
)
