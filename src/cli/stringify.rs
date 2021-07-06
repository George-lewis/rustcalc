use std::{borrow::Cow, fmt::Display, iter};

use colored::{ColoredString, Colorize};
use rustmatheval::model::{functions::Functions, operators::FUNCTIONAL_STYLE_OPERATORS, tokens::{PartialToken, StringToken}};

use crate::{funcs::format_func_name, vars::format_var_name};

use super::lib::model::{
    operators::{Associativity, OperatorType},
    tokens::{ParenType, Token},
};

/// Creates a colored string representation of the input tokens
pub fn stringify<FormatToken>(tokens: &[FormatToken]) -> String
where
    FormatToken: StringableToken,
{
    _stringify(tokens, color_cli)
}

pub trait StringableToken {
    fn to_string(&self) -> Cow<'_, str>;
    fn spaces(&self, other: &Self) -> usize;
    fn token(&self) -> Option<&Token<'_>>;
}

impl StringableToken for Token<'_> {
    fn to_string(&self) -> Cow<'_, str> {
        Cow::Owned(ideal_repr(self))
    }

    fn spaces(&self, other: &Self) -> usize {
        if exclude_space(other) {
            0
        } else {
            spaces(self)
        }
    }

    fn token(&self) -> Option<&Token<'_>> {
        Some(self)
    }
}

impl StringableToken for StringToken<'_, '_> {
    fn to_string(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.repr)
    }

    fn spaces(&self, other: &Self) -> usize {
        other.idx - (self.idx + self.repr.len())
    }

    fn token(&self) -> Option<&Token<'_>> {
        Some(&self.inner)
    }
}

impl StringableToken for PartialToken<'_, '_> {
    fn to_string(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.repr)
    }

    fn spaces(&self, other: &Self) -> usize {
        other.idx - (self.idx + self.repr.len())
    }

    fn token(&self) -> Option<&Token<'_>> {
        self.inner.as_ref().ok()
    }
}

/// Construct the ideal representation of a `Token`
fn ideal_repr(tok: &Token) -> String {
    match tok {
        Token::Number { value } => value.to_string(),
        Token::Operator {
            inner: Functions::Builtin(inner),
        } => inner.repr[0].to_string(),
        Token::Operator {
            inner: Functions::User(inner),
        } => inner.name.to_string(),
        Token::Paren { kind } => match kind {
            ParenType::Left => '('.to_string(),
            ParenType::Right => ')'.to_string(),
        },
        Token::Constant { inner } => inner.repr[0].to_string(),
        Token::Variable { inner } => inner.repr.to_string(),
        Token::Comma => ",".to_string(),
    }
}

/// Color tokens for the CLI
fn color_cli(string: &str, token: &Token) -> ColoredString {
    match token {
        Token::Number { .. } | Token::Comma => string.clear(),
        Token::Operator { inner: op } => match op {
            Functions::Builtin(_) => {
                if op.associativity() == Associativity::Left {
                    string.green().bold()
                } else {
                    string.blue().bold()
                }
            }
            Functions::User(_) => format_func_name(string),
        },
        Token::Paren { .. } => string.red(),
        Token::Constant { .. } => string.yellow(),
        Token::Variable { .. } => format_var_name(string),
    }
}

/// Determine if a space should be come after `cur` in a string representation.
#[allow(clippy::unnested_or_patterns)]
fn spaces(cur: &Token) -> usize {
    // Cases:
    // - Spaces after value types: numbers, variables, and constants
    // - Spaces after r_parens and commas
    // - Spaces after all operators except function-style ones: sin, cos, tan, sqrt, ..
    //   - and pow
    // - Otherwise no spaces
    match cur {
        Token::Operator {
            inner: Functions::Builtin(op),
        } => (!FUNCTIONAL_STYLE_OPERATORS.contains(&op.kind) && op.kind != OperatorType::Pow) as _,
        Token::Paren {
            kind: ParenType::Right,
        }
        | Token::Number { .. }
        | Token::Variable { .. }
        | Token::Constant { .. }
        | Token::Comma => 1,

        // Otherwise none
        _ => 0,
    }
}

/// Determine if there can or cannot be a space before `next` in a string representation
///
/// ## Returns
/// True -> There cannot be a space between these tokens
///
/// False -> Spaces are permitted between these tokens
fn exclude_space(next: &Token) -> bool {
    // Cases:
    // - No spaces before an r_paren
    // - No spaces before certain operators: pow, and factorial
    // - All else is permitted
    match next {
        Token::Paren {
            kind: ParenType::Right,
        }
        | Token::Comma => true,
        Token::Operator {
            inner: Functions::Builtin(op),
        } => [OperatorType::Pow, OperatorType::Factorial].contains(&op.kind),
        _ => false,
    }
}

trait FromBorrowedStr {
    fn convert(s: &str) -> Self;
}

impl FromBorrowedStr for ColoredString {
    fn convert(s: &str) -> Self {
        s.into()
    }
}

fn _stringify<'a, Colorize, Formatted>(
    tokens: &'a [impl StringableToken],
    colorize: Colorize,
) -> String
where
    Colorize: Fn(&str, &Token) -> Formatted,
    Formatted: Display + for<'b> From<&'b str>,
{
    // The last element of the slice
    // `std::slice::windows` does not include the last element as its own window
    // So we must add it ourselves
    //
    // The tuple is `(&Token, number_of_space: usize)`
    // There are not spaces after the last token, thus it is always zero
    let last = iter::once((
        tokens
            .last()
            .expect("Expected `tokens` to contain at least one token"),
        0,
    ));

    // Windows of size two let us determine if we want to insert a space
    // between them, given the context of the "current" and "next" token
    // Caveat: There will be no window for the last element, see above.
    tokens
        .windows(2)
        .map(|window| {
            let (cur, next) = (&window[0], &window[1]);

            // `exclude_space` determines if any conditions prevent there from being a space
            // and then `spaces` determines the number of spaces to insert, if they are permitted
            #[allow(clippy::collapsible_else_if)]
            let space = cur.spaces(next);

            (cur, space)
        })
        // Insert the last token
        .chain(last)
        // Color
        .map(|(token, space)| {
            let formatted = token.to_string();

            #[allow(clippy::option_if_let_else)]
            let colored = if let Some(token) = token.token() {
                colorize(&formatted, token)
            } else {
                formatted.as_ref().into()
            };
            format!("{}{}", colored, " ".repeat(space))
        })
        .collect()
}
