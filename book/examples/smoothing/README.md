# Smoothing

All degrees of freedom in the mesh must be in one, and only one, of the
following *smoothing categories*:

* Prescribed
  * Homogeneous
  * Inhomogeneous
* Free
  * Exterior
  * Interface
  * Interior

![../unit_tests/double_x.png](../unit_tests/double_x.png)

Figure: Two element test problem.

Table: Nodal coordinates 1-12, with x, y, z, degrees of freedom.

node | `x` | `y` | `z` | `->` |  |  | dof
:---: | :---: | :---: | :---: | :---: | :---: | :---: | :---:
1  | 0.0 | 0.0 | 0.0 | | 1 | 2 | 3
2  | 1.0 | 0.0 | 0.0 | | 4 | 5 | 6
3  | 2.0 | 0.0 | 0.0 | | 7 | 8 | 9
4  | 0.0 | 1.0 | 0.0 | | 10 | 11 | 12
5  | 1.0 | 1.0 | 0.0 | | 13 | 14 | 15
6  | 2.0 | 1.0 | 0.0 | | 16 | 17 | 18
7  | 0.0 | 0.0 | 1.0 | | 19 | 20 | 21
8  | 1.0 | 0.0 | 1.0 | | 22 | 23 | 24
9  | 2.0 | 0.0 | 1.0 | | 25 | 26 | 27
10 | 0.0 | 1.0 | 1.0 | | 28 | 29 | 30
11 | 1.0 | 1.0 | 1.0 | | 31 | 32 | 33
12 | 2.0 | 1.0 | 1.0 | | 34 | 36 | 36

Table. The node neighbors.

node | neighbor node(s)
:---: | :---:
1  | 2, 4, 7
2  | 1, 3, 5, 8
3  | 2, 6, 9
4  | 1, 5, 10
5  | 2, 4, 6, 11
6  | 3, 5, 12
7  | 1, 8, 10
8  | 2, 7, 9, 11
9  | 3, 8, 12
10 | 4, 7, 11
11 | 5, 8, 10, 12
12 | 6, 9, 11
