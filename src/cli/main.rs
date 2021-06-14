#![warn(clippy::pedantic, clippy::nursery)]

mod cli;
mod config;
mod error;
mod funcs;
mod rcfile;
mod stringify;
mod utils;
mod vars;
mod helper;

use lib::{doeval, model::EvaluationContext};
pub use rustmatheval as lib;

use config::HISTORY_FILE;
use rustyline::Editor;

use error::Error;
use std::{cell::RefCell, env, process};

use cli::{handle_errors, handle_input};

use crate::{cli::handle_library_errors, helper::MyHelper};

pub fn main() -> ! {
    // One-shot mode
    let args = env::args();

    // Usually the first argument is the path to the executable
    // So if there's more than one argument we interpret that as a one-shot
    if args.len() > 1 {
        // Combine all of the args into a string
        let fold = |acc: String, x: String| format!("{} {}", acc, x);
        let input: String = args.skip(1).fold(String::new(), fold);

        // Evaluate
        let context = EvaluationContext::default();
        let code = match doeval(&input, context) {
            Ok((result, _)) => {
                println!("{:.3}", result);
                0
            }
            Err(contextual_error) => {
                let msg = handle_library_errors(&contextual_error, &input);
                eprintln!("{}", msg);
                1
            }
        };

        // Exit
        process::exit(code);
    }

    let vars = RefCell::new(vec![]);
    let funcs = RefCell::new(vec![]);

    if let Err(inner) = rcfile::load(&mut vars.borrow_mut(), &mut funcs.borrow_mut()) {
        match inner {
            Error::Io(inner) => {
                println!("Error loading RCFile: {:#?}", inner);
            }
            _ => unreachable!(),
        }
    };

    let mut editor = Editor::<MyHelper>::new();

    let helper = MyHelper {
        funcs: &funcs,
        vars: &vars,

        valid: RefCell::new(false)
    };

    editor.set_helper(Some(helper));

    if let Some(path) = HISTORY_FILE.as_deref() {
        editor.load_history(path).ok();
    }

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

        match handle_input(&input, &mut vars.borrow_mut(), &mut funcs.borrow_mut()) {
            Ok(formatted) => println!("{}", formatted),
            Err(error) => {
                let msg = handle_errors(&error, &input);
                println!("{}", msg);
            }
        }
    }
}
