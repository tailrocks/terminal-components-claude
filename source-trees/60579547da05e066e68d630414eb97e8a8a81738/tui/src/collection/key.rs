//! Keys, selection sets and the default accessors (`COMPONENT_ARCHITECTURE.md` §12.2, §17.0 A8, §21 item 5).

use core::cell::Cell;

use crate::id::ItemKey;

use super::rowui::RowUi;

/// Selection behaviour of a collection.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SelectMode {
    /// One item at a time.
    #[default]
    Single,
    /// Any number of items.
    Multi,
    /// A contiguous range from an anchor.
    Range,
    /// No selection.
    None,
}

/// Default key accessor: `ItemKey::index(i)` — UNSTABLE under insert,
/// remove and reorder; call `.key(…)` for stable identity.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ByIndex;

/// Default row painter: the item's `Display` through `RowUi::label_fmt`,
/// no allocation.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct DefaultRow;

/// A key accessor.
pub trait KeyFn<T> {
    /// The key of `item` at display `index`.
    fn key(&self, item: &T, index: usize) -> ItemKey;
}

impl<T, F: Fn(&T) -> ItemKey> KeyFn<T> for F {
    fn key(&self, item: &T, _index: usize) -> ItemKey {
        self(item)
    }
}

impl<T> KeyFn<T> for ByIndex {
    fn key(&self, _item: &T, index: usize) -> ItemKey {
        ItemKey::index(index)
    }
}

/// A row painter.
pub trait RowFn<T> {
    /// Paint `item` into `u`.
    fn row(&self, item: &T, u: &mut RowUi<'_>);
}

impl<T, F: Fn(&T, &mut RowUi<'_>)> RowFn<T> for F {
    fn row(&self, item: &T, u: &mut RowUi<'_>) {
        self(item, u);
    }
}

impl<T: core::fmt::Display> RowFn<T> for DefaultRow {
    fn row(&self, item: &T, u: &mut RowUi<'_>) {
        u.label_fmt(format_args!("{item}"));
    }
}

/// A selection set with an inverted representation, so "select all" never
/// materialises every key. The `Vec` is kept sorted; `contains` is a binary
/// search.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeySet {
    /// Exactly these keys.
    Only(Vec<ItemKey>),
    /// Every key except these.
    AllExcept(Vec<ItemKey>),
}

impl Default for KeySet {
    fn default() -> Self {
        KeySet::Only(Vec::new())
    }
}

impl KeySet {
    /// The empty set.
    pub const fn new() -> Self {
        KeySet::Only(Vec::new())
    }

    fn list(&self) -> &Vec<ItemKey> {
        match self {
            KeySet::Only(v) | KeySet::AllExcept(v) => v,
        }
    }

    fn list_mut(&mut self) -> &mut Vec<ItemKey> {
        match self {
            KeySet::Only(v) | KeySet::AllExcept(v) => v,
        }
    }

    const fn inverted(&self) -> bool {
        matches!(self, KeySet::AllExcept(_))
    }

    /// Whether `k` is in the set (binary search).
    pub fn contains(&self, k: ItemKey) -> bool {
        self.contains_counting(k).0
    }

    /// `contains` plus the number of key comparisons made (for the
    /// binary-search test).
    pub fn contains_counting(&self, k: ItemKey) -> (bool, usize) {
        let count = Cell::new(0usize);
        let found = self
            .list()
            .binary_search_by(|probe| {
                count.set(count.get().saturating_add(1));
                probe.cmp(&k)
            })
            .is_ok();
        (found != self.inverted(), count.get())
    }

    fn add(list: &mut Vec<ItemKey>, k: ItemKey) {
        if let Err(i) = list.binary_search(&k) {
            list.insert(i, k);
        }
    }

    fn drop(list: &mut Vec<ItemKey>, k: ItemKey) {
        if let Ok(i) = list.binary_search(&k) {
            list.remove(i);
        }
    }

