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

#[derive(Clone)]
pub struct ImSlice<'a> {
    data: im_rc::vector::Focus<'a, u8>
}

impl<'a> From<&'a im_rc::Vector<u8>> for ImSlice<'a> {
    fn from(data: &'a im_rc::Vector<u8>) -> Self {
        ImSlice {
            data: data.focus()
        }
    }
}

impl<'a> ImSlice<'a> {
    pub fn slice_with_length(&self, offset: usize, length: usize) -> Self {
        ImSlice {
            data: self.data.clone().narrow(offset..offset + length )
        }
    }
}

impl<'a> From<ImSlice<'a>> for Vec<u8> {
    fn from(s: ImSlice<'a>) -> Self {
        // TODO: maybe remove need for this, or do something better (use chunk_at?)
        s.data
            .into_iter()
            .map(|x| *x)
            .collect::<Vec<_>>()
    }
}
