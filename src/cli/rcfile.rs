use super::config::{DEFAULT_RCFILE, RCFILE};
use super::error::{Error, IoError};
use super::lib::model::{functions::Function, variables::Variable};
use colored::Colorize;
use std::{fs, io::ErrorKind::NotFound};

use super::cli::{handle_errors, handle_input};

/// Load Rustcalc's rcfile from the default location, `RCFILE`.
/// This function has side effects:
/// * Files may be created
/// * May write to stdout
///
/// ## Input
/// * `vars` - A mutable reference to the applications variables. Executing the rcfile may create variables.
///
/// ## Output
/// Returns an empty `Result` on success, or a `CliError` from io operations
pub fn load<'a>(vars: &mut Vec<Variable>, funcs: &mut Vec<Function>) -> Result<(), Error> {
    let path = match RCFILE.as_deref() {
        Some(path) => path,
        None => {
            return Err(IoError::new(NotFound, "Couldn't get path for config directory").into())
        }
    };

    // If RCFile doesn't exist, create it and write the default contents
    if !path.exists() {
        println!(
            "RCFile doesn't exist. Creating default at [{}]",
            path.to_string_lossy()
        );
        fs::write(path, DEFAULT_RCFILE)?;
    }

    // Read
    let lines = fs::read_to_string(path)?;

    // Filter out empty and comment lines
    let lines = lines
        .lines()
        .map(str::trim)
        .enumerate()
        .filter(|(_, line)| !(line.is_empty() || line.starts_with("//")));

    // Feed each line through `handle_input` and make use of `handle_errors`
    // Succesfully executing statements are silent
    for (n, line) in lines {
        if let Err(inner) = handle_input(line, vars, funcs) {
            let message = handle_errors(inner, line);
            println!(
                "Error in RCFile on line [{}]: {}",
                format!("{}", n).red(),
                message
            );
        }
    }
    Ok(())
}
