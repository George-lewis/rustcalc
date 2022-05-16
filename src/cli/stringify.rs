use std::iter;

use colored::{ColoredString, Colorize};
use rustmatheval::model::{
    functions::Functions,
    operators::FUNCTIONAL_STYLE_OPERATORS,
    tokens::{PartialToken, StringToken, Tokens},
};

use crate::Cow;
use crate::{funcs::format_func_name, vars::format_var_name};

use super::lib::model::{
    operators::{Associativity, OperatorType},
    tokens::{ParenType, Token},
};

/// Creates a colored string representation of the input tokens
pub fn stringify<FormatToken>(tokens: &[FormatToken]) -> String
where
    FormatToken: StringableToken,
{
    _stringify(tokens, FormatToken::Options::default())
}

#[allow(dead_code, clippy::module_name_repetitions)]
pub fn stringify_opts<FormatToken>(tokens: &[FormatToken], opts: FormatToken::Options) -> String
where
    FormatToken: StringableToken,
{
    _stringify(tokens, opts)
}

#[allow(clippy::module_name_repetitions)]
pub fn stringify_off(tokens: &[Tokens]) -> (String, Vec<Offset>) {
    _stringify_off(tokens, StringTokenOpts::default())
}

#[allow(dead_code, clippy::module_name_repetitions)]
pub fn stringify_off_opts(tokens: &[Tokens], opts: StringTokenOpts) -> (String, Vec<Offset>) {
    _stringify_off(tokens, opts)
}

pub trait StringableToken {
    type Options: Default + Copy;

    fn spaces(&self, other: &Self, opts: Self::Options) -> usize;
    fn token(&self) -> Option<&Token<'_>>;
    fn colorize(&self) -> (ColoredString, usize);
    fn repr(&self) -> Cow<'_, str>;
}

impl StringableToken for Token<'_> {
    type Options = ();

    fn spaces(&self, other: &Self, _opts: ()) -> usize {
        if exclude_space(other) {
            0
        } else {
            spaces(self)
        }
    }

    fn token(&self) -> Option<&Token<'_>> {
        Some(self)
    }

    fn colorize(&self) -> (ColoredString, usize) {
        let repr = ideal_repr(self);
        (color_cli(&repr, self), repr.chars().count())
    }

    fn repr(&self) -> Cow<'_, str> {
        Cow::owned(ideal_repr(self))
    }
}

#[derive(Clone, Copy)]
pub struct StringTokenOpts {
    pub ideal_spacing: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for StringTokenOpts {
    fn default() -> Self {
        Self {
            ideal_spacing: false,
        }
    }
}

impl StringableToken for StringToken<'_, '_> {
    type Options = StringTokenOpts;

    fn spaces(&self, other: &Self, opts: StringTokenOpts) -> usize {
        if opts.ideal_spacing {
            // Determine if this token is a multicharacter representation of a builtin operator (non functional style)
            let is_multichar_operator = |tok: &StringToken| {
                if let Token::Operator { inner } = &self.inner {
                    if !inner.is_function() && tok.repr.chars().count() != 1 {
                        return true;
                    }
                }
                false
            };

            // Assumption: Multicharacter operators needs spaces on all sides
            // This assumption subsumes the need for additional complexity
            if is_multichar_operator(self) || is_multichar_operator(other) {
                return 1;
            }

            self.inner.spaces(&other.inner, ())
        } else {
            // If this token has a prefix, the idx was adjusted during tokenization
            let sub = if other.inner.has_prefix() { 1 } else { 0 };

            other.idx - (self.idx + self.repr.chars().count()) - sub
        }
    }

    fn token(&self) -> Option<&Token<'_>> {
        Some(&self.inner)
    }

    fn colorize(&self) -> (ColoredString, usize) {
        (color_cli(self.repr, &self.inner), self.repr.chars().count())
    }

    fn repr(&self) -> Cow<'_, str> {
        Cow::borrowed(self.repr)
    }
}

