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
/// * `Intermediate` - The output type
/// * `ToIntermediate` - A `Fn` type capable of converting `Item`s to `Intermediate`
///
/// ## Arguments
/// * `line` - The string to find items within
/// * `items` - A slice of items, these are candidates for the search
/// * `create_intermediate` - A `Fn` that:
///   * accepts:
///     * `stride: usize`: The stride of the matching section
///     * `item: &Item`: The matching item
///   * and produces an `Intermediate`
///
/// ## Returns
/// Returns `None` if there are no possible matches inside the input stringThere are two reasons this could occur:
///   * There is no prefix in the string, and thus no matches are possible
///   * There is a prefix in the string, but none of the items could possibly complete the identifier
///
/// Otherwise: Returns a vector of intermediates
pub(super) fn find_items<Item, Intermediate, ToIntermediate>(
    line: &str,
    items: &[Item],
    create_intermediate: ToIntermediate,
) -> Option<Vec<Intermediate>>
where
    Item: Findable,
    ToIntermediate: Fn(usize, &Item) -> Intermediate,
{
    if let Some(pos) = find_last(Item::prefix(), line) {
        // +1 removes prefix
        // e.g. "#foobar" => "foobar"
        let line = &line[pos + 1..];
        let stride = line.len();

        let matches: Vec<Intermediate> = items
            .iter()
            .filter(|it| it.name().starts_with(line))
            .map(|it| create_intermediate(stride, it))
            .collect();
        if !matches.is_empty() {
            return Some(matches);
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
