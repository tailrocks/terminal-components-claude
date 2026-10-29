use crate::id::fnv1a;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Variant(u16);

impl Variant {
    pub const DEFAULT: Variant = Variant(0);
    pub const PRIMARY: Variant = Variant(1);
    pub const SECONDARY: Variant = Variant(2);
    pub const SUBTLE: Variant = Variant(3);
    pub const DANGER: Variant = Variant(4);
    pub const TOGGLE: Variant = Variant(5);
    pub const QUIET: Variant = Variant(6);
    pub const GHOST: Variant = Variant(7);

    pub const ALL: &'static [Variant] = &[
        Variant::DEFAULT,
        Variant::PRIMARY,
        Variant::SECONDARY,
        Variant::SUBTLE,
        Variant::DANGER,
        Variant::TOGGLE,
        Variant::QUIET,
        Variant::GHOST,
    ];

    pub const fn custom(name: &'static str) -> Variant {
        let h = fnv1a(FNV_OFFSET, name.as_bytes());
        Variant(0x8000 | ((h as u16) & 0x7FFF))
    }

    pub const fn name(self) -> Option<&'static str> {
        match self {
            Variant::DEFAULT => Some("DEFAULT"),
            Variant::PRIMARY => Some("PRIMARY"),
            Variant::SECONDARY => Some("SECONDARY"),
            Variant::SUBTLE => Some("SUBTLE"),
            Variant::DANGER => Some("DANGER"),
            Variant::TOGGLE => Some("TOGGLE"),
            Variant::QUIET => Some("QUIET"),
            Variant::GHOST => Some("GHOST"),
            _ => None,
        }
    }

    pub const fn raw(self) -> u16 {
        self.0
    }
}

impl core::fmt::Debug for Variant {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(n) = self.name() {
            write!(f, "Variant::{n}")
        } else {
            write!(f, "Variant::custom(#{:04x})", self.0)
        }
    }
}

impl Default for Variant {
    fn default() -> Self {
        Variant::DEFAULT
    }
}
