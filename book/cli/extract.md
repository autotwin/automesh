# Extract

`extract` pulls a rectangular sub-range of voxels out of a larger
segmentation, given inclusive `--xmin`/`--xmax`, `--ymin`/`--ymax`, and
`--zmin`/`--zmax` bounds — useful for isolating a region of interest without
regenerating the whole domain.

```sh
automesh extract --help
<!-- cmdrun automesh extract --help -->
```
