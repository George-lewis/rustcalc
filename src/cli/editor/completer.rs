use rustyline::completion::{Candidate, Completer, Pair};

use super::{
    finder::{find_items, Findable},
    MyHelper,
};

fn find_candidates<Item: Findable>(
    line: &str,
    items: &[Item],
) -> Option<(usize, Vec<Pair>)> {
    let create_intermediate = |stride: usize, item: &Item| {
        let replacement = item.name()[stride..].to_string();
        let display = item.format();
        Pair { display, replacement }
    };
    let create_output = |stride: usize, candidates: Vec<Pair>| (stride, candidates);
    let c = find_items(line, items, create_intermediate, create_output);
    c
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
        // let x: String = line.lines().collect();
        // dbg!(x, start, end, elected);
        line.replace(start..end, elected);
    }
}
