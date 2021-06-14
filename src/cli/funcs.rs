use itertools::Itertools;
use rustmatheval::{
    model::{errors::ErrorContext, functions::Function, variables::Variable, EvaluationContext},
    tokenize_and_transform,
};

use colored::{ColoredString, Colorize};

use crate::{error::Error, stringify::stringify, utils::insert_or_swap_sort};

fn color_arg(arg: impl AsRef<str>) -> ColoredString {
    arg.as_ref().yellow()
}

fn stringify_func_code(func: &Function, funcs: &[Function], vars: &[Variable]) -> String {
    // We don't care about the actual value of the arguments here
    // Because we're just going to tokenize it
    let args = [0.0].repeat(func.arity());

    // Creates args and merges with variables in-scope (`vars`)
    let vars = func.create_variables(&args, vars);

    // Depth and context also don't matter here
    let context = EvaluationContext {
        vars: &vars,
        funcs,
        depth: 0,
        context: ErrorContext::Main,
    };

    // If the function code references variables or other functions
    // That don't exist right now, the tokenize will fail
    // So we just fall back to a copy of the function's code
    match tokenize_and_transform(&func.code, &context) {
        Ok(tokens) => stringify(&tokens),
        Err(_) => func.code.clone(),
    }
}

pub fn format_func_name(name: &str) -> ColoredString {
    format!("#{}", name.magenta().bold()).normal()
}

fn format_func(func: &Function, funcs: &[Function], vars: &[Variable]) -> String {
    format!(
        "[ {}({}) = {} ]",
        format_func_name(&func.name),
        func.args.iter().map(color_arg).join(", "),
        stringify_func_code(func, funcs, vars)
    )
}

#[allow(clippy::module_name_repetitions)]
pub fn format_funcs(funcs: &[Function], vars: &[Variable]) -> String {
    funcs.iter().map(|f| format_func(f, funcs, vars)).join("\n")
}

pub fn assign_func_command(
    input: &str,
    funcs: &mut Vec<Function>,
    vars: &[Variable],
) -> Result<String, Error> {
    let sides: Vec<&str> = input.split('=').map(str::trim).collect();

    if sides.len() != 2 {
        // Multiple = signs || Assigning without using a $ prefix
        return Err(Error::Assignment);
    }

    let left = sides[0];
    let right = sides[1];

    let mut split = left.split_whitespace();

    let name = split.next().unwrap()[1..].to_string();
    let args: Vec<String> = split.map(|arg| arg[1..].to_string()).collect();
    let code = right.to_string();

    if code.is_empty() {
        return Err(Error::Assignment);
    }

    let func = Function { name, args, code };

    assign_func(func.clone(), funcs);

    let formatted = format_func(&func, funcs, vars);

    Ok(formatted)
}

pub fn assign_func(func: Function, funcs: &mut Vec<Function>) {
    let name = func.name.clone();
    let cmp = |f: &Function| name.cmp(&f.name);

    insert_or_swap_sort(funcs, func, cmp);
}
