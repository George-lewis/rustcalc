#![allow(clippy::non_ascii_literal, clippy::bind_instead_of_map)]

use super::{
    constants::{Constant, ConstantType},
    operators::{Operator, OperatorType},
    variables::Variable,
};

const NUMBER_CHARACTERS: [char; 11] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.'];
const PAREN_CHARACTERS: [char; 2] = ['(', ')'];

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ParenType {
    Left,
    Right,
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Token<'a> {
    Number { value: f64 },
    Operator { kind: OperatorType },
    Paren { kind: ParenType },
    Constant { kind: ConstantType },
    Variable { inner: &'a Variable },
}

impl Token<'_> {
    pub fn paren(c: char) -> Option<(Self, ParenType)> {
        Self::paren_type(c).map(|kind| (Self::Paren { kind }, kind))
    }
    pub const fn paren_type(c: char) -> Option<ParenType> {
        match c {
            '(' => Some(ParenType::Left),
            ')' => Some(ParenType::Right),
            _ => None,
        }
    }
    pub fn number(string: &str) -> Option<(Self, usize)> {
        let repr = Self::next_number(string);
        match repr.parse::<f64>() {
            Ok(value) => Some((Self::Number { value }, repr.len())),
            Err(..) => None,
        }
    }
    fn next_number(string: &str) -> String {
        string
            .chars()
            .take_while(|c| NUMBER_CHARACTERS.contains(c))
            .collect::<String>()
    }
    fn is_next_t(string: &str, list: &[char]) -> bool {
        string.chars().next().map_or(false, |c| list.contains(&c))
    }
    pub fn is_next_number(string: &str) -> bool {
        Self::is_next_t(string, &NUMBER_CHARACTERS)
    }
    pub fn is_next_paren(string: &str) -> bool {
        Self::is_next_t(string, &PAREN_CHARACTERS)
    }
    pub fn ideal_repr(&self) -> String {
        match self {
            Self::Number { value } => value.to_string(),
            Self::Operator { kind } => Operator::by_type(*kind).repr[0].to_string(),
            Self::Paren { kind } => match kind {
                ParenType::Left => '('.to_string(),
                ParenType::Right => ')'.to_string(),
            },
            Self::Constant { kind } => Constant::by_type(*kind).repr[0].to_string(),
            Self::Variable { inner } => format!("${}", inner.name),
        }
    }
}
