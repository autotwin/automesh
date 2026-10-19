"""Figures and distances for the unit cube example on the Hausdorff Distance page.

The script builds four cases on the unit cube [0, 1]^3.  Each case moves one
point at the center of the top face, (0.5, 0.5, 1), along z by 0.3:

* tessellation spike: the tessellation B rises to an apex at z = 1.3, and the
  mesh A is one hexahedron;
* tessellation dimple: the apex sits at z = 0.7 instead;
* mesh spike: A is a 2 x 2 x 2 grid of hexahedra whose top-center node moves
  to z = 1.3, and B is the 12-triangle cube;
* mesh dimple: the node moves to z = 0.7 instead.

For each case it prints the sampled directed distances, which `automesh
hausdorff` reports, and the continuous directed distances, estimated by dense
sampling of each surface.  It writes hausdorff_cases.png.

Run: uv run --with numpy --with matplotlib python hausdorff_figures.py
"""

from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np
from mpl_toolkits.mplot3d.art3d import Line3DCollection, Poly3DCollection

HERE = Path(__file__).parent
OFFSET = 0.3
MESH = "#2a78d6"
TESSELLATION = "#eb6834"
INK = "#2b2b2b"
MUTED = "#6b6b6b"

HEX_FACES = [
    [0, 3, 2, 1],
    [0, 1, 5, 4],
    [1, 2, 6, 5],
    [2, 3, 7, 6],
    [3, 0, 4, 7],
    [4, 5, 6, 7],
]


def cube_tessellation(apex_z):
    """Outward-wound triangles of the unit cube, with an optional top apex."""
    c = np.array(
        [[i, j, k] for k in (0, 1) for j in (0, 1) for i in (0, 1)], dtype=float
    )
    quads = [[0, 2, 3, 1], [0, 1, 5, 4], [1, 3, 7, 5], [3, 2, 6, 7], [2, 0, 4, 6]]
    triangles = []
    for a, b, cc, d in quads:
        triangles += [[c[a], c[b], c[cc]], [c[a], c[cc], c[d]]]
    top = [c[4], c[5], c[7], c[6]]
    if apex_z is None:
        triangles += [[top[0], top[1], top[2]], [top[0], top[2], top[3]]]
        vertices = c
    else:
        apex = np.array([0.5, 0.5, apex_z])
        for n in range(4):
            triangles.append([top[n], top[(n + 1) % 4], apex])
        vertices = np.vstack([c, apex])
    return np.array(triangles), vertices


def hex_grid(cells, moved_z):
    """Nodes and hexahedra of a cells^3 grid, with an optional moved top-center node."""
    n = cells + 1
    nodes = np.array(
        [
            [i / cells, j / cells, k / cells]
            for k in range(n)
            for j in range(n)
            for i in range(n)
        ]
    )
    if moved_z is not None:
        center = np.all(np.isclose(nodes, [0.5, 0.5, 1.0]), axis=1)
        nodes[center, 2] = moved_z

    def index(i, j, k):
        return i + n * (j + n * k)

    hexes = [
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
        for k in range(cells)
        for j in range(cells)
        for i in range(cells)
    ]
    return nodes, hexes


def mesh_surface(nodes, hexes):
    """Exterior quads, split along the shorter diagonal (a tie picks n0-n2)."""
    count = {}
    for h in hexes:
        for face in HEX_FACES:
            quad = [h[f] for f in face]
            key = tuple(sorted(quad))
            count.setdefault(key, []).append(quad)
    quads = [faces[0] for faces in count.values() if len(faces) == 1]
    triangles = []
    for n0, n1, n2, n3 in quads:
        d02 = np.linalg.norm(nodes[n0] - nodes[n2])
        d13 = np.linalg.norm(nodes[n1] - nodes[n3])
        if d13 < d02:
            triangles += [[n0, n1, n3], [n1, n2, n3]]
        else:
            triangles += [[n0, n1, n2], [n0, n2, n3]]
    boundary = sorted({n for q in quads for n in q})
    return nodes[np.array(triangles)], nodes[boundary], quads


def segment_distance(points, a, b):
    ab = b - a
    t = np.clip(((points - a) @ ab) / (ab @ ab), 0.0, 1.0)
    return np.linalg.norm(points - (a + t[:, None] * ab), axis=1)


