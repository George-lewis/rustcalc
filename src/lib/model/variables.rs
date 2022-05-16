use std::cell::Cell;
use std::{borrow::Borrow, rc::Rc};

use super::representable::{get_by_repr, Searchable};

pub const PREFIX: char = '$';

#[derive(Clone, Debug, PartialEq)]
/// Represents a variable, a value with a name
pub struct Variable {
    pub repr: String,
    pub value: Cell<f64>,
}

impl Searchable for Variable {
    fn search<'a, 'b>(&'a self, search: &'b str) -> Option<(&'a Self, &'b str)> {
        // Case sensitive
        if search.starts_with(&self.repr) {
            Some((self, &search[..self.repr.chars().count()]))
        } else {
            None
        }
    }
}

impl Variable {
    /// Searches for the first variable in `vars` that matches the representation given by the start of `text`
    /// * `text` - The string to search. Must start with the name of a variable (not a '$') but can
    /// be arbitrarily long. Matches are case sensitive.
    /// * `vars` - A slice of [Variable]s to check for
    pub fn next_variable<'a, 'b, Zelf: Borrow<Self>>(
        text: &'b str,
        vars: &'a [Zelf],
    ) -> Option<(&'a Zelf, &'b str)> {
        get_by_repr(text, vars)
    }

    /// Returns whether or not the given representation could reference a valid variable
    pub fn is(repr: &str) -> bool {
        repr.starts_with(PREFIX)
    }

    #[inline]
    pub fn rc<S: Into<String>>(repr: S, value: f64) -> Rc<Self> {
        let var = Self {
            repr: repr.into(),
            value: Cell::new(value),
        };
        Rc::new(var)
    }
}

#[cfg(test)]
mod tests {

    #![allow(clippy::non_ascii_literal)]

    use super::Variable;

    #[test]
    fn test_is() {
        assert!(Variable::is("$"));
        assert!(Variable::is("$a"));
        assert!(Variable::is("$b"));
        assert!(Variable::is("$1234"));
    }

    // #[allow(clippy::shadow_unrelated)]
    // #[test]
    // fn test_next_variable() {
    //     let vars = [
    //         Variable {
    //             repr: "abc".to_string(),
    //             value: Cell::new(1.0),
    //         },
    //         Variable {
    //             repr: "😂❤😂".to_string(),
    //             value: Cell::new(5.5),
    //         },
    //     ];
    //     let search = Variable::next_variable("abc", &vars).unwrap();
    //     assert_eq!(*search.0, vars[0]);
    //     assert_eq!(search.1, "abc");
    //     let search = Variable::next_variable("qqq", &vars);
    //     assert!(search.is_none());
    //     let search = Variable::next_variable("😂❤😂", &vars).unwrap();
    //     assert_eq!(*search.0, vars[1]);
    //     assert_eq!(search.1, "😂❤😂");
    // }
}
