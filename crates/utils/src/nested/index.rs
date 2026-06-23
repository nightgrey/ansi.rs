use crate::{Nested, NestedMut, NestedSlice};
use std::ops;

pub trait NestedIndex<T> {
    type Output<'a>
    where
        T: 'a;

    /// Returns a shared reference to the output at this location, if in
    /// bounds.
    fn get(self, nested: &(impl Nested<T> + ?Sized)) -> Option<Self::Output<'_>>;

    /// Returns a pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    unsafe fn get_unchecked(self, nested: &(impl Nested<T> + ?Sized)) -> Self::Output<'_>;

    /// Returns a shared reference to the output at this location, panicking
    /// if out of bounds.
    #[track_caller]
    fn index(self, nested: &(impl Nested<T> + ?Sized)) -> Self::Output<'_>;

    /// Returns the length of the output at this location, if in bounds.
    fn get_len(self, nested: &(impl Nested<T> + ?Sized)) -> Option<usize>;
}

pub trait NestedIndexMut<T> {
    type Output<'a>
    where
        T: 'a;

    /// Returns a mutable reference to the output at this location, if in bounds.
    fn get_mut(self, nested: &mut (impl NestedMut<T> + ?Sized)) -> Option<Self::Output<'_>>;

    /// Returns a mutable pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    unsafe fn get_unchecked_mut(
        self,
        nested: &mut (impl NestedMut<T> + ?Sized),
    ) -> Self::Output<'_>;

    /// Returns a mutable reference to the output at this location, panicking
    /// if out of bounds.
    #[track_caller]
    fn index_mut(self, nested: &mut (impl NestedMut<T> + ?Sized)) -> Self::Output<'_>;
}

impl<T> NestedIndex<T> for usize {
    type Output<'a>
        = &'a [T]
    where
        T: 'a;

    fn get(self, nested: &(impl Nested<T> + ?Sized)) -> Option<Self::Output<'_>> {
        if self >= nested.len() {
            return None;
        }
        Some(unsafe { NestedIndex::get_unchecked(self, nested) })
    }

    unsafe fn get_unchecked(self, nested: &(impl Nested<T> + ?Sized)) -> Self::Output<'_> {
        unsafe {
            let starts = nested.starts();
            let start = starts.get_unchecked(self);
            let end = starts.get_unchecked(self + 1);
            nested.values().get_unchecked(*start..*end)
        }
    }

    #[track_caller]
    fn index(self, nested: &(impl Nested<T> + ?Sized)) -> Self::Output<'_> {
        if self < nested.len() {
            let starts = nested.starts();
            unsafe {
                let start = starts.get_unchecked(self);
                let end = starts.get_unchecked(self + 1);
                &nested.values()[*start..*end]
            }
        } else {
            nested_index_fail(self, self + 1, nested.len())
        }
    }

    fn get_len(self, nested: &(impl Nested<T> + ?Sized)) -> Option<usize> {
        if self < nested.len() {
            let starts = nested.starts();
            unsafe {
                let start = *starts.get_unchecked(self);
                let end = *starts.get_unchecked(self + 1);
                Some(end - start)
            }
        } else {
            nested_index_fail(self, self + 1, nested.len())
        }
    }
}
impl<T> NestedIndex<T> for ops::Range<usize> {
    type Output<'a>
        = NestedSlice<'a, T>
    where
        T: 'a;

    fn get(self, nested: &(impl Nested<T> + ?Sized)) -> Option<Self::Output<'_>> {
        if self.end <= nested.len() {
            let starts = nested.starts();
            let inner = nested.values();
            // need range.end+1 starts to bound the last group
            unsafe {
                let starts = starts.get_unchecked(self.start..=(self.end));
                let inner = inner.get_unchecked(
                    starts.first().copied().unwrap()..starts.last().copied().unwrap(),
                );
                Some(NestedSlice::from_raw(inner, starts))
            }
        } else {
            None
        }
    }

    unsafe fn get_unchecked(self, nested: &(impl Nested<T> + ?Sized)) -> Self::Output<'_> {
        let starts = nested.starts();
        let inner = nested.values();
        let starts = starts.get_unchecked(self.start..=self.end);
        let inner =
            inner.get_unchecked(starts.first().copied().unwrap()..starts.last().copied().unwrap());
        NestedSlice::from_raw(inner, starts)
    }

    fn index(self, nested: &(impl Nested<T> + ?Sized)) -> Self::Output<'_> {
        if self.end <= nested.len() {
            unsafe {
                let starts = nested.starts().get_unchecked(self.start..=self.end);
                let inner = nested.values().get_unchecked(
                    starts.first().copied().unwrap()..starts.last().copied().unwrap(),
                );

                NestedSlice::from_raw(inner, starts)
            }
        } else {
            nested_index_fail(self.start, self.end, nested.len())
        }
    }

    fn get_len(self, nested: &(impl Nested<T> + ?Sized)) -> Option<usize> {
        if self.end <= nested.len() {
            let starts = nested.starts();

            unsafe {
                let start = starts.get_unchecked(self.start);
                let end = starts.get_unchecked(self.end);
                Some(end - start)
            }
        } else {
            None
        }
    }
}
impl<T> NestedIndex<T> for ops::RangeInclusive<usize> {
    type Output<'a>
        = NestedSlice<'a, T>
    where
        T: 'a;

