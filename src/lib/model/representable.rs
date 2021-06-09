pub trait Representable {
    fn repr(&self) -> &[&str];
}

pub trait Searchable {
    fn search<'a>(&'a self, search: &str) -> Option<(&'a Self, &'a str)>;
}

impl<Repr: Representable> Searchable for Repr {
    fn search<'a>(&'a self, search: &str) -> Option<(&'a Self, &'a str)> {
        self.repr()
            .iter()
            .find(|repr| search.to_lowercase().starts_with(&repr.to_lowercase()))
            .map(|&repr| (self, repr))
    }
}

pub(super) fn get_by_repr<'a, T: Searchable>(
    search: &str,
    list: &'a [T],
) -> Option<(&'a T, &'a str)> {
    list.iter().find_map(|t| t.search(search))
}
