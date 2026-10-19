"""Schematic diagrams for the Hausdorff Distance page.

Writes seven figures, each a 2D schematic:

* hausdorff_circles.png: the two directed distances on a circle with a bulge;
* hausdorff_circles_discrete.png: the same circles as polygons, with the
  signed distance at every sample;
* hausdorff_sign.png: the sign convention, and the pseudonormal at a sharp tip;
* hausdorff_split.png: the shorter-diagonal split of a quadrilateral, with a tie;
* hausdorff_surface_checks.png: the closed, consistently wound, and outward checks;
* hausdorff_inverted.png: the inverted element cases, drawn with quadrilaterals
  as the 2D analogue of hexahedra;
* hausdorff_frame.png: a mesh compared with its tessellation in mismatched and
  matched frames.

Run: uv run --with numpy --with matplotlib python hausdorff_diagrams.py
"""

from pathlib import Path

import matplotlib.pyplot as plt
from matplotlib.path import Path as Polygon
import numpy as np

HERE = Path(__file__).parent
MESH = "#2a78d6"
TESSELLATION = "#eb6834"
FLAG = "#e34948"
INK = "#2b2b2b"
MUTED = "#6b6b6b"
FILL = 0.12


def cross(u, v):
    return u[0] * v[1] - u[1] * v[0]


def blank(ax, title):
    ax.set_aspect("equal")
    ax.set_axis_off()
    ax.set_title(title, fontsize=10, color=INK)


def gridded(ax, title, xticks, yticks):
    ax.set_aspect("equal")
    ax.set_title(title, fontsize=10, color=INK)
    ax.set_xticks(xticks)
    ax.set_yticks(yticks)
    ax.grid(color="#dddddd", linewidth=0.6)
    ax.set_axisbelow(True)
    ax.set_xlabel("x", fontsize=9, color=MUTED)
    ax.set_ylabel("y", fontsize=9, color=MUTED)
    ax.tick_params(labelsize=8, colors=MUTED)
    for spine in ax.spines.values():
        spine.set_color("#bbbbbb")


def arrow(ax, start, end, color=INK, width=1.2, style="-|>", linestyle="-"):
    ax.annotate(
        "",
        xy=end,
        xytext=start,
        arrowprops=dict(
            arrowstyle=style, color=color, linewidth=width, linestyle=linestyle,
            shrinkA=0, shrinkB=0, mutation_scale=10,
        ),
    )


def note(ax, xy, text, color=INK, **kwargs):
    kwargs.setdefault("fontsize", 8.5)
    ax.text(*xy, text, color=color, **kwargs)


