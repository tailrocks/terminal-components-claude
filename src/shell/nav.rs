//! Navigation (plan §1.8, `Router` and `Nav`): one page-stack model for
//! every app, so no app keeps its own `goto`, route enum switch or tab
//! stacks.
//!
//! - [`Router`] holds the pages of one or more tabs, each a stack whose
//!   last entry is shown (the macOS Finder's back stack, a navigation
//!   controller). An app with one page per route keeps one tab with a
//!   one-page stack and [`Nav::Reset`]s it.
//! - [`Nav`] is a navigation request a page or a menu makes; the router
//!   applies it with [`Router::apply`], building the page of a new route
//!   through the app's own constructor.
//! - Every move that drops a page asks first: [`Router::request`] runs
//!   the [`LeaveGuard`] with the stakes of the pages it would drop (K4,
//!   K6), so a route change never throws edits away without a question.
//! - The trail of the shown tab is a [`Breadcrumb`] ([`Router::breadcrumb`]):
//!   a click on an earlier level is [`Nav::PopTo`] that level (M5).

use crate::core::id::WidgetId;
use crate::theme::Strings;
use crate::ui::leave::{Leave, LeaveGuard, Stakes};
use crate::widgets::breadcrumb::Breadcrumb;
use crate::widgets::dialog::Dialog;

use super::PageKey;

/// A navigation request. `R` is the app's route: what it takes to build a
/// page (a page id, a path, a plan to review).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Nav<R> {
    /// Show `R` on top of the shown page; Back returns to it (a link
    /// into a detail, "Open", "Review…").
    Push(R),
    /// Show `R` in place of the shown page (the shown page is dropped).
    Replace(R),
    /// Start the shown tab over at `R` (a sidebar entry, a menu jump):
    /// every page of the tab is dropped.
    Reset(R),
    /// Back one page. At the root of a tab there is nowhere to go.
    Back,
    /// Back to the page at `depth` (0 is the tab's root): a click on a
    /// breadcrumb level.
    PopTo(usize),
    /// Show tab `i`. Nothing is dropped.
    Tab(usize),
}

/// One page on a tab's stack: the page and the key focus upkeep uses
/// ([`PageKey`]), unique for the router's lifetime.
#[derive(Debug)]
struct Entry<P> {
    key: u64,
    page: P,
}

/// The pages of an app: one stack per tab, the active tab's last page
/// shown. `P` is what the app keeps per page (a boxed page, a route id).
#[derive(Debug)]
pub struct Router<P> {
    tabs: Vec<Vec<Entry<P>>>,
    active: usize,
    next_key: u64,
}

impl<P> Router<P> {
    /// One tab whose root is `root`.
    pub fn new(root: P) -> Self {
        Self::with_tabs(vec![root])
    }

