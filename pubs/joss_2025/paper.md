---
title: 'automesh: Automatic mesh generation in Rust'
tags:
  - Rust
  - segmentation
  - computational geometry
  - finite element mesh
authors:
  - name: Chad B. Hovey
    orcid: 0000-0003-0915-5765
    equal-contrib: true
    affiliation: 1
  - name: Michael R. Buche
    orcid: 0000-0003-1892-0502
    equal-contrib: true
    affiliation: 1
affiliations:
 - name: Sandia National Laboratories, United States
   index: 1
date: 30 April 2025
bibliography: paper.bib
---

# Summary

`automesh` is an open-source Rust software program that uses a **segmentation**,
typically generated from a 3D image stack,
to create a finite element **mesh**,
composed either of hexahedral (volumetric)
or triangular (isosurface) elements.
`automesh` **converts** between
segmentation formats (`.npy`, `.spn`)
and
mesh formats (`.exo`, `.inp`, `.mesh`, `.stl`, `.vtk`).
`automesh` can **defeature** voxel domains,
apply Laplacian and Taubin **smoothing**,
and output mesh quality **metrics**.
`automesh` uses an internal octree for fast performance.

# Introduction

Computed Tomography (CT), Magnetic Resonance Imaging (MRI), and other similar
imaging modalities that stack a sequence of 2D digital images (composed of
pixels) to create a 3D voxel (volumetric) representation of a subject have
revolutionized the fields of engineering and medicine by providing
non-invasive and non-destructive insights into the internal structures of
complex systems.

* In the medical field, 3D voxel data sets are instrumental for visualizing internal organs and soft tissues, facilitating accurate diagnoses and enhancing surgical planning.
* In dentistry, they play a vital role in jaw surgery and the placement of dental implants.
* Biomedical engineering benefits include fit and integrity of surgical implants and prosthetic devices.
* In biomechanical engineering, the human digital twin can be used in injury risk prediction in environments not suited for human surrogate models; for example, automotive barrier crash testing and military environments that have blast and ballistic exposure.
* In materials engineering, 3D imaging is employed to analyze the internal structures of materials, enabling the detection of voids, inclusions, and defects.
* Aerospace engineering utilizes these techniques to inspect critical components, such as turbine blades, ensuring their structural integrity.
* Civil and geotechnical engineering applications include imaging voids, rebar, and anomalies in concrete structures, as well as characterizing subsurface geology through seismic reflection and refraction.
* Petroleum engineering applications include characterization of reservoir rocks and fluids, providing insights into porosity and permeability.

This compendium of applications underscores the versatility and significance of
3D voxel data sets derived from CT and MRI as a fundamental ingredient of
model development for patient-specific, site-specific, or as-built
geometry.  This concept is referred to as a *digital twin*.
The geometric aspects of a digital twin involve image processing,[^1] and in particular,
segmentation.

[^1]: Image processing subtopics can include acquisition details (exposure
time, magnification, resolution), pre-processing (calibration, distortion
control, noise reduction), stitching (feature detection, feature matching,
homography estimation, warping, blending), baseline correction (background
subtraction, normalization) and post-processing (image enhancement,
downscaling, and segmentation).

Segmentation is the process of classifying each voxel in a data set as
belonging to a specific class within the geometric
domain, typically as a unique material, air, or void.
We use a segmented 3D voxel data set as our start point.
We seek to transform the voxel set into a finite element mesh suitable
for numerical analysis, our end point.

The most direct way to undertake this conversion is to create a one-to-one map
from voxel to hexahedral element.  This rudimentary approach, while attractive due to its effectiveness and simplicity, can result in billions to trillions of elements that can overwhelm current-day solvers, create massive input/output bottlenecks, and result in excessive consumption of digital storage resources.

To achieve a more tractable number of elements, on the order of millions,
**downsampling** of the original image data can be used to decrease the effective
resolution, resulting in fewer voxels.  Additionally, **adaptivity** of the finite
element mesh, allowing non-uniform element
sizes to span several length scales, can be used.

