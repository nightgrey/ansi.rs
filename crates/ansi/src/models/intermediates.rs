use derive_more::{AsMut, Deref, DerefMut, Index, IndexMut, IntoIterator};
use smallvec::SmallVec;
use std::borrow::{Borrow, BorrowMut};
use std::fmt;
use std::mem::MaybeUninit;

#[derive(Copy, Hash, Debug)]
#[derive_const(Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
#[repr(u8)]
pub enum Intermediate {
    #[default]
    /// U+0000 (The default variant)
    Null = 0,
    /// U+0001
    StartOfHeading = 1,
    /// U+0002
    StartOfText = 2,
    /// U+0003
    EndOfText = 3,
    /// U+0004
    EndOfTransmission = 4,
    /// U+0005
    Enquiry = 5,
    /// U+0006
    Acknowledge = 6,
    /// U+0007
    Bell = 7,
    /// U+0008
    Backspace = 8,
    /// U+0009
    CharacterTabulation = 9,
    /// U+000A
    LineFeed = 10,
    /// U+000B
    LineTabulation = 11,
    /// U+000C
    FormFeed = 12,
    /// U+000D
    CarriageReturn = 13,
    /// U+000E
    ShiftOut = 14,
    /// U+000F
    ShiftIn = 15,
    /// U+0010
    DataLinkEscape = 16,
    /// U+0011
    DeviceControlOne = 17,
    /// U+0012
    DeviceControlTwo = 18,
    /// U+0013
    DeviceControlThree = 19,
    /// U+0014
    DeviceControlFour = 20,
    /// U+0015
    NegativeAcknowledge = 21,
    /// U+0016
    SynchronousIdle = 22,
    /// U+0017
    EndOfTransmissionBlock = 23,
    /// U+0018
    Cancel = 24,
    /// U+0019
    EndOfMedium = 25,
    /// U+001A
    Substitute = 26,
    /// U+001B
    Escape = 27,
    /// U+001C
    InformationSeparatorFour = 28,
    /// U+001D
    InformationSeparatorThree = 29,
    /// U+001E
    InformationSeparatorTwo = 30,
    /// U+001F
    InformationSeparatorOne = 31,
    /// U+0020
    Space = 32,
    /// U+0021
    ExclamationMark = 33,
    /// U+0022
    QuotationMark = 34,
    /// U+0023
    NumberSign = 35,
    /// U+0024
    DollarSign = 36,
    /// U+0025
    PercentSign = 37,
    /// U+0026
    Ampersand = 38,
    /// U+0027
    Apostrophe = 39,
    /// U+0028
    LeftParenthesis = 40,
    /// U+0029
    RightParenthesis = 41,
    /// U+002A
    Asterisk = 42,
    /// U+002B
    PlusSign = 43,
    /// U+002C
    Comma = 44,
    /// U+002D
    HyphenMinus = 45,
    /// U+002E
    FullStop = 46,
    /// U+002F
    Solidus = 47,
    /// U+0030
    Digit0 = 48,
    /// U+0031
    Digit1 = 49,
    /// U+0032
    Digit2 = 50,
    /// U+0033
    Digit3 = 51,
    /// U+0034
    Digit4 = 52,
    /// U+0035
    Digit5 = 53,
    /// U+0036
    Digit6 = 54,
    /// U+0037
    Digit7 = 55,
    /// U+0038
    Digit8 = 56,
    /// U+0039
    Digit9 = 57,
    /// U+003A
    Colon = 58,
    /// U+003B
    Semicolon = 59,
    /// U+003C
    LessThanSign = 60,
    /// U+003D
    EqualsSign = 61,
    /// U+003E
    GreaterThanSign = 62,
    /// U+003F
    QuestionMark = 63,
    /// U+0040
    CommercialAt = 64,
    /// U+0041
    CapitalA = 65,
    /// U+0042
    CapitalB = 66,
    /// U+0043
    CapitalC = 67,
    /// U+0044
    CapitalD = 68,
    /// U+0045
    CapitalE = 69,
    /// U+0046
    CapitalF = 70,
    /// U+0047
    CapitalG = 71,
    /// U+0048
    CapitalH = 72,
    /// U+0049
    CapitalI = 73,
    /// U+004A
    CapitalJ = 74,
    /// U+004B
    CapitalK = 75,
    /// U+004C
    CapitalL = 76,
    /// U+004D
    CapitalM = 77,
    /// U+004E
    CapitalN = 78,
    /// U+004F
    CapitalO = 79,
    /// U+0050
    CapitalP = 80,
    /// U+0051
    CapitalQ = 81,
    /// U+0052
    CapitalR = 82,
    /// U+0053
    CapitalS = 83,
    /// U+0054
    CapitalT = 84,
    /// U+0055
    CapitalU = 85,
    /// U+0056
    CapitalV = 86,
    /// U+0057
    CapitalW = 87,
    /// U+0058
    CapitalX = 88,
    /// U+0059
    CapitalY = 89,
    /// U+005A
    CapitalZ = 90,
    /// U+005B
    LeftSquareBracket = 91,
    /// U+005C
    ReverseSolidus = 92,
    /// U+005D
    RightSquareBracket = 93,
    /// U+005E
    CircumflexAccent = 94,
    /// U+005F
    LowLine = 95,
    /// U+0060
    GraveAccent = 96,
    /// U+0061
    SmallA = 97,
    /// U+0062
    SmallB = 98,
    /// U+0063
    SmallC = 99,
    /// U+0064
    SmallD = 100,
    /// U+0065
    SmallE = 101,
    /// U+0066
    SmallF = 102,
    /// U+0067
    SmallG = 103,
    /// U+0068
    SmallH = 104,
    /// U+0069
    SmallI = 105,
    /// U+006A
    SmallJ = 106,
    /// U+006B
    SmallK = 107,
    /// U+006C
    SmallL = 108,
    /// U+006D
    SmallM = 109,
    /// U+006E
    SmallN = 110,
    /// U+006F
    SmallO = 111,
    /// U+0070
    SmallP = 112,
    /// U+0071
    SmallQ = 113,
    /// U+0072
    SmallR = 114,
    /// U+0073
    SmallS = 115,
    /// U+0074
    SmallT = 116,
    /// U+0075
    SmallU = 117,
    /// U+0076
    SmallV = 118,
    /// U+0077
    SmallW = 119,
    /// U+0078
    SmallX = 120,
    /// U+0079
    SmallY = 121,
    /// U+007A
    SmallZ = 122,
    /// U+007B
    LeftCurlyBracket = 123,
    /// U+007C
    VerticalLine = 124,
    /// U+007D
    RightCurlyBracket = 125,
    /// U+007E
    Tilde = 126,
    /// U+007F
    Delete = 127,
}
impl Intermediate {
    /// The character with the lowest ASCII code.
    pub const MIN: Self = Self::Null;

