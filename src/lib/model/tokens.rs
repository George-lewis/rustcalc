#![allow(clippy::non_ascii_literal, clippy::bind_instead_of_map)]

use std::rc::Rc;

use super::{
    constants::{Constant, ConstantType},
    errors::ParserError,
    functions::Functions,
    operators::{Operator, OperatorType},
    variables::Variable,
};

const NUMBER_CHARACTERS: [char; 11] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.'];
const PAREN_CHARACTERS: [char; 2] = ['(', ')'];
const COMMA_CHARACTERS: [char; 1] = [','];

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ParenType {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct StringTokenInterface<'repr, Inner> {
    pub inner: Inner,
    pub repr: &'repr str,
    pub idx: usize,
}

impl<T> StringTokenInterface<'_, T> {
    pub const fn stride(&self) -> usize {
        self.repr.len()
    }
    pub const fn end(&self) -> usize {
        self.idx + self.stride()
    }
}

pub type PartialToken<'repr, 'funcs> =
    StringTokenInterface<'repr, Result<Token<'funcs>, ParserError>>;
pub type StringToken<'repr, 'funcs> = StringTokenInterface<'repr, Token<'funcs>>;

#[derive(Debug, Clone)]
pub enum Tokens<'vars, 'funcs> {
    String(StringToken<'vars, 'funcs>),
    Synthetic(Token<'funcs>),
}

impl<'vars, 'funcs> Tokens<'vars, 'funcs> {
    pub const fn token(&self) -> &Token<'funcs> {
        match self {
            Tokens::String(st) => &st.inner,
            Tokens::Synthetic(t) => t,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'funcs> {
    Number { value: f64 },
    Operator { inner: Functions<'funcs> },
    Paren { kind: ParenType },
    Constant { inner: &'static Constant },
    Variable { inner: Rc<Variable> },
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
    pub fn number(string: &str) -> Option<(Self, &str)> {
        let repr = Self::next_number(string);
        match repr.parse::<f64>() {
            Ok(value) => Some((Self::Number { value }, repr)),
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
    fn next_number(string: &str) -> &str {
        let len = string
            .chars()
            .take_while(|c| NUMBER_CHARACTERS.contains(c))
            .count();
        &string[..len]
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
    pub fn is_next_comma(string: &str) -> bool {
        Self::is_next_t(string, &COMMA_CHARACTERS)
    }

    pub const fn has_prefix(&self) -> bool {
        matches!(self, Token::Operator { inner: Functions::User(_) } | Token::Variable { .. })
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

    // #[test]
    // fn test_number() {
    //     let result = Token::number("123").unwrap();
    //     assert_eq!(result.1, 3);
    //     match result.0 {
    //         Token::Number { value } => assert_same!(value, 123.0),
    //         _ => panic!("Expected a number"),
    //     };

    //     let result = Token::number("999.544").unwrap();
    //     assert_eq!(result.1, 7);
    //     match result.0 {
    //         Token::Number { value } => assert_same!(value, 999.544),
    //         _ => panic!("Expected a number"),
    //     };
    // }

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
