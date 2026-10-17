//! Row selection: which rows of a table or list are selected, independent
//! of the cursor (keyboard position) and of the display order.
//!
//! Rows are keyed by their *source* index, so a selection survives sorting
//! and scrolling. Ranges are resolved against the current display order
//! (`order[display] == source`), which is what the user sees when they
//! Shift-extend.
//!
//! The model follows the macOS table conventions:
//!
//! - a plain click selects only that row and sets the anchor;
//! - Shift extends from the anchor to the target, keeping what was selected
//!   before the anchor was set;
//! - Cmd/Ctrl toggles one row and moves the anchor;
//! - select-all and clear act on every *selectable* row (disabled rows are
//!   never selected);
//! - a right click inside the selection keeps it, outside it selects only
//!   the clicked row ([`RowSelection::context_select`]).
//!
//! Every widget that keeps a selection changes it through one entry point,
//! [`RowSelection::apply`], with the [`SelectGesture`] a click
//! ([`SelectGesture::from_mods`]) or a key ([`SelectGesture::from_key`])
//! stands for. What a click does beyond selecting (activate, edit, toggle a
//! check) is the widget's [`ClickPolicy`].
//!
//! Standards: macOS table and outline views (click, Shift-click,
//! Cmd-click), WAI-ARIA APG multi-select listbox and grid (Space toggles,
//! Shift+arrows extend, Ctrl+A selects all), Windows UX mouse guidelines
//! (one click selects, a double click performs the default action).

use std::collections::BTreeSet;

use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::core::event::{ClientPlatform, Key};

/// How many rows a widget lets the user select.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionMode {
    /// Rows can be activated but never stay selected.
    None,
    /// At most one row is selected (the chosen row).
    #[default]
    Single,
    /// Any set of rows: Space and Cmd/Ctrl-click toggle, Shift extends a
    /// range, Ctrl/Cmd+A selects all.
    Multi,
}

/// Aggregate state of a selection over its rows: drives a "select all"
/// header checkbox (`[ ]`, `[–]`, `[✓]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriState {
    /// No selectable row is selected.
    None,
    /// Some, but not all, selectable rows are selected.
    Mixed,
    /// Every selectable row is selected (and there is at least one).
    All,
}

impl TriState {
    /// The checkbox glyph for this state: `[ ]`, `[–]` or `[✓]`.
    pub fn glyph(self) -> &'static str {
        self.glyph_in(&crate::theme::Glyphs::UNICODE)
    }

    /// The checkbox glyph for this state from `g`
    /// ([`Glyphs::check_off`](crate::theme::Glyphs::check_off),
    /// `check_mixed` or `check_on`).
    pub fn glyph_in(self, g: &crate::theme::Glyphs) -> &'static str {
        match self {
            TriState::None => g.check_off,
            TriState::Mixed => g.check_mixed,
            TriState::All => g.check_on,
        }
    }
}

/// How one click or key changes a [`RowSelection`]
/// ([`RowSelection::apply`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectGesture {
    /// Select only the target row and anchor there: a plain click, or a
    /// right click outside the selection.
    Only,
    /// Toggle the target row and anchor there: Space, the platform's toggle
    /// click ([`SelectGesture::toggle_mods_for`]), a click on the row's
    /// check box.
    Toggle,
    /// Extend from the anchor to the target: Shift-click, Shift+arrows.
    /// `from` is the display row the cursor stood on before the move; it
    /// becomes the anchor when there is none yet (so a click followed by
    /// Shift+arrows extends from the clicked row: the fallback where the
    /// terminal keeps Shift-click for its own text selection, SIg2).
    Extend { from: usize },
    /// Select every selectable row (Select All).
    All,
}

