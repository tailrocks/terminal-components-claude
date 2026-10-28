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

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerIntentClaim {
    Claimed,

    AlreadyClaimed,

    NotPending,

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
        }
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
