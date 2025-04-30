# logs

**Goal:** A command line Rust application to produce ABAQUS and Exodus outputs.

*In order of most recent to least recent.*

## 2025-03-30

* Extraction
  * mdbook update?
* Concurrency
  * looked at rayon, https://docs.rs/rayon/latest/rayon/
  * current not needed
  * defeaturing is currently the slowest
  * surface triangularization
* isosurface
  * still working toward full-manifold
  * lurking challenge (?) multi-material with non-manifold
* JOSS submission
  * Defeaturing
    * canonical defeature A and B figure
    * mdbook in need of update
* Funding
  * tbd
* ONR annual review
  * slides
  * dates of attendance
* Matrix multiplication

## 2025-01-29

* Goals
  * "skin" of a segmentation -> quads -> (2 tris) -> stl (isosurface)
    * similar to "dual marching cubes"
    * smoothing of any generic edge connectivity (adjacency matrix in graph theory?)
    * tris tests (CBH) - test and documentation
    * stl -> tri (MRB) - implementation and architecture
  * octree
    * segementation - balanced quad tree origami "skin" surface -> quad surface -> tri surface
    * 6 templates conference paper, turn in to connected trianges, https://www.cs.umd.edu/users/hjs//pubs/reddyasae01.pdf, page 12
  * defeaturing - currently paused
  * dualization - paused

## 2025-01-22

### Toward isosurfaces

* **Option 1** (NW example)
  * origami quadtree -> trimesh (.stl) -> Taubin smoothing
* **Option 2** (NE example)
  * marching cubes from *dual* segmentation

### Questions

* volume preservation (avoid shrinking volumes)
* interface conformity (surface-to-surface matching)

### TODO

* MRB - refactor smoothing on generalized types, such as tris, quads, and hexes
* CBH - smoothing unit tests
  * The actual tessellation example from the Taubin paper
  * Single 3D cube into 12 triangles, then 36 quads, then continued refinement
* CBH - Sculpt to generate an isosurface without smoothing on a sphere or cube
* CBH - continue smoothed simulations, compare with conforming and voxel cases

## 2025-01-02

* schedule meeting
  * value proposition: open, auto, fast
* private client repo
* pause dualization, concerns about resulting mesh quality
* defeaturing
* surface reconstruction
  * `npy -> stl`
  * marching cubes
  * dual marching cubes
  * dual contouring (DC)
  * cubical marching squares (CMS)
  * https://www.reddit.com/r/VoxelGameDev/comments/qxi0u3/cubical_marching_squares_vs_dual_contouring_which/?rdt=44360
* mesh from isosurface
  * `stl -> exo`
* perfect tets to 4-hexs, what are the element qualities

## 2024-12-18

* .svg figures instead of .png
  * inherit colors from webpage
  * more lightweight
  * `custom.css` into mdbook, and point to it in the mdbook
  * CBH to create a cross-correlation .svg images
* order to get a dualized mesh - MRB to do next
  * dualization with one material block
  * then dualization with two material blocks, how to handle the interfaces?
  * strong balance, regular pairing
  * weak balance, regular pairing
  * generalized pairing
* IPREDICT input segmentations
  * show existing mesh workflow, conformity of .stl boundaries
  * how might we improve their workflow

## 2024-12-09

* Speed up 10x conspire - on benchmarks
* Tet meshing
  * Literature review https://github.com/autotwin/automesh/issues/217
* 2D weakly balanced templates - still work in progress
* Hook up recon3d to Python API
* rebase quadtree branch
* Budget
* Brainstorm - pivot from dev to analysis
  * Case study #1 - three material spheres
    * conforming versus voxelated versus voxelated smoothed
      * displacement field (maybe also strain and strain rate)?
      * existing CBH library for cross-correlation
    * https://github.com/autotwin/basis/blob/main/README_three_material_spheres.md
    * Write up as a chapter in mdbook as a case study?
  * Case study #2 - 100 brains
    * https://github.com/autotwin/basis
    * shape metric - MSJ, possibly skew, aspect ratio (-> "edge ratio" instead 2025-01-16)
  * Case study #3 - IPREDICT
    * https://mtec-sc.org/project-awards/18-04-i-predict/

