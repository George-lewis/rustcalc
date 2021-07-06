use std::cell::Cell;
use std::intrinsics::transmute;
use std::rc::Rc;

use colored::{ColoredString, Colorize};
use itertools::Itertools;
use rustmatheval::DoEvalResult;

use super::error::{Error};
use super::lib::doeval;
use super::lib::model::{
    errors::ErrorContext, functions::Function, variables::Variable, EvaluationContext,
};
use super::stringify::stringify;
use super::utils::insert_or_swap_sort;

pub fn format_var_name(name: &str) -> ColoredString {
    format!("${}", name.green().bold()).normal()
}

fn format_var(var: &Variable) -> String {
    format!(
        "[ {} => {} ]",
        format_var_name(&var.repr),
        format!("{:.3}", var.value.get()).blue()
    )
}

#[allow(clippy::module_name_repetitions)]
/// Formats a printable string listing all the `Variables` in the given slice `vars`
pub fn format_vars(vars: &[Rc<Variable>]) -> String {
    vars.iter().map(|v| format_var(&v)).join("\n")
}

/// Takes the given user `input` and splits it up into a name and value to be assigned or reassigned to a [Variable] in `vars`
pub fn assign_var_command<'a>(
    input: &'a str,
    vars: &'a mut Vec<Rc<Variable>>,
    funcs: &'a [Function],
) -> Result<String, Error<'a>> {
    // Variable assignment / reassignment

    let sides: Vec<&str> = input.split('=').collect();
    let trimmed_left = sides[0].trim(); // Trim here to remove space between end of variable name and = sign

    if sides.len() != 2 {
        // Multiple = signs or Assigning without using a $ prefix
        return Err(Error::Assignment);
    }

    // Trim again to remove whitespace between end of variable name and = sign
    let user_repr: String = trimmed_left[1..].to_string();

    let _vars = vars.iter().map(Rc::clone).collect_vec();

    let context = EvaluationContext {
        vars: &_vars,
        funcs,
        depth: 0,
        context: ErrorContext::Main,
    };

    // Get value for variable
    let result = doeval(sides[1], context);
    // match result {
    //     DoEvalResult::RecursionLimit { context } => todo!(),
    //     DoEvalResult::ParsingError { context, partial_tokens } => todo!(),
    //     DoEvalResult::RpnError { context, error } => todo!(),
    //     DoEvalResult::EvalError { context, error } => todo!(),
    //     DoEvalResult::Ok { string_tokens, result } => todo!(),
    // }
    // if let Err(ContextualLibError {
    //     error: LibError::Parsing,
    //     ..
    // }) = result
    // {
    //     return Err(Error::Library(
    //         LibError::Parsing.with_context(ErrorContext::Main),
    //     ));
    //     // Length of untrimmed lefthand side
    //     // Offset is added so that highlighting can be added to expressions that come after an '=' during assignment
    // }

    if let DoEvalResult::Ok {
        string_tokens,
        result
    } = result {

        let conf_string = format!(
            "[ ${} {} {} ] => {}",
            user_repr.green().bold(),
            "=".cyan(),
            stringify(&string_tokens),
            format!("{:.3}", result).blue()
        );

        drop(string_tokens);

        let var = Variable {
            repr: user_repr,
            value: Cell::new(result),
        };

        assign_var(var, vars);

        Ok(conf_string)
    } else {
        unsafe {
            Err(Error::Library( transmute(result)) )
        }
    }
    
}

pub fn assign_var(var: Variable, vars: &mut Vec<Rc<Variable>>) {
    let repr = var.repr.clone();
    let cmp = |v: &Rc<Variable>| repr.cmp(&v.repr);

    insert_or_swap_sort(vars, Rc::new(var), cmp);
}
