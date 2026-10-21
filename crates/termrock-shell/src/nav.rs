//! Page routing: the navigation stack, tabs and page identity.
//!
//! Ported from PR #26 `src/shell/nav.rs` (commit `f4867e9b5`). The parity
//! adaptation returns [`Option`] where the source asserted internal
//! invariants, and the breadcrumb is the shell-owned [`Crumb`] model because
//! the parity breadcrumb widget is owned by another crate wave.

use std::hash;

use termrock_core::id::{Id, ItemKey};

/// Opaque identity of the page currently shown. Focus upkeep compares keys to
/// detect a page change; two visits to the same route get different keys so a
/// re-entered page re-focuses its primary control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PageKey(pub u64);

impl PageKey {
    /// The key of a router before its first entry.
    pub const ROOT: Self = Self(0);

    /// Derive a key from any hashable value.
    pub fn of(value: impl hash::Hash) -> Self {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        hash::Hash::hash(&value, &mut h);
        Self(hash::Hasher::finish(&h))
    }
}

/// A navigation request the app resolves through its quit guard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Nav<R> {
    /// Show `R` on top of the active stack.
    Push(R),
    /// Swap the top of the active stack for `R`.
    Replace(R),
    /// Discard the active stack and show only `R`.
    Reset(R),
    /// Pop the top of the active stack.
    Back,
    /// Pop the active stack down to `depth` entries.
    PopTo(usize),
    /// Switch to tab `i`, keeping its stack.
    Tab(usize),
}

#[derive(Debug)]
struct Entry<P> {
    key: u64,
    page: P,
}

/// Per-tab navigation stacks with one active tab.
///
/// Each entry keeps a monotonically increasing key so focus upkeep can tell a
/// new page from a revisit.
#[derive(Debug)]
pub struct Router<P> {
    tabs: Vec<Vec<Entry<P>>>,
    active: usize,
    next_key: u64,
}

impl<P> Router<P> {
    /// A router with one tab whose stack starts at `root`.
    pub fn new(root: P) -> Self {
        Self::with_tabs(vec![root])
    }

    /// A router with one tab per root. At least one root is required.
    ///
    /// # Panics
    /// Panics when `roots` is empty.
    pub fn with_tabs(roots: Vec<P>) -> Self {
        assert!(!roots.is_empty(), "a router needs at least one tab");
        let mut r = Self {
            tabs: Vec::new(),
            active: 0,
            next_key: 0,
        };
        for p in roots {
            let e = r.entry(p);
            r.tabs.push(vec![e]);
        }
        r
    }

    fn entry(&mut self, page: P) -> Entry<P> {
        self.next_key = self.next_key.wrapping_add(1);
        Entry {
            key: self.next_key,
            page,
        }
    }

    fn active_stack(&self) -> &[Entry<P>] {
        self.tabs
            .get(self.active)
            .map_or(&[][..], std::vec::Vec::as_slice)
    }

    /// The active stack, mutably. `None` when the active tab is gone.
    ///
    /// A router always keeps its active tab, so the live case is the only
    /// one the app sees; the accessor stays total so a broken invariant
    /// degrades to "no active stack" instead of panicking.
    fn active_stack_mut(&mut self) -> Option<&mut Vec<Entry<P>>> {
        let active = self.active;
        self.tabs.get_mut(active)
    }

    /// The page on top of the active stack.
    pub fn current(&self) -> Option<&P> {
        self.active_stack().last().map(|e| &e.page)
    }

    /// The page on top of the active stack, mutably.
    pub fn current_mut(&mut self) -> Option<&mut P> {
        self.active_stack_mut()
            .and_then(|stack| stack.last_mut())
            .map(|e| &mut e.page)
    }

    /// The key of the top entry of the active stack.
    pub fn key(&self) -> PageKey {
        PageKey(self.active_stack().last().map_or(0, |e| e.key))
    }

    /// The active stack, root first.
    pub fn stack(&self) -> impl DoubleEndedIterator<Item = &P> + ExactSizeIterator {
        self.active_stack().iter().map(|e| &e.page)
    }

