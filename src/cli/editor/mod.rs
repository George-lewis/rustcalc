use std::cell::RefCell;

use rustmatheval::model::{functions::Function, variables::Variable};
use rustyline::{completion::Candidate, hint::Hint, Editor, Helper};

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

        valid: RefCell::new(false),
    };

    editor.set_helper(Some(helper));

    editor
}

pub struct MyHelper<'cell> {
    pub funcs: &'cell RefCell<Vec<Function>>,
    pub vars: &'cell RefCell<Vec<Variable>>,

    pub valid: RefCell<bool>,
}

impl Helper for MyHelper<'_> {}

pub struct MyCandidate(String);

impl Candidate for MyCandidate {
    fn display(&self) -> &str {
        &self.0
    }

    fn replacement(&self) -> &str {
        &self.0
    }
}

impl Hint for MyCandidate {
    fn display(&self) -> &str {
        &self.0
    }

    fn completion(&self) -> Option<&str> {
        Some(&self.0)
    }
}