#[allow(clippy::cast_lossless)]
impl StringableToken for PartialToken<'_, '_> {
    type Options = ();

    fn spaces(&self, other: &Self, _opts: ()) -> usize {
        let sub = other
            .inner
            .as_ref()
            .map(|tok| tok.has_prefix() as usize)
            .unwrap_or(0);
        other.idx - (self.idx + self.repr.chars().count()) - sub
    }

    fn token(&self) -> Option<&Token<'_>> {
        self.inner.as_ref().ok()
    }

    fn colorize(&self) -> (ColoredString, usize) {
        let fmt = if let Ok(ref token) = self.inner {
            color_cli(self.repr, token)
        } else {
            self.repr.on_magenta()
        };
        (fmt, self.repr.chars().count())
    }

    fn repr(&self) -> Cow<'_, str> {
        Cow::borrowed(self.repr)
    }
}

impl StringableToken for Tokens<'_, '_> {
    type Options = StringTokenOpts;

    fn spaces(&self, other: &Self, opts: StringTokenOpts) -> usize {
        match self {
            Tokens::String(st) => match other {
                Tokens::String(st_) => st.spaces(st_, opts),
                Tokens::Synthetic(syn) => st.inner.spaces(syn, ()),
            },
            Tokens::Synthetic(syn) => {
                let other = other.token();
                syn.spaces(other, ())
            }
        }
    }

    fn token(&self) -> Option<&Token<'_>> {
        match self {
            Tokens::String(st) => Some(&st.inner),
            Tokens::Synthetic(syn) => Some(syn),
        }
    }

    fn colorize(&self) -> (ColoredString, usize) {
        match self {
            Tokens::String(st) => st.colorize(),
            Tokens::Synthetic(syn) => syn.colorize(),
        }
    }

    fn repr(&self) -> Cow<'_, str> {
        match self {
            Tokens::String(st) => Cow::borrowed(st.repr),
            Tokens::Synthetic(t) => Cow::owned(ideal_repr(t)),
        }
    }
}

/// Construct the ideal representation of a `Token`
fn ideal_repr(tok: &Token) -> String {
    match tok {
        Token::Number { value } => value.to_string(),
        Token::Operator {
            inner: Functions::Builtin(inner),
        } => inner.repr[0].to_string(),
        Token::Operator {
            inner: Functions::User(inner),
        } => inner.name.to_string(),
        Token::Paren { kind } => match kind {
            ParenType::Left => '('.to_string(),
            ParenType::Right => ')'.to_string(),
        },
        Token::Constant { inner } => inner.repr[0].to_string(),
        Token::Variable { inner } => inner.repr.to_string(),
        Token::Comma => ",".to_string(),
    }
}

/// Color tokens for the CLI
fn color_cli(string: &str, token: &Token) -> ColoredString {
    match token {
        Token::Number { .. } | Token::Comma => string.clear(),
        Token::Operator { inner: op } => match op {
            Functions::Builtin(_) => {
                if op.associativity() == Associativity::Left {
                    string.green().bold()
                } else {
                    string.blue().bold()
                }
            }
            Functions::User(_) => format_func_name(string),
        },
        Token::Paren { .. } => string.red(),
        Token::Constant { .. } => string.yellow(),
        Token::Variable { .. } => format_var_name(string),
    }
}

/// Determine if a space should be come after `cur` in a string representation.
#[allow(clippy::unnested_or_patterns, clippy::cast_lossless)]
fn spaces(cur: &Token) -> usize {
    // Cases:
    // - Spaces after value types: numbers, variables, and constants
    // - Spaces after r_parens and commas
    // - Spaces after all operators except function-style ones: sin, cos, tan, sqrt, ..
    //   - and pow
    // - Otherwise no spaces
    match cur {
        Token::Operator {
            inner: Functions::Builtin(op),
        } => (!FUNCTIONAL_STYLE_OPERATORS.contains(&op.kind) && op.kind != OperatorType::Pow) as _,
        Token::Paren {
            kind: ParenType::Right,
        }
        | Token::Number { .. }
        | Token::Variable { .. }
        | Token::Constant { .. }
        | Token::Comma => 1,

        // Otherwise none
        _ => 0,
    }
}

