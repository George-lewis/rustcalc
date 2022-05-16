// Rustyline forces us
use std::{borrow::Cow, rc::Rc};

use colored::Colorize;
use itertools::Itertools;
use rustmatheval::{
    model::{
        errors::ErrorContext,
        functions::Function,
        tokens::StringTokenInterface,
        variables::{self, Variable},
        EvaluationContext,
    },
    tokenize,
    utils::{self, Pos},
};
use rustyline::highlight::Highlighter;

use crate::{
    funcs::{color_arg, format_func_name},
    stringify::stringify,
    utils::StripPrefix,
    vars::format_var_name,
};

use super::MyHelper;

struct Arg<'s> {
    repr: &'s str,
    offset: usize,
}

fn parse_func_def<'s>(string: &'s str) -> (&str, Vec<Arg>) {
    let fname = string.split_whitespace().next().unwrap();
    let mut args = vec![];

    let mut idx = fname.len();
    let mut start = None;
    let mut white = 0;

    let mut push_arg = |string: &'s str,
                        start_: usize,
                        end: &Pos,
                        start: &mut Option<usize>,
                        white: &mut usize| {
        let repr = utils::slice(string, start_, end);
        args.push(Arg {
            repr,
            offset: *white,
        });
        *start = None;
        *white = 0;
    };

    while idx < string.len() {
        if string.chars().nth(idx).unwrap().is_whitespace() {
            if let Some(start_) = start {
                push_arg(string, start_, &Pos::Idx(idx), &mut start, &mut white);
            }
            white += 1;
        } else if start.is_none() {
            start = Some(idx);
        }
        idx += 1;
    }

    if let Some(start_) = start {
        push_arg(string, start_, &Pos::Idx(idx), &mut start, &mut white);
    }

    (&fname[1..], args)
}

fn _format_func_def(fname: &str, args: &[Arg]) -> String {
    let mut fmt = format_func_name(fname).to_string();

    for arg in args {
        fmt.push_str(&" ".repeat(arg.offset));
        let repr = if arg.repr.starts_with(variables::PREFIX) {
            fmt.push(variables::PREFIX);
            &arg.repr[1..]
        } else {
            arg.repr
        };
        fmt.push_str(&color_arg(repr).to_string());
    }

    fmt
}

fn format_fun_def(def: &str, right: &str, context: &EvaluationContext) -> (String, String) {
    let (fname, args) = parse_func_def(def);

    let def = _format_func_def(fname, &args);

    let vars = args
        .iter()
        .map(|arg| {
            let repr = arg.repr.strip_pre(variables::PREFIX).to_owned();
            Variable::rc(repr, 0.0)
        })
        .chain(context.vars.iter().map(Rc::clone))
        .collect_vec();

    let context = EvaluationContext {
        vars: &vars,
        funcs: context.funcs,
        context: ErrorContext::Main,
        depth: 0,
    };

    let code = match tokenize(right, &context) {
        Ok(stoks) => stringify(&stoks),
        Err(ptoks) => stringify(&ptoks),
    };

    (def, code)
}

impl Highlighter for MyHelper<'_> {
    #[allow(clippy::similar_names)]
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
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

        let stringify = |line: &'l str| match tokenize(line, &context) {
            Ok(tokens) => stringify(&tokens),
            Err(tokens) => stringify(&tokens),
        };

        if line.contains('=') {
            let split: Vec<&str> = line.split('=').collect();
            if split.len() != 2 {
                // Bail
                return Cow::Borrowed(line);
            }

            let left = split[0];
            let right = split[1];

            let not_whitespace = |idx: usize, c: char| {
                if c.is_whitespace() {
                    None
                } else {
                    Some(idx)
                }
            };

            let lspace = left
                .char_indices()
                .rev()
                .find_map(|(idx, c)| not_whitespace(idx, c))
                .map_or(0, |x| left.chars().count() - x - 1);
            let rspace = right
                .char_indices()
                .find_map(|(idx, c)| not_whitespace(idx, c))
                .unwrap_or(0);

            let left = left.trim_end();
            let right = right.trim_start();

            let (left, right) = if Function::is(line) {
                format_fun_def(left, right, &context)
            } else if Variable::is(line) {
                (format_var_name(&left[1..]).to_string(), stringify(right))
            } else {
                Default::default()
            };

            return Cow::Owned(format!(
                "{left}{}={}{right}",
                " ".repeat(lspace),
                " ".repeat(rspace)
            ));
        }
        let string = stringify(line);
        Cow::Owned(string)
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> Cow<'b, str> {
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
