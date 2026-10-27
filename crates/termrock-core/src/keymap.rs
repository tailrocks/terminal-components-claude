//! Navigation keys (ported from PR #26 `src/core/keymap.rs`, the `Nav` /
//! `NavKeys` vocabulary, commit f4867e9b5).

use crate::event::Key;
use crate::keys::{KeyCode, KeyModifiers};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NavKeys {
    pub arrows: bool,

    pub vim: bool,

    pub page: bool,

    pub home_end: bool,
}

impl NavKeys {
    pub const DESKTOP: Self = Self {
        arrows: true,
        vim: false,
        page: true,
        home_end: true,
    };

    pub const VIM: Self = Self {
        vim: true,
        ..Self::DESKTOP
    };

    pub const DEFAULT: Self = Self::DESKTOP;

    pub const NO_VIM: Self = Self::DESKTOP;

    pub const fn vim(mut self, on: bool) -> Self {
        self.vim = on;
        self
    }

    pub const fn page(mut self, on: bool) -> Self {
        self.page = on;
        self
    }

    pub const fn home_end(mut self, on: bool) -> Self {
        self.home_end = on;
        self
    }

    pub const fn arrows(mut self, on: bool) -> Self {
        self.arrows = on;
        self
    }

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

    pub const fn move_hint(&self, type_ahead: bool) -> Option<&'static str> {
        if self.arrows {
            Some("↑↓")
        } else if self.vim && !type_ahead {
            Some("j k")
        } else {
            None
        }
    }

    pub const fn page_hint(&self) -> Option<&'static str> {
        if self.page { Some("PgUp PgDn") } else { None }
    }

    pub const fn ends_hint(&self, type_ahead: bool) -> Option<&'static str> {
        if self.vim && !type_ahead {
            Some("g G")
        } else if self.home_end {
            Some("Home End")
        } else {
            None
        }
    }

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
