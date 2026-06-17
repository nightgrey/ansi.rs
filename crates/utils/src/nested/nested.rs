//! Traits for nested arrays
//!
//! A "nested" collection is a flattened storage of elements `T` together with an
//! array of start indices that define sub‑slices. This module defines three levels
//! of capability: read‑only access (`Nested`), mutable access (`NestedMut`), and
//! fallible mutation (`TryNestedMut`). A convenience constructor trait
//! (`NestedConstructor`) is also provided.

use crate::{NestedError, NestedIndex, NestedIndexMut, NestedIter, NestedSlice, NestedVec};
use smallvec::SmallVec;
use std::ops::{Index, IndexMut};
/// A trait representing a **nested** (jagged) sequence of values.
///
/// A nested structure is a sequence of *groups*, each group being a contiguous slice
/// of elements of type `T`. Internally, the elements are stored in a flat array (`values`),
/// and each group is defined by its starting index in that array (`starts`).
/// This layout enables efficient storage and indexed access.
///
/// # Naming conventions
///
/// - **Group**: a single contiguous slice of elements (inner array).
/// - **Element**: a single value of type `T` within a group.
///
/// | Method         | Meaning                                                                |
/// |----------------|------------------------------------------------------------------------|
/// | `push`         | Add a **new group** with the given items.                              |
/// | `push_one`     | Add a **new group** consisting of a single element.                    |
/// | `extend`       | Extend the **last group** with items; creates an empty group if needed.|
/// | `extend_one`   | Extend the **last group** with a single element; creates an empty group if needed. |
/// | `len_of`       | Returns the number of elements in a given group.                       |
///
/// # Implementors
///
/// - [`NestedVec`](crate::NestedVec) – an owned, growable nested vector.
/// - [`NestedSlice`](crate::NestedSlice) – a read-only borrowed nested view.
/// - [`NestedSliceMut`](crate::NestedSliceMut) – a mutable borrowed nested view.
/// - [`NestedIter`](crate::NestedIter) – an iterator over the groups.
///
/// # Indexing
///
/// The trait provides indexing via [`Index<usize>`] returning `&[T]` (the group at that index).
/// For more flexible indexing (e.g. by a pair of indices), see [`NestedIndex`].
pub trait Nested<T>: AsRef<[T]> + Index<usize, Output = [T]> {
    /// Returns a reference to the elements of the group specified by `index`.
    ///
    /// Returns `None` if the index is out of bounds.
    fn get<I: NestedIndex<T>>(&self, index: I) -> Option<I::Output<'_>> {
        index.get(self)
    }

    /// Returns a reference to the elements of the group specified by `index` without
    /// bounds checking.
    ///
    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is **undefined behavior**.
    /// The caller must ensure that `index` is valid.
    unsafe fn get_unchecked<I: NestedIndex<T>>(&self, index: I) -> I::Output<'_> {
        index.get_unchecked(self)
    }

    /// Returns the number of elements in the group at the given `index`.
    ///
    /// Returns `None` if the index is out of bounds.
    fn get_len<I: NestedIndex<T>>(&self, index: I) -> Option<usize> {
        index.get_len(self)
    }

    /// Returns a reference to the first group (slice), or `None` if empty.
    fn first(&self) -> Option<&[T]>;

    /// Returns a reference to the last group (slice), or `None` if empty.
    fn last(&self) -> Option<&[T]>;

    /// Returns the total number of groups.
    fn len(&self) -> usize;

    /// Returns `true` if there are no groups.
    fn is_empty(&self) -> bool;

    /// Creates an iterator over all groups.
    fn iter(&self) -> NestedIter<'_, T> {
        NestedIter::from_parts(self.starts(), self.values(), 0, self.len())
    }

    /// Returns the flat array of all element values (concatenated group contents).

    fn values(&self) -> &[T];

    /// Returns the slice of start indices for each group.
    ///
    /// The length of this slice equals the number of groups.
    /// The start indices refer to positions in `values()`.

    fn starts(&self) -> &[usize];

    /// Returns a slice of all element values.
    #[inline]
    fn as_slice(&self) -> &[T] {
        self.values()
    }

    /// Returns a raw pointer to the flat element array.
    #[inline]
    fn as_ptr(&self) -> *const T {
        self.values().as_ptr()
    }

    /// Converts this nested structure into a borrowed [`NestedSlice`].
    fn as_nested_slice(&self) -> NestedSlice<'_, T> {
        NestedSlice::from_parts(self.values(), self.starts())
    }

    /// Creates a new [`NestedVec<T, N, M>`] by cloning all values and start indices.
    ///
    /// The constants `N` and `M` define the inline capacities for values and start indices,
    /// respectively (see [`NestedVec`] for details).
    #[inline]
    fn to_nested_vec<const N: usize, const M: usize>(&self) -> NestedVec<T, N, M>
    where
        T: Clone,
    {
        NestedVec {
            inner: SmallVec::from(self.values()),
            starts: SmallVec::from(self.starts()),
        }
    }
}