def sign_figure():
    figure, (left, right) = plt.subplots(1, 2, figsize=(10, 4.2), facecolor="white")

    face = np.array([[-1.6, 0.0], [1.6, 0.0]])
    left.fill_between(face[:, 0], -1.3, 0.0, color=TESSELLATION, alpha=FILL, linewidth=0)
    left.plot(*face.T, color=TESSELLATION, linewidth=2)
    note(left, (-1.55, -1.15), "inside", MUTED)
    note(left, (-1.55, 1.1), "outside", MUTED)
    arrow(left, (-1.1, 0.0), (-1.1, 0.55), TESSELLATION)
    note(left, (-1.03, 0.4), r"$\mathbf{n}$", TESSELLATION, fontsize=11)
    for x, label in (((0.2, 0.9), r"$s = +d$"), ((0.9, -0.8), r"$s = -d$")):
        p = (x[0], 0.0)
        left.plot(*zip(x, p), color=INK, linewidth=0.8, linestyle=":")
        left.scatter(*x, color=MESH, s=36, zorder=3)
        left.scatter(*p, facecolor="white", edgecolor=INK, s=28, zorder=3)
        note(left, (x[0] + 0.1, x[1]), r"$\mathbf{x}$", INK, fontsize=11, va="center")
        note(left, (p[0] + 0.08, 0.08 if x[1] > 0 else -0.2), r"$\mathbf{p}$", INK, fontsize=11)
        note(left, (x[0] + 0.1, x[1] / 2), label, INK, va="center")
    left.set_xlim(-1.7, 1.7)
    left.set_ylim(-1.3, 1.3)
    blank(left, "sign from the face normal")

    half = np.radians(20)
    upper = np.array([-np.cos(half), np.sin(half)]) * 2.2
    lower = np.array([-np.cos(half), -np.sin(half)]) * 2.2
    tip = np.zeros(2)
    right.fill([tip[0], upper[0], lower[0]], [tip[1], upper[1], lower[1]], color=TESSELLATION, alpha=FILL, linewidth=0)
    right.plot([upper[0], 0, lower[0]], [upper[1], 0, lower[1]], color=TESSELLATION, linewidth=2)
    n_upper = np.array([np.sin(half), np.cos(half)])
    n_lower = np.array([np.sin(half), -np.cos(half)])
    for n, at, label in ((n_upper, upper / 2, r"$\mathbf{n}_1$"), (n_lower, lower / 2, r"$\mathbf{n}_2$")):
        arrow(right, at, at + 0.55 * n, TESSELLATION)
        note(right, at + 0.62 * n, label, TESSELLATION, fontsize=11, ha="center", va="center")
    x = np.array([0.35, -0.6])
    right.plot(*zip(x, tip), color=INK, linewidth=0.8, linestyle=":")
    right.scatter(*x, color=MESH, s=36, zorder=3)
    right.scatter(*tip, facecolor="white", edgecolor=INK, s=28, zorder=3)
    note(right, x + (0.08, -0.05), r"$\mathbf{x}$ (outside)", INK, fontsize=10, va="top")
    note(right, (0.08, 0.1), r"$\mathbf{p}$ (the tip)", INK, fontsize=10)
    arrow(right, tip, (0.6, 0.0), INK, linestyle="--")
    note(right, (0.66, 0.0), "pseudonormal", MUTED, va="center")
    d = x - tip
    note(
        right,
        (-2.15, -1.3),
        rf"$(\mathbf{{x}} - \mathbf{{p}}) \cdot \mathbf{{n}}_1 = {d @ n_upper:+.2f}$: wrong sign"
        "\n"
        rf"$(\mathbf{{x}} - \mathbf{{p}}) \cdot \mathbf{{n}}_2 = {d @ n_lower:+.2f}$: right sign",
        INK,
        va="top",
    )
    right.set_xlim(-2.2, 1.6)
    right.set_ylim(-1.75, 1.0)
    blank(right, "a sharp convex tip")
    figure.tight_layout()
    figure.savefig(HERE / "hausdorff_sign.png", dpi=150)


def split_panel(ax, quad, title):
    n0, n1, n2, n3 = quad
    first, second = np.linalg.norm(n1 - n3), np.linalg.norm(n0 - n2)
    if first < second:
        chosen, other = (n1, n3), (n0, n2)
        triangles = ((n0, n1, n3), (n1, n2, n3))
        names = (r"$[n_0, n_1, n_3]$", r"$[n_1, n_2, n_3]$")
    else:
        chosen, other = (n0, n2), (n1, n3)
        triangles = ((n0, n1, n2), (n0, n2, n3))
        names = (r"$[n_0, n_1, n_2]$", r"$[n_0, n_2, n_3]$")
    for corners, alpha in zip(triangles, (FILL, 2 * FILL)):
        ax.fill(*np.array(corners).T, color=MESH, alpha=alpha, linewidth=0)
    for corners, name in zip(triangles, names):
        centroid = np.mean(corners, axis=0)
        direction = (other[1] - other[0]) / np.linalg.norm(other[1] - other[0])
        offset = centroid - other[0]
        side = offset - (offset @ direction) * direction
        at = centroid + 0.22 * side / np.linalg.norm(side)
        note(ax, at, name, MESH, ha="center", va="center", fontsize=8)
    ax.plot(*np.vstack([quad, quad[:1]]).T, color=MESH, linewidth=1.8)
    ax.plot(*np.array(chosen).T, color=MESH, linewidth=1.8)
    ax.plot(*np.array(other).T, color=MUTED, linewidth=1.0, linestyle="--")
    center = quad.mean(axis=0)
    for k, point in enumerate(quad):
        ax.scatter(*point, color=MESH, s=18, zorder=3)
        outward = (point - center) / np.linalg.norm(point - center)
        note(ax, point + 0.18 * outward, rf"$n_{k}$", INK, ha="center", va="center", fontsize=10)
    note(
        ax,
        (center[0], quad[:, 1].min() - 0.35),
        rf"$\| x_{{n_1}} - x_{{n_3}} \| = {first:.2f}$"
        "\n"
        rf"$\| x_{{n_0}} - x_{{n_2}} \| = {second:.2f}$",
        INK,
        ha="center",
        va="top",
    )
    ax.set_xlim(quad[:, 0].min() - 0.5, quad[:, 0].max() + 0.5)
    ax.set_ylim(quad[:, 1].min() - 0.85, quad[:, 1].max() + 0.4)
    blank(ax, title)


