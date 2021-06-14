use std::borrow::Cow;

use colored::Colorize;
use rustyline::highlight::Highlighter;

use super::MyHelper;

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