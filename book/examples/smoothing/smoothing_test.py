r"""This module tests the smoothing modules.
Example:
    cd ~/autotwin/automesh
    Activate the venv with one of the following:
    source .venv/bin/activate       # for bash shell
    source .venv/bin/activate.csh   # for c shell
    source .venv/bin/activate.fish  # for fish shell
    .\.venv\Scripts\activate        # for powershell
    cd ~/autotwin/automesh/book/examples/smooothing
    python -m pytest smoothing_test.py
Reference: DoubleX unit test
    https://autotwin.github.io/automesh/examples/unit_tests/index.html#double-x
"""

from typing import Final

import smoothing as sm
import smoothing_types as ty
# import sandbox.smoothing as sm
# import sandbox.smoothing_types as ty

# Type alias for functional style methods
# https://docs.python.org/3/library/typing.html#type-aliases
DofSet = ty.DofSet
Hexes = ty.Hexes
Neighborhoods = ty.Neighborhoods
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


def test_add():
    """Unit test for the addition of Vertex v1 and Vertex v2."""
    v1 = Vertex(x=1.0, y=2.0, z=3.0)
    v2 = Vertex(x=4.0, y=7.0, z=1.0)
    vv = sm.add(v1=v1, v2=v2)
    assert vv.x == 5.0
    assert vv.y == 9.0
    assert vv.z == 4.0


def test_subtract():
    """Unit test for the subtraction of Vertex v2 from Vertex v1, (v1-v2)."""
    v1 = Vertex(x=1.0, y=2.0, z=3.0)
    v2 = Vertex(x=4.0, y=7.0, z=1.0)
    vv = sm.subtract(v1=v1, v2=v2)
    assert vv.x == -3.0
    assert vv.y == -5.0
    assert vv.z == 2.0


def test_scale():
    """Unit test for the scale function."""
    v1 = Vertex(x=1.0, y=2.0, z=3.0)
    ss = 10.0
    result = sm.scale(vertex=v1, scale_factor=ss)
    assert result.x == 10.0
    assert result.y == 20.0
    assert result.z == 30.0


def test_xyz():
    """Unit test to assure the (x, y, z) coordinate tuple is returned
    correctly.
    """
    vv = Vertex(x=1.1, y=2.2, z=3.3)
    gold = (1.1, 2.2, 3.3)
    result = sm.xyz(vv)
    assert result == gold


def test_laplace_smoothing():
    """Unit test for Laplace smoothing with all dofs as FREE_INTERIOR."""
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

    nn: Neighborhoods = (
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

    ds: DofSet = (
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

    scale_lambda: Final[float] = 0.3  # lambda for Laplace smoothing

    # iteration 1
    num_iters = 1  # single iteration of smoothing

    algo = SmoothingAlgorithm.LAPLACE

    aa = sm.smooth(
        vv=vv, nn=nn, ds=ds, sf=scale_lambda, num_iters=num_iters, algo=algo
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
        vv=vv, nn=nn, ds=ds, sf=scale_lambda, num_iters=num_iters, algo=algo
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


# def test_taubin_smoothing(
#     vv: Vertices,
#     nn: Neighborhoods,
#     ds: DofSet,
#     sf_lambda: float,
#     sf_mu: float
# ):
#     """Unit test for Taubin smoothing with all dofs as FREE_INTERIOR."""
#     aa = smooth(vv=vv, nn=nn, ds=ds, sf=sf_lambda)
#     bb = smooth(vv=aa, nn=nn, ds=ds, sf=sf_mu)
#     breakpoint()
#     cc: Final[float] = sf / 3.0  # delta corner
#     ee: Final[float] = sf / 4.0  # delta edge
#     # define the gold standard fiducial
#     gold = (
#         Vertex(x=cc, y=cc, z=cc),  # node 1, corner
#         Vertex(x=1.0, y=ee, z=ee),  # node 2, edge
#         Vertex(x=2.0-cc, y=cc, z=cc),  # node 3, corner
#         #
#         Vertex(x=cc, y=1.0-cc, z=cc),  # node 4, corner
#         Vertex(x=1.0, y=1.0-ee, z=ee),  # node 5, edge
#         Vertex(x=2.0-cc, y=1.0-cc, z=cc),  # node 6, corner
#         #
#         Vertex(x=cc, y=cc, z=1-cc),  # node 7, corner
#         Vertex(x=1.0, y=ee, z=1-ee),  # node 8, edge
#         Vertex(x=2.0-cc, y=cc, z=1-cc),  # node 9, corner
#         #
#         Vertex(x=cc, y=1.0-cc, z=1-cc),  # node 10, corner
#         Vertex(x=1.0, y=1.0-ee, z=1-ee),  # node 11, edge
#         Vertex(x=2.0-cc, y=1.0-cc, z=1-cc),  # node 12, corner
#     )
#     assert bb == gold


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
    # breakpoint()


def test_edge_pairs():
    """Units test to assure edge pairs are computed correctly."""
    hexes = (
        (1, 2, 5, 4, 7, 8, 11, 10),
        (2, 3, 6, 5, 8, 9, 12, 11),
    )
    found = sm.edge_pairs(hexes=hexes)
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


def test_hierarchical_neighborhoods():
    """Given a neighborhoods table based on connectivity, and a degree of
    freedom dofset table, test that the returned neighborhoods table is
    correctly calculated.
    """
    given: Neighborhoods
