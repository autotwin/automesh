# Subdivision

**Surface subdivision** is a geometric modeling technique that defines smooth curves or surfaces as the limit of a sequence of successive refinements. 

Developed as a generalization of spline surfaces, subdivision allows for the representation of complex, arbitrary control meshes while avoiding the topological constraints and "cracking" issues often associated with traditional Non-Uniform Rational B-Splines (NURBS).

Various algorithms have been established to handle different mesh types and desired continuity:

* The **Catmull-Clark** scheme is frequently used for **quadrilateral** meshes to produce $G^2$ continuous surfaces, while
* The **Loop subdivision** scheme is a popular approximating method specifically designed for triangular meshes.

By iteratively applying simple refinement rules—typically involving a "splitting" step to increase resolution and an "averaging" step to relocate vertices—subdivision transforms a coarse initial shape into a highly detailed, smooth limit surface suitable for high-end animation and scalable rendering.

## Octa Loop

This example uses Loop subdivision to transform an octahedron into a sphere.  The results below are based on [Octa-Loop Subdivision Scheme (GitHub)](https://github.com/autotwin/mesh/blob/main/doc/octa_loop.md).

We create a **unit radius** octahedron template, and successively refine it into a sphere.  The sphere is a useful baseline subject of study because it:

* Can easily be approximated by a voxel stack at various resolutions,
* Can easily be approximated by a finite element mesh,
* Has a known analytic volume, and
* Has a known analytic local curvature.

### Base Octahedron

We created a unit radius template, [`octa_base.obj`](subdivision/octa_base.obj), with contents listed below:

```code
v 1.0 0.0 0.0
v 0.0 1.0 0.0
v -1.0 0.0 0.0
v 0.0 -1.0 0.0
v 0.0 0.0 1.0
v 0.0 0.0 -1.0
f 1 2 5
f 2 3 5
f 3 4 5
f 4 1 5
f 2 1 6
f 3 2 6
f 4 3 6
f 1 4 6
```

### Refinement

The refinement below was created with MeshLab 2022.02, *Subdivision Surfaces LS3 Loop*, based on Boye *et al.*[^Boye2010]

| name                                                                                                       | image                                                                     | file size |
| ---------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- | --------: |
| [`octa_loop0.stl`](subdivision/octa_loop00.stl)                        | ![loop0](subdivision/octa_loop00.png) |     2.1kB |
| [`octa_loop1.stl`](subdivision/octa_loop01.stl)                        | ![loop1](subdivision/octa_loop01.png) |     8.3kB |
| [`octa_loop2.stl`](subdivision/octa_loop02.stl)                        | ![loop2](subdivision/octa_loop02.png) |      33kB |
| [`octa_loop3.stl`](subdivision/octa_loop03.stl)                        | ![loop3](subdivision/octa_loop03.png) |     132kB |
| [`octa_loop4.stl`](subdivision/octa_loop04.stl)                        | ![loop4](subdivision/octa_loop04.png) |     526kB |
| [`octa_loop5.stl`](https://drive.google.com/file/d/1EtlgQH40alzRsy5u-mcUiKF1UjI4uTux/view?usp=sharing) `G` | ![loop5](subdivision/octa_loop05.png) |     2.1MB |
| [`octa_loop6.stl`](https://drive.google.com/file/d/1oUuHunLHgbF2BIY2qkEKzQXsBh0RZqc0/view?usp=sharing) `G` | ![loop6](subdivision/octa_loop06.png) |     8.6MB |
| [`octa_loop7.stl`](https://drive.google.com/file/d/15z9_C09LAXwFgarI-HPwSQpgPYKk1oAM/view?usp=sharing) `G` | ![loop7](subdivision/octa_loop07.png) |      33MB |

> Items with `G` are not on the repository; they are on Google Drive because of their large file size.  

#### Geometric Metrics

* The surface area of a sphere is $A = 4 \pi r^2$, and when $r=1$, $A \approx 12.566371$.
* The volume of a sphere is $\frac{4}{3} \pi r^3$, and when $r=1$, $V \approx 4.188790$.

Using the **Euler Characteristic** for a sphere ($v - e + f = 2$), we can verify the progression from the base octahedron ($v=6, e=12, f=8$):

Iteration ($i$) | Vertices ($v_i$​) | Edges ($e_i$​) | Faces ($f_i$) | Calculation ($v_{i+1}​=v_i​+e_i​$)
--- | --- | --- | --- | ---
0 (Base) | 6 | 12 | 8 | 6+12=18
1|18|48|32|18+48=66
2|66|192|128|66+192=258
3|258|768|512|258+768=1,026
4|1,026|3,072|2,048|1,026+3,072=4,098
5|4,098|12,288|8,192|4,098+12,288=16,386
6|16,386|49,152|32,768|16,386+49,152=65,538
7|65,538|196,608|131,072|65,538+196,608=262,146

The recursive relationships for a closed triangular mesh:

* **Faces:** $f_{i+1} = 4 \times f_i$
* **Edges:** $e_{i+1} = 2 e_i + 3 f_i$
* **Vertices:** $v_{i+1} = v_i + e_i$

### Sculpt Baseline

We created Sculpt baseline meshes with the [`sculpt_stl_to_inp.py` script](https://github.com/autotwin/mesh/blob/main/src/atmesh/sculpt_stl_to_inp.py) as

```sh
(atmeshenv) ~/autotwin/mesh/src/atmesh> python sculpt_stl_to_inp.py
```

and create standard views in Cubit with

```sh
Cubit>
graphics perspective off  # orthogonal, not perspective view
up 0 0 1  # z-axis points up
view iso # isometric x, y, z camera
quality volume 1 scaled jacobian global draw histogram draw mesh list
```

to produce the following results:


| iter  | image                                                                     |    cells | nodes `nnp` | elements `nel` | element density `nel`$/ V$ |
| :---: | ------------------------------------------------------------------------- | -------: | ----------: | -------------: | -------------------------: |
|   0   | ![sculpt00](subdivision/sculpt00.png) | 35x35x35 |       8,696 |          7,343 |                      5,507 |
|   1   | ![sculpt01](subdivision/sculpt01.png) | 28x28x28 |       8,133 |          6,960 |                      2,365 |
|   2   | ![sculpt02](subdivision/sculpt02.png) | 26x26x26 |       7,833 |          6,744 |                      1,762 |
|   3   | ![sculpt03](subdivision/sculpt03.png) | 26x26x26 |       7,731 |          6,672 |                      1,630 |
|   4   | ![sculpt04](subdivision/sculpt04.png) | 26x26x26 |       7,731 |          6,672 |                      1,600 |
|   5   | ![sculpt05](subdivision/sculpt05.png) | 26x26x26 |       7,731 |          6,672 |                      1,596 |
|   6   | ![sculpt06](subdivision/sculpt06.png) | 26x26x26 |       7,731 |          6,672 |                      1,595 |
|   7   | ![sculpt07](subdivision/sculpt07.png) | 26x26x26 |       7,731 |          6,672 |                      1,595 |

## References

* [Octa-Loop Subdivision Scheme (GitHub)](https://github.com/autotwin/mesh/blob/main/doc/octa_loop.md)
  * Documentation detailing the Octa-Loop scheme, a variant of Loop subdivision optimized for octahedrally refined meshes.
* Subdivision Surfaces Lecture Notes (Stanford University)
  * A comprehensive academic overview of subdivision concepts, including the mathematical foundations of the Catmull-Clark and Loop schemes.
  * Stanford cs468-10-fall Subdivision http://graphics.stanford.edu/courses/cs468-10-fall/LectureSlides/10_Subdivision.pdf and [Google Drive repo copy](https://drive.google.com/file/d/1bg5YYrGWduuQ3CPM9FI5e6OyvKD_u8Iy/view?usp=sharing)
* [Catmull–Clark Subdivision Surface (Rosetta Code)](https://rosettacode.org/wiki/Catmull–Clark_subdivision_surface)
  * A technical resource providing algorithmic steps and multi-language code implementations for the Catmull-Clark subdivision process.
* [Recursively Generated B-Spline Surfaces (Original Paper)](https://people.eecs.berkeley.edu/~sequin/CS284/PAPERS/CatmullClark_SDSurf.pdf)
  * The seminal 1978 paper by Edwin Catmull and James Clark that introduced the method for generating smooth surfaces from arbitrary topological meshes (referenced within the other materials).
*  Catmull, E., & Clark, J. (1978). Recursively generated B-spline surfaces on arbitrary topological meshes. Computer-Aided Design, 10(6), 350-355. https://doi.org/10.1016/0010-4485(78)90110-0
* Loop, C. (1987). Smooth subdivision surfaces based on triangles [Master's thesis, University of Utah]. https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/thesis-1.pdf
* https://docs.juliahub.com/Meshes/FuRcu/0.17.1/algorithms/refinement.html#Catmull-Clark

[^Boye2010]: Boyé S, Guennebaud G, Schlick C. Least squares subdivision surfaces. In Computer Graphics Forum 2010 Sep (Vol. 29, No. 7, pp. 2021-2028). Oxford, UK: Blackwell Publishing Ltd.