def split_figure():
    figure, axes = plt.subplots(1, 3, figsize=(11, 3.8), facecolor="white")
    quads = [
        (np.array([[0.0, 0.0], [1.6, -0.1], [2.4, 1.3], [0.4, 1.1]]), r"$n_1$–$n_3$ shorter"),
        (np.array([[0.0, 0.0], [2.0, 0.1], [1.7, 1.2], [-0.5, 1.0]]), r"$n_0$–$n_2$ shorter"),
        (np.array([[0.0, 0.0], [1.8, 0.0], [1.4, 1.2], [0.4, 1.2]]), r"tie: $n_0$–$n_2$"),
    ]
    for ax, (quad, title) in zip(axes, quads):
        split_panel(ax, quad, title)
    figure.tight_layout()
    figure.savefig(HERE / "hausdorff_split.png", dpi=150)


def winding_arrows(ax, triangle, color):
    """Arrows along each edge in the triangle's node order, set just inside the triangle."""
    centroid = triangle.mean(axis=0)
    for a, b in zip(triangle, np.roll(triangle, -1, axis=0)):
        mid = (a + b) / 2
        inward = centroid - mid
        mid = mid + 0.22 * inward
        direction = (b - a) / np.linalg.norm(b - a)
        arrow(ax, mid - 0.09 * direction, mid + 0.09 * direction, color, width=1.0)


def page_normal(ax, triangle, color):
    """Out of the page for counterclockwise order, into the page for clockwise."""
    a, b, c = triangle
    ccw = cross(b - a, c - a) > 0
    centroid = triangle.mean(axis=0)
    ax.scatter(*centroid, facecolor="white", edgecolor=color, s=70, zorder=3)
    ax.scatter(*centroid, color=color, s=20 if ccw else 30, marker="." if ccw else "x", zorder=4)


def triangle(ax, points, color=TESSELLATION, edge=None, linestyle="-", alpha=FILL):
    ax.fill(*points.T, color=color, alpha=alpha, linewidth=0)
    closed = np.vstack([points, points[:1]])
    ax.plot(*closed.T, color=edge or color, linewidth=1.5, linestyle=linestyle)


