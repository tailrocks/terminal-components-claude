//! Termrock Grid surface: unified engine supporting Table row-presentation and Cell/DataGrid presentation.
//!
//! Provides the canonical [`Grid`], [`GridModel`], [`GridEditor`], [`GridState`],
//! [`GridAction`], [`GridColumn`], and [`CellValue`] according to canonical contracts.

#![allow(clippy::collapsible_if)]

use std::collections::BTreeSet;
use std::fmt;

use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::{Modifier, Style};

use crate::core::event::MouseKind;
pub use crate::termrock::collections::SelectionMode;
use crate::termrock::empty::Readiness;
use crate::termrock::identity::{CellKey, ColumnKey, Id, ItemKey, Revision};
use crate::termrock::layout::{Constraints, Rect, Size};
use crate::termrock::response::{Flow, Input, Invalidate, Response, UpdateCause};
use crate::termrock::runtime::{Cx, MeasureCx, Ui};
use crate::termrock::scroll::ScrollState;
use crate::termrock::text::{TextEditorCore, truncate, width};
use crate::termrock::theme::{Role, StylePatch, Surface};

/// Sort direction for grid columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

impl SortDirection {
    pub fn toggled(self) -> Self {
        match self {
            Self::Asc => Self::Desc,
            Self::Desc => Self::Asc,
        }
    }

    pub fn arrow(self) -> &'static str {
        match self {
            Self::Asc => "▴",
            Self::Desc => "▾",
        }
    }
}

pub type SortDir = SortDirection;

/// Presentation and navigation mode for the Grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GridMode {
    /// Row-oriented table presentation and navigation.
    #[default]
    Table,
    /// Cell-oriented data-grid presentation, navigation, and editing.
    Cell,
}

/// Navigation unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NavUnit {
    #[default]
    Row,
    Cell,
}

/// Grid presentation recipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GridPresentation {
    #[default]
    Table,
    DataGrid,
}

/// Column width sizing policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ColumnFit {
    #[default]
    Fixed,
    Auto,
    Fill,
}

/// Supported typed cell values.
#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    Null,
    Default,
    Text(String),
    Int(i64),
    Num(f64),
    Bool(bool),
    Json(String),
}

impl CellValue {
    pub fn text(&self) -> String {
        match self {
            Self::Null => "NULL".into(),
            Self::Default => "DEFAULT".into(),
            Self::Text(s) | Self::Json(s) => s.clone(),
            Self::Int(i) => i.to_string(),
            Self::Num(n) => format!("{n:.2}"),
            Self::Bool(b) => b.to_string(),
        }
    }

    pub fn edit_text(&self) -> String {
        match self {
            Self::Null | Self::Default => String::new(),
            _ => self.text(),
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn is_default(&self) -> bool {
        matches!(self, Self::Default)
    }
}

impl fmt::Display for CellValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text())
    }
}

impl From<&str> for CellValue {
    fn from(s: &str) -> Self {
        Self::Text(s.to_owned())
    }
}

impl From<String> for CellValue {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<i64> for CellValue {
    fn from(i: i64) -> Self {
        Self::Int(i)
    }
}

impl From<f64> for CellValue {
    fn from(n: f64) -> Self {
        Self::Num(n)
    }
}

impl From<bool> for CellValue {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

/// Cell semantic kind for alignment and formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CellKind {
    #[default]
    Text,
    Id,
    Number,
    Bool,
    Timestamp,
    Json,
    Enum,
}

impl CellKind {
    pub fn right_aligned(self) -> bool {
        self == Self::Number
    }

    pub fn default_width(self) -> (u16, u16) {
        match self {
            Self::Id => (9, 36),
            Self::Text => (6, 40),
            Self::Number => (4, 22),
            Self::Bool => (5, 5),
            Self::Timestamp => (10, 29),
            Self::Json => (8, 40),
            Self::Enum => (6, 16),
        }
    }
}

/// Specification for a single column in the Grid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridColumn {
    pub key: ColumnKey,
    pub title: String,
    pub width: u16,
    pub sortable: bool,
    pub kind: CellKind,
    pub primary: bool,
    pub nullable: bool,
    pub read_only: bool,
    pub type_label: String,
}

