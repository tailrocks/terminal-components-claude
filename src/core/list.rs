//! The one cursor model for every row and item collection: lists, trees,
//! tables, grids, menus, selects, pickers, tab strips, chips and radio
//! groups.
//!
//! - [`RowSource`] is what a collection tells the cursor: how many rows,
//!   which are enabled, their labels (for type-ahead) and their keys.
//! - [`Cursor`] is the position plus its rules (wrap, axis, disabled-row
//!   skipping). [`Cursor::apply`] is the only place that steps, pages,
//!   jumps to the ends, skips disabled rows and wraps;
//!   [`Cursor::type_ahead`] is the only place that type-selects;
//!   [`Cursor::select`] the only place a pointer or context menu places it.
//! - [`NavProfile`] is the app-wide layer of the override stack (plan §3):
//!   `Desktop` (the default: arrows, paging and Home/End; letters type
//!   ahead) or `Vim` (`h j k l g G` navigate; no type-ahead). A widget's
//!   own `nav(..)` and `type_ahead(..)` builders override the profile, and
//!   its key map overrides both.
//!
//! Standards: WAI-ARIA APG listbox, tree and grid keyboard interaction
//! (arrows step, Home/End jump to the first and last *enabled* item,
//! PageUp/PageDown move by a viewport, "type-ahead is recommended for all
//! listboxes"); macOS lists, outline views, popup buttons and menus (type
//! select, disabled items skipped, no wrap except in menus).

use std::cell::Cell;

use crate::core::event::{Key, Outcome};
use crate::core::hit::Axis;
use crate::core::id::ItemKey;
use crate::core::keymap::{Nav, NavKeys};
use crate::core::typeahead::TypeAhead;

/// What a collection tells its [`Cursor`].
pub trait RowSource {
    /// How many rows (items, tabs, options) there are.
    fn len(&self) -> usize;

    /// True when there are no rows.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Whether row `i` can take the cursor. Disabled rows, notes and
    /// separators are skipped by keyboard navigation and type-ahead.
    fn enabled(&self, i: usize) -> bool {
        let _ = i;
        true
    }

    /// The text type-ahead matches for row `i`.
    fn label(&self, i: usize) -> &str;

    /// A stable key for row `i` (the bridge to `main`'s `ItemKey` cursors).
    fn key(&self, i: usize) -> ItemKey {
        ItemKey::index(i)
    }
}

impl RowSource for [&str] {
    fn len(&self) -> usize {
        <[&str]>::len(self)
    }
    fn label(&self, i: usize) -> &str {
        self[i]
    }
}

impl RowSource for [String] {
    fn len(&self) -> usize {
        <[String]>::len(self)
    }
    fn label(&self, i: usize) -> &str {
        &self[i]
    }
}

/// A [`RowSource`] made of closures, for collections whose rows are not one
/// slice (a filtered view, a column of a table).
pub struct RowFn<'a> {
    /// Number of rows.
    pub len: usize,
    /// Whether row `i` is enabled.
    pub enabled: &'a dyn Fn(usize) -> bool,
    /// Row `i`'s type-ahead label.
    pub label: &'a dyn Fn(usize) -> &'a str,
}

impl RowSource for RowFn<'_> {
    fn len(&self) -> usize {
        self.len
    }
    fn enabled(&self, i: usize) -> bool {
        (self.enabled)(i)
    }
    fn label(&self, i: usize) -> &str {
        (self.label)(i)
    }
}

/// What a cursor operation did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Moved {
    /// The key is not navigation for this cursor (a ←/→ on a vertical
    /// list, an empty collection): let it bubble.
    Ignored,
    /// Handled, but the cursor stays at this index (an end of a list that
    /// does not wrap, no enabled row that way, a type-ahead miss).
    Stayed(usize),
    /// The cursor moved to this index.
    To(usize),
}

impl Moved {
    /// The cursor's index after the operation, `None` when ignored.
    pub fn index(self) -> Option<usize> {
        match self {
            Moved::Ignored => None,
            Moved::Stayed(i) | Moved::To(i) => Some(i),
        }
    }

    /// True when the cursor changed rows.
    pub fn moved(self) -> bool {
        matches!(self, Moved::To(_))
    }

