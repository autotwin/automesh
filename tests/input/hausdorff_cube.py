"""Writes the unit cube fixtures for the `automesh hausdorff` tests.

Each fixture is a unit cube, [0, 1]^3, with at most one point moved along z
by 0.3 at the center of the top face (x = y = 0.5, z = 1).  A spike moves the
point out to z = 1.3.  A dimple moves it in to z = 0.7.

Tessellations (ASCII STL):

* hausdorff_cube.stl: 12 triangles.
* hausdorff_cube_spike.stl, hausdorff_cube_dimple.stl: the top face is four
  triangles that meet at the moved apex.

Meshes (Abaqus):

* hausdorff_cube_hex1.inp: one hexahedron.
* hausdorff_cube_tet6.inp: the same cube as six tetrahedra.
* hausdorff_cube_hex8_spike.inp, hausdorff_cube_hex8_dimple.inp: a 2 x 2 x 2
  grid of hexahedra with the top-center node moved.
* hausdorff_cube_tet48_spike.inp, hausdorff_cube_tet48_dimple.inp: the same
  grids, each hexahedron split into six tetrahedra.

Run from any directory: python3 hausdorff_cube.py
"""

from itertools import permutations
from pathlib import Path

HERE = Path(__file__).parent
OFFSET = 0.3
CENTER = (0.5, 0.5, 1.0)


def sub(a, b):
    return tuple(x - y for x, y in zip(a, b))


def cross(a, b):
    return (
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    )


def dot(a, b):
    return sum(x * y for x, y in zip(a, b))


def unit(a):
    norm = dot(a, a) ** 0.5
    return tuple(x / norm for x in a)


def cube_triangles(top):
    """Outward-wound triangles of the unit cube; `top` gives the top face."""
    p = {
        (i, j, k): (float(i), float(j), float(k))
        for i in (0, 1)
        for j in (0, 1)
        for k in (0, 1)
    }
    quads = [
        [p[0, 0, 0], p[0, 1, 0], p[1, 1, 0], p[1, 0, 0]],  # z = 0
        [p[0, 0, 0], p[1, 0, 0], p[1, 0, 1], p[0, 0, 1]],  # y = 0
        [p[1, 0, 0], p[1, 1, 0], p[1, 1, 1], p[1, 0, 1]],  # x = 1
        [p[1, 1, 0], p[0, 1, 0], p[0, 1, 1], p[1, 1, 1]],  # y = 1
        [p[0, 1, 0], p[0, 0, 0], p[0, 0, 1], p[0, 1, 1]],  # x = 0
    ]
    triangles = []
    for a, b, c, d in quads:
        triangles += [(a, b, c), (a, c, d)]
    corners = [p[0, 0, 1], p[1, 0, 1], p[1, 1, 1], p[0, 1, 1]]
    if top is None:
        a, b, c, d = corners
        triangles += [(a, b, c), (a, c, d)]
    else:
        for n in range(4):
            triangles.append((corners[n], corners[(n + 1) % 4], top))
    return triangles


def write_stl(name, triangles):
    lines = [f"solid {name}"]
    for a, b, c in triangles:
        n = unit(cross(sub(b, a), sub(c, a)))
        lines.append(f"  facet normal {n[0]:.17g} {n[1]:.17g} {n[2]:.17g}")
        lines.append("    outer loop")
        for v in (a, b, c):
            lines.append(f"      vertex {v[0]:.17g} {v[1]:.17g} {v[2]:.17g}")
        lines.append("    endloop")
        lines.append("  endfacet")
    lines.append(f"endsolid {name}")
    (HERE / f"{name}.stl").write_text("\n".join(lines) + "\n")


