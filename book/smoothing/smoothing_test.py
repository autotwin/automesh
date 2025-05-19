r"""This module, smoothing_test.py, tests the smoothing modules.

Example:
--------
source ~/autotwin/automesh/.venv/bin/activate
cd ~/autotwin/automesh/book/examples/smoothing
python -m pytest smoothing_test.py

Reference:
----------
DoubleX unit test
https://autotwin.github.io/automesh/examples/unit_tests/index.html#double-x
"""

from typing import Final

# import sandbox.smoothing as sm
# import sandbox.smoothing_types as ty
import smoothing as sm
import smoothing_examples as examples
import smoothing_types as ty

# Type alias for functional style methods
# https://docs.python.org/3/library/typing.html#type-aliases
Hexes = ty.Hexes
Hierarchy = ty.Hierarchy
Neighbors = ty.Neighbors
NodeHierarchy = ty.NodeHierarchy
Vertex = ty.Vertex
Vertices = ty.Vertices
SmoothingAlgorithm = ty.SmoothingAlgorithm


def test_average_position():
    """Unit test for average_position"""
    v1 = Vertex(x=1.0, y=2.0, z=3.0)
    v2 = Vertex(x=4.0, y=5.0, z=6.0)
    v3 = Vertex(x=7.0, y=8.0, z=9.0)

    v_ave = sm.average_position((v1, v2, v3))
    assert v_ave.x == 4.0
    assert v_ave.y == 5.0
    assert v_ave.z == 6.0

    # docstring example
    v1, v2 = Vertex(1, 2, 3), Vertex(4, 5, 6)
    assert sm.average_position((v1, v2)) == Vertex(2.5, 3.5, 4.5)


def test_add():
    """Unit test for the addition of Vertex v1 and Vertex v2."""
    v1 = Vertex(x=1.0, y=2.0, z=3.0)
    v2 = Vertex(x=4.0, y=7.0, z=1.0)
    vv = sm.add(v1=v1, v2=v2)
    assert vv.x == 5.0
    assert vv.y == 9.0
    assert vv.z == 4.0

    # docstring example
    v1, v2 = Vertex(1, 2, 3), Vertex(4, 5, 6)
    assert sm.add(v1, v2) == Vertex(5, 7, 9)


def test_subtract():
    """Unit test for the subtraction of Vertex v2 from Vertex v1."""
    v1 = Vertex(x=1.0, y=2.0, z=3.0)
    v2 = Vertex(x=4.0, y=7.0, z=1.0)
    vv = sm.subtract(v1=v1, v2=v2)
    assert vv.x == -3.0
    assert vv.y == -5.0
    assert vv.z == 2.0

    # docstring example
    v1, v2 = Vertex(8, 5, 2), Vertex(1, 2, 3)
    assert sm.subtract(v1, v2) == Vertex(7, 3, -1)


def test_scale():
    """Unit test for the scale function."""
    v1 = Vertex(x=1.0, y=2.0, z=3.0)
    ss = 10.0
    result = sm.scale(vertex=v1, scale_factor=ss)
    assert result.x == 10.0
    assert result.y == 20.0
    assert result.z == 30.0

    # docstring example
    v = Vertex(1, 2, 3)
    scale_factor = 2
    assert sm.scale(v, scale_factor) == Vertex(2, 4, 6)


def test_xyz():
    """Unit test to assure the (x, y, z) coordinate tuple is returned
    correctly.
    """
    vv = Vertex(x=1.1, y=2.2, z=3.3)
    gold = (1.1, 2.2, 3.3)
    result = sm.xyz(vv)
    assert result == gold

    # docstring example
    v = Vertex(1, 2, 3)
    assert sm.xyz(v) == (1, 2, 3)


