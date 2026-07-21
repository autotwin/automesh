"""This module, torus.py, creates a voxelized torus segmentation, used to
demonstrate `automesh mesh hex smooth` chaining on a genus-1 shape.

Example
-------
source ~/autotwin/automesh/.venv/bin/activate
cd ~/autotwin/automesh/book/examples/mesh
python torus.py
"""

import numpy as np
from numpy.typing import NDArray

# Torus parameters, in voxel units.
MAJOR_R = 13.0  # distance from the center of the tube to the center of the torus
MINOR_R = 4.0  # radius of the tube
PAD = 2  # voxels of void padding around the torus


def erode(a: NDArray[np.bool_]) -> NDArray[np.bool_]:
    """One step of binary erosion with a 6-connected (face) structuring
    element: a voxel survives only if it and all its face-neighbors are
    filled.  Out-of-bounds neighbors count as empty."""
    out = a.copy()
    for axis in range(3):
        out &= np.roll(a, 1, axis=axis)
        out &= np.roll(a, -1, axis=axis)
        # np.roll wraps around; undo that wraparound by clearing the edge
        # slices it incorrectly pulled in from the opposite side.
        edge_lo = [slice(None)] * 3
        edge_lo[axis] = 0
        out[tuple(edge_lo)] = False
        edge_hi = [slice(None)] * 3
        edge_hi[axis] = -1
        out[tuple(edge_hi)] = False
    return out


def dilate(a: NDArray[np.bool_]) -> NDArray[np.bool_]:
    """One step of binary dilation with a 6-connected (face) structuring
    element: a voxel is filled if it or any face-neighbor is filled."""
    out = a.copy()
    for axis in range(3):
        shifted_up = np.roll(a, 1, axis=axis)
        shifted_down = np.roll(a, -1, axis=axis)
        edge_lo = [slice(None)] * 3
        edge_lo[axis] = 0
        shifted_up[tuple(edge_lo)] = False
        edge_hi = [slice(None)] * 3
        edge_hi[axis] = -1
        shifted_down[tuple(edge_hi)] = False
        out |= shifted_up
        out |= shifted_down
    return out


# The torus lies flat in the x-y plane, so it needs a much larger extent in
# x and y (to span its outer diameter) than in z (to span the tube only).
xy_extent = int(MAJOR_R + MINOR_R) + PAD
z_extent = int(MINOR_R) + PAD
xy_coords = np.arange(-xy_extent, xy_extent + 1)
z_coords = np.arange(-z_extent, z_extent + 1)
x, y, z = np.meshgrid(xy_coords, xy_coords, z_coords, indexing="ij")

rho = np.sqrt(x**2 + y**2) - MAJOR_R
inside = (rho**2 + z**2) <= MINOR_R**2

# Voxelizing a smooth implicit surface leaves single-voxel "horns": spurs
# that touch the torus body only edge- or corner-adjacent, an aliasing
# artifact of the discretization, not a feature of the torus itself.
# Morphological opening (erosion then dilation, twice) removes them while
# leaving the ring's connectivity and overall shape unchanged.
for _ in range(2):
    inside = erode(inside)
for _ in range(2):
    inside = dilate(inside)

segmentation = np.where(inside, 1, 0).astype(np.uint8)

FILE_NAME = "torus.npy"
np.save(FILE_NAME, segmentation)
print(f"Saved {FILE_NAME} with shape {segmentation.shape}.")
