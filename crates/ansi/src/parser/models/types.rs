use utils::{SmallByteStr, SmallByteString, NestedSlice, NestedVec};

/// Represents ANSI intermediates parameters, a sequence of bytes.
pub type ByteString = SmallByteString;
/// Represents borrowed ANSI intermediate parameters, a sequence of bytes.
pub type ByteStr = SmallByteStr;

/// Represents ANSI parameters, a nested sequence of parameter values.
pub type Parameters<const N: usize = 2, const M: usize = 0> = NestedVec<u16, N, M>;
/// Represents borrowed ANSI parameters, an immutable view into the parameters.
pub type Params<'a> = NestedSlice<'a, u16>;

#[macro_export]
macro_rules! params {
    // Empty
    () => {
        {
        use utils::NestedConstructor;
        Parameters::new()
        }
    };
    // [_]
    ($($elem:literal),+ $(,)?) => (
        {
            Parameters::from_values([$($elem),*])
        }
    );
    // [[_]]
    ($([$($elem:literal),* $(,)?]),+ $(,)?) => (
        {
            use utils::{NestedConstructor, NestedMut};
            let mut p = Parameters::new();
            $(
            p.push([$($elem),*]);
            )+
            p
        }
    );
}
