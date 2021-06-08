use std::fmt::Display;

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

pub fn ideal_repr(tok: &Token) -> String {
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
    }
}

/// Color
fn color_cli(string: &str, token: &Token) -> ColoredString {
    match token {
        Token::Number { .. } => string.clear(),
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

/// Convert a list of `Token`s into a string representation
/// * `tokens` - The tokens
/// * `colorize` - A function that colors tokens
#[allow(clippy::too_many_lines)]
fn _stringify<F, T: Display>(tokens: &[Token], colorize: F) -> String
where
    F: Fn(&str, &Token) -> T,
{
    let mut out = String::new();
    let mut implicit_paren: usize = 0;
    let make_implicit_paren = |n: usize| {
        colorize(
            &")".repeat(n),
            &Token::Paren {
                kind: ParenType::Right,
            },
        )
    };
    for (idx, token) in tokens.iter().enumerate() {
        let colored: T = colorize(&ideal_repr(token), token);
        let append = match *token {
            Token::Number { .. } | Token::Constant { .. } | Token::Variable { .. } => {
                let is_r_paren = matches!(
                    tokens.get(idx + 1),
                    Some(Token::Paren {
                        kind: ParenType::Right
                    })
                );

                let is_op = matches!(tokens.get(idx + 1), Some(Token::Operator { .. }));

                let no_space = match tokens.get(idx + 1) {
                    Some(Token::Operator {
                        inner: Functions::Builtin(inner),
                    }) => inner.kind == OperatorType::Pow || inner.kind == OperatorType::Factorial,
                    _ => false,
                };

                let last = idx == tokens.len() - 1;

                let appendix = if implicit_paren > 0 {
                    let space = if last || no_space { "" } else { " " };
                    let r_paren: T = colorize(
                        &")".repeat(implicit_paren),
                        &Token::Paren {
                            kind: ParenType::Right,
                        },
                    );
                    format!("{}{}", r_paren, space)
                } else if last {
                    "".to_string()
                } else if !(is_r_paren || is_op) {
                    ", ".to_string()
                } else if is_op && !no_space {
                    " ".to_string()
                } else {
                    "".to_string()
                };

                implicit_paren = 0;

                format!("{}{}", colored, appendix)
            }
            Token::Operator { inner: op } => {
                match op.associativity() {
                    Associativity::Left => {
                        let space = if idx == tokens.len() - 1 { "" } else { " " };
                        format!("{}{}", colored, space)
                    }
                    Associativity::Right => {
                        let is_l_paren = matches!(
                            tokens.get(idx + 1),
                            Some(Token::Paren {
                                kind: ParenType::Left
                            })
                        );

                        let wants_implicit_paren = match op {
                            Functions::Builtin(op) => ![
                                OperatorType::Positive,
                                OperatorType::Negative,
                                OperatorType::Pow,
                            ]
                            .contains(&op.kind),
                            Functions::User(_) => true,
                        };

                        if wants_implicit_paren && !is_l_paren {
                            implicit_paren += 1;
                            let l_paren: T = colorize(
                                "(",
                                &Token::Paren {
                                    kind: ParenType::Left,
                                },
                            );

                            // Special case for functions with no arguments
                            // We just add the `)` immediately, because this is the easiest way
                            // This will usually be user-defined [Function]s
                            let r_paren = if op.arity() == 0 {
                                let formatted = format!("{} ", make_implicit_paren(implicit_paren));
                                implicit_paren = 0;
                                formatted
                            } else {
                                "".to_string()
                            };
                            format!("{}{}{}", colored, l_paren, r_paren)
                        } else {
                            format!("{}", colored)
                        }
                    }
                }
            }
            Token::Paren { kind } => match kind {
                ParenType::Left => {
                    // Subtracts one bottoming out at 0 because `implicit_paren` is a `usize`
                    implicit_paren = implicit_paren.saturating_sub(1);
                    format!("{}", colored)
                }
                ParenType::Right => {
                    // Is this token the last one
                    let is_last = idx + 1 == tokens.len();

                    // Is the next token:
                    //   - Pow
                    //   - An R Paren
                    let is_pow_or_r_paren = match tokens.get(idx + 1) {
                        Some(Token::Operator {
                            inner: Functions::Builtin(inner),
                        }) => inner.kind == OperatorType::Pow,
                        Some(Token::Paren { kind }) => *kind == ParenType::Right,
                        _ => false,
                    };

                    if is_last || is_pow_or_r_paren {
                        format!("{}", colored)
                    } else {
                        format!("{} ", colored)
                    }
                }
            },
        };
        out.push_str(&append)
    }

    // In some cases, there may be some implicit parens left over
    format!("{}{}", out, make_implicit_paren(implicit_paren))
}
