use std::io;

/// Creates an ANSI sequence.
///
/// You can use this macro to create your own ANSI sequence. All sequences are
/// created with this macro.
///
/// # Credits
//  https://github.com/qwandor/anes-rs/blob/main/anes/src/macros.rs
///
/// # Examples
///
/// An unit struct:
///
/// ```
/// use ansi::{command};
///
/// sequence!(
///   /// Saves the cursor position.
///   struct SaveCursorPosition => "\x1B7"
/// );
///
/// assert_eq!(&format!("{}", SaveCursorPosition), "\x1B7");
/// ```
///
/// An enum:
///
/// ```
/// use ansi::{csi, command};
///
/// sequence!(
///     /// Clears part of the buffer.
///     enum ClearBuffer {
///         /// Clears from the cursor position to end of the screen.
///         Below => "\x1BJ",
///         /// Clears from the cursor position to beginning of the screen.
///         Above => "\x1B1J",
///         /// Clears the entire buffer.
///         All => "\x1B2J",
///         /// Clears the entire buffer and all saved lines in the scrollback buffer.
///         SavedLines => "\x1B3J",
///     }
/// );
///
/// assert_eq!(&format!("{}", ClearBuffer::Below), "\x1B[J");
/// assert_eq!(&format!("{}", ClearBuffer::Above), "\x1B[1J");
/// assert_eq!(&format!("{}", ClearBuffer::All), "\x1B[2J");
/// assert_eq!(&format!("{}", ClearBuffer::SavedLines), "\x1B[3J");
/// ```
///
/// A dynamic struct:
///
/// ```
/// use ansi::{csi, command};
///
/// sequence!(
///     /// Moves the cursor to the given location (column, row).
///     ///
///     /// # Notes
///     ///
///     /// Top/left cell is represented as `1, 1`.
///     struct MoveCursorTo(u16, u16) =>
///     |this, f| write!(f, "\x1B{};{}H", this.0, this.1)
/// );
///
/// assert_eq!(&format!("{}", MoveCursorTo(10, 5)), "\x1B[10;5H");
/// ```
#[macro_export]
macro_rules! sequence {
    // Field-less struct
    // `struct Foo`
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident => |$this:ident, $w:ident| $write:block
    ) => {
        $(#[$meta])*
            #[derive(Copy, Clone, Debug,  PartialEq)]
        $vis struct $name;
        
        impl $crate::Escape for $name {
            fn escape(&self, $w: &mut impl std::io::Write) -> std::io::Result<()> {
                let $this = self;
                $write
            }
        }
    };
    // Transparent struct
    // `struct Foo(field)`
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident(
            $(#[$field_meta:meta])*
            $field_vis:vis
            $field:ty
        )  => |$this:ident, $w:ident| $write:block
    ) => {
        $(#[$meta])*
        #[derive(Copy, Clone, Debug,  PartialEq, derive_more::Constructor, derive_more::From, derive_more::Into)]
        #[repr(transparent)]
        $vis struct $name($(#[$field_meta])* $field_vis $field);

        impl const $name {
            #[inline]
            pub fn value(&self) -> $field {
                self.0
            }
        }
        
        impl $crate::Escape for $name {
            fn escape(&self, $w: &mut impl std::io::Write) -> std::io::Result<()> {
                let $this = self;
                $write
            }
        }
    };

    // Multi-field struct
    // `struct Foo(field1, field2, ...)`
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident(
            $(
                $(#[$fields_meta:meta])*
                $field_vis:vis
                $fields:ty
            ),*
            $(,)?
        ) => |$this:ident, $w:ident| $write:block
    ) => {
        $(#[$meta])*
        #[derive(Copy, Clone, Debug, Eq, PartialEq, derive_more::Constructor, derive_more::From, derive_more::Into)]
        $vis struct $name($($(#[$fields_meta])* $field_vis $fields),*);
        
        impl $crate::Escape for $name {
            fn escape(&self, $w: &mut impl std::io::Write) -> std::io::Result<()> {
                let $this = self;
                $write
            }
        }
    };
    // Struct
    // `struct Foo { fields, .. }`
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$fields_meta:meta])*
                $field_vis:vis
                $fields:ident: $fields_ty:ty,
            )*
        } => |$this:ident, $w:ident| $write:block
    ) => {
        $(#[$meta])*
        #[derive(Copy, Clone, Debug, Eq, PartialEq, derive_more::Constructor, derive_more::From, derive_more::Into)]
        $vis struct $name {
            $(
                $(#[$fields_meta])*
                pub $fields: $fields_ty,
            )*
        }
        
        impl $crate::Escape for $name {
            fn escape(&self, $w: &mut impl std::io::Write) -> std::io::Result<()> {
                let $this = self;
                $write
            }
        }
    };


    // Enum
    // `enum Foo { field = value, .. }`
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident = $variant_value:expr
            ),*
            $(,)?
        } => |$this:ident, $w:ident| $write:block
    ) => {
        $(#[$meta])*
        #[derive(Copy, Clone, Debug, PartialEq)]
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant = $variant_value,
            )*
        }
        
        impl $crate::Escape for $name {
            fn escape(&self, $w: &mut impl std::io::Write) -> std::io::Result<()> {
                let $this = self;
                $write
            }
        }
    };
    // `enum Foo { field, .. }`
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident
            ),*
            $(,)?
        } => |$this:ident, $w:ident| $write:block
    ) => {
        $(#[$meta])*
        #[derive(Copy, Clone, Debug, PartialEq)]
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant,
            )*
        }
        
        impl $crate::Escape for $name {
            fn escape(&self, $w: &mut impl std::io::Write) -> std::io::Result<()> {
                let $this = self;
                $write
            }
        }
    };
}

