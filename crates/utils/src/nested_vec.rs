use smallvec::SmallVec;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::ops::{self, Deref, Index, IndexMut, Sub};
use std::slice::Iter;
use std::usize;

trait AsNestedRef<'a, T> {
    fn as_ref(self) -> NestedSlice<'a, T>;
}

/// An owned, growable container for a sequence of element slices ("groups").
///
/// [`NestedVec`] stores multiple groups of elements efficiently by keeping all
/// elements in a single contiguous buffer, with a separate index array tracking
/// where each group begins and ends. This avoids per-group allocations while
/// still providing slice-based access to individual groups.
///
/// # Memory Layout
///
/// ```text
/// indices:  [0, 3, 5, 8]               // N+1 indices for N groups
/// elements: [a, b, c, d, e, f, g, h]
///
/// Resulting groups:
///   [0] → [a, b, c]
///   [1] → [d, e]
///   [2] → [f, g, h]
/// ```
///
/// # Examples
///
/// The most common way to build a [`NestedVec`] is with [`push_group`]:
///
/// ```
/// # use nested_vec::NestedVec;
/// let mut nested = NestedVec::new();
/// nested.push_group([1, 2, 3]);
/// nested.push_group([4, 5]);
/// nested.push_group([6, 7, 8, 9]);
///
/// assert_eq!(nested.len(), 3);
/// assert_eq!(nested.get(1), Some(&[4, 5][..]));
/// ```
///
/// For incremental group construction, use [`extend`] and [`separate`]:
///
/// ```
/// # use nested_vec::NestedVec;
/// let mut nested = NestedVec::new();
/// nested.extend([1, 2]);
/// nested.extend([3]);
/// nested.separate(); // Group 0 is now [1, 2, 3]
///
/// nested.extend([4, 5]); // Start of group 1
/// nested.separate();
///
/// assert_eq!(nested.get(0), Some(&[1, 2, 3][..]));
/// assert_eq!(nested.get(1), Some(&[4, 5][..]));
/// ```
///
/// # Type Parameters
///
/// - `T`: The element type stored in groups.
/// - `N`: The number of elements/indices to store inline before spilling to
///   the heap. Defaults to 8. Use `NestedVec<T, 0>` to always heap-allocate.
///
/// # See Also
///
/// - [`NestedSlice`] for borrowed views into nested data.
///
/// [`push_group`]: NestedVec::push_group
/// [`extend`]: NestedVec::extend
/// [`separate`]: NestedVec::separate
#[derive(Debug, Clone, PartialEq, Hash, PartialOrd, Eq)]
pub struct NestedVec<T, const N: usize = 8> {
    indices: SmallVec<usize, N>,
    elements: SmallVec<T, N>,
}

impl<T, const N: usize> NestedVec<T, N> {
    pub const DEFAULT: Self = Self {
        indices: SmallVec::from_buf([0]),
        elements: SmallVec::new(),
    };

