r"""This module, smoothing.py, contains the core computations for
smoothing algorithms.
"""

# import sandbox.smoothing_types as ty
import smoothing_types as ty


# Type alias for functional style methods
# https://docs.python.org/3/library/typing.html#type-aliases
Hexes = ty.Hexes
Hierarchy = ty.Hierarchy
Neighbors = ty.Neighbors
NodeHierarchy = ty.NodeHierarchy
PrescribedNodes = ty.PrescribedNodes
Vertex = ty.Vertex
Vertices = ty.Vertices
SmoothingAlgorithm = ty.SmoothingAlgorithm


def average_position(vertices: Vertices) -> Vertex:
    """Calculate the average position of a list of vertices.

    This function computes the average coordinates (x, y, z) of a given
    list of Vertex objects. It raises an assertion error if the input
    list is empty.

    Parameters:
    vertices (Vertices): A list or collection of Vertex objects, where
                         each Vertex has x, y, and z attributes
                         representing its coordinates in 3D space.

    Returns:
    Vertex: A new Vertex object representing the average position of the
            input vertices, with x, y, and z attributes set to the
            average coordinates.

    Raises:
    AssertionError: If the number of vertices is zero, indicating that
                    the input list must contain at least one vertex.

    Example:
    >>> v1 = Vertex(1, 2, 3)
    >>> v2 = Vertex(4, 5, 6)
    >>> average_position([v1, v2])
    Vertex(x=2.5, y=3.5, z=4.5)
    """

    n_vertices = len(vertices)
    assert n_vertices > 0, "Error: number of vertices must be positive."
    xs = [v.x for v in vertices]
    ys = [v.y for v in vertices]
    zs = [v.z for v in vertices]
    x_ave = sum(xs) / n_vertices
    y_ave = sum(ys) / n_vertices
    z_ave = sum(zs) / n_vertices

    return Vertex(x=x_ave, y=y_ave, z=z_ave)


def add(v1: Vertex, v2: Vertex) -> Vertex:
    """
    Add two Vertex objects component-wise.

    This function takes two Vertex objects and returns a new Vertex
    object that represents the component-wise addition of the two
    input vertices.

    Parameters:
    v1 (Vertex): The first Vertex object to be added.
    v2 (Vertex): The second Vertex object to be added.

    Returns:
    Vertex: A new Vertex object representing the result of the addition,
            with x, y, and z attributes set to the sum of the corresponding
            attributes of v1 and v2.

    Example:
    >>> v1 = Vertex(1, 2, 3)
    >>> v2 = Vertex(4, 5, 6)
    >>> add(v1, v2)
    Vertex(x=5, y=7, z=9)
    """
    dx = v1.x + v2.x
    dy = v1.y + v2.y
    dz = v1.z + v2.z
    return Vertex(x=dx, y=dy, z=dz)


def subtract(v1: Vertex, v2: Vertex) -> Vertex:
    """
    Subtract one Vertex object from another component-wise.

    This function takes two Vertex objects and returns a new Vertex
    object that represents the component-wise subtraction of the second
    vertex from the first.

    Parameters:
    v1 (Vertex): The Vertex object from which v2 will be subtracted.
    v2 (Vertex): The Vertex object to be subtracted from v1.

    Returns:
    Vertex: A new Vertex object representing the result of the subtraction,
            (v1 - v2), with x, y, and z attributes set to the difference
            of the corresponding attributes of v1 and v2.

    Example:
    >>> v1 = Vertex(8, 5, 2)
    >>> v2 = Vertex(1, 2, 3)
    >>> subtract(v1, v2)
    Vertex(x=7, y=3, z=-1)
    """
    dx = v1.x - v2.x
    dy = v1.y - v2.y
    dz = v1.z - v2.z
    return Vertex(x=dx, y=dy, z=dz)


def scale(vertex: Vertex, scale_factor: float) -> Vertex:
    """
    Scale a Vertex object by a given scale factor.

    This function takes a Vertex object and a scale factor, and returns
    a new Vertex object that represents the original vertex scaled by
    the specified factor.

    Parameters:
    vertex (Vertex): The Vertex object to be scaled.
    scale_factor (float): The factor by which to scale the vertex.
                          This can be any real number, including
                          positive, negative, or zero.

    Returns:
    Vertex: A new Vertex object representing the scaled vertex, with
            x, y, and z attributes set to the original coordinates
            multiplied by the scale factor.

    Example:
    >>> v = Vertex(1, 2, 3)
    >>> scale_factor = 2
    >>> scale(v, scale_factor)
    Vertex(x=2, y=4, z=6)
    """
    x = scale_factor * vertex.x
    y = scale_factor * vertex.y
    z = scale_factor * vertex.z
    return Vertex(x=x, y=y, z=z)


