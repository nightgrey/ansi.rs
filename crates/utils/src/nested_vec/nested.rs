use crate::{NestedError, NestedIter, NestedSlice, NestedVec};
use std::ops;
use std::ops::{Index, IndexMut};

pub trait Nested<T>: AsRef<[T]> + Index<usize, Output = [T]> {
    fn get<I: NestedIndex<T>>(&self, index: I) -> Option<I::Output<'_>> {
        index.get(self)
    }
    unsafe fn get_unchecked<I: NestedIndex<T>>(&self, index: I) -> I::Output<'_> {
        index.get_unchecked(self)
    }

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;

    fn iter(&self) -> NestedIter<'_, T>;
    fn iter_flat(&self) -> std::slice::Iter<'_, T>;

    fn first(&self) -> Option<&[T]>;
    fn last(&self) -> Option<&[T]>;

    fn as_slice(&self) -> &[T];
    fn as_slices(&self) -> (&[T], &[usize]);

    fn as_ptr(&self) -> *const T;
    fn as_ptrs(&self) -> (*const T, *const usize);

    fn as_nested_slice(&self) -> NestedSlice<'_, T>;

    fn to_nested_vec<const N: usize, const M: usize>(&self) -> NestedVec<T, N, M>
    where
        T: Clone;
}

pub trait NestedMut<T>: Nested<T> + IndexMut<usize, Output = [T]> + Extend<T>  {
    fn get_mut<I: NestedIndexMut<T>>(&mut self, index: I) -> Option<I::Output<'_>> {
        index.get_mut(self)
    }
    unsafe fn get_unchecked_mut<I: NestedIndexMut<T>>(&mut self, index: I) -> I::Output<'_> {
        index.get_unchecked_mut(self)
    }

    fn push(&mut self, items: impl IntoIterator<Item = T>);
    fn push_one(&mut self, val: T);

    // NOTE: Not needed yet. Maybe later.
    // fn iter_mut(&mut self) -> NestedIterMut<'_, T>;
    // fn iter_flat_mut(&mut self) -> std::slice::IterMut<'_, T>;
    //
    // fn as_nested_mut(&mut self) -> NestedSliceMut<'_, T>;

    fn as_mut_slice(&mut self) -> &mut [T];
    fn as_mut_slices(&mut self) -> (&mut [T], &mut [usize]);

    fn as_mut_ptr(&mut self) -> *mut T;
    fn as_ptrs(&mut self) -> (*mut T, *mut usize);

    // Not needed yet. Maybe later.
    // fn as_mut_nested_slice(&mut self) -> NestedSliceMut<'_, T>;

    fn clear(&mut self);
}

pub trait TryNestedMut<T>: NestedMut<T> {
    fn try_push(&mut self, items: impl IntoIterator<Item = T>) -> Result<(), NestedError>;
    fn try_push_one(&mut self, val: T) -> Result<(), NestedError>;

    fn try_extend<I: IntoIterator<Item = T>>(&mut self, items: I) -> Result<(), NestedError>;
    fn try_extend_one(&mut self, val: T) -> Result<(), NestedError>;
}

pub trait NestedConstructor<T>: Default {
    fn new() -> Self;
}

pub trait NestedFromIterator<T, Group: IntoIterator<Item = T>>: Nested<T> + FromIterator<Group> {}

pub trait NestedIndex<T> {
    type Output<'a>
    where
        Self: 'a,
        T: 'a;

    /// Returns a shared reference to the output at this location, if in
    /// bounds.
    fn get<N: Nested<T> + ?Sized>(self, nested: &N) -> Option<Self::Output<'_>>;

    /// Returns a pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    unsafe fn get_unchecked<N: Nested<T> + ?Sized>(self, nested: &N) -> Self::Output<'_>;

    /// Returns a shared reference to the output at this location, panicking
    /// if out of bounds.
    #[track_caller]
    fn index<N: Nested<T> + ?Sized>(self, nested: &N) -> Self::Output<'_>;
}

impl<T> NestedIndex<T> for usize {
    type Output<'a>
        = &'a [T]
    where
        T: 'a;

