# Academic Models

These are the standard test models used throughout the computational geometry
and computer graphics literature — scanned or synthetic surfaces chosen for
specific geometric challenges (sharp features, high genus, curvature
variation, thin structures).
Several are already familiar from elsewhere in this book, notably the
[Stanford bunny](../remesh/bunny.md), and most of the remaining candidates
below are drawn from the same general source: Alec Jacobson's
[`common-3d-test-models`](https://github.com/alecjacobson/common-3d-test-models)
repository, already cited on the bunny page.

The table below tracks candidate models and their status.  "Planned" entries
have no worked example yet — they are a roadmap, not a claim that the model
or a download link is already in the repository.

| Model | Description | Status |
| --- | --- | --- |
| Stanford Bunny | scanned rabbit figurine; sharp folds, ears, a few holes | See [Remesh: Stanford Bunny](../remesh/bunny.md) |
| Utah Teapot | canonical synthetic test surface | Planned |
| Stanford Armadillo | scanned figure; high genus-0 detail, thin limbs | Planned |
| Stanford Dragon | scanned figure; fine surface detail at scale | Planned |
| Fandisk | synthetic CAD-like surface with sharp creases | Planned |
| Rocker Arm / Bimba | scanned mechanical-part test models | Planned |
| Suzanne | Blender's low-poly monkey head; common regression case | Planned |

## Stanford Bunny

1. Download the `stanford-bunny.stl` model from [Remesh: Stanford Bunny](../remesh/bunny.md).

#TODO: [Image of original tesselation]

2. Remesh the `stl`

```sh
# TODO: add appropriate remesh command, e.g.,
automesh remesh -i stanford_bunny.stl -o bunny_adaptive.stl adaptive --minimum 0.002 --maximum 0.040 -n 25 -t 0.02
```

#TODO: Image of the updated tesselation

3. Create an all-hexahedral volume mesh from the STL surface

```sh
automesh mesh hex ... # TODO finish exact specification
```

#TODO: Images of the resulting hex mesh, including cut-through surfaces exposing interior adaptivity.

4. Metrics

#TODO 

* Node count, element count
* Four histograms showing quality measures: Max Edge Ratio, Min Scaled Jacobian, Max Skew, Element Volume