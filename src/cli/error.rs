pub use super::lib::model::errors::{ContextualError as ContextualLibError, Error as LibError};
pub use std::io::Error as IoError;

/// Error type for errors stemming from cli code, which includes `Errors` thrown by the library
pub enum Error {
    Assignment,
    Io(IoError),
    Library(ContextualLibError),
}

impl<'a> From<ContextualLibError> for Error {
    fn from(error: ContextualLibError) -> Self {
        Self::Library(error)
    }
}

impl From<IoError> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
