//! `Grid` — the one tabular component (`COMPONENT_ARCHITECTURE.md` §12.3,
//! §17.0 A3/A7, §23 K2 (Adjudication K), Appendix A 4I).
//!
//! Capability is chosen by the **entry point**, never by a flag ([`Grid::update`]
//! for a read-only grid, [`Grid::update_editable`] for an editing one), so a
//! read-only grid is structurally incapable of mutating its model (§23 K2, G1
//! and G4). `GridCellActions` does not exist: its `actions` is a defaulted
//! [`GridModel`] method, because `draw` paints the affordance and must see it
//! (§23 K2, G3).
//!
//! ## Sorting is a request, never a permutation
//!
//! The grid paints sortable headers and emits [`GridAction::Sort`]. The model
//! adapter owns display order and every domain comparison (including numeric
//! and NULL ordering); this module contains no order vector, comparator or
//! rendered-text fallback (§52). Rows are addressed by [`ItemKey`] and columns
//! by [`ColumnKey`], so an adapter reorder leaves cursor, range, selection and
//! pending edit bound to the same logical cells.

use core::fmt;

use ratatui_core::layout::Rect;
use ratatui_core::style::Style;

use super::input::{TextAction, TextInput, TextInputState};
use super::scroll_region::ScrollRegion;
use super::{Acc, PartStyle, SlotFn};
use crate::action::ActionKey;
use crate::collection::{
    CellDecor, CollectionCore, EmptyState, KeySet, Reconcile, Reconciliation, RowDecor, RowTotal,
    SelectMode,
};
use crate::event::{Chord, KeyCode, KeyModifiers};
use crate::focus::Focusability;
use crate::id::{Id, ItemKey, Part, PartRef, custom_hash16};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::text::width;
use crate::theme::{Align, Family, GlyphRole, Modifier, Role, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};

const CTRL: KeyModifiers = KeyModifiers::CONTROL;
const SHIFT: KeyModifiers = KeyModifiers::SHIFT;

/// A stable key for one column.
///
/// The column analogue of [`ItemKey`]: every action names a column by key,
/// never by index, so a model that reorders or hides columns cannot silently
/// move the cursor to a different one. `0..=0x7FFF` is the numbered range for
/// callers that have a natural numbering; [`ColumnKey::of`] hashes a name
/// into the disjoint high range with the same fold every other 16-bit custom
/// key in the crate uses.
///
/// The two ranges cannot meet: [`ColumnKey::num`] *rejects* a number that
/// would land in the hashed half rather than masking it into a different
/// column. Two hashed names can still fold together, and a grid checks its
/// own column list for that in debug builds ([`Grid::draw`]), because the
/// column list is the only place both names exist at once.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ColumnKey(u16);

/// The largest [`ColumnKey::num`]; `0x8000..` is [`ColumnKey::of`]'s range.
const COLUMN_KEY_MAX: u16 = 0x7FFF;

impl ColumnKey {
    /// A numbered column key, `0..=0x7FFF`.
    ///
    /// # Panics
    /// If `n > 0x7FFF`. Masking instead would silently alias one
    /// column onto another (`0x8000` onto column `0`) and, worse, onto
    /// [`ColumnKey::of`]'s range; in a `const` this panic is a compile error.
    pub const fn num(n: u16) -> Self {
        assert!(
            n <= COLUMN_KEY_MAX,
            "ColumnKey::num is 0..=0x7FFF; 0x8000.. belongs to ColumnKey::of"
        );
        ColumnKey(n)
    }

    /// A named column key: the crate's FNV-1a fold into the high range.
    pub const fn of(name: &str) -> Self {
        ColumnKey(custom_hash16(name))
    }

    /// The raw key.
    pub const fn raw(self) -> u16 {
        self.0
    }
}

impl fmt::Debug for ColumnKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 > COLUMN_KEY_MAX {
            write!(f, "ColumnKey::of(#{:04x})", self.0)
        } else {
            write!(f, "ColumnKey::num({})", self.0)
        }
    }
}

/// What the cursor moves over.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum NavUnit {
    /// Whole rows: `←`/`→` do not move the cursor and `Enter` activates.
    Row,
    /// Individual cells: `←`/`→` move between columns.
    #[default]
    Cell,
}

/// One cell, borrowed from the model.
///
/// Borrowed, never owned: the 500 × 12 load benchmark's budget is one owned
/// conversion for the whole result, so the per-cell type may not allocate
/// (§16.6 `grid_500x12_load`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CellRef<'a> {
    /// The text to paint.
    pub text: &'a str,
    /// A foreground role, overriding the cell recipe.
    pub tone: Option<Role>,
    /// Horizontal alignment override. `None` inherits the column alignment.
    pub align: Option<Align>,
}

impl<'a> CellRef<'a> {
    /// A cell with no tone that inherits its column's alignment.
    pub const fn new(text: &'a str) -> Self {
        CellRef {
            text,
            tone: None,
            align: None,
        }
    }

    /// Set the alignment.
    #[must_use]
    pub const fn align(mut self, a: Align) -> Self {
        self.align = Some(a);
        self
    }

    /// Set the tone.
    #[must_use]
    pub const fn tone(mut self, r: Role) -> Self {
        self.tone = Some(r);
        self
    }
}

impl Default for CellRef<'_> {
    fn default() -> Self {
        CellRef::new("")
    }
}

/// One affordance offered on a cell (§23 K2, G3).
///
/// Absorbed from the deleted `GridCellActions`: `draw` paints the affordance
/// and registers its hot zone, so it must be reachable from `GridModel`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CellAction {
    /// The action's identity, carried back in [`GridAction::CellAction`].
    pub key: ActionKey,
    /// The glyph painted at the right edge of the cell.
    pub glyph: GlyphRole,
    /// A chord the owning screen advertises for it; the grid does not bind it.
    pub chord: Option<Chord>,
}

impl CellAction {
    /// A follow affordance for `key`.
    pub const fn new(key: ActionKey) -> Self {
        CellAction {
            key,
            glyph: GlyphRole::FollowRef,
            chord: None,
        }
    }

    /// Set the glyph.
    #[must_use]
    pub const fn glyph(mut self, g: GlyphRole) -> Self {
        self.glyph = g;
        self
    }

    /// Advertise a chord.
    #[must_use]
    pub const fn chord(mut self, c: Chord) -> Self {
        self.chord = Some(c);
        self
    }
}

/// What editing one cell means (§21 item 30 A8, §23 K2, G5).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EditIntent<'a> {
    /// Open an inline editor seeded with `initial`.
    Inline {
        /// The text the editor starts from.
        initial: &'a str,
    },
    /// Advance the cell through a closed set of values, with no editor.
    Cycle,
    /// Emit [`GridAction::EditRequested`] and begin **no** inline edit; the
    /// application opens its own editor.
    External,
    /// Refuse, with a reason the grid shows.
    Refuse {
        /// Why the cell cannot be edited.
        reason: &'a str,
    },
}

/// One column of a [`Grid`].
///
/// Public fields with a `const` constructor, the shape [`RowDecor`] and
/// [`CellDecor`] already use: an adapter builds column literals with struct
/// update syntax, and no builder method is named `editable`, because
/// capability on the *grid* is chosen by the entry point (§23 K2, G4) and a
/// method of that name would be indistinguishable from the deleted one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Column<'a> {
    /// Stable identity; every action names this, never an index.
    pub key: ColumnKey,
    /// The header title.
    pub title: &'a str,
    /// A second header line's worth of detail, shown beside the title when
    /// the column is wide enough.
    pub subtitle: Option<&'a str>,
    /// Cell alignment when the model does not override it.
    pub align: Align,
    /// Narrowest painted width.
    pub min_width: u16,
    /// Widest painted width.
    pub max_width: u16,
    /// Whether activating the header requests a sort.
    pub sortable: bool,
    /// Whether cells in this column can be edited at all. A per-column
    /// property of the *data*; the model's `is_editable` still decides per
    /// cell, and a read-only grid can never reach either.
    pub editable: bool,
    /// Pinned to the left: never scrolled out of the column window.
    pub sticky: bool,
    /// A glyph painted before the cell text.
    pub prefix_glyph: Option<GlyphRole>,
    /// A short badge painted after the title.
    pub badge: Option<&'a str>,
}

impl<'a> Column<'a> {
    /// A left-aligned, non-editing, non-sticky column.
    pub const fn new(key: ColumnKey, title: &'a str) -> Self {
        Column {
            key,
            title,
            subtitle: None,
            align: Align::Left,
            min_width: 3,
            max_width: 40,
            sortable: false,
            editable: false,
            sticky: false,
            prefix_glyph: None,
            badge: None,
        }
    }
}

/// Direction requested from a sortable column header.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SortDir {
    /// Ascending domain order.
    Asc,
    /// Descending domain order.
    Desc,
}

/// The rows a [`Grid`] paints (§12.3, §23 K2).
///
/// Three required methods. `read_only_reason` and `actions` live here, not on
/// [`GridEditor`], because `draw` renders both and is bound by this trait
/// (§23 K2, G3).
pub trait GridModel {
    /// The number of rows.
    fn row_count(&self) -> usize;

    /// The stable key of row `row`.
    fn row_key(&self, row: usize) -> ItemKey;

    /// One cell, borrowed, or `None` for a blank cell.
    ///
    /// Columns are the sole schema. A missing cell remains addressable for
    /// rectangular navigation, but has no decoration, actions or editor
    /// hooks.
    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>>;

    /// Owner-supplied row decoration.
    fn row_decor(&self, _row: usize) -> RowDecor<'_> {
        RowDecor::default()
    }

    /// Owner-supplied cell decoration.
    fn cell_decor(&self, _row: usize, _col: usize) -> CellDecor<'_> {
        CellDecor::default()
    }

    /// How many rows the source has, when that is more than are loaded.
    fn total(&self) -> RowTotal {
        RowTotal::Unknown
    }

    /// Whether more rows can be fetched.
    fn has_more(&self) -> bool {
        false
    }

    /// Why this grid cannot be edited, if it cannot.
    ///
    /// On `GridModel` and not on [`GridEditor`] because the reason is
    /// *rendered*, and `draw` is bound by this trait (§23 K2, G3).
    fn read_only_reason(&self) -> Option<&str> {
        None
    }

    /// The affordances offered on one cell.
    ///
    /// Absorbed from the deleted `GridCellActions` (§23 K2, G3); the `&[]`
    /// default keeps a display-only model to three methods.
    fn actions(&self, _row: usize, _col: usize) -> &[CellAction] {
        &[]
    }
}

/// The editing half, reachable **only** from [`Grid::update_editable`]
/// (§23 K2, G2).
///
/// With `draw`'s `&self` and `&GridState`, and `update`'s `&M`, "rendering
/// stages a mutation" is unrepresentable.
pub trait GridEditor: GridModel {
    /// What editing this cell means.
    fn edit_intent(&self, row: usize, col: usize) -> EditIntent<'_>;

    /// Advance a `Cycle` cell to its next value.
    fn apply_cycle(&mut self, row: usize, col: usize);

    /// Write `text` to a cell.
    ///
    /// # Errors
    /// The model's own validation error; the inline editor stays open and
    /// shows it.
    fn commit_cell(
        &mut self,
        row: usize,
        col: usize,
        text: &str,
    ) -> Result<(), crate::validate::FieldError>;

    /// Whether this cell can be edited right now.
    fn is_editable(&self, row: usize, col: usize) -> bool;
}

/// What a grid reports. Every action carries keys, never indices.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum GridAction {
    /// The cursor moved.
    Moved,
    /// A row was activated (`Enter` under [`NavUnit::Row`], double-click).
    Activated(ItemKey),
    /// A sortable header requested adapter-owned ordering.
    Sort(ColumnKey, SortDir),
    /// The fetch-more row was reached or clicked.
    FetchMore,
    /// The selection was copied as TSV.
    Copy(String),
    /// `EditIntent::External`: the application opens its own editor (G5).
    EditRequested(ItemKey, ColumnKey),
    /// A cell affordance was activated.
    CellAction(ItemKey, ColumnKey, ActionKey),
    /// `Tab` past the last cell.
    LeaveForward,
    /// `Shift+Tab` before the first cell.
    LeaveBackward,
}

/// The const-constructible commands of the grid keymap.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GridCmd {
    /// Cursor up.
    Up,
    /// Cursor down.
    Down,
    /// Cursor left one column.
    Left,
    /// Cursor right one column.
    Right,
    /// Cursor up one viewport.
    PageUp,
    /// Cursor down one viewport.
    PageDown,
    /// Cursor to the first column of the row.
    RowStart,
    /// Cursor to the last column of the row.
    RowEnd,
    /// Cursor to the first row.
    First,
    /// Cursor to the last row.
    Last,
    /// Extend the rectangular range up.
    ExtendUp,
    /// Extend the rectangular range down.
    ExtendDown,
    /// Extend the rectangular range left.
    ExtendLeft,
    /// Extend the rectangular range right.
    ExtendRight,
    /// Toggle the cursor row's selection.
    ToggleRow,
    /// Select or clear every row.
    ToggleAll,
    /// Activate the row, or begin editing the cell.
    Activate,
    /// Begin editing the cursor cell.
    BeginEdit,
    /// Copy the selection as TSV.
    Copy,
}

const fn b(
    action: &'static str,
    chord: Chord,
    cmd: GridCmd,
    label: &'static str,
    visible: bool,
) -> Binding<GridCmd> {
    Binding {
        action: ActionKey::custom(action),
        chord: Some(chord),
        cmd,
        label,
        priority: if visible { 60 } else { 10 },
        visible,
    }
}

/// The one table. The grid is a single focus stop, so it has one binding
/// state; the edit lifecycle runs on the inline editor's own table.
const BINDINGS: [Binding<GridCmd>; 25] = [
    b("grid.up", Chord::key(KeyCode::Up), GridCmd::Up, "Up", true),
    b(
        "grid.down",
        Chord::key(KeyCode::Down),
        GridCmd::Down,
        "Down",
        true,
    ),
    b(
        "grid.left",
        Chord::key(KeyCode::Left),
        GridCmd::Left,
        "Left",
        true,
    ),
    b(
        "grid.right",
        Chord::key(KeyCode::Right),
        GridCmd::Right,
        "Right",
        true,
    ),
    b(
        "grid.up-vim",
        Chord::key(KeyCode::Char('k')),
        GridCmd::Up,
        "Up",
        false,
    ),
    b(
        "grid.down-vim",
        Chord::key(KeyCode::Char('j')),
        GridCmd::Down,
        "Down",
        false,
    ),
    b(
        "grid.left-vim",
        Chord::key(KeyCode::Char('h')),
        GridCmd::Left,
        "Left",
        false,
    ),
    b(
        "grid.right-vim",
        Chord::key(KeyCode::Char('l')),
        GridCmd::Right,
        "Right",
        false,
    ),
    b(
        "grid.page-up",
        Chord::key(KeyCode::PageUp),
        GridCmd::PageUp,
        "Page up",
        false,
    ),
    b(
        "grid.page-down",
        Chord::key(KeyCode::PageDown),
        GridCmd::PageDown,
        "Page down",
        false,
    ),
    b(
        "grid.row-start",
        Chord::key(KeyCode::Home),
        GridCmd::RowStart,
        "Row start",
        false,
    ),
    b(
        "grid.row-end",
        Chord::key(KeyCode::End),
        GridCmd::RowEnd,
        "Row end",
        false,
    ),
    b(
        "grid.first",
        Chord::with(KeyCode::Home, CTRL),
        GridCmd::First,
        "First row",
        false,
    ),
    b(
        "grid.last",
        Chord::with(KeyCode::End, CTRL),
        GridCmd::Last,
        "Last row",
        false,
    ),
    b(
        "grid.first-vim",
        Chord::key(KeyCode::Char('g')),
        GridCmd::First,
        "First row",
        false,
    ),
    b(
        "grid.last-vim",
        Chord::key(KeyCode::Char('G')),
        GridCmd::Last,
        "Last row",
        false,
    ),
    b(
        "grid.extend-up",
        Chord::with(KeyCode::Up, SHIFT),
        GridCmd::ExtendUp,
        "Extend up",
        false,
    ),
    b(
        "grid.extend-down",
        Chord::with(KeyCode::Down, SHIFT),
        GridCmd::ExtendDown,
        "Extend down",
        false,
    ),
    b(
        "grid.extend-left",
        Chord::with(KeyCode::Left, SHIFT),
        GridCmd::ExtendLeft,
        "Extend left",
        false,
    ),
    b(
        "grid.extend-right",
        Chord::with(KeyCode::Right, SHIFT),
        GridCmd::ExtendRight,
        "Extend right",
        false,
    ),
    b(
        "grid.toggle-row",
        Chord::key(KeyCode::Char(' ')),
        GridCmd::ToggleRow,
        "Select row",
        true,
    ),
    b(
        "grid.toggle-all",
        Chord::with(KeyCode::Char('a'), CTRL),
        GridCmd::ToggleAll,
        "All",
        false,
    ),
    b(
        "grid.activate",
        Chord::key(KeyCode::Enter),
        GridCmd::Activate,
        "Open",
        true,
    ),
    b(
        "grid.begin-edit",
        Chord::key(KeyCode::F(2)),
        GridCmd::BeginEdit,
        "Edit",
        false,
    ),
    b(
        "grid.copy",
        Chord::with(KeyCode::Char('c'), CTRL),
        GridCmd::Copy,
        "Copy",
        false,
    ),
];

