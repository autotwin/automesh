# Defeature

**Defeaturing** is accomplished by specifying a voxel threshold.  A cluster of
voxels, defined as two or more voxels that share a face (edge and node sharing
do not constitute a cluster) with count at or above the threshold will be
preserved, whereas a cluster of voxels with a count below the threshold will
be eliminated through resorption into the surrounding material.

```sh
automesh defeature --help
<!-- cmdrun automesh defeature --help -->
```

## Examples

* [Blobs](defeature_blobs.md) — four synthetic circular blobs with random
  noise, illustrating how the threshold determines which clusters are
  preserved and which are resorbed.