impl GridColumn {
    pub fn new(key: ColumnKey, title: impl Into<String>, width: u16) -> Self {
        Self {
            key,
            title: title.into(),
            width,
            sortable: true,
            kind: CellKind::Text,
            primary: false,
            nullable: true,
            read_only: false,
            type_label: String::new(),
        }
    }

    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    pub fn kind(mut self, kind: CellKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn primary(mut self, primary: bool) -> Self {
        self.primary = primary;
        self
    }

    pub fn nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }

    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn type_label(mut self, label: impl Into<String>) -> Self {
        self.type_label = label.into();
        self
    }
}

/// Read-only source model for Grid.
pub trait GridModel {
    fn revision(&self) -> Revision;
    fn row_count(&self) -> usize;
    fn row_key(&self, index: usize) -> ItemKey;
    fn columns(&self) -> &[GridColumn];
    fn cell_value(&self, row: ItemKey, col: ColumnKey) -> Option<CellValue>;
    fn readiness(&self) -> Readiness<'_> {
        Readiness::Empty
    }
    fn has_more(&self) -> bool {
        false
    }

    fn row_index(&self, key: ItemKey) -> Option<usize> {
        (0..self.row_count()).find(|&i| self.row_key(i) == key)
    }

    fn column_index(&self, key: ColumnKey) -> Option<usize> {
        self.columns().iter().position(|c| c.key == key)
    }
}

/// Explicit opt-in editor model for local drafts, validation, and commits.
pub trait GridEditor: GridModel {
    fn is_editable(&self, row: ItemKey, col: ColumnKey) -> bool;
    fn commit_cell(&mut self, row: ItemKey, col: ColumnKey, value: CellValue) -> bool;

    fn validate_cell(
        &self,
        _row: ItemKey,
        col: ColumnKey,
        text: &str,
    ) -> Result<CellValue, String> {
        let trimmed = text.trim();
        if trimmed.eq_ignore_ascii_case("null") {
            return Ok(CellValue::Null);
        }
        if let Some(c) = self.columns().iter().find(|c| c.key == col) {
            match c.kind {
                CellKind::Number => {
                    if let Ok(i) = trimmed.parse::<i64>() {
                        Ok(CellValue::Int(i))
                    } else if let Ok(n) = trimmed.parse::<f64>() {
                        Ok(CellValue::Num(n))
                    } else {
                        Err("Must be a number".into())
                    }
                }
                CellKind::Bool => match trimmed.to_ascii_lowercase().as_str() {
                    "true" | "t" | "1" | "yes" => Ok(CellValue::Bool(true)),
                    "false" | "f" | "0" | "no" => Ok(CellValue::Bool(false)),
                    _ => Err("Must be true or false".into()),
                },
                CellKind::Json => {
                    if (trimmed.starts_with('{') && trimmed.ends_with('}'))
                        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
                    {
                        Ok(CellValue::Json(trimmed.into()))
                    } else {
                        Err("Must be JSON object or array".into())
                    }
                }
                _ => Ok(CellValue::Text(text.to_owned())),
            }
        } else {
            Ok(CellValue::Text(text.to_owned()))
        }
    }
}

/// Active edit draft within a cell.
#[derive(Debug, Clone)]
pub struct CellDraft {
    pub cell: CellKey,
    pub buffer: TextEditorCore,
    pub error: Option<String>,
}

/// Durable interaction state for the Grid.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GridState {
    pub cursor_row: Option<ItemKey>,
    pub cursor_col: Option<ColumnKey>,
    pub scroll: ScrollState,
    pub h_offset: usize,
    pub selected_rows: BTreeSet<ItemKey>,
    pub selected_cells: BTreeSet<CellKey>,
    pub editing: Option<CellDraft>,
    pub sort: Option<(ColumnKey, SortDirection)>,
    pub last_area: Rect,
    pub drag_anchor: Option<ItemKey>,
}

impl PartialEq for CellDraft {
    fn eq(&self, other: &Self) -> bool {
        self.cell == other.cell
            && self.buffer.text() == other.buffer.text()
            && self.error == other.error
    }
}

