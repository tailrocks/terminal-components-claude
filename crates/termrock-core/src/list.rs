//! List cursor and type-ahead routing (ported from PR #26
//! `src/core/list.rs`, commit f4867e9b5).
//!
//! The pieces every list-like widget shares: a row source abstraction, a
//! cursor that steps over enabled rows, type-ahead, the key-to-step
//! classification and the shortcuts-versus-type-ahead routing pass. This is
//! a capability for widgets to adopt; existing widgets keep their current
//! behavior until they opt in.

use std::cell::Cell;

use crate::event::{Axis, Key};
use crate::keymap::{Nav, NavKeys};
use crate::outcome::Outcome;
use crate::typeahead::TypeAhead;

/// The rows a cursor moves over.
pub trait RowSource {
    /// The number of rows.
    fn len(&self) -> usize;

    /// Whether there are no rows.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Whether row `i` can take the cursor.
    fn enabled(&self, i: usize) -> bool {
        let _ = i;
        true
    }

    /// The type-ahead label of row `i`.
    fn label(&self, i: usize) -> &str;
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

/// A row source built from closures.
pub struct RowFn<'a> {
    /// The number of rows.
    pub len: usize,
    /// Whether a row can take the cursor.
    pub enabled: &'a dyn Fn(usize) -> bool,
    /// The type-ahead label of a row.
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

/// Where a cursor move went.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Moved {
    /// The move does not apply to this cursor (wrong axis, empty list).
    Ignored,
    /// The cursor stayed where it was (edge, all disabled, a type-ahead miss).
    Stayed(usize),
    /// The cursor moved to a new index.
    To(usize),
}

impl Moved {
    /// The resulting index, if the move applied.
    pub fn index(self) -> Option<usize> {
        match self {
            Moved::Ignored => None,
            Moved::Stayed(i) | Moved::To(i) => Some(i),
        }
    }

    /// Whether the cursor actually moved.
    pub fn moved(self) -> bool {
        matches!(self, Moved::To(_))
    }

    /// The input outcome for the move.
    pub fn outcome(self) -> Outcome {
        match self {
            Moved::Ignored => Outcome::Ignored,
            Moved::Stayed(_) => Outcome::Consumed,
            Moved::To(_) => Outcome::Changed,
        }
    }
}