def surface_checks_figure():
    figure, axes = plt.subplots(1, 5, figsize=(13, 3.4), facecolor="white")
    p0, p1 = np.array([0.0, 0.0]), np.array([0.0, 1.0])
    left_vertex, right_vertex = np.array([-0.9, 0.5]), np.array([0.9, 0.5])

    ax = axes[0]
    for t in (np.array([p0, p1, left_vertex]), np.array([p1, p0, right_vertex])):
        triangle(ax, t)
        winding_arrows(ax, t, INK)
        page_normal(ax, t, INK)
    ax.plot(*zip(p0, p1), color=INK, linewidth=2.2)
    note(ax, (0, -0.25), "shared edge traversed\nin opposite directions", INK, ha="center", va="top")
    blank(ax, "passes")

    ax = axes[1]
    gap = np.array([0.12, 0.0])
    for t in (np.array([p0, p1, left_vertex]), np.array([p1 + gap, p0 + gap, right_vertex + gap])):
        triangle(ax, t)
    ax.plot(*zip(p0, p1), color=FLAG, linewidth=2.2)
    ax.plot(*zip(p0 + gap, p1 + gap), color=FLAG, linewidth=2.2)
    note(ax, (0.06, -0.25), "vertices differ by round-off:\ntwo edges, one triangle each", INK, ha="center", va="top")
    blank(ax, "fails closed: a hole")

    ax = axes[2]
    for t in (np.array([p0, p1, left_vertex]), np.array([p1, p0, right_vertex])):
        triangle(ax, t)
    triangle(ax, np.array([p0, p1, np.array([0.45, 1.45])]), TESSELLATION, FLAG, "--", 0.05)
    ax.plot(*zip(p0, p1), color=FLAG, linewidth=2.2)
    note(ax, (0, -0.25), "three triangles\non one edge", INK, ha="center", va="top")
    blank(ax, "fails closed: non-manifold")

    ax = axes[3]
    for t in (np.array([p0, p1, left_vertex]), np.array([p0, p1, right_vertex])):
        triangle(ax, t)
        winding_arrows(ax, t, INK)
        page_normal(ax, t, INK if cross(t[1] - t[0], t[2] - t[0]) > 0 else FLAG)
    ax.plot(*zip(p0, p1), color=FLAG, linewidth=2.2)
    note(ax, (0, -0.25), "shared edge traversed\nin the same direction", INK, ha="center", va="top")
    blank(ax, "fails consistently wound")

    ax = axes[4]
    angles = np.linspace(0, 2 * np.pi, 9)[:-1]
    ring = 0.6 * np.column_stack([np.cos(angles), np.sin(angles)]) + (0, 0.5)
    ax.fill(*ring.T, color=TESSELLATION, alpha=FILL, linewidth=0)
    ax.plot(*np.vstack([ring, ring[:1]]).T, color=TESSELLATION, linewidth=1.5)
    for a, b in zip(ring, np.roll(ring, -1, axis=0)):
        mid = (a + b) / 2
        inward = (np.array([0, 0.5]) - mid) / np.linalg.norm(np.array([0, 0.5]) - mid)
        arrow(ax, mid, mid + 0.25 * inward, FLAG, width=1.0)
    note(ax, (0, -0.25), "every normal points in:\nnegative enclosed volume", INK, ha="center", va="top")
    blank(ax, "fails outward: inside out")

    for ax in axes:
        ax.set_xlim(-1.05, 1.05)
        ax.set_ylim(-0.75, 1.55)
    figure.tight_layout()
    figure.savefig(HERE / "hausdorff_surface_checks.png", dpi=150)


def quad_patch(moved=None):
    nodes = {(i, j): np.array([i / 2, j / 2]) for i in range(3) for j in range(3)}
    if moved is not None:
        nodes[(2, 2)] = np.array(moved)
    elements = [
        [(i, j), (i + 1, j), (i + 1, j + 1), (i, j + 1)] for j in range(2) for i in range(2)
    ]
    return nodes, elements


def corner_jacobian(points, k):
    here = points[k]
    after = points[(k + 1) % 4] - here
    before = points[(k - 1) % 4] - here
    return cross(after, before) / (np.linalg.norm(after) * np.linalg.norm(before))


def inverted_panel(ax, title, outcome, mirrored=(), moved=None):
    nodes, elements = quad_patch(moved)
    for e, element in enumerate(elements):
        order = element[::-1] if e in mirrored else element
        points = np.array([nodes[n] for n in order])
        flagged = e in mirrored
        ax.fill(*points.T, color=FLAG if flagged else MESH, alpha=FILL, linewidth=0)
        ax.plot(*np.vstack([points, points[:1]]).T, color=MESH, linewidth=1.5)
        winding_arrows(ax, points, INK)
        for a, b, na, nb in zip(points, np.roll(points, -1, axis=0), order, order[1:] + order[:1]):
            on_boundary = any(
                all(n[axis] == side for n in (na, nb)) for axis in (0, 1) for side in (0, 2)
            )
            if on_boundary:
                d = (b - a) / np.linalg.norm(b - a)
                normal = np.array([d[1], -d[0]])
                mid = (a + b) / 2
                if flagged:
                    arrow(ax, mid - 0.2 * normal, mid, FLAG, width=1.1)
                else:
                    arrow(ax, mid, mid + 0.18 * normal, MESH, width=1.1)
        if not flagged:
            for k in range(4):
                if corner_jacobian(points, k) < 0:
                    ax.scatter(*points[k], color=FLAG, s=40, zorder=4)
                    note(ax, points[k] + (0.42, -0.2), rf"$\hat{{J}} = {corner_jacobian(points, k):.2f}$", FLAG, va="center")
    note(ax, (0.5, -0.33), outcome, INK, ha="center", va="top")
    ax.set_xlim(-0.3, 1.45)
    ax.set_ylim(-0.62, 1.3)
    blank(ax, title)