    /// The active stack, mutably, root first.
    pub fn stack_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut P> + ExactSizeIterator {
        let stack: &mut [Entry<P>] = match self.active_stack_mut() {
            Some(stack) => stack.as_mut_slice(),
            None => &mut [],
        };
        stack.iter_mut().map(|e| &mut e.page)
    }

    /// How deep the active stack is.
    pub fn depth(&self) -> usize {
        self.active_stack().len()
    }

    /// Tab `i`'s stack, root first.
    pub fn tab(&self, i: usize) -> impl DoubleEndedIterator<Item = &P> + ExactSizeIterator {
        self.tabs
            .get(i)
            .map_or(&[][..], std::vec::Vec::as_slice)
            .iter()
            .map(|e| &e.page)
    }

    /// The top page of tab `i`.
    pub fn top_of(&self, i: usize) -> Option<&P> {
        self.tabs.get(i)?.last().map(|e| &e.page)
    }

    /// The top page of tab `i`, mutably.
    pub fn top_of_mut(&mut self, i: usize) -> Option<&mut P> {
        self.tabs.get_mut(i)?.last_mut().map(|e| &mut e.page)
    }

    /// The active tab index.
    pub fn active(&self) -> usize {
        self.active
    }

    /// How many tabs exist.
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Every page in every tab.
    pub fn pages(&self) -> impl Iterator<Item = &P> {
        self.tabs.iter().flatten().map(|e| &e.page)
    }

    /// Every page in every tab, mutably.
    pub fn pages_mut(&mut self) -> impl Iterator<Item = &mut P> {
        self.tabs.iter_mut().flatten().map(|e| &mut e.page)
    }

    /// Push `page` onto the active stack.
    pub fn push(&mut self, page: P) {
        let e = self.entry(page);
        if let Some(stack) = self.active_stack_mut() {
            stack.push(e);
        }
    }

    /// Pop the top of the active stack. The root stays.
    pub fn back(&mut self) -> Option<P> {
        let stack = self.active_stack_mut()?;
        if stack.len() < 2 {
            return None;
        }
        stack.pop().map(|e| e.page)
    }

    /// Pop the active stack down to `depth + 1` entries and return what left.
    pub fn pop_to(&mut self, depth: usize) -> Vec<P> {
        let Some(stack) = self.active_stack_mut() else {
            return Vec::new();
        };
        if depth.saturating_add(1) >= stack.len() {
            return Vec::new();
        }
        stack.drain(depth + 1..).map(|e| e.page).collect()
    }

    /// Swap the top of the active stack for `page` and return the old top.
    pub fn replace(&mut self, page: P) -> Option<P> {
        let e = self.entry(page);
        self.active_stack_mut()
            .and_then(|stack| stack.last_mut())
            .map(|top| core::mem::replace(top, e).page)
    }

    /// Discard the active stack, show only `page`, and return what left.
    pub fn reset(&mut self, page: P) -> Vec<P> {
        let e = self.entry(page);
        let active = self.active;
        let old = self
            .tabs
            .get_mut(active)
            .map(|stack| core::mem::replace(stack, vec![e]))
            .unwrap_or_default();
        old.into_iter().map(|e| e.page).collect()
    }

    /// Show tab `i`. `false` when `i` is out of range or already shown.
    pub fn select(&mut self, i: usize) -> bool {
        if i >= self.tabs.len() || i == self.active {
            return false;
        }
        self.active = i;
        true
    }

    /// Open a new tab rooted at `root` and show it, returning its index.
    pub fn open_tab(&mut self, root: P) -> usize {
        let e = self.entry(root);
        self.tabs.push(vec![e]);
        self.active = self.tabs.len().saturating_sub(1);
        self.active
    }

    /// Close tab `i` and return its pages. The last tab stays.
    pub fn close_tab(&mut self, i: usize) -> Option<Vec<P>> {
        if self.tabs.len() < 2 || i >= self.tabs.len() {
            return None;
        }
        let pages = self.tabs.remove(i).into_iter().map(|e| e.page).collect();
        if self.active > i || (self.active == i && i > 0) {
            self.active = self.active.saturating_sub(1);
        }
        Some(pages)
    }

