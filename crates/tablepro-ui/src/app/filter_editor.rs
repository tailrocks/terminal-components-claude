use std::sync::Weak;

use tablepro_demo::db::{Catalog, ColType, Table};
use tablepro_domain::filter_editor::{Filter, FilterOp};
use tablepro_domain::tabs::{GridView, Tab, TabKey, TableTab};
use tablepro_sql::sql;
use termrock::{
    Button, ColumnKey, FgStep, Field, GridModel, Id, Intent, ItemKey, KeyCode, Modifier, Panel,
    PanelKind, Part, Reconcile, Rect, Role, RowUi, Select, SelectAction, SelectField, SelectState,
    StylePatch, Surface, TextAction, TextInput, TextInputState, Ui, Variant, truncate,
};

use crate::workbench::Workbench;

use super::{FILTER_LAYER, FILTER_VALUE, Surface as AppSurface};

pub(super) const FILTER_COLUMN: Id = Id::root("tablepro.filter.column");
pub(super) const FILTER_OPERATOR: Id = Id::root("tablepro.filter.operator");
pub(super) const FILTER_VALUE2: Id = Id::root("tablepro.filter.value2");
pub(super) const FILTER_CANCEL: Id = Id::root("tablepro.filter.cancel");
pub(super) const FILTER_APPLY: Id = Id::root("tablepro.filter.apply");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FilterOutcome {
    Keep,
    Apply,
    Cancel,
}

#[derive(Debug, Clone)]
pub(super) struct FilterEditor {
    owner: Weak<()>,
    tab_key: TabKey,
    table_identity: Option<Table>,
    original_filters: Vec<Filter>,
    original_index: Option<usize>,
    original_filter: Option<Filter>,
    source_column: String,
    columns: Vec<(String, ColType)>,
    column_index: usize,
    operators: Vec<FilterOp>,
    operator: FilterOp,
    column_state: SelectState,
    operator_state: SelectState,
    value: String,
    value2: String,
    enabled: bool,
    value_state: TextInputState,
    value2_state: TextInputState,
    pub(super) modal: bool,
    pub(super) error: Option<String>,
    return_surface: AppSurface,
}

impl FilterEditor {
    pub(super) fn capture(
        workbench: &Workbench,
        column_index: usize,
        modal: bool,
    ) -> Result<Self, String> {
        let tab_key = workbench
            .active_key()
            .ok_or_else(|| "No active table is available".to_owned())?;
        let Some(Tab::Table(table)) = workbench.tab(tab_key) else {
            return Err("The active tab is not a table".to_owned());
        };
        if table.is_structure() {
            return Err("Filters are available in table data view".to_owned());
        }
        if table.result.pending_total() > 0 {
            return Err("Apply or discard pending edits before filtering".to_owned());
        }
        let columns = table
            .table
            .columns
            .iter()
            .map(|column| (column.name.clone(), column.ty))
            .collect::<Vec<_>>();
        if columns.is_empty() {
            return Err("The table has no columns to filter".to_owned());
        }
        let column_index = column_index.min(columns.len().saturating_sub(1));
        let source_column = columns
            .get(column_index)
            .map_or_else(String::new, |(name, _)| name.clone());
        let original_index = table
            .filters
            .iter()
            .position(|filter| filter.column.eq_ignore_ascii_case(&source_column));
        let original_filter = original_index.and_then(|index| table.filters.get(index).cloned());
        let mut editor = Self::from_parts(columns, column_index, original_filter, modal);
        editor.owner = workbench.owner_token();
        editor.tab_key = tab_key;
        editor.table_identity = Some(table.table.clone());
        editor.original_filters.clone_from(&table.filters);
        editor.original_index = original_index;
        editor.source_column = source_column;
        Ok(editor)
    }

