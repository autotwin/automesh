# Hexahedral Metrics

```sh
automesh metrics hex --help
<!-- cmdrun automesh metrics hex --help -->
```

`automesh` implements the following **hexahedral** element quality metrics[^Knupp_2006]:

* Maximum edge ratio ${\rm ER}_{\max}$
* Minimum scaled Jacobian ${\rm SJ}_{\min}$
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

* Skew measures how much an element deviates from being a regular shape (e.g., in 3D a cube or regular tetrahedron; in 2D a square or equilateral triangle). A skew value of 0 indicates a perfectly regular shape, while higher values indicate increasing levels of distortion.
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

Figure: Hexahedral metrics.  Leading values are from `automesh`.  Values in parenthesis are results from [HexaLab](https://www.hexalab.net).[^Hexalab_2023] Items with ** indicate where `automesh` and Cubit agree, but HexaLab disagrees.  Cubit uses the term *Aspect Ratio* for Edge Ratio for hexahedral elements.  All values were also verified with Cubit.

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

## Formulation

For a hexahedral element with eight nodes, the scaled Jacobian at each node is computed as:

$$J_{\text{scaled}} = \frac{\mathbf{n} \cdot \mathbf{w}}{\|\mathbf{u}\| \|\mathbf{v}\| \|\mathbf{w}\|}
$$

where:

* $\mathbf{u}$, $\mathbf{v}$, and $\mathbf{w}$ are edge vectors emanating from the node,
* $\mathbf{n} = \mathbf{u} \times \mathbf{v}$ is the cross product of the first two edge vectors, and
* $\|\cdot\|$ denotes the Euclidean norm.

The minimum scaled Jacobian for the element is:

$$
J_{\text{min}} = \min_{i=0}^{7} J_{\text{scaled}}^{(i)}
$$

### Node Numbering Convention

The hexahedral element uses the following local node numbering (standard convention):
       
```src
       7----------6
      /|         /|
     / |        / |
    4----------5  |
    |  |       |  |
    |  3-------|--2
    | /        | /
    |/         |/
    0----------1
```

## Edge Vectors at Each Node

For each node $i$, three edge vectors are defined that point to adjacent nodes. The connectivity follows this pattern:

| Node | $\mathbf{u}$ | $\mathbf{v}$ | $\mathbf{w}$ | Edge Vector Definitions |
|------|--------------|--------------|--------------|-------------------------|
| 0    | →  1     | →  3     | →  4     | $\mathbf{u} = \mathbf{x}_1 - \mathbf{x}_0$, $\mathbf{v} = \mathbf{x}_3 - \mathbf{x}_0$, $\mathbf{w} = \mathbf{x}_4 - \mathbf{x}_0$ |
| 1    | →  2     | →  0     | →  5     | $\mathbf{u} = \mathbf{x}_2 - \mathbf{x}_1$, $\mathbf{v} = \mathbf{x}_0 - \mathbf{x}_1$, $\mathbf{w} = \mathbf{x}_5 - \mathbf{x}_1$ |
| 2    | →  3     | →  1     | →  6     | $\mathbf{u} = \mathbf{x}_3 - \mathbf{x}_2$, $\mathbf{v} = \mathbf{x}_1 - \mathbf{x}_2$, $\mathbf{w} = \mathbf{x}_6 - \mathbf{x}_2$ |
| 3    | →  0     | →  2     | →  7     | $\mathbf{u} = \mathbf{x}_0 - \mathbf{x}_3$, $\mathbf{v} = \mathbf{x}_2 - \mathbf{x}_3$, $\mathbf{w} = \mathbf{x}_7 - \mathbf{x}_3$ |
| 4    | →  7     | →  5     | →  0     | $\mathbf{u} = \mathbf{x}_7 - \mathbf{x}_4$, $\mathbf{v} = \mathbf{x}_5 - \mathbf{x}_4$, $\mathbf{w} = \mathbf{x}_0 - \mathbf{x}_4$ |
| 5    | →  4     | →  6     | →  1     | $\mathbf{u} = \mathbf{x}_4 - \mathbf{x}_5$, $\mathbf{v} = \mathbf{x}_6 - \mathbf{x}_5$, $\mathbf{w} = \mathbf{x}_1 - \mathbf{x}_5$ |
| 6    | →  5     | →  7     | →  2     | $\mathbf{u} = \mathbf{x}_5 - \mathbf{x}_6$, $\mathbf{v} = \mathbf{x}_7 - \mathbf{x}_6$, $\mathbf{w} = \mathbf{x}_2 - \mathbf{x}_6$ |
| 7    | →  6     | →  4     | →  3     | $\mathbf{u} = \mathbf{x}_6 - \mathbf{x}_7$, $\mathbf{v} = \mathbf{x}_4 - \mathbf{x}_7$, $\mathbf{w} = \mathbf{x}_3 - \mathbf{x}_7$ |

where $\mathbf{x}_i$ is the position of node $i$.

## Algorithm

1. **For each element in the mesh:**

   a. Extract the 8 node indices from the connectivity array
   
   b. **For each node $i \in \{0, 1, \ldots, 7\}$:**
      
      - Compute edge vectors:
        $$\mathbf{u} = \mathbf{x}_j - \mathbf{x}_i$$
        $$\mathbf{v} = \mathbf{x}_k - \mathbf{x}_i$$
        $$\mathbf{w} = \mathbf{x}_\ell - \mathbf{x}_i$$
        where $j$, $k$, $\ell$ are adjacent nodes per the table above
      
      - Compute cross product: $\mathbf{n} = \mathbf{u} \times \mathbf{v}$
      
      - Compute scaled Jacobian:
        $$J_{\text{scaled}}^{(i)} = \frac{\mathbf{n} \cdot \mathbf{w}}{\|\mathbf{u}\| \|\mathbf{v}\| \|\mathbf{w}\|}$$
   
   c. Take minimum over all 8 nodes:
      $$J_{\text{min}} = \min_{i=0}^{7} J_{\text{scaled}}^{(i)}$$

2. **Return** the vector of minimum scaled Jacobians, one per element

## Implementation

This prototyptical Rust implementation calculates the MSJ by evaluating the Jacobian at each of the eight corners using the edges connected to that corner.

```rust
struct Vector3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Vector3 {
    fn sub(a: &Vector3, b: &Vector3) -> Vector3 {
        Vector3 { x: a.x - b.x, y: a.y - b.y, z: a.z - b.z }
    }

    fn dot(a: &Vector3, b: &Vector3) -> f64 {
        a.x * b.x + a.y * b.y + a.z * b.z
    }

    fn cross(a: &Vector3, b: &Vector3) -> Vector3 {
        Vector3 {
            x: a.y * b.z - a.z * b.y,
            y: a.z * b.x - a.x * b.z,
            z: a.x * b.y - a.y * b.x,
        }
    }

    fn norm(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }
}

/// Calculates the Minimum Scaled Jacobian for a hexahedron.
/// nodes: An array of 8 points ordered by standard FEM convention.
pub fn min_scaled_jacobian(nodes: &[Vector3; 8]) -> f64 {
    // Define the three edges meeting at each of the 8 corners
    // Corners are ordered: 0-3 (bottom face), 4-7 (top face)
    // For each node index: (current_node, u_target, v_target, w_target)
    let corner_indices = [
        (0, 1, 3, 4), // Corner 0: edges to 1, 3, 4
        (1, 2, 0, 5), // Corner 1: edges to 2, 0, 5
        (2, 3, 1, 6), // Corner 2: edges to 3, 1, 6
        (3, 0, 2, 7), // Corner 3: edges to 0, 2, 7
        (4, 7, 5, 0), // Corner 4: edges to 7, 5, 0
        (5, 4, 6, 1), // Corner 5: edges to 4, 6, 1
        (6, 5, 7, 2), // Corner 6: edges to 5, 7, 2
        (7, 6, 4, 3), // Corner 7: edges to 6, 4, 3
    ];

    let mut min_sj = f64::MAX;
    const EPSILON: f64 = 1e-15; // Small threshold to avoid division by zero

    for &(curr, i, j, k) in &corner_indices {
        // Calculate edge vectors: u, v, w from current node
        let u = Vector3::sub(&nodes[i], &nodes[curr]);
        let v = Vector3::sub(&nodes[j], &nodes[curr]);
        let w = Vector3::sub(&nodes[k], &nodes[curr]);

        // Calculate n = u × v
        let n = Vector3::cross(&u, &v);

        // Calculate scaled Jacobian: (n · w) / (||u|| * ||v|| * ||w||)
        let det = Vector3::dot(&n, &w);
        let lengths = u.norm() * v.norm() * w.norm();

        // Avoid division by zero for degenerate elements
        let sj = if lengths > EPSILON {
            det / lengths
        } else {
            f64::NEG_INFINITY // Flag completely degenerate elements
        };

        if sj < min_sj {
            min_sj = sj;
        }
    }

    min_sj
}
```

## Interpretation

- **$J_{\text{min}} = 1.0$**: Perfect rectangular element
- **$J_{\text{min}} > 0$**: Element is valid (positive Jacobian)
- **$J_{\text{min}} = 0$**: Degenerate element (zero volume)
- **$J_{\text{min}} < 0$**: Invalid element (inverted/negative Jacobian)

Typically, mesh quality requirements specify $J_{\text{min}} > 0.3$ for acceptable elements.

## Implementation Notes

The implementation evaluates all 8 nodes of each element and returns the minimum value. This ensures that element distortion at any corner is captured, as poor quality at a single node can affect finite element solution accuracy.

## References

[^Knupp_2006]: Knupp PM, Ernst CD, Thompson DC, Stimpson CJ, Pebay PP. The verdict geometric quality library. SAND2007-1751. Sandia National Laboratories (SNL), Albuquerque, NM, and Livermore, CA (United States); 2006 Mar 1. [link](https://www.osti.gov/servlets/purl/901967)

[^Hovey_2023]: Hovey CB. Naval Force Health Protection Program Review 2023 Presentation Slides. SAND2023-05198PE. Sandia National Lab.(SNL-NM), Albuquerque, NM (United States); 2023 Jun 26.  [link](https://1drv.ms/p/s!ApVSeeLlvsE8g9UPEHLqBCVxT2jfCQ?e=iEAcgr)

[^Livesu_2021]: Livesu M, Pitzalis L, Cherchi G. Optimal dual schemes for adaptive grid based hexmeshing. ACM Transactions on Graphics (TOG). 2021 Dec 6;41(2):1-4. [link](https://dl.acm.org/doi/pdf/10.1145/3494456)

[^Hexalab_2023]: Bracci M, Tarini M, Pietroni N, Livesu M, Cignoni P. HexaLab.net: An online viewer for hexahedral meshes. Computer-Aided Design. 2019 May 1;110:24-36. [link](https://doi.org/10.1016/j.cad.2018.12.003)
