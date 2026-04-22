pub use utils::{NestedIter, NestedSlice, NestedVec};

/// ANSI parameters
///
/// [`Parameter`] stores multiple nested groups of ANSI parameters efficiently.
///
/// # Examples
///
/// The most common way to build a [`Parameter`] is with [`push_group`]:
///
/// ```
/// # use ansi::Parameter;
/// let mut param = Parameter::new();
/// param.push_group([1, 2, 3]);
/// param.push_group([4, 5]);
/// param.push_group([6, 7, 8, 9]);
///
/// assert_eq!(param.len(), 3);
/// assert_eq!(param.get(1), Some(&[4, 5][..]));
/// ```
///
/// For incremental group construction, use [`extend`] and [`separate`]:
///
/// ```
/// # use ansi::Parameter;
/// let mut param = Parameter::new();
/// param.extend([1, 2]);
/// param.extend([3]);
/// param.separate(); // Group 0 is now [1, 2, 3]
///
/// param.extend([4, 5]); // Start of group 1
/// param.separate();
///
/// assert_eq!(nested.get(0), Some(&[1, 2, 3][..]));
/// assert_eq!(nested.get(1), Some(&[4, 5][..]));
/// ```
///
/// # Type Parameters
///
/// - `N`: The number of **total** elements/indices to store inline before spilling to
///   the heap.
///
/// [`push_group`]: Parameter::push_group
/// [`extend`]: Parameter::extend
/// [`separate`]: Parameter::separate
pub type Params<const N: usize = 16> = NestedVec<u16, N>;

/// A borrowed, nested ANSI parameter structure - an immutable view into the parameters.
pub type Paras<'a> = NestedSlice<'a, u16>;
pub type ParamIter<'a> = NestedIter<'a, u16>;