impl SelectGesture {
    /// Ctrl or Cmd toggles a row: the default toggle click off macOS.
    pub const TOGGLE_CTRL_CMD: KeyModifiers = KeyModifiers::CONTROL.union(KeyModifiers::SUPER);
    /// Option (Alt) or Cmd toggles a row: the default toggle click on
    /// macOS, where Ctrl+click is a secondary click
    /// (`InputConfig::ctrl_click_secondary`) and Cmd rarely reaches a
    /// terminal app. iTerm2 and Terminal.app can keep Option+click (cursor
    /// moves, block selection), so it is a fallback beside Space and a
    /// row's check box, never the only way.
    pub const TOGGLE_OPTION_CMD: KeyModifiers = KeyModifiers::ALT.union(KeyModifiers::SUPER);
    /// Ctrl, Cmd or Alt (Option) toggles a row.
    pub const TOGGLE_WITH_ALT: KeyModifiers = Self::TOGGLE_CTRL_CMD.union(KeyModifiers::ALT);

    /// The default toggle-click modifiers on `platform`:
    /// [`Self::TOGGLE_OPTION_CMD`] on macOS, [`Self::TOGGLE_CTRL_CMD`]
    /// elsewhere.
    pub const fn toggle_mods_for(platform: ClientPlatform) -> KeyModifiers {
        match platform {
            ClientPlatform::MacOs => Self::TOGGLE_OPTION_CMD,
            ClientPlatform::Other => Self::TOGGLE_CTRL_CMD,
        }
    }

    /// [`Self::toggle_mods_for`] the client platform
    /// ([`ClientPlatform::current`]).
    pub fn toggle_mods() -> KeyModifiers {
        Self::toggle_mods_for(ClientPlatform::current())
    }

    /// The gesture of a click with `mods` on a row while the cursor stands
    /// on display row `from`: Shift extends (it wins over a toggle
    /// modifier, as in Finder), any of `toggle` toggles, anything else
    /// selects only the row.
    pub fn from_mods(mods: KeyModifiers, toggle: KeyModifiers, from: usize) -> Self {
        if mods.contains(KeyModifiers::SHIFT) {
            Self::Extend { from }
        } else if mods.intersects(toggle) {
            Self::Toggle
        } else {
            Self::Only
        }
    }

    /// The gesture a key stands for while the cursor is on display row
    /// `from`: plain Space toggles; Shift on a movement key extends (call
    /// it for a key that moved the cursor). `None` for anything else: a
    /// plain cursor move leaves the selection alone (APG multi-select
    /// listbox and grid).
    pub fn from_key(key: &Key, from: usize) -> Option<Self> {
        let other = key.ctrl() || key.cmd() || key.alt();
        match key.code {
            KeyCode::Char(' ') if key.mods.is_empty() => Some(Self::Toggle),
            KeyCode::Char(_) => None,
            _ if key.shift() && !other => Some(Self::Extend { from }),
            _ => None,
        }
    }
}

/// Whether a click (or the `count`th click of a multi-click) performs an
/// action: activates a row, starts editing a cell. Terminals report single
/// clicks only; the pointer router counts them (`PointerConfig`'s double
/// click time).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Activate {
    /// One click does it: navigation lists, sidebars, menus, launchers.
    Single,
    /// A double click does it and one click only selects: data lists,
    /// tables and trees (macOS Finder and Mail, Windows, GNOME).
    #[default]
    Double,
    /// A click on the row (or cell) the cursor already stood on does it,
    /// with no time window: the tables' earlier rule, kept as an option.
    ClickAgain,
    /// Clicks never do it (Return still does).
    Never,
}

impl Activate {
    /// Whether the `count`th click fires; `again` is whether it landed on
    /// the row the cursor already stood on before the click.
    pub fn fires(self, count: u8, again: bool) -> bool {
        match self {
            Self::Single => true,
            Self::Double => count >= 2,
            Self::ClickAgain => again,
            Self::Never => false,
        }
    }
}

/// What a click on a checkable row toggles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CheckClick {
    /// Only a click on the row's check box toggles it; a click on the rest
    /// of the row selects it (macOS lists and tables, M15). Space toggles
    /// either way.
    #[default]
    BoxOnly,
    /// A click anywhere on the row toggles its check: a checklist whose row
    /// is the control, as a macOS checkbox's label is.
    Row,
}

