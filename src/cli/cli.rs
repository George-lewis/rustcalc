use crate::funcs::{assign_func_command, format_funcs};

use super::lib::model::{
    errors::ErrorContext, functions::Function, variables::Variable, EvaluationContext,
};
use super::lib::utils;

use colored::Colorize;
use utils::Pos;

use super::error::{ContextualLibError, Error, LibError};

use super::vars::{assign_var, assign_var_command, format_vars};

use super::lib::doeval;

use super::funcs::format_func_name;

use super::stringify::stringify;

/// Interprets a given user `input` and executes the given command or evaluates the given expression.
/// * `input` - The user submitted string to be interpreted
/// * `vars` - The vector of `Variables` the user has already entered / will add to
pub fn handle_input<'a>(
    input: &str,
    vars: &'a mut Vec<Variable>,
    funcs: &'a mut Vec<Function>,
) -> Result<String, Error> {
    if input.len() == 1 {
        if Variable::is(input) {
            // Variable list command
            return if vars.is_empty() {
                Ok("No vars".to_string())
            } else {
                Ok(format_vars(vars))
            };
        } else if Function::is(input) {
            return if funcs.is_empty() {
                Ok("No funcs".to_string())
            } else {
                Ok(format_funcs(funcs))
            };
        }
    }

    if input.contains('=') {
        if Variable::is(input) {
            // Assign / Reassign variable command
            assign_var_command(input, vars, funcs)
        } else if Function::is(input) {
            assign_func_command(input, funcs)
        } else {
            Err(Error::Assignment)
        }
    } else {
        // Evaluate as normal
        let context = EvaluationContext {
            vars,
            funcs,
            depth: 0,
            context: ErrorContext::Main,
        };
        let result = doeval(input, context);
        // if let Err(ContextLibError{ error: LibError::Parsing(idx), ..}) = result {
        //     return Err(LibError::Parsing(idx).into());
        // }
        let (x, repr) = result?;

        let formatted = stringify(&repr);
        let eval_string = format!("[ {} ] => {}", formatted, format!("{:.3}", x).blue());

        let ans = Variable {
            repr: "ans".to_string(),
            value: x,
        };

        assign_var(ans, vars); // Set ans to new value

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
        "{} at index [{}]\n{}{}{}\n{}{}",
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
        Error::Library(ContextualLibError { context, error }) => match error {
            LibError::Parsing(idx) => {
                make_highlighted_error("Couldn't parse the token", input, idx)
            }
            LibError::Operand(_op) => {
                // format!(
                //     "Couldn't evaluate. Operator [{}] requires an operand.",
                //     format!("{:?}", kind).green()
                // )
                "".to_string()
            }
            LibError::EmptyStack => "Couldn't evalutate. Stack was empty?".to_string(),
            LibError::MismatchingParens => "Couldn't evaluate. Mismatched parens.".to_string(),
            LibError::UnknownVariable(idx) => match context {
                ErrorContext::Main => make_highlighted_error("Unknown variable", input, idx),
                ErrorContext::Scoped(func) => {
                    let name = format_func_name(&func.name);
                    let msg = format!("Unknown variable in function {}", name);
                    make_highlighted_error(&msg, &func.code, idx)
                }
            },
            LibError::UnknownFunction(idx) => match context {
                ErrorContext::Main => make_highlighted_error("Unknown function", input, idx),
                ErrorContext::Scoped(func) => {
                    let name = format_func_name(&func.name);
                    let msg = format!("Unknown function within {}", name);
                    make_highlighted_error(&msg, &func.code, idx)
                }
            },
            LibError::RecursionLimit => "Exceeded recursion limit.".to_string(),
        },
        Error::Io(..) => unreachable!(),
    }
}
