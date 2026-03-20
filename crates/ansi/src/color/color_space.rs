/// The color space.
///
/// Defines the color space of a color.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub enum ColorSpace {
    #[default]
    None,
    Ansi,
    Rgb,
}