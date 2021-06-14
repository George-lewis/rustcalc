use rustyline::hint::Hinter;

use crate::utils::find_last;

use super::{MyHelper, MyHint};

impl Hinter for MyHelper<'_> {
    type Hint = MyHint;

    fn hint(&self, line: &str, pos: usize, _ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        if let Some(npos) = find_last('#', &line[..pos]) {
            let line = &line[npos + 1..pos];
            let funcs = self.funcs.borrow();

            if let Some(func) = funcs.iter().find(|f| f.name.starts_with(line)) {
                let s = func.name[pos - npos - 1..].to_string();
                return Some(MyHint(s));
            }
        } else if let Some(npos) = find_last('$', &line[..pos]) {
            let line = &line[npos + 1..pos];
            let vars = self.vars.borrow();

            if let Some(var) = vars.iter().find(|v| v.repr.starts_with(line)) {
                let s = var.repr[pos - npos - 1..].to_string();
                return Some(MyHint(s));
            }
        }

        None
    }
}
