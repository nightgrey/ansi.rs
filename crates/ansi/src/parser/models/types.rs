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

#[test]
fn qew() {
    let a: Parameters<8, 8> = params![1,2,3];

}