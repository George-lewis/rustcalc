use colored::Colorize;
use rustmatheval::{
    model::{errors::ErrorContext, EvaluationContext},
    tokenize,
};
use rustyline::highlight::Highlighter;

use crate::stringify::stringify;
use crate::Cow;

use super::MyHelper;

impl Highlighter for MyHelper<'_> {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> std::borrow::Cow<'l, str> {
        if line.trim().is_empty() {
            return Cow::Borrowed(line);
        }
        let funcs = &self.funcs.borrow();
        let vars = &self.vars.borrow();
        let context = EvaluationContext {
            vars,
            funcs,
            context: ErrorContext::Main,
            depth: 0,
        };
        let string = match tokenize(line, &context) {
            Ok(tokens) => stringify(&tokens),
            Err(tokens) => stringify(&tokens),
        };
        Cow::Owned(string)
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> std::borrow::Cow<'b, str> {
        Cow::Borrowed(prompt)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(hint.black().on_white().to_string())
    }

    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str,
        _completion: rustyline::CompletionType,
    ) -> Cow<'c, str> {
        // We don't highlight the candidate because the completer formats with color
        Cow::Borrowed(candidate)
    }

    fn highlight_char(&self, _line: &str, _pos: usize) -> bool {
        true
    }
}