    /// `Ignored`, `Consumed` (stayed) or `Changed` (moved).
    pub fn outcome(self) -> Outcome {
        match self {
            Moved::Ignored => Outcome::Ignored,
            Moved::Stayed(_) => Outcome::Consumed,
            Moved::To(_) => Outcome::Changed,
        }
    }
}

/// A cursor over a [`RowSource`]: its index and its rules.
///
/// Widgets keep their `cursor` index field (and their `wrap` knob); they
/// build a `Cursor` from them for each key and write the index back (until
/// `ListCore` holds the cursor, plan S13).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    /// The row under the cursor.
    pub index: usize,
    /// Past either end lands on the other end (menus). Default off, as in
    /// macOS lists, outline views and tab strips.
    pub wrap: bool,
    /// Which arrows step: ↑/↓ for [`Axis::Vertical`] (lists, trees,
    /// menus), ←/→ for [`Axis::Horizontal`] (tab strips, chips, segmented
    /// controls, button rows). Default vertical.
    pub axis: Axis,
    /// Keyboard steps pass over rows [`RowSource::enabled`] rejects.
    /// Default on (macOS, APG). Type-ahead always skips them.
    pub skip_disabled: bool,
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Cursor {
    /// A vertical cursor at `index` that does not wrap and skips disabled
    /// rows.
    pub const fn new(index: usize) -> Self {
        Self {
            index,
            wrap: false,
            axis: Axis::Vertical,
            skip_disabled: true,
        }
    }

    /// See [`Self::wrap`].
    pub const fn wrap(mut self, on: bool) -> Self {
        self.wrap = on;
        self
    }

    /// See [`Self::axis`].
    pub const fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    /// See [`Self::skip_disabled`].
    pub const fn skip_disabled(mut self, on: bool) -> Self {
        self.skip_disabled = on;
        self
    }

    fn usable(&self, src: &dyn RowSource, i: usize) -> bool {
        !self.skip_disabled || src.enabled(i)
    }

    fn settle(&mut self, from: usize, to: usize) -> Moved {
        self.index = to;
        if to == from {
            Moved::Stayed(to)
        } else {
            Moved::To(to)
        }
    }

    /// Apply a navigation step. `page` is the number of rows a viewport
    /// shows (PageUp/PageDown move by it; at least 1).
    ///
    /// | `Nav` | vertical | horizontal |
    /// |---|---|---|
    /// | ↑ ↓ | previous / next enabled row | ignored |
    /// | ← → | ignored | previous / next enabled item |
    /// | PageUp PageDown | `page` rows, then the nearest enabled row (never wraps) | ignored |
    /// | Home End | first / last enabled row | first / last enabled item |
    ///
    /// An empty source ignores every step.
    pub fn apply(&mut self, nav: Nav, src: &dyn RowSource, page: usize) -> Moved {
        let n = src.len();
        if n == 0 {
            return Moved::Ignored;
        }
        let vertical = self.axis == Axis::Vertical;
        match nav {
            Nav::Up | Nav::Down if vertical => self.step(if nav == Nav::Up { -1 } else { 1 }, src),
            Nav::Left | Nav::Right if !vertical => {
                self.step(if nav == Nav::Left { -1 } else { 1 }, src)
            }
            Nav::PageUp | Nav::PageDown if vertical => {
                let from = self.index.min(n - 1);
                let page = page.max(1);
                let to = if nav == Nav::PageUp {
                    from.saturating_sub(page)
                } else {
                    (from + page).min(n - 1)
                };
                let to = self.nearest(src, to, from);
                self.settle(from, to)
            }
            Nav::Home => self.first(src),
            Nav::End => self.last(src),
            _ => Moved::Ignored,
        }
    }

    /// Step `delta` rows (±1 for the arrows), passing over disabled rows
    /// and wrapping when [`Self::wrap`] is on.
    pub fn step(&mut self, delta: isize, src: &dyn RowSource) -> Moved {
        let n = src.len();
        if n == 0 {
            return Moved::Ignored;
        }
        let from = self.index.min(n - 1);
        let mut i = from;
        for _ in 0..n {
            let next = crate::core::nav::step_index(i, n, delta, self.wrap);
            if next == i {
                // an end of a list that does not wrap
                break;
            }
            i = next;
            if self.usable(src, i) {
                return self.settle(from, i);
            }
        }
        self.settle(from, from)
    }

    /// Jump to the first enabled row.
    pub fn first(&mut self, src: &dyn RowSource) -> Moved {
        let n = src.len();
        if n == 0 {
            return Moved::Ignored;
        }
        let from = self.index.min(n - 1);
        let to = (0..n).find(|&i| self.usable(src, i)).unwrap_or(from);
        self.settle(from, to)
    }

    /// Jump to the last enabled row.
    pub fn last(&mut self, src: &dyn RowSource) -> Moved {
        let n = src.len();
        if n == 0 {
            return Moved::Ignored;
        }
        let from = self.index.min(n - 1);
        let to = (0..n).rev().find(|&i| self.usable(src, i)).unwrap_or(from);
        self.settle(from, to)
    }

    /// The usable row nearest `to`, searching away from `from` first (the
    /// direction of travel), then back toward it; `from` when none is.
    fn nearest(&self, src: &dyn RowSource, to: usize, from: usize) -> usize {
        let n = src.len();
        if self.usable(src, to) {
            return to;
        }
        let ahead: Box<dyn Iterator<Item = usize>> = if to >= from {
            Box::new(to..n)
        } else {
            Box::new((0..=to).rev())
        };
        let behind: Box<dyn Iterator<Item = usize>> = if to >= from {
            Box::new((0..to).rev())
        } else {
            Box::new(to + 1..n)
        };
        ahead
            .chain(behind)
            .find(|&i| self.usable(src, i))
            .unwrap_or(from)
    }

    /// Place the cursor on row `i` (a click, a context menu, a programmatic
    /// move). Out of range is ignored; the row need not be enabled (a
    /// pointer may point at a disabled row).
    pub fn select(&mut self, i: usize, src: &dyn RowSource) -> Moved {
        if i >= src.len() {
            return Moved::Ignored;
        }
        let from = self.index;
        self.settle(from, i)
    }

    /// Type-select: add `c` to the prefix in `ta` and move to the first
    /// enabled row at or after the cursor (wrapping) whose label starts with
    /// it; a repeated single letter cycles through the rows starting with
    /// it. A miss is [`Moved::Stayed`]: the letter is consumed, never
    /// passed on half-typed (plan §3).
    pub fn type_ahead(&mut self, c: char, ta: &mut TypeAhead, src: &dyn RowSource) -> Moved {
        let n = src.len();
        if n == 0 {
            return Moved::Ignored;
        }
        let from = self.index.min(n - 1);
        let prefix = ta.feed(c).to_owned();
        let found = TypeAhead::find(&prefix, (0..n).map(|i| src.label(i)), from, |i| {
            src.enabled(i)
        });
        match found {
            Some(i) => self.settle(from, i),
            None => Moved::Stayed(from),
        }
    }
}

