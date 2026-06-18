pub mod convert;
pub mod defeature;
pub mod diff;
pub mod extract;
pub mod io;
pub mod mesh;
pub mod metrics;
pub mod remesh;
pub mod segment;
pub mod smooth;

use std::{
    fmt::{self, Debug, Formatter},
    io::Error as ErrorIO,
};

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
