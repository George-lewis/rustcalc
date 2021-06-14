use rustmatheval::model::{functions::{Function, PREFIX as FUNCTION_PREFIX}, variables::{Variable, PREFIX as VARIABLE_PREFIX}};

use crate::funcs::format_func_with_args;
use crate::vars::format_var_name;

/// Find the position of the last instance of `c`
///
/// ## Examples
///
/// ```
/// let s = "abc #foo bar";
/// let pos = find_last('#', s).unwrap();
/// assert_eq!(s.chars().nth(pos), '#');
/// ```
pub fn find_last(c: char, str: &str) -> Option<usize> {
    str.chars()
        .into_iter()
        .rev()
        .position(|ch| ch == c)
        .map(|pos| str.chars().count() - pos - 1)
}

pub(super) fn find_items<Item, Intermediate, Output, ToIntermediate, ToOutput>(
    line: &str,
    items: &[Item],
    create_item: ToIntermediate,
    create_output: ToOutput,
) -> Option<Output>
where
    Item: Findable,
    ToIntermediate: Fn(usize, &Item) -> Intermediate,
    ToOutput: FnOnce(usize, Vec<Intermediate>) -> Output,
{
    if let Some(pos) = find_last(Item::prefix(), line) {
        // +1 because of prefix
        let line = &line[pos + 1..];
        let stride = line.len() - pos;
        let matches: Vec<Intermediate> = items
            .iter()
            .filter(|it| it.name().starts_with(line))
            .map(|it| create_item(stride, it))
            .collect();
        if !matches.is_empty() {
            // +1 because of prefix
            return Some(create_output(stride + 1, matches));
        }
    }
    None
}

pub trait Findable {
    fn name(&self) -> &str;
    fn format(&self) -> String;
    fn prefix() -> char;
    // fn replacement(stride: usize) 
}

impl Findable for Function {
    fn name(&self) -> &str {
        &self.name
    }

    fn format(&self) -> String {
        format_func_with_args(self)
    }

    fn prefix() -> char {
        FUNCTION_PREFIX
    }
}

impl Findable for Variable {
    fn name(&self) -> &str {
        &self.repr
    }

    fn format(&self) -> String {
        format_var_name(&self.repr).to_string()
    }

    fn prefix() -> char {
        VARIABLE_PREFIX
    }
}