/// What a key means to a navigable collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyStep {
    /// A navigation step ([`NavKeys::action`]).
    Nav(Nav),
    /// A character to type-select with ([`TypeAhead::char_of`]).
    Type(char),
}

impl KeyStep {
    /// Classify `key`: with type-ahead on, a printable character types
    /// (vim letters included, so `j` finds "Jobs"); otherwise `nav`
    /// decides. `None` when the key is neither.
    pub fn of(key: &Key, nav: &NavKeys, type_ahead: bool) -> Option<Self> {
        if type_ahead && let Some(c) = TypeAhead::char_of(key) {
            return Some(KeyStep::Type(c));
        }
        nav.action(key).map(KeyStep::Nav)
    }
}

/// The app-wide navigation profile: the first layer of the override stack
/// (profile, then a widget's `nav(..)`/`type_ahead(..)` builders, then its
/// key map). Every navigable widget reads it when it is built, so install
/// it before building widgets (at app start).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum NavProfile {
    /// Desktop conventions (macOS, Windows, GNOME; APG): arrows, paging and
    /// Home/End navigate; printable keys type-select in lists, trees,
    /// selects and menus. Letters are never navigation, so they stay free
    /// for typing (WCAG 2.1.4). The default.
    #[default]
    Desktop,
    /// Vim keys: `h j k l` and `g`/`G` navigate everywhere, and type-ahead
    /// is off so the letters do not collide.
    Vim,
}

thread_local! {
    static PROFILE: Cell<NavProfile> = const { Cell::new(NavProfile::Desktop) };
}

