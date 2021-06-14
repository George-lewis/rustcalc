#![warn(clippy::pedantic, clippy::nursery)]

mod cli;
mod config;
mod error;
mod funcs;
mod rcfile;
mod stringify;
mod utils;
mod vars;

use itertools::Itertools;
use lib::{doeval, model::EvaluationContext};
pub use rustmatheval as lib;

use config::HISTORY_FILE;
use rustyline::Editor;

use error::Error;
use std::{env, process};

use cli::{handle_errors, handle_input};

pub fn main() -> ! {
    // One-shot mode
    let args = env::args();
    if args.len() > 1 {
        // Combine all of the args into a string
        let fold = |acc: String, x: String| format!("{} {}", acc, x);
        let input: String = args
            .skip(1)
            .fold1(fold).unwrap_or_default();

        // Evaluate
        let context = EvaluationContext::default();
        let code = match doeval(&input, context) {
            Ok((result, _)) => {
                println!("{:.3}", result);
                0
            }
            Err(ce) => {
                let error = Error::Library(ce);
                let msg = handle_errors(error, &input);
                eprintln!("{}", msg);
                1
            }
        };

        // Exit
        process::exit(code);
    }

    let mut editor = Editor::<()>::new();

    if let Some(path) = HISTORY_FILE.as_deref() {
        editor.load_history(path).ok();
    }

    let mut vars = vec![];
    let mut funcs = vec![];

    if let Err(inner) = rcfile::load(&mut vars, &mut funcs) {
        match inner {
            Error::Io(inner) => {
                println!("Error loading RCFile: {:#?}", inner);
            }
            _ => unreachable!(),
        }
    };

    loop {
        #[allow(clippy::single_match_else)]
        let input = match editor.readline("> ") {
            Ok(line) => line.trim().to_string(),
            Err(_) => {
                if let Some(path) = HISTORY_FILE.as_deref() {
                    editor.save_history(&path).ok();
                }
                process::exit(0)
            }
        };

        if input.is_empty() {
            continue;
        }

        // Add the line to the history
        editor.add_history_entry(&input);

        match handle_input(&input, &mut vars, &mut funcs) {
            Ok(formatted) => println!("{}", formatted),
            Err(error) => {
                let msg = handle_errors(error, &input);
                println!("{}", msg);
            }
        }
    }
}