    /// Does this move drop any page (and so owe its stakes a guard)?
    pub fn drops<R>(&self, nav: &Nav<R>) -> bool {
        match nav {
            Nav::Push(_) | Nav::Tab(_) => false,
            Nav::Replace(_) | Nav::Reset(_) => true,
            Nav::Back => self.depth() > 1,
            Nav::PopTo(d) => d.saturating_add(1) < self.depth(),
        }
    }

    /// The pages a move would drop, top first.
    pub fn dropped_by<R>(&self, nav: &Nav<R>) -> impl Iterator<Item = &P> {
        let stack = self.active_stack();
        let n = stack.len();
        let from = match nav {
            Nav::Push(_) | Nav::Tab(_) => n,
            Nav::Replace(_) => n.saturating_sub(1),
            Nav::Back if n < 2 => n,
            Nav::Back => n - 1,
            Nav::Reset(_) => 0,
            Nav::PopTo(d) => (*d).saturating_add(1).min(n),
        };
        stack[from..].iter().rev().map(|e| &e.page)
    }

    /// Apply a resolved move, building pages from their routes, and return
    /// what the move dropped, top first.
    pub fn apply<R>(&mut self, nav: Nav<R>, mut build: impl FnMut(R) -> P) -> Option<Vec<P>> {
        match nav {
            Nav::Push(r) => {
                self.push(build(r));
                Some(Vec::new())
            }
            Nav::Replace(r) => self.replace(build(r)).map(|p| vec![p]),
            Nav::Reset(r) => Some(self.reset(build(r))),
            Nav::Back => self.back().map(|p| vec![p]),
            Nav::PopTo(d) => {
                let dropped = self.pop_to(d);
                (!dropped.is_empty()).then_some(dropped)
            }
            Nav::Tab(i) => self.select(i).then(Vec::new),
        }
    }

    /// The active stack as a breadcrumb model, one [`Crumb`] segment per page.
    pub fn crumb(&self, id: Id, label: impl Fn(&P) -> String) -> Crumb {
        Crumb::new(id, self.stack().map(label).collect())
    }

    /// A click on breadcrumb segment `hit` maps to the pop that shows it.
    /// `levels` extra segments the shell renders above this router's stack
    /// shift the mapping the same way they do for [`Crumb::on_click`].
    pub fn crumb_click<R>(&self, crumb: &Crumb, levels: usize, hit: Id) -> Option<Nav<R>> {
        let level = crumb.on_click(hit)?;
        let depth = level.checked_sub(levels)?;
        (depth.saturating_add(1) < self.depth()).then_some(Nav::PopTo(depth))
    }
}

/// The shell-owned breadcrumb model: the segments of one location trail and
/// which of them is a link back to its level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Crumb {
    /// The widget id the trail renders under; each segment is a child id.
    pub id: Id,
    /// The segment labels, root first.
    pub segments: Vec<String>,
}

impl Crumb {
    /// A trail with one segment per page.
    pub fn new(id: Id, segments: Vec<String>) -> Self {
        Self { id, segments }
    }

    /// The id of segment `i`, as the renderer should publish it.
    pub fn segment_id(&self, i: usize) -> Id {
        self.id.index(i)
    }

    /// Which level a click on `hit` names, counting segments from 0.
    pub fn on_click(&self, hit: Id) -> Option<usize> {
        self.segments
            .iter()
            .enumerate()
            .find_map(|(i, _)| (self.segment_id(i) == hit).then_some(i))
    }

    /// The ids of the segments that link back (every segment but the last).
    pub fn links(&self) -> impl Iterator<Item = (usize, Id)> + '_ {
        let last = self.segments.len().saturating_sub(1);
        (0..last).map(|i| (i, self.segment_id(i)))
    }
}

impl From<PageKey> for ItemKey {
    fn from(key: PageKey) -> Self {
        ItemKey::index(key.0 as usize)
    }
}

#[cfg(test)]
mod tests;