impl NavProfile {
    /// The keys that navigate under this profile.
    pub const fn nav(self) -> NavKeys {
        match self {
            NavProfile::Desktop => NavKeys::DESKTOP,
            NavProfile::Vim => NavKeys::VIM,
        }
    }

    /// Whether lists, trees, selects and menus type-select by default.
    pub const fn type_ahead(self) -> bool {
        matches!(self, NavProfile::Desktop)
    }

    /// Make this the profile widgets built from now on (on this thread)
    /// take their defaults from.
    pub fn install(self) {
        PROFILE.with(|p| p.set(self));
    }

    /// The installed profile ([`NavProfile::Desktop`] unless an app
    /// installed another).
    pub fn current() -> Self {
        PROFILE.with(Cell::get)
    }
}

/// A navigable collection: a [`RowSource`] with a [`Cursor`]. Every list,
/// tree, table, grid, menu, select, picker, tab strip, chip bar and radio
/// group implements it, so one table-driven test covers them all.
pub trait Navigable: RowSource {
    /// The cursor as the collection holds it now.
    fn cursor(&self) -> Cursor;
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::{KeyCode, KeyModifiers};
    use std::time::Duration;

    struct Rows {
        labels: Vec<&'static str>,
        disabled: Vec<usize>,
    }

    impl RowSource for Rows {
        fn len(&self) -> usize {
            self.labels.len()
        }
        fn enabled(&self, i: usize) -> bool {
            !self.disabled.contains(&i)
        }
        fn label(&self, i: usize) -> &str {
            self.labels[i]
        }
    }

    fn rows(disabled: &[usize]) -> Rows {
        Rows {
            labels: vec!["Alpha", "Beta", "Bravo", "Charlie", "Delta", "Echo"],
            disabled: disabled.to_vec(),
        }
    }

    #[test]
    fn apply_steps_pages_and_jumps_over_enabled_rows() {
        let src = rows(&[0, 2, 5]);
        // (start, nav, page, wrap, expected)
        let cases: &[(usize, Nav, usize, bool, Moved)] = &[
            (1, Nav::Down, 1, false, Moved::To(3)),
            (3, Nav::Up, 1, false, Moved::To(1)),
            (1, Nav::Up, 1, false, Moved::Stayed(1)),
            (4, Nav::Down, 1, false, Moved::Stayed(4)),
            (4, Nav::Down, 1, true, Moved::To(1)),
            (1, Nav::Up, 1, true, Moved::To(4)),
            (1, Nav::Home, 1, false, Moved::Stayed(1)),
            (3, Nav::Home, 1, false, Moved::To(1)),
            (1, Nav::End, 1, false, Moved::To(4)),
            (1, Nav::PageDown, 4, false, Moved::To(4)),
            (1, Nav::PageDown, 1, false, Moved::To(3)),
            (4, Nav::PageUp, 2, false, Moved::To(1)),
            (4, Nav::PageUp, 10, true, Moved::To(1)),
            (1, Nav::Left, 1, false, Moved::Ignored),
            (1, Nav::Right, 1, false, Moved::Ignored),
        ];
        for &(start, nav, page, wrap, want) in cases {
            let mut c = Cursor::new(start).wrap(wrap);
            assert_eq!(
                c.apply(nav, &src, page),
                want,
                "{start} {nav:?} {page} {wrap}"
            );
            assert_eq!(c.index, want.index().unwrap_or(start));
        }
    }

    #[test]
    fn a_horizontal_cursor_takes_left_and_right() {
        let src = rows(&[]);
        let mut c = Cursor::new(0).axis(Axis::Horizontal);
        assert_eq!(c.apply(Nav::Right, &src, 1), Moved::To(1));
        assert_eq!(c.apply(Nav::Left, &src, 1), Moved::To(0));
        assert_eq!(c.apply(Nav::Down, &src, 1), Moved::Ignored);
        assert_eq!(c.apply(Nav::PageDown, &src, 3), Moved::Ignored);
        assert_eq!(c.apply(Nav::End, &src, 1), Moved::To(5));
    }

