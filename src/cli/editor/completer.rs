use rustyline::completion::Completer;

use crate::{funcs::format_func_with_args, utils::find_last, vars::format_var_name};

use super::{MyCandidate, MyHelper};

impl Completer for MyHelper<'_> {
    type Candidate = MyCandidate;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        if let Some(npos) = find_last('#', &line[..pos]) {
            let line = &line[npos + 1..pos];
            let funcs = self.funcs.borrow();

            let matches: Vec<_> = funcs.iter().filter(|f| f.name.starts_with(line))
            .map(|f| MyCandidate(f.name[pos - npos - 1..].to_string(), format_func_with_args(f))).collect();

            if !matches.is_empty() {
                return rustyline::Result::Ok((
                    pos - npos,
                    matches,
                ));
            }
        } else if let Some(npos) = find_last('$', &line[..pos]) {
            let line = &line[npos + 1..pos];
            let vars = self.vars.borrow();

            let matches: Vec<_> = vars.iter().filter(|v| v.repr.starts_with(line))
            .map(|v| MyCandidate(v.repr[pos - npos - 1..].to_string(), format_var_name(&v.repr).to_string())).collect();

            if !matches.is_empty() {
                // return Some(MyCandidate(var.repr[pos-npos-1..].to_string()));
                return rustyline::Result::Ok((
                    pos - npos,
                    matches,
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
