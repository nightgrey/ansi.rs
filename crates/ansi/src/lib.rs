#![feature(ascii_char)]
#![feature(bstr)]
#![feature(const_trait_impl)]
#![feature(const_destruct)]
#![feature(iter_intersperse)]
#![feature(const_ops)]
#![feature(formatting_options)]
#![feature(derive_const)]
#![feature(const_clone)]
#![feature(const_default)]
#![feature(const_cmp)]
mod color;
mod style;

#[macro_use]
mod escape;
pub mod sequences;

pub use escape::*;
pub use style::*;
pub use color::*;
pub use sequences::*;


#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod bit_ops {
        use std::fmt::Debug;
        use super::*;
        use std::ops::*;

        /// Adapter for comparing bit-wise operations between [`Attribute`] (reference) and [`Color`].
        #[derive(PartialEq, Eq, Clone, Copy)]
        enum Bit {
            Reset,
            None,
            Some
        }

        impl Debug for Bit {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Bit::Reset => f.write_str("Bit::Reset"),
                    Bit::None => f.write_str("Bit::None"),
                    Bit::Some => f.write_str("Bit::Some"),
                }
            }
        }

        impl From<Attribute> for Bit {
            fn from(value: Attribute) -> Self {
                match value {
                    Attribute::Reset => Bit::Reset,
                    Attribute::None => Bit::None,
                    _ => Bit::Some,
                }
            }
        }
        impl From<Color> for Bit {
            fn from(value: Color) -> Self {
                match value {
                    Color::Reset => Bit::Reset,
                    Color::None => Bit::None,
                    _ => Bit::Some,
                }
            }
        }

        impl From<Bit> for Attribute {
            fn from(value: Bit) -> Self {
                match value {
                    Bit::Reset => Attribute::Reset,
                    Bit::None => Attribute::None,
                    Bit::Some => Attribute::Bold,
                }
            }
        }
        impl From<Bit> for Color {
            fn from(value: Bit) -> Self {
                match value {
                    Bit::Reset => Color::Reset,
                    Bit::None => Color::None,
                    Bit::Some => Color::Red,
                }
            }
        }

        macro_rules! assert_bit {
            ($lhs:expr, $op:ident) => {
                let (color, attribute) = (
                    Color::from($lhs).$op(),
                    Attribute::from($lhs).$op()
                );

                let (actual, expected) = (Bit::from(color), Bit::from(attribute));
                assert_eq!(actual, expected, "{:?}.{}", $lhs, stringify!($op));
            };
            ($lhs:expr, $rhs:expr, $op:ident) => {
                let (color, attribute) = (
                    Color::from($lhs).$op(Color::from($rhs)),
                    Attribute::from($lhs).$op(Attribute::from($rhs))
                );

                let (actual, expected) = (Bit::from(color), Bit::from(attribute));
                assert_eq!(actual, expected, "{:?}.{}({:?})", $lhs, stringify!($op), $rhs);
            };
        }
        macro_rules! dbg_bit {
            ($lhs:expr, $op:ident) => {
                let (color, attribute) = (
                    Color::from($lhs).$op(),
                    Attribute::from($lhs).$op()
                );

                if Bit::from(color) == Bit::from(attribute) {
                    eprintln!("✅ {:?}.{}() = {:?}", $lhs, stringify!($op), Bit::from(color));
                    eprintln!("{:?} vs {:?}", color, attribute);
                } else {
                    eprintln!("❌ {:?}.{}() = ({:?} / {:?})", $lhs, stringify!($op), color, attribute);
                };
            };
            ($lhs:expr, $rhs:expr, $op:ident) => {
                let (color, attribute) = (
                    Color::from($lhs).$op(Color::from($rhs)),
                    Attribute::from($lhs).$op(Attribute::from($rhs))
                );

                if Bit::from(color) == Bit::from(attribute) {
                    eprintln!("✅ ({:?}.{}({:?})) = ({:?})", $lhs, stringify!($op), $rhs, Bit::from(color));
                } else {
                    eprintln!("❌ ({:?}.{}({:?})) = ({:?} / {:?})", $lhs, stringify!($op), $rhs, color, attribute);
                }
            };
        }

        #[test]
        fn test_bitops() {
            for (lhs, rhs) in  [
                (Bit::None, Bit::None),
                (Bit::None, Bit::Some),
                (Bit::None, Bit::Reset),

                (Bit::Some, Bit::None),
                (Bit::Some, Bit::Some),
                (Bit::Some, Bit::Reset),

                (Bit::Reset, Bit::None),
                (Bit::Reset, Bit::Some),
                (Bit::Reset, Bit::Reset),
            ] {
                dbg_bit!(lhs, rhs, bitxor);
                assert_bit!(lhs, rhs, bitxor);
                assert_bit!(lhs, rhs, bitand);
            }

        }

        #[test]
        fn test_not() {
            for value in [
                Bit::None,
                Bit::Some,
                Bit::Reset,
            ] {
                dbg_bit!(value, not);
                assert_bit!(value, not);
            }
        }
    }
}