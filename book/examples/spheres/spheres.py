r"""This module, spheres.py, creates a voxelized sphere and exports
it as a .npy file.

Example
-------
source ~/autotwin/automesh/.venv/bin/activate
cd book/examples/spheres
python spheres.py
"""

from pathlib import Path
from typing import Final

from matplotlib.colors import LightSource
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d import Axes3D
import numpy as np


def sphere(radius: int, dtype=np.uint8) -> np.ndarray:
    """Generate a 3D voxelized representation of a sphere.

    Parameters
    ----------
    radius: int
        The radius of the sphere.  Minimum value is 1.

    dtype: data-type, optional
        The data type of the output array.  Default is np.uint8.

    Returns
    -------
    np.ndarray
        A 3D numpy array of shape (2*radius+1, 2*radius+1, 2*radius+1)
        representing the voxelized sphere.  Voxels within the sphere are
        set to 1, and those outside are set to 0.

    Raises
    ------
    ValueError
        If the radius is less than 1.

    Example
    -------
    >>> sphere(radius=1) returns
        array(
            [
                [[0, 0, 0], [0, 1, 0], [0, 0, 0]],
                [[0, 1, 0], [1, 1, 1], [0, 1, 0]],
                [[0, 0, 0], [0, 1, 0], [0, 0, 0]]
            ],
            dtype=uint8
        )

    Reference
    ---------
    Adapted from:
    https://github.com/scikit-image/scikit-image/blob/v0.24.0/skimage/morphology/footprints.py#L763-L833
    """
    if radius < 1:
        raise ValueError("Radius must be >= 1")

    n_voxels_per_side = 2 * radius + 1
    vox_z, vox_y, vox_x = np.mgrid[
        -radius : radius : n_voxels_per_side * 1j,
        -radius : radius : n_voxels_per_side * 1j,
        -radius : radius : n_voxels_per_side * 1j,
    ]
    voxel_radius_squared = vox_x**2 + vox_y**2 + vox_z**2
    result = np.array(voxel_radius_squared <= radius * radius, dtype=dtype)
    return result


# User input begin

spheres = {
    "radius_1": sphere(radius=1),
    "radius_3": sphere(radius=3),
    "radius_5": sphere(radius=5),
}

aa = Path(__file__)
bb = aa.with_suffix(".png")

# Visualize the elements.
width, height = 10, 5
# width, height = 8, 4
# width, height = 6, 3
fig = plt.figure(figsize=(width, height))

el, az, roll = 63, -110, 0
cmap = plt.get_cmap(name="tab10")
# NUM_COLORS = len(spheres)
NUM_COLORS = 10  # consistent with tab10 color scheme
VOXEL_ALPHA: Final[float] = 0.9

colors = cmap(np.linspace(0, 1, NUM_COLORS))
lightsource = LightSource(azdeg=325, altdeg=45)  # azimuth, elevation
# lightsource = LightSource(azdeg=325, altdeg=90)  # azimuth, elevation
DPI: Final[int] = 300  # resolution, dots per inch
SHOW: Final[bool] = False  # turn to True to show the figure on screen
SAVE: Final[bool] = False  # turn to True to save .png and .npy files
# User input end


N_SUBPLOTS = len(spheres)
IDX = 1
for index, (key, value) in enumerate(spheres.items()):
    ax = fig.add_subplot(1, N_SUBPLOTS, index + 1, projection=Axes3D.name)
    ax.voxels(
        value,
        facecolors=colors[index],
        edgecolor=colors[index],
        alpha=VOXEL_ALPHA,
        lightsource=lightsource,
    )
    ax.set_title(key.replace("_", "="))
    IDX += 1

    # Set labels for the axes
    ax.set_xlabel("x (voxels)")
    ax.set_ylabel("y (voxels)")
    ax.set_zlabel("z (voxels)")

    # Set the camera view
    ax.set_aspect("equal")
    ax.view_init(elev=el, azim=az, roll=roll)

    if SAVE:
        cc = aa.with_stem("spheres_" + key)
        dd = cc.with_suffix(".npy")
        # Save the data in .npy format
        np.save(dd, value)
        print(f"Saved: {dd}")

fig.tight_layout()
if SHOW:
    plt.show()

if SAVE:
    fig.savefig(bb, dpi=DPI)
    print(f"Saved: {bb}")
