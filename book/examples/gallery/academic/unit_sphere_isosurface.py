#!/usr/bin/env python3
"""Isosurfaces of the sphere segmentations, by marching cubes.

For each segmentation `unit_sphere_nNNN.npy` of radius `n` voxels, the script

1. pads the segmentation by one voxel of zeros, so the surface closes,
2. runs `skimage.measure.marching_cubes` with the Lewiner method at level 0.5,
3. reverses each face, so every triangle winds outward,
4. translates by -(n + 1), which moves the sphere center to the origin,
5. scales by 1/n, which makes the radius 1, and
6. writes a binary STL.

It then checks each surface and prints two tables.  The first gives the
topology, orientation, and radius of each surface.  The second compares the
Lewiner method with the original Lorensen method, for speed and for the mesh
each one returns.

Example
-------
cd ~/autotwin/automesh/book/examples/gallery/academic
uv run --with numpy unit_sphere_segmentation.py
uv run --with numpy --with scikit-image unit_sphere_isosurface.py

Output
------
unit_sphere_mc_n010.stl, unit_sphere_mc_n020.stl, unit_sphere_mc_n040.stl,
unit_sphere_mc_n080.stl, unit_sphere_mc_n160.stl, and two tables on the
terminal.
"""

import time
from collections import Counter
from pathlib import Path

import numpy as np
from skimage.measure import marching_cubes

RADII = (10, 20, 40, 80, 160)
LEVEL = 0.5
PAD = 1
REPEATS = 5


def isosurface(*, voxels: np.ndarray, method: str = "lewiner") -> tuple:
    """Returns the vertices and outward-wound faces, in voxel units."""
    padded = np.pad(voxels, PAD)
    vertices, faces, _, _ = marching_cubes(padded, level=LEVEL, method=method)
    # skimage's default winding is inward, opposite to its returned normals.
    return vertices, faces[:, ::-1]


def edge_counts(*, faces: np.ndarray) -> Counter:
    """Returns how many faces use each undirected edge."""
    edges = np.vstack([faces[:, [0, 1]], faces[:, [1, 2]], faces[:, [2, 0]]])
    return Counter(map(tuple, np.sort(edges, axis=1)))


def pieces(*, faces: np.ndarray) -> int:
    """Returns the number of connected pieces of the surface."""
    parent = {int(v): int(v) for v in np.unique(faces)}

    def root(v: int) -> int:
        while parent[v] != v:
            parent[v] = parent[parent[v]]
            v = parent[v]
        return v

    for a, b, c in faces:
        for u, w in ((a, b), (b, c)):
            parent[root(int(u))] = root(int(w))
    return len({root(v) for v in parent})


def topology(*, faces: np.ndarray) -> dict:
    """Returns the counts and checks that describe a closed surface."""
    counts = edge_counts(faces=faces)
    directed = np.vstack([faces[:, [0, 1]], faces[:, [1, 2]], faces[:, [2, 0]]])
    vertices = len(np.unique(faces))
    edges = len(counts)
    return {
        "vertices": vertices,
        "edges": edges,
        "faces": len(faces),
        "euler": vertices - edges + len(faces),
        "open": sum(1 for used in counts.values() if used == 1),
        "pinched": sum(1 for used in counts.values() if used > 2),
        "oriented": len(np.unique(directed, axis=0)) == len(directed),
        "pieces": pieces(faces=faces),
    }


def volume_signed(*, vertices: np.ndarray, faces: np.ndarray) -> float:
    """Returns the signed volume, positive when the faces wind outward."""
    a, b, c = (vertices[faces[:, i]] for i in range(3))
    return float(np.einsum("ij,ij->i", a, np.cross(b, c)).sum() / 6.0)


def stl_write(*, path: Path, vertices: np.ndarray, faces: np.ndarray) -> None:
    """Writes a binary STL."""
    triangles = vertices[faces]
    normals = np.cross(
        triangles[:, 1] - triangles[:, 0], triangles[:, 2] - triangles[:, 0]
    )
    normals /= np.linalg.norm(normals, axis=1, keepdims=True)
    record = np.dtype(
        [("normal", "<f4", 3), ("vertices", "<f4", (3, 3)), ("attribute", "<u2")]
    )
    data = np.zeros(len(faces), dtype=record)
    data["normal"] = normals
    data["vertices"] = triangles
    with path.open("wb") as file:
        file.write(b"unit sphere, marching cubes (Lewiner)".ljust(80, b" "))
        file.write(np.uint32(len(faces)).tobytes())
        file.write(data.tobytes())


def triangles(*, vertices: np.ndarray, faces: np.ndarray) -> set:
    """Returns the triangles as a set, independent of vertex numbering."""
    points = np.round(vertices, 9)
    return {frozenset(map(tuple, points[face])) for face in faces}


def seconds_best(*, voxels: np.ndarray, method: str) -> float:
    """Returns the fastest of several marching-cubes runs, in seconds."""
    best = np.inf
    for _ in range(REPEATS):
        start = time.perf_counter()
        isosurface(voxels=voxels, method=method)
        best = min(best, time.perf_counter() - start)
    return best


def main() -> None:
    here = Path(__file__).parent
    exact = 4.0 * np.pi / 3.0

    print(
        f"{'n':>3}  {'v':>7}  {'e':>9}  {'f':>7}  {'v-e+f':>5}  {'open':>4}  "
        f"{'pinched':>7}  {'oriented':>8}  {'pieces':>6}  {'volume':>7}  "
        f"{'error':>7}  {'r min':>6}  {'r max':>6}  {'r mean':>6}  {'r CoV':>6}"
    )
    for n in RADII:
        voxels = np.load(here / f"unit_sphere_n{n:03d}.npy")
        vertices, faces = isosurface(voxels=voxels)
        vertices = (vertices - (n + PAD)) / n
        stl_write(
            path=here / f"unit_sphere_mc_n{n:03d}.stl",
            vertices=vertices,
            faces=faces,
        )
        t = topology(faces=faces)
        radius = np.linalg.norm(vertices, axis=1)
        volume = volume_signed(vertices=vertices, faces=faces)
        error = (volume - exact) / exact
        print(
            f"{n:>3}  {t['vertices']:>7,}  {t['edges']:>9,}  {t['faces']:>7,}  "
            f"{t['euler']:>5}  {t['open']:>4}  {t['pinched']:>7}  "
            f"{t['oriented']!s:>8}  {t['pieces']:>6}  {volume:7.4f}  "
            f"{error:+7.2%}  {radius.min():6.4f}  {radius.max():6.4f}  "
            f"{radius.mean():6.4f}  {radius.std() / radius.mean():6.2%}"
        )
    print(f"exact volume 4π/3 = {exact:.4f}")

    print()
    print(f"{'n':>3}  {'Lewiner (ms)':>12}  {'Lorensen (ms)':>13}  {'same mesh':>9}")
    for n in RADII:
        voxels = np.load(here / f"unit_sphere_n{n:03d}.npy")
        v1, f1 = isosurface(voxels=voxels, method="lewiner")
        v2, f2 = isosurface(voxels=voxels, method="lorensen")
        same = triangles(vertices=v1, faces=f1) == triangles(vertices=v2, faces=f2)
        lewiner = 1e3 * seconds_best(voxels=voxels, method="lewiner")
        lorensen = 1e3 * seconds_best(voxels=voxels, method="lorensen")
        print(f"{n:>3}  {lewiner:>12.2f}  {lorensen:>13.2f}  {same!s:>9}")


if __name__ == "__main__":
    main()