    /// The character with the highest ASCII code.
    pub const MAX: Self = Self::Solidus;

    /// Creates an ASCII character from the byte `b`,
    /// or returns `None` if it's too large.
    #[inline]
    pub const fn from_byte(b: u8) -> Option<Self> {
        if b <= 127 {
            // SAFETY: Just checked that `b` is in-range
            Some(unsafe { Self::from_byte_unchecked(b) })
        } else {
            None
        }
    }

    /// Creates an ASCII character from the byte `b`,
    /// without checking whether it's valid.
    ///
    /// # Safety
    ///
    /// `b` must be in `0..=127`, or else this is UB.
    #[inline]
    pub const unsafe fn from_byte_unchecked(b: u8) -> Self {
        // SAFETY: Our safety precondition is that `b` is in-range.
        unsafe { std::mem::transmute(b) }
    }

    #[inline]
    pub const fn from_byte_or(b: u8, default: Self) -> Self {
        Self::from_byte(b).unwrap_or(default)
    }

    #[inline]
    pub const fn from_byte_or_default(b: u8) -> Self {
        Self::from_byte(b).unwrap_or_default()
    }

    /// Gets this ASCII character as a byte.
    #[inline]
    pub const fn to_byte(self) -> u8 {
        self as u8
    }

    /// Gets this ASCII character as a `char` Unicode Scalar Value.
    #[inline]
    pub const fn to_char(self) -> char {
        self as u8 as char
    }

    /// Gets this ASCII character as a `char` Unicode Scalar Value.
    #[inline]
    pub const fn to_ascii_char(self) -> std::ascii::Char {
        unsafe { std::ascii::Char::from_u8_unchecked(self as u8) }
    }