def test_smoothing_neighbors():
    """Given the Double X test problem with completely made up
    node hierarchy, assure that `smoothing_neighbors` returns
    the correct neighbors.
    """
    ex = examples.double_x
    # neighbors = ex.neighbors  # borrow the neighbor connections
    neighbors = sm.node_node_connectivity(ex.elements)

    node_hierarchy = (
        Hierarchy.INTERIOR,
        Hierarchy.BOUNDARY,
        Hierarchy.PRESCRIBED,
        Hierarchy.PRESCRIBED,
        Hierarchy.BOUNDARY,
        Hierarchy.INTERIOR,
        Hierarchy.INTERIOR,
        Hierarchy.BOUNDARY,
        Hierarchy.BOUNDARY,
        Hierarchy.INTERIOR,
        Hierarchy.INTERIOR,
        Hierarchy.INTERIOR,
    )

    result = sm.smoothing_neighbors(neighbors=neighbors, node_hierarchy=node_hierarchy)
    gold_smoothing_neighbors = (
        (2, 4, 7),
        (3, 5, 8),
        (),
        (),
        (2, 4),
        (3, 5, 12),
        (1, 8, 10),
        (2, 9),
        (3, 8),
        (4, 7, 11),
        (5, 8, 10, 12),
        (6, 9, 11),
    )

    assert result == gold_smoothing_neighbors

    # doctring example
    neighbors = ((2, 3), (1, 4), (1, 5), (2, 6), (3,), (4,))
    node_hierarchy = (
        Hierarchy.INTERIOR,
        Hierarchy.BOUNDARY,
        Hierarchy.PRESCRIBED,
        Hierarchy.BOUNDARY,
        Hierarchy.INTERIOR,
        Hierarchy.INTERIOR,
    )
    gold = ((2, 3), (4,), (), (2,), (3,), (4,))
    assert sm.smoothing_neighbors(neighbors, node_hierarchy) == gold