/// A cursor over a row source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    /// The index the cursor is on.
    pub index: usize,
    /// Whether movement wraps at the edges.
    pub wrap: bool,
    /// Which axis the arrows move on.
    pub axis: Axis,
    /// Whether movement skips rows that cannot take the cursor.
    pub skip_disabled: bool,
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Cursor {
    /// A cursor at `index` that skips disabled rows and clamps.
    pub const fn new(index: usize) -> Self {
        Self {
            index,
            wrap: false,
            axis: Axis::V,
            skip_disabled: true,
        }
    }

    /// Set whether movement wraps.
    pub const fn wrap(mut self, on: bool) -> Self {
        self.wrap = on;
        self
    }

    /// Set which axis the arrows move on.
    pub const fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    /// Set whether movement skips disabled rows.
    pub const fn skip_disabled(mut self, on: bool) -> Self {
        self.skip_disabled = on;
        self
    }

    fn usable<S: RowSource + ?Sized>(&self, src: &S, i: usize) -> bool {
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

    /// Apply a navigation action.
    pub fn apply<S: RowSource + ?Sized>(&mut self, nav: Nav, src: &S, page: usize) -> Moved {
        let n = src.len();
        if n == 0 {
            return Moved::Ignored;
        }
        let vertical = self.axis == Axis::V;
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

    /// Step `delta` rows, skipping disabled rows.
    pub fn step<S: RowSource + ?Sized>(&mut self, delta: isize, src: &S) -> Moved {
        let n = src.len();
        if n == 0 {
            return Moved::Ignored;
        }
        let from = self.index.min(n - 1);
        let mut i = from;
        for _ in 0..n {
            let next = crate::nav::step_index(i, n, delta, self.wrap);
            if next == i {
                // The wrap came back to the start: nowhere to go.
                break;
            }
            i = next;
            if self.usable(src, i) {
                return self.settle(from, i);
            }
        }
        self.settle(from, from)
    }

    /// Move to the first enabled row.
    pub fn first<S: RowSource + ?Sized>(&mut self, src: &S) -> Moved {
        let n = src.len();
        if n == 0 {
            return Moved::Ignored;
        }
        let from = self.index.min(n - 1);
        let to = (0..n).find(|&i| self.usable(src, i)).unwrap_or(from);
        self.settle(from, to)
    }

    /// Move to the last enabled row.
    pub fn last<S: RowSource + ?Sized>(&mut self, src: &S) -> Moved {
        let n = src.len();
        if n == 0 {
            return Moved::Ignored;
        }
        let from = self.index.min(n - 1);
        let to = (0..n).rev().find(|&i| self.usable(src, i)).unwrap_or(from);
        self.settle(from, to)
    }

    fn nearest<S: RowSource + ?Sized>(&self, src: &S, to: usize, from: usize) -> usize {
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

    /// Move to row `i` if it exists.
    pub fn select<S: RowSource + ?Sized>(&mut self, i: usize, src: &S) -> Moved {
        if i >= src.len() {
            return Moved::Ignored;
        }
        let from = self.index;
        self.settle(from, i)
    }

    /// Type a character: feed the prefix and move to the next matching row.
    pub fn type_ahead<S: RowSource + ?Sized>(
        &mut self,
        c: char,
        ta: &mut TypeAhead,
        src: &S,
    ) -> Moved {
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

/// What a key press does to a list: navigate, or type into type-ahead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyStep {
    /// A navigation action.
    Nav(Nav),
    /// A character to type.
    Type(char),
}

impl KeyStep {
    /// Classify a key with the default precedence (shortcuts first).
    pub fn of(key: &Key, nav: &NavKeys, type_ahead: bool) -> Option<Self> {
        Self::ordered(key, nav, type_ahead, TypeAheadPrecedence::TypeAhead)
    }

    /// Classify a key with an explicit precedence.
    pub fn ordered(
        key: &Key,
        nav: &NavKeys,
        type_ahead: bool,
        precedence: TypeAheadPrecedence,
    ) -> Option<Self> {
        if type_ahead && let Some(c) = TypeAhead::char_of(key) {
            if precedence == TypeAheadPrecedence::Shortcuts
                && ROUTE.with(Cell::get) == Pass::Shortcuts
            {
                // Inside the shortcuts pass the bound key wins; record the
                // yield so the caller can retry the character as type-ahead.
                YIELDED.with(|y| y.set(true));
            } else {
                return Some(KeyStep::Type(c));
            }
        }
        nav.action(key).map(KeyStep::Nav)
    }
}

/// Whether a printable character goes to type-ahead or to shortcuts first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum TypeAheadPrecedence {
    /// A bound shortcut letter wins; the rest type.
    #[default]
    Shortcuts,
    /// Every printable character types at once.
    TypeAhead,
}

thread_local! {
    static PRECEDENCE: Cell<TypeAheadPrecedence> =
        const { Cell::new(TypeAheadPrecedence::Shortcuts) };
    static ROUTE: Cell<Pass> = const { Cell::new(Pass::Direct) };
    static YIELDED: Cell<bool> = const { Cell::new(false) };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pass {
    /// Not inside `route_key`.
    Direct,
    /// The first pass: shortcuts handle their letters.
    Shortcuts,
    /// The second pass: the list types.
    TypeAhead,
}

impl TypeAheadPrecedence {
    /// Set the precedence for the whole process.
    pub fn install(self) {
        PRECEDENCE.with(|p| p.set(self));
    }

    /// The current process-wide precedence.
    pub fn current() -> Self {
        PRECEDENCE.with(Cell::get)
    }
}

struct PassGuard(Pass);

impl PassGuard {
    fn enter(pass: Pass) -> Self {
        Self(ROUTE.with(|r| r.replace(pass)))
    }
}

impl Drop for PassGuard {
    fn drop(&mut self) {
        ROUTE.with(|r| r.set(self.0));
    }
}

/// Route a printable key: let shortcuts take their bound letters first, then
/// let the list type what is left. A handler yields by leaving the letter
/// `Outcome::Ignored`; if the first pass yielded and did not change anything,
/// the same key is offered to the handler again in the type-ahead pass.
pub fn route_key(key: &Key, mut handle: impl FnMut() -> Outcome) -> Outcome {
    if ROUTE.with(Cell::get) != Pass::Direct || TypeAhead::char_of(key).is_none() {
        return handle();
    }
    let first = {
        let _pass = PassGuard::enter(Pass::Shortcuts);
        YIELDED.with(|y| y.set(false));
        handle()
    };
    if !YIELDED.with(|y| y.replace(false)) || first == Outcome::Changed {
        return first;
    }
    let _pass = PassGuard::enter(Pass::TypeAhead);
    handle().or(first)
}

/// Which navigation keys a profile uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum NavProfile {
    /// Arrows, paging, Home / End; letters go to type-ahead.
    #[default]
    Desktop,
    /// The desktop keys plus vim letters; letters do not type.
    Vim,
}

thread_local! {
    static PROFILE: Cell<NavProfile> = const { Cell::new(NavProfile::Desktop) };
}

impl NavProfile {
    /// The navigation keys of the profile.
    pub const fn nav(self) -> NavKeys {
        match self {
            NavProfile::Desktop => NavKeys::DESKTOP,
            NavProfile::Vim => NavKeys::VIM,
        }
    }

    /// Whether letters type by default under the profile.
    pub const fn type_ahead(self) -> bool {
        matches!(self, NavProfile::Desktop)
    }

    /// Set the profile for the whole process.
    pub fn install(self) {
        PROFILE.with(|p| p.set(self));
    }

    /// The current process-wide profile.
    pub fn current() -> Self {
        PROFILE.with(Cell::get)
    }
}

/// A row source that owns a cursor.
pub trait Navigable: RowSource {
    /// The cursor.
    fn cursor(&self) -> Cursor;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::{KeyCode, KeyModifiers};
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
        let mut c = Cursor::new(0).axis(Axis::H);
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
        let question = key(KeyCode::Char('?'), KeyModifiers::NONE);
        assert_eq!(
            KeyStep::of(&question, &NavKeys::DESKTOP, true),
            None,
            "punctuation stays the app's (`?` help, `/` find)"
        );
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

    struct Page {
        cursor: Cursor,
        ta: TypeAhead,
        shortcut_runs: usize,
        swallow: bool,
        precedence: TypeAheadPrecedence,
    }

    impl Page {
        const ROWS: &[&str] = &["Alpha", "Delta", "Quit log"];

        fn new() -> Self {
            Self {
                cursor: Cursor::new(0),
                ta: TypeAhead::with_timeout(Duration::ZERO),
                shortcut_runs: 0,
                swallow: false,
                precedence: TypeAheadPrecedence::current(),
            }
        }

        fn on_key(&mut self, k: &Key) -> Outcome {
            if let Some(KeyStep::Type(c)) =
                KeyStep::ordered(k, &NavKeys::DESKTOP, true, self.precedence)
            {
                return self
                    .cursor
                    .type_ahead(c, &mut self.ta, Self::ROWS)
                    .outcome();
            }
            if k.code == KeyCode::Char('q') {
                self.shortcut_runs += 1;
                return Outcome::Changed;
            }
            if self.swallow {
                Outcome::Consumed
            } else {
                Outcome::Ignored
            }
        }

        fn press(&mut self, c: char) -> Outcome {
            let k = key(KeyCode::Char(c), KeyModifiers::NONE);
            route_key(&k, || self.on_key(&k))
        }
    }

    #[test]
    fn a_bound_letter_wins_over_type_ahead_and_the_rest_type() {
        let mut p = Page::new();
        assert_eq!(p.press('q'), Outcome::Changed);
        assert_eq!(p.shortcut_runs, 1, "the shortcut ran");
        assert_eq!(p.cursor.index, 0, "and the list did not type it");
        assert_eq!(p.press('d'), Outcome::Changed);
        assert_eq!(p.cursor.index, 1, "an unbound letter types");
        assert_eq!(p.press('z'), Outcome::Consumed, "a miss is consumed");
        assert_eq!(p.press('?'), Outcome::Ignored, "punctuation is never typed");

        let mut m = Page::new();
        m.swallow = true;
        assert_eq!(m.press('d'), Outcome::Changed);
        assert_eq!(m.cursor.index, 1);
        assert_eq!(m.shortcut_runs, 0);
    }

    #[test]
    fn type_ahead_first_is_an_app_wide_and_a_per_widget_choice() {
        let mut p = Page::new();
        p.precedence = TypeAheadPrecedence::TypeAhead;
        p.press('q');
        assert_eq!(
            (p.shortcut_runs, p.cursor.index),
            (0, 2),
            "the list typed q"
        );

        TypeAheadPrecedence::TypeAhead.install();
        let mut a = Page::new();
        a.press('q');
        assert_eq!((a.shortcut_runs, a.cursor.index), (0, 2));
        TypeAheadPrecedence::Shortcuts.install();
        assert_eq!(
            TypeAheadPrecedence::current(),
            TypeAheadPrecedence::Shortcuts
        );
    }

    #[test]
    fn outside_route_key_and_in_popups_a_letter_types_at_once() {
        let q = key(KeyCode::Char('q'), KeyModifiers::NONE);

        let mut p = Page::new();
        p.on_key(&q);
        assert_eq!((p.shortcut_runs, p.cursor.index), (0, 2));

        let step = route_key(&q, || {
            assert_eq!(
                KeyStep::of(&q, &NavKeys::DESKTOP, true),
                Some(KeyStep::Type('q'))
            );
            Outcome::Changed
        });
        assert_eq!(step, Outcome::Changed);
    }
}
