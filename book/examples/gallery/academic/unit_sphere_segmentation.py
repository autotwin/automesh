#!/usr/bin/env python3
"""Segmentations of a sphere on a grid of integer voxels.

A sphere of radius `n` voxels sits at the center of a cube of `2n + 1` voxels
per side.  A voxel is inside, with value 1, when its center satisfies
`x² + y² + z² <= n²`.  Every other voxel has value 0.  The array axes are
(x, y, z), which is the order `automesh` reads from a `.npy` file.

Example
-------
cd ~/autotwin/automesh/book/examples/gallery/academic
uv run --with numpy unit_sphere_segmentation.py

Output
------
unit_sphere_n010.npy, unit_sphere_n020.npy, unit_sphere_n040.npy,
unit_sphere_n080.npy, unit_sphere_n160.npy, and a table on the terminal.
"""

from pathlib import Path

import numpy as np

RADII = (10, 20, 40, 80, 160)


def sphere(*, radius: int) -> np.ndarray:
    """Returns the segmentation of a sphere of `radius` voxels."""
    if radius < 1:
        raise ValueError("radius must be >= 1")
    k = np.arange(-radius, radius + 1)
    x, y, z = np.meshgrid(k, k, k, indexing="ij", sparse=True)
    return (x * x + y * y + z * z <= radius * radius).astype(np.uint8)


def main() -> None:
    here = Path(__file__).parent
    exact = 4.0 * np.pi / 3.0
    print(
        f"{'n':>3}  {'grid':>11}  {'total':>10}  {'inside':>10}  {'volume':>7}  "
        f"{'error':>7}"
    )
    for n in RADII:
        voxels = sphere(radius=n)
        path = here / f"unit_sphere_n{n:03d}.npy"
        np.save(path, voxels)
        inside = int(voxels.sum())
        volume = inside / n**3
        grid = "x".join(str(m) for m in voxels.shape)
        print(
            f"{n:>3}  {grid:>11}  {voxels.size:>10,}  {inside:>10,}  "
            f"{volume:7.4f}  {(volume - exact) / exact:+7.2%}"
        )


if __name__ == "__main__":
    main()
