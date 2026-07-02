use compact_str::CompactString;
use geometry::{Point, Size};
use utils::const_bitflags;

#[derive(Debug, Clone)]
#[derive_const(PartialEq, Eq)]
/// A color scheme change event.
pub enum ColorScheme {
    Dark,
    Light,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// A named key on the keyboard
    Key(KeyEvent),
    /// A mouse event.
    Pointer(PointerEvent),
    // /// A scroll event.
    Scroll(ScrollEvent),

    /// A terminal resize event.
    Resize(Size),
    /// A focus event.
    Focus,
    /// A blur event.
    Blur,
    // /// A paste event.
    // Paste(PasteEvent),
    // /// A copy event.
    // Copy(CopyEvent),

    /// A color scheme change event.
    ColorScheme(ColorScheme),

    Ignored(Vec<u8>),
    // /// An unknown sequence.
    // Unknown(Sequence)
}

#[derive(Default, Debug, Clone,PartialEq, Eq)]
pub struct KeyEvent {
    pub code: Code,
    pub kind: KeyKind,
    pub meta: Meta,
    pub repeat: bool,
}

impl KeyEvent {
    pub fn Up(code: Code) -> Self {
        Self {
            code,
            kind: KeyKind::Up,
           ..Default::default()
        }
    }
    pub fn Down(code: Code) -> Self {
        Self {
            code,
            kind: KeyKind::Down,
            ..Default::default()
        }
    }
}

#[derive(Default, Debug, Copy)]
#[derive_const(Clone, PartialEq, Eq)]
pub enum KeyKind {
    Up,
    #[default]
    Down
}


#[derive(Debug, Copy)]
#[derive_const(Clone, PartialEq, Eq, Default)]
pub struct PointerEvent {
    pub button: PointerButton,
    pub kind: PointerKind,
    pub meta: Meta,
    pub position: Point,
}

const impl PointerEvent {
    pub fn Up(button: PointerButton) -> Self {
        Self {
            button,
            kind: PointerKind::Up,
            ..Default::default()
        }
    }
    pub fn Down(button: PointerButton) -> Self {
        Self {
            button,
            kind: PointerKind::Down,
            ..Default::default()
        }
    }
}

#[derive(Debug, Copy)]
#[derive_const(Clone, PartialOrd, PartialEq, Eq, Default)]
pub enum PointerButton {
    #[default]
    None,
    Left,
    Middle,
    Right,
    WheelUp,
    WheelDown,
    WheelLeft,
    WheelRight,
    Backward,
    Forward,
    Button10,
    Button11,
}

#[derive(Debug, Copy)]
#[derive_const(Clone, PartialEq, Eq, Default)]
pub enum PointerKind {
    Up,
    #[default]
    Down,

    Over,
    Out,

    Enter,
    Leave,

    Move,
    Cancel,
}

#[derive(Default, Debug, Copy)]
#[derive_const(Clone, PartialEq, Eq)]
pub struct ScrollEvent {
    pub button: ScrollButton,
    pub kind: ScrollKind,
    pub meta: Meta,
    pub position: Point,
}

#[derive(Default, Debug, Copy)]
#[derive_const(Clone, PartialEq, Eq)]
pub enum ScrollButton {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

#[derive(Default, Debug, Copy)]
#[derive_const(Clone, PartialEq, Eq)]
pub enum ScrollKind {
    #[default]
    Move,
    End
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum Code {
    #[default]
    Null,
    Char(char),
    Extended(CompactString),

    Left,
    Right,
    Up,
    Down,

    Space,
    Enter,
    Backspace,
    Escape,
    Tab,
    Begin,
    CapsLock,
    Delete,
    End,
    Find,
    Select,

    F(u8),

    Home,
    Insert,
    Media(MediaCode),
    Keypad(u8),
    Menu,

    NumLock,
    PageDown,
    PageUp,
    Pause,
    PrintScreen,
    ScrollLock,
}
#[derive(Default, Debug, Copy)]
#[derive_const(Clone, PartialEq, Eq)]
pub enum MediaCode {
    #[default]
    None,

    FastForward,
    LowerVolume,
    MuteVolume,
    Pause,
    Play,
    PlayPause,
    RaiseVolume,
    Record,
    Reverse,
    Rewind,
    Stop,
    TrackNext,
    TrackPrevious,
}

const_bitflags! {
    pub struct Meta(u16);
    pub struct MetaIter;

    Shift = 0,
    Ctrl = 1,
    Alt = 2,
    Super = 3,
    Hyper = 4,
    Meta = 5,
    CapsLock = 6,
    NumLock = 7,
    ScrollLock = 8,
    Keypad = 9,
}
