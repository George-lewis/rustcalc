use rustyline::completion::{Candidate, Completer};

use super::{
    finder::{find_items, Findable},
    MyHelper,
};

#[derive(Debug)]
pub struct MyCandidate(String, String);

impl Candidate for MyCandidate {
    fn display(&self) -> &str {
        &self.1
    }

    fn replacement(&self) -> &str {
        &self.0
    }
}

fn find_candidates<Item: Findable>(line: &str, items: &[Item]) -> Option<(usize, Vec<MyCandidate>)> {
    let create_item = |stride: usize, item: &Item| {
        let replacement = item.name()[stride..].to_string();
        let formatted = item.format();
        MyCandidate(replacement, formatted)
    };
    let create_output = |stride: usize, candidates: Vec<MyCandidate>| (stride, candidates);
    find_items(line, items, create_item, create_output)
}

impl Completer for MyHelper<'_> {
    type Candidate = MyCandidate;

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
            candidates
        } else if let Some(candidates) = find_candidates(line, &vars) {
            candidates
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