    /// Creates a new empty [`NestedVec`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use nested_vec::NestedVec;
    /// let nested: NestedVec<i32> = NestedVec::new();
    /// assert!(nested.is_empty());
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self::DEFAULT
    }

    /// Creates a new empty [`NestedVec`] with capacity for at least `capacity`
    /// elements.
    ///
    /// The actual capacity may be greater. Use this when you know approximately
    /// how many total elements will be stored to avoid reallocations.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nested_vec::NestedVec;
    /// let mut nested = NestedVec::<i32>::with_capacity(100);
    /// assert!(nested.capacity() >= 100);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            indices: {
                let mut indices = SmallVec::with_capacity(capacity.max(1));
                indices.push(0);
                indices
            },
            elements: SmallVec::with_capacity(capacity),
        }
    }

    /// Pushes a complete group of elements.
    ///
    /// This is the most ergonomic way to add groups. Each call creates a new
    /// group containing all elements from the iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nested_vec::NestedVec;
    /// let mut nested = NestedVec::new();
    /// nested.push_group([1, 2, 3]);
    /// nested.push_group(vec![4, 5]);
    ///
    /// assert_eq!(nested.len(), 2);
    /// assert_eq!(nested.get(0), Some(&[1, 2, 3]));
    /// assert_eq!(nested.get(1), Some(&[4, 5]));
    /// ```
    pub fn push_group(&mut self, group: impl IntoIterator<Item = T>) {
        self.elements.extend(group);
        self.indices.push(self.elements.len());
    }

    /// Extends the current group with elements from an iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nested_vec::NestedVec;
    /// let mut nested = NestedVec::new();
    /// nested.push_one(1);
    /// nested.extend([2, 3]);
    /// nested.extend([4, 5]);
    /// nested.finish();
    /// nested.extend([6, 7]);
    ///
    /// assert_eq!(nested.get(0), Some(&[1, 2, 3, 4, 5]));
    /// ```
    ///
    /// [`finish`]: NestedVec::finish
    pub fn extend(&mut self, elements: impl IntoIterator<Item = T>) {
        self.elements.extend(elements);
    }

    /// Pushes a single element to the last group.
    ///
    /// If the [`NestedVec`] is empty, this implicitly starts the first group.
    /// Call [`finish`] to mark the end of the group.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nested_vec::NestedVec;
    /// let mut nested = NestedVec::new();
    /// nested.push_element(1);
    /// nested.push_element(2);
    /// nested.finish();
    ///
    /// assert_eq!(nested.get(0), Some(&[1, 2][..]));
    /// ```
    ///
    /// [`finish`]: NestedVec::finish
    pub fn push_one(&mut self, element: T) {
        self.elements.push(element);
    }

    /// Marks the end of the current group.
    ///
    /// Call this after using [`extend`] or [`push_one`] to delimit
    /// where one group ends and the next begins. Has no effect if the current
    /// group is empty (i.e., no elements have been added since the last call
    /// to `finish` or since creation).
    ///
    /// # Examples
    ///
    /// ```
    /// # use nested_vec::NestedVec;
    /// let mut nested = NestedVec::new();
    /// nested.extend([1, 2, 3]);
    /// nested.finish();
    /// nested.extend([4, 5]);
    /// nested.finish();
    ///
    /// assert_eq!(nested.len(), 2);
    /// ```
    ///
    /// [`extend`]: NestedVec::extend
    /// [`push_one`]: NestedVec::push_one
    pub fn separate(&mut self) {
        if self.elements.len() == self.indices.len() - 1 {
            return;
        }
        self.indices.push(self.elements.len());
    }

    /// Returns the group at `index`, or `None` if out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nested_vec::NestedVec;
    /// let mut nested = NestedVec::new();
    /// nested.push_group([1, 2, 3]);
    ///
    /// assert_eq!(nested.get(0), Some(&[1, 2, 3][..]));
    /// assert_eq!(nested.get(1), None);
    /// ```
    pub fn get(&self, index: usize) -> Option<&[T]> {
        if index >= self.len() {
            return None;
        }

        // SAFETY: Bounds check performed above.
        unsafe { Some(self.get_unchecked(index)) }
    }

    /// Returns the group at `index`, or `None` if out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nested_vec::NestedVec;
    /// let mut nested = NestedVec::new();
    /// nested.push_group([1, 2, 3]);
    ///
    /// assert_eq!(nested.get_mut(0), Some(&mut [1, 2, 3]));
    /// assert_eq!(nested.get_mut(1), None);
    /// ```
    pub fn get_mut(&mut self, index: usize) -> Option<&mut [T]> {
        if index >= self.len() {
            return None;
        }

        // SAFETY: Bounds check performed above.
        unsafe { Some(self.get_unchecked_mut(index)) }
    }

    /// Returns the group at `index` without bounds checking.
    ///
    /// # Safety
    ///
    /// `index` must be less than [`len`](Self::len). Calling this with an
    /// out-of-bounds index is undefined behavior.
    pub unsafe fn get_unchecked(&self, index: usize) -> &[T] {
        let start = *self.indices.get_unchecked(index);
        let end = *self.indices.get_unchecked(index + 1);
        self.elements.get_unchecked(start..end)
    }

    /// Returns the group at `index` without bounds checking.
    ///
    /// # Safety
    ///
    /// `index` must be less than [`len`](Self::len). Calling this with an
    /// out-of-bounds index is undefined behavior.
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut [T] {
        let start = *self.indices.get_unchecked(index);
        let end = *self.indices.get_unchecked(index + 1);
        self.elements.get_unchecked_mut(start..end)
    }

    /// Returns the number of groups.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nested_vec::NestedVec;
    /// let mut nested = NestedVec::new();
    /// assert_eq!(nested.len(), 0);
    ///
    /// nested.push_group([1, 2, 3]);
    /// assert_eq!(nested.len(), 1);
    ///
    /// nested.push_group([4]);
    /// assert_eq!(nested.len(), 2);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.indices.len().saturating_sub(1)
    }

    /// Returns the total number of elements across all groups.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nested_vec::NestedVec;
    /// let mut nested = NestedVec::new();
    /// nested.push_group([1, 2, 3]);
    /// nested.push_group([4, 5]);
    ///
    /// assert_eq!(nested.count(), 5);
    /// ```
    #[inline]
    pub fn count(&self) -> usize {
        self.elements.len()
    }

    /// Returns `true` if there are no groups.
    ///
    /// Note: This checks for the absence of *groups*, not elements. A
    /// [`NestedVec`] with an empty group (`[]`) is not considered empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nested_vec::NestedVec;
    /// let mut nested = NestedVec::new();
    /// assert!(nested.is_empty());
    ///
    /// nested.push_group::<i32>([]);
    /// assert!(!nested.is_empty()); // Has one (empty) group
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the capacity for elements (not groups).
    ///
    /// # Examples
    ///
    /// ```
    /// # use nested_vec::NestedVec;
    /// let nested = NestedVec::<i32>::with_capacity(100);
    /// assert!(nested.capacity() >= 100);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.elements.capacity()
    }

    /// Reserves capacity for at least `additional` more elements.
    ///
    /// Does nothing if capacity is already sufficient.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.indices.reserve(additional + 1);
        self.elements.reserve(additional);
    }

    /// Returns an iterator over the groups as slices.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nested_vec::NestedVec;
    /// let mut nested = NestedVec::new();
    /// nested.push_group([1, 2]);
    /// nested.push_group([3, 4, 5]);
    ///
    /// let groups: Vec<&[i32]> = nested.iter().collect();
    /// assert_eq!(groups, vec![&[1, 2][..], &[3, 4, 5][..]]);
    /// ```
    pub fn iter(&self) -> NestedIter<'_, T> {
        NestedIter::new(self)
    }

    /// Returns an iterator over all elements in all groups.
    ///
    /// Elements are yielded in order, group by group.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nested_vec::NestedVec;
    /// let mut nested = NestedVec::new();
    /// nested.push_group([1, 2]);
    /// nested.push_group([3, 4, 5]);
    ///
    /// let elements: Vec<&i32> = nested.iter_elements().collect();
    /// assert_eq!(elements, vec![&1, &2, &3, &4, &5]);
    /// ```
    pub fn iter_elements(&self) -> Iter<'_, T> {
        self.elements.iter()
    }

    /// Removes all groups and elements.
    ///
    /// The capacity is preserved.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nested_vec::NestedVec;
    /// let mut nested = NestedVec::new();
    /// nested.push_group([1, 2, 3]);
    /// nested.clear();
    ///
    /// assert!(nested.is_empty());
    /// assert!(nested.capacity() > 0);
    /// ```
    pub fn clear(&mut self) {
        self.elements.clear();
        self.indices.clear();
        self.indices.push(0);
    }

    /// Returns a borrowed [`NestedSlice`] view of this container.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nested_vec::NestedVec;
    /// let mut nested = NestedVec::new();
    /// nested.push_group([1, 2, 3]);
    ///
    /// let slice = nested.as_slice();
    /// assert_eq!(slice.get(0), Some(&[1, 2, 3][..]));
    /// ```
    #[inline]
    pub fn as_slice(&self) -> NestedSlice<'_, T> {
        NestedSlice {
            indices: &self.indices,
            elements: &self.elements,
        }
    }

    /// Returns the raw indices slice.
    ///
    /// The returned slice has length `len() + 1`, where `indices[i]` to
    /// `indices[i + 1]` defines the range of elements in group `i`.
    #[inline]
    pub fn as_indices(&self) -> &[usize] {
        &self.indices
    }

    /// Returns the raw elements slice.
    ///
    /// This is the contiguous buffer containing all elements from all groups.
    #[inline]
    pub fn as_elements(&self) -> &[T] {
        &self.elements
    }

    /// Creates a [`NestedVec`] from an iterator of group iterators.
    ///
    /// Each item from the outer iterator becomes one group.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nested_vec::NestedVec;
    /// let groups = vec![vec![1, 2, 3], vec![4, 5], vec![6]];
    /// let nested = NestedVec::from_group_iter(groups);
    ///
    /// assert_eq!(nested.len(), 3);
    /// assert_eq!(nested.get(0), Some(&[1, 2, 3][..]));
    /// assert_eq!(nested.get(1), Some(&[4, 5][..]));
    /// assert_eq!(nested.get(2), Some(&[6][..]));
    /// ```
    pub fn from_iter_nested<I, Group>(iter: I) -> Self
    where
        I: IntoIterator<Item = Group>,
        Group: IntoIterator<Item = T>,
    {
        let iter = iter.into_iter();

        let (lower, upper) = iter.size_hint();
        let estimated_capacity = upper.unwrap_or(lower).saturating_mul(2).max(N);

        let mut nested = Self::with_capacity(estimated_capacity);

        for group in iter {
            nested.push_group(group);
        }

        nested
    }
}

