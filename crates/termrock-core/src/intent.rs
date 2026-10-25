use core::cell::Cell;
use core::fmt;
use core::ops::Range;
use std::sync::Arc;

use ratatui_core::layout::Position;

use crate::action::ActionKey;
use crate::event::{Axis, Key, KeyModifiers};
use crate::id::{Id, PartRef};
use crate::layer_types::LayerEvent;

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Intent<'f> {
    Binding(ActionKey),
    Key(Key),
    Paste(&'f str),
    Pointer {
        phase: Phase,
        part: PartRef,
        pos: Position,
        local: Position,
        mods: KeyModifiers,
    },
    Wheel {
        axis: Axis,
        delta: i16,
        part: PartRef,
        pos: Position,
    },
    FocusIn {
        via: FocusVia,
    },
    FocusOut {
        to: Option<Id>,
    },
    Layer(LayerEvent),
    Cancel,
}

impl fmt::Debug for Intent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Intent::Binding(action) => f.debug_tuple("Binding").field(action).finish(),
            Intent::Key(key) => f.debug_tuple("Key").field(key).finish(),
            Intent::Paste(text) => f
                .debug_struct("Paste")
                .field("len", &text.len())
                .field("text", &"[redacted]")
                .finish(),
            Intent::Pointer {
                phase,
                part,
                pos,
                local,
                mods,
            } => f
                .debug_struct("Pointer")
                .field("phase", phase)
                .field("part", part)
                .field("pos", pos)
                .field("local", local)
                .field("mods", mods)
                .finish(),
            Intent::Wheel {
                axis,
                delta,
                part,
                pos,
            } => f
                .debug_struct("Wheel")
                .field("axis", axis)
                .field("delta", delta)
                .field("part", part)
                .field("pos", pos)
                .finish(),
            Intent::FocusIn { via } => f.debug_struct("FocusIn").field("via", via).finish(),
            Intent::FocusOut { to } => f.debug_struct("FocusOut").field("to", to).finish(),
            Intent::Layer(event) => f.debug_tuple("Layer").field(event).finish(),
            Intent::Cancel => f.write_str("Cancel"),
        }
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Move,
    Middle,
    Press,
    Release,
    Click,
    DoubleClick,
    Secondary,
    DragStart,
    Drag,
    DragEnd,
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusVia {
    Keyboard,
    Pointer,
    Programmatic,
    Restore,
}

/// Opaque identity for one exact layer intent queued by `IntentQueue::layer_tracked`.
///
/// Clones share the same single claim. A token cannot be constructed outside this
/// module, and it does not keep a cleared or dropped queue entry pending.
#[derive(Clone)]
pub struct LayerIntentToken {
    owner: Id,
    marker: Arc<()>,
}

impl fmt::Debug for LayerIntentToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LayerIntentToken(..)")
    }
}

/// Result of claiming a layer event through its exact tracked token.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerIntentClaim {
    /// The exact pending intent was claimed by this call.
    Claimed,
    /// The exact intent is still in this queue but was already claimed.
    AlreadyClaimed,
    /// The token's exact intent is no longer pending in this queue.
    NotPending,
    /// The token identifies a pending intent with a different event payload.
    Mismatch,
}

#[derive(Debug)]
enum Stored {
    Binding {
        action: ActionKey,
        chord: crate::event::Chord,
        claimed: Cell<bool>,
    },
    Key(Key),
    Paste(u32, u32),
    Pointer {
        phase: Phase,
        part: PartRef,
        pos: Position,
        local: Position,
        mods: KeyModifiers,
    },
    Wheel {
        axis: Axis,
        delta: i16,
        part: PartRef,
        pos: Position,
    },
    FocusIn(FocusVia),
    FocusOut(Option<Id>),
    Layer {
        event: LayerEvent,
        claimed: Cell<bool>,
        marker: Option<Arc<()>>,
    },
    Cancel,
}

#[derive(Debug)]
struct Bucket {
    owner: Id,
    items: Vec<Stored>,
    drained: Cell<bool>,
}

const TABLE: usize = 32;
const TABLE_MAX_OWNERS: usize = 24;

pub struct IntentQueue {
    arena: String,
    buckets: Vec<Bucket>,
    used: usize,
    table: [Option<(u64, u16)>; TABLE],
    overflow: bool,
    #[cfg(feature = "testing")]
    probes: Cell<usize>,
}

