# Isosurface

Isosurfacing is a method to extract the surface from a three-dimensional scalar field.
A scalar field $\phi = \phi(x, y, z): \mathbb{R}^3 \mapsto \mathbb{R}$,
is a function that assigns a scalar value to every point in three-dimensional space.
For the special case when all points in the domain $(x, y, z) \in \mathbb{R}^3$ are aligned into a
regular (i.e., uniform) three-dimensional grid, the scalar field composes a *voxel* field.

The simplest example of non-trivial voxel field consists of a range of only two integer values, for example,
`0` and `1`, where `0` indicates a point in the grid is outside the field, and where `1` indicates a
point in the grid is inside (or on the boundary of) the field.  The interface between `0` and `1` everywhere in the grid composes the **isosurface** of the field.

Given a voxel field, the isosurface can estimated with two common techniques: Marching Cubes (MC) and Dual Contouring (DC).  Lorensen and Cline[^Lorensen_1987] originally proposed MC in 1987.  DC was originally proposed by Ju *et al.*[^Ju_2002] in 2002.

## Marching Cubes

MC operates on each voxel in the 3D grid on a independent basis.
For each voxel, the eight nodes of the voxel are evaluated as outside (`0`) the scalar field or inside (`1`) the scalar field.  The eight nodes, classified as either `0` or `1`, create 256 ($2^8$) possible configurations.  Of these combinations, only 15 are unique configurations, after symmmetry and rotation considerations.  For each configuration, MC generates a set of triangles to approximate the isosurface.

### Advantages

* Simple implementation; uses only interpolation between voxel corners.
* Results in smooth surfaces because it interpolates along edges between voxel corners.  This can be an advantage when smooth meshes are desired but is a disadvantage when sharp edges are desired.

### Disadvantages

* Can produce ambiguous cases wherein the isosurface can be represented in multiple (non-unique) ways.  This can result in a surface artifacts.
* Can produce non-manifold edges.

> **Manifold:** "The mesh forms a 2D manifold if the local topology is everywhere equivalent to a disc; that is, if the neighborhood of every feature consists of a connected ring of polygons forming a single surface (see Figure 2 of Luebke[^Luebke_2001] reproduced below). In a triangulated mesh displaying manifold topology, exactly two triangles share every edge, and every triangle shares an edge with exactly three neighboring triangles. A 2D manifold with boundary permits boundary edges, which belong to only one triangle."

maniford | non-manifold
:---: | :---:
![](img/Luebke_2001_manifold.png) | ![](img/Luebke_2001_non-manifold.png)

Figure: Reproduction of Luebke[^Luebke_2001] Figure 2 (left) showing a manifold mesh, and Figure 3 (right) showing a non-manifold mesh because of (a) an edge shared by more than two triangles, (b) a vertex shared by two unconnected sets of triangles, and (c) a T-junction vertex.

## Dual Contouring

DC improves upon the MC algorithm.  DC uses the dual grid of the voxel data, locating nodes of the surface *within* the voxel, rather than on the edge of the voxel (as done with MC).

Boris[^Boris_2025] created a figure, reproduced below, that illustrates the differences between MC and DC.

![](img/Boris_MC_DC.png)

Figure: Reproduction of the figure from Boris[^Boris_2025], illustrating, in two dimentions, the differences between MC and DC.  White circle are outside points.  Black circles are inside points.  In MC, the red points indicate surface vertices at edge intersections.  In DC, the red points indicate surface vertices within a voxel.

### Advantages

* "[C]an produce sharp features by inserting vertices anywhere inside the grid cube, as opposed to the Marching Cubes (MC) algorithm that can insert vertices only on grid edges."[^Rashid_2016]

### Disadvantages

* More complicated than MC since DC uses both position and normal (gradient) information at voxel edges to locate the surface intersection.
* "...unable to guarantee 2-manifold and watertight meshes due to the fact that it produces only one vertex for each grid cube." "DC is that it does not guarantee 2-manifold and intersection-free surfaces. A polygonal mesh is considered as being 2-manifold if each edge of the mesh is shared by only two faces, and if the neighborhood of each vertex of the mesh is the topological equivalent of a disk." [^Rashid_2016]


## References

[^Lorensen_1987]: Lorensen WF. Marching cubes: A high resolution 3D surface construction algorithm. Computer Graphics. 1987;21. [link](http://academy.cba.mit.edu/classes/scanning_printing/MarchingCubes.pdf)

[^Ju_2002]: Ju T, Losasso F, Schaefer S, Warren J. Dual contouring of hermite data. In Proceedings of the 29th annual conference on Computer graphics and interactive techniques 2002 Jul 1 (pp. 339-346).  [link](https://dl.acm.org/doi/pdf/10.1145/566570.566586)

[^Luebke_2001]: Luebke DP. A developer's survey of polygonal simplification algorithms. IEEE Computer Graphics and Applications. 2001 May;21(3):24-35. [link](https://ieeexplore.ieee.org/iel5/38/19913/00920624.pdf)

[^Boris_2025]: Boris. Dual Contouring Tutorial. Available from: https://www.boristhebrave.com/2018/04/15/dual-contouring-tutorial/ [Accessed 18 Jan 2025]. [link](https://www.boristhebrave.com/2018/04/15/dual-contouring-tutorial/)

[^Rashid_2016]: Rashid T, Sultana S, Audette MA. Watertight and 2-manifold surface meshes using dual contouring with tetrahedral decomposition of grid cubes. Procedia engineering. 2016 Jan 1;163:136-48. [link](https://doi.org/10.1016/j.proeng.2016.11.037)