/// The largest number of columns a grid lays out.
///
/// The whole column geometry lives in two fixed arrays of this length, which
/// is what keeps `draw` allocation-free (§16.6 `grid_500x12_render`, **< 100**
/// allocations per frame). Columns beyond the cap are **not painted** and
/// never reachable by the cursor; a model with more columns than this is a
/// design error, not a runtime one.
pub const GRID_MAX_COLUMNS: usize = 64;

/// Durable state of a [`Grid`].
///
/// Holds the cursor cell, the rectangular range anchor, the row selection,
/// the two-axis window, the reconcile stamp and the inline edit lifecycle —
/// all keyed. `Debug` redacts the editor's draft, which `TextInputState`
/// already does.
///
/// Its public readers expose only durable state owned by the grid (§52), not
/// model indices, editor drafts or derived viewport geometry.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GridState {
    /// Row cursor key, row selection, vertical scroll and the stamp.
    core: CollectionCore,
    /// Cursor column, keyed; the index is a cache re-derived every phase.
    col: Option<ColumnKey>,
    col_index: usize,
    /// The rectangular range anchor, keyed on both axes.
    anchor: Option<(ItemKey, ColumnKey)>,
    /// First non-sticky column shown.
    col_offset: usize,
    /// The cell being edited, keyed.
    edit: Option<(ItemKey, ColumnKey)>,
    /// The inline editor's draft, phase and error.
    editor: TextInputState,
    /// Last requested sort, used only to alternate the header affordance.
    /// The adapter remains the sole owner of the actual row permutation.
    sort: Option<(ColumnKey, SortDir)>,
}

impl Default for GridState {
    fn default() -> Self {
        let mut editor = TextInputState::default();
        editor.set_sensitive(false);
        Self {
            core: CollectionCore::default(),
            col: None,
            col_index: 0,
            anchor: None,
            col_offset: 0,
            edit: None,
            editor,
            sort: None,
        }
    }
}

impl GridState {
    /// The keyed cursor cell.
    pub const fn cursor(&self) -> Option<(ItemKey, ColumnKey)> {
        match (self.core.cursor(), self.col) {
            (Some(row), Some(col)) => Some((row, col)),
            _ => None,
        }
    }

    /// The keyed row selection.
    pub const fn selected_rows(&self) -> &KeySet {
        self.core.checked()
    }

    /// Whether an inline edit is active.
    pub const fn is_editing(&self) -> bool {
        self.edit.is_some()
    }

    /// The typed error retained by the inline editor.
    pub const fn edit_error(&self) -> Option<&crate::validate::FieldError> {
        self.editor.error()
    }

    /// Number of non-sticky columns hidden on the left.
    pub const fn col_offset(&self) -> usize {
        self.col_offset
    }

    /// Point the cursor at `(row, key)` in `col`, and reveal it.
    fn set_cursor(&mut self, row: usize, key: ItemKey, col_index: usize, col: ColumnKey) {
        self.core.set_cursor(row, key);
        self.col_index = col_index;
        self.col = Some(col);
    }

    /// Cancel the inline editor and discard any cell-local error.
    ///
    /// `TextInputState::cancel` preserves plain validation errors for callers
    /// that want to display them after a commit. A Grid error belongs to the
    /// edited cell, so retaining it after that cell is gone or the edit is
    /// cancelled would paint stale text on the next cursor cell.
    fn cancel_editor(&mut self) {
        self.edit = None;
        self.editor.cancel();
        self.editor.set_error(None);
    }
}

impl Reconcile for GridState {
    fn reconcile(&mut self, len: usize, key: impl Fn(usize) -> ItemKey) -> Reconciliation {
        let r = self.core.reconcile(len, &key);
        if let Some((a, _)) = self.anchor
            && !(0..len).any(|i| key(i) == a)
        {
            self.anchor = None;
        }
        if let Some((e, _)) = self.edit
            && !(0..len).any(|i| key(i) == e)
        {
            self.cancel_editor();
        }
        r
    }

    fn invalidate(&mut self) {
        self.core.invalidate();
    }
}

/// The column geometry of one frame: two fixed arrays and a window, computed
/// identically by both phases from `(area, columns, state, model)`.
#[derive(Clone, Copy, Debug)]
struct Geometry {
    /// Painted width of each column.
    width: [u16; GRID_MAX_COLUMNS],
    /// Painted `x` of each column; `0` for a column outside the window.
    x: [u16; GRID_MAX_COLUMNS],
    /// Whether each column is painted this frame.
    shown: [bool; GRID_MAX_COLUMNS],
    /// Declared columns, capped at [`GRID_MAX_COLUMNS`].
    n: usize,
    /// Non-sticky columns hidden to the left and to the right.
    hidden_left: usize,
    hidden_right: usize,
    /// The rect the rows are laid into (gutter and marker included).
    body: Rect,
    /// The first column of cell content, past the gutter and marker.
    content_x: u16,
}

struct RowPaint<'a, M: ?Sized> {
    content: Rect,
    geometry: &'a Geometry,
    state: &'a GridState,
    model: &'a M,
    cursor: (usize, usize),
    range: Option<((usize, usize), (usize, usize))>,
    live: StateFlags,
}

impl Geometry {
    const fn empty(body: Rect) -> Self {
        Geometry {
            width: [0; GRID_MAX_COLUMNS],
            x: [0; GRID_MAX_COLUMNS],
            shown: [false; GRID_MAX_COLUMNS],
            n: 0,
            hidden_left: 0,
            hidden_right: 0,
            body,
            content_x: body.x,
        }
    }

    /// The rect of column `i` on row `y`, or an empty rect.
    fn cell(&self, i: usize, y: u16) -> Rect {
        if !self.shown.get(i).copied().unwrap_or(false) {
            return Rect::ZERO;
        }
        Rect {
            x: self.x.get(i).copied().unwrap_or(0),
            y,
            width: self.width.get(i).copied().unwrap_or(0),
            height: 1,
        }
        .intersection(Rect {
            y,
            height: 1,
            ..self.body
        })
    }

    /// The column under `x`, if any.
    fn column_at(&self, x: u16) -> Option<usize> {
        (0..self.n).find(|&i| {
            let r = self.cell(i, self.body.y);
            r.width > 0 && x >= r.x && x < r.x.saturating_add(r.width)
        })
    }
}

/// A two-axis, keyed, editable-by-entry-point table over a borrowed model.
///
/// ## Construction
/// `Grid::new(id, columns)`; the model is passed to each phase, never held
/// (§21 item 1, B15). `Grid::update` takes `&M: GridModel`,
/// `Grid::update_editable` takes `&mut M: GridEditor`, `Grid::draw` takes
/// `&M: GridModel` — the same bound and the same shared borrow as `update`.
///
/// ## Ownership
/// The caller owns the columns, the model and a [`GridState`]; the runtime
/// owns focus, hover, press, wheel routing and the scrollbar capture. The
/// inline editor is a child component addressed by `id.part(Part::TEXT)`.
///
/// ## Configuration
/// `.nav(NavUnit)` (`Cell`), `.select_mode(SelectMode)` (`Single`),
/// `.empty(EmptyState)` (a default "Nothing here yet"), `.actions_slot(&dyn
/// Fn(&mut Ui, Rect))`, `.patch`, `.patch_part`, `.slot`,
/// There is **no** `.editable(bool)` (§23 K2, G4).
///
/// ## Variants
/// `Family::GRID`, `DEFAULT` only.
///
/// ## States
/// The grid wears `FOCUSED`, `FOCUS_VISIBLE` and `HOVERED` from the
/// runtime and `READ_ONLY` when the model gives a reason. The cursor row
/// derives `FOCUSED`, the cursor cell `ACTIVE`, a selected row `SELECTED`, a
/// checked row `CHECKED`, a cell in the range `SELECTED`, an edited cell
/// `EDITING`, and `ERROR`/`DIRTY` from [`CellDecor`]. Empty loading and error
/// states are explicit [`EmptyState`] values.
///
/// ## Actions
/// [`GridAction`]. Sort is a keyed adapter request; filter, refresh and
/// row-lifecycle behavior remains in application bindings.
///
/// ## Focus
/// One `Focusable` stop for the whole grid. While a cell is being edited the
/// inline editor holds focus and swallows typing, so the grid's own table
/// stops firing without any flag.
///
/// ## Keyboard
/// `↑`/`k`, `↓`/`j`, `←`/`h`, `→`/`l`, `PgUp`, `PgDn`, `Home`/`End` (row
/// ends), `Ctrl+Home`/`Ctrl+End` and `g`/`G` (first / last row),
/// `Shift+arrows` (extend the rectangular range), `Space` (select the row),
/// `Ctrl+A` (all), `Enter` (activate, or begin an edit under
/// `update_editable`), `F2` (begin an edit), `Ctrl+C` (copy as TSV).
///
/// ## Mouse
/// `PartRef::item(Part::CELL, row)`: press moves the cursor to the cell under
/// the pointer, double-click activates or begins an edit.
/// `PartRef::item(Part::ACTIONS, row)`: the cell affordance's hot zone, which
/// is registered after the cell and therefore wins the click.
/// `PartRef::of(Part::ROW)` is the fetch-more row. `TRACK`/`THUMB` and the
/// wheel go to the embedded [`ScrollRegion`].
///
/// ## Layout
/// A header row, an optional read-only reason row, the body, and an optional
/// action row. Column widths are **sampled** from the visible rows and the
/// titles, clamped into each column's `min_width..=max_width`; sticky columns
/// are pinned to the left and the rest scroll in a window with `‹N` / `N›`
/// overflow indicators. `measure` is `(the sampled total, rows + chrome)`;
/// `draw` returns `area`. `0×0` registers nothing (R5).
///
/// ## Parts
/// `CONTAINER` (the whole surface, filled on **every** non-degenerate frame,
/// which is why it is `PARTS[0]`), `HEADER`, `ROW`, `CELL`, `TRACK`, `THUMB`,
/// `OVERFLOW`, `EMPTY`, `ACTIONS` — exactly §17.0 A7's list. The focus gutter
/// and the selection marker are painted from the `ROW` resolution rather than
/// resolving `GUTTER` and `MARKER`, because `PARTS` is what `draw` resolves
/// and nothing more (§33, Invariant P).
///
/// ## Overrides
/// `.patch` and `.patch_part` reach `Part::CONTAINER`, `Part::HEADER`,
/// `Part::ROW`, `Part::CELL`, `Part::TRACK`, `Part::THUMB`, `Part::OVERFLOW`,
/// `Part::EMPTY` and `Part::ACTIONS`. `.slot` replaces `Part::HEADER`,
/// `Part::EMPTY`, `Part::ACTIONS`, `Part::TRACK` and `Part::THUMB`.
/// `ACTIONS` reaches both configured action surfaces and cell affordances.
/// The last two are forwarded into the embedded [`ScrollRegion`], as are
/// `.patch` and `.patch_part`.
///
/// ## Identity
/// Rows are [`ItemKey`]s from `GridModel::row_key`, columns are
/// [`ColumnKey`]s from the column list. The cursor, the range anchor, the
/// selection and the edited cell are **all** stored keyed and re-resolved to
/// indices every phase, so a model that reorders itself between frames moves
/// none of them.
///
/// ## Testing
/// `GridCase` with `ACTIVATES | FOCUSABLE | COLLECTION | SCROLLS | EDITS`,
/// registered once and run through both entry points from `Fixture.read_only`
/// (§23 K2); `crates/tui/tests/fixtures/grid_model.rs` is a display-only model
/// that implements `GridModel` **alone**, so it is a compile-time witness that
/// the read-only entry point needs nothing from `GridEditor`.
///
/// ## Invariants
/// `reconcile` runs before any action is emitted (G7); only visible cells
/// invoke the model; a frame allocates nothing per row or per cell; `draw`
/// takes `&GridState` and `&M`, so it can neither commit nor cancel an edit
/// (G2).
pub struct Grid<'a> {
    id: Id,
    columns: &'a [Column<'a>],
    nav: NavUnit,
    select_mode: SelectMode,
    empty: Option<EmptyState<'a>>,
    actions: Option<SlotFn<'a>>,
    ov: PartStyle<'a>,
}

impl fmt::Debug for Grid<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Grid")
            .field("id", &self.id)
            .field("columns", &self.columns.len())
            .field("nav", &self.nav)
            .field("select_mode", &self.select_mode)
            .field("empty", &self.empty)
            .field("overrides", &self.ov)
            .finish_non_exhaustive()
    }
}

