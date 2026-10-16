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
//!   never selected).

use std::collections::BTreeSet;

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
        match self {
            TriState::None => "[ ]",
            TriState::Mixed => "[–]",
            TriState::All => "[✓]",
        }
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
