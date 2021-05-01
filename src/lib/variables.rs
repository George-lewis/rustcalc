use macros::Searchable;

use super::searchable::{get_by_repr, Searchable};

#[derive(Searchable, Clone, Debug, PartialEq)]
/// Represents a user definable variable
pub struct Variable {
    #[representation]
    pub repr: String,
    pub value: f64,
}

impl Variable {
    /// Searches for the first variable in `vars` that matches the representation given by the start of `text`
    /// * `text` - The string to search. Must start with the name of a variable (not a '$') but can
    /// be arbitrarily long. Matches are case sensitive.
    /// * `vars` - A slice of [Variable]s to check for
    pub fn next_variable<'a>(text: &str, vars: &'a [Self]) -> Option<(&'a Self, usize)> {
        get_by_repr(text, vars)
    }

    /// Returns whether or not the given representation could reference a valid variable
    pub fn is(repr: &str) -> bool {
        repr.starts_with('$')
    }
}
