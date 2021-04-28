#![allow(clippy::non_ascii_literal)]

use super::{
    representable::{get_by_repr, Searchable},
    utils,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Variable {
    pub repr: String,
    pub value: f64,
}

impl Searchable for Variable {
    fn search<'a, 'b>(&'a self, search: &'b str) -> Option<(&'a Self, usize)> {
        // Case sensitive
        let search_str = utils::slice(search, 1, -0); //to ignore $
        if search_str.starts_with(&self.repr) {
            Some((self, self.repr.len()))
        } else {
            None
        }
    }
}

impl Variable {
    pub fn next_variable<'a, 'b>(
        text: &'a str,
        vars: &'b [Variable],
    ) -> Option<(&'b Variable, usize)> {
        get_by_repr(text, vars)
    }
    pub fn is(repr: &str) -> bool {
        repr.starts_with("$")
    }
}
