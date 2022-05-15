#![warn(clippy::pedantic, clippy::nursery)]

mod cli;
mod config;
mod editor;
mod error;
mod funcs;
mod rcfile;
mod stringify;
mod utils;
mod vars;

use lib::{
    doeval,
    model::{functions::Function, variables::Variable, EvaluationContext},
    DoEvalResult,
};
pub use rustmatheval as lib;

use config::HISTORY_FILE;

use error::Error;
use std::{cell::RefCell, env, process, rc::Rc};

use cli::{handle_errors, handle_input};

use crate::{cli::handle_library_errors, editor::editor};

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
            DoEvalResult::Ok { result, .. } => {
                println!("{result:.3}");
                0
            }
            error => {
                let msg = handle_library_errors(&error, &input);
                eprintln!("{}", msg);
                1
            }
        };

        // Exit
        process::exit(code);
    }

    let vars: RefCell<Vec<Rc<Variable>>> = RefCell::new(vec![]);
    let funcs: RefCell<Vec<Function>> = RefCell::new(vec![]);

    if let Err(error) = rcfile::load(&mut vars.borrow_mut(), &mut funcs.borrow_mut()) {
        println!("Error loading RCFile: {:#?}", error);
    };

    let mut editor = editor(&funcs, &vars);

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

        match handle_input(&input, &mut vars.borrow_mut(), &mut funcs.borrow_mut()) {
            Ok(formatted) => println!("{}", formatted),
            Err(error) => {
                let msg = handle_errors(&error, &input);
                println!("{}", msg);
            }
        }
    }
}
