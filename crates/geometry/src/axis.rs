
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct Axis<T> {
    pub horizontal: T,
    pub vertical: T,
}
impl<T> Axis<T> {
    /// Create a new axis.
    pub const fn new(horizontal: T, vertical: T) -> Self {
        Self { horizontal, vertical }
    }

    pub fn transpose(self) -> Self {
        Self { horizontal: self.vertical, vertical: self.horizontal }
    }
    
}