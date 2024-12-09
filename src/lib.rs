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
mod tree;
mod voxel;

pub use fem::{Element, FiniteElements, Smoothing};
pub use tree::{OcTree, Tree};
pub use voxel::{VoxelData, Voxels};

use flavio::{math::TensorRank1Vec, mechanics::Vector as VectorFlavio};

pub type Coordinate = VectorFlavio<1>;
pub type Coordinates = TensorRank1Vec<3, 1>;
pub type Points = TensorRank1Vec<3, 1>;
pub type Vector = VectorFlavio<1>;

const ELEMENT_NUM_NODES: usize = 8;
const ELEMENT_NUMBERING_OFFSET: usize = 1;
const ELEMENT_TYPE: &str = "C3D8R";
const NODE_NUMBERING_OFFSET: usize = 1;
const NSD: usize = 3;
