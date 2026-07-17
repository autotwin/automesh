# Defeature

`defeature` removes small, undesired clusters of voxels using a voxel-count
threshold.  A cluster of voxels, defined as two or more voxels that share a
face (edge and node sharing do not constitute a cluster) with count at or
above the threshold, is preserved, whereas a cluster with a count below the
threshold is eliminated through resorption into the surrounding material.

```sh
automesh defeature --help
<!-- cmdrun automesh defeature --help -->
```

## Examples

* [Blobs](../examples/defeature/blobs.md) — four synthetic circular blobs with
  random noise, illustrating how the threshold determines which clusters are
  preserved and which are resorbed.
