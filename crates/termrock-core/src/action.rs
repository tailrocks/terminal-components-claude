use crate::event::Chord;
use crate::id::fnv1a;
use crate::variant::Variant;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct ActionKey(u16);

macro_rules! action_keys {
    ($( $(#[$m:meta])* $name:ident = $v:expr ),* $(,)?) => {
        impl ActionKey {
            $( $(#[$m])* pub const $name: ActionKey = ActionKey($v); )*

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
    CONFIRM = 0,
    CANCEL = 1,
    CLOSE = 2,
    SAVE = 3,
    DISCARD = 4,
    RETRY = 5,
}

impl ActionKey {
    pub const fn application(name: &'static str) -> ActionKey {
        let h = fnv1a(0xcbf2_9ce4_8422_2325, name.as_bytes());
        ActionKey(0x4000 | ((h as u16) & 0x3FFF))
    }

    pub const fn custom(name: &'static str) -> ActionKey {
        let h = fnv1a(0xcbf2_9ce4_8422_2325, name.as_bytes());
        ActionKey(0x8000 | ((h as u16) & 0x7FFF))
    }

    pub const fn raw(self) -> u16 {
        self.0
    }
}

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

    #[must_use]
    pub const fn chord(mut self, c: Chord) -> Self {
        self.chord = Some(c);
        self
    }

    #[must_use]
    pub const fn enabled(mut self, yes: bool) -> Self {
        self.enabled = yes;
        self
    }

    pub const fn key(&self) -> ActionKey {
        self.key
    }

    pub const fn label(&self) -> &'a str {
        self.label
    }

    pub const fn variant(&self) -> Variant {
        self.variant
    }

    pub const fn chord_ref(&self) -> Option<Chord> {
        self.chord
    }

    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

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
