pub trait Representable {
    fn repr(&self) -> &[&str];
}

pub trait Searchable {
    fn search<'a, 'b>(&'a self, search: &'b str) -> Option<(&'a Self, usize)>;
}

impl<Repr: Representable> Searchable for Repr {
    fn search<'a, 'b>(&'a self, search: &'b str) -> Option<(&'a Self, usize)> {
        self.repr()
            .iter()
            .find(|repr| search.to_lowercase().starts_with(&repr.to_lowercase()))
            .map(|repr| (self, repr.len()))
    }
}

pub(super) fn get_by_repr<'a, T: Searchable>(
    search: &str,
    list: &'a [T],
) -> Option<(&'a T, usize)> {
    list.iter().find_map(|t| t.search(search))
}
