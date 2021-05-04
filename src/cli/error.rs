pub use super::lib::model::errors::Error as LibError;
pub use std::io::Error as IoError;

/// Error type for errors stemming from cli code, which includes `Errors` thrown by the library
pub enum Error {
    Assignment,
    Io(IoError),
    Library(LibError),
}

impl From<LibError> for Error {
    fn from(error: LibError) -> Self {
        Self::Library(error)
    }
}

impl From<IoError> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
