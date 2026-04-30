use crate::NestedSlice;

/// Iterator over groups of a `Params`.
#[derive(Debug, Clone)]
pub struct NestedIter<'a, T> {
    starts: &'a [usize],
    inner: &'a [T],
    i: usize,
    len: usize,
}

impl<'a, T> NestedIter<'a, T>{

    pub const fn from_parts(starts: &'a [usize], inner: &'a [T], start: usize, end: usize) -> Self {
        Self { starts, inner, i: start, len: end }
    }
}

impl<'a, T> Iterator for NestedIter<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.i;
        
        if i >= self.len {
            return None;
        }
        let start = self.starts[i];
        let end = self.starts[i + 1];
        let item = unsafe { self.inner.get_unchecked(start..end) };
        self.i += 1;
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.starts.len().saturating_sub(self.i);
        (remaining, Some(remaining))
    }
}

impl<'a, T> ExactSizeIterator for NestedIter<'a, T> {}
