# Metrics

```sh
automesh metrics --help
<!-- cmdrun automesh metrics --help -->
```

## Hexahedral Metrics

```sh
automesh metrics hex --help
<!-- cmdrun automesh metrics hex --help -->

`automesh` implements the following hexahedral element quality metrics defined in the Verdict report.[^Knupp_2006]

* Maximum edge ratio ${\rm ER}_{\max}$
* Minium scaled Jacobian ${\rm SJ}_{\min}$
* Maximum skew
* Element volume

A brief description of each metric follows.

### Maximum Edge Ratio

* ${\rm ER}_{\max}$ measures the ratio of the longest edge to the shortest edge in a mesh element.
* A ratio of 1.0 indicates perfect element quality, whereas a very large ratio indicates bad element quality.
* Knupp *et al.*[^Knupp_2006] (page 87) indicate an acceptable range of `[1.0, 1.3]`.

### Minimum Scaled Jacobian

* ${\rm SJ}_{\min}$ evaluates the determinant of the Jacobian matrix at each of the corners nodes, normalized by the corresponding edge lengths, and returns the minimum value of those evaluations.
* Knupp *et al.*[^Knupp_2006] (page 92) indicate an acceptable range of `[0.5, 1.0]`, though in practice, minimum values as low as `0.2` and `0.3` are often used.

![](img/metrics_msj.png)

Figure. Illustrate of minimum scaled Jacobian[^Hovey_2023] with acceptable range for quality occurring in `[0.3, 1.0]`.

### Maximum Skew

* Skew measures how much an element deviates from being a regular shape (e.g., in 3D a cube; in 2D a square or equilateral triangle). A skew value of 0 indicates a perfectly regular shape, while higher values indicate increasing levels of distortion.
* Knupp *et al.*[^Knupp_2006] (page 97) indicate an acceptable range of `[0.0, 0.5]`.

### Element Volume

* Measures the volume of the element.

## Hexahedral Unit Tests

Inspired by Figure 2 of Livesu *et al.*[^Livesu_2021] reproduced here below

![](img/Livesu_Fig_2.png)

we examine several unit test singleton elements and their metrics.

valence | singleton | ${\rm ER}_{\max}$ | ${\rm SJ}_{\min}$ | ${\rm skew_{\max}}$ | volume
:---: | :---: | :---: | :---: | :---: | :---:
3           | ![](img/single_valence_03.png)        | 1.000000e0 (1.000)    | 8.660253e-1 (0.866)   | 5.000002e-1 (0.500)   | 8.660250e-1 (0.866)
3' (noised) | ![](img/single_valence_03_noise1.png) | 1.292260e0 (2.325) ** *Cubit (aspect ratio): 1.292* | 1.917367e-1 (0.192)   | 6.797483e-1 (0.680)   | 1.247800e0  (1.248)
4           | ![](img/single_valence_04.png)        | 1.000000e0 (1.000)    | 1.000000e0  (1.000)   | 0.000000e0  (0.000)   | 1.000000e0  (1.000)
4' (noised) | ![](img/single_valence_04_noise2.png) | 1.167884e0 (1.727) ** *Cubit (aspect ratio): 1.168* | 3.743932e-1 (0.374)   | 4.864936e-1 (0.486)   | 9.844008e-1 (0.984)
5           | ![](img/single_valence_05.png)        | 1.000000e0 (1.000)    | 9.510566e-1 (0.951)   | 3.090169e-1 (0.309)   | 9.510570e-1 (0.951)
6           | ![](img/single_valence_06.png)        | 1.000000e0 (1.000)    | 8.660253e-1 (0.866)   | 5.000002e-1 (0.500)   | 8.660250e-1 (0.866)
...         | ...                                   | ...                   | ...                   | ...                   | ...
10          | ![](img/single_valence_10.png)        | 1.000000e0 (1.000)    | 5.877851e-1 (0.588)   | 8.090171e-1 (0.809)   |  5.877850e-1 (0.588)

Figure: Maximum edge ratio, minimum scaled Jacobian, maximum skew, and volume.
Leading values are from `automesh`.
Values in parenthesis are results from [HexaLab](https://www.hexalab.net).[^Hexalab_2023]
Items with ** indicate where `automesh` and Cubit agree, but HexaLab disagrees.
  Cubit uses the term *Aspect Ratio* for Edge Ratio.

The connectivity for all elements:

```sh
1,    2,    4,    3,    5,    6,    8,    7
```

with prototype:

![](../examples/unit_tests/single.png)

The element coordinates follow:

```sh
# 3
    1,      0.000000e0,      0.000000e0,      0.000000e0
    2,      1.000000e0,      0.000000e0,      0.000000e0
    3,     -0.500000e0,      0.866025e0,      0.000000e0
    4,      0.500000e0,      0.866025e0,      0.000000e0
    5,      0.000000e0,      0.000000e0,      1.000000e0
    6,      1.000000e0,      0.000000e0,      1.000000e0
    7,     -0.500000e0,      0.866025e0,      1.000000e0
    8,      0.500000e0,      0.866025e0,      1.000000e0

