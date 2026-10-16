//! How the showcase demonstrates each library capability.
//!
//! One declaration per capability id of `junie_tui::catalog`. The coverage
//! gate (`app_tests_catalog.rs`) checks every declaration against the
//! running showcase, so a declaration cannot exist without the thing it
//! claims:
//!
//! - [`Shown::Demo`]: the page (its `--page` slug) renders the label text;
//! - [`Shown::Option`]: a live option of a page's option table (SC1), which
//!   `--opt <page>.<key>` flips;
//! - [`Shown::StateRow`]: a row of a page's States grid (SC2).
//!
//! The option tables (SC1) and States grids (SC2) generate their entries
//! from these declarations. A capability that is not declared here must be
//! listed in `tests/showcase_coverage_allow.txt` with a reason.

// The showcase binary reads these declarations once the option tables
// (SC1) land; until then only the gate does.
#![cfg_attr(not(test), allow(dead_code))]

/// How one capability is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shown {
    /// An option key of a page's option table: `(page slug, key)`.
    Option(&'static str, &'static str),
    /// A row of a page's States grid: `(page slug, state row)`.
    StateRow(&'static str, &'static str),
    /// Rendered on a page: `(page slug, text the page shows)`.
    Demo(&'static str, &'static str),
}

use Shown::{Demo, Option, StateRow};

