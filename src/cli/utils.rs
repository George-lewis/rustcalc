use std::cmp::Ordering;

#[allow(clippy::let_underscore_drop)]
pub fn insert_or_swap_sort<Element, Cmp>(vec: &mut Vec<Element>, element: Element, cmp: Cmp)
where
    Cmp: Fn(&Element) -> Ordering,
{
    match vec.binary_search_by(cmp) {
        Ok(idx) => {
            let _ = std::mem::replace(&mut vec[idx], element);
        }
        Err(idx) => {
            vec.insert(idx, element);
        }
    }
}

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
