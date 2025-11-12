//! [![book](https://img.shields.io/badge/automesh-Book-blue?logo=mdbook&logoColor=000000)](https://autotwin.github.io/automesh)
//! [![crates](https://img.shields.io/crates/v/automesh?logo=rust&logoColor=000000&label=Crates&color=32592f)](https://crates.io/crates/automesh)
//! [![docs](https://img.shields.io/badge/Docs-API-e57300?logo=docsdotrs&logoColor=000000)](https://docs.rs/automesh)
//! [![pypi](https://img.shields.io/pypi/v/automesh?logo=pypi&logoColor=FBE072&label=PyPI&color=4B8BBE)](https://pypi.org/project/automesh)
//! [![docs](https://img.shields.io/badge/Docs-API-8CA1AF?logo=readthedocs)](https://automesh.readthedocs.io)
//! [![DOI](https://img.shields.io/badge/DOI-10.5281/zenodo.13845433-blue)](https://doi.org/10.5281/zenodo.13845433)
//!
//! Automatic mesh generation.

#![doc(html_logo_url = "https://github.com/autotwin/automesh/blob/main/docs/logo.png?raw=true")]

#[cfg(feature = "python")]
mod py;

mod fem;
mod tessellation;
mod tree;
mod voxel;

pub use fem::{
    Blocks, Connectivity, FiniteElementMethods, FiniteElementSpecifics, FiniteElements, HEX,
    HexahedralFiniteElements, Size, Smoothing, TET, TRI, TetrahedralFiniteElements,
    TriangularFiniteElements,
};
pub use tessellation::Tessellation;
pub use tree::Octree;
pub use voxel::{Extraction, Nel, Remove, Scale, Translate, VoxelData, Voxels};

use conspire::math::{TensorRank1, TensorRank1Vec};

/// The number of spatial dimensions.
pub const NSD: usize = 3;

/// A three-dimensional coordinate.
pub type Coordinate = TensorRank1<NSD, 1>;

/// A vector of three-dimensional coordinates.
pub type Coordinates = TensorRank1Vec<NSD, 1>;

/// A three-dimensional vector.
pub type Vector = TensorRank1<NSD, 1>;

/// A vector of three-dimensional vectors.
pub type Vectors = TensorRank1Vec<NSD, 1>;