impl<T, const N: usize> Default for NestedVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Index<usize> for NestedVec<T, N> {
    type Output = [T];
    fn index(&self, index: usize) -> &Self::Output {
        unsafe { Self::get_unchecked(self, index) }
    }
}

impl<T, const N: usize> IndexMut<usize> for NestedVec<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { self.get_unchecked_mut(index) }
    }
}

impl<T, const N: usize> Extend<T> for NestedVec<T, N> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        Self::extend(self, iter);
    }

    fn extend_one(&mut self, item: T) {
        Self::push_one(self, item);
    }

    fn extend_reserve(&mut self, additional: usize) {
        Self::reserve(self, additional);
    }
}

impl<T, const N: usize> FromIterator<T> for NestedVec<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut nested = match iter.size_hint() {
            (0, None) => NestedVec::default(),
            (_, Some(l)) | (l, None) => NestedVec::<T, N>::with_capacity(l),
        };

        nested.extend(iter);
        nested
    }
}

impl<'a, T, const N: usize> AsNestedRef<'a, T> for &'a NestedVec<T, N> {
    fn as_ref(self) -> NestedSlice<'a, T> {
        self.as_slice()
    }
}

impl<T, const N: usize> AsRef<[T]> for NestedVec<T, N> {
    fn as_ref(&self) -> &[T] {
        &self.elements
    }
}