def point_triangle_distance(points, triangle):
    """Exact Euclidean distance from each point to one triangle."""
    a, b, c = triangle
    normal = np.cross(b - a, c - a)
    normal /= np.linalg.norm(normal)
    height = (points - a) @ normal
    projected = points - height[:, None] * normal
    v0, v1, v2 = b - a, c - a, projected - a
    d00, d01, d11 = v0 @ v0, v0 @ v1, v1 @ v1
    d20, d21 = v2 @ v0, v2 @ v1
    denominator = d00 * d11 - d01 * d01
    v = (d11 * d20 - d01 * d21) / denominator
    w = (d00 * d21 - d01 * d20) / denominator
    inside = (v >= 0) & (w >= 0) & (v + w <= 1)
    edges = np.minimum.reduce(
        [
            segment_distance(points, a, b),
            segment_distance(points, b, c),
            segment_distance(points, c, a),
        ]
    )
    return np.where(inside, np.abs(height), edges)


def signed_distance(points, triangles):
    """Signed distance to a closed, outward-wound triangle surface."""
    distances = np.array([point_triangle_distance(points, t) for t in triangles])
    closest = np.argmin(distances, axis=0)
    unsigned = distances[closest, np.arange(len(points))]
    sign = np.ones(len(points))
    for i, point in enumerate(points):
        best = np.flatnonzero(np.isclose(distances[:, i], unsigned[i], atol=1e-12))
        a, b, c = triangles[best[0]]
        normal = np.cross(b - a, c - a)
        sign[i] = 1.0 if (point - a) @ normal >= 0 else -1.0
    sign[unsigned < 1e-12] = 0.0
    return sign * unsigned


def dense_samples(triangles, divisions=48):
    """Points on a barycentric grid over every triangle."""
    samples = []
    for a, b, c in triangles:
        for i in range(divisions + 1):
            for j in range(divisions + 1 - i):
                u, v = i / divisions, j / divisions
                samples.append(a + u * (b - a) + v * (c - a))
    return np.array(samples)


def case(name, mesh_cells, mesh_z, apex_z):
    nodes, hexes = hex_grid(mesh_cells, mesh_z)
    a_triangles, a_samples, quads = mesh_surface(nodes, hexes)
    b_triangles, b_samples = cube_tessellation(apex_z)
    a_to_b = signed_distance(a_samples, b_triangles)
    b_to_a = signed_distance(b_samples, a_triangles)
    a_dense = dense_samples(a_triangles)
    b_dense = dense_samples(b_triangles)
    continuous_a = np.min(
        [point_triangle_distance(a_dense, t) for t in b_triangles], axis=0
    ).max()
    continuous_b = np.min(
        [point_triangle_distance(b_dense, t) for t in a_triangles], axis=0
    ).max()
    return {
        "name": name,
        "nodes": nodes,
        "quads": quads,
        "a_triangles": a_triangles,
        "a_samples": a_samples,
        "b_triangles": b_triangles,
        "b_samples": b_samples,
        "a_to_b": a_to_b,
        "b_to_a": b_to_a,
        "continuous_a_to_b": continuous_a,
        "continuous_b_to_a": continuous_b,
    }


def report(c):
    i = np.argmax(np.abs(c["a_to_b"]))
    j = np.argmax(np.abs(c["b_to_a"]))
    hh = max(abs(c["a_to_b"][i]), abs(c["b_to_a"][j]))
    h = max(c["continuous_a_to_b"], c["continuous_b_to_a"])
    print(
        f"{c['name']:20s} "
        f"sampled A->B {c['a_to_b'][i]:+.3f}  B->A {c['b_to_a'][j]:+.3f}  H_h {hh:.3f} | "
        f"continuous A->B {c['continuous_a_to_b']:.3f}  B->A {c['continuous_b_to_a']:.3f}  H {h:.3f}"
    )