impl Default for IntentQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl IntentQueue {
    pub const fn new() -> Self {
        IntentQueue {
            arena: String::new(),
            buckets: Vec::new(),
            used: 0,
            table: [None; TABLE],
            overflow: false,
            #[cfg(feature = "testing")]
            probes: Cell::new(0),
        }
    }

    #[cfg(feature = "testing")]
    pub fn probes(&self) -> usize {
        self.probes.get()
    }

    pub fn clear(&mut self) {
        zeroize_string(&mut self.arena);
        for b in &mut self.buckets {
            b.items.clear();
            b.drained.set(false);
        }
        self.used = 0;
        self.table = [None; TABLE];
        self.overflow = false;
    }

    pub fn is_empty(&self) -> bool {
        self.used == 0
    }

    fn live(&self) -> &[Bucket] {
        self.buckets.get(..self.used).unwrap_or(&[])
    }

    fn bucket_index(&self, owner: Id) -> Option<usize> {
        #[cfg(feature = "testing")]
        self.probes.set(self.probes.get().saturating_add(1));
        if self.overflow {
            return self.live().iter().position(|b| b.owner == owner);
        }
        let h = owner.hash();
        let mut slot = (h as usize) % TABLE;
        for _ in 0..TABLE {
            match self.table.get(slot).copied().flatten() {
                None => return None,
                Some((hash, idx)) if hash == h => return Some(usize::from(idx)),
                Some(_) => slot = slot.wrapping_add(1) % TABLE,
            }
        }
        None
    }

    fn bucket_slot(&mut self, owner: Id) -> usize {
        if let Some(i) = self.bucket_index(owner) {
            return i;
        }
        let idx = self.used;
        if let Some(b) = self.buckets.get_mut(idx) {
            b.owner = owner;
            b.items.clear();
            b.drained.set(false);
        } else {
            self.buckets.push(Bucket {
                owner,
                items: Vec::new(),
                drained: Cell::new(false),
            });
        }
        self.used = self.used.saturating_add(1);
        if !self.overflow {
            if idx >= TABLE_MAX_OWNERS {
                self.overflow = true;
            } else {
                let h = owner.hash();
                let mut slot = (h as usize) % TABLE;
                for _ in 0..TABLE {
                    match self.table.get_mut(slot) {
                        Some(s @ None) => {
                            *s = Some((h, idx as u16));
                            break;
                        }
                        _ => slot = slot.wrapping_add(1) % TABLE,
                    }
                }
            }
        }
        idx
    }

    fn push(&mut self, owner: Id, s: Stored) {
        let i = self.bucket_slot(owner);
        if let Some(b) = self.buckets.get_mut(i) {
            b.items.push(s);
        }
    }

    pub fn key(&mut self, owner: Id, k: Key) {
        self.push(owner, Stored::Key(k));
    }

    pub fn binding(&mut self, owner: Id, action: ActionKey, chord: crate::event::Chord) {
        self.push(
            owner,
            Stored::Binding {
                action,
                chord,
                claimed: Cell::new(false),
            },
        );
    }

    pub fn paste(&mut self, owner: Id, text: &str) {
        let start = self.arena.len() as u32;
        self.arena.push_str(text);
        let end = self.arena.len() as u32;
        self.push(owner, Stored::Paste(start, end));
    }

    pub fn pointer(
        &mut self,
        owner: Id,
        phase: Phase,
        part: PartRef,
        pos: Position,
        local: Position,
        mods: KeyModifiers,
    ) {
        self.push(
            owner,
            Stored::Pointer {
                phase,
                part,
                pos,
                local,
                mods,
            },
        );
    }

    pub fn wheel(&mut self, owner: Id, axis: Axis, delta: i16, part: PartRef, pos: Position) {
        self.push(
            owner,
            Stored::Wheel {
                axis,
                delta,
                part,
                pos,
            },
        );
    }

    pub fn focus_in(&mut self, owner: Id, via: FocusVia) {
        self.push(owner, Stored::FocusIn(via));
    }

    pub fn focus_out(&mut self, owner: Id, to: Option<Id>) {
        self.push(owner, Stored::FocusOut(to));
    }

    pub fn layer(&mut self, owner: Id, ev: LayerEvent) {
        self.push(
            owner,
            Stored::Layer {
                event: ev,
                claimed: Cell::new(false),
                marker: None,
            },
        );
    }