    fn get(self, nested: &(impl Nested<T> + ?Sized)) -> Option<Self::Output<'_>> {
        if *self.end() <= nested.len() {
            let starts = nested.starts();
            let values = nested.values();

            unsafe {
                // need range.end+1 starts to bound the last group
                let starts = starts.get_unchecked(*self.start()..=*self.end());
                let inner = values.get_unchecked(
                    starts.first().copied().unwrap()..starts.last().copied().unwrap(),
                );
                Some(NestedSlice::from_raw(inner, starts))
            }
        } else {
            None
        }
    }

    unsafe fn get_unchecked(self, nested: &(impl Nested<T> + ?Sized)) -> Self::Output<'_> {
        unsafe {
            let starts = nested.starts();
            let values = nested.values();

            let starts = starts.get_unchecked(*self.start()..=*self.end());
            let inner = values
                .get_unchecked(starts.first().copied().unwrap()..starts.last().copied().unwrap());
            NestedSlice::from_raw(inner, starts)
        }
    }

    fn index(self, nested: &(impl Nested<T> + ?Sized)) -> Self::Output<'_> {
        if *self.end() <= nested.len() {
            let starts = nested.starts();
            let values = nested.values();
            unsafe {
                let starts = starts.get_unchecked(*self.start()..=*self.end());
                let inner = values.get_unchecked(
                    starts.first().copied().unwrap()..starts.last().copied().unwrap(),
                );

                NestedSlice::from_raw(inner, starts)
            }
        } else {
            nested_index_fail(*self.start(), *self.end(), nested.len())
        }
    }

    fn get_len(self, nested: &(impl Nested<T> + ?Sized)) -> Option<usize> {
        if *self.end() <= nested.len() {
            let starts = nested.starts();
            let _values = nested.values();

            unsafe {
                // need range.end+1 starts to bound the last group
                let start = starts.get_unchecked(*self.start());
                let end = starts.get_unchecked(*self.end());
                Some(end - start)
            }
        } else {
            None
        }
    }
}

impl<T> NestedIndexMut<T> for usize {
    type Output<'a>
        = &'a mut [T]
    where
        T: 'a;

    fn get_mut(self, nested: &mut (impl NestedMut<T> + ?Sized)) -> Option<Self::Output<'_>> {
        if self >= nested.len() {
            return None;
        }
        let starts = nested.starts_mut();
        // indices[self] and indices[self+1] are guaranteed valid because self < len
        let range = starts[self]..starts[self + 1];
        nested.values_mut().get_mut(range)
    }

    unsafe fn get_unchecked_mut(
        self,
        nested: &mut (impl NestedMut<T> + ?Sized),
    ) -> Self::Output<'_> {
        let starts = nested.starts_mut();
        let range = starts[self]..starts[self + 1];
        nested.values_mut().get_unchecked_mut(range)
    }

    #[track_caller]
    fn index_mut(self, nested: &mut (impl NestedMut<T> + ?Sized)) -> Self::Output<'_> {
        if self < nested.len() {
            let starts = nested.starts_mut();
            let range = starts[self]..starts[self + 1];
            &mut nested.values_mut()[range]
        } else {
            nested_index_fail(self, self + 1, nested.len())
        }
    }
}

// TODO: ops::Range
#[cfg_attr(not(panic = "immediate-abort"), inline(never), cold)]
#[cfg_attr(panic = "immediate-abort", inline)]
#[track_caller]
fn nested_index_fail(start: usize, end: usize, len: usize) -> ! {
    if start > len {
        panic!("range start index {start} out of range for nested slice of length {len}",)
    }

    if end > len {
        panic!("range end index {end} out of range for nested slice of length {len}",)
    }

    if start > end {
        panic!("nested index starts at {start} but ends at {end}",)
    }

    // Only reachable if the range was a `RangeInclusive` or a
    // `RangeToInclusive`, with `end == len`.
    panic!("range end index {end} out of range for nested slice of length {len}",)
}
