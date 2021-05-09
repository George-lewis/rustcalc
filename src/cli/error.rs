pub use super::lib::model::errors::{ContextError as ContextLibError, Error as LibError};
pub use std::io::Error as IoError;

/// Error type for errors stemming from cli code, which includes `Errors` thrown by the library
pub enum Error {
    Assignment,
    Io(IoError),
    Library(ContextLibError),
}

impl<'a> From<ContextLibError> for Error {
    fn from(error: ContextLibError) -> Self {
        Self::Library(error)
    }
}

impl From<IoError> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
