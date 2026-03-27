pub trait Pack<T>: Sized {
    /// Converts to this type from the input type.
    #[must_use]
    fn pack(self) -> T;
}

pub trait TryPack<T>: Sized {
    /// The type returned in the event of a conversion error.
    type Error;

    /// Performs the conversion.
    fn try_pack(self) -> Result<T, Self::Error>;
}

pub trait Unpack<T>: Sized {
    /// Converts this type into the (usually inferred) input type.
    #[must_use]
    fn unpack(packed: T) -> Self;
}

pub trait TryUnpack<T>: Sized {
    type Error;

    /// Converts this type into the (usually inferred) input type.
    fn try_unpack(packed: T) -> Result<Self, Self::Error>;
}
