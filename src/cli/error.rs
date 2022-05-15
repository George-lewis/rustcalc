#![allow(clippy::module_name_repetitions)]

pub use super::lib::DoEvalResult;

/// Error type for errors stemming from cli code, which includes `Errors` thrown by the library
pub enum Error<'a> {
    Assignment,
    Library(DoEvalResult<'a, 'a>),
}
impl<'a> From<DoEvalResult<'a, 'a>> for Error<'a> {
    fn from(error: DoEvalResult<'a, 'a>) -> Self {
        Self::Library(error)
    }
}