    /// Add `k`.
    pub fn insert(&mut self, k: ItemKey) {
        match self {
            KeySet::Only(v) => Self::add(v, k),
            KeySet::AllExcept(v) => Self::drop(v, k),
        }
    }

    /// Remove `k`.
    pub fn remove(&mut self, k: ItemKey) {
        match self {
            KeySet::Only(v) => Self::drop(v, k),
            KeySet::AllExcept(v) => Self::add(v, k),
        }
    }

    /// Toggle `k`.
    pub fn toggle(&mut self, k: ItemKey) {
        if self.contains(k) {
            self.remove(k);
        } else {
            self.insert(k);
        }
    }

    /// Select everything (0 allocations).
    pub fn all(&mut self) {
        *self = KeySet::AllExcept(Vec::new());
    }

    /// Select nothing.
    pub fn none(&mut self) {
        *self = KeySet::Only(Vec::new());
    }

    /// The number of selected keys given `total` items.
    pub fn len_in(&self, total: usize) -> usize {
        match self {
            KeySet::Only(v) => v.len().min(total),
            KeySet::AllExcept(v) => total.saturating_sub(v.len()),
        }
    }

    /// Whether nothing is selected (for a non-inverted set).
    pub fn is_empty(&self) -> bool {
        matches!(self, KeySet::Only(v) if v.is_empty())
    }

    /// Keep only keys for which `keep` holds; returns the dropped count.
    /// For an inverted set the exclusions are pruned instead and `0` is
    /// reported, because vanished keys were never selected.
    pub fn retain(&mut self, mut keep: impl FnMut(ItemKey) -> bool) -> usize {
        let inverted = self.inverted();
        let list = self.list_mut();
        let before = list.len();
        list.retain(|k| keep(*k));
        if inverted {
            0
        } else {
            before.saturating_sub(list.len())
        }
    }

    /// The explicit keys (selected for `Only`, excluded for `AllExcept`).
    pub fn keys(&self) -> &[ItemKey] {
        self.list()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(n: u64) -> ItemKey {
        ItemKey::num(n)
    }

    #[test]
    fn key_set_stays_sorted_after_insert_remove_toggle_retain() {
        let mut s = KeySet::new();
        for n in [5, 1, 9, 3, 7] {
            s.insert(k(n));
        }
        assert_eq!(s.keys(), &[k(1), k(3), k(5), k(7), k(9)]);
        s.remove(k(5));
        s.toggle(k(4));
        s.toggle(k(1));
        assert_eq!(s.keys(), &[k(3), k(4), k(7), k(9)]);
        assert_eq!(s.retain(|key| key != k(7)), 1);
        assert_eq!(s.keys(), &[k(3), k(4), k(9)]);
        assert_eq!(s.len_in(100), 3);
        s.all();
        assert!(s.contains(k(42)) && s.len_in(100) == 100);
        s.remove(k(42));
        assert!(!s.contains(k(42)) && s.len_in(100) == 99);
        s.insert(k(42));
        assert!(s.contains(k(42)));
        assert_eq!(s.retain(|_| false), 0);
        s.none();
        assert!(s.is_empty());
    }

    #[test]
    fn key_set_contains_is_binary_search() {
        let mut s = KeySet::new();
        for n in 0..1024u64 {
            s.insert(k(n));
        }
        let (found, cmps) = s.contains_counting(k(700));
        assert!(found);
        assert!(cmps <= 11, "{cmps} comparisons for 1024 keys");
        let (found, cmps) = s.contains_counting(k(5000));
        assert!(!found && cmps <= 11);
    }

    #[test]
    fn default_accessors_are_index_and_display() {
        assert_eq!(ByIndex.key(&"x", 3), ItemKey::index(3));
        let f = |s: &&str| ItemKey::text(s);
        assert_eq!(f.key(&"x", 3), ItemKey::text("x"));
        assert_eq!(SelectMode::default(), SelectMode::Single);
    }
}
