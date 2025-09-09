pub mod convert;
pub mod defeature;
pub mod diff;
pub mod extract;
pub mod input;
pub mod mesh;
pub mod metrics;
pub mod output;
pub mod remesh;
pub mod segment;
pub mod smooth;

use ndarray_npy::{ReadNpyError, WriteNpyError};
use netcdf::Error as ErrorNetCDF;
use std::{
    fmt::{self, Debug, Formatter},
    io::Error as ErrorIO,
};
use vtkio::Error as ErrorVtk;

pub struct ErrorWrapper {
    message: String,
}

impl Debug for ErrorWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\x1b[1;91m{}.\x1b[0m", self.message)
    }
}

impl From<ErrorIO> for ErrorWrapper {
    fn from(error: ErrorIO) -> ErrorWrapper {
        ErrorWrapper {
            message: error.to_string(),
        }
    }
}

impl From<ErrorNetCDF> for ErrorWrapper {
    fn from(error: ErrorNetCDF) -> ErrorWrapper {
        ErrorWrapper {
            message: error.to_string(),
        }
    }
}

impl From<ErrorVtk> for ErrorWrapper {
    fn from(error: ErrorVtk) -> ErrorWrapper {
        ErrorWrapper {
            message: error.to_string(),
        }
    }
}

impl From<ReadNpyError> for ErrorWrapper {
    fn from(error: ReadNpyError) -> ErrorWrapper {
        ErrorWrapper {
            message: error.to_string(),
        }
    }
}

impl From<String> for ErrorWrapper {
    fn from(message: String) -> ErrorWrapper {
        ErrorWrapper { message }
    }
}

impl From<&str> for ErrorWrapper {
    fn from(message: &str) -> ErrorWrapper {
        ErrorWrapper {
            message: message.to_string(),
        }
    }
}

impl From<WriteNpyError> for ErrorWrapper {
    fn from(error: WriteNpyError) -> ErrorWrapper {
        ErrorWrapper {
            message: error.to_string(),
        }
    }
}