def test_laplace_hierarchical_bracket():
    """Unit test for Laplace smoothing with hierarhical control
    on the Bracket example."""
    bracket = examples.bracket

    node_hierarchy = bracket.node_hierarchy
    # neighbors = bracket.neighbors
    neighbors = sm.node_node_connectivity(bracket.elements)
    node_hierarchy = bracket.node_hierarchy

    # If a node is PRESCRIBED, then it has no smoothing neighbors
    smoothing_neighbors = sm.smoothing_neighbors(
        neighbors=neighbors, node_hierarchy=node_hierarchy
    )
    gold_smoothing_neighbors = (
        (),  # 1
        (),  # 2
        (),  # 3
        (),  # 4
        (),  # 5
        (),  # 6
        (2, 6, 8, 12, 28),  # 7
        (3, 7, 9, 13, 29),  # 8
        (4, 8, 10, 14, 30),  # 9
        (),  # 10
        (),  # 11
        (7, 11, 13, 17, 33),  # 12
        (8, 12, 14, 18, 34),  # 13
        (9, 13, 15, 35),  # 14
        (),  # 15
        (),  # 16
        (12, 16, 18, 20, 38),  # 17
        (13, 17, 21, 39),  # 18
        (),  # 19
        (),  # 20
        (),
        (),  # 22
        (),
        (),  # 24
        (),
        (),  # 26
        (),
        (7, 23, 27, 29, 33),  # 28
        (8, 24, 28, 30, 34),  # 29
        (9, 25, 29, 31, 35),  # 30
        (),  # 31
        (),  # 32
        (12, 28, 32, 34, 38),  # 33
        (13, 29, 33, 35, 39),  # 34
        (14, 30, 34, 36),  # 35
        (),  # 36
        (),  # 37
        (17, 33, 37, 39, 41),  # 38
        (18, 34, 38, 42),  # 39
        (),  # 40
        (),  # 41
        (),  # 42
    )

    assert smoothing_neighbors == gold_smoothing_neighbors

    # specific test with lambda = 0.3 and num_iters = 10
    scale_lambda_test = 0.3
    num_iters_test = 10

    result = sm.smooth(
        vv=bracket.vertices,
        hexes=bracket.elements,
        node_hierarchy=bracket.node_hierarchy,
        prescribed_nodes=bracket.prescribed_nodes,
        scale_lambda=scale_lambda_test,
        num_iters=num_iters_test,
        algorithm=bracket.algorithm,
    )

    gold_vertices_10_iter = (
        Vertex(x=0, y=0, z=0),
        Vertex(x=1, y=0, z=0),
        Vertex(x=2, y=0, z=0),
        Vertex(x=3, y=0, z=0),
        Vertex(x=4, y=0, z=0),
        Vertex(x=0, y=1, z=0),
        Vertex(x=0.9974824535030984, y=0.9974824535030984, z=0.24593434133370803),
        Vertex(x=1.9620726956646117, y=1.0109475009958278, z=0.2837944855813176),
        Vertex(x=2.848322987789396, y=1.1190213008349328, z=0.24898414051620496),
        Vertex(x=3.695518130045147, y=1.5307337294603591, z=0),
        Vertex(x=0, y=2, z=0),
        Vertex(x=1.0109475009958275, y=1.9620726956646117, z=0.2837944855813176),
        Vertex(x=1.9144176939366933, y=1.9144176939366933, z=0.3332231502067546),
        Vertex(x=2.5912759493290007, y=1.961874667390146, z=0.29909606343914835),
        Vertex(x=2.8284271247461903, y=2.82842712474619, z=0),
        Vertex(x=0, y=3, z=0),
        Vertex(x=1.119021300834933, y=2.848322987789396, z=0.24898414051620493),
        Vertex(x=1.9618746673901462, y=2.5912759493290007, z=0.29909606343914835),
        Vertex(x=0, y=4, z=0),
        Vertex(x=1.5307337294603593, y=3.695518130045147, z=0),
        Vertex(x=2.8284271247461903, y=2.82842712474619, z=0),
        Vertex(x=0, y=0, z=1),
        Vertex(x=1, y=0, z=1),
        Vertex(x=2, y=0, z=1),
        Vertex(x=3, y=0, z=1),
        Vertex(x=4, y=0, z=1),
        Vertex(x=0, y=1, z=1),
        Vertex(x=0.9974824535030984, y=0.9974824535030984, z=0.7540656586662919),
        Vertex(x=1.9620726956646117, y=1.0109475009958278, z=0.7162055144186824),
        Vertex(x=2.848322987789396, y=1.119021300834933, z=0.7510158594837951),
        Vertex(x=3.695518130045147, y=1.5307337294603591, z=1),
        Vertex(x=0, y=2, z=1),
        Vertex(x=1.0109475009958275, y=1.9620726956646117, z=0.7162055144186824),
        Vertex(x=1.9144176939366933, y=1.9144176939366933, z=0.6667768497932453),
        Vertex(x=2.591275949329001, y=1.9618746673901462, z=0.7009039365608517),
        Vertex(x=2.8284271247461903, y=2.82842712474619, z=1),
        Vertex(x=0, y=3, z=1),
        Vertex(x=1.1190213008349328, y=2.848322987789396, z=0.751015859483795),
        Vertex(x=1.9618746673901462, y=2.5912759493290007, z=0.7009039365608516),
        Vertex(x=0, y=4, z=1),
        Vertex(x=1.5307337294603593, y=3.695518130045147, z=1),
        Vertex(x=2.8284271247461903, y=2.82842712474619, z=1),
    )

    assert result == gold_vertices_10_iter


