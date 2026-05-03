// Trait for types that can be converted into type holding a reference.
pub trait AsRefd<T> {
    fn as_refd(self) -> T;
}