/// Determine if there can or cannot be a space before `next` in a string representation
///
/// ## Returns
/// True -> There cannot be a space between these tokens
///
/// False -> Spaces are permitted between these tokens
fn exclude_space(next: &Token) -> bool {
    // Cases:
    // - No spaces before an r_paren
    // - No spaces before certain operators: pow, and factorial
    // - All else is permitted
    match next {
        Token::Paren {
            kind: ParenType::Right,
        }
        | Token::Comma => true,
        Token::Operator {
            inner: Functions::Builtin(op),
        } => [OperatorType::Pow, OperatorType::Factorial].contains(&op.kind),
        _ => false,
    }
}

pub struct Offset {
    pub old_idx: usize,
    pub new_idx: usize,
}

fn _stringify_off(tokens: &[Tokens], opts: StringTokenOpts) -> (String, Vec<Offset>) {
    type Folded = (String, Vec<Offset>);
    type Mapped = (String, Option<Offset>);

    let mut acc = 0;
    let map = |tok: &Tokens, fmt: String, stride| {
        let (skip, off) = if let Tokens::String(st) = tok {
            let prefix_skip = if st.inner.has_prefix() { 1 } else { 0 };
            let off = Offset {
                old_idx: st.idx,
                new_idx: acc + prefix_skip,
            };
            (prefix_skip, Some(off))
        } else {
            (0, None)
        };
        // println!("{stride}; [{fmt}]");
        acc += stride + skip;
        (fmt, off)
    };

    let fold = |(mut astr, mut avec): Folded, (fmt, off): Mapped| {
        astr.push_str(&fmt);
        if let Some(off) = off {
            avec.push(off);
        }
        (astr, avec)
    };
    let init: Folded = (String::new(), vec![]);

    __stringify(tokens, map, fold, init, opts)
}

fn _stringify<Tok: StringableToken>(tokens: &[Tok], opts: Tok::Options) -> String {
    let map = |_, string, _| string;
    let fold = |mut acc: String, s: String| {
        acc.push_str(&s);
        acc
    };
    let init = String::new();

    __stringify(tokens, map, fold, init, opts)
}

fn __stringify<'tok, Tok, Mapper, Mapped, Folder, Folded>(
    tokens: &'tok [Tok],
    mut map: Mapper,
    fold: Folder,
    init: Folded,
    opts: Tok::Options,
) -> Folded
where
    Tok: StringableToken,
    Mapper: FnMut(&'tok Tok, String, usize) -> Mapped,
    Folder: Fn(Folded, Mapped) -> Folded,
{
    if tokens.is_empty() {
        return init;
    }

    // The last element of the slice
    // `std::slice::windows` does not include the last element as its own window
    // So we must add it ourselves
    //
    // The tuple is `(&Token, number_of_space: usize)`
    // There are not spaces after the last token, thus it is always zero
    let last = iter::once((
        tokens
            .last()
            .expect("Expected `tokens` to contain at least one token"),
        0,
    ));

    // Windows of size two let us determine if we want to insert a space
    // between them, given the context of the "current" and "next" token
    // Caveat: There will be no window for the last element, see above.
    tokens
        .windows(2)
        .map(|window| {
            let (cur, next) = (&window[0], &window[1]);

            // `exclude_space` determines if any conditions prevent there from being a space
            // and then `spaces` determines the number of spaces to insert, if they are permitted
            let space = cur.spaces(next, opts);

            (cur, space)
        })
        // Insert the last token
        .chain(last)
        // Color
        .map(|(token, space)| {
            let (colored, len) = token.colorize();
            let fmt = format!("{}{}", colored, " ".repeat(space));
            map(token, fmt, len + space)
        })
        .fold(init, fold)
}
