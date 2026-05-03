
#[macro_export]
macro_rules! nested {
    // Empty
    () => {
        NestedVec {
            inner: smallvec::SmallVec::new(),
            starts: smallvec::SmallVec::new(),
        }
    };
    // [_]
    ($($elem:literal),+ $(,)?) => (
        NestedVec::from_iter([$($elem),*])
    );
    // [[_]]
    ($([$($elem:literal),* $(,)?]),+ $(,)?) => (
        {
            let mut nested = NestedVec::new();
            $(
            nested.push([$($elem),*]);
            )+
            nested
        }
    );
}