def section(triangles):
    """Segments where the triangles cross the diagonal plane x = y."""
    segments = []
    for triangle in triangles:
        f = triangle[:, 0] - triangle[:, 1]
        points = []
        for p, q in ((0, 1), (1, 2), (2, 0)):
            fp, fq = f[p], f[q]
            if abs(fp) < 1e-12:
                points.append(triangle[p])
            if fp * fq < 0:
                t = fp / (fp - fq)
                points.append(triangle[p] + t * (triangle[q] - triangle[p]))
        unique = []
        for point in points:
            if not any(np.allclose(point, u) for u in unique):
                unique.append(point)
        if len(unique) == 2:
            segments.append(unique)
    return [
        [(np.hypot(p[0], p[1]), p[2]) for p in segment] for segment in segments
    ]


ELEVATION = 20
AZIMUTH = -60
EYE = np.array(
    [
        np.cos(np.radians(ELEVATION)) * np.cos(np.radians(AZIMUTH)),
        np.cos(np.radians(ELEVATION)) * np.sin(np.radians(AZIMUTH)),
        np.sin(np.radians(ELEVATION)),
    ]
)


def front(face):
    """True when the outward (Newell) normal of a planar or warped polygon faces the viewer."""
    normal = np.zeros(3)
    for p, q in zip(face, np.roll(face, -1, axis=0)):
        normal += np.cross(p, q)
    return normal @ EYE > 0


def key(point):
    return tuple(np.round(point, 9))


def edge_visibility(faces):
    """Splits polygon edges into visible and hidden, by the faces that share them."""
    edges = {}
    for face in faces:
        facing = front(face)
        for p, q in zip(face, np.roll(face, -1, axis=0)):
            k = tuple(sorted((key(p), key(q))))
            edges.setdefault(k, [p, q, False])
            edges[k][2] |= facing
    visible = [[p, q] for p, q, seen in edges.values() if seen]
    hidden = [[p, q] for p, q, seen in edges.values() if not seen]
    return visible, hidden, set(edges)


def lines(ax, segments, **style):
    if segments:
        ax.add_collection3d(Line3DCollection(segments, **style))


def draw_surface(ax, outlines, triangles, samples, color, marker, title):
    """Draws a closed surface with hidden lines dashed and light.

    Outlines are the element faces (quadrilaterals or triangles).  Triangle
    edges that are not outline edges are the quadrilateral splits, drawn thin.
    """
    visible, hidden, outline_keys = edge_visibility(outlines)
    split_visible, split_hidden, _ = edge_visibility(triangles)
    split_visible = [e for e in split_visible if tuple(sorted(map(key, e))) not in outline_keys]
    split_hidden = [e for e in split_hidden if tuple(sorted(map(key, e))) not in outline_keys]
    dashed = (0, (3, 2))
    lines(ax, hidden, colors=color, linewidths=0.6, alpha=0.4, linestyles=dashed)
    lines(ax, split_hidden, colors=color, linewidths=0.4, alpha=0.25, linestyles=dashed)
    ax.add_collection3d(
        Poly3DCollection(
            [t for t in triangles if front(t)], facecolor=color, alpha=0.12, edgecolor="none"
        )
    )
    lines(ax, split_visible, colors=color, linewidths=0.5, alpha=0.7)
    lines(ax, visible, colors=color, linewidths=1.2)
    shown = {key(p) for face in outlines if front(face) for p in face}
    seen = np.array([key(p) in shown for p in samples])
    ax.scatter(*samples[~seen].T, color=color, s=8, marker=marker, alpha=0.35, depthshade=False)
    ax.scatter(*samples[seen].T, color=color, s=10, marker=marker, depthshade=False)
    ax.set_xlim(0, 1)
    ax.set_ylim(0, 1)
    ax.set_zlim(0, 1.3)
    ax.set_box_aspect((1, 1, 1.3))
    ax.view_init(elev=ELEVATION, azim=AZIMUTH)
    ax.set_axis_off()
    ax.set_title(title, fontsize=10, color=INK, y=0.98)


