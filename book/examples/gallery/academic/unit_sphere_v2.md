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

The script `unit_sphere_segmentation.py` builds
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

<figure>
    <img src="unit_sphere_v2_voxels.png" alt="voxel segmentations of the sphere for n = 10, 40, and 160" />
    <figcaption>The segmentations for $n = 10$ (left), $n = 40$ (middle), and $n = 160$ (right), in voxel units.  The figure is produced by <code>unit_sphere_v2_meshes.py</code>.</figcaption>
</figure>


## Marching Cubes

Marching cubes turns a grid of values into a triangulated surface at one
level of those values.  The command `automesh mesh tri --cubes marching` runs
the method of Lewiner *et al.*[^Lewiner2003]  For each segmentation, `automesh`

1. pads the segmentation by one voxel of zeros, so the surface closes,
2. runs marching cubes at level `0.5`, halfway between outside (`0`) and
   inside (`1`),
3. winds every triangle outward, and
4. scales and translates the vertices to a unit sphere (see
   [Transform](#transform)).

The command below meshes the $n = 10$ segmentation.  The scale is $1/n$ and
the translation is $-(n + 0.5)/n$ in each direction.

```sh
automesh mesh tri -i unit_sphere_n010.npy -o unit_sphere_mc_n010.stl --cubes marching \
    --xscale 0.1 --yscale 0.1 --zscale 0.1 \
    --xtranslate -1.05 --ytranslate -1.05 --ztranslate -1.05
```

The loop below makes the surface for each of the five segmentations.

```sh
for n in 10 20 40 80 160; do
  s=$(python3 -c "print(1/$n)")
  t=$(python3 -c "print(-($n + 0.5)/$n)")
  automesh mesh tri -i unit_sphere_n$(printf %03d $n).npy \
    -o unit_sphere_mc_n$(printf %03d $n).stl --cubes marching \
    --xscale $s --yscale $s --zscale $s \
    --xtranslate $t --ytranslate $t --ztranslate $t
done
```

The surfaces below have the following counts.  $v$, $e$, and $f$ count the
vertices, edges, and faces.  The volume is the signed volume that the faces enclose.  The
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

Five integrity checks run on the five surfaces.  All five surfaces give the
same result on every check.

* **Closed.**  No edge is open.  An open edge belongs to one face only.
* **Manifold.**  No edge is pinched.  A pinched edge belongs to more than two
  faces.
* **Oriented.**  No two faces traverse the same edge in the same direction.
* **Connected.**  The surface is one piece.
* **Euler characteristic.**  $\chi = v - e + f = 2$.

For a closed, connected, orientable surface, $\chi = 2 - 2g$, where $g$ is
the genus, the number of handles.  Here $\chi = 2$, so $g = 0$, and each
surface is topologically a sphere.

<figure>
    <img src="unit_sphere_v2_surfaces.png" alt="marching cubes surfaces of the sphere for n = 10, 40, and 160" />
    <figcaption>The marching cubes surfaces for $n = 10$ (left), $n = 40$ (middle), and $n = 160$ (right), scaled to the unit sphere.  A binary mask places every vertex at the midpoint of a cube edge, so the surface cannot round off the steps.  The steps shrink as $n$ grows, but the rings around each pole remain visible even at $n = 160$.  The figure is produced by <code>unit_sphere_v2_meshes.py</code>.</figcaption>
</figure>


### Convergence


The script
`unit_sphere_v2_convergence.py` computes both
volumes at every $n$ from 4 to 160, 157 values in all.  It runs
`automesh mesh tri --cubes marching` once per $n$.

<figure>
    <img src="unit_sphere_v2_convergence.png" alt="volume and volume error of the voxel sphere against n" />
    <figcaption>The volume (left) and the magnitude of its error (right) against $n$, for the voxels and for the marching cubes surface.  Thin lines connect every $n$ from 4 to 160.  Markers show the five values of $n$ in the tables. On the left, the dotted line marks the exact volume $4\pi/3$, and a few values at $n \le 7$ fall off the scale.  On the right, the gray guides have slopes of −1 and −2.</figcaption>
</figure>


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
table.[^Lewiner2003]  `automesh` uses the Lewiner method.

The Lewiner method is better than the original table because neighboring
cubes always agree.  Each ambiguity is resolved from the data, so two cubes
that share a face choose the same connection across it.  The surface then
has the topology of the interpolant's level set.

The sphere reaches no ambiguous case, so the Lewiner method has nothing to
resolve here.  On data that do reach such cases, the original table can
return a surface of different topology.  Newman and Yi illustrate each
ambiguous case and survey the methods that resolve them.[^Newman2006]

### Pitfalls

1. **Exact ties on a binary mask.**  On a mask of 0 and 1, level 0.5 is
   exactly the saddle value of an ambiguous face.  The Lewiner method can
   then pinch an edge between four faces.  A level slightly off 0.5, such
   as 0.49, avoids the tie.  The unit sphere has no pinched edge at any
   $n$, so this page keeps level 0.5.

## Transform

Marching cubes returns vertices in voxel units.  Voxel $i$ spans $[i, i+1]$,
so the voxel at index $n$ spans $[n, n+1]$.  That voxel is the center of the
sphere, so the center sits at its midpoint, $(n+0.5, n+0.5, n+0.5)$.  Two
operations map the surface to a unit sphere at the origin.

1. **Scale** by $1/n$, which makes the radius 1.
2. **Translate** by $-(n+0.5)/n$, which moves the center $(n+0.5)/n$ to the
   origin.

The `--xscale` and `--xtranslate` options of `automesh` scale first and
translate second, in that order.  The translation must use $n + 0.5$.  A
translation of $-(n+1)/n$ treats the center as the upper face of voxel $n$,
at $n + 1$.  It leaves the surface off center by $0.5/n$ in each direction.

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

The script `unit_sphere_comparison.py` computes the
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
`unit_sphere_comparison.py` adds up the areas of
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

The script `unit_sphere_comparison.py` also runs
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

<figure>
    <img src="unit_sphere_v2_sculpt.png" alt="Sculpt meshes of Octa-Loop level 3 and the n = 10 and n = 160 marching cubes surfaces" />
    <figcaption>The Sculpt meshes of Octa-Loop level 3 (left), the marching cubes surface for $n = 10$ (middle), and for $n = 160$ (right), on the same grid. Each hex is painted by its Minimum Scaled Jacobian, on a fixed scale from 0 to 1.  In all three, the best hexes form bands, and the worst sit where those bands meet.  The figure is produced by <code>unit_sphere_v2_meshes.py</code>.</figcaption>
</figure>


The difference between the middle and right panels comes from the size of a
voxel step against the size of a Sculpt cell, about 0.095.

* At $n = 10$, a voxel is 0.1 wide, so each step is about as large as a cell.
  The mesh boundary follows the steps, and the sphere looks lumpy.
* At $n = 160$, a voxel is 0.00625 wide, so a cell spans about 15 voxels.  The
  steps are far smaller than the elements, and the mesh looks like the
  baseline.
* From $n = 20$ on, a cell spans about two voxels or more, and the mesh stops
  changing.

<figure>
    <img src="unit_sphere_v2_sculpt_cut.png" alt="the same three Sculpt meshes cut at z = 0" />
    <figcaption>A cut through the middle of the same three meshes, at $z = 0$, on the same 0 to 1 scale.  Every hex is clipped at the plane, and the lower half stays.  The Sculpt grid has a cell boundary at $z = 0$, so the interior of the cut is flat.  Near the surface, smoothing moves some nodes off the plane, so 66, 60, and 42 hexes cross it, from left to right.  The interior is a regular grid of hexes near 1.0.  The low values form a ring one hex deep at the boundary.  The figure is produced by <code>unit_sphere_v2_meshes.py</code>.</figcaption>
</figure>


The run used Sculpt 16.08, from Cubit 16.08.  `automesh` reads Sculpt's Exodus
files directly, and `automesh metrics` gives the same minimum (0.343) and mean
(0.888) as Sculpt for the baseline mesh.

### `automesh`

`automesh mesh hex` meshes the $n = 10$ marching cubes surface in two ways.
The uniform mesh uses the Sculpt cell size, $2 \times 1.240409 / 26 \approx
0.095416$, so its lattice matches the Sculpt grid.  The adaptive mesh uses the
defaults.

```sh
automesh mesh hex -i unit_sphere_v2_n010.stl -o uniform.inp \
  -u 0.095416 --metrics uniform.csv
automesh mesh hex -i unit_sphere_v2_n010.stl -o adaptive.inp \
  --metrics adaptive.csv
```

<figure id="fig-v2-meshes">
    <img src="unit_sphere_v2_meshes.png" alt="the n = 10 marching cubes surface and its uniform and adaptive automesh meshes" />
    <figcaption>(left) marching cubes surface mesh <code>unit_sphere_v2_n010.stl</code>, (center) <code>automesh</code> uniform, (right) <code>automesh</code> adaptive.</figcaption>
</figure>

<figure>
    <img src="unit_sphere_v2_meshes_cut.png" alt="the same surface and meshes cut at z = 0" />
    <figcaption>Cut plane $z = 0$ of meshes in <a href="#fig-v2-meshes">Figure</a>.</figcaption>
</figure>

<figure>
    <img src="unit_sphere_v2_quality.png" alt="quality histograms of the Sculpt, uniform, and adaptive meshes" />
    <figcaption>Quality measures for Sculpt (baseline) and <code>automesh</code> uniform and adaptive, both without prior smoothing of the marching cubes surface.</figcaption>
</figure>

### Taubin Smoothing

Taubin smoothing rounds the voxel steps of the $n = 10$ surface.  This section
measures how the number of smoothing iterations changes the Minimum Scaled
Jacobian of the uniform and adaptive meshes.

`automesh smooth` applies Taubin smoothing with its defaults.  The pass band is
0.1 and the scale is $\lambda = 0.6307$, which gives $\mu = -0.6732$.  Each
iteration is one Laplacian step.  Odd-numbered iterations shrink the surface
with $\lambda$.  Even-numbered iterations inflate it with $\mu$.  The sweep runs
every integer from 0 to 100 iterations, every tenth from 110 to 300, and every
fiftieth from 350 to 1,000.  Each smoothed surface gets a uniform mesh with the
Sculpt cell size and an adaptive mesh with the defaults.

```sh
automesh smooth -i unit_sphere_v2_n010.stl -o smooth.stl --iterations 260
automesh mesh hex -i smooth.stl -o uniform.inp -u 0.095416 --metrics uniform.csv
automesh mesh hex -i smooth.stl -o adaptive.inp --metrics adaptive.csv
```

<figure id="fig-v2-smooth-sweep">
    <img src="unit_sphere_v2_smooth_sweep.png" alt="minimum scaled Jacobian and hex count of the uniform and adaptive meshes against the number of Taubin smoothing iterations" />
    <figcaption>Minimum Scaled Jacobian (top) and hex count (bottom) of the <code>automesh</code> uniform and adaptive meshes of the $n = 10$ surface, against the number of Taubin smoothing iterations.  Stars mark the peak of each series up to 300 iterations.  Each peak is also the highest value of the whole sweep.  The vertical line at 300 iterations marks a surface volume 6.8% above the sphere's.  The x-axis is linear to 10 iterations and logarithmic beyond.</figcaption>
</figure>

**Uniform.**  The minimum rises with scatter from 0.161 at 0 iterations to
0.613 at 260 iterations.  The mean rises from 0.827 to 0.901.  Up to 300
iterations the hex count stays between 5,747 and 6,150, against 6,024 without
smoothing.  The peak is a plateau: 0.613, 0.612, and 0.612 at 260, 270, and 280
iterations, against 0.574 and 0.575 at 240 and 250.  The minimum then falls to
0.483 at 300 iterations and scatters between 0.459 and 0.510 up to 1,000.

**Adaptive.**  The hex count falls from 8,243 at 0 iterations to 243 at 1
iteration.  From 34 to 700 iterations it stays at 111 hexes.  The minimum peaks
at 0.564 at 11 iterations, on 207 hexes.  The peak is a spike, not a plateau.
The minimum is 0.367 at 10 iterations and 0.395 at 12.  The next highest minimum
is 0.544 at 51 iterations, on 111 hexes.  The 111-hex meshes start at 0.323 at
34 iterations and scatter between 0.299 and 0.544 up to 700.

**Surface drift.**  The defaults inflate this surface, because $|\mu|$ exceeds
$\lambda$.  The mean vertex radius grows from 0.9992 at 0 iterations to 1.0207 at
260 and 1.0862 at 1,000.  The enclosed volume is 4.126 at 11 iterations, 1.5%
below $4\pi/3 \approx 4.1888$.  It is 4.428 at 260 iterations, 5.7% above, and
5.322 at 1,000, 27.1% above.  The peak search therefore stops at 300
iterations.

**Odd and even counts.**  An odd count ends on a shrink step and an even count
on an inflate step, so neighboring counts give different surfaces.  The
enclosed volume is 4.115 at 1 iteration and 4.149 at 2.  The adaptive minimum is
0.328 at 1 iteration and 0.085 at 2.  The uniform minimum is 0.223 at 3
iterations and 0.266 at 4.

None of the 270 meshes has an inverted hex.

<figure id="fig-v2-smooth-uniform">
    <img src="unit_sphere_v2_smooth_uniform.png" alt="the smoothed n = 10 surface, its uniform mesh, and the mesh cut at z = 0, after 260 Taubin smoothing iterations" />
    <figcaption>After 260 Taubin smoothing iterations, the peak of the uniform minimum up to 300 iterations (0.613, against 0.161 without smoothing): the smoothed $n = 10$ surface (left), the uniform mesh (center), and the mesh cut at $z = 0$ (right).</figcaption>
</figure>

<figure id="fig-v2-smooth-adaptive">
    <img src="unit_sphere_v2_smooth_adaptive.png" alt="the smoothed n = 10 surface, its adaptive mesh, and the mesh cut at z = 0, after 11 Taubin smoothing iterations" />
    <figcaption>After 11 Taubin smoothing iterations, the peak of the adaptive minimum up to 300 iterations (0.564, against 0.086 without smoothing): the smoothed $n = 10$ surface (left), the adaptive mesh (center), and the mesh cut at $z = 0$ (right).</figcaption>
</figure>

### Quality After Smoothing

The figure below compares the two best meshes of the sweep with the same meshes
before smoothing and with the Sculpt baseline.  The uniform mesh comes from the
surface smoothed 260 times, with a minimum of 0.613.  The adaptive mesh comes from
the surface smoothed 11 times, with a minimum of 0.564.  Without smoothing, the
minima are 0.161 and 0.086.  The Sculpt minimum is 0.388.

<figure>
    <img src="unit_sphere_v2_smooth_quality.png" alt="quality histograms of the best uniform and adaptive automesh meshes after Taubin smoothing" />
    <figcaption>Quality measures for the <code>automesh</code> uniform mesh after 260 Taubin smoothing iterations and the adaptive mesh after 11, the best Minimum Scaled Jacobian of each, against the same meshes without smoothing (faint lines) and the Sculpt mesh for $n = 10$.  The figure is produced by <code>unit_sphere_v2_smooth.py</code>.</figcaption>
</figure>

| input | nodes | elements | minimum quality | mean quality |
| :--- | ---: | ---: | ---: | ---: |
| <span style="color:#eb6834">Octa-Loop level 3 (baseline)</span> | <span style="color:#eb6834">7,731</span> | <span style="color:#eb6834">6,672</span> | <span style="color:#eb6834">0.343</span> | <span style="color:#eb6834">0.888</span> |
| <span style="color:#eb6834">marching cubes, $n = 10$</span> | <span style="color:#eb6834">8,007</span> | <span style="color:#eb6834">6,912</span> | <span style="color:#eb6834">0.388</span> | <span style="color:#eb6834">0.885</span> |
| automesh uniform, before smoothing | 7,035 | 6,024 | 0.161 | 0.827 |
| automesh adaptive, before smoothing | 10,016 | 8,243 | 0.086 | 0.694 |
| automesh uniform, best result with smoothing (260 iterations) | 7,120 | 6,121 | 0.613 | 0.901 |
| automesh adaptive, best result with smoothing (11 iterations) | 288 | 207 | 0.564 | 0.793 |

The quality is the minimum scaled Jacobian, as in the Sculpt table.  The first two
rows, the Sculpt meshes of the [Sculpt](#sculpt) section, are the reference.
Smoothing raises the uniform minimum from 0.161 to 0.613 and the adaptive minimum
from 0.086 to 0.564.  Of the two best meshes, the uniform mesh has the higher
minimum, by 0.049, and the higher mean, by 0.108.

The adaptive mesh has 207 hexes, 3.4% of the uniform count, so its histograms
sit about a factor of 30 lower.  Compare the shapes of the curves, not their
heights.

The two minima differ in how robust they are.  The uniform value belongs to a
plateau: 0.613, 0.612, and 0.612 at 260, 270, and 280 iterations.  The adaptive
value belongs to a spike: 0.367 at 10 iterations, 0.564 at 11, and 0.395 at 12.
A small change in the iteration count would lose the adaptive result.

[^Lewiner2003]: Thomas Lewiner, Hélio Lopes, Antônio Wilson Vieira, and
    Geovan Tavares.  "Efficient implementation of Marching Cubes' cases with
    topological guarantees."  *Journal of Graphics Tools* 8(2) (2003) 1–15.
    <https://doi.org/10.1080/10867651.2003.10487582>

[^Lorensen1987]: William E. Lorensen and Harvey E. Cline.  "Marching cubes: A
    high resolution 3D surface construction algorithm."  *ACM SIGGRAPH
    Computer Graphics* 21(4) (1987) 163–169.
    <https://doi.org/10.1145/37402.37422>

[^Chernyaev1995]: Evgeni V. Chernyaev.  *Marching Cubes 33: Construction of
    topologically correct isosurfaces.*  Technical Report CN/95-17, CERN,
    1995.

[^Newman2006]: Timothy S. Newman and Hong Yi.  "A survey of the marching
    cubes algorithm."  *Computers & Graphics* 30(5) (2006) 854–879.
    <https://doi.org/10.1016/j.cag.2006.07.021>

[^Ericson2005]: Christer Ericson.  *Real-Time Collision Detection.*  Morgan
    Kaufmann, 2005.  Section 5.1.5, the closest point on a triangle to a
    point.
