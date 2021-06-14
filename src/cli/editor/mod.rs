use std::cell::RefCell;

use rustmatheval::model::{functions::Function, variables::Variable};
use rustyline::{config::Configurer, Editor, Helper};

mod finder;
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
