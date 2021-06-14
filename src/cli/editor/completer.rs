use rustyline::completion::Completer;

use crate::utils::find_last;

use super::{MyCandidate, MyHelper};

impl Completer for MyHelper<'_> {
    type Candidate = MyCandidate;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        if let Some(npos) = find_last('#', &line[..pos]) {
            let line = &line[npos + 1..pos];
            let funcs = self.funcs.borrow();

            if let Some(func) = funcs.iter().find(|f| f.name.starts_with(line)) {
                return rustyline::Result::Ok((
                    pos - npos,
                    vec![MyCandidate(func.name[pos - npos - 1..].to_string())],
                ));
            }
        } else if let Some(npos) = find_last('$', &line[..pos]) {
            let line = &line[npos + 1..pos];
            let vars = self.vars.borrow();

            if let Some(var) = vars.iter().find(|v| v.repr.starts_with(line)) {
                // return Some(MyCandidate(var.repr[pos-npos-1..].to_string()));
                return rustyline::Result::Ok((
                    pos - npos,
                    vec![MyCandidate(var.repr[pos - npos - 1..].to_string())],
                ));
            }
        }

        rustyline::Result::Ok((0, vec![]))
    }

    fn update(&self, line: &mut rustyline::line_buffer::LineBuffer, start: usize, elected: &str) {
        let end = line.pos();
        line.replace(start..end, elected);
    }
}