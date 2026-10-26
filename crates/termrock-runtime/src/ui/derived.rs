use core::any::{Any, TypeId};

use crate::id::Id;

use super::ReferenceTarget;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ReferenceCacheKey {
    Target(ReferenceTarget),
    Targetless(u64),
}

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
    pub(crate) fn begin_frame(&mut self, generation: u32) {
        if self.generation == Some(generation) {
            return;
        }
        if let Some(previous) = self.generation {
            self.entries.retain(|entry| entry.last_seen == previous);
        }
        self.generation = Some(generation);
    }

    pub(crate) fn get_mut<T: Default + 'static>(&mut self, id: Id) -> &mut T {
        self.get_mut_in(id, None)
    }

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

    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }
}
