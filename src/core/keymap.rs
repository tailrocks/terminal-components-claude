use crate::core::event::{ClientPlatform, Key};
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

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
        let nav = match key.code {
            KeyCode::Up if self.arrows => Nav::Up,
            KeyCode::Down if self.arrows => Nav::Down,
            KeyCode::Left if self.arrows => Nav::Left,
            KeyCode::Right if self.arrows => Nav::Right,
            KeyCode::PageUp if self.page => Nav::PageUp,
            KeyCode::PageDown if self.page => Nav::PageDown,
            KeyCode::Home if self.home_end => Nav::Home,
            KeyCode::End if self.home_end => Nav::End,
            KeyCode::Char(c) if self.vim && !key.chord() => match c {
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
        crate::core::list::NavProfile::current().nav()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub code: KeyCode,
    pub mods: KeyModifiers,

    pub alt_stand_in: bool,
}

impl KeyBinding {
    pub const fn new(code: KeyCode, mods: KeyModifiers) -> Self {
        Self {
            code,
            mods,
            alt_stand_in: false,
        }
    }

    pub const fn alt_stand_in(mut self) -> Self {
        self.alt_stand_in = true;
        self
    }

    pub const fn key(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::NONE)
    }

    pub const fn char(c: char) -> Self {
        Self::key(KeyCode::Char(c))
    }

    pub const fn ctrl(c: char) -> Self {
        Self::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    pub const fn alt(c: char) -> Self {
        Self::new(KeyCode::Char(c), KeyModifiers::ALT)
    }

    pub const fn cmd(c: char) -> Self {
        Self::new(KeyCode::Char(c), KeyModifiers::SUPER)
    }

    pub const fn shift(self) -> Self {
        Self::new(self.code, self.mods.union(KeyModifiers::SHIFT))
    }

    pub fn deliverable(&self) -> bool {
        use crate::core::keyboard::{alt_delivered, super_delivered};
        (!self.mods.contains(KeyModifiers::SUPER) || super_delivered())
            && (!self.mods.contains(KeyModifiers::ALT) || alt_delivered())
            && (!self.alt_stand_in || !alt_delivered())
    }

    pub fn matches(&self, key: &Key) -> bool {
        if self.code != key.code {
            return false;
        }
        if self.alt_stand_in && crate::core::keyboard::alt_delivered() {
            return false;
        }
        if matches!(self.code, KeyCode::Char(_)) {
            (!self.mods.contains(KeyModifiers::SHIFT) || key.shift())
                && self.mods.difference(KeyModifiers::SHIFT)
                    == key.mods.difference(KeyModifiers::SHIFT)
        } else {
            self.mods == key.mods
        }
    }

    pub fn label(&self) -> String {
        self.label_on(ClientPlatform::current())
    }

    pub fn label_on(&self, platform: ClientPlatform) -> String {
        let mut out = String::new();
        let char_shift =
            self.mods.contains(KeyModifiers::SHIFT) && matches!(self.code, KeyCode::Char(_));
        if self.mods.contains(KeyModifiers::SUPER) {
            if platform.is_macos() {
                out.push_str(if self.mods.contains(KeyModifiers::SHIFT) {
                    "⇧⌘"
                } else {
                    "⌘"
                });
            } else {
                out.push_str("Super+");
            }
        }
        if self.mods.contains(KeyModifiers::CONTROL) {
            out.push_str("Ctrl+");
        }
        if self.mods.contains(KeyModifiers::ALT) {
            out.push_str("Alt+");
        }
        let ctrl_or_alt = self
            .mods
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER);
        let mac_cmd = self.mods.contains(KeyModifiers::SUPER) && platform.is_macos();

        let back_tab = self.code == KeyCode::BackTab;
        if self.mods.contains(KeyModifiers::SHIFT)
            && !mac_cmd
            && !back_tab
            && (!char_shift || ctrl_or_alt)
        {
            out.push_str("Shift+");
        }
        match self.code {
            KeyCode::Char(' ') => out.push_str("Space"),
            KeyCode::Char(c) if ctrl_or_alt => out.extend(c.to_uppercase()),
            KeyCode::Char(c) => out.push(c),
            KeyCode::Enter => out.push_str("Enter"),
            KeyCode::Esc => out.push_str("Esc"),
            KeyCode::Tab => out.push_str("Tab"),
            KeyCode::BackTab => out.push_str("Shift+Tab"),
            KeyCode::Backspace => out.push_str("Backspace"),
            KeyCode::Delete => out.push_str("Delete"),
            KeyCode::Insert => out.push_str("Ins"),
            KeyCode::Up => out.push('↑'),
            KeyCode::Down => out.push('↓'),
            KeyCode::Left => out.push('←'),
            KeyCode::Right => out.push('→'),
            KeyCode::Home => out.push_str("Home"),
            KeyCode::End => out.push_str("End"),
            KeyCode::PageUp => out.push_str("PgUp"),
            KeyCode::PageDown => out.push_str("PgDn"),
            KeyCode::F(n) => out.push_str(&format!("F{n}")),
            other => out.push_str(&format!("{other:?}")),
        }
        out
    }
}

