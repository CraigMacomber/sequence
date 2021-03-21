pub struct CloneIterator<T> {
    pub t: T,
}

impl<'a, TItem, T> Iterator for CloneIterator<T>
where
    T: Iterator<Item = &'a TItem>,
    TItem: 'a + Clone,
{
    type Item = TItem;

    fn next(&mut self) -> Option<Self::Item> {
        match self.t.next() {
            Some(x) => Some(x.clone()),
            None => None,
        }
    }
}
