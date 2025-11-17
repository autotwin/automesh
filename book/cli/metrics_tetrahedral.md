# Tetrahedral Metrics

**Work in progress.**

```sh
automesh metrics tet --help
<!-- cmdrun automesh metrics tet --help -->
```

`automesh` implements the following hexahedral element quality metrics defined in the Verdict report.[^Knupp_2006]

* Maximum edge ratio ${\rm ER}_{\max}$
* Minimum scaled Jacobian ${\rm SJ}_{\min}$
* Maximum skew
* Element volume

A brief description of each metric follows.

## Unit Tests


## Local Numbering Scheme

### Nodes

The local numbering scheme for nodes of a tetrahedral element:

```sh
        3
       /|\
 L3   / | \  L5
     /  |  \
    0---|---2  (horizontal line is L2)
     \  |  /
  L0  \ | / L1
       \|/
        1 

        (vertical line is L4)

where

    L0 = p1 - p0        L3 = p3 - p0
    L1 = p2 - p1        L4 = p3 - p1
    L3 = p0 - p2        L5 = p3 - p2
```

node | connected nodes
:---: | :---:
0 | 1, 2, 3
1 | 0, 2, 3
2 | 0, 1, 3
3 | 0, 1, 2

### Faces

A tetrahedron has four triangular faces.  The faces are typically numbered opposite
to the node they do not contain (e.g., face 0 is opposite to node 0).

From the exterior of the element, view the (0, 1, 3) face and upwarp the remaining faces; the four face normals now point out out of the page.  The local numbering scheme for faces of a tetrahedral element:

```sh
    2-------3-------2
     \  1  / \  0  /
      \   /   \   /
       \ /  2  \ /
        0-------1
         \  3  /
          \   /
           \ /
            2
```

face | nodes
:---: | :---:
0 | 1, 2, 3
1 | 0, 2, 3
2 | 0, 1, 3
3 | 0, 1, 2

## References

[^Knupp_2006]: Knupp PM, Ernst CD, Thompson DC, Stimpson CJ, Pebay PP. The verdict geometric quality library. SAND2007-1751. Sandia National Laboratories (SNL), Albuquerque, NM, and Livermore, CA (United States); 2006 Mar 1. [link](https://www.osti.gov/servlets/purl/901967)
