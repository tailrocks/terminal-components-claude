use ratatui::buffer::Buffer;
use std::collections::BTreeSet;

use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Modifier};

use crate::core::event::{ClientPlatform, Key, Outcome};
use crate::core::focus::BlurAction;
use crate::core::hit::PartMap;
use crate::core::id::{ItemKey, Part, PartRef, WidgetId};
use crate::core::keymap::{Actions, KeyBinding, KeyMap, Nav, NavKeys, lookup, select_all_bindings};
use crate::core::scroll::{HScroll, ScrollState, ScrollbarVisibility};
use crate::core::selection::{RowSelection, SelectionMode, TriState};
use crate::core::text::TextBuffer;
use crate::ui::columns::{ColumnSizing, HeaderAction, HeaderLabels};
use crate::ui::ctx::{RenderCtx, fill};
use crate::widgets::field_common::{EditAction, edit_key};
use crate::widgets::option_editor::{
    Affordance, ChoiceEditor, ChoiceResult, ColumnEditor, EditorConfig,
};
use crate::widgets::scrollbar;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDir {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct Column {
    pub title: String,
    pub width: Constraint,
    pub align: Align,
    pub editable: bool,
    pub sortable: bool,
    /// How an editable cell is edited: free text (default) or a value from
    /// a fixed set ([`ColumnEditor`]).
    pub editor: ColumnEditor,
}

impl Column {
    pub fn new(title: &str, width: Constraint) -> Self {
        Self {
            title: title.to_owned(),
            width,
            align: Align::Left,
            editable: false,
            sortable: true,
            editor: ColumnEditor::Text,
        }
    }
    /// Edit the column with `editor` (and make it editable).
    pub fn editor(mut self, editor: ColumnEditor) -> Self {
        self.editable = true;
        self.editor = editor;
        self
    }
    pub fn right(mut self) -> Self {
        self.align = Align::Right;
        self
    }
    pub fn editable(mut self) -> Self {
        self.editable = true;
        self
    }
    pub fn min_width(&self) -> u16 {
        match self.width {
            Constraint::Length(n) | Constraint::Min(n) => n,
            _ => 6,
        }
    }
}

/// A cell value with an optional per-cell error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub text: String,
    pub error: Option<String>,
    pub tone: Tone,
}

pub use crate::theme::Tone;

impl Cell {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            error: None,
            tone: Tone::Normal,
        }
    }
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
}

/// A constrained editor open on one cell ([`DataTable::choice`]).
#[derive(Debug, Clone)]
pub struct ChoiceEdit {
    /// Source row index.
    pub row: usize,
    pub col: usize,
    pub editor: ChoiceEditor,
}

#[derive(Debug, Clone)]
pub struct EditState {
    /// Source row index, independent of the current display sort order.
    pub row: usize,
    pub col: usize,
    pub buffer: TextBuffer,
    pub error: Option<String>,
}

/// Data table with header sorting, row/cell keyboard navigation, hover,
/// row selection and optional in-place cell editing.
///
/// Selection follows [`DataTable::selection_mode`]:
///
/// - [`SelectionMode::Single`] (default): Enter, Space or a click chooses
///   one row, marked `›` ([`DataTable::selected`]).
/// - [`SelectionMode::Multi`]: [`DataTable::selection`] holds any set of
///   rows, marked `✓` (or `[✓]` in the optional checkbox column). Space
///   toggles the cursor row, Shift+↑↓ (Home/End, PageUp/PageDown) extends a
///   range, Ctrl/Cmd+A selects all, Esc clears ([`DataTable::esc_clears`]).
///   A click selects only that row, Shift-click extends, a click with
///   [`DataTable::toggle_mods`] toggles: Option or Cmd when the client
///   terminal is on macOS, Ctrl or Cmd elsewhere
///   ([`DataTable::toggle_mods_for`]). The modified click is a shortcut,
///   not the only way: Space and a click on the row's checkbox
///   ([`DataTable::selection_column`]) toggle with no modifier, which
///   matters on macOS, where iTerm2 and Terminal.app may keep
///   Option+click for themselves (moving the cursor, block selection).
/// - [`SelectionMode::None`]: Enter activates, nothing stays marked.
///
/// Selection is keyed by source row, so it survives sorting and scrolling.
/// Rows in [`DataTable::disabled`] can be read but never selected.
/// A [`DataTable`] cell validator: `(col, text)` to `Some(message)` to
/// reject the value.
#[derive(Clone)]
pub struct CellValidator(pub std::rc::Rc<CellValidateFn>);

/// The function a [`CellValidator`] wraps.
pub type CellValidateFn = dyn Fn(usize, &str) -> Option<String>;

impl std::fmt::Debug for CellValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CellValidator")
    }
}

#[derive(Debug, Clone)]
pub struct DataTable {
    /// The scrollbar track as last drawn: presses and drags map through it.
    track: Rect,
    /// The rows, cells, checkboxes and `▾`s registered in the last render.
    parts: PartMap,
    pub id: WidgetId,
    pub columns: Vec<Column>,
    pub rows: Vec<Vec<Cell>>,
    /// Display order → source row index (sorting is a permutation, so edits
    /// land in the right row).
    order: Vec<usize>,
    pub sort: Option<(usize, SortDir)>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    /// When false the whole row is the navigation unit.
    pub cell_nav: bool,
    pub selected: Option<usize>,
    pub scroll: ScrollState,
    pub hscroll: ScrollState,
    pub edit: Option<EditState>,
    /// Checks a cell edit: `Some(message)` rejects the value. A closure,
    /// so it can capture state (an enum list, a schema).
    pub validator: Option<CellValidator>,
    /// What a cell edit in progress does when the table loses focus
    /// ([`Self::on_blur`]). Default [`BlurAction::Commit`].
    pub blur_action: BlurAction,
    pub empty_text: String,
    /// The theme's strings as of the last render: default text still
    /// reading them follows a change ([`crate::theme::Strings::follow`]).
    strings: crate::theme::Strings,
    /// A full empty state (title, hint, error tone) shown instead of
    /// [`Self::empty_text`] when set. Default `None`.
    pub empty: Option<crate::widgets::empty::EmptyState>,
    pub area: Rect,
    body: Rect,
    col_rects: Vec<Rect>,
    /// User column widths from dragging a header seam or autofit
    /// ([`ColumnSizing`]); `sizing.resizable` turns the seams off.
    pub sizing: ColumnSizing,
    /// Labels of the header context menu ([`Self::header_menu`]).
    pub header_labels: HeaderLabels,
    /// Numeric sort for these columns.
    pub numeric: Vec<bool>,
    /// How a cut cell reveals its full text on hover (see
    /// [`crate::ui::reveal`]); `None` follows [`RenderCtx::reveal`].
    pub reveal: Option<crate::ui::reveal::Reveal>,
    /// Horizontal scrolling policy; `None` follows [`RenderCtx::hscroll`].
    /// The table moves by whole columns and its bar counts columns.
    pub horizontal: Option<HScroll>,
    /// The `…` marks at the header's edges when columns are hidden on
    /// that side.
    pub overflow_marks: bool,
    /// The policy resolved at the last render (events use it).
    h_cfg: HScroll,
    /// The horizontal scrollbar track as last drawn.
    htrack: Rect,
    /// The last scrollbar press landed on the horizontal bar, so the drag
    /// that follows moves columns.
    h_held: bool,
    /// What a press on the bare vertical track does; `None` follows
    /// [`RenderCtx::track_click`](crate::ui::ctx::RenderCtx::track_click).
    pub track_click: Option<crate::core::scroll::TrackClick>,
    /// The vertical track policy resolved at the last render.
    v_click: crate::core::scroll::TrackClick,
    /// How many rows can be selected. Default [`SelectionMode::Single`].
    pub selection_mode: SelectionMode,
    /// The selected source rows in [`SelectionMode::Multi`].
    pub selection: RowSelection,
    /// Multi mode draws a checkbox column (`[✓]`) before the cells, with a
    /// tri-state "select all / none" box in the header. Default `false`:
    /// the `✓` mark sits in the gutter.
    pub selection_column: bool,
    /// Esc clears a multi selection (and is consumed) before it reaches
    /// the app. Default `true`, as in Finder and Mail.
    pub esc_clears: bool,
    /// Modifiers that make a click toggle one row in multi mode; Shift
    /// always extends. Default [`DataTable::toggle_mods_for`] the client
    /// terminal's platform ([`ClientPlatform::current`]):
    /// [`DataTable::TOGGLE_OPTION_CMD`] (Option or Cmd) on macOS, where
    /// Ctrl+click is a secondary click (`InputConfig::ctrl_click_secondary`)
    /// and many terminals do not report Cmd; [`DataTable::TOGGLE_CTRL_CMD`]
    /// (Ctrl or Cmd) elsewhere. [`DataTable::TOGGLE_WITH_ALT`] accepts all
    /// three. Option+click is a fallback (some macOS terminals keep it);
    /// Space and the checkbox column always toggle without a modifier.
    pub toggle_mods: KeyModifiers,
    /// Source rows that can be read but never selected or activated.
    pub disabled: BTreeSet<usize>,
    /// Which keys move the cursor (default [`NavKeys::DEFAULT`]).
    pub nav: NavKeys,
    /// The table's own action keys (default [`TableCmd::default_keymap`]).
    pub keymap: KeyMap<TableCmd>,
    /// Typing a prefix jumps to the first enabled row whose cell in
    /// [`Self::type_ahead_column`] starts with it (macOS table type-select).
    /// Off by default: plain letters keep their vim and action bindings.
    pub type_ahead: bool,
    /// The column type-ahead matches; `None` (default) is the cursor's
    /// column.
    pub type_ahead_column: Option<usize>,
    typeahead: crate::core::typeahead::TypeAhead,
    /// The constrained editor open on a cell, if any.
    pub choice: Option<ChoiceEdit>,
    /// Behaviour of every constrained column editor
    /// ([`EditorConfig::default`]: macOS popup conventions).
    pub editor_config: EditorConfig,
}

/// A [`DataTable`] action a key can be bound to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableCmd {
    /// Sort by the cursor column (again: reverse). Default `s`.
    SortByColumn,
    /// Multi mode: select every enabled row. Default Cmd+A where the
    /// terminal delivers Cmd, and Ctrl+A
    /// ([`crate::core::keymap::SELECT_ALL`]); single selection ignores it.
    SelectAll,
}

