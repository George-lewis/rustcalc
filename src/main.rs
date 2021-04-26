#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::wildcard_imports)]

use std::{
    fmt::Display,
    io::Write,
    process::exit,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    thread,
    time::Duration,
};

mod lib;

use colored::*;
use lib::doeval;
use lib::errors::Error;
use lib::operators::*;
use lib::tokens::*;

use lib::utils;
use rustyline::Editor;

const HISTORY_FILE: &str = "rustcalc-history.txt";

fn main() -> ! {
    let mut editor = Editor::<()>::new();

    let cache_file = dirs::cache_dir().map(|mut dir| {
        dir.push(HISTORY_FILE);
        dir
    });
    let cache_file = cache_file.as_deref();

    if let Some(path) = cache_file {
        editor.load_history(path).ok();
    }

    loop {
        #[allow(clippy::single_match_else)]
        let input = match editor.readline("> ") {
            Ok(line) => line.trim_end().to_string(),
            Err(_) => {
                if let Some(path) = cache_file {
                    editor.save_history(path).ok();
                }
                exit(0)
            }
        };

        if input.is_empty() {
            continue;
        }

        let (x, repr) = match doeval(&input) {
            Ok((a, b)) => (a, b),
            Err(e) => {
                match e {
                    Error::Parsing(idx) => {
                        let first = if idx > 0 {
                            utils::slice(&input, 0, (idx) as i64)
                        } else {
                            "".to_string()
                        };
                        println!(
                            "Couldn't parse the token at index [{}]\n{}{}{}\n{}{}",
                            idx.to_string().red(),
                            first,
                            input.chars().nth(idx).unwrap().to_string().on_red().white(),
                            utils::slice(&input, idx + 1, -0),
                            "~".repeat(idx).red().bold(),
                            "^".red()
                        );
                    }
                    Error::Operand(kind) => {
                        println!(
                            "Couldn't evaluate. Operator [{}] requires an operand.",
                            format!("{:?}", kind).green()
                        );
                    }
                    Error::EmptyStack => {
                        println!("Couldn't evalutate. Stack was empty?");
                    }
                    Error::MismatchingParens => {
                        println!("Couldn't evaluate. Mismatched parens.");
                    }
                }

                continue;
            }
        };

        // Add the line to the history
        editor.add_history_entry(input);

        let formatted = stringify(&repr, color_cli);

        println!("[ {} ] => {}", formatted, format!("{:.3}", x).blue());
    }
}

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
    }
}

