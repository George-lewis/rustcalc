// pub trait Representable {
//     fn repr(&self) -> &[&str];
// }

pub trait Searchable {
    fn search<'a>(&'a self, search: &str) -> Option<(&'a Self, usize)>;
}

// impl<Repr: Representable> Searchable for Repr {
//     fn search<'a>(&'a self, search: &str) -> Option<(&'a Self, usize)> {
//         self.repr()
//             .iter()
//             .find(|repr| search.to_lowercase().starts_with(&repr.to_lowercase()))
//             .map(|repr| (self, repr.len()))
//     }
// }

pub(super) fn get_by_repr<'a, T, L>(search: &str, list: L) -> Option<(&'a T, usize)>
where
    T: Searchable,
    L: IntoIterator<Item = &'a T>,
{
    list.into_iter().find_map(|t| t.search(search))
}
