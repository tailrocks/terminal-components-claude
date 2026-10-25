use crate::id::ItemKey;
use crate::scroll::ScrollState;

use super::key::KeySet;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StepDir {
    Next,
    Prev,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Reconciliation {
    Unchanged,
    CursorMoved(ItemKey),
    CursorLost,
    SelectionDropped(usize),
}

pub trait Reconcile {
    fn reconcile(&mut self, len: usize, key: impl Fn(usize) -> ItemKey) -> Reconciliation;
    fn invalidate(&mut self);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Stamp {
    len: usize,
    first: Option<ItemKey>,
    last: Option<ItemKey>,
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct CollectionCore {
    cursor: Option<ItemKey>,
    cursor_index: usize,
    checked: KeySet,
    scroll: ScrollState,
    stamp: Option<Stamp>,
}

impl CollectionCore {
    pub const fn new() -> Self {
        CollectionCore {
            cursor: None,
            cursor_index: 0,
            checked: KeySet::new(),
            scroll: ScrollState::new(0),
            stamp: None,
        }
    }

    pub const fn cursor(&self) -> Option<ItemKey> {
        self.cursor
    }

    pub const fn cursor_index(&self) -> usize {
        self.cursor_index
    }

    pub fn set_cursor(&mut self, index: usize, key: ItemKey) {
        self.cursor = Some(key);
        self.cursor_index = index;
        self.scroll.ensure_visible_on_next_layout(index);
    }

    pub fn clear_cursor(&mut self) {
        self.cursor = None;
        self.cursor_index = 0;
    }

    pub const fn checked(&self) -> &KeySet {
        &self.checked
    }

    pub const fn checked_mut(&mut self) -> &mut KeySet {
        &mut self.checked
    }

    pub const fn scroll(&self) -> &ScrollState {
        &self.scroll
    }

    pub const fn scroll_mut(&mut self) -> &mut ScrollState {
        &mut self.scroll
    }

    pub fn first_enabled(
        len: usize,
        key: impl Fn(usize) -> ItemKey,
        enabled: impl Fn(usize) -> bool,
    ) -> Option<(usize, ItemKey)> {
        (0..len).find(|&i| enabled(i)).map(|i| (i, key(i)))
    }

    pub fn last_enabled(
        len: usize,
        key: impl Fn(usize) -> ItemKey,
        enabled: impl Fn(usize) -> bool,
    ) -> Option<(usize, ItemKey)> {
        (0..len).rev().find(|&i| enabled(i)).map(|i| (i, key(i)))
    }

    pub fn seek(
        len: usize,
        from: usize,
        dir: StepDir,
        key: impl Fn(usize) -> ItemKey,
        enabled: impl Fn(usize) -> bool,
    ) -> Option<(usize, ItemKey)> {
        if len == 0 {
            return None;
        }
        match dir {
            StepDir::Next => (from.min(len)..len)
                .find(|&i| enabled(i))
                .map(|i| (i, key(i))),
            StepDir::Prev => (0..=from.min(len.saturating_sub(1)))
                .rev()
                .find(|&i| enabled(i))
                .map(|i| (i, key(i))),
        }
    }

    pub fn step(
        len: usize,
        from: usize,
        dir: StepDir,
        key: impl Fn(usize) -> ItemKey,
        enabled: impl Fn(usize) -> bool,
    ) -> Option<(usize, ItemKey)> {
        if len == 0 {
            return None;
        }
        let from = from.min(len.saturating_sub(1));
        match dir {
            StepDir::Next => Self::seek(len, from.saturating_add(1), StepDir::Next, key, enabled),
            StepDir::Prev => from
                .checked_sub(1)
                .and_then(|f| Self::seek(len, f, StepDir::Prev, key, enabled)),
        }
    }

    pub fn step_wrapping(
        len: usize,
        from: usize,
        dir: StepDir,
        key: impl Fn(usize) -> ItemKey,
        enabled: impl Fn(usize) -> bool,
    ) -> Option<(usize, ItemKey)> {
        if len == 0 {
            return None;
        }
        let mut index = from.min(len.saturating_sub(1));
        for _ in 0..len {
            index = match dir {
                StepDir::Next => {
                    if index.saturating_add(1) >= len {
                        0
                    } else {
                        index.saturating_add(1)
                    }
                }
                StepDir::Prev => index.checked_sub(1).unwrap_or(len.saturating_sub(1)),
            };
            if enabled(index) {
                return Some((index, key(index)));
            }
        }
        None
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

        for i in start..len {
            if enabled(i) {
                return Some((i, key(i)));
            }
        }

        for i in (0..start).rev() {
            if enabled(i) {
                return Some((i, key(i)));
            }
        }
        None
    }

    pub fn reconcile_with(
        &mut self,
        len: usize,
        key: impl Fn(usize) -> ItemKey,
        enabled: impl Fn(usize) -> bool,
    ) -> Reconciliation {
        self.reconcile_with_extent(len, len, key, enabled)
    }

    pub fn reconcile_first_with(
        &mut self,
        len: usize,
        key: impl Fn(usize) -> ItemKey,
        enabled: impl Fn(usize) -> bool,
    ) -> Reconciliation {
        self.reconcile_inner(len, len, key, enabled, true)
    }

    pub fn reconcile_with_extent(
        &mut self,
        len: usize,
        extent: usize,
        key: impl Fn(usize) -> ItemKey,
        enabled: impl Fn(usize) -> bool,
    ) -> Reconciliation {
        self.reconcile_inner(len, extent, key, enabled, false)
    }

    fn reconcile_inner(
        &mut self,
        len: usize,
        extent: usize,
        key: impl Fn(usize) -> ItemKey,
        enabled: impl Fn(usize) -> bool,
        prefer_first: bool,
    ) -> Reconciliation {
        let stamp = Self::stamp_of(len, &key);
        if self.stamp == Some(stamp) {
            return Reconciliation::Unchanged;
        }
        self.stamp = Some(stamp);
        self.scroll.set_content(extent);

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
                        let fallback = if prefer_first {
                            (0..len).find(|&i| enabled(i)).map(|i| (i, key(i)))
                        } else {
                            self.nearest(len, &key, &enabled)
                        };
                        if let Some((i, k)) = fallback {
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
