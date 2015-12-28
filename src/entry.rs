use std;
use std::cmp::Ordering;
use iter_util::common_count_eq;

pub struct Entry<'a> {
    slice_: &'a [u8],
    id_: u32,
}

impl<'a> Entry<'a> {
    pub fn new(slice: &'a [u8], id: u32) -> Entry<'a> {
        Entry { slice_: slice, id_: id }
    }
    pub fn common_count<'b>(&'a self, rhs: &Entry<'b>) -> usize {
        common_count_eq(self.slice_.iter(), rhs.slice_.iter())
    }
    pub fn len(&self) -> usize {
        self.slice_.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn iter(&self) -> std::iter::Rev<std::slice::Iter<'a, u8> > {
        self.slice_.iter().rev()
    }
    pub fn set_slice(&mut self, slice: &'a [u8]) {
        self.slice_ = slice;
    }
    pub fn get_id(&self) -> u32 {
        self.id_
    }
    pub fn set_id(&mut self, id: u32) {
        self.id_ = id;
    }
}

impl<'a> IntoIterator for &'a Entry<'a> {
    type Item = &'a u8;
    type IntoIter = std::iter::Rev<std::slice::Iter<'a, u8> >;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub fn cmp_slice(l: &Entry, r: &Entry) -> Ordering {
    l.slice_.cmp(&r.slice_)
}

pub fn cmp_id(l: &Entry, r: &Entry) -> Ordering {
    l.id_.cmp(&r.id_)
}

