//! The one reconcile rule (`COMPONENT_ARCHITECTURE.md` §12.2, §20.9-3, §21 item 21).
//!
//! Keep the cursor key if it is still present; else the nearest surviving
//! key by the previous index (forward first, then backward); else the first
//! enabled key; else `None`. Checked sets drop vanished keys and report the
//! count. Scroll offset is clamped to the new length. A generation stamp
//! `(len, key(first), key(last))` short-circuits the common no-op, and a
//! cached index is probed before any scan.

use crate::id::ItemKey;
use crate::scroll::ScrollState;

use super::key::KeySet;

/// What reconciliation did.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Reconciliation {
    /// Everything still names the same items.
    Unchanged,
    /// The cursor moved to a surviving neighbour.
    CursorMoved(ItemKey),
    /// Nothing is left to point at.
    CursorLost,
    /// Checked keys vanished; the count.
    SelectionDropped(usize),
}

/// Implemented on every collection state type.
pub trait Reconcile {
    /// Reconcile against `len` items keyed by `key`.
    fn reconcile(&mut self, len: usize, key: impl Fn(usize) -> ItemKey) -> Reconciliation;
    /// The caller mutated items in place without changing `len` or the end
    /// keys: force the next `reconcile` to scan.
    fn invalidate(&mut self);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Stamp {
    len: usize,
    first: Option<ItemKey>,
    last: Option<ItemKey>,
}

/// The reusable core every keyed collection state embeds: cursor key,
/// cached index, checked set, scroll and the generation stamp.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct CollectionCore {
    cursor: Option<ItemKey>,
    cursor_index: usize,
    checked: KeySet,
    scroll: ScrollState,
    stamp: Option<Stamp>,
}

impl CollectionCore {
    /// An empty core.
    pub const fn new() -> Self {
        CollectionCore {
            cursor: None,
            cursor_index: 0,
            checked: KeySet::new(),
            scroll: ScrollState::new(0),
            stamp: None,
        }
    }

    /// The cursor key.
    pub const fn cursor(&self) -> Option<ItemKey> {
        self.cursor
    }

    /// The cursor's display index as of the last reconcile.
    pub const fn cursor_index(&self) -> usize {
        self.cursor_index
    }

    /// Point the cursor at `(index, key)`.
    pub fn set_cursor(&mut self, index: usize, key: ItemKey) {
        self.cursor = Some(key);
        self.cursor_index = index;
        self.scroll.ensure_visible_on_next_layout(index);
    }

    /// Clear the cursor.
    pub fn clear_cursor(&mut self) {
        self.cursor = None;
        self.cursor_index = 0;
    }

    /// The checked set.
    pub const fn checked(&self) -> &KeySet {
        &self.checked
    }

    /// The checked set, mutably.
    pub const fn checked_mut(&mut self) -> &mut KeySet {
        &mut self.checked
    }

    /// The scroll state.
    pub const fn scroll(&self) -> &ScrollState {
        &self.scroll
    }

    /// The scroll state, mutably.
    pub const fn scroll_mut(&mut self) -> &mut ScrollState {
        &mut self.scroll
    }

    fn stamp_of(len: usize, key: &impl Fn(usize) -> ItemKey) -> Stamp {
        Stamp {
            len,
            first: (len > 0).then(|| key(0)),
            last: (len > 0).then(|| key(len.saturating_sub(1))),
        }
    }

    fn nearest(
        &self,
        len: usize,
        key: &impl Fn(usize) -> ItemKey,
        enabled: &impl Fn(usize) -> bool,
    ) -> Option<(usize, ItemKey)> {
        if len == 0 {
            return None;
        }
        let start = self.cursor_index.min(len.saturating_sub(1));
        // forward first
        for i in start..len {
            if enabled(i) {
                return Some((i, key(i)));
            }
        }
        // then backward
        for i in (0..start).rev() {
            if enabled(i) {
                return Some((i, key(i)));
            }
        }
        None
    }

    /// The one rule, with an enabled predicate for the fallbacks.
    pub fn reconcile_with(
        &mut self,
        len: usize,
        key: impl Fn(usize) -> ItemKey,
        enabled: impl Fn(usize) -> bool,
    ) -> Reconciliation {
        let stamp = Self::stamp_of(len, &key);
        if self.stamp == Some(stamp) {
            return Reconciliation::Unchanged;
        }
        self.stamp = Some(stamp);
        self.scroll.set_content(len);
        // checked keys: keep those present; one pass over the items
        let dropped = if self.checked.keys().is_empty() {
            0
        } else {
            let keys = self.checked.keys();
            let mut seen = vec![false; keys.len()];
            for i in 0..len {
                if let Ok(pos) = keys.binary_search(&key(i))
                    && let Some(s) = seen.get_mut(pos)
                {
                    *s = true;
                }
            }
            let mut idx = 0usize;
            self.checked.retain(|_| {
                let keep = seen.get(idx).copied().unwrap_or(false);
                idx = idx.saturating_add(1);
                keep
            })
        };
        let outcome = match self.cursor {
            None => Reconciliation::Unchanged,
            Some(cur) => {
                // probe the cached index before any scan
                let probe = self.cursor_index < len && key(self.cursor_index) == cur;
                let found = if probe {
                    Some(self.cursor_index)
                } else {
                    (0..len).find(|&i| key(i) == cur)
                };
                match found {
                    Some(i) => {
                        self.cursor_index = i;
                        Reconciliation::Unchanged
                    }
                    None => {
                        if let Some((i, k)) = self.nearest(len, &key, &enabled) {
                            self.cursor = Some(k);
                            self.cursor_index = i;
                            Reconciliation::CursorMoved(k)
                        } else {
                            self.cursor = None;
                            self.cursor_index = 0;
                            Reconciliation::CursorLost
                        }
                    }
                }
            }
        };
        if dropped > 0 && outcome == Reconciliation::Unchanged {
            Reconciliation::SelectionDropped(dropped)
        } else {
            outcome
        }
    }
}

