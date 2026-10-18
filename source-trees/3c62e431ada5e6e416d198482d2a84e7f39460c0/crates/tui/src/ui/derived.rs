//! The derived per-component cache (`COMPONENT_ARCHITECTURE.md` §5 R8).
//!
//! Live entries are keyed by `(Id, TypeId)`; the isolated reference store
//! additionally keys explicit targets by their stable `ReferenceTarget` and
//! targetless scopes by a unique one-shot identity. Both are cleared on
//! resize, theme change and generation gap. Nothing semantic lives here: a
//! dropped entry must only cost work.
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

use super::ReferenceTarget;

/// The cache namespace of one reference scope.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ReferenceCacheKey {
    /// Explicit targets retain derived work across frames and draw-order changes.
    Target(ReferenceTarget),
    /// `reference(None, ..)` has no caller-supplied stable identity, so each
    /// invocation is isolated instead of aliasing an unrelated sibling.
    Targetless(u64),
}

/// The derived cache store.
#[derive(Default)]
pub(crate) struct DerivedCache {
    entries: Vec<Entry>,
    generation: Option<u32>,
}

struct Entry {
    id: Id,
    reference: Option<ReferenceCacheKey>,
    tid: TypeId,
    value: Box<dyn Any>,
    last_seen: u32,
}

impl core::fmt::Debug for DerivedCache {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DerivedCache")
            .field("entries", &self.entries.len())
            .field("generation", &self.generation)
            .finish()
    }
}

impl DerivedCache {
    /// Start a cache generation, dropping entries unseen in the prior one.
    pub(crate) fn begin_frame(&mut self, generation: u32) {
        if self.generation == Some(generation) {
            return;
        }
        if let Some(previous) = self.generation {
            self.entries.retain(|entry| entry.last_seen == previous);
        }
        self.generation = Some(generation);
    }

    /// The `T` derived for `id`, created with `T::default()` if absent.
    pub(crate) fn get_mut<T: Default + 'static>(&mut self, id: Id) -> &mut T {
        self.get_mut_in(id, None)
    }

    /// The `T` derived for one stable reference target and component id.
    pub(crate) fn get_mut_reference<T: Default + 'static>(
        &mut self,
        id: Id,
        reference: ReferenceCacheKey,
    ) -> &mut T {
        self.get_mut_in(id, Some(reference))
    }

    #[expect(
        clippy::expect_used,
        reason = "the slot is keyed by TypeId::of::<T>() and is only ever written as \
                  Box::new(T::default()), so downcast_mut::<T>() is infallible by \
                  construction; the alternative is a livelock (BL-2)"
    )]
    fn get_mut_in<T: Default + 'static>(
        &mut self,
        id: Id,
        reference: Option<ReferenceCacheKey>,
    ) -> &mut T {
        let tid = TypeId::of::<T>();
        let found = self
            .entries
            .iter()
            .position(|entry| entry.id == id && entry.reference == reference && entry.tid == tid);
        let pos = if let Some(p) = found {
            p
        } else {
            self.entries.push(Entry {
                id,
                reference,
                tid,
                value: Box::new(T::default()),
                last_seen: self.generation.unwrap_or(0),
            });
            self.entries.len().saturating_sub(1)
        };
        self.entries
            .get_mut(pos)
            .map(|entry| {
                entry.last_seen = self.generation.unwrap_or(0);
                &mut entry.value
            })
            .and_then(|value| value.downcast_mut::<T>())
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
    use crate::ui::{ReferenceState, ReferenceTarget};

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

    #[test]
    fn derived_cache_drops_component_after_generation_gap() {
        let mut cache = DerivedCache::default();
        let tree = Id::root("tree");

        cache.begin_frame(u32::MAX);
        *cache.get_mut::<u32>(tree) = 7;
        cache.begin_frame(0);
        assert_eq!(cache.len(), 1, "one absent frame retains the prior entry");
        cache.begin_frame(1);
        assert_eq!(
            *cache.get_mut::<u32>(tree),
            0,
            "reappearing after a frame gap gets a fresh cache"
        );
    }

    #[test]
    fn reference_targets_survive_cross_frame_draw_order_changes() {
        let mut cache = DerivedCache::default();
        let component = Id::root("component");
        let first = ReferenceTarget::new(Id::root("first"), ReferenceState::FOCUSED);
        let second = ReferenceTarget::new(Id::root("second"), ReferenceState::HOVERED);

        cache.begin_frame(1);
        *cache.get_mut_reference::<u32>(component, ReferenceCacheKey::Target(first)) = 7;
        *cache.get_mut_reference::<u32>(component, ReferenceCacheKey::Target(second)) = 11;

        cache.begin_frame(2);
        assert_eq!(
            *cache.get_mut_reference::<u32>(component, ReferenceCacheKey::Target(second)),
            11
        );
        assert_eq!(
            *cache.get_mut_reference::<u32>(component, ReferenceCacheKey::Target(first)),
            7
        );
    }
}