/// A trait for **mutable** access to a nested sequence.
///
/// Extends [`Nested<T>`] with methods that allow modification of groups and elements.
///
/// # Mutable indexing
///
/// The trait provides indexing via [`NestedIndexMut`].
/// # Safety
///
/// Implementations must ensure that the `values` and `starts` arrays remain consistent
/// (i.e., `starts` is sorted and all indices are within the bounds of `values`).
///
/// # Implementors
///
/// - [`NestedVec`](crate::NestedVec)
/// - [`NestedSliceMut`](crate::NestedSliceMut)
pub trait NestedMut<T>: Nested<T> + IndexMut<usize, Output = [T]> {
    /// Returns a mutable reference to the elements of the group specified by `index`.
    ///
    /// Returns `None` if the index is out of bounds.
    fn get_mut<I: NestedIndexMut<T>>(&mut self, index: I) -> Option<I::Output<'_>> {
        index.get_mut(self)
    }

    /// Returns a mutable reference to the elements of the group specified by `index`
    /// without bounds checking.
    ///
    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is **undefined behavior**.
    unsafe fn get_unchecked_mut<I: NestedIndexMut<T>>(&mut self, index: I) -> I::Output<'_> {
        index.get_unchecked_mut(self)
    }

    /// Appends a **new group** containing the given items to the end of the nested structure.
    ///
    /// # Examples
    ///
    /// ```
    /// use nested::NestedVec;
    /// let mut vec = NestedVec::<i32, 4, 2>::new();
    /// vec.push([1, 2, 3]);
    /// vec.push(vec![4, 5]);
    /// assert_eq!(vec.len(), 2);
    /// assert_eq!(&vec[0], &[1, 2, 3]);
    /// assert_eq!(&vec[1], &[4, 5]);
    /// ```
    fn push(&mut self, items: impl IntoIterator<Item = T>);

    /// Appends a **new group** containing a single element.
    ///
    /// Equivalent to `push(std::iter::once(val))`.
    fn push_one(&mut self, val: T);

    /// Extends the **last group** with the given items.
    ///
    /// If the nested structure is empty, a new (empty) group is implicitly created first,
    /// and then the items are added to it.
    ///
    /// # Examples
    ///
    /// ```
    /// use nested::NestedVec;
    /// let mut vec = NestedVec::<i32, 10, 2>::new();
    /// vec.extend([1, 2]);   // creates a new group [1, 2]
    /// vec.extend([3, 4]);   // extends the last group to [1, 2, 3, 4]
    /// assert_eq!(&vec[0], &[1, 2, 3, 4]);
    /// ```
    fn extend(&mut self, items: impl IntoIterator<Item = T>);

    /// Extends the **last group** with a single element.
    ///
    /// If the nested structure is empty, a new group is implicitly created.
    /// Equivalent to `extend(std::iter::once(value))`.
    fn extend_one(&mut self, value: T);

    // NOTE: Not needed yet. Maybe later.
    // fn iter_mut(&mut self) -> NestedIterMut<'_, T>;
    // fn iter_flat_mut(&mut self) -> std::slice::IterMut<'_, T>;
    //
    // fn as_nested_mut(&mut self) -> NestedSliceMut<'_, T>;

    /// Returns a mutable reference to the flat array of all element values.
    fn values_mut(&mut self) -> &mut [T];

    /// Returns a mutable reference to the slice of start indices.
    fn starts_mut(&mut self) -> &mut [usize];

    /// Returns a mutable slice of all element values.
    #[inline]
    fn as_mut_slice(&mut self) -> &mut [T] {
        self.values_mut()
    }

    /// Returns a raw mutable pointer to the flat element array.
    #[inline]
    fn as_mut_ptr(&mut self) -> *mut T {
        self.values_mut().as_mut_ptr()
    }

    // Not needed yet. Maybe later.
    // fn as_mut_nested_slice(&mut self) -> NestedSliceMut<'_, T>;

    /// Removes all groups, leaving the nested structure empty.
    fn clear(&mut self);
}

/// A trait for **fallible mutable** operations on a nested sequence.
///
/// Extends [`NestedMut<T>`] with methods that may fail, for example because
/// allocation or capacity limits are exceeded (e.g., in a fixed‑capacity
/// implementation like a small‑vector backed structure).
///
/// All methods return a [`Result`] where the error type is [`NestedError`].
///
/// # Implementors
///
/// - [`NestedVec`](crate::NestedVec)
pub trait TryNestedMut<T>: NestedMut<T> {
    /// Tries to append a new group with the given items.
    ///
    /// On success, returns `Ok(())`. On failure, returns
    /// `Err(NestedError::CapacityExceeded)` (or another appropriate variant).
    fn try_push(&mut self, items: impl IntoIterator<Item = T>) -> Result<(), NestedError>;

    /// Tries to append a new group with a single element.
    fn try_push_one(&mut self, val: T) -> Result<(), NestedError>;

    /// Tries to extend the last group with the given items.
    fn try_extend<I: IntoIterator<Item = T>>(&mut self, items: I) -> Result<(), NestedError>;

    /// Tries to extend the last group with a single element.
    fn try_extend_one(&mut self, val: T) -> Result<(), NestedError>;
}

/// A trait for nested structures that can be constructed with a default (empty) state.
///
/// # Implementors
///
/// - [`NestedVec`](crate::NestedVec)
pub trait NestedConstructor<T>: Default {
    /// Creates a new, empty nested structure.
    ///
    /// # Examples
    ///
    /// ```
    /// use nested::{NestedVec, NestedConstructor};
    /// let vec: NestedVec<i32, 4, 2> = NestedConstructor::new();
    /// assert!(vec.is_empty());
    /// ```
    fn new() -> Self;
}