## 2024-11-27

* Not all hanging nodes need to be connected; make (a complete set of) templates, and then match templates.
* Algorithmic connection of all hanging nodes causes unnecessarily large number of quad/hex element.
* Match small to large
* Match interior to exterior
* Path forward
  * Complete the quad templates (CBH) in Rust
  * Taubin iteration recast in code and docs (MRB)

## 2024-11-13

* stuck on local macOS
  * vtk, depends on xz, which depends on a homebrew version of liblzma
  * how to proceed, as it is holding up use in recon3d
* x,y,z versus z,y,x ordering, reference Polonsky
  * need to determine if the adapter goes into automesh or into client side (e.g., recon3d)
* Taubin smoothing semantic
  * 1 cycle = 2 iterations, cycle contains both smoothing and inflating iterations
* How to get a dualized sphere mesh
* Dualization papers
  * Marechal 2009
  * Livesu 2021
* #todo
  * MRB
    * Ask Marco if he will provide us dualized meshes of voxelized spheres
    * Investigate data structure for octree in Rust
  * CBH
    * Work with Andrew to determine where the xyz versus zyx adapter should live
    * Integrate automesh v0.2.4 into recon3d
    * Work with Anu for the IMECE slides

## 2024-11-07

* convert
  * spn -> npy
  * npy -> spn
  * inp -> exo | mesh | vtk
  * (todo) mesh -> exo | inp | vtk
* mesh [optional smoothing]
  * spn -> exo | inp | mesh | vtk
  * npy -> exo | inp | mesh | vtk
  * (beta development) tif -> exo | inp | vtk | mesh
* smooth
  * inp ->  exo | inp | mesh | vtk

* Answer questions from email 2024-11-05 14:28
* `.mesh`
  * CLI for `.mesh` output
  * Open brain model in hexalab
* `.exo` writer comparison with HDF5 https://www.hdfgroup.org/download-hdfview/
* binary version for exodus /netcdf/bin (ncdump)
* brain models
  * Emma's Google Drive for brain models
  * https://drive.google.com/drive/folders/158MXz03QCuockuRoSBpY-YuO4fni3RVD?usp=sharing
* smoothing
  * Hierarchical smoothing https://github.com/autotwin/automesh/issues/184
  * Define in words hierarchical control based on code implementation
  * this is an open research question bc too much smoothing destroys element quality
* Quality metrics
  * https://github.com/autotwin/automesh/issues/185
  * automesh quality -i foo.inp -o foo.csv
    * # element, msj, skew, aspect ratio
    * (element 1) 0.953, etc.
* Repo clean up, delete: __init__.py, deprecated_pyproject.toml, hello.py, test_hello.py

## 2024-11-05

* Look at https://github.com/beartype/beartype
* compare `.exo` and `.e` in HDFView 3.3.1
* hd5 expert Jay Lofstead

## 2024-10-30

* Portland
  * IMECE conference - November 17-21, 2024
  * What is the `automesh` milestone for the presentation?
  * smooth existing meshes
    * `.inp` reader
      * hierarchical smoothing, for now, won't work for non-perfect hex meshes
  * 1-2 weeks from now: finish Taubin smoothing, with hierarchichal option
  * remaining: CLI for inhomogenous hierarchical is paused
