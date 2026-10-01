# Unit Sphere v2

A sphere is a useful model because, given its radius, the surface area and
volume are known quantities.  The error of any surface that approximates
the analytical surface of the sphere can be easily quantified.

This page builds a unit sphere from a voxel segmentation.  Voxels inside the
sphere have value `1`.  Voxels outside have value `0`.  Marching cubes turns
the segmentation into a triangulated surface.  That surface approximates the
[isosurface](../../../theory/isosurface.md) of the segmentation at the value
0.5, halfway between `0` and `1`.  We measure the triangulated surface against
the exact sphere.

## Related Studies

Two other sections build unit spheres by other methods.

* [Octa-Loop](../../../theory/subdivision.md#octa-loop), on the Subdivision
  page, refines a unit octahedron into a sphere by Loop subdivision.
* [Remesh: Unit Sphere](../../remesh/sphere.md) remeshes a unit sphere of
  1,088 facets.

## Segmentation

The script [`unit_sphere_segmentation.py`](#unit_sphere_segmentationpy) builds
a sphere of radius $n$ voxels.  The sphere sits at the center of a cube of
$2n+1$ voxels per side.  A voxel is inside when its center satisfies
$x^2 + y^2 + z^2 \le n^2$.  An inside voxel has value `1`, and every other
voxel has value `0`.  The array axes are $(x, y, z)$, the order `automesh`
reads from a `.npy` file.

```sh
uv run --with numpy unit_sphere_segmentation.py
```

The script writes five segmentations of increasing resolution: $n$ = 10, 20,
40, 80, and 160.  Each voxel has side length $1/n$ once the sphere is scaled
to radius 1.  The volume column counts the inside voxels and multiplies by
$1/n^3$.  The error,
$(\text{volume} - 4\pi/3) / (4\pi/3)$, compares that volume with the exact
volume $4\pi/3 \approx 4.1888$.

| $n$ | grid | total voxels | inside voxels | volume | error |
| ---: | :---: | ---: | ---: | ---: | ---: |
| 10 | 21×21×21 | 9,261 | 4,169 | 4.1690 | −0.47% |
| 20 | 41×41×41 | 68,921 | 33,401 | 4.1751 | −0.33% |
| 40 | 81×81×81 | 531,441 | 267,761 | 4.1838 | −0.12% |
| 80 | 161×161×161 | 4,173,281 | 2,143,641 | 4.1868 | −0.05% |
| 160 | 321×321×321 | 33,076,161 | 17,155,325 | 4.1883 | −0.01% |

The voxel volume falls short at every $n$ in the table, and the shortfall
shrinks as $n$ grows.

![unit_sphere_voxels.png](unit_sphere_voxels.png)

Figure: The segmentations for $n = 10$ (left), $n = 40$ (middle), and
$n = 160$ (right), in voxel units.  The figure is produced by
[`unit_sphere_figures.py`](#unit_sphere_figurespy).

## Marching Cubes

Marching cubes turns a grid of values into a triangulated surface at one
level of those values.  The script
[`unit_sphere_marching_cubes.py`](#unit_sphere_marching_cubespy) wraps the
implementation in [scikit-image](https://scikit-image.org),
`skimage.measure.marching_cubes`, with the method of Lewiner
*et al.*[^Lewiner2003]  For each segmentation, the script

1. pads the segmentation by one voxel of zeros, so the surface closes,
2. runs marching cubes at level `0.5`, halfway between outside (`0`) and
   inside (`1`),
3. reverses each face, so every triangle winds outward (see
   [Pitfalls](#pitfalls)),
4. translates and scales the vertices to a unit sphere (see
   [Transform](#transform)), and
5. writes a binary STL.

```sh
uv run --with numpy --with scikit-image unit_sphere_marching_cubes.py
```

It then checks each surface.  $v$, $e$, and $f$ count the vertices, edges,
and faces.  The volume is the signed volume that the faces enclose.  The
error compares that volume with the exact volume $4\pi/3 \approx 4.1888$.

| $n$ | $v$ | $e$ | $f$ | volume | error |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 10 | 1,902 | 5,700 | 3,800 | 4.1478 | −0.98% |
| 20 | 7,542 | 22,620 | 15,080 | 4.1699 | −0.45% |
| 40 | 30,150 | 90,444 | 60,296 | 4.1825 | −0.15% |
| 80 | 120,486 | 361,452 | 240,968 | 4.1865 | −0.06% |
| 160 | 482,286 | 1,446,852 | 964,568 | 4.1882 | −0.01% |

The volume created from marching cubes
underestimates the true $4\pi/3$ value at every $n$ in the table.  The surface
also encloses less volume than the voxels it came from.  This pattern
holds at every $n$ from 4 to 160.  Marching cubes cuts each corner of the
voxel staircase with a flat triangle, and every cut removes a small
volume.

The script also runs five surface integrity checks.  All five surfaces give
the same result on every check.

* **Closed.**  No edge is open.  An open edge belongs to one face only.
* **Manifold.**  No edge is pinched.  A pinched edge belongs to more than two
  faces.
* **Oriented.**  No two faces traverse the same edge in the same direction.
* **Connected.**  The surface is one piece.
* **Euler characteristic.**  $\chi = v - e + f = 2$.

For a closed, connected, orientable surface, $\chi = 2 - 2g$, where $g$ is
the genus, the number of handles.  Here $\chi = 2$, so $g = 0$, and each
surface is topologically a sphere.

![unit_sphere_marching_cubes.png](unit_sphere_marching_cubes.png)

Figure: The marching cubes surfaces for $n = 10$ (left), $n = 40$ (middle),
and $n = 160$ (right), scaled to the unit sphere.  
A binary mask places every vertex at the
midpoint of a cube edge, so the surface cannot round off the steps.  The
steps shrink as $n$ grows, but the rings around each pole remain visible
even at $n = 160$.  The figure is produced by
[`unit_sphere_figures.py`](#unit_sphere_figurespy).

### Convergence


The script
[`unit_sphere_figures.py`](#unit_sphere_figurespy) also computes both
volumes at every $n$ from 4 to 160, 157 values in all.

![unit_sphere_convergence.png](unit_sphere_convergence.png)

Figure: The volume (left) and the magnitude of its error (right) against
$n$, for the voxels and for the marching cubes surface.  Thin lines connect
every $n$ from 4 to 160.  Markers show the five values of $n$ in the tables.
On the left, the dotted line marks the exact volume $4\pi/3$, and a few
values at $n \le 7$ fall off the scale.  On the right, the gray guides have
slopes of −1 and −2.

Both volumes settle onto $4\pi/3$, but neither settles smoothly.  The **voxel
error** changes sign 46 times, and 25 of the 157 voxel volumes exceed
$4\pi/3$.  The **marching cubes error** changes sign 24 times, and 12 of its
volumes exceed $4\pi/3$.  The five values in the tables just happened to fall
short.

Under the oscillation, both errors fall toward zero.  A least-squares fit of
$\log|\text{error}|$ against $\log n$ gives a slope of −1.71 for the voxels
and −1.76 for marching cubes.  Both lie between the two guides.  The
marching cubes volume stays below the voxel volume at all 157 values.

### Why Lewiner?

Lorensen and Cline introduced marching cubes in 1987.[^Lorensen1987]  Their
method visits each cube of eight neighboring samples.  It classifies each
corner as inside or outside, which gives 256 cases.  Symmetry reduces those
to 15 base cases, and each base case has one fixed triangulation.

Some cases are ambiguous.  On a face with two inside corners on one
diagonal and two outside corners on the other, the corners alone cannot say
whether the inside corners connect across the face.  An interior ambiguity
arises the same way inside a cube.  A fixed table picks one answer without
looking at the data.  Neighboring cubes can then disagree, and the surface
can take the wrong topology.

Chernyaev extended the table to 33 cases.[^Chernyaev1995]  His table
resolves each ambiguity with the trilinear interpolant of the eight corner
values, the smooth function the samples define inside the cube.  Lewiner
*et al.* gave an efficient and complete implementation of that
table.[^Lewiner2003]
[scikit-image](https://scikit-image.org/docs/stable/api/skimage.measure.html#skimage.measure.marching_cubes)
uses the Lewiner method by default.[^skimage]

The list below separates what the Lewiner method adds from what the
scikit-image implementation gives either method.  Each item says where the
evidence comes from.

**What the Lewiner method adds**

1. **Topology that matches the data.**  The 33-case table resolves face and
   interior ambiguities with the trilinear interpolant.  The surface then
   has the topology of the interpolant's level set, so neighboring cubes
   always agree.  *From the paper.*[^Lewiner2003]

**What the scikit-image implementation gives either method**

2. **Watertight.**  No edge is open, at any $n$.  The one-voxel pad keeps
   the surface closed where the sphere touches the edge of the grid.
   *Measured here.*
3. **Manifold.**  Every edge belongs to exactly two faces, at any $n$.
   *Measured here.*
4. **Consistently oriented.**  No two faces traverse an edge in the same
   direction, and the signed volume is positive once the faces are reversed.
   *Measured here.*
5. **An indexed mesh.**  The output is an array of vertices and an array of
   faces that index into it.  Neighboring triangles share vertices, so the
   mesh needs no welding.  *From the API.*
6. **Normals and values.**  The call also returns a normal at each vertex,
   from the gradient of the data, and the data value there.  *From the API.*

For the unit sphere the two methods agree.  At every $n$ they return the
same triangles, and they take the same time.  The time below is the fastest
of five runs, on the machine that built this page.

| $n$ | Lewiner (ms) | Lorensen (ms) | same triangles |
| ---: | ---: | ---: | :---: |
| 10 | 0.49 | 0.49 | yes |
| 20 | 1.99 | 2.03 | yes |
| 40 | 9.19 | 9.41 | yes |
| 80 | 48.10 | 49.52 | yes |
| 160 | 285.98 | 296.59 | yes |

The sphere reaches no ambiguous case, so the Lewiner method has nothing to
resolve.  On data that do reach such cases, the two methods can return
surfaces of different topology.  Newman and Yi illustrate each ambiguous
case and survey the methods that resolve them.[^Newman2006]

### Pitfalls

1. **Winding.**  scikit-image returns vertices in array-axis order.  The
   segmentation has axes $(x, y, z)$, so the vertices come back as
   $(x, y, z)$ too.  With the default `gradient_direction="descent"`, the
   returned normals point outward, but every triangle winds inward.  The
   signed volume is then negative.  The script reverses each face, which
   makes the winding agree with the normals.
2. **Exact ties on a binary mask.**  On a mask of 0 and 1, level 0.5 is
   exactly the saddle value of an ambiguous face.  The Lewiner method can
   then pinch an edge between four faces.  A level slightly off 0.5, such
   as 0.49, avoids the tie.  The unit sphere has no pinched edge at any
   $n$, so this page keeps level 0.5.

## Transform

Marching cubes returns vertices in voxel units.  After the one-voxel pad, the
sphere center sits at $(n+1, n+1, n+1)$.  Two operations map the surface to a
unit sphere at the origin.

1. **Translate** by $-(n+1)$.  The shift is a whole number of voxels, so it
   is exact.
2. **Scale** by $1/n$.

`automesh` applies the same two operations in the other order.  Its
`--xscale` and `--xtranslate` options scale first and translate second.  The
same map in the `automesh` order scales by $1/n$ and then translates by
$-(n+1)/n$.  Both orders give the same coordinates.

The vertex radii show how close the surface comes to the unit sphere.  The
coefficient of variation, CoV $= \text{std} / \text{mean}$, measures how
much the radii spread.  A perfect sphere has CoV $= 0$.

| $n$ | $r$ min | $r$ max | $r$ mean | $r$ CoV |
| ---: | ---: | ---: | ---: | ---: |
| 10 | 0.9552 | 1.0500 | 0.9993 | 2.15% |
| 20 | 0.9763 | 1.0250 | 0.9991 | 1.01% |
| 40 | 0.9878 | 1.0125 | 0.9996 | 0.51% |
| 80 | 0.9938 | 1.0063 | 0.9999 | 0.26% |
| 160 | 0.9969 | 1.0031 | 1.0000 | 0.13% |

The mean radius stays within 0.1% of 1 at every $n$, so a scale of $1/n$
leaves no bias.  The largest radius is always $1 + 0.5/n$, at the six poles.
There a vertex sits halfway between the last inside voxel, at radius $n$, and
the first outside voxel, at radius $n + 1$.

The CoV halves each time $n$ doubles, so the spread of the radii falls as
$1/n$.  That is first order, slower than the volume error in
[Convergence](#convergence).  The volume averages over the whole surface, and
the staircase steps partly cancel in that average.  The radius spread sees
every step.

## Surface Comparison

### Hausdorff Distance

The [Hausdorff distance](https://mathworld.wolfram.com/HausdorffDistance.html)
between two surfaces is the farthest that any point
of either one lies from the other.  For a surface $M$ and the unit sphere
$S$,

$$
d_H(M, S) = \max \left\{ \sup_{p \in M} d(p, S), \; \sup_{q \in S} d(q, M) \right\}.
$$

The script [`unit_sphere_comparison.py`](#unit_sphere_comparisonpy) computes the
first term exactly.  A point $p$ lies $d(p, S) = \big| |p| - 1 \big|$ from
the unit sphere.  Over a triangle, $|p|$ is convex.  Its largest value sits at
a vertex, and its smallest sits at the point of the triangle closest to the
origin.[^Ericson2005]  The largest $|p| - 1$ over all triangles gives the
outward distance.  The largest $1 - |p|$ gives the inward distance.

The second term needs no computation.  Each surface is closed and encloses
the origin.  The ray from the origin through a sphere point $q$ therefore
crosses $M$ at some point $p$.  Then
$d(q, M) \le |q - p| = \big| |p| - 1 \big| \le \sup_{p \in M} d(p, S)$.
The second term never exceeds the first, so the Hausdorff distance is the
larger of the outward and inward distances.

```sh
uv run --with numpy unit_sphere_comparison.py
```

| $n$ | $f$ | outward | inward | $d_H$ | $n \, d_H$ |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 10 | 3,800 | 0.05000 | 0.04737 | 0.05000 | 0.500 |
| 20 | 15,080 | 0.02500 | 0.02436 | 0.02500 | 0.500 |
| 40 | 60,296 | 0.01250 | 0.01234 | 0.01250 | 0.500 |
| 80 | 240,968 | 0.00625 | 0.00621 | 0.00625 | 0.500 |
| 160 | 964,568 | 0.00312 | 0.00312 | 0.00312 | 0.500 |

The Hausdorff distance is exactly $0.5/n$ at every $n$, half a voxel.  The
outward distance sets it, at the six poles (see [Transform](#transform)).
The inward distance approaches it from below.  The Hausdorff distance falls
as $1/n$, first order, like the radius CoV.  The triangle count $f$ grows as
$n^2$, so in terms of triangles the distance falls only as $f^{-1/2}$.

### Surface Area

The volume converges, as the [Convergence](#convergence) section shows.  The
surface area does not converge.  The script
[`unit_sphere_comparison.py`](#unit_sphere_comparisonpy) adds up the areas of
the marching cubes triangles.  The error compares that area with the exact
area $4\pi \approx 12.5664$.

| $n$ | area | error |
| ---: | ---: | ---: |
| 10 | 13.7204 | +9.18% |
| 20 | 13.6324 | +8.48% |
| 40 | 13.6566 | +8.68% |
| 80 | 13.6608 | +8.71% |
| 160 | 13.6673 | +8.76% |

The voxel staircase itself would overstate the area by 50% at any $n$.
Marching cubes cuts the voxel staircase corners, which removes most of that excess.
Howver, the surface area
still settles near 8.8% too large.

Smaller voxels make smaller steps, but
the triangles keep the same few orientations relative to the true surface.
So the Hausdorff distance falls to zero while the area error does not.  A
surface can come arbitrarily close to the sphere and still have the wrong
area.

### Triangle Quality

The script [`unit_sphere_comparison.py`](#unit_sphere_comparisonpy) also runs
`automesh metrics` on each surface.  The
[Triangular Metrics](../../../theory/metrics_triangular.md) page defines its
measures.

A binary mask puts every vertex at the midpoint of a cube edge.  So marching
cubes can make only a few triangle shapes.  Grouped by maximum edge ratio and
minimum scaled Jacobian, the triangular surfaces created by marching cubes
produce exactly **five shapes**.  The table gives the fraction of triangles
of each shape.

| shape | angles | edge ratio | scaled Jacobian | $n = 10$ | $n = 20$ | $n = 40$ | $n = 80$ | $n = 160$ |
| :--- | :---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| equilateral | 60°, 60°, 60° | 1.000 | 1.000 | 14.74% | 14.80% | 14.30% | 13.99% | 13.98% |
| right isosceles | 45°, 45°, 90° | 1.414 | 0.816 | 24.63% | 25.15% | 25.04% | 24.92% | 24.85% |
| right | 35.26°, 54.74°, 90° | 1.732 | 0.667 | 31.58% | 30.56% | 32.08% | 33.13% | 33.20% |
| obtuse isosceles | 30°, 30°, 120° | 1.732 | 0.577 | 14.53% | 14.75% | 14.29% | 13.98% | 13.98% |
| 30-60-90 | 30°, 60°, 90° | 2.000 | 0.577 | 14.53% | 14.75% | 14.29% | 13.98% | 13.98% |

**Refinement does not change the quality.**  The same five shapes appear at every
$n$, in nearly the same proportions.  More than a quarter of the triangles
have the worst scaled Jacobian, $1/\sqrt{3} \approx 0.577$, with a smallest
angle of 30°.  No triangle is worse.

**Smaller voxels shrink the triangles but
never reshape them.**  A histogram of any measure would show only these five
spikes, so the table is shown instead of a histogram.

## Mesh

### Sculpt

The [Subdivision](../../../theory/subdivision.md#sculpt-baseline) page meshes
its Octa-Loop spheres with Sculpt.  At levels 3 to 7, it reports a grid of
26×26×26 cells, 7,731 nodes, and 6,672 elements.  Here Sculpt meshes the
marching cubes surfaces on the same grid.

The Subdivision baseline runs Sculpt through Cubit's `sculpt parallel`
command, which chooses the grid itself.  For Octa-Loop level 3, Cubit places 26
cells in each direction across a box of $\pm 1.240409$, a cell size of about
0.0954.  Standalone Sculpt with those options reproduces the baseline exactly,
at 7,731 nodes and 6,672 elements.  The same options mesh each marching cubes
surface.

```sh
B=1.240409
GRID="-x 26 -y 26 -z 26 -t -$B -u -$B -v -$B -q $B -r $B -s $B -SS 2"
# octa_loop03.stl downloads from the Refinement table of the Subdivision page
sculpt -j 1 -stl octa_loop03.stl $GRID -e unit_sphere_sculpt_octa03
for n in 010 020 040 080 160; do
  sculpt -j 1 -stl unit_sphere_mc_n$n.stl $GRID -e unit_sphere_sculpt_n$n
done
```

Left to choose its own grid, Cubit sizes the cells to the small marching cubes
facets.  For $n = 40$ it picks 73×73×73 cells and makes 180,201 elements.
Fixing the grid keeps the meshes comparable with the baseline.

The quality is the minimum scaled Jacobian, as Sculpt reports it.

| input | nodes | elements | minimum quality | mean quality |
| :--- | ---: | ---: | ---: | ---: |
| [Octa-Loop level 3](../../../theory/subdivision.md#refinement) (baseline) | 7,731 | 6,672 | 0.343 | 0.888 |
| marching cubes, $n = 10$ | 8,007 | 6,912 | 0.388 | 0.885 |
| marching cubes, $n = 20$ | 7,827 | 6,768 | 0.404 | 0.900 |
| marching cubes, $n = 40$ | 7,827 | 6,768 | 0.420 | 0.902 |
| marching cubes, $n = 80$ | 7,827 | 6,768 | 0.401 | 0.901 |
| marching cubes, $n = 160$ | 7,827 | 6,768 | 0.427 | 0.901 |

From $n = 20$ on, every surface gives the same 7,827 nodes and 6,768
elements, 96 more of each than the baseline, or about 1.4%.  The worst
element is better than the baseline's at every $n$.

![unit_sphere_sculpt.png](unit_sphere_sculpt.png)

Figure: The Sculpt meshes of Octa-Loop level 3 (left), the marching cubes
surface for $n = 10$ (middle), and for $n = 160$ (right), on the same grid.
Each element is painted by its Minimum Scaled Jacobian, on a fixed scale from
0 to 1.  In all three, the best elements form bands of regular hexes, and the
worst sit where those bands meet.  The figure is produced by
[`unit_sphere_figures.py`](#unit_sphere_figurespy).

The difference between the middle and right panels comes from the size of a
voxel step against the size of a Sculpt cell, about 0.095.

* At $n = 10$, a voxel is 0.1 wide, so each step is about as large as a cell.
  The mesh boundary follows the steps, and the sphere looks lumpy.
* At $n = 160$, a voxel is 0.00625 wide, so a cell spans about 15 voxels.  The
  steps are far smaller than the elements, and the mesh looks like the
  baseline.
* From $n = 20$ on, a cell spans about two voxels or more, and the mesh stops
  changing.

![unit_sphere_sculpt_cut.png](unit_sphere_sculpt_cut.png)

Figure: A cut through the middle of the same three meshes, at $z = 0$, on
the same 0 to 1 scale.  Only the elements whose centers lie below the plane
are drawn.  The Sculpt grid has a cell boundary at $z = 0$, so the interior
of the cut is flat.  Near the surface, smoothing moves some nodes off the
plane, and a few elements cross it: 72, 60, and 48 from left to right.  The
interior is a regular grid of elements near 1.0.  The low values form a ring
one element deep at the boundary.  The figure is produced by
[`unit_sphere_figures.py`](#unit_sphere_figurespy).

The run used Sculpt 16.08, from Cubit 16.08.  `automesh` 0.4.8 cannot read
Sculpt's Exodus files.  Sculpt writes single-precision coordinates, and
`conspire` 0.7.9 reads only double precision.[^conspire226]  So the script
[`unit_sphere_mesh.py`](#unit_sphere_meshpy) rewrites each Sculpt mesh as an
Abaqus `.inp` file, which `automesh` reads.  Exodus and Abaqus number the eight
nodes of a hex the same way, so the rewrite keeps every element intact.  For
the baseline mesh, `automesh metrics` on the rewritten file gives the same
minimum (0.343) and mean (0.888) as Sculpt.

[^conspire226]: conspire issue #226, "Exodus reader: accept single-precision
    (float) coordinates, as Sculpt writes."
    <https://github.com/mrbuche/conspire.rs/issues/226>
