# Smoothing

## Introduction

Both Laplacian smoothing[^Sorkine_2005] and Taubin smoothing[^Taubin_1995a] [^Taubin_1995b] are smoothing operations that adjust the positions of the nodes in a finite element mesh.

Laplacian smoothing, based on the Laplacian operator, computes the average position of a point's neighbors and moves the point toward the average.  This reduces high-frequency noise but can result in a loss of shape and detail, with overall shrinkage.

Taubin smoothing is an extension of Laplacian smoothing that seeks to overcome the shrinkage drawback associated with the Laplacian approach.   Taubin is a two-pass approach.  The first pass smooths the mesh.  The second pass re-expands the mesh.

## Laplacian Smoothing

Consider a subject node with position $\boldsymbol{p} = \boldsymbol{p}(x, y, z) \in \mathbb{R}^3$.  The subject node connects to $n$ neighbor points $\boldsymbol{q}_i$ for $i \in [1, n]$ through $n$ edges.

For concereteness, consider a node with four neighbors, shown in the figure below.

![node_p_q](fig/node_p_q.png)

Figure: The subject node $\boldsymbol{p}$ with edge connections (dotted lines) to neighbor nodes $\boldsymbol{q}_{i}$ with $i \in [1, n]$ (withouth loss of generality, the specific example of $n=4$ is shown).  The average position of all neighbors of $\boldsymbol{p}$ is denoted $\bar{\boldsymbol{q}}$, and the gap $\Delta \boldsymbol{p}$ (dashed line) originates at $\boldsymbol{p}$ and terminates at $\bar{\boldsymbol{q}}$.

Define $\bar{\boldsymbol{q}}$ as the average position of all $\boldsymbol{q}_i$ neighbors of $\boldsymbol{p}$,

$$ \bar{\boldsymbol{q}} := \frac{1}{n} \sum_{i=1}^{n} \boldsymbol{q}_{i}.  $$

Define the **gap vector** $\Delta\boldsymbol{p}$ as originating at $\boldsymbol{p}$ and terminating at $\bar{\boldsymbol{q}}$ (*viz.*, $\boldsymbol{p} + \Delta\boldsymbol{p} = \bar{\boldsymbol{q}}$),

$$ \Delta\boldsymbol{p} := \bar{\boldsymbol{q}} - \boldsymbol{p}. $$

Let $\lambda \in \mathbb{R}^+$ be the positive scaling factor for the gap $\Delta\boldsymbol{p}$.

Since

$$ \bar{\boldsymbol{q}} = \boldsymbol{p} + \lambda\Delta\boldsymbol{p} \hspace{0.5cm} \rm{when} \hspace{0.5cm} \lambda = 1, $$

subdivision of this relationship into several substeps gives rise to an iterative approach.
We typically select $\lambda < 1$ to avoid overshoot of the update, $\lambda \in \mathbb{R}^+ \subset (0, 1)$.

At iteration $k$, we update the position of $\boldsymbol{p}^{(k)}$ by an amount $\lambda \Delta\boldsymbol{p}^{(k)}$ to $\boldsymbol{p}^{(k+1)}$ as

$$ \boldsymbol{p}^{(k+1)} := \boldsymbol{p}^{(k)} + \lambda \Delta\boldsymbol{p}^{(k)}, $$

with

$$ \Delta\boldsymbol{p}^{(k)} = \bar{\boldsymbol{q}}^{(k)} - \boldsymbol{p}^{(k)}. $$

Thus

$$ \boldsymbol{p}^{(k+1)} := \boldsymbol{p}^{(k)} + \lambda \left( \Delta\boldsymbol{p}^{(k)}\right), $$

$$ \boldsymbol{p}^{(k+1)} := \boldsymbol{p}^{(k)} + \lambda \left( \bar{\boldsymbol{q}}^{(k)} - \boldsymbol{p}^{(k)} \right), $$

and finally

$$ \boxed{\boldsymbol{p}^{(k+1)} := \boldsymbol{p}^{(k)} + \lambda \left( \frac{1}{n} \sum_{i=1}^n \boldsymbol{q}_i^{(k)} - \boldsymbol{p}^{(k)} \right).} $$

