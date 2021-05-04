use colored::Colorize;
use itertools::Itertools;

use super::error::{Error, LibError};
use super::lib::doeval;
use super::lib::model::variables::Variable;
use super::stringify::stringify;

#[allow(clippy::module_name_repetitions)]
/// Formats a printable string listing all the `Variables` in the given slice `vars`
pub fn format_vars(vars: &[Variable]) -> String {
    vars.iter()
        .map(|var| {
            format!(
                "[ ${} => {} ]",
                var.repr.green().bold(),
                format!("{:.3}", var.value).blue()
            )
        })
        .join("\n")
}

/// Takes the given user `input` and splits it up into a name and value to be assigned or reassigned to a [Variable] in `vars`
pub fn assign_var_command(input: &str, vars: &mut Vec<Variable>) -> Result<String, Error> {
    // Variable assignment/reassignment

    let sides: Vec<&str> = input.split('=').collect();
    let trimmed_left = sides[0].trim(); // Trim here to remove space between end of variable name and = sign

    if sides.len() != 2 || !trimmed_left.starts_with('$') {
        // Multiple = signs || Assigning without using a $ prefix
        return Err(Error::Assignment);
    }

    let user_repr: String = trimmed_left[1..].to_string(); // Trim again to remove whitespace between end of variable name and = sign

    // Get value for variable
    let result = doeval(sides[1], vars);
    if let Err(LibError::Parsing(idx)) = result {
        return Err(Error::Library(LibError::Parsing(idx + sides[0].len() + 1)));
        // Length of untrimmed lefthand side
        // Offset is added so that highlighting can be added to expressions that come after an '=' during assignment
    }
    let (user_value, repr) = result?;

    // Get printable confirmation string
    let conf_string = format!(
        "[ ${} {} {} ] => {}",
        user_repr.green().bold(),
        "=".cyan(),
        stringify(&repr),
        format!("{:.3}", user_value).blue()
    );

    assign_var(vars, &user_repr, user_value);

    Ok(conf_string)
}

/// Searches `vars` for the given `user_repr` to find if a [Variable] exists, and either reassigns it to, or creates it with, the given `user_value`
pub fn assign_var(vars: &mut Vec<Variable>, repr: &str, value: f64) {
    let cmp = |var: &Variable| repr.cmp(&var.repr);
    let search = vars.binary_search_by(cmp);
    match search {
        Ok(idx) => {
            vars[idx].value = value;
        }
        Err(idx) => {
            let var = Variable {
                repr: repr.to_string(),
                value,
            };
            vars.insert(idx, var);
        }
    }
}

#[cfg(test)]
mod tests {
    
}