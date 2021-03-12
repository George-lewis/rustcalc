#![allow(clippy::non_ascii_literal)]

use super::tokens::get_by_repr;
use super::tokens::Representable;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ConstantType {
    PI,
    E,
    Tau,
}

pub struct Constant {
    pub kind: ConstantType,
    pub repr: &'static [&'static str],
    pub value: f64,
}

impl Representable for Constant {
    fn repr(&self) -> &[&str] {
        self.repr
    }
}

static CONSTANTS: &[Constant] = &[
    Constant {
        kind: ConstantType::PI,
        repr: &["π", "pi"],
        value: std::f64::consts::PI,
    },
    Constant {
        kind: ConstantType::Tau,
        repr: &["τ", "tau"],
        value: std::f64::consts::TAU,
    },
    Constant {
        kind: ConstantType::E,
        repr: &["e"],
        value: std::f64::consts::E,
    },
];

impl Constant {
    pub fn by_type(kind: ConstantType) -> &'static Self {
        CONSTANTS.iter().find(|c| c.kind == kind).unwrap()
    }
    pub fn by_repr(repr: &str) -> Option<(&'static Self, &'static &'static str)> {
        get_by_repr(repr, CONSTANTS)
    }
    pub fn is(repr: &str) -> bool {
        Self::by_repr(repr).is_some()
    }
}
