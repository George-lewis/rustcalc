#![allow(clippy::non_ascii_literal, clippy::bind_instead_of_map)]

use super::{
    constants::{Constant, ConstantType},
    operators::{Operator, OperatorType},
};

const NUMBER_CHARACTERS: [char; 11] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.'];
const PAREN_CHARACTERS: [char; 2] = ['(', ')'];

pub trait Representable {
    fn repr(&self) -> &[&str];
}

pub(super) fn get_by_repr<'a, T: Representable>(
    search: &str,
    list: &'a [T],
) -> Option<(&'a T, &'a &'a str)> {
    list.iter().find_map(|t| {
        t.repr()
            .iter()
            .find(|repr| search.to_lowercase().starts_with(&repr.to_lowercase()))
            .and_then(|repr| Option::Some((t, repr)))
    })
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ParenType {
    Left,
    Right,
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Token {
    Number { value: f64 },
    Operator { kind: OperatorType },
    Paren { kind: ParenType },
    Constant { kind: ConstantType },
}

impl Token {
    pub const fn paren(c: char) -> Option<(Self, ParenType)> {
        let kind = match c {
            '(' => ParenType::Left,
            ')' => ParenType::Right,
            _ => return None
        };
        Some((Self::Paren { kind }, kind))
    }
    pub fn next_number(string: &str) -> String {
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
                ParenType::Left => "(".to_string(),
                ParenType::Right => ")".to_string(),
            },
            Self::Constant { kind } => Constant::by_type(*kind).repr[0].to_string(),
        }
    }
}
