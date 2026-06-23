mod nested;
pub use nested::*;

mod nested_slice;
pub use nested_slice::*;

mod nested_vec;
pub use nested_vec::*;

mod nested_array;
pub use nested_array::*;

pub mod iter;
pub use iter::*;

mod error;
pub use error::*;

#[macro_use]
pub mod macros;

mod index;
mod soa_nested_vec;

pub use index::*;

#[cfg(test)]
mod consistency_tests {
    use super::*;

    // ── empty ──────────────────────────────────────────────────────────
    type NestedVec<T> = super::NestedVec<T, 16, 8>;
    #[test]
    fn empty_is_consistent() {
        let nv: NestedVec<u8> = nested![];
        let nr = NestedArray::<u8, 16, 8>::new();
        assert_eq!(nv.len(), nr.len());
        assert_eq!(nv.is_empty(), nr.is_empty());
        assert!(nv.is_empty());
        assert_eq!(nv.len(), 0);
        assert!(nv.first().is_none());
        assert!(nv.last().is_none());
        assert!(nv.get(0).is_none());
    }

    #[test]
    fn empty_iter_returns_nothing() {
        let nv: NestedVec<u8> = nested![];
        let nr = NestedArray::<u8, 16, 8>::new();
        assert_eq!(nv.iter().count(), 0);
        assert_eq!(nr.iter().count(), 0);
    }

    // ── push_one + extend_one ─────────────────────────────────────────

    #[test]
    fn push_one_then_extend_one_grows_group() {
        let mut nv: NestedVec<u8> = nested![];
        nv.push_one(1);
        nv.extend_one(2);

        let mut nr = NestedArray::<u8, 16, 8>::new();
        nr.push_one(1);
        nr.extend_one(2);

        assert_eq!(nv.len(), nr.len());
        assert_eq!(nv.len(), 1);
        assert_eq!(nv.as_slice(), nr.as_slice());
        assert_eq!(&nv[0], &nr[0]);
        assert_eq!(&nv[0], &[1u8, 2]);
    }

    #[test]
    fn extend_one_on_empty_starts_group() {
        let mut nv: NestedVec<u8> = nested![];
        nv.extend_one(99);

        let mut nr = NestedArray::<u8, 16, 8>::new();
        nr.extend_one(99);

        assert_eq!(nv.len(), nr.len());
        assert_eq!(nv.len(), 1);
        assert_eq!(&nv[0], &[99]);
        assert_eq!(&nr[0], &[99]);
    }

    // ── push / extend (multi-value) ───────────────────────────────────

    #[test]
    fn push_creates_new_groups() {
        let mut nv: NestedVec<u8> = nested![];
        nv.push([1, 2]);
        nv.push([3]);

        let mut nr = NestedArray::<u8, 16, 8>::new();
        nr.push([1, 2]);
        nr.push([3]);

        assert_eq!(nv.len(), nr.len());
        assert_eq!(nv.len(), 2);
        assert_eq!(nv.as_slice(), nr.as_slice());
        assert_eq!(&nv[0], &[1, 2]);
        assert_eq!(&nv[1], &[3]);
    }

    #[test]
    fn extend_grows_last_group() {
        let mut nv: NestedVec<u8> = nested![];
        nv.push([1]);
        nv.extend([2, 3]);

        let mut nr = NestedArray::<u8, 16, 8>::new();
        nr.push([1]);
        nr.extend([2, 3]);

        assert_eq!(nv.len(), nr.len());
        assert_eq!(nv.len(), 1);
        assert_eq!(nv.as_slice(), nr.as_slice());
        assert_eq!(&nv[0], &[1, 2, 3]);
    }

    #[test]
    fn extend_on_empty_starts_group() {
        let mut nv: NestedVec<u8> = nested![];
        nv.extend([5, 6]);

        let mut nr = NestedArray::<u8, 16, 8>::new();
        nr.extend([5, 6]);

        assert_eq!(nv.len(), nr.len());
        assert_eq!(nv.len(), 1);
        assert_eq!(&nv[0], &[5, 6]);
    }

    // ── iteration ─────────────────────────────────────────────────────

    #[test]
    fn multiple_groups_iter_consistent() {
        let mut nv: NestedVec<u8> = nested![];
        nv.push([10]);
        nv.push([20, 30]);
        nv.push([40]);

        let mut nr = NestedArray::<u8, 16, 8>::new();
        nr.push([10]);
        nr.push([20, 30]);
        nr.push([40]);

        let nv_groups: Vec<&[u8]> = nv.iter().collect();
        let nr_groups: Vec<&[u8]> = nr.iter().collect();
        assert_eq!(nv_groups, nr_groups);
        assert_eq!(nv_groups, vec![&[10u8] as &[u8], &[20u8, 30], &[40u8]]);
    }

    // ── clear + reuse ─────────────────────────────────────────────────

