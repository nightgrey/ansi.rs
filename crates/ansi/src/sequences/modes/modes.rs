use crate::{Mode, ModeSetting};
use derive_more::{Deref, DerefMut, From};
use std::collections::HashMap;

/// Modes
///
/// Manages multiple terminal modes and their settings, providing a convenient
/// interface to query, modify, and generate sequences for multiple modes at once.
#[derive(Debug,Clone, Eq, PartialEq, Default, From, )]
#[repr(transparent)]
pub struct Modes(HashMap<Mode, ModeSetting>);

impl Modes {
    #[inline]
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity(capacity))
    }

    #[inline]
    pub fn set(&mut self, mode: Mode) -> &'_ mut ModeSetting {
        self.0.entry(mode)
            .and_modify(|setting| *setting = ModeSetting::Set)
            .or_insert(ModeSetting::Set)
    }

    #[inline]
    pub fn set_permanently(&mut self, mode: Mode) -> &'_ mut ModeSetting {
        self.0.entry(mode)
            .and_modify(|setting| *setting = ModeSetting::PermanentlySet)
            .or_insert(ModeSetting::PermanentlySet)
    }

    #[inline]
    pub fn reset(&mut self, mode: Mode) -> &'_ mut ModeSetting {
        self.0.entry(mode)
            .and_modify(|setting| *setting = ModeSetting::Reset)
            .or_insert(ModeSetting::Reset)
    }

    #[inline]
    pub fn reset_permanently(&mut self, mode: Mode) -> &'_ mut ModeSetting {
        self.0.entry(mode)
            .and_modify(|setting| *setting = ModeSetting::PermanentlyReset)
            .or_insert(ModeSetting::PermanentlyReset)
    }

    #[inline]
    pub fn is_not_recognized(&self, mode: &Mode) -> bool {
        self.0.get(mode).is_none_or(ModeSetting::is_not_recognized)
    }

    #[inline]
    pub fn is_set(&self, mode: &Mode) -> bool {
        self.0.get(mode).is_some_and(ModeSetting::is_set)
    }

    #[inline]
    pub fn is_permanently_set(&self, mode: &Mode) -> bool {
        self.0.get(mode).is_some_and(ModeSetting::is_permanently_set)
    }

    #[inline]
    pub fn is_reset(&self, mode: &Mode) -> bool {
        self.0.get(mode).is_some_and(ModeSetting::is_reset)
    }

    #[inline]
    pub fn is_permanently_reset(&self, mode: &Mode) -> bool {
        self.0.get(mode).is_some_and(ModeSetting::is_permanently_reset)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&Mode, &ModeSetting)> {
        self.0.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Mode, &mut ModeSetting)> {
        self.0.iter_mut()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.clear()
    }
}

impl FromIterator<(Mode, ModeSetting)> for Modes {
    /// Constructs a `Modes` from an iterator of key-value pairs.
    ///
    /// If the iterator produces any pairs with equal keys,
    /// all but one of the corresponding values will be dropped.
    fn from_iter<T: IntoIterator<Item = (Mode, ModeSetting)>>(iter: T) -> Self {
        Self(HashMap::from_iter(iter))
    }
}
/// Inserts all new key-values from the iterator and replaces values with existing
/// keys with new values returned from the iterator.
impl Extend<(Mode, ModeSetting)> for Modes {
    #[inline]
    fn extend<T: IntoIterator<Item = (Mode, ModeSetting)>>(&mut self, iter: T) {
        self.0.extend(iter)
    }

    #[inline]
    fn extend_one(&mut self, (k, v): (Mode, ModeSetting)) {
        self.0.insert(k, v);
    }

    #[inline]
    fn extend_reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }
}