def grid(cells, moved_z):
    """Nodes and hexahedra of a cells^3 grid on the unit cube."""
    n = cells + 1
    nodes = []
    for k in range(n):
        for j in range(n):
            for i in range(n):
                point = (i / cells, j / cells, k / cells)
                if moved_z is not None and point == CENTER:
                    point = (0.5, 0.5, moved_z)
                nodes.append(point)

    def index(i, j, k):
        return i + n * (j + n * k)

    hexes = []
    for k in range(cells):
        for j in range(cells):
            for i in range(cells):
                hexes.append(
                    [
                        index(i, j, k),
                        index(i + 1, j, k),
                        index(i + 1, j + 1, k),
                        index(i, j + 1, k),
                        index(i, j, k + 1),
                        index(i + 1, j, k + 1),
                        index(i + 1, j + 1, k + 1),
                        index(i, j + 1, k + 1),
                    ]
                )
    return nodes, hexes


def volume(nodes, tet):
    a, b, c, d = (nodes[t] for t in tet)
    return dot(sub(b, a), cross(sub(c, a), sub(d, a))) / 6.0


def tets(nodes, hexes):
    """Splits each hexahedron into six tetrahedra about its 0-6 diagonal.

    Every cell shares the same diagonal direction, so the split conforms
    across neighboring cells.
    """
    local = {
        (0, 0, 0): 0,
        (1, 0, 0): 1,
        (1, 1, 0): 2,
        (0, 1, 0): 3,
        (0, 0, 1): 4,
        (1, 0, 1): 5,
        (1, 1, 1): 6,
        (0, 1, 1): 7,
    }
    result = []
    for h in hexes:
        for axes in permutations(range(3)):
            corner = [0, 0, 0]
            path = [local[tuple(corner)]]
            for axis in axes:
                corner[axis] = 1
                path.append(local[tuple(corner)])
            tet = [h[q] for q in path]
            if volume(nodes, tet) < 0.0:
                tet[2], tet[3] = tet[3], tet[2]
            result.append(tet)
    return result


def hex_volume_sign_check(nodes, hexes):
    """Checks every corner tetrahedron of every hexahedron for positive volume."""
    corners = [
        (0, 1, 3, 4),
        (1, 2, 0, 5),
        (2, 3, 1, 6),
        (3, 0, 2, 7),
        (4, 7, 5, 0),
        (5, 4, 6, 1),
        (6, 5, 7, 2),
        (7, 6, 4, 3),
    ]
    for h in hexes:
        for c in corners:
            assert volume(nodes, [h[q] for q in c]) > 0.0, "inverted hexahedron"


def write_inp(name, nodes, elements, element_type):
    lines = ["*Heading", f" {name}", "*Node"]
    for number, (x, y, z) in enumerate(nodes, start=1):
        lines.append(f"{number}, {x:.17g}, {y:.17g}, {z:.17g}")
    lines.append(f"*Element, type={element_type}, elset=EB1")
    for number, element in enumerate(elements, start=1):
        lines.append(f"{number}, " + ", ".join(str(e + 1) for e in element))
    (HERE / f"{name}.inp").write_text("\n".join(lines) + "\n")


def main():
    write_stl("hausdorff_cube", cube_triangles(None))
    write_stl("hausdorff_cube_spike", cube_triangles((0.5, 0.5, 1.0 + OFFSET)))
    write_stl("hausdorff_cube_dimple", cube_triangles((0.5, 0.5, 1.0 - OFFSET)))

    nodes, hexes = grid(1, None)
    hex_volume_sign_check(nodes, hexes)
    write_inp("hausdorff_cube_hex1", nodes, hexes, "C3D8")
    write_inp("hausdorff_cube_tet6", nodes, tets(nodes, hexes), "C3D4")

    for label, z in (("spike", 1.0 + OFFSET), ("dimple", 1.0 - OFFSET)):
        reference, _ = grid(2, None)
        nodes, hexes = grid(2, z)
        hex_volume_sign_check(nodes, hexes)
        tetrahedra = tets(reference, hexes)
        assert all(volume(nodes, t) > 0.0 for t in tetrahedra), "inverted tet"
        write_inp(f"hausdorff_cube_hex8_{label}", nodes, hexes, "C3D8")
        write_inp(f"hausdorff_cube_tet48_{label}", nodes, tetrahedra, "C3D4")


if __name__ == "__main__":
    main()
