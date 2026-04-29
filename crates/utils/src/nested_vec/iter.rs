use crate::NestedSlice;

/// Iterator over groups of a `Params`.
pub struct NestedIter<'a, T> {
    starts: &'a [usize],
    inner: &'a [T],
    pos: usize,
}

impl<'a, T> NestedIter<'a, T>{
    pub const fn new(params: &NestedSlice<'a, T>) -> Self {
        Self { starts: params.starts, inner: params.inner, pos: 0 }
    }
    pub const fn from_parts(starts: &'a [usize], inner: &'a [T]) -> Self {
        Self { starts, inner, pos: 0 }
    }
}

impl<'a, T> Iterator for NestedIter<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.starts.len() {
            return None;
        }
        let item = unsafe { self.inner.get_unchecked(self.starts[self.pos]..self.starts[self.pos + 1]) };
        self.pos += 1;
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.starts.len().saturating_sub(self.pos);
        (remaining, Some(remaining))
    }
}

impl<'a, T> ExactSizeIterator for NestedIter<'a, T> {}
