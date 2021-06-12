use std::{
    fmt::Display,
    iter
};

use colored::{ColoredString, Colorize};
use rustmatheval::model::functions::Functions;

use crate::{funcs::format_func_name, vars::format_var_name};

use super::lib::model::{
    operators::{Associativity, OperatorType},
    tokens::{ParenType, Token},
};

/// Creates a colored string representation of the input tokens
pub fn stringify(tokens: &[Token]) -> String {
    _stringify(tokens, color_cli)
}

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

/// Color
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
            Functions::User(func) => format_func_name(&func.name),
        },
        Token::Paren { .. } => string.red(),
        Token::Constant { .. } => string.yellow(),
        Token::Variable { .. } => format_var_name(string),
    }
}

#[allow(clippy::unnested_or_patterns)]
fn wants_space(cur: &Token, next: &Token) -> usize {
    match (cur, next) {
        (
            Token::Operator {
                inner: Functions::Builtin(op),
            },
            _,
        ) => ![
            OperatorType::Sin,
            OperatorType::Cos,
            OperatorType::Tan,
            OperatorType::Sqrt,
        ]
        .contains(&op.kind) as _,
        | (
            _,
            Token::Paren {
                kind: ParenType::Right,
            },
        )
        | (Token::Number { .. }, _)
        | (Token::Variable { .. }, _)
        | (Token::Constant { .. }, _) => 1,
        _ => 0,
    }
}

fn exclude_space(_cur: &Token, next: &Token) -> bool {
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

fn _stringify<F, T: Display>(tokens: &[Token], colorize: F) -> String
where
    F: Fn(&str, &Token) -> T,
{
    let last = iter::once((tokens.last().expect("Expected `tokens` to contain at least one token"), 0));
    tokens
        .windows(2)
        .map(|window| {
            let (cur, next) = (&window[0], &window[1]);
            let spaces = if exclude_space(cur, next) {
                0
            } else {
                wants_space(cur, next)
            };
            (cur, spaces)
        })
        .chain(last)
        .map(|(token, space)| {
            let colored = colorize(&ideal_repr(token), token);
            format!("{}{}", colored, " ".repeat(space))
        })
        .collect()
}
