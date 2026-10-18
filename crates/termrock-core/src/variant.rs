#![doc = r" Visual variant newtype."]

use crate::id::fnv1a;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

#[doc = r" A variant within a family, the second key of a recipe."]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Variant(u16);

impl Variant {
    #[doc = r" The family's default look."]
    pub const DEFAULT: Variant = Variant(0);
    #[doc = r" Primary emphasis."]
    pub const PRIMARY: Variant = Variant(1);
    #[doc = r" Secondary emphasis."]
    pub const SECONDARY: Variant = Variant(2);
    #[doc = r" Subtle."]
    pub const SUBTLE: Variant = Variant(3);
    #[doc = r" Destructive."]
    pub const DANGER: Variant = Variant(4);
    #[doc = r" Toggle."]
    pub const TOGGLE: Variant = Variant(5);
    #[doc = r" Quiet."]
    pub const QUIET: Variant = Variant(6);
    #[doc = r" Ghost."]
    pub const GHOST: Variant = Variant(7);

    #[doc = r" Every library constant, in declaration order."]
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

    #[doc = r" A custom value named by a downstream author; lands in the high range."]
    pub const fn custom(name: &'static str) -> Variant {
        let h = fnv1a(FNV_OFFSET, name.as_bytes());
        Variant(0x8000 | ((h as u16) & 0x7FFF))
    }

    #[doc = r" The library name, or `None` for a custom value."]
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

    #[doc = r" The raw index / hash value."]
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
