pub use super::lib::model::errors::{ContextError as ContextLibError, Error as LibError};
pub use std::io::Error as IoError;

/// Error type for errors stemming from cli code, which includes `Errors` thrown by the library
pub enum Error<'a> {
    Assignment,
    Io(IoError),
    Library(ContextLibError<'a>),
}

impl<'a> From<ContextLibError<'a>> for Error<'a> {
    fn from(error: ContextLibError<'a>) -> Self {
        Self::Library(error)
    }
}

impl From<IoError> for Error<'_> {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