impl Actions for TableCmd {
    fn default_keymap() -> KeyMap<Self> {
        let mut map = vec![(KeyBinding::char('s'), TableCmd::SortByColumn)];
        map.extend(select_all_bindings(TableCmd::SelectAll));
        map
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableAction {
    Committed {
        row: usize,
        col: usize,
    },
    Cancelled,
    Activated(usize),
    /// The multi selection changed (keyboard, click or header checkbox).
    SelectionChanged,
    /// Editing ended via Tab at the last cell; caller may move focus.
    LeaveForward,
    LeaveBackward,
}

impl DataTable {
    pub fn new(id: WidgetId, columns: Vec<Column>, rows: Vec<Vec<Cell>>) -> Self {
        let n = rows.len();
        let numeric = vec![false; columns.len()];
        Self {
            track_click: None,
            v_click: crate::core::scroll::TrackClick::Page,
            id,
            columns,
            rows,
            order: (0..n).collect(),
            sort: None,
            cursor_row: 0,
            cursor_col: 0,
            cell_nav: false,
            selected: None,
            scroll: ScrollState::new(n),
            hscroll: ScrollState::default(),
            edit: None,
            validator: None,
            blur_action: BlurAction::Commit,
            empty_text: crate::theme::Strings::DEFAULT.no_rows.to_owned(),
            strings: crate::theme::Strings::DEFAULT,
            empty: None,
            area: Rect::ZERO,
            track: Rect::ZERO,
            parts: PartMap::default(),
            body: Rect::ZERO,
            col_rects: vec![],
            sizing: ColumnSizing::default(),
            header_labels: HeaderLabels::DEFAULT,
            numeric,
            reveal: None,
            horizontal: None,
            overflow_marks: true,
            h_cfg: HScroll::default(),
            htrack: Rect::ZERO,
            h_held: false,
            selection_mode: SelectionMode::Single,
            selection: RowSelection::new(),
            selection_column: false,
            esc_clears: true,
            toggle_mods: Self::toggle_mods_for(ClientPlatform::current()),
            disabled: BTreeSet::new(),
            nav: NavKeys::DEFAULT,
            keymap: TableCmd::default_keymap(),
            type_ahead: false,
            type_ahead_column: None,
            typeahead: Default::default(),
            choice: None,
            editor_config: EditorConfig::default(),
        }
    }

    /// See [`Self::type_ahead`].
    pub fn type_ahead(mut self, on: bool) -> Self {
        self.type_ahead = on;
        self
    }

    /// See [`Self::type_ahead_column`].
    pub fn type_ahead_column(mut self, col: Option<usize>) -> Self {
        self.type_ahead_column = col;
        self
    }

    /// See [`Self::editor_config`].
    pub fn editor_config(mut self, cfg: EditorConfig) -> Self {
        self.editor_config = cfg;
        self
    }

    /// The id of the popup a constrained editor opens.
    pub fn choice_id(&self) -> WidgetId {
        self.id.sub("choice")
    }

    /// The `▾` of a constrained cell (drawn on hover and on the cursor).
    /// The `▾` of a constrained cell: `Part::ACTIONS` of the row (as on
    /// `main`), one id per cell.
    pub fn open_id(&self, display: usize, col: usize) -> WidgetId {
        self.cell_id(display, col).part(PartRef::of(Part::ACTIONS))
    }

    /// The cell at (source row, column) holds a value outside its column's
    /// options (imported data).
    pub fn out_of_set(&self, src: usize, col: usize) -> bool {
        let (Some(row), Some(c)) = (self.rows.get(src), self.columns.get(col)) else {
            return false;
        };
        c.editor.out_of_set(&row[col].text, &self.editor_config)
    }

    /// Every (source row, column) holding a value outside its options.
    pub fn out_of_set_cells(&self) -> Vec<(usize, usize)> {
        (0..self.rows.len())
            .flat_map(|r| (0..self.columns.len()).map(move |c| (r, c)))
            .filter(|&(r, c)| self.out_of_set(r, c))
            .collect()
    }

    /// See [`Self::nav`].
    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.nav = nav;
        self
    }

    /// See [`Self::keymap`].
    pub fn keymap(mut self, keymap: KeyMap<TableCmd>) -> Self {
        self.keymap = keymap;
        self
    }

    /// How many rows can be selected (default [`SelectionMode::Single`]).
    pub fn selection_mode(mut self, mode: SelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    /// Draw the multi-selection checkbox column (default off).
    pub fn selection_column(mut self, on: bool) -> Self {
        self.selection_column = on;
        self
    }

    /// Whether Esc clears a multi selection (default on).
    pub fn esc_clears(mut self, on: bool) -> Self {
        self.esc_clears = on;
        self
    }

    /// Ctrl or Cmd toggles a row: the default [`Self::toggle_mods`] off
    /// macOS.
    pub const TOGGLE_CTRL_CMD: KeyModifiers = KeyModifiers::CONTROL.union(KeyModifiers::SUPER);
    /// Option (Alt) or Cmd toggles a row: the default [`Self::toggle_mods`]
    /// on macOS, where Ctrl+click is a secondary click and Cmd rarely
    /// reaches a terminal app. iTerm2 and Terminal.app can keep
    /// Option+click (cursor moves, block selection), so it is a fallback
    /// beside Space and the checkbox column, not the only way.
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

    /// Modifiers that make a click toggle a row (default
    /// [`Self::toggle_mods_for`] the client platform; pass
    /// `DataTable::toggle_mods_for(platform)` to follow another platform,
    /// or any set such as [`Self::TOGGLE_WITH_ALT`]).
    pub fn toggle_mods(mut self, mods: KeyModifiers) -> Self {
        self.toggle_mods = mods;
        self
    }

    /// Mark source rows as disabled: readable, never selectable.
    pub fn disabled_rows(mut self, rows: &[usize]) -> Self {
        self.disabled.extend(rows.iter().copied());
        self
    }

    /// Enable or disable one source row; disabling drops it from the
    /// selection.
    pub fn set_row_disabled(&mut self, src: usize, disabled: bool) {
        if disabled {
            self.disabled.insert(src);
            self.selection.retain(|r| r != src);
            if self.selected == Some(src) {
                self.selected = None;
            }
        } else {
            self.disabled.remove(&src);
        }
    }

    pub fn is_disabled(&self, src: usize) -> bool {
        self.disabled.contains(&src)
    }

    fn multi(&self) -> bool {
        self.selection_mode == SelectionMode::Multi
    }

    /// Selected source rows, ascending: the multi selection, or the chosen
    /// row in single mode.
    pub fn selected_rows(&self) -> Vec<usize> {
        match self.selection_mode {
            SelectionMode::Multi => self.selection.to_vec(),
            SelectionMode::Single => self.selected.into_iter().collect(),
            SelectionMode::None => vec![],
        }
    }

    /// Panel meta for the selection: `"3 selected"`, or empty when nothing
    /// is selected.
    pub fn selection_label(&self) -> String {
        match self.selected_rows().len() {
            0 => String::new(),
            n => format!("{n} selected"),
        }
    }

    /// Aggregate state of the multi selection (the header checkbox).
    pub fn selection_state(&self) -> TriState {
        let disabled = &self.disabled;
        self.selection
            .tri_state(0..self.rows.len(), |r| !disabled.contains(&r))
    }

    /// Select every enabled row (multi mode). Returns whether it changed.
    pub fn select_all(&mut self) -> bool {
        if !self.multi() {
            return false;
        }
        let disabled = &self.disabled;
        self.selection
            .select_all(self.order.iter().copied(), |r| !disabled.contains(&r))
    }

    /// Clear the selection (both modes). Returns whether it changed.
    pub fn clear_selection(&mut self) -> bool {
        let single = self.selected.take().is_some();
        self.selection.clear() || single
    }

    /// Toggle one source row in the multi selection.
    pub fn toggle_row(&mut self, src: usize) -> bool {
        let disabled = &self.disabled;
        src < self.rows.len() && self.selection.toggle(src, |r| !disabled.contains(&r))
    }

    /// Replace the cells of source row `src` (an edit made elsewhere, such
    /// as an inspector form), keeping the selection, re-applying the sort
    /// and keeping the cursor on the row it was on.
    pub fn set_row(&mut self, src: usize, cells: Vec<Cell>) {
        if src >= self.rows.len() {
            return;
        }
        let cursor_src = self.order.get(self.cursor_row).copied();
        self.rows[src] = cells;
        if let Some((c, d)) = self.sort {
            self.apply_sort(c, d);
            if let Some(pos) = cursor_src.and_then(|s| self.order.iter().position(|&r| r == s)) {
                self.cursor_row = pos;
                self.scroll.ensure_visible(pos);
            }
        }
    }

    /// Remove source rows from the dataset, keeping the selection, the
    /// disabled set, the sort and the cursor on the rows that remain.
    pub fn remove_rows(&mut self, srcs: &[usize]) {
        let gone: BTreeSet<usize> = srcs.iter().copied().collect();
        let map = |r: usize| (!gone.contains(&r)).then(|| r - gone.range(..r).count());
        let cursor_src = self.order.get(self.cursor_row).copied();
        let len = self.rows.len();
        self.edit = None;
        self.selection.remap(map);
        self.disabled = self.disabled.iter().filter_map(|&r| map(r)).collect();
        self.selected = self.selected.and_then(map);
        let mut i = 0;
        self.rows.retain(|_| {
            let keep = !gone.contains(&i);
            i += 1;
            keep
        });
        self.order = (0..self.rows.len()).collect();
        self.scroll.set_content(self.rows.len());
        if let Some((c, d)) = self.sort {
            self.apply_sort(c, d);
        }
        // the cursor stays on its row, or on the next one that remains
        let next = cursor_src.and_then(|c| {
            (c..len)
                .find_map(map)
                .or_else(|| (0..c).rev().find_map(map))
        });
        self.cursor_row = next
            .and_then(|s| self.order.iter().position(|&r| r == s))
            .unwrap_or(0)
            .min(self.rows.len().saturating_sub(1));
        self.scroll.ensure_visible(self.cursor_row);
    }

    /// Id of the selection checkbox on display row `display`.
    pub fn check_id(&self, display: usize) -> WidgetId {
        self.id.part(Self::check_part(display))
    }

    fn check_part(display: usize) -> PartRef {
        PartRef::item(Part::MARKER, ItemKey::index(display))
    }

    /// Id of the header's "select all / none" checkbox.
    pub fn check_all_id(&self) -> WidgetId {
        self.id.part(PartRef::of(Part::MARKER))
    }

    /// The display row whose checkbox `id` is.
    pub fn locate_check(&self, id: WidgetId) -> Option<usize> {
        self.parts.index(id, Part::MARKER)
    }

    /// A click on a row's checkbox: toggles that row and moves the cursor
    /// there.
    pub fn on_click_check(&mut self, display: usize) -> (Outcome, Option<TableAction>) {
        if display >= self.rows.len() || !self.multi() {
            return (Outcome::Consumed, None);
        }
        self.set_cursor(display, self.cursor_col);
        let src = self.order[display];
        if self.toggle_row(src) {
            (Outcome::Changed, Some(TableAction::SelectionChanged))
        } else {
            (Outcome::Changed, None)
        }
    }

    /// A click on the header checkbox: all when not everything is
    /// selected, none otherwise.
    pub fn on_click_check_all(&mut self) -> (Outcome, Option<TableAction>) {
        if !self.multi() {
            return (Outcome::Consumed, None);
        }
        let disabled = &self.disabled;
        if self
            .selection
            .toggle_all(self.order.iter().copied(), |r| !disabled.contains(&r))
        {
            (Outcome::Changed, Some(TableAction::SelectionChanged))
        } else {
            (Outcome::Consumed, None)
        }
    }

    /// Multi-mode keys: Space, Shift+movement, Esc ([`TableCmd::SelectAll`]
    /// is a key-map action).
    fn on_key_multi(&mut self, key: &Key) -> Option<(Outcome, Option<TableAction>)> {
        let changed = |c: bool| {
            if c {
                (Outcome::Changed, Some(TableAction::SelectionChanged))
            } else {
                (Outcome::Consumed, None)
            }
        };
        let command = key.ctrl() || key.cmd();
        match key.code {
            KeyCode::Char(' ') if key.mods.is_empty() => {
                let src = self.order[self.cursor_row];
                Some(changed(self.toggle_row(src)))
            }
            KeyCode::Esc
                if key.mods.is_empty() && self.esc_clears && !self.selection.is_empty() =>
            {
                self.selection.clear();
                Some((Outcome::Changed, Some(TableAction::SelectionChanged)))
            }
            _ if key.shift()
                && !command
                && !key.alt()
                && matches!(
                    self.nav.action(key),
                    Some(Nav::Up | Nav::Down | Nav::Home | Nav::End | Nav::PageUp | Nav::PageDown)
                ) =>
            {
                let from = self.cursor_row;
                let page = self.scroll.viewport_len.max(1);
                let to = match self.nav.action(key) {
                    Some(Nav::Up) => from.saturating_sub(1),
                    Some(Nav::Down) => from + 1,
                    Some(Nav::Home) => 0,
                    Some(Nav::End) => usize::MAX,
                    Some(Nav::PageUp) => from.saturating_sub(page),
                    _ => from + page,
                };
                self.set_cursor(to, self.cursor_col);
                let disabled = &self.disabled;
                let c = self
                    .selection
                    .extend(&self.order, from, self.cursor_row, |r| {
                        !disabled.contains(&r)
                    });
                Some((Outcome::Changed, c.then_some(TableAction::SelectionChanged)))
            }
            _ => None,
        }
    }

    /// Override the app-wide horizontal scrolling policy.
    pub fn horizontal(mut self, policy: HScroll) -> Self {
        self.horizontal = Some(policy);
        self
    }

    /// Show or hide the `…` hidden-column marks in the header.
    pub fn overflow_marks(mut self, on: bool) -> Self {
        self.overflow_marks = on;
        self
    }

    /// Header seams resize columns by drag (double-click autofits).
    /// Default on.
    pub fn resizable_columns(mut self, on: bool) -> Self {
        self.sizing.resizable = on;
        self
    }

    /// Hit width of a header seam in cells (default
    /// [`crate::ui::columns::DEFAULT_SEAM_W`]).
    pub fn seam_hit_w(mut self, cells: u16) -> Self {
        self.sizing.seam_w = cells.max(1);
        self
    }

    /// The resize seam after column `c` in the header.
    pub fn seam_id(&self, c: usize) -> WidgetId {
        self.id.part(crate::ui::columns::seam_part(c))
    }

    pub fn locate_seam(&self, id: WidgetId) -> Option<usize> {
        self.id.item_of(id, Part::SEAM, 0..self.columns.len())
    }

    /// The column whose header menu a secondary click on `id` opens (its
    /// header or its seam).
    pub fn locate_header_menu(&self, id: WidgetId) -> Option<usize> {
        self.locate_header(id).or_else(|| self.locate_seam(id))
    }

    /// Button down on `id` with the click count it starts: a press on a
    /// header seam starts a resize (keeping the grab offset); the second
    /// press of a double click fits the column to its content.
    pub fn on_press(&mut self, id: WidgetId, pos: Position, count: u8) -> Outcome {
        let Some(c) = self.locate_seam(id) else {
            return Outcome::Ignored;
        };
        if !self.sizing.resizable {
            return Outcome::Consumed;
        }
        if count >= 2 {
            self.sizing.release();
            self.autofit(c);
            return Outcome::Changed;
        }
        let w = self.col_rects.get(c).map_or(0, |r| r.width);
        self.sizing.press(c, w, pos.x);
        Outcome::Changed
    }

    /// A drag held since a press on `pressed`: a header seam resizes its
    /// column cell by cell, a scrollbar scrolls.
    pub fn on_drag(&mut self, pressed: WidgetId, pos: Position) -> Outcome {
        if self.locate_seam(pressed).is_some() {
            return match self.sizing.drag_to(pos.x) {
                Some(_) => Outcome::Changed,
                None => Outcome::Consumed,
            };
        }
        if scrollbar::is_bar_of(pressed, self.id) {
            return self.on_bar_drag(pressed, pos);
        }
        Outcome::Ignored
    }

    /// Fit column `c` to its title and the text of the visible rows (no
    /// wider than the table), as a user width.
    pub fn autofit(&mut self, c: usize) {
        let Some(col) = self.columns.get(c) else {
            return;
        };
        let cap = (self.area.width.saturating_sub(4) as usize).max(6);
        let mut need = crate::ui::text::width(&col.title)
            + if self.sort.is_some_and(|(sc, _)| sc == c) {
                2
            } else {
                0
            };
        for d in self.scroll.visible_range() {
            if let Some(cell) = self.order.get(d).and_then(|&r| self.rows[r].get(c)) {
                need = need.max(crate::ui::text::width_capped(&cell.text, cap));
            }
        }
        self.sizing.set(c, need.min(cap) as u16);
    }

    /// The header context menu of column `c`: sort, autofit and reset
    /// widths, each paired with the action [`Self::on_header_action`] runs.
    pub fn header_menu(&self, c: usize) -> Vec<(HeaderAction, crate::widgets::menu::MenuItem)> {
        let sortable = self.columns.get(c).is_some_and(|col| col.sortable);
        let sorted = self.sort.and_then(|(sc, d)| (sc == c).then_some(d));
        crate::ui::columns::header_menu(
            &self.header_labels,
            sortable,
            sorted,
            self.sizing.resizable,
            !self.sizing.is_default(),
        )
    }

    /// Run a header menu command on column `c`.
    pub fn on_header_action(&mut self, c: usize, action: HeaderAction) -> Outcome {
        let want = match action {
            HeaderAction::SortAsc => Some(Some((c, SortDir::Asc))),
            HeaderAction::SortDesc => Some(Some((c, SortDir::Desc))),
            HeaderAction::ClearSort => Some(None),
            HeaderAction::Autofit => {
                self.autofit(c);
                return Outcome::Changed;
            }
            HeaderAction::ResetWidths => {
                self.sizing.reset();
                return Outcome::Changed;
            }
        };
        if let Some(want) = want {
            if c >= self.columns.len() || !self.columns[c].sortable {
                return Outcome::Consumed;
            }
            // sort_by cycles asc, desc, none: step until it lands on `want`
            for _ in 0..3 {
                if self.sort == want {
                    break;
                }
                self.sort_by(c);
            }
        }
        Outcome::Changed
    }

    /// How cut cells reveal their full text (default: [`RenderCtx::reveal`]).
    pub fn reveal(mut self, config: crate::ui::reveal::Reveal) -> Self {
        self.reveal = Some(config);
        self
    }

    pub fn cell_nav(mut self, on: bool) -> Self {
        self.cell_nav = on;
        self
    }
    pub fn numeric(mut self, cols: &[usize]) -> Self {
        for &c in cols {
            if c < self.numeric.len() {
                self.numeric[c] = true;
            }
        }
        self
    }
    /// See [`Self::validator`]: `v(col, text)`.
    pub fn validator(mut self, v: impl Fn(usize, &str) -> Option<String> + 'static) -> Self {
        self.validator = Some(CellValidator(std::rc::Rc::new(v)));
        self
    }
    /// See [`Self::empty`].
    pub fn empty(mut self, state: crate::widgets::empty::EmptyState) -> Self {
        self.empty = Some(state);
        self
    }

    pub fn empty_text(mut self, s: &str) -> Self {
        self.empty_text = s.to_owned();
        self
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
    pub fn is_editing(&self) -> bool {
        self.edit.is_some() || self.choice.is_some()
    }

    /// Source index of the row at display position `i`.
    pub fn source_row(&self, i: usize) -> usize {
        self.order[i]
    }

    pub fn header_id(&self, col: usize) -> WidgetId {
        self.id
            .part(PartRef::item(Part::HEADER, ItemKey::index(col)))
    }
    pub fn row_id(&self, display: usize) -> WidgetId {
        self.id.part(Self::row_part(display))
    }
    fn row_part(display: usize) -> PartRef {
        PartRef::item(Part::ROW, ItemKey::index(display))
    }
    /// One id per cell, under its row's id; the part is the row's
    /// `Part::CELL` (the column comes from the position on `main`).
    pub fn cell_id(&self, display: usize, col: usize) -> WidgetId {
        self.row_id(display).child(col)
    }
    fn cell_part(display: usize) -> PartRef {
        PartRef::item(Part::CELL, ItemKey::index(display))
    }

    /// Replace the dataset, cancelling any edit and clearing row selection.
    /// Keeps column configuration and sorting; clamps the navigation cursor.
    pub fn set_rows(&mut self, rows: Vec<Vec<Cell>>) {
        self.edit = None;
        self.choice = None;
        self.selected = None;
        self.selection.clear();
        self.disabled.retain(|&r| r < rows.len());
        self.rows = rows;
        self.order = (0..self.rows.len()).collect();
        self.scroll.set_content(self.rows.len());
        if let Some((c, d)) = self.sort {
            self.apply_sort(c, d);
        }
        self.cursor_row = self.cursor_row.min(self.rows.len().saturating_sub(1));
    }

    fn apply_sort(&mut self, col: usize, dir: SortDir) {
        let numeric = self.numeric.get(col).copied().unwrap_or(false);
        let rows = &self.rows;
        self.order.sort_by(|&a, &b| {
            let ca = &rows[a][col].text;
            let cb = &rows[b][col].text;
            let ord = if numeric {
                // non-numeric cells (e.g. "—") sort before every number
                parse_num(ca).cmp(&parse_num(cb))
            } else {
                ca.to_lowercase().cmp(&cb.to_lowercase())
            };
            let ord = ord.then_with(|| a.cmp(&b));
            match dir {
                SortDir::Asc => ord,
                SortDir::Desc => ord.reverse(),
            }
        });
    }

    /// Cycle sort on a column: asc → desc → none.
    pub fn sort_by(&mut self, col: usize) -> Outcome {
        if col >= self.columns.len() || !self.columns[col].sortable {
            return Outcome::Consumed;
        }
        // keep the cursor on the same source row
        let src = self.order.get(self.cursor_row).copied();
        self.sort = match self.sort {
            Some((c, SortDir::Asc)) if c == col => Some((col, SortDir::Desc)),
            Some((c, SortDir::Desc)) if c == col => None,
            _ => Some((col, SortDir::Asc)),
        };
        match self.sort {
            Some((c, d)) => self.apply_sort(c, d),
            None => self.order = (0..self.rows.len()).collect(),
        }
        if let Some(src) = src
            && let Some(pos) = self.order.iter().position(|&r| r == src)
        {
            self.cursor_row = pos;
            self.scroll.ensure_visible(pos);
        }
        Outcome::Changed
    }

    fn set_cursor(&mut self, row: usize, col: usize) {
        self.cursor_row = row.min(self.rows.len().saturating_sub(1));
        self.cursor_col = col.min(self.columns.len().saturating_sub(1));
        self.scroll.ensure_visible(self.cursor_row);
        self.ensure_col_visible();
    }

    fn ensure_col_visible(&mut self) {
        // horizontal scroll is in columns
        let n = self.columns.len();
        if self.hscroll.viewport_len == 0 {
            return;
        }
        if self.cursor_col < self.hscroll.offset {
            self.hscroll.offset = self.cursor_col;
        } else if self.cursor_col >= self.hscroll.offset + self.hscroll.viewport_len {
            self.hscroll.offset = self.cursor_col + 1 - self.hscroll.viewport_len;
        }
        self.hscroll.offset = self
            .hscroll
            .offset
            .min(n.saturating_sub(self.hscroll.viewport_len));
    }

    pub fn begin_edit(&mut self) -> Outcome {
        self.start_edit().0
    }

    /// Start editing the cursor cell: a text editor, a constrained
    /// editor's popup, or (Checkbox) the flipped value committed at once.
    pub fn start_edit(&mut self) -> (Outcome, Option<TableAction>) {
        if self.rows.is_empty() {
            return (Outcome::Consumed, None);
        }
        let col = self.cursor_col;
        if !self.columns[col].editable {
            return (Outcome::Consumed, None);
        }
        let src = self.order[self.cursor_row];
        let text = self.rows[src][col].text.clone();
        let editor = &self.columns[col].editor;
        if let Some(next) = editor.toggled(&text) {
            return (Outcome::Changed, self.apply_value(src, col, next).ok());
        }
        if editor.is_constrained() {
            let opened = ChoiceEditor::open(self.choice_id(), editor, &text, &self.editor_config);
            self.choice = opened.map(|editor| ChoiceEdit {
                row: src,
                col,
                editor,
            });
            return (Outcome::Changed, None);
        }
        self.edit = Some(EditState {
            row: src,
            col,
            buffer: TextBuffer::single(text),
            error: None,
        });
        (Outcome::Changed, None)
    }

    /// Store a chosen value through the edit pipeline: the validator may
    /// reject it (the message comes back), a sorted column re-sorts.
    fn apply_value(&mut self, src: usize, col: usize, text: String) -> Result<TableAction, String> {
        if let Some(err) = self.validator.as_ref().and_then(|v| (v.0)(col, &text)) {
            return Err(err);
        }
        self.rows[src][col].text = text;
        self.rows[src][col].error = None;
        if let Some((c, d)) = self.sort
            && c == col
        {
            self.apply_sort(c, d);
            if let Some(pos) = self.order.iter().position(|&r| r == src) {
                self.cursor_row = pos;
                self.scroll.ensure_visible(pos);
            }
        }
        Ok(TableAction::Committed { row: src, col })
    }

    /// Resolve what an open constrained editor reported.
    fn on_choice(&mut self, r: Option<ChoiceResult>) -> Option<TableAction> {
        let (value, tab) = match r? {
            ChoiceResult::Cancel => {
                self.choice = None;
                return Some(TableAction::Cancelled);
            }
            ChoiceResult::Commit(v) => (v, None),
            ChoiceResult::CommitAndTab { value, backward } => (value, Some(backward)),
        };
        let (src, col) = self.choice.as_ref().map(|c| (c.row, c.col))?;
        match self.apply_value(src, col, value) {
            Ok(ev) => {
                self.choice = None;
                match tab {
                    Some(backward) => match self.tab_to_editable(backward) {
                        Some(leave) => Some(leave),
                        None => {
                            self.start_edit();
                            Some(ev)
                        }
                    },
                    None => Some(ev),
                }
            }
            Err(msg) => {
                if let Some(c) = self.choice.as_mut() {
                    c.editor.error = Some(msg);
                }
                None
            }
        }
    }

    /// Move the cell cursor to the next (previous) editable column of the
    /// row; `Some(Leave…)` when there is none.
    fn tab_to_editable(&mut self, backward: bool) -> Option<TableAction> {
        let n = self.columns.len();
        let mut col = self.cursor_col;
        loop {
            if backward {
                if col == 0 {
                    return Some(TableAction::LeaveBackward);
                }
                col -= 1;
            } else {
                col += 1;
                if col >= n {
                    return Some(TableAction::LeaveForward);
                }
            }
            if self.columns[col].editable {
                break;
            }
        }
        self.set_cursor(self.cursor_row, col);
        None
    }

    /// A click on an open constrained editor (an option, the popup, the
    /// backdrop) or on a cell's `▾`; `None` when `id` is neither, so the
    /// owner routes it on.
    pub fn on_click_editor(&mut self, id: WidgetId) -> Option<(Outcome, Option<TableAction>)> {
        if let Some(c) = self.choice.as_mut()
            && c.editor.owns(id)
        {
            let (out, r) = c.editor.on_click(id, &self.editor_config);
            return Some((out, self.on_choice(r)));
        }
        let (di, ci) = self.parts.cell(id, Part::ACTIONS)?;
        self.set_cursor(di, ci);
        Some(self.start_edit())
    }

    pub fn commit_edit(&mut self) -> Option<TableAction> {
        let e = self.edit.take()?;
        let src = e.row;
        let text = e.buffer.text().to_owned();
        let err = self.validator.as_ref().and_then(|v| (v.0)(e.col, &text));
        if let Some(err) = err {
            // keep editing, show error
            self.edit = Some(EditState {
                error: Some(err),
                ..e
            });
            return None;
        }
        self.rows[src][e.col].text = text;
        self.rows[src][e.col].error = None;
        if let Some((c, d)) = self.sort
            && c == e.col
        {
            self.apply_sort(c, d);
            if let Some(pos) = self.order.iter().position(|&r| r == src) {
                self.cursor_row = pos;
                self.scroll.ensure_visible(pos);
            }
        }
        Some(TableAction::Committed {
            row: src,
            col: e.col,
        })
    }

    pub fn cancel_edit(&mut self) -> Option<TableAction> {
        if self.choice.take().is_some() {
            return Some(TableAction::Cancelled);
        }
        self.edit.take().map(|_| TableAction::Cancelled)
    }

    /// Focus left the table (see [`crate::core::focus::Focus::take_changes`]).
    /// A cell edit in progress ends per [`Self::blur_action`] and reports
    /// `Committed` or `Cancelled` like Enter or Esc (an invalid value stays
    /// open with its error and reports nothing).
    pub fn on_blur(&mut self) -> Option<TableAction> {
        if let Some(c) = &self.choice {
            return match self.editor_config.blur {
                BlurAction::Commit => match c.editor.blur_value(&self.editor_config) {
                    Some(v) => self.on_choice(Some(ChoiceResult::Commit(v))),
                    None => self.on_choice(Some(ChoiceResult::Cancel)),
                },
                BlurAction::Cancel => self.on_choice(Some(ChoiceResult::Cancel)),
                BlurAction::Keep => None,
            };
        }
        self.edit.as_ref()?;
        match self.blur_action {
            BlurAction::Commit => self.commit_edit(),
            BlurAction::Cancel => self.cancel_edit(),
            BlurAction::Keep => None,
        }
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<TableAction>) {
        if let Some(c) = self.choice.as_mut() {
            let (out, r) = c.editor.on_key(key, &self.editor_config);
            return (out, self.on_choice(r));
        }
        if let Some(e) = self.edit.as_mut() {
            return match edit_key(key, false) {
                EditAction::Commit => match self.commit_edit() {
                    Some(ev) => (Outcome::Changed, Some(ev)),
                    None => (Outcome::Changed, None),
                },
                EditAction::Cancel => (Outcome::Changed, self.cancel_edit()),
                EditAction::Tab { backward } => {
                    let ev = self.commit_edit();
                    if ev.is_none() {
                        return (Outcome::Changed, None);
                    }
                    // move to next editable cell
                    if let Some(leave) = self.tab_to_editable(backward) {
                        return (Outcome::Changed, Some(leave));
                    }
                    self.start_edit();
                    (Outcome::Changed, ev)
                }
                EditAction::Apply(f) => {
                    f(&mut e.buffer);
                    (Outcome::Changed, None)
                }
                action @ (EditAction::Undo
                | EditAction::Redo
                | EditAction::Copy
                | EditAction::Cut
                | EditAction::Paste
                | EditAction::SelectAll) => {
                    // a cell edit is short-lived: clipboard keys work, its
                    // undo is Esc
                    crate::widgets::field_common::apply_edit(
                        &mut e.buffer,
                        &mut crate::core::edit::UndoStack::new(),
                        action,
                        key.code,
                        false,
                    );
                    (Outcome::Changed, None)
                }
                EditAction::Insert(c) => {
                    e.buffer.insert_char(c);
                    if e.error.is_some() {
                        e.error = self
                            .validator
                            .as_ref()
                            .and_then(|v| (v.0)(e.col, e.buffer.text()));
                    }
                    (Outcome::Changed, None)
                }
                EditAction::None => (crate::widgets::field_common::unbound_key(key), None),
            };
        }
        if self.rows.is_empty() {
            return (Outcome::Ignored, None);
        }
        if let KeyCode::Char(c) = key.code
            && self.type_ahead
            && !c.is_whitespace()
            && !key.chord()
        {
            let col = self.type_ahead_column.unwrap_or(self.cursor_col);
            let prefix = self.typeahead.feed(c).to_owned();
            let found = crate::core::typeahead::TypeAhead::find(
                &prefix,
                self.order.iter().map(|&src| {
                    self.rows[src]
                        .get(col)
                        .map_or("", |cell| cell.text.as_str())
                }),
                self.cursor_row,
                |d| !self.is_disabled(self.order[d]),
            );
            return match found {
                Some(d) => {
                    self.set_cursor(d, self.cursor_col);
                    (Outcome::Changed, None)
                }
                None => (Outcome::Consumed, None),
            };
        }
        if self.multi()
            && let Some(r) = self.on_key_multi(key)
        {
            return r;
        }
        if matches!(key.code, KeyCode::Char(_))
            && key.chord()
            && lookup(&self.keymap, key).is_none()
        {
            // an unassigned modified chord never performs a plain action
            return (Outcome::Ignored, None);
        }
        let (r, c) = (self.cursor_row, self.cursor_col);
        if let Some(action) = lookup(&self.keymap, key) {
            return match action {
                TableCmd::SortByColumn => (self.sort_by(c), None),
                TableCmd::SelectAll if self.multi() => {
                    if self.select_all() {
                        (Outcome::Changed, Some(TableAction::SelectionChanged))
                    } else {
                        (Outcome::Consumed, None)
                    }
                }
                // single selection: nothing to select all of
                TableCmd::SelectAll => (Outcome::Ignored, None),
            };
        }
        // Alt+↓ opens a constrained cell's popup; plain ↓ stays row
        // navigation (APG grid)
        if key.code == KeyCode::Down
            && key.alt()
            && self.cell_nav
            && self.columns[c].editable
            && self.columns[c].editor.has_popup()
        {
            return self.start_edit();
        }
        if let Some(nav) = self.nav.action(key) {
            return self.on_nav(nav);
        }
        match key.code {
            KeyCode::Enter | KeyCode::F(2) if self.cell_nav && self.columns[c].editable => {
                self.start_edit()
            }
            // Space opens a constrained cell (and flips a checkbox)
            KeyCode::Char(' ')
                if self.cell_nav
                    && self.columns[c].editable
                    && self.columns[c].editor.is_constrained() =>
            {
                self.start_edit()
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let src = self.order[r];
                if self.is_disabled(src) {
                    return (Outcome::Consumed, None);
                }
                if self.selection_mode == SelectionMode::Single {
                    self.selected = Some(src);
                }
                (Outcome::Changed, Some(TableAction::Activated(src)))
            }
            _ => (Outcome::Ignored, None),
        }
    }

    fn on_nav(&mut self, nav: Nav) -> (Outcome, Option<TableAction>) {
        let (r, c) = (self.cursor_row, self.cursor_col);
        match nav {
            Nav::Up => {
                self.set_cursor(r.saturating_sub(1), c);
                (Outcome::Changed, None)
            }
            Nav::Down => {
                self.set_cursor(r + 1, c);
                (Outcome::Changed, None)
            }
            Nav::Left if self.cell_nav => {
                self.set_cursor(r, c.saturating_sub(1));
                (Outcome::Changed, None)
            }
            Nav::Right if self.cell_nav => {
                self.set_cursor(r, c + 1);
                (Outcome::Changed, None)
            }
            Nav::Left => {
                self.hscroll.scroll_by(-1);
                (Outcome::Changed, None)
            }
            Nav::Right => {
                self.hscroll.scroll_by(1);
                (Outcome::Changed, None)
            }
            Nav::PageUp => {
                self.set_cursor(r.saturating_sub(self.scroll.viewport_len.max(1)), c);
                (Outcome::Changed, None)
            }
            Nav::PageDown => {
                self.set_cursor(r + self.scroll.viewport_len.max(1), c);
                (Outcome::Changed, None)
            }
            Nav::Home => {
                self.set_cursor(0, c);
                (Outcome::Changed, None)
            }
            Nav::End => {
                self.set_cursor(usize::MAX, c);
                (Outcome::Changed, None)
            }
        }
    }

    pub fn on_paste(&mut self, text: &str) -> Outcome {
        if let Some(c) = self.choice.as_mut() {
            return c.editor.on_paste(text);
        }
        match self.edit.as_mut() {
            Some(e) => {
                e.buffer.insert_str(text);
                Outcome::Changed
            }
            None => Outcome::Ignored,
        }
    }

    pub fn on_click_header(&mut self, col: usize) -> Outcome {
        if self.edit.is_some() {
            self.commit_edit();
        }
        self.sort_by(col)
    }

    /// Click on a row (display index) / cell. Double click is emulated by
    /// clicking the already-current cell.
    pub fn on_click_cell(
        &mut self,
        display: usize,
        col: usize,
        pos: Position,
    ) -> (Outcome, Option<TableAction>) {
        self.on_click_cell_with(display, col, pos, KeyModifiers::NONE)
    }

    /// [`Self::on_click_cell`] with the modifiers held during the click:
    /// in multi mode Shift extends from the anchor and
    /// [`Self::toggle_mods`] toggle the row; a plain click selects only
    /// that row, and a plain click on the sole selected cursor row
    /// activates it.
    pub fn on_click_cell_with(
        &mut self,
        display: usize,
        col: usize,
        pos: Position,
        mods: KeyModifiers,
    ) -> (Outcome, Option<TableAction>) {
        if display >= self.rows.len() {
            return (Outcome::Consumed, None);
        }
        // a click elsewhere ends an open popup without a choice
        self.choice = None;
        let col = col.min(self.columns.len().saturating_sub(1));
        if let Some(e) = &self.edit {
            if e.row == self.order[display] && e.col == col {
                // click inside the editor places the cursor
                if let Some(rect) = self.col_rects.get(col) {
                    let cw = rect.width.saturating_sub(1) as usize;
                    let off = e
                        .buffer
                        .cursor_pos()
                        .col
                        .saturating_sub(cw.saturating_sub(1));
                    let c = pos.x.saturating_sub(rect.x) as usize + off;
                    self.edit.as_mut().unwrap().buffer.set_cursor_line_col(0, c);
                }
                return (Outcome::Changed, None);
            }
            self.commit_edit();
            if self.edit.is_some() {
                return (Outcome::Changed, None);
            }
        }
        let same = self.cursor_row == display && (!self.cell_nav || self.cursor_col == col);
        let from = self.cursor_row;
        self.set_cursor(display, if self.cell_nav { col } else { self.cursor_col });
        if self.multi() && !self.cell_nav {
            let src = self.order[display];
            let disabled = &self.disabled;
            let enabled = |r: usize| !disabled.contains(&r);
            let changed = if mods.contains(KeyModifiers::SHIFT) {
                self.selection.extend(&self.order, from, display, enabled)
            } else if mods.intersects(self.toggle_mods) {
                self.selection.toggle(src, enabled)
            } else if same && self.selection.len() == 1 && self.selection.contains(src) {
                return (Outcome::Changed, Some(TableAction::Activated(src)));
            } else if !enabled(src) {
                // a disabled row takes the cursor, the selection stays
                false
            } else {
                self.selection.select_only(src, enabled)
            };
            return (
                Outcome::Changed,
                changed.then_some(TableAction::SelectionChanged),
            );
        }
        if same && self.cell_nav && self.columns[col].editable {
            return self.start_edit();
        }
        if !self.cell_nav {
            let src = self.order[display];
            if self.is_disabled(src) {
                return (Outcome::Changed, None);
            }
            if self.selection_mode == SelectionMode::Single {
                self.selected = Some(src);
            }
            return (Outcome::Changed, Some(TableAction::Activated(src)));
        }
        (Outcome::Changed, None)
    }

    /// Horizontal wheel: one column per notch.
    pub fn on_wheel_h(&mut self, delta: i32) -> Outcome {
        if !self.h_cfg.enabled {
            return Outcome::Ignored;
        }
        let before = self.hscroll.offset;
        self.hscroll.scroll_by(delta.signum() as isize);
        if self.hscroll.offset == before {
            Outcome::Consumed
        } else {
            Outcome::Changed
        }
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        if let Some(c) = self.choice.as_mut() {
            // the open popup takes the wheel
            return c.editor.on_wheel(delta);
        }
        if self.scroll.scroll_by(delta as isize) {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    fn track(&self) -> Rect {
        if self.track.is_empty() {
            Rect::new(
                self.area.right().saturating_sub(1),
                self.body.y,
                1,
                self.body.height,
            )
        } else {
            self.track
        }
    }

    /// The pointer went down on either scrollbar (or a completed click):
    /// a press on the thumb grabs it, a press on the track jumps to it. A
    /// press on the horizontal bar (its id is [`scrollbar::h_id_for`])
    /// moves columns.
    pub fn on_scrollbar(&mut self, pos: Position) -> Outcome {
        self.h_held = !self.htrack.is_empty() && self.htrack.contains(pos);
        let moved = if self.h_held {
            scrollbar::press_h(self.htrack, pos, &mut self.hscroll, self.h_cfg.track_click)
        } else {
            scrollbar::press_with(self.track(), pos, &mut self.scroll, self.v_click)
        };
        if moved {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    /// The pointer dragged while holding `pressed` (either of this
    /// widget's scrollbar ids): the axis follows the id, so a drag works
    /// whether or not the press was routed first.
    pub fn on_bar_drag(&mut self, pressed: WidgetId, pos: Position) -> Outcome {
        self.h_held = pressed == scrollbar::h_id_for(self.id);
        self.on_scrollbar_drag(pos)
    }

    /// The pointer dragged along the scrollbar it pressed.
    pub fn on_scrollbar_drag(&mut self, pos: Position) -> Outcome {
        let moved = if self.h_held && !self.htrack.is_empty() {
            scrollbar::drag_h(self.htrack, pos, &mut self.hscroll)
        } else {
            scrollbar::drag(self.track(), pos, &mut self.scroll)
        };
        if moved {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    /// Compute column rects for the visible columns.
    fn layout_columns(&mut self, area: Rect) {
        let n = self.columns.len();
        // decide how many columns fit starting at hscroll.offset
        let gap = 2u16;
        let mut fit = 0usize;
        let mut used = 0u16;
        for i in self.hscroll.offset..n {
            let w = self.sizing.width(i, self.columns[i].min_width());
            let need = if fit == 0 { w } else { w + gap };
            if used + need > area.width {
                break;
            }
            used += need;
            fit += 1;
        }
        let fit = fit.max(1).min(n - self.hscroll.offset.min(n));
        self.hscroll.content_len = n;
        self.hscroll.viewport_len = fit;
        self.hscroll.clamp();
        let visible = self.hscroll.offset..(self.hscroll.offset + fit).min(n);
        let constraints: Vec<Constraint> = visible
            .clone()
            .map(|i| match self.sizing.user(i) {
                Some(w) => Constraint::Length(w),
                None => self.columns[i].width,
            })
            .collect();
        let rects = Layout::horizontal(constraints).spacing(gap).split(area);
        self.col_rects = vec![Rect::ZERO; n];
        for (k, i) in visible.enumerate() {
            self.col_rects[i] = rects[k];
        }
    }

    /// Override the app-wide vertical track-click policy
    /// (`TrackClick::Page` pages toward the pointer).
    pub fn track_click(mut self, click: crate::core::scroll::TrackClick) -> Self {
        self.track_click = Some(click);
        self
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        self.parts.clear();
        let s = ctx.theme.strings;
        let old = std::mem::replace(&mut self.strings, s);
        crate::theme::Strings::follow(&mut self.empty_text, old.no_rows, s.no_rows);
        self.v_click = self.track_click.unwrap_or(ctx.track_click);
        let area = area.intersection(*buf.area());
        if area.is_empty() || area.height < 2 {
            return;
        }
        self.area = area;
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.id);
        let editing = self.edit.is_some() && focused;
        // a cell edit that lost focus is ended by `on_blur`, never here
        // where an open constrained editor's popup anchors this frame
        let mut choice_anchor: Option<Rect> = None;
        let mut open_marks: Vec<(usize, usize, Rect)> = Vec::new();

        ctx.control(self.id, area, false);
        ctx.scrollable(self.id, area);
        // gutter (1) + columns + scrollbar (1)
        let h_cfg = self.horizontal.unwrap_or(ctx.hscroll);
        self.h_cfg = h_cfg;
        // the viewport is settled once both bars are known: setting a
        // provisional taller one first would clamp the offset and drop
        // the cursor row off the bottom
        let rows_len = self.rows.len();
        let mut has_sb = rows_len > area.height as usize - 1;
        // the multi-selection checkbox column: `[✓]` and a blank
        let check_w: u16 = if self.multi() && self.selection_column {
            4
        } else {
            0
        };
        let cols_for = |has_sb: bool| {
            Rect::new(
                area.x + 3 + check_w,
                area.y,
                area.width.saturating_sub(5 + check_w + u16::from(has_sb)),
                area.height,
            )
        };
        self.layout_columns(cols_for(has_sb));
        let header_y = area.y;
        let mut body = Rect::new(area.x, area.y + 1, area.width, area.height - 1);
        // the horizontal bar takes the bottom row; losing it can make the
        // rows overflow, which costs a column for the vertical bar
        let has_hb = !self.rows.is_empty() && body.height > 1 && h_cfg.shows_bar(&self.hscroll);
        if has_hb {
            body.height -= 1;
            if !has_sb && rows_len > body.height as usize {
                has_sb = true;
                self.layout_columns(cols_for(true));
            }
        }
        let cols_area = cols_for(has_sb);
        self.scroll.set_content(rows_len);
        self.scroll.set_viewport(body.height as usize);
        self.body = body;

        // header
        let header_bg = bg;
        fill(
            buf,
            Rect::new(area.x, header_y, area.width, 1),
            Style::new().bg(header_bg),
        );
        for (ci, col) in self.columns.iter().enumerate() {
            let r = self.col_rects[ci];
            if r.is_empty() {
                continue;
            }
            let hid = self.header_id(ci);
            let sorted = self.sort.map(|(c, _)| c == ci).unwrap_or(false);
            let st = crate::widgets::grid::header_style(ctx, hid, header_bg, col.sortable, sorted);
            let ind = match self.sort {
                Some((c, SortDir::Asc)) if c == ci => format!(" {}", t.glyphs.sort_asc),
                Some((c, SortDir::Desc)) if c == ci => format!(" {}", t.glyphs.sort_desc),
                _ => String::new(),
            };
            let ind = ind.as_str();
            let ind_w = crate::ui::text::width(ind);
            let shown =
                crate::ui::text::truncate(&col.title, (r.width as usize).saturating_sub(ind_w));
            let cut = shown != col.title;
            let title = format!("{shown}{ind}");
            let text = match col.align {
                Align::Left => crate::ui::text::fit(&title, r.width as usize),
                Align::Right => crate::ui::text::fit_right(&title, r.width as usize),
            };
            buf.set_string(r.x, header_y, &text, st);
            let head = Rect::new(r.x, header_y, r.width, 1);
            if col.sortable {
                ctx.register_part(
                    self.id,
                    PartRef::item(Part::HEADER, ItemKey::index(ci)),
                    head,
                );
            } else if cut && !ctx.inert {
                // a cut title reveals on hover even when it does not sort
                ctx.hits.register_hover(hid, head);
            }
            if cut {
                crate::ui::reveal::truncated(ctx, self.reveal, hid, head, &col.title);
            }
            if self.sizing.resizable {
                let hit = self.sizing.seam_rect(r.right(), header_y, 2, area);
                crate::ui::columns::render_seam(
                    buf,
                    ctx,
                    (self.id, ci),
                    hit,
                    r.right(),
                    header_bg,
                    false,
                );
            }
        }
        if check_w > 0 && area.width > 3 + check_w {
            // the header's "select all / none" box
            let cid = self.check_all_id();
            let state = self.selection_state();
            let lifted = ctx.interaction.hovered(cid) || ctx.interaction.pressed(cid);
            let st = crate::widgets::grid::header_style(ctx, cid, header_bg, true, false);
            let st = match state {
                // hover lifts the box like a sortable title
                _ if lifted => st,
                TriState::None => st,
                _ => st.fg(t.legible(t.chosen(focused), st.bg.unwrap_or(header_bg))),
            };
            let r = Rect::new(area.x + 3, header_y, 3, 1);
            buf.set_string(r.x, r.y, state.glyph_in(&t.glyphs), st);
            if !self.rows.is_empty() {
                ctx.register_part(self.id, PartRef::of(Part::MARKER), r);
            }
        }
        // horizontal overflow indicators
        if self.overflow_marks && self.hscroll.overflows() {
            let more_right = self.hscroll.offset + self.hscroll.viewport_len < self.columns.len();
            let more_left = self.hscroll.offset > 0;
            if more_left {
                let st = t.faint().bg(header_bg);
                buf.set_string(area.x + 1, header_y, t.glyphs.ellipsis, st);
            }
            if more_right {
                buf.set_string(
                    cols_area.right() + 1,
                    header_y,
                    t.glyphs.ellipsis,
                    t.faint().bg(header_bg),
                );
            }
        }

        if self.rows.is_empty() {
            if let Some(e) = &self.empty {
                crate::widgets::empty::render(body, buf, t, e, bg);
                return;
            }
            let msg = crate::ui::text::truncate(&self.empty_text, area.width as usize);
            let y = body.y + body.height / 2;
            let x = body.x
                + body
                    .width
                    .saturating_sub(crate::ui::text::width(&msg) as u16)
                    / 2;
            buf.set_string(x, y, &msg, t.muted().bg(bg));
            return;
        }

        // rows
        for (i, di) in self.scroll.visible_range().enumerate() {
            let y = body.y + i as u16;
            let src = self.order[di];
            let row_rect = Rect::new(
                area.x,
                y,
                area.width.saturating_sub(if has_sb { 1 } else { 0 }),
                1,
            );
            let rid = self.row_id(di);
            let mut s = ctx.child_state(self.id, rid, di == self.cursor_row);
            // cell hover and press count as the row's
            let hovered_cell = (0..self.columns.len()).find(|&c| {
                ctx.interaction.hovered(self.cell_id(di, c))
                    || ctx.interaction.hovered(self.open_id(di, c))
            });
            s.hovered |= hovered_cell.is_some();
            s.pressed |= ctx
                .interaction
                .pressed_any((0..self.columns.len()).map(|c| self.cell_id(di, c)));
            s.focused = focused && di == self.cursor_row;
            s.selected = match self.selection_mode {
                SelectionMode::Single => self.selected == Some(src),
                SelectionMode::Multi => self.selection.contains(src),
                SelectionMode::None => false,
            };
            let row_disabled = self.is_disabled(src);
            if row_disabled {
                // a disabled row does not react to hover or press; it keeps
                // the cursor's focus bar so the keyboard position stays seen
                s.hovered = false;
                s.pressed = false;
            }
            let mut row_style = t.row(s, bg);
            fill(buf, row_rect, row_style);
            buf.set_string(
                row_rect.x,
                y,
                t.gutter_symbol(s),
                t.gutter(s, row_style.bg.unwrap_or(bg), false),
            );
            if row_disabled {
                row_style = row_style.fg(t.legible(t.text_muted, row_style.bg.unwrap_or(bg)));
            }
            let marker = match (self.selection_mode, s.selected) {
                (SelectionMode::Single, true) => t.glyphs.current,
                (SelectionMode::Multi, true) if check_w == 0 => t.glyphs.tick,
                _ => " ",
            };
            let ms = if s.selected {
                row_style.fg(if focused { t.accent } else { t.text_secondary })
            } else {
                row_style
            };
            buf.set_string(row_rect.x + 1, y, marker, ms);
            if check_w > 0 && area.width > 3 + check_w {
                let r = Rect::new(area.x + 3, y, 3, 1);
                let (glyph, gs) = if row_disabled {
                    (
                        t.glyphs.check_off,
                        row_style.fg(t.disabled).add_modifier(Modifier::DIM),
                    )
                } else if s.selected {
                    (t.glyphs.check_on, row_style.fg(t.chosen(focused)))
                } else {
                    (t.glyphs.check_off, row_style.fg(t.text_muted))
                };
                // its hit region goes on top of the row's, below
                buf.set_string(r.x, r.y, glyph, gs);
            }
            for (ci, col) in self.columns.iter().enumerate() {
                let r = self.col_rects[ci];
                if r.is_empty() {
                    continue;
                }
                let cell_rect = Rect::new(r.x, y, r.width, 1);
                let cell = &self.rows[src][ci];
                let is_cursor_cell = self.cell_nav && s.focused && ci == self.cursor_col;
                let is_edit = editing
                    && self
                        .edit
                        .as_ref()
                        .map(|e| e.row == src && e.col == ci)
                        .unwrap_or(false);
                let mut st = row_style;
                let cfg = &self.editor_config;
                let constrained = col.editor.is_constrained();
                // a constrained cell shows its option's label and tone; a
                // value outside the options is flagged like an error
                let shown: std::borrow::Cow<str> = if let Some(on) = col.editor.checked(&cell.text)
                {
                    // a two-value cell is a checkbox
                    if on {
                        t.glyphs.check_on
                    } else {
                        t.glyphs.check_off
                    }
                    .into()
                } else if constrained {
                    col.editor.display(&cell.text, cfg).into()
                } else {
                    cell.text.as_str().into()
                };
                let flagged = cell.error.is_some()
                    || (cfg.flag_out_of_set && col.editor.out_of_set(&cell.text, cfg));
                let tone = match (cell.tone, col.editor.tone(&cell.text)) {
                    (Tone::Normal, Some(tone)) => tone,
                    (tone, _) => tone,
                };
                let is_choice_cell = self
                    .choice
                    .as_ref()
                    .is_some_and(|c| c.row == src && c.col == ci);
                if is_choice_cell {
                    choice_anchor = Some(cell_rect);
                }
                st = match tone {
                    Tone::Normal => st,
                    _ if row_disabled => st,
                    other => st.fg(t.tone(other)),
                };
                if s.pressed {
                    st = row_style;
                }
                if is_edit {
                    let e = self.edit.as_ref().unwrap();
                    let es = t.field_style(crate::ui::ctx::VisualState {
                        editing: true,
                        ..Default::default()
                    });
                    fill(buf, cell_rect, es);
                    let text = e.buffer.text();
                    let cw = r.width.saturating_sub(1) as usize;
                    let cur = e.buffer.cursor_pos().col;
                    let off = cur.saturating_sub(cw.saturating_sub(1));
                    let shown = crate::ui::text::slice_cells(text, off, cw);
                    let mut ts = es
                        .add_modifier(Modifier::UNDERLINED)
                        .underline_color(t.accent);
                    if e.error.is_some() {
                        ts = ts.underline_color(t.error);
                    }
                    buf.set_string(r.x, y, &shown, ts);
                    if cw > 0 {
                        ctx.set_cursor(Position::new(r.x + (cur - off) as u16, y));
                    }
                    if e.error.is_some() {
                        buf.set_string(
                            r.right().saturating_sub(1),
                            y,
                            "!",
                            es.fg(t.error).add_modifier(Modifier::BOLD),
                        );
                    }
                } else {
                    if is_cursor_cell || is_choice_cell {
                        // cell cursor: the cursor fill on the cell only
                        st = t.cell_cursor(flagged, s.pressed);
                    } else if flagged {
                        st = st.fg(t.error);
                    } else if col.editable && self.cell_nav && s.hovered && hovered_cell == Some(ci)
                    {
                        st = st
                            .add_modifier(Modifier::UNDERLINED)
                            .underline_color(t.border_strong);
                    }
                    // editable popup cells advertise their `▾` on hover and
                    // on the cursor cell; read-only cells never do
                    let affordance = col.editable
                        && col.editor.has_popup()
                        && r.width > 3
                        && match cfg.affordance {
                            Affordance::Never => false,
                            Affordance::Always => true,
                            Affordance::Focused => {
                                is_cursor_cell
                                    || is_choice_cell
                                    || (s.hovered && hovered_cell == Some(ci))
                            }
                        };
                    let text_w = if affordance { r.width - 2 } else { r.width } as usize;
                    let text = match col.align {
                        Align::Left => crate::ui::text::fit(&shown, text_w),
                        Align::Right => crate::ui::text::fit_right(&shown, text_w),
                    };
                    if affordance {
                        fill(buf, cell_rect, st);
                    }
                    buf.set_string(r.x, y, &text, st);
                    crate::ui::reveal::truncated(
                        ctx,
                        self.reveal,
                        self.cell_id(di, ci),
                        Rect::new(r.x, y, text_w as u16, 1),
                        &shown,
                    );
                    if affordance {
                        let mark = Rect::new(r.right() - 1, y, 1, 1);
                        let ms = if is_cursor_cell || is_choice_cell {
                            st
                        } else {
                            st.fg(t.text_secondary)
                        };
                        buf.set_string(mark.x, y, t.glyphs.disclosure_open, ms);
                        open_marks.push((di, ci, mark));
                    }
                    if flagged && !is_cursor_cell && !is_choice_cell && !affordance {
                        buf.set_string(
                            r.right().saturating_sub(1),
                            y,
                            "!",
                            st.add_modifier(Modifier::BOLD),
                        );
                    }
                }
                let cid = self.cell_id(di, ci);
                let cell = (Self::cell_part(di), ci);
                ctx.part_cell(&mut self.parts, cid, self.id, cell, cell_rect);
            }
            ctx.part(&mut self.parts, self.id, Self::row_part(di), row_rect);
            // re-register cells on top so cell hover resolves before row
            for ci in 0..self.columns.len() {
                let r = self.col_rects[ci];
                if !r.is_empty() {
                    let cid = self.cell_id(di, ci);
                    let cell = (Self::cell_part(di), ci);
                    let rect = Rect::new(r.x, y, r.width, 1);
                    ctx.part_cell(&mut self.parts, cid, self.id, cell, rect);
                }
            }
            if check_w > 0 && area.width > 3 + check_w && !row_disabled {
                let rect = Rect::new(area.x + 3, y, 3, 1);
                ctx.part(&mut self.parts, self.id, Self::check_part(di), rect);
            }
        }
        // a cell's `▾` sits above the cell: a click on it opens the editor
        for (di, ci, rect) in open_marks {
            let oid = self.open_id(di, ci);
            let open = (PartRef::item(Part::ACTIONS, ItemKey::index(di)), ci);
            ctx.part_cell(&mut self.parts, oid, self.id, open, rect);
        }
        if let (Some(anchor), Some(c)) = (choice_anchor, self.choice.as_mut()) {
            c.editor.render(anchor, buf, ctx, &self.editor_config);
        }
        // the scroll region also sits above the rows: a wheel burst handled
        // before the next frame (see `runtime::Coalesce`) then routes to the
        // widget, not to a row whose display index it already scrolled away
        ctx.scrollable(self.id, area);
        self.track = Rect::ZERO;
        if has_sb {
            crate::ui::fade::scroll_edges_except(
                buf,
                ctx,
                Rect::new(
                    area.x,
                    body.y,
                    (area.right() - 1).saturating_sub(area.x),
                    body.height,
                ),
                &self.scroll,
                &self
                    .scroll
                    .visible_range()
                    .position(|i| i == self.cursor_row)
                    .map(|k| body.y + k as u16)
                    .into_iter()
                    .collect::<Vec<_>>(),
            );
            let sb = Rect::new(area.right() - 1, body.y, 1, body.height);
            self.track = sb;
            scrollbar::render_vertical(sb, buf, ctx, self.id, &self.scroll, focused);
        }
        self.htrack = Rect::ZERO;
        if has_hb {
            // under the columns; the corner under the vertical bar stays blank
            let y = body.bottom();
            let right = area.right() - u16::from(has_sb);
            fill(
                buf,
                Rect::new(area.x, y, area.width, 1),
                Style::new().bg(bg),
            );
            let hb = Rect::new(cols_area.x, y, right.saturating_sub(cols_area.x), 1);
            self.htrack = hb;
            scrollbar::render_horizontal_with(
                hb,
                buf,
                ctx,
                self.id,
                &self.hscroll,
                focused,
                h_cfg.bar == ScrollbarVisibility::Always,
            );
        }
    }

    /// Resolve which (display row, col) a widget id refers to.
    pub fn locate(&self, id: WidgetId) -> Option<(usize, Option<usize>)> {
        if let Some(di) = self.parts.index(id, Part::ROW) {
            return Some((di, None));
        }
        self.parts
            .cell(id, Part::CELL)
            .map(|(di, ci)| (di, Some(ci)))
    }

    pub fn owns(&self, id: WidgetId) -> bool {
        self.choice.as_ref().is_some_and(|c| c.editor.owns(id))
            || id == self.id
            || scrollbar::is_bar_of(id, self.id)
            || self.parts.contains(id)
            || self.locate_header(id).is_some()
            || self.locate_seam(id).is_some()
            || id == self.check_all_id()
    }

    pub fn locate_header(&self, id: WidgetId) -> Option<usize> {
        self.id.item_of(id, Part::HEADER, 0..self.columns.len())
    }

    pub fn edit_error(&self) -> Option<&str> {
        if let Some(c) = &self.choice {
            return c.editor.error.as_deref();
        }
        self.edit.as_ref().and_then(|e| e.error.as_deref())
    }
}

use ratatui::style::Style;

/// Numeric key with a total order: `None` for cells without a number.
fn parse_num(s: &str) -> Option<ordered::F64> {
    let cleaned: String = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect();
    cleaned
        .parse::<f64>()
        .ok()
        .filter(|f| !f.is_nan())
        .map(ordered::F64)
}

mod ordered {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct F64(pub f64);
    impl Eq for F64 {}
    impl PartialOrd for F64 {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for F64 {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.0.total_cmp(&other.0)
        }
    }
}

impl crate::core::focus::BlurTarget for DataTable {
    fn blur_id(&self) -> WidgetId {
        self.id
    }
    fn blur(&mut self) -> Outcome {
        let editing = self.edit.is_some();
        self.on_blur();
        if editing && self.edit.is_none() {
            Outcome::Changed
        } else {
            Outcome::Ignored
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::theme::{ColorLevel, Theme};
    use crate::ui::ctx::Interaction;

    fn render_editor(table: &mut DataTable, level: ColorLevel) -> (Buffer, Position) {
        let area = Rect::new(0, 0, 40, 5);
        let mut buf = Buffer::empty(area);
        let theme = Theme::for_level(level);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(
            &theme,
            Interaction {
                focus: Some(table.id),
                ..Default::default()
            },
            &mut hits,
            &mut ring,
        );
        table.render(area, &mut buf, &mut ctx, theme.surface);
        let cursor = ctx.cursor.unwrap();
        (buf, cursor)
    }

    fn table() -> DataTable {
        let cols = vec![
            Column::new("Name", Constraint::Min(8)).editable(),
            Column::new("N", Constraint::Length(4)).right().editable(),
        ];
        let rows = vec![
            vec![Cell::new("beta"), Cell::new("2")],
            vec![Cell::new("alpha"), Cell::new("10")],
            vec![Cell::new("Gamma"), Cell::new("1")],
        ];
        DataTable::new(WidgetId::of("t"), cols, rows)
            .cell_nav(true)
            .numeric(&[1])
    }

    fn wide_table() -> DataTable {
        let cols = (0..9)
            .map(|i| Column::new(&format!("col{i}"), Constraint::Length(10)))
            .collect();
        let rows = (0..3)
            .map(|r| (0..9).map(|c| Cell::new(format!("r{r}c{c}"))).collect())
            .collect();
        DataTable::new(WidgetId::of("wide"), cols, rows)
    }

    fn draw(t: &mut DataTable, area: Rect) -> (Buffer, HitRegistry) {
        let mut buf = Buffer::empty(area);
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        {
            let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
            t.render(area, &mut buf, &mut ctx, theme.surface);
        }
        (buf, hits)
    }

    #[test]
    fn header_seams_resize_autofit_and_the_menu_resets() {
        let mut t = wide_table();
        let area = Rect::new(0, 0, 60, 8);
        let (_, hits) = draw(&mut t, area);
        let head = hits.area_of(t.header_id(0)).unwrap();
        let sid = t.seam_id(0);
        let seam = hits.area_of(sid).expect("a seam after the first column");
        assert_eq!(seam.x, head.right(), "in the gap after the title");
        assert_eq!(head.width, 10);
        // the grab offset is kept: press one cell into the seam
        t.on_press(sid, Position::new(seam.x + 1, seam.y), 1);
        t.on_drag(sid, Position::new(seam.x + 1, seam.y));
        t.on_drag(sid, Position::new(seam.x + 5, seam.y));
        let (_, hits) = draw(&mut t, area);
        assert_eq!(hits.area_of(t.header_id(0)).unwrap().width, 14);
        assert_eq!(hits.area_of(t.header_id(1)).unwrap().x, head.x + 14 + 2);
        // a double press fits "col0" / "r0c0" (4 cells)
        t.on_press(sid, Position::new(seam.x, seam.y), 2);
        let (_, hits) = draw(&mut t, area);
        assert_eq!(hits.area_of(t.header_id(0)).unwrap().width, 4);
        // the header menu sorts and resets
        assert_eq!(t.locate_header_menu(sid), Some(0));
        t.on_header_action(0, HeaderAction::SortDesc);
        assert_eq!(t.sort, Some((0, SortDir::Desc)));
        t.on_header_action(0, HeaderAction::ClearSort);
        assert_eq!(t.sort, None);
        t.on_header_action(0, HeaderAction::ResetWidths);
        let (_, hits) = draw(&mut t, area);
        assert_eq!(hits.area_of(t.header_id(0)).unwrap().width, 10);
        // off: no seam
        let mut t = wide_table().resizable_columns(false);
        let (_, hits) = draw(&mut t, area);
        assert!(hits.area_of(t.seam_id(0)).is_none());
    }

    #[test]
    fn a_cut_header_title_reveals_on_hover() {
        let long = "a column title much wider than its column";
        let cols = vec![Column::new(long, Constraint::Length(8))];
        let mut t = DataTable::new(WidgetId::of("cut"), cols, vec![vec![Cell::new("x")]]);
        let area = Rect::new(0, 0, 70, 6);
        let theme = Theme::junie();
        let mut buf = Buffer::empty(area);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let hover = Interaction {
            hover: Some(t.header_id(0)),
            ..Default::default()
        };
        let mut ctx = RenderCtx::new(&theme, hover, &mut hits, &mut ring);
        t.render(area, &mut buf, &mut ctx, theme.surface);
        ctx.flush_overlays(&mut buf);
        let all: String = (0..area.height)
            .flat_map(|y| (0..area.width).map(move |x| (x, y)))
            .map(|(x, y)| buf[(x, y)].symbol().to_string())
            .collect();
        assert!(all.contains(long), "{all}");
    }

    fn thumb_x(buf: &Buffer, y: u16) -> Option<u16> {
        (0..buf.area.width).find(|&x| buf[(x, y)].symbol() == scrollbar::H_THUMB)
    }

    #[test]
    fn wide_columns_show_a_horizontal_bar_that_moves_by_columns() {
        let mut t = wide_table();
        let area = Rect::new(0, 0, 40, 7);
        let (buf, hits) = draw(&mut t, area);
        let y = area.bottom() - 1;
        assert_eq!(thumb_x(&buf, y), Some(t.htrack.x), "thumb at the left");
        assert_eq!(
            hits.hit(Position::new(t.htrack.x + 2, y)),
            Some(scrollbar::h_id_for(t.id))
        );
        assert!(t.owns(scrollbar::h_id_for(t.id)));

        assert_eq!(t.on_wheel_h(3), Outcome::Changed);
        assert_eq!(t.hscroll.offset, 1, "one column per notch");
        let (buf, _) = draw(&mut t, area);
        let grab = thumb_x(&buf, y).unwrap();
        assert!(grab > t.htrack.x);

        // the shared scrollbar entry point tells the bars apart by position
        assert_eq!(t.on_scrollbar(Position::new(grab, y)), Outcome::Consumed);
        assert_eq!(
            t.on_scrollbar_drag(Position::new(area.right(), y)),
            Outcome::Changed
        );
        assert_eq!(t.hscroll.offset, t.hscroll.max_offset());
        assert_eq!(t.scroll.offset, 0, "rows did not move");

        let mut off = wide_table().horizontal(HScroll::off());
        let (buf, _) = draw(&mut off, area);
        assert_eq!(thumb_x(&buf, y), None);
        assert_eq!(off.on_wheel_h(1), Outcome::Ignored);
    }

    #[test]
    fn sort_cycles_asc_desc_none() {
        let mut t = table();
        t.sort_by(0);
        assert_eq!(t.sort, Some((0, SortDir::Asc)));
        assert_eq!(t.order, vec![1, 0, 2]);
        t.sort_by(0);
        assert_eq!(t.sort, Some((0, SortDir::Desc)));
        assert_eq!(t.order, vec![2, 0, 1]);
        t.sort_by(0);
        assert_eq!(t.sort, None);
        assert_eq!(t.order, vec![0, 1, 2]);
    }

    #[test]
    fn numeric_sort_is_not_lexicographic() {
        let mut t = table();
        t.sort_by(1);
        assert_eq!(t.order, vec![2, 0, 1]);
    }

    #[test]
    fn replacing_rows_cancels_edits_without_touching_the_new_dataset() {
        for row_count in [0, 1, 3] {
            let mut t = table();
            t.cursor_row = 2;
            t.begin_edit();
            t.edit.as_mut().unwrap().buffer.set_text("stale edit");
            let rows = (0..row_count)
                .map(|i| vec![Cell::new(format!("new {i}")), Cell::new("0")])
                .collect();

            t.set_rows(rows);

            assert_eq!(t.commit_edit(), None);
            assert!(!t.is_editing());
            for (i, row) in t.rows.iter().enumerate() {
                assert_eq!(row[0].text, format!("new {i}"));
            }
        }
    }

    #[test]
    fn replacing_rows_clears_selection_and_preserves_sort_and_valid_cursor() {
        let mut t = table();
        t.sort_by(1);
        t.cursor_row = 2;
        t.cursor_col = 1;
        t.selected = Some(2);
        t.scroll.set_viewport(1);
        t.scroll.scroll_to(2);

        t.set_rows(vec![
            vec![Cell::new("new first"), Cell::new("20")],
            vec![Cell::new("new second"), Cell::new("3")],
        ]);

        assert_eq!(t.selected, None);
        assert_eq!(t.sort, Some((1, SortDir::Asc)));
        assert_eq!(t.source_row(0), 1);
        assert_eq!((t.cursor_row, t.cursor_col), (1, 1));
        assert!(t.scroll.visible_range().contains(&t.cursor_row));
    }

    #[test]
    fn sort_keeps_cursor_on_same_row() {
        let mut t = table();
        t.cursor_row = 1; // alpha
        t.sort_by(0);
        assert_eq!(t.source_row(t.cursor_row), 1);
    }

    #[test]
    fn rendering_without_focus_never_ends_the_cell_edit() {
        let mut t = table();
        t.begin_edit();
        t.edit.as_mut().unwrap().buffer.set_text("renamed");
        draw(&mut t, Rect::new(0, 0, 60, 10));
        assert!(t.edit.is_some(), "render is not a blur");
        assert_eq!(t.on_blur(), Some(TableAction::Committed { row: 0, col: 0 }));
    }

    #[test]
    fn blur_commits_cancels_or_keeps_the_cell_edit() {
        let edited = |action| {
            let mut t = table();
            t.blur_action = action;
            assert_eq!(t.on_blur(), None);
            t.begin_edit();
            t.edit.as_mut().unwrap().buffer.set_text("renamed");
            t
        };
        let mut commit = edited(BlurAction::Commit);
        assert_eq!(
            commit.on_blur(),
            Some(TableAction::Committed { row: 0, col: 0 })
        );
        assert_eq!(commit.rows[0][0].text, "renamed");
        let mut cancel = edited(BlurAction::Cancel);
        assert_eq!(cancel.on_blur(), Some(TableAction::Cancelled));
        assert_ne!(cancel.rows[0][0].text, "renamed");
        let mut keep = edited(BlurAction::Keep);
        assert_eq!(keep.on_blur(), None);
        assert!(keep.edit.is_some());
    }

    #[test]
    fn rejected_edit_stays_on_its_source_row_after_header_sort() {
        // a closure that captures state (the reserved values)
        let reserved = ["!".to_owned()];
        let mut t = table().validator(move |_, value| {
            reserved
                .iter()
                .any(|r| r == value)
                .then(|| "Invalid".to_owned())
        });
        t.begin_edit();
        t.edit.as_mut().unwrap().buffer.set_text("!");
        t.on_click_header(0);
        assert!(t.edit.as_ref().unwrap().error.is_some());
        assert_eq!(t.source_row(t.cursor_row), 0);
        t.edit.as_mut().unwrap().buffer.set_text("fixed");
        assert_eq!(
            t.commit_edit(),
            Some(TableAction::Committed { row: 0, col: 0 })
        );
        assert_eq!(t.rows[0][0].text, "fixed");
        assert_eq!(t.rows[1][0].text, "alpha");
    }

    #[test]
    fn direct_sort_during_edit_keeps_source_identity_in_every_order() {
        for sort_count in 1..=3 {
            let mut t = table();
            t.cursor_row = 1;
            t.begin_edit();
            t.edit.as_mut().unwrap().buffer.set_text("fixed");
            for _ in 0..sort_count {
                t.sort_by(0);
            }
            assert_eq!(
                t.commit_edit(),
                Some(TableAction::Committed { row: 1, col: 0 })
            );
            assert_eq!(t.rows[1][0].text, "fixed");
            assert_eq!(t.rows[0][0].text, "beta");
        }
    }

    #[test]
    fn clicking_another_cell_preserves_invalid_edit_and_cursor() {
        let mut t = table().validator(|_, value| (value == "!").then(|| "Invalid".to_owned()));
        t.begin_edit();
        t.edit.as_mut().unwrap().buffer.set_text("!");
        t.on_click_cell(1, 0, Position::ORIGIN);
        assert_eq!(t.cursor_row, 0);
        assert_eq!(t.edit.as_ref().unwrap().buffer.text(), "!");
        assert!(t.edit.as_ref().unwrap().error.is_some());
        t.cancel_edit();
        assert_eq!(t.rows[0][0].text, "beta");
        assert_eq!(t.rows[1][0].text, "alpha");
    }

    #[test]
    fn unicode_edit_window_keeps_cells_bounded_and_clicks_at_display_position() {
        for level in [ColorLevel::TrueColor, ColorLevel::Mono] {
            let mut t = DataTable::new(
                WidgetId::of("unicode-table"),
                vec![
                    Column::new("Name", Constraint::Length(8)).editable(),
                    Column::new("Other", Constraint::Length(8)),
                ],
                vec![vec![Cell::new(""), Cell::new("KEEP")]],
            )
            .cell_nav(true);
            t.begin_edit();
            let (before, _) = render_editor(&mut t, level);
            let text = format!("{}終", "日本語👩‍💻cafe\u{301}".repeat(8));
            t.on_paste(&text);
            let (after, cursor) = render_editor(&mut t, level);
            let cell = Rect::new(t.col_rects[0].x, cursor.y, t.col_rects[0].width, 1);
            assert!(cell.contains(cursor));
            assert_eq!(after[(cursor.x - 2, cursor.y)].symbol(), "終");
            let expected = t
                .edit
                .as_ref()
                .unwrap()
                .buffer
                .offset_at(0, crate::ui::text::width(&text) - 2);
            t.on_click_cell(0, 0, Position::new(cursor.x - 2, cursor.y));
            assert_eq!(t.edit.as_ref().unwrap().buffer.cursor_offset(), expected);
            t.on_key(&Key {
                code: KeyCode::Home,
                mods: ratatui::crossterm::event::KeyModifiers::NONE,
            });
            let (home, _) = render_editor(&mut t, level);
            for x in 0..40 {
                if !cell.contains(Position::new(x, cursor.y)) {
                    assert_eq!(
                        after[(x, cursor.y)].symbol(),
                        before[(x, cursor.y)].symbol()
                    );
                    assert_eq!(home[(x, cursor.y)].symbol(), before[(x, cursor.y)].symbol());
                }
            }
        }
    }

    #[test]
    fn edit_commit_and_cancel() {
        let mut t = table();
        t.begin_edit();
        assert!(t.is_editing());
        let e = t.edit.as_mut().unwrap();
        e.buffer.select_all();
        e.buffer.insert_str("zeta");
        let ev = t.commit_edit();
        assert_eq!(ev, Some(TableAction::Committed { row: 0, col: 0 }));
        assert_eq!(t.rows[0][0].text, "zeta");
        t.begin_edit();
        t.edit.as_mut().unwrap().buffer.insert_str("!!!");
        t.cancel_edit();
        assert_eq!(t.rows[0][0].text, "zeta");
    }

    #[test]
    fn validation_blocks_commit() {
        let mut t = table().validator(|col, s| {
            if col == 1 && s.parse::<u32>().is_err() {
                Some("Must be a number".into())
            } else {
                None
            }
        });
        t.cursor_col = 1;
        t.begin_edit();
        t.edit.as_mut().unwrap().buffer.insert_str("x");
        assert!(t.commit_edit().is_none());
        assert!(t.is_editing());
        assert_eq!(t.edit_error(), Some("Must be a number"));
    }

    #[test]
    fn tab_moves_to_next_editable_cell_and_leaves_at_end() {
        let mut t = table();
        t.begin_edit();
        let tab = Key {
            code: KeyCode::Tab,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        let (_, ev) = t.on_key(&tab);
        assert_eq!(ev, Some(TableAction::Committed { row: 0, col: 0 }));
        assert_eq!(t.cursor_col, 1);
        assert!(t.is_editing());
        let (_, ev) = t.on_key(&tab);
        assert_eq!(ev, Some(TableAction::LeaveForward));
        assert!(!t.is_editing());
    }

    fn render_with(table: &mut DataTable, interaction: Interaction) -> Buffer {
        let area = Rect::new(0, 0, 40, 5);
        let mut buf = Buffer::empty(area);
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, interaction, &mut hits, &mut ring);
        table.render(area, &mut buf, &mut ctx, theme.surface);
        buf
    }

    fn row_of(buf: &Buffer, text: &str) -> u16 {
        (0..buf.area.height)
            .find(|&y| {
                (0..buf.area.width)
                    .map(|x| buf[(x, y)].symbol())
                    .collect::<String>()
                    .contains(text)
            })
            .unwrap()
    }

    #[test]
    fn a_press_on_a_cell_or_the_keyboard_flash_presses_the_row() {
        let theme = Theme::junie();
        let pressed = theme.ladder(crate::theme::Plane::Surface).pressed;
        let mut t = table();
        // pointer held on a cell (cells register above the row)
        let cell = t.cell_id(0, 0);
        let buf = render_with(
            &mut t,
            Interaction {
                hover: Some(cell),
                pressed: Some(cell),
                ..Default::default()
            },
        );
        let y = row_of(&buf, "beta");
        assert_eq!(buf[(0, y)].bg, pressed, "cell press is the row's press");
        // Enter on the focused table flashes the container id
        let id = t.id;
        let buf = render_with(
            &mut t,
            Interaction {
                focus: Some(id),
                flash: Some(id),
                ..Default::default()
            },
        );
        let y = row_of(&buf, "beta");
        assert_eq!(
            buf[(0, y)].bg,
            pressed,
            "keyboard flash lands on the cursor row"
        );
    }

    #[test]
    fn the_cell_cursor_is_the_cursor_fill_not_a_white_flood() {
        let theme = Theme::junie();
        let mut t = table();
        let id = t.id;
        let buf = render_with(
            &mut t,
            Interaction {
                focus: Some(id),
                ..Default::default()
            },
        );
        let y = row_of(&buf, "beta");
        let x = (0..40).find(|&x| buf[(x, y)].symbol() == "b").unwrap();
        assert_eq!(buf[(x, y)].bg, theme.highlight);
        assert_eq!(buf[(x, y)].fg, theme.text_primary);
    }

    #[test]
    fn hovering_a_cut_cell_reveals_its_full_text() {
        let long = "a cell value far wider than its column";
        let mut t = table();
        t.rows[0][0] = Cell::new(long);
        let draw = |t: &mut DataTable, hover| {
            let area = Rect::new(0, 0, 30, 5);
            let mut buf = Buffer::empty(Rect::new(0, 0, 60, 5));
            let theme = Theme::junie();
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut ctx = RenderCtx::new(
                &theme,
                Interaction {
                    hover,
                    ..Default::default()
                },
                &mut hits,
                &mut ring,
            );
            t.render(area, &mut buf, &mut ctx, theme.surface);
            ctx.flush_overlays(&mut buf);
            (0..5)
                .map(|y| (0..60).map(|x| buf[(x, y)].symbol()).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n")
        };
        assert!(!draw(&mut t, None).contains(long));
        let cell = t.cell_id(0, 0);
        assert!(draw(&mut t, Some(cell)).contains(long));
        let short = t.cell_id(1, 0);
        assert!(!draw(&mut t, Some(short)).contains(long));
        let mut off = table().reveal(crate::ui::reveal::Reveal::OFF);
        off.rows[0][0] = Cell::new(long);
        assert!(!draw(&mut off, Some(cell)).contains(long));
    }

    // ------------------------------------------------------------ selection

    fn members() -> DataTable {
        let cols = vec![
            Column::new("Name", Constraint::Min(8)),
            Column::new("Role", Constraint::Length(8)),
        ];
        let rows = ["Dana", "Ari", "Cleo", "Bo", "Eli", "Fay"]
            .iter()
            .enumerate()
            .map(|(i, n)| vec![Cell::new(*n), Cell::new(format!("r{i}"))])
            .collect();
        DataTable::new(WidgetId::of("m"), cols, rows)
            .selection_mode(SelectionMode::Multi)
            .disabled_rows(&[4])
    }

    fn k(code: KeyCode, mods: KeyModifiers) -> Key {
        Key { code, mods }
    }

    #[test]
    fn multi_space_toggles_the_cursor_row_and_reports_it() {
        let mut t = members();
        let (out, ev) = t.on_key(&k(KeyCode::Char(' '), KeyModifiers::NONE));
        assert_eq!(out, Outcome::Changed);
        assert_eq!(ev, Some(TableAction::SelectionChanged));
        assert_eq!(t.selected_rows(), vec![0]);
        assert_eq!(t.selection_label(), "1 selected");
        t.on_key(&k(KeyCode::Char(' '), KeyModifiers::NONE));
        assert!(t.selected_rows().is_empty());
        assert_eq!(t.selection_label(), "");
        assert_eq!(t.selected, None, "multi mode never sets the single choice");
    }

    #[test]
    fn multi_shift_arrows_extend_a_range_and_skip_disabled_rows() {
        let mut t = members();
        t.on_key(&k(KeyCode::Down, KeyModifiers::NONE));
        t.on_key(&k(KeyCode::Down, KeyModifiers::NONE));
        assert!(t.selected_rows().is_empty(), "plain arrows only move");
        for _ in 0..3 {
            t.on_key(&k(KeyCode::Down, KeyModifiers::SHIFT));
        }
        assert_eq!(t.cursor_row, 5);
        assert_eq!(t.selected_rows(), vec![2, 3, 5], "row 4 is disabled");
        t.on_key(&k(KeyCode::Up, KeyModifiers::SHIFT));
        t.on_key(&k(KeyCode::Up, KeyModifiers::SHIFT));
        assert_eq!(t.selected_rows(), vec![2, 3], "shrinking deselects");
    }

    #[test]
    fn multi_select_all_and_esc_clear_are_configurable() {
        let mut t = members();
        let (_, ev) = t.on_key(&k(KeyCode::Char('a'), KeyModifiers::CONTROL));
        assert_eq!(ev, Some(TableAction::SelectionChanged));
        assert_eq!(t.selected_rows(), vec![0, 1, 2, 3, 5]);
        assert_eq!(t.selection_state(), TriState::All);
        // Cmd+A works the same where the terminal reports it
        t.clear_selection();
        t.on_key(&k(KeyCode::Char('a'), KeyModifiers::SUPER));
        assert_eq!(t.selected_rows().len(), 5);
        let (out, _) = t.on_key(&k(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(out, Outcome::Changed);
        assert!(t.selected_rows().is_empty());
        // with nothing selected Esc is not the table's
        assert_eq!(
            t.on_key(&k(KeyCode::Esc, KeyModifiers::NONE)).0,
            Outcome::Ignored
        );
        let mut keep = members().esc_clears(false);
        keep.on_key(&k(KeyCode::Char(' '), KeyModifiers::NONE));
        assert_eq!(
            keep.on_key(&k(KeyCode::Esc, KeyModifiers::NONE)).0,
            Outcome::Ignored
        );
        assert_eq!(keep.selected_rows(), vec![0]);
    }

    #[test]
    fn multi_selection_survives_sorting_and_scrolling() {
        let mut t = members();
        t.toggle_row(1); // Ari
        t.toggle_row(3); // Bo
        t.sort_by(0);
        let names = |t: &DataTable| -> Vec<String> {
            t.selected_rows()
                .iter()
                .map(|&r| t.rows[r][0].text.clone())
                .collect()
        };
        assert_eq!(names(&t), vec!["Ari", "Bo"]);
        // Ari and Bo now display first
        assert!(t.selection.contains(t.source_row(0)));
        assert!(t.selection.contains(t.source_row(1)));
        let (buf, _) = draw(&mut t, Rect::new(0, 0, 30, 3));
        assert_eq!(buf[(1, 1)].symbol(), "✓");
        t.on_wheel(3);
        assert_eq!(t.selected_rows(), vec![1, 3]);
        // a shift range follows the display order
        t.clear_selection();
        t.cursor_row = 0;
        t.on_key(&k(KeyCode::Char(' '), KeyModifiers::NONE));
        t.on_key(&k(KeyCode::Down, KeyModifiers::SHIFT));
        t.on_key(&k(KeyCode::Down, KeyModifiers::SHIFT));
        // display order Ari, Bo, Cleo; reported in source order
        assert_eq!(names(&t), vec!["Ari", "Cleo", "Bo"]);
    }

    #[test]
    fn disabled_rows_are_never_selected_or_activated() {
        let mut t = members();
        t.cursor_row = 4;
        let (_, ev) = t.on_key(&k(KeyCode::Char(' '), KeyModifiers::NONE));
        assert_eq!(ev, None);
        assert_eq!(
            t.on_key(&k(KeyCode::Enter, KeyModifiers::NONE)),
            (Outcome::Consumed, None)
        );
        let (_, ev) = t.on_click_cell_with(4, 0, Position::new(0, 0), KeyModifiers::CONTROL);
        assert_eq!(ev, None);
        assert!(t.selected_rows().is_empty());
        let mut single = members().selection_mode(SelectionMode::Single);
        let (_, ev) = single.on_click_cell(4, 0, Position::new(0, 0));
        assert_eq!(ev, None);
        assert_eq!(single.selected, None);
        // disabling a selected row drops it
        t.toggle_row(2);
        t.set_row_disabled(2, true);
        assert!(t.selected_rows().is_empty());
    }

    #[test]
    fn multi_clicks_follow_desktop_modifiers() {
        let mut t = members().toggle_mods(DataTable::toggle_mods_for(ClientPlatform::Other));
        let p = Position::new(0, 0);
        let (_, ev) = t.on_click_cell_with(1, 0, p, KeyModifiers::NONE);
        assert_eq!(ev, Some(TableAction::SelectionChanged));
        assert_eq!(t.selected_rows(), vec![1]);
        t.on_click_cell_with(3, 0, p, KeyModifiers::SHIFT);
        assert_eq!(t.selected_rows(), vec![1, 2, 3]);
        t.on_click_cell_with(2, 0, p, KeyModifiers::CONTROL);
        assert_eq!(t.selected_rows(), vec![1, 3]);
        t.on_click_cell_with(5, 0, p, KeyModifiers::SUPER);
        assert_eq!(t.selected_rows(), vec![1, 3, 5]);
        // a plain click replaces the selection
        t.on_click_cell_with(0, 0, p, KeyModifiers::NONE);
        assert_eq!(t.selected_rows(), vec![0]);
        // a plain click on the sole selected cursor row activates it
        let (_, ev) = t.on_click_cell_with(0, 0, p, KeyModifiers::NONE);
        assert_eq!(ev, Some(TableAction::Activated(0)));
        // a plain click on a disabled row moves the cursor only
        t.on_click_cell_with(4, 0, p, KeyModifiers::NONE);
        assert_eq!(t.cursor_row, 4);
        assert_eq!(t.selected_rows(), vec![0]);
        // toggle modifiers are configurable
        let mut alt = members().toggle_mods(KeyModifiers::ALT);
        alt.on_click_cell_with(1, 0, p, KeyModifiers::NONE);
        alt.on_click_cell_with(2, 0, p, KeyModifiers::ALT);
        assert_eq!(alt.selected_rows(), vec![1, 2]);
        // Ctrl or Cmd off macOS: Alt+click selects only the row
        let mut def = members().toggle_mods(DataTable::toggle_mods_for(ClientPlatform::Other));
        def.on_click_cell_with(1, 0, p, KeyModifiers::NONE);
        def.on_click_cell_with(2, 0, p, KeyModifiers::ALT);
        assert_eq!(def.selected_rows(), vec![2]);
        // TOGGLE_WITH_ALT accepts Ctrl, Cmd and Option
        let mut all = members().toggle_mods(DataTable::TOGGLE_WITH_ALT);
        all.on_click_cell_with(1, 0, p, KeyModifiers::NONE);
        all.on_click_cell_with(2, 0, p, KeyModifiers::ALT);
        all.on_click_cell_with(3, 0, p, KeyModifiers::SUPER);
        all.on_click_cell_with(5, 0, p, KeyModifiers::CONTROL);
        assert_eq!(all.selected_rows(), vec![1, 2, 3, 5]);
    }

    #[test]
    fn toggle_click_default_is_option_on_macos_and_ctrl_elsewhere() {
        assert_eq!(
            DataTable::toggle_mods_for(ClientPlatform::MacOs),
            DataTable::TOGGLE_OPTION_CMD
        );
        assert_eq!(
            DataTable::toggle_mods_for(ClientPlatform::Other),
            DataTable::TOGGLE_CTRL_CMD
        );
        // the default follows the one shared client-platform detector
        assert_eq!(
            members().toggle_mods,
            DataTable::toggle_mods_for(ClientPlatform::current())
        );
        let p = Position::new(0, 0);
        // macOS: Option+click (or Cmd+click) toggles; a Ctrl+click that
        // still reaches the table (ctrl_click_secondary off) selects only
        let mut mac = members().toggle_mods(DataTable::toggle_mods_for(ClientPlatform::MacOs));
        mac.on_click_cell_with(1, 0, p, KeyModifiers::NONE);
        mac.on_click_cell_with(2, 0, p, KeyModifiers::ALT);
        mac.on_click_cell_with(3, 0, p, KeyModifiers::SUPER);
        assert_eq!(mac.selected_rows(), vec![1, 2, 3]);
        mac.on_click_cell_with(2, 0, p, KeyModifiers::ALT);
        assert_eq!(mac.selected_rows(), vec![1, 3], "Option+click toggles off");
        mac.on_click_cell_with(5, 0, p, KeyModifiers::CONTROL);
        assert_eq!(mac.selected_rows(), vec![5]);
        // elsewhere: Ctrl+click toggles, Alt+click selects only
        let mut other = members().toggle_mods(DataTable::toggle_mods_for(ClientPlatform::Other));
        other.on_click_cell_with(1, 0, p, KeyModifiers::NONE);
        other.on_click_cell_with(2, 0, p, KeyModifiers::CONTROL);
        assert_eq!(other.selected_rows(), vec![1, 2]);
        other.on_click_cell_with(3, 0, p, KeyModifiers::ALT);
        assert_eq!(other.selected_rows(), vec![3]);
    }

    #[test]
    fn checkbox_and_space_toggle_without_a_modifier_on_every_platform() {
        // the modified click is a shortcut; a terminal that keeps
        // Option+click (or an app that turns modifiers off) still toggles
        for mods in [
            DataTable::toggle_mods_for(ClientPlatform::MacOs),
            DataTable::toggle_mods_for(ClientPlatform::Other),
            KeyModifiers::NONE,
        ] {
            let mut t = members().selection_column(true).toggle_mods(mods);
            t.on_click_check(1);
            t.on_click_check(3);
            assert_eq!(t.selected_rows(), vec![1, 3], "{mods:?}");
            t.on_click_check(1);
            assert_eq!(t.selected_rows(), vec![3], "{mods:?}");
            t.cursor_row = 2;
            t.on_key(&k(KeyCode::Char(' '), KeyModifiers::NONE));
            assert_eq!(t.selected_rows(), vec![2, 3], "{mods:?}");
            t.on_key(&k(KeyCode::Char(' '), KeyModifiers::NONE));
            assert_eq!(t.selected_rows(), vec![3], "{mods:?}");
        }
    }

    #[test]
    fn checkbox_column_draws_tri_state_header_and_toggles_by_click() {
        let mut t = members().selection_column(true);
        let area = Rect::new(0, 0, 30, 8);
        let (buf, hits) = draw(&mut t, area);
        let boxed =
            |buf: &Buffer, y: u16| -> String { (3..6).map(|x| buf[(x, y)].symbol()).collect() };
        assert_eq!(boxed(&buf, 0), "[ ]");
        assert_eq!(hits.hit(Position::new(4, 0)), Some(t.check_all_id()));
        assert_eq!(hits.hit(Position::new(4, 2)), Some(t.check_id(1)));
        assert_ne!(
            hits.hit(Position::new(4, 5)),
            Some(t.check_id(4)),
            "a disabled row has no checkbox target"
        );
        assert!(t.owns(t.check_id(1)) && t.owns(t.check_all_id()));
        // cells start after the box
        assert_eq!(buf[(7, 1)].symbol(), "D");

        assert_eq!(t.locate_check(t.check_id(1)), Some(1));
        let (_, ev) = t.on_click_check(1);
        assert_eq!(ev, Some(TableAction::SelectionChanged));
        assert_eq!(t.cursor_row, 1);
        let (buf, _) = draw(&mut t, area);
        assert_eq!(boxed(&buf, 2), "[✓]");
        assert_eq!(boxed(&buf, 0), "[–]", "some selected: indeterminate");
        assert_eq!(
            buf[(1, 2)].symbol(),
            " ",
            "the box replaces the gutter mark"
        );

        t.on_click_check_all();
        assert_eq!(t.selection_state(), TriState::All);
        assert_eq!(t.selected_rows(), vec![0, 1, 2, 3, 5]);
        let (buf, _) = draw(&mut t, area);
        assert_eq!(boxed(&buf, 0), "[✓]");
        t.on_click_check_all();
        assert!(t.selected_rows().is_empty(), "all → none");
    }

    #[test]
    fn remove_rows_keeps_the_selection_on_the_remaining_rows() {
        let mut t = members();
        t.toggle_row(1);
        t.toggle_row(3);
        t.toggle_row(5);
        t.cursor_row = 3;
        t.remove_rows(&[1, 3]);
        assert_eq!(t.len(), 4);
        assert_eq!(t.selected_rows(), vec![3], "Fay moved from 5 to 3");
        assert_eq!(t.rows[3][0].text, "Fay");
        assert!(t.is_disabled(2), "Eli moved from 4 to 2 and stays disabled");
        assert_eq!(t.rows[t.source_row(t.cursor_row)][0].text, "Eli");
    }

    #[test]
    fn single_mode_keeps_its_chosen_row_marker() {
        let mut t = members().selection_mode(SelectionMode::Single);
        let (_, ev) = t.on_key(&k(KeyCode::Char(' '), KeyModifiers::NONE));
        assert_eq!(ev, Some(TableAction::Activated(0)));
        assert_eq!(t.selected_rows(), vec![0]);
        assert!(!t.select_all(), "select all is a multi-mode action");
        let mut none = members().selection_mode(SelectionMode::None);
        let (_, ev) = none.on_key(&k(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(ev, Some(TableAction::Activated(0)));
        assert!(none.selected_rows().is_empty());
    }

    #[test]
    fn set_row_resorts_and_keeps_selection_and_cursor() {
        let mut t = members();
        t.sort_by(0); // Ari, Bo, Cleo, Dana, Eli, Fay
        t.toggle_row(2); // Cleo
        t.cursor_row = 2;
        t.set_row(2, vec![Cell::new("Zed"), Cell::new("r2")]);
        assert_eq!(t.selected_rows(), vec![2], "the selection keeps the row");
        assert_eq!(t.source_row(5), 2, "re-sorted to the end");
        assert_eq!(t.cursor_row, 5, "the cursor follows its row");
    }
}

#[cfg(test)]
#[path = "table_choice_tests.rs"]
mod choice_tests;

/// Select All in a multi-selection table goes through its key map.
#[cfg(test)]
mod select_all_tests {
    use super::*;

    fn table(mode: SelectionMode) -> DataTable {
        DataTable::new(
            WidgetId::of("t"),
            vec![Column::new("Name", Constraint::Min(8))],
            ["a", "b", "c"]
                .iter()
                .map(|n| vec![Cell::new(*n)])
                .collect(),
        )
        .selection_mode(mode)
    }

    #[test]
    fn select_all_is_a_remappable_action() {
        let ctrl_a = Key {
            code: KeyCode::Char('a'),
            mods: KeyModifiers::CONTROL,
        };
        let mut single = table(SelectionMode::Single);
        assert_eq!(single.on_key(&ctrl_a).0, Outcome::Ignored);
        let mut t =
            table(SelectionMode::Multi).keymap(vec![(KeyBinding::alt('a'), TableCmd::SelectAll)]);
        assert_eq!(
            t.on_key(&ctrl_a).0,
            Outcome::Ignored,
            "the default is unbound"
        );
        assert!(t.selected_rows().is_empty());
        let (out, ev) = t.on_key(&Key {
            code: KeyCode::Char('a'),
            mods: KeyModifiers::ALT,
        });
        assert_eq!(out, Outcome::Changed);
        assert_eq!(ev, Some(TableAction::SelectionChanged));
        assert_eq!(t.selected_rows(), vec![0, 1, 2]);
    }
}
