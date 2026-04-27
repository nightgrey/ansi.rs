use std::ascii;
use utils::{ByteStr, ByteString, NestedIter, NestedSlice, NestedVec};

pub type FinalChar = char;
pub type FinalByte = u8;
/// Represents ANSI intermediates parameters, a sequence of bytes.
pub type Intermediates = ByteString<2>;
/// Represents borrowed ANSI intermediate parameters, a sequence of bytes.
pub type Inter = ByteStr;

/// Represents ANSI parameters, a nested sequence of parameter values.
pub type Parameter<const N: usize = 16> = NestedVec<u16, N>;
/// Represents borrowed ANSI parameters, an immutable view into the parameters.
pub type Params<'a> = NestedSlice<'a, u16>;
/// An iterator over nested ANSI parameters.
pub type ParamIter<'a> = NestedIter<'a, u16>;

/// Represents ANSI data, a sequence of human-readable bytes.
pub type DataString = ByteString<1024>;
/// Represents borrowed ANSI data, a sequence of human-readable bytes.
pub type DataStr = ByteStr<1024>;
#[macro_export]
macro_rules! params {
    () => {
        $crate::Params::empty()
    };

    // Nested, same length
    ($([$($elem:literal),* $(,)?]),+ $(,)?) => (
        Parameter::from_iter([$(&[$($elem as u16),*] as &[u16],)+])
    );

    // Nested, same length
    // ($([$($elem:literal),* $(,)?]),+ $(,)?) => (
    //     Parameter::from_iter_nested([$([$($elem),*] as [_],)+])
    // );
    ($($elem:literal),+) => (
        Parameter::from_iter([$($elem),*])
    );
        // ($($elem:expr),+ $(,)?) => (
    //     Parameter::from_iter([$($elem),*])
    // );
    // ($([$($elem:expr),* $(,)?]),+ $(,)?) => (
    //     Parameter::from_iter_nested([$([$($elem),*],)+])
    // );
    // ($($elem:expr),+ $(,)?) => (
    //     Parameter::from_iter([$($elem),*])
    // );
}
