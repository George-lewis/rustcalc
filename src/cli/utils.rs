use std::cmp::Ordering;

use colored::ColoredString;

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

pub trait Effect {
    type Inner;

    fn effect<F>(self, f: F) -> Self
    where
        F: FnMut(&mut Self::Inner);
}

impl<T> Effect for Option<T> {
    type Inner = T;

    fn effect<F>(mut self, mut f: F) -> Self
    where
        F: FnMut(&mut T),
    {
        if let Some(t) = &mut self {
            f(t);
        }
        self
    }
}

pub trait Format {
    fn format(&self) -> String;
}

impl Format for ColoredString {
    fn format(&self) -> String {
        format!("{}", self)
    }
}
