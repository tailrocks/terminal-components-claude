//! The derived per-component cache (`COMPONENT_ARCHITECTURE.md` §5 R8).
//!
//! Keyed by `(Id, TypeId)`, cleared on resize, theme change and generation
//! gap. Nothing semantic lives here: a dropped entry must only cost work.
//!
//! This file is the **one** place in `crates/tui/src` allowed to name
//! `expect` (`xtask` forbidden-pattern rule 19's path exception). `dyn
//! Any::downcast_mut` returns `Option` and safe Rust cannot express "this
//! slot holds a `Box<T>` because it was keyed by `TypeId::of::<T>()`", so the
//! invariant is stated once, here, next to the code that establishes it —
//! rather than being laundered into a livelock (BL-2) or spread across the
//! crate.

use core::any::{Any, TypeId};

use crate::id::Id;

/// The derived cache store.
#[derive(Default)]
pub(crate) struct DerivedCache {
    entries: Vec<(Id, TypeId, Box<dyn Any>)>,
}

impl core::fmt::Debug for DerivedCache {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DerivedCache")
            .field("entries", &self.entries.len())
            .finish()
    }
}

impl DerivedCache {
    /// The `T` derived for `id`, created with `T::default()` if absent.
    #[expect(
        clippy::expect_used,
        reason = "the slot is keyed by TypeId::of::<T>() and is only ever written as \
                  Box::new(T::default()), so downcast_mut::<T>() is infallible by \
                  construction; the alternative is a livelock (BL-2)"
    )]
    pub(crate) fn get_mut<T: Default + 'static>(&mut self, id: Id) -> &mut T {
        let tid = TypeId::of::<T>();
        let found = self
            .entries
            .iter()
            .position(|(i, t, _)| *i == id && *t == tid);
        let pos = if let Some(p) = found {
            p
        } else {
            self.entries.push((id, tid, Box::new(T::default())));
            self.entries.len().saturating_sub(1)
        };
        self.entries
            .get_mut(pos)
            .and_then(|(_, _, b)| b.downcast_mut::<T>())
            .expect("slot keyed by TypeId::of::<T>() holds a Box<T>")
    }

    /// Drop every entry.
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }

    /// The number of live entries.
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_cache_is_keyed_by_id_and_type() {
        let mut c = DerivedCache::default();
        let a = Id::root("a");
        let b = Id::root("b");
        *c.get_mut::<u32>(a) = 7;
        *c.get_mut::<u64>(a) = 9;
        *c.get_mut::<u32>(b) = 11;
        assert_eq!(*c.get_mut::<u32>(a), 7);
        assert_eq!(*c.get_mut::<u64>(a), 9);
        assert_eq!(*c.get_mut::<u32>(b), 11);
        assert_eq!(c.len(), 3);
        c.clear();
        assert_eq!(*c.get_mut::<u32>(a), 0);
    }
}
