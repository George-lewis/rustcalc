#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::wildcard_imports)]

use std::{fmt::Display, process};

mod lib;

use colored::*;
use lib::doeval;
use lib::errors::Error;
use lib::operators::*;
use lib::tokens::*;
use lib::variables::Variable;

use lib::utils;
use rustyline::Editor;

use itertools::{self, Itertools};

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

    let mut vars = vec![
        Variable {
            repr: String::from("ans"),
            value: 0.0
        }
    ];

    loop {
        #[allow(clippy::single_match_else)]
        let input = match editor.readline("> ") {
            Ok(line) => line.trim_end().to_string(),
            Err(_) => {
                if let Some(path) = cache_file {
                    editor.save_history(path).ok();
                }
                process::exit(0)
            }
        };

        if input.is_empty() {
            continue;
        }

        // Add the line to the history
        editor.add_history_entry(&input);


        match handle_input(&input, &mut vars) {
            Ok(formatted) => println!("{}", formatted),
            Err(error) => handle_errors(error, input),
        }
        
    }
}

fn list_vars_command(vars: &[Variable]) -> String {
    vars.iter()
        .map(|var| {
          format!(
            "[ ${} => {} ]",
            var.repr.green().bold(),
            format!("{:.3}", var.value).blue()
          )
        }).join("\n")
  }

  fn assign_var(vars: &mut Vec<Variable>, user_value: f64, user_repr: String) {
    // Search to see if given representation exists
    let found_var = vars.iter_mut().find(|x| x.repr == user_repr);
    if let Some(found_var) = found_var {
        // Reassign
        found_var.value = user_value;
    } else {
        // Assign
        let user_var = Variable {
            repr: user_repr,
            value: user_value,
        };
        vars.push(user_var);
    }
  }

fn handle_input (input: &str, vars: &mut Vec<Variable>) -> Result<String, Error> {
    if input == "$" {
        // Variable list command
        Ok(list_vars_command(&vars))
    } else if input.contains("=") {
            // Variable assignment/reassignment
            let sides: Vec<&str> = input.split('=').collect();
            if sides.len() != 2 {
                // Multiple = signs
                return Err(Error::Assignment);
            } else if !sides[0].trim().starts_with("$") {
                // Assigning without using a $ prefix
                return Err(Error::Assignment);
            }

            let user_repr: String = sides[0].trim()[1..].to_string();

            // Get value for variable
            let result = doeval(sides[1], vars);
            if let Err(Error::Parsing(idx)) = result {
                return Err(Error::Parsing(idx + sides[0].len() + 1));
                // Offset is added so that highlighting can be added to expressions that come after an '=' during assignment
            } 
            let (user_value, repr) = result?;

            // Format assignment confirmation
            let formatted = stringify(&repr, color_cli);
            let assign_confirmation = format!(
                "[ {}{} {} {} => {} ]",
                "$".green(),
                user_repr.green(),
                "=".cyan(),
                formatted,
                format!("{:.3}", user_value).blue()
            );

            assign_var(vars, user_value, user_repr);
            
            Ok(assign_confirmation)
        }
    else {
        // Evaluate as normal
        let result = doeval(&input, &vars);
        if let Err(Error::Parsing(idx)) = result {
            return Err(Error::Parsing(idx));
        }
        let (x, repr) = result?;

        let formatted = stringify(&repr, color_cli);
        let eval_string = format!("[ {} ] => {}", formatted, format!("{:.3}", x).blue());

        vars[0].value = x; // Set ans to new value

        Ok(eval_string)
    }
}

fn handle_errors(e: Error, input: String) {
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
                input
                    .chars()
                    .nth(idx)
                    .unwrap()
                    .to_string()
                    .on_red()
                    .white(),
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
        Error::Assignment => {
            println!("Couldn't assign to variable. Malformed assignment statement.")
        }
        Error::UnknownVariable(idx) => {
            let first = if idx > 0 {
                utils::slice(&input, 0, (idx) as i64)
            } else {
                "".to_string()
            };
            println!(
                "Unknown variable at index [{}]\n{}{}{}\n{}{}",
                idx.to_string().red(),
                first,
                input
                    .chars()
                    .nth(idx)
                    .unwrap()
                    .to_string()
                    .on_red()
                    .white(),
                utils::slice(&input, idx + 1, -0),
                "~".repeat(idx).red().bold(),
                "^".red()
            );
        }
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
        Token::Variable { .. } => string.green(),
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
            Token::Number { .. } | Token::Constant { .. } | Token::Variable { .. } => {
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
        lib::constants::*,
        lib::doeval,
        lib::errors::Error,
        lib::operators::*,
        lib::{tokens::*, variables::Variable},
        stringify,
    };

    fn same(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.000_001
    }

    #[test]
    fn test() {
        let mut test_vars: Vec<Variable> = Vec::new();
        let test_var_repr_a = "v".to_string();
        let test_var_value_a = 5.0;
        let test_var_a = Variable {
            repr: test_var_repr_a,
            value: test_var_value_a,
        };
        test_vars.push(test_var_a);

        let test_var_repr_b = "pi".to_string();
        let test_var_value_b = 7.0;
        let test_var_b = Variable {
            repr: test_var_repr_b,
            value: test_var_value_b,
        };
        test_vars.push(test_var_b);

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
            (
                "$v",
                "$v",
                5.0,
                vec![Token::Variable {
                    inner: &test_vars[0],
                }],
            ),
            (
                "$v + 5",
                "$v + 5",
                10.0,
                vec![
                    Token::Variable {
                        inner: &test_vars[0],
                    },
                    Token::Operator {
                        kind: OperatorType::Add,
                    },
                    Token::Number { value: 5.0 },
                ],
            ),
            (
                "  5 +    $v    ",
                "5 + $v",
                10.0,
                vec![
                    Token::Number { value: 5.0 },
                    Token::Operator {
                        kind: OperatorType::Add,
                    },
                    Token::Variable {
                        inner: &test_vars[0],
                    },
                ],
            ),
            (
                "pi + $pi",
                "π + $pi",
                std::f64::consts::PI + 7.0,
                vec![
                    Token::Constant {
                        kind: ConstantType::PI,
                    },
                    Token::Operator {
                        kind: OperatorType::Add,
                    },
                    Token::Variable {
                        inner: &test_vars[1],
                    },
                ],
            ),
        ]
        .iter()
        .for_each(|(a, b, c, d)| {
            let (result, tokens) = match doeval(a, &test_vars) {
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
            ("3 + $a", Error::UnknownVariable(4)),
        ]
        .iter()
        .for_each(|(a, b)| assert_eq!(doeval(a, &Vec::new()).unwrap_err(), *b));
    }
}
