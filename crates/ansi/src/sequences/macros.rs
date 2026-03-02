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
    // struct Foo;
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident => $write:expr
    ) => {
        $(#[$meta])*
            #[derive(Copy, Clone, Debug,  PartialEq)]
        $vis struct $name;

        impl $crate::Escape for $name {
            fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
                $write(self, w)
            }
        }
    };
    // enum Foo { .. }
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident = $variant_value:expr
            ),*
            $(,)?
        } => $write:expr
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
            fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
                $write(self, w)
            }
        }
    };
    // struct Foo(field)
    (
        $(#[$meta:meta])*
        $vis:vis struct $type:ident(
            $(#[$field_meta:meta])*
            $field_vis:vis
            $field:ty
        ) => $write:expr
    ) => {
        $(#[$meta])*
        #[derive(Copy, Clone, Debug,  PartialEq, derive_more::Constructor, derive_more::From, derive_more::Into)]
        #[repr(transparent)]
        $vis struct $type($(#[$field_meta])* $field_vis $field);

        impl $crate::Escape for $type {
            fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
                $write(self, w)
            }
        }
    };
    // struct Foo(field1, field2, ...)
    (
        $(#[$meta:meta])*
        $vis:vis struct $type:ident(
            $(
                $(#[$fields_meta:meta])*
                $field_vis:vis
                $fields:ty
            ),*
            $(,)?
        ) => $write:expr
    ) => {
        $(#[$meta])*
        #[derive(Copy, Clone, Debug, Eq, PartialEq, derive_more::Constructor, derive_more::From, derive_more::Into)]
        $vis struct $type($($(#[$fields_meta])* $field_vis $fields),*);

        impl $crate::Escape for $type {
            fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
                $write(self, w)
            }
        }
    };
    
}
