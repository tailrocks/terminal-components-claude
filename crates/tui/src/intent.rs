//! Pre-resolved input (`COMPONENT_ARCHITECTURE.md` §6.1, §21 item 6).
//!
//! The runtime resolves every `Input` to an owner and a part against the
//! last frame's registry and focus ring (§3.3 steps 3–6) and files the
//! resulting [`Intent`]s into a per-frame [`IntentQueue`] keyed by owner.
//! The queue is frozen for the whole of `app.update`; a component drains
//! its own bucket through [`Cx::intents`](crate::ui::Cx::intents), which
//! borrows only the queue, so services on `Cx` stay usable inside the loop.

use core::cell::Cell;
use core::fmt;
use core::ops::Range;

use ratatui_core::layout::Position;

use crate::action::ActionKey;
use crate::event::{Axis, Key, KeyModifiers};
use crate::id::{Id, PartRef};
use crate::layer::LayerEvent;

/// What a component actually receives.
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Intent<'f> {
    /// A declared component action resolved through its effective chord.
    Binding(ActionKey),
    /// A key press delivered to the focused owner.
    Key(Key),
    /// Pasted text delivered to the focused owner iff it declared `EDITING`.
    Paste(&'f str),
    /// A pointer phase over one of the owner's parts.
    Pointer {
        /// The phase.
        phase: Phase,
        /// The part under the pointer.
        part: PartRef,
        /// Absolute position.
        pos: Position,
        /// Position relative to the part's area (or the captured area).
        local: Position,
        /// Modifiers held.
        mods: KeyModifiers,
    },
    /// Wheel motion over a scroll region; `delta` is already multiplied by
    /// `design.motion.wheel_rows`.
    Wheel {
        /// The axis.
        axis: Axis,
        /// Rows or columns to scroll, signed.
        delta: i16,
        /// The scroll region's part.
        part: PartRef,
        /// Absolute position.
        pos: Position,
    },
    /// Focus arrived.
    FocusIn {
        /// How.
        via: FocusVia,
    },
    /// Focus left, towards `to`.
    FocusOut {
        /// The new owner, if any.
        to: Option<Id>,
    },
    /// A layer owned by this component changed lifecycle.
    Layer(LayerEvent),
    /// Esc reached this owner after layer dismissal.
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

/// Pointer phases.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Pointer moved over the topmost live non-decorative part.
    Move,
    /// Primary button down on the part.
    Press,
    /// Primary button up (anywhere, after a press on the owner).
    Release,
    /// Press + release on the same part.
    Click,
    /// A second click within the double-click window on the same part.
    DoubleClick,
    /// Secondary button down on the part.
    Secondary,
    /// The first motion with the button held.
    DragStart,
    /// Motion with the button held.
    Drag,
    /// The button was released after a drag.
    DragEnd,
}

/// How focus arrived.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusVia {
    /// Tab / Shift+Tab.
    Keyboard,
    /// A press focused the owner.
    Pointer,
    /// `cx.focus(id)` or reconciliation.
    Programmatic,
    /// Restored after a layer closed.
    Restore,
}

/// Owned form of an intent; `Paste` indexes the queue's arena.
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
    Layer(LayerEvent),
    Cancel,
}

#[derive(Debug)]
struct Bucket {
    owner: Id,
    items: Vec<Stored>,
    drained: Cell<bool>,
}

/// Number of open-addressed slots for the owner table.
const TABLE: usize = 32;
/// Owners beyond this fall back to a linear scan.
const TABLE_MAX_OWNERS: usize = 24;

/// The per-frame intent queue, keyed by owner.
pub(crate) struct IntentQueue {
    arena: String,
    buckets: Vec<Bucket>,
    /// Buckets in use this frame; the rest are retained for reuse.
    used: usize,
    table: [Option<(u64, u16)>; TABLE],
    overflow: bool,
    /// Bucket probes performed since construction (adjudication 2.6): the
    /// deterministic replacement for a ±10 % wall-clock band on a ~600 ns
    /// measurement.
    #[cfg(feature = "testing")]
    probes: Cell<usize>,
}

