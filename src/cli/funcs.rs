use itertools::Itertools;
use rustmatheval::model::functions::Function;

use colored::{ColoredString, Colorize};

use crate::{error::Error, utils::insert_or_swap_sort};

pub fn format_func_name(name: &str) -> ColoredString {
    format!("#{}", name.magenta().bold()).normal()
}

fn format_func(func: &Function) -> String {
    format!(
        "[ {}({}) = {} ]",
        format_func_name(&func.name),
        func.args.join(", "),
        func.code
    )
}

#[allow(clippy::module_name_repetitions)]
pub fn format_funcs(funcs: &[Function]) -> String {
    funcs.iter().map(format_func).join("\n")
}

pub fn assign_func_command(input: &str, funcs: &mut Vec<Function>) -> Result<String, Error> {
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

    let func = Function { name, args, code };

    let formatted = format_func(&func);

    assign_func(func, funcs);

    Ok(formatted)
}

pub fn assign_func(func: Function, funcs: &mut Vec<Function>) {
    let name = func.name.clone();
    let cmp = |f: &Function| name.cmp(&f.name);

    insert_or_swap_sort(funcs, func, cmp);
}
