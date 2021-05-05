#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::must_use_candidate, clippy::missing_panics_doc)]

mod cli;
mod config;
mod error;
mod rcfile;
mod stringify;
mod vars;
mod funcs;
mod utils;

use lib::model::functions::Function;
pub use rustmatheval as lib;

use config::HISTORY_FILE;
use rustyline::Editor;

use error::Error;
use std::process;

use cli::{handle_errors, handle_input};

pub fn main() -> ! {
    let mut editor = Editor::<()>::new();

    if let Some(path) = HISTORY_FILE.as_deref() {
        editor.load_history(path).ok();
    }

    let mut vars = vec![];
    let mut funcs = vec![
        Function {
            name: "sussy_baka".to_string(),
            args: vec!["a".to_string()],
            code: "$a + 1".to_string()
        }
    ];

    if let Err(inner) = rcfile::load(&mut vars, &mut funcs) {
        match inner {
            Error::Io(inner) => {
                println!("Error loading RCFile: {:#?}", inner)
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
