pub trait Representable {
    fn repr(&self) -> &[&str];
}

pub trait Searchable {
    fn search<'a>(&'a self, search: &str) -> Option<(&'a Self, usize)>;
}

impl<Repr: Representable> Searchable for Repr {
    fn search<'a>(&'a self, search: &str) -> Option<(&'a Self, usize)> {
        self.repr()
            .iter()
            .find(|repr| search.to_lowercase().starts_with(&repr.to_lowercase()))
            .map(|repr| (self, repr.chars().count()))
    }
}

pub(super) fn get_by_repr<'a, T: Searchable>(
    search: &str,
    list: &'a [T],
) -> Option<(&'a T, usize)> {
    list.iter().find_map(|t| t.search(search))
}
