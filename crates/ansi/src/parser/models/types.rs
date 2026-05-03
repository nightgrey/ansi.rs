use utils::{ByteStr, ByteString, NestedIter, NestedSlice, NestedVec};

pub use utils::Nested;

/// Represents ANSI intermediates parameters, a sequence of bytes.
pub type Intermediates = ByteString;
/// Represents borrowed ANSI intermediate parameters, a sequence of bytes.
pub type Inter = ByteStr;

/// Represents ANSI parameters, a nested sequence of parameter values.
pub type Parameters<const N: usize = 2, const M: usize = 2> = NestedVec<u16, N, M>;
/// Represents borrowed ANSI parameters, an immutable view into the parameters.
pub type Params<'a> = NestedSlice<'a, u16>;
/// An iterator over nested ANSI parameters.
pub type ParamsIter<'a> = NestedIter<'a, u16>;

pub type DataString = ByteString;
pub type DataStr = ByteStr;

#[macro_export]
macro_rules! params {
    // Empty
    () => {
       use utils::{Nested as _, NestedConstructor as _};
        utils::NestedVec::new()
    };
    // [_]
    ($($elem:literal),+ $(,)?) => (
            use utils::{Nested as _, NestedConstructor as _};

        let mut nested = utils::NestedVec::new();
        let inner = SmallVec::from_iter($($elem),*.into_iter());
        utils::NestedVec {
            starts: SmallVec::from_iter(0..=inner.len()),
            inner,
        }
    );
    // [[_]]
    ($([$($elem:literal),* $(,)?]),+ $(,)?) => (
        {
            use utils::{Nested as _, NestedConstructor as _};
            let mut nested = utils::NestedVec::new();
            $(
            nested.push([$($elem),*]);
            )+
            nested
        }
    );
}
