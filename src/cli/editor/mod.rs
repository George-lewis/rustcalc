use std::cell::RefCell;

use rustmatheval::model::{functions::Function, variables::Variable};
use rustyline::{Editor, Helper, completion::Candidate, config::Configurer, hint::Hint};

mod candidate;

mod completer;
mod highlighter;
mod hinter;
mod validator;

pub fn editor<'a>(
    funcs: &'a RefCell<Vec<Function>>,
    vars: &'a RefCell<Vec<Variable>>,
) -> Editor<MyHelper<'a>> {
    let mut editor = Editor::<MyHelper>::new();

    let helper = MyHelper {
        funcs,
        vars,

        valid: RefCell::new(true),
    };

    editor.set_helper(Some(helper));

    editor.set_completion_type(rustyline::CompletionType::List);

    editor
}

pub struct MyHelper<'cell> {
    pub funcs: &'cell RefCell<Vec<Function>>,
    pub vars: &'cell RefCell<Vec<Variable>>,

    pub valid: RefCell<bool>,
}

impl Helper for MyHelper<'_> {}

pub struct MyCandidate(String, String);

impl Candidate for MyCandidate {
    fn display(&self) -> &str {
        &self.1
    }

    fn replacement(&self) -> &str {
        &self.0
    }
}

pub struct MyHint(String);

impl Hint for MyHint {
    fn display(&self) -> &str {
        &self.0
    }

    fn completion(&self) -> Option<&str> {
        Some(&self.0)
    }
}