# 3'
    1,      0.110000e0,      0.120000e0,     -0.130000e0
    2,      1.200000e0,     -0.200000e0,      0.000000e0
    3,     -0.500000e0,      1.866025e0,     -0.200000e0
    4,      0.500000e0,      0.866025e0,     -0.400000e0
    5,      0.000000e0,      0.000000e0,      1.000000e0
    6,      1.000000e0,      0.000000e0,      1.000000e0
    7,     -0.500000e0,      0.600000e0,      1.400000e0
    8,      0.500000e0,      0.866025e0,      1.200000e0

# 4
    1,      0.000000e0,      0.000000e0,      0.000000e0
    2,      1.000000e0,      0.000000e0,      0.000000e0
    3,      0.000000e0,      1.000000e0,      0.000000e0
    4,      1.000000e0,      1.000000e0,      0.000000e0
    5,      0.000000e0,      0.000000e0,      1.000000e0
    6,      1.000000e0,      0.000000e0,      1.000000e0
    7,      0.000000e0,      1.000000e0,      1.000000e0
    8,      1.000000e0,      1.000000e0,      1.000000e0

# 4'
    1,      0.100000e0,      0.200000e0,      0.300000e0
    2,      1.200000e0,      0.300000e0,      0.400000e0
    3,     -0.200000e0,      1.200000e0,     -0.100000e0
    4,      1.030000e0,      1.102000e0,     -0.250000e0
    5,     -0.001000e0,     -0.021000e0,      1.002000e0
    6,      1.200000e0,     -0.100000e0,      1.100000e0
    7,      0.000000e0,      1.000000e0,      1.000000e0
    8,      1.010000e0,      1.020000e0,      1.030000e0

# 5
    1,      0.000000e0,      0.000000e0,      0.000000e0
    2,      1.000000e0,      0.000000e0,      0.000000e0
    3,      0.309017e0,      0.951057e0,      0.000000e0
    4,      1.309017e0,      0.951057e0,      0.000000e0
    5,      0.000000e0,      0.000000e0,      1.000000e0
    6,      1.000000e0,      0.000000e0,      1.000000e0
    7,      0.309017e0,      0.951057e0,      1.000000e0
    8,      1.309017e0,      0.951057e0,      1.000000e0

# 6
    1,      0.000000e0,      0.000000e0,      0.000000e0
    2,      1.000000e0,      0.000000e0,      0.000000e0
    3,      0.500000e0,      0.866025e0,      0.000000e0
    4,      1.500000e0,      0.866025e0,      0.000000e0
    5,      0.000000e0,      0.000000e0,      1.000000e0
    6,      1.000000e0,      0.000000e0,      1.000000e0
    7,      0.500000e0,      0.866025e0,      1.000000e0
    8,      1.500000e0,      0.866025e0,      1.000000e0

# 10
    1,      0.000000e0,      0.000000e0,      0.000000e0
    2,      1.000000e0,      0.000000e0,      0.000000e0
    3,      0.809017e0,      0.587785e0,      0.000000e0
    4,      1.809017e0,      0.587785e0,      0.000000e0
    5,      0.000000e0,      0.000000e0,      1.000000e0
    6,      1.000000e0,      0.000000e0,      1.000000e0
    7,      0.809017e0,      0.587785e0,      1.000000e0
    8,      1.809017e0,      0.587785e0,      1.000000e0
```

## Triangular Metrics

```sh
automesh metrics tri --help
<!-- cmdrun automesh metrics tri --help -->

`automesh` implements the following triangular element quality metrics:

* Maximum edge ratio ${\rm ER}_{\max}$
* Minium scaled Jacobian ${\rm SJ}_{\min}$
* Maximum skew
* Element area
* Minimum angle

A brief description of each metric follows.

### Maximum Edge Ratio

* ${\rm ER}_{\max}$ measures the ratio of the longest edge to the shortest edge in a mesh element.
* A ratio of 1.0 indicates perfect element quality, whereas a very large ratio indicates bad element quality.
* Knupp *et al.*[^Knupp_2006] (page 26) indicate an acceptable range of `[1.0, 1.3]`.

