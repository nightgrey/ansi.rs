#[derive(Copy, Debug)]
#[derive_const(Clone, Default, PartialEq, Eq)]
pub struct Axis<T> {
    pub horizontal: T,
    pub vertical: T,
}
impl<T> Axis<T> {
    /// Create a new axis.
    pub const fn new(horizontal: T, vertical: T) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    pub const fn both(value: T) -> Self where T: Copy {
        Self {
            horizontal: value,
            vertical: value,
        }
    }

    pub fn transpose(self) -> Self {
        Self {
            horizontal: self.vertical,
            vertical: self.horizontal,
        }
    }
}
