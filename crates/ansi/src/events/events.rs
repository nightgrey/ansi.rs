use bitflags::bitflags;
use geometry::{Point, Size};
use utils::const_bitflags;

#[derive(Debug)]
#[derive(Clone, PartialEq, Eq)]
pub enum Event {
    /// A named key on the keyboard
    Key(KeyEvent),
    /// A mouse event.
    Pointer(PointerEvent),
    // /// A scroll event.
    Scroll(ScrollEvent),

    Resize(Size),
    // /// A focus event.
    // Focus(FocusEvent),
    // /// A blur event.
    // Blur(BlurEvent),
    // /// A paste event.
    // Paste(PasteEvent),
    // /// A copy event.
    // Copy(CopyEvent),
    //
    // /// An unknown sequence.
    // Unknown(Sequence)
}

#[derive(Debug)]
#[derive(Clone, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: Key,
    pub kind: KeyKind,
    pub meta: Meta,
}

#[derive(Debug)]
#[derive(Clone, PartialEq, Eq)]
pub enum KeyKind {}


#[derive(Debug)]
#[derive(Clone, PartialEq, Eq)]
pub struct PointerEvent {
    pub button: PointerButton,
    pub kind: PointerKind,
    pub meta: Meta,
    pub position: Point,
}

#[derive(Debug)]
#[derive(Clone, PartialEq, Eq)]
pub enum PointerButton {}

#[derive(Debug)]
#[derive(Clone, PartialEq, Eq)]
pub enum PointerKind {}

#[derive(Debug)]
#[derive(Clone, PartialEq, Eq)]
pub struct ScrollEvent {
    pub kind: ScrollKind,
    pub meta: Meta,
    pub position: Point,
}

#[derive(Debug)]
#[derive(Clone, PartialEq, Eq)]
pub enum ScrollKind {}

#[derive(Debug)]
#[derive(Clone, PartialEq, Eq)]
pub enum Key {
}

const_bitflags! {
    pub struct Meta(u16);
    pub struct MetaIter;
    
    Alt = 0,
    Ctrl = 1,
    Shift = 2,
    CapsLock = 3,
    NumLock = 4,
    ScrollLock = 5,
}