    fn from_parts(
        columns: Vec<(String, ColType)>,
        column_index: usize,
        original_filter: Option<Filter>,
        modal: bool,
    ) -> Self {
        let column_index = column_index.min(columns.len().saturating_sub(1));
        let filter = original_filter.clone();
        let operator = filter.as_ref().map_or(FilterOp::Eq, |filter| filter.op);
        let operators = ordered_operators(
            columns
                .get(column_index)
                .map_or(ColType::Text, |(_, ty)| *ty),
        );
        let mut column_state = SelectState::default();
        column_state.set_value(columns.get(column_index).map(column_key));
        let mut operator_state = SelectState::default();
        operator_state.set_value(Some(operator_key(&operator)));
        Self {
            owner: Weak::new(),
            tab_key: TabKey::new(0),
            table_identity: None,
            original_filters: Vec::new(),
            original_index: None,
            original_filter: filter.clone(),
            source_column: columns
                .get(column_index)
                .map_or_else(String::new, |(name, _)| name.clone()),
            operators,
            value: filter
                .as_ref()
                .map_or_else(String::new, |filter| filter.value.clone()),
            value2: filter
                .as_ref()
                .map_or_else(String::new, |filter| filter.value2.clone()),
            enabled: filter.as_ref().is_none_or(|filter| filter.enabled),
            value_state: TextInputState::default(),
            value2_state: TextInputState::default(),
            columns,
            column_index,
            operator,
            column_state,
            operator_state,
            modal,
            error: None,
            return_surface: AppSurface::WorkbenchDefault,
        }
    }

    pub(super) fn current_column(&self) -> Option<&str> {
        self.columns
            .get(self.column_index)
            .map(|(column, _)| column.as_str())
    }

    pub(super) const fn tab_key(&self) -> TabKey {
        self.tab_key
    }

    pub(super) fn initial_focus(&self) -> Id {
        if self.operator.needs_value() {
            FILTER_VALUE
        } else {
            FILTER_APPLY
        }
    }

    pub(super) fn select_modal(&mut self) {
        self.modal = true;
    }

    pub(super) fn set_return_surface(&mut self, surface: AppSurface) {
        self.return_surface = surface;
    }

    pub(super) const fn return_surface(&self) -> AppSurface {
        self.return_surface
    }

    fn choose_column(&mut self, key: ItemKey) {
        if let Some(index) = self
            .columns
            .iter()
            .position(|column| column_key(column) == key)
            && let Some((_, ty)) = self.columns.get(index)
        {
            self.column_index = index;
            self.operators = ordered_operators(*ty);
            self.operator_state
                .set_value(Some(operator_key(&self.operator)));
            self.error = None;
        }
    }

    fn choose_operator(&mut self, key: ItemKey) {
        if let Some(operator) = self
            .operators
            .iter()
            .copied()
            .find(|operator| operator_key(operator) == key)
        {
            self.operator = operator;
            self.error = None;
        }
    }

    pub(super) fn to_filter(&self) -> Filter {
        Filter {
            column: self.current_column().unwrap_or_default().to_owned(),
            op: self.operator,
            value: self.value.clone(),
            value2: self.value2.clone(),
            enabled: self.enabled,
        }
    }

    fn preview_filter(&self) -> Filter {
        let mut filter = self.to_filter();
        if let Some(draft) = self.value_state.draft_text() {
            filter.value = draft.to_owned();
        }
        if let Some(draft) = self.value2_state.draft_text() {
            filter.value2 = draft.to_owned();
        }
        filter
    }

