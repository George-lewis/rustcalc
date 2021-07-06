use std::{cell::RefCell, rc::Rc};

use rustmatheval::model::{functions::Function, variables::Variable};
use rustyline::{config::Configurer, Editor, Helper};

mod completer;
mod finder;
mod highlighter;
mod hinter;
mod validator;

pub fn editor<'cell>(
    funcs: &'cell RefCell<Vec<Function>>,
    vars: &'cell RefCell<Vec<Rc<Variable>>>,
) -> Editor<MyHelper<'cell>> {
    let mut editor = Editor::<MyHelper>::new();

    let helper = MyHelper { funcs, vars };

    editor.set_helper(Some(helper));

    editor.set_completion_type(rustyline::CompletionType::List);
    editor.set_auto_add_history(true);

    editor
}

pub struct MyHelper<'cell> {
    pub funcs: &'cell RefCell<Vec<Function>>,
    pub vars: &'cell RefCell<Vec<Rc<Variable>>>,
}

impl Helper for MyHelper<'_> {}
