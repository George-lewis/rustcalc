#![allow(clippy::module_name_repetitions)]

pub use super::lib::DoEvalResult;
pub use std::io::Error as IoError;

/// Error type for errors stemming from cli code, which includes `Errors` thrown by the library
pub enum Error<'a> {
    Assignment,
    Io(IoError),
    Library(DoEvalResult<'a, 'a>),
}

impl<'a> From<DoEvalResult<'a, 'a>> for Error<'a> {
    fn from(error: DoEvalResult<'a, 'a>) -> Self {
        Self::Library(error)
    }
}

impl From<IoError> for Error<'_> {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
