use std::ops;
use crate::{Nested, NestedMut, NestedSlice};

pub trait NestedIndex<T> {
    type Output<'a> where T: 'a;

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
}

impl<T> NestedIndex<T> for usize {
    type Output<'a> = &'a [T] where T: 'a;

    fn get(self, nested: &(impl Nested<T> + ?Sized)) -> Option<Self::Output<'_>> {
        if self >= nested.len() {
            return None;
        }
        Some(&*unsafe { nested.get_unchecked(self) })
    }

    unsafe fn get_unchecked(self, nested: &(impl Nested<T> + ?Sized)) -> Self::Output<'_> {
        let (values, indices) = nested.as_slices();

        let starts = indices.get_unchecked((indices[self]..=indices[self + 1]));
        values.get_unchecked(starts.first().copied().unwrap()..starts.last().copied().unwrap())
    }

    #[track_caller]
    fn index(self, nested: &(impl Nested<T> + ?Sized)) -> Self::Output<'_> {
        if self < nested.len() {
            let (values, indices) = (*nested).as_slices();
            &values[indices[self]..=indices[self + 1]]
        } else {
            nested_index_fail(self, self + 1, nested.len())
        }
    }
}
impl<T> NestedIndex<T> for ops::Range<usize> {
    type Output<'a> = NestedSlice<'a, T> where T: 'a;

    fn get(self, nested: &(impl Nested<T> + ?Sized)) -> Option<Self::Output<'_>> {
        if self.end <= nested.len() {
            let (values, indices) = nested.as_slices();
            // need range.end+1 starts to bound the last group
            let starts = indices.get(self.start..=(self.end + 1))?;
            let inner = &values.get(starts.first().copied()?..starts.last().copied()?)?;
            Some(unsafe { NestedSlice::from_parts(inner, starts) })
        } else {
            None
        }
    }

    unsafe fn get_unchecked(self, nested: &(impl Nested<T> + ?Sized)) -> Self::Output<'_> {
        let (values, indices) = (nested).as_slices();

        let starts = indices.get_unchecked((self.start..=(self.end + 1)));
        let inner = (values)
            .get_unchecked(starts.first().copied().unwrap()..starts.last().copied().unwrap());
        unsafe { NestedSlice::from_parts(inner, starts) }
    }

    fn index(self, nested: &(impl Nested<T> + ?Sized)) -> Self::Output<'_> {
        if self.end <= nested.len() {
            let (values, indices) = (nested).as_slices();

            unsafe {
                let starts = (indices).get_unchecked(self.start..=(self.end + 1));
                let inner = (values).get_unchecked(
                    starts.first().copied().unwrap()..starts.last().copied().unwrap(),
                );

                NestedSlice::from_parts(inner, starts)
            }
        } else {
            nested_index_fail(self.start, self.end, nested.len())
        }
    }
}
impl<T> NestedIndex<T> for ops::RangeInclusive<usize> {
    type Output<'a> = NestedSlice<'a, T> where T: 'a;

    fn get(self, nested: &(impl Nested<T> + ?Sized)) -> Option<Self::Output<'_>> {
        if *self.end() <= nested.len() {
            let (values, indices) = nested.as_slices();
            // need range.end+1 starts to bound the last group
            let starts = indices.get(*self.start()..=*self.end() + 1)?;
            let inner = &values.get(starts.first().copied()?..starts.last().copied()?)?;
            Some(unsafe { NestedSlice::from_parts(inner, starts) })
        } else {
            None
        }
    }

    unsafe fn get_unchecked(self, nested: &(impl Nested<T> + ?Sized)) -> Self::Output<'_> {
        let (values, indices) = (nested).as_slices();

        let starts = indices.get_unchecked((*(self.start())..=((*self.end()) + 1)));
        let inner = (values)
            .get_unchecked(starts.first().copied().unwrap()..starts.last().copied().unwrap());
        unsafe { NestedSlice::from_parts(inner, starts) }
    }

    fn index(self, nested: &(impl Nested<T> + ?Sized)) -> Self::Output<'_>  {
        if *self.end() <= nested.len() {
            let (values, indices) = (nested).as_slices();

            unsafe {
                let starts = (indices.get_unchecked((*(self.start())..=(*self.end() + 1))));
                let inner = (values).get_unchecked(
                    starts.first().copied().unwrap()..starts.last().copied().unwrap(),
                );

                NestedSlice::from_parts(inner, starts)
            }
        } else {
            nested_index_fail(*self.start(), *self.end(), nested.len())
        }
    }
}

pub trait NestedIndexMut<T> {
    type Output<'a> where T: 'a;

    /// Returns a mutable reference to the output at this location, if in bounds.
    fn get_mut(self, nested: &mut (impl NestedMut<T> + ?Sized)) -> Option<Self::Output<'_>>;

    /// Returns a mutable pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    unsafe fn get_unchecked_mut(self, nested: &mut (impl NestedMut<T> + ?Sized)) -> Self::Output<'_>;

    /// Returns a mutable reference to the output at this location, panicking
    /// if out of bounds.
    #[track_caller]
    fn index_mut(self, nested: &mut (impl NestedMut<T> + ?Sized)) -> Self::Output<'_>;
}

impl<T> NestedIndexMut<T> for usize {
    type Output<'a> = &'a mut [T] where T: 'a;

    fn get_mut(self, nested: &mut (impl NestedMut<T> + ?Sized)) -> Option<Self::Output<'_>> {
        if self >= nested.len() {
            return None;
        }
        let (values, indices) = nested.as_mut_slices();
        // indices[self] and indices[self+1] are guaranteed valid because self < len
        let range = indices[self]..=indices[self + 1];
        values.get_mut(range)
    }

    unsafe fn get_unchecked_mut(self, nested: &mut (impl NestedMut<T> + ?Sized)) -> Self::Output<'_> {
        let (values, indices) = nested.as_mut_slices();
        let range = indices[self]..=indices[self];
        values.get_unchecked_mut(range)
    }

    #[track_caller]
    fn index_mut(self, nested: &mut (impl NestedMut<T> + ?Sized)) -> Self::Output<'_> {
        if self < nested.len() {
            let (values, indices) = nested.as_mut_slices();
            let range = indices[self]..=indices[self + 1];
            &mut values[range]
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

