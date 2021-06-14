use rustmatheval::model::{
    functions::{Function, PREFIX as FUNCTION_PREFIX},
    variables::{Variable, PREFIX as VARIABLE_PREFIX},
};

use crate::funcs::format_func_with_args;
use crate::vars::format_var_name;

/// Find the position of the last instance of `c`
///
/// ## Examples
///
/// ```
/// use  rustmatheval::model::functions::PREFIX;
/// let s = format1("abc {}foo bar", PREFIX);
/// let pos = find_last(PREFIX, s).unwrap();
/// assert_eq!(s.chars().nth(pos), PREFIX);
/// ```
pub fn find_last(c: char, str: &str) -> Option<usize> {
    str.chars()
        .into_iter()
        .rev()
        .position(|ch| ch == c)
        .map(|pos| str.chars().count() - pos - 1)
}

/// Find all possibly completable `Item`s in `line` at the position closest to the end as indicated by [`Findable::prefix`]
/// and perform transformations on it using the two `Fn` parameters.
///
/// ## Parameters
/// * `Item` - A [`Findable`] type to search for in `line`
/// * `Intermediate` - A type that is part of the desired result.
/// * `Output` - This type is used to construct the output of the function
/// * `ToIntermediate` - A `Fn` type capable of converting `Item`s to `Intermediate`
/// * `ToOutput` - A `Fn` type capable of converting `Intermediate`s to `Output`
///
/// ## Arguments
/// * `line` - The string to find items within
/// * `items` - A slice of items, these are candidates for the search
/// * `create_intermediate` - A `Fn` that:
///   * accepts:
///     * `stride: usize`: The length of the matching string in `line` according to the prefix
///     * `item: &Item`: The matching item
///   * and produces an `Intermediate`
/// * `create_output` - a `FnOnce` that:
///   * accepts:
///     * `stride: usize` - ""
///     * `intermediates: Vec<Intermediate>` - A list of intermediates
///   * and produces the final output of the function (which is then wrapped in an Option, see the Returns subheader)
///
/// ## Returns
/// Returns `None` if there are no possible matches inside the input string. There are two reasons this could occur:
///   * There is no prefix in the string, and thus no matches are possible
///   * There is a prefix in the string, but none of the items could possibly complete the identifier
pub(super) fn find_items<Item, Intermediate, Output, ToIntermediate, ToOutput>(
    line: &str,
    items: &[Item],
    create_intermediate: ToIntermediate,
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
            .map(|it| create_intermediate(stride, it))
            .collect();
        if !matches.is_empty() {
            // +1 because of prefix
            return Some(create_output(stride + 1, matches));
        }
    }
    None
}

/// Represents a type that can be found (using [`find_items`])
pub trait Findable {
    fn name(&self) -> &str;
    fn format(&self) -> String;
    fn prefix() -> char;
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
