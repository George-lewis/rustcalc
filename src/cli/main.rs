#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::must_use_candidate, clippy::missing_panics_doc)]

mod cli;
mod config;
mod error;
mod rcfile;
mod stringify;
mod vars;

use colored::Colorize;
use lib::{model::variables::Variable, utils::{self, Pos}};

pub use rustmatheval as lib;

use config::HISTORY_FILE;
use rustyline::{Editor, Helper, completion::{Candidate, Completer}, config::Configurer, highlight::Highlighter, hint::{Hint, Hinter}, validate::Validator};

use error::Error;
use stringify::{StringToken, _stringify, color_cli, stringify};
use std::{borrow::Cow, cell::RefCell, ops::DerefMut, process};

use cli::{handle_errors, handle_input};

struct MyHelper<'a> {
    vars: &'a RefCell<Vec<Variable>>
}

struct MyCandidate;

impl Candidate for MyCandidate {
    fn display(&self) -> &str {
        "xx"
    }

    fn replacement(&self) -> &str {
        "xx"
    }
}

impl Completer for MyHelper<'_> {
    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let _ = (line, pos, ctx);
        Ok((0, vec![MyCandidate]))
    }

    fn update(&self, line: &mut rustyline::line_buffer::LineBuffer, start: usize, elected: &str) {
        let end = line.pos();
        line.replace(start..end, elected)
    }

    type Candidate = MyCandidate;
}


struct Myhint(u8);
impl Hint for Myhint {
    fn display(&self) -> &str {
        "HINT"
    }

    fn completion(&self) -> Option<&str> {
        Some("HINT")
    }
}

impl Hinter for MyHelper<'_> {
    fn hint(&self, line: &str, pos: usize, ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        let _ = (line, pos, ctx);
        None
    }

    type Hint = Myhint;
}

impl Highlighter for MyHelper<'_> {
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> std::borrow::Cow<'l, str> {
        let _ = pos;
        // println!("color");
        // let tokens = lib::doeval(line, &[]);
        let borrow = &*self.vars.borrow();
        let tokens = lib::tokenize1(line, borrow, |a, b| StringToken {
            token: a,
            repr: b,
        });
        match tokens {
            Ok(tokens) => {
                let color = _stringify(&tokens, color_cli);
                let len: usize = tokens.iter().map(|s| s.repr.as_ref().map(|x| x.chars().count()).unwrap_or_default()).sum();
                let extra = utils::slice(line, len, &Pos::End);
                Cow::Owned(format!("{}{}", color, extra.red().clear()))
            },
            Err((a, _)) => {
                let color = _stringify(&a, color_cli);
                let len: usize = a.iter().map(|s| s.repr.as_ref().map(|x| x.chars().count()).unwrap_or_default()).sum();
                let extra = utils::slice(line, len, &Pos::End);
                Cow::Owned(format!("{}{}", color, extra.red().clear()))
            },
        }
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> std::borrow::Cow<'b, str> {
        // dbg!("prompt");
        let _ = default;
        std::borrow::Cow::Borrowed(prompt)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> std::borrow::Cow<'h, str> {
        dbg!("hint");
        std::borrow::Cow::Borrowed(hint)
    }

    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str,
        completion: rustyline::CompletionType,
    ) -> std::borrow::Cow<'c, str> {
        dbg!("cand");
        let _ = completion;
        std::borrow::Cow::Borrowed(candidate)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        // dbg!("char");
        // let _ = (line, pos);
        // false
        true
    }
}

impl Validator for MyHelper<'_> {
    fn validate(&self, ctx: &mut rustyline::validate::ValidationContext) -> rustyline::Result<rustyline::validate::ValidationResult> {
        let _ = ctx;
        Ok(rustyline::validate::ValidationResult::Valid(None))
    }

    fn validate_while_typing(&self) -> bool {
        false
    }
}

impl Helper for MyHelper<'_> {}

pub fn main() -> ! {
    let mut editor = Editor::<MyHelper>::new();

    let vars = RefCell::new(vec![]);

    let helper = MyHelper {
        vars: &vars
    };

    editor.set_helper(Some(helper));

    if let Some(path) = HISTORY_FILE.as_deref() {
        editor.load_history(path).ok();
    }

    if let Err(inner) = rcfile::load(&mut *vars.borrow_mut()) {
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
            Ok(line) => line.trim_end().to_string(),
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

        match handle_input(&input, &mut *vars.borrow_mut()) {
            Ok(formatted) => println!("{}", formatted),
            Err(error) => {
                let msg = handle_errors(error, &input);
                println!("{}", msg);
            }
        }
    }
}