impl<'a> Grid<'a> {
    /// The parts this component styles (§17.0 A7).
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::HEADER,
        Part::ROW,
        Part::CELL,
        Part::TRACK,
        Part::THUMB,
        Part::OVERFLOW,
        Part::EMPTY,
        Part::ACTIONS,
    ];

    /// A grid over `columns`.
    pub const fn new(id: Id, columns: &'a [Column<'a>]) -> Self {
        Grid {
            id,
            columns,
            nav: NavUnit::Cell,
            select_mode: SelectMode::Single,
            empty: None,
            actions: None,
            ov: PartStyle::new(),
        }
    }

    /// The id.
    pub const fn id(&self) -> Id {
        self.id
    }

    /// The id of the inline editor this grid owns.
    pub const fn editor_id(&self) -> Id {
        self.id.part(Part::TEXT)
    }

    /// What the cursor moves over.
    #[must_use]
    pub const fn nav(mut self, u: NavUnit) -> Self {
        self.nav = u;
        self
    }

    /// How rows are selected.
    #[must_use]
    pub const fn select_mode(mut self, m: SelectMode) -> Self {
        self.select_mode = m;
        self
    }

    /// What to paint when there are no rows.
    #[must_use]
    pub const fn empty(mut self, e: EmptyState<'a>) -> Self {
        self.empty = Some(e);
        self
    }

    /// A row of application affordances under the grid (§12.3's action
    /// surface). The grid reserves the row and registers nothing in it.
    #[must_use]
    pub const fn actions_slot(mut self, f: SlotFn<'a>) -> Self {
        self.actions = Some(f);
        self
    }

    /// An instance patch over every part (precedence 6).
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    /// Per-part patches.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    /// Replace one part's painting.
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    /// The embedded scroll region carrying every owning override (§45.1).
    fn bar(&self) -> ScrollRegion<'a> {
        let mut r = ScrollRegion::new(self.id).patch_part(self.ov.parts);
        if let Some(p) = self.ov.patch {
            r = r.patch(p);
        }
        if let Some((part, f)) = self.ov.slot {
            r = r.slot(part, f);
        }
        r
    }

    /// Declared columns, capped at [`GRID_MAX_COLUMNS`].
    fn column_count(&self) -> usize {
        self.columns.len().min(GRID_MAX_COLUMNS)
    }

    /// Debug-only: the column list must hold no key twice.
    ///
    /// Keyed identity (§33) assumes one column per key: [`Self::col_index`]
    /// resolves the cursor, the range anchor and the edited cell to the
    /// *first* column carrying a key, so a repeated key — a copy-pasted
    /// [`ColumnKey::num`], or two [`ColumnKey::of`] names folding to the same
    /// 15 bits — moves them to a column the caller never meant. Nothing
    /// downstream can see that; the column list is the one place both keys
    /// and both titles exist, so the check belongs here. Debug builds only.
    fn assert_distinct_column_keys(&self) {
        #[cfg(debug_assertions)]
        {
            let shown = self.columns.get(..self.column_count()).unwrap_or(&[]);
            for (i, a) in shown.iter().enumerate() {
                for b in shown.get(i.saturating_add(1)..).unwrap_or(&[]) {
                    assert!(
                        a.key != b.key,
                        "grid {:?}: columns {:?} and {:?} share {:?}",
                        self.id,
                        a.title,
                        b.title,
                        a.key
                    );
                }
            }
        }
    }

    /// The index of `key` in the capped column list.
    fn col_index(&self, key: ColumnKey) -> Option<usize> {
        self.columns
            .get(..self.column_count())
            .and_then(|cs| cs.iter().position(|c| c.key == key))
    }

    /// The key of column `i`.
    fn col_key(&self, i: usize) -> Option<ColumnKey> {
        self.columns.get(i).map(|c| c.key)
    }

    /// The cursor column index, re-derived from the stored key so a column
    /// reorder cannot move it (§33: identity is keyed, not positional).
    fn cursor_col(&self, st: &GridState) -> usize {
        let column_count = self.column_count();
        st.col
            .and_then(|k| self.col_index(k))
            .unwrap_or_else(|| st.col_index.min(column_count.saturating_sub(1)))
    }

    /// The chrome rows: header, read-only reason, action surface.
    fn chrome(&self, area: Rect, reason: Option<&str>) -> (Rect, Rect, Rect, Rect) {
        let header = Rect {
            height: u16::from(!self.columns.is_empty()),
            ..area
        };
        let rest = Rect {
            y: area.y.saturating_add(header.height),
            height: area.height.saturating_sub(header.height),
            ..area
        };
        let note = Rect {
            height: u16::from(reason.is_some()).min(rest.height),
            ..rest
        };
        let rest = Rect {
            y: rest.y.saturating_add(note.height),
            height: rest.height.saturating_sub(note.height),
            ..rest
        };
        let bar = Rect {
            y: rest.bottom().saturating_sub(1),
            height: u16::from(self.actions.is_some()).min(rest.height),
            ..rest
        };
        let body = Rect {
            height: rest.height.saturating_sub(bar.height),
            ..rest
        };
        (header, note, body, bar)
    }

    /// Sample column widths and place the window. Pure in
    /// `(body, columns, st, model, rows)`; both phases call it, so a pointer
    /// resolved in `update` lands on the column `draw` painted.
    fn geometry<M: GridModel + ?Sized>(
        &self,
        body: Rect,
        st: &GridState,
        model: &M,
        rows: core::ops::Range<usize>,
    ) -> Geometry {
        let mut g = Geometry::empty(body);
        g.n = self.column_count();
        g.content_x = body.x.saturating_add(2);
        if g.n == 0 || body.is_empty() {
            return g;
        }
        let avail = body.width.saturating_sub(2);
        if avail == 0 {
            g.hidden_right = g.n;
            return g;
        }
        for (i, c) in self.columns.iter().enumerate().take(g.n) {
            let mut w = width(c.title);
            let mut has_actions = false;
            if let Some(b) = c.badge {
                w = w.saturating_add(width(b)).saturating_add(1);
            }
            if let Some(s) = c.subtitle {
                w = w.max(width(s));
            }
            for r in rows.clone() {
                if r >= model.row_count() {
                    break;
                }
                if let Some(cell) = model.cell(r, i) {
                    w = w.max(width(cell.text));
                    has_actions |= !model.actions(r, i).is_empty();
                }
            }
            if c.prefix_glyph.is_some() {
                w = w.saturating_add(2);
            }
            if has_actions {
                w = w.saturating_add(2);
            }
            if let Some(slot) = g.width.get_mut(i) {
                *slot = w.clamp(c.min_width.max(1), c.max_width.max(c.min_width).max(1));
            }
        }
        // sticky columns first, then a window over the rest
        let gap = 1u16;
        let mut x = g.content_x;
        let mut used = 0u16;
        for i in 0..g.n {
            if !self.columns.get(i).is_some_and(|c| c.sticky) {
                continue;
            }
            if used >= avail {
                g.hidden_right = g.hidden_right.saturating_add(1);
                continue;
            }
            let w = g.width.get(i).copied().unwrap_or(0);
            if let (Some(px), Some(sh)) = (g.x.get_mut(i), g.shown.get_mut(i)) {
                *px = x;
                *sh = true;
            }
            x = x.saturating_add(w).saturating_add(gap);
            used = used.saturating_add(w).saturating_add(gap);
        }
        let first = st.col_offset;
        let mut seen = 0usize;
        let mut last_shown = 0usize;
        for i in 0..g.n {
            if self.columns.get(i).is_some_and(|c| c.sticky) {
                continue;
            }
            if seen < first {
                seen = seen.saturating_add(1);
                g.hidden_left = g.hidden_left.saturating_add(1);
                continue;
            }
            let w = g.width.get(i).copied().unwrap_or(0);
            if used >= avail || (used.saturating_add(w) > avail && last_shown > 0) {
                g.hidden_right = g.hidden_right.saturating_add(1);
                continue;
            }
            if let (Some(px), Some(sh)) = (g.x.get_mut(i), g.shown.get_mut(i)) {
                *px = x;
                *sh = true;
            }
            last_shown = last_shown.saturating_add(1);
            x = x.saturating_add(w).saturating_add(gap);
            used = used.saturating_add(w).saturating_add(gap);
        }
        g
    }
}

/// Paint `text` into `area` under `align`, ending with the ellipsis glyph
/// when it does not fit. Allocation-free, and never writes outside `area`.
fn paint_aligned(ui: &mut Ui<'_>, area: Rect, text: &str, align: Align, style: Style) {
    if area.is_empty() || text.is_empty() {
        return;
    }
    let w = width(text);
    if w > area.width {
        let head = Rect {
            width: area.width.saturating_sub(1),
            ..area
        };
        let used = ui.paint_str(head, text, style);
        let tail = Rect {
            x: area.x.saturating_add(used),
            width: area.width.saturating_sub(used),
            ..area
        };
        ui.glyph(tail, GlyphRole::Ellipsis, style);
        return;
    }
    let pad = area.width.saturating_sub(w);
    let off = match align {
        Align::Left => 0,
        Align::Center => pad / 2,
        Align::Right => pad,
    };
    let at = Rect {
        x: area.x.saturating_add(off),
        width: area.width.saturating_sub(off),
        ..area
    };
    ui.paint_str(at, text, style);
}

fn apply_style_delta(ui: &Ui<'_>, base: Style, delta: StylePatch) -> Style {
    if delta.is_empty() {
        return base;
    }
    let top = crate::theme::resolve::bind(ui.theme_ref(), delta, None, ui.surface()).style;
    base.patch(top)
}

/// Append `n` with thousands separators.
fn push_grouped(out: &mut String, n: usize) {
    let mut digits = [0u8; 20];
    let mut len = 0usize;
    let mut v = n;
    loop {
        if let Some(d) = digits.get_mut(len) {
            *d = b'0'.saturating_add((v % 10) as u8);
        }
        len = len.saturating_add(1);
        v /= 10;
        if v == 0 || len == digits.len() {
            break;
        }
    }
    for i in (0..len).rev() {
        if let Some(d) = digits.get(i) {
            out.push(char::from(*d));
        }
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
    }
}

/// A number formatted into a stack buffer, so the overflow indicators and the
/// header badges never allocate on the paint path.
#[derive(Clone, Copy, Debug)]
struct Num {
    buf: [u8; 20],
    len: usize,
}

impl Num {
    fn new(n: usize) -> Self {
        let mut digits = [0u8; 20];
        let mut len = 0usize;
        let mut v = n;
        loop {
            if let Some(d) = digits.get_mut(len) {
                *d = b'0'.saturating_add((v % 10) as u8);
            }
            len = len.saturating_add(1);
            v /= 10;
            if v == 0 || len == digits.len() {
                break;
            }
        }
        let mut buf = [0u8; 20];
        for i in 0..len {
            if let (Some(dst), Some(src)) = (
                buf.get_mut(i),
                digits.get(len.saturating_sub(1).saturating_sub(i)),
            ) {
                *dst = *src;
            }
        }
        Num { buf, len }
    }

    fn as_str(&self) -> &str {
        self.buf
            .get(..self.len)
            .and_then(|b| core::str::from_utf8(b).ok())
            .unwrap_or("")
    }
}

/// Why navigation requested the edit path.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum EditCause {
    /// `Enter` or a double-click: activate when editing is unavailable.
    Activate,
    /// `F2`: edit only, with no activation fallback.
    Explicit,
}

#[derive(Clone, Copy, Debug)]
struct EditRequest {
    row: usize,
    col: usize,
    cause: EditCause,
}

/// What a navigation pass asked the caller to do with the cursor cell.
#[derive(Clone, Copy, Debug, Default)]
struct Pending {
    edit: Option<EditRequest>,
}

/// How an edit begins, extracted from [`EditIntent`] so the model's borrow
/// ends before `apply_cycle` / `commit_cell` need `&mut`.
enum Begin {
    Inline(String),
    Cycle,
    External,
    Refuse(String),
}