    fn get<N: Nested<T> + ?Sized>(self, nested: &N) -> Option<Self::Output<'_>> {
        if self >= nested.len() {
            return None;
        }
        Some(&*unsafe { nested.get_unchecked(self) })
    }

    unsafe fn get_unchecked<N: Nested<T> + ?Sized>(self, nested: &N) -> Self::Output<'_> {
        let (values, indices) = (nested).as_slices();

        let starts = indices.get_unchecked((indices[self]..=indices[self]));
        values.get_unchecked(starts.first().copied().unwrap()..starts.last().copied().unwrap())
    }

    #[track_caller]
    fn index<N: Nested<T> + ?Sized>(self, nested: &N) -> Self::Output<'_> {
        if self < nested.len() {
            let (values, indices) = (*nested).as_slices();
            &values[indices[self]..=indices[self]]
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

    fn get<N: Nested<T> + ?Sized>(self, nested: &N) -> Option<Self::Output<'_>> {
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

    unsafe fn get_unchecked<N: Nested<T> + ?Sized>(self, nested: &N) -> Self::Output<'_> {
        let (values, indices) = (*nested).as_slices();

        let starts = indices.get_unchecked((self.start..=self.end));
        let inner = (values)
            .get_unchecked(starts.first().copied().unwrap()..starts.last().copied().unwrap());
        unsafe { NestedSlice::from_parts(inner, starts) }
    }

    fn index<N: Nested<T> + ?Sized>(self, nested: &N) -> Self::Output<'_> {
        if self.end <= nested.len() {
            let (values, indices) = (nested).as_slices();

            unsafe {
                let starts = (indices).get_unchecked(self.start..=self.end);
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
    type Output<'a>
        = NestedSlice<'a, T>
    where
        T: 'a;

    fn get<N: Nested<T> + ?Sized>(self, nested: &N) -> Option<Self::Output<'_>> {
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

    unsafe fn get_unchecked<N: Nested<T> + ?Sized>(self, nested: &N) -> Self::Output<'_> {
        let (values, indices) = (*nested).as_slices();

        let starts = indices.get_unchecked((*(self.start())..=((*self.end()) + 1)));
        let inner = (values)
            .get_unchecked(starts.first().copied().unwrap()..starts.last().copied().unwrap());
        unsafe { NestedSlice::from_parts(inner, starts) }
    }

    fn index<N: Nested<T> + ?Sized>(self, nested: &N) -> Self::Output<'_> {
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
/// Trait for mutable indexing into a `Nested<T>` container.
pub trait NestedIndexMut<T> {
    type Output<'a>
    where
        Self: 'a,
        T: 'a;

    /// Returns a mutable reference to the output at this location, if in bounds.
    fn get_mut<N: NestedMut<T> + ?Sized>(self, nested: &mut N) -> Option<Self::Output<'_>>;

    /// Returns a mutable pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    unsafe fn get_unchecked_mut<N: NestedMut<T> + ?Sized>(self, nested: &mut N) -> Self::Output<'_>;

    /// Returns a mutable reference to the output at this location, panicking
    /// if out of bounds.
    #[track_caller]
    fn index_mut<N: NestedMut<T> + ?Sized>(self, nested: &mut N) -> Self::Output<'_>;
}

// ---------------------------------------------------------------------------
// Implementation for `usize` – returns a mutable slice for a single inner list
// ---------------------------------------------------------------------------

impl<T> NestedIndexMut<T> for usize {
    type Output<'a>
    = &'a mut [T]
    where
        T: 'a;

    fn get_mut<N: NestedMut<T> + ?Sized>(self, nested: &mut N) -> Option<Self::Output<'_>> {
        if self >= nested.len() {
            return None;
        }
        let (values, indices) = nested.as_mut_slices();
        // indices[self] and indices[self+1] are guaranteed valid because self < len
        let range = indices[self]..=indices[self];
        values.get_mut(range)
    }

    unsafe fn get_unchecked_mut<N: NestedMut<T> + ?Sized>(self, nested: &mut N) -> Self::Output<'_> {
        let (values, indices) = nested.as_mut_slices();
        let range = indices[self]..=indices[self];
        values.get_unchecked_mut(range)
    }

    #[track_caller]
    fn index_mut<N: NestedMut   <T> + ?Sized>(self, nested: &mut N) -> Self::Output<'_> {
        if self < nested.len() {
            let (values, indices) = nested.as_mut_slices();
            let range = indices[self]..=indices[self];
            &mut values[range]
        } else {
            nested_index_fail(self, self + 1, nested.len())
        }
    }
}
// impl<T> NestedIndexMut<T> for ops::Range<usize> {
//     type Output<'a>
//     = NestedSliceMut<'a, T>
//     where
//         T: 'a;
//
//     fn get_mut<N: Nested<T> + ?Sized>(self, nested: &mut N) -> Option<Self::Output<'_>> {
//         if self.end <= nested.len() {
//             let (values, indices) = nested.as_mut_slices();
//
//             // The sub‑slice of start offsets for the selected groups.
//             // We need indices up to self.end+1 to correctly bound the last group.
//             let starts = indices.get(self.start..=self.end)?;
//
//             // The inner mutable slice of all values spanned by these groups.
//             let inner = values.get_mut(
//                 *starts.first()?..*starts.last()?,
//             )?;
//
//             Some(unsafe { NestedSliceMut::from_parts(inner, starts) })
//         } else {
//             None
//         }
//     }
//
//     unsafe fn get_unchecked_mut<N: Nested<T> + ?Sized>(self, nested: &mut N) -> Self::Output<'_> {
//         let (values, indices) = nested.as_mut_slices();
//
//         let starts = indices.get_unchecked(self.start..=self.end);
//         let inner = values.get_unchecked_mut(
//             *starts.first().unwrap()..*starts.last().unwrap(),
//         );
//
//         unsafe { NestedSliceMut::from_parts(inner, starts) }
//     }
//
//     #[track_caller]
//     fn index_mut<N: Nested<T> + ?Sized>(self, nested: &mut N) -> Self::Output<'_> {
//         if self.end <= nested.len() {
//             let (values, indices) = nested.as_mut_slices();
//
//             unsafe {
//                 let starts = indices.get_unchecked(self.start..=self.end);
//                 let inner = values.get_unchecked_mut(
//                     *starts.first().unwrap()..*starts.last().unwrap(),
//                 );
//
//                 NestedSliceMut::from_parts(inner, starts)
//             }
//         } else {
//             nested_index_fail_range(self.start, self.end, nested.len())
//         }
//     }
// }
// impl<T> NestedIndexMut<T> for ops::RangeInclusive<usize> {
//     type Output<'a>
//     = NestedSliceMut<'a, T>
//     where
//         T: 'a;
//
//     fn get_mut<N: Nested<T> + ?Sized>(self, nested: &mut N) -> Option<Self::Output<'_>> {
//         if *self.end() <= nested.len() {
//             let (values, indices) = nested.as_mut_slices();
//
//             // Need `end + 1` to capture the start of the group after the last one.
//             let starts = indices.get(*self.start()..=*self.end() + 1)?;
//
//             let inner = values.get_mut(
//                 *starts.first()?..*starts.last()?,
//             )?;
//
//             Some(unsafe { NestedSliceMut::from_parts(inner, starts) })
//         } else {
//             None
//         }
//     }
//
//     unsafe fn get_unchecked_mut<N: Nested<T> + ?Sized>(self, nested: &mut N) -> Self::Output<'_> {
//         let (values, indices) = nested.as_mut_slices();
//
//         let starts = indices.get_unchecked(*self.start()..=*self.end() + 1);
//         let inner = values.get_unchecked_mut(
//             *starts.first().unwrap()..*starts.last().unwrap(),
//         );
//
//         unsafe { NestedSliceMut::from_parts(inner, starts) }
//     }
//
//     #[track_caller]
//     fn index_mut<N: Nested<T> + ?Sized>(self, nested: &mut N) -> Self::Output<'_> {
//         if *self.end() <= nested.len() {
//             let (values, indices) = nested.as_mut_slices();
//
//             unsafe {
//                 let starts = indices.get_unchecked(*self.start()..=*self.end() + 1);
//                 let inner = values.get_unchecked_mut(
//                     *starts.first().unwrap()..*starts.last().unwrap(),
//                 );
//
//                 NestedSliceMut::from_parts(inner, starts)
//             }
//         } else {
//             nested_index_fail_range(*self.start(), *self.end(), nested.len())
//         }
//     }
// }
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