impl GridState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cursor_cell(&self) -> Option<CellKey> {
        match (self.cursor_row, self.cursor_col) {
            (Some(row), Some(col)) => Some(CellKey::new(row, col)),
            _ => None,
        }
    }

    pub fn is_editing(&self) -> bool {
        self.editing.is_some()
    }

    pub fn select_row(&mut self, key: ItemKey) {
        self.selected_rows.insert(key);
    }

    pub fn select_cell(&mut self, cell: CellKey) {
        self.selected_cells.insert(cell);
    }

    pub fn clear_selection(&mut self) {
        self.selected_rows.clear();
        self.selected_cells.clear();
    }

    pub fn is_row_selected(&self, key: ItemKey) -> bool {
        self.selected_rows.contains(&key)
    }

    pub fn is_cell_selected(&self, cell: CellKey) -> bool {
        self.selected_cells.contains(&cell)
    }
}

/// Typed actions emitted by Grid.
#[derive(Debug, Clone, PartialEq)]
pub enum GridAction {
    Activate {
        cell: CellKey,
    },
    SelectRow(ItemKey),
    SelectCell(CellKey),
    SelectionRequested(Vec<ItemKey>),
    SortRequested {
        column: ColumnKey,
        direction: SortDirection,
    },
    Edited {
        cell: CellKey,
    },
    EditCommitted {
        cell: CellKey,
        value: CellValue,
    },
    CopyRequested(String),
    FetchMore,
    FilterRequested {
        column: ColumnKey,
        value: CellValue,
    },
    Refresh,
}

/// Unified Grid component engine.
#[derive(Clone)]
pub struct Grid<'a> {
    pub id: Id,
    pub mode: GridMode,
    pub selection_mode: SelectionMode,
    pub readiness: Readiness<'a>,
    pub patch: Option<StylePatch>,
    pub type_row: bool,
}