    #[test]
    fn clear_then_reuse() {
        let mut nv: NestedVec<u8> = nested![];
        nv.push([1, 2, 3]);
        nv.clear();
        nv.push([4]);
        nv.extend_one(5);

        let mut nr = NestedArray::<u8, 16, 8>::new();
        nr.push([1, 2, 3]);
        nr.clear();
        nr.push([4]);
        nr.extend_one(5);

        assert_eq!(nv.len(), nr.len());
        assert_eq!(nv.len(), 1);
        assert_eq!(&nv[0], &[4, 5]);
        assert_eq!(&nr[0], &[4, 5]);
    }

    // ── NestedSlice roundtrip ─────────────────────────────────────────

    #[test]
    fn as_nested_slice_roundtrip() {
        let mut nv: NestedVec<u8> = nested![];
        nv.push([1, 2]);
        nv.push([3]);

        let ns = nv.as_nested_slice();
        assert_eq!(ns.len(), 2);
        assert_eq!(&ns[0], &[1, 2]);
        assert_eq!(&ns[1], &[3]);

        let nv2: NestedVec<u8> = ns.to_nested_vec();
        assert_eq!(nv, nv2);
    }

    #[test]
    fn nested_raw_as_params_roundtrip() {
        let mut nr = NestedArray::<u8, 16, 8>::new();
        nr.push([10, 20]);
        nr.push([30]);
        let ps = nr.as_nested_slice();
        assert_eq!(ps.len(), 2);
        assert_eq!(&ps[0], &[10, 20]);
        assert_eq!(&ps[1], &[30]);

        let nv: NestedVec<u8> = ps.to_nested_vec();
        assert_eq!(nv.len(), 2);
        assert_eq!(&nv[0], &[10, 20]);
        assert_eq!(&nv[1], &[30]);
    }

    // ── bounds checking ───────────────────────────────────────────────

    #[test]
    fn get_out_of_bounds_consistent() {
        let mut nv: NestedVec<u8> = nested![];
        nv.push_one(1);

        let mut nr = NestedArray::<u8, 16, 8>::new();
        nr.push_one(1);

        assert_eq!(nv.len(), nr.len());
        assert!(nv.get(0).is_some());
        assert!(nv.get(1).is_none());
        assert!(nr.get(0).is_some());
        assert!(nr.get(1).is_none());
    }

    // ── from_iter_flat ────────────────────────────────────────────────

    #[test]
    fn from_iter_and_indexing() {
        let nv: NestedVec<u8> = NestedVec::from_iter([[10], [20], [30]]);
        assert_eq!(nv.len(), 3);
        assert_eq!(&nv[0], &[10]);
        assert_eq!(&nv[1], &[20]);
        assert_eq!(&nv[2], &[30]);
        assert_eq!(nv.as_slice(), &[10, 20, 30]);
    }

    // ── index_mut ─────────────────────────────────────────────────────

    #[test]
    fn index_mut_consistent() {
        let mut nv: NestedVec<u8> = nested![];
        nv.push([1, 2, 3]);
        nv[0][1] = 99;

        let mut nr = NestedArray::<u8, 16, 8>::new();
        nr.push([1, 2, 3]);
        nr[0][1] = 99;

        assert_eq!(&nv[0], &nr[0]);
        assert_eq!(&nv[0], &[1, 99, 3]);
    }

    // ── first / last ──────────────────────────────────────────────────

    #[test]
    fn first_and_last_consistent() {
        let mut nv: NestedVec<u8> = nested![];
        let mut nr = NestedArray::<u8, 16, 8>::new();

        assert!(nv.first().is_none());
        assert!(nr.first().is_none());

        nv.push([1]);
        nv.push([2, 3]);
        nr.push([1]);
        nr.push([2, 3]);

        assert_eq!(nv.first(), nr.first());
        assert_eq!(nv.last(), nr.last());
        assert_eq!(nv.first(), Some(&[1u8] as &[u8]));
        assert_eq!(nv.last(), Some(&[2u8, 3] as &[u8]));
    }

    // ── FromIterator ──────────────────────────────────────────────────

    #[test]
    fn from_iter_array_creates_groups() {
        let nv: NestedVec<u8> = nested![[1, 2], [3, 3], [4, 5]];
        let v: Vec<[u8; 2]> = vec![[1u8, 2u8], [3u8, 3u8], [4u8, 5u8]];
        assert_eq!(nv.len(), 3);
        assert_eq!(&nv[0], &[1, 2]);
        assert_eq!(&nv[1], &[3, 3]);
        assert_eq!(&nv[2], &[4, 5]);
    }

    #[test]
    fn from_iter_slice_creates_groups() {
        let nv: NestedVec<u8> = nested![[1u8, 2], [3u8], [4u8, 5, 6]];
        assert_eq!(nv.len(), 3);
        assert_eq!(&nv[0], &[1, 2]);
        assert_eq!(&nv[1], &[3]);
        assert_eq!(&nv[2], &[4, 5, 6]);
    }
}
