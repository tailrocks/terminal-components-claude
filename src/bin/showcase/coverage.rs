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
// (SC1) and States grids (SC2) land; until then only the gate does.
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

use Shown::Demo;

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
        Demo("inputs", "disabled jb_live_••••"),
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
    (
        "widgets::grid::DataGrid",
        Demo("datagrid", "rows 1–26 of 40 loaded"),
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
        Demo("progress", "low ≤ 59 % white"),
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
        Demo("progress", "Line and block visuals"),
    ),
    (
        "widgets::progress::MeterVisual::Line",
        Demo("progress", "38% used 38% used"),
    ),
    (
        "widgets::progress::MeterVisual::Block",
        Demo("progress", "38% used 38% used"),
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
        Demo("progress", "Stale ━"),
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
        Demo("progress", "Unknown — —"),
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
];