/// A borrowed nested structure - immutable view into nested data.
///
/// This is a lightweight wrapper around two slices that provides
/// convenient access to nested slice structures.
///
/// Unlike `NestedVec`, this does not own the data. It is typically used
/// when you have a reference to a `NestedVec` or want to pass data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NestedSlice<'a, T> {
    indices: &'a [usize],
    elements: &'a [T],
}

impl<'a, T> NestedSlice<'a, T> {
    /// Creates a new `NestedSlice` from indices and values.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if invariants are violated:
    /// - indices must not be empty
    /// - first index must be 0
    /// - last index must equal values.len()
    #[inline]
    pub const fn new(indices: &'a [usize], values: &'a [T]) -> Self {
        if indices[0] != 0 {
            panic!("first index must be 0");
        }
        NestedSlice {
            indices,
            elements: values,
        }
    }

    /// Returns the number of groups in this nested structure.
    #[inline]
    pub fn len(&self) -> usize {
        self.indices.len().saturating_sub(1)
    }

    /// Returns the total number of elements across all groups.
    #[inline]
    pub fn count(&self) -> usize {
        self.elements.len()
    }

    /// Returns true if there are no groups.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the slice at the given index, if in bounds.
    pub fn get(&self, index: usize) -> Option<&'a [T]> {
        if index >= self.len() {
            None
        } else {
            // SAFETY: We've checked bounds above
            unsafe { Some(self.get_unchecked(index)) }
        }
    }

    /// Returns a reference to the slice at the given index without bounds checking.
    ///
    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is undefined behavior.
    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> &'a [T] {
        let start = *self.indices.get_unchecked(index);
        let end = *self.indices.get_unchecked(index + 1);
        self.elements.get_unchecked(start..end)
    }

    /// Returns an iterator over the sub group slices.
    pub fn iter(&self) -> NestedIter<'a, T> {
        NestedIter {
            values: self.elements,
            windows: self.indices.windows(2),
        }
    }

    /// Returns an iterator over the values.
    pub fn iter_values(&self) -> Iter<'_, T> {
        self.elements.iter()
    }

    /// Returns the underlying values as a single slice.
    #[inline]
    pub fn as_slice(&self) -> &'a [T] {
        self.elements
    }

    /// Returns the indices slice.
    #[inline]
    pub fn as_indices(&self) -> &[usize] {
        self.indices
    }

    /// Returns the underlying values as a slice.
    #[inline]
    pub fn as_elements(&self) -> &[T] {
        self.elements
    }
}

