use rustmatheval::model::{functions::Function, variables::Variable};
use rustyline::completion::Completer;

use crate::{funcs::format_func_with_args, utils::find_last, vars::format_var_name};

use super::{MyCandidate, MyHelper};

fn find_candidates<Item: Named>(line: &str, items: &[Item]) -> Option<(usize, Vec<MyCandidate>)> {
    if let Some(pos) = find_last(Item::prefix(), line) {
        // +1 because of `key`
        let line = &line[pos + 1..];
        let stride = line.len() - pos;
        let matches: Vec<MyCandidate> = items
            .iter()
            .filter(|it| it.name().starts_with(line))
            .map(|it| {
                // -1 because of `key`
                let replacement = it.name()[stride - 1..].to_string();
                let formatted = it.format();
                MyCandidate(replacement, formatted)
            })
            .collect();
        if !matches.is_empty() {
            return Some((stride, matches));
        }
    }
    None
}

trait Named {
    fn name(&self) -> &str;
    fn format(&self) -> String;
    fn prefix() -> char;
}

impl Named for Function {
    fn name(&self) -> &str {
        &self.name
    }

    fn format(&self) -> String {
        format_func_with_args(self)
    }

    fn prefix() -> char {
        '#'
    }
}

impl Named for Variable {
    fn name(&self) -> &str {
        &self.repr
    }

    fn format(&self) -> String {
        format_var_name(&self.repr).to_string()
    }

    fn prefix() -> char {
        '$'
    }
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