impl IntentQueue {
    pub(crate) const fn new() -> Self {
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

    /// Bucket probes performed since construction.
    #[cfg(feature = "testing")]
    pub(crate) fn probes(&self) -> usize {
        self.probes.get()
    }

    /// Empty the queue, keeping bucket allocations while wiping paste data.
    pub(crate) fn clear(&mut self) {
        zeroize_string(&mut self.arena);
        for b in &mut self.buckets {
            b.items.clear();
            b.drained.set(false);
        }
        self.used = 0;
        self.table = [None; TABLE];
        self.overflow = false;
    }

    pub(crate) fn is_empty(&self) -> bool {
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

    pub(crate) fn key(&mut self, owner: Id, k: Key) {
        self.push(owner, Stored::Key(k));
    }

    pub(crate) fn binding(&mut self, owner: Id, action: ActionKey, chord: crate::event::Chord) {
        self.push(
            owner,
            Stored::Binding {
                action,
                chord,
                claimed: Cell::new(false),
            },
        );
    }

    pub(crate) fn paste(&mut self, owner: Id, text: &str) {
        let start = self.arena.len() as u32;
        self.arena.push_str(text);
        let end = self.arena.len() as u32;
        self.push(owner, Stored::Paste(start, end));
    }

    pub(crate) fn pointer(
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

    pub(crate) fn wheel(
        &mut self,
        owner: Id,
        axis: Axis,
        delta: i16,
        part: PartRef,
        pos: Position,
    ) {
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

    pub(crate) fn focus_in(&mut self, owner: Id, via: FocusVia) {
        self.push(owner, Stored::FocusIn(via));
    }

    pub(crate) fn focus_out(&mut self, owner: Id, to: Option<Id>) {
        self.push(owner, Stored::FocusOut(to));
    }

    pub(crate) fn layer(&mut self, owner: Id, ev: LayerEvent) {
        self.push(owner, Stored::Layer(ev));
    }

    pub(crate) fn cancel(&mut self, owner: Id) {
        self.push(owner, Stored::Cancel);
    }

    /// Owners that hold intents and drained nothing (§3.3 step 9).
    pub(crate) fn undrained(&self) -> impl Iterator<Item = Id> + '_ {
        self.live()
            .iter()
            .filter(|b| !b.drained.get() && !b.items.is_empty())
            .map(|b| b.owner)
    }

    /// Whether `owner`'s bucket holds an intent the **runtime itself**
    /// addressed to it (§3.3 step 9).
    ///
    /// `Layer`, `Cancel`, `FocusIn` and `FocusOut` are addressed by the
    /// runtime to a known owner, so a lost one is always a defect — whatever
    /// that owner registered. Pointer intents are different: `deliverable`
    /// already keeps them away from a `Decorative` region, which is why the
    /// undelivered-intent guard can widen for these four without re-opening
    /// §21 item 13's exemption for container regions.
    pub(crate) fn has_runtime_addressed(&self, owner: Id) -> bool {
        self.bucket_index(owner)
            .and_then(|i| self.live().get(i))
            .is_some_and(|b| {
                b.items.iter().any(|s| {
                    matches!(
                        s,
                        Stored::Binding { .. }
                            | Stored::Layer(_)
                            | Stored::Cancel
                            | Stored::FocusIn(_)
                            | Stored::FocusOut(_)
                    )
                })
            })
    }

    /// Whether `owner` drained its bucket this pass.
    #[cfg(test)]
    pub(crate) fn was_drained(&self, owner: Id) -> bool {
        self.bucket_index(owner)
            .and_then(|i| self.live().get(i))
            .is_some_and(|b| b.drained.get())
    }

    /// The frozen-queue probe (§20.9-12): one `bool` check when empty,
    /// otherwise one open-addressed probe.
    pub(crate) fn iter(&self, owner: Id) -> IntentIter<'_> {
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

    pub(crate) fn claim_binding_chord(
        &self,
        owner: Id,
        chord: crate::event::Chord,
    ) -> Option<ActionKey> {
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
            Stored::Layer(ev) => Intent::Layer(*ev),
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
    let mut bytes = core::mem::take(value).into_bytes();
    bytes.fill(0);
    core::hint::black_box(&bytes);
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    bytes.clear();
    *value = String::new();
}

/// Iterator over one owner's intents for the frame. A named type so it can
/// outlive the `&Cx` borrow that produced it (§22.3).
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
            if matches!(s, Stored::Binding { claimed, .. } if claimed.get()) {
                continue;
            }
            return Some(self.queue.materialize(s));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Chord, KeyCode};

    #[test]
    fn paste_reaches_only_an_editing_owner() {
        // the queue-level half: paste text lives in the arena and is delivered
        // to exactly the owner it was filed under; the runtime files it only
        // when the focused owner declared EDITING (runtime tests cover that)
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
}
