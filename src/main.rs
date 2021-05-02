#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::wildcard_imports)]

use std::{
    fmt::Display,
    fs,
    io::{self, ErrorKind::NotFound},
    path::PathBuf,
    process,
};

mod lib;

use colored::*;
use lazy_static::lazy_static;
use lib::model::errors::Error;
use lib::{
    doeval,
    model::{
        operators::{Associativity, Operator, OperatorType},
        tokens::{ParenType, Token},
    },
};

use lib::model::variables::Variable;

use lib::utils;
use rustyline::Editor;

use itertools::{self, Itertools};

lazy_static! {
    static ref HISTORY_FILE: Option<PathBuf> = dirs::cache_dir().map(|mut dir| {
        dir.push("rustcalc-history.txt");
        dir
    });
    static ref RCFILE: Option<PathBuf> = dirs::config_dir().map(|mut dir| {
        dir.push("rustcalc.rc");
        dir
    });
}

const DEFAULT_RCFILE: &str = include_str!("../res/rustcalc.rc");

/// Error type for errors stemming from cli code, which includes `Errors` thrown by the library
enum CliError {
    Assignment,
    Io(io::Error),
    Library(Error),
}

impl From<Error> for CliError {
    fn from(error: Error) -> Self {
        Self::Library(error)
    }
}

impl From<io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

/// Load Rustcalc's rcfile from the default location, `RCFILE`.
/// This function has side effects:
/// * Files may be created
/// * May write to stdout
///
/// ## Input
/// * `vars` - A mutable reference to the applications variables. Executing the rcfile may create variables.
///
/// ## Output
/// Returns an empty `Result` on success, or a `CliError` from io operations
fn load_rcfile(vars: &mut Vec<Variable>) -> Result<(), CliError> {
    let path = match RCFILE.as_deref() {
        Some(path) => path,
        None => {
            return Err(io::Error::new(NotFound, "Couldn't get path for config directory").into())
        }
    };

    // If RCFile doesn't exist, create it and write the default contents
    if !path.exists() {
        println!(
            "RCFile doesn't exist. Creating default at [{}]",
            path.to_string_lossy()
        );
        fs::write(path, DEFAULT_RCFILE)?;
    }

    // Read
    let lines = fs::read_to_string(path)?;

    // Filter out empty and comment lines
    let lines = lines
        .lines()
        .enumerate()
        .filter(|(_, l)| !(l.trim().is_empty() || l.starts_with("//")));

    // Feed each line through `handle_input` and make use of `handle_errors`
    // Succesfully executing statements are silent
    for (n, line) in lines {
        if let Err(inner) = handle_input(line, vars) {
            let message = handle_errors(inner, line);
            println!(
                "Error in RCFile on line [{}]: {}",
                format!("{}", n).red(),
                message
            );
        }
    }
    Ok(())
}