def inverted_figure():
    figure, axes = plt.subplots(1, 4, figsize=(12, 3.7), facecolor="white")
    inverted_panel(axes[0], "right-handed elements", "outward normals:\naccepted")
    inverted_panel(axes[1], "mirrored mesh", "every normal points in:\nrejected (outward check)", mirrored=(0, 1, 2, 3))
    inverted_panel(axes[2], "mirrored boundary element", "normals disagree:\nrejected (winding check)", mirrored=(3,))
    inverted_panel(axes[3], "tangled boundary element", "consistent and outward, but $\\hat{J} < 0$ at a corner:\naccepted, with a count", moved=(0.65, 0.65))
    figure.tight_layout()
    figure.savefig(HERE / "hausdorff_inverted.png", dpi=150)


def ellipse(count, scale, translate):
    t = np.linspace(0, 2 * np.pi, count + 1)
    return np.column_stack([np.cos(t), np.sin(t)]) * scale + translate


def polyline_distance(points, polyline):
    best = np.full(len(points), np.inf)
    for a, b in zip(polyline[:-1], polyline[1:]):
        ab = b - a
        t = np.clip(((points - a) @ ab) / (ab @ ab), 0, 1)
        best = np.minimum(best, np.linalg.norm(points - (a + t[:, None] * ab), axis=1))
    return best


def dense(polyline, per_edge=40):
    t = np.linspace(0, 1, per_edge, endpoint=False)[:, None]
    return np.vstack([a + t * (b - a) for a, b in zip(polyline[:-1], polyline[1:])])


def hausdorff(a, b):
    return max(polyline_distance(dense(a), b).max(), polyline_distance(dense(b), a).max())


def closest(point, polyline):
    best, at = np.inf, None
    for a, b in zip(polyline[:-1], polyline[1:]):
        ab = b - a
        t = np.clip((point - a) @ ab / (ab @ ab), 0, 1)
        candidate = a + t * ab
        distance = np.linalg.norm(point - candidate)
        if distance < best:
            best, at = distance, candidate
    return at, best


def bulged(radius, bump, width, count=720):
    theta = np.linspace(0, 2 * np.pi, count + 1)
    away = np.abs(np.angle(np.exp(1j * (theta - np.pi / 2))))
    r = radius + bump * np.exp(-((away / width) ** 2))
    return np.column_stack([r * np.cos(theta), r * np.sin(theta)])


def directed_arrow(ax, curve, other, color, label, at, ha):
    distances = polyline_distance(curve, other)
    worst = int(np.argmax(distances))
    p, d = closest(curve[worst], other)
    arrow(ax, curve[worst], p, color, width=1.4)
    middle = (curve[worst] + p) / 2
    ax.plot(*zip(middle, at), color=color, linewidth=0.6)
    note(ax, at, label.format(d=d), color, ha=ha, va="center", fontsize=10)
    return d


def circles_figure():
    width = np.radians(12)
    figure, axes = plt.subplots(1, 2, figsize=(10, 5.2), facecolor="white")
    inner, outer = (-0.65, 0.4), (-1.3, 0.95)
    cases = [
        (ellipse(720, np.ones(2), np.zeros(2)), bulged(1.1, 0.4, width), "tessellation bulges out", inner, outer),
        (bulged(1.1, 0.4, width), ellipse(720, np.ones(2), np.zeros(2)), "mesh bulges out", outer, inner),
    ]
    values = []
    for ax, (a, b, title, a_label, b_label) in zip(axes, cases):
        ax.plot(*b.T, color=TESSELLATION, linewidth=2, solid_capstyle="round")
        ax.plot(*a.T, color=MESH, linewidth=2, solid_capstyle="round")
        h_ab = directed_arrow(ax, a, b, MESH, r"$h(\mathcal{{A}} \rightarrow \mathcal{{B}}) = {d:.2f}$", (0.75, 1.55), "left")
        h_ba = directed_arrow(ax, b, a, TESSELLATION, r"$h(\mathcal{{B}} \rightarrow \mathcal{{A}}) = {d:.2f}$", (-0.75, 1.55), "right")
        values.append((h_ab, h_ba))
        note(ax, (0.0, -1.45), rf"$H = \max({h_ab:.2f},\ {h_ba:.2f}) = {max(h_ab, h_ba):.2f}$", INK, ha="center", va="top", fontsize=10)
        note(ax, a_label, r"$\mathcal{A}$", MESH, fontsize=12, ha="center", va="center")
        note(ax, (-b_label[0], -b_label[1]), r"$\mathcal{B}$", TESSELLATION, fontsize=12, ha="center", va="center")
        ax.set_xlim(-2.3, 2.3)
        ax.set_ylim(-1.8, 1.9)
        ticks = np.arange(-2.0, 2.01, 0.5)
        gridded(ax, title, ticks, ticks[ticks <= 1.5])
    figure.tight_layout()
    figure.savefig(HERE / "hausdorff_circles.png", dpi=150, bbox_inches="tight")
    for (h_ab, h_ba), (_, _, title, _, _) in zip(values, cases):
        print(f"circles, {title}: h(A->B) = {h_ab:.3f}, h(B->A) = {h_ba:.3f}")


