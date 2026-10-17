//! `TablePro`'s typed filter editor and local filtering policy.

use tablepro_domain::{ColType, Value};

/// All operators exposed by the filter editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterOp {
    /// Equal.
    Eq,
    /// Not equal.
    Ne,
    /// Substring.
    Contains,
    /// Negated substring.
    NotContains,
    /// Prefix.
    StartsWith,
    /// Suffix.
    EndsWith,
    /// Greater than.
    Gt,
    /// Greater than or equal.
    Ge,
    /// Less than.
    Lt,
    /// Less than or equal.
    Le,
    /// NULL.
    IsNull,
    /// Not NULL.
    IsNotNull,
    /// Empty text.
    IsEmpty,
    /// Non-empty text.
    IsNotEmpty,
    /// Membership.
    In,
    /// Negated membership.
    NotIn,
    /// Inclusive range.
    Between,
    /// Wildcard pattern.
    Regex,
}

impl FilterOp {
    /// Stable menu order.
    pub const ALL: [Self; 18] = [
        Self::Eq,
        Self::Ne,
        Self::Contains,
        Self::NotContains,
        Self::StartsWith,
        Self::EndsWith,
        Self::Gt,
        Self::Ge,
        Self::Lt,
        Self::Le,
        Self::IsNull,
        Self::IsNotNull,
        Self::IsEmpty,
        Self::IsNotEmpty,
        Self::In,
        Self::NotIn,
        Self::Between,
        Self::Regex,
    ];

    /// Display label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Ne => "!=",
            Self::Contains => "contains",
            Self::NotContains => "not contains",
            Self::StartsWith => "starts with",
            Self::EndsWith => "ends with",
            Self::Gt => ">",
            Self::Ge => ">=",
            Self::Lt => "<",
            Self::Le => "<=",
            Self::IsNull => "is NULL",
            Self::IsNotNull => "is not NULL",
            Self::IsEmpty => "is empty",
            Self::IsNotEmpty => "is not empty",
            Self::In => "in list",
            Self::NotIn => "not in list",
            Self::Between => "between",
            Self::Regex => "matches",
        }
    }

    /// Whether the operator needs a value.
    pub const fn needs_value(self) -> bool {
        !matches!(
            self,
            Self::IsNull | Self::IsNotNull | Self::IsEmpty | Self::IsNotEmpty
        )
    }

    /// Put type-appropriate options first, retaining the complete menu.
    pub fn ordered_for(ty: ColType) -> Vec<Self> {
        let preferred: &[Self] = match ty {
            ColType::Int | ColType::Numeric | ColType::Timestamp | ColType::Date => &[
                Self::Eq,
                Self::Ne,
                Self::Gt,
                Self::Ge,
                Self::Lt,
                Self::Le,
                Self::Between,
                Self::IsNull,
                Self::IsNotNull,
            ],
            ColType::Bool => &[Self::Eq, Self::IsNull, Self::IsNotNull],
            ColType::Enum => &[
                Self::Eq,
                Self::Ne,
                Self::In,
                Self::NotIn,
                Self::IsNull,
                Self::IsNotNull,
            ],
            ColType::Json => &[Self::Contains, Self::IsNull, Self::IsNotNull],
            _ => &[
                Self::Eq,
                Self::Ne,
                Self::Contains,
                Self::StartsWith,
                Self::EndsWith,
                Self::IsNull,
                Self::IsNotNull,
                Self::IsEmpty,
            ],
        };
        let mut out = preferred.to_vec();
        for op in Self::ALL {
            if !out.contains(&op) {
                out.push(op);
            }
        }
        out
    }
}

/// One filter predicate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filter {
    /// Column name.
    pub column: String,
    /// Operator.
    pub op: FilterOp,
    /// Primary value.
    pub value: String,
    /// Secondary value.
    pub value2: String,
    /// Whether this filter is active.
    pub enabled: bool,
}

impl Filter {
    /// Compact chip text.
    pub fn chip_label(&self) -> String {
        match self.op {
            FilterOp::Between => {
                format!("{} between {} and {}", self.column, self.value, self.value2)
            }
            op if !op.needs_value() => format!("{} {}", self.column, op.label()),
            op => format!(
                "{} {} '{}'",
                self.column,
                op.label(),
                self.value.replace('\'', "''")
            ),
        }
    }