/// What a click on a row does beyond moving the cursor: the one place a
/// list, tree, table or grid reads its activation, edit-entry and
/// check-toggle rules from. Default [`ClickPolicy::DATA`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClickPolicy {
    /// When a click activates the row (opens, chooses, follows).
    pub activate: Activate,
    /// When a click starts editing an editable cell. The click that gives
    /// a cell the cursor never edits under the default (M9).
    pub edit_on: Activate,
    /// What a click on a checkable row toggles.
    pub check: CheckClick,
}

impl ClickPolicy {
    /// Data views (lists of records, tables, trees, grids): one click
    /// selects, a double click activates or edits, only the check box
    /// toggles a check.
    pub const DATA: Self = Self {
        activate: Activate::Double,
        edit_on: Activate::Double,
        check: CheckClick::BoxOnly,
    };
    /// Navigation lists, sidebars and menus: one click activates (SI14).
    pub const NAVIGATION: Self = Self {
        activate: Activate::Single,
        edit_on: Activate::Never,
        check: CheckClick::BoxOnly,
    };

    /// See [`Self::activate`].
    pub const fn activate(mut self, activate: Activate) -> Self {
        self.activate = activate;
        self
    }

    /// See [`Self::edit_on`].
    pub const fn edit_on(mut self, edit_on: Activate) -> Self {
        self.edit_on = edit_on;
        self
    }

    /// See [`Self::check`].
    pub const fn check(mut self, check: CheckClick) -> Self {
        self.check = check;
        self
    }

    /// Whether the `count`th click activates (`again`: it landed on the
    /// cursor row).
    pub fn activates(&self, count: u8, again: bool) -> bool {
        self.activate.fires(count, again)
    }

    /// Whether the `count`th click on an editable cell starts editing
    /// (`again`: it landed on the cursor cell).
    pub fn edits(&self, count: u8, again: bool) -> bool {
        self.edit_on.fires(count, again)
    }

    /// Whether a click toggles the row's check: always on the check box,
    /// anywhere on the row under [`CheckClick::Row`].
    pub fn toggles(&self, on_box: bool) -> bool {
        on_box || self.check == CheckClick::Row
    }
}

impl Default for ClickPolicy {
    fn default() -> Self {
        Self::DATA
    }
}

/// A set of selected source rows plus the range anchor.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RowSelection {
    rows: BTreeSet<usize>,
    /// Source row the next Shift-extension starts from.
    anchor: Option<usize>,
    /// What was selected when the anchor was set: a Shift-extension is
    /// this base plus the anchor range, so shrinking a range deselects the
    /// rows it leaves.
    base: BTreeSet<usize>,
}