def polygon(radius, bump, width, step):
    theta = np.radians(np.arange(0, 360, step))
    away = np.abs(np.angle(np.exp(1j * (theta - np.pi / 2))))
    r = radius + bump * np.exp(-((away / width) ** 2))
    points = np.column_stack([r * np.cos(theta), r * np.sin(theta)])
    return np.vstack([points, points[:1]]), np.degrees(theta)


def signed(samples, other):
    distances = polyline_distance(samples, other)
    inside = Polygon(other).contains_points(samples)
    return np.where(inside, -distances, distances)


def discrete_circles_figure():
    width = np.radians(12)
    unit, unit_angles = polygon(1.0, 0.0, width, 15)
    bump, bump_angles = polygon(1.1, 0.4, width, 10)
    cases = [
        ((unit, unit_angles), (bump, bump_angles), "tessellation bulges out"),
        ((bump, bump_angles), (unit, unit_angles), "mesh bulges out"),
    ]
    figure, axes = plt.subplots(2, 2, figsize=(10, 8.6), facecolor="white", gridspec_kw={"height_ratios": [1.25, 1]})
    for column, ((a, a_angles), (b, b_angles), title) in enumerate(cases):
        top, bottom = axes[0, column], axes[1, column]
        s_i = signed(a[:-1], b)
        s_j = signed(b[:-1], a)
        top.plot(*b.T, color=TESSELLATION, linewidth=1.6)
        top.plot(*a.T, color=MESH, linewidth=1.6)
        top.scatter(*b[:-1].T, color=TESSELLATION, marker="s", s=14, zorder=3)
        top.scatter(*a[:-1].T, color=MESH, s=14, zorder=3)
        i, j = int(np.argmax(np.abs(s_i))), int(np.argmax(np.abs(s_j)))
        p, _ = closest(a[i], b)
        q, _ = closest(b[j], a)
        arrow(top, a[i], p, MESH, width=1.3)
        arrow(top, b[j], q, TESSELLATION, width=1.3)
        for start, end, value, color, at, ha in (
            (a[i], p, s_i[i], MESH, (0.7, 1.55), "left"),
            (b[j], q, s_j[j], TESSELLATION, (-0.7, 1.55), "right"),
        ):
            top.plot(*zip((start + end) / 2, at), color=color, linewidth=0.6)
            note(top, at, rf"$s = {value:+.2f}$", color, ha=ha, va="center", fontsize=10)
        top.set_xlim(-2.3, 2.3)
        top.set_ylim(-1.5, 1.9)
        ticks = np.arange(-2.0, 2.01, 0.5)
        gridded(top, title, ticks, ticks[(ticks >= -1.5) & (ticks <= 1.5)])

        bottom.axhline(0.0, color=MUTED, linewidth=0.8)
        bottom.vlines(a_angles, 0, s_i, color=MESH, linewidth=1.2)
        bottom.scatter(a_angles, s_i, color=MESH, s=16, zorder=3, label=r"$s_i$, nodes of $\mathcal{A}_h$")
        bottom.vlines(b_angles + 2, 0, s_j, color=TESSELLATION, linewidth=1.2)
        bottom.scatter(b_angles + 2, s_j, color=TESSELLATION, marker="s", s=16, zorder=3, label=r"$s_j$, vertices of $\mathcal{B}_h$")
        h_ab, h_ba = np.abs(s_i).max(), np.abs(s_j).max()
        note(bottom, (365, -0.56), rf"$h_h(\mathcal{{A}} \rightarrow \mathcal{{B}}) = {h_ab:.2f}$", MESH, ha="right", va="bottom", fontsize=9)
        note(bottom, (365, -0.68), rf"$h_h(\mathcal{{B}} \rightarrow \mathcal{{A}}) = {h_ba:.2f}$", TESSELLATION, ha="right", va="bottom", fontsize=9)
        note(bottom, (365, -0.80), rf"$H_h = {max(h_ab, h_ba):.2f}$", INK, ha="right", va="bottom", fontsize=9)
        bottom.set_xlim(-10, 370)
        bottom.set_ylim(-0.9, 0.65)
        bottom.set_xticks(range(0, 361, 90))
        profile_axes(bottom, "angle of the sample (degrees)", "signed distance $s$")
        bottom.legend(fontsize=8, loc="lower left", frameon=False)
        print(f"discrete, {title}: s_i in [{s_i.min():+.3f}, {s_i.max():+.3f}], s_j in [{s_j.min():+.3f}, {s_j.max():+.3f}], h_h(A->B) = {h_ab:.3f}, h_h(B->A) = {h_ba:.3f}")
    figure.tight_layout()
    figure.savefig(HERE / "hausdorff_circles_discrete.png", dpi=150, bbox_inches="tight")


