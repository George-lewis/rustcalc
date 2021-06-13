#![allow(clippy::non_ascii_literal, clippy::bind_instead_of_map)]

use super::{
    constants::{Constant, ConstantType},
    functions::Functions,
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
    Operator { inner: Functions<'a> },
    Paren { kind: ParenType },
    Constant { inner: &'a Constant },
    Variable { inner: &'a Variable },
    Comma,
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
    pub fn operator(kind: OperatorType) -> Self {
        Self::Operator {
            inner: Functions::Builtin(Operator::by_type(kind)),
        }
    }
    pub fn constant(kind: ConstantType) -> Self {
        Self::Constant {
            inner: Constant::by_type(kind),
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
}

#[cfg(test)]
mod tests {
    #![allow(clippy::shadow_unrelated)]

    use super::{ParenType, Token};

    #[test]
    fn test_paren() {
        let l_paren = Token::paren('(').unwrap();
        assert!(matches!(
            l_paren.0,
            Token::Paren {
                kind: ParenType::Left
            }
        ));
        assert_eq!(l_paren.1, ParenType::Left);
        let r_paren = Token::paren(')').unwrap();
        assert!(matches!(
            r_paren.0,
            Token::Paren {
                kind: ParenType::Right
            }
        ));
        assert_eq!(r_paren.1, ParenType::Right);

        let none = Token::paren('a');
        assert!(matches!(none, None));
    }

    #[test]
    fn test_next_number() {
        assert_eq!(Token::next_number("1234567890"), "1234567890");
        assert_eq!(Token::next_number("1.234"), "1.234");
        assert_eq!(Token::next_number("555"), "555");
    }

    #[test]
    fn test_number() {
        let result = Token::number("123").unwrap();
        assert_eq!(result.1, 3);
        match result.0 {
            Token::Number { value } => assert_same!(value, 123.0),
            _ => panic!("Expected a number"),
        };

        let result = Token::number("999.544").unwrap();
        assert_eq!(result.1, 7);
        match result.0 {
            Token::Number { value } => assert_same!(value, 999.544),
            _ => panic!("Expected a number"),
        };
    }

    #[test]
    fn text_is_next_t() {
        let result = Token::is_next_t("a", &['a']);
        assert!(result);
        let result = Token::is_next_t("b", &['a']);
        assert!(!result);
        let result = Token::is_next_t("4b4bab4bb", &['b', 'a', '4']);
        assert!(result);
    }
}