    /// Queues a layer event and returns an opaque token for this exact entry.
    ///
    /// The token may be cloned to share its claim identity, but a clone does not
    /// create another queue entry or permit a second claim.
    pub fn layer_tracked(&mut self, owner: Id, event: LayerEvent) -> LayerIntentToken {
        let marker = Arc::new(());
        self.push(
            owner,
            Stored::Layer {
                event,
                claimed: Cell::new(false),
                marker: Some(Arc::clone(&marker)),
            },
        );
        LayerIntentToken { owner, marker }
    }

    /// Claims only the next unclaimed layer event for this owner when its
    /// payload matches. Other intents in the owner bucket remain observable.
    pub fn claim_next_layer_event(&self, owner: Id, expected: LayerEvent) -> bool {
        let Some(bucket) = self.bucket_index(owner).and_then(|i| self.live().get(i)) else {
            return false;
        };
        let claimed = bucket.items.iter().find_map(|stored| match stored {
            Stored::Layer { event, claimed, .. } if !claimed.get() => Some((event, claimed)),
            _ => None,
        });
        let Some((event, claimed)) = claimed else {
            return false;
        };
        if *event != expected {
            return false;
        }
        claimed.set(true);
        if bucket.items.iter().all(|stored| match stored {
            Stored::Binding { claimed, .. } | Stored::Layer { claimed, .. } => claimed.get(),
            _ => false,
        }) {
            bucket.drained.set(true);
        }
        true
    }

    /// Claims only the intent represented by token; it never searches for a
    /// different event with the same owner and payload.
    ///
    /// `NotPending` means the token's queue entry was cleared, removed with its
    /// queue, or belongs to a different queue. `Mismatch` leaves the exact
    /// entry and all neighboring intents unchanged.
    pub fn claim_layer_event_token(
        &self,
        token: &LayerIntentToken,
        expected_event: LayerEvent,
    ) -> LayerIntentClaim {
        let Some(bucket) = self
            .bucket_index(token.owner)
            .and_then(|index| self.live().get(index))
        else {
            return LayerIntentClaim::NotPending;
        };
        let exact = bucket.items.iter().find_map(|stored| match stored {
            Stored::Layer {
                event,
                claimed,
                marker: Some(marker),
            } if Arc::ptr_eq(marker, &token.marker) => Some((*event, claimed)),
            _ => None,
        });
        let Some((event, claimed)) = exact else {
            return LayerIntentClaim::NotPending;
        };
        if event != expected_event {
            return LayerIntentClaim::Mismatch;
        }
        if claimed.get() {
            return LayerIntentClaim::AlreadyClaimed;
        }

        claimed.set(true);
        if bucket.items.iter().all(|stored| match stored {
            Stored::Binding { claimed, .. } | Stored::Layer { claimed, .. } => claimed.get(),
            _ => false,
        }) {
            bucket.drained.set(true);
        }
        LayerIntentClaim::Claimed
    }

    pub fn cancel(&mut self, owner: Id) {
        self.push(owner, Stored::Cancel);
    }

    pub fn undrained(&self) -> impl Iterator<Item = Id> + '_ {
        self.live()
            .iter()
            .filter(|b| {
                !b.drained.get()
                    && b.items.iter().any(|stored| match stored {
                        Stored::Binding { claimed, .. } | Stored::Layer { claimed, .. } => {
                            !claimed.get()
                        }
                        _ => true,
                    })
            })
            .map(|b| b.owner)
    }

    pub fn has_runtime_addressed(&self, owner: Id) -> bool {
        self.bucket_index(owner)
            .and_then(|i| self.live().get(i))
            .is_some_and(|b| {
                b.items.iter().any(|s| {
                    matches!(s, Stored::Cancel | Stored::FocusIn(_) | Stored::FocusOut(_))
                        || matches!(s, Stored::Binding { claimed, .. } if !claimed.get())
                        || matches!(s, Stored::Layer { claimed, .. } if !claimed.get())
                })
            })
    }

    #[cfg(test)]
    pub(crate) fn was_drained(&self, owner: Id) -> bool {
        self.bucket_index(owner)
            .and_then(|i| self.live().get(i))
            .is_some_and(|b| b.drained.get())
    }

    pub fn iter(&self, owner: Id) -> IntentIter<'_> {
        if self.used == 0 {
            return IntentIter {
                queue: self,
                bucket: None,
                pos: 0,
            };
        }
        let bucket = self.bucket_index(owner);
        if let Some(b) = bucket.and_then(|i| self.live().get(i)) {
            b.drained.set(true);
        }
        IntentIter {
            queue: self,
            bucket,
            pos: 0,
        }
    }

    pub fn claim_binding_chord(&self, owner: Id, chord: crate::event::Chord) -> Option<ActionKey> {
        let bucket = self.bucket_index(owner)?;
        let bucket = self.live().get(bucket)?;
        let claimed = bucket.items.iter().find_map(|stored| match stored {
            Stored::Binding {
                action,
                chord: effective,
                claimed,
            } if *effective == chord && !claimed.replace(true) => Some(*action),
            _ => None,
        });
        if claimed.is_some() {
            bucket.drained.set(true);
        }
        claimed
    }

    fn materialize(&self, s: &Stored) -> Intent<'_> {
        match s {
            Stored::Binding { action, .. } => Intent::Binding(*action),
            Stored::Key(k) => Intent::Key(*k),
            Stored::Paste(a, b) => {
                let range: Range<usize> = (*a as usize)..(*b as usize);
                Intent::Paste(self.arena.get(range).unwrap_or(""))
            }
            Stored::Pointer {
                phase,
                part,
                pos,
                local,
                mods,
            } => Intent::Pointer {
                phase: *phase,
                part: *part,
                pos: *pos,
                local: *local,
                mods: *mods,
            },
            Stored::Wheel {
                axis,
                delta,
                part,
                pos,
            } => Intent::Wheel {
                axis: *axis,
                delta: *delta,
                part: *part,
                pos: *pos,
            },
            Stored::FocusIn(via) => Intent::FocusIn { via: *via },
            Stored::FocusOut(to) => Intent::FocusOut { to: *to },
            Stored::Layer { event, .. } => Intent::Layer(*event),
            Stored::Cancel => Intent::Cancel,
        }
    }
}