macro_rules! sequence_only {
    // Field-less struct
    // `struct Foo`
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident
    ) => {
        $(#[$meta])*
            #[derive(Copy, Clone, Debug,  PartialEq)]
        $vis struct $name;
    };
    // Transparent struct
    // `struct Foo(field)`
    (
        $(#[$meta:meta])*
        $vis:vis struct $type:ident(
            $(#[$field_meta:meta])*
            $field_vis:vis
            $field:ty
        )
    ) => {
        $(#[$meta])*
        #[derive(Copy, Clone, Debug,  PartialEq, derive_more::Constructor, derive_more::From, derive_more::Into)]
        #[repr(transparent)]
        $vis struct $type($(#[$field_meta])* $field_vis $field);

        impl const $type {
            #[inline]
            pub fn value(&self) -> $field {
                self.0
            }
        }

    };

    // Multi-field struct
    // `struct Foo(field1, field2, ...)`
    (
        $(#[$meta:meta])*
        $vis:vis struct $type:ident(
            $(
                $(#[$fields_meta:meta])*
                $field_vis:vis
                $fields:ty
            ),*
            $(,)?
        )
    ) => {
        $(#[$meta])*
        #[derive(Copy, Clone, Debug, Eq, PartialEq, derive_more::Constructor, derive_more::From, derive_more::Into)]
        $vis struct $type($($(#[$fields_meta])* $field_vis $fields),*);

    };
    // Struct
    // `struct Foo { fields, .. }`
    (
        $(#[$meta:meta])*
        $vis:vis struct $type:ident {
            $(
                $(#[$fields_meta:meta])*
                $field_vis:vis
                $fields:ident: $fields_ty:ty,
            )*
        }
    ) => {
        $(#[$meta])*
        #[derive(Copy, Clone, Debug, Eq, PartialEq, derive_more::Constructor, derive_more::From, derive_more::Into)]
        $vis struct $type {
            $(
                $(#[$fields_meta])*
                pub $fields: $fields_ty,
            )*
        }

    };


    // Enum
    // `enum Foo { field = value, .. }`
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident = $variant_value:expr
            ),*
            $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Copy, Clone, Debug, PartialEq)]
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant = $variant_value,
            )*
        }
    };
    // `enum Foo { field, .. }`
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident
            ),*
            $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Copy, Clone, Debug, PartialEq)]
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant,
            )*
        }
    };
}

#[macro_export]
macro_rules! cost{
    ($ty:ident => $calc:expr) => {
        impl $crate::Cost for $ty {
            #[inline]
            fn cost(&self) -> usize {
                $calc(self)
            }
        }
    };
}