impl Reconcile for CollectionCore {
    fn reconcile(&mut self, len: usize, key: impl Fn(usize) -> ItemKey) -> Reconciliation {
        self.reconcile_with(len, key, |_| true)
    }

    fn invalidate(&mut self) {
        self.stamp = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;

    fn keys(v: &[u64]) -> impl Fn(usize) -> ItemKey + '_ {
        move |i| ItemKey::num(v.get(i).copied().unwrap_or(0))
    }

    #[test]
    fn reconcile_keeps_a_surviving_key() {
        let mut c = CollectionCore::new();
        c.set_cursor(2, ItemKey::num(30));
        let items = [10, 20, 30, 40];
        assert_eq!(c.reconcile(4, keys(&items)), Reconciliation::Unchanged);
        let reordered = [40, 30, 20, 10];
        assert_eq!(c.reconcile(4, keys(&reordered)), Reconciliation::Unchanged);
        assert_eq!(c.cursor(), Some(ItemKey::num(30)));
        assert_eq!(c.cursor_index(), 1);
    }

    #[test]
    fn reconcile_takes_the_nearest_forward_then_backward() {
        let mut c = CollectionCore::new();
        c.set_cursor(2, ItemKey::num(30));
        let _ = c.reconcile(4, keys(&[10, 20, 30, 40]));
        let removed = [10, 20, 40];
        assert_eq!(
            c.reconcile(3, keys(&removed)),
            Reconciliation::CursorMoved(ItemKey::num(40))
        );
        c.set_cursor(2, ItemKey::num(40));
        let _ = c.reconcile(3, keys(&removed));
        assert_eq!(
            c.reconcile(2, keys(&[10, 20])),
            Reconciliation::CursorMoved(ItemKey::num(20))
        );
    }

    #[test]
    fn reconcile_falls_back_to_the_first_enabled_key() {
        let mut c = CollectionCore::new();
        c.set_cursor(3, ItemKey::num(99));
        let items = [1, 2, 3];
        let r = c.reconcile_with(3, keys(&items), |i| i == 1);
        assert_eq!(r, Reconciliation::CursorMoved(ItemKey::num(2)));
    }

    #[test]
    fn reconcile_yields_cursor_lost_when_empty() {
        let mut c = CollectionCore::new();
        c.set_cursor(0, ItemKey::num(1));
        assert_eq!(c.reconcile(0, keys(&[])), Reconciliation::CursorLost);
        assert_eq!(c.cursor(), None);
    }

    #[test]
    fn reconcile_drops_vanished_checked_keys_and_reports_the_count() {
        let mut c = CollectionCore::new();
        c.checked_mut().insert(ItemKey::num(2));
        c.checked_mut().insert(ItemKey::num(3));
        c.checked_mut().insert(ItemKey::num(9));
        let r = c.reconcile(3, keys(&[1, 2, 3]));
        assert_eq!(r, Reconciliation::SelectionDropped(1));
        assert_eq!(c.checked().keys(), &[ItemKey::num(2), ItemKey::num(3)]);
    }

    #[test]
    fn reconcile_clamps_the_scroll_offset() {
        let mut c = CollectionCore::new();
        c.scroll_mut().set_viewport(5);
        let _ = c.reconcile(100, ItemKey::index);
        c.scroll_mut().scroll_to(90);
        let _ = c.reconcile(10, ItemKey::index);
        assert_eq!(c.scroll().offset(), 5);
    }

    #[test]
    fn reconcile_runs_before_any_action_is_emitted() {
        // the core's contract: a collection calls `reconcile` first, so an
        // action produced afterwards names a surviving key
        let mut c = CollectionCore::new();
        c.set_cursor(1, ItemKey::num(20));
        let _ = c.reconcile(3, keys(&[10, 20, 30]));
        let r = c.reconcile(2, keys(&[10, 30]));
        assert_eq!(r, Reconciliation::CursorMoved(ItemKey::num(30)));
        let action_key = c.cursor();
        assert_eq!(action_key, Some(ItemKey::num(30)));
    }

    #[test]
    fn generation_stamp_skips_a_no_op_reconcile() {
        let mut c = CollectionCore::new();
        c.set_cursor(0, ItemKey::num(10));
        let calls = Cell::new(0usize);
        let counting = |i: usize| {
            calls.set(calls.get().saturating_add(1));
            ItemKey::num(10u64.saturating_add(i as u64 * 10))
        };
        let _ = c.reconcile(1000, counting);
        let after_first = calls.get();
        let _ = c.reconcile(1000, counting);
        // only the two stamp keys were read the second time
        assert_eq!(calls.get().saturating_sub(after_first), 2);
        c.invalidate();
        let _ = c.reconcile(1000, counting);
        assert!(calls.get().saturating_sub(after_first) > 2);
    }

    #[test]
    fn cached_index_probe_hits_before_a_scan() {
        let mut c = CollectionCore::new();
        c.set_cursor(500, ItemKey::num(500));
        let calls = Cell::new(0usize);
        let counting = |i: usize| {
            calls.set(calls.get().saturating_add(1));
            ItemKey::num(i as u64)
        };
        let _ = c.reconcile(1000, counting);
        // stamp (2) + probe (1): no scan of 500 items
        assert_eq!(calls.get(), 3);
    }
}