/// Every declared demonstration, keyed by capability id.
pub static SHOWN: &[(&str, Shown)] = &[
    ("theme::Theme", Demo("overview", "Tokens and principles")),
    ("theme::ColorLevel", Demo("overview", "truecolor ·")),
    (
        "theme::ColorLevel::TrueColor",
        Demo("overview", "truecolor ·"),
    ),
    ("theme::Theme::canvas", Demo("overview", "▏ canvas #")),
    ("theme::Theme::surface", Demo("overview", "▏ surface #")),
    (
        "theme::Theme::surface_elevated",
        Demo("overview", "▏ surface.elevated #"),
    ),
    (
        "theme::Theme::surface_overlay",
        Demo("overview", "▏ surface.overlay #"),
    ),
    ("theme::Theme::field", Demo("overview", "▏ field #")),
    ("theme::Theme::popover", Demo("overview", "▏ popover #")),
    (
        "theme::Theme::border_subtle",
        Demo("overview", "▏ border.subtle #"),
    ),
    (
        "theme::Theme::border_strong",
        Demo("overview", "▏ border.strong #"),
    ),
    (
        "theme::Theme::text_primary",
        Demo("overview", "▏ text.primary #"),
    ),
    (
        "theme::Theme::text_secondary",
        Demo("overview", "▏ text.secondary #"),
    ),
    (
        "theme::Theme::text_muted",
        Demo("overview", "▏ text.muted #"),
    ),
    (
        "theme::Theme::text_faint",
        Demo("overview", "▏ text.faint #"),
    ),
    ("theme::Theme::accent", Demo("overview", "▏ accent #")),
    (
        "theme::Theme::accent_hover",
        Demo("overview", "▏ accent.hover #"),
    ),
    (
        "theme::Theme::accent_pressed",
        Demo("overview", "▏ accent.pressed #"),
    ),
    ("theme::Theme::accent_bg", Demo("overview", "▏ accent.bg #")),
    ("theme::Theme::highlight", Demo("overview", "▏ highlight #")),
    (
        "theme::Theme::selection_bg",
        Demo("overview", "▏ selection #"),
    ),
    ("theme::Theme::success", Demo("overview", "▏ success #")),
    ("theme::Theme::info", Demo("overview", "▏ info #")),
    ("theme::Theme::warning", Demo("overview", "▏ warning #")),
    ("theme::Theme::error", Demo("overview", "▏ error #")),
    ("theme::Theme::danger_label", Demo("overview", "▏ danger #")),
    ("widgets::brand::Lockup", Demo("chrome", "app❯")),
    (
        "widgets::menu::MenuBar",
        Demo("chrome", "app❯ File View Help"),
    ),
    (
        "widgets::menu::ContextMenu",
        Demo("chrome", "right-click or Shift+F10 for the tab menu"),
    ),
    (
        "widgets::statusbar::StatusBar",
        Demo("chrome", "payments-platform PR #482"),
    ),
    (
        "widgets::statusbar::StatusItem",
        Demo("chrome", "Claude Code · working · 2 tabs"),
    ),
    (
        "widgets::hintbar::HintBar",
        Demo("chrome", "hint bar · topmost layer wins"),
    ),
    (
        "widgets::button::Button",
        Demo("buttons", "Run task Preview Cancel Delete branch"),
    ),
    (
        "widgets::button::Button::disabled",
        Demo("buttons", "Disabled primary"),
    ),
    (
        "theme::ButtonKind",
        Demo("buttons", "Primary Secondary Subtle Danger"),
    ),
    (
        "theme::ButtonKind::Primary",
        Demo("buttons", "Primary Secondary Subtle Danger"),
    ),
    (
        "theme::ButtonKind::Secondary",
        Demo("buttons", "Primary Secondary Subtle Danger"),
    ),
    (
        "theme::ButtonKind::Subtle",
        Demo("buttons", "Primary Secondary Subtle Danger"),
    ),
    (
        "theme::ButtonKind::Danger",
        Demo("buttons", "Primary Secondary Subtle Danger"),
    ),
    (
        "widgets::input::TextInput",
        Demo("inputs", "Project name *"),
    ),
    (
        "widgets::input::TextInput::value",
        Demo("inputs", "payments-gateway"),
    ),
    (
        "widgets::input::TextInput::required",
        Demo("inputs", "Owner email *"),
    ),
    (
        "widgets::input::TextInput::placeholder",
        Demo("inputs", "Type a path or symbol…"),
    ),
    (
        "widgets::input::TextInput::help",
        Demo("inputs", "Used as the working directory name"),
    ),
    (
        "widgets::input::TextInput::validator",
        Demo("inputs", "Enter a valid email address"),
    ),
    (
        "widgets::input::TextInput::masked",
        Demo("inputs", "jb_live_••••"),
    ),
    (
        "widgets::input::TextInput::reveal_tail",
        Demo("inputs", "••••c1f2"),
    ),
    (
        "widgets::input::TextInput::disabled",
        StateRow("inputs", "disabled"),
    ),
    (
        "widgets::textarea::TextArea",
        Demo("textareas", "Task description"),
    ),
    (
        "widgets::textarea::TextArea::value",
        Demo("textareas", "1. Read src/api/billing.rs"),
    ),
    (
        "widgets::textarea::TextArea::placeholder",
        Demo("textareas", "Anything the agent should know…"),
    ),
    (
        "widgets::textarea::TextArea::help",
        Demo("textareas", "Use the imperative mood and explain why"),
    ),
    (
        "widgets::textarea::TextArea::error",
        Demo("textareas", "fix stuff !"),
    ),
    (
        "widgets::textarea::TextArea::disabled",
        Demo("textareas", "Read-only transcript"),
    ),
    (
        "widgets::chips::ChipBar",
        Demo("chipsselects", "+ Add filter"),
    ),
    (
        "widgets::chips::Chip",
        Demo("chipsselects", "status = 'pending' ×"),
    ),
    (
        "widgets::select::Select",
        Demo("chipsselects", "created_at ▾"),
    ),
    (
        "widgets::segments::Segment",
        Demo("chipsselects", "◆ production acme_prod"),
    ),
    (
        "widgets::props::Prop",
        Demo("chipsselects", "Host prod-db-1.acme.io:5432"),
    ),
    (
        "widgets::empty::EmptyState",
        Demo("chipsselects", "No results yet"),
    ),
    (
        "widgets::choice::Toggle",
        Demo("forms", "Auto-approve changes"),
    ),
    ("widgets::choice::RadioGroup", Demo("forms", "(●) Balanced")),
    (
        "widgets::choice::Checkbox",
        Demo("forms", "[✓] Run tests before opening a PR"),
    ),
    (
        "widgets::table::DataTable",
        Demo("basictable", "Sort by header"),
    ),
    (
        "widgets::table::Column",
        Demo("basictable", "ID Task Owner Status Branch Changes Duration"),
    ),
    (
        "widgets::table::DataTable::empty_text",
        Demo("basictable", "No checks have run yet"),
    ),
    (
        "widgets::table::DataTable::validator",
        Demo("editable", "fix/checkout flake !"),
    ),
    // Editable, Selection and Cell editors (SC13)
    (
        "widgets::option_editor::OptionItem",
        Demo("celleditors", "Priority P1 (Page on-call now)"),
    ),
    (
        "widgets::option_editor::OptionItem::label",
        Demo("celleditors", "Env Development · Staging · Production"),
    ),
    (
        "widgets::option_editor::OptionItem::tone",
        Demo("celleditors", "Rollout Off · Canary · 50 % · Full"),
    ),
    (
        "widgets::option_editor::OptionItem::description",
        Demo("celleditors", "P2 (Fix today)"),
    ),
    (
        "widgets::option_editor::OptionItem::disabled",
        Demo("celleditors", "P4 (off: Use P3 or close)"),
    ),
    (
        "widgets::option_editor::Options",
        Demo(
            "celleditors",
            "Labels api · web · worker · db · cache · edge",
        ),
    ),
    (
        "widgets::option_editor::ColumnEditor::Text",
        Demo("editable", "Task left · editable · text"),
    ),
    (
        "widgets::option_editor::EditorConfig",
        Option("celleditors", "celleditors.max_rows"),
    ),
    (
        "widgets::option_editor::Affordance",
        Option("celleditors", "celleditors.affordance"),
    ),
    (
        "widgets::option_editor::Affordance::Focused",
        Option("celleditors", "celleditors.affordance=focused"),
    ),
    (
        "widgets::option_editor::Affordance::Always",
        Option("celleditors", "celleditors.affordance=always"),
    ),
    (
        "widgets::option_editor::Affordance::Never",
        Option("celleditors", "celleditors.affordance=never"),
    ),
    (
        "widgets::option_editor::ChoiceResult",
        Demo("celleditors", "Enter or a click: Commit"),
    ),
    (
        "widgets::option_editor::ChoiceResult::Commit",
        Demo("celleditors", "Enter or a click: Commit"),
    ),
    (
        "widgets::option_editor::ChoiceResult::Cancel",
        Demo("celleditors", "Esc or outside: Cancel"),
    ),
    (
        "widgets::option_editor::ChoiceResult::CommitAndTab",
        Demo("celleditors", "Tab: CommitAndTab"),
    ),
    (
        "widgets::option_editor::ChoiceEditor",
        Demo("celleditors", "Default region eu-west-1 ▾"),
    ),
    (
        "widgets::table::SortDir",
        Demo("editable", "Sort ▴ ascending, again ▾ descending"),
    ),
    (
        "widgets::table::SortDir::Asc",
        Demo("editable", "Sort ▴ ascending"),
    ),
    (
        "widgets::table::SortDir::Desc",
        Demo("editable", "again ▾ descending"),
    ),
    (
        "widgets::table::Align",
        Demo("editable", "Changes right · editable · text"),
    ),
    (
        "widgets::table::Align::Left",
        Demo("editable", "Branch left · editable · text"),
    ),
    (
        "widgets::table::Align::Right",
        Demo("editable", "Changes right · editable · text"),
    ),
    (
        "widgets::table::Column::editor",
        Demo("editable", "Owner left · editable · ▾ Select"),
    ),
    (
        "widgets::table::Column::right",
        Demo("editable", "Changes right · editable · text"),
    ),
    (
        "widgets::table::Column::editable",
        Demo("editable", "Task left · editable · text"),
    ),
    (
        "widgets::table::Cell",
        Demo("editable", "#1040 Add rate limiting to auth endpoints"),
    ),
    (
        "widgets::table::Cell::tone",
        Demo("editable", "Status left · read-only"),
    ),
    (
        "widgets::table::ChoiceEdit",
        Demo("celleditors", "▾ Select Env"),
    ),
    ("widgets::table::EditState", StateRow("editable", "editing")),
    (
        "widgets::table::CellValidator",
        StateRow("editable", "error"),
    ),
    (
        "widgets::table::TableCmd",
        Option("editable", "editable.keymap"),
    ),
    (
        "widgets::table::TableCmd::SortByColumn",
        Option("editable", "editable.keymap"),
    ),
    (
        "widgets::table::TableCmd::SelectAll",
        Option("editable", "editable.keymap"),
    ),
    (
        "widgets::table::TableAction",
        Demo("editable", "Esc Cancelled"),
    ),
    (
        "widgets::table::TableAction::Committed",
        Demo("editable", "Edit · Enter again Committed"),
    ),
    (
        "widgets::table::TableAction::Cancelled",
        Demo("editable", "Esc Cancelled"),
    ),
    (
        "widgets::table::TableAction::Activated",
        Demo("editable", "Enter on a row (Basic table) Activated"),
    ),
    (
        "widgets::table::TableAction::SelectionChanged",
        Demo("editable", "Space (Selection) SelectionChanged"),
    ),
    (
        "widgets::table::TableAction::LeaveForward",
        Demo("editable", "LeaveForward · Shift+Tab LeaveBackward"),
    ),
    (
        "widgets::table::TableAction::LeaveBackward",
        Demo("editable", "LeaveForward · Shift+Tab LeaveBackward"),
    ),
    (
        "widgets::table::DataTable::type_ahead",
        Option("editable", "editable.type_ahead=any"),
    ),
    (
        "widgets::table::DataTable::type_ahead_column",
        Option("editable", "editable.type_ahead=task"),
    ),
    (
        "widgets::table::DataTable::editor_config",
        Option("editable", "editable.affordance"),
    ),
    (
        "widgets::table::DataTable::nav",
        Option("editable", "editable.nav"),
    ),
    (
        "widgets::table::DataTable::keymap",
        Option("editable", "editable.keymap"),
    ),
    (
        "widgets::table::DataTable::esc_clears",
        Option("selection", "selection.esc_clears"),
    ),
    (
        "widgets::table::DataTable::toggle_mods",
        Option("selection", "selection.toggle_mods"),
    ),
    (
        "widgets::table::DataTable::disabled_rows",
        Option("selection", "selection.disabled_rows"),
    ),
    (
        "widgets::table::DataTable::horizontal",
        Option("editable", "editable.horizontal"),
    ),
    (
        "widgets::table::DataTable::overflow_marks",
        Option("editable", "editable.overflow_marks"),
    ),
    (
        "widgets::table::DataTable::reveal",
        Option("editable", "editable.reveal"),
    ),
    (
        "widgets::table::DataTable::cell_nav",
        Demo("editable", "reversed cell cursor (navigation)"),
    ),
    (
        "widgets::table::DataTable::numeric",
        Option("editable", "editable.numeric"),
    ),
    (
        "widgets::table::DataTable::empty",
        Option("editable", "editable.rows"),
    ),
    (
        "widgets::table::DataTable::track_click",
        Option("editable", "editable.track_click"),
    ),
    // Data grid (SC12)
    (
        "widgets::grid::CellValue",
        Demo("datagrid", "1001 Northwind Traders enterprise 3 57.00 true"),
    ),
    (
        "widgets::grid::CellValue::Null",
        Demo("datagrid", "0.00 false NULL"),
    ),
    (
        "widgets::grid::CellValue::Default",
        Demo("datagrid", "+ 3 DEFAULT"),
    ),
    (
        "widgets::grid::CellValue::Text",
        Demo("datagrid", "Northwind Traders enterprise"),
    ),
    (
        "widgets::grid::CellValue::Int",
        Demo("datagrid", "1001 Northwind Traders"),
    ),
    (
        "widgets::grid::CellValue::Num",
        Demo("datagrid", "57.00 true"),
    ),
    (
        "widgets::grid::CellValue::Bool",
        Demo("datagrid", "57.00 true"),
    ),
    (
        "widgets::grid::CellValue::Json",
        Demo("datagrid", "{\"owner\":\"mira\",\"seats\":3}"),
    ),
    (
        "widgets::grid::CellKind",
        Demo("datagrid", "id Id · integer"),
    ),
    (
        "widgets::grid::CellKind::Text",
        Demo("datagrid", "customer Text · text"),
    ),
    (
        "widgets::grid::CellKind::Id",
        Demo("datagrid", "id Id · integer"),
    ),
    (
        "widgets::grid::CellKind::Number",
        Demo("datagrid", "seats Number · integer"),
    ),
    (
        "widgets::grid::CellKind::Bool",
        Demo("datagrid", "active Bool · boolean"),
    ),
    (
        "widgets::grid::CellKind::Timestamp",
        Demo("datagrid", "renewed_at Timestamp · date"),
    ),
    (
        "widgets::grid::CellKind::Json",
        Demo("datagrid", "notes Json · jsonb"),
    ),
    (
        "widgets::grid::CellKind::Enum",
        Demo("datagrid", "plan Enum · plan_tier"),
    ),
    (
        "widgets::grid::ColumnSpec",
        Demo(
            "datagrid",
            "id Id · integer · primary key · not null · read-only",
        ),
    ),
    (
        "widgets::grid::ColumnSpec::editor",
        Demo("datagrid", "▾ picks from a fixed set"),
    ),
    (
        "widgets::grid::ColumnSpec::primary",
        Demo("datagrid", "integer · primary key"),
    ),
    (
        "widgets::grid::ColumnSpec::nullable",
        Demo("datagrid", "customer Text · text · not null"),
    ),
    (
        "widgets::grid::ColumnSpec::read_only",
        Demo("datagrid", "mrr Number · numeric(10,2) · read-only"),
    ),
    (
        "widgets::grid::ColumnSpec::references",
        Demo("datagrid", "integer · references accounts"),
    ),
    (
        "widgets::grid::ColumnSpec::type_label",
        Demo("datagrid", "plan Enum · plan_tier"),
    ),
    (
        "widgets::grid::ColumnSpec::enum_values",
        Demo("datagrid", "one of free, pro, team, enterprise"),
    ),
    (
        "widgets::grid::RowTotal",
        Option("datagrid", "datagrid.total"),
    ),
    (
        "widgets::grid::RowTotal::Exact",
        Option("datagrid", "datagrid.total=exact"),
    ),
    (
        "widgets::grid::RowTotal::Estimated",
        Option("datagrid", "datagrid.total=estimated"),
    ),
    (
        "widgets::grid::RowTotal::Unknown",
        Option("datagrid", "datagrid.total=unknown"),
    ),
    (
        "widgets::grid::GridRows",
        Demo("datagrid", "of 40 loaded · ~4,812 total"),
    ),
    (
        "widgets::grid::UndoAction",
        Demo("datagrid", "Undo a cell, delete or insert"),
    ),
    (
        "widgets::grid::UndoAction::Cell",
        Demo("datagrid", "Undo a cell, delete or insert"),
    ),
    (
        "widgets::grid::UndoAction::Delete",
        Demo("datagrid", "Undo a cell, delete or insert"),
    ),
    (
        "widgets::grid::UndoAction::Insert",
        Demo("datagrid", "Undo a cell, delete or insert"),
    ),
    (
        "widgets::grid::PendingChanges",
        Demo("datagrid", "• 3 pending"),
    ),
    (
        "widgets::grid::RowState",
        Demo(
            "datagrid",
            "Change marks • modified + inserted − deleted ! error",
        ),
    ),
    (
        "widgets::grid::RowState::Clean",
        Demo("datagrid", "Pending no pending changes"),
    ),
    (
        "widgets::grid::RowState::Modified",
        Demo("datagrid", "• modified"),
    ),
    (
        "widgets::grid::RowState::Inserted",
        Demo("datagrid", "+ inserted"),
    ),
    (
        "widgets::grid::RowState::Deleted",
        Demo("datagrid", "− deleted"),
    ),
    (
        "widgets::grid::RowState::Error",
        Demo("datagrid", "! error"),
    ),
    ("widgets::grid::EditState", StateRow("datagrid", "editing")),
    (
        "widgets::grid::GridAction",
        Demo("datagrid", "Set NULL · CellChanged"),
    ),
    (
        "widgets::grid::GridAction::CellChanged",
        Demo("datagrid", "Set NULL · CellChanged"),
    ),
    (
        "widgets::grid::GridAction::RowInserted",
        Demo("datagrid", "Insert row · RowInserted"),
    ),
    (
        "widgets::grid::GridAction::RowDeleted",
        Demo("datagrid", "Delete row · RowDeleted"),
    ),
    (
        "widgets::grid::GridAction::SortRequested",
        Demo("datagrid", "Sort · SortRequested"),
    ),
    (
        "widgets::grid::GridAction::FetchMore",
        Demo("datagrid", "past the last row FetchMore"),
    ),
    (
        "widgets::grid::GridAction::Refresh",
        Demo("datagrid", "Reload · Refresh"),
    ),
    (
        "widgets::grid::GridAction::CommitRequested",
        Demo("datagrid", "Save review · CommitRequested"),
    ),
    (
        "widgets::grid::GridAction::DiscardRequested",
        Demo("datagrid", "Discard · DiscardRequested"),
    ),
    (
        "widgets::grid::GridAction::PreviewSql",
        Demo("datagrid", "Preview SQL · PreviewSql"),
    ),
    (
        "widgets::grid::GridAction::Copy",
        Demo("datagrid", "Copy · Copy"),
    ),
    (
        "widgets::grid::GridAction::FollowReference",
        Demo("datagrid", "Follow account_id · FollowReference"),
    ),
    (
        "widgets::grid::GridAction::OpenViewer",
        Demo("datagrid", "OpenViewer on JSON"),
    ),
    (
        "widgets::grid::GridAction::FilterOnCell",
        Demo("datagrid", "Filter on cell · FilterOnCell"),
    ),
    (
        "widgets::grid::GridAction::OpenFilters",
        Demo("datagrid", "Filters · OpenFilters"),
    ),
    (
        "widgets::grid::GridAction::ClearFilters",
        Demo("datagrid", "Clear filters · ClearFilters"),
    ),
    (
        "widgets::grid::GridAction::Activated",
        Demo("datagrid", "Enter (read-only) Activated"),
    ),
    (
        "widgets::grid::GridAction::LeaveForward",
        Demo("datagrid", "LeaveForward · Shift+Tab LeaveBackward"),
    ),
    (
        "widgets::grid::GridAction::LeaveBackward",
        Demo("datagrid", "LeaveForward · Shift+Tab LeaveBackward"),
    ),
    (
        "widgets::grid::GridAction::ReadOnly",
        Demo("datagrid", "ReadOnly says why not"),
    ),
    (
        "widgets::grid::Validator",
        Option("datagrid", "datagrid.validator"),
    ),
    (
        "widgets::grid::GridChoice",
        Demo("datagrid", "▾ picks from a fixed set"),
    ),
    (
        "widgets::grid::GridCmd",
        Option("datagrid", "datagrid.keymap"),
    ),
    (
        "widgets::grid::GridCmd::ClearCell",
        Demo("datagrid", "Set NULL · CellChanged"),
    ),
    (
        "widgets::grid::GridCmd::ToggleDelete",
        Demo("datagrid", "Delete row · RowDeleted"),
    ),
    (
        "widgets::grid::GridCmd::InsertRow",
        Demo("datagrid", "Insert row · RowInserted"),
    ),
    (
        "widgets::grid::GridCmd::DuplicateRow",
        Demo("datagrid", "Duplicate row · RowInserted"),
    ),
    (
        "widgets::grid::GridCmd::Copy",
        Demo("datagrid", "Copy · Copy"),
    ),
    (
        "widgets::grid::GridCmd::CopyWithHeader",
        Demo("datagrid", "Copy with header · Copy"),
    ),
    (
        "widgets::grid::GridCmd::Commit",
        Demo("datagrid", "Save review · CommitRequested"),
    ),
    (
        "widgets::grid::GridCmd::Sort",
        Demo("datagrid", "Sort · SortRequested"),
    ),
    (
        "widgets::grid::GridCmd::ClearSort",
        Demo("datagrid", "Clear sort · SortRequested"),
    ),
    (
        "widgets::grid::GridCmd::FilterOnCell",
        Demo("datagrid", "Filter on cell · FilterOnCell"),
    ),
    (
        "widgets::grid::GridCmd::OpenFilters",
        Demo("datagrid", "Filters · OpenFilters"),
    ),
    (
        "widgets::grid::GridCmd::ClearFilters",
        Demo("datagrid", "Clear filters · ClearFilters"),
    ),
    (
        "widgets::grid::GridCmd::Refresh",
        Demo("datagrid", "Reload · Refresh"),
    ),
    (
        "widgets::grid::GridCmd::Undo",
        Demo("datagrid", "Undo a cell, delete or insert"),
    ),
    (
        "widgets::grid::GridCmd::Discard",
        Demo("datagrid", "Discard · DiscardRequested"),
    ),
    (
        "widgets::grid::GridCmd::PreviewSql",
        Demo("datagrid", "Preview SQL · PreviewSql"),
    ),
    (
        "widgets::grid::GridCmd::FollowReference",
        Demo("datagrid", "Follow account_id · FollowReference"),
    ),
    (
        "widgets::grid::GridCmd::SelectAll",
        Demo("datagrid", "Mark every row"),
    ),
    (
        "widgets::grid::DataGrid::editor_config",
        Option("datagrid", "datagrid.affordance"),
    ),
    (
        "widgets::grid::DataGrid::type_ahead",
        Option("datagrid", "datagrid.type_ahead=any"),
    ),
    (
        "widgets::grid::DataGrid::type_ahead_column",
        Option("datagrid", "datagrid.type_ahead=customer"),
    ),
    (
        "widgets::grid::DataGrid::horizontal",
        Option("datagrid", "datagrid.horizontal"),
    ),
    (
        "widgets::grid::DataGrid::overflow_badges",
        Option("datagrid", "datagrid.overflow_badges"),
    ),
    (
        "widgets::grid::DataGrid::bar_labels",
        Option("datagrid", "datagrid.labels"),
    ),
    (
        "widgets::grid::DataGrid::empty",
        Option("datagrid", "datagrid.rows"),
    ),
    (
        "widgets::grid::DataGrid::validator",
        Option("datagrid", "datagrid.validator"),
    ),
    (
        "widgets::grid::DataGrid::editable",
        Option("datagrid", "datagrid.access"),
    ),
    (
        "widgets::grid::DataGrid::duplicate_keys",
        Option("datagrid", "datagrid.duplicate_keys"),
    ),
    (
        "widgets::grid::DataGrid::nav",
        Option("datagrid", "datagrid.nav"),
    ),
    (
        "widgets::grid::DataGrid::keymap",
        Option("datagrid", "datagrid.keymap"),
    ),
    (
        "widgets::grid::DataGrid::width_sample_rows",
        Option("datagrid", "datagrid.width_sample_rows"),
    ),
    (
        "widgets::grid::DataGrid::resizable_columns",
        Option("datagrid", "datagrid.seams=off"),
    ),
    (
        "widgets::grid::DataGrid::seam_hit_w",
        Option("datagrid", "datagrid.seams=wide"),
    ),
    (
        "widgets::grid::DataGrid::track_click",
        Option("datagrid", "datagrid.track_click"),
    ),
    // Columns and Basic table (SC11)
    (
        "core::selection::SelectionMode",
        Option("basictable", "basictable.selection"),
    ),
    (
        "core::selection::SelectionMode::None",
        Option("basictable", "basictable.selection=none"),
    ),
    (
        "core::selection::SelectionMode::Single",
        Option("basictable", "basictable.selection=single"),
    ),
    (
        "core::selection::SelectionMode::Multi",
        Option("basictable", "basictable.selection=multi"),
    ),
    (
        "widgets::table::DataTable::selection_mode",
        Option("basictable", "basictable.selection"),
    ),
    (
        "widgets::table::DataTable::selection_column",
        Option("basictable", "basictable.selection=multi"),
    ),
    (
        "core::selection::RowSelection",
        Demo("basictable", "2 selected · header mixed"),
    ),
    (
        "core::selection::TriState",
        Demo("basictable", "2 selected · header mixed"),
    ),
    (
        "core::selection::TriState::Mixed",
        Demo("basictable", "[–] Reviewer"),
    ),
    (
        "core::selection::TriState::None",
        Demo("basictable", "[ ] ID Task Status"),
    ),
    (
        "core::selection::TriState::All",
        Demo("basictable", "[✓] ID Task Status"),
    ),
    (
        "ui::columns::Col",
        Demo("columns", "Column Rule min ideal max flex drop"),
    ),
    (
        "ui::columns::Col::flex",
        Demo("columns", "content 8–18 · flex 1, max 30"),
    ),
    (
        "ui::columns::Col::max",
        Demo("columns", "content 6–24 · max 40 · drop 0"),
    ),
    (
        "ui::columns::Col::droppable",
        Demo("columns", "content 6–10 · droppable 2"),
    ),
    (
        "ui::columns::ColumnSizing",
        Option("columns", "columns.min_width"),
    ),
    (
        "widgets::table::DataTable::resizable_columns",
        Option("columns", "columns.resizable"),
    ),
    (
        "widgets::table::DataTable::seam_hit_w",
        Option("columns", "columns.seam_w"),
    ),
    (
        "ui::columns::HeaderLabels",
        Option("columns", "columns.header_labels"),
    ),
    (
        "ui::columns::HeaderAction",
        Demo("columns", "Header menu · ID"),
    ),
    (
        "ui::columns::HeaderAction::SortAsc",
        Demo("columns", "Sort Ascending available"),
    ),
    (
        "ui::columns::HeaderAction::SortDesc",
        Demo("columns", "Sort Descending available"),
    ),
    (
        "ui::columns::HeaderAction::ClearSort",
        Demo("columns", "Clear Sort Not sorted"),
    ),
    (
        "ui::columns::HeaderAction::Autofit",
        Demo("columns", "Size Column to Fit available"),
    ),
    (
        "ui::columns::HeaderAction::ResetWidths",
        Demo(
            "columns",
            "Reset Column Widths Columns have their default widths",
        ),
    ),
    (
        "widgets::grid::DataGrid",
        Demo("datagrid", "of 40 loaded · ~4,812 total"),
    ),
    (
        "widgets::option_editor::ColumnEditor",
        Demo(
            "celleditors",
            "Columns that only take values from a fixed set",
        ),
    ),
    (
        "widgets::option_editor::ColumnEditor::Select",
        Demo("celleditors", "▾ Select Env"),
    ),
    (
        "widgets::option_editor::ColumnEditor::Combo",
        Demo("celleditors", "/ Combo Region"),
    ),
    (
        "widgets::option_editor::ColumnEditor::Segmented",
        Demo("celleditors", "│ Segmented Rollout"),
    ),
    (
        "widgets::option_editor::ColumnEditor::Checkbox",
        Demo("celleditors", "[✓] Checkbox Notify"),
    ),
    (
        "widgets::option_editor::ColumnEditor::Tags",
        Demo("celleditors", "[✓] Tags Labels"),
    ),
    (
        "widgets::option_editor::ColumnEditor::Radio",
        Demo("celleditors", "(●) Radio Priority"),
    ),
    (
        "widgets::list::ListBox",
        Demo("lists", "Language 1–13 of 20"),
    ),
    (
        "widgets::list::ListBox::mode",
        Demo("lists", "Files to include 2 selected"),
    ),
    (
        "widgets::list::ListBox::empty_text",
        Demo("lists", "No results for “retry”"),
    ),
    (
        "widgets::list::SelectMode",
        Demo("lists", "Files to include 2 selected"),
    ),
    (
        "widgets::list::SelectMode::Single",
        Demo("lists", "Chosen: Rust"),
    ),
    (
        "widgets::list::SelectMode::Multi",
        Demo("lists", "Files to include 2 selected"),
    ),
    (
        "widgets::tree::TreeView",
        Demo("trees", "Project 1–15 of 16"),
    ),
    ("widgets::tree::TreeNode", Demo("trees", "config.rs 1.9 KB")),
    (
        "widgets::tabs::Tabs",
        Demo("settings", "General Members Environment"),
    ),
    (
        "widgets::dialog::Dialog",
        Demo("dialogs", "Confirm run Rename task…"),
    ),
    (
        "widgets::picker::Picker",
        Demo("pickers", "Open quickly Switch tab Choose a level"),
    ),
    (
        "widgets::progress::ProgressStatus",
        Demo("progress", "Completed ━"),
    ),
    (
        "widgets::progress::ProgressStatus::Done",
        Demo("progress", "Completed ━"),
    ),
    (
        "widgets::progress::ProgressStatus::Error",
        Demo("progress", "Failed ━"),
    ),
    (
        "widgets::progress::ProgressStatus::Paused",
        Demo("progress", "Paused ━"),
    ),
    (
        "widgets::progress::Meter",
        Demo("progress", "Capacity meters are never green"),
    ),
    (
        "widgets::progress::MeterLevel",
        Demo("progress", "low ≤ 59 % blue"),
    ),
    (
        "widgets::progress::MeterLevel::Low",
        Demo("progress", "Low ━"),
    ),
    (
        "widgets::progress::MeterLevel::Medium",
        Demo("progress", "Medium ━"),
    ),
    (
        "widgets::progress::MeterLevel::High",
        Demo("progress", "High ━"),
    ),
    (
        "widgets::progress::MeterVisual",
        Demo("progress", "38% used · block"),
    ),
    (
        "widgets::progress::MeterVisual::Line",
        Demo("progress", "Low ━"),
    ),
    (
        "widgets::progress::MeterVisual::Block",
        Demo("progress", "38% used · block"),
    ),
    (
        "widgets::progress::MeterTone",
        Demo("progress", "Exhausted ━"),
    ),
    (
        "widgets::progress::MeterTone::Warning",
        Demo("progress", "Warning ━"),
    ),
    (
        "widgets::progress::MeterTone::Exhausted",
        Demo("progress", "Exhausted ━"),
    ),
    (
        "widgets::progress::MeterTone::Stale",
        Demo("progress", "Stale ╍"),
    ),
    (
        "widgets::progress::MeterTone::Refreshing",
        Demo("progress", "⠋ refreshing"),
    ),
    (
        "widgets::progress::MeterTone::Error",
        Demo("progress", "quota read failed !"),
    ),
    (
        "widgets::progress::MeterTone::Unknown",
        Demo("progress", "Unknown —"),
    ),
    (
        "widgets::progress::ProgressStatus::Active",
        Demo("progress", "Halfway ━"),
    ),
    (
        "widgets::progress::MeterTone::Normal",
        Demo("progress", "Low ━"),
    ),
    (
        "widgets::progress::MeterTone::Level",
        Demo("progress", "72% disk, warns at 80"),
    ),
    (
        "widgets::progress::MeterTone::Fixed",
        Demo("progress", "30% · fixed high !"),
    ),
    (
        "widgets::progress::Meter::value",
        Demo("progress", "38% used"),
    ),
    (
        "widgets::progress::Meter::tone",
        Demo("progress", "88% disk, critical at 95 ▲"),
    ),
    (
        "widgets::progress::Meter::visual",
        Demo("progress", "38% used · block"),
    ),
    (
        "widgets::progress::Meter::stale",
        Demo("progress", "82% · stale warning ▲"),
    ),
    (
        "widgets::progress::Meter::stale_style",
        Demo("progress", "76% · faint stale style"),
    ),
    (
        "widgets::progress::StaleStyle",
        Demo("progress", "Stale keeps its level, dashed"),
    ),
    (
        "widgets::progress::StaleStyle::Dashed",
        Demo("progress", "Stale ╍"),
    ),
    (
        "widgets::progress::StaleStyle::Faint",
        Demo("progress", "76% · faint stale style"),
    ),
    (
        "theme::SyntaxTone::Literal",
        Demo("codeeditor", "gzip(true)"),
    ),
    ("widgets::panel::Panel", Demo("panels", "Titled card")),
    (
        "widgets::panel::PanelKind",
        Demo("panels", "Framed · split pane"),
    ),
    (
        "widgets::panel::PanelKind::Card",
        Demo("panels", "Titled card"),
    ),
    (
        "widgets::panel::PanelKind::Framed",
        Demo("panels", "╭─ Framed · split pane ─╮"),
    ),
    (
        "widgets::panel::ScrollPanel",
        Demo("scrolling", "Wrapped text"),
    ),
    (
        "widgets::code::CodeEditor",
        Demo("codeeditor", "retry.rs 3 blocks"),
    ),
    (
        "widgets::diff::DiffView",
        Demo("diff", "M config/service.toml +10 −5"),
    ),
    (
        "widgets::diff::DiffMode",
        Demo("diff", "Unified or old / new review"),
    ),
    (
        "widgets::diff::DiffMode::Unified",
        Demo("diff", "@@ -1,6 +1,7 @@"),
    ),
    ("widgets::diff::DiffMode::Review", Demo("diff", "○ Review")),
    (
        "widgets::viewport::TextViewport",
        Demo("terminal", "payments-platform ❯ jackin launch"),
    ),
    (
        "widgets::steps::StepRail",
        Demo("terminal", "Step rail 0 of 7 · running"),
    ),
    (
        "widgets::steps::Step",
        Demo("terminal", "01 Resolve workspace"),
    ),
    (
        "widgets::steps::StepState",
        Demo("terminal", "02 Pull base image queued"),
    ),
    (
        "widgets::steps::StepState::Running",
        Demo("terminal", "⠋ 01 Resolve workspace"),
    ),
    (
        "widgets::steps::StepState::Queued",
        Demo("terminal", "02 Pull base image queued"),
    ),
    (
        "widgets::splitter::Splitter",
        Demo("terminal", "drag the seam"),
    ),
    // States grids (SC2): rows the page's States section draws
    ("ui::ctx::VisualState", StateRow("buttons", "focused")),
    ("state::rest", StateRow("buttons", "rest")),
    ("state::hover", StateRow("buttons", "hover")),
    ("state::pressed", StateRow("buttons", "pressed")),
    ("state::focused", StateRow("buttons", "focused")),
    ("state::focused_hover", StateRow("buttons", "focused_hover")),
    ("state::selected", StateRow("buttons", "selected")),
    ("state::disabled", StateRow("buttons", "disabled")),
    ("state::busy", StateRow("buttons", "busy")),
    ("state::error", StateRow("inputs", "error")),
    ("state::empty", StateRow("inputs", "empty")),
    ("state::overflow", StateRow("buttons", "overflow")),
    ("state::narrow", StateRow("inputs", "narrow")),
    ("ui::specimen::SpecimenState", StateRow("buttons", "rest")),
    (
        "ui::specimen::SpecimenState::Rest",
        StateRow("buttons", "rest"),
    ),
    (
        "ui::specimen::SpecimenState::Hover",
        StateRow("buttons", "hover"),
    ),
    (
        "ui::specimen::SpecimenState::Pressed",
        StateRow("buttons", "pressed"),
    ),
    (
        "ui::specimen::SpecimenState::Focused",
        StateRow("buttons", "focused"),
    ),
    (
        "ui::specimen::SpecimenState::FocusedHover",
        StateRow("buttons", "focused_hover"),
    ),
    (
        "ui::specimen::SpecimenState::Selected",
        StateRow("buttons", "selected"),
    ),
    (
        "ui::specimen::SpecimenState::Disabled",
        StateRow("buttons", "disabled"),
    ),
    (
        "ui::specimen::SpecimenState::Busy",
        StateRow("buttons", "busy"),
    ),
    (
        "ui::specimen::SpecimenState::Error",
        StateRow("inputs", "error"),
    ),
    (
        "ui::specimen::SpecimenState::Editing",
        StateRow("inputs", "editing"),
    ),
    (
        "ui::specimen::SpecimenState::Empty",
        StateRow("inputs", "empty"),
    ),
    (
        "ui::specimen::SpecimenState::Overflow",
        StateRow("buttons", "overflow"),
    ),
    (
        "ui::specimen::SpecimenState::Narrow",
        StateRow("inputs", "narrow"),
    ),
    (
        "ui::specimen::StateGrid",
        StateRow("buttons", "focused_hover"),
    ),
    (
        "ui::specimen::StateGrid::row",
        StateRow("buttons", "overflow"),
    ),
    ("ui::specimen::StateGrid::rows", StateRow("inputs", "rest")),
    (
        "ui::specimen::StateGrid::narrow_width",
        StateRow("inputs", "narrow"),
    ),
    (
        "ui::specimen::StateGrid::column",
        Demo("buttons", "Primary Secondary Subtle Danger Toggle"),
    ),
    (
        "ui::specimen::StateGrid::column_width",
        Demo("inputs", "Text Masked"),
    ),
    (
        "theme::StatusKind",
        Demo("overview", "✓ ok ✗ failed ▲ warn • pending"),
    ),
    ("theme::StatusKind::Ok", Demo("overview", "✓ ok")),
    ("theme::StatusKind::Fail", Demo("overview", "✗ failed")),
    ("theme::StatusKind::Warn", Demo("overview", "▲ warn")),
    ("theme::StatusKind::Pending", Demo("overview", "• pending")),
    ("theme::StatusKind::Running", Demo("overview", "⠋ running")),
    ("theme::StatusKind::Queued", Demo("overview", "○ queued")),
    ("theme::StatusKind::Info", Demo("overview", "i info")),
    ("theme::StatusKind::Skipped", Demo("overview", "– skipped")),
    ("theme::EnvKind", Demo("overview", "◆ production ◇ staging")),
    (
        "theme::EnvKind::Production",
        Demo("overview", "◆ production"),
    ),
    ("theme::EnvKind::Staging", Demo("overview", "◇ staging")),
    (
        "theme::EnvKind::Development",
        Demo("overview", "· development"),
    ),
    (
        "theme::EnvKind::Local",
        Demo("overview", "· development local"),
    ),
    (
        "theme::Tone",
        Demo("overview", "Tone normal secondary muted faint active"),
    ),
    ("theme::Tone::Normal", Demo("overview", "Tone normal")),
    (
        "theme::Tone::Secondary",
        Demo("overview", "normal secondary"),
    ),
    ("theme::Tone::Muted", Demo("overview", "secondary muted")),
    ("theme::Tone::Faint", Demo("overview", "muted faint")),
    ("theme::Tone::Active", Demo("overview", "faint active")),
    ("theme::Tone::Info", Demo("overview", "info success")),
    ("theme::Tone::Success", Demo("overview", "info success")),
    ("theme::Tone::Warning", Demo("overview", "success warning")),
    ("theme::Tone::Error", Demo("overview", "warning error")),
    ("theme::Tone::Danger", Demo("overview", "error danger")),
    (
        "theme::Theme::link",
        Demo("overview", "Link docs/colour.md"),
    ),
    (
        "theme::Theme::syntax_literal",
        Demo("overview", "let ok = true, 42, \"text\"; // note"),
    ),
    (
        "theme::Theme::syntax_keyword",
        Demo("overview", "let ok = true, 42, \"text\"; // note"),
    ),
    (
        "theme::Theme::syntax_string",
        Demo("overview", "let ok = true, 42, \"text\"; // note"),
    ),
    (
        "theme::SyntaxTone",
        Demo("overview", "let ok = true, 42, \"text\"; // note"),
    ),
    (
        "theme::SyntaxTone::Keyword",
        Demo("overview", "let ok = true, 42, \"text\"; // note"),
    ),
    (
        "theme::SyntaxTone::Ident",
        Demo("overview", "let ok = true, 42, \"text\"; // note"),
    ),
    (
        "theme::SyntaxTone::Number",
        Demo("overview", "let ok = true, 42, \"text\"; // note"),
    ),
    (
        "theme::SyntaxTone::Str",
        Demo("overview", "let ok = true, 42, \"text\"; // note"),
    ),
    (
        "theme::SyntaxTone::Operator",
        Demo("overview", "let ok = true, 42, \"text\"; // note"),
    ),
    (
        "theme::SyntaxTone::Punct",
        Demo("overview", "let ok = true, 42, \"text\"; // note"),
    ),
    (
        "theme::SyntaxTone::Comment",
        Demo("overview", "let ok = true, 42, \"text\"; // note"),
    ),
    (
        "theme::SyntaxTone::Plain",
        Demo("overview", "let ok = true, 42, \"text\"; // note"),
    ),
    (
        "widgets::dialog::Dialog::title_status",
        Demo("dialogs", "▲ title status"),
    ),
    (
        "widgets::list::ListItem::meta_tone",
        Demo("lists", "tests/checkout.rs new"),
    ),
    (
        "widgets::tree::TreeNode::glyph_tone",
        Demo("taskrunner", "◆ production"),
    ),
    (
        "widgets::tree::TreeNode::label_tone",
        Demo("taskrunner", "◆ production"),
    ),
    (
        "widgets::hintbar::HintLayer::status_parts",
        Demo("chrome", "last: nothing yet"),
    ),
    // ---- shell-wide options (the Inspector's Options tab, `--opt`; SC1)
    (
        "theme::ColorLevel::Ansi256",
        Option("overview", "shell.color=256"),
    ),
    (
        "theme::ColorLevel::Ansi16",
        Option("overview", "shell.color=16"),
    ),
    (
        "theme::ColorLevel::Mono",
        Option("overview", "shell.color=mono"),
    ),
    (
        "theme::LevelOverrides",
        Option("overview", "shell.overrides"),
    ),
    ("theme::Glyphs", Option("overview", "shell.glyphs")),
    ("theme::Theme::glyphs", Option("overview", "shell.glyphs")),
    (
        "theme::Theme::with_glyphs",
        Option("overview", "shell.glyphs"),
    ),
    ("theme::Strings", Option("overview", "shell.strings")),
    ("theme::Theme::strings", Option("overview", "shell.strings")),
    (
        "theme::Theme::with_strings",
        Option("overview", "shell.strings"),
    ),
    (
        "theme::Theme::edit_entry",
        Option("inputs", "shell.edit_entry"),
    ),
    (
        "theme::Theme::with_edit_entry",
        Option("inputs", "shell.edit_entry"),
    ),
    (
        "theme::Theme::field_keys",
        Option("inputs", "shell.field_keys"),
    ),
    (
        "theme::Theme::with_field_keys",
        Option("inputs", "shell.field_keys"),
    ),
    (
        "core::event::ClientPlatform",
        Option("inputs", "shell.platform"),
    ),
    (
        "core::event::ClientPlatform::MacOs",
        Option("inputs", "shell.platform=macos"),
    ),
    (
        "core::event::ClientPlatform::Other",
        Option("inputs", "shell.platform=other"),
    ),
    (
        "core::pointer::PointerConfig",
        Option("overview", "shell.wheel_lines"),
    ),
    (
        "core::pointer::PointerConfig::focus_on",
        Option("overview", "shell.focus_on"),
    ),
    (
        "core::pointer::PointerConfig::double_click_ms",
        Option("overview", "shell.double_click_ms"),
    ),
    (
        "core::pointer::PointerConfig::double_click_slop",
        Option("overview", "shell.double_click_slop"),
    ),
    (
        "core::pointer::PointerConfig::wheel_lines",
        Option("overview", "shell.wheel_lines"),
    ),
    (
        "core::pointer::PointerConfig::hover_refresh_after_scroll",
        Option("overview", "shell.hover_refresh_after_scroll"),
    ),
    (
        "core::pointer::PointerConfig::flash_ms",
        Option("overview", "shell.flash_ms"),
    ),
    (
        "core::pointer::PointerConfig::secondary_outside",
        Option("overview", "shell.secondary_outside"),
    ),
    (
        "core::pointer::PointerConfig::pointer_shapes",
        Option("overview", "shell.pointer_shapes"),
    ),
    (
        "core::pointer::PointerConfig::caret_shape",
        Option("overview", "shell.caret_shape"),
    ),
    (
        "core::pointer::PointerConfig::end_edit_on_dead_space",
        Option("overview", "shell.end_edit_on_dead_space"),
    ),
    (
        "core::pointer::PointerConfig::click_skips_animation",
        Option("overview", "shell.click_skips_animation"),
    ),
    (
        "core::pointer::PointerConfig::autoscroll",
        Option("overview", "shell.autoscroll"),
    ),
    (
        "core::pointer::PointerConfig::autoscroll_ms",
        Option("overview", "shell.autoscroll_ms"),
    ),
    (
        "core::pointer::PointerConfig::track_repeat",
        Option("overview", "shell.track_repeat"),
    ),
    (
        "core::pointer::PointerConfig::track_repeat_ms",
        Option("overview", "shell.track_repeat_ms"),
    ),
    (
        "core::pointer::FocusOn",
        Option("overview", "shell.focus_on"),
    ),
    (
        "core::pointer::FocusOn::Press",
        Option("overview", "shell.focus_on=press"),
    ),
    (
        "core::pointer::FocusOn::Release",
        Option("overview", "shell.focus_on=release"),
    ),
    (
        "core::pointer::SecondaryOutside",
        Option("overview", "shell.secondary_outside"),
    ),
    (
        "core::pointer::SecondaryOutside::Reopen",
        Option("overview", "shell.secondary_outside=reopen"),
    ),
    (
        "core::pointer::SecondaryOutside::Close",
        Option("overview", "shell.secondary_outside=close"),
    ),
    (
        "core::pointer::SecondaryOutside::Ignore",
        Option("overview", "shell.secondary_outside=ignore"),
    ),
    (
        "core::shape::CaretShape",
        Option("overview", "shell.caret_shape"),
    ),
    (
        "core::shape::CaretShape::Bar",
        Option("overview", "shell.caret_shape=bar"),
    ),
    (
        "core::shape::CaretShape::Block",
        Option("overview", "shell.caret_shape=block"),
    ),
    (
        "core::shape::CaretShape::Underline",
        Option("overview", "shell.caret_shape=underline"),
    ),
    (
        "core::shape::CaretShape::Terminal",
        Option("overview", "shell.caret_shape=terminal"),
    ),
];
