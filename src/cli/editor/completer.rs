use rustyline::completion::{Completer, Pair};

use super::{
    finder::{find_items, Findable},
    MyHelper,
};

fn find_candidates<Item: Findable>(line: &str, items: &[Item]) -> Option<Vec<Pair>> {
    let create_intermediate = |stride, item: &Item| {
        let replacement = item.name()[stride..].to_string();
        let display = item.format();
        Pair {
            display,
            replacement,
        }
    };
    find_items(line, items, create_intermediate)
}

impl Completer for MyHelper<'_> {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let line = &line[..pos];

        let funcs = self.funcs.borrow();
        let vars = self.vars.borrow();

        let candidates = if let Some(candidates) = find_candidates(line, &funcs) {
            (pos, candidates)
        } else if let Some(candidates) = find_candidates(line, &vars) {
            (pos, candidates)
        } else {
            (0, vec![])
        };

        rustyline::Result::Ok(candidates)
    }

    fn update(&self, line: &mut rustyline::line_buffer::LineBuffer, start: usize, elected: &str) {
        let end = line.pos();
        line.replace(start..end, elected);
    }
}