def xyz(v1: Vertex) -> tuple[float, float, float]:
    """
    Extract the coordinates of a Vertex object.

    This function takes a Vertex object and returns its coordinates
    as a tuple in the form (x, y, z).

    Parameters:
    v1 (Vertex): The Vertex object from which to extract the coordinates.

    Returns:
    tuple[float, float, float]: A tuple containing the x, y, and z
                                 coordinates of the vertex.

    Example:
    >>> v = Vertex(1, 2, 3)
    >>> xyz(v)
    (1, 2, 3)
    """
    aa, bb, cc = v1.x, v1.y, v1.z
    return (aa, bb, cc)


def smoothing_neighbors(neighbors: Neighbors, node_hierarchy: NodeHierarchy):
    """
    Determine the smoothing neighbors for each node based on its
    hierarchy level.

    This function takes an original neighbors structure, which is defined
    by the connectivity of a mesh, and a node hierarchy. It returns a
    subset of the original neighbors that are used for smoothing, based
    on the hierarchy levels of the nodes.

    Parameters:
    neighbors (Neighbors): A structure containing the original neighbors
                           for each node in the mesh.
    node_hierarchy (NodeHierarchy): A structure that defines the hierarchy
                                     levels of the nodes, which can be
                                     INTERIOR, BOUNDARY, or PRESCRIBED.

    Returns:
    tuple: A new structure containing the neighbors used for smoothing,
           which is a subset of the original neighbors based on the
           hierarchy levels.

    Raises:
    ValueError: If a hierarchy value is not in the expected range
                of [INTERIOR, BOUNDAR, PRESCRIBED, or [0, 1, 2],
                respectively.

    Example:
    INTERIOR     PRESCRIBED      INTERIOR
       (1) -------- (3) ----------- (5)
        |
       (2) -------- (4) ----------- (6)
    BOUNDARY     BOUNDARY        INTERIOR

    >>> neighbors = ((2, 3), (1, 4), (1, 5), (2, 6), (3,), (4,))
    >>> node_hierarchy = (Hierarchy.INTERIOR, Hierarchy.BOUNDARY,
                          Hierarchy.PRESCRIBED, Hierarchy.BOUNDARY,
                          Hierarchy.INTERIOR, Hierarchy.INTERIOR)
    >>> smoothing_neighbors(neighbors, node_hierarchy)
    ((2, 3), (4,), (), (2,), (3,), (4,))
    """
    neighbors_new = ()

    for node, level in enumerate(node_hierarchy):
        nei_old = neighbors[node]
        # print(f"Processing node {node+1}, neighbors: {nei_old}")
        levels = [int(node_hierarchy[x - 1].value) for x in nei_old]
        nei_new = ()

        # breakpoint()
        if level == Hierarchy.INTERIOR:
            # print("INTERIOR node")
            nei_new = nei_old
        elif level == Hierarchy.BOUNDARY:
            # print("BOUNDARY node")
            for nn, li in zip(nei_old, levels):
                if li >= level.value:
                    nei_new += (nn,)
        elif level == Hierarchy.PRESCRIBED:
            # print("PRESCRIBED node")
            nei_new = ()
        else:
            raise ValueError("Hierarchy value must be in [0, 1, 2]")

        neighbors_new += (nei_new,)

    return neighbors_new


