# Sphere with Shells

This section presents a model composed of a sphere with two concentric shells.
We use the model to explore answers to the following questions:

1. What compute time is required to create successively refined resolutions in `automesh`?
2. What compute time is required to create these same resolutions in Sculpt?
3. Given a rotational boundary condition, what are the displacement and strain fields for the voxel mesh?
4. How do the results for the voxel mesh compare with the results for a conforming mesh?
5. To what degree may smoothing the voxel mesh improve the results?
6. To what degree may dualization of the voxel mesh improve the results?

## Model

[Python](#source) is used to create a segementations, saved as `.npy` files, and visualize the results.

### Given

Given three concentric spheres of radius 10, 11, and 12 cm, as shown in the figure below,

![spheres_cont_dim](img/spheres_cont_dim.png)

Figure: Schematic cross-section of three concentric spheres of radius 10, 11, and 12 cm.  Grid spacing is 1 cm.

### Find

Use the following segmentation resolutions,

resolution (vox/cm) | element side length (cm) | `nelx` | # voxels
---: | :---: | ---: | ---:
1 | 1.0 | 24 | 13,824
2 | 0.5 | 48 | 110,592
4 | 0.25 | 96 | 884,736
10 | 0.1 | 240 | 13,824,000

with a cubic domain (`nelx = nely = nelz`),
to create finite element meshes.

## Solution

### Python Segmentation

The Python code used to generate the figures is included [below](#source).

![spheres_cont](img/spheres_cont.png)

Figure: Sphere segmentations at selected resolutions, shown in the voxel domain.
Because plotting large domains with [*Matplotlib*](https://matplotlib.org)
is slow, only the first two resolutions are shown.

## Source

### `spheres_cont.py`

```python
<!-- cmdrun cat spheres_cont.py -->
```
