use im_rc::vector::Focus;

pub type ImSlice<'a> = im_rc::vector::Focus<'a, u8>;

pub fn slice_with_length(
    focus: im_rc::vector::Focus<'_, u8>,
    offset: usize,
    length: usize,
) -> Focus<'_, u8> {
    focus.narrow(offset..offset + length)
}

pub type ImHashMap<K, V> = im_rc::HashMap<K, V, ahash::RandomState>;

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