impl Grid<'_> {
    /// The row window `draw` paints, derived from the same scroll state so
    /// both phases agree (§12.2).
    fn window(st: &GridState, content: Rect, len: usize) -> core::ops::Range<usize> {
        let view = ScrollRegion::view(st.core.scroll(), content, len);
        let r = view.visible_range();
        r.start.min(len)..r.end.min(len)
    }

    /// The index of `key`, probing the cursor's cached index before scanning.
    fn row_index<M: GridModel + ?Sized>(model: &M, key: ItemKey, hint: usize) -> Option<usize> {
        let len = model.row_count();
        if hint < len && model.row_key(hint) == key {
            return Some(hint);
        }
        (0..len).find(|&i| model.row_key(i) == key)
    }

    /// `rows a–b of n`, with the source total when the model knows one.
    ///
    /// Composed by the owning screen into a `StatusBar`; it is never on the
    /// paint path, which is why it may allocate.
    pub fn rows_label<M: GridModel + ?Sized>(
        &self,
        f: &impl FrameRead,
        st: &GridState,
        model: &M,
    ) -> String {
        let len = model.row_count();
        let rows = f
            .layout(self.id)
            .map_or_else(|| st.core.scroll().viewport_len(), |l| l.viewport_len);
        let content = Rect {
            height: rows.min(usize::from(u16::MAX)) as u16,
            ..Rect::ZERO
        };
        let w = Self::window(st, content, len);
        let mut s = String::new();
        s.push_str("rows ");
        push_grouped(&mut s, w.start.saturating_add(1).min(len));
        s.push('\u{2013}');
        push_grouped(&mut s, w.end);
        s.push_str(" of ");
        push_grouped(&mut s, len);
        match model.total() {
            RowTotal::Exact(t) | RowTotal::Estimated(t) if t > len => {
                s.push_str(" loaded \u{b7} ");
                push_grouped(&mut s, t);
                s.push_str(" total");
            }
            _ => {}
        }
        s
    }

    /// `cols a–b of n`, or `None` when every column is on screen.
    pub fn cols_label<M: GridModel + ?Sized>(
        &self,
        f: &impl FrameRead,
        st: &GridState,
        model: &M,
    ) -> Option<String> {
        let area = f.area(self.id)?;
        let len = model.row_count();
        let (_, _, body, _) = self.chrome(area, model.read_only_reason());
        let rows = Self::window(st, body, len);
        let g = self.geometry(body, st, model, rows);
        if g.hidden_left == 0 && g.hidden_right == 0 {
            return None;
        }
        let mut shown = (0..g.n).filter(|&i| g.shown.get(i).copied().unwrap_or(false));
        let first = shown.clone().next()?;
        let last = shown.next_back()?;
        let mut s = String::new();
        s.push_str("cols ");
        push_grouped(&mut s, first.saturating_add(1));
        s.push('\u{2013}');
        push_grouped(&mut s, last.saturating_add(1));
        s.push_str(" of ");
        push_grouped(&mut s, g.n);
        Some(s)
    }

    /// The rectangular range, resolved to indices, or `None` when no anchor
    /// is set or the anchor's row or column has gone.
    ///
    /// Resolved from the stored **keys** every phase — this is the whole
    /// reason the anchor is `(ItemKey, ColumnKey)` and not a pair of indices:
    /// a model that reorders itself between frames keeps the same logical
    /// rectangle rather than a rectangle at the same coordinates (§33).
    fn range<M: GridModel + ?Sized>(
        &self,
        st: &GridState,
        model: &M,
        cursor: (usize, usize),
    ) -> Option<((usize, usize), (usize, usize))> {
        let (ak, ac) = st.anchor?;
        let ar = Self::row_index(model, ak, cursor.0)?;
        let (r0, r1) = (ar.min(cursor.0), ar.max(cursor.0));
        let (c0, c1) = match self.nav {
            NavUnit::Row => (0, self.column_count().saturating_sub(1)),
            NavUnit::Cell => {
                let acol = self.col_index(ac)?;
                (acol.min(cursor.1), acol.max(cursor.1))
            }
        };
        Some(((r0, r1), (c0, c1)))
    }

    /// The selection as TSV: the rectangular range if one is active, else the
    /// selected rows, else the cursor cell. Allocates — it is the payload of
    /// [`GridAction::Copy`], produced once per copy, never per frame.
    fn copy_tsv<M: GridModel + ?Sized>(
        &self,
        st: &GridState,
        model: &M,
        cursor: (usize, usize),
    ) -> String {
        let len = model.row_count();
        let cols = self.column_count();
        let mut out = String::new();
        if len == 0 || cols == 0 {
            return out;
        }
        let last_col = cols.saturating_sub(1);
        let last_row = len.saturating_sub(1);
        let row_to = |out: &mut String, r: usize, c0: usize, c1: usize| {
            for c in c0..=c1.min(last_col) {
                if c > c0 {
                    out.push('\t');
                }
                if let Some(cell) = model.cell(r, c) {
                    out.push_str(cell.text);
                }
            }
            out.push('\n');
        };
        if let Some(((r0, r1), (c0, c1))) = self.range(st, model, cursor) {
            for r in r0..=r1.min(last_row) {
                row_to(&mut out, r, c0, c1);
            }
        } else if !st.core.checked().is_empty() {
            for r in 0..len {
                if st.core.checked().contains(model.row_key(r)) {
                    row_to(&mut out, r, 0, last_col);
                }
            }
        } else {
            row_to(&mut out, cursor.0.min(last_row), cursor.1, cursor.1);
        }
        out
    }

    /// Move the cursor to `(row, col)`, extending the range when asked.
    fn move_to<M: GridModel + ?Sized>(
        &self,
        st: &mut GridState,
        model: &M,
        row: usize,
        col: usize,
        extend: bool,
        acc: &mut Acc<GridAction>,
    ) {
        let len = model.row_count();
        let column_count = self.column_count();
        if len == 0 || column_count == 0 {
            return;
        }
        let row = row.min(len.saturating_sub(1));
        let col = col.min(column_count.saturating_sub(1));
        let key = model.row_key(row);
        let Some(ck) = self.col_key(col) else { return };
        if !st.is_editing() {
            st.editor.set_error(None);
        }
        if extend {
            if st.anchor.is_none() {
                let cur = self.cursor_col(st);
                let cur_row = st.core.cursor_index().min(len.saturating_sub(1));
                if let Some(cur_key) = self.col_key(cur) {
                    st.anchor = Some((model.row_key(cur_row), cur_key));
                }
            }
        } else {
            st.anchor = None;
        }
        st.set_cursor(row, key, col, ck);
        self.reveal_column(st, col);
        acc.action(GridAction::Moved);
    }

    /// Scroll the column window so `col` is inside it. Sticky columns are
    /// always shown, so they never move the window.
    fn reveal_column(&self, st: &mut GridState, col: usize) {
        if self.columns.get(col).is_some_and(|c| c.sticky) {
            return;
        }
        let scroll_index = self
            .columns
            .get(..col)
            .map_or(0, |cs| cs.iter().filter(|c| !c.sticky).count());
        st.col_offset = scroll_index;
    }

    /// Toggle the cursor row's selection under the current select mode.
    fn toggle_row<M: GridModel + ?Sized>(
        &self,
        st: &mut GridState,
        model: &M,
        row: usize,
        acc: &mut Acc<GridAction>,
    ) {
        if row >= model.row_count() {
            acc.consumed();
            return;
        }
        let key = model.row_key(row);
        match self.select_mode {
            SelectMode::None => acc.consumed(),
            SelectMode::Single => {
                let was = st.core.checked().contains(key);
                st.core.checked_mut().none();
                if !was {
                    st.core.checked_mut().insert(key);
                }
                acc.changed();
            }
            SelectMode::Multi | SelectMode::Range => {
                st.core.checked_mut().toggle(key);
                acc.changed();
            }
        }
    }

    /// One navigation pass, shared by both entry points (§23 K2 risk 2: the
    /// two `GridAction` paths cannot drift because there is one body).
    #[expect(
        clippy::too_many_lines,
        reason = "the keymap dispatch and the three pointer surfaces in one drain loop"
    )]
    fn navigate<M: GridModel + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut GridState,
        model: &M,
        acc: &mut Acc<GridAction>,
    ) -> Pending {
        let len = model.row_count();
        let column_count = self.column_count();
        // A same-length reorder with unchanged end keys is invisible to the
        // collection stamp. Probe the cached cursor so keyed identity still
        // forces reconciliation without scanning an unchanged model.
        if st.core.cursor().is_some_and(|key| {
            st.core.cursor_index() >= len || model.row_key(st.core.cursor_index()) != key
        }) {
            st.core.invalidate();
        }
        // G7: reconcile before anything can be emitted
        let outcome = st.core.reconcile_with(len, |i| model.row_key(i), |_| true);
        if outcome != Reconciliation::Unchanged {
            if let Some((a, _)) = st.anchor
                && Self::row_index(model, a, st.core.cursor_index()).is_none()
            {
                st.anchor = None;
            }
            if let Some((e, _)) = st.edit
                && Self::row_index(model, e, st.core.cursor_index()).is_none()
            {
                st.edit = None;
                st.editor.cancel();
            }
        }
        if st.core.cursor().is_none() && len > 0 {
            st.core.set_cursor(0, model.row_key(0));
        }
        if column_count == 0 {
            st.col = None;
            st.col_index = 0;
            st.col_offset = 0;
            st.anchor = None;
            st.cancel_editor();
        } else if st.col.is_none() {
            st.col = self.col_key(0);
            st.col_index = 0;
        } else if st.col.is_some_and(|key| self.col_index(key).is_none()) {
            st.col_index = st.col_index.min(column_count.saturating_sub(1));
            st.col = self.col_key(st.col_index);
            st.anchor = None;
            st.cancel_editor();
        }
        let mut pending = Pending::default();
        let total = len.saturating_add(usize::from(model.has_more()));
        let bar = self.bar().update(cx, st.core.scroll_mut(), total);
        acc.fold(&bar);
        let viewport = st.core.scroll().viewport_len().max(1);
        let area = cx.area(self.id);
        let geometry = area.map(|a| {
            let (_, _, body, _) = self.chrome(a, model.read_only_reason());
            let rows = Self::window(st, body, total);
            self.geometry(body, st, model, rows)
        });
        for it in cx.intents(self.id) {
            match it {
                Intent::Binding(action) => {
                    let row = st.core.cursor_index().min(len.saturating_sub(1));
                    let col = self.cursor_col(st);
                    match Binding::command(&BINDINGS, action) {
                        Some(GridCmd::Up) => {
                            self.move_to(st, model, row.saturating_sub(1), col, false, acc);
                        }
                        Some(GridCmd::Down) => {
                            if row.saturating_add(1) >= len && model.has_more() {
                                acc.action(GridAction::FetchMore);
                            } else {
                                self.move_to(st, model, row.saturating_add(1), col, false, acc);
                            }
                        }
                        Some(GridCmd::Left) => match self.nav {
                            NavUnit::Row => acc.consumed(),
                            NavUnit::Cell if col > 0 => {
                                self.move_to(st, model, row, col.saturating_sub(1), false, acc);
                            }
                            NavUnit::Cell if row > 0 => self.move_to(
                                st,
                                model,
                                row.saturating_sub(1),
                                column_count.saturating_sub(1),
                                false,
                                acc,
                            ),
                            NavUnit::Cell => acc.action(GridAction::LeaveBackward),
                        },
                        Some(GridCmd::Right) => match self.nav {
                            NavUnit::Row => acc.consumed(),
                            NavUnit::Cell if col.saturating_add(1) < column_count => {
                                self.move_to(st, model, row, col.saturating_add(1), false, acc);
                            }
                            NavUnit::Cell if row.saturating_add(1) < len => {
                                self.move_to(st, model, row.saturating_add(1), 0, false, acc);
                            }
                            NavUnit::Cell => acc.action(GridAction::LeaveForward),
                        },
                        Some(GridCmd::PageUp) => {
                            self.move_to(st, model, row.saturating_sub(viewport), col, false, acc);
                        }
                        Some(GridCmd::PageDown) => {
                            self.move_to(st, model, row.saturating_add(viewport), col, false, acc);
                        }
                        Some(GridCmd::RowStart) => self.move_to(st, model, row, 0, false, acc),
                        Some(GridCmd::RowEnd) => {
                            self.move_to(
                                st,
                                model,
                                row,
                                column_count.saturating_sub(1),
                                false,
                                acc,
                            );
                        }
                        Some(GridCmd::First) => self.move_to(st, model, 0, col, false, acc),
                        Some(GridCmd::Last) => {
                            self.move_to(st, model, len.saturating_sub(1), col, false, acc);
                        }
                        Some(GridCmd::ExtendUp) => {
                            self.move_to(st, model, row.saturating_sub(1), col, true, acc);
                        }
                        Some(GridCmd::ExtendDown) => {
                            self.move_to(st, model, row.saturating_add(1), col, true, acc);
                        }
                        Some(GridCmd::ExtendLeft) => {
                            self.move_to(st, model, row, col.saturating_sub(1), true, acc);
                        }
                        Some(GridCmd::ExtendRight) => {
                            self.move_to(st, model, row, col.saturating_add(1), true, acc);
                        }
                        Some(GridCmd::ToggleRow) => self.toggle_row(st, model, row, acc),
                        Some(GridCmd::ToggleAll) => {
                            if matches!(self.select_mode, SelectMode::None) {
                                acc.consumed();
                            } else {
                                let all =
                                    (0..len).all(|i| st.core.checked().contains(model.row_key(i)));
                                if all {
                                    st.core.checked_mut().none();
                                } else {
                                    st.core.checked_mut().all();
                                }
                                acc.changed();
                            }
                        }
                        Some(GridCmd::Activate) => {
                            if len == 0 || column_count == 0 {
                                acc.consumed();
                            } else if matches!(self.nav, NavUnit::Row) {
                                acc.action(GridAction::Activated(model.row_key(row)));
                            } else {
                                pending.edit = Some(EditRequest {
                                    row,
                                    col,
                                    cause: EditCause::Activate,
                                });
                                acc.consumed();
                            }
                        }
                        Some(GridCmd::BeginEdit) => {
                            if len == 0 || column_count == 0 {
                                acc.consumed();
                            } else {
                                pending.edit = Some(EditRequest {
                                    row,
                                    col,
                                    cause: EditCause::Explicit,
                                });
                                acc.consumed();
                            }
                        }
                        Some(GridCmd::Copy) => {
                            if len > 0 {
                                let tsv = self.copy_tsv(st, model, (row, col));
                                acc.action(GridAction::Copy(tsv));
                            } else {
                                acc.consumed();
                            }
                        }
                        None => {}
                    }
                }
                Intent::Pointer {
                    phase,
                    part:
                        PartRef {
                            part: Part::HEADER,
                            item: Some(header_key),
                        },
                    ..
                } => {
                    let col = (0..column_count).find(|&i| {
                        self.columns.get(i).is_some_and(|column| {
                            column.sortable && column_item_key(column.key) == header_key
                        })
                    });
                    match (phase, col.and_then(|i| self.col_key(i))) {
                        (Phase::Click | Phase::DoubleClick, Some(key)) => {
                            let dir = match st.sort {
                                Some((current, SortDir::Asc)) if current == key => SortDir::Desc,
                                _ => SortDir::Asc,
                            };
                            st.sort = Some((key, dir));
                            acc.action(GridAction::Sort(key, dir));
                        }
                        _ => acc.consumed(),
                    }
                }
                Intent::Pointer {
                    phase,
                    part:
                        PartRef {
                            part: Part::ACTIONS,
                            item: Some(row_key),
                        },
                    pos,
                    ..
                } => {
                    let hit = geometry
                        .as_ref()
                        .and_then(|geometry| geometry.column_at(pos.x))
                        .and_then(|column| {
                            Some((
                                Self::row_index(model, row_key, st.core.cursor_index())?,
                                column,
                            ))
                        });
                    match (phase, hit) {
                        (Phase::Click, Some((row, column))) => {
                            let action = model
                                .cell(row, column)
                                .and_then(|_| model.actions(row, column).first());
                            match (self.col_key(column), action) {
                                (Some(column_key), Some(action)) => {
                                    acc.action(GridAction::CellAction(
                                        row_key, column_key, action.key,
                                    ));
                                }
                                _ => acc.consumed(),
                            }
                        }
                        _ => acc.consumed(),
                    }
                }
                Intent::Pointer {
                    phase,
                    part:
                        PartRef {
                            part: Part::CELL,
                            item: Some(row_key),
                        },
                    pos,
                    ..
                } => {
                    let Some(row) = Self::row_index(model, row_key, st.core.cursor_index()) else {
                        acc.consumed();
                        continue;
                    };
                    let col = geometry
                        .as_ref()
                        .and_then(|geometry| geometry.column_at(pos.x))
                        .unwrap_or_else(|| self.cursor_col(st));
                    match phase {
                        Phase::Press => self.move_to(st, model, row, col, false, acc),
                        Phase::DoubleClick => {
                            if matches!(self.nav, NavUnit::Row) {
                                acc.action(GridAction::Activated(row_key));
                            } else {
                                pending.edit = Some(EditRequest {
                                    row,
                                    col,
                                    cause: EditCause::Activate,
                                });
                                acc.changed();
                            }
                        }
                        _ => acc.consumed(),
                    }
                }
                Intent::Pointer {
                    phase: Phase::Click,
                    part:
                        PartRef {
                            part: Part::ROW,
                            item: None,
                        },
                    ..
                } => acc.action(GridAction::FetchMore),
                Intent::Pointer { .. } => acc.consumed(),
                _ => {}
            }
        }
        pending
    }

    /// Read-only navigation, selection, copy, fetch-more and cell actions.
    ///
    /// `&M`: a read-only grid **cannot** mutate its model — a compile-time
    /// fact, not a runtime refusal (§23 K2, G1).
    pub fn update<M: GridModel + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut GridState,
        model: &M,
    ) -> Response<GridAction> {
        self.assert_distinct_column_keys();
        let mut acc = Acc::<GridAction>::new();
        let pending = self.navigate(cx, st, model, &mut acc);
        // a read-only grid has one meaning for `Enter` on a cell
        if let Some(EditRequest {
            row,
            cause: EditCause::Activate,
            ..
        }) = pending.edit
            && row < model.row_count()
        {
            acc.action(GridAction::Activated(model.row_key(row)));
        }
        acc.finish(self.id)
    }

    /// Everything [`Grid::update`] does, plus the inline edit lifecycle.
    ///
    /// The **only** place [`GridEditor`]'s `&mut self` methods are reachable
    /// (§23 K2, G2).
    pub fn update_editable<M: GridEditor + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut GridState,
        model: &mut M,
    ) -> Response<GridAction> {
        self.assert_distinct_column_keys();
        let mut acc = Acc::<GridAction>::new();
        self.drive_editor(cx, st, model, &mut acc);
        let pending = self.navigate(cx, st, model, &mut acc);
        if let Some(request) = pending.edit
            && st.edit.is_none()
            && request.row < model.row_count()
        {
            self.begin_edit(cx, st, model, request, &mut acc);
        }
        acc.finish(self.id)
    }

    /// Open, or refuse to open, an inline edit on `(row, col)`.
    fn begin_edit<M: GridEditor + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut GridState,
        model: &mut M,
        request: EditRequest,
        acc: &mut Acc<GridAction>,
    ) {
        let EditRequest { row, col, cause } = request;
        let key = model.row_key(row);
        let Some(ck) = self.col_key(col) else { return };
        st.editor.set_error(None);
        if model.cell(row, col).is_none()
            || !self.columns.get(col).is_some_and(|c| c.editable)
            || !model.is_editable(row, col)
        {
            if cause == EditCause::Activate {
                acc.action(GridAction::Activated(key));
            }
            return;
        }
        let what = match model.edit_intent(row, col) {
            EditIntent::Inline { initial } => Begin::Inline(initial.to_owned()),
            EditIntent::Cycle => Begin::Cycle,
            EditIntent::External => Begin::External,
            EditIntent::Refuse { reason } => Begin::Refuse(reason.to_owned()),
        };
        match what {
            Begin::Inline(initial) => {
                st.editor.set_error(None);
                st.editor.begin(&initial);
                st.edit = Some((key, ck));
                cx.focus(self.editor_id());
                acc.changed();
            }
            Begin::Cycle => {
                model.apply_cycle(row, col);
                acc.changed();
            }
            // G5: External emits and begins no inline edit
            Begin::External => acc.action(GridAction::EditRequested(key, ck)),
            Begin::Refuse(reason) => {
                st.editor
                    .set_error(Some(crate::validate::FieldError::new(reason)));
                acc.changed();
            }
        }
    }

    /// Drive the open inline editor, if there is one.
    fn drive_editor<M: GridEditor + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut GridState,
        model: &mut M,
        acc: &mut Acc<GridAction>,
    ) {
        let Some((rk, ck)) = st.edit else { return };
        let (Some(col), Some(row)) = (
            self.col_index(ck),
            Self::row_index(model, rk, st.core.cursor_index()),
        ) else {
            st.cancel_editor();
            return;
        };
        // one owned copy per frame **while editing only**; the editor writes
        // its draft back into it on commit
        let Some(cell) = model.cell(row, col) else {
            st.cancel_editor();
            return;
        };
        let mut value = cell.text.to_owned();
        let mut r = TextInput::new(self.editor_id()).update(cx, &mut st.editor, &mut value);
        let action = r.take_action();
        let erased = r.erase();
        acc.fold(&erased);
        match action {
            Some(TextAction::Committed) => match model.commit_cell(row, col, &value) {
                Ok(()) => {
                    st.edit = None;
                    cx.focus(self.id);
                    acc.changed();
                }
                Err(e) => {
                    // the editor stays open, showing the model's own error
                    st.editor.begin(&value);
                    st.editor.set_error(Some(e));
                    acc.changed();
                }
            },
            Some(TextAction::Cancelled) => {
                st.cancel_editor();
                cx.focus(self.id);
                acc.changed();
            }
            _ => {}
        }
    }
}

