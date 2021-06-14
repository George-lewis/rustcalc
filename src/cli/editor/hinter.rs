use rustyline::hint::Hinter;

use super::{
    finder::{find_items, Findable},
    MyHelper,
};

pub fn find_hint<Item: Findable>(line: &str, items: &[Item]) -> Option<String> {
    let create_intermediate = |stride, item: &Item| item.replacement()[stride..].to_string();
    let hints = find_items(line, items, create_intermediate);
    hints.and_then(|hints| hints.into_iter().max_by_key(String::len))
}

impl Hinter for MyHelper<'_> {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        let line = &line[..pos];

        let funcs = self.funcs.borrow();
        let vars = self.vars.borrow();

        find_hint(line, &funcs).or_else(|| find_hint(line, &vars))
    }
}