impl<'a, T: Clone> NestedSlice<'a, T> {
    /// Converts this `NestedSlice` into an owned `NestedVec`.
    ///
    /// This performs a deep copy of the data.
    pub fn to_params<const N: usize>(&self) -> NestedVec<T, N> {
        NestedVec {
            indices: SmallVec::from_iter(self.indices.iter().copied()),
            elements: SmallVec::from_iter(self.elements.iter().cloned()),
        }
    }
}

impl<'a, T> Index<usize> for NestedSlice<'a, T> {
    type Output = [T];

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("index out of bounds")
    }
}

impl<'a, T> AsRef<[T]> for NestedSlice<'a, T> {
    fn as_ref(&self) -> &[T] {
        self.elements
    }
}

impl<'a, T> AsNestedRef<'a, T> for NestedSlice<'a, T> {
    fn as_ref(self) -> NestedSlice<'a, T> {
        self
    }
}

/// An iterator over the nested slices of a `NestedVec`.
///
/// This iterator yields references to the slices defined by the structure.
#[derive(Debug, Clone)]
pub struct NestedIter<'a, T: 'a> {
    values: &'a [T],
    windows: std::slice::Windows<'a, usize>,
}

impl<'a, T> NestedIter<'a, T> {
    /// Creates a new iterator over the given `NestedVec`.
    pub fn new(nested: impl AsNestedRef<'a, T>) -> Self {
        let nested = nested.as_ref();

        Self {
            windows: nested.indices.windows(2),
            values: nested.elements,
        }
    }
}

impl<'a, T> Iterator for NestedIter<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        self.windows.next().map(|w| &self.values[w[0]..w[1]])
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.windows.size_hint()
    }
}

impl<'a, T> ExactSizeIterator for NestedIter<'a, T> {
    fn len(&self) -> usize {
        self.windows.len()
    }
}

impl<'a, T> DoubleEndedIterator for NestedIter<'a, T> {
    fn next_back(&mut self) -> Option<<Self as Iterator>::Item> {
        self.windows.next_back().map(|w| &self.values[w[0]..w[1]])
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_len() {
        let mut nested = NestedVec::<u8, 4>::default();
        nested.extend([1]);
        nested.extend([2, 3]);
        nested.extend([4]);

        assert_eq!(nested.len(), 3);
    }

    #[test]
    fn test_count() {
        let mut nested = NestedVec::<u8, 4>::default();
        nested.extend([1]);
        nested.extend([2, 3]);
        nested.extend([4]);

        assert_eq!(nested.count(), 4);
    }

    #[test]
    fn test_iter() {
        let mut nested = NestedVec::<u8, 4>::default();
        nested.extend([5]);
        nested.extend([6]);
        nested.extend([7]);
        nested.extend([8, 9, 10, 12]);

        let mut iter = nested.iter();

        assert_eq!(iter.next().unwrap(), &[5]);
        assert_eq!(iter.next().unwrap(), &[6]);
        assert_eq!(iter.next().unwrap(), &[7]);
        assert_eq!(iter.next().unwrap(), &[8, 9, 10, 12]);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn from_iter() {
        let from_iter_nested: NestedVec<i32> =
            NestedVec::from_iter_nested(vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]]);

        assert_eq!(from_iter_nested.len(), 3);
        assert_eq!(from_iter_nested.count(), 9);

        let from_iter: NestedVec<i32> = NestedVec::from_iter(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);

        assert_eq!(from_iter.len(), 1);
        assert_eq!(from_iter.count(), 9);
    }
}
