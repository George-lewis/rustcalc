use colored::{ColoredString, Colorize};
use itertools::Itertools;

use super::error::{Error, LibError, ContextLibError};
use super::lib::doeval;
use super::lib::model::{functions::Function, variables::Variable, EvaluationContext, errors::ErrorContext};
use super::stringify::stringify;
use super::utils::insert_or_swap_sort;

pub fn format_var_name(name: &str) -> ColoredString {
    format!("${}", name.green().bold()).normal()
}

fn format_var(var: &Variable) -> String {
    format!(
        "[ {} => {} ]",
        format_var_name(&var.repr),
        format!("{:.3}", var.value).blue()
    )
}

#[allow(clippy::module_name_repetitions)]
/// Formats a printable string listing all the `Variables` in the given slice `vars`
pub fn format_vars(vars: &[Variable]) -> String {
    vars.iter().map(format_var).join("\n")
}

/// Takes the given user `input` and splits it up into a name and value to be assigned or reassigned to a [Variable] in `vars`
pub fn assign_var_command<'var, 'func>(
    input: &str,
    vars: &'var mut Vec<Variable>,
    funcs: &'func [Function],
) -> Result<String, Error> {
    // Variable assignment/reassignment

    let sides: Vec<&str> = input.split('=').collect();
    let trimmed_left = sides[0].trim(); // Trim here to remove space between end of variable name and = sign

    if sides.len() != 2 {
        // Multiple = signs || Assigning without using a $ prefix
        return Err(Error::Assignment);
    }

    let user_repr: String = trimmed_left[1..].to_string(); // Trim again to remove whitespace between end of variable name and = sign

    let context = EvaluationContext {
        vars,
        funcs,
        depth: 0,
        context: ErrorContext::Main
    };

    // Get value for variable
    let result = doeval(sides[1], context);
    if let Err(ContextLibError {
        error: LibError::Parsing(idx),
        ..
    }) = result {
        return Err(Error::Library(LibError::Parsing(idx + sides[0].len() + 1).with_context(ErrorContext::Main)));
        // Length of untrimmed lefthand side
        // Offset is added so that highlighting can be added to expressions that come after an '=' during assignment
    }
    let (user_value, repr) =  result?;

    let conf_string = format!(
        "[ ${} {} {} ] => {}",
        user_repr.green().bold(),
        "=".cyan(),
        stringify(&repr),
        format!("{:.3}", user_value).blue()
    );

    let var = Variable {
        repr: user_repr,
        value: user_value,
    };

    assign_var(var, vars);

    Ok(conf_string)
}

pub fn assign_var(var: Variable, vars: &mut Vec<Variable>) {
    let repr = var.repr.clone();
    let cmp = |v: &Variable| repr.cmp(&v.repr);

    insert_or_swap_sort(vars, var, cmp)
}
