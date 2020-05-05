use redis::Value;
use std::borrow::Cow;
use std::{slice, vec};

/// An iterator that can iterate over borrowed or owned values from a borrowed or
/// owned vec
pub enum CowIter<'a> {
    Borrowed(slice::Iter<'a, Value>),
    Owned(vec::IntoIter<Value>),
}

impl<'a> CowIter<'a> {
    pub fn new(values: impl Into<Cow<'a, Vec<Value>>>) -> Self {
        match values.into() {
            Cow::Borrowed(values) => CowIter::Borrowed(values.iter()),
            Cow::Owned(values) => CowIter::Owned(values.into_iter()),
        }
    }
}

impl<'a> Iterator for CowIter<'a> {
    type Item = Cow<'a, Value>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CowIter::Borrowed(iter) => iter.next().map(Cow::Borrowed),
            CowIter::Owned(iter) => iter.next().map(Cow::Owned),
        }
    }
}
