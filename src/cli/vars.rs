use colored::{ColoredString, Colorize};
use itertools::Itertools;
use lazy_static::__Deref;
use rustmatheval::DoEvalResult;

use std::cell::Cell;
use std::rc::Rc;

use super::error::Error;
use super::lib::doeval;
use super::lib::model::{
    errors::ErrorContext, functions::Function, variables::Variable, EvaluationContext,
};
use super::stringify::stringify;

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
    let user_repr = &trimmed_left[1..];

    let context = EvaluationContext {
        vars,
        funcs,
        depth: 0,
        context: ErrorContext::Main,
    };

    // Get value for variable
    let result = doeval(sides[1], context);

    if let DoEvalResult::Ok { tokens, result } = &result {
        let conf_string = format!(
            "[ ${} {} {} ] => {}",
            user_repr.green().bold(),
            "=".cyan(),
            stringify(tokens),
            format!("{:.3}", result).blue()
        );

        // Safety: This is a non-lexical lifetime that Rust
        // can't understand yet. By this point in the code
        // vars is no longer being borrowed
        // (the other branch of this if statement requires vars to be borrowed)
        // (which is why the compiler complains)
        let vars = unsafe {
            // transmute::<& _, &mut _>(&vars)
            #[allow(clippy::cast_ref_to_mut)]
            &mut *((vars as *const _) as *mut _)
        };

        assign_var(user_repr, *result, vars);

        Ok(conf_string)
    } else {
        Err(Error::Library(result))
    }
}

pub fn assign_var(name: &str, value: f64, vars: &mut Vec<Rc<Variable>>) {
    match vars.binary_search_by(|v| v.repr.deref().cmp(name)) {
        Ok(idx) => vars[idx].value.set(value),
        Err(idx) => {
            let var = Variable {
                repr: name.to_string(),
                value: Cell::new(value),
            };
            let var = Rc::new(var);
            vars.insert(idx, var);
        }
    }
}
