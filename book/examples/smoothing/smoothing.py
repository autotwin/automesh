"""This module contains the core computations for smoothing algorithms.
"""

# import sandbox.smoothing_types as ty
import smoothing_types as ty


# Type alias for functional style methods
# https://docs.python.org/3/library/typing.html#type-aliases
DofSet = ty.DofSet
Elements = ty.Elements
Neighbors = ty.Neighbors
Vertex = ty.Vertex
Vertices = ty.Vertices
SmoothingAlgorithm = ty.SmoothingAlgorithm


def average_position(vv: Vertices) -> Vertex:
    """Give a list of vertices, returns the average position of the
    vertices."""

    n_vertices = len(vv)
    assert n_vertices > 0, "Error: number of vertices must be positive."
    xs = [v.x for v in vv]
    ys = [v.y for v in vv]
    zs = [v.z for v in vv]
    x_ave = sum(xs) / n_vertices
    y_ave = sum(ys) / n_vertices
    z_ave = sum(zs) / n_vertices

    return Vertex(x=x_ave, y=y_ave, z=z_ave)


def add(v1: Vertex, v2: Vertex) -> Vertex:
    """Returns the Vertex addition of (v1 + v2)."""
    dx = v1.x + v2.x
    dy = v1.y + v2.y
    dz = v1.z + v2.z
    return Vertex(x=dx, y=dy, z=dz)


def subtract(v1: Vertex, v2: Vertex) -> Vertex:
    """Returns the Vertex subtraction of (v1 - v2)."""
    dx = v1.x - v2.x
    dy = v1.y - v2.y
    dz = v1.z - v2.z
    return Vertex(x=dx, y=dy, z=dz)


def scale(vertex: Vertex, scale_factor: float) -> Vertex:
    """Scales a vertex by a scale factor."""
    x = scale_factor * vertex.x
    y = scale_factor * vertex.y
    z = scale_factor * vertex.z
    return Vertex(x=x, y=y, z=z)


def xyz(v1: Vertex) -> tuple[float, float, float]:
    """Given a vertex, returns the coordinates as (x, y, z)."""
    aa, bb, cc = v1.x, v1.y, v1.z
    return (aa, bb, cc)


def smooth(
    vv: Vertices,
    nn: Neighbors,
    ds: DofSet,
    sf: float,
    num_iters: int,
    algo: SmoothingAlgorithm,
) -> Vertices:
    """Given an initial position of vertices, the vertex neighbors,
    and the dof classification of each vertex, perform Laplace
    smoothing for num_iter iterations, and return the updated
    coordinates.
    """
    assert num_iters >= 1, "`num_iters` must be 1 or greater"

    print(f"Smoothing algorithm: {algo.value}")

    vertices_old = vv

    for k in range(num_iters):

        print(f"Iteration: {k+1}")
        vertices_new = []

        for vertex, neighbors, dof in zip(vertices_old, nn, ds):
            # debug vertex by vertex
            # print(f"vertex {vertex}, dof {dof}, neighbors {neighbors}")
            # for now, no hierarchical smoohing
            # assume all dofs are FREE_INTERIOR

            # account for zero-index instead of 1-index:
            neighbor_vertices = tuple(
                vertices_old[i - 1] for i in neighbors
            )  # zero index

            neighbor_average = average_position(neighbor_vertices)
            delta = subtract(v1=neighbor_average, v2=vertex)
            lambda_delta = scale(vertex=delta, scale_factor=sf)
            vertex_new = add(v1=vertex, v2=lambda_delta)
            vertices_new.append(vertex_new)
            # breakpoint()

        # breakpoint()
        vertices_old = vertices_new  # overwrite for new k loop

    # breakpoint()
    return tuple(vertices_new)


def pair_ordered(ab: tuple[tuple[int, int], ...]) -> tuple:
    """Given a tuple of form ((a, b), (c, d), ...) orders all the subpairs
    such that the first index drives global order, and the second index
    drives secondary order.
    """
    firsts, seconds = zip(*ab)

    ab_ordered = ()

    for a, b in zip(firsts, seconds):
        if a < b:
            ab_ordered += ((a, b),)
        else:
            ab_ordered += ((b, a),)

    # for a in firsts:
    #     print(f"a = {a}")

    # for b in seconds:
    #     print(f"b = {b}")

    result = tuple(sorted(ab_ordered))
    # breakpoint()
    return result


def edge_pairs(ees: Elements):
    """Returns all the line pairs from element connectivity, for use
    with drawing edges of elements."""

    # almost perfect with collecting unique pairs, but there are some
    # overlapping pairs, not a big dealbptt
    pairs = ()
    for ee in ees:
        # bottom_face = tuple(sorted(list(zip(ee[0:4], ee[1:4] + (ee[0],)))))
        bottom_face = pair_ordered(tuple(zip(ee[0:4], ee[1:4] + (ee[0],))))
        # top_face = tuple(list(zip(ee[4:8], ee[5:8] + (ee[4],))))
        top_face = pair_ordered(tuple(zip(ee[4:8], ee[5:8] + (ee[4],))))
        verticals = pair_ordered(
            (
                (ee[0], ee[4]),
                (ee[1], ee[5]),
                (ee[2], ee[6]),
                (ee[3], ee[7]),
            )
        )
        t3 = bottom_face + top_face + verticals
        pairs = pairs + tuple(t3)
        # breakpoint()

    return tuple(sorted(set(pairs)))
