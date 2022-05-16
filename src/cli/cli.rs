use std::rc::Rc;

use crate::funcs::{assign_func_command, format_func_name, format_func_with_args, format_funcs};
use crate::stringify::{stringify_off, stringify_opts, StringTokenOpts};
use crate::utils::Format;
use crate::Cow;

use super::lib::model::{
    errors::ErrorContext, functions::Function, variables::Variable, EvaluationContext,
};

use colored::Colorize;
use rustmatheval::model::errors::{EvalError, RpnError};
use rustmatheval::model::functions::Functions;
use rustmatheval::model::tokens::{PartialToken, StringToken, Tokens};
use rustmatheval::DoEvalResult;

use super::error::Error;
use super::lib::doeval;
use super::stringify::stringify;
use super::vars::{assign_var, assign_var_command, format_vars};

/// Interprets a given user `input` and executes the given command or evaluates the given expression.
/// * `input` - The user submitted string to be interpreted
/// * `vars` - The vector of `Variables` the user has already entered / will add to
pub fn handle_input<'a>(
    input: &'a str,
    vars: &'a mut Vec<Rc<Variable>>,
    funcs: &'a mut Vec<Function>,
) -> Result<String, Error<'a>> {
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
                Ok(format_funcs(funcs, vars))
            };
        }
    }

    if input.contains('=') {
        if Variable::is(input) {
            // Assign / Reassign variable command
            assign_var_command(input, vars, funcs)
        } else if Function::is(input) {
            assign_func_command(input, funcs, vars)
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

        if let DoEvalResult::Ok {
            tokens,
            result: result_,
        } = &result
        {
            let formatted = stringify_opts(tokens, StringTokenOpts {
                ideal_spacing: true,
            });
            let eval_string = format!("[ {} ] => {}", formatted, format!("{:.3}", result_).blue());

            // Set ans to new value
            assign_var("ans", *result_, vars);

            Ok(eval_string)
        } else {
            Err(result.into())
        }
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
fn make_highlighted_error(msg: &str, tokens: &[Tokens], tok: &StringToken) -> String {
    let (styled, off) = stringify_off(tokens);
    let off = off
        .iter()
        .find(|off| off.old_idx == tok.idx)
        .expect("New index for string token could not be found.");
    format!(
        "{msg}\n{styled}\n{}{}{}",
        "-".repeat(off.new_idx).blue(),
        "^".red(),
        "~".repeat(tok.stride() - 1).red()
    )
}

fn highlight_parsing_error(input_len: usize, tokens: &[PartialToken]) -> String {
    let errors = tokens.iter().filter_map(|tok| match tok.inner {
        Ok(_) => None,
        Err(_) => Some((tok.idx, tok.repr.len())),
    });

    let mut line = String::new();
    let mut last = 0;

    for (idx, stride) in errors {
        line.push_str(&"-".repeat(idx - last).blue().format());
        line.push_str(&"-".repeat(stride).red().format());
        last = idx + stride;
    }

    line.push_str(&"-".repeat(input_len - last).blue().format());

    let styled = stringify(tokens);
    format!("Failed to parse some tokens.\n{styled}\n{line}")
}

fn handle_eval_operand_error(tokens: &[Tokens], tok: &StringToken, op: &Functions) -> String {
    let arity: Cow<str> = if op.arity() == 1 {
        "an argument".into()
    } else {
        format!("[{}] arguments", format!("{}", op.arity()).red()).into()
    };

    let msg = match op {
        Functions::Builtin(op) => {
            format!(
                "Built in operator [{}] requires {arity}.",
                format!("{:?}", op.kind).green()
            )
        }
        Functions::User(func) => {
            format!(
                "User function [{}] requires {arity}.",
                format_func_with_args(func),
            )
        }
    };

    make_highlighted_error(&msg, tokens, tok)
}

/// Produce an error message for a given [`super::lib::ContextualError`] and input string
/// * `error` - The error
/// * `input` - The user's input
pub fn handle_library_errors(result: &DoEvalResult, input: &str) -> Cow<'static, str> {
    let (ctxt, msg): (_, Cow<'static, str>) = match result {
        DoEvalResult::RecursionLimit { context } => (context, "Exceeded recursion limit.".into()),
        DoEvalResult::ParsingError {
            context,
            partial_tokens,
        } => (
            context,
            highlight_parsing_error(input.len(), partial_tokens).into(),
        ),
        DoEvalResult::RpnError { context, error } => match error {
            RpnError::MismatchingParens => {
                (context, "Couldn't evaluate. Mismatched parantheses.".into())
            }
        },
        DoEvalResult::EvalError {
            context,
            tokens,
            error,
        } => {
            let msg = match error {
                EvalError::EmptyStack => "Couldn't evaluate. Stack was empty?".into(),
                EvalError::Operand { op, tok } => handle_eval_operand_error(tokens, tok, op).into(),
            };
            (context, msg)
        }
        DoEvalResult::Ok { .. } => unreachable!(),
    };

    if let ErrorContext::Scoped(func) = ctxt {
        format!("In function {}: {}", format_func_name(&func.name), msg).into()
    } else {
        msg
    }
}

/// Produces an error message to show to the user
/// * `error` - The `Error`
/// * `input` - The user's input
///
/// Returns a formatted error string
///
/// ## Panics
/// Does not handle `Error::Io`
pub fn handle_errors(error: &Error, input: &str) -> Cow<'static, str> {
    match error {
        Error::Assignment => "Couldn't assign. Malformed assignment statement.".into(),
        Error::Library(eval_result) => handle_library_errors(eval_result, input),
    }
}
