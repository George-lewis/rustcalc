#![allow(clippy::module_name_repetitions)]

use std::{borrow::Cow, cell::RefCell};

use colored::Colorize;
use rustmatheval::{
    model::{functions::Function, variables::Variable, EvaluationContext},
    tokenize,
};

use rustyline::{
    completion::{Candidate, Completer},
    highlight::Highlighter,
    hint::{Hint, Hinter},
    validate::{ValidationResult, Validator},
    Helper,
};

use crate::utils::find_last;

#[allow(dead_code)]
pub struct MyHelper<'cell> {
    pub funcs: &'cell RefCell<Vec<Function>>,
    pub vars: &'cell RefCell<Vec<Variable>>,

    pub valid: RefCell<bool>,
}

pub struct MyCandidate(String);

impl Candidate for MyCandidate {
    fn display(&self) -> &str {
        &self.0
    }

    fn replacement(&self) -> &str {
        &self.0
    }
}

impl Hint for MyCandidate {
    fn display(&self) -> &str {
        &self.0
    }

    fn completion(&self) -> Option<&str> {
        Some(&self.0)
    }
}

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

impl Highlighter for MyHelper<'_> {
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> std::borrow::Cow<'l, str> {
        Cow::Borrowed(line)
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> std::borrow::Cow<'b, str> {
        let valid = *self.valid.borrow();
        if valid {
            Cow::Borrowed(prompt)
        } else {
            Cow::Owned(prompt.red().to_string())
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> std::borrow::Cow<'h, str> {
        Cow::Owned(hint.black().on_white().to_string())
    }

    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str,
        completion: rustyline::CompletionType,
    ) -> std::borrow::Cow<'c, str> {
        let form = candidate.red();
        Cow::Owned(form.to_string())
    }

    fn highlight_char(&self, _line: &str, _pos: usize) -> bool {
        true
    }
}

impl Validator for MyHelper<'_> {
    fn validate(
        &self,
        ctx: &mut rustyline::validate::ValidationContext,
    ) -> rustyline::Result<ValidationResult> {
        let line = ctx.input();
        let context = EvaluationContext::default();

        // dbg!(line);

        let (valid_, result) = if tokenize(line, &context).is_err() {
            (false, ValidationResult::Incomplete)
        } else {
            (true, ValidationResult::Valid(None))
        };
        *self.valid.borrow_mut() = valid_;
        rustyline::Result::Ok(result);
        Ok(rustyline::validate::ValidationResult::Valid(None))
    }

    fn validate_while_typing(&self) -> bool {
        // true
        false
    }
}

impl Helper for MyHelper<'_> {}
