use crate::{Mode, ModeSetting, ReportMode};
use derive_more::{Deref, DerefMut,  From};
use std::collections::HashMap;

/// Modes
///
/// Manages multiple terminal modes and their settings, providing a convenient
/// interface to query, modify, and generate sequences for multiple modes at once.
#[derive(Debug, Clone, Default, Deref, DerefMut, From)]
#[repr(transparent)]
pub struct Modes(pub HashMap<Mode, ModeSetting>);

impl Modes {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity(capacity))
    }

    pub fn set(&mut self, mode: Mode) -> &'_ mut ModeSetting {
        self.entry(mode)
            .and_modify(|setting| setting.set())
            .or_insert(ModeSetting::Set)
    }

    pub fn set_permanently(&mut self, mode: Mode) -> &'_ mut ModeSetting {
        self.entry(mode)
            .and_modify(|setting| setting.set_permanently())
            .or_insert(ModeSetting::PermanentlySet)
    }

    pub fn reset(&mut self, mode: Mode) -> &'_ mut ModeSetting {
        self.entry(mode)
            .and_modify(|setting| setting.reset())
            .or_insert(ModeSetting::Reset)
    }

    pub fn reset_permanently(&mut self, mode: Mode) -> &'_ mut ModeSetting {
        self.entry(mode)
            .and_modify(|setting| setting.reset_permanently())
            .or_insert(ModeSetting::PermanentlyReset)
    }

    pub fn is_not_recognized(&self, mode: &Mode) -> bool {
        self.get(mode).map_or(true, ModeSetting::is_not_recognized)
    }

    pub fn is_set(&self, mode: &Mode) -> bool {
        self.get(mode).map_or(false, ModeSetting::is_set)
    }

    pub fn is_permanently_set(&self, mode: &Mode) -> bool {
        self.get(mode)
            .map_or(false, ModeSetting::is_permanently_set)
    }

    pub fn is_reset(&self, mode: &Mode) -> bool {
        self.get(mode).map_or(false, ModeSetting::is_reset)
    }

    pub fn is_permanently_reset(&self, mode: &Mode) -> bool {
        self.get(mode)
            .map_or(false, ModeSetting::is_permanently_reset)
    }
}
