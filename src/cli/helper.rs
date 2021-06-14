#![allow(clippy::module_name_repetitions)]

use std::cell::RefCell;

use rustmatheval::model::{functions::Function, variables::Variable};

use rustyline::{Helper, completion::{Candidate, Completer}, highlight::Highlighter, hint::{Hint, Hinter}, validate::Validator};

#[allow(dead_code)]
pub struct MyHelper<'cell> {
    funcs: &'cell RefCell<Vec<Function>>,
    vars: &'cell RefCell<Vec<Variable>>
}

pub struct MyCandidate;

impl Candidate for MyCandidate {
    fn display(&self) -> &str {
        todo!()
    }

    fn replacement(&self) -> &str {
        todo!()
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
        let _ = (line, pos, ctx);
        Ok((0, Vec::with_capacity(0)))
    }

    fn update(&self, line: &mut rustyline::line_buffer::LineBuffer, start: usize, elected: &str) {
        let end = line.pos();
        line.replace(start..end, elected);
    }
}

pub struct MyHint;

impl Hint for MyHint {
    fn display(&self) -> &str {
        todo!()
    }

    fn completion(&self) -> Option<&str> {
        todo!()
    }
}

impl Hinter for MyHelper<'_> {
    type Hint = MyHint;

    fn hint(&self, line: &str, pos: usize, ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        let _ = (line, pos, ctx);
        None
    }
}

impl Highlighter for MyHelper<'_> {
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> std::borrow::Cow<'l, str> {
        let _ = pos;
        std::borrow::Cow::Borrowed(line)
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> std::borrow::Cow<'b, str> {
        let _ = default;
        std::borrow::Cow::Borrowed(prompt)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> std::borrow::Cow<'h, str> {
        std::borrow::Cow::Borrowed(hint)
    }

    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str,
        completion: rustyline::CompletionType,
    ) -> std::borrow::Cow<'c, str> {
        let _ = completion;
        std::borrow::Cow::Borrowed(candidate)
    }

    fn highlight_char(&self, _line: &str, _pos: usize) -> bool {
        true
    }
}

impl Validator for MyHelper<'_> {
    fn validate(&self, ctx: &mut rustyline::validate::ValidationContext) -> rustyline::Result<rustyline::validate::ValidationResult> {
        let _ = ctx;
        Ok(rustyline::validate::ValidationResult::Valid(None))
    }

    fn validate_while_typing(&self) -> bool {
        false
    }
}

impl Helper for MyHelper<'_> {}