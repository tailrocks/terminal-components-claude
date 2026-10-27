use core::cmp::Ordering;
use core::ops::Range;
use std::time::Duration;

use ratatui_core::layout::Rect;

use crate::grid::{
    CellAction, CellRef, Column, ColumnKey, Grid, GridAction, GridGutter, GridModel,
    GridSortIndicator, GridState, NavUnit, SortDir, TypeAheadColumn,
};
use crate::id::{Id, ItemKey, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::response::Response;
use crate::response::StateFlags;
use crate::ui::{Cx, FrameRead, Ui};
use termrock_collections::ScrollState;
use termrock_collections::collection::KeySet;
use termrock_runtime::decor::{CellDecor, RowDecor};
use termrock_runtime::empty::RowTotal;
use termrock_theme::{Family, GlyphRole, Role, StylePatch, Variant};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TableSort {
    #[default]
    Text,
    NumericDisplay,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TableAction {
    Grid(GridAction),
    SortChanged(Option<(ColumnKey, SortDir)>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableColumn<'a> {
    pub column: Column<'a>,
    pub sort: TableSort,
    pub flexible: bool,
}

impl<'a> TableColumn<'a> {
    pub const fn new(column: Column<'a>) -> Self {
        Self {
            column,
            sort: TableSort::Text,
            flexible: false,
        }
    }

    #[must_use]
    pub const fn sort(mut self, sort: TableSort) -> Self {
        self.sort = sort;
        self
    }

    #[must_use]
    pub const fn flexible(mut self) -> Self {
        self.flexible = true;
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TableState {
    grid: GridState,
    order: Vec<usize>,
    sort: Option<(ColumnKey, SortDir)>,
    model_keys: Vec<ItemKey>,
    model_revision: Option<u64>,
    model_revision_seen: bool,
    horizontal_scroll: ScrollState,
}

impl TableState {
    pub const fn cursor(&self) -> Option<(ItemKey, ColumnKey)> {
        self.grid.cursor()
    }

    pub const fn selected_rows(&self) -> &KeySet {
        self.grid.selected_rows()
    }

    pub const fn sort(&self) -> Option<(ColumnKey, SortDir)> {
        self.sort
    }

    pub const fn grid_state(&self) -> &GridState {
        &self.grid
    }

    pub fn visible_rows(&self, total: usize) -> Range<usize> {
        self.grid.visible_rows(total)
    }

    fn sync_model<M: GridModel + ?Sized>(&mut self, model: &M) {
        let revision = model.revision();
        let changed = match revision {
            Some(revision) if self.model_revision_seen => self.model_revision != Some(revision),
            _ => {
                self.model_keys.len() != model.row_count()
                    || self
                        .model_keys
                        .iter()
                        .enumerate()
                        .any(|(index, key)| model.row_key(index) != *key)
            }
        };
        if changed {
            self.grid.invalidate_model();
        }
        if changed || !self.model_revision_seen {
            self.model_keys.clear();
            self.model_keys
                .extend((0..model.row_count()).map(|index| model.row_key(index)));
        }
        self.model_revision = revision;
        self.model_revision_seen = true;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Table<'a> {
    id: Id,
    columns: &'a [TableColumn<'a>],
    sort_indicator: GridSortIndicator,
    tone_styles: &'a [(Role, StylePatch)],
    type_ahead: bool,
    type_ahead_column: TypeAheadColumn,
    type_ahead_timeout: Duration,
}

impl<'a> Table<'a> {
    pub const fn new(id: Id, columns: &'a [TableColumn<'a>]) -> Self {
        Self {
            id,
            columns,
            sort_indicator: GridSortIndicator::ActiveOnly,
            tone_styles: &[],
            type_ahead: false,
            type_ahead_column: TypeAheadColumn::Cursor,
            type_ahead_timeout: termrock_core::typeahead::TypeAhead::DEFAULT_TIMEOUT,
        }
    }

    #[must_use]
    pub const fn sort_indicator(mut self, policy: GridSortIndicator) -> Self {
        self.sort_indicator = policy;
        self
    }

    #[must_use]
    pub const fn tone_styles(mut self, styles: &'a [(Role, StylePatch)]) -> Self {
        self.tone_styles = styles;
        self
    }

    #[must_use]
    pub const fn type_ahead(mut self, enabled: bool) -> Self {
        self.type_ahead = enabled;
        self
    }

    #[must_use]
    pub const fn type_ahead_column(mut self, column: Option<usize>) -> Self {
        self.type_ahead_column = match column {
            Some(index) => TypeAheadColumn::Index(index),
            None => TypeAheadColumn::Cursor,
        };
        self
    }

    #[must_use]
    pub const fn type_ahead_column_key(mut self, key: Option<ColumnKey>) -> Self {
        self.type_ahead_column = match key {
            Some(key) => TypeAheadColumn::Key(key),
            None => TypeAheadColumn::Cursor,
        };
        self
    }

    #[must_use]
    pub const fn type_ahead_timeout(mut self, timeout: Duration) -> Self {
        self.type_ahead_timeout = timeout;
        self
    }

    pub fn position_label(&self, state: &TableState, total: usize) -> String {
        let sort = match state.sort {
            Some((key, SortDir::Asc)) => self
                .columns
                .iter()
                .find(|column| column.column.key == key)
                .map_or_else(
                    || "sorted ▴".to_owned(),
                    |column| format!("sorted by {} ▴", column.column.title.to_lowercase()),
                ),
            Some((key, SortDir::Desc)) => self
                .columns
                .iter()
                .find(|column| column.column.key == key)
                .map_or_else(
                    || "sorted ▾".to_owned(),
                    |column| format!("sorted by {} ▾", column.column.title.to_lowercase()),
                ),
            None => "unsorted".to_owned(),
        };
        let range = state.visible_rows(total);
        if !state.grid.rows_overflow() {
            return sort;
        }
        format!("{sort} · {}–{} of {total}", range.start + 1, range.end)
    }

    fn horizontal_scroll_id(&self) -> Id {
        self.id.sub("horizontal-scrollbar")
    }

    fn grid_columns<M: GridModel + ?Sized>(
        &self,
        area: Rect,
        state: &TableState,
        model: &M,
    ) -> (Vec<Column<'a>>, bool, bool, usize, usize) {
        let mut horizontal = false;
        let mut vertical = false;
        for _ in 0..3 {
            let body_rows = usize::from(area.height.saturating_sub(1 + u16::from(horizontal)));
            vertical = model.row_count() > body_rows;
            let viewport = area.width.saturating_sub(5 + u16::from(vertical));
            let mut widths = self
                .columns
                .iter()
                .enumerate()
                .map(|(column_index, column)| {
                    if column.flexible {
                        column
                            .column
                            .min_width
                            .max(crate::text::width(column.column.title))
                    } else {
                        let mut width = column
                            .column
                            .min_width
                            .max(crate::text::width(column.column.title));
                        for row in 0..model.row_count() {
                            if let Some(cell) = model.cell(row, column_index) {
                                width = width.max(crate::text::width(cell.text));
                            }
                        }
                        width.min(column.column.max_width.max(column.column.min_width))
                    }
                })
                .collect::<Vec<_>>();
            let full_width = widths
                .iter()
                .fold(0u16, |sum, width| sum.saturating_add(*width))
                .saturating_add(self.columns.len().saturating_sub(1).saturating_mul(2) as u16);
            let all_fit = full_width <= viewport;
            let offset = if all_fit {
                0
            } else {
                state
                    .grid
                    .col_offset()
                    .min(self.columns.len().saturating_sub(1))
            };
            let mut used = 0u16;
            let mut visible = 0usize;
            for width in widths.iter().skip(offset) {
                let next = used
                    .saturating_add(*width)
                    .saturating_add(u16::from(visible > 0) * 2);
                if next > viewport && visible > 0 {
                    break;
                }
                used = next;
                visible = visible.saturating_add(1);
            }
            visible = visible.max(usize::from(offset < widths.len()));
            let end = offset.saturating_add(visible).min(widths.len());
            if let Some(flex) = (offset..end).find(|&index| self.columns[index].flexible) {
                let fixed = widths
                    .iter()
                    .skip(offset)
                    .take(visible)
                    .fold(0u16, |sum, width| sum.saturating_add(*width));
                let gaps = visible.saturating_sub(1).saturating_mul(2) as u16;
                let available = viewport.saturating_sub(gaps).saturating_sub(fixed);
                if let Some(width) = widths.get_mut(flex) {
                    *width = width.saturating_add(available);
                }
            }
            let hidden = !all_fit && (offset > 0 || end < self.columns.len());
            let result = self
                .columns
                .iter()
                .enumerate()
                .map(|(index, column)| {
                    let width = widths
                        .get(index)
                        .copied()
                        .unwrap_or(column.column.min_width);
                    Column {
                        min_width: width,
                        max_width: width,
                        ..column.column
                    }
                })
                .collect();
            if horizontal == hidden {
                return (result, horizontal, vertical, visible, offset);
            }
            horizontal = hidden;
        }
        (Vec::new(), horizontal, vertical, 0, 0)
    }

    fn grid<'b>(&'b self, columns: &'b [Column<'a>], max_offset: usize) -> Grid<'b>
    where
        'a: 'b,
    {
        let grid = Grid::new(self.id, columns);
        let grid = match self.type_ahead_column {
            TypeAheadColumn::Cursor => grid.type_ahead_column(None),
            TypeAheadColumn::Index(index) => grid.type_ahead_column(Some(index)),
            TypeAheadColumn::Key(key) => grid.type_ahead_column_key(Some(key)),
        };
        grid.type_ahead(self.type_ahead)
            .type_ahead_timeout(self.type_ahead_timeout)
            .nav(NavUnit::Row)
            .gutter(GridGutter::Detailed {
                row_numbers: false,
                min_digits: 0,
            })
            .right_reserve(2)
            .column_gap(2)
            .row_nav_scrolls_columns(true)
            .row_nav_scroll_limit(max_offset)
            .sort_current_column(true)
            .sort_indicator(self.sort_indicator)
            .cell_tone_styles(self.tone_styles)
    }

    fn ordered_rows<M: GridModel + ?Sized>(
        &self,
        model: &M,
        sort: Option<(ColumnKey, SortDir)>,
    ) -> Vec<usize> {
        let mut order = (0..model.row_count()).collect::<Vec<_>>();
        let Some((key, direction)) = sort else {
            return order;
        };
        let Some(column_index) = self
            .columns
            .iter()
            .position(|column| column.column.key == key && column.column.sortable)
        else {
            return order;
        };
        let sort_kind = self.columns[column_index].sort;
        order.sort_by(|&a, &b| {
            let left = model.cell(a, column_index).map_or("", |cell| cell.text);
            let right = model.cell(b, column_index).map_or("", |cell| cell.text);
            let compared = match sort_kind {
                TableSort::Text => left.to_lowercase().cmp(&right.to_lowercase()),
                TableSort::NumericDisplay => parse_display_number(left)
                    .partial_cmp(&parse_display_number(right))
                    .unwrap_or(Ordering::Equal),
            }
            .then_with(|| a.cmp(&b));
            match direction {
                SortDir::Asc => compared,
                SortDir::Desc => compared.reverse(),
            }
        });
        order
    }

    pub fn update<M: GridModel + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        state: &mut TableState,
        model: &M,
    ) -> Response<TableAction> {
        state.sync_model(model);
        let mut area = cx.area(self.id).unwrap_or_default();
        let bar_id = self.horizontal_scroll_id();
        if cx
            .area_of_part(bar_id, PartRef::of(Part::TRACK))
            .is_some_and(|track| track.height == 1)
        {
            area.height = area.height.saturating_add(1);
        }
        let (columns, horizontal, vertical, visible, offset) =
            self.grid_columns(area, state, model);
        let max_offset = self.columns.len().saturating_sub(visible.max(1));
        state.grid.set_col_offset(offset.min(max_offset));
        state.horizontal_scroll.set_content(self.columns.len());
        state.horizontal_scroll.set_viewport(visible.max(1));
        state
            .horizontal_scroll
            .scroll_to(state.grid.col_offset().min(max_offset));
        let mut horizontal_input = false;
        let mut horizontal_changed = false;
        if horizontal {
            let intents = cx.intents(bar_id).collect::<Vec<_>>();
            for intent in intents {
                let Intent::Pointer {
                    phase, part, pos, ..
                } = intent
                else {
                    continue;
                };
                if !matches!(part.part, Part::TRACK | Part::THUMB) {
                    continue;
                }
                horizontal_input = true;
                let before = state.horizontal_scroll.offset();
                let track_pos = usize::from(pos.x.saturating_sub(area.x.saturating_add(3)));
                match phase {
                    Phase::Press if part.part == Part::THUMB => {
                        if cx.capture(bar_id, PartRef::of(Part::THUMB)) {
                            state.horizontal_scroll.press_track(
                                track_pos,
                                usize::from(area.width.saturating_sub(5 + u16::from(vertical))),
                            );
                        }
                    }
                    Phase::Press => {
                        let _ = state.horizontal_scroll.press_track(
                            track_pos,
                            usize::from(area.width.saturating_sub(5 + u16::from(vertical))),
                        );
                    }
                    Phase::Drag => {
                        let _ = state.horizontal_scroll.drag_track(
                            track_pos,
                            usize::from(area.width.saturating_sub(5 + u16::from(vertical))),
                        );
                    }
                    Phase::Release | Phase::DragEnd => {
                        state.horizontal_scroll.release_track();
                        if cx.capture_owner() == Some(bar_id) {
                            cx.release_capture();
                        }
                    }
                    _ => {}
                }
                horizontal_changed |= before != state.horizontal_scroll.offset();
            }
            state.grid.set_col_offset(state.horizontal_scroll.offset());
        }
        let grid = self.grid(&columns, max_offset);
        state.order = self.ordered_rows(model, state.sort);
        let mut response = {
            let ordered = OrderedModel {
                source: model,
                order: &state.order,
            };
            grid.update(cx, &mut state.grid, &ordered)
        };
        if horizontal_changed {
            response = response.repaint();
        } else if horizontal_input && !response.is_consumed() {
            response = Response::consumed();
        }
        state.grid.set_col_offset(
            state
                .grid
                .col_offset()
                .min(state.horizontal_scroll.max_offset()),
        );
        state.horizontal_scroll.scroll_to(state.grid.col_offset());

        if let Some(GridAction::Sort(key, _)) = response.action_ref() {
            let key = *key;
            state.sort = match state.sort {
                Some((current, SortDir::Asc)) if current == key => Some((key, SortDir::Desc)),
                Some((current, SortDir::Desc)) if current == key => None,
                _ => Some((key, SortDir::Asc)),
            };
            state.grid.set_sort(state.sort);
            state.order = self.ordered_rows(model, state.sort);
            if let Some((row, column)) = state.grid.cursor() {
                let ordered = OrderedModel {
                    source: model,
                    order: &state.order,
                };
                let _ = grid.move_cursor_to(&mut state.grid, &ordered, row, column);
            }
            let cleared = state.sort.is_none();
            return response.map_action(|action| match action {
                GridAction::Sort(_, _) if cleared => TableAction::SortChanged(None),
                GridAction::Sort(_, _) => TableAction::SortChanged(state.sort),
                other => TableAction::Grid(other),
            });
        }
        response.map_action(TableAction::Grid)
    }

    pub fn draw<M: GridModel + ?Sized>(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        state: &TableState,
        model: &M,
    ) -> Rect {
        let (columns, horizontal, vertical, visible, offset) =
            self.grid_columns(area, state, model);
        let grid = self.grid(&columns, self.columns.len().saturating_sub(visible.max(1)));
        let mut grid_state = state.grid.clone();
        grid_state.set_col_offset(offset);
        let order = self.ordered_rows(model, state.sort);
        let ordered = OrderedModel {
            source: model,
            order: &order,
        };
        let grid_area = if horizontal {
            Rect {
                height: area.height.saturating_sub(1),
                ..area
            }
        } else {
            area
        };
        let drawn = grid.draw(ui, grid_area, &grid_state, &ordered);
        if horizontal {
            let mut scroll = state.horizontal_scroll;
            scroll.set_content(self.columns.len());
            scroll.set_viewport(visible.max(1));
            scroll.scroll_to(offset);
            self.draw_horizontal_bar(ui, area, &scroll, vertical);
        }
        drawn
    }

    fn draw_horizontal_bar(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        scroll: &ScrollState,
        vertical: bool,
    ) {
        if area.is_empty() || area.height == 0 {
            return;
        }
        let track_area = Rect {
            x: area.x.saturating_add(3),
            y: area.bottom().saturating_sub(1),
            width: area.width.saturating_sub(5 + u16::from(vertical)),
            height: 1,
        };
        let bar_id = self.horizontal_scroll_id();
        let track_flags = if ui.pressed_part(bar_id) == Some(PartRef::of(Part::TRACK)) {
            StateFlags::PRESSED
        } else if ui.hovered_part(bar_id) == Some(PartRef::of(Part::TRACK)) {
            StateFlags::HOVERED
        } else {
            StateFlags::empty()
        };
        let thumb_flags = if ui.pressed_part(bar_id) == Some(PartRef::of(Part::THUMB)) {
            StateFlags::PRESSED
        } else if ui.hovered_part(bar_id) == Some(PartRef::of(Part::THUMB)) {
            StateFlags::HOVERED
        } else {
            StateFlags::empty()
        };
        let track = ui.style(
            Family::SCROLLBAR,
            Variant::DEFAULT,
            Part::TRACK,
            track_flags,
        );
        let thumb = ui.style(
            Family::SCROLLBAR,
            Variant::DEFAULT,
            Part::THUMB,
            thumb_flags,
        );
        let track_glyph = ui.glyph_str(GlyphRole::ScrollTrack);
        paint_repeated_glyph(ui, track_area, track_glyph, track.style);
        let (left, thumb_width) = scroll.thumb(usize::from(track_area.width));
        let thumb_area = Rect {
            x: track_area
                .x
                .saturating_add(left.min(usize::from(u16::MAX)) as u16),
            width: thumb_width.min(usize::from(u16::MAX)) as u16,
            ..track_area
        };
        let thumb_glyph = ui.glyph_str(GlyphRole::ScrollThumb);
        paint_repeated_glyph(ui, thumb_area, thumb_glyph, thumb.style);
        ui.register_part(bar_id, PartRef::of(Part::TRACK), track_area);
        ui.register_part(bar_id, PartRef::of(Part::THUMB), thumb_area);
    }
}

fn paint_repeated_glyph(
    ui: &mut Ui<'_>,
    area: Rect,
    glyph: &str,
    style: termrock_theme::PaintStyle,
) {
    let glyph_width = crate::text::width(glyph);
    if area.is_empty() || glyph_width == 0 {
        return;
    }
    let repetitions = usize::from(area.width).div_ceil(usize::from(glyph_width));
    let pattern = glyph.repeat(repetitions);
    ui.paint_str(area, &pattern, style);
}

struct OrderedModel<'a, M: ?Sized> {
    source: &'a M,
    order: &'a [usize],
}

