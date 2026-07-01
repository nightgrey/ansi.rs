use crate::{Grapheme, Graphemes, GraphemesError};

/// A source value that can be converted into an [`Grapheme`].
///
/// See [`ExtendedSource`] and [`InlineSource`] for more details.
///
/// # Examples
///
/// ```
/// # use crate::{Encode, Grapheme, Graphemes};
/// let extended = Grapheme::new("👩‍👩‍👧‍👦".extended(&mut Graphemes::new()));
/// let inline = Grapheme::new('A');
/// ```
pub const trait Source {
    fn try_into(self) -> Result<Grapheme, GraphemesError>;
    fn into(self) -> Grapheme
    where
        Self: Sized,
    {
        match self.try_into() {
            Ok(g) => g,
            Err(_err) => panic!("failed to convert into grapheme"),
        }
    }
}

const impl Source for Grapheme {
    fn try_into(self) -> Result<Grapheme, GraphemesError> {
        Ok(self)
    }
}

const impl Source for char {
    fn try_into(self) -> Result<Grapheme, GraphemesError> {
        Ok(Source::into(self))
    }
    
    fn into(self) -> Grapheme {
        let mut bytes = [0; Grapheme::MAX_LEN];
        self.encode_utf8(&mut bytes);
        Grapheme::from_bytes_unchecked(bytes)
    }
}

const impl Source for &str {
    fn try_into(self) -> Result<Grapheme, GraphemesError> {
        match self.len() {
            0 => Ok(Grapheme::EMPTY),
            1..=Grapheme::MAX_LEN => {
                let mut bytes = [0; Grapheme::MAX_LEN];
                bytes[..self.len()].copy_from_slice(self.as_bytes());
                Ok(Grapheme::from_bytes_unchecked(bytes))
            },
            len => Err(GraphemesError::RequiresArena { len }),
        }
    }
}

impl<'a> Source for Extended<'a, &str> {
    fn try_into(self) -> Result<Grapheme, GraphemesError> {
        let (str, arena) = self;
        match str.len() {
            0 => Ok(Grapheme::EMPTY),
            1..=Grapheme::MAX_LEN => {
                let mut bytes = [0; Grapheme::MAX_LEN];
                bytes[..str.len()].copy_from_slice(str.as_bytes());
                Ok(Grapheme::from_bytes_unchecked(bytes))
            },
            _ => arena.try_insert(str),
        }
    }
}


/// A source value that can be converted into an inline [`Grapheme`].
///
/// # Examples
///
/// ```
/// # use crate::{Encode, Grapheme, Graphemes};
/// let mut arena = Graphemes::new();
/// let grapheme = Grapheme::new('a');
/// assert_eq!(grapheme.as_char(), 'a');
/// ```
pub const trait InlineSource: [const] Source {
}
const impl InlineSource for Grapheme {}
const impl InlineSource for char {}
const impl InlineSource for &str {}

/// A source value that can be converted into an extended [`Grapheme`].
///
/// Extended graphemes are stored in an [`Arena`](Graphemes). Use
/// [`.extended(&mut arena)`][`ExtendedBy::extended`] to create an extended grapheme from a string.
///
/// # Examples
///
/// ```
/// # use crate::{Encode, Grapheme, Graphemes};
/// let mut arena = Graphemes::new();
/// let grapheme = Grapheme::new("👩‍👩‍👧‍👦".extended(&mut arena));
/// assert_eq!(grapheme.as_str(&arena), "👩‍👩‍👧‍👦");
/// ```
pub const trait ExtendedSource: Source {}
const impl<'a> ExtendedSource for Extended<'a, &str> {}

const impl ExtendedSource for Grapheme {}

/// A pair of a value and a mutable reference to an arena.
pub type Extended<'a, T> = (T, &'a mut Graphemes);

pub const trait ExtendedBy<'a>: Sized {
    fn extended(&'a self, graphemes: &'a mut Graphemes) -> Extended<'a, Self>;
}

const impl<'a> ExtendedBy<'a> for &'a str {
    fn extended(&'a self, graphemes: &'a mut Graphemes) -> Extended<'a, Self> {
        (self, graphemes)
    }
}