    /// One tab per root, the first active. `roots` must not be empty.
    pub fn with_tabs(roots: Vec<P>) -> Self {
        assert!(!roots.is_empty(), "a router needs at least one tab");
        let mut r = Self {
            tabs: vec![],
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
        self.next_key += 1;
        Entry {
            key: self.next_key,
            page,
        }
    }

    /// The shown page.
    pub fn current(&self) -> &P {
        &self.tabs[self.active]
            .last()
            .expect("a tab is never empty")
            .page
    }

    /// The shown page, mutably.
    pub fn current_mut(&mut self) -> &mut P {
        &mut self.tabs[self.active]
            .last_mut()
            .expect("a tab is never empty")
            .page
    }

    /// The key of the shown page, for focus upkeep: it changes whenever
    /// another page is shown, and only then.
    pub fn key(&self) -> PageKey {
        PageKey(self.tabs[self.active].last().map_or(0, |e| e.key))
    }

    /// The pages of the shown tab, root first.
    pub fn stack(&self) -> impl DoubleEndedIterator<Item = &P> + ExactSizeIterator {
        self.tabs[self.active].iter().map(|e| &e.page)
    }

    /// The pages of the shown tab, mutably.
    pub fn stack_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut P> + ExactSizeIterator {
        self.tabs[self.active].iter_mut().map(|e| &mut e.page)
    }

    /// How many pages the shown tab holds (1 at its root).
    pub fn depth(&self) -> usize {
        self.tabs[self.active].len()
    }

    /// The pages of tab `i`, root first.
    pub fn tab(&self, i: usize) -> impl DoubleEndedIterator<Item = &P> + ExactSizeIterator {
        self.tabs[i].iter().map(|e| &e.page)
    }

    /// The shown page of tab `i`.
    pub fn top_of(&self, i: usize) -> Option<&P> {
        self.tabs.get(i)?.last().map(|e| &e.page)
    }

    /// The shown page of tab `i`, mutably.
    pub fn top_of_mut(&mut self, i: usize) -> Option<&mut P> {
        self.tabs.get_mut(i)?.last_mut().map(|e| &mut e.page)
    }

    /// The active tab.
    pub fn active(&self) -> usize {
        self.active
    }

    /// How many tabs there are.
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Every page of every tab (the blur fan-out, the quit stakes).
    pub fn pages(&self) -> impl Iterator<Item = &P> {
        self.tabs.iter().flatten().map(|e| &e.page)
    }

    /// Every page of every tab, mutably.
    pub fn pages_mut(&mut self) -> impl Iterator<Item = &mut P> {
        self.tabs.iter_mut().flatten().map(|e| &mut e.page)
    }

    /// Show `page` on top of the shown one.
    pub fn push(&mut self, page: P) {
        let e = self.entry(page);
        self.tabs[self.active].push(e);
    }

    /// Back one page: the dropped page, or `None` at the root.
    pub fn back(&mut self) -> Option<P> {
        let stack = &mut self.tabs[self.active];
        if stack.len() < 2 {
            return None;
        }
        stack.pop().map(|e| e.page)
    }

    /// Back to the page at `depth` (0 the root): the dropped pages, top
    /// last. Nothing happens at or past the shown page.
    pub fn pop_to(&mut self, depth: usize) -> Vec<P> {
        let stack = &mut self.tabs[self.active];
        if depth + 1 >= stack.len() {
            return vec![];
        }
        stack.drain(depth + 1..).map(|e| e.page).collect()
    }

    /// Show `page` in place of the shown one: the dropped page.
    pub fn replace(&mut self, page: P) -> P {
        let e = self.entry(page);
        let stack = &mut self.tabs[self.active];
        std::mem::replace(stack.last_mut().expect("a tab is never empty"), e).page
    }

    /// Start the shown tab over at `page`: every dropped page, root first.
    pub fn reset(&mut self, page: P) -> Vec<P> {
        let e = self.entry(page);
        std::mem::replace(&mut self.tabs[self.active], vec![e])
            .into_iter()
            .map(|e| e.page)
            .collect()
    }

    /// Show tab `i`: `false` when there is no such tab or it is shown.
    pub fn select(&mut self, i: usize) -> bool {
        if i >= self.tabs.len() || i == self.active {
            return false;
        }
        self.active = i;
        true
    }

    /// Open a tab whose root is `root` after the last one and show it:
    /// its index.
    pub fn open_tab(&mut self, root: P) -> usize {
        let e = self.entry(root);
        self.tabs.push(vec![e]);
        self.active = self.tabs.len() - 1;
        self.active
    }

    /// Close tab `i`: its pages, root first, or `None` when it is the
    /// only tab or there is no such tab. The shown tab stays shown; when
    /// it was the one closed, the tab before it is shown (the first when
    /// it was the first).
    pub fn close_tab(&mut self, i: usize) -> Option<Vec<P>> {
        if self.tabs.len() < 2 || i >= self.tabs.len() {
            return None;
        }
        let pages = self.tabs.remove(i).into_iter().map(|e| e.page).collect();
        if self.active > i || (self.active == i && i > 0) {
            self.active -= 1;
        }
        Some(pages)
    }

    /// Whether `nav` would drop the shown page (its stakes are asked for
    /// first): Replace and Reset do, Back and PopTo do above the root.
    /// Push and Tab keep every page.
    pub fn drops<R>(&self, nav: &Nav<R>) -> bool {
        match nav {
            Nav::Push(_) | Nav::Tab(_) => false,
            Nav::Replace(_) | Nav::Reset(_) => true,
            Nav::Back => self.depth() > 1,
            Nav::PopTo(d) => d + 1 < self.depth(),
        }
    }

    /// The pages `nav` would drop, top first (for their stakes).
    pub fn dropped_by<R>(&self, nav: &Nav<R>) -> impl Iterator<Item = &P> {
        let stack = &self.tabs[self.active];
        let n = stack.len();
        let from = match nav {
            Nav::Push(_) | Nav::Tab(_) => n,
            Nav::Replace(_) => n - 1,
            // Back at the root drops nothing
            Nav::Back if n < 2 => n,
            Nav::Back => n - 1,
            Nav::Reset(_) => 0,
            Nav::PopTo(d) => (*d + 1).min(n),
        };
        stack[from..].iter().rev().map(|e| &e.page)
    }

    /// Ask to move: `Ok(nav)` to apply now (nothing would be lost), or
    /// `Err(dialog)` to show first, with `nav` kept on `guard` for its
    /// answer ([`LeaveGuard::resolve`]). `stakes` names what the dropped
    /// pages ([`Self::dropped_by`]) would lose; it is not asked for a
    /// move that drops nothing.
    pub fn request<R: Clone>(
        &self,
        guard: &mut LeaveGuard<Nav<R>>,
        nav: Nav<R>,
        stakes: impl FnOnce() -> Stakes,
        strings: &Strings,
    ) -> Result<Nav<R>, Box<Dialog>> {
        if !self.drops(&nav) {
            return Ok(nav);
        }
        match guard.request(Leave::To(nav), &stakes(), strings)? {
            Leave::To(nav) => Ok(nav),
            Leave::Quit => unreachable!("a route request resolves to its route"),
        }
    }

    /// Apply `nav`, building the page of a new route with `build`: the
    /// dropped pages (top last), or `None` when nothing moved (Back at
    /// the root, a PopTo at or past the shown page, an unknown tab).
    pub fn apply<R>(&mut self, nav: Nav<R>, build: impl FnOnce(R) -> P) -> Option<Vec<P>> {
        match nav {
            Nav::Push(r) => {
                self.push(build(r));
                Some(vec![])
            }
            Nav::Replace(r) => Some(vec![self.replace(build(r))]),
            Nav::Reset(r) => Some(self.reset(build(r))),
            Nav::Back => self.back().map(|p| vec![p]),
            Nav::PopTo(d) => {
                let dropped = self.pop_to(d);
                (!dropped.is_empty()).then_some(dropped)
            }
            Nav::Tab(i) => self.select(i).then(Vec::new),
        }
    }

    /// The shown tab's trail as a [`Breadcrumb`] `id`, one level per
    /// page (`label` names it); every earlier level is a link
    /// ([`Self::crumb_click`]).
    pub fn breadcrumb(&self, id: WidgetId, label: impl Fn(&P) -> String) -> Breadcrumb {
        Breadcrumb::new(id, self.stack().map(label).collect())
    }

    /// A click on `hit` of `crumb` (drawn from [`Self::breadcrumb`] with
    /// `levels` levels before the stack, such as an app's group label):
    /// the navigation it asks for.
    pub fn crumb_click<R>(
        &self,
        crumb: &Breadcrumb,
        levels: usize,
        hit: WidgetId,
    ) -> Option<Nav<R>> {
        let level = crumb.on_click(hit)?;
        let depth = level.checked_sub(levels)?;
        (depth + 1 < self.depth()).then_some(Nav::PopTo(depth))
    }
}

#[cfg(test)]
#[path = "nav_tests.rs"]
mod tests;