    pub(super) fn validate<'a>(&self, workbench: &'a Workbench) -> Result<&'a TableTab, String> {
        if !workbench.matches_owner(&self.owner) {
            return Err("The workbench changed while the filter editor was open".to_owned());
        }
        if workbench.active_key() != Some(self.tab_key) {
            return Err("The active tab changed while the filter editor was open".to_owned());
        }
        let Some(Tab::Table(table)) = workbench.tab(self.tab_key) else {
            return Err("The source table tab is no longer available".to_owned());
        };
        if self.table_identity.as_ref() != Some(&table.table) {
            return Err("The source table or schema changed while editing".to_owned());
        }
        if table.is_structure() {
            return Err("The table view changed while editing".to_owned());
        }
        if table.filters != self.original_filters {
            return Err("The table filters changed while editing; reopen the editor".to_owned());
        }
        if table.result.pending_total() > 0 {
            return Err("Apply or discard pending edits before filtering".to_owned());
        }
        Ok(table)
    }

    pub(super) fn stage(
        &self,
        workbench: &Workbench,
        catalog: &Catalog,
    ) -> Result<TableTab, String> {
        let source = self.validate(workbench)?;
        let mut candidate = source.clone();
        let filter = self.to_filter();
        let changed_column = !self.source_column.eq_ignore_ascii_case(&filter.column);

        if changed_column {
            if let Some(original) = &self.original_filter {
                let Some(index) = self.original_index else {
                    return Err("The original filter is no longer available".to_owned());
                };
                if candidate.filters.get(index) != Some(original) {
                    return Err("The original filter changed while editing".to_owned());
                }
                candidate.filters.remove(index);
            }
            if candidate
                .filters
                .iter()
                .any(|existing| existing.column.eq_ignore_ascii_case(&filter.column))
            {
                return Err(format!(
                    "A filter for '{}' already exists; choose another column",
                    filter.column
                ));
            }
            candidate.set_filter(filter.clone());
        } else {
            candidate.set_filter(filter.clone());
        }

        if let Some(unsupported) = candidate.filters.iter().find(|filter| {
            filter.enabled
                && matches!(
                    filter.op,
                    FilterOp::NotContains | FilterOp::NotIn | FilterOp::Between | FilterOp::Regex
                )
        }) {
            return Err(format!(
                "This filter operator is not supported by the query engine yet: {}",
                unsupported.op.label()
            ));
        }

        let query = super::table_query(&candidate.table, &candidate.filters);
        let statement = sql::parse(&query)
            .map_err(|error| format!("The filter query could not be parsed: {error:?}"))?;
        let sql::Statement::Select(select) = statement else {
            return Err("The filter did not produce a table query".to_owned());
        };
        let result = sql::run_select(catalog, &select)
            .map_err(|error| format!("The filter query failed: {error:?}"))?;

        let old_cursor_column = source
            .result
            .state
            .cursor()
            .and_then(|(_, column)| usize::from(column.raw()).checked_sub(1))
            .and_then(|index| source.result.columns.get(index))
            .map(|(name, _)| name.clone());
        let model = tablepro_domain::domain::ResultGrid::from_filter_refresh(
            &result,
            &source.result.model,
            &source.result.columns,
            &candidate.table,
        )
        .ok_or_else(|| "The table result could not preserve row identity".to_owned())?;
        let mut state = source.result.state.clone();
        state.clear_sampled_widths();
        state.invalidate();
        state.reconcile(model.row_count(), |row| GridModel::row_key(&model, row));

        if let (Some((row, _)), Some(column_name)) = (state.cursor(), old_cursor_column)
            && let Some(column_index) = result
                .columns
                .iter()
                .position(|(name, _)| name.eq_ignore_ascii_case(&column_name))
            && let Ok(column_number) = u16::try_from(column_index.saturating_add(1))
        {
            let id = self.tab_key.control("data");
            let (specs, count) =
                super::TableProApp::column_specs(&result.columns, model.is_editable());
            let visible_columns = specs.get(..count).unwrap_or(&[]);

            let _ = termrock::Grid::new(id, visible_columns).move_cursor_to(
                &mut state,
                &model,
                row,
                ColumnKey::num(1),
            );
            let _ = super::result_grid::result_grid(id, visible_columns).move_cursor_to(
                &mut state,
                &model,
                row,
                ColumnKey::num(column_number),
            );
        }
        candidate.result = GridView {
            columns: result.columns.clone(),
            model,
            state,
        };
        Ok(candidate)
    }

    pub(super) fn update(
        &mut self,
        cx: &mut termrock::Cx<'_>,
    ) -> (termrock::Response<()>, FilterOutcome) {
        if self.modal && !cx.is_open(FILTER_LAYER) {
            return (termrock::Response::ignored(), FilterOutcome::Cancel);
        }

        if !self.modal {
            let value_blurred = cx
                .intents(FILTER_VALUE)
                .any(|intent| matches!(intent, Intent::FocusOut { .. }));
            let response =
                TextInput::new(FILTER_VALUE).update(cx, &mut self.value_state, &mut self.value);
            match response.action_ref() {
                Some(TextAction::MoveNext) => cx.focus_next(),
                Some(TextAction::MovePrev) => cx.focus_prev(),
                _ => {}
            }
            let outcome = match response.action_ref() {
                Some(TextAction::Committed) if !value_blurred => FilterOutcome::Apply,
                Some(TextAction::Cancelled) => FilterOutcome::Cancel,
                _ => FilterOutcome::Keep,
            };
            for id in [super::FILTER_STEP, FILTER_APPLY] {
                for intent in cx.intents(id) {
                    if let Intent::Key(key) = intent {
                        match key.code {
                            KeyCode::Tab => cx.focus_next(),
                            KeyCode::BackTab => cx.focus_prev(),
                            KeyCode::Esc => return (response.erase(), FilterOutcome::Cancel),
                            KeyCode::Enter if id == FILTER_APPLY => {
                                return (response.erase(), FilterOutcome::Apply);
                            }
                            _ => {}
                        }
                    }
                }
            }
            return (response.erase(), outcome);
        }

        let mut response = termrock::Response::ignored();
        let column = Select::new(FILTER_COLUMN)
            .key(column_key)
            .row(column_row)
            .update(cx, &mut self.column_state, &self.columns);
        if let Some(SelectAction::Chose(key)) = column.action_ref() {
            self.choose_column(*key);
        }
        response |= column.erase();

        let operator = Select::new(FILTER_OPERATOR)
            .key(operator_key)
            .row(operator_row)
            .update(cx, &mut self.operator_state, &self.operators);
        if let Some(SelectAction::Chose(key)) = operator.action_ref() {
            self.choose_operator(*key);
        }
        response |= operator.erase();

        let value_blurred = cx
            .intents(FILTER_VALUE)
            .any(|intent| matches!(intent, Intent::FocusOut { .. }));
        let value = TextInput::new(FILTER_VALUE).update(cx, &mut self.value_state, &mut self.value);
        let value_action = value.action_ref().copied();
        response |= value.erase();
        match value_action {
            Some(TextAction::MoveNext) => cx.focus_next(),
            Some(TextAction::MovePrev) => cx.focus_prev(),
            _ => {}
        }

        let value2_blurred = cx
            .intents(FILTER_VALUE2)
            .any(|intent| matches!(intent, Intent::FocusOut { .. }));
        let value2 =
            TextInput::new(FILTER_VALUE2).update(cx, &mut self.value2_state, &mut self.value2);
        let value2_action = value2.action_ref().copied();
        response |= value2.erase();
        match value2_action {
            Some(TextAction::MoveNext) => cx.focus_next(),
            Some(TextAction::MovePrev) => cx.focus_prev(),
            _ => {}
        }

        let cancel = Button::new(FILTER_CANCEL, "Cancel")
            .variant(Variant::SUBTLE)
            .update(cx);
        let cancelled = cancel.activated();
        response |= cancel.erase();

        let apply = Button::new(FILTER_APPLY, "Apply")
            .variant(Variant::PRIMARY)
            .update(cx);
        let applied = apply.activated();
        response |= apply.erase();

        let mut escape_on_popup = false;
        for id in [
            FILTER_COLUMN,
            FILTER_OPERATOR,
            FILTER_VALUE,
            FILTER_VALUE2,
            FILTER_CANCEL,
            FILTER_APPLY,
        ] {
            for intent in cx.intents(id) {
                if let Intent::Key(key) = intent {
                    match key.code {
                        KeyCode::Tab => cx.focus_next(),
                        KeyCode::BackTab => cx.focus_prev(),
                        KeyCode::Esc
                            if self.column_state.is_open() || self.operator_state.is_open() =>
                        {
                            escape_on_popup = true;
                        }
                        KeyCode::Esc => return (response, FilterOutcome::Cancel),
                        _ => {}
                    }
                }
            }
        }

        if cancelled
            || (!escape_on_popup
                && (matches!(value_action, Some(TextAction::Cancelled))
                    || matches!(value2_action, Some(TextAction::Cancelled))))
        {
            return (response, FilterOutcome::Cancel);
        }
        if applied
            || (matches!(value2_action, Some(TextAction::Committed)) && !value2_blurred)
            || (matches!(value_action, Some(TextAction::Committed))
                && !value_blurred
                && self.operator != FilterOp::Between)
        {
            return (response, FilterOutcome::Apply);
        }
        if matches!(value_action, Some(TextAction::Committed))
            && !value_blurred
            && self.operator == FilterOp::Between
        {
            cx.focus(FILTER_VALUE2);
        }
        (response, FilterOutcome::Keep)
    }

    pub(super) fn draw(&self, ui: &mut Ui<'_>) {
        if !self.modal {
            TextInput::new(FILTER_VALUE).draw(ui, Rect::new(0, 0, 12, 1), &self.value_state);
            ui.register_focus_only(super::FILTER_STEP, termrock::Focusability::Focusable);
            ui.register_focus_only(FILTER_APPLY, termrock::Focusability::Focusable);
            return;
        }

        ui.layer(FILTER_LAYER, |ui, area| {
            ui.with_surface(Surface::Elevated, |ui| {
                Panel::new(FILTER_LAYER)
                    .kind(PanelKind::Framed)
                    .focused(true)
                    .slot(Part::GUTTER, &|_, _| {})
                    .draw(ui, area, |ui, _| {
                        let inner_x = area.x.saturating_add(2);
                        let inner_width = area.width.saturating_sub(4);
                        let gap = 2;
                        let left_width = inner_width.saturating_sub(gap) / 2;
                        let right_width =
                            inner_width.saturating_sub(gap).saturating_sub(left_width);
                        let right_x = inner_x.saturating_add(left_width).saturating_add(gap);
                        let title = if self.original_filter.is_some() {
                            "Edit filter"
                        } else {
                            "Add filter"
                        };
                        let title_style = ui.surface_style().patch(
                            ui.paint_patch(
                                &StylePatch::new()
                                    .set_fg(Role::Fg(FgStep::Primary))
                                    .add(Modifier::BOLD),
                            ),
                        );
                        ui.paint_str(
                            Rect::new(inner_x, area.y.saturating_add(1), inner_width, 1),
                            title,
                            title_style,
                        );

                        Field::new(
                            "Column",
                            SelectField::new(
                                Select::new(FILTER_COLUMN).key(column_key).row(column_row),
                                &self.columns,
                            ),
                        )
                        .plain(true)
                        .optional_suffix(false)
                        .draw(
                            ui,
                            Rect::new(inner_x, area.y.saturating_add(2), left_width, 3),
                            &self.column_state,
                        );
                        Field::new(
                            "Operator",
                            SelectField::new(
                                Select::new(FILTER_OPERATOR)
                                    .key(operator_key)
                                    .row(operator_row),
                                &self.operators,
                            ),
                        )
                        .plain(true)
                        .optional_suffix(false)
                        .draw(
                            ui,
                            Rect::new(right_x, area.y.saturating_add(2), right_width, 3),
                            &self.operator_state,
                        );

                        if self.operator.needs_value() {
                            let between = self.operator == FilterOp::Between;
                            let value_width = if between { left_width } else { inner_width };
                            Field::new(
                                "Value",
                                TextInput::new(FILTER_VALUE)
                                    .value(&self.value)
                                    .placeholder("value"),
                            )
                            .plain(true)
                            .draw(
                                ui,
                                Rect::new(inner_x, area.y.saturating_add(5), value_width, 3),
                                &self.value_state,
                            );
                            if between {
                                Field::new(
                                    "and",
                                    TextInput::new(FILTER_VALUE2)
                                        .value(&self.value2)
                                        .placeholder("value"),
                                )
                                .plain(true)
                                .draw(
                                    ui,
                                    Rect::new(right_x, area.y.saturating_add(5), right_width, 3),
                                    &self.value2_state,
                                );
                            }
                        } else {
                            ui.paint_str(
                                Rect::new(inner_x, area.y.saturating_add(6), inner_width, 1),
                                "This operator does not use a value",
                                ui.surface_style(),
                            );
                        }

                        let sql = format!("WHERE {}", self.preview_filter().to_sql());
                        let preview = truncate(&sql, inner_width);
                        ui.paint_str(
                            Rect::new(inner_x, area.y.saturating_add(9), inner_width, 1),
                            &preview,
                            ui.surface_style(),
                        );
                        if let Some(error) = &self.error {
                            let message = truncate(error, inner_width);
                            ui.paint_str(
                                Rect::new(inner_x, area.y.saturating_add(10), inner_width, 1),
                                &message,
                                ui.surface_style(),
                            );
                        }

                        let button_y = area.bottom().saturating_sub(2);
                        let apply_width: u16 = 9;
                        let cancel_width: u16 = 10;
                        let apply_x = area.right().saturating_sub(apply_width.saturating_add(3));
                        let cancel_x = apply_x.saturating_sub(cancel_width.saturating_add(1));
                        Button::new(FILTER_CANCEL, "Cancel")
                            .variant(Variant::SUBTLE)
                            .draw(ui, Rect::new(cancel_x, button_y, cancel_width, 1));
                        Button::new(FILTER_APPLY, "Apply")
                            .variant(Variant::PRIMARY)
                            .draw(ui, Rect::new(apply_x, button_y, apply_width, 1));
                    });
            });
        });
    }
}