    /// Views this ASCII character as a one-code-unit UTF-8 `str`.
    #[inline]
    pub const fn as_str(&self) -> &str {
        std::slice::from_ref(unsafe { &*(self as *const Self as *const std::ascii::Char) }).as_str()
    }
}

macro_rules! into_int_impl {
    ($($ty:ty)*) => {
        $(
            const impl From<Intermediate> for $ty {
                #[inline]
                fn from(chr: Intermediate) -> $ty {
                    chr as u8 as $ty
                }
            }
        )*
    }
}

into_int_impl!(u8 u16 u32 u64 u128 char);

impl AsRef<u8> for Intermediate {
    #[inline]
    fn as_ref(&self) -> &u8 {
        unsafe { &*(self as *const Self as *const u8) }
    }
}

#[derive(
    Eq, Hash, Clone, PartialOrd, Ord, Deref, DerefMut, Index, IndexMut, AsMut, IntoIterator,
)]
#[as_mut(forward)]
#[into_iterator(owned, ref, ref_mut)]
pub struct Intermediates<const N: usize = 2>(SmallVec<Intermediate, N>);

impl<const N: usize> Intermediates<N> {
    #[inline]
    pub const fn empty() -> Self {
        Self(SmallVec::new())
    }

    #[inline]
    pub fn new(intermediates: &[Intermediate]) -> Self {
        Self(SmallVec::from_slice_copy(&intermediates))
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(SmallVec::with_capacity(capacity))
    }

    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(SmallVec::from_iter(bytes.utf8_chunks().flat_map(|b| {
            b.valid()
                .chars()
                .map(|c| Intermediate::from_byte_or_default(c as u8))
        })))
    }

    #[inline]
    pub fn from_str(str: &str) -> Self {
        Self(SmallVec::from_iter(
            str.chars()
                .map(|c| Intermediate::from_byte_or_default(c as u8)),
        ))
    }

    /// Constructs a new instance on the stack from an array without copying elements."
    #[inline]
    pub const fn from_array<const M: usize>(value: [Intermediate; M]) -> Self {
        Self(SmallVec::from_buf(value))
    }

    /// Constructs a new instance on the stack from an array without copying elements. Also sets the length, which must be less or equal to the size of buf."
    #[inline]
    pub fn from_array_and_len(value: [Intermediate; N], len: usize) -> Self {
        Self(SmallVec::from_buf_and_len(value, len))
    }

    /// Constructs a new instance on the stack from an array without copying elements. Also sets the length. The user is responsible for ensuring that `len <= N`."
    ///
    /// # Safety"
    /// - The user is responsible for ensuring that `len <= N`."
    #[inline]
    pub const unsafe fn from_const_with_len_unchecked(
        value: MaybeUninit<[Intermediate; N]>,
        len: usize,
    ) -> Self {
        Self(SmallVec::from_buf_and_len_unchecked(value, len))
    }

    #[inline]
    pub const fn as_slice(&self) -> &[Intermediate] {
        self.0.as_slice()
    }

    #[inline]
    pub const fn as_mut_slice(&mut self) -> &mut [Intermediate] {
        self.0.as_mut_slice()
    }

    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        unsafe { &*(self.0.as_slice() as *const [Intermediate] as *const [u8]) }
    }

    #[inline]
    pub const fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.as_bytes()) }
    }
}

const impl<const N: usize> Default for Intermediates<N> {
    fn default() -> Self {
        Self::empty()
    }
}

const impl<const N: usize> AsRef<[u8]> for Intermediates<N> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

const impl<const N: usize> AsRef<str> for Intermediates<N> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

const impl<const N: usize> AsRef<[Intermediate]> for Intermediates<N> {
    #[inline]
    fn as_ref(&self) -> &[Intermediate] {
        self.0.as_slice()
    }
}

const impl<const N: usize> Borrow<[Intermediate]> for Intermediates<N> {
    fn borrow(&self) -> &[Intermediate] {
        self.as_slice()
    }
}

const impl<const N: usize> BorrowMut<[Intermediate]> for Intermediates<N> {
    fn borrow_mut(&mut self) -> &mut [Intermediate] {
        self.as_mut_slice()
    }
}

