use std::ops::Index;

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
    data: &'a im_rc::Vector<u8>,
    start: usize,
    length: usize,
}

impl<'a> From<&'a im_rc::Vector<u8>> for ImSlice<'a> {
    fn from(data: &'a im_rc::Vector<u8>) -> Self {
        ImSlice {
            data: &data,
            start: 0,
            length: data.len(),
        }
    }
}

impl<'a> ImSlice<'a> {
    pub fn slice_with_length(&self, offset: usize, length: usize) -> Self {
        assert!(offset + length <= self.length);
        ImSlice {
            data: self.data,
            start: self.start + offset,
            length,
        }
    }
}

impl<'a> From<ImSlice<'a>> for Vec<u8> {
    fn from(s: ImSlice<'a>) -> Self {
        // TODO: maybe remove need for this, or do something better (maybe im_vec could allow direct slicing if it happens to not span chunks?)
        s.data
            .clone()
            .slice(s.start..s.start + s.length)
            .iter()
            .map(|x| *x)
            .collect::<Vec<_>>()
    }
}

impl<'a> Index<usize> for ImSlice<'a> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.length);
        &self.data[index + self.start]
    }
}