impl Grid<'_> {
    fn right_overflow_rect(head: Rect, hidden_right: usize) -> Rect {
        let count = Num::new(hidden_right);
        let indicator_width = width(count.as_str()).saturating_add(1).min(head.width);
        Rect {
            x: head.right().saturating_sub(indicator_width),
            width: indicator_width,
            ..head
        }
    }

    fn register_sort_headers(&self, ui: &mut Ui<'_>, head: Rect, geometry: &Geometry) {
        if ui.is_inert() {
            return;
        }
        for index in 0..geometry.n {
            let rect = geometry.cell(index, head.y);
            if rect.is_empty() {
                continue;
            }
            if let Some(column) = self.columns.get(index)
                && column.sortable
            {
                ui.register_part(
                    self.id,
                    PartRef::item(Part::HEADER, column_item_key(column.key)),
                    rect,
                );
            }
        }
    }

    fn draw_header_overflow(
        &self,
        ui: &mut Ui<'_>,
        head: Rect,
        geometry: &Geometry,
        live: StateFlags,
    ) {
        if geometry.hidden_left == 0 && geometry.hidden_right == 0 {
            return;
        }
        let style = self
            .ov
            .style(
                ui,
                self.id,
                Family::GRID,
                Variant::DEFAULT,
                Part::OVERFLOW,
                live,
            )
            .style;
        if geometry.hidden_left > 0 {
            let count = Num::new(geometry.hidden_left);
            let at = Rect {
                width: head.width.min(2),
                ..head
            };
            let used = ui.glyph(at, GlyphRole::OverflowLeft, style);
            ui.paint_str(
                Rect {
                    x: at.x.saturating_add(used),
                    width: at.width.saturating_sub(used),
                    ..at
                },
                count.as_str(),
                style,
            );
        }
        if geometry.hidden_right > 0 {
            let count = Num::new(geometry.hidden_right);
            let at = Self::right_overflow_rect(head, geometry.hidden_right);
            let used = ui.paint_str(at, count.as_str(), style);
            ui.glyph(
                Rect {
                    x: at.x.saturating_add(used),
                    width: at.width.saturating_sub(used),
                    ..at
                },
                GlyphRole::OverflowRight,
                style,
            );
        }
    }

    /// Paint the header row: titles, badges and the `‹N` / `N›` overflow
    /// indicators.
    fn draw_header(
        &self,
        ui: &mut Ui<'_>,
        head: Rect,
        g: &Geometry,
        st: &GridState,
        live: StateFlags,
    ) {
        if head.is_empty() {
            return;
        }
        self.register_sort_headers(ui, head, g);
        if let Some(f) = self.ov.slot_for(Part::HEADER) {
            f(ui, head);
            return;
        }
        let hs = self.ov.style(
            ui,
            self.id,
            Family::GRID,
            Variant::DEFAULT,
            Part::HEADER,
            live,
        );
        ui.fill(head, hs.style);
        let right_overflow =
            (g.hidden_right > 0).then(|| Self::right_overflow_rect(head, g.hidden_right));
        for i in 0..g.n {
            let raw_rect = g.cell(i, head.y);
            let Some(col) = self.columns.get(i) else {
                break;
            };
            let rect = right_overflow.map_or(raw_rect, |overflow| Rect {
                width: raw_rect.right().min(overflow.x).saturating_sub(raw_rect.x),
                ..raw_rect
            });
            if rect.width == 0 {
                continue;
            }
            let sort_width = u16::from(col.sortable).min(rect.width);
            let title = Rect {
                width: rect
                    .width
                    .saturating_sub(
                        col.badge
                            .map_or(0, |b| width(b).saturating_add(1).min(rect.width)),
                    )
                    .saturating_sub(sort_width),
                ..rect
            };
            paint_aligned(ui, title, col.title, col.align, hs.style);
            if let Some(badge) = col.badge {
                let bw = width(badge).min(rect.width);
                let at = Rect {
                    x: rect.right().saturating_sub(sort_width).saturating_sub(bw),
                    width: bw,
                    ..rect
                };
                paint_aligned(ui, at, badge, Align::Right, hs.style);
            }
            if col.sortable {
                let glyph = match st.sort {
                    Some((key, SortDir::Desc)) if key == col.key => GlyphRole::SortDesc,
                    _ => GlyphRole::SortAsc,
                };
                ui.glyph(
                    Rect {
                        x: rect.right().saturating_sub(1),
                        width: sort_width,
                        ..rect
                    },
                    glyph,
                    hs.style,
                );
            }
        }
        self.draw_header_overflow(ui, head, g, live);
    }

    /// Paint one body row and register its cell parts.
    #[expect(
        clippy::too_many_lines,
        reason = "one pass over the visible cells: fill, gutter, marker, cells, affordances and the inline editor"
    )]
    fn draw_row<M: GridModel + ?Sized>(
        &self,
        ui: &mut Ui<'_>,
        y: u16,
        row: usize,
        paint: &RowPaint<'_, M>,
    ) {
        let RowPaint {
            content,
            geometry,
            state,
            model,
            cursor,
            range,
            live,
        } = paint;
        let key = model.row_key(row);
        let decor = model.row_decor(row);
        let checked = state.core.checked().contains(key);
        let is_cursor = row == cursor.0;
        let pressed = ui.pressed_part(self.id);
        let mut rflags = decor.flags();
        if is_cursor {
            rflags |= *live & (StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE);
        }
        if checked {
            rflags |= StateFlags::CHECKED | StateFlags::SELECTED;
        }
        if live.contains(StateFlags::DISABLED) {
            rflags |= StateFlags::DISABLED;
        }
        if pressed == Some(PartRef::item(Part::ROW, key)) {
            rflags |= StateFlags::PRESSED;
        }
        let band = Rect {
            x: content.x,
            y,
            width: content.width,
            height: 1,
        };
        let rs = self.ov.style(
            ui,
            self.id,
            Family::GRID,
            Variant::DEFAULT,
            Part::ROW,
            rflags,
        );
        let mut row_delta = StylePatch::new();
        if decor.strike {
            row_delta = row_delta.add(Modifier::CROSSED_OUT);
        }
        if decor.faint {
            row_delta = row_delta.add(Modifier::DIM);
        }
        let row_style = apply_style_delta(ui, rs.style, row_delta);
        ui.fill(band, row_style);
        // The focus gutter and the selection marker are painted from the ROW
        // resolution: `PARTS` is exactly what `draw` resolves (§33), and
        // §17.0 A7's list has no GUTTER or MARKER.
        if is_cursor && live.contains(StateFlags::FOCUSED) {
            ui.glyph(super::cell_at(band, band.x), GlyphRole::FocusBar, row_style);
        }
        let marker_cell = super::cell_at(band, band.x.saturating_add(1));
        if checked {
            ui.glyph(marker_cell, GlyphRole::Checked, row_style);
        } else if let Some(m) = decor.marker {
            ui.glyph(marker_cell, m, row_style);
        }
        let inert = ui.is_inert();
        for i in 0..geometry.n {
            let rect = geometry.cell(i, y);
            if rect.width == 0 {
                continue;
            }
            let cell = model.cell(row, i);
            // `None` — a column index past the list — matches no edit at
            // all. A sentinel key would have to live in some real key's
            // range; absence lives in none.
            let editing = cell.is_some()
                && self
                    .col_key(i)
                    .is_some_and(|col| state.edit == Some((key, col)));
            let refused_error = (!editing && is_cursor && i == cursor.1)
                .then(|| state.edit_error())
                .flatten();
            let mut cflags = rflags.difference(StateFlags::PRESSED);
            if is_cursor && i == cursor.1 {
                cflags |= StateFlags::ACTIVE;
            }
            if range
                .is_some_and(|((r0, r1), (c0, c1))| row >= r0 && row <= r1 && i >= c0 && i <= c1)
            {
                cflags |= StateFlags::SELECTED;
            }
            if editing {
                cflags |= StateFlags::EDITING;
            }
            if refused_error.is_some() {
                cflags |= StateFlags::ERROR;
            }
            if pressed == Some(PartRef::item(Part::CELL, key)) {
                cflags |= StateFlags::PRESSED;
            }
            let Some(cell) = cell else {
                let cs = self.ov.style(
                    ui,
                    self.id,
                    Family::GRID,
                    Variant::DEFAULT,
                    Part::CELL,
                    cflags,
                );
                ui.fill(rect, cs.style);
                if !inert {
                    ui.register_part(self.id, PartRef::item(Part::CELL, key), rect);
                }
                continue;
            };
            let cdecor = model.cell_decor(row, i);
            let actions = model.actions(row, i);
            cflags |= cdecor.flags();
            let cs = self.ov.style(
                ui,
                self.id,
                Family::GRID,
                Variant::DEFAULT,
                Part::CELL,
                cflags,
            );
            let mut cell_delta = StylePatch::new();
            if let Some(role) = cdecor.tone.or(cell.tone) {
                cell_delta = cell_delta.set_fg(role);
            }
            if cdecor.italic {
                cell_delta = cell_delta.add(Modifier::ITALIC);
            }
            let style = apply_style_delta(ui, cs.style, cell_delta);
            let mut text_rect = rect;
            if let Some(gl) = self.columns.get(i).and_then(|c| c.prefix_glyph) {
                let used = ui.glyph(rect, gl, style);
                text_rect = Rect {
                    x: rect.x.saturating_add(used).saturating_add(1),
                    width: rect.width.saturating_sub(used).saturating_sub(1),
                    ..rect
                };
            }
            let affordance = if actions.is_empty() {
                Rect::ZERO
            } else {
                let w = text_rect.width.min(1);
                let r = Rect {
                    x: text_rect.right().saturating_sub(w),
                    width: w,
                    ..text_rect
                };
                text_rect = Rect {
                    width: text_rect
                        .width
                        .saturating_sub(w.saturating_add(1).min(text_rect.width)),
                    ..text_rect
                };
                r
            };
            let align = cell
                .align
                .or_else(|| self.columns.get(i).map(|column| column.align))
                .unwrap_or(Align::Left);
            let text = refused_error.map_or(cell.text, |error| error.message.as_ref());
            paint_aligned(ui, text_rect, text, align, style);
            if !inert {
                ui.register_part(self.id, PartRef::item(Part::CELL, key), rect);
            }
            if let Some(a) = actions.first() {
                if let Some(f) = self.ov.slot_for(Part::ACTIONS) {
                    f(ui, affordance);
                } else {
                    let mut action_flags = cflags.difference(StateFlags::PRESSED);
                    if pressed == Some(PartRef::item(Part::ACTIONS, key)) {
                        action_flags |= StateFlags::PRESSED;
                    }
                    let as_ = self.ov.style(
                        ui,
                        self.id,
                        Family::GRID,
                        Variant::DEFAULT,
                        Part::ACTIONS,
                        action_flags,
                    );
                    ui.glyph(affordance, a.glyph, as_.style);
                }
                if !inert {
                    // registered AFTER the cell, so it wins the click
                    ui.register_part(self.id, PartRef::item(Part::ACTIONS, key), affordance);
                }
            }
            if editing {
                // G6: the inline editor's Control region is registered after
                // the cell's Part region, so a click inside it goes to the
                // editor and not to the grid
                TextInput::new(self.id.part(Part::TEXT))
                    .value(cell.text)
                    .draw(ui, rect, &state.editor);
            }
        }
    }

    /// The draw phase; returns `area`.
    #[expect(
        clippy::too_many_lines,
        reason = "the chrome, the scroll region, the body and the four conditional surfaces in one pass"
    )]
    pub fn draw<M: GridModel + ?Sized>(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &GridState,
        model: &M,
    ) -> Rect {
        self.assert_distinct_column_keys();
        if area.is_empty() {
            return area;
        }
        let len = model.row_count();
        let total = len.saturating_add(usize::from(model.has_more()));
        let inert = ui.is_inert();
        if !inert {
            ui.register_control(self.id, area, Focusability::Focusable);
        }
        let reason = model.read_only_reason();
        let derived = if reason.is_some() {
            StateFlags::READ_ONLY
        } else {
            StateFlags::empty()
        };
        let live = PartStyle::flags(ui.state(self.id), derived);
        if !inert {
            ui.publish_bindings(self.id, live, &BINDINGS);
        }
        let container = self.ov.style(
            ui,
            self.id,
            Family::GRID,
            Variant::DEFAULT,
            Part::CONTAINER,
            live.difference(StateFlags::FOCUSED | StateFlags::PRESSED | StateFlags::SELECTED),
        );
        ui.fill(area, container.style);
        let (header, note, body, bar) = self.chrome(area, reason);
        let content = self.bar().draw(ui, body, st.core.scroll(), total);
        let rows = Self::window(st, content, total);
        let g = self.geometry(content, st, model, rows.clone());
        let head = Rect {
            x: content.x,
            width: content.width,
            ..header
        };
        self.draw_header(ui, head, &g, st, live);
        if let Some(r) = reason
            && note.height > 0
        {
            let ns = self.ov.style(
                ui,
                self.id,
                Family::GRID,
                Variant::DEFAULT,
                Part::HEADER,
                live | StateFlags::READ_ONLY,
            );
            ui.fill(note, ns.style);
            paint_aligned(
                ui,
                Rect {
                    x: note.x.saturating_add(2),
                    width: note.width.saturating_sub(2),
                    ..note
                },
                r,
                Align::Left,
                ns.style,
            );
        }
        if len == 0 {
            let empty = self.empty.unwrap_or(EmptyState::Empty {
                title: "Nothing here yet",
                hint: None,
            });
            let mid = Rect {
                y: content.y.saturating_add(content.height / 2),
                height: content.height.saturating_sub(content.height / 2),
                ..content
            };
            if let Some(f) = self.ov.slot_for(Part::EMPTY) {
                f(ui, mid);
            } else {
                let _ = self.ov.style(
                    ui,
                    self.id,
                    Family::GRID,
                    Variant::DEFAULT,
                    Part::EMPTY,
                    live,
                );
                empty.draw(ui, mid, 0);
            }
            self.draw_actions(ui, bar, live);
            return area;
        }
        let cursor = (
            st.core.cursor_index().min(len.saturating_sub(1)),
            self.cursor_col(st),
        );
        let range = self.range(st, model, cursor);
        let row_paint = RowPaint {
            content,
            geometry: &g,
            state: st,
            model,
            cursor,
            range,
            live,
        };
        for (offset, row) in rows.clone().enumerate() {
            let y = content
                .y
                .saturating_add(offset.min(usize::from(u16::MAX)) as u16);
            if y >= content.bottom() {
                break;
            }
            if row >= len {
                // the fetch-more row
                let more = Rect {
                    x: content.x,
                    y,
                    width: content.width,
                    height: 1,
                };
                let ms =
                    self.ov
                        .style(ui, self.id, Family::GRID, Variant::DEFAULT, Part::ROW, live);
                ui.fill(more, ms.style);
                let used = ui.glyph(
                    Rect {
                        x: more.x.saturating_add(2),
                        width: more.width.saturating_sub(2),
                        ..more
                    },
                    GlyphRole::MoreRows,
                    ms.style,
                );
                ui.paint_str(
                    Rect {
                        x: more.x.saturating_add(3).saturating_add(used),
                        width: more.width.saturating_sub(3).saturating_sub(used),
                        ..more
                    },
                    "more",
                    ms.style,
                );
                if !inert {
                    ui.register_part(self.id, PartRef::of(Part::ROW), more);
                }
                continue;
            }
            self.draw_row(ui, y, row, &row_paint);
        }
        self.draw_actions(ui, bar, live);
        area
    }

    /// Paint the action surface (§12.3's action-surface slot).
    fn draw_actions(&self, ui: &mut Ui<'_>, bar: Rect, live: StateFlags) {
        let Some(f) = self.actions else { return };
        if bar.is_empty() {
            return;
        }
        let s = self.ov.style(
            ui,
            self.id,
            Family::GRID,
            Variant::DEFAULT,
            Part::ACTIONS,
            live,
        );
        ui.fill(bar, s.style);
        self.ov.slot_for(Part::ACTIONS).unwrap_or(f)(ui, bar);
    }

    /// The natural size: the sampled column total, and whatever height is
    /// offered.
    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        let mut w: u16 = 2;
        for col in self.columns.iter().take(self.column_count()) {
            w = w
                .saturating_add(col.min_width.max(width(col.title)))
                .saturating_add(1);
        }
        Size {
            min: (12, 3),
            preferred: (w, c.max.1),
        }
        .fit(c)
    }
}

/// Encode a column key in the registry's collection-item namespace.
const fn column_item_key(key: ColumnKey) -> ItemKey {
    ItemKey::num(key.raw() as u64)
}