impl fmt::Debug for IntentQueue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("IntentQueue");
        debug
            .field("arena", &"[redacted]")
            .field("arena_len", &self.arena.len())
            .field("buckets", &self.buckets)
            .field("used", &self.used)
            .field("table", &self.table)
            .field("overflow", &self.overflow);
        #[cfg(feature = "testing")]
        debug.field("probes", &self.probes);
        debug.finish()
    }
}

impl Drop for IntentQueue {
    fn drop(&mut self) {
        zeroize_string(&mut self.arena);
    }
}

fn zeroize_string(value: &mut String) {
    crate::secret::wipe_string(core::mem::take(value));
    *value = String::new();
}

#[derive(Debug, Clone)]
pub struct IntentIter<'f> {
    queue: &'f IntentQueue,
    bucket: Option<usize>,
    pos: usize,
}

impl<'f> Iterator for IntentIter<'f> {
    type Item = Intent<'f>;

    fn next(&mut self) -> Option<Self::Item> {
        let b = self.queue.live().get(self.bucket?)?;
        loop {
            let s = b.items.get(self.pos)?;
            self.pos = self.pos.wrapping_add(1);
            if matches!(s, Stored::Binding { claimed, .. } | Stored::Layer { claimed, .. } if claimed.get())
            {
                continue;
            }
            return Some(self.queue.materialize(s));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Axis, Chord, KeyCode};
    use crate::id::{Part, PartRef};
    use ratatui_core::layout::Position;

    #[test]
    fn paste_reaches_only_an_editing_owner() {
        let mut q = IntentQueue::new();
        let a = Id::root("a");
        let b = Id::root("b");
        q.paste(a, "hello");
        let got: Vec<Intent<'_>> = q.iter(a).collect();
        assert_eq!(got, vec![Intent::Paste("hello")]);
        assert_eq!(q.iter(b).count(), 0);
        assert!(q.was_drained(a));
        assert!(!q.was_drained(b));
    }

    #[test]
    fn paste_debug_output_redacts_payload_and_queue_arena() {
        let mut q = IntentQueue::new();
        let owner = Id::root("debug");
        q.paste(owner, "hunter2");
        let intent = q.iter(owner).next().expect("paste intent");
        assert!(!format!("{intent:?}").contains("hunter2"));
        assert!(!format!("{q:?}").contains("hunter2"));
        q.clear();
        assert!(!format!("{q:?}").contains("hunter2"));
    }

    #[test]
    fn empty_queue_probe_is_a_bool_check_and_owners_hash_into_the_table() {
        let mut q = IntentQueue::new();
        assert!(q.is_empty());
        assert_eq!(q.iter(Id::root("x")).count(), 0);
        for i in 0..40usize {
            q.key(
                Id::root("o").index(i),
                Key {
                    code: KeyCode::Enter,
                    mods: KeyModifiers::NONE,
                },
            );
        }
        assert!(q.overflow);
        for i in 0..40usize {
            assert_eq!(q.iter(Id::root("o").index(i)).count(), 1);
        }
        assert_eq!(q.undrained().count(), 0);
        q.clear();
        assert!(q.is_empty() && !q.overflow);
        assert_eq!(
            q.iter(Id::root("o").index(3)).count(),
            0,
            "cleared buckets deliver nothing"
        );
    }

    #[test]
    fn undrained_buckets_are_reported() {
        let mut q = IntentQueue::new();
        let a = Id::root("a");
        q.cancel(a);
        assert_eq!(q.undrained().collect::<Vec<_>>(), vec![a]);
        let _ = q.iter(a);
        assert_eq!(q.undrained().count(), 0);
    }

    #[test]
    fn claiming_layer_receipt_preserves_unrelated_intents_and_diagnostics() {
        let mut q = IntentQueue::new();
        let owner = Id::root("completion");
        let event = LayerEvent::Dismissed(crate::layer_types::DismissReason::Programmatic);
        q.layer(owner, event);
        q.key(
            owner,
            Key {
                code: KeyCode::Enter,
                mods: KeyModifiers::NONE,
            },
        );

        assert!(!q.claim_next_layer_event(
            owner,
            LayerEvent::Dismissed(crate::layer_types::DismissReason::Esc)
        ));
        assert!(q.claim_next_layer_event(owner, event));
        assert_eq!(q.undrained().collect::<Vec<_>>(), vec![owner]);
        assert_eq!(
            q.iter(owner).collect::<Vec<_>>(),
            vec![Intent::Key(Key {
                code: KeyCode::Enter,
                mods: KeyModifiers::NONE,
            })]
        );
        assert_eq!(q.undrained().count(), 0);
    }

    #[test]
    fn binding_claim_marks_only_a_successful_exact_chord_and_preserves_order() {
        let mut q = IntentQueue::new();
        let owner = Id::root("claim");
        let enter = Chord::key(KeyCode::Enter);
        q.binding(owner, ActionKey::CONFIRM, enter);
        q.key(
            owner,
            Key {
                code: KeyCode::Char('x'),
                mods: KeyModifiers::NONE,
            },
        );
        assert_eq!(
            q.claim_binding_chord(owner, Chord::key(KeyCode::Char(' '))),
            None
        );
        assert!(!q.was_drained(owner));
        assert_eq!(
            q.claim_binding_chord(owner, enter),
            Some(ActionKey::CONFIRM)
        );
        assert!(q.was_drained(owner));
        assert_eq!(
            q.iter(owner).collect::<Vec<_>>(),
            vec![Intent::Key(Key {
                code: KeyCode::Char('x'),
                mods: KeyModifiers::NONE,
            })]
        );
    }

    #[test]
    fn exact_layer_tokens_claim_once_and_clones_share_the_claim() {
        let mut queue = IntentQueue::new();
        let owner = Id::root("tracked-layer");
        let event = LayerEvent::Dismissed(crate::layer_types::DismissReason::Esc);
        let token = queue.layer_tracked(owner, event);
        let clone = token.clone();

        assert_eq!(
            queue.claim_layer_event_token(&token, event),
            LayerIntentClaim::Claimed
        );
        assert_eq!(
            queue.claim_layer_event_token(&clone, event),
            LayerIntentClaim::AlreadyClaimed
        );
        assert!(
            !queue.is_empty(),
            "claimed entries remain stored until clear"
        );
    }

    #[test]
    fn exact_claim_does_not_skip_legacy_claimed_identical_event_or_drain_neighbors() {
        let mut queue = IntentQueue::new();
        let owner = Id::root("tracked-layer-neighbors");
        let event = LayerEvent::Dismissed(crate::layer_types::DismissReason::Esc);
        let owned = queue.layer_tracked(owner, event);
        let successor = queue.layer_tracked(owner, event);
        let key = crate::event::Key {
            code: KeyCode::Enter,
            mods: crate::event::KeyModifiers::NONE,
        };
        let pointer_part = PartRef::of(Part::ROW);
        let pointer_pos = Position::new(2, 3);
        queue.binding(owner, ActionKey::CONFIRM, Chord::key(KeyCode::Enter));
        queue.key(owner, key);
        queue.pointer(
            owner,
            Phase::Move,
            pointer_part,
            pointer_pos,
            pointer_pos,
            crate::event::KeyModifiers::NONE,
        );
        queue.focus_in(owner, FocusVia::Programmatic);
        queue.focus_out(owner, None);
        queue.wheel(owner, Axis::V, 1, pointer_part, pointer_pos);

        assert!(queue.claim_next_layer_event(owner, event));
        assert_eq!(
            queue.claim_layer_event_token(&owned, event),
            LayerIntentClaim::AlreadyClaimed
        );
        assert_eq!(queue.undrained().collect::<Vec<_>>(), vec![owner]);
        assert_eq!(
            queue.iter(owner).collect::<Vec<_>>(),
            vec![
                Intent::Layer(event),
                Intent::Binding(ActionKey::CONFIRM),
                Intent::Key(key),
                Intent::Pointer {
                    phase: Phase::Move,
                    part: pointer_part,
                    pos: pointer_pos,
                    local: pointer_pos,
                    mods: crate::event::KeyModifiers::NONE,
                },
                Intent::FocusIn {
                    via: FocusVia::Programmatic,
                },
                Intent::FocusOut { to: None },
                Intent::Wheel {
                    axis: Axis::V,
                    delta: 1,
                    part: pointer_part,
                    pos: pointer_pos,
                },
            ]
        );
        assert_eq!(
            queue.claim_layer_event_token(&successor, event),
            LayerIntentClaim::Claimed
        );
    }

    #[test]
    fn exact_token_mismatch_leaves_target_and_neighbor_claimable() {
        let mut queue = IntentQueue::new();
        let owner = Id::root("tracked-layer-mismatch");
        let target = LayerEvent::Opened;
        let neighbor = LayerEvent::Dismissed(crate::layer_types::DismissReason::Esc);
        let token = queue.layer_tracked(owner, target);
        queue.layer_tracked(owner, neighbor);

        assert_eq!(
            queue.claim_layer_event_token(&token, neighbor),
            LayerIntentClaim::Mismatch
        );
        assert_eq!(
            queue.claim_layer_event_token(&token, target),
            LayerIntentClaim::Claimed
        );
        assert!(queue.claim_next_layer_event(owner, neighbor));
    }

    #[test]
    fn exact_tokens_cannot_alias_foreign_cleared_or_reused_queue_entries() {
        let owner = Id::root("tracked-layer-queue-identity");
        let event = LayerEvent::Opened;
        let mut queue = IntentQueue::new();
        let stale = queue.layer_tracked(owner, event);
        queue.clear();
        let current = queue.layer_tracked(owner, event);

        assert_eq!(
            queue.claim_layer_event_token(&stale, event),
            LayerIntentClaim::NotPending
        );
        assert_eq!(
            queue.claim_layer_event_token(&current, event),
            LayerIntentClaim::Claimed
        );

        let mut live_current_queue = IntentQueue::new();
        let live_current = live_current_queue.layer_tracked(owner, event);
        let mut live_foreign_queue = IntentQueue::new();
        let live_foreign = live_foreign_queue.layer_tracked(owner, event);
        assert_eq!(
            live_current_queue.claim_layer_event_token(&live_foreign, event),
            LayerIntentClaim::NotPending,
            "a live queue with the same owner and payload still cannot claim a foreign token"
        );
        assert_eq!(
            live_current_queue.claim_layer_event_token(&live_current, event),
            LayerIntentClaim::Claimed
        );
        assert_eq!(
            live_foreign_queue.claim_layer_event_token(&live_current, event),
            LayerIntentClaim::NotPending
        );
        assert_eq!(
            live_foreign_queue.claim_layer_event_token(&live_foreign, event),
            LayerIntentClaim::Claimed
        );

        let foreign = {
            let mut foreign_queue = IntentQueue::default();
            foreign_queue.layer_tracked(owner, event)
        };
        let mut fresh_queue = IntentQueue::new();
        let fresh = fresh_queue.layer_tracked(owner, event);
        assert_eq!(
            fresh_queue.claim_layer_event_token(&foreign, event),
            LayerIntentClaim::NotPending
        );
        assert_eq!(
            fresh_queue.claim_layer_event_token(&fresh, event),
            LayerIntentClaim::Claimed
        );

        let mut movable = IntentQueue::default();
        let moved_token = movable.layer_tracked(owner, event);
        let moved_queue = movable;
        assert_eq!(
            moved_queue.claim_layer_event_token(&moved_token, event),
            LayerIntentClaim::Claimed
        );
    }

    #[test]
    fn queue_and_token_keep_their_existing_send_contracts() {
        fn assert_send<T: Send>() {}
        fn assert_send_sync<T: Send + Sync>() {}

        assert_send::<IntentQueue>();
        assert_send_sync::<LayerIntentToken>();
    }
}