### Minimum Scaled Jacobian

* ${\rm SJ}_{\min}$ evaluates the determinant of the Jacobian matrix at each of the corners nodes, normalized by the corresponding edge lengths, and returns the minimum value of those evaluations.
* Knupp *et al.*[^Knupp_2006] (page 29) indicate an acceptable range of `[0.5, 2*sqrt(3)/3]` $\approx$ `[0.5, 1.2]`.
* A scaled Jacobian close to `0` indicates that the triangle is poorly shaped (e.g., very thin or degenerate), which can lead to numerical instability.
* A scaled Jacobian of `1` indicates that the triangle is equilateral, which is the ideal shape for numerical methods.  This is achieved through scaling as follows:

$$
{\rm SJ_{\min} = \frac{\sin(\theta_{\min})}{\sin(60^{\circ})}}
$$

* In the preceeding equation,
  * $60^{\circ}$ is the smallest angle of an equilateral triangle, and
  * $\theta_{\min}$ is the smallest angle of the subject triangle.

### Maximum Skew

* Skew measures how much an element deviates from being a regular shape (e.g., in 3D a cube; in 2D a square or equilateral triangle). A skew value of 0 indicates a perfectly regular shape, while higher values indicate increasing levels of distortion.
* Knupp *et al.*[^Knupp_2006] does not give a definition of skew for triangles, so we provide our definition below.
For a triangle where $\theta_{\min}$ is the smallest angle of the triangle,

$$
{\rm skew_{\max}} = \frac{60^{\circ} - \theta_{\min}}{60^{\circ}}
$$

* For an equilateral triangle, $\theta_{\min} = 60^{\circ}$ and ${\rm skew_{\max}} = 0$.
* In the limit as $\theta_{\min} \rightarrow 0^{\circ}$ ${\rm skew_{\max}} \rightarrow 1$.

### Element Area

* Measures the area of the element.

### Minimum Angle

* The smallest the three angles of a triangle.

## Triangular Unit Tests

We use the ABAQUS input file `single_valence_04_noise2.inp`.
We import the file into Cubit and create a triangular surface mesh:

```sh
import abaqus mesh geometry  "/Users/chovey/autotwin/automesh/tests/input/single_valence_04_noise2.inp" feature_angle 135.00
surface 1 scheme trimesh minimum size 100
surface 2 scheme trimesh minimum size 100
surface 3 scheme trimesh minimum size 100
surface 4 scheme trimesh minimum size 100
surface 5 scheme trimesh minimum size 100
# surface 6 scheme trimesh minimum size 100 # there is no side 6, two sides were merged
delete mesh surface all propagate
surface all scheme trimesh
mesh surface all
quality tri all aspect ratio global draw mesh list detail
quality tri all scaled jacobian global draw mesh list detail
quality tri all element area global draw mesh list detail
export stl ascii "/Users/chovey/autotwin/automesh/tests/input/single_valence_04_noise2.stl" mesh  overwrite
```

We also examine several `.stl` files and process them, for example,

```sh
import stl "/Users/chovey/autotwin/automesh/tests/input/one_facet.stl" feature_angle 135.00 merge make_elements
surface 1 scheme trimesh minimum size 100
delete mesh surface 1  propagate
surface 1  scheme trimesh
mesh surface 1
quality tri all aspect ratio global draw mesh list detail
quality tri all scaled jacobian global draw mesh list detail
quality tri all element area global draw mesh list detail
```

We verify the following element qualities:

