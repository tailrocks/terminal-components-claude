//! Row selection model (ported from PR #26 `src/core/selection.rs`,
//! commit f4867e9b5).
//!
//! One selection vocabulary shared by every list-like widget: modes,
//! gestures (click and key), tri-state check state and click policies.
//! The model never renders; widgets map rows through `enabled` closures so
//! disabled rows are never selected.

use std::collections::BTreeSet;

use crate::event::Key;
use crate::keys::{KeyCode, KeyModifiers};
use crate::platform::ClientPlatform;

/// How many rows a widget can have selected at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionMode {
    /// No selection.
    None,
    /// One row at a time.
    #[default]
    Single,
    /// Many rows, toggled and range-extended.
    Multi,
}

/// The three states of a select-all control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriState {
    /// Nothing selected.
    None,
    /// Some rows selected.
    Mixed,
    /// Every selectable row selected.
    All,
}

impl TriState {
    /// The ASCII glyph for the state.
    pub fn glyph(self) -> &'static str {
        self.glyph_with("[ ]", "[–]", "[✓]")
    }

    /// The glyph for the state from the given off / mixed / on glyphs.
    pub fn glyph_with(
        self,
        off: &'static str,
        mixed: &'static str,
        on: &'static str,
    ) -> &'static str {
        match self {
            TriState::None => off,
            TriState::Mixed => mixed,
            TriState::All => on,
        }
    }
}

/// A selection gesture, derived from click or key modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectGesture {
    /// Select only this row.
    Only,
    /// Flip this row's membership.
    Toggle,
    /// Select the display range between the anchor and this row.
    Extend {
        /// The display index the extension starts from.
        from: usize,
    },
    /// Select every selectable row.
    All,
}

impl SelectGesture {
    /// Toggle modifiers on non-macOS clients: Ctrl or Cmd.
    pub const TOGGLE_CTRL_CMD: KeyModifiers = KeyModifiers::CONTROL.union(KeyModifiers::SUPER);

    /// Toggle modifiers on macOS clients: Option or Cmd.
    pub const TOGGLE_OPTION_CMD: KeyModifiers = KeyModifiers::ALT.union(KeyModifiers::SUPER);

    /// Every modifier that can express a toggle.
    pub const TOGGLE_WITH_ALT: KeyModifiers = Self::TOGGLE_CTRL_CMD.union(KeyModifiers::ALT);

    /// The toggle modifiers for the client platform.
    pub const fn toggle_mods_for(platform: ClientPlatform) -> KeyModifiers {
        match platform {
            ClientPlatform::MacOs => Self::TOGGLE_OPTION_CMD,
            ClientPlatform::Other => Self::TOGGLE_CTRL_CMD,
        }
    }

    /// The toggle modifiers for the current client platform.
    pub fn toggle_mods() -> KeyModifiers {
        Self::toggle_mods_for(ClientPlatform::current())
    }

    /// Classify a click: Shift extends, toggle modifiers flip, plain selects only.
    pub fn from_mods(mods: KeyModifiers, toggle: KeyModifiers, from: usize) -> Self {
        if mods.contains(KeyModifiers::SHIFT) {
            Self::Extend { from }
        } else if mods.intersects(toggle) {
            Self::Toggle
        } else {
            Self::Only
        }
    }

    /// Classify a key press: Space toggles, Shift plus a movement key extends.
    pub fn from_key(key: &Key, from: usize) -> Option<Self> {
        let other = key.ctrl() || key.mods.contains(KeyModifiers::SUPER) || key.alt();
        match key.code {
            KeyCode::Char(' ') if key.mods.is_empty() => Some(Self::Toggle),
            KeyCode::Char(_) => None,
            _ if key.shift() && !other => Some(Self::Extend { from }),
            _ => None,
        }
    }
}

/// When a click activates (opens) a row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Activate {
    /// Every click activates.
    Single,
    /// A double click activates.
    #[default]
    Double,
    /// The second click on an already-selected row activates.
    ClickAgain,
    /// Clicks never activate.
    Never,
}

impl Activate {
    /// Whether a click with `count` in the press, `again` on a selected row, fires.
    pub fn fires(self, count: u8, again: bool) -> bool {
        match self {
            Self::Single => true,
            Self::Double => count >= 2,
            Self::ClickAgain => again,
            Self::Never => false,
        }
    }
}

/// Where a click may toggle the checkbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CheckClick {
    /// Only clicks on the checkbox glyph itself toggle.
    #[default]
    BoxOnly,
    /// A click anywhere in the row toggles.
    Row,
}

/// How clicks activate, edit and toggle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClickPolicy {
    /// When a click activates the row.
    pub activate: Activate,
    /// When a click starts editing the row.
    pub edit_on: Activate,
    /// Where a click may toggle the checkbox.
    pub check: CheckClick,
}

impl ClickPolicy {
    /// Data-grid default: double click activates and edits.
    pub const DATA: Self = Self {
        activate: Activate::Double,
        edit_on: Activate::Double,
        check: CheckClick::BoxOnly,
    };

    /// Navigation default: single click activates, never edits.
    pub const NAVIGATION: Self = Self {
        activate: Activate::Single,
        edit_on: Activate::Never,
        check: CheckClick::BoxOnly,
    };

