# Defeaturing

**Defeaturing** is accomplished by specifying a voxel threshold.  A cluster of voxels, defined as two or more voxels that share a face (edge and node sharing do not constitute a cluster) with count at or above the threshold will be preserved, whereas a cluster of voxels with a count below the threshold will be eliminated through resorption into the surrounding material.

## Example

With [Python](#source), we created a segmentation of four circular blobs with noise placed randomly in a bounding box, shown in left of the figure below.  The segmentation file `blobs.npy` was then used as the input to `automesh` with the **defeature** command.  The output file `blobs_defeatured.npy` is shown in the right of the figure.   The threshold was set to 20 voxels.

```sh
automesh defeature -i blobs.npy -o blobs_defeatured.npy -m 20
```

Both the segmentation files, original and defeatured, were then converted to a
mesh and visualized in [Hexalab](https://www.hexalab.net).

![Example of defeaturing: (left) mesh prior to defeaturing, (right) mesh after defeaturing.](blob_defeatured_iso2_high_res.png)

Figure:  (left) Four circular blobs with noise (`blobs.npy`) used as input to the **defeature** command, (right) the output defeatured segmentation (`blobs_defeatured.npy`).

## Source

### `defeature.py`

```python
<!-- cmdrun cat defeature.py -->
```