# Statement of Need

Our endeavor to create a finite element mesh suitable for analysis from a segmentation
has ten specific requirements.  These requirements arise from the
unique character of our mission space, collaborators, and sponsors.
The first requirement comes from our start point.
The next five requirements stem from our end point.
The remaining four requirements follow from the workflow,
converting a segmentation into a mesh.

We list each of these requirements below:

(1) **Segmentation start point.**  Many finite element mesh applications use geometric
primitives (e.g., tessellations) or analytical geometry (e.g., splines, NURBS,
isosurfaces) as a start point.  In contrast, our start point is a segmentation
composed of a 3D stack of voxels.

(2) **All-hex mesh.**  While tetrahedral finite elements offer excellent
adaptivity characteristics, hexahedral finite elements offer superior performance
characteristics.  Moreover, higher-order tetrahedral elements, such as the 10-noded
composite formulation, can result in a very large number of degrees of freedom,
potentially overwhelming solver and serialization capabilities.  In contrast,
8-noded hexahedral elements provide an economy of degrees of freedom and can
be underintegrated to offer excellent performance with incompressible materials,
so long as hourglass energy modes are monitored and controlled.

(3) **Multi-material.**  The mesh must be able to accommodate numerous materials
throughout the domain.  For example, mesh generation for microstructures that have
prolific grain boundaries must be handled.

(4) **Multi-scale.**  The characteristic length scale of the features we seek to resolve
can be significantly smaller than the length scales of the overall domain.  Mesh
adaptivity can be helpful to reconcile these differences of scale.

(5) **Quality.**  Elements must retain sufficient quality to ensure use with numerical
computation.  Common quality metrics include minimum scaled Jacobian, skew, edge
ratio, and minimum angle.  Poor element quality can lead to inaccurate results
and convergence issues.

(6) **Input/Output Format.** The segmentation input format must support
both NumPy [@numpy_format]
and SPN [@cubit_sandia].
The mesh output must support both
ExodusII [@schoof1994exodus] and
ABAQUS [@simulia_abaqus].

(7) **Automated.**  Creating finite element meshes from image stacks can be a
time-consuming task, often representing the biggest bottleneck in an analyst's
workflow.  The workflow from segmentation to mesh should be completely automated,
without the need for human intervention.

(8) **Fast.**  The program must be as fast or faster than other available mesh generation
software.

(9) **Open-source.**  All the source code must be freely available as open-source software.

(10) **Professional.**  The application source code must reside on GitHub, Gitlab, or
an equivalent platform, have a comprehensive test suite to assure quality and reproducibility,
use continuous integration and continuous delivery principles (CI/CD), and be
supported by the authors.

# Existing Art

While there are many applications suitable for creation of finite element meshes,
we found none that could meet all of our requirements.

Commercial (closed-source) applications such as
Altair HyperMesh [@altair_hypermesh],
ANSYS Meshing [@ansys_meshing],
COMSOL Multiphysics [@comsol_multiphysics],
Cubit [@cubit_sandia],
Gridgen [@gridgen_pointwise],
Hexotic [@hexotic; @marechal2009advances; @marechal2016all],
Sculpt [@owen2014parallel; @owen2014validation],
Siemens Simcenter [@siemens_simcenter],
and
TrueGrid [@truegrid]
were not suitable due to our open-source requirement.

While existing open-source software packages provide valuable features for
finite element mesh generation, they often lack one or more of our specific
requirements, particularly in terms of segmentation input, all-hex meshing, and
comprehensive input/output format support.  We found specific limitations as
follows:

* cinolib supports all-hex meshing, but it does not support NumPy, SPN, ExodusII, and ABAQUS file types [@livesu2019cinolib].

<!-- * deal.II does not support NumPy, SPN, and ABAQUS file types [@dealII]. -->
<!-- deal.II is a library to support FEA ; it is not a mesher.  It relies on external mesh generators (such as GMSH, Netgen, or TetGen) -->

* FEniCS supports segmentation, but does not create an all-hex only mesh.  It does not support SPN and ABAQUS file types [@fenics].

