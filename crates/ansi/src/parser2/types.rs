use smallvec::SmallVec;
use utils::{ByteStr, ByteString, NestedIter, NestedSlice, NestedVec};

/// Represents ANSI intermediates parameters, a sequence of bytes.
pub type Intermediates = ByteString;
/// Represents borrowed ANSI intermediate parameters, a sequence of bytes.
pub type Inter = ByteStr;

/// Represents ANSI parameters, a nested sequence of parameter values.
pub type Parameters<const N: usize = 16> = NestedVec<u16, N>;
/// Represents borrowed ANSI parameters, an immutable view into the parameters.
pub type Params<'a> = NestedSlice<'a, u16>;
/// An iterator over nested ANSI parameters.
pub type ParamsIter<'a> = NestedIter<'a, u16>;

pub type DataString = ByteString;
pub type DataStr = ByteStr;