impl From<KeyCode> for KeyBinding {
    fn from(code: KeyCode) -> Self {
        Self::key(code)
    }
}

impl From<char> for KeyBinding {
    fn from(c: char) -> Self {
        Self::char(c)
    }
}

pub type KeyMap<A> = Vec<(KeyBinding, A)>;

pub trait Actions: Copy + PartialEq + Sized + 'static {
    fn default_keymap() -> KeyMap<Self>;

    fn label(self) -> &'static str;
}

pub fn lookup<A: Copy>(map: &[(KeyBinding, A)], key: &Key) -> Option<A> {
    map.iter().find(|(b, _)| b.matches(key)).map(|&(_, a)| a)
}

pub fn unassigned_chord<A: Copy>(key: &Key, map: &[(KeyBinding, A)]) -> bool {
    matches!(key.code, KeyCode::Char(_)) && key.chord() && lookup(map, key).is_none()
}

pub fn binding_for<A: PartialEq>(map: &[(KeyBinding, A)], action: A) -> Option<KeyBinding> {
    map.iter().find(|(_, a)| *a == action).map(|&(b, _)| b)
}

pub fn deliverable_binding_for<A: PartialEq>(
    map: &[(KeyBinding, A)],
    action: A,
) -> Option<KeyBinding> {
    map.iter()
        .find(|(b, a)| *a == action && b.deliverable())
        .map(|&(b, _)| b)
}

pub fn label_for<A: PartialEq>(map: &[(KeyBinding, A)], action: A) -> Option<String> {
    deliverable_binding_for(map, action).map(|b| b.label())
}

pub const SELECT_ALL: [KeyBinding; 2] = [KeyBinding::cmd('a'), KeyBinding::ctrl('a')];

pub fn select_all_bindings<A: Copy>(action: A) -> impl Iterator<Item = (KeyBinding, A)> {
    SELECT_ALL.into_iter().map(move |b| (b, action))
}

pub fn hint_key<A: PartialEq>(map: &[(KeyBinding, A)], action: A) -> Option<&'static str> {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static LABELS: OnceLock<Mutex<HashMap<KeyBinding, &'static str>>> = OnceLock::new();
    let b = deliverable_binding_for(map, action)?;
    let mut labels = LABELS
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    Some(labels.entry(b).or_insert_with(|| intern(b.label())))
}

pub fn intern(label: String) -> &'static str {
    use std::collections::HashSet;
    use std::sync::{Mutex, OnceLock};
    static LABELS: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
    let mut labels = LABELS
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if let Some(&l) = labels.get(label.as_str()) {
        return l;
    }
    let l: &'static str = Box::leak(label.into_boxed_str());
    labels.insert(l);
    l
}