    /// Set when a click activates.
    pub const fn activate(mut self, activate: Activate) -> Self {
        self.activate = activate;
        self
    }

    /// Set when a click edits.
    pub const fn edit_on(mut self, edit_on: Activate) -> Self {
        self.edit_on = edit_on;
        self
    }

    /// Set where a click toggles the checkbox.
    pub const fn check(mut self, check: CheckClick) -> Self {
        self.check = check;
        self
    }

    /// Whether this click activates.
    pub fn activates(&self, count: u8, again: bool) -> bool {
        self.activate.fires(count, again)
    }

    /// Whether this click starts editing.
    pub fn edits(&self, count: u8, again: bool) -> bool {
        self.edit_on.fires(count, again)
    }

    /// Whether this click toggles the checkbox.
    pub fn toggles(&self, on_box: bool) -> bool {
        on_box || self.check == CheckClick::Row
    }
}

impl Default for ClickPolicy {
    fn default() -> Self {
        Self::DATA
    }
}

/// A set of selected row source indices with an extend anchor.
///
/// Rows are addressed by their source (data) index; display order is
/// supplied per gesture through an `order` slice.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RowSelection {
    rows: BTreeSet<usize>,
    anchor: Option<usize>,
    /// Rows selected before the current extend drag started.
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

    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.rows.iter().copied()
    }

    pub fn to_vec(&self) -> Vec<usize> {
        self.iter().collect()
    }

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

    /// Clear every selection. Returns whether anything changed.
    pub fn clear(&mut self) -> bool {
        let changed = !self.rows.is_empty();
        self.rows.clear();
        self.set_anchor(None);
        changed
    }

    /// Select only `src`. A disabled row clears instead. Returns whether the
    /// selection changed.
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

    /// Flip `src`'s membership and anchor there. Disabled rows never toggle.
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

    /// Select the display range between the anchor and `to`, keeping rows
    /// selected before the drag. Disabled rows inside the range stay as they
    /// were.
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

    /// Add every selectable row to the selection.
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

    /// Select all, or if everything selectable is already selected, clear.
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

    /// The select-all state over the given rows.
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

    /// Apply a gesture at display index `to` over `order`.
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

    /// A context (right-click) selection: inside an existing selection it
    /// keeps the selection, outside it selects only that row.
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

    /// Set one row's membership directly (checkbox semantics, no anchor change).
    pub fn set(&mut self, src: usize, on: bool) -> bool {
        if on {
            self.rows.insert(src)
        } else {
            self.rows.remove(&src)
        }
    }

    /// Keep only rows passing `keep` (filter).
    pub fn retain(&mut self, keep: impl Fn(usize) -> bool) {
        self.rows.retain(|&s| keep(s));
        self.base.retain(|&s| keep(s));
        if self.anchor.is_some_and(|a| !keep(a)) {
            self.anchor = None;
        }
    }

    /// Remap rows through `map`, dropping the ones it maps to `None`
    /// (row removal / reordering).
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
        let order = [4, 2, 0, 3, 1];
        let mut s = RowSelection::new();
        s.select_only(2, any);
        s.extend(&order, 1, 3, any);
        assert_eq!(s.to_vec(), vec![0, 2, 3]);

        s.extend(&order, 3, 2, any);
        assert_eq!(s.to_vec(), vec![0, 2]);

        s.extend(&order, 2, 0, any);
        assert_eq!(s.to_vec(), vec![2, 4]);
    }

    #[test]
    fn extend_keeps_rows_selected_before_the_anchor() {
        let order = [0, 1, 2, 3, 4, 5];
        let mut s = RowSelection::new();
        s.toggle(0, any);
        s.toggle(3, any);
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

        let table: &[Row] = &[
            // Only: a plain click replaces the selection.
            (&[], None, Only, 1, &[2], Some(2)),
            (&[0, 1, 4], Some(4), Only, 3, &[0], Some(0)),
            // A disabled row clears and anchors nowhere.
            (&[0, 1], Some(0), Only, 2, &[], None),
            // Toggle flips the clicked row and anchors there.
            (&[0], Some(0), Toggle, 0, &[0, 4], Some(4)),
            (&[0, 4], Some(4), Toggle, 0, &[0], Some(4)),
            // Toggling a disabled row changes nothing.
            (&[0], Some(0), Toggle, 2, &[0], Some(0)),
            // Extend follows the anchor's display position.
            (&[2], Some(2), Extend { from: 1 }, 4, &[0, 1, 2], Some(2)),
            (&[2], Some(2), Extend { from: 4 }, 3, &[0, 2], Some(2)),
            (&[2], Some(2), Extend { from: 3 }, 0, &[2, 4], Some(2)),
            (&[1, 2], Some(2), Extend { from: 1 }, 0, &[1, 2, 4], Some(2)),
            (&[], None, Extend { from: 0 }, 1, &[2, 4], Some(4)),
            // All selects everything selectable, keeping the anchor.
            (&[2], Some(2), All, 0, &[0, 1, 2, 4], Some(2)),
            // Out of range is a no-op.
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

        s.remap(|r| match r {
            2 => None,
            r if r > 2 => Some(r - 1),
            r => Some(r),
        });
        assert_eq!(s.to_vec(), vec![1, 2]);
        assert_eq!(s.anchor(), Some(2));
    }
}