fn main() -> ! {
    let mut editor = Editor::<()>::new();

    if let Some(path) = HISTORY_FILE.as_deref() {
        editor.load_history(path).ok();
    }

    let mut vars = vec![];

    if let Err(inner) = load_rcfile(&mut vars) {
        match inner {
            CliError::Io(inner) => {
                println!("Error loading RCFile: {:#?}", inner)
            }
            _ => unreachable!(),
        }
    };

    loop {
        #[allow(clippy::single_match_else)]
        let input = match editor.readline("> ") {
            Ok(line) => line.trim_end().to_string(),
            Err(_) => {
                if let Some(path) = HISTORY_FILE.as_deref() {
                    editor.save_history(&path).ok();
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
            Err(error) => {
                let msg = handle_errors(error, &input);
                println!("{}", msg);
            }
        }
    }
}

/// Formats a printable string listing all the `Variables` in the given slice `vars`
fn format_vars(vars: &[Variable]) -> String {
    vars.iter()
        .map(|var| {
            format!(
                "[ ${} => {} ]",
                var.name.green().bold(),
                format!("{:.3}", var.value).blue()
            )
        })
        .join("\n")
}

/// Takes the given user `input` and splits it up into a name and value to be assigned or reassigned to a [Variable] in `vars`
fn assign_var_command(input: &str, vars: &mut Vec<Variable>) -> Result<String, CliError> {
    // Variable assignment/reassignment

    let sides: Vec<&str> = input.split('=').collect();
    let trimmed_left = sides[0].trim(); // Trim here to remove space between end of variable name and = sign

    if sides.len() != 2 || !trimmed_left.starts_with('$') {
        // Multiple = signs || Assigning without using a $ prefix
        return Err(CliError::Assignment);
    }

    let user_repr: String = trimmed_left[1..].to_string(); // Trim again to remove whitespace between end of variable name and = sign

    // Get value for variable
    let result = doeval(sides[1], vars);
    if let Err(Error::Parsing(idx)) = result {
        return Err(CliError::Library(Error::Parsing(idx + sides[0].len() + 1)));
        // Length of untrimmed lefthand side
        // Offset is added so that highlighting can be added to expressions that come after an '=' during assignment
    }
    let (user_value, repr) = result?;

    // Get printable confirmation string
    let conf_string = format!(
        "[ ${} {} {} ] => {}",
        user_repr.green().bold(),
        "=".cyan(),
        stringify(&repr, color_cli),
        format!("{:.3}", user_value).blue()
    );

    assign_var(vars, &user_repr, user_value);

    Ok(conf_string)
}

/// Searches `vars` for the given `user_repr` to find if a [Variable] exists, and either reassigns it to, or creates it with, the given `user_value`
fn assign_var(vars: &mut Vec<Variable>, repr: &str, value: f64) {
    let cmp = |var: &Variable| repr.cmp(&var.name);
    let search = vars.binary_search_by(cmp);
    match search {
        Ok(idx) => {
            vars[idx].value = value;
        }
        Err(idx) => {
            let var = Variable {
                name: repr.to_string(),
                value,
            };
            vars.insert(idx, var);
        }
    }
}

/// Interprets a given user `input` and executes the given command or evaluates the given expression.
/// * `input` - The user submitted string to be interpreted
/// * `vars` - The vector of `Variables` the user has already entered / will add to
fn handle_input(input: &str, vars: &mut Vec<Variable>) -> Result<String, CliError> {
    if input == "$" {
        // Variable list command
        if vars.is_empty() {
            Ok("No vars".to_string())
        } else {
            Ok(format_vars(vars))
        }
    } else if input.contains('=') {
        // Assign / Reassign variable command
        assign_var_command(input, vars)
    } else {
        // Evaluate as normal
        let result = doeval(input, vars);
        if let Err(Error::Parsing(idx)) = result {
            return Err(Error::Parsing(idx).into());
        }
        let (x, repr) = result?;

        let formatted = stringify(&repr, color_cli);
        let eval_string = format!("[ {} ] => {}", formatted, format!("{:.3}", x).blue());

        assign_var(vars, "ans", x); // Set ans to new value

        Ok(eval_string)
    }
}

/// Makes a highlighted error message for use with `Error::Parsing` and `Error::UnknownVariable`
fn make_highlighted_error(msg: &str, input_str: &str, idx: usize) -> String {
    let first = if idx > 0 {
        utils::slice(input_str, 0, (idx) as i64)
    } else {
        "".to_string()
    };
    format!(
        "{} [{}]\n{}{}{}\n{}{}",
        msg,
        idx.to_string().red(),
        first,
        input_str
            .chars()
            .nth(idx)
            .unwrap()
            .to_string()
            .on_red()
            .white(),
        utils::slice(input_str, idx + 1, -0),
        "~".repeat(idx).red().bold(),
        "^".red()
    )
}

/// Prints error messages for the given `CliError`, referencing the `input` that caused them for clarity
fn handle_errors(error: CliError, input: &str) -> String {
    match error {
        CliError::Assignment => {
            "Couldn't assign to variable. Malformed assignment statement.".to_string()
        }
        CliError::Library(inner) => match inner {
            Error::Parsing(idx) => {
                make_highlighted_error("Couldn't parse the token at index", input, idx)
            }
            Error::Operand(kind) => {
                format!(
                    "Couldn't evaluate. Operator [{}] requires an operand.",
                    format!("{:?}", kind).green()
                )
            }
            Error::EmptyStack => "Couldn't evalutate. Stack was empty?".to_string(),
            Error::MismatchingParens => "Couldn't evaluate. Mismatched parens.".to_string(),
            Error::UnknownVariable(idx) => {
                make_highlighted_error("Unknown variable at index", input, idx)
            }
        },
        CliError::Io(..) => unreachable!(),
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

#[allow(clippy::too_many_lines)]
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

                let no_space = matches!(
                    tokens.get(idx + 1),
                    Some(Token::Operator {
                        kind: OperatorType::Pow
                    }) | Some(Token::Operator {
                        kind: OperatorType::Factorial
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
        lib::doeval,
        lib::model::constants::*,
        lib::model::errors::Error,
        lib::model::operators::*,
        lib::model::{tokens::*, variables::Variable},
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
                "sin (66) pow 2 plus cos(66)^2",
                "sin(66)^2 + cos(66)^2",
                1.0,
                vec![
                    Token::Operator {
                        kind: OperatorType::Sin,
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
            let (result, tokens) = match doeval(a, &[]) {
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
        .for_each(|(a, b)| assert_eq!(doeval(a, &[]).unwrap_err(), *b));
    }

    #[test]
    fn test_vars() {
        let test_vars = vec![
            Variable {
                name: String::from('v'),
                value: 5.0,
            },
            Variable {
                name: String::from("pi"),
                value: 7.0,
            },
        ];

        [
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
    fn fail_vars() {
        vec![("3 + $a", Error::UnknownVariable(4))]
            .iter()
            .for_each(|(a, b)| assert_eq!(doeval(a, &[]).unwrap_err(), *b));
    }
}