impl RowSelection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
    pub fn contains(&self, src: usize) -> bool {
        self.rows.contains(&src)
    }
    /// Selected source rows in ascending source order.
    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.rows.iter().copied()
    }
    /// Selected source rows in ascending source order.
    pub fn to_vec(&self) -> Vec<usize> {
        self.iter().collect()
    }
    /// The range anchor (a source row), if any.
    pub fn anchor(&self) -> Option<usize> {
        self.anchor
    }

    fn set_anchor(&mut self, src: Option<usize>) {
        self.anchor = src;
        self.base = self.rows.clone();
        if let Some(a) = src {
            self.base.remove(&a);
        }
    }

    /// Clear the selection and the anchor. Returns whether anything changed.
    pub fn clear(&mut self) -> bool {
        let changed = !self.rows.is_empty();
        self.rows.clear();
        self.set_anchor(None);
        changed
    }

    /// Select only `src` (a plain click) and anchor there. A row that is
    /// not selectable clears the selection and leaves no anchor.
    pub fn select_only(&mut self, src: usize, selectable: impl Fn(usize) -> bool) -> bool {
        let before = self.rows.clone();
        self.rows.clear();
        if selectable(src) {
            self.rows.insert(src);
            self.set_anchor(Some(src));
        } else {
            self.set_anchor(None);
        }
        self.rows != before
    }

    /// Toggle `src` (Space, Cmd/Ctrl-click) and anchor there. Returns
    /// whether it changed; a row that is not selectable never changes.
    pub fn toggle(&mut self, src: usize, selectable: impl Fn(usize) -> bool) -> bool {
        if !selectable(src) {
            return false;
        }
        if !self.rows.remove(&src) {
            self.rows.insert(src);
        }
        self.set_anchor(Some(src));
        true
    }

    /// Extend from the anchor to the row at display position `to`
    /// (Shift+arrow, Shift-click). `order` maps display position to source
    /// row. Without an anchor the row at `from` (the cursor before the
    /// move) becomes the anchor. Rows that are not selectable inside the
    /// range are skipped.
    pub fn extend(
        &mut self,
        order: &[usize],
        from: usize,
        to: usize,
        selectable: impl Fn(usize) -> bool,
    ) -> bool {
        if order.is_empty() {
            return false;
        }
        let to = to.min(order.len() - 1);
        let anchor_display = self
            .anchor
            .and_then(|a| order.iter().position(|&s| s == a))
            .unwrap_or_else(|| {
                let from = from.min(order.len() - 1);
                self.set_anchor(Some(order[from]));
                from
            });
        let (lo, hi) = if anchor_display <= to {
            (anchor_display, to)
        } else {
            (to, anchor_display)
        };
        let mut next = self.base.clone();
        next.extend(order[lo..=hi].iter().copied().filter(|&s| selectable(s)));
        let changed = next != self.rows;
        self.rows = next;
        changed
    }

    /// Select every selectable row of `rows`. Returns whether it changed.
    pub fn select_all(
        &mut self,
        rows: impl IntoIterator<Item = usize>,
        selectable: impl Fn(usize) -> bool,
    ) -> bool {
        let before = self.rows.len();
        self.rows
            .extend(rows.into_iter().filter(|&s| selectable(s)));
        self.set_anchor(self.anchor);
        self.rows.len() != before
    }

    /// The header checkbox action: select all when not everything is
    /// selected, clear otherwise (macOS / Gmail behaviour for `[–]`).
    pub fn toggle_all(
        &mut self,
        rows: impl IntoIterator<Item = usize> + Clone,
        selectable: impl Fn(usize) -> bool + Copy,
    ) -> bool {
        match self.tri_state(rows.clone(), selectable) {
            TriState::All => self.clear(),
            _ => self.select_all(rows, selectable),
        }
    }

    /// Aggregate state over the selectable rows of `rows`.
    pub fn tri_state(
        &self,
        rows: impl IntoIterator<Item = usize>,
        selectable: impl Fn(usize) -> bool,
    ) -> TriState {
        let mut total = 0usize;
        let mut on = 0usize;
        for s in rows.into_iter().filter(|&s| selectable(s)) {
            total += 1;
            if self.rows.contains(&s) {
                on += 1;
            }
        }
        if on == 0 {
            TriState::None
        } else if on == total {
            TriState::All
        } else {
            TriState::Mixed
        }
    }

    /// The one entry point for a gesture: `g` aimed at display row `to`,
    /// with `order` mapping display rows to source rows. Rows `enabled`
    /// rejects are never selected. Returns whether the selection changed;
    /// a target past the rows changes nothing.
    pub fn apply(
        &mut self,
        g: SelectGesture,
        to: usize,
        order: &[usize],
        enabled: impl Fn(usize) -> bool,
    ) -> bool {
        if g == SelectGesture::All {
            return self.select_all(order.iter().copied(), enabled);
        }
        let Some(&src) = order.get(to) else {
            return false;
        };
        match g {
            SelectGesture::Only => self.select_only(src, enabled),
            SelectGesture::Toggle => self.toggle(src, enabled),
            SelectGesture::Extend { from } => self.extend(order, from, to, enabled),
            SelectGesture::All => false,
        }
    }

    /// The selection rule of a right click (or the menu key) on display row
    /// `to`: inside the selection it stays, so the menu acts on every
    /// selected row; outside, only that row is selected
    /// ([`SelectGesture::Only`]), so the menu never acts on rows the user
    /// did not mean (macOS, M10). Returns whether the selection changed.
    pub fn context_select(
        &mut self,
        to: usize,
        order: &[usize],
        enabled: impl Fn(usize) -> bool,
    ) -> bool {
        match order.get(to) {
            Some(&src) if !self.contains(src) => {
                self.apply(SelectGesture::Only, to, order, enabled)
            }
            _ => false,
        }
    }

    /// Check or uncheck source row `src` without moving the anchor (an
    /// owner restoring or seeding marks). Returns whether it changed.
    pub fn set(&mut self, src: usize, on: bool) -> bool {
        if on {
            self.rows.insert(src)
        } else {
            self.rows.remove(&src)
        }
    }

    /// Drop rows that no longer qualify (deleted, now disabled).
    pub fn retain(&mut self, keep: impl Fn(usize) -> bool) {
        self.rows.retain(|&s| keep(s));
        self.base.retain(|&s| keep(s));
        if self.anchor.is_some_and(|a| !keep(a)) {
            self.anchor = None;
        }
    }

    /// Re-key the selection after rows were removed: `map(old)` gives the
    /// new source index, or `None` when the row is gone.
    pub fn remap(&mut self, map: impl Fn(usize) -> Option<usize>) {
        self.rows = self.rows.iter().filter_map(|&s| map(s)).collect();
        self.base = self.base.iter().filter_map(|&s| map(s)).collect();
        self.anchor = self.anchor.and_then(&map);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn any(_: usize) -> bool {
        true
    }

    #[test]
    fn toggle_flips_one_row_and_skips_disabled() {
        let mut s = RowSelection::new();
        assert!(s.toggle(2, any));
        assert!(s.contains(2));
        assert!(s.toggle(2, any));
        assert!(s.is_empty());
        assert!(!s.toggle(3, |r| r != 3), "a disabled row never toggles");
        assert!(s.is_empty());
    }

    #[test]
    fn select_only_replaces_the_selection() {
        let mut s = RowSelection::new();
        s.toggle(1, any);
        s.toggle(4, any);
        assert!(s.select_only(2, any));
        assert_eq!(s.to_vec(), vec![2]);
        assert_eq!(s.anchor(), Some(2));
        s.select_only(5, |r| r != 5);
        assert!(
            s.is_empty(),
            "clicking a disabled row clears, selects nothing"
        );
    }

    #[test]
    fn extend_selects_the_display_range_from_the_anchor() {
        // display order differs from source order (a sorted table)
        let order = [4, 2, 0, 3, 1];
        let mut s = RowSelection::new();
        s.select_only(2, any); // display 1
        s.extend(&order, 1, 3, any);
        assert_eq!(s.to_vec(), vec![0, 2, 3]);
        // shrinking the range deselects what it leaves
        s.extend(&order, 3, 2, any);
        assert_eq!(s.to_vec(), vec![0, 2]);
        // crossing the anchor flips the range direction
        s.extend(&order, 2, 0, any);
        assert_eq!(s.to_vec(), vec![2, 4]);
    }

    #[test]
    fn extend_keeps_rows_selected_before_the_anchor() {
        let order = [0, 1, 2, 3, 4, 5];
        let mut s = RowSelection::new();
        s.toggle(0, any);
        s.toggle(3, any); // anchor 3, base {0}
        s.extend(&order, 3, 5, any);
        assert_eq!(s.to_vec(), vec![0, 3, 4, 5]);
    }

    #[test]
    fn extend_without_anchor_starts_at_the_cursor_and_skips_disabled() {
        let order = [0, 1, 2, 3];
        let mut s = RowSelection::new();
        s.extend(&order, 0, 3, |r| r != 2);
        assert_eq!(s.to_vec(), vec![0, 1, 3]);
        assert_eq!(s.anchor(), Some(0));
    }

    #[test]
    fn tri_state_and_select_all_ignore_disabled_rows() {
        let rows = 0..4;
        let enabled = |r: usize| r != 1;
        let mut s = RowSelection::new();
        assert_eq!(s.tri_state(rows.clone(), enabled), TriState::None);
        s.toggle(0, enabled);
        assert_eq!(s.tri_state(rows.clone(), enabled), TriState::Mixed);
        assert!(s.select_all(rows.clone(), enabled));
        assert_eq!(s.to_vec(), vec![0, 2, 3]);
        assert_eq!(s.tri_state(rows.clone(), enabled), TriState::All);
        // the header toggle: all → none, mixed → all
        assert!(s.toggle_all(rows.clone(), enabled));
        assert!(s.is_empty());
        s.toggle(2, enabled);
        s.toggle_all(rows.clone(), enabled);
        assert_eq!(s.tri_state(rows, enabled), TriState::All);
    }

    #[test]
    fn tri_state_glyphs() {
        assert_eq!(TriState::None.glyph(), "[ ]");
        assert_eq!(TriState::Mixed.glyph(), "[–]");
        assert_eq!(TriState::All.glyph(), "[✓]");
    }

    /// The selection model as one table: start state, gesture, target
    /// display row, then the selected source rows and the anchor. The order
    /// is a sorted table's (display → source); source row 3 (display 2) is
    /// disabled.
    #[test]
    fn apply_follows_the_selection_model_table() {
        use SelectGesture::*;
        let order = [4, 2, 3, 0, 1];
        let enabled = |r: usize| r != 3;
        type Row = (
            &'static [usize],
            Option<usize>,
            SelectGesture,
            usize,
            &'static [usize],
            Option<usize>,
        );
        // (selected sources, anchor source, gesture, to, expected, anchor)
        let table: &[Row] = &[
            // a plain click selects only the row and anchors there
            (&[], None, Only, 1, &[2], Some(2)),
            (&[0, 1, 4], Some(4), Only, 3, &[0], Some(0)),
            // a plain click on a disabled row clears and leaves no anchor
            (&[0, 1], Some(0), Only, 2, &[], None),
            // the toggle click flips one row and moves the anchor
            (&[0], Some(0), Toggle, 0, &[0, 4], Some(4)),
            (&[0, 4], Some(4), Toggle, 0, &[0], Some(4)),
            // a disabled row never toggles
            (&[0], Some(0), Toggle, 2, &[0], Some(0)),
            // Shift extends from the anchor across the display order and
            // skips the disabled row
            (&[2], Some(2), Extend { from: 1 }, 4, &[0, 1, 2], Some(2)),
            // the range is the anchor to the target, whatever `from` was
            // (shrinking a range: `extend_selects_the_display_range_..`)
            (&[2], Some(2), Extend { from: 4 }, 3, &[0, 2], Some(2)),
            // ...across the anchor the other way
            (&[2], Some(2), Extend { from: 3 }, 0, &[2, 4], Some(2)),
            // rows selected before the anchor was set stay
            (&[1, 2], Some(2), Extend { from: 1 }, 0, &[1, 2, 4], Some(2)),
            // without an anchor the cursor row (`from`) anchors the range
            (&[], None, Extend { from: 0 }, 1, &[2, 4], Some(4)),
            // Select All takes every enabled row, the anchor stays
            (&[2], Some(2), All, 0, &[0, 1, 2, 4], Some(2)),
            // a target past the rows changes nothing
            (&[2], Some(2), Only, 9, &[2], Some(2)),
        ];
        for (k, &(rows, anchor, g, to, want, want_anchor)) in table.iter().enumerate() {
            let mut s = RowSelection::new();
            for &r in rows {
                s.set(r, true);
            }
            s.set_anchor(anchor);
            s.apply(g, to, &order, enabled);
            assert_eq!(s.to_vec(), want, "row {k}: {g:?} on display {to}");
            assert_eq!(s.anchor(), want_anchor, "row {k}: anchor");
        }
    }

    /// M10: a right click inside the selection keeps it; outside it selects
    /// only the clicked row.
    #[test]
    fn context_select_keeps_a_selection_it_lands_in() {
        let order = [0, 1, 2, 3];
        let mut s = RowSelection::new();
        s.apply(SelectGesture::Only, 0, &order, any);
        s.apply(SelectGesture::Extend { from: 0 }, 2, &order, any);
        assert!(!s.context_select(1, &order, any), "inside: kept");
        assert_eq!(s.to_vec(), vec![0, 1, 2]);
        assert!(s.context_select(3, &order, any), "outside: only that row");
        assert_eq!(s.to_vec(), vec![3]);
        assert_eq!(s.anchor(), Some(3));
    }

    #[test]
    fn gestures_come_from_the_click_modifiers_and_keys() {
        let toggle = SelectGesture::TOGGLE_CTRL_CMD;
        let m = |mods| SelectGesture::from_mods(mods, toggle, 7);
        assert_eq!(m(KeyModifiers::NONE), SelectGesture::Only);
        assert_eq!(m(KeyModifiers::CONTROL), SelectGesture::Toggle);
        assert_eq!(m(KeyModifiers::SUPER), SelectGesture::Toggle);
        assert_eq!(
            m(KeyModifiers::ALT),
            SelectGesture::Only,
            "not a toggle here"
        );
        assert_eq!(
            m(KeyModifiers::SHIFT | KeyModifiers::CONTROL),
            SelectGesture::Extend { from: 7 },
            "Shift wins"
        );
        assert_eq!(
            SelectGesture::toggle_mods_for(ClientPlatform::MacOs),
            SelectGesture::TOGGLE_OPTION_CMD
        );
        assert_eq!(
            SelectGesture::toggle_mods_for(ClientPlatform::Other),
            SelectGesture::TOGGLE_CTRL_CMD
        );
        let k = |code, mods| SelectGesture::from_key(&Key { code, mods }, 3);
        assert_eq!(
            k(KeyCode::Char(' '), KeyModifiers::NONE),
            Some(SelectGesture::Toggle)
        );
        assert_eq!(
            k(KeyCode::Down, KeyModifiers::SHIFT),
            Some(SelectGesture::Extend { from: 3 })
        );
        assert_eq!(k(KeyCode::Down, KeyModifiers::NONE), None, "a plain move");
        assert_eq!(
            k(KeyCode::Down, KeyModifiers::SHIFT | KeyModifiers::ALT),
            None
        );
        assert_eq!(k(KeyCode::Char('A'), KeyModifiers::SHIFT), None);
    }

    #[test]
    fn click_policies_fire_on_their_click() {
        // (activate, count, again) -> fires
        let table = [
            (Activate::Single, 1, false, true),
            (Activate::Double, 1, true, false),
            (Activate::Double, 2, true, true),
            (Activate::Double, 3, true, true),
            (Activate::ClickAgain, 1, false, false),
            (Activate::ClickAgain, 1, true, true),
            (Activate::Never, 2, true, false),
        ];
        for (a, count, again, want) in table {
            assert_eq!(
                a.fires(count, again),
                want,
                "{a:?} count {count} again {again}"
            );
        }
        let data = ClickPolicy::default();
        assert_eq!(data, ClickPolicy::DATA);
        assert!(!data.activates(1, true) && data.activates(2, true));
        assert!(!data.edits(1, true), "the focusing click never edits");
        assert!(data.toggles(true) && !data.toggles(false));
        assert!(ClickPolicy::NAVIGATION.activates(1, false));
        assert!(ClickPolicy::DATA.check(CheckClick::Row).toggles(false));
    }

    #[test]
    fn remap_follows_removed_rows() {
        let mut s = RowSelection::new();
        s.toggle(1, any);
        s.toggle(3, any);
        // row 2 deleted: 3 becomes 2, 1 stays
        s.remap(|r| match r {
            2 => None,
            r if r > 2 => Some(r - 1),
            r => Some(r),
        });
        assert_eq!(s.to_vec(), vec![1, 2]);
        assert_eq!(s.anchor(), Some(2));
    }
}