impl<'a> Grid<'a> {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            mode: GridMode::Table,
            selection_mode: SelectionMode::Single,
            readiness: Readiness::Empty,
            patch: None,
            type_row: false,
        }
    }

    pub fn mode(mut self, mode: GridMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn presentation(mut self, presentation: GridPresentation) -> Self {
        self.mode = match presentation {
            GridPresentation::Table => GridMode::Table,
            GridPresentation::DataGrid => GridMode::Cell,
        };
        self
    }

    pub fn navigation(mut self, nav: NavUnit) -> Self {
        self.mode = match nav {
            NavUnit::Row => GridMode::Table,
            NavUnit::Cell => GridMode::Cell,
        };
        self
    }

    pub fn selection_mode(mut self, mode: SelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    pub fn readiness(mut self, readiness: Readiness<'a>) -> Self {
        self.readiness = readiness;
        self
    }

    pub fn patch(mut self, patch: StylePatch) -> Self {
        self.patch = Some(patch);
        self
    }

    pub fn type_row(mut self, type_row: bool) -> Self {
        self.type_row = type_row;
        self
    }

    /// Read-only update pass. Cannot mutate model.
    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        state: &mut GridState,
        model: &dyn GridModel,
    ) -> Response<GridAction> {
        self.update_internal(cx, state, model, None)
    }

    /// Explicit opt-in editable update pass.
    pub fn update_editable(
        &self,
        cx: &mut Cx<'_>,
        state: &mut GridState,
        model: &mut dyn GridEditor,
    ) -> Response<GridAction> {
        self.update_internal(cx, state, model, Some(()))
    }

    fn update_internal(
        &self,
        cx: &mut Cx<'_>,
        state: &mut GridState,
        model: &dyn GridModel,
        editor_opt: Option<()>,
    ) -> Response<GridAction> {
        let intended = cx.intended_owner();
        let is_target = intended == Some(&self.id)
            || intended.and_then(|id| id.parent()).as_ref() == Some(&self.id);

        let area = cx
            .published_geometry
            .and_then(|g| g.get(&self.id))
            .copied()
            .unwrap_or(state.last_area);

        let row_count = model.row_count();
        let columns = model.columns();

        // Reconcile cursor_row
        if state.cursor_row.is_none() && row_count > 0 {
            state.cursor_row = Some(model.row_key(0));
        } else if let Some(cr) = state.cursor_row {
            if model.row_index(cr).is_none() {
                state.cursor_row = if row_count > 0 {
                    Some(model.row_key(0))
                } else {
                    None
                };
            }
        }

        // Reconcile cursor_col
        if state.cursor_col.is_none() && !columns.is_empty() {
            state.cursor_col = Some(columns[0].key);
        } else if let Some(cc) = state.cursor_col {
            if model.column_index(cc).is_none() {
                state.cursor_col = columns.first().map(|c| c.key);
            }
        }

        let header_height = if self.type_row { 2 } else { 1 };
        let body_height = area.height.saturating_sub(header_height) as usize;
        state.scroll.total = row_count;
        state.scroll.viewport = body_height;
        state.scroll.clamp();

        // Mouse hit handling
        if matches!(cx.cause(), UpdateCause::Input(Input::Mouse(_), _)) {
            if let UpdateCause::Input(Input::Mouse(m), _) = cx.cause() {
                let pos: crate::termrock::layout::Position = m.pos.into();
                match m.kind {
                    MouseKind::WheelUp => {
                        if state.scroll.scroll_up(3) {
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone());
                        }
                    }
                    MouseKind::WheelDown => {
                        if state.scroll.scroll_down(3) {
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone());
                        }
                    }
                    MouseKind::Down if is_target || area.contains(pos) => {
                        cx.request_focus(self.id.clone());
                        // Check header row click for sort
                        let header_rect = Rect::new(area.x, area.y, area.width, header_height);
                        if header_rect.contains(pos) {
                            let rel_x = pos.x.saturating_sub(area.x + 8); // Skip row gutter prefix
                            let mut curr_x = 0u16;
                            for col in columns {
                                let col_w = col.width;
                                if rel_x >= curr_x && rel_x < curr_x + col_w {
                                    if col.sortable {
                                        let next_dir = match state.sort {
                                            Some((k, d)) if k == col.key => d.toggled(),
                                            _ => SortDirection::Asc,
                                        };
                                        state.sort = Some((col.key, next_dir));
                                        cx.request_invalidate(Invalidate::Paint);
                                        return Response::action(
                                            self.id.clone(),
                                            GridAction::SortRequested {
                                                column: col.key,
                                                direction: next_dir,
                                            },
                                        )
                                        .with_flow(Flow::Consumed);
                                    }
                                    break;
                                }
                                curr_x += col_w;
                            }
                        }

                        // Check body row click
                        let rel_y = pos.y.saturating_sub(area.y + header_height) as usize;
                        let clicked_idx = state.scroll.offset + rel_y;
                        if clicked_idx < row_count {
                            let row_key = model.row_key(clicked_idx);
                            let old_row = state.cursor_row;
                            state.cursor_row = Some(row_key);

                            // Find clicked column
                            let rel_x = pos.x.saturating_sub(area.x + 8);
                            let mut curr_x = 0u16;
                            let mut clicked_col = state.cursor_col;
                            for col in columns {
                                if rel_x >= curr_x && rel_x < curr_x + col.width {
                                    clicked_col = Some(col.key);
                                    break;
                                }
                                curr_x += col.width;
                            }
                            let old_col = state.cursor_col;
                            state.cursor_col = clicked_col;

                            // Handle mode selection and editing
                            if self.mode == GridMode::Cell {
                                if let (Some(rk), Some(ck)) = (state.cursor_row, state.cursor_col) {
                                    let cell_key = CellKey::new(rk, ck);
                                    state.selected_cells.clear();
                                    state.selected_cells.insert(cell_key);

                                    // Second click on current cell begins editing
                                    if editor_opt.is_some()
                                        && old_row == Some(rk)
                                        && old_col == Some(ck)
                                        && state.editing.is_none()
                                    {
                                        let val =
                                            model.cell_value(rk, ck).unwrap_or(CellValue::Null);
                                        let mut buffer = TextEditorCore::new();
                                        buffer.set_text(val.edit_text());
                                        state.editing = Some(CellDraft {
                                            cell: cell_key,
                                            buffer,
                                            error: None,
                                        });
                                    }

                                    cx.request_invalidate(Invalidate::Paint);
                                    return Response::action(
                                        self.id.clone(),
                                        GridAction::SelectCell(cell_key),
                                    )
                                    .with_flow(Flow::Consumed);
                                }
                            } else {
                                state.selected_rows.clear();
                                state.selected_rows.insert(row_key);
                                cx.request_invalidate(Invalidate::Paint);
                                return Response::action(
                                    self.id.clone(),
                                    GridAction::SelectRow(row_key),
                                )
                                .with_flow(Flow::Consumed);
                            }
                        } else if model.has_more() && clicked_idx == row_count {
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::action(self.id.clone(), GridAction::FetchMore)
                                .with_flow(Flow::Consumed);
                        }
                    }
                    _ => {}
                }
            }
        }

        // Key handling
        if let UpdateCause::Input(Input::Key(k), _) = cx.cause() {
            if is_target {
                // If editing is active
                if let Some(ref mut draft) = state.editing {
                    match k.code {
                        KeyCode::Esc => {
                            state.editing = None;
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone());
                        }
                        KeyCode::Enter => {
                            let text = draft.buffer.text().to_owned();
                            let cell = draft.cell;
                            let val = CellValue::Text(text.clone());
                            state.editing = None;
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::action(
                                self.id.clone(),
                                GridAction::EditCommitted { cell, value: val },
                            )
                            .with_flow(Flow::Consumed);
                        }
                        KeyCode::Left => {
                            draft.buffer.move_left(false);
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone());
                        }
                        KeyCode::Right => {
                            draft.buffer.move_right(false);
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone());
                        }
                        KeyCode::Backspace => {
                            draft.buffer.delete_backward();
                            draft.error = None;
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone());
                        }
                        KeyCode::Delete => {
                            draft.buffer.delete_forward();
                            draft.error = None;
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone());
                        }
                        KeyCode::Char(c) if !k.mods.contains(KeyModifiers::CONTROL) => {
                            draft.buffer.insert_char(c);
                            draft.error = None;
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone());
                        }
                        _ => {
                            return Response::consumed(self.id.clone());
                        }
                    }
                }

                // Navigation mode
                let current_row_idx = state
                    .cursor_row
                    .and_then(|r| model.row_index(r))
                    .unwrap_or(0);
                let current_col_idx = state
                    .cursor_col
                    .and_then(|c| model.column_index(c))
                    .unwrap_or(0);

                match k.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if current_row_idx > 0 {
                            let next_row = model.row_key(current_row_idx - 1);
                            state.cursor_row = Some(next_row);
                            if current_row_idx - 1 < state.scroll.offset {
                                state.scroll.scroll_to(current_row_idx - 1);
                            }
                            if self.mode == GridMode::Table
                                && self.selection_mode == SelectionMode::Single
                            {
                                state.selected_rows.clear();
                                state.selected_rows.insert(next_row);
                            }
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::action(
                                self.id.clone(),
                                GridAction::SelectRow(next_row),
                            )
                            .with_flow(Flow::Consumed);
                        }
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if current_row_idx + 1 < row_count {
                            let next_row = model.row_key(current_row_idx + 1);
                            state.cursor_row = Some(next_row);
                            if current_row_idx + 1 >= state.scroll.offset + state.scroll.viewport {
                                state
                                    .scroll
                                    .scroll_to(current_row_idx + 2 - state.scroll.viewport);
                            }
                            if self.mode == GridMode::Table
                                && self.selection_mode == SelectionMode::Single
                            {
                                state.selected_rows.clear();
                                state.selected_rows.insert(next_row);
                            }
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::action(
                                self.id.clone(),
                                GridAction::SelectRow(next_row),
                            )
                            .with_flow(Flow::Consumed);
                        }
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        if self.mode == GridMode::Cell {
                            if current_col_idx > 0 {
                                let next_col = columns[current_col_idx - 1].key;
                                state.cursor_col = Some(next_col);
                                if let Some(rk) = state.cursor_row {
                                    let cell = CellKey::new(rk, next_col);
                                    state.selected_cells.clear();
                                    state.selected_cells.insert(cell);
                                    cx.request_invalidate(Invalidate::Paint);
                                    return Response::action(
                                        self.id.clone(),
                                        GridAction::SelectCell(cell),
                                    )
                                    .with_flow(Flow::Consumed);
                                }
                            }
                        } else {
                            state.h_offset = state.h_offset.saturating_sub(4);
                            cx.request_invalidate(Invalidate::Paint);
                        }
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        if self.mode == GridMode::Cell {
                            if current_col_idx + 1 < columns.len() {
                                let next_col = columns[current_col_idx + 1].key;
                                state.cursor_col = Some(next_col);
                                if let Some(rk) = state.cursor_row {
                                    let cell = CellKey::new(rk, next_col);
                                    state.selected_cells.clear();
                                    state.selected_cells.insert(cell);
                                    cx.request_invalidate(Invalidate::Paint);
                                    return Response::action(
                                        self.id.clone(),
                                        GridAction::SelectCell(cell),
                                    )
                                    .with_flow(Flow::Consumed);
                                }
                            }
                        } else {
                            state.h_offset = state.h_offset.saturating_add(4);
                            cx.request_invalidate(Invalidate::Paint);
                        }
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::PageUp => {
                        let delta = state.scroll.viewport.max(1);
                        state.scroll.scroll_up(delta);
                        let new_idx = state.scroll.offset;
                        if new_idx < row_count {
                            state.cursor_row = Some(model.row_key(new_idx));
                        }
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::PageDown => {
                        let delta = state.scroll.viewport.max(1);
                        state.scroll.scroll_down(delta);
                        let new_idx = (state.scroll.offset
                            + state.scroll.viewport.saturating_sub(1))
                        .min(row_count.saturating_sub(1));
                        if row_count > 0 {
                            state.cursor_row = Some(model.row_key(new_idx));
                        }
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::Home => {
                        if row_count > 0 {
                            state.cursor_row = Some(model.row_key(0));
                            state.scroll.scroll_to(0);
                            cx.request_invalidate(Invalidate::Paint);
                        }
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::End => {
                        if row_count > 0 {
                            state.cursor_row = Some(model.row_key(row_count - 1));
                            state.scroll.scroll_to(state.scroll.max_offset());
                            cx.request_invalidate(Invalidate::Paint);
                        }
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::Char(' ') => {
                        if let Some(rk) = state.cursor_row {
                            if self.mode == GridMode::Cell {
                                if let Some(ck) = state.cursor_col {
                                    let cell = CellKey::new(rk, ck);
                                    if state.selected_cells.contains(&cell) {
                                        state.selected_cells.remove(&cell);
                                    } else {
                                        state.selected_cells.insert(cell);
                                    }
                                }
                            } else if state.selected_rows.contains(&rk) {
                                state.selected_rows.remove(&rk);
                            } else {
                                state.selected_rows.insert(rk);
                            }
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone());
                        }
                    }
                    KeyCode::Enter | KeyCode::F(2) => {
                        if let (Some(rk), Some(ck)) = (state.cursor_row, state.cursor_col) {
                            let cell = CellKey::new(rk, ck);
                            if self.mode == GridMode::Cell && editor_opt.is_some() {
                                let val = model.cell_value(rk, ck).unwrap_or(CellValue::Null);
                                let mut buffer = TextEditorCore::new();
                                buffer.set_text(val.edit_text());
                                state.editing = Some(CellDraft {
                                    cell,
                                    buffer,
                                    error: None,
                                });
                                cx.request_invalidate(Invalidate::Paint);
                            }
                            return Response::action(
                                self.id.clone(),
                                GridAction::Activate { cell },
                            )
                            .with_flow(Flow::Consumed);
                        }
                    }
                    KeyCode::Esc => {
                        state.clear_selection();
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        // Copy selection as TSV or text
                        let mut copy_lines = Vec::new();
                        for i in 0..row_count {
                            let rk = model.row_key(i);
                            if state.selected_rows.contains(&rk)
                                || (state.selected_rows.is_empty() && state.cursor_row == Some(rk))
                            {
                                let mut row_vals = Vec::new();
                                for col in columns {
                                    let v = model
                                        .cell_value(rk, col.key)
                                        .map(|v| v.text())
                                        .unwrap_or_default();
                                    row_vals.push(v);
                                }
                                copy_lines.push(row_vals.join("\t"));
                            }
                        }
                        let text = copy_lines.join("\n");
                        return Response::action(self.id.clone(), GridAction::CopyRequested(text))
                            .with_flow(Flow::Consumed);
                    }
                    _ => {}
                }
            }
        }

        if is_target {
            Response::consumed(self.id.clone())
        } else {
            Response::bubble(self.id.clone())
        }
    }

    /// Measure minimum and preferred sizes.
    pub fn measure(
        &self,
        _cx: &MeasureCx<'_>,
        model: &dyn GridModel,
        constraints: Constraints,
    ) -> Size {
        let total_w: u16 = 8 + model.columns().iter().map(|c| c.width).sum::<u16>();
        let header_h = if self.type_row { 2 } else { 1 };
        let total_h: u16 =
            (header_h + model.row_count() as u16 + if model.has_more() { 1 } else { 0 }).max(3);
        constraints.clamp(Size::new(total_w, total_h))
    }

    /// Read-only draw pass.
    pub fn draw(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        state: &GridState,
        model: &dyn GridModel,
    ) -> Rect {
        if area.is_empty() {
            return area;
        }

        ui.register_hit(self.id.clone(), area);

        let row_count = model.row_count();
        let columns = model.columns();

        // Background
        let theme = ui.theme;
        let bg_color = match ui.current_surface {
            Surface::Elevated => theme.tokens.surface_elevated,
            Surface::Canvas => theme.tokens.canvas,
            _ => theme.tokens.surface,
        };
        ui.fill_rect(area, Style::new().bg(bg_color));

        let header_h = if self.type_row { 2 } else { 1 };
        let header_rect = Rect::new(area.x, area.y, area.width, header_h);
        ui.fill_rect(header_rect, Style::new().bg(theme.tokens.surface_elevated));

        // Draw header row
        let mut x = area.x.saturating_add(8); // Row gutter offset
        // Gutter header label
        ui.set_string(
            area.x + 1,
            area.y,
            "#",
            Style::new().fg(theme.tokens.text_muted),
        );

        for col in columns {
            let col_w = col.width as usize;
            let mut title = col.title.clone();
            if col.primary {
                title.insert_str(0, "▪ ");
            }
            if let Some((sk, dir)) = state.sort {
                if sk == col.key {
                    title.push(' ');
                    title.push_str(dir.arrow());
                }
            }
            let display_title = if title.len() > col_w {
                truncate(&title, col_w).into_owned()
            } else {
                title
            };

            let header_style = Style::new()
                .fg(theme.tokens.text_primary)
                .add_modifier(Modifier::BOLD);
            ui.set_string(x, area.y, &display_title, header_style);

            if self.type_row && !col.type_label.is_empty() {
                let type_style = Style::new().fg(theme.tokens.text_muted);
                let label = truncate(&col.type_label, col_w);
                ui.set_string(x, area.y + 1, &label, type_style);
            }

            x = x.saturating_add(col.width);
        }

        // Draw rows
        let body_y = area.y.saturating_add(header_h);
        let body_h = area.height.saturating_sub(header_h) as usize;

        if row_count == 0 {
            let empty_text = match self.readiness {
                Readiness::Loading => "Loading items...",
                Readiness::Error(_) => "Failed to load items",
                _ => "No items",
            };
            let empty_style = Style::new().fg(theme.tokens.text_muted);
            ui.set_string(area.x + 2, body_y + 1, empty_text, empty_style);
            return area;
        }

        let visible_range = state.scroll.visible_range();
        for (slot, row_idx) in visible_range.enumerate() {
            if slot >= body_h {
                break;
            }
            let y = body_y.saturating_add(slot as u16);
            let row_key = model.row_key(row_idx);
            let is_cursor_row = state.cursor_row == Some(row_key);
            let is_selected_row = state.selected_rows.contains(&row_key);

            // Row background highlight for Table mode
            if self.mode == GridMode::Table {
                if is_selected_row {
                    let sel_style = theme.resolve_style(Role::Selection, ui.current_surface);
                    ui.fill_rect(Rect::new(area.x, y, area.width, 1), sel_style);
                } else if is_cursor_row {
                    let cur_style = Style::new().bg(theme.tokens.surface_elevated);
                    ui.fill_rect(Rect::new(area.x, y, area.width, 1), cur_style);
                }
            }

            // Gutter prefix: `▎` cursor, `✓` selected, row number
            let focus_marker = if is_cursor_row { "▎" } else { " " };
            let sel_marker = if is_selected_row { "✓" } else { " " };
            let num_str = format!("{:>3}", row_idx + 1);

            let marker_style = Style::new().fg(theme.tokens.accent);
            ui.set_string(area.x, y, focus_marker, marker_style);
            ui.set_string(
                area.x + 1,
                y,
                sel_marker,
                Style::new().fg(theme.tokens.text_muted),
            );
            ui.set_string(
                area.x + 3,
                y,
                &num_str,
                Style::new().fg(theme.tokens.text_muted),
            );

            // Render columns
            let mut cx = area.x.saturating_add(8);
            for col in columns {
                let cell_key = CellKey::new(row_key, col.key);
                let is_cursor_cell = is_cursor_row && state.cursor_col == Some(col.key);
                let is_selected_cell = state.selected_cells.contains(&cell_key);
                let col_w = col.width as usize;

                let cell_rect = Rect::new(cx, y, col.width, 1);
                ui.register_hit(self.id.child(row_key).sub(&col.title), cell_rect);

                if let Some(ref draft) = state.editing {
                    if draft.cell == cell_key {
                        // Draw editor buffer
                        let edit_style = if draft.error.is_some() {
                            Style::new().bg(theme.tokens.danger).fg(theme.tokens.canvas)
                        } else {
                            Style::new()
                                .bg(theme.tokens.canvas)
                                .fg(theme.tokens.text_primary)
                        };
                        ui.fill_rect(cell_rect, edit_style);
                        let buf_text = draft.buffer.text();
                        let display = truncate(buf_text, col_w);
                        ui.set_string(cx, y, &display, edit_style);
                        cx = cx.saturating_add(col.width);
                        continue;
                    }
                }

                // Render normal cell value
                let val_opt = model.cell_value(row_key, col.key);
                let cell_str = match &val_opt {
                    Some(v) => v.text(),
                    None => String::new(),
                };

                let mut cell_style = Style::new().fg(theme.tokens.text_primary);
                if val_opt
                    .as_ref()
                    .is_some_and(|v| v.is_null() || v.is_default())
                {
                    cell_style = Style::new()
                        .fg(theme.tokens.text_muted)
                        .add_modifier(Modifier::ITALIC);
                }

                if self.mode == GridMode::Cell {
                    if is_cursor_cell {
                        cell_style = Style::new()
                            .bg(theme.tokens.text_primary)
                            .fg(theme.tokens.canvas);
                        ui.fill_rect(cell_rect, cell_style);
                    } else if is_selected_cell {
                        cell_style = theme.resolve_style(Role::Selection, ui.current_surface);
                        ui.fill_rect(cell_rect, cell_style);
                    }
                }

                // Right alignment for numbers
                let display_str = if col.kind.right_aligned() {
                    let w = width(&cell_str);
                    if w < col_w {
                        format!("{}{}", " ".repeat(col_w - w), cell_str)
                    } else {
                        truncate(&cell_str, col_w).into_owned()
                    }
                } else if width(&cell_str) > col_w {
                    truncate(&cell_str, col_w).into_owned()
                } else {
                    cell_str
                };

                ui.set_string(cx, y, &display_str, cell_style);
                cx = cx.saturating_add(col.width);
            }
        }

        // Draw FetchMore row if model has more and visible
        if model.has_more() {
            let last_slot = row_count.saturating_sub(state.scroll.offset);
            if last_slot < body_h {
                let fetch_y = body_y.saturating_add(last_slot as u16);
                let fetch_style = Style::new()
                    .fg(theme.tokens.accent)
                    .add_modifier(Modifier::DIM);
                ui.set_string(area.x + 8, fetch_y, "↓ Fetch more...", fetch_style);
            }
        }

        area
    }
}