fn stringify<F, T: Display>(tokens: &[Token], colorize: F) -> String
where
    F: Fn(&str, &Token) -> T,
{
    let mut out = String::new();
    let mut implicit_paren: usize = 0;
    for (idx, token) in tokens.iter().enumerate() {
        let colored: T = colorize(&token.ideal_repr(), token);
        let append = match *token {
            Token::Number { .. } | Token::Constant { .. } => {
                let is_r_paren = matches!(
                    tokens.get(idx + 1),
                    Some(Token::Paren {
                        kind: ParenType::Right
                    })
                );

                let is_op = matches!(tokens.get(idx + 1), Some(Token::Operator { .. }));

                let is_pow = matches!(
                    tokens.get(idx + 1),
                    Some(Token::Operator {
                        kind: OperatorType::Pow
                    })
                );

                let last = idx == tokens.len() - 1;

                let appendix = if implicit_paren > 0 {
                    let space = if last || is_pow { "" } else { " " };
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
                } else if is_op && !is_pow {
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
                    Associativity::Left => format!("{} ", colored),
                    Associativity::Right => {
                        let is_l_paren = matches!(
                            tokens.get(idx + 1),
                            Some(Token::Paren {
                                kind: ParenType::Left
                            })
                        );

                        if op.implicit_paren() && !is_l_paren {
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

#[cfg(test)]
mod tests {

    #![allow(
        clippy::float_cmp,
        clippy::non_ascii_literal,
        clippy::clippy::too_many_lines
    )]

    use crate::{
        lib::constants::*, lib::doeval, lib::errors::Error, lib::operators::*, lib::tokens::*,
        stringify,
    };

    fn same(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.000_001
    }

    #[test]
    fn test() {
        [
            (
                "1 + 1",
                "1 + 1",
                2.0,
                vec![
                    Token::Number { value: 1.0 },
                    Token::Operator {
                        kind: OperatorType::Add,
                    },
                    Token::Number { value: 1.0 },
                ],
            ),
            (
                "sin pi",
                "sin(π)",
                std::f64::consts::PI.sin(),
                vec![
                    Token::Operator {
                        kind: OperatorType::Sin,
                    },
                    Token::Constant {
                        kind: ConstantType::PI,
                    },
                ],
            ),
            (
                "1 plus 7 sub 2 times 3",
                "1 + 7 - 2 × 3",
                2.0,
                vec![
                    Token::Number { value: 1.0 },
                    Token::Operator {
                        kind: OperatorType::Add,
                    },
                    Token::Number { value: 7.0 },
                    Token::Operator {
                        kind: OperatorType::Sub,
                    },
                    Token::Number { value: 2.0 },
                    Token::Operator {
                        kind: OperatorType::Mul,
                    },
                    Token::Number { value: 3.0 },
                ],
            ),
            (
                "sin(1 + 2 + 3)",
                "sin(1 + 2 + 3)",
                ((1.0 + 2.0 + 3.0) as f64).sin(),
                vec![
                    Token::Operator {
                        kind: OperatorType::Sin,
                    },
                    Token::Paren {
                        kind: ParenType::Left,
                    },
                    Token::Number { value: 1.0 },
                    Token::Operator {
                        kind: OperatorType::Add,
                    },
                    Token::Number { value: 2.0 },
                    Token::Operator {
                        kind: OperatorType::Add,
                    },
                    Token::Number { value: 3.0 },
                    Token::Paren {
                        kind: ParenType::Right,
                    },
                ],
            ),
            (
                "345.67",
                "345.67",
                345.67,
                vec![Token::Number { value: 345.67 }],
            ),
            (
                "sin 66 pow 2 plus cos(66)^2",
                "sin(66)^2 + cos(66)^2",
                1.0,
                vec![
                    Token::Operator {
                        kind: OperatorType::Sin,
                    },
                    Token::Number { value: 66.0 },
                    Token::Operator {
                        kind: OperatorType::Pow,
                    },
                    Token::Number { value: 2.0 },
                    Token::Operator {
                        kind: OperatorType::Add,
                    },
                    Token::Operator {
                        kind: OperatorType::Cos,
                    },
                    Token::Paren {
                        kind: ParenType::Left,
                    },
                    Token::Number { value: 66.0 },
                    Token::Paren {
                        kind: ParenType::Right,
                    },
                    Token::Operator {
                        kind: OperatorType::Pow,
                    },
                    Token::Number { value: 2.0 },
                ],
            ),
            (
                "(1)",
                "(1)",
                1.0,
                vec![
                    Token::Paren {
                        kind: ParenType::Left,
                    },
                    Token::Number { value: 1.0 },
                    Token::Paren {
                        kind: ParenType::Right,
                    },
                ],
            ),
            (
                "((1))",
                "((1))",
                1.0,
                vec![
                    Token::Paren {
                        kind: ParenType::Left,
                    },
                    Token::Paren {
                        kind: ParenType::Left,
                    },
                    Token::Number { value: 1.0 },
                    Token::Paren {
                        kind: ParenType::Right,
                    },
                    Token::Paren {
                        kind: ParenType::Right,
                    },
                ],
            ),
            (
                "-1",
                "-1",
                -1.0,
                vec![
                    Token::Operator {
                        kind: OperatorType::Negative,
                    },
                    Token::Number { value: 1.0 },
                ],
            ),
            (
                "1 + -1",
                "1 + -1",
                0.0,
                vec![
                    Token::Number { value: 1.0 },
                    Token::Operator {
                        kind: OperatorType::Add,
                    },
                    Token::Operator {
                        kind: OperatorType::Negative,
                    },
                    Token::Number { value: 1.0 },
                ],
            ),
            (
                "-   (  1.1 +  2.2)",
                "-(1.1 + 2.2)",
                -3.3,
                vec![
                    Token::Operator {
                        kind: OperatorType::Negative,
                    },
                    Token::Paren {
                        kind: ParenType::Left,
                    },
                    Token::Number { value: 1.1 },
                    Token::Operator {
                        kind: OperatorType::Add,
                    },
                    Token::Number { value: 2.2 },
                    Token::Paren {
                        kind: ParenType::Right,
                    },
                ],
            ),
        ]
        .iter()
        .for_each(|(a, b, c, d)| {
            let (result, tokens) = match doeval(a) {
                Ok((x, y)) => (x, y),
                Err(e) => panic!("error! {:?}; {}", e, a),
            };
            assert_eq!(tokens, *d, "Checking tokenization of [{}]", a);
            assert!(same(result, *c), "Checking evaluation of [{}]", a);
            assert_eq!(stringify(&tokens, |a, _| a.to_string()), *b);
        });
    }

    #[test]
    fn fail() {
        vec![
            ("1 +", Error::Operand(OperatorType::Add)),
            ("1 + 2 + 3 + h", Error::Parsing(12)),
            ("h", Error::Parsing(0)),
            ("(1", Error::MismatchingParens),
        ]
        .iter()
        .for_each(|(a, b)| assert_eq!(doeval(a).unwrap_err(), *b));
    }
}
