use core::cell::Cell;

use crate::id::ItemKey;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SelectMode {
    #[default]
    Single,
    Multi,
    Range,
    None,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ByIndex;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct DefaultRow;

pub trait KeyFn<T> {
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

pub fn key_at<T, K: KeyFn<T>>(key: &K, items: &[T], i: usize) -> ItemKey {
    items.get(i).map_or(ItemKey::index(i), |it| key.key(it, i))
}

pub fn index_of<T, K: KeyFn<T>>(
    key: &K,
    items: &[T],
    item: ItemKey,
    hint: Option<usize>,
) -> Option<usize> {
    if let Some(h) = hint
        && h < items.len()
        && key_at(key, items, h) == item
    {
        return Some(h);
    }
    (0..items.len()).find(|&i| key_at(key, items, i) == item)
}

pub fn key_at_with<T>(items: &[T], i: usize, f: impl Fn(&T) -> ItemKey) -> ItemKey {
    items.get(i).map_or(ItemKey::index(i), f)
}

pub fn index_of_with<T>(
    items: &[T],
    item: ItemKey,
    hint: Option<usize>,
    f: impl Fn(&T) -> ItemKey,
) -> Option<usize> {
    if let Some(h) = hint
        && let Some(it) = items.get(h)
        && f(it) == item
    {
        return Some(h);
    }
    items.iter().position(|it| f(it) == item)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeySet {
    Only(Vec<ItemKey>),
    AllExcept(Vec<ItemKey>),
}

impl Default for KeySet {
    fn default() -> Self {
        KeySet::Only(Vec::new())
    }
}

impl KeySet {
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

    pub fn contains(&self, k: ItemKey) -> bool {
        self.contains_counting(k).0
    }

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

    pub fn insert(&mut self, k: ItemKey) {
        match self {
            KeySet::Only(v) => Self::add(v, k),
            KeySet::AllExcept(v) => Self::drop(v, k),
        }
    }

    pub fn remove(&mut self, k: ItemKey) {
        match self {
            KeySet::Only(v) => Self::drop(v, k),
            KeySet::AllExcept(v) => Self::add(v, k),
        }
    }

    pub fn toggle(&mut self, k: ItemKey) {
        if self.contains(k) {
            self.remove(k);
        } else {
            self.insert(k);
        }
    }

    pub fn all(&mut self) {
        *self = KeySet::AllExcept(Vec::new());
    }

    pub fn none(&mut self) {
        *self = KeySet::Only(Vec::new());
    }

    pub fn len_in(&self, total: usize) -> usize {
        match self {
            KeySet::Only(v) => v.len().min(total),
            KeySet::AllExcept(v) => total.saturating_sub(v.len()),
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, KeySet::Only(v) if v.is_empty())
    }

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

    pub fn keys(&self) -> &[ItemKey] {
        self.list()
    }
}
