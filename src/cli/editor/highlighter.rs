use std::borrow::Cow;

use colored::Colorize;
use rustyline::highlight::Highlighter;

use super::MyHelper;

impl Highlighter for MyHelper<'_> {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> std::borrow::Cow<'l, str> {
        Cow::Borrowed(line)
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> std::borrow::Cow<'b, str> {
        Cow::Borrowed(prompt)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> std::borrow::Cow<'h, str> {
        Cow::Owned(hint.black().on_white().to_string())
    }

    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str,
        _completion: rustyline::CompletionType,
    ) -> std::borrow::Cow<'c, str> {
        // We don't highlight the candidate because the completer formats with color
        Cow::Borrowed(candidate)
    }

    fn highlight_char(&self, _line: &str, _pos: usize) -> bool {
        true
    }
}