file  |  `e`  | ${\rm ER}_{\max}$ | ${\rm SJ}_{\min}$ | ${\rm skew_{\max}}$  | area | $\theta_{\min}$ (deg)
:---: | :---: | :---: | :---: | :---: | :---: | :---:
`A`   |   1   | 1.508 [1.508] | 0.761 [0.761] | 0.313 [0.331] | 0.610 [0.610] | 41.2 [41.2]
`A`   |   2   | 1.550 [1.550] | 0.739 [0.739] | 0.337 [0.337] | 0.550 [0.550] | 39.8 [39.8]
`A`   |   3   | 1.787 [1.787] | 0.639 [0.639] | 0.440 [0.440] | 0.569 [0.569] | 33.6 [33.6]
`A`   |   4   | 1.915 [1.915] | 0.595 [0.595] | 0.483 [0.483] | 0.402 [0.402] | 31.0 [31.0]
`A`   |   5   | 2.230 [2.230] | 0.426 [0.426] | 0.639 [0.639] | 0.342 [0.342] | 21.7 [21.7]
`A`   |   6   | 1.623 [1.623] | 0.700 [0.700] | 0.378 [0.378] | 0.571 [0.571] | 37.3 [37.1]
`A`   |   7   | 1.240 [1.240] | 0.898 [0.898] | 0.149 [0.149] | 0.424 [0.424] | 51.0 [51.0]
`A`   |   8   | 1.385 [1.385] | 0.831 [0.831] | 0.233 [0.233] | 0.443 [0.443] | 46.1 [46.1]
`A`   |   9   | 1.606 [1.606] | 0.719 [0.719] | 0.358 [0.358] | 0.648 [0.648] | 38.5 [38.5]
`A`   |  10   | 1.429 [1.429] | 0.806 [0.806] | 0.262 [0.262] | 0.704 [0.704] | 44.3 [44.3]
`A`   |  11   | 1.275 [1.275] | 0.880 [0.880] | 0.172 [0.172] | 0.668 [0.668] | 49.7 [49.7]
`A`   |  12   | 1.436 [1.436] | 0.804 [0.804] | 0.264 [0.264] | 0.516 [0.516] | 44.1 [44.1]
`B`   |   1   | 1.414 [1.414] | 0.816 [0.816] | 0.250 [0.250] | 0.500 [0.500] | 45.0 [45.0]
`C`   |   1   | 1.000 [1.000] | 1.000 [1.000] | 0.000 [0.000] | 6.928 [6.928] | 60.0 [60.0]
`D`   |   1   | 1.000 [1.000] | 1.000 [1.000] | 0.000 [0.000] | 0.433 [0.433] | 60.0 [60.0]
`E`   |   1   | 1.256 [1.256] | 0.869 [0.869] | 0.187 [0.187] | 3.273 [3.273] | 48.8 [48.8]

Figure: Triangle metrics.
Leading values are from `automesh`.
Values in [brackets] are from an independent Python calculation, (see [`metrics_triangle.py`](#metrics_trianglepy)) and agree with `automesh` in double precision with a tolerance of less than `2.22e-15`.
Except for edge ratio, all values were also verified with Cubit.
Cubit uses the term *Aspect Ratio*; it is **not the same** as Edge Ratio.

* File `A` is `single_valence_04_noise2.inp`.
* File `B` is `one_facet.stl`.
* File `C` is an equilateral triangle with nodal coordinates at `(-2, 0, 0)`, `(2, 0, 0)`, and `(0, 2*sqrt(3), 0)` and has side length `4.0`, saved to `tests/input/equilateral_4.stl`.
* File `D` is an equilateral triangle with nodal coordinates at `(-0.5, 0, 0)`, `(0.5, 0, 0)`, and `(0, sqrt(3) / 2, 0)` and has side length `1.0`, saved to `tests/input/equilateral_1.stl`.
* File `E` is an off axis triangle with **approximate** (30, 60, 90) degree inner angles, with coordinates at `(0.0, 1.0, 3.0)`, `(2.0, 0.0, 2.0)`, and `(1.0, sqrt(3.0) + 1.0, 1.0)`, saved to `tests/input/off_axis.stl`.
* `e` is the element number in the mesh.

## Source

### `metrics_triangle.py`

```python
<!-- cmdrun cat metrics_triangle.py -->
```

## References

[^Knupp_2006]: Knupp PM, Ernst CD, Thompson DC, Stimpson CJ, Pebay PP. The verdict geometric quality library. SAND2007-1751. Sandia National Laboratories (SNL), Albuquerque, NM, and Livermore, CA (United States); 2006 Mar 1. [link](https://www.osti.gov/servlets/purl/901967)

[^Hovey_2023]: Hovey CB. Naval Force Health Protection Program Review 2023 Presentation Slides. SAND2023-05198PE. Sandia National Lab.(SNL-NM), Albuquerque, NM (United States); 2023 Jun 26.  [link](https://1drv.ms/p/s!ApVSeeLlvsE8g9UPEHLqBCVxT2jfCQ?e=iEAcgr)

[^Livesu_2021]: Livesu M, Pitzalis L, Cherchi G. Optimal dual schemes for adaptive grid based hexmeshing. ACM Transactions on Graphics (TOG). 2021 Dec 6;41(2):1-4. [link](https://dl.acm.org/doi/pdf/10.1145/3494456)

[^Hexalab_2023]: Bracci M, Tarini M, Pietroni N, Livesu M, Cignoni P. HexaLab.net: An online viewer for hexahedral meshes. Computer-Aided Design. 2019 May 1;110:24-36. [link](https://doi.org/10.1016/j.cad.2018.12.003)