> The formulation above, based on the average position of the neighbors, is a special case of the more generalized presentation of Laplace smoothing, wherein a normalized weighting factor, $w_i$, is used:

$$ \boldsymbol{p}^{(k+1)} := \boldsymbol{p}^{(k)} + \lambda \sum_{i=1}^n w_i \left( \boldsymbol{q}_i^{(k)} - \boldsymbol{p}^{(k)} \right). $$

> When all weights are equal and normalized by the number of neighbors, $w_i = \frac{1}{n}$, the special case presented in the box above is recovered.

### Example

For a 1D configuration, consider a node with initial position $\boldsymbol{p} = 1.5$ with two neighbors (that never move) with positions $\boldsymbol{q}_1 = 0.0$ and $\boldsymbol{q}_2 = 1.0$ ($\bar{\boldsymbol{q}} = 0.5$).  With $\lambda = 0.3$, the table below shows updates for for position $\boldsymbol{p}$.

Table: Iteration updates of a 1D example.

$k$ | $\bar{\boldsymbol{q}}^{(k)}$ | $\boldsymbol{p}^{(k)}$ | $\Delta\boldsymbol{p}^{(k)} = \bar{\boldsymbol{q}} - \boldsymbol{p}^{(k)}$ | $\lambda \Delta\boldsymbol{p}^{(k)}$
--- | --- | --- | --- | ---
0 | 0.5 | 1.5 | -1 | -0.3
1 | 0.5 | 1.2 | -0.7 | -0.21
2 | 0.5 | 0.99 | -0.49 | -0.147
3 | 0.5 | 0.843 | -0.343 | -0.1029
4 | 0.5 | 0.7401 | -0.2401 | -0.07203
5 | 0.5 | 0.66807 | -0.16807 | -0.050421
6 | 0.5 | 0.617649 | -0.117649 | -0.0352947
7 | 0.5 | 0.5823543 | -0.0823543 | -0.02470629
8 | 0.5 | 0.55764801 | -0.05764801 | -0.017294403
9 | 0.5 | 0.540353607 | -0.040353607 | -0.012106082
10 | 0.5 | 0.528247525 | -0.028247525 | -0.008474257

![laplace_smoothing.png](fig/laplace_smoothing.png)

Figure: Convergence of position $\boldsymbol{p}$ toward $0.5$ as a function of iteration $k$.

## Taubin Smoothing

Taubin smoothing is a two-parameter, two-pass iterative variation of Laplace smoothing.
Specifically with the definitions used in Laplacian smoothing, a second negative parameter $\mu$ is used, where

$$ \mu \in \mathbb{R}^- \subset (-1, 0) \hspace{0.75cm} \lambda \in \mathbb{R}^+ \subset (0, 1) \hspace{0.75cm} \rm{and} \hspace{0.5cm} \lambda < -\mu. $$

The first parameter, $\lambda$, tends to smooth (and shrink) the domain.  The second parameter, $\mu$, tends to expand the domain.

Taubin smoothing is written, for $k = 0$, $k < k_{\rm{max}}$, $k = k+1$, with $k_{\rm{max}}$ typically being even, as

* **First pass** (if $k$ is even):

$$ {\boldsymbol{p}^{(k+1)} := \boldsymbol{p}^{(k)} + \lambda \left( \frac{1}{n} \sum_{i=1}^n \boldsymbol{q}_i^{(k)} - \boldsymbol{p}^{(k)} \right),} $$

* **Second pass** (if $k$ is odd):

$$ {\boldsymbol{p}^{(k+1)} := \boldsymbol{p}^{(k)} + \mu \left( \frac{1}{n} \sum_{i=1}^n \boldsymbol{q}_i^{(k)} - \boldsymbol{p}^{(k)} \right),} $$

> In any second pass (any pass with $k$ odd), the algorithm uses the updated positions from the previous (even) iteration to compute the new positions.  So, the average is taken from the updated neighbor positions rather than the original neighbor positions.  Some presentation of Taubin smoothing do not carefully state the second pass update, and so we emphasize it here.

### Taubin Parameters