* Exodus
  * bug (#168) may actually not be a bug, it may just need documentation - done
  * status: to Chad as TODO - done
* Taubin smoothing
  * default values specification - done
    * kPB = 0.1
    * lambda = 0.6307
    * mdbooks: add bounds for all variables - done
  * command line interface (CLI) specification (except for any prescribed nodes)
  * compare spheres in #169 to result of Sculpt smoothing
  * Python implementation
    * visualization
    * gold standards
  * `.inp` and `.exo` as (non-smoothed) input files exist?
  * #TODO later: automatically calculate frequency content of the mesh surface
* Hierarchical smoothing
  * Homoegeneous and inhomogeneous both exist
  * Inhomomgeneous paused until Projection step
* ------ Future ------
* VTK file type - Hexalab.net
  * VTK crate exists
* Dualization
  * Ingredients - stepping stones
    * Quadtree
      * Chad to put links to existing dualization
    * Octree
* Projection
  * STL
  * applies to inhomogeneous hierarchical
* Untangling - internal energy - unsmoothing
  * has connections with smoothing
  * Mesh quality
  * Minimum scaled Jacobian (MSJ)

## 2024-10-23

* Review smoothing theory and nomenclature
* Smoothing roadmap
  * Unmerge the visuals in inhomogeneous bracket example
* Python - node neighbor algorithm
  * `src/fem/mod.rs`,
    * `/// Calculates and sets the node-to-node connectivity.`
    * `pub fn calculate_node_node_connectivity(&mut self) -> Result<(), &str> {`
  * `src/fem/test.rs`
    * `let node_node_connectivity_gold = vec![`
* Rust - smoothing algorithms architecture
  * Pause CLI support of prescribed inhomogeneous until have `.stl` support, do algorithmically not with sidesets
* vtk file type (for visualization in https://www.hexalab.net) (.mesh is not as interesting as .vtk)

## 2024-10-16

* benchmarks and bottlenecks
  * benchmarks only in rust nightly, may not be officially (?) supported
  * https://doc.rust-lang.org/cargo/commands/cargo-bench.html
  * prefer .npy to .spn bc .npy is more performant (or figure out how to better do .spn)
  * our writing of .npy doesn't compress as much as writing a .npy directly from Python (see Issue #140)
  * example problems where hex meshes could not previously be made
  * future steps with .stl isosurface and dualization
  * element quality
  * doc examples https://katex.org/docs/supported.html
* standards for smoothing documentation and test prior to implementation
  * stopped dependency on itertools, as unique is very slow; standard library has sufficient functionality, everything is sorted (a) we want things sorted and (b) when we do a manual unique on our own, we sort it anyway.
  * number of interations - try it and see what looks good, currently just one iteration
  * differentiator - node smoothing versus degree of freedom structure
  * illustration of 7-voxel sphere to a perfect sphere with voxel subdivision
  * possibly do single cube instead of double cube
* data structure of a neighborhoods (all nodes with all of the neighbors)
* node sets and node sets
* pause sidesets, not needed now, focus on nodesets for hierarchical smoothing
* possibility of Adam assisting for some short-term cycles
* svgbob ruined katex rendering, MRB to delete out the svgbob from the mdbook

## 2024-10-10

Touch base

* `letter_f_3d.npy` file updates (?)
  * Chad to take a lap and revisit
* `clap`
  * works well, but code is a bit W.E.T.
  * possible default to meshing
  * MRB prefers explicit, CBH also prefers explicit
  * still need to update the tests
* smoothing
  * implementation to come
* timing - take up tomorrow

## 2024-10-09

No interval meeting today due to travel.

## 2024-10-02

* CLI
  * feedback https://github.com/autotwin/automesh/issues/107
  * description https://github.com/autotwin/automesh/issues/106
* Smoothing
  * Building blocks
    * node-to-node connectivity
    * nodes classification into interior, interface, and surface - MRB
    * additional unit test example meshes
      * 2x2 rubix cube
      * 3x3 rubix cube - CBH
  * Laplace everywhere - MRB
  * Laplace on the interior - MRB
  * Node connectivity to output file? No.
  * Node classification for hierarchical smoothing (surface, interfaced, interior)
  * Algorithm
* High-res spheres https://autotwin.github.io/automesh/examples/spheres_cont/index.html
  * Compare to Sculpt with a single processor

## 2024-09-25

* Versioning
  * And maybe we should version the RTD page as well, instead of just showing the latest build from merging into main? And then there is versioning of the book...
* Zenodo
  * Citation example BibTeX and markdown
* Bug: translate with negative value
* Line length error, see `.pre-commit-config.yaml`


## 2024-09-18

CBH OOO.  No pair programming.

## 2024-09-11

* Reproduce *Voxelated* spheres (4 resolutions) with `automesh`
  * Reference https://github.com/autotwin/basis/blob/main/README_three_material_spheres.md#voxelated
  * Create `.py` file, to live currently in `automesh/sandbox/`
  * Command line example, input as `.npy` file, output as `.inp` file.
  * Anu: test the `.inp` files.

## 2024-09-04

* CBH
  * done - Review new documentation, moved figures, and possible deprecation of `doc` folder
    * pending - make figure colors consistent
  * pending - Migrate `.md` files from `doc` folder to `book`
    * remaining: `dev_workflow.md`
  * done - How to build MDBook without a full pull request into `main`?
  * done - Python implementation of `figures.py` (formerly `voxels.py`) to produce element connectivity without gaps, match MRB results
  * done - Standardize on `.png`, and recast the `.jpg` as `.png`
  * MDBook
    * How to get MDBook to run and test the Python API
    * Python documentation in MDBook (so ReadTheDocs is an option, and not Sphinx)
    * Small rust examples can "play" in the mdbook
* MRB
  * pending - `Sparse` example.
    * bug - swap first in last out for block 1 to be set 1, block 2 to be set 2 in .inp
  * done - `Command Line Interface` update from static to dynamic reflection of the CLI.
  * pending - Review https://github.com/autotwin/mesh/blob/main/doc/npy_to_mesh.md
    * plug out Sculpt
    * plug in automesh
  * pending - Currently the figures are static build, but we want to eventually migrate to building the images on checkin, done with `.yml` CI/CD, makes sure the source script still works.  Preprocessor command that runs prior to mdbook.

```bash
cargo install mdbook mdbook-katex
mdbook build
mdbook serve
```

## 2024-08-28

* Merge the `inp` into `main` and the `docs` into `main`
  * CBH to finish out the tests `.rs` and `.py`
  * MRB merge `inp` into `main`
* MRB
  * Random/crazy test, connectivity gaps, voids (called `Sparse`)
  * Review https://github.com/autotwin/mesh/blob/main/doc/npy_to_mesh.md
    * plug out Sculpt
    * plug in automesh
  * mdBook documentation - seed this as part of the build process on GitHub
* CBH
  * How to capture the `.png`
  * expand tests to include randomized OFF/ON of various blocks.
  * update the 3D letter F
  * update documentation Python side to include the Python-generated figures
  * mdBook documentation
  * Review funding doc
  * Move mtg on 04 Sept to later time - no longer relevant/needed
  * OOO 18 Sept

## 2024-08-21

MRB OOO.  No pair programming.

## 2024-08-14

* PR from issue
* https://github.com/hovey/mwe/tree/main/python/cicd_release
* review CBH code
  * refactor for *n* test cases
  * isolate the IO testing in `spn.rs`
* ask about test input file
* formatting of input file (tabs, spaces, alignment)
* refactor automesh connectivity and change test
  * MRB - refactor code
  * CBH - refactor documentation, and the Python scripts to generate `f.spn` and `f.npy`, with `.py` into `tests/input`
    * `(x, y)` plane for pixels, `z` axis for stacked pixles into voxels, stack of two slices (`NELZ = 2`), `NELX = 3`, `NELY = 5`.
  * CBH to dry out code `spn.rs` and `spn_single.rs` (CH made the code WET with the duplicated tests)
  * CBH refactor `f_fiducial.inp`
* Change from `spn` type to `voxel` type.


```bash

    ^ y  (z out of page, right hand rule)
    |
    |
    @@@
    @
    @@
    @
    @ ---------->  x
```

### Docs

Notes on creating documentation:

* `cargo doc --verbose`
* [The rustdoc book](https://doc.rust-lang.org/rustdoc/what-is-rustdoc.html)


## 2024-08-07

* refactor some existing code from C-style to idiomatic Rust style

## 2024-07-31

### For next week

* CBH
  * yaml input/ouput
  * tilde bug
* MRB
  * clap
  * exodus with node numbering (and numbering gaps)
  * exodus connectivity

### This week

- [ ] CFC would like a Work Planning Agreement (WPA)
- [x] Tutorial: How to update the exo branch, which is currently 12 commits behind and 10 commits ahead of the main branch.
- [ ] Module load mechanism on HPC, via SMT > Utility > PythonModule > Deployer
- [x] Tutorial: Outline of a complete [development workflow](dev_workflow.md)
  * Configuration - especially a Python virtual environment
  * Q: Is there a virtual environment equivalent for Rust?  A: Nope, not necessary.
  * Check in and review
- [x] Code Review: Minimum working example: https://github.com/hovey/rustschool/tree/main/yml_io
  * can we have `main.rs` and `lib.rs` (???), so how to architect if we want both a library and a command line tool?
  * yamlio or ymlio be in `lib.rs` equivalent
  * `eprintln!`
  * tilde bug
  * serde (serialize-deserialize) crate dependency
  * serde_yaml
    * downloads 70,632,177
    * Rust library for using the Serde serialization framework with data in YAML file format. (This project is no longer maintained.)
    * https://github.com/dtolnay/serde-yaml
       * This repository has been archived by the owner on Mar 24, 2024. It is now read-only.
  * serde_yml (a fork of serde_yaml)
    * downloads 39,956
  * alternatives on crates.io
    * yaml-rust = "0.4"
      * downloads 61,005,944
      * http://chyh1990.github.io/yaml-rust/
    * yaml-merge-keys = "0.4"
      * downloads 3,062,559
      * KitWare: https://gitlab.kitware.com/utils/rust-yaml-merge-keys
      * uses serde_yaml and yaml_rust
    * yaml = "0.1"
      * downloads 24,016
- [x] clap: https://github.com/clap-rs/clap
  * `cargo run - --help`, `cargo run recipe.yml`
- [x] * clap alternatives: quicli, structopt
- [ ] Code Review: continuation from last week, especially node numbering with gaps
- [x] Questions for MRB
  * in `/tests/` folder, the `test_utility.py` has the `test_` prefix so that it is picked up by the `pytest` module.  In that same folder, `npy.py` and `spn.py` have tests, and therein has function definitions with the leading `test_foo` format, but the filenames themselves do not have the `test_` prefix.
- [x] Deployment
  * [crates.io](https://crates.io)
    * [rust binary](https://crates.io/crates/automesh)
    * rust library
  * [PyPI](https://pypi.org)
    * [Python wheel](https://pypi.org/project/automesh/)
- [x] Decisions
  * not require `test_` prefix (as done with Python) for names of Rust test files
  * tell python testing to look at `.py` files in the `tests/` folders, as shown below from the [`pyproject.toml`](../pyproject.toml)
  * stop using `pip install -e .` and instead, use Maturin, which will build the Python wheel (`maturin develop --release --features python`) and then use code from the wheel

```bash
[tool.pytest.ini_options]
python_files = [
  '*.py'
]
testpaths = [
  'tests/'
]
```

```bash
(.venv)  (housekeeping) chovey@s1088757/Users/chovey/autotwin/automesh> pip install -e .
Obtaining file:///Users/chovey/autotwin/automesh
  Installing build dependencies ... done
  Checking if build backend supports build_editable ... done
  Getting requirements to build editable ... done
  Preparing editable metadata (pyproject.toml) ... done
Requirement already satisfied: cffi in ./.venv/lib/python3.11/site-packages (from automesh==0.1.3) (1.16.0)
Requirement already satisfied: numpy in ./.venv/lib/python3.11/site-packages (from automesh==0.1.3) (2.0.0)
Requirement already satisfied: pyyaml in ./.venv/lib/python3.11/site-packages (from automesh==0.1.3) (6.0.1)
Requirement already satisfied: pycparser in ./.venv/lib/python3.11/site-packages (from cffi->automesh==0.1.3) (2.22)
Building wheels for collected packages: automesh
  Building editable for automesh (pyproject.toml) ... error
  error: subprocess-exited-with-error

  × Building editable for automesh (pyproject.toml) did not run successfully.
  │ exit code: 1
  ╰─> [23 lines of output]
      Running `maturin pep517 build-wheel -i /Users/chovey/autotwin/automesh/.venv/bin/python3.11 --compatibility off --editable`
      📦 Including license file "/Users/chovey/autotwin/automesh/LICENSE"
      🍹 Building a mixed python/rust project
      🔗 Found cffi bindings
      🐍 Using CPython 3.11 at /Users/chovey/autotwin/automesh/.venv/bin/python3.11 to generate the cffi bindings
         Compiling automesh v0.1.3 (/Users/chovey/autotwin/automesh)
          Finished `release` profile [optimized] target(s) in 0.22s

      ===================================================================
      maturin has panicked. This is a bug in maturin. Please report this
      at https://github.com/PyO3/maturin/issues/new/choose.
      If you can reliably reproduce this panic, include the
      reproduction steps and re-run with the RUST_BACKTRACE=1 environment
      variable set and include the backtrace in your report.

      Platform: macos aarch64
      Version: 1.7.0
      Args: maturin pep517 build-wheel -i /Users/chovey/autotwin/automesh/.venv/bin/python3.11 --compatibility off --editable

      thread 'main' panicked at /Users/runner/.cargo/registry/src/index.crates.io-6f17d22bba15001f/cbindgen-0.26.0/src/bindgen/mangle.rs:132:17:
      not implemented: Unable to mangle generic parameter Array(Primitive(Integer { zeroable: true, signed: false, kind: Size }), Value("8")) for 'Vec'
      note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
      Error: command ['maturin', 'pep517', 'build-wheel', '-i', '/Users/chovey/autotwin/automesh/.venv/bin/python3.11', '--compatibility', 'off', '--editable'] returned non-zero exit status 101
      [end of output]

  note: This error originates from a subprocess, and is likely not a problem with pip.
  ERROR: Failed building editable for automesh
Failed to build automesh
ERROR: ERROR: Failed to build installable wheels for some pyproject.toml based projects (automesh)
```

### MRB accomplishments

* I added clap as a dependency and set up a main.rs to run automesh as a binary, i.e.,
`cargo run --release -- -i tests/input/f.npy -o foo.exo`
it automatically changes methods based on IO file extensions, and it fails for now (we don’t have anything that writes exodus files yet)
* I moved the functionality of the NPY type into the SPN type after realizing that the internal data was essentially the same. Meaning the methods in NPY that read `.npy` files just converted it into SPN equivalent data anyway. So now SPN types can be created from `.spn` or `.npy` files.
* I added the functionality that renumbers the nodes to avoid gaps, I’m pretty sure it’s working. Itertools is now a dependency so I could use `.unique()`
* Still no nodal coordinates yet, will work on next.

### Where do the different editable installs originate?

```bash
cd ~/autotwin/mesh
source .venv/bin/activate.fish

pip list
Package         Version     Editable project location
--------------- ----------- ---------------------------
atmesh          0.0.7       /Users/chovey/autotwin/mesh
...
numpy           1.26.4

python

import atmesh
print(atmesh)
<module 'atmesh' from '/Users/chovey/autotwin/mesh/src/atmesh/__init__.py'>

import numpy
print(numpy)
<module 'numpy' from '/Users/chovey/autotwin/mesh/.venv/lib/python3.11/site-packages/numpy/__init__.py'>

quit()

deactivate

cd ~/autotwin/automesh
source .venv/bin/activate.fish

pip list
Package      Version Editable project location
------------ ------- -------------------------------
automesh     0.1.3   /Users/chovey/autotwin/automesh
...
numpy        2.0.0

python

import automesh
print(automesh)
<module 'automesh' from '/Users/chovey/autotwin/automesh/.venv/lib/python3.11/site-packages/automesh/__init__.py'>

print(numpy)
module 'numpy' from '/Users/chovey/autotwin/automesh/.venv/lib/python3.11/site-packages/numpy/__init__.py'>

quit()
```

### CBH accomplishments

* documentation
* error handling

### Continued discussion (2024-08-02)

* Decisions:
  * command line arguments (via clap) is prioritized; `.yml` file recipe is paused in favor of CLI interacion
  * command line runs produce a `.log` file by default (optional is to have no logging, command line flag for no logging is to be determined), the `.log` file will
    * echo the command issued to the CLI (support reproducibility)
    * have a name `<output_file>.log` that matches the file name of the output Exodus file `<output_file>.exo`
    * the contents of the `.log` file will also be echoed to the screen (stdout) during the run and capture any errors (stderr) if they are encountered
  * command line interface to be expanded to specify `nelx`, `nely`, and `nelz` (and have error checking to assure that `nelx x nely x nelz = n_voxels`)
  * CLI to be expanded to consume two types of input files: `.npy` and `.spn`.

## 2024-07-24

* No pair programming today, MB at [WCCM](https://www.wccm2024.org).

## 2024-07-17

* ONR annual review completed
  * Void to be optionally includeded or excluded
  * Generalization: included/exclude any material number that is in the segmentation
* IMECE conference podium abstract submitted
* Python dev working in autotwin repo instead of mwe repo
  * Review unit test documentation for 2D and 3D, how to implement and test w Rust
* Element connectivity - filters out void, but the current node numbering has gaps
  * ndarray, ndarray_npy crate for file io of npy files
  * npy type is uint8
* new Rust feature: Specialization

## 2024-07-05

* [Maturin demo](https://github.com/hovey/mwe/tree/main/maturin)

## 2024-07-03

* [Exodus II file format](exodus.md)
* weekly interval pair programming Wed 1100-1300 EST (0900-1100 MST)
* repo updates
* iterators are great, https://doc.rust-lang.org/std/iter/trait.Iterator.html
* pre-commit, prevent a local from commiting prior to push
* [PyO3](https://pyo3.rs) is the Rust package for Python binding in Rust
* [muturin](https://www.maturin.rs) is the packager
* [pre-commit](https://pre-commit.com) A Python package for multi-language pre-commit hooks
  * See the [.pre-commit-config.yml](../.pre-commit-config.yaml)
  * Clippy is a pre-commit Rust hook, see https://github.com/backplane/pre-commit-rust-hooks
  * See also [Rust CI Tooling: Clippy, commitlint, pre‑commit and More](https://rodneylab.com/rust-ci-tooling/)

```bash
python -m pip install --upgrade pip
pip install maturin
maturin develop --release --extras dev
# pip install pre-commit # already installed with maturin
pre-commit install
pre-commit run --all-files
```

**Decision:** Pause use of PyO3 to wrap Rust and expose as a Python function.  Develop a pure Rust command line program, and use as a `subprocess`, e.g.,

```bash
# example
import subprocess

result = subprocess.run([MD5_BINARY, fin], check=False, stdout=subprocess.PIPE)
        output = result.stdout.decode("utf-8")
```
