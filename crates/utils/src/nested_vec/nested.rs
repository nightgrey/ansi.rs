use crate::{NestedError, NestedIndex, NestedIndexMut, NestedIter, NestedSlice, NestedVec};
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