def draw_section(ax, c, missed):
    for segment in section(c["a_triangles"]):
        (s0, z0), (s1, z1) = segment
        ax.plot([s0, s1], [z0, z1], color=MESH, linewidth=2)
    for segment in section(c["b_triangles"]):
        (s0, z0), (s1, z1) = segment
        ax.plot([s0, s1], [z0, z1], color=TESSELLATION, linewidth=2, linestyle="--")
    on = lambda p: np.isclose(p[:, 0], p[:, 1])
    a = c["a_samples"][on(c["a_samples"])]
    b = c["b_samples"][on(c["b_samples"])]
    ax.scatter(np.hypot(a[:, 0], a[:, 1]), a[:, 2], color=MESH, s=24, zorder=3)
    ax.scatter(
        np.hypot(b[:, 0], b[:, 1]), b[:, 2], color=TESSELLATION, s=24, marker="D", zorder=3
    )
    r = np.sqrt(0.5)
    ax.annotate(
        "",
        xy=(r, 1.0),
        xytext=(r, 1.3),
        arrowprops=dict(arrowstyle="<->", color=INK, linewidth=1),
    )
    ax.text(r - 0.04, 1.09, "0.3, sampled", fontsize=8, color=INK, va="center", ha="right")
    if missed is not None:
        z, text = missed
        ax.scatter([r], [z], facecolor="white", edgecolor=INK, s=30, zorder=4)
        ax.text(r, z - 0.1, text, fontsize=8, color=MUTED, va="top", ha="center")
    ax.set_xlim(-0.08, np.sqrt(2) + 0.08)
    ax.set_ylim(-0.08, 1.42)
    ax.set_aspect("equal")
    ax.set_xticks([0, r, np.sqrt(2)], ["0", r"$\sqrt{2}/2$", r"$\sqrt{2}$"])
    ax.set_yticks([0, 1, 1.3])
    ax.tick_params(labelsize=7, colors=MUTED)
    for spine in ("top", "right"):
        ax.spines[spine].set_visible(False)
    for spine in ("left", "bottom"):
        ax.spines[spine].set_color(MUTED)
    ax.set_xlabel("distance along the diagonal $x = y$", fontsize=8, color=MUTED)
    ax.set_ylabel("$z$", fontsize=8, color=MUTED)
    ax.set_title("section on the plane $x = y$", fontsize=10, color=INK)


def main():
    cases = [
        case("tessellation spike", 1, None, 1.0 + OFFSET),
        case("tessellation dimple", 1, None, 1.0 - OFFSET),
        case("mesh spike", 2, 1.0 + OFFSET, None),
        case("mesh dimple", 2, 1.0 - OFFSET, None),
    ]
    for c in cases:
        report(c)

    spike_t, spike_m = cases[0], cases[2]
    figure = plt.figure(figsize=(10, 7), facecolor="white")
    rows = [
        (spike_t, "tessellation spike", (1.0, f"{spike_t['continuous_a_to_b']:.3f} to B, not sampled")),
        (spike_m, "mesh spike", (1.0, f"{spike_m['continuous_b_to_a']:.3f} to A, not sampled")),
    ]
    for row, (c, label, missed) in enumerate(rows):
        draw_surface(
            figure.add_subplot(2, 3, 3 * row + 1, projection="3d", computed_zorder=False),
            [c["nodes"][q] for q in c["quads"]],
            c["a_triangles"],
            c["a_samples"],
            MESH,
            "o",
            f"{label}: mesh A",
        )
        draw_surface(
            figure.add_subplot(2, 3, 3 * row + 2, projection="3d", computed_zorder=False),
            c["b_triangles"],
            c["b_triangles"],
            c["b_samples"],
            TESSELLATION,
            "D",
            f"{label}: tessellation B",
        )
        draw_section(figure.add_subplot(2, 3, 3 * row + 3), c, missed)
    handles = [
        plt.Line2D([], [], color=MESH, linewidth=2, marker="o", markersize=5, label="mesh A, boundary nodes"),
        plt.Line2D([], [], color=TESSELLATION, linewidth=2, linestyle="--", marker="D", markersize=5, label="tessellation B, vertices"),
        plt.Line2D([], [], color=INK, linestyle="none", marker="o", markerfacecolor="white", markersize=6, label="worst point, not a sample"),
    ]
    figure.legend(handles=handles, loc="lower center", ncol=3, frameon=False, fontsize=9)
    figure.tight_layout(rect=(0, 0.05, 1, 1), h_pad=2.5)
    figure.savefig(HERE / "hausdorff_cases.png", dpi=150)


if __name__ == "__main__":
    main()
