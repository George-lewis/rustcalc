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