* FreeFEM supports segmentation, but primarily focuses on geometric primitives.  It does not support all-hex meshing.  It does not support NumPy or SPN input formats.  It can output to ExodusII but not to ABAQUS [@freefem].

* GetFEM++ supports segmentation, but does not produce a mesh that is all-hex. It does not have NumPy or SPN support.  It supports ExodusII but not ABAQUS outputs [@getfem].

* GMSH primarily uses geometric primitives, with limited support for voxel-based segmentation.  It supports all-hex meshing, but not exclusively.  It lacks direct NumPy and SPN input formats.  It can output to ExodusII but not to ABAQUS [@gmshtool].

* MeshLab supports segmentation, but not all-hex meshing.  It has limited multi-material support and no mesh adaptivity.  It does not support NumPy, SPN, ExodusII, and ABAQUS file types [@meshlab].

* NETGEN is an automatic tetrahedral mesh generator.  It supports neither segmentation input nor all-hex mesh generation.  It does not support NumPy and SPN file types [@netgen].

* OpenFOAM primarily uses geometric definitions, has limited voxel support, and does not support exclusively all-hex meshing.  It does not support NumPy, SPN, and ABAQUS file types. [@openfoam].

* TetGen primarily works with geometric input, such as points, edges, and faces, and creates tetrahedral elements. It supports neither segmentation input nor all-hex mesh generation.  It does not support NumPy, SPN, and ExodusII file types [@tetgen].

# Features and Implementation

