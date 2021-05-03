use crate::lib::model::variables::Variable;
use crate::lib::utils;

use colored::Colorize;
use utils::Pos;

use super::error::{Error, LibError};

use super::vars::{assign_var, assign_var_command, format_vars};

use crate::lib::doeval;

use super::stringify::stringify;

/// Interprets a given user `input` and executes the given command or evaluates the given expression.
/// * `input` - The user submitted string to be interpreted
/// * `vars` - The vector of `Variables` the user has already entered / will add to
pub fn handle_input(input: &str, vars: &mut Vec<Variable>) -> Result<String, Error> {
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
        if let Err(LibError::Parsing(idx)) = result {
            return Err(LibError::Parsing(idx).into());
        }
        let (x, repr) = result?;

        let formatted = stringify(&repr);
        let eval_string = format!("[ {} ] => {}", formatted, format!("{:.3}", x).blue());

        assign_var(vars, "ans", x); // Set ans to new value

        Ok(eval_string)
    }
}

/// Makes a highlighted error message pointing to a particular index
/// * `msg` - The message to print
/// * `input_str` - The erroneous input
/// * `idx` - The location of the error
///
/// Returns the formatted error string
///
/// ## Panics
/// Panics if `idx > input_str.chars().count()`
fn make_highlighted_error(msg: &str, input_str: &str, idx: usize) -> String {
    let first = if idx > 0 {
        utils::slice(input_str, 0, &Pos::Idx(idx))
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
        utils::slice(input_str, idx + 1, &Pos::End),
        "~".repeat(idx).red().bold(),
        "^".red()
    )
}

/// Produces an error message to show to the user
/// * `error` - The `Error`
/// * `input` - The user's input
///
/// Returns a formatted error string
///
/// ## Panics
/// Does not handle `Error::Io`
pub fn handle_errors(error: Error, input: &str) -> String {
    match error {
        Error::Assignment => {
            "Couldn't assign to variable. Malformed assignment statement.".to_string()
        }
        Error::Library(inner) => match inner {
            LibError::Parsing(idx) => {
                make_highlighted_error("Couldn't parse the token at index", input, idx)
            }
            LibError::Operand(kind) => {
                format!(
                    "Couldn't evaluate. Operator [{}] requires an operand.",
                    format!("{:?}", kind).green()
                )
            }
            LibError::EmptyStack => "Couldn't evalutate. Stack was empty?".to_string(),
            LibError::MismatchingParens => "Couldn't evaluate. Mismatched parens.".to_string(),
            LibError::UnknownVariable(idx) => {
                make_highlighted_error("Unknown variable at index", input, idx)
            }
        },
        Error::Io(..) => unreachable!(),
    }
}
