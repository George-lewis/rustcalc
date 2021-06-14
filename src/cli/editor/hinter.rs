use rustyline::hint::{Hint, Hinter};

use super::{
    finder::{find_items, Named},
    MyHelper,
};

pub struct MyHint(String);

impl Hint for MyHint {
    fn display(&self) -> &str {
        &self.0
    }

    fn completion(&self) -> Option<&str> {
        Some(&self.0)
    }
}

pub fn find_hint<Item: Named>(line: &str, items: &[Item]) -> Option<MyHint> {
    let create_item = |stride: usize, item: &Item| MyHint(item.name()[stride..].to_string());
    let create_output = |_, hints: Vec<MyHint>| hints;
    let hints = find_items(line, items, create_item, create_output);
    hints.and_then(|hints| hints.into_iter().max_by_key(|hint| hint.0.len()))
}

impl Hinter for MyHelper<'_> {
    type Hint = MyHint;

    fn hint(&self, line: &str, pos: usize, _ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        let line = &line[..pos];

        let funcs = self.funcs.borrow();
        let vars = self.vars.borrow();

        find_hint(line, &funcs).or_else(|| find_hint(line, &vars))
    }
}