    #[test]
    fn skip_disabled_off_reaches_disabled_rows_but_type_ahead_never_does() {
        let src = rows(&[1]);
        let mut c = Cursor::new(0).skip_disabled(false);
        assert_eq!(c.apply(Nav::Down, &src, 1), Moved::To(1));
        let mut ta = TypeAhead::default();
        let mut c = Cursor::new(0);
        assert_eq!(c.type_ahead('b', &mut ta, &src), Moved::To(2));
    }

    #[test]
    fn an_empty_source_ignores_everything() {
        let src = rows(&[]);
        let empty = Rows {
            labels: vec![],
            ..src
        };
        let mut c = Cursor::new(0);
        for nav in [Nav::Up, Nav::Down, Nav::Home, Nav::End, Nav::PageDown] {
            assert_eq!(c.apply(nav, &empty, 5), Moved::Ignored);
        }
        assert_eq!(
            c.type_ahead('a', &mut TypeAhead::default(), &empty),
            Moved::Ignored
        );
    }

    #[test]
    fn all_disabled_stays_put() {
        let src = rows(&[0, 1, 2, 3, 4, 5]);
        let mut c = Cursor::new(2);
        assert_eq!(c.apply(Nav::Down, &src, 1), Moved::Stayed(2));
        assert_eq!(c.apply(Nav::Home, &src, 1), Moved::Stayed(2));
        assert_eq!(c.apply(Nav::PageDown, &src, 3), Moved::Stayed(2));
    }

    #[test]
    fn type_ahead_finds_cycles_and_a_miss_is_consumed() {
        let src = rows(&[]);
        let mut ta = TypeAhead::default();
        let mut c = Cursor::new(0);
        assert_eq!(c.type_ahead('c', &mut ta, &src), Moved::To(3));
        ta.reset();
        assert_eq!(c.type_ahead('b', &mut ta, &src), Moved::To(1));
        assert_eq!(c.type_ahead('b', &mut ta, &src), Moved::To(2), "bb cycles");
        ta.reset();
        let m = c.type_ahead('z', &mut ta, &src);
        assert_eq!(m, Moved::Stayed(2));
        assert_eq!(m.outcome(), Outcome::Consumed, "a miss is consumed");
    }

    #[test]
    fn select_places_the_cursor_in_range_only() {
        let src = rows(&[3]);
        let mut c = Cursor::new(0);
        assert_eq!(c.select(3, &src), Moved::To(3));
        assert_eq!(c.select(9, &src), Moved::Ignored);
        assert_eq!(c.index, 3);
    }

    fn key(code: KeyCode, mods: KeyModifiers) -> Key {
        Key { code, mods }
    }

    #[test]
    fn key_step_types_letters_when_type_ahead_is_on() {
        let j = key(KeyCode::Char('j'), KeyModifiers::NONE);
        let down = key(KeyCode::Down, KeyModifiers::NONE);
        let ctrl_j = key(KeyCode::Char('j'), KeyModifiers::CONTROL);
        let space = key(KeyCode::Char(' '), KeyModifiers::NONE);
        assert_eq!(
            KeyStep::of(&j, &NavKeys::VIM, true),
            Some(KeyStep::Type('j'))
        );
        assert_eq!(
            KeyStep::of(&j, &NavKeys::VIM, false),
            Some(KeyStep::Nav(Nav::Down))
        );
        assert_eq!(KeyStep::of(&j, &NavKeys::DESKTOP, false), None);
        assert_eq!(
            KeyStep::of(&down, &NavKeys::DESKTOP, true),
            Some(KeyStep::Nav(Nav::Down))
        );
        assert_eq!(KeyStep::of(&ctrl_j, &NavKeys::DESKTOP, true), None);
        assert_eq!(KeyStep::of(&space, &NavKeys::DESKTOP, true), None);
    }

    #[test]
    fn the_profile_sets_nav_keys_and_type_ahead_defaults() {
        assert_eq!(NavProfile::current(), NavProfile::Desktop);
        assert_eq!(NavProfile::Desktop.nav(), NavKeys::DESKTOP);
        assert!(NavProfile::Desktop.type_ahead());
        NavProfile::Vim.install();
        assert_eq!(NavProfile::current().nav(), NavKeys::VIM);
        assert!(!NavProfile::current().type_ahead());
        NavProfile::Desktop.install();
        assert_eq!(TypeAhead::default().timeout, Duration::from_millis(1000));
    }
}