def test_laplace_smoothing_double_x():
    """Unit test for Laplace smoothing with all dofs as BOUNDARY
    on the Double X example."""
    vv: Vertices = (
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

    hexes: Hexes = (
        (1, 2, 5, 4, 7, 8, 11, 10),
        (2, 3, 6, 5, 8, 9, 12, 11),
    )

    # nn: Neighbors = (
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
    # )

    nh: NodeHierarchy = (
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
    )

    scale_lambda: Final[float] = 0.3  # lambda for Laplace smoothing

    # iteration 1
    num_iters = 1  # single iteration of smoothing

    algo = SmoothingAlgorithm.LAPLACE

    aa = sm.smooth(
        vv=vv,
        hexes=hexes,
        node_hierarchy=nh,
        prescribed_nodes=None,
        scale_lambda=scale_lambda,
        num_iters=num_iters,
        algorithm=algo,
    )
    cc: Final[float] = scale_lambda / 3.0  # delta corner
    ee: Final[float] = scale_lambda / 4.0  # delta edge
    # define the gold standard fiducial
    gold = (
        Vertex(x=cc, y=cc, z=cc),  # node 1, corner
        Vertex(x=1.0, y=ee, z=ee),  # node 2, edge
        Vertex(x=2.0 - cc, y=cc, z=cc),  # node 3, corner
        #
        Vertex(x=cc, y=1.0 - cc, z=cc),  # node 4, corner
        Vertex(x=1.0, y=1.0 - ee, z=ee),  # node 5, edge
        Vertex(x=2.0 - cc, y=1.0 - cc, z=cc),  # node 6, corner
        #
        Vertex(x=cc, y=cc, z=1 - cc),  # node 7, corner
        Vertex(x=1.0, y=ee, z=1 - ee),  # node 8, edge
        Vertex(x=2.0 - cc, y=cc, z=1 - cc),  # node 9, corner
        #
        Vertex(x=cc, y=1.0 - cc, z=1 - cc),  # node 10, corner
        Vertex(x=1.0, y=1.0 - ee, z=1 - ee),  # node 11, edge
        Vertex(x=2.0 - cc, y=1.0 - cc, z=1 - cc),  # node 12, corner
    )
    assert aa == gold

    # iteration 2
    num_iters = 2  # overwrite, double iteration of smoothing

    aa2 = sm.smooth(
        vv=vv,
        hexes=hexes,
        node_hierarchy=nh,
        prescribed_nodes=None,
        scale_lambda=scale_lambda,
        num_iters=num_iters,
        algorithm=algo,
    )
    # define the gold standard fiducial
    gold2 = (
        (0.19, 0.1775, 0.1775),
        (1.0, 0.1425, 0.1425),
        (1.8099999999999998, 0.1775, 0.1775),
        (0.19, 0.8225, 0.1775),
        (1.0, 0.8575, 0.1425),
        (1.8099999999999998, 0.8225, 0.1775),
        (0.19, 0.1775, 0.8225),
        (1.0, 0.1425, 0.8575),
        (1.8099999999999998, 0.1775, 0.8225),
        (0.19, 0.8225, 0.8225),
        (1.0, 0.8575, 0.8575),
        (1.8099999999999998, 0.8225, 0.8225),
    )
    assert aa2 == gold2


def test_pair_ordered():
    """Unit test for pair ordered."""

    # small toy example
    given = ((3, 1), (2, 1))
    found = sm.pair_ordered(given)
    gold = ((1, 2), (1, 3))
    assert found == gold

    # example from 12 edges of a hex element
    given = (
        (1, 2),
        (2, 5),
        (4, 1),
        (5, 4),
        (7, 8),
        (8, 11),
        (11, 10),
        (10, 7),
        (1, 7),
        (2, 8),
        (5, 11),
        (4, 10),
    )  # overwrite
    gold = (
        (1, 2),
        (1, 4),
        (1, 7),
        (2, 5),
        (2, 8),
        (4, 5),
        (4, 10),
        (5, 11),
        (7, 8),
        (7, 10),
        (8, 11),
        (10, 11),
    )  # overwrite
    found = sm.pair_ordered(given)  # overwrite
    assert found == gold

    # docstring example
    pairs = ((3, 1), (2, 4), (5, 0))
    assert sm.pair_ordered(pairs) == ((0, 5), (1, 3), (2, 4))


def test_edge_pairs():
    """Units test to assure edge pairs are computed correctly."""
    elements = (
        (1, 2, 5, 4, 7, 8, 11, 10),
        (2, 3, 6, 5, 8, 9, 12, 11),
    )
    found = sm.edge_pairs(hexes=elements)
    gold = (
        (1, 2),
        (1, 4),
        (1, 7),
        (2, 3),
        (2, 5),
        (2, 8),
        (3, 6),
        (3, 9),
        (4, 5),
        (4, 10),
        (5, 6),
        (5, 11),
        (6, 12),
        (7, 8),
        (7, 10),
        (8, 9),
        (8, 11),
        (9, 12),
        (10, 11),
        (11, 12),
    )
    assert found == gold


def test_node_node_connectivity():
    """Tests that the node_node_connectivity function is properly
    implemented.
    """

    # from the Double X unit test

    hexes = (
        (1, 2, 5, 4, 7, 8, 11, 10),
        (2, 3, 6, 5, 8, 9, 12, 11),
    )

    gold_neighbors = (
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

    result = sm.node_node_connectivity(hexes)

    assert gold_neighbors == result

    # now with node number modifications to assure the
    # algorithm does not assume sequential node numbers:
    # 2 -> 22
    # 5 -> 55
    # 8 -> 88
    # 11 -> 111
    hexes_2 = (
        (1, 22, 55, 4, 7, 88, 111, 10),
        (22, 3, 6, 55, 88, 9, 12, 111),
    )

    gold_neighbors_2 = (
        (4, 7, 22),  # 1
        (6, 9, 22),  # 3
        (1, 10, 55),  # 4
        (3, 12, 55),  # 6
        (1, 10, 88),  # 7
        (3, 12, 88),  # 9
        (4, 7, 111),  # 10
        (6, 9, 111),  # 12
        (1, 3, 55, 88),  # 2 -> 22
        (4, 6, 22, 111),  # 5 -> 55
        (7, 9, 22, 111),  # 8 -> 88
        (10, 12, 55, 88),  # 11 -> 111
    )

    result_2 = sm.node_node_connectivity(hexes_2)

    assert gold_neighbors_2 == result_2

    # example from the L-bracket example
    hexes_bracket = (
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
    )

    gold_neighbors_bracket = (
        (2, 6, 22),
        (1, 3, 7, 23),
        (2, 4, 8, 24),
        (3, 5, 9, 25),
        (4, 10, 26),
        #
        (1, 7, 11, 27),
        (2, 6, 8, 12, 28),
        (3, 7, 9, 13, 29),
        (4, 8, 10, 14, 30),
        (5, 9, 15, 31),
        #
        (6, 12, 16, 32),
        (7, 11, 13, 17, 33),
        (8, 12, 14, 18, 34),
        (9, 13, 15, 35),
        (10, 14, 36),
        #
        (11, 17, 19, 37),
        (12, 16, 18, 20, 38),
        (13, 17, 21, 39),
        #
        (16, 20, 40),
        (17, 19, 21, 41),
        (18, 20, 42),
        # top layer
        (1, 23, 27),
        (2, 22, 24, 28),
        (3, 23, 25, 29),
        (4, 24, 26, 30),
        (5, 25, 31),
        #
        (6, 22, 28, 32),
        (7, 23, 27, 29, 33),
        (8, 24, 28, 30, 34),
        (9, 25, 29, 31, 35),
        (10, 26, 30, 36),
        #
        (11, 27, 33, 37),
        (12, 28, 32, 34, 38),
        (13, 29, 33, 35, 39),
        (14, 30, 34, 36),
        (15, 31, 35),
        #
        (16, 32, 38, 40),
        (17, 33, 37, 39, 41),
        (18, 34, 38, 42),
        #
        (19, 37, 41),
        (20, 38, 40, 42),
        (21, 39, 41),
    )

    result_bracket = sm.node_node_connectivity(hexes_bracket)

    assert gold_neighbors_bracket == result_bracket