const impl<const N: usize> Borrow<str> for Intermediates<N> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> FromIterator<Intermediate> for Intermediates<N> {
    fn from_iter<__T: IntoIterator<Item = Intermediate>>(iter: __T) -> Self {
        Self(iter.into_iter().collect())
    }
}
impl<const N: usize> Extend<Intermediate> for Intermediates<N> {
    #[inline]
    fn extend<I: IntoIterator<Item = Intermediate>>(&mut self, iter: I) {
        self.0.extend(iter)
    }
}

impl<const N: usize> fmt::Display for Intermediates<N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl<const N: usize> fmt::Debug for Intermediates<N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

const impl<const N: usize> PartialEq<Intermediates<N>> for [u8] {
    #[inline]
    fn eq(&self, other: &Intermediates<N>) -> bool {
        self == other.as_bytes()
    }
}

const impl<const N: usize> PartialEq<&[u8]> for Intermediates<N> {
    #[inline]
    fn eq(&self, other: &&[u8]) -> bool {
        self.as_bytes() == *other
    }
}

const impl<const N: usize> PartialEq<Intermediates<N>> for &[u8] {
    #[inline]
    fn eq(&self, other: &Intermediates<N>) -> bool {
        *self == other.as_bytes()
    }
}

impl<const N: usize, const M: usize> PartialEq<[u8; M]> for Intermediates<N> {
    #[inline]
    fn eq(&self, other: &[u8; M]) -> bool {
        self.as_bytes() == other
    }
}

impl<const N: usize, const M: usize> PartialEq<Intermediates<N>> for [u8; M] {
    #[inline]
    fn eq(&self, other: &Intermediates<N>) -> bool {
        self.as_slice() == other.as_bytes()
    }
}

const impl<const N: usize> PartialEq<str> for Intermediates<N> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}
const impl<const N: usize> PartialEq<Intermediates<N>> for str {
    #[inline]
    fn eq(&self, other: &Intermediates<N>) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

const impl<const N: usize> PartialEq<&str> for Intermediates<N> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

const impl<const N: usize> PartialEq<Intermediates<N>> for &str {
    #[inline]
    fn eq(&self, other: &Intermediates<N>) -> bool {
        *self == other.as_str()
    }
}

const impl<const N: usize> PartialEq<String> for Intermediates<N> {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

const impl<const N: usize> PartialEq<Intermediates<N>> for String {
    #[inline]
    fn eq(&self, other: &Intermediates<N>) -> bool {
        self.as_str() == other.as_str()
    }
}

const impl<const N: usize> PartialEq<Vec<u8>> for Intermediates<N> {
    #[inline]
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.as_bytes() == other.as_slice()
    }
}

const impl<const N: usize> PartialEq<Intermediates<N>> for Vec<u8> {
    #[inline]
    fn eq(&self, other: &Intermediates<N>) -> bool {
        self.as_slice() == other.as_bytes()
    }
}

const impl<const N: usize, const M: usize> PartialEq<Intermediates<M>> for Intermediates<N> {
    #[inline]
    fn eq(&self, other: &Intermediates<M>) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

const impl<const N: usize> From<Intermediate> for Intermediates<N> {
    fn from(value: Intermediate) -> Self {
        Self(SmallVec::from_buf([value; 1]))
    }
}

impl<const N: usize> From<&[u8]> for Intermediates<N> {
    #[inline]
    fn from(value: &[u8]) -> Self {
        Self::from_bytes(value)
    }
}

impl<const N: usize, const M: usize> From<&[u8; M]> for Intermediates<N> {
    #[inline]
    fn from(value: &[u8; M]) -> Self {
        Self::from_bytes(&value[..])
    }
}

impl<const N: usize> From<&str> for Intermediates<N> {
    #[inline]
    fn from(value: &str) -> Self {
        Self::from_str(value)
    }
}

impl<const N: usize> From<String> for Intermediates<N> {
    #[inline]
    fn from(value: String) -> Self {
        Self::from_str(&value)
    }
}

impl<const N: usize> From<Vec<u8>> for Intermediates<N> {
    #[inline]
    fn from(value: Vec<u8>) -> Self {
        Self::from_bytes(&value)
    }
}
