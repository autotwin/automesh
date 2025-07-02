# Dualization

Dualization is the process of using a primal mesh to construct a dual mesh.
Dualization can be performed on 2D/3D surface meshes composed of quadrilateral
elements, and 3D volumetric meshes composed of hexahedral elements.
Both quadrilateral and hexahedral elements will be discussed.

## Quadtree

With [`plot_quadtree_convention.py`](https://github.com/sandialabs/sibl/blob/master/geo/doc/plot_quadtree_convention.py), we create the following index scheme:

![](mwe/plot_quadtree_convention.png)

With [`fig_quadtree.tex`](https://github.com/sandialabs/sibl/blob/master/geo/doc/fig_quadtree.tex), we create the following image of the inverted tree:

![](mwe/fig_quadtree.png)

With [`plot_quadtree.py`](https://github.com/sandialabs/sibl/blob/master/geo/doc/plot_quadtree.py), we plot a domain 

* A square domain `L0` $$(x, y) \in ([1, 3] \otimes  [-1, 1])$$
* Single point at `(2.6, 0.6)` to trigger refinement.

Level 0 | 1 | 2
--- | --- | ---
![](mwe/plot_quadtree_L0.png) | ![](mwe/plot_quadtree_L1.png) | ![](mwe/plot_quadtree_L2.png)

3 | 4 | 5
--- | --- | ---
![](mwe/plot_quadtree_L3.png) | ![](mwe/plot_quadtree_L4.png) | ![](mwe/plot_quadtree_L5.png)

### Circle from Segmentation

3 | 4 | 5 | 6
--- | --- | --- | ---
![](code/circle_segmentation_diam_3.svg) | ![](code/circle_segmentation_diam_4.svg) | ![](code/circle_segmentation_diam_5.svg) | ![](code/circle_segmentation_diam_6.svg)

13 | 14 | 15 | 16
--- | --- | --- | ---
![](code/circle_segmentation_diam_13.svg) | ![](code/circle_segmentation_diam_14.svg) | ![](code/circle_segmentation_diam_15.svg) | ![](code/circle_segmentation_diam_16.svg)

![](code/circle_segmentation_diam_100.svg)

### Circle from Boundary

Consider a boundary of a circle defined by discrete `(x, y)` points.

![](code/circle_loop_r_50_npts_36.svg)

Level 0 | 1 | 2
--- | --- | ---
| ![](code/quadtree_circle_level_0.svg) | ![](code/quadtree_circle_level_1.svg) | ![](code/quadtree_circle_level_2.svg)

3 | 4 | 5
--- | --- | ---
![](code/quadtree_circle_level_3.svg) | ![](code/quadtree_circle_level_4.svg) | ![](code/quadtree_circle_level_5.svg)


### Circle from Tesellation

### Quarter Plate

With [Python](#source), we produce a Quadtree with zero to five levels of refinement.  Refinement is triggered based on whether or not a cell contains one or more seed points, shown as points along the quarter circle centered at `(4, 0)`.

Level 0 | 1 | 2
--- | --- | ---
| ![](code/quadtree_quarter_plate_level_0.svg) | ![](code/quadtree_quarter_plate_level_1.svg) | ![](code/quadtree_quarter_plate_level_2.svg)

3 | 4 | 5
--- | --- | ---
![](code/quadtree_quarter_plate_level_3.svg) | ![](code/quadtree_quarter_plate_level_4.svg) | ![](code/quadtree_quarter_plate_level_5.svg)

## Octree


## Sphere

Consider a boundary of a sphere defined by a discrete triangular
tesselation.

## References

* [https://github.com/sandialabs/sibl/blob/master/geo/doc/quadtree.md](https://github.com/sandialabs/sibl/blob/master/geo/doc/quadtree.md)
* [https://github.com/sandialabs/sibl/blob/master/geo/doc/dual_quad_transitions.md](https://github.com/sandialabs/sibl/blob/master/geo/doc/dual_quad_transitions.md)
* [https://github.com/sandialabs/sibl/blob/master/geo/doc/dual/lesson_11.md](https://github.com/sandialabs/sibl/blob/master/geo/doc/dual/lesson_11.md)

## Source

### `quadtree_plot.py`

```python
<!-- cmdrun cat code/quadtree_plot.py -->
```
