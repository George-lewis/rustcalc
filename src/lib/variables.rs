use super::representable::{get_by_repr, Searchable};

#[derive(Clone, Debug, PartialEq)]
/// Represents a user definable variable
pub struct Variable {
    pub repr: String,
    pub value: f64,
}

impl Searchable for Variable {
    fn search<'a, 'b>(&'a self, search: &'b str) -> Option<(&'a Self, usize)> {
        // Case sensitive
        if search.starts_with(&self.repr) {
            Some((self, self.repr.len()))
        } else {
            None
        }
    }
}

impl Variable {
    /// Searches for the first variable in `vars` that matches the representation given by `text`
    pub fn next_variable<'a, 'b>(
        text: &'a str,
        vars: &'b [Variable],
    ) -> Option<(&'b Variable, usize)> {
        get_by_repr(text, vars)
    }

    /// Returns whether or not the given representation could reference a valid variable
    pub fn is(repr: &str) -> bool {
        repr.starts_with('$')
    }
}
