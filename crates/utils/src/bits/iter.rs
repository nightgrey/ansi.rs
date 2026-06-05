use super::*;
/// Yields the individual flags present in a [`Bits`].
///
/// Walks [`Bit::LIST`] in order. A flag is yielded when all of its bits are in
/// the source set *and* it still covers bits no earlier flag has claimed — so
/// overlapping flags both appear, while a convenience alias whose bits are
/// fully covered by already-yielded flags does not.
#[derive(Copy, Clone)]
pub struct BitsIter<S: Bits> {
    source: S,
    remaining: S,
    idx: usize,
}

impl<S: [ const ] Bits> const BitsIter<S> {
    #[inline]
    pub fn new(source: impl [ const ] Into<S>) -> Self {
        let bits = source.into();
        Self { source: bits, remaining: bits, idx: 0 }
    }

    #[inline]
    pub fn with_remaining(mut self, remaining: impl [ const ] Into<S>) -> Self {
        self.remaining = remaining.into();
        self
    }
}

impl<Bs: [ const ] Bits> const Iterator for BitsIter<Bs> {
    type Item = Bs::Bit;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < <Bs::Bit as Bit>::COUNT {
            let (bit, _) = <Bs::Bit as Bit>::LIST[self.idx];

            let bits = Bs::from_repr(bit.into_repr());
            if self.source.contains(bits) && self.remaining.intersects(bits) {
                self.remaining.remove(bits);
                return Some(bit);
            }
            if self.remaining.is_none() {
                self.idx = <Bs::Bit as Bit>::COUNT;
                return None;
            }
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // Upper bound only: the exact count depends on the overlap rules above.
        (0, Some(<Bs::Bit as Bit>::COUNT - self.idx))
    }
}

impl<S: Bits> ExactSizeIterator for BitsIter<S> {
    #[inline]
    fn len(&self) -> usize {
        // Drain a copy — correct under overlapping/alias flags, and COUNT is tiny.
        let mut probe = *self;
        let mut n = 0;
        while probe.next().is_some() {
            n += 1;
        }
        n
    }
}