We follow the recommendations of Taubin[^Taubin_1995b] for selecting values of $\lambda$ and $\mu$, with specific details noted as follows:  We use the second degree polynomial transfer function,

$$f(k) = (1 - \lambda k) (1 - \mu k),$$

with $0 < k < 2$ as the domain of interest since the eigenvalues of the discrete Laplacian being approximated all are within $[0, 2]$.[^Taubin_1995b]

There is a value of $k$ called the *pass-band frequency*, $k_{\rm{\tiny PB}}$,

$$ k_{\rm{\tiny PB}} := \frac{1}{\lambda} + \frac{1}{\mu}$$

such that $f(k_{\rm{\tiny PB}}) = 1$ for all values of $\lambda$ and $\mu$.

Given that $k > 0$, the pass-band $k_{\rm{\tiny PB}} > 0$ and

$$ \frac{1}{\lambda} + \frac{1}{\mu} > 0 \hspace{0.5cm} \implies \hspace{0.5cm} \lambda < -\mu.  $$

Taubin noted that values of $k_{\rm{\tiny PB}}$ "...from 0.01 to 0.1 produce good results, and all examples shown in this paper were computed with $k_{\rm{\tiny PB}} \approx 0.1$."  Taubin also noted that for $k_{\rm{\tiny PB}} < 1$, choice of $\lambda$ such that $f(1) = -f(2)$ "...ensures a stable and fast filter."

We implement the following default values:

* $k_{\rm{\tiny PB}} = 0.1$
* $\lambda = 0.6307$,
* $\mu = \displaystyle\frac{\lambda}{\lambda \; k_{\rm{\tiny PB}} - 1} = -0.6732$

which provides $f(1) = 0.6179$ and $f(2) = -0.6133$.

## Hierarchical Control

As a default, all nodes in the mesh are free nodes, meaning they are subject to updates in position due to smoothing.

* **Free nodes**

For the purpose of *hierarchical* smoothing, we categorize all nodes
as belonging to one of the following categories.

* **Boundary nodes**
  * Nodes on the **exterior** of the domain and nodes that lie at the **interface** of two different blocks are reclassified from free nodes to boundary nodes.
  * Like free nodes, these nodes are also subject to updates in position due to smoothing.
  * Unlike free nodes, which are influenced by positions of neighboring nodes of any category, boundary nodes are only influenced positions of other boundary nodes, or prescribed nodes (described below).
* **Interior nodes**
  * The free nodes not categorized as boundary nodes are categorized as interior nodes.
  * Interior nodes are influenced by neighboring nodes of all categories.
* **Prescribed nodes**
  * Finally, we may wish to select nodes, typically but not necessarily from boundary nodes, to move to a specific location, often to match the desired shape of a mesh.  These nodes are reclassified as prescribed nodes.
  * Prescribed nodes are not subject to updates in position due to smoothing because they are *a priori* prescribed to reside at a given location.

This classification is shown below in figures.  All nodes in the mesh are categorized as `FREE` nodes:

![free_nodes.png](fig/free_nodes.png)

Nodes that lie on the exterior and/or an interface are categoried as `BOUNDARY` nodes.  The remaining free nodes that are not `BOUNDARY` nodes are `INTERIOR` nodes.

![boundary_and_interior_nodes.png](fig/boundary_and_interior_nodes.png)

Some `INTERIOR` and `BOUNDARY` nodes may be recategorized as `PRESCRIBED` nodes.

![prescribed_nodes.png](fig/prescribed_nodes.png)

Note that this focuses on regular volumetric finite element meshes, and does not apply to certain other meshes.
For example, manifold surface meshes embedded in three dimensions have only interior nodes, so hierarchical control would not apply.

### The `Hierarchy` enum

These three categories, `INTERIOR`, `BOUNDARY`, and `PRESCRIBED`, compose the hierarchical structure of hierarchical smoothing.  Nodes are classified in code with the following `enum`,

```python
class Hierarchy(Enum):
    """All nodes must be categorized as beloning to one, and only one,
    of the following hierarchical categories.
    """

    INTERIOR = 0
    BOUNDARY = 1
    PRESCRIBED = 2
```

