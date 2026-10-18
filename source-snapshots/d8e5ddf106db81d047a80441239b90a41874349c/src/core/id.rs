//! Stable widget identifiers.
//!
//! Every interactive element owns a [`WidgetId`]. Ids are derived from a
//! human readable path (`"forms.name"`) so they are stable across frames and
//! easy to debug, and can be extended with numeric children for repeated rows.

use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WidgetId(u64);

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

const fn fnv1a(mut hash: u64, bytes: &[u8]) -> u64 {
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
        i += 1;
    }
    hash
}

impl WidgetId {
    /// Build an id from a static path such as `"buttons.primary"`.
    pub const fn of(path: &str) -> Self {
        Self(fnv1a(FNV_OFFSET, path.as_bytes()))
    }

    /// Derive a child id, e.g. one per table row or list item.
    pub const fn child(self, index: usize) -> Self {
        Self(fnv1a(self.0, &index.to_le_bytes()))
    }

    /// Derive a named child id.
    pub const fn sub(self, name: &str) -> Self {
        Self(fnv1a(self.0, name.as_bytes()))
    }
}

impl fmt::Debug for WidgetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WidgetId({:016x})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_stable_and_distinct() {
        assert_eq!(WidgetId::of("a.b"), WidgetId::of("a.b"));
        assert_ne!(WidgetId::of("a.b"), WidgetId::of("a.c"));
        assert_ne!(WidgetId::of("a").child(0), WidgetId::of("a").child(1));
        assert_ne!(WidgetId::of("a").child(0), WidgetId::of("a"));
        assert_ne!(WidgetId::of("a").sub("x"), WidgetId::of("a").sub("y"));
    }
}