def smooth(
    vv: Vertices,
    hexes: Hexes,
    node_hierarchy: NodeHierarchy,
    prescribed_nodes: PrescribedNodes,
    scale_lambda: float,
    num_iters: int,
    algorithm: SmoothingAlgorithm,
) -> Vertices:
    """
    Given an initial position of vertices, the vertex neighbors,
    and the dof classification of each vertex, perform Laplace
    smoothing for num_iter iterations, and return the updated
    coordinates.
    """
    print(f"Smoothing algorithm: {algorithm.value}")

    assert num_iters >= 1, "`num_iters` must be 1 or greater"

    nn = node_node_connectivity(hexes)

    # if the node_hierarchy contains a Hierarchy.PRESCRIBED type; or
    # the the PrescribedNodes must not be None
    if Hierarchy.PRESCRIBED in node_hierarchy:
        info = "Smoothing algorithm with hierarchical control"
        info += " and PRESCRIBED node positions."
        print(info)
        estr = "Error, NodeHierarchy desigates PRESCRIBED nodes, but no values"
        estr += " for (x, y, z) prescribed positions were given."
        assert prescribed_nodes is not None, estr

        n_nodes_prescribed = node_hierarchy.count(Hierarchy.PRESCRIBED)
        n_prescribed_xyz = len(prescribed_nodes)
        estr = f"Error: number of PRESCRIBED nodes: {n_nodes_prescribed}"
        estr += " must match the number of"
        estr += f" prescribed Vertices(x, y, z): {n_prescribed_xyz}"
        assert n_nodes_prescribed == n_prescribed_xyz, estr

        # update neighbors
        nn = smoothing_neighbors(
            neighbors=nn, node_hierarchy=node_hierarchy
        )  # overwrite

        # update vertex positions
        vv_list = list(vv)  # make mutable
        for node_id, node_xyz in prescribed_nodes:
            # print(f"Update node {node_id}")
            # print(f"  from {vv_list[node_id-1]}")
            # print(f"  to {node_xyz}")
            vv_list[node_id - 1] = node_xyz  # zero index, overwrite xyz

        # revert to immutable
        vv = tuple(vv_list)  # overwrite

    vertices_old = vv

    # breakpoint()
    for k in range(num_iters):
        print(f"Iteration: {k + 1}")
        vertices_new = []

        for vertex, neighbors in zip(vertices_old, nn):
            # debug vertex by vertex
            # print(f"vertex {vertex}, neighbors {neighbors}")

            # account for zero-index instead of 1-index:
            neighbor_vertices = tuple(
                vertices_old[i - 1] for i in neighbors
            )  # zero index

            if len(neighbor_vertices) > 0:
                neighbor_average = average_position(neighbor_vertices)
                delta = subtract(v1=neighbor_average, v2=vertex)
                lambda_delta = scale(vertex=delta, scale_factor=scale_lambda)
                vertex_new = add(v1=vertex, v2=lambda_delta)
            elif len(neighbor_vertices) == 0:
                # print("Prescribed node, no smoothing update.")
                vertex_new = vertex
            else:
                estr = "Error: neighbor_vertices negative length"
                raise ValueError(estr)

            vertices_new.append(vertex_new)
            # breakpoint()

        vertices_old = vertices_new  # overwrite for new k loop

    # breakpoint()
    return tuple(vertices_new)


def pair_ordered(ab: tuple[tuple[int, int], ...]) -> tuple:
    """
    Order pairs of integers based on their values.

    Given a tuple of pairs in the form ((a, b), (c, d), ...), this
    function orders each pair such that the smaller integer comes
    first. It then sorts the resulting pairs primarily by the first
    element and secondarily by the second element.

    Parameters:
    ab (tuple[tuple[int, int], ...]): A tuple containing pairs of integers.

    Returns:
    tuple: A new tuple containing the ordered pairs, where each pair
           is sorted internally and the entire collection is sorted
           based on the first and second elements.

    Example:
    >>> pairs = ((3, 1), (2, 4), (5, 0))
    >>> pair_ordered(pairs)
    ((0, 5), (1, 3), (2, 4))
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
    return result


def edge_pairs(hexes: Hexes):
    """
    Extract unique edge pairs from hex element connectivity.

    This function takes a collection of hex elements and returns all
    unique line pairs that represent the edges of the hex elements.
    The edges are derived from the connectivity of the hex elements,
    including both the horizontal edges (bottom and top faces) and
    the vertical edges.

    Used for drawing edges of finite elements.

    Parameters:
    hexes (Hexes): A collection of hex elements, where each hex is
                   represented by a tuple of vertex indices.

    Returns:
    tuple: A sorted tuple of unique edge pairs, where each pair is
           represented as a tuple of two vertex indices.
    """
    pairs = ()
    for ee in hexes:
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


def node_node_connectivity(hexes: Hexes) -> Neighbors:
    """
    Determine the connectivity of nodes to other nodes from
    a list of hexahedral elements.

    This function takes a list of hexahedral elements and returns a
    list of nodes connected to each node based on the edges define
    by the hexahedral elements. Each node's connectivity is represented
    as a tuple of neighboring nodes.

    Parameters:
    hexes (Hexes): A collection of hexahedral elements, where each
                   element is represented by a tuple of node indices.

    Returns:
    Neighbors: A tuple of tuples, where each inner tuple contains the
               indices of nodes connected to the corresponding node
               in the input list.
    """

    # create an empty dictionary from the node numbers
    edict = {item: () for sublist in hexes for item in sublist}

    ep = edge_pairs(hexes)

    for edge in ep:
        aa, bb = edge
        # existing value at edict[a] is a_old
        a_old = edict[aa]
        # existing value at edict[b] is b_old
        b_old = edict[bb]

        # new value
        a_new = (bb,)
        b_new = (aa,)

        # update dictionary
        edict[aa] = a_old + a_new
        edict[bb] = b_old + b_new

    # create a new dictionary, sorted by keys
    sorted_edict = dict(sorted(edict.items()))
    neighbors = tuple(sorted_edict.values())
    return neighbors
