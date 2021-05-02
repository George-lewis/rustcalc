pub trait Searchable {
    fn search<'a>(&'a self, search: &str) -> Option<(&'a Self, usize)>;
}

pub(super) fn get_by_repr<'a, T, L>(search: &str, list: L) -> Option<(&'a T, usize)>
where
    T: Searchable,
    L: IntoIterator<Item = &'a T>,
{
    list.into_iter().find_map(|t| t.search(search))
}
