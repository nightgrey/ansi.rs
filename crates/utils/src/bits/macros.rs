#[macro_export]
macro_rules! bits {
// Explicit layout: each variant carries its own bit pattern, so aliases
    // / combined flags (`All = Bold as $repr | Italic as $repr`) are allowed.
    (
        $(#[$bits_meta:meta])* $bits:ident,
        $(#[$bit_meta:meta])* $vis:vis enum $bit:ident: $repr:ty {
            $($(#[$variant_meta:meta])* $variant:ident = $value:expr),+ $(,)?
        }
        $(,)?
    ) => {
        // Bit
        $(#[$bit_meta])*
        #[repr($repr)]
        #[derive(Copy, Debug)]
        #[derive_const(Clone, Default, PartialOrd, Ord, PartialEq, Eq)]
        $vis enum $bit { $($(#[$variant_meta])* $variant = $value),+ }

        /// A set of [`
        #[doc = stringify!($bit)]
        /// `] flags.
        $vis type $bits = $crate::Bits<$bit>;

        impl const $crate::Bit for $bit {
            type Repr = $repr;

            const LIST: &'static [Self] = &[ $( $bit::$variant ),+ ];
            const COUNT: usize = Self::LIST.len();

            const ALL: $repr = $( ($bit::$variant as $repr) )|+;
            // The empty set is always the zero integer, independent of any
            // declared flag. Tying it to a variant (e.g. a `None = 1 << 0`
            // flag) would give a non-zero "empty" and break `is_empty`,
            // `contains`, `intersects` and iteration.
            const EMPTY: $repr = 0;
        }
        
        impl const $bit {
            #[allow(non_upper_case_globals)]
            /// The empty set of flags.
            pub const None: Self = Self::EMPTY.into();
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

        impl const core::convert::From<$repr> for $bit {
            fn from(value: $repr) -> Self {
               unsafe { std::mem::transmute(value) }
            }
        }

        impl const core::convert::Into<$crate::Bits<$bit>> for $bit {
            fn into(self) -> $crate::Bits<$bit> {
                 $crate::Bits::new(self)
            }
        }
    };
}
