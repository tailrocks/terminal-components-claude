//! Navigation keys (ported from PR #26 `src/core/keymap.rs`, the `Nav` /
//! `NavKeys` vocabulary, commit f4867e9b5).

use crate::event::Key;
use crate::keys::{KeyCode, KeyModifiers};

/// A navigation action every list-like widget understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nav {
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Home,
    End,
}

/// Which key families map to navigation actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NavKeys {
    /// Arrow keys move.
    pub arrows: bool,
    /// Vim letters (j/k/h/l/g/G) move.
    pub vim: bool,
    /// PageUp / PageDown page.
    pub page: bool,
    /// Home / End jump to the ends.
    pub home_end: bool,
}

impl NavKeys {
    /// The desktop profile: arrows, paging, Home / End, no vim letters.
    pub const DESKTOP: Self = Self {
        arrows: true,
        vim: false,
        page: true,
        home_end: true,
    };

    /// The desktop profile plus vim letters.
    pub const VIM: Self = Self {
        vim: true,
        ..Self::DESKTOP
    };

    /// The default profile.
    pub const DEFAULT: Self = Self::DESKTOP;

    /// The desktop profile (no vim).
    pub const NO_VIM: Self = Self::DESKTOP;

    /// Turn the vim letters on or off.
    pub const fn vim(mut self, on: bool) -> Self {
        self.vim = on;
        self
    }

    /// Turn paging on or off.
    pub const fn page(mut self, on: bool) -> Self {
        self.page = on;
        self
    }

    /// Turn Home / End on or off.
    pub const fn home_end(mut self, on: bool) -> Self {
        self.home_end = on;
        self
    }

    /// Turn the arrows on or off.
    pub const fn arrows(mut self, on: bool) -> Self {
        self.arrows = on;
        self
    }

    /// The navigation action for a key, if this profile maps it.
    pub fn action(&self, key: &Key) -> Option<Nav> {
        let chord = key.ctrl() || key.alt() || key.mods.contains(KeyModifiers::SUPER);
        let nav = match key.code {
            KeyCode::Up if self.arrows => Nav::Up,
            KeyCode::Down if self.arrows => Nav::Down,
            KeyCode::Left if self.arrows => Nav::Left,
            KeyCode::Right if self.arrows => Nav::Right,
            KeyCode::PageUp if self.page => Nav::PageUp,
            KeyCode::PageDown if self.page => Nav::PageDown,
            KeyCode::Home if self.home_end => Nav::Home,
            KeyCode::End if self.home_end => Nav::End,
            KeyCode::Char(c) if self.vim && !chord => match c {
                'k' | 'K' => Nav::Up,
                'j' | 'J' => Nav::Down,
                'h' | 'H' => Nav::Left,
                'l' | 'L' => Nav::Right,
                'g' => Nav::Home,
                'G' => Nav::End,
                _ => return None,
            },
            _ => return None,
        };
        Some(nav)
    }

    /// The hint for the move keys under this profile.
    pub const fn move_hint(&self, type_ahead: bool) -> Option<&'static str> {
        if self.arrows {
            Some("↑↓")
        } else if self.vim && !type_ahead {
            Some("j k")
        } else {
            None
        }
    }

    /// The hint for the paging keys.
    pub const fn page_hint(&self) -> Option<&'static str> {
        if self.page { Some("PgUp PgDn") } else { None }
    }

    /// The hint for the ends keys.
    pub const fn ends_hint(&self, type_ahead: bool) -> Option<&'static str> {
        if self.vim && !type_ahead {
            Some("g G")
        } else if self.home_end {
            Some("Home End")
        } else {
            None
        }
    }

    /// Whether a key is one of the vim movement letters.
    pub fn is_vim_letter(key: &Key) -> bool {
        matches!(
            key.code,
            KeyCode::Char('k' | 'K' | 'j' | 'J' | 'h' | 'H' | 'l' | 'L' | 'g' | 'G')
        )
    }
}

impl Default for NavKeys {
    fn default() -> Self {
        crate::list::NavProfile::current().nav()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::KeyModifiers;

    fn k(code: KeyCode) -> Key {
        Key {
            code,
            mods: KeyModifiers::NONE,
        }
    }

    #[test]
    fn desktop_maps_arrows_paging_and_ends_only() {
        assert_eq!(NavKeys::DESKTOP.action(&k(KeyCode::Up)), Some(Nav::Up));
        assert_eq!(
            NavKeys::DESKTOP.action(&k(KeyCode::PageDown)),
            Some(Nav::PageDown)
        );
        assert_eq!(NavKeys::DESKTOP.action(&k(KeyCode::End)), Some(Nav::End));
        assert_eq!(NavKeys::DESKTOP.action(&k(KeyCode::Char('j'))), None);
        assert_eq!(NavKeys::VIM.action(&k(KeyCode::Char('j'))), Some(Nav::Down));
        assert_eq!(NavKeys::VIM.action(&k(KeyCode::Char('G'))), Some(Nav::End));
        assert_eq!(
            NavKeys::VIM.action(&Key {
                code: KeyCode::Char('j'),
                mods: KeyModifiers::CONTROL,
            }),
            None,
            "a chorded letter is never vim"
        );
    }

    #[test]
    fn hints_follow_the_profile() {
        assert_eq!(NavKeys::DESKTOP.move_hint(true), Some("↑↓"));
        assert_eq!(NavKeys::DESKTOP.ends_hint(true), Some("Home End"));
        assert_eq!(NavKeys::VIM.move_hint(false), Some("↑↓"));
        assert_eq!(NavKeys::VIM.move_hint(true), Some("↑↓"));
        assert_eq!(
            NavKeys::DESKTOP.vim(false).arrows(false).move_hint(true),
            None
        );
        assert_eq!(NavKeys::DESKTOP.page_hint(), Some("PgUp PgDn"));
    }
}
