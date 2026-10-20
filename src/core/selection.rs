use std::collections::BTreeSet;

use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::core::event::{ClientPlatform, Key};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionMode {
    None,

    #[default]
    Single,

    Multi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriState {
    None,

    Mixed,

    All,
}

impl TriState {
    pub fn glyph(self) -> &'static str {
        self.glyph_in(&crate::theme::Glyphs::UNICODE)
    }

    pub fn glyph_in(self, g: &crate::theme::Glyphs) -> &'static str {
        match self {
            TriState::None => g.check_off,
            TriState::Mixed => g.check_mixed,
            TriState::All => g.check_on,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectGesture {
    Only,

    Toggle,

    Extend { from: usize },

    All,
}

impl SelectGesture {
    pub const TOGGLE_CTRL_CMD: KeyModifiers = KeyModifiers::CONTROL.union(KeyModifiers::SUPER);

    pub const TOGGLE_OPTION_CMD: KeyModifiers = KeyModifiers::ALT.union(KeyModifiers::SUPER);

    pub const TOGGLE_WITH_ALT: KeyModifiers = Self::TOGGLE_CTRL_CMD.union(KeyModifiers::ALT);

    pub const fn toggle_mods_for(platform: ClientPlatform) -> KeyModifiers {
        match platform {
            ClientPlatform::MacOs => Self::TOGGLE_OPTION_CMD,
            ClientPlatform::Other => Self::TOGGLE_CTRL_CMD,
        }
    }

    pub fn toggle_mods() -> KeyModifiers {
        Self::toggle_mods_for(ClientPlatform::current())
    }

    pub fn from_mods(mods: KeyModifiers, toggle: KeyModifiers, from: usize) -> Self {
        if mods.contains(KeyModifiers::SHIFT) {
            Self::Extend { from }
        } else if mods.intersects(toggle) {
            Self::Toggle
        } else {
            Self::Only
        }
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Activate {
    Single,

    #[default]
    Double,

    ClickAgain,

    Never,
}

impl Activate {
    pub fn fires(self, count: u8, again: bool) -> bool {
        match self {
            Self::Single => true,
            Self::Double => count >= 2,
            Self::ClickAgain => again,
            Self::Never => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CheckClick {
    #[default]
    BoxOnly,

    Row,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClickPolicy {
    pub activate: Activate,

    pub edit_on: Activate,

    pub check: CheckClick,
}

impl ClickPolicy {
    pub const DATA: Self = Self {
        activate: Activate::Double,
        edit_on: Activate::Double,
        check: CheckClick::BoxOnly,
    };

    pub const NAVIGATION: Self = Self {
        activate: Activate::Single,
        edit_on: Activate::Never,
        check: CheckClick::BoxOnly,
    };

    pub const fn activate(mut self, activate: Activate) -> Self {
        self.activate = activate;
        self
    }

    pub const fn edit_on(mut self, edit_on: Activate) -> Self {
        self.edit_on = edit_on;
        self
    }

    pub const fn check(mut self, check: CheckClick) -> Self {
        self.check = check;
        self
    }

    pub fn activates(&self, count: u8, again: bool) -> bool {
        self.activate.fires(count, again)
    }

    pub fn edits(&self, count: u8, again: bool) -> bool {
        self.edit_on.fires(count, again)
    }

    pub fn toggles(&self, on_box: bool) -> bool {
        on_box || self.check == CheckClick::Row
    }
}

impl Default for ClickPolicy {
    fn default() -> Self {
        Self::DATA
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RowSelection {
    rows: BTreeSet<usize>,

    anchor: Option<usize>,

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

    pub fn clear(&mut self) -> bool {
        let changed = !self.rows.is_empty();
        self.rows.clear();
        self.set_anchor(None);
        changed
    }

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

    pub fn set(&mut self, src: usize, on: bool) -> bool {
        if on {
            self.rows.insert(src)
        } else {
            self.rows.remove(&src)
        }
    }

    pub fn retain(&mut self, keep: impl Fn(usize) -> bool) {
        self.rows.retain(|&s| keep(s));
        self.base.retain(|&s| keep(s));
        if self.anchor.is_some_and(|a| !keep(a)) {
            self.anchor = None;
        }
    }

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
            (&[], None, Only, 1, &[2], Some(2)),
            (&[0, 1, 4], Some(4), Only, 3, &[0], Some(0)),
            (&[0, 1], Some(0), Only, 2, &[], None),
            (&[0], Some(0), Toggle, 0, &[0, 4], Some(4)),
            (&[0, 4], Some(4), Toggle, 0, &[0], Some(4)),
            (&[0], Some(0), Toggle, 2, &[0], Some(0)),
            (&[2], Some(2), Extend { from: 1 }, 4, &[0, 1, 2], Some(2)),
            (&[2], Some(2), Extend { from: 4 }, 3, &[0, 2], Some(2)),
            (&[2], Some(2), Extend { from: 3 }, 0, &[2, 4], Some(2)),
            (&[1, 2], Some(2), Extend { from: 1 }, 0, &[1, 2, 4], Some(2)),
            (&[], None, Extend { from: 0 }, 1, &[2, 4], Some(4)),
            (&[2], Some(2), All, 0, &[0, 1, 2, 4], Some(2)),
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
