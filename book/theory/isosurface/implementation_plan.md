# Implementation Plan

> **DRAFT.** Scopes work on the `surfacing` branch, to implement Marching
> Cubes and Dual Contouring alongside the existing cuberille isosurfacing in
> [`mesh tri`](../../cli/mesh.md). Not yet reviewed.

## MeshLab as a reference — verified

[MeshLab](https://www.meshlab.net) and its underlying library,
[vcglib](https://github.com/cnr-isti-vclab/vcglib), were checked directly
rather than assumed:

* **Marching Cubes: confirmed.** vcglib implements it at
  `vcg/complex/algorithms/create/marching_cubes.h`, with a lookup table
  (`mc_lookup_table.h`) and a walker (`mc_trivial_walker.h`). MeshLab
  exposes it to users as the **Marching Cubes (APSS)** and **Marching
  Cubes (RIMLS)** filters — MLS-surface variants of the same core
  algorithm.
* **Dual Contouring: not present.** No file, filter, or entry in
  [PyMeshLab's filter list](https://pymeshlab.readthedocs.io/en/latest/filter_list.html)
  implements it. MeshLab is a useful reference for Marching Cubes only.

**License:** vcglib and MeshLab are GPL-3.0, matching both `conspire` and
`automesh`. There is no conflict in consulting vcglib's design — its
lookup-table approach and walker pattern for streaming large volumes are
worth studying. The recommendation is still to implement from the primary
papers ([Marching Cubes](marching_cubes.md),
[Dual Contouring](dual_contouring.md)) rather than port C++, for idiomatic
Rust and because vcglib's walker carries decades of accreted
special-casing that need not be inherited.

## `autotwin/pixel` as a reference — verified

[`autotwin/pixel`](https://github.com/autotwin/pixel) — co-owned by this
repo's author — builds STL isosurfaces from MRI volumes, in
`src/atpixel/MRI_to_stl.py`. Checked directly:

* **Marching Cubes, via `skimage.measure.marching_cubes`.** Confirmed
  which variant: scikit-image's default is **Lewiner's algorithm**[^Lewiner_2003]
  — not classic Lorensen & Cline — which resolves the ambiguous-face
  cases and guarantees a topologically correct, watertight, manifold
  result matching the trilinear interpolant of the scalar field. `pixel`
  uses this default as-is (`gradient_direction="ascent"`, plus a
  `step_size` for coarser/faster extraction).
* **No Dual Contouring.** Same finding as MeshLab above — nothing in
  `pixel` implements it either.
* **Padding before extraction.** `pixel` zero-pads its mask on all six
  faces (`pad_array`) before running Marching Cubes, so anatomy touching
  the array boundary still produces a closed surface. `automesh`'s
  cuberille already gets this for free, more cheaply: its `label()`
  closure in `tessellation/from/grid/mod.rs` returns `void` for any
  out-of-bounds index rather than indexing real data, instead of
  materializing a padded copy. Worth preserving that lazy-boundary
  approach for MC/DC rather than adopting `pixel`'s explicit padding.
* **License:** MIT (`pixel`) and BSD-3-Clause (`scikit-image`) — no
  conflict with `conspire`/`automesh`'s GPL-3.0, same conclusion as the
  MeshLab check above.

`pixel` solves a problem `automesh` doesn't have — building a binary mask
from continuous MRI intensities (Otsu thresholding, alpha shapes,
morphological cleanup) — none of which is relevant here, since `mesh tri`
already receives a discrete segmentation. The only overlapping step is
the last one, mask to triangulated surface, and specifically, which MC
variant to target — see [Phase 2](#phase-2--marching-cubes).

## Where `conspire` stops and `automesh` begins

The rule falls out of tracing `mesh tri`, `mesh hex`, `smooth`, and
`remesh` end to end: **if it changes geometry, it's `conspire`'s job. If
it's how a human invokes that and what they see, it's `automesh`'s job.**

`conspire` owns every data type (`Voxels`, `Segmentation`, `Mesh<D>`,
`Tessellation`, `Connectivity`) and every algorithm that transforms one
into another: octree construction/equilibration/dualization, smoothing,
remeshing, quality metrics, buffer fitting, and today's isosurfacing
(cuberille, via `Tessellation::from<Voxels>`). It also owns the
mesh/tessellation file I/O (exodus, abaqus, medit, vtk, stl read/write) —
anything that's "given this data, produce that data." It's a
general-purpose library on crates.io with its own version, its own test
suite, and consumers beyond `automesh` could exist — so its API should be
shaped for that, not molded to `automesh`'s CLI.

`automesh` is a thin `clap` binary. Every `match` arm in
`src/mesh/mod.rs::mesh()` is one line calling straight into `conspire` —
`Tessellation::from(voxels)`, `tessellation.dualize(...)`,
`tessellation.cut(...)`, `Mesh::from_segmentation(...)`. Its own code is
argument parsing, orchestration order (read → defeature → mesh → smooth →
remesh → metrics → write), user-facing error text, and progress
narration. It has no geometry algorithms of its own.

The one visible exception proves the rule: the smoothing guard on
`doc-interval` (`e4df400` in `src/smooth/mod.rs`) exists only because
`conspire` 0.7.3 panicked (`todo!()`) on polytopal connectivity instead of
returning an error. `automesh` papered over an upstream gap. Now that
`conspire` 0.7.4 actually implements polytopal smoothing, that patch is
dead weight to remove, not something to build on. `automesh` reaching
into algorithm territory is always a temporary workaround for an upstream
gap, never a destination.

For MC/DC specifically: the lookup tables, QEF minimization,
distance-transform field, connectivity assembly, and pinch repair are
100% `conspire`'s, in `tessellation/from/grid/{marching_cubes,dual_contouring}/`.
`automesh`'s entire footprint is a `--method` flag on `mesh tri`, one line
plumbing it into `conspire`, and these book pages. Practically, since you
hold write access to both, this becomes two sequenced pull requests per
feature: land it in `conspire`, get it tested and merged, bump and
publish the crate, then move `automesh`'s `Cargo.toml` pin and land the
CLI flag — not one PR touching both repos at once, since `automesh`'s pin
is intentionally exact (`=0.7.x`) and dependabot/CI treat a `conspire`
bump as its own reviewable change.

## Phase 0 — Scope the API shape

Before writing `conspire`-side code, settle the shape of the new entry
point so existing callers don't break:

* An `Isosurfacing` enum (`Cuberille | MarchingCubes | DualContouring`)
  passed into a generalized constructor, e.g.
  `Tessellation::from_voxels(voxels, method)`, with today's
  `From<Voxels<T>>` becoming the `Cuberille` arm.
* Module layout following `conspire`'s existing convention — one
  submodule per variant, each with its own `test.rs`:
  ```
  tessellation/from/grid/
    cuberille/mod.rs      (today's grid/mod.rs, renamed)
    marching_cubes/mod.rs
    dual_contouring/mod.rs
  ```
* Naming caution: `conspire` already uses "dual" heavily, for the
  octree-dualization hex pipeline (`ntree::dual`, `mesh hex`'s dual mesh).
  The new module is named `dual_contouring` in full, never bare `dual`,
  to avoid collision in code and in these docs.

Since both repos are under the same hand for this work, this phase is a
short design pass, not a cross-team negotiation — but `conspire` is a
shared, published crate with its own history and conventions, so the
shape is worth fixing deliberately before code accumulates around it. See
[above](#where-conspire-stops-and-automesh-begins) for how that work then
sequences across the two repos.

## Phase 1 — Shared groundwork: a scalar field from a segmentation

Both algorithms want more than raw voxel labels to do good work. MC needs
something continuous to interpolate along an edge. DC needs a position
*and* a normal at each sign-changing edge — Hermite data. A segmentation
alone has neither.

The recommended approach: build a distance transform over the voxel grid
once — Euclidean or chamfer distance to the nearest label boundary — and
derive both algorithms' inputs from it.

* **MC** interpolates the zero-crossing along this field instead of
  always landing at the edge midpoint. This is what actually produces
  the smooth-surface property [Marching Cubes](marching_cubes.md)
  describes. MC run directly on raw label data degenerates to edge
  midpoints everywhere — visually close to cuberille.
* **DC** gets its normal at each edge from the local gradient of the same
  field, and a QEF (quadratic error function) minimization — per Ju
  *et al.* — picks the interior vertex position per cell.
* **Multi-label caution:** a single globally inside/outside-signed field
  only makes sense for two labels. As [Introduction](introduction.md)
  notes, `automesh` segmentations are rarely binary, so the field needs
  to carry, per grid edge, *which two labels* are separated there —
  effectively a distance-to-nearest-boundary rather than one global
  signed distance. Design this in from the start rather than bolting
  multi-label support on afterward.

This is the highest-risk, highest-value piece of the plan. Get it wrong
and both algorithms just reproduce cuberille with extra steps. It is
worth a short design note and a couple of throwaway prototypes before
committing to the module layout above.

## Phase 2 — Marching Cubes

Implement the topologically-guaranteed variant directly — not classic
Lorensen & Cline, and not a naive 256-entry lookup table. Both are
superseded: the classic table is known to produce cracks and
inconsistent, non-manifold, non-watertight topology on ambiguous faces,
which is exactly the weakness [Marching Cubes](marching_cubes.md)
documents. There is no reason to build the broken version first and fix
it later.

* Implement Lewiner *et al.*'s algorithm[^Lewiner_2003], the standard fix
  for MC's ambiguous cases: it completes Chernyaev's Marching Cubes 33
  case analysis and guarantees a manifold, crack-free result matching the
  topology of the trilinear interpolant over each cube. This is also what
  `scikit-image` implements by default, confirmed via
  [`autotwin/pixel`](#autotwinpixel-as-a-reference--verified) above — a
  tested, working reference for the exact algorithm to target, not just
  the paper.
* Case table: the correct implementation needs the full 33-subcase
  disambiguation, not the naive 256-entry table — vcglib's
  `mc_lookup_table.h` and `scikit-image`'s Cython implementation
  (`_marching_cubes_lewiner_cy.pyx`) are both usable as structural
  references for the table layout, alongside the paper itself.
* **Multi-material requirement.** Single-isovalue MC only finds one
  boundary — material versus void. Run naively per material, adjacent
  regions get independently-extracted, non-conforming surfaces at their
  shared interface: gaps, overlaps, or duplicate geometry, exactly what
  [Introduction](introduction.md) says cuberille already avoids today.
  This is not a hypothetical failure mode; it is documented and solved —
  Wu & Sullivan's multi-material Marching Cubes[^Wu_Sullivan_2003]
  classifies each cube by the *set* of labels touching it rather than a
  single in/out bit, so adjacent materials share a conforming boundary
  by construction. Target this from the start, not single-isovalue MC
  with multi-material bolted on later.
* Output: `Connectivity::Triangular`, identical to cuberille today, so it
  drops directly into everything downstream — `mesh tri smooth`,
  `mesh tri smooth remesh`, `--metrics`.

## Phase 3 — Dual Contouring

* One vertex per grid cell the surface passes through, positioned by QEF
  minimization over that cell's Hermite data.
* Non-manifold repair — [Dual Contouring](dual_contouring.md) documents
  this as DC's known weakness. Reuse the pinch-vertex resolution already
  written for cuberille (`resolve_pinches`, in today's
  `tessellation/from/grid/mod.rs`) as a post-pass, rather than
  reimplementing it.
* Feature preservation, DC's main advantage over MC, is a stretch goal —
  ship a working, manifold-repaired DC first, tune sharp-feature handling
  after.
* **Multi-material requirement**, same as [Phase 2](#phase-2--marching-cubes):
  a single vertex per cell, positioned against one implicit surface, is a
  two-label formulation. Frisken's multi-label Surface Nets[^Frisken_2022]
  — the DC family's answer to Wu & Sullivan above — extends exactly this
  QEF-per-cell approach to place one vertex per cell against *all* of the
  labels touching it, preserving sharp boundaries between materials as
  well as at the outer surface. Design the QEF and cell classification
  for the multi-label case directly, rather than a two-label DC with
  multi-material bolted on later.

## Choosing a method

`automesh` segmentations come from more than one kind of source, and that
should drive which method a user reaches for — there is no single best
default beyond cuberille itself.

* **Medical (CT/MRI-derived segmentation).** Noisy boundaries, organic
  curved anatomy, no sharp edges to preserve. Marching Cubes is the
  better fit once it lands: designed for smooth, closed organic surfaces,
  and DC's sharp-feature machinery buys nothing here — complexity spent
  on a problem the input doesn't have.
* **CAD-derived or synthetic-with-sharp-features segmentation.** The
  *original* geometry had flat faces and precise edges before
  voxelization destroyed them. This is exactly the case Dual Contouring
  targets: it uses gradient information at edge crossings specifically
  to recover sharp features that Marching Cubes rounds off.
* **Cuberille remains the safe default regardless.** It makes no claim
  about the input's character — it reproduces the voxel topology
  exactly, always watertight from the segmentation's own connectivity.
  Adding MC and DC gives users a choice; it does not obsolete cuberille
  as `mesh tri`'s default.

`automesh` doesn't yet have a medical-segmentation example in this book —
[Analysis](../../analysis.md) and [Examples](../../examples.md) are
synthetic and CAD-like geometry (concentric spheres, a torus, blobs) —
but medical segmentation is squarely in scope for the `autotwin` org:
[`autotwin/pixel`](https://github.com/autotwin/pixel) exists specifically
to build STL surfaces from MRI volumes, as documented above. This is
exactly why [Phase 4](#phase-4--automesh-cli-wiring) makes the
isosurfacing method a per-invocation `--method` flag rather than a single
global default — `automesh` needs to serve both kinds of input well, not
pick one.

## Phase 4 — `automesh` CLI wiring

Once `conspire` exposes the method selector, add to `MeshArgs` (mirrors
`smooth`'s existing `--method Laplace|Taubin` pattern):

```rust
/// Isosurfacing method for a segmentation (npy | spn) input [default: Cuberille]
#[arg(long, value_name = "NAME")]
pub method: Option<String>,
```

Applies only when meshing triangles (`mesh tri`) from a segmentation;
rejected — with a clean CLI error, matching the existing
`Invalid smoothing method` pattern — for `hex`/`hexdom`/`poly`. Default
stays `Cuberille`, so existing usage is unaffected.

## Phase 5 — Validation

* Unit tests in `conspire`, mirroring its existing convention: known
  small voxel fields — a single labeled cube, a sphere — with
  hand-verified triangle counts and positions.
* Cross-validation against MeshLab for MC specifically, since it is the
  one place a trusted second implementation exists: run the same
  segmentation through both, compare element counts and `--metrics`
  output (minimum scaled Jacobian, maximum skew). No equivalent
  reference exists for DC — its validation leans on manifoldness checks
  and visual inspection instead.
* Extend the quality-metrics comparison style already used for
  [Hexahedral Meshing from a Surface](../hex_from_surface.md#choosing-a-scale)
  to a per-method table for a couple of the existing example shapes
  (sphere, torus).

## Phase 6 — Documentation

[Introduction](introduction.md), [Marching Cubes](marching_cubes.md), and
[Dual Contouring](dual_contouring.md) already describe all three
algorithms conceptually. Once implemented, the latter two each gain an
"as implemented in `automesh`" section — CLI example, a figure, a quality
comparison against cuberille — and the DRAFT banners on this page and on
[Introduction](introduction.md) come off.

## Sizing and sequencing

| Phase | Repo | Rough size | Blocking dependency |
| --- | --- | --- | --- |
| 0. Scope the API shape | conspire | 1 design pass | none |
| 1. Distance-field groundwork | conspire | largest single chunk | Phase 0 |
| 2. Marching Cubes | conspire | medium | Phase 1 |
| 3. Dual Contouring | conspire | largest overall (QEF + repair) | Phase 1; benefits from Phase 2's lookup-table experience |
| 4. CLI wiring | automesh | small | Phase 2 or 3 individually — MC can wire in before DC is done |
| 5. Validation | both | medium, ongoing | each phase as it lands |
| 6. Documentation | automesh (this book) | small | each phase as it lands |

Marching Cubes can ship end to end (Phases 1 → 2 → 4 → 5 → 6) as a
self-contained first milestone without waiting on Dual Contouring — the
distance-field groundwork built for it is reused, not redone, when Dual
Contouring starts. That is the natural place to cut a first release.

## Additional Items

Verified and worth keeping, but deliberately not woven into the phases
above — parked here for later, not forgotten.

**Related methods, named but not currently planned:**

* **Dual Marching Cubes** — an octree/dual-grid hybrid combining
  DC-style adaptive resolution with MC-style manifold
  guarantees.[^Schaefer_Warren_2005]
* **Surface Nets** — the simpler, QEF-free ancestor of Dual Contouring:
  one vertex per active cell, placed at the centroid of edge crossings
  rather than solved for. A candidate checkpoint partway through
  [Phase 3](#phase-3--dual-contouring), not currently written into it.
  Proposed originally for exactly `automesh`'s kind of
  input.[^Gibson_1998] (Its multi-label extension, Frisken 2022, is
  already cited in Phase 3.)
* **Flying Edges** — a much faster, parallelizable reformulation of
  Marching Cubes; VTK's current default. A performance optimization to
  weigh after Lewiner's correctness-focused MC ships, not a first-version
  concern.[^Schroeder_2015]
* **Manifold Dual Contouring** — a more canonical, earlier citation for
  DC's manifold-repair fix than Rashid et al. 2016, currently cited on
  [Dual Contouring](dual_contouring.md). Worth adding there when that
  page is next revised.[^Schaefer_Ju_Warren_2007]

**Rust prior art**, same-language reference implementations, more
directly comparable to what `conspire` will build than vcglib (C++) or
`scikit-image` (Python/Cython):

* [`isosurface`](https://github.com/swiftcoder/isosurface) (Apache-2.0)
  — implements both Marching Cubes and Dual Contouring, plus
  adaptive/octree variants (`extended_marching_cubes.rs`,
  `linear_hashed_marching_cubes.rs`), and a standalone QEF solver
  (`feature/qef.rs`) directly reusable as a design reference for
  [Phase 3](#phase-3--dual-contouring).
* [`fast-surface-nets`](https://github.com/bonsairobo/fast-surface-nets-rs)
  (MIT/Apache-2.0) — a maintained, dedicated Naive Surface Nets
  implementation.

## References

[^Lewiner_2003]: Lewiner T, Lopes H, Vieira AW, Tavares G. Efficient implementation of Marching Cubes' cases with topological guarantees. Journal of Graphics Tools. 2003;8(2):1-15. [link](http://thomas.lewiner.org/pdfs/marching_cubes_jgt.pdf)

[^Wu_Sullivan_2003]: Wu Z, Sullivan JM Jr. Multiple material marching cubes algorithm. International Journal for Numerical Methods in Engineering. 2003;58(2):189-207. [link](https://doi.org/10.1002/nme.775)

[^Frisken_2022]: Frisken SF. SurfaceNets for multi-label segmentations with preservation of sharp boundaries. Journal of Computer Graphics Techniques. 2022;11(1):34-54. [link](https://jcgt.org/published/0011/01/03/paper.pdf)

[^Schaefer_Warren_2005]: Schaefer S, Warren J. Dual marching cubes: primal contouring of dual grids. Computer Graphics Forum. 2005;24(2):195-201. [link](https://www.cs.rice.edu/~jwarren/papers/dmc.pdf)

[^Gibson_1998]: Gibson SFF. Constrained elastic surface nets: generating smooth surfaces from binary segmented data. In: Medical Image Computing and Computer-Assisted Intervention (MICCAI). Lecture Notes in Computer Science, vol 1496. 1998:888-898. [link](https://doi.org/10.1007/BFb0056277)

[^Schroeder_2015]: Schroeder W, Maynard R, Geveci B. Flying edges: a high-performance scalable isocontouring algorithm. In: IEEE 5th Symposium on Large Data Analysis and Visualization (LDAV). 2015:33-40. [link](https://doi.org/10.1109/LDAV.2015.7348069)

[^Schaefer_Ju_Warren_2007]: Schaefer S, Ju T, Warren J. Manifold dual contouring. IEEE Transactions on Visualization and Computer Graphics. 2007;13(3):610-619. [link](https://doi.org/10.1109/TVCG.2007.1012)
