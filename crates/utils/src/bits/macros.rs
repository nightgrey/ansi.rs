#[macro_export]
macro_rules! bits {
    // Explicit layout: each variant carries its own bit pattern, so aliases
    // / combined flags (`All = Bold as $repr | Italic as $repr`) are allowed.
    (
        bits = $bits:ident,
        bit = $(#[$m:meta])* $vis:vis enum $bit:ident: $repr:ty {
            $($(#[$am:meta])* $variant:ident = $value:expr),+ $(,)?
        },
        empty = $empty:ident
    ) => {
        // Bit
        $(#[$m])*
        #[repr($repr)]
        #[derive(Copy, Debug)]
        #[derive_const(Clone, Default, PartialOrd, Ord, PartialEq, Eq)]
        $vis enum $bit { $($(#[$am])* $variant = $value),+ }

        impl const $crate::Bit for $bit {
            type Repr = $repr;

            const LIST: &'static [Self] = &[ $( $bit::$variant ),+ ];
            const COUNT: usize = Self::LIST.len();

            const ALL: $repr = $( ($bit::$variant as $repr) )|+;
            const EMPTY: Self::Repr = Self::$empty.into();
        }

        impl const std::ops::BitAnd for $bit {
                type Output = $crate::Bits<$bit>;

            fn bitand(self, rhs: Self) -> Self::Output {
                $crate::Bits::new((self as $repr) & (rhs as $repr))
            }
        }

        impl const std::ops::BitOr for $bit {
            type Output = $crate::Bits<$bit>;

            fn bitor(self, rhs: Self) -> Self::Output {
                $crate::Bits::new((self as $repr) | (rhs as $repr))
            }
        }

        impl const std::ops::BitXor for $bit {
            type Output = $crate::Bits<$bit>;

            fn bitxor(self, rhs: Self) -> Self::Output {
                $crate::Bits::new((self as $repr) ^ (rhs as $repr))
            }
        }

        impl const std::ops::Not for $bit {
            type Output = $crate::Bits<$bit>;

            fn not(self) -> Self::Output {
                $crate::Bits::new(!(self as $repr))
            }
        }

        impl const core::convert::From<$bit> for $repr {
            fn from(value: $bit) -> Self {
                value as $repr
            }
        }

        impl const core::convert::Into<$crate::Bits<$bit>> for $bit {
            fn into(self) -> $crate::Bits<$bit> {
                 $crate::Bits::new(self)
            }
        }
    };
}
