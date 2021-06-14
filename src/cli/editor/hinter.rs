use rustyline::hint::Hinter;

use crate::utils::find_last;

use super::{MyCandidate, MyHelper};

impl Hinter for MyHelper<'_> {
    type Hint = MyCandidate;

    fn hint(&self, line: &str, pos: usize, _ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        if let Some(npos) = find_last('#', &line[..pos]) {
            let line = &line[npos + 1..pos];
            let funcs = self.funcs.borrow();

            if let Some(func) = funcs.iter().find(|f| f.name.starts_with(line)) {
                return Some(MyCandidate(func.name[pos - npos - 1..].to_string()));
            }
        } else if let Some(npos) = find_last('$', &line[..pos]) {
            let line = &line[npos + 1..pos];
            let vars = self.vars.borrow();

            if let Some(var) = vars.iter().find(|v| v.repr.starts_with(line)) {
                return Some(MyCandidate(var.repr[pos - npos - 1..].to_string()));
            }
        }

        None
    }
}