### Hierarchical Control

Hierarchical control classifies all nodes in a mesh as belonging to a interior $\mathbb{A}$, boundary $\mathbb{B}$, or prescribed $\mathbb{C}$.  These categories are mutually exclusive.  Any and all nodes must belong to one, and only one, of these three categories.  For a given node $\boldsymbol{p}$, let

* the set of *interior* neighbors be denoted $\boldsymbol{q}_{\mathbb{A}}$,
* the set of *boundary* neighbors be denoted $\boldsymbol{q}_{\mathbb{B}}$, and
* the set of *prescribed* neighbors be denoted $\boldsymbol{q}_{\mathbb{C}}$.

Hierarchical control redefines a node's neighborhood according to the following hierarchical rules:

* for any *interior* node $\boldsymbol{p} \in \mathbb{A}$, nodes $\boldsymbol{q}_{\mathbb{A}}$, $\boldsymbol{q}_{\mathbb{B}}$, and $\boldsymbol{q}_{\mathbb{C}}$ are neighbors; there is no change in the neighborhood,
* for any *boundary* node $\boldsymbol{p} \in \mathbb{B}$, only boundary nodes $\boldsymbol{q}_{\mathbb{B}}$ and prescribed nodes $\boldsymbol{q}_{\mathbb{C}}$ are neighbors; a boundary node neighborhood exludes interior nodes, and
* for any *prescribed* node $\boldsymbol{p} \in \mathbb{C}$, all neighbors of any category are excluded; the prescribed node's position does not change during smoothing.

The following figure shows this concept:

![hierarchy_sets_refactored](fig/hierarchy_sets_refactored.png)

Figure: Classification of nodes into categories of interior nodes $\mathbb{A}$, boundary nodes $\mathbb{B}$, and prescribed nodes $\mathbb{C}$.  Hierarchical relationship: An interior node's smooothing neighbors are nodes of any category, a boundary node's smoothing neighbors are other boundary nodes or other prescribed nodes, and prescribed nodes have no smoothing neighbors.

### Relationship to a `SideSet`

A `SideSet` is a set of nodes on the boundary of a domain, used to prescribe a boundary condition on the finite element mesh.

* A subset of nodes on the boundary nodes is classified as **exterior nodes**.
* A different subset of nodes on the boundary is classified as **interface nodes**.
* A `SideSet` is composed of either exterior nodes or interface nodes.
* Because a node can lie both on the exterior and on an interface, some nodes (shown in red) are included in both the exterior nodes and the interface nodes.

![exterior_interface_nodes.png](fig/exterior_interface_nodes.png)

### Chen Example

Chen[^Chen_2010] used medical image voxel data to create a structured hexahedral mesh.   They noded that the approach generated a mesh with "jagged edges on mesh surface and material interfaces," which can cause numerical artifacts.

Chen used hierarchical Taubin mesh smoothing for eight (8) iterations, with $\lambda = 0.6307$ and $\mu = -0.6732$ to smooth the outer and inner surfaces of the mesh.

## References

[^Sorkine_2005]: Sorkine O. Laplacian mesh processing. Eurographics (State of the Art Reports). 2005 Sep;4(4):1.  [paper](https://doi.org/10.2312/egst.20051044)

[^Taubin_1995a]: Taubin G. Curve and surface smoothing without shrinkage. In *Proceedings of IEEE international conference on computer vision* 1995 Jun 20 (pp. 852-857). IEEE.  [paper](https://ieeexplore.ieee.org/stamp/stamp.jsp?arnumber=466848)

[^Taubin_1995b]: Taubin G. A signal processing approach to fair surface design. In *Proceedings of the 22nd annual conference on Computer graphics and interactive techniques* 1995 Sep 15 (pp. 351-358). [paper](https://dl.acm.org/doi/pdf/10.1145/218380.218473)

[^Chen_2010]: Chen Y, Ostoja-Starzewski M. MRI-based finite element modeling of head trauma: spherically focusing shear waves. Acta mechanica. 2010 Aug;213(1):155-67. [paper](https://link.springer.com/content/pdf/10.1007/s00707-009-0274-0.pdf)
