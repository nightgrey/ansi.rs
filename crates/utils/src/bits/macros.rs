/// Define a flag enum and a concrete set newtype around it.
///
/// Emits, on the *caller's* side:
///
/// * `enum $bit` — the individual flags (with their explicit bit patterns, so
///   aliases / combined flags like `All = Bold as u16 | Italic as u16` work).
/// * `struct $bits($repr)` — a concrete, nameable set type. Because it is a
///   real struct rather than a `type` alias, callers can hang their own
///   inherent methods and trait impls off it.
///
/// All the set algebra comes from the [`BitSet`](crate::Bits) trait's default
/// methods; the macro only stamps out the wiring and the operator/conversion
/// impls that the orphan rules require to sit next to the concrete type.
#[macro_export]
macro_rules! bits {
    (
        $(#[$bits_meta:meta])* $bits:ident,
        $(#[$bit_meta:meta])* $vis:vis enum $bit:ident: $repr:ty {
            $($(#[$variant_meta:meta])* $variant:ident = $value:expr),+ $(,)?
        }
        $($bit_iter:ident),* $(,)?
        $($bit_error:ident),*
    ) => {
        // Iter & error types
        $($vis type $bit_iter = $crate::BitsIter<$bits>;)*
        $($vis type $bit_error = $crate::BitsError;)*

        // Bit
        $(#[$bit_meta])*
        #[repr($repr)]
        #[derive(Copy, Debug)]
        #[derive_const(Clone, PartialOrd, Ord, PartialEq, Eq)]
        $vis enum $bit { $($(#[$variant_meta])* $variant = $value),+ }


        // Bits
        $(#[$bits_meta])*
        /// A set of [`
        #[doc = stringify!($bit)]
        /// `] bits.
        #[repr(transparent)]
        #[derive(Copy)]
        #[derive_const(Clone, Default)]
        $vis struct $bits($repr);

        impl const $crate::Base for $bit {
           type Repr = $repr;

            #[inline]
            fn from_repr(repr: $repr) -> Self {
                unsafe { std::mem::transmute(repr) }
            }

            #[inline]
            fn into_repr(self) -> $repr {
                self as $repr
            }
        }
        impl const $crate::Base for $bits {
            type Repr = $repr;

            #[inline]
            fn from_repr(repr: $repr) -> Self {
                $bits(repr)
            }

            #[inline]
            fn into_repr(self) -> $repr {
                self.0
            }
        }

        impl const $crate::Bit for $bit {
            const LIST: &'static [(Self, &'static str)] = &[
                $(
                ($bit::$variant, stringify!($variant)),
                )*
            ];
         }
        impl const $crate::Bits for $bits {
            type Bit = $bit;

            // The empty set is always the zero integer, independent of any
            // declared flag. Tying it to a variant (e.g. a `None = 1 << 0`
            // flag) would give a non-zero "empty" and break `is_empty`,
            // `contains`, `intersects` and iteration.
            #[allow(non_upper_case_globals)]
            const None: Self = Self::from_repr(0);

            #[allow(non_upper_case_globals)]
            const All: Self = Self::from_repr($( ($bit::$variant as $repr) )|+);
        }

        #[allow(non_upper_case_globals)]
        impl $bits {
            $(
            pub const $variant: Self = Self($value as $repr);
            )*
        }

        impl core::str::FromStr for $bit {
            type Err = $crate::BitsError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                <$repr>::from_str_radix(s, 16)
                    .map_err(|_| $crate::BitsError::Invalid)
                    .map($crate::Base::from_repr)
            }
        }

        impl core::str::FromStr for $bits {
            type Err = $crate::BitsError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let mut parsed_flags = <Self as $crate::Bits>::None;

                // If the input is empty then return an empty set of flags
                if s.trim().is_empty() {
                    return Ok(parsed_flags);
                }

                for flag in s.split('|') {
                    let flag = flag.trim();

                    // If the flag is empty then we've got missing input
                    if flag.is_empty() {
                        return Err($crate::BitsError::Empty);
                    }

                    // If the flag starts with `0x` then it's a hex number;
                    // parse it directly to the underlying bits type. Otherwise
                    // look the flag up by name in the declared list.
                    let parsed_flag = if let Some(hex) = flag.strip_prefix("0x") {
                        <Self as $crate::Bits>::from_bits_retained(
                            <$repr>::from_str_radix(hex, 16)
                                .map_err(|_| $crate::BitsError::Invalid)?,
                        )
                    } else {
                        let mut found = None;
                        for (bit, name) in <Self as $crate::Bits>::LIST {
                            if *name == flag {
                                found = Some(<Self as $crate::Bits>::from_bits_retained(
                                    $crate::Base::into_repr(*bit),
                                ));
                                break;
                            }
                        }
                        found.ok_or($crate::BitsError::Invalid)?
                    };

                    $crate::Bits::insert(&mut parsed_flags, parsed_flag);
                }

                Ok(parsed_flags)
            }
        }


        // ---- flag-level operators (produce a set) --------------------------
        impl const std::ops::BitAnd for $bit {
            type Output = $bits;
            #[inline]
            fn bitand(self, rhs: Self) -> $bits {
                $bits((self as $repr) & (rhs as $repr))
            }
        }

        impl const std::ops::BitOr for $bit {
            type Output = $bits;
            #[inline]
            fn bitor(self, rhs: Self) -> $bits {
                $bits((self as $repr) | (rhs as $repr))
            }
        }

        impl const std::ops::BitXor for $bit {
            type Output = $bits;
            #[inline]
            fn bitxor(self, rhs: Self) -> $bits {
                $bits((self as $repr) ^ (rhs as $repr))
            }
        }

        impl const std::ops::Not for $bit {
            type Output = $bits;
            #[inline]
            fn not(self) -> $bits {
                $bits(!(self as $repr) & $bits::All.bits())
            }
        }

        // ---- conversions ---------------------------------------------------
        impl const core::convert::From<$bit> for $repr {
            #[inline]
            fn from(value: $bit) -> Self {
                value as $repr
            }
        }

        impl const core::convert::Into<$bits> for $bit {
            #[inline]
            fn into(self) -> $bits {
                $bits(self as $repr)
            }
        }

        // ---- set-level operators -------------------------------------------
        impl<I: [ const ] Into<$bits>> const std::ops::BitOr<I> for $bits {
            type Output = Self;
            #[inline]
            fn bitor(self, rhs: I) -> Self {
                $crate::Bits::union(self, rhs)
            }
        }
        impl<I: [ const ] Into<$bits>> const std::ops::BitOrAssign<I> for $bits {
            #[inline]
            fn bitor_assign(&mut self, rhs: I) {
                *self = $crate::Bits::union(*self, rhs);
            }
        }

        impl<I: [ const ] Into<$bits>> const std::ops::BitAnd<I> for $bits {
            type Output = Self;
            #[inline]
            fn bitand(self, rhs: I) -> Self {
                $crate::Bits::intersection(self, rhs)
            }
        }
        impl<I: [ const ] Into<$bits>> const std::ops::BitAndAssign<I> for $bits {
            #[inline]
            fn bitand_assign(&mut self, rhs: I) {
                *self = $crate::Bits::intersection(*self, rhs);
            }
        }

        impl<I: [ const ] Into<$bits>> const std::ops::Sub<I> for $bits {
            type Output = Self;
            #[inline]
            fn sub(self, rhs: I) -> Self {
                $crate::Bits::difference(self, rhs)
            }
        }
        impl<I: [ const ] Into<$bits>> const std::ops::SubAssign<I> for $bits {
            #[inline]
            fn sub_assign(&mut self, rhs: I) {
                *self = $crate::Bits::difference(*self, rhs);
            }
        }

        impl<I: [ const ] Into<$bits>> const std::ops::BitXor<I> for $bits {
            type Output = Self;
            #[inline]
            fn bitxor(self, rhs: I) -> Self {
                $crate::Bits::symmetric_difference(self, rhs)
            }
        }
        impl<I: [ const ] Into<$bits>> const std::ops::BitXorAssign<I> for $bits {
            #[inline]
            fn bitxor_assign(&mut self, rhs: I) {
                *self = $crate::Bits::symmetric_difference(*self, rhs);
            }
        }

        impl<I: [ const ] Into<$bits>> const std::ops::Rem<I> for $bits {
            type Output = Self;
            #[inline]
            fn rem(self, rhs: I) -> Self {
                $crate::Bits::symmetric_difference(self, rhs)
            }
        }
        impl<I: [ const ] Into<$bits>> const std::ops::RemAssign<I> for $bits {
            #[inline]
            fn rem_assign(&mut self, rhs: I) {
                *self = $crate::Bits::symmetric_difference(*self, rhs);
            }
        }

        impl const std::ops::Not for $bits {
            type Output = Self;
            #[inline]
            fn not(self) -> Self {
                $crate::Bits::complement(self)
            }
        }

        // ---- equality (against anything that converts into the set) --------
        impl<I: core::marker::Copy + [ const ] Into<$bits>> const core::cmp::PartialEq<I> for $bits {
            #[inline]
            fn eq(&self, other: &I) -> bool {
                self.0 == $crate::Base::into_repr((*other).into())
            }
        }

        impl core::cmp::Eq for $bits {}

        // ---- debug ---------------------------------------------------------
        impl core::fmt::Debug for $bits {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}(", stringify!($bits))?;
                for (i, flag) in $crate::Bits::iter(*self).enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{flag:?}")?;
                }
                write!(f, ")")
            }
        }

        // ---- iteration -----------------------------------------------------
        impl const core::iter::IntoIterator for $bits {
            type Item = $bit;
            type IntoIter = $crate::BitsIter<$bits>;
            #[inline]
            fn into_iter(self) -> Self::IntoIter {
                $crate::BitsIter::new(self)
            }
        }

        impl<I: Into<$bits>> Extend<I> for $bits {
            fn extend<T: IntoIterator<Item=I>>(&mut self, iter: T) {
                for item in iter {
                    $crate::Bits::insert(self, item);
                }
            }
        }

        impl<I: Into<$bits>> FromIterator<I> for $bits {
            fn from_iter<T: IntoIterator<Item=I>>(iter: T) -> Self {
                let mut set = <$bits as $crate::Bits>::None;
                set.extend(iter);
                set
            }
        }

        // REST
    };

}

