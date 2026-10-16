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
    // SC3: Overview theme reference
    (
        "theme::Theme::field_hover",
        Demo("overview", "▏ field.hover #"),
    ),
    (
        "theme::Theme::highlight_danger",
        Demo("overview", "▏ highlight.danger #"),
    ),
    (
        "theme::Theme::error_soft",
        Demo("overview", "▏ error.soft #"),
    ),
    (
        "theme::Theme::text_ghost",
        Demo("overview", "▏ text.ghost #"),
    ),
    (
        "theme::Theme::scroll_track",
        Demo("overview", "▏ scroll.track #"),
    ),
    (
        "theme::Theme::text_on_accent",
        Demo("overview", "▏ text.on_accent #"),
    ),
    (
        "theme::Theme::accent_bg_subtle",
        Demo("overview", "▏ accent.bg.subtle #"),
    ),
    ("theme::Theme::focus", Demo("overview", "▏ focus #")),
    ("theme::Theme::disabled", Demo("overview", "▏ disabled #")),
    ("theme::Theme::error_bg", Demo("overview", "▏ error.bg #")),
    (
        "theme::Theme::error_strong",
        Demo("overview", "▏ error.strong #"),
    ),
    (
        "theme::Theme::success_strong",
        Demo("overview", "▏ success.strong #"),
    ),
    (
        "theme::Theme::accent_bg_hover",
        Demo("overview", "▏ accent.bg.hover #"),
    ),
    (
        "theme::Theme::level",
        Demo("overview", "theme.level truecolor"),
    ),
    (
        "theme::Theme::tint_selection",
        Demo("overview", "theme.tint_selection on"),
    ),
    (
        "theme::Theme::hintbar",
        Demo("overview", "theme.hintbar Start"),
    ),
    (
        "theme::Theme::spacing",
        Demo("overview", "theme.spacing button_gap 2"),
    ),
    (
        "theme::Spacing",
        Demo("overview", "theme.spacing button_gap 2"),
    ),
    ("theme::Insets", Demo("overview", "Insets { x: 2, y: 1 }")),
    (
        "theme::Theme::ladders",
        Demo("overview", "Ladders · theme.ladders"),
    ),
    (
        "theme::Ladders",
        Demo("overview", "Ladders · theme.ladders"),
    ),
    (
        "theme::Ladder",
        Demo("overview", "Ladder: rest hover pressed"),
    ),
    ("theme::Plane::up", Demo("overview", "Plane::up")),
    (
        "theme::Theme::mods",
        Demo("overview", "State modifiers · theme.mods"),
    ),
    (
        "theme::Interactive",
        Demo(
            "overview",
            "Theme::interactive(kind, state, plane) → Interactive",
        ),
    ),
    ("theme::BadgeKind", Demo("overview", "BadgeKind::Edit EDIT")),
    (
        "theme::BadgeKind::Edit",
        Demo("overview", "BadgeKind::Edit EDIT"),
    ),
    ("theme::StateModifiers", Option("overview", "overview.mods")),
    (
        "theme::Theme::with_state_modifiers",
        Option("overview", "overview.mods"),
    ),
    (
        "theme::Theme::with_ladder",
        Option("overview", "overview.ladder"),
    ),
    ("theme::Plane", Option("overview", "overview.plane")),
    (
        "theme::Plane::Canvas",
        Option("overview", "overview.plane=canvas"),
    ),
    (
        "theme::Plane::Surface",
        Option("overview", "overview.plane=surface"),
    ),
    (
        "theme::Plane::Card",
        Option("overview", "overview.plane=card"),
    ),
    (
        "theme::Plane::Field",
        Option("overview", "overview.plane=field"),
    ),
    (
        "theme::Plane::Raised",
        Option("overview", "overview.plane=raised"),
    ),
    (
        "theme::Plane::Popover",
        Option("overview", "overview.plane=popover"),
    ),
    (
        "theme::Plane::Highlight",
        Option("overview", "overview.plane=highlight"),
    ),
    (
        "theme::Plane::HighlightDanger",
        Option("overview", "overview.plane=highlight-danger"),
    ),
    (
        "theme::Plane::Accent",
        Option("overview", "overview.plane=accent"),
    ),
    (
        "theme::Plane::Danger",
        Option("overview", "overview.plane=danger"),
    ),
    ("theme::Kind", Option("overview", "overview.kind")),
    ("theme::Kind::Row", Option("overview", "overview.kind=row")),
    (
        "theme::Kind::CursorRow",
        Option("overview", "overview.kind=cursor-fill"),
    ),
    (
        "theme::Kind::Button",
        Option("overview", "overview.kind=primary"),
    ),
    ("theme::Kind::Tab", Option("overview", "overview.kind=tab")),
    (
        "theme::Kind::Title",
        Option("overview", "overview.kind=title"),
    ),
    (
        "theme::Kind::Chip",
        Option("overview", "overview.kind=chip"),
    ),
    (
        "theme::Kind::Close",
        Option("overview", "overview.kind=close"),
    ),
    (
        "theme::Kind::Field",
        Option("overview", "overview.kind=field"),
    ),
    (
        "theme::Kind::Handle",
        Option("overview", "overview.kind=handle"),
    ),
    (
        "theme::Kind::Seam",
        Option("overview", "overview.kind=seam"),
    ),
    ("theme::CursorStyle", Option("overview", "overview.kind")),
    (
        "theme::CursorStyle::Fill",
        Option("overview", "overview.kind=cursor-fill"),
    ),
    (
        "theme::CursorStyle::Bar",
        Option("overview", "overview.kind=cursor-bar"),
    ),
    (
        "theme::ButtonKind::Toggle",
        Option("overview", "overview.kind=toggle"),
    ),
    ("ui::specimen::KindSample", Demo("overview", "Cursor fill")),
    (
        "ui::specimen::KindSample::plane",
        Option("overview", "overview.plane"),
    ),
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
    // SC7 Buttons
    (
        "ui::ctx::FocusVisible",
        Option("buttons", "buttons.focus_visible"),
    ),
    (
        "ui::ctx::FocusVisible::Always",
        Option("buttons", "buttons.focus_visible=always"),
    ),
    (
        "ui::ctx::FocusVisible::KeyboardOnly",
        Option("buttons", "buttons.focus_visible=keyboard-only"),
    ),
    (
        "widgets::button::Button::focus_visible",
        Option("buttons", "buttons.focus_visible"),
    ),
    (
        "widgets::button::Button::focusable",
        Option("buttons", "buttons.toolbar_focus"),
    ),
    (
        "widgets::button::Button::tooltip",
        Demo("buttons", "hover a tool for its tooltip"),
    ),
    (
        "widgets::button::Button::shortcut",
        Demo("buttons", "F5 runs · Shift+F5 cancels"),
    ),
    ("widgets::button::ButtonRow", Demo("buttons", "Button row")),
    (
        "widgets::button::RowLayout",
        Demo("buttons", "columns · every button fits"),
    ),
    (
        "widgets::button::RowAlign",
        Option("buttons", "buttons.row_align"),
    ),
    (
        "widgets::button::RowAlign::Start",
        Option("buttons", "buttons.row_align=start"),
    ),
    (
        "widgets::button::RowAlign::End",
        Option("buttons", "buttons.row_align=end"),
    ),
    (
        "widgets::button::ButtonRow::align",
        Option("buttons", "buttons.row_align"),
    ),
    (
        "widgets::button::RowOverflow",
        Option("buttons", "buttons.row_overflow"),
    ),
    (
        "widgets::button::RowOverflow::More",
        Option("buttons", "buttons.row_overflow=more"),
    ),
    (
        "widgets::button::RowOverflow::Wrap",
        Option("buttons", "buttons.row_overflow=wrap"),
    ),
    (
        "widgets::button::ButtonRow::overflow",
        Option("buttons", "buttons.row_overflow"),
    ),
    (
        "widgets::button::ButtonRow::gap",
        Option("buttons", "buttons.row_gap"),
    ),
    (
        "widgets::button::ButtonRow::wrap",
        Option("buttons", "buttons.row_wrap"),
    ),
    (
        "widgets::icon_button::IconButton",
        Demo("buttons", "Icon buttons"),
    ),
    (
        "widgets::icon_button::IconLook",
        Demo("buttons", "Icon buttons"),
    ),
    (
        "widgets::icon_button::IconLook::Close",
        Demo("buttons", "Close ×"),
    ),
    (
        "widgets::icon_button::IconLook::Title",
        Demo("buttons", "Title · 5-cell plate"),
    ),
    (
        "widgets::icon_button::IconButton::plate",
        Demo("buttons", "Title · 5-cell plate"),
    ),
    (
        "widgets::icon_button::IconButton::pad",
        Option("buttons", "buttons.icon_pad"),
    ),
    (
        "widgets::icon_button::IconButton::danger_hover",
        Option("buttons", "buttons.icon_danger"),
    ),
    (
        "widgets::icon_button::IconButton::tip",
        Demo("buttons", "hover a plate for its tooltip"),
    ),
    (
        "ui::specimen::StateGrid::label_width",
        Option("buttons", "buttons.label_width"),
    ),
    (
        "ui::specimen::StateGrid::focus_mark",
        Option("buttons", "buttons.focus_mark"),
    ),
    // SC9 Checks & radios
    (
        "widgets::choice::CheckHit",
        Option("checksradios", "checksradios.hit"),
    ),
    (
        "widgets::choice::CheckHit::Label",
        Option("checksradios", "checksradios.hit=label"),
    ),
    (
        "widgets::choice::CheckHit::Row",
        Option("checksradios", "checksradios.hit=row"),
    ),
    (
        "widgets::choice::Checkbox::hit",
        Option("checksradios", "checksradios.hit"),
    ),
    (
        "widgets::choice::Checkbox::mixed",
        Option("checksradios", "checksradios.mixed"),
    ),
    (
        "widgets::choice::RadioGroup::nav",
        Option("checksradios", "checksradios.radio_nav"),
    ),
    (
        "widgets::choice::Toggle::disabled",
        Option("checksradios", "checksradios.disabled"),
    ),
    // SC8 Inputs and Text areas
    (
        "core::edit::EditEntry",
        Option("inputs", "inputs.edit_entry"),
    ),
    (
        "core::edit::EditEntry::Immediate",
        Option("inputs", "inputs.edit_entry=immediate"),
    ),
    (
        "core::edit::EditEntry::TypeToEdit",
        Option("inputs", "inputs.edit_entry=type-to-edit"),
    ),
    (
        "core::edit::EditEntry::Explicit",
        Option("inputs", "inputs.edit_entry=explicit"),
    ),
    (
        "core::edit::FocusSelect",
        Option("inputs", "inputs.focus_select"),
    ),
    (
        "core::edit::FocusSelect::SelectAll",
        Option("inputs", "inputs.focus_select=select-all"),
    ),
    (
        "core::edit::FocusSelect::CaretEnd",
        Option("inputs", "inputs.focus_select=caret-end"),
    ),
    (
        "core::edit::FocusSelect::CaretStart",
        Option("inputs", "inputs.focus_select=caret-start"),
    ),
    (
        "core::edit::FieldKeys",
        Option("inputs", "inputs.field_keys"),
    ),
    (
        "core::edit::FieldKeys::Emacs",
        Option("inputs", "inputs.field_keys=emacs"),
    ),
    (
        "core::edit::FieldKeys::Desktop",
        Option("inputs", "inputs.field_keys=desktop"),
    ),
    (
        "core::edit::TextCmd",
        Option("inputs", "inputs.text_keymap"),
    ),
    (
        "core::edit::TextCmd::SelectAll",
        Demo("inputs", "Select all Ctrl+A"),
    ),
    ("core::edit::TextCmd::Copy", Demo("inputs", "Copy Ctrl+C")),
    ("core::edit::TextCmd::Cut", Demo("inputs", "Cut Ctrl+X")),
    ("core::edit::TextCmd::Paste", Demo("inputs", "Paste Ctrl+V")),
    ("core::edit::TextCmd::Undo", Demo("inputs", "Undo Ctrl+Z")),
    ("core::edit::TextCmd::Redo", Demo("inputs", "Redo Ctrl+Y")),
    (
        "core::edit::TextCmd::LineStart",
        Demo("inputs", "Line start Cmd chord"),
    ),
    (
        "core::edit::TextCmd::LineEnd",
        Demo("inputs", "Line end Cmd chord"),
    ),
    (
        "core::edit::TextCmd::DocStart",
        Demo("inputs", "Text start Cmd chord"),
    ),
    (
        "core::edit::TextCmd::DocEnd",
        Demo("inputs", "Text end Cmd chord"),
    ),
    (
        "core::edit::TextCmd::SelectToLineStart",
        Demo("inputs", "Select to line start Cmd chord"),
    ),
    (
        "core::edit::TextCmd::SelectToLineEnd",
        Demo("inputs", "Select to line end Cmd chord"),
    ),
    (
        "core::edit::TextCmd::SelectToDocStart",
        Demo("inputs", "Select to text start Cmd chord"),
    ),
    (
        "core::edit::TextCmd::SelectToDocEnd",
        Demo("inputs", "Select to text end Cmd chord"),
    ),
    ("core::edit::EditKind", Demo("inputs", "(Typing)")),
    ("core::edit::EditKind::Typing", Demo("inputs", "(Typing)")),
    (
        "core::edit::EditKind::Deleting",
        Demo("inputs", "(Deleting)"),
    ),
    ("core::edit::EditKind::Other", Demo("inputs", "(Other)")),
    ("core::edit::UndoStack", Demo("inputs", "Undo history")),
    (
        "core::focus::BlurAction",
        Option("inputs", "inputs.blur_action"),
    ),
    (
        "core::focus::BlurAction::Commit",
        Option("inputs", "inputs.blur_action=commit"),
    ),
    (
        "core::focus::BlurAction::Cancel",
        Option("inputs", "inputs.blur_action=cancel"),
    ),
    (
        "core::focus::BlurAction::Keep",
        Option("inputs", "inputs.blur_action=keep"),
    ),
    (
        "widgets::field_common::PointerSelect",
        Option("inputs", "inputs.pointer_select"),
    ),
    (
        "widgets::field_common::PointerSelect::drag",
        Option("inputs", "inputs.pointer_select=no-drag"),
    ),
    (
        "widgets::field_common::PointerSelect::word",
        Option("inputs", "inputs.pointer_select=no-word"),
    ),
    (
        "widgets::field_common::PointerSelect::line",
        Option("inputs", "inputs.pointer_select=no-line"),
    ),
    (
        "widgets::field_common::PointerSelect::drag_by_word",
        Option("inputs", "inputs.pointer_select=by-word"),
    ),
    (
        "widgets::input::TextValidator",
        Demo("inputs", "Enter a valid email address"),
    ),
    (
        "widgets::input::TextInput::required_message",
        Option("inputs", "inputs.required_message"),
    ),
    (
        "widgets::input::TextInput::plain_label",
        Option("inputs", "inputs.plain_label"),
    ),
    (
        "widgets::input::TextInput::wrap_label",
        Option("inputs", "inputs.wrap_label"),
    ),
    (
        "widgets::input::TextInput::blur_action",
        Option("inputs", "inputs.blur_action"),
    ),
    (
        "widgets::input::TextInput::edit_entry",
        Option("inputs", "inputs.edit_entry"),
    ),
    (
        "widgets::input::TextInput::focus_select",
        Option("inputs", "inputs.focus_select"),
    ),
    (
        "widgets::input::TextInput::text_keymap",
        Option("inputs", "inputs.text_keymap"),
    ),
    (
        "widgets::input::TextInput::field_keys",
        Option("inputs", "inputs.field_keys"),
    ),
    (
        "widgets::input::TextInput::pointer_select",
        Option("inputs", "inputs.pointer_select"),
    ),
    (
        "widgets::input::TextInput::label_hit",
        Option("inputs", "inputs.label_hit"),
    ),
    (
        "widgets::textarea::TextArea::edit_entry",
        Option("textareas", "textareas.edit_entry"),
    ),
    (
        "widgets::textarea::TextArea::focus_select",
        Option("textareas", "textareas.focus_select"),
    ),
    (
        "widgets::textarea::TextArea::esc_reverts",
        Option("textareas", "textareas.esc_reverts"),
    ),
    (
        "widgets::textarea::TextArea::text_keymap",
        Option("textareas", "textareas.text_keymap"),
    ),
    (
        "widgets::textarea::TextArea::field_keys",
        Option("textareas", "textareas.field_keys"),
    ),
    (
        "widgets::textarea::TextArea::nav",
        Option("textareas", "textareas.nav"),
    ),
    (
        "widgets::textarea::TextArea::pointer_select",
        Option("textareas", "textareas.pointer_select"),
    ),
    (
        "widgets::textarea::TextArea::blur_action",
        Option("textareas", "textareas.blur_action"),
    ),
    (
        "widgets::textarea::TextArea::horizontal",
        Option("textareas", "textareas.horizontal"),
    ),
    (
        "widgets::textarea::TextArea::label_hit",
        Option("textareas", "textareas.label_hit"),
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
    // Tabs (SC14)
    ("widgets::tabs::TabItem", Demo("tabs", "◆ Overview")),
    (
        "widgets::tabs::TabItem::closable",
        Demo("tabs", "Import ⠋ ×"),
    ),
    (
        "widgets::tabs::TabItem::prefix",
        Demo("tabs", "▷ orders.sql"),
    ),
    (
        "widgets::tabs::TabItem::suffix",
        Demo("tabs", "▦ users ● ×"),
    ),
    ("widgets::tabs::TabItem::done", Demo("tabs", "Export ✓ ×")),
    (
        "widgets::tabs::TabItem::dirty",
        Demo("tabs", "orders.sql • ×"),
    ),
    ("widgets::tabs::TabItem::busy", Demo("tabs", "Import ⠋")),
    ("widgets::tabs::TabItem::error", Demo("tabs", "Migrate ! ×")),
    (
        "widgets::tabs::CloseReveal",
        Option("tabs", "tabs.close_reveal"),
    ),
    (
        "widgets::tabs::CloseReveal::Always",
        Option("tabs", "tabs.close_reveal=always"),
    ),
    (
        "widgets::tabs::CloseReveal::Hover",
        Option("tabs", "tabs.close_reveal=hover"),
    ),
    ("widgets::tabs::TabRule", Option("tabs", "tabs.rule")),
    (
        "widgets::tabs::TabRule::Auto",
        Option("tabs", "tabs.rule=auto"),
    ),
    (
        "widgets::tabs::TabRule::Accent",
        Option("tabs", "tabs.rule=accent"),
    ),
    (
        "widgets::tabs::TabRule::Neutral",
        Option("tabs", "tabs.rule=neutral"),
    ),
    (
        "widgets::tabs::TabRule::FocusOnly",
        Option("tabs", "tabs.rule=focus-only"),
    ),
    ("widgets::tabs::TabCmd", Option("tabs", "tabs.keymap")),
    ("widgets::tabs::TabCmd::Close", Demo("tabs", "x close")),
    ("widgets::tabs::TabCmd::New", Demo("tabs", "n new tab")),
    ("widgets::tabs::TabCmd::Jump", Demo("tabs", "1–9 jump")),
    (
        "widgets::tabs::TabAction",
        Demo("tabs", "TabAction::Activated · Close · New"),
    ),
    (
        "widgets::tabs::TabAction::Activated",
        Demo("tabs", "TabAction::Activated"),
    ),
    (
        "widgets::tabs::TabAction::Close",
        Demo("tabs", "Activated · Close"),
    ),
    ("widgets::tabs::TabAction::New", Demo("tabs", "Close · New")),
    ("widgets::tabs::Tabs::rule", Option("tabs", "tabs.rule")),
    ("widgets::tabs::Tabs::wrap", Option("tabs", "tabs.wrap")),
    ("widgets::tabs::Tabs::nav", Option("tabs", "tabs.nav")),
    ("widgets::tabs::Tabs::keymap", Option("tabs", "tabs.keymap")),
    (
        "widgets::tabs::Tabs::digit_jump",
        Option("tabs", "tabs.digit_jump"),
    ),
    (
        "widgets::tabs::Tabs::close_keys",
        Option("tabs", "tabs.close_keys"),
    ),
    (
        "widgets::tabs::Tabs::new_key",
        Option("tabs", "tabs.new_key"),
    ),
    (
        "widgets::tabs::Tabs::allow_new",
        Option("tabs", "tabs.allow_new"),
    ),
    (
        "widgets::tabs::Tabs::overflow_menu",
        Option("tabs", "tabs.overflow_menu"),
    ),
    (
        "widgets::tabs::Tabs::more_button",
        Option("tabs", "tabs.more_button"),
    ),
    (
        "widgets::tabs::Tabs::close_target_w",
        Option("tabs", "tabs.close_w"),
    ),
    (
        "widgets::tabs::Tabs::close_button",
        Option("tabs", "tabs.close_button"),
    ),
    (
        "widgets::tabs::Tabs::close_reveal",
        Option("tabs", "tabs.close_reveal"),
    ),
    (
        "widgets::tabs::Tabs::middle_click_closes",
        Option("tabs", "tabs.middle_click"),
    ),
    // Menus (SC16)
    (
        "ui::context::ContextKeys",
        Option("menus", "menus.shift_f10"),
    ),
    (
        "ui::context::ContextKeys::shift_f10",
        Option("menus", "menus.shift_f10"),
    ),
    (
        "ui::context::ContextKeys::menu_key",
        Option("menus", "menus.menu_key"),
    ),
    (
        "ui::context::ContextKeys::extra",
        Option("menus", "menus.extra_key"),
    ),
    (
        "ui::context::ContextConfig",
        Demo("menus", "a click moves the cursor"),
    ),
    (
        "ui::context::ContextConfig::keys",
        Option("menus", "menus.shift_f10"),
    ),
    (
        "ui::context::ContextConfig::moves_cursor",
        Option("menus", "menus.moves_cursor"),
    ),
    (
        "ui::context::ContextAnchor",
        Demo("menus", "Pointer at a right-click, Control under the row"),
    ),
    (
        "ui::context::ContextAnchor::Pointer",
        Demo("menus", "Pointer at a right-click"),
    ),
    (
        "ui::context::ContextAnchor::Control",
        Demo("menus", "Control under the row for a key"),
    ),
    (
        "ui::context::ContextRequest",
        Demo("menus", "Request none yet"),
    ),
    (
        "ui::context::ContextRequest::map",
        Demo("menus", "Request none yet"),
    ),
    ("widgets::menu::MenuItem", Demo("menus", "Open    Enter")),
    (
        "widgets::menu::MenuItem::shortcut",
        Demo("menus", "Rename     F2"),
    ),
    (
        "widgets::menu::MenuItem::disabled",
        Demo("menus", "Share…    Off"),
    ),
    (
        "widgets::menu::MenuItem::reason",
        Demo("menus", "Share…    Off"),
    ),
    ("widgets::menu::MenuItem::danger", Demo("menus", "Delete")),
    ("widgets::menu::MenuItem::danger_if", Demo("menus", "Trash")),
    (
        "widgets::menu::MenuItem::separator",
        Demo("menus", "Copy path"),
    ),
    (
        "widgets::menu::MenuItem::separator_if",
        Demo("menus", "Share…    Off"),
    ),
    (
        "widgets::menu::MenuAction",
        Demo("menus", "Chosen(i) or Dismissed"),
    ),
    (
        "widgets::menu::MenuAction::Chosen",
        Demo("menus", "Chosen(i)"),
    ),
    (
        "widgets::menu::MenuAction::Dismissed",
        Demo("menus", "or Dismissed"),
    ),
    (
        "widgets::menu::ContextMenu::release_selects",
        Option("menus", "menus.release_selects"),
    ),
    (
        "widgets::menu::ContextMenu::max_rows",
        Option("menus", "menus.max_rows"),
    ),
    (
        "widgets::menu::ContextMenu::min_width",
        Option("menus", "menus.min_width"),
    ),
    (
        "widgets::menu::ContextMenu::nav",
        Option("menus", "menus.nav"),
    ),
    (
        "widgets::menu::ContextMenu::edge_margin",
        Option("menus", "menus.edge_margin"),
    ),
    (
        "widgets::menu::ContextMenu::wrap",
        Option("menus", "menus.wrap"),
    ),
    (
        "widgets::menu::ContextMenu::type_ahead",
        Option("menus", "menus.type_ahead"),
    ),
    (
        "widgets::menu::ContextMenu::fit",
        Option("menus", "menus.fit"),
    ),
    (
        "widgets::menu::ContextMenu::dismiss",
        Option("menus", "menus.dismiss"),
    ),
    (
        "widgets::menu::ContextMenu::hide_owner_focus",
        Option("menus", "menus.hide_owner_focus"),
    ),
    (
        "widgets::menu::ContextMenu::anchor",
        Option("menus", "menus.placement"),
    ),
    (
        "widgets::menu::ContextMenu::at",
        Demo("menus", "Pointer at a right-click"),
    ),
    (
        "widgets::menu::ContextMenu::anchor_control",
        Demo("menus", "Actions ▾"),
    ),
    (
        "widgets::menu::ContextMenu::title",
        Option("menus", "menus.title"),
    ),
    (
        "widgets::menu::ContextMenu::unknown_key",
        Option("menus", "menus.unknown_key"),
    ),
    (
        "widgets::menu::UnknownKey",
        Option("menus", "menus.unknown_key"),
    ),
    (
        "widgets::menu::UnknownKey::CloseAndPass",
        Option("menus", "menus.unknown_key=close-and-pass"),
    ),
    (
        "widgets::menu::UnknownKey::Swallow",
        Option("menus", "menus.unknown_key=swallow"),
    ),
];
