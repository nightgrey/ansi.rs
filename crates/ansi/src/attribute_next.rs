use flagset::{FlagSet, flags};

flags! {
    enum Attribute: u32 {
         /// Resets all the attributes.
        Reset = 1 << 0,
        /// Increases the text intensity.
        Bold = 1 << 1,
        /// Decreases the text intensity.
        Faint = 1 << 2,
        /// Emphasises the text.
        Italic = 1 << 3,
        /// Underlines the text.
        Underline = 1 << 4,
        /// Makes the text blink.
        Blink = 1 << 5,
        /// Makes the text blink rapidly.
        RapidBlink = 1 << 6,
        /// Swaps the foreground and background colors.
        Reverse = 1 << 7,
        /// Hides the text.
        Conceal = 1 << 8,
        /// Crosses the text out.
        Strikethrough = 1 << 9,

        /// Sets the underline style to "none".
        UnderlineStyleNone = 1 << 10,
        /// Sets the underline style to "single".
        UnderlineStyleSingle = 1 << 11,
        /// Sets the underline style to "double".
        UnderlineStyleDouble = 1 << 12,
        /// Sets the underline style to "curly".
        UnderlineStyleCurly = 1 << 13,
        /// Sets the underline style to "dotted".
        UnderlineStyleDotted = 1 << 14,
        /// Sets the underline style to "dashed".
        UnderlineStyleDashed = 1 << 15,

        /// Turns off the `Bold` attribute.
        NoBold = 1 << 16,
        /// Turns off the `Italic` and `Bold` attributes.
        NormalIntensity = 1 << 17,
        /// Turns off the `Italic` attribute.
        NoItalic = 1 << 18,
        /// Turns off the `Underline` attribute.
        NoUnderline = 1 << 19,
        /// Turns off the text blinking.
        NoBlink = 1 << 20,
        /// Turns off the `Reverse` and `Conceal` attributes.
        NoReverse = 1 << 21,
        /// Turns off the `Conceal` attribute.
        NoConceal = 1 << 22,
        /// Turns off the `Strikethrough` attribute.
        NoStrikethrough = 1 << 23,

        /// Frames the text.
        Frame = 1 << 24,
        /// Encircles the text.
        Encircle = 1 << 25,
        /// Draws a line at the top of the text.
        Overline = 1 << 26,
        /// Turns off the `Frame` and `Encircle` attributes.
        NoFrameOrEncircle = 1 << 27,
        /// Turns off the `Overline` attribute.
        NoOverline = 1 << 28,
    }
}


#[test]
fn qwe() {
    let attributes = Attribute::Blink;

}