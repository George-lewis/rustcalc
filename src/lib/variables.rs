#![allow(clippy::non_ascii_literal)]
use super::tokens::Representable;

#[derive(Clone, Debug, PartialEq)]
pub struct Variable {
    pub repr: String,
    pub value: f64
}

impl Representable for Variable {
    fn repr(&self) -> String {
        self.repr
    }
}
