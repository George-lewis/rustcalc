use std::borrow::Borrow;

pub trait Representable {
    fn repr(&self) -> &[&str];
}

pub trait Searchable {
    fn search<'a, 'b>(&'a self, search: &'b str) -> Option<(&'a Self, &'b str)>;
}

impl<Repr: Representable> Searchable for Repr {
    fn search<'this, 'str>(&'this self, search: &'str str) -> Option<(&'this Self, &'str str)> {
        self.repr()
            .iter()
            .find(|repr| search.to_lowercase().starts_with(&repr.to_lowercase()))
            .map(|repr| (self, &search[..repr.chars().count()]))
    }
}

pub(super) fn get_by_repr<'list, 'str, T: Searchable, U: Borrow<T>>(
    search: &'str str,
    list: &'list [U],
) -> Option<(&'list U, &'str str)> {
    list.iter().find_map(|t| t.borrow().search(search).map(|(a, b)| (t, b)))
}