`automesh` is available for download from [crates.io](https://crates.io/crates/automesh), the official package registry for the Rust programming language.  A Python interface is also available from [PyPI](https://pypi.org/project/automesh/).  The [User Guide](https://autotwin.github.io/automesh/) introduces the segmentation-to-mesh workflow, covers software installation, and contains examples based on unit tests and larger analysis sets.  Source code can be obtained from [GitHub](https://github.com/autotwin/automesh) [@automesh].

One significant attribute of `automesh` is its implementation in Rust, a modern programming language known for its performance and memory safety. Rust's ownership model prevents common programming errors such as null pointer dereferencing and buffer overflows, ensuring that `automesh` operates reliably and securely. Furthermore, Rust's zero-cost abstractions allow `automesh` to deliver high performance without sacrificing code readability or maintainability. The language's strong type system and concurrency features also enhance the robustness, making it well-suited for handling complex computational tasks efficiently [@rust_programming_language].

`automesh` is used via its command line interface.  `automesh` directly converts voxels classified by a non-negative integer in the segmentation input file into a hexahedral finite element mesh.  Segmented classes can be arbitrarily eliminated from the resultant mesh.  For example, air or void surrounding a subject, often segmented with integer `0`, can be eliminated, which can significantly reduce the resultant mesh size and complexity.

**Defeaturing** is accomplished by specifying a voxel threshold.  A cluster of voxels, defined as two or more voxels that share a face (edge and node sharing do not constitute a cluster) with count at or above the threshold will be preserved, whereas a cluster of voxels with a count below the threshold will be eliminated through resorption into the surrounding material.  Figure&nbsp;1 shows an example of a mesh before and after defeaturing.

![Example of defeaturing: (left) mesh prior to defeaturing, (right) mesh after defeaturing.](figures/blob_defeatured_iso2_high_res.png)

`automesh` provides both Laplace [@sorkine2005laplacian] and Taubin [@taubin1995signal] formulations for mesh **smoothing**, with optional hierarchical control.  Hierarchical control classifies all nodes as prescribed, boundary, or interior.  With hierarchical control, the updated (smoothed) position of a boundary node is influenced only by prescribed nodes and other boundary nodes; whereas, the updated position of an interior node is influenced by all types of nodes (prescribed, boundary, and interior).   Smoothing with hierarchical control can help maintain domain boundaries better than smoothing alone [@chen2010mri].

Figure&nbsp;2 shows `automesh` results of Taubin smoothing on an all-hexahedral mesh in the shape of a sphere (green) with a concentric shell (yellow).  Similar to the original demonstration of Taubin smoothing on a tessellation [@taubin1995signal], we apply Taubin smoothing to a hexahedral domain, with noise added to the $x > 0$ hemisphere.

![Example of Taubin smoothing applied to a volume mesh: (top left) isometric view of mesh with noised $x > 0$ hemisphere, (top right) noised domain with cut plane, (bottom left) smoothed mesh after 200 iterations, (bottom right) smoothed mesh with cut plane.](figures/taubin_composite.png)

Unlike Laplace smoothing, which drastically reduces volume, Taubin smoothing nearly preserves volumes.  For the example in Figure&nbsp;2, Laplace reduced the volume by 16% in 10 iterations; whereas, Taubin increased the volume by 1% in 200 iterations, while effectively smoothing.  With Taubin, the noise in the $x > 0$ hemisphere was removed, with a very small volumetric change.  The $x < 0$ hemisphere did not degrade from its original configuration.  Notice the smoothing capability of `automesh` is available for any `.inp` or `.stl` input format, regardless of whether the mesh originated from a segmentation.

Surface reconstruction of a 3D, all-triangular mesh **isosurface** can also be created from a segmentation.  Figure&nbsp;3 shows a two-material laser weld segmentation, composed of approximately 6.7&nbsp;million voxels [@polonsky2023toward; @karlson2023toward]. The workflow used defeaturing and smoothing to create an `.stl` isosurface output composed of 762,396 facets.

![Surface reconstruction of a real weld: (top) isosurface mesh generated from a CT segmentation, (bottom) example of voxel domain (left) used to generate a smooth isosurface (right).](figures/weld_composite.png)

`automesh` uses an **octree** for efficient performance [@meagher1980].  The octree serves as an adaptive segmentation, which accelerates defeaturing.  Our current implementation requires the octree to be strongly balanced, not weakly balanced.

Octree balancing refers to how much neighboring cells can differ in their level of refinement and the type of adjacency (face, edge, vertex) that is considered [@livesu2021optimal]:

* A **strongly balanced** octree requires neighboring cells that share a face, edge, or vertex to differ by no more than one level of refinement.
* A **weakly balanced** octree requires neighboring cells that share a face to differ by no more than one level of refinement, while edge and vertex neighbors may differ by more than one level of refinement.

The top two items of Figure&nbsp;4 show a visualization in
HexaLab [@bracci2019hexalab] of an `automesh` octree composing a spherical domain.
The cut plane on the right item exposes the octree adaptivity.  Four levels of cell
refinement are present.

The bottom two items of Figure&nbsp;4 show application of `automesh` to a large problem,
illustrating the importance of the octree implementation.  The micro CT segmentation
input is composed of one billion voxels, representable in an octree with
only 10&nbsp;million cells.  Five million of those octree cells are removable void.
With this 200$\times$ reduction, only 36 seconds is required to create the mesh.

![Application of the `automesh` octree: (top left) octree mesh of a spherical domain, (top right) cut plane through the domain to expose the adaptivity of the octree, (bottom left), sagittal view of a micro CT of a spinal unit, courtesy of [@nicolella2025] (used with permission) with segmentation IDs `0` through `8`, (bottom right) axial view of `automesh` octree, used to create a finite element mesh from the segmentation.](figures/spine_composite.png)

## Conclusion

`automesh` presents a robust solution for automatic mesh generation from segmentation data, bridging the gap between 3D imaging modalities and finite element analysis.
By leveraging defeaturing, smoothing, and an efficient octree structure, `automesh` not only enhances the quality of the generated meshes but also significantly reduces computational overhead.
Its ability to handle multiple input and output formats, including support for both hexahedral and triangular elements, makes it versatile for various applications across engineering and biomedical fields.
The integration of automated workflows ensures that users can efficiently convert complex voxel data into high-quality meshes without manual intervention.


# Acknowledgements

This work was supported by the Office of Naval Research (Dr. Timothy Bentley)
under grant N0001424IP00025.

# References
