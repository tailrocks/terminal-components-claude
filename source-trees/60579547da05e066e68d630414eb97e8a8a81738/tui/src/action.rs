//! Action identity for dialogs, menus, forms and cell actions
//! (`COMPONENT_ARCHITECTURE.md` §17.0 A4).

use crate::event::Chord;
use crate::id::fnv1a;
use crate::theme::Variant;

/// A typed action identity, `const`-constructible so binding tables, key
/// maps and dialog action rows can name it without allocation.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct ActionKey(u16);

macro_rules! action_keys {
    ($( $(#[$m:meta])* $name:ident = $v:expr ),* $(,)?) => {
        impl ActionKey {
            $( $(#[$m])* pub const $name: ActionKey = ActionKey($v); )*

            /// The library name of an action key, or `None` for a custom key.
            pub const fn name(self) -> Option<&'static str> {
                match self {
                    $( ActionKey::$name => Some(stringify!($name)), )*
                    _ => None,
                }
            }
        }
    };
}

action_keys! {
    /// Confirm / OK.
    CONFIRM = 0,
    /// Cancel.
    CANCEL = 1,
    /// Close.
    CLOSE = 2,
    /// Save.
    SAVE = 3,
    /// Discard.
    DISCARD = 4,
    /// Retry.
    RETRY = 5,
}

impl ActionKey {
    /// An application-owned key in the middle range, isolated from component keys.
    pub const fn application(name: &'static str) -> ActionKey {
        let h = fnv1a(0xcbf2_9ce4_8422_2325, name.as_bytes());
        ActionKey(0x4000 | ((h as u16) & 0x3FFF))
    }

    /// A component-owned custom key; lands in the high range.
    pub const fn custom(name: &'static str) -> ActionKey {
        let h = fnv1a(0xcbf2_9ce4_8422_2325, name.as_bytes());
        ActionKey(0x8000 | ((h as u16) & 0x7FFF))
    }

    /// The raw key number.
    pub const fn raw(self) -> u16 {
        self.0
    }
}

/// One action in a dialog or form action row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Action<'a> {
    key: ActionKey,
    label: &'a str,
    variant: Variant,
    chord: Option<Chord>,
    enabled: bool,
    danger: bool,
}

impl<'a> Action<'a> {
    /// A default-variant action.
    pub const fn new(key: ActionKey, label: &'a str) -> Self {
        Action {
            key,
            label,
            variant: Variant::DEFAULT,
            chord: None,
            enabled: true,
            danger: false,
        }
    }

    /// A destructive action.
    pub const fn danger(key: ActionKey, label: &'a str) -> Self {
        Action {
            key,
            label,
            variant: Variant::DANGER,
            chord: None,
            enabled: true,
            danger: true,
        }
    }

    /// A quiet (low-emphasis) action.
    pub const fn quiet(key: ActionKey, label: &'a str) -> Self {
        Action {
            key,
            label,
            variant: Variant::QUIET,
            chord: None,
            enabled: true,
            danger: false,
        }
    }

    /// Attach a chord that both renders as the hint and registers the binding.
    #[must_use]
    pub const fn chord(mut self, c: Chord) -> Self {
        self.chord = Some(c);
        self
    }

    /// The §9.2 arming predicate, evaluated in `update`.
    #[must_use]
    pub const fn enabled(mut self, yes: bool) -> Self {
        self.enabled = yes;
        self
    }

    /// The key.
    pub const fn key(&self) -> ActionKey {
        self.key
    }

    /// The label.
    pub const fn label(&self) -> &'a str {
        self.label
    }

    /// The variant.
    pub const fn variant(&self) -> Variant {
        self.variant
    }

    /// The chord, if any.
    pub const fn chord_ref(&self) -> Option<Chord> {
        self.chord
    }

    /// Whether the action is armed.
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Whether the action is destructive.
    pub const fn is_danger(&self) -> bool {
        self.danger
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_keys_land_in_the_high_range_and_names_resolve() {
        assert!(ActionKey::application("jackin.quit").raw() >= 0x4000);
        assert!(ActionKey::application("jackin.quit").raw() < 0x8000);
        assert_ne!(
            ActionKey::application("jackin.quit"),
            ActionKey::custom("jackin.quit")
        );
        assert!(ActionKey::custom("delete").raw() >= 0x8000);
        assert_eq!(ActionKey::SAVE.name(), Some("SAVE"));
        assert_eq!(ActionKey::custom("x").name(), None);
        assert_ne!(ActionKey::custom("a"), ActionKey::custom("b"));
        let a = Action::danger(ActionKey::custom("delete"), "Delete").enabled(false);
        assert!(a.is_danger() && !a.is_enabled());
        assert_eq!(a.variant(), Variant::DANGER);
    }
}