impl Bindings for Grid<'_> {
    type Cmd = GridCmd;

    fn bindings(&self, _s: BindingState) -> &'static [Binding<GridCmd>] {
        &BINDINGS
    }
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::{Position, Rect};

    use super::*;
    use crate::event::MouseKind;
    use crate::runtime::stub::{key, mouse};
    use crate::runtime::{App, Runtime};
    use crate::theme::Theme;
    use crate::validate::FieldError;

    const ID: Id = Id::root("grid.tests");
    const AREA: Rect = Rect::new(0, 0, 30, 6);
    const CELL_ACTION: ActionKey = ActionKey::custom("follow");
    const CELL_ACTIONS: [CellAction; 1] = [CellAction::new(CELL_ACTION)];

    fn columns() -> [Column<'static>; 2] {
        [
            Column {
                key: ColumnKey::num(1),
                title: "Name",
                subtitle: None,
                align: Align::Left,
                min_width: 8,
                max_width: 8,
                sortable: true,
                editable: true,
                sticky: false,
                prefix_glyph: None,
                badge: None,
            },
            Column {
                key: ColumnKey::num(2),
                title: "Value",
                subtitle: None,
                align: Align::Left,
                min_width: 8,
                max_width: 8,
                sortable: false,
                editable: true,
                sticky: false,
                prefix_glyph: None,
                badge: None,
            },
        ]
    }

    #[derive(Clone, Copy, Debug, Default)]
    enum Mode {
        #[default]
        Inline,
        Cycle,
        External,
        Refuse,
    }

    #[derive(Default)]
    struct Model {
        rows: Vec<(ItemKey, [&'static str; 2])>,
        mode: Mode,
        cycles: usize,
        commits: Vec<String>,
        fail_commit: bool,
        locked: bool,
        reason: Option<&'static str>,
        cell_action: bool,
    }

    impl Model {
        fn two() -> Self {
            Model {
                rows: vec![
                    (ItemKey::num(10), ["alpha", "1"]),
                    (ItemKey::num(20), ["beta", "2"]),
                ],
                ..Model::default()
            }
        }
    }

    impl GridModel for Model {
        fn row_count(&self) -> usize {
            self.rows.len()
        }

        fn row_key(&self, row: usize) -> ItemKey {
            self.rows.get(row).map_or(ItemKey::index(row), |r| r.0)
        }

        fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
            self.rows
                .get(row)
                .and_then(|r| r.1.get(col))
                .copied()
                .map(CellRef::new)
        }

        fn read_only_reason(&self) -> Option<&str> {
            self.reason
        }

        fn actions(&self, row: usize, col: usize) -> &[CellAction] {
            if self.cell_action && row == 0 && col == 0 {
                &CELL_ACTIONS
            } else {
                &[]
            }
        }
    }

    impl GridEditor for Model {
        fn edit_intent(&self, _row: usize, _col: usize) -> EditIntent<'_> {
            match self.mode {
                Mode::Inline => EditIntent::Inline { initial: "alpha" },
                Mode::Cycle => EditIntent::Cycle,
                Mode::External => EditIntent::External,
                Mode::Refuse => EditIntent::Refuse { reason: "locked" },
            }
        }

        fn apply_cycle(&mut self, _row: usize, _col: usize) {
            self.cycles = self.cycles.saturating_add(1);
        }

        fn commit_cell(&mut self, _row: usize, _col: usize, text: &str) -> Result<(), FieldError> {
            if self.fail_commit {
                Err(FieldError::coded("rejected", "grid-test"))
            } else {
                self.commits.push(text.to_owned());
                Ok(())
            }
        }

        fn is_editable(&self, _row: usize, _col: usize) -> bool {
            !self.locked
        }
    }

    #[derive(Default)]
    struct RaggedModel {
        second_present: Cell<bool>,
        absent_decor: Cell<usize>,
        absent_actions: Cell<usize>,
        editor_hooks: Cell<usize>,
    }

    impl GridModel for RaggedModel {
        fn row_count(&self) -> usize {
            1
        }

        fn row_key(&self, _row: usize) -> ItemKey {
            ItemKey::num(1)
        }

        fn cell(&self, _row: usize, col: usize) -> Option<CellRef<'_>> {
            match col {
                0 => Some(CellRef::new("present")),
                1 if self.second_present.get() => Some(CellRef::new("action")),
                _ => None,
            }
        }

        fn cell_decor(&self, _row: usize, col: usize) -> CellDecor<'_> {
            if col == 1 {
                self.absent_decor
                    .set(self.absent_decor.get().saturating_add(1));
            }
            CellDecor::default()
        }

        fn actions(&self, _row: usize, col: usize) -> &[CellAction] {
            if col == 1 && !self.second_present.get() {
                self.absent_actions
                    .set(self.absent_actions.get().saturating_add(1));
            }
            if col == 1 && self.second_present.get() {
                &CELL_ACTIONS
            } else {
                &[]
            }
        }
    }

    impl GridEditor for RaggedModel {
        fn edit_intent(&self, _row: usize, _col: usize) -> EditIntent<'_> {
            self.editor_hooks
                .set(self.editor_hooks.get().saturating_add(1));
            EditIntent::Cycle
        }

        fn apply_cycle(&mut self, _row: usize, _col: usize) {
            self.editor_hooks
                .set(self.editor_hooks.get().saturating_add(1));
        }

        fn commit_cell(&mut self, _row: usize, _col: usize, _text: &str) -> Result<(), FieldError> {
            self.editor_hooks
                .set(self.editor_hooks.get().saturating_add(1));
            Ok(())
        }

        fn is_editable(&self, _row: usize, _col: usize) -> bool {
            self.editor_hooks
                .set(self.editor_hooks.get().saturating_add(1));
            true
        }
    }

    struct RaggedApp {
        state: GridState,
        model: RaggedModel,
        actions: Vec<GridAction>,
    }

    impl App for RaggedApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let mut columns = columns();
            if let Some(column) = columns.get_mut(1) {
                column.sortable = true;
            }
            let mut response =
                Grid::new(ID, &columns).update_editable(cx, &mut self.state, &mut self.model);
            if let Some(action) = response.take_action() {
                self.actions.push(action);
            }
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let mut columns = columns();
            if let Some(column) = columns.get_mut(1) {
                column.sortable = true;
            }
            Grid::new(ID, &columns).draw(ui, AREA, &self.state, &self.model);
        }
    }

    struct DisplayOnlyModel;

    impl GridModel for DisplayOnlyModel {
        fn row_count(&self) -> usize {
            1
        }

        fn row_key(&self, _row: usize) -> ItemKey {
            ItemKey::num(1)
        }

        fn cell(&self, _row: usize, col: usize) -> Option<CellRef<'_>> {
            (col == 0).then_some(CellRef::new("display only"))
        }
    }

    struct DisplayOnlyApp {
        state: GridState,
        model: DisplayOnlyModel,
    }

    impl App for DisplayOnlyApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            Grid::new(ID, &columns())
                .update(cx, &mut self.state, &self.model)
                .erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            Grid::new(ID, &columns()).draw(ui, AREA, &self.state, &self.model);
        }
    }

    struct GridApp {
        state: GridState,
        model: Model,
        editable: bool,
        actions: Vec<GridAction>,
    }

    impl GridApp {
        fn new(model: Model, editable: bool) -> Self {
            GridApp {
                state: GridState::default(),
                model,
                editable,
                actions: Vec::new(),
            }
        }
    }

    impl App for GridApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let columns = columns();
            let grid = Grid::new(ID, &columns).select_mode(SelectMode::Multi);
            let mut response = if self.editable {
                grid.update_editable(cx, &mut self.state, &mut self.model)
            } else {
                grid.update(cx, &mut self.state, &self.model)
            };
            if let Some(action) = response.take_action() {
                self.actions.push(action);
            }
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            Grid::new(ID, &columns())
                .select_mode(SelectMode::Multi)
                .draw(ui, AREA, &self.state, &self.model);
        }
    }

    struct AreaGridApp {
        state: GridState,
        model: Model,
        area: Rect,
    }

    impl App for AreaGridApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            Grid::new(ID, &columns())
                .update(cx, &mut self.state, &self.model)
                .erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            Grid::new(ID, &columns()).draw(ui, self.area, &self.state, &self.model);
        }
    }

    fn runtime(model: Model, editable: bool) -> (Runtime<GridApp>, Buffer) {
        let mut runtime = Runtime::new(GridApp::new(model, editable), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer);
        runtime.draw_buffer(AREA, &mut buffer);
        (runtime, buffer)
    }

    /// Two columns declared with the same [`ColumnKey::of`] name that folds
    /// alike, or a copy-pasted [`ColumnKey::num`], used to resolve silently
    /// to the first column. `Column`s carry the titles, so the grid can name
    /// both offenders.
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "columns \"Name\" and \"Value\" share ColumnKey::num(1)")]
    fn duplicate_column_keys_are_rejected_by_name() {
        let mut columns = columns();
        if let Some(column) = columns.get_mut(1) {
            column.key = ColumnKey::num(1);
        }
        Grid::new(ID, &columns).assert_distinct_column_keys();
    }

    #[test]
    fn right_header_overflow_reserves_space_before_truncating_the_title() {
        let columns = [
            Column {
                key: ColumnKey::num(1),
                title: "total_amount_and_more",
                subtitle: None,
                align: Align::Left,
                min_width: 18,
                max_width: 18,
                sortable: false,
                editable: false,
                sticky: false,
                prefix_glyph: None,
                badge: None,
            },
            Column {
                key: ColumnKey::num(2),
                title: "second",
                subtitle: None,
                align: Align::Left,
                min_width: 8,
                max_width: 8,
                sortable: false,
                editable: false,
                sticky: false,
                prefix_glyph: None,
                badge: None,
            },
        ];
        let model = Model::two();
        let area = Rect::new(0, 0, 20, 5);
        let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime.draw_scene(area, &mut buffer, |ui, _| {
            Grid::new(ID, &columns).draw(ui, area, &GridState::default(), &model);
        });

        assert_eq!(
            buffer
                .cell(Position::new(17, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some("…")
        );
        assert_eq!(
            buffer
                .cell(Position::new(18, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some("1")
        );
        assert_eq!(
            buffer
                .cell(Position::new(19, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some("›")
        );
    }

    /// The check is wired to the entry points, not merely available.
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "share ColumnKey::num(1)")]
    fn draw_rejects_duplicate_column_keys() {
        let mut columns = columns();
        if let Some(column) = columns.get_mut(1) {
            column.key = ColumnKey::num(1);
        }
        let model = Model::two();
        let screen = AREA;
        let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(screen);
        runtime.draw_scene(screen, &mut buffer, |ui, _| {
            Grid::new(ID, &columns).draw(ui, screen, &GridState::default(), &model);
        });
    }

    /// `0x8000..` is `ColumnKey::of`'s half. Masking a number into it — the
    /// old behaviour — turned `num(0x8000)` into column `0` and `num(0xFFFF)`
    /// into `num(0x7FFF)`, two silent aliases; rejection is the fix.
    #[test]
    #[should_panic(expected = "ColumnKey::num is 0..=0x7FFF")]
    fn column_key_num_rejects_the_hashed_range() {
        let _ = ColumnKey::num(core::hint::black_box(0x8000));
    }

    #[test]
    fn column_key_num_keeps_every_number_it_accepts() {
        for n in [0_u16, 1, 255, 0x7FFE, COLUMN_KEY_MAX] {
            assert_eq!(ColumnKey::num(n).raw(), n, "num must not fold {n}");
            assert!(ColumnKey::num(n).raw() <= COLUMN_KEY_MAX);
        }
    }

    /// The two constructors partition the key space, so no numbered column
    /// can ever be confused with a named one.
    #[test]
    fn numbered_and_named_column_keys_never_meet() {
        for name in ["id", "name", "schema.table", "", "\u{1f600}"] {
            let named = ColumnKey::of(name);
            assert!(named.raw() > COLUMN_KEY_MAX, "{name:?} left the high half");
            for n in [0_u16, 1, 255, 0x7FFF] {
                assert_ne!(named, ColumnKey::num(n));
            }
        }
    }

    /// One fold for every 16-bit custom key in the crate: the grid no longer
    /// carries a private 32-bit FNV of its own.
    #[test]
    fn column_key_of_uses_the_shared_custom_fold() {
        for name in ["id", "name", "schema.table", ""] {
            assert_eq!(ColumnKey::of(name).raw(), custom_hash16(name));
            assert_eq!(ColumnKey::of(name).raw(), Part::custom(name).raw());
        }
        assert_ne!(ColumnKey::of("id"), ColumnKey::of("name"));
        assert_eq!(ColumnKey::of("id"), ColumnKey::of("id"));
    }

    /// A key prints the constructor that could have made it, so a capture or
    /// a failed assertion names the range it came from.
    #[test]
    fn column_key_debug_names_its_range() {
        assert_eq!(format!("{:?}", ColumnKey::num(2)), "ColumnKey::num(2)");
        assert_eq!(
            format!("{:?}", ColumnKey::of("id")),
            format!("ColumnKey::of(#{:04x})", ColumnKey::of("id").raw())
        );
    }

    /// A column index past the list has *no* key. The old sentinel
    /// (`ColumnKey(u16::MAX)`) lived inside `ColumnKey::of`'s range, so a
    /// named column could equal "the column that does not exist".
    #[test]
    fn an_absent_column_index_has_no_key_rather_than_a_sentinel() {
        let columns = columns();
        let grid = Grid::new(ID, &columns);
        assert_eq!(grid.col_key(0), Some(ColumnKey::num(1)));
        assert_eq!(grid.col_key(1), Some(ColumnKey::num(2)));
        assert_eq!(grid.col_key(2), None);
        assert_eq!(grid.col_key(usize::MAX), None);
        // the sentinel that used to stand in for "absent" is a reachable
        // named key, which is why absence may not be spelled as a key
        const { assert!(u16::MAX > COLUMN_KEY_MAX) };
    }

    #[test]
    fn parts_and_slot_contract_are_exact() {
        assert_eq!(
            Grid::PARTS,
            &[
                Part::CONTAINER,
                Part::HEADER,
                Part::ROW,
                Part::CELL,
                Part::TRACK,
                Part::THUMB,
                Part::OVERFLOW,
                Part::EMPTY,
                Part::ACTIONS,
            ]
        );
        let addressable = [
            Part::HEADER,
            Part::EMPTY,
            Part::ACTIONS,
            Part::TRACK,
            Part::THUMB,
        ];
        assert!(addressable.iter().all(|part| Grid::PARTS.contains(part)));
    }

    #[test]
    fn grid_has_no_whole_surface_status_configuration() {
        let columns = columns();
        let debug = format!("{:?}", Grid::new(ID, &columns));
        assert!(!debug.contains("status"));
        assert_eq!(Grid::PARTS.len(), 9);
        assert!(!Grid::PARTS.contains(&Part::ICON));
    }

    #[test]
    fn zero_rows_render_explicit_loading_and_error_empty_states() {
        fn render(empty: EmptyState<'_>) -> String {
            let columns = columns();
            let model = Model::default();
            let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(AREA);
            runtime.draw_scene(AREA, &mut buffer, |ui, _| {
                Grid::new(ID, &columns)
                    .empty(empty)
                    .draw(ui, AREA, &GridState::default(), &model);
            });
            buffer
                .content()
                .iter()
                .map(ratatui_core::buffer::Cell::symbol)
                .collect()
        }

        assert!(
            render(EmptyState::Loading {
                label: "Loading rows"
            })
            .contains("Loading rows")
        );
        assert!(
            render(EmptyState::Error {
                message: "Load failed",
                detail: Some("Retry later"),
            })
            .contains("Load failed")
        );
    }

    #[test]
    fn slot_surface_equals_the_documented_invariant_r_set() {
        fn render(slot: Option<Part>) -> Buffer {
            let screen = Rect::new(0, 0, 20, 10);
            let replace = |ui: &mut Ui<'_>, area: Rect| {
                ui.paint_str(area, "#", ui.surface_style());
            };
            let columns = columns();
            let mut model = Model::two();
            model.cell_action = true;
            model
                .rows
                .extend((0..20).map(|i| (ItemKey::num(100 + i), ["overflow", "row"])));
            let empty = Model::default();
            let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(screen);
            runtime.draw_scene(screen, &mut buffer, |ui, _| {
                let mut full = Grid::new(ID, &columns).actions_slot(&replace);
                let mut blank = Grid::new(ID.sub("empty"), &columns);
                if let Some(part) = slot {
                    full = full.slot(part, &replace);
                    blank = blank.slot(part, &replace);
                }
                full.draw(ui, Rect::new(0, 0, 12, 6), &GridState::default(), &model);
                blank.draw(ui, Rect::new(0, 7, 12, 3), &GridState::default(), &empty);
            });
            buffer
        }

        let baseline = render(None);
        let addressable = [
            Part::HEADER,
            Part::EMPTY,
            Part::ACTIONS,
            Part::TRACK,
            Part::THUMB,
        ];
        for &part in Grid::PARTS {
            assert_eq!(
                render(Some(part)) != baseline,
                addressable.contains(&part),
                "slot truth disagrees for {part:?}"
            );
        }
    }

    #[test]
    fn state_accessors_expose_only_keyed_owned_state() {
        let mut state = GridState::default();
        state.set_cursor(1, ItemKey::num(20), 1, ColumnKey::num(2));
        state.core.checked_mut().insert(ItemKey::num(20));
        state.col_offset = 3;
        state.edit = Some((ItemKey::num(20), ColumnKey::num(2)));
        state
            .editor
            .set_error(Some(FieldError::coded("bad", "typed")));

        assert_eq!(state.cursor(), Some((ItemKey::num(20), ColumnKey::num(2))));
        assert!(state.selected_rows().contains(ItemKey::num(20)));
        assert!(state.is_editing());
        assert_eq!(state.edit_error().and_then(|e| e.code), Some("typed"));
        assert_eq!(state.col_offset(), 3);
    }

    #[test]
    fn grid_model_only_supports_read_only_update_and_draw() {
        let app = DisplayOnlyApp {
            state: GridState::default(),
            model: DisplayOnlyModel,
        };
        let mut runtime = Runtime::new(app, Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer);
        runtime.draw_buffer(AREA, &mut buffer);
        let _ = runtime.handle(key(KeyCode::Right));

        assert_eq!(
            runtime.app().state.cursor(),
            Some((ItemKey::num(1), ColumnKey::num(2)))
        );
    }

    #[test]
    fn read_only_f2_never_activates_but_enter_still_does() {
        let (mut runtime, _) = runtime(Model::two(), false);

        let f2 = runtime.handle(key(KeyCode::F(2)));
        assert!(f2.is_consumed());
        assert!(runtime.app().actions.is_empty());

        let enter = runtime.handle(key(KeyCode::Enter));
        assert!(enter.is_consumed());
        assert_eq!(
            runtime.app().actions,
            [GridAction::Activated(ItemKey::num(10))]
        );
    }

    #[test]
    fn explicit_edit_has_no_activation_fallback_for_a_locked_cell() {
        let mut model = Model::two();
        model.locked = true;
        let (mut runtime, _) = runtime(model, true);

        let f2 = runtime.handle(key(KeyCode::F(2)));
        assert!(f2.is_consumed());
        assert!(!runtime.app().state.is_editing());
        assert!(runtime.app().actions.is_empty());

        let enter = runtime.handle(key(KeyCode::Enter));
        assert!(enter.is_consumed());
        assert!(!runtime.app().state.is_editing());
        assert_eq!(
            runtime.app().actions,
            [GridAction::Activated(ItemKey::num(10))]
        );
    }

    #[test]
    fn ragged_holes_remain_navigable_without_cell_hooks() {
        let app = RaggedApp {
            state: GridState::default(),
            model: RaggedModel::default(),
            actions: Vec::new(),
        };
        let mut runtime = Runtime::new(app, Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer);
        runtime.draw_buffer(AREA, &mut buffer);
        let _ = runtime.handle(key(KeyCode::Right));
        let _ = runtime.handle(key(KeyCode::Enter));

        assert_eq!(
            runtime.app().state.cursor(),
            Some((ItemKey::num(1), ColumnKey::num(2)))
        );
        assert_eq!(
            runtime.app().actions,
            [GridAction::Moved, GridAction::Activated(ItemKey::num(1))]
        );
        assert_eq!(runtime.app().model.absent_decor.get(), 0);
        assert_eq!(runtime.app().model.absent_actions.get(), 0);
        assert_eq!(runtime.app().model.editor_hooks.get(), 0);

        let header = runtime
            .area_of_part(
                ID,
                PartRef::item(Part::HEADER, column_item_key(ColumnKey::num(2))),
            )
            .unwrap_or(Rect::ZERO);
        assert!(!header.is_empty());
        let _ = runtime.handle(mouse(MouseKind::Down, header.x, header.y));
        let _ = runtime.handle(mouse(MouseKind::Up, header.x, header.y));
        assert_eq!(
            runtime.app().actions.last(),
            Some(&GridAction::Sort(ColumnKey::num(2), SortDir::Asc))
        );

        let pointer_app = RaggedApp {
            state: GridState::default(),
            model: RaggedModel::default(),
            actions: Vec::new(),
        };
        let mut pointer = Runtime::new(pointer_app, Theme::junie());
        pointer.draw_buffer(AREA, &mut buffer);
        pointer.draw_buffer(AREA, &mut buffer);
        let hole = pointer
            .area_of_part(ID, PartRef::item(Part::CELL, ItemKey::num(1)))
            .unwrap_or(Rect::ZERO);
        assert!(!hole.is_empty());
        assert_eq!(hole.x, 11);
        let _ = pointer.handle(mouse(MouseKind::Down, hole.x, hole.y));
        assert_eq!(
            pointer.app().state.cursor(),
            Some((ItemKey::num(1), ColumnKey::num(2)))
        );
        assert_eq!(pointer.app().model.absent_decor.get(), 0);
        assert_eq!(pointer.app().model.absent_actions.get(), 0);
        assert_eq!(pointer.app().model.editor_hooks.get(), 0);

        let columns = columns();
        let grid = Grid::new(ID, &columns);
        assert_eq!(
            grid.copy_tsv(&runtime.app().state, &runtime.app().model, (0, 1)),
            "\n"
        );
    }

    #[test]
    fn ragged_hole_resolves_and_fills_cell_patch() {
        let patch = [(Part::CELL, StylePatch::new().add(Modifier::BOLD))];
        let columns = columns();
        let model = RaggedModel::default();
        let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_scene(AREA, &mut buffer, |ui, _| {
            Grid::new(ID, &columns).patch_part(&patch).draw(
                ui,
                AREA,
                &GridState::default(),
                &model,
            );
        });

        assert!(
            buffer
                .cell(Position::new(11, 1))
                .is_some_and(|cell| cell.modifier.contains(Modifier::BOLD))
        );
        assert_eq!(model.absent_decor.get(), 0);
        assert_eq!(model.absent_actions.get(), 0);
    }

    #[test]
    fn stale_cell_action_intent_rechecks_cell_presence() {
        let model = RaggedModel {
            second_present: Cell::new(true),
            ..RaggedModel::default()
        };
        let app = RaggedApp {
            state: GridState::default(),
            model,
            actions: Vec::new(),
        };
        let mut runtime = Runtime::new(app, Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer);
        runtime.draw_buffer(AREA, &mut buffer);
        let affordance = runtime
            .area_of_part(ID, PartRef::item(Part::ACTIONS, ItemKey::num(1)))
            .unwrap_or(Rect::ZERO);
        assert!(!affordance.is_empty());

        let _ = runtime.handle(mouse(MouseKind::Down, affordance.x, affordance.y));
        runtime.app().model.second_present.set(false);
        let _ = runtime.handle(mouse(MouseKind::Up, affordance.x, affordance.y));

        assert!(
            runtime
                .app()
                .actions
                .iter()
                .all(|action| !matches!(action, GridAction::CellAction(..)))
        );
        assert_eq!(runtime.app().model.absent_actions.get(), 0);
    }

    #[test]
    fn cell_alignment_distinguishes_inheritance_from_explicit_left() {
        struct AlignmentModel(Option<Align>);

        impl GridModel for AlignmentModel {
            fn row_count(&self) -> usize {
                1
            }

            fn row_key(&self, _row: usize) -> ItemKey {
                ItemKey::num(1)
            }

            fn cell(&self, _row: usize, _col: usize) -> Option<CellRef<'_>> {
                Some(match self.0 {
                    Some(align) => CellRef::new("x").align(align),
                    None => CellRef::new("x"),
                })
            }
        }

        fn render(column_align: Align, cell_align: Option<Align>) -> Buffer {
            let screen = Rect::new(0, 0, 12, 3);
            let columns = [Column {
                align: column_align,
                min_width: 8,
                max_width: 8,
                ..Column::new(ColumnKey::num(1), "value")
            }];
            let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(screen);
            runtime.draw_scene(screen, &mut buffer, |ui, _| {
                Grid::new(ID, &columns).draw(
                    ui,
                    screen,
                    &GridState::default(),
                    &AlignmentModel(cell_align),
                );
            });
            buffer
        }

        let inherited = render(Align::Right, None);
        let explicit_left = render(Align::Right, Some(Align::Left));
        let explicit_center = render(Align::Left, Some(Align::Center));
        let explicit_right = render(Align::Left, Some(Align::Right));
        assert_eq!(CellRef::new("x").align, None);
        assert_eq!(
            CellRef::new("x").align(Align::Left).align,
            Some(Align::Left)
        );
        assert_eq!(
            inherited
                .cell(Position::new(9, 1))
                .map(ratatui_core::buffer::Cell::symbol),
            Some("x")
        );
        assert_eq!(
            explicit_left
                .cell(Position::new(2, 1))
                .map(ratatui_core::buffer::Cell::symbol),
            Some("x")
        );
        assert_eq!(
            explicit_center
                .cell(Position::new(5, 1))
                .map(ratatui_core::buffer::Cell::symbol),
            Some("x")
        );
        assert_eq!(
            explicit_right
                .cell(Position::new(9, 1))
                .map(ratatui_core::buffer::Cell::symbol),
            Some("x")
        );
    }

    #[test]
    fn model_cell_calls_never_exceed_max_columns() {
        struct BoundedModel {
            largest_column: Cell<usize>,
            out_of_bounds_calls: Cell<usize>,
        }

        impl BoundedModel {
            fn record(&self, col: usize) {
                self.largest_column.set(self.largest_column.get().max(col));
                if col >= GRID_MAX_COLUMNS {
                    self.out_of_bounds_calls
                        .set(self.out_of_bounds_calls.get().saturating_add(1));
                }
            }
        }

        impl GridModel for BoundedModel {
            fn row_count(&self) -> usize {
                1
            }

            fn row_key(&self, _row: usize) -> ItemKey {
                ItemKey::num(1)
            }

            fn cell(&self, _row: usize, col: usize) -> Option<CellRef<'_>> {
                self.record(col);
                Some(CellRef::new("x"))
            }

            fn cell_decor(&self, _row: usize, col: usize) -> CellDecor<'_> {
                self.record(col);
                CellDecor::default()
            }

            fn actions(&self, _row: usize, col: usize) -> &[CellAction] {
                self.record(col);
                &[]
            }
        }

        // distinct keys: a repeated key is itself a rejected mistake now
        let mut columns = [Column::new(ColumnKey::num(0), "x"); GRID_MAX_COLUMNS + 2];
        for (index, column) in columns.iter_mut().enumerate() {
            column.key = ColumnKey::num(index as u16);
        }
        let model = BoundedModel {
            largest_column: Cell::new(0),
            out_of_bounds_calls: Cell::new(0),
        };
        let screen = Rect::new(0, 0, 200, 3);
        let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(screen);
        runtime.draw_scene(screen, &mut buffer, |ui, _| {
            Grid::new(ID, &columns).draw(ui, screen, &GridState::default(), &model);
        });
        let mut state = GridState::default();
        state.set_cursor(
            0,
            ItemKey::num(1),
            GRID_MAX_COLUMNS - 1,
            ColumnKey::num(GRID_MAX_COLUMNS.saturating_sub(1) as u16),
        );
        let _ = Grid::new(ID, &columns).copy_tsv(
            &state,
            &model,
            (0, GRID_MAX_COLUMNS.saturating_sub(1)),
        );

        assert_eq!(model.largest_column.get(), GRID_MAX_COLUMNS - 1);
        assert_eq!(model.out_of_bounds_calls.get(), 0);
    }

    #[test]
    fn nonzero_origin_tiny_grid_never_places_a_column_outside_its_area() {
        let screen = Rect::new(0, 0, 12, 10);
        let area = Rect::new(5, 5, 2, 2);
        let columns = columns();
        let model = Model::two();
        let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(screen);
        runtime.draw_scene(screen, &mut buffer, |ui, _| {
            Grid::new(ID, &columns).draw(ui, area, &GridState::default(), &model);
        });

        for y in 0..screen.height {
            for x in 0..screen.width {
                if !area.contains(Position::new(x, y)) {
                    assert_eq!(
                        buffer
                            .cell(Position::new(x, y))
                            .map(ratatui_core::buffer::Cell::symbol),
                        Some(" "),
                        "grid painted outside {area:?} at ({x}, {y})"
                    );
                }
            }
        }
        let geometry = Grid::new(ID, &columns).geometry(
            Rect::new(5, 6, 2, 1),
            &GridState::default(),
            &model,
            0..1,
        );
        assert!(geometry.shown.iter().all(|shown| !shown));
    }

    #[test]
    fn oversized_first_column_is_clipped_to_positive_available_width() {
        let screen = Rect::new(0, 0, 14, 10);
        let area = Rect::new(5, 5, 4, 3);
        let body = Rect::new(5, 6, 4, 2);
        let columns = columns();
        let model = Model::two();
        let mut state = GridState::default();
        state.set_cursor(0, ItemKey::num(10), 1, ColumnKey::num(2));
        let grid = Grid::new(ID, &columns);
        let geometry = grid.geometry(body, &state, &model, 0..2);
        assert_eq!(geometry.cell(0, body.y), Rect::new(7, 6, 2, 1));
        assert!(geometry.cell(1, body.y).is_empty());
        assert_eq!(geometry.column_at(8), Some(0));
        assert_eq!(geometry.column_at(body.right()), None);

        let app = AreaGridApp { state, model, area };
        let mut runtime = Runtime::new(app, Theme::junie());
        let mut buffer = Buffer::empty(screen);
        runtime.draw_buffer(screen, &mut buffer);
        runtime.draw_buffer(screen, &mut buffer);
        for y in 0..screen.height {
            for x in 0..screen.width {
                if !area.contains(Position::new(x, y)) {
                    assert_eq!(
                        buffer
                            .cell(Position::new(x, y))
                            .map(ratatui_core::buffer::Cell::symbol),
                        Some(" "),
                        "grid painted outside {area:?} at ({x}, {y})"
                    );
                }
            }
        }
        let cell = runtime
            .area_of_part(ID, PartRef::item(Part::CELL, ItemKey::num(10)))
            .unwrap_or(Rect::ZERO);
        assert_eq!(cell, Rect::new(7, 6, 2, 1));
        let header = runtime
            .area_of_part(
                ID,
                PartRef::item(Part::HEADER, column_item_key(ColumnKey::num(1))),
            )
            .unwrap_or(Rect::ZERO);
        assert_eq!(header, Rect::new(7, 5, 2, 1));

        let _ = runtime.handle(mouse(MouseKind::Down, cell.right() - 1, cell.y));
        assert_eq!(
            runtime.app().state.cursor(),
            Some((ItemKey::num(10), ColumnKey::num(1)))
        );
    }

    #[test]
    fn oversized_sticky_column_is_clipped_and_hides_nonsticky_columns_truthfully() {
        let area = Rect::new(0, 0, 10, 3);
        let mut columns = columns();
        if let Some(sticky) = columns.get_mut(0) {
            sticky.sticky = true;
            sticky.min_width = 9;
            sticky.max_width = 9;
        }
        if let Some(nonsticky) = columns.get_mut(1) {
            nonsticky.sortable = true;
        }
        let model = Model::two();
        let state = GridState::default();
        let grid = Grid::new(ID, &columns);
        let geometry = grid.geometry(Rect::new(0, 1, 10, 2), &state, &model, 0..2);
        assert_eq!(geometry.shown.first().copied(), Some(true));
        assert_eq!(geometry.shown.get(1).copied(), Some(false));
        assert_eq!(geometry.hidden_right, 1);
        assert_eq!(geometry.cell(0, 1), Rect::new(2, 1, 8, 1));
        assert!(geometry.cell(1, 1).is_empty());
        assert_eq!(geometry.column_at(area.right()), None);

        let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime.draw_scene(area, &mut buffer, |ui, _| {
            grid.draw(ui, area, &state, &model);
        });
        let mut label = None;
        runtime.draw_scene(area, &mut buffer, |ui, _| {
            grid.draw(ui, area, &state, &model);
            label = grid.cols_label(ui, &state, &model);
        });
        assert_eq!(label.as_deref(), Some("cols 1–1 of 2"));
        assert!(
            runtime
                .area_of_part(
                    ID,
                    PartRef::item(Part::HEADER, column_item_key(ColumnKey::num(2))),
                )
                .is_none()
        );
        let overflow = Theme::junie().design.glyphs.get(GlyphRole::OverflowRight);
        assert!(
            buffer
                .content()
                .iter()
                .any(|cell| cell.symbol() == overflow)
        );
    }

    #[test]
    fn variable_ragged_rows_survive_tiny_layout_measure_and_tsv() {
        struct VariableModel {
            rows: [&'static [&'static str]; 5],
            cell_calls: Cell<usize>,
        }

        impl GridModel for VariableModel {
            fn row_count(&self) -> usize {
                self.rows.len()
            }

            fn row_key(&self, row: usize) -> ItemKey {
                ItemKey::index(row)
            }

            fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
                self.cell_calls.set(self.cell_calls.get().saturating_add(1));
                self.rows
                    .get(row)
                    .and_then(|cells| cells.get(col))
                    .copied()
                    .map(CellRef::new)
            }
        }

        let mut columns = [Column::new(ColumnKey::num(0), "x"); 4];
        for (index, column) in columns.iter_mut().enumerate() {
            column.key = ColumnKey::num(index as u16);
        }
        let model = VariableModel {
            rows: [
                &[],
                &["a"],
                &["b", "c"],
                &["d", "e", "f"],
                &["g", "h", "i", "j"],
            ],
            cell_calls: Cell::new(0),
        };
        let grid = Grid::new(ID, &columns);
        let screen = Rect::new(0, 0, 12, 10);
        for width in 0..=3 {
            for height in 0..=3 {
                let area = Rect::new(5, 5, width, height);
                let mut runtime =
                    Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
                let mut buffer = Buffer::empty(screen);
                let before_measure = model.cell_calls.get();
                runtime.draw_scene(screen, &mut buffer, |ui, _| {
                    let _ = grid.measure(ui, Constraints::loose(width, height));
                    assert_eq!(model.cell_calls.get(), before_measure);
                    grid.draw(ui, area, &GridState::default(), &model);
                });
                for y in 0..screen.height {
                    for x in 0..screen.width {
                        if !area.contains(Position::new(x, y)) {
                            assert_eq!(
                                buffer
                                    .cell(Position::new(x, y))
                                    .map(ratatui_core::buffer::Cell::symbol),
                                Some(" "),
                                "{width}x{height} grid painted outside at ({x}, {y})"
                            );
                        }
                    }
                }
            }
        }

        let mut state = GridState::default();
        state.set_cursor(4, ItemKey::index(4), 3, ColumnKey::num(3));
        state.anchor = Some((ItemKey::index(0), ColumnKey::num(0)));
        assert_eq!(
            grid.copy_tsv(&state, &model, (4, 3)),
            "\t\t\t\na\t\t\t\nb\tc\t\t\nd\te\tf\t\ng\th\ti\tj\n"
        );
    }

    #[test]
    fn sort_is_a_permutation_and_edits_stay_bound_to_the_source_row() {
        let columns = columns();
        let grid = Grid::new(ID, &columns).select_mode(SelectMode::Multi);
        let mut model = Model::two();
        let mut state = GridState::default();
        state.set_cursor(1, ItemKey::num(20), 0, ColumnKey::num(1));
        state.core.checked_mut().insert(ItemKey::num(10));
        state.anchor = Some((ItemKey::num(10), ColumnKey::num(1)));
        state.edit = Some((ItemKey::num(20), ColumnKey::num(1)));
        let _ = state.reconcile(model.row_count(), |i| model.row_key(i));

        model.rows.reverse();
        if state
            .core
            .cursor()
            .is_some_and(|key| model.row_key(state.core.cursor_index()) != key)
        {
            state.invalidate();
        }
        let _ = state.reconcile(model.row_count(), |i| model.row_key(i));

        assert_eq!(state.cursor(), Some((ItemKey::num(20), ColumnKey::num(1))));
        assert!(state.selected_rows().contains(ItemKey::num(10)));
        assert_eq!(state.edit, Some((ItemKey::num(20), ColumnKey::num(1))));
        assert_eq!(grid.range(&state, &model, (0, 0)), Some(((0, 1), (0, 0))));
    }

    #[test]
    fn range_copy_is_tsv() {
        let columns = columns();
        let grid = Grid::new(ID, &columns);
        let model = Model::two();
        let mut state = GridState::default();
        state.set_cursor(1, ItemKey::num(20), 1, ColumnKey::num(2));
        state.anchor = Some((ItemKey::num(10), ColumnKey::num(1)));
        assert_eq!(grid.copy_tsv(&state, &model, (1, 1)), "alpha\t1\nbeta\t2\n");
    }

    #[test]
    fn sortable_header_emits_keyed_requests_without_reordering_the_model() {
        let (mut rt, _) = runtime(Model::two(), false);
        let part = PartRef::item(Part::HEADER, column_item_key(ColumnKey::num(1)));
        let rect = rt.area_of_part(ID, part).unwrap_or(Rect::ZERO);
        assert!(!rect.is_empty());
        let x = rect.x.saturating_add(rect.width / 2);
        let _ = rt.handle(mouse(MouseKind::Down, x, rect.y));
        let _ = rt.handle(mouse(MouseKind::Up, x, rect.y));
        assert_eq!(
            rt.app().actions,
            [GridAction::Sort(ColumnKey::num(1), SortDir::Asc)]
        );
        assert_eq!(
            rt.app().model.rows.iter().map(|r| r.0).collect::<Vec<_>>(),
            [ItemKey::num(10), ItemKey::num(20)]
        );

        let (mut descending, _) = runtime(Model::two(), false);
        descending.app_mut().state.sort = Some((ColumnKey::num(1), SortDir::Asc));
        let rect = descending.area_of_part(ID, part).unwrap_or(Rect::ZERO);
        let x = rect.x.saturating_add(rect.width / 2);
        let _ = descending.handle(mouse(MouseKind::Down, x, rect.y));
        let _ = descending.handle(mouse(MouseKind::Up, x, rect.y));
        assert_eq!(
            descending.app().actions,
            [GridAction::Sort(ColumnKey::num(1), SortDir::Desc)]
        );
    }

    #[test]
    fn edit_intent_inline_cycle_external_refuse() {
        let (mut inline, _) = runtime(Model::two(), true);
        let _ = inline.handle(key(KeyCode::Enter));
        assert!(inline.app().state.is_editing());

        let mut cycle_model = Model::two();
        cycle_model.mode = Mode::Cycle;
        let (mut cycle, _) = runtime(cycle_model, true);
        let _ = cycle.handle(key(KeyCode::Enter));
        assert_eq!(cycle.app().model.cycles, 1);

        let mut external_model = Model::two();
        external_model.mode = Mode::External;
        let (mut external, _) = runtime(external_model, true);
        let _ = external.handle(key(KeyCode::Enter));
        assert_eq!(
            external.app().actions,
            [GridAction::EditRequested(
                ItemKey::num(10),
                ColumnKey::num(1)
            )]
        );

        let mut refuse_model = Model::two();
        refuse_model.mode = Mode::Refuse;
        let (mut refuse, mut refuse_buffer) = runtime(refuse_model, true);
        let _ = refuse.handle(key(KeyCode::Enter));
        refuse.draw_buffer(AREA, &mut refuse_buffer);
        assert_eq!(
            refuse.app().state.edit_error().map(ToString::to_string),
            Some("locked".to_owned())
        );
        assert!(!refuse.app().state.is_editing());
        let refused_text = refuse_buffer
            .content()
            .iter()
            .map(ratatui_core::buffer::Cell::symbol)
            .collect::<String>();
        assert!(refused_text.contains("locked"));
    }

    #[test]
    fn update_editable_commits_through_the_editor() {
        let (mut runtime, _) = runtime(Model::two(), true);
        let _ = runtime.handle(key(KeyCode::Enter));
        runtime.draw_buffer(AREA, &mut Buffer::empty(AREA));
        let _ = runtime.handle(key(KeyCode::Char('x')));
        let _ = runtime.handle(key(KeyCode::Enter));
        assert_eq!(runtime.app().model.commits.len(), 1);
        assert!(!runtime.app().state.is_editing());

        runtime.app_mut().model.fail_commit = true;
        runtime.draw_buffer(AREA, &mut Buffer::empty(AREA));
        let _ = runtime.handle(key(KeyCode::Enter));
        runtime.draw_buffer(AREA, &mut Buffer::empty(AREA));
        let _ = runtime.handle(key(KeyCode::Enter));
        assert!(runtime.app().state.is_editing());
        assert_eq!(
            runtime.app().state.edit_error().and_then(|e| e.code),
            Some("grid-test")
        );

        let _ = runtime.handle(key(KeyCode::Esc));
        assert!(!runtime.app().state.is_editing());
        assert!(runtime.app().state.edit_error().is_none());
    }

    #[test]
    fn click_inside_an_active_inline_edit_goes_to_the_editor() {
        let (mut runtime, _) = runtime(Model::two(), true);
        let _ = runtime.handle(key(KeyCode::Enter));
        runtime.draw_buffer(AREA, &mut Buffer::empty(AREA));
        let editor = runtime.area_of(ID.part(Part::TEXT)).unwrap_or(Rect::ZERO);
        assert!(!editor.is_empty());
        let before = runtime.app().actions.len();
        let _ = runtime.handle(mouse(MouseKind::Down, editor.x, editor.y));
        let _ = runtime.handle(mouse(MouseKind::Up, editor.x, editor.y));
        assert_eq!(runtime.app().actions.len(), before);
        assert!(runtime.app().state.is_editing());
    }

    #[test]
    fn read_only_reason_is_rendered_from_a_grid_model() {
        let mut model = Model::two();
        model.reason = Some("read only by source");
        let (_, buffer) = runtime(model, false);
        let text: String = buffer
            .content()
            .iter()
            .map(ratatui_core::buffer::Cell::symbol)
            .collect();
        assert!(text.contains("read only by source"));
    }

    #[test]
    fn cell_actions_affordance_is_painted_for_a_read_only_model() {
        let mut model = Model::two();
        model.cell_action = true;
        let (mut runtime, buffer) = runtime(model, false);
        let glyph = Theme::junie().design.glyphs.get(GlyphRole::FollowRef);
        assert!(buffer.content().iter().any(|cell| cell.symbol() == glyph));
        let affordance = runtime
            .area_of_part(ID, PartRef::item(Part::ACTIONS, ItemKey::num(10)))
            .unwrap_or(Rect::ZERO);
        assert!(!affordance.is_empty());
        let _ = runtime.handle(mouse(MouseKind::Down, affordance.x, affordance.y));
        let _ = runtime.handle(mouse(MouseKind::Up, affordance.x, affordance.y));
        assert_eq!(
            runtime.app().actions,
            [GridAction::CellAction(
                ItemKey::num(10),
                ColumnKey::num(1),
                CELL_ACTION
            )]
        );
    }

    #[test]
    fn nested_scroll_overrides_and_actions_slot_change_the_declared_parts() {
        let replace = |ui: &mut Ui<'_>, area: Rect| {
            ui.paint_str(area, "#", ui.surface_style());
        };
        let patch = StylePatch::new().add(Modifier::BOLD);
        let parts = [(Part::TRACK, patch), (Part::THUMB, patch)];
        let mut model = Model::two();
        model.cell_action = true;
        model
            .rows
            .extend((0..20).map(|i| (ItemKey::num(100 + i), ["overflow", "row"])));
        let columns = columns();
        let grid = Grid::new(ID, &columns)
            .actions_slot(&replace)
            .patch_part(&parts)
            .slot(Part::ACTIONS, &replace);
        let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_scene(AREA, &mut buffer, |ui, _| {
            grid.draw(ui, AREA, &GridState::default(), &model);
        });

        let hashes = buffer
            .content()
            .iter()
            .filter(|cell| cell.symbol() == "#")
            .count();
        assert!(hashes >= 2, "ACTIONS slot must reach cell and action row");
        for part in [Part::TRACK, Part::THUMB] {
            let area = runtime
                .area_of_part(ID, PartRef::of(part))
                .unwrap_or(Rect::ZERO);
            assert!(!area.is_empty(), "{part:?} was not registered");
            assert!(
                buffer
                    .cell(Position::new(area.x, area.y))
                    .is_some_and(|cell| cell.modifier.contains(Modifier::BOLD)),
                "{part:?} dropped its forwarded patch"
            );
        }
    }

    #[test]
    fn reference_grid_makes_its_nested_scroll_region_inert() {
        let columns = columns();
        let grid = Grid::new(ID, &columns);
        let mut model = Model::two();
        model
            .rows
            .extend((0..20).map(|i| (ItemKey::num(100 + i), ["overflow", "row"])));
        let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_scene(AREA, &mut buffer, |ui, _| {
            ui.reference(
                Some(crate::ReferenceTarget::new(
                    ID,
                    crate::ReferenceState::FOCUSED,
                )),
                |ui| grid.draw(ui, AREA, &GridState::default(), &model),
            );
        });
        assert!(runtime.area_of(ID).is_none());
        assert!(runtime.area_of_part(ID, PartRef::of(Part::TRACK)).is_none());
    }

    #[test]
    fn reference_focus_and_press_cannot_fabricate_row_selection() {
        let columns = columns();
        let grid = Grid::new(ID, &columns).select_mode(SelectMode::Multi);
        let model = Model::two();
        let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        let target = crate::ReferenceTarget::new(
            ID,
            crate::ReferenceState::FOCUSED | crate::ReferenceState::PRESSED,
        )
        .part(PartRef::item(Part::ROW, ItemKey::num(10)));
        runtime.draw_scene(AREA, &mut buffer, |ui, _| {
            ui.reference(Some(target), |ui| {
                grid.draw(ui, AREA, &GridState::default(), &model);
            });
        });
        assert_eq!(
            buffer
                .cell(Position::new(1, 1))
                .map(ratatui_core::buffer::Cell::symbol),
            Some(" ")
        );

        let mut selected = GridState::default();
        selected.core.checked_mut().insert(ItemKey::num(10));
        runtime.draw_scene(AREA, &mut buffer, |ui, _| {
            ui.reference(Some(target), |ui| grid.draw(ui, AREA, &selected, &model));
        });
        assert_eq!(
            buffer
                .cell(Position::new(1, 1))
                .map(ratatui_core::buffer::Cell::symbol),
            Some(Theme::junie().design.glyphs.get(GlyphRole::Checked))
        );
    }

    #[test]
    fn column_reveal_updates_the_exact_horizontal_offset_reader() {
        let mut many = [Column::new(ColumnKey::num(0), "0"); 6];
        for (i, column) in many.iter_mut().enumerate() {
            column.key = ColumnKey::num(i as u16);
            column.title = "column";
        }
        let grid = Grid::new(ID, &many);
        let mut state = GridState::default();
        grid.reveal_column(&mut state, 5);
        assert_eq!(state.col_offset(), 5);
    }

    #[test]
    fn sortable_headers_use_typed_theme_glyphs() {
        let (_, buffer) = runtime(Model::two(), false);
        let glyph = Theme::junie().design.glyphs.get(GlyphRole::SortAsc);
        assert!(buffer.content().iter().any(|cell| cell.symbol() == glyph));
        assert!(buffer.cell(Position::new(0, 0)).is_some());
    }
}
