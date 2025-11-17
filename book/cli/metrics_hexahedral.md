# Hexahedral Metrics

```sh
automesh metrics hex --help
<!-- cmdrun automesh metrics hex --help -->
```

`automesh` implements the following hexahedral element quality metrics defined in the Verdict report.[^Knupp_2006]

* Maximum edge ratio ${\rm ER}_{\max}$
* Minium scaled Jacobian ${\rm SJ}_{\min}$
* Maximum skew
* Element volume

A brief description of each metric follows.

## Maximum Edge Ratio

* ${\rm ER}_{\max}$ measures the ratio of the longest edge to the shortest edge in a mesh element.
* A ratio of 1.0 indicates perfect element quality, whereas a very large ratio indicates bad element quality.
* Knupp *et al.*[^Knupp_2006] (page 87) indicate an acceptable range of `[1.0, 1.3]`.

## Minimum Scaled Jacobian

* ${\rm SJ}_{\min}$ evaluates the determinant of the Jacobian matrix at each of the corners nodes, normalized by the corresponding edge lengths, and returns the minimum value of those evaluations.
* Knupp *et al.*[^Knupp_2006] (page 92) indicate an acceptable range of `[0.5, 1.0]`, though in practice, minimum values as low as `0.2` and `0.3` are often used.

![](img/metrics_msj.png)

Figure. Illustration of minimum scaled Jacobian[^Hovey_2023] with acceptable range `[0.3, 1.0]`.

## Maximum Skew

* Skew measures how much an element deviates from being a regular shape (e.g., in 3D a cube; in 2D a square or equilateral triangle). A skew value of 0 indicates a perfectly regular shape, while higher values indicate increasing levels of distortion.
* Knupp *et al.*[^Knupp_2006] (page 97) indicate an acceptable range of `[0.0, 0.5]`.

## Element Volume

* Measures the volume of the element.

## Unit Tests

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

## Local Numbering Scheme

### Nodes

The local numbering scheme for nodes of a hexadedral element:

```sh
       7---------6
      /|        /|
     / |       / |
    4---------5  |
    |  3------|--2
    | /       | /
    |/        |/
    0---------1
```

node | connected nodes
:---: | :---:
0 | 1, 3, 4
1 | 0, 2, 5
2 | 1, 3, 6
3 | 0, 2, 7
4 | 0, 5, 7
5 | 1, 4, 6
6 | 2, 5, 7
7 | 3, 4, 6

### Faces

From the exterior of the element, view the (0, 1, 5, 4) face and unwarp the remaining faces; the six face normals now point out of the page.  The local numbering scheme for faces of a hexadedral element:

```sh
              7---------6
              |         |
              |    5    |
              |         |
    7---------4---------5---------6---------7
    |         |         |         |         |
    |    3    |    0    |    1    |    2    |
    |         |         |         |         |
    3---------0---------1---------2---------3
              |         |
              |    4    |
              |         |
              3---------2
```

face | nodes
:---: | :---:
0 | 0, 1, 5, 4
1 | 1, 2, 6, 5
2 | 2, 3, 7, 6
3 | 3, 0, 4, 7
4 | 3, 2, 1, 0
5 | 4, 5, 6, 7


## References

[^Knupp_2006]: Knupp PM, Ernst CD, Thompson DC, Stimpson CJ, Pebay PP. The verdict geometric quality library. SAND2007-1751. Sandia National Laboratories (SNL), Albuquerque, NM, and Livermore, CA (United States); 2006 Mar 1. [link](https://www.osti.gov/servlets/purl/901967)

[^Hovey_2023]: Hovey CB. Naval Force Health Protection Program Review 2023 Presentation Slides. SAND2023-05198PE. Sandia National Lab.(SNL-NM), Albuquerque, NM (United States); 2023 Jun 26.  [link](https://1drv.ms/p/s!ApVSeeLlvsE8g9UPEHLqBCVxT2jfCQ?e=iEAcgr)

[^Livesu_2021]: Livesu M, Pitzalis L, Cherchi G. Optimal dual schemes for adaptive grid based hexmeshing. ACM Transactions on Graphics (TOG). 2021 Dec 6;41(2):1-4. [link](https://dl.acm.org/doi/pdf/10.1145/3494456)

[^Hexalab_2023]: Bracci M, Tarini M, Pietroni N, Livesu M, Cignoni P. HexaLab.net: An online viewer for hexahedral meshes. Computer-Aided Design. 2019 May 1;110:24-36. [link](https://doi.org/10.1016/j.cad.2018.12.003)