fn column_key(column: &(String, ColType)) -> ItemKey {
    ItemKey::text(&column.0)
}

fn column_row(column: &(String, ColType), row: &mut RowUi<'_>) {
    row.label(&column.0);
}

fn operator_key(operator: &FilterOp) -> ItemKey {
    ItemKey::num(*operator as u64)
}

fn ordered_operators(ty: ColType) -> Vec<FilterOp> {
    let preferred: &[FilterOp] = match ty {
        ColType::Int | ColType::Numeric | ColType::Timestamp | ColType::Date => &[
            FilterOp::Eq,
            FilterOp::Ne,
            FilterOp::Gt,
            FilterOp::Ge,
            FilterOp::Lt,
            FilterOp::Le,
            FilterOp::Between,
            FilterOp::IsNull,
            FilterOp::IsNotNull,
        ],
        ColType::Bool => &[FilterOp::Eq, FilterOp::IsNull, FilterOp::IsNotNull],
        ColType::Enum => &[
            FilterOp::Eq,
            FilterOp::Ne,
            FilterOp::In,
            FilterOp::NotIn,
            FilterOp::IsNull,
            FilterOp::IsNotNull,
        ],
        ColType::Json => &[FilterOp::Contains, FilterOp::IsNull, FilterOp::IsNotNull],
        ColType::Uuid | ColType::Text => &[
            FilterOp::Eq,
            FilterOp::Ne,
            FilterOp::Contains,
            FilterOp::StartsWith,
            FilterOp::EndsWith,
            FilterOp::IsNull,
            FilterOp::IsNotNull,
            FilterOp::IsEmpty,
        ],
    };
    let mut operators = preferred.to_vec();
    for operator in FilterOp::ALL {
        if !operators.contains(&operator) {
            operators.push(operator);
        }
    }
    operators
}

fn operator_row(operator: &FilterOp, row: &mut RowUi<'_>) {
    row.label(operator.label());
}