def profile_axes(ax, xlabel, ylabel):
    ax.set_xlabel(xlabel, fontsize=9, color=MUTED)
    ax.set_ylabel(ylabel, fontsize=9, color=MUTED)
    ax.tick_params(labelsize=8, colors=MUTED)
    ax.grid(color="#dddddd", linewidth=0.6)
    ax.set_axisbelow(True)
    for spine in ax.spines.values():
        spine.set_color("#bbbbbb")


def frame_figure():
    scale, translate = np.array([2.0, 1.0]), np.array([1.0, 0.0])
    mesh = ellipse(16, scale, translate)
    raw = ellipse(64, np.ones(2), np.zeros(2))
    transformed = ellipse(64, scale, translate)
    figure, axes = plt.subplots(1, 2, figsize=(10, 3.6), facecolor="white")
    panels = [
        (axes[0], raw, "mismatched frames", "surface.stl, as read"),
        (axes[1], transformed, "matched frames", "surface_scale_2_1_1_translate_1_0_0.stl"),
    ]
    for ax, surface, title, label in panels:
        ax.fill(*mesh.T, color=MESH, alpha=FILL, linewidth=0)
        ax.plot(*mesh.T, color=MESH, linewidth=1.8)
        ax.scatter(*mesh[:-1].T, color=MESH, s=12, zorder=3)
        ax.plot(*surface.T, color=TESSELLATION, linewidth=1.8, linestyle="--")
        note(ax, (1.0, -1.35), rf"$H = {hausdorff(mesh, surface):.3f}$", INK, ha="center", va="top", fontsize=10)
        note(ax, (1.0, 1.25), "mesh: --xscale 2 --xtranslate 1", MESH, ha="center")
        note(ax, (1.0, -1.15), label, TESSELLATION, ha="center", va="top")
        ax.set_xlim(-1.3, 3.3)
        ax.set_ylim(-1.75, 1.5)
        if surface is raw:
            arrow(ax, (1.0, 0.0), (3.0, 0.0), INK, width=1.0, style="<|-|>")
            note(ax, (2.0, 0.08), r"$H = 2$", INK, ha="center", va="bottom", fontsize=9)
        gridded(ax, title, range(-1, 4), range(-1, 2))
    figure.tight_layout()
    figure.savefig(HERE / "hausdorff_frame.png", dpi=150)
    print(f"frame: mismatched H = {hausdorff(mesh, raw):.3f}, matched H = {hausdorff(mesh, transformed):.3f}")


def main():
    circles_figure()
    discrete_circles_figure()
    sign_figure()
    split_figure()
    surface_checks_figure()
    inverted_figure()
    frame_figure()


if __name__ == "__main__":
    main()
