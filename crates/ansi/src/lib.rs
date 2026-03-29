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

pub use color::*;
pub use escape::*;
pub use sequences::*;
pub use style::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod color_vs_attribute {
        use super::*;
        use std::fmt::Debug;
        use std::ops::*;

        /// Adapter for comparing bit-wise operations between [`Attribute`] (reference) and [`Color`].
        #[derive(PartialEq, Eq, Clone, Copy)]
        enum Bit {
            None,
            Some,
        }

        impl Debug for Bit {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Bit::None => f.write_str("Bit::None"),
                    Bit::Some => f.write_str("Bit::Some"),
                }
            }
        }

        impl From<Attribute> for Bit {
            fn from(value: Attribute) -> Self {
                match value {
                    Attribute::None => Bit::None,
                    _ => Bit::Some,
                }
            }
        }
        impl From<Color> for Bit {
            fn from(value: Color) -> Self {
                match value {
                    Color::None => Bit::None,
                    _ => Bit::Some,
                }
            }
        }

        impl From<Bit> for Attribute {
            fn from(value: Bit) -> Self {
                match value {
                    Bit::None => Attribute::None,
                    Bit::Some => Attribute::Bold,
                }
            }
        }
        impl From<Bit> for Color {
            fn from(value: Bit) -> Self {
                match value {
                    Bit::None => Color::None,
                    Bit::Some => Color::Black,
                }
            }
        }

        macro_rules! assert_bit {
            ($lhs:expr, $rhs:expr, $op:ident) => {
                let (color, attribute) = (
                    Color::from($lhs).$op(Color::from($rhs)),
                    Attribute::from($lhs).$op(Attribute::from($rhs)),
                );

                let (actual, expected) = (Bit::from(color), Bit::from(attribute));
                assert_eq!(
                    actual,
                    expected,
                    "{:?}.{}({:?})",
                    $lhs,
                    stringify!($op),
                    $rhs
                );
            };
        }

        macro_rules! dbg_bit {
            ($lhs:expr, $rhs:expr, $op:ident) => {
                let (color, attribute) = (
                    Color::from($lhs).$op(Color::from($rhs)),
                    Attribute::from($lhs).$op(Attribute::from($rhs)),
                );

                if Bit::from(color) == Bit::from(attribute) {
                    eprintln!(
                        "✅ ({:?}.{}({:?})) = ({:?})",
                        $lhs,
                        stringify!($op),
                        $rhs,
                        Bit::from(color)
                    );
                } else {
                    eprintln!(
                        "❌ ({:?}.{}({:?})) = ({:?} / {:?})",
                        $lhs,
                        stringify!($op),
                        $rhs,
                        color,
                        attribute
                    );
                }
            };
        }

        const CASES: [(Bit, Bit); 6] = [
            (Bit::None, Bit::Some),
            (Bit::None, Bit::None),
            (Bit::Some, Bit::None),
            (Bit::Some, Bit::Some),
            (Bit::None, Bit::None),
            (Bit::None, Bit::Some),
        ];

        #[test]
        fn test_bitor() {
            for (lhs, rhs) in CASES {
                dbg_bit!(lhs, rhs, bitor);
            }

            for (lhs, rhs) in CASES {
                assert_bit!(lhs, rhs, bitor);
            }
        }

        #[test]
        fn test_bitand() {
            for (lhs, rhs) in CASES {
                dbg_bit!(lhs, rhs, bitand);
                assert_bit!(lhs, rhs, bitand);
            }
        }
    }
}
