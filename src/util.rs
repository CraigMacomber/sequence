#[derive(Clone)]
pub struct ImSlice<'a> {
    data: im_rc::vector::Focus<'a, u8>,
}

impl<'a> From<&'a im_rc::Vector<u8>> for ImSlice<'a> {
    fn from(data: &'a im_rc::Vector<u8>) -> Self {
        ImSlice { data: data.focus() }
    }
}

impl<'a> ImSlice<'a> {
    pub fn slice_with_length(&self, offset: usize, length: usize) -> Self {
        ImSlice {
            data: self.data.clone().narrow(offset..offset + length),
        }
    }
}

impl<'a> From<ImSlice<'a>> for Vec<u8> {
    fn from(s: ImSlice<'a>) -> Self {
        // TODO: maybe remove need for this, or do something better (use chunk_at?)
        s.data.into_iter().cloned().collect()
    }
}

mod tests {
    #[test]
    fn narrow() {
        // Confirms focus.narrow indexes relative to the beginning of the restricted range (not clear from docs)
        let data = im_rc::vector![0, 1, 2, 3, 4, 5];

        let mut s1 = data.focus().narrow(3..4);
        let mut s2 = s1.clone().narrow(0..1);

        assert_eq!(*s1.get(0).unwrap(), 3);
        assert_eq!(*s2.get(0).unwrap(), 3);
    }
}
