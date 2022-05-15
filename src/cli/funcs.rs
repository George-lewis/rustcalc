use std::rc::Rc;

use itertools::Itertools;
use rustmatheval::{
    model::{
        errors::ErrorContext,
        functions::Function,
        variables::{self, Variable},
        EvaluationContext,
    },
    tokenize,
};

use colored::{ColoredString, Colorize};

use crate::{error::Error, stringify::stringify, utils::insert_or_swap_sort};

fn color_arg(arg: impl AsRef<str>) -> ColoredString {
    arg.as_ref().yellow()
}

fn stringify_func_code(func: &Function, funcs: &[Function], vars: &[Rc<Variable>]) -> String {
    // We don't care about the actual value of the arguments here
    // Because we're just going to tokenize it
    let args = [0_f64].repeat(func.arity());

    // Creates args and merges with variables in-scope (`vars`)
    let args = func.create_arguments(&args);

    let vars = args
        .into_iter()
        .chain(vars.iter().map(Rc::clone))
        .collect_vec();

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
    let tokens = tokenize(&func.code, &context).expect("Function code is invalid?");
    stringify(&tokens)
}

pub fn format_func_name(name: &str) -> ColoredString {
    format!("#{}", name.magenta().bold()).normal()
}

pub fn format_func_with_args(func: &Function) -> String {
    format!(
        "{}({})",
        format_func_name(&func.name),
        func.args.iter().map(color_arg).join(", ")
    )
}

fn format_func(func: &Function, funcs: &[Function], vars: &[Rc<Variable>]) -> String {
    format!(
        "[ {} = {} ]",
        format_func_with_args(func),
        stringify_func_code(func, funcs, vars)
    )
}

#[allow(clippy::module_name_repetitions)]
pub fn format_funcs(funcs: &[Function], vars: &[Rc<Variable>]) -> String {
    funcs.iter().map(|f| format_func(f, funcs, vars)).join("\n")
}

pub fn assign_func_command<'a>(
    input: &'a str,
    funcs: &'a mut Vec<Function>,
    vars: &'a [Rc<Variable>],
) -> Result<String, Error<'a>> {
    let sides: Vec<&str> = input.split('=').map(str::trim).collect();

    if sides.len() != 2 {
        // Multiple = signs || Assigning without using a $ prefix
        return Err(Error::Assignment);
    }

    let left = sides[0];
    let right = sides[1];

    let mut split = left.split_whitespace();

    let name = split.next().unwrap()[1..].to_string();
    let args = split
        .map(|arg| {
            if arg.starts_with(variables::PREFIX) {
                arg[1..].to_string()
            } else {
                arg.to_string()
            }
        })
        .collect_vec();
    let code = right.to_string();
    let func = Function { name, args, code };

    let formatted = format_func(&func, funcs, vars);

    assign_func(func, funcs);

    Ok(formatted)
}

pub fn assign_func(func: Function, funcs: &mut Vec<Function>) {
    let name = func.name.clone();
    let cmp = |f: &Function| name.cmp(&f.name);

    insert_or_swap_sort(funcs, func, cmp);
}
