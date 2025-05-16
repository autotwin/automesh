use super::{Coordinate, Coordinates, NSD, voxel::IntermediateError};
use conspire::math::{Tensor, TensorArray};
use ndarray_npy::{ReadNpyError, WriteNpyError};
use netcdf::Error as ErrorNetCDF;
use pyo3::{exceptions::PyTypeError, prelude::*};
use std::{convert::From, io::Error as ErrorIO};
use vtkio::Error as ErrorVtk;

/// [![book](https://img.shields.io/badge/automesh-Book-blue?logo=mdbook&logoColor=000000)](https://autotwin.github.io/automesh)
/// [![crates](https://img.shields.io/crates/v/automesh?logo=rust&logoColor=000000&label=Crates&color=32592f)](https://crates.io/crates/automesh)
/// [![docs](https://img.shields.io/badge/Docs-API-e57300?logo=docsdotrs&logoColor=000000)](https://docs.rs/automesh)
/// [![pypi](https://img.shields.io/pypi/v/automesh?logo=pypi&logoColor=FBE072&label=PyPI&color=4B8BBE)](https://pypi.org/project/automesh)
/// [![docs](https://img.shields.io/badge/Docs-API-8CA1AF?logo=readthedocs)](https://automesh.readthedocs.io)
/// [![DOI](https://img.shields.io/badge/DOI-10.5281/zenodo.13845433-blue)](https://doi.org/10.5281/zenodo.13845433)
///
/// Automatic mesh generation.
#[pymodule]
fn automesh(m: &Bound<'_, PyModule>) -> PyResult<()> {
    super::fem::py::register_module(m)?;
    super::voxel::py::register_module(m)?;
    Ok(())
}

pub type PyCoordinates = Vec<[f64; NSD]>;

pub trait IntoFoo<T> {
    fn as_foo(&self) -> T;
}

impl IntoFoo<Coordinates> for PyCoordinates {
    fn as_foo(&self) -> Coordinates {
        self.iter().map(|entry| Coordinate::new(*entry)).collect()
    }
}

impl IntoFoo<PyCoordinates> for Coordinates {
    fn as_foo(&self) -> PyCoordinates {
        self.iter().map(|entry| entry.as_array()).collect()
    }
}

pub struct PyIntermediateError {
    message: String,
}

impl From<ErrorIO> for PyIntermediateError {
    fn from(error: ErrorIO) -> PyIntermediateError {
        PyIntermediateError {
            message: error.to_string(),
        }
    }
}

impl From<ErrorNetCDF> for PyIntermediateError {
    fn from(error: ErrorNetCDF) -> PyIntermediateError {
        PyIntermediateError {
            message: error.to_string(),
        }
    }
}

impl From<ErrorVtk> for PyIntermediateError {
    fn from(error: ErrorVtk) -> PyIntermediateError {
        PyIntermediateError {
            message: error.to_string(),
        }
    }
}

impl From<ReadNpyError> for PyIntermediateError {
    fn from(error: ReadNpyError) -> PyIntermediateError {
        PyIntermediateError {
            message: error.to_string(),
        }
    }
}

impl From<String> for PyIntermediateError {
    fn from(message: String) -> PyIntermediateError {
        PyIntermediateError { message }
    }
}

impl From<&str> for PyIntermediateError {
    fn from(error: &str) -> PyIntermediateError {
        PyIntermediateError {
            message: error.to_string(),
        }
    }
}

impl From<WriteNpyError> for PyIntermediateError {
    fn from(error: WriteNpyError) -> PyIntermediateError {
        PyIntermediateError {
            message: error.to_string(),
        }
    }
}

impl From<PyIntermediateError> for PyErr {
    fn from(error: PyIntermediateError) -> PyErr {
        PyTypeError::new_err(error.message)
    }
}

impl From<IntermediateError> for PyIntermediateError {
    fn from(error: IntermediateError) -> PyIntermediateError {
        PyIntermediateError {
            message: error.message,
        }
    }
}