    /// SQL predicate.
    pub fn to_sql(&self) -> String {
        let c = identifier(&self.column);
        let q = |s: &str| format!("'{}'", s.replace('\'', "''"));
        match self.op {
            FilterOp::Eq => format!("{c} = {}", q(&self.value)),
            FilterOp::Ne => format!("{c} <> {}", q(&self.value)),
            FilterOp::Contains => format!("{c} ILIKE {}", q(&format!("%{}%", self.value))),
            FilterOp::NotContains => format!("{c} NOT ILIKE {}", q(&format!("%{}%", self.value))),
            FilterOp::StartsWith => format!("{c} ILIKE {}", q(&format!("{}%", self.value))),
            FilterOp::EndsWith => format!("{c} ILIKE {}", q(&format!("%{}", self.value))),
            FilterOp::Gt => format!("{c} > {}", q(&self.value)),
            FilterOp::Ge => format!("{c} >= {}", q(&self.value)),
            FilterOp::Lt => format!("{c} < {}", q(&self.value)),
            FilterOp::Le => format!("{c} <= {}", q(&self.value)),
            FilterOp::IsNull => format!("{c} IS NULL"),
            FilterOp::IsNotNull => format!("{c} IS NOT NULL"),
            FilterOp::IsEmpty => format!("{c} = ''"),
            FilterOp::IsNotEmpty => format!("{c} <> ''"),
            FilterOp::In => format!(
                "{c} IN ({})",
                self.value
                    .split(',')
                    .map(|s| q(s.trim()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            FilterOp::NotIn => format!(
                "{c} NOT IN ({})",
                self.value
                    .split(',')
                    .map(|s| q(s.trim()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            FilterOp::Between => format!("{c} BETWEEN {} AND {}", q(&self.value), q(&self.value2)),
            FilterOp::Regex => format!("{c} ~ {}", q(&self.value)),
        }
    }

    /// Predicates the demo engine can evaluate.
    pub fn predicates(&self) -> Vec<tablepro_sql::Predicate> {
        use tablepro_sql::Cmp;
        let p = |cmp: Cmp, value: &str| tablepro_sql::Predicate {
            column: self.column.clone(),
            cmp,
            value: value.to_owned(),
        };
        match self.op {
            FilterOp::Eq => vec![p(Cmp::Eq, &self.value)],
            FilterOp::Ne => vec![p(Cmp::Ne, &self.value)],
            FilterOp::Contains => vec![p(Cmp::Like, &format!("%{}%", self.value))],
            FilterOp::NotContains => vec![],
            FilterOp::StartsWith => vec![p(Cmp::Like, &format!("{}%", self.value))],
            FilterOp::EndsWith => vec![p(Cmp::Like, &format!("%{}", self.value))],
            FilterOp::Gt => vec![p(Cmp::Gt, &self.value)],
            FilterOp::Ge => vec![p(Cmp::Ge, &self.value)],
            FilterOp::Lt => vec![p(Cmp::Lt, &self.value)],
            FilterOp::Le => vec![p(Cmp::Le, &self.value)],
            FilterOp::IsNull => vec![p(Cmp::IsNull, "")],
            FilterOp::IsNotNull => vec![p(Cmp::IsNotNull, "")],
            FilterOp::IsEmpty => vec![p(Cmp::Eq, "")],
            FilterOp::IsNotEmpty => vec![p(Cmp::Ne, "")],
            FilterOp::In => vec![p(
                Cmp::In(self.value.split(',').map(|s| s.trim().to_owned()).collect()),
                "",
            )],
            FilterOp::NotIn => vec![],
            FilterOp::Between => vec![p(Cmp::Ge, &self.value), p(Cmp::Le, &self.value2)],
            FilterOp::Regex => vec![],
        }
    }

    /// Apply this filter to one value.
    pub fn matches(&self, value: &Value) -> bool {
        if !self.enabled {
            return true;
        }
        let text = value.display();
        let left = text.to_ascii_lowercase();
        let right = self.value.to_ascii_lowercase();
        match self.op {
            FilterOp::Eq => eq(value, &self.value),
            FilterOp::Ne => !eq(value, &self.value),
            FilterOp::Contains => left.contains(&right),
            FilterOp::NotContains => !left.contains(&right),
            FilterOp::StartsWith => left.starts_with(&right),
            FilterOp::EndsWith => left.ends_with(&right),
            FilterOp::Gt => cmp(value, &self.value).is_gt(),
            FilterOp::Ge => cmp(value, &self.value).is_ge(),
            FilterOp::Lt => cmp(value, &self.value).is_lt(),
            FilterOp::Le => cmp(value, &self.value).is_le(),
            FilterOp::IsNull => matches!(value, Value::Null),
            FilterOp::IsNotNull => !matches!(value, Value::Null),
            FilterOp::IsEmpty => text.is_empty(),
            FilterOp::IsNotEmpty => !text.is_empty(),
            FilterOp::In => self.value.split(',').any(|item| eq(value, item.trim())),
            FilterOp::NotIn => self.value.split(',').all(|item| !eq(value, item.trim())),
            FilterOp::Between => {
                cmp(value, &self.value).is_ge() && cmp(value, &self.value2).is_le()
            }
            FilterOp::Regex => wildcard(&left, &right),
        }
    }
}

fn identifier(name: &str) -> String {
    if name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        name.to_owned()
    } else {
        format!("\"{}\"", name.replace('"', "\"\""))
    }
}

fn eq(value: &Value, text: &str) -> bool {
    value
        .display()
        .eq_ignore_ascii_case(text.trim_matches('\''))
}

fn cmp(value: &Value, text: &str) -> std::cmp::Ordering {
    match (value.as_f64(), text.trim().parse::<f64>()) {
        (Some(a), Ok(b)) => a.total_cmp(&b),
        _ => value
            .display()
            .to_ascii_lowercase()
            .cmp(&text.to_ascii_lowercase()),
    }
}

fn wildcard(value: &str, pattern: &str) -> bool {
    let mut at = 0usize;
    let parts: Vec<&str> = pattern.split('*').filter(|part| !part.is_empty()).collect();
    for part in parts {
        let Some(rest) = value.get(at..) else {
            return false;
        };
        let Some(offset) = rest.find(part) else {
            return false;
        };
        let consumed = offset.saturating_add(part.len());
        at = at.saturating_add(consumed);
    }
    pattern.ends_with('*') || at == value.len()
}

pub const FILTER_EDITOR: termrock::Id = termrock::Id::root("tablepro.filter-editor");
pub const FILTER_COL: termrock::Id = termrock::Id::root("tablepro.filter.col");
pub const FILTER_OP: termrock::Id = termrock::Id::root("tablepro.filter.op");
pub const FILTER_VALUE: termrock::Id = termrock::Id::root("tablepro.filter.value");
pub const FILTER_VALUE2: termrock::Id = termrock::Id::root("tablepro.filter.value2");
pub const FILTER_CANCEL: termrock::Id = termrock::Id::root("tablepro.filter.cancel");
pub const FILTER_APPLY: termrock::Id = termrock::Id::root("tablepro.filter.apply");
pub const FILTER_TITLE: termrock::Id = termrock::Id::root("tablepro.filter.title");
pub const FILTER_NOTE: termrock::Id = termrock::Id::root("tablepro.filter.note");
pub const FILTER_PREVIEW: termrock::Id = termrock::Id::root("tablepro.filter.preview");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterFocus {
    Column,
    Op,
    Value,
    Value2,
    Cancel,
    Apply,
}

impl FilterFocus {
    /// The component id that owns this stop.
    pub(crate) fn id(self) -> termrock::Id {
        match self {
            FilterFocus::Column => FILTER_COL,
            FilterFocus::Op => FILTER_OP,
            FilterFocus::Value => FILTER_VALUE,
            FilterFocus::Value2 => FILTER_VALUE2,
            FilterFocus::Cancel => FILTER_CANCEL,
            FilterFocus::Apply => FILTER_APPLY,
        }
    }
}

/// Stable key for one column option: the column name.
pub fn column_key(col: &(String, ColType)) -> termrock::ItemKey {
    termrock::ItemKey::text(&col.0)
}

/// Row painter for one column option: the column name.
pub fn column_row(col: &(String, ColType), row: &mut termrock::RowUi<'_>) {
    row.label(&col.0);
}

/// Stable key for one operator option: the operator discriminant. The op
/// list reorders per column type, so positional keys would be unstable.
pub fn op_key(op: &FilterOp) -> termrock::ItemKey {
    termrock::ItemKey::num(*op as u64)
}

/// Row painter for one operator option: the operator label.
pub fn op_row(op: &FilterOp, row: &mut termrock::RowUi<'_>) {
    row.label(op.label());
}

#[derive(Debug, Clone)]
pub struct FilterEditor {
    pub index: Option<usize>,
    pub columns: Vec<(String, ColType)>,
    pub column_idx: usize,
    pub col_select: termrock::SelectState,
    pub op: FilterOp,
    pub op_select: termrock::SelectState,
    pub ops: Vec<FilterOp>,
    pub value: String,
    pub value_state: termrock::TextInputState,
    pub value2: String,
    pub value2_state: termrock::TextInputState,
    pub focus: FilterFocus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterOutcome {
    Keep,
    Apply(Filter),
    Cancel,
}

impl FilterEditor {
    pub fn new(
        columns: Vec<(String, ColType)>,
        index: Option<usize>,
        prefill: Option<(usize, FilterOp, String)>,
        cursor_col: usize,
    ) -> Self {
        let (col_idx, op, value, value2) = match prefill {
            Some((c, op, v)) => (c.min(columns.len().saturating_sub(1)), op, v, String::new()),
            None => (
                cursor_col.min(columns.len().saturating_sub(1)),
                FilterOp::Eq,
                String::new(),
                String::new(),
            ),
        };
        let ty = columns
            .get(col_idx)
            .map(|(_, ty)| *ty)
            .unwrap_or(ColType::Text);
        let ops = FilterOp::ordered_for(ty);
        let initial_focus = if value.is_empty() {
            FilterFocus::Value
        } else {
            FilterFocus::Apply
        };
        let mut col_select = termrock::SelectState::default();
        col_select.set_value(columns.get(col_idx).map(column_key));
        let mut op_select = termrock::SelectState::default();
        op_select.set_value(Some(op_key(&op)));
        Self {
            index,
            columns,
            column_idx: col_idx,
            col_select,
            op,
            op_select,
            ops,
            value,
            value_state: termrock::TextInputState::default(),
            value2,
            value2_state: termrock::TextInputState::default(),
            focus: initial_focus,
        }
    }

    /// Apply a `SelectAction::Chose` key from the column dropdown: resolve
    /// the stable key to a column and rebuild the type-appropriate op list.
    /// Unknown keys are ignored.
    pub fn choose_column(&mut self, key: termrock::ItemKey) {
        if let Some(idx) = self.columns.iter().position(|col| column_key(col) == key) {
            self.column_idx = idx;
            self.update_ops();
        }
    }

    /// Apply a `SelectAction::Chose` key from the operator dropdown.
    /// Unknown keys are ignored.
    pub fn choose_op(&mut self, key: termrock::ItemKey) {
        if let Some(op) = FilterOp::ALL.iter().copied().find(|op| op_key(op) == key) {
            self.op = op;
        }
    }

    pub fn to_filter(&self) -> Filter {
        Filter {
            column: self
                .columns
                .get(self.column_idx)
                .map(|(c, _)| c.clone())
                .unwrap_or_default(),
            op: self.op,
            value: self.value.trim().to_owned(),
            value2: self.value2.trim().to_owned(),
            enabled: true,
        }
    }

    pub(crate) fn next_focus(&self) -> FilterFocus {
        match self.focus {
            FilterFocus::Column => FilterFocus::Op,
            FilterFocus::Op => {
                if self.op.needs_value() {
                    FilterFocus::Value
                } else {
                    FilterFocus::Cancel
                }
            }
            FilterFocus::Value => {
                if self.op == FilterOp::Between {
                    FilterFocus::Value2
                } else {
                    FilterFocus::Cancel
                }
            }
            FilterFocus::Value2 => FilterFocus::Cancel,
            FilterFocus::Cancel => FilterFocus::Apply,
            FilterFocus::Apply => FilterFocus::Column,
        }
    }

    pub(crate) fn prev_focus(&self) -> FilterFocus {
        match self.focus {
            FilterFocus::Column => FilterFocus::Apply,
            FilterFocus::Op => FilterFocus::Column,
            FilterFocus::Value => FilterFocus::Op,
            FilterFocus::Value2 => FilterFocus::Value,
            FilterFocus::Cancel => {
                if self.op == FilterOp::Between {
                    FilterFocus::Value2
                } else if self.op.needs_value() {
                    FilterFocus::Value
                } else {
                    FilterFocus::Op
                }
            }
            FilterFocus::Apply => FilterFocus::Cancel,
        }
    }

    fn update_ops(&mut self) {
        if let Some((_, ty)) = self.columns.get(self.column_idx) {
            self.ops = FilterOp::ordered_for(*ty);
            if !self.ops.contains(&self.op) {
                self.op = self.ops.first().copied().unwrap_or(FilterOp::Eq);
            }
        }
    }

    pub fn on_key(&mut self, key: termrock::Key) -> FilterOutcome {
        if key.code == termrock::KeyCode::Esc {
            // An open Select popup is dismissed by the runtime bubble pass;
            // the editor stays open and `Closed` arrives on the next pass.
            if self.col_select.is_open() || self.op_select.is_open() {
                return FilterOutcome::Keep;
            }
            return FilterOutcome::Cancel;
        }

        if !self.col_select.is_open()
            && !self.op_select.is_open()
            && !self.value_state.is_editing()
            && !self.value2_state.is_editing()
        {
            match key.code {
                termrock::KeyCode::Tab => {
                    self.focus = self.next_focus();
                    return FilterOutcome::Keep;
                }
                termrock::KeyCode::BackTab => {
                    self.focus = self.prev_focus();
                    return FilterOutcome::Keep;
                }
                _ => {}
            }
        }

        match self.focus {
            FilterFocus::Column => {
                // Owned by termrock::Select via update_filter_editor and
                // draw below. Keys arriving here were not consumed by the
                // component (idle, unbound); ignore them.
            }
            FilterFocus::Op => {
                // Owned by termrock::Select via update_filter_editor and
                // draw below. Keys arriving here were not consumed by the
                // component (idle, unbound); ignore them.
            }
            FilterFocus::Value => {
                // Owned by termrock::TextInput via update_filter_editor and
                // draw below. Keys arriving here were not consumed by the
                // component (idle, unbound); ignore them.
            }
            FilterFocus::Value2 => {
                // Owned by termrock::TextInput via update_filter_editor and
                // draw below. Keys arriving here were not consumed by the
                // component (idle, unbound); ignore them.
            }
            FilterFocus::Cancel => {
                // Owned by termrock::Button via update_filter_editor and
                // draw below. Keys arriving here were not consumed by the
                // component (idle, unbound); ignore them.
            }
            FilterFocus::Apply => {
                // Owned by termrock::Button via update_filter_editor and
                // draw below. Keys arriving here were not consumed by the
                // component (idle, unbound); ignore them.
            }
        }
        FilterOutcome::Keep
    }

    pub fn draw(&self, ui: &mut termrock::Ui<'_>, area: termrock::Rect) {
        use termrock::{
            FgStep, Field, Insets, List, ListState, Modifier, Panel, PanelKind, Part, Role, RowUi,
            SelectField, StylePatch, Surface, truncate,
        };

        // Draw order is the ring order: the title `List` first, then the
        // fields and controls in ring sequence. Static `List` rows register
        // `Focusable` stops; `update_filter_editor` bounces or skips them so
        // they never hold focus at draw.
        ui.with_surface(Surface::Elevated, |ui| {
            Panel::new(FILTER_EDITOR)
                .kind(PanelKind::Framed)
                // Props-derived: the region holds focus, so the border paints
                // BorderStrong via the recipe.
                .focused(true)
                // Keep the frame rule: the head cell stays recipe-owned `─`.
                .slot(Part::GUTTER, &|_, _| {})
                // Control column x+2 inside the clip; freezes all four sides.
                .inner_inset(Insets {
                    l: 2,
                    t: 1,
                    r: 2,
                    b: 1,
                })
                .draw(ui, area, |ui, _body| {
                    // Title row: a 1-item `List`; chrome at x/x+1 clips (both
                    // are left of the inner rect), text lands at x+3.
                    let title_text = if self.index.is_some() {
                        "Edit filter"
                    } else {
                        "Add filter"
                    };
                    let clear = StylePatch::new().clear_fg();
                    let parts = [(Part::CONTAINER, clear)];
                    let title_label = StylePatch::new()
                        .set_fg(Role::Fg(FgStep::Primary))
                        .add(Modifier::BOLD);
                    let title_items = [title_text];
                    let title_state = ListState::default();
                    List::new(FILTER_TITLE)
                        .row(|item: &&str, r: &mut RowUi<'_>| {
                            r.label_patched(item, &title_label);
                        })
                        .patch_part(&parts)
                        .draw(
                            ui,
                            termrock::Rect::new(area.x, area.y + 1, area.width, 1),
                            &title_state,
                            &title_items,
                        );

                    // Column + Operator labels and controls (owned by
                    // termrock::Field over the Select adapter).
                    Field::new(
                        "Column",
                        SelectField::new(
                            termrock::Select::new(FILTER_COL)
                                .key(column_key)
                                .row(column_row),
                            &self.columns,
                        ),
                    )
                    .plain(true)
                    .optional_suffix(false)
                    .draw(
                        ui,
                        termrock::Rect::new(area.x + 2, area.y + 3, 28, 2),
                        &self.col_select,
                    );

                    Field::new(
                        "Operator",
                        SelectField::new(
                            termrock::Select::new(FILTER_OP).key(op_key).row(op_row),
                            &self.ops,
                        ),
                    )
                    .plain(true)
                    .optional_suffix(false)
                    .help("Operators that fit the col…")
                    .draw(
                        ui,
                        termrock::Rect::new(area.x + 32, area.y + 3, 29, 3),
                        &self.op_select,
                    );

                    // Value field(s). `Between` renders value + value2 side by side
                    // on the SAME row: oracle `Split::new(50, 12, 12).horizontal` with
                    // gap 2 over the 59-wide value field is usable 57, first 28, gap 2,
                    // second 29 (the same 28/29 split as the Column/Op row above).
                    if self.op.needs_value() {
                        let between = self.op == FilterOp::Between;
                        Field::new(
                            "Value",
                            termrock::TextInput::new(FILTER_VALUE)
                                .value(&self.value)
                                .placeholder("value"),
                        )
                        .plain(true)
                        .draw(
                            ui,
                            termrock::Rect::new(
                                area.x + 2,
                                area.y + 6,
                                if between { 28 } else { 59 },
                                2,
                            ),
                            &self.value_state,
                        );

                        // Second value field, `Between` only (owned by
                        // termrock::TextInput). Label mirrors the sibling value label
                        // exactly; legacy text is "and".
                        if between {
                            Field::new(
                                "and",
                                termrock::TextInput::new(FILTER_VALUE2)
                                    .value(&self.value2)
                                    .placeholder("value"),
                            )
                            .plain(true)
                            .draw(
                                ui,
                                termrock::Rect::new(area.x + 32, area.y + 6, 29, 2),
                                &self.value2_state,
                            );
                        }
                    } else {
                        let note_label = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                        let note_items = ["No value needed for this operator"];
                        let note_state = ListState::default();
                        List::new(FILTER_NOTE)
                            .row(|item: &&str, r: &mut RowUi<'_>| {
                                r.label_patched(item, &note_label);
                            })
                            .patch_part(&parts)
                            .draw(
                                ui,
                                termrock::Rect::new(area.x + 1, area.y + 7, area.width - 2, 1),
                                &note_state,
                                &note_items,
                            );
                    }

                    // SQL Preview (live: draft while editing, committed otherwise)
                    let mut preview_filter = self.to_filter();
                    if let Some(draft) = self.value_state.draft_text() {
                        preview_filter.value = draft.trim().to_owned();
                    }
                    if let Some(draft) = self.value2_state.draft_text() {
                        preview_filter.value2 = draft.trim().to_owned();
                    }
                    let sql_filter = preview_filter.to_sql();
                    let preview = format!("WHERE {sql_filter}");
                    let preview_label = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                    let fitted = truncate(&preview, 58);
                    let preview_items = [fitted.as_str()];
                    let preview_state = ListState::default();
                    List::new(FILTER_PREVIEW)
                        .row(|item: &&str, r: &mut RowUi<'_>| {
                            r.label_patched(item, &preview_label);
                        })
                        .patch_part(&parts)
                        .draw(
                            ui,
                            termrock::Rect::new(area.x, area.y + 10, area.width, 1),
                            &preview_state,
                            &preview_items,
                        );

                    // Buttons (owned by termrock::Button: focus, hover, press and
                    // activation all come from the component).
                    let confirm_label: &str = if self.index.is_some() {
                        "Update filter"
                    } else {
                        "Add filter"
                    };
                    let confirm_w = (confirm_label.len() + 2) as u16;
                    let cancel_w = 8u16;
                    let confirm_x = area.right().saturating_sub(3 + confirm_w);
                    let cancel_x = confirm_x.saturating_sub(1 + cancel_w);

                    let cancel_rect = termrock::Rect::new(cancel_x, area.y + 13, cancel_w, 1);
                    let confirm_rect = termrock::Rect::new(confirm_x, area.y + 13, confirm_w, 1);
                    // The body already runs on the Elevated plane via the
                    // framed panel, so no inner `with_surface` is needed.
                    termrock::Button::new(FILTER_CANCEL, "Cancel")
                        .variant(termrock::Variant::SUBTLE)
                        .draw(ui, cancel_rect);
                    termrock::Button::new(FILTER_APPLY, confirm_label)
                        .variant(termrock::Variant::PRIMARY)
                        .draw(ui, confirm_rect);
                })
        });
    }
}
