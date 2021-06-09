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
pub fn color_cli<'a>(string: &str, token: &impl Stringable<'a>) -> ColoredString {
    match token.matchable() {
        Token::Number { .. } => string.clear(),
        Token::Operator { kind } => {
            let op = Operator::by_type(kind);
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

pub trait Stringable<'a> {
    fn matchable(&self) -> Token;
    fn ideal_repr(&self) -> String;
}

impl<'a> Stringable<'a> for Token<'a> {
    fn matchable(&self) -> Token {
        *self
    }

    fn ideal_repr(&self) -> String {
        self.ideal_repr()
    }
}

// impl<'a> Into<Token<'a>> for (Token<'a>, Option<String>) {
//     fn into(self) -> Token<'a> {
//         self.0
//     }
// }

#[derive(Debug)]
pub struct StringToken<'a> {
    pub token: Token<'a>,
    pub repr: Option<String>
}

impl<'a> From<Token<'a>> for StringToken<'a> {
    fn from(t: Token<'a>) -> Self {
        Self {
            token: t,
            repr: None,
        }
    }
} 

impl<'a> Into<Token<'a>> for StringToken<'a> {
    fn into(self) -> Token<'a> {
        self.token
    }
}

impl<'a> Stringable<'a> for StringToken<'a> {
    fn matchable(&self) -> Token {
        self.token
    }

    fn ideal_repr(&self) -> String {
        match &self.repr {
            Some(s) => s.to_string(),
            None => self.matchable().ideal_repr(),
        }
    }
}

/// Convert a list of `Token`s into a string representation
/// * `tokens` - The tokens
/// * `colorize` - A function that colors tokens
// TODO: Allowing unnested or patterns because while they are stabilized, they're not release yet
#[allow(clippy::too_many_lines, clippy::unnested_or_patterns)]
pub fn _stringify<'a, F, T: Display, S: Stringable<'a> + From<Token<'a>>>(tokens: &[S], colorize: F) -> String
where
    F: Fn(&str, &S) -> T,
{
    let mut out = String::new();
    let mut implicit_paren: usize = 0;
    for (idx, token) in tokens.iter().enumerate() {
        let colored: T = colorize(&token.ideal_repr(), token);
        let append = match token.matchable() {
            Token::Number { .. } | Token::Constant { .. } | Token::Variable { .. } => {
                let is_r_paren = matches!(
                    tokens.get(idx + 1).map(|t| t.matchable()),
                    Some(Token::Paren {
                        kind: ParenType::Right
                    })
                );

                let is_op = matches!(tokens.get(idx + 1).map(|t| t.matchable()), Some(Token::Operator { .. }));

                let no_space = matches!(
                    tokens.get(idx + 1).map(|t| t.matchable()),
                    Some(Token::Operator {
                        kind: OperatorType::Pow
                    }) | Some(Token::Operator {
                        kind: OperatorType::Factorial
                    }) | Some(Token::Paren {
                        kind: ParenType::Right
                    })
                );

                // We delay the r_parens when the next operator is pow
                // Because exponents have a higher precedence in BEDMAS
                // So, `sin 5^2` should become `sin(5^2)` NOT `sin(5)^2`
                let delay_implicit_paren = matches!(
                    tokens.get(idx + 1).map(|t| t.matchable()),
                    Some(Token::Operator {
                        kind: OperatorType::Pow
                    })
                );

                let last = idx == tokens.len() - 1;

                let appendix = if implicit_paren > 0 && !delay_implicit_paren {
                    let space = if last || no_space { "" } else { " " };
                    let r_paren: T = colorize(
                        &")".repeat(implicit_paren),
                        &Token::Paren {
                            kind: ParenType::Right,
                        }.into(),
                    );
                    implicit_paren = 0;
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
                            tokens.get(idx + 1).map(|t| t.matchable()),
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
                                }.into(),
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
                        tokens.get(idx + 1).map(S::matchable),
                        Some(Token::Operator {
                            kind: OperatorType::Pow
                        }) | Some(Token::Paren {
                            kind: ParenType::Right,
                        })
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
