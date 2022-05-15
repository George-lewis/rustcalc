use std::{iter, borrow::Cow};

use colored::{ColoredString, Colorize};
use rustmatheval::model::{
    functions::Functions,
    operators::FUNCTIONAL_STYLE_OPERATORS,
    tokens::{PartialToken, StringToken, Tokens},
};

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
    _stringify(tokens)
}

pub trait StringableToken {
    fn spaces(&self, other: &Self) -> usize;
    fn token(&self) -> Option<&Token<'_>>;
    fn colorize(&self) -> ColoredString;
    fn repr(&self) -> Cow<'_, str>;
}

impl StringableToken for Token<'_> {
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

    fn colorize(&self) -> ColoredString {
        color_cli(&ideal_repr(self), self)
    }

    fn repr(&self) -> Cow<'_, str> {
        Cow::Owned(ideal_repr(self))
    }
}

impl StringableToken for StringToken<'_, '_> {
    fn spaces(&self, other: &Self) -> usize {
        other.idx - (self.idx + self.repr.len())
    }

    fn token(&self) -> Option<&Token<'_>> {
        Some(&self.inner)
    }

    fn colorize(&self) -> ColoredString {
        color_cli(self.repr, &self.inner)
    }

    fn repr(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.repr)
    }
}

impl StringableToken for PartialToken<'_, '_> {
    fn spaces(&self, other: &Self) -> usize {
        other.idx - (self.idx + self.repr.chars().count())
    }

    fn token(&self) -> Option<&Token<'_>> {
        self.inner.as_ref().ok()
    }

    fn colorize(&self) -> ColoredString {
        if let Ok(ref token) = self.inner {
            color_cli(self.repr, token)
        } else {
            self.repr.on_magenta()
        }
    }

    fn repr(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.repr)
    }
}

impl StringableToken for Tokens<'_, '_> {
    fn spaces(&self, other: &Self) -> usize {
        match self {
            Tokens::String(st) => match other {
                Tokens::String(st_) => st.spaces(st_),
                Tokens::Synthetic(syn) => st.inner.spaces(syn),
            },
            Tokens::Synthetic(syn) => {
                let other = other.token();
                syn.spaces(other)
            }
        }
    }

    fn token(&self) -> Option<&Token<'_>> {
        match self {
            Tokens::String(st) => Some(&st.inner),
            Tokens::Synthetic(syn) => Some(syn),
        }
    }

    fn colorize(&self) -> ColoredString {
        match self {
            Tokens::String(st) => st.colorize(),
            Tokens::Synthetic(syn) => syn.colorize(),
        }
    }

    fn repr(&self) -> Cow<'_, str> {
        match self {
            Tokens::String(st) => Cow::Borrowed(st.repr),
            Tokens::Synthetic(t) => Cow::Owned(ideal_repr(t)),
        }   
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
#[allow(clippy::unnested_or_patterns, clippy::cast_lossless)]
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

fn _stringify(tokens: &[impl StringableToken]) -> String {
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
            let space = cur.spaces(next);

            (cur, space)
        })
        // Insert the last token
        .chain(last)
        // Color
        .map(|(token, space)| format!("{}{}", token.colorize(), " ".repeat(space)))
        .collect()
}
