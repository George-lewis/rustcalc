use std::fmt::Display;

use colored::{ColoredString, Colorize};

use super::lib::model::{
    operators::{Associativity, Operator, OperatorType},
    tokens::{ParenType, Token},
};

/// Creates a colored string representation of the input tokens
pub fn stringify(tokens: &[Token]) -> String {
    _stringify(tokens, color_cli)
}

/// Color
fn color_cli(string: &str, token: &Token) -> ColoredString {
    match token {
        Token::Number { .. } => string.clear(),
        Token::Operator { kind } => {
            let op = Operator::by_type(*kind);
            if op.associativity == Associativity::Left {
                string.green().bold()
            } else {
                string.blue().bold()
            }
        }
        Token::Paren { .. } => string.magenta(),
        Token::Constant { .. } => string.yellow(),
        Token::Variable { .. } => string.green(),
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
    for (idx, token) in tokens.iter().enumerate() {
        let colored: T = colorize(&token.ideal_repr(), token);
        let append = match *token {
            Token::Number { .. } | Token::Constant { .. } | Token::Variable { .. } => {
                let is_r_paren = matches!(
                    tokens.get(idx + 1),
                    Some(Token::Paren {
                        kind: ParenType::Right
                    })
                );

                let is_op = matches!(tokens.get(idx + 1), Some(Token::Operator { .. }));

                let no_space = matches!(
                    tokens.get(idx + 1),
                    Some(Token::Operator {
                        kind: OperatorType::Pow | OperatorType::Factorial
                    })
                );

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
            Token::Operator { kind } => {
                let op = Operator::by_type(kind);

                match op.associativity {
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

                        let wants_implicit_paren = ![
                            OperatorType::Positive,
                            OperatorType::Negative,
                            OperatorType::Pow,
                        ]
                        .contains(&op.kind);

                        if wants_implicit_paren && !is_l_paren {
                            implicit_paren += 1;
                            let l_paren: T = colorize(
                                "(",
                                &Token::Paren {
                                    kind: ParenType::Left,
                                },
                            );
                            format!("{}{}", colored, l_paren)
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
                    let is_pow_or_r_paren = matches!(
                        tokens.get(idx + 1),
                        Some(
                            Token::Operator {
                                kind: OperatorType::Pow
                            } | Token::Paren {
                                kind: ParenType::Right,
                            }
                        )
                    );

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
    out
}