impl<M: GridModel + ?Sized> GridModel for OrderedModel<'_, M> {
    fn row_count(&self) -> usize {
        self.order.len()
    }

    fn row_key(&self, row: usize) -> ItemKey {
        self.source.row_key(self.order[row])
    }

    fn row_enabled(&self, row: usize) -> bool {
        self.source.row_enabled(self.order[row])
    }

    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        self.source.cell(self.order[row], col)
    }

    fn row_decor(&self, row: usize) -> RowDecor<'_> {
        self.source.row_decor(self.order[row])
    }

    fn cell_decor(&self, row: usize, col: usize) -> CellDecor<'_> {
        self.source.cell_decor(self.order[row], col)
    }

    fn total(&self) -> RowTotal {
        self.source.total()
    }

    fn has_more(&self) -> bool {
        self.source.has_more()
    }

    fn read_only_reason(&self) -> Option<&str> {
        self.source.read_only_reason()
    }

    fn actions(&self, row: usize, col: usize) -> &[CellAction] {
        self.source.actions(self.order[row], col)
    }
}

fn parse_display_number(value: &str) -> Option<f64> {
    let cleaned = value
        .chars()
        .filter(|character| character.is_ascii_digit() || *character == '.' || *character == '-')
        .collect::<String>();
    cleaned
        .parse::<f64>()
        .ok()
        .filter(|number| !number.is_nan())
}
