use utils::{ByteStr, ByteString, NestedIter, NestedSlice, NestedVec};
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
    () => {
        Parameters::empty()
    };

    // Nested, same length
    ($([$($elem:literal),* $(,)?]),+ $(,)?) => (
        Parameters::from_iter([$(&[$($elem as u16),*] as &[u16],)+])
    );

    // Nested, same length
    // ($([$($elem:literal),* $(,)?]),+ $(,)?) => (
    //     Parameter::from_iter_nested([$([$($elem),*] as [_],)+])
    // );
    ($($elem:literal),+) => (
        Parameters::from_iter([$($elem),*])
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
