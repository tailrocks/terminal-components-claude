//! Live options (plan §7, SCP3): one declaration per knob, read by the
//! Inspector's Options tab, a page's Options section and `--opt`.
//!
//! An option is a key (`shell.<name>` for the shell-wide ones,
//! `<page slug>.<name>` for a page's own), a label and a closed list of
//! values. Setting one rebuilds the configuration through the same builder
//! calls an application makes (`Theme::for_level_with`, `with_glyphs`,
//! `PointerConfig::focus_on`, ...); see `App::apply_shell`.
//!
//! The controls are library widgets: an `off | on` option is a `Toggle`,
//! any other a `Select` ([`OptionsView`]). Values live only for the
//! session; nothing is written to disk.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::app::PageId;
use crate::pages::layout::Sections;
use crate::pages::{Cx, Hint, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::core::intent::Phase;
use junie_tui::core::pointer::MouseTarget;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::layout::StackChild;
use junie_tui::widgets::choice::Toggle;
use junie_tui::widgets::select::{Select, SelectAction};

/// What setting an option changes, so the coverage gate knows what to
/// compare.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    /// The rendered frame (colours, glyphs, strings, a widget's look).
    Frame,
    /// Behaviour only (pointer timing, key presets, motion): the frame is
    /// the same until the user acts, the live configuration is not.
    Behaviour,
}

/// One live option.
#[derive(Debug, Clone, Copy)]
pub struct Opt {
    /// `shell.<name>` or `<page slug>.<name>`.
    pub key: &'static str,
    pub label: &'static str,
    /// The values `--opt` accepts and the control offers, in order.
    pub values: &'static [&'static str],
    /// Index into [`Self::values`] of the default.
    pub default: usize,
    pub effect: Effect,
    /// Options the effect needs first (`(key, value)`): level overrides
    /// only show at the level they override.
    pub needs: &'static [(&'static str, &'static str)],
}

impl Opt {
    const fn new(key: &'static str, label: &'static str, values: &'static [&'static str]) -> Self {
        Self {
            key,
            label,
            values,
            default: 0,
            effect: Effect::Frame,
            needs: &[],
        }
    }
    const fn default(mut self, i: usize) -> Self {
        self.default = i;
        self
    }
    const fn behaviour(mut self) -> Self {
        self.effect = Effect::Behaviour;
        self
    }
    const fn needs(mut self, needs: &'static [(&'static str, &'static str)]) -> Self {
        self.needs = needs;
        self
    }
    /// An `off | on` option, shown as a `Toggle`.
    pub fn is_toggle(&self) -> bool {
        self.values == OFF_ON
    }
    /// Index of `value` (any case).
    pub fn index_of(&self, value: &str) -> Option<usize> {
        self.values
            .iter()
            .position(|v| v.eq_ignore_ascii_case(value.trim()))
    }
}

const OFF_ON: &[&str] = &["off", "on"];

/// The shell-wide options, in Options-tab order.
pub static SHELL: &[Opt] = &[
    // appearance
    Opt::new(
        "shell.color",
        "Colour level",
        &["truecolor", "256", "16", "mono"],
    ),
    Opt::new(
        "shell.overrides",
        "Level overrides",
        &["none", "ascii-mono"],
    )
    .needs(&[("shell.color", "mono")]),
    Opt::new("shell.glyphs", "Glyphs", &["unicode", "ascii"]),
    // the copy shows where a widget says something (a picker's placeholder, a
    // "… N more" cut, the help footer), not on a page at rest
    Opt::new("shell.strings", "Strings", &["default", "ascii"]).behaviour(),
    Opt::new("shell.motion", "Motion", &["full", "reduced", "paused"]).behaviour(),
    // pointer
    // the library default (plan §3): a press focuses, as on macOS
    Opt::new("shell.focus_on", "Focus on", &["press", "release"]).behaviour(),
    Opt::new(
        "shell.double_click_ms",
        "Double-click ms",
        &["250", "500", "800"],
    )
    .default(1)
    .behaviour(),
    Opt::new(
        "shell.double_click_slop",
        "Double-click slop",
        &["0", "1", "2"],
    )
    .default(1)
    .behaviour(),
    Opt::new("shell.wheel_lines", "Wheel lines", &["1", "3", "5"])
        .default(1)
        .behaviour(),
    Opt::new(
        "shell.hover_refresh_after_scroll",
        "Re-hover on scroll",
        OFF_ON,
    )
    .default(1)
    .behaviour(),
    Opt::new("shell.flash_ms", "Press flash ms", &["70", "140", "280"])
        .default(1)
        .behaviour(),
    Opt::new(
        "shell.secondary_outside",
        "Right-click outside",
        &["reopen", "close", "ignore"],
    )
    .behaviour(),
    Opt::new("shell.end_edit_on_dead_space", "End edit on blank", OFF_ON)
        .default(1)
        .behaviour(),
    Opt::new("shell.autoscroll", "Edge autoscroll", OFF_ON)
        .default(1)
        .behaviour(),
    Opt::new("shell.autoscroll_ms", "Autoscroll ms", &["25", "50", "100"])
        .default(1)
        .behaviour(),
    Opt::new("shell.track_repeat", "Track repeat", OFF_ON)
        .default(1)
        .behaviour(),
    Opt::new(
        "shell.track_repeat_ms",
        "Track repeat ms",
        &["150/25", "300/50", "500/100"],
    )
    .default(1)
    .behaviour(),
    Opt::new("shell.track_click", "Track click", &["page", "jump"]).behaviour(),
    Opt::new("shell.pointer_shapes", "Pointer shapes", OFF_ON)
        .default(1)
        .behaviour(),
    Opt::new(
        "shell.caret_shape",
        "Caret shape",
        &["bar", "block", "underline", "terminal"],
    )
    .behaviour(),
    Opt::new("shell.click_skips_animation", "Click skips intro", OFF_ON)
        .default(1)
        .behaviour(),
    Opt::new("shell.tooltips", "Tooltips", OFF_ON)
        .default(1)
        .behaviour(),
    Opt::new("shell.reveal", "Reveal cut text", OFF_ON)
        .default(1)
        .behaviour(),
    // keys and editing
    Opt::new(
        "shell.platform",
        "Client platform",
        &["auto", "macos", "other"],
    )
    .behaviour(),
    Opt::new(
        "shell.field_keys",
        "Field keys",
        &["auto", "emacs", "desktop"],
    )
    .behaviour(),
    Opt::new(
        "shell.edit_entry",
        "Edit entry",
        &["immediate", "type-to-edit", "explicit"],
    )
    .behaviour(),
    // focus and input (SC28)
    Opt::new(
        "shell.focus_fallback",
        "Focus fallback",
        &["first", "nearest"],
    )
    // the library default (`FocusFallback::default()`)
    .default(1)
    .behaviour(),
    Opt::new(
        "shell.shift_wheel_horizontal",
        "Shift+wheel sideways",
        OFF_ON,
    )
    .default(1)
    .behaviour(),
    Opt::new(
        "shell.legacy_ctrl_punctuation",
        "Legacy Ctrl punctuation",
        OFF_ON,
    )
    .default(1)
    .behaviour(),
    // read once, when the terminal session starts; auto honours
    // JUNIE_TUI_KEYBOARD_PROTOCOL
    Opt::new(
        "shell.keyboard_protocol",
        "Keyboard protocol",
        &["auto", "on", "off"],
    )
    .behaviour(),
    // whether the terminal delivers Alt (Option) chords (A29); auto
    // honours JUNIE_TUI_ALT_KEYS and the terminal detection
    Opt::new(
        "shell.alt_keys",
        "Alt keys",
        &[
            "auto",
            "delivered",
            "refused",
            "until-seen",
            "with-protocol",
        ],
    )
    .behaviour(),
];

/// A page's own option table. Each page step (SC3 onward) adds its page's
/// arm with keys `<slug>.<name>`, and the page implements
/// `Page::option_value` and `Page::set_option` for them.
pub fn page_options(page: PageId) -> &'static [Opt] {
    match page {
        PageId::Columns => COLUMNS,
        PageId::Tables => BASIC_TABLE,
        PageId::Overview => OVERVIEW,
        PageId::Tabs => TABS,
        PageId::Menus => MENUS,
        PageId::Dialogs => DIALOGS,
        PageId::DialogKinds => DIALOG_KINDS,
        PageId::PopupsTooltips => POPUPS_TOOLTIPS,
        PageId::Pickers => PICKERS,
        PageId::Buttons => BUTTONS,
        PageId::Inputs => INPUTS,
        PageId::TextAreas => TEXTAREAS,
        PageId::ChecksRadios => CHECKS,
        PageId::Chips => CHIPS,
        PageId::Chrome => CHROME,
        PageId::StatusHints => STATUS_HINTS,
        PageId::TextUtilities => TEXT_UTILITIES,
        PageId::Grid => DATA_GRID,
        PageId::Editable => EDITABLE,
        PageId::Selectable => SELECTION,
        PageId::CellEditors => CELL_EDITORS,
        PageId::FocusNavigation => FOCUS_NAV,
        PageId::Toolbars => TOOLBARS,
        PageId::Progress => PROGRESS,
        PageId::EmptyError => EMPTY_ERROR,
        PageId::Meters => METERS,
        PageId::Notices => NOTICES,
        PageId::DiskCleanup => DISK_CLEANUP,
        PageId::Activity => ACTIVITY,
        PageId::Workspaces => WORKSPACES,
        PageId::FileBrowser => FILE_BROWSER,
        PageId::SplitsStacks => SPLITS_STACKS,
        PageId::MasterDetail => MASTER_DETAIL,
        PageId::PaneTree => PANE_TREE,
        PageId::BreadcrumbsRails => BREADCRUMBS_RAILS,
        PageId::Editor => CODE_EDITOR,
        PageId::HelpKeys => HELP_KEYS,
        PageId::QuitLeave => QUIT_LEAVE,
        PageId::Lists => LISTS,
        PageId::Trees => TREES,
        PageId::Panels => PANELS,
        PageId::Scrolling => SCROLLING,
        PageId::Diff => DIFF,
        PageId::Terminal => TERMINAL,
        PageId::QueryWorkbench => QUERY_WORKBENCH,
        PageId::Connections => CONNECTIONS,
        PageId::Finder => FINDER,
        PageId::ReadOnlyReport => READONLY_REPORT,
        PageId::LaunchCockpit => LAUNCH_COCKPIT,
        PageId::AccountsUsage => ACCOUNTS_USAGE,
        _ => &[],
    }
}

/// Query workbench (SC46): the screen's layout and the builders of its
/// tabs, editor and status line (`pages/query_workbench.rs`, `rebuild`).
pub static QUERY_WORKBENCH: &[Opt] = &[
    Opt::new("queryworkbench.explorer", "Explorer", OFF_ON).default(1),
    Opt::new(
        "queryworkbench.explorer_width",
        "Explorer %",
        &["32", "24", "40"],
    ),
    Opt::new(
        "queryworkbench.editor_share",
        "Editor %",
        &["40", "25", "60"],
    ),
    Opt::new("queryworkbench.allow_new", "New-tab button", OFF_ON).default(1),
    Opt::new(
        "queryworkbench.close_reveal",
        "Tab close",
        &["always", "hover"],
    ),
    Opt::new("queryworkbench.read_only", "Read-only editor", OFF_ON),
    Opt::new(
        "queryworkbench.result",
        "Result",
        &["run", "empty", "error", "plan"],
    ),
    Opt::new(
        "queryworkbench.env",
        "Environment",
        &["production", "staging", "development", "local"],
    ),
];

/// Connections (SC47): the list's grouping, meta and width, the empty
/// list, the selected connection's connect state and the form's width
/// (`pages/connections.rs`).
pub static CONNECTIONS: &[Opt] = &[
    Opt::new(
        "connections.group_by",
        "Group by",
        &["folder", "environment"],
    ),
    Opt::new("connections.meta", "Engine column", OFF_ON).default(1),
    Opt::new(
        "connections.list_width",
        "List share %",
        &["40", "30", "50"],
    ),
    Opt::new("connections.list", "List", &["sample", "empty"]),
    Opt::new(
        "connections.state",
        "Connect state",
        &["idle", "connecting", "connected", "failed"],
    ),
    Opt::new("connections.form_width", "Form width", &["60", "52", "72"]).behaviour(),
];

/// Finder (SC48): the query, the scope, the risk column and the preview
/// layout (`pages/finder.rs`).
pub static FINDER: &[Opt] = &[
    Opt::new("finder.query", "Query", &["typed", "empty", "nomatch"]),
    Opt::new("finder.scope", "Scope", &["here", "project", "everywhere"]),
    Opt::new("finder.risk_column", "Risk column", OFF_ON).default(1),
    Opt::new("finder.preview", "Preview", &["auto", "line"]),
];

/// Panels (SC22): `Panel` chrome and the `ScrollPanel` knobs.
pub static PANELS: &[Opt] = &[
    Opt::new("panels.focus_mark", "Card focus mark", &["title", "bar"]),
    Opt::new("panels.title_min", "Title keeps", &["12", "4", "20"]),
    Opt::new("panels.pad_x", "Card padding", &["2", "1", "0"]),
    Opt::new("panels.meta_short", "Short meta", &["auto", "custom"]),
    Opt::new("panels.wrap", "Wrap prose", OFF_ON).default(1),
    Opt::new("panels.tail", "Log follows its tail", OFF_ON),
    Opt::new(
        "panels.hscroll",
        "Log sideways",
        &["theme", "off", "bar-always", "bar-never", "step-1", "jump"],
    ),
    Opt::new(
        "panels.track_click",
        "Track click",
        &["app", "page", "jump"],
    ),
    Opt::new("panels.nav", "Keys", &["desktop", "vim", "no-paging"]),
    Opt::new("panels.keymap", "Follow key", &["f", "t"]),
];

/// Scrolling (SC22, SC29): `ScrollState`, `ScrollPane` and the cursor
/// model (`NavProfile`, type-ahead, its pause and precedence).
pub static SCROLLING: &[Opt] = &[
    Opt::new("scrolling.context", "Context rows", &["0", "1", "2"]).default(1),
    Opt::new("scrolling.follow", "Mirror follows", &["on-move", "always"]),
    Opt::new(
        "scrolling.track_click",
        "Track click",
        &["app", "page", "jump"],
    ),
    Opt::new(
        "scrolling.hbar",
        "Sideways bar",
        &["auto", "always", "never"],
    ),
    Opt::new("scrolling.columns", "Sideways by column", OFF_ON).default(1),
    Opt::new("scrolling.profile", "Profile", &["desktop", "vim"]),
    Opt::new(
        "scrolling.type_ahead",
        "Type-ahead",
        &["profile", "on", "off"],
    ),
    Opt::new(
        "scrolling.timeout",
        "Type-ahead pause",
        &["0.5s", "1s", "2s"],
    )
    .default(1),
    Opt::new(
        "scrolling.precedence",
        "Letters go to",
        &["shortcuts", "type-ahead"],
    ),
    Opt::new("scrolling.wrap", "Wrap at the ends", OFF_ON),
    Opt::new("scrolling.skip_disabled", "Skip disabled", OFF_ON).default(1),
];

/// Buttons (SC7): `Button`, `ButtonRow`, `IconButton` and the States grid.
pub static BUTTONS: &[Opt] = &[
    Opt::new(
        "buttons.focus_visible",
        "Focus ring",
        &["always", "keyboard-only"],
    ),
    Opt::new("buttons.toolbar_focus", "Toolbar in Tab order", OFF_ON),
    Opt::new("buttons.row_align", "Row align", &["start", "end"]),
    Opt::new("buttons.row_overflow", "Row overflow", &["more", "wrap"]),
    Opt::new("buttons.row_gap", "Row gap", &["theme", "1", "4"]),
    Opt::new("buttons.row_width", "Row width", &["full", "48", "30"]),
    Opt::new("buttons.row_wrap", "Arrows wrap in the row", OFF_ON),
    Opt::new("buttons.next_align", "Next row align", &["end", "start"]),
    Opt::new("buttons.next_gap", "Next row gap", &["1", "2"]),
    Opt::new("buttons.next_message", "Next row message", OFF_ON).default(1),
    Opt::new("buttons.icon_pad", "Icon plate pad", &["0", "1", "2"]).default(1),
    Opt::new("buttons.icon_danger", "Danger hover on ×", OFF_ON),
    Opt::new(
        "buttons.label_width",
        "States label column",
        &["15", "10", "20"],
    ),
    Opt::new(
        "buttons.focus_mark",
        "States focus mark",
        &["thin", "colon"],
    ),
];

/// Inputs (SC8): how a `TextInput` starts, ends and selects, and its keys.
pub static INPUTS: &[Opt] = &[
    Opt::new(
        "inputs.edit_entry",
        "Edit entry",
        &["theme", "immediate", "type-to-edit", "explicit"],
    ),
    Opt::new(
        "inputs.focus_select",
        "Focus selects",
        &["select-all", "caret-end", "caret-start"],
    ),
    Opt::new(
        "inputs.field_keys",
        "Field keys",
        &["theme", "emacs", "desktop"],
    ),
    Opt::new("inputs.text_keymap", "Text key map", &["preset", "custom"]),
    Opt::new(
        "inputs.blur_action",
        "On blur",
        &["commit", "cancel", "keep"],
    ),
    Opt::new(
        "inputs.pointer_select",
        "Pointer selects",
        &["default", "no-drag", "no-word", "no-line", "by-word", "off"],
    ),
    Opt::new("inputs.label_hit", "Label click focuses", OFF_ON).default(1),
    Opt::new("inputs.wrap_label", "Wrap long labels", OFF_ON),
    Opt::new("inputs.plain_label", "Hide \"optional\"", OFF_ON),
    Opt::new("inputs.readout", "Readout", OFF_ON).default(1),
    Opt::new(
        "inputs.required_message",
        "Required message",
        &["theme", "custom"],
    ),
];

/// Text areas (SC8): the `TextInput` knobs a `TextArea` shares, and its
/// own.
pub static TEXTAREAS: &[Opt] = &[
    Opt::new(
        "textareas.edit_entry",
        "Edit entry",
        &["theme", "immediate", "type-to-edit", "explicit"],
    ),
    Opt::new(
        "textareas.focus_select",
        "Focus selects",
        &["caret-end", "select-all", "caret-start"],
    ),
    Opt::new("textareas.esc_reverts", "Esc reverts", OFF_ON),
    Opt::new(
        "textareas.field_keys",
        "Field keys",
        &["theme", "emacs", "desktop"],
    ),
    Opt::new(
        "textareas.text_keymap",
        "Text key map",
        &["preset", "custom"],
    ),
    Opt::new(
        "textareas.nav",
        "Scroll keys",
        &["default", "vim", "no-page"],
    ),
    Opt::new(
        "textareas.pointer_select",
        "Pointer selects",
        &["default", "no-drag", "no-word", "no-line", "by-word", "off"],
    ),
    Opt::new(
        "textareas.blur_action",
        "On blur",
        &["commit", "cancel", "keep"],
    ),
    Opt::new(
        "textareas.horizontal",
        "Horizontal scroll",
        &["theme", "on", "off"],
    ),
    Opt::new("textareas.label_hit", "Label click focuses", OFF_ON).default(1),
];

/// Checks & radios (SC9): `Checkbox`, `RadioGroup` and `Toggle`.
pub static CHECKS: &[Opt] = &[
    Opt::new(
        "checksradios.hit",
        "Checkbox click target",
        &["label", "row"],
    ),
    Opt::new("checksradios.mixed", "\"Select all\" starts mixed", OFF_ON).default(1),
    Opt::new(
        "checksradios.radio_nav",
        "Radio keys",
        &["default", "vim", "no-home-end"],
    ),
    Opt::new("checksradios.disabled", "Disable every control", OFF_ON),
];

/// Chips & selects (SC10): every `ChipBar` and `Select` knob.
pub static CHIPS: &[Opt] = &[
    // chip bar
    Opt::new(
        "chipsselects.bar_width",
        "Chip bar width",
        &["full", "60", "40"],
    ),
    Opt::new("chipsselects.overflow_marker", "+N marker", OFF_ON).default(1),
    Opt::new("chipsselects.overflow_menu", "+N opens a menu", OFF_ON).default(1),
    Opt::new(
        "chipsselects.close_target",
        "× plate width",
        &["3", "1", "5"],
    ),
    Opt::new(
        "chipsselects.close_button",
        "× button",
        &["default", "danger"],
    ),
    Opt::new(
        "chipsselects.more_button",
        "+N button",
        &["default", "wide"],
    ),
    Opt::new(
        "chipsselects.chip_nav",
        "Chip keys",
        &["default", "vim", "no-home-end"],
    ),
    Opt::new(
        "chipsselects.chip_keymap",
        "Chip key map",
        &["default", "custom"],
    ),
    Opt::new("chipsselects.digit_toggles", "1–9 toggle streams", OFF_ON).default(1),
    // selects
    Opt::new("chipsselects.arrows", "Select arrows", &["open", "step"]),
    Opt::new(
        "chipsselects.wheel",
        "Wheel over the list",
        &["viewport", "cursor"],
    ),
    Opt::new(
        "chipsselects.tab_in_open",
        "Tab in an open list",
        &["choose", "close", "swallow"],
    ),
    Opt::new("chipsselects.max_rows", "List rows", &["8", "4", "12"]),
    Opt::new(
        "chipsselects.placement",
        "List placement",
        &["below", "above"],
    ),
    Opt::new(
        "chipsselects.popup_fit",
        "List fit",
        &["default", "flush", "menu"],
    ),
    Opt::new(
        "chipsselects.popup_width",
        "List width",
        &["12-40", "8-16", "30-60"],
    ),
    Opt::new("chipsselects.cursor_style", "List cursor", &["fill", "bar"]),
    Opt::new(
        "chipsselects.hover_moves_cursor",
        "Hover moves cursor",
        OFF_ON,
    )
    .default(1),
    Opt::new("chipsselects.press_drag", "Press-drag-release", OFF_ON).default(1),
    Opt::new("chipsselects.label_hit", "Label click opens", OFF_ON).default(1),
    Opt::new(
        "chipsselects.track_click",
        "List track click",
        &["theme", "jump", "page"],
    ),
    Opt::new("chipsselects.wrap", "Arrows wrap", OFF_ON),
    Opt::new("chipsselects.type_ahead", "Type-ahead", OFF_ON),
    Opt::new("chipsselects.close_on_blur", "Close on blur", OFF_ON).default(1),
    Opt::new("chipsselects.dismiss", "Dismiss", &["light", "escape"]),
    Opt::new(
        "chipsselects.select_nav",
        "Select keys",
        &["default", "vim"],
    ),
    Opt::new("chipsselects.mixed", "Sort shows mixed", OFF_ON),
];

/// A page's option values: an index into each option's values, in the
/// order of the page's table. The page reads them when it rebuilds its
/// widgets through their builders (`Page::set_option`).
#[derive(Debug, Clone)]
pub struct PageValues {
    opts: &'static [Opt],
    values: Vec<usize>,
}

impl PageValues {
    pub fn new(opts: &'static [Opt]) -> Self {
        Self {
            opts,
            values: opts.iter().map(|o| o.default).collect(),
        }
    }
    fn index(&self, key: &str) -> Option<usize> {
        self.opts.iter().position(|o| o.key == key)
    }
    /// The value index of `key`; `None` for a key the table does not have.
    pub fn get(&self, key: &str) -> Option<usize> {
        self.index(key).map(|i| self.values[i])
    }
    /// Set `key` to value index `value` (clamped); `false` for an unknown
    /// key.
    pub fn set(&mut self, key: &str, value: usize) -> bool {
        let Some(i) = self.index(key) else {
            return false;
        };
        self.values[i] = value.min(self.opts[i].values.len() - 1);
        true
    }
    /// The value of `key` as `--opt` spells it (`""` for an unknown key).
    pub fn value(&self, key: &str) -> &'static str {
        self.index(key)
            .map_or("", |i| self.opts[i].values[self.values[i]])
    }
    /// An `off | on` option is on.
    pub fn on(&self, key: &str) -> bool {
        self.value(key) == "on"
    }
    /// A numeric value (`None` for a word such as `theme` or `full`).
    pub fn number(&self, key: &str) -> Option<u16> {
        self.value(key).parse().ok()
    }
}

/// Cell editors (SC13): every `EditorConfig` field
/// (`pages/celleditors.rs`, `config`).
static CELL_EDITORS: &[Opt] = &[
    Opt::new("celleditors.dismiss", "Dismiss", &["light", "escape"]),
    Opt::new("celleditors.blur", "On blur", &["cancel", "commit"]),
    Opt::new("celleditors.max_rows", "Popup rows", &["8", "4", "12"]),
    Opt::new(
        "celleditors.popup_width",
        "Popup width",
        &["14-48", "14-24", "30-60"],
    ),
    Opt::new("celleditors.cursor_style", "Popup cursor", &["fill", "bar"]),
    Opt::new("celleditors.type_ahead", "Type-ahead", OFF_ON).default(1),
    Opt::new(
        "celleditors.affordance",
        "▾ affordance",
        &["focused", "always", "never"],
    ),
    Opt::new(
        "celleditors.flag_out_of_set",
        "Flag values outside the set",
        OFF_ON,
    )
    .default(1),
    Opt::new(
        "celleditors.tag_separator",
        "Tag separator",
        &["comma", "dot"],
    ),
    Opt::new(
        "celleditors.placement",
        "Popup placement",
        &["below", "above"],
    ),
];

/// Selection (SC13): the multi-select members table
/// (`pages/selectable.rs`, `members_table`).
static SELECTION: &[Opt] = &[
    Opt::new("selection.esc_clears", "Esc clears the selection", OFF_ON).default(1),
    Opt::new(
        "selection.toggle_mods",
        "Toggle click",
        &["platform", "ctrl", "alt"],
    ),
    Opt::new(
        "selection.disabled_rows",
        "Disabled rows",
        &["owner", "none"],
    ),
    Opt::new("selection.check_column", "Checkbox column", OFF_ON).default(1),
    Opt::new(
        "selection.activate",
        "Click opens",
        &["double", "again", "single", "never"],
    ),
];

/// Editable (SC13): the cell-navigating `DataTable`'s knobs
/// (`pages/editable.rs`, `build`).
static EDITABLE: &[Opt] = &[
    Opt::new("editable.rows", "Rows", &["tasks", "empty"]),
    Opt::new("editable.type_ahead", "Type-ahead", &["off", "any", "task"]),
    Opt::new(
        "editable.affordance",
        "▾ affordance",
        &["focused", "always", "never"],
    ),
    Opt::new("editable.overflow_marks", "Overflow marks", OFF_ON).default(1),
    Opt::new("editable.numeric", "Numeric ID and Changes", OFF_ON).default(1),
    Opt::new("editable.nav", "Vim keys", OFF_ON),
    Opt::new("editable.keymap", "Key map", &["default", "custom"]),
    Opt::new("editable.horizontal", "Horizontal bar", &["auto", "never"]),
    Opt::new("editable.reveal", "Reveal cut cells", OFF_ON).default(1),
    Opt::new(
        "editable.track_click",
        "Track click",
        &["shell", "page", "jump"],
    ),
    Opt::new("editable.blur_action", "On blur", &["commit", "cancel"]),
    Opt::new(
        "editable.header_labels",
        "Header menu labels",
        &["default", "custom"],
    ),
];

/// Data grid (SC12): every `DataGrid` knob, each rebuilding the grid
/// through its builder (`pages/grid.rs`, `build`).
static DATA_GRID: &[Opt] = &[
    Opt::new(
        "datagrid.access",
        "Access",
        &["editable", "view", "no-key", "query"],
    ),
    Opt::new("datagrid.rows", "Rows", &["loaded", "empty", "error"]),
    Opt::new(
        "datagrid.total",
        "Row total",
        &["estimated", "exact", "unknown"],
    ),
    Opt::new("datagrid.row_numbers", "Row numbers", OFF_ON).default(1),
    Opt::new("datagrid.overflow_badges", "Overflow badges", OFF_ON).default(1),
    Opt::new(
        "datagrid.labels",
        "Bar and menu labels",
        &["default", "custom"],
    ),
    Opt::new(
        "datagrid.type_ahead",
        "Type-ahead",
        &["off", "any", "customer"],
    ),
    Opt::new(
        "datagrid.affordance",
        "▾ affordance",
        &["focused", "always", "never"],
    ),
    Opt::new("datagrid.keymap", "Key map", &["default", "custom"]),
    Opt::new(
        "datagrid.duplicate_keys",
        "Duplicate row keys",
        &["ctrl-d", "shift-d"],
    ),
    Opt::new("datagrid.nav", "Vim keys", OFF_ON),
    Opt::new("datagrid.width_sample_rows", "Width sample", &["200", "8"]),
    Opt::new("datagrid.seams", "Column seams", &["on", "off", "wide"]),
    Opt::new(
        "datagrid.track_click",
        "Track click",
        &["shell", "page", "jump"],
    ),
    Opt::new("datagrid.horizontal", "Horizontal bar", &["auto", "never"]),
    Opt::new("datagrid.validator", "Validator", &["default", "strict"]),
    Opt::new("datagrid.blur_action", "On blur", &["commit", "cancel"]),
];

/// Columns (SC11): the width solver and `ColumnSizing` on a `DataTable`.
static COLUMNS: &[Opt] = &[
    Opt::new(
        "columns.width",
        "Row width",
        &["full", "72", "56", "40", "28"],
    ),
    Opt::new("columns.resizable", "Resizable columns", OFF_ON).default(1),
    Opt::new("columns.seam_w", "Seam hit width", &["1", "2", "3"]).default(1),
    Opt::new("columns.min_width", "Narrowest drag", &["3", "6", "10"]),
    Opt::new(
        "columns.header_labels",
        "Header menu labels",
        &["default", "custom"],
    ),
];

/// Basic table (SC11): the selection model.
static BASIC_TABLE: &[Opt] = &[Opt::new(
    "basictable.selection",
    "Selection",
    &["none", "single", "multi"],
)
.default(1)];
/// Chrome (SC4): the menu bar's builders. The readout under the bar reads
/// the built widget back, so every knob shows in the frame.
static CHROME: &[Opt] = &[
    // narrow enough that the titles overflow (`chrome.overflow`)
    Opt::new("chrome.bar", "Bar width", &["full", "narrow"]),
    Opt::new("chrome.open_on", "Open on", &["press", "release"]),
    Opt::new("chrome.overflow", "Overflow", &["hide", "chevron"])
        .needs(&[("chrome.bar", "narrow")]),
    Opt::new("chrome.hover_switch", "Hover switches menus", OFF_ON).default(1),
    Opt::new("chrome.press_drag", "Press-drag-release", OFF_ON).default(1),
    Opt::new("chrome.hide_owner_focus", "Hide page focus", OFF_ON).default(1),
    Opt::new("chrome.wrap", "Wrap ←/→", OFF_ON).default(1),
    Opt::new("chrome.type_ahead", "Type-ahead", OFF_ON),
    Opt::new("chrome.nav", "Nav keys", &["default", "vim"]),
    Opt::new("chrome.dismiss", "Dismiss", &["light", "escape"]),
    Opt::new("chrome.brand", "Brand", &["lockup", "compact", "none"]),
];

/// Status & hints (SC4): the resolved hint bar and the theme's
/// `HintBarStyle`.
static STATUS_HINTS: &[Opt] = &[
    // which layers exist: the topmost one wins
    Opt::new(
        "statushints.layer",
        "Top layer",
        &["screen", "menu", "modal"],
    ),
    Opt::new("statushints.align", "Align", &["start", "center"]),
    Opt::new("statushints.width", "Bar width", &["full", "narrow"]),
    Opt::new("statushints.compact", "Compact keys", OFF_ON)
        .needs(&[("statushints.width", "narrow")]),
    Opt::new("statushints.share", "Status share %", &["50", "30", "80"]).needs(&[
        ("statushints.width", "narrow"),
        ("statushints.status", "plain"),
    ]),
    Opt::new(
        "statushints.priority",
        "F5 priority",
        &["auto", "normal", "pinned"],
    )
    .needs(&[("statushints.width", "narrow")]),
    Opt::new(
        "statushints.status",
        "Status",
        &["none", "plain", "warning", "error", "parts"],
    ),
    Opt::new("statushints.badge", "EDIT badge", OFF_ON),
    Opt::new("statushints.hidden", "Hidden files shown", OFF_ON),
    // the too-small screen's builders (SC30)
    Opt::new(
        "statushints.too_small_min",
        "Too small below",
        &["72x20", "100x30"],
    ),
    Opt::new(
        "statushints.quit_hint",
        "Too-small quit hint",
        &["default", "custom"],
    ),
    Opt::new("statushints.brand", "Too-small brand", &["title", "accent"]),
];

/// Text utilities (SC26): the helpers' width and styles, and the
/// `PropsList` builders (its readout reads the built list back).
static TEXT_UTILITIES: &[Opt] = &[
    Opt::new("textutilities.width", "Width", &["8", "12", "20", "32"]).default(2),
    Opt::new(
        "textutilities.sheet",
        "Label column",
        &["default", "narrow-labels"],
    ),
    Opt::new(
        "textutilities.wrap",
        "Wrap",
        &["default", "plain", "no-hang", "no-indent"],
    ),
    Opt::new(
        "textutilities.duration",
        "Duration style",
        &["spoken", "compact"],
    ),
    Opt::new("textutilities.case", "Match case", OFF_ON),
    Opt::new("textutilities.nav", "List nav keys", &["default", "vim"]),
    Opt::new("textutilities.copy_key", "Copy key", &["y", "c"]),
    Opt::new(
        "textutilities.track_click",
        "Track click",
        &["page", "jump"],
    ),
    Opt::new(
        "textutilities.scrollbar_gap",
        "Scrollbar gap",
        &["0", "1", "2"],
    )
    .default(1),
    Opt::new(
        "textutilities.reveal",
        "Reveal",
        &["tooltip", "inline", "off"],
    ),
];

/// Breadcrumbs & rails (SC5): the trail's links, its width, and the
/// collapsible sidebar's rail.
static BREADCRUMBS_RAILS: &[Opt] = &[
    Opt::new("breadcrumbsrails.clickable", "Links", OFF_ON).default(1),
    Opt::new(
        "breadcrumbsrails.width",
        "Deep trail width",
        &["full", "48", "32", "20"],
    ),
    Opt::new("breadcrumbsrails.collapsed", "Sidebar collapsed", OFF_ON),
    Opt::new("breadcrumbsrails.collapsed_rail", "Collapsed rail", OFF_ON)
        .default(1)
        .needs(&[("breadcrumbsrails.collapsed", "on")]),
];

/// Help & keys (SC6): the `KeyReference` layout and the `HelpOverlay`
/// builders (the readout reads the overlay the options build).
static HELP_KEYS: &[Opt] = &[
    Opt::new("helpkeys.width", "Reference width", &["full", "40", "28"]),
    Opt::new(
        "helpkeys.key_max_share",
        "Key column share",
        &["40", "25", "60"],
    ),
    Opt::new("helpkeys.gap", "Column gap", &["3", "6"]),
    Opt::new("helpkeys.title", "Overlay title", &["default", "custom"]),
    Opt::new("helpkeys.max_percent", "Overlay size", &["80", "60", "100"]),
    Opt::new("helpkeys.nav", "Scroll keys", &["default", "vim"]),
    Opt::new("helpkeys.keymap", "Close keys", &["default", "f1"]),
    Opt::new("helpkeys.dismiss", "Dismiss", &["light", "escape"]),
];

/// Quit & leave (SC6): the page's `LeaveGuard`, the stakes at start, and
/// the `QuitKeys` its table classifies with.
static QUIT_LEAVE: &[Opt] = &[
    Opt::new(
        "quitleave.stakes",
        "Stakes at start",
        &["clean", "unsaved", "running"],
    ),
    Opt::new("quitleave.save", "Guard can save", OFF_ON).default(1),
    Opt::new("quitleave.letter", "Quit letter", &["q", "x", "none"]),
    Opt::new("quitleave.esc_quits", "Esc quits", OFF_ON),
    Opt::new("quitleave.ctrl_c", "Ctrl+C quits", OFF_ON).default(1),
    Opt::new("quitleave.ctrl_q", "Ctrl+Q quits", OFF_ON).default(1),
];

/// Lists (SC15): every `ListBox` builder (the readout reads the built
/// lists back).
static LISTS: &[Opt] = &[
    Opt::new("lists.nav", "Nav keys", &["default", "vim"]),
    Opt::new("lists.type_ahead", "Type-ahead", OFF_ON).default(1),
    Opt::new("lists.click", "Click", &["double", "single"]),
    Opt::new("lists.check", "Check click", &["box", "row"]),
    Opt::new(
        "lists.toggle_mods",
        "Toggle click",
        &["platform", "alt", "any"],
    ),
    Opt::new("lists.wrap", "Arrows wrap", OFF_ON),
    Opt::new("lists.skip_disabled", "Skip disabled", OFF_ON).default(1),
    Opt::new("lists.scrollbar_gap", "Scrollbar gap", &["0", "1", "2"]).default(1),
    Opt::new("lists.reveal", "Reveal", &["tooltip", "inline", "off"]),
    Opt::new("lists.track_click", "Track click", &["page", "jump"]),
    Opt::new("lists.keymap", "Key map", &["default", "custom"]),
    Opt::new("lists.empty", "Empty", &["text", "state"]),
    Opt::new("lists.content", "Content", &["ready", "loading", "error"]),
];

/// Trees (SC15): every `TreeView` builder (the readout reads the built
/// tree back).
static TREES: &[Opt] = &[
    Opt::new(
        "trees.branch",
        "Enter on a branch",
        &["toggle", "activate", "both"],
    ),
    Opt::new("trees.click", "Click", &["double", "single"]),
    Opt::new("trees.indent", "Indent", &["2", "1", "4"]),
    Opt::new("trees.divider", "Divider", OFF_ON),
    Opt::new("trees.alt_click", "Alt-click recursive", OFF_ON).default(1),
    Opt::new("trees.reserve_right", "Reserve right", &["0", "10"]),
    Opt::new("trees.show_meta", "Show meta", OFF_ON).default(1),
    Opt::new("trees.track_click", "Track click", &["page", "jump"]),
    Opt::new("trees.nav", "Nav keys", &["default", "vim"]),
    Opt::new("trees.wrap", "Arrows wrap", OFF_ON),
    Opt::new("trees.type_ahead", "Type-ahead", OFF_ON).default(1),
    Opt::new("trees.keymap", "Key map", &["default", "custom"]),
    Opt::new("trees.marker", "Selected marker", &["none", "chevron"]),
];

/// Overview (SC3): the explorer resolves one `Kind` on one `Plane` through
/// `Theme::interactive`; the States grid draws every kind on the plane.
static OVERVIEW: &[Opt] = &[
    Opt::new(
        "overview.plane",
        "Plane",
        &[
            "canvas",
            "surface",
            "card",
            "field",
            "raised",
            "popover",
            "highlight",
            "highlight-danger",
            "accent",
            "danger",
        ],
    )
    .default(2),
    Opt::new(
        "overview.kind",
        "Kind",
        &[
            "row",
            "cursor-fill",
            "cursor-bar",
            "primary",
            "secondary",
            "subtle",
            "danger",
            "toggle",
            "tab",
            "title",
            "chip",
            "close",
            "field",
            "handle",
            "seam",
        ],
    )
    .default(4),
    // `Theme::with_ladder` on the chosen plane
    Opt::new("overview.ladder", "Ladder", &["theme", "custom"]),
    // `Theme::with_state_modifiers`
    Opt::new(
        "overview.mods",
        "State modifiers",
        &["level", "color", "16", "mono"],
    ),
];

/// Toolbars (SC28): the `Toolbar` builders.
pub static TOOLBARS: &[Opt] = &[
    Opt::new("toolbars.nav", "Nav keys", &["desktop", "vim"]),
    Opt::new("toolbars.wrap", "Wrap", OFF_ON),
    Opt::new("toolbars.skip_disabled", "Skip disabled", OFF_ON).default(1),
    Opt::new("toolbars.gap", "Gap", &["0", "1", "2"]).default(1),
    Opt::new("toolbars.overflow", "Overflow button", &["»", "…", "off"]),
];

/// Focus & navigation (SC28): one list's navigation keys and what Tab
/// does in a field. The app-wide focus and input options are shell ones.
pub static FOCUS_NAV: &[Opt] = &[
    Opt::new(
        "focusnavigation.nav",
        "List nav keys",
        &["desktop", "vim", "no-page", "no-home-end", "vim-no-arrows"],
    ),
    Opt::new(
        "focusnavigation.tab_in_field",
        "Tab in a field",
        &["moves", "stays"],
    ),
];

/// The Menus page (SC16): the menu builders and the row menus'
/// `ContextConfig`.
pub static MENUS: &[Opt] = &[
    Opt::new(
        "menus.placement",
        "Actions placement",
        &["below", "above", "right", "left", "center"],
    ),
    Opt::new("menus.fit", "Fit", &["menu", "flush", "default"]),
    Opt::new("menus.edge_margin", "Edge margin", &["0", "1", "2"]).default(1),
    Opt::new("menus.min_width", "Min width", &["8", "16", "24"]),
    Opt::new("menus.max_rows", "Max rows", &["all", "8", "4"]),
    Opt::new("menus.title", "Title", OFF_ON).default(1),
    Opt::new("menus.release_selects", "Release selects", OFF_ON).default(1),
    Opt::new("menus.hide_owner_focus", "Hide page focus", OFF_ON).default(1),
    Opt::new("menus.nav", "Nav keys", &["desktop", "vim"]),
    Opt::new("menus.wrap", "Wrap", OFF_ON).default(1),
    Opt::new("menus.type_ahead", "Type-ahead", OFF_ON),
    Opt::new("menus.dismiss", "Dismiss", &["light", "escape"]),
    Opt::new(
        "menus.unknown_key",
        "Other keys",
        &["close-and-pass", "swallow"],
    ),
    Opt::new("menus.moves_cursor", "Click moves cursor", OFF_ON).default(1),
    Opt::new("menus.shift_f10", "Shift+F10", OFF_ON).default(1),
    Opt::new("menus.menu_key", "Menu key", OFF_ON).default(1),
    Opt::new("menus.extra_key", "Extra key", &["none", "m"]),
];

/// The Dialogs page (SC17): the `Dialog` builders, applied to every
/// dialog the page opens and read back in its "Applied" card.
pub static DIALOGS: &[Opt] = &[
    Opt::new("dialogs.width", "Width", &["auto", "44", "80"]),
    Opt::new(
        "dialogs.placement",
        "Placement",
        &["center", "below", "above", "right", "left"],
    ),
    Opt::new("dialogs.chrome", "Chrome rows", &["0-0", "3-2"]),
    Opt::new(
        "dialogs.outer_margin",
        "Outer margin",
        &["3x1", "0x0", "8x3"],
    ),
    Opt::new("dialogs.max_height", "Max height", &["none", "14", "9"]),
    Opt::new("dialogs.body_scroll", "Body scrolls", OFF_ON).default(1),
    Opt::new(
        "dialogs.backdrop",
        "Backdrop",
        &["default", "full", "keep-top"],
    ),
    Opt::new(
        "dialogs.dismiss",
        "Dismiss",
        &["auto", "light", "escape", "never"],
    ),
    Opt::new("dialogs.close_button", "× button", &["default", "danger"]),
    Opt::new("dialogs.accelerators", "Accelerators", &["off", "y-n"]),
    Opt::new("dialogs.keymap", "Key map", &["default", "ctrl"]),
    Opt::new("dialogs.mnemonics", "Mnemonics", OFF_ON).default(1),
    Opt::new("dialogs.nav", "Nav keys", &["desktop", "vim"]),
    Opt::new(
        "dialogs.ack_enter",
        "Enter in token",
        &["submit", "next-field"],
    ),
    Opt::new("dialogs.guard_dirty", "Guard edited prompt", OFF_ON).default(1),
    Opt::new("dialogs.guard_destructive", "Guard danger cancel", OFF_ON).default(1),
    Opt::new(
        "dialogs.edit_entry",
        "Field edit entry",
        &["immediate", "type-to-edit", "explicit"],
    ),
    Opt::new(
        "dialogs.button_row",
        "Button row",
        &["end-wrap", "start-wrap", "end-more"],
    ),
    Opt::new("dialogs.cancel_label", "Cancel label", &["cancel", "keep"]),
    Opt::new(
        "dialogs.confirm_index",
        "Token gates",
        &["action", "nothing"],
    ),
];

/// The Dialog kinds page (SC18): the form, info, choice and help
/// builders, read back in the page's "Applied" card.
pub static DIALOG_KINDS: &[Opt] = &[
    Opt::new("dialogkinds.width", "Width", &["auto", "48", "72"]),
    Opt::new("dialogkinds.nav", "Nav keys", &["desktop", "vim"]),
    Opt::new(
        "dialogkinds.close_button",
        "× button",
        &["default", "danger"],
    ),
    Opt::new("dialogkinds.button_wrap", "Buttons wrap", OFF_ON),
    Opt::new("dialogkinds.keep_open", "Form stays open on Save", OFF_ON),
    Opt::new("dialogkinds.form_meta", "Form meta", OFF_ON).default(1),
    Opt::new("dialogkinds.info_error", "Info error title", OFF_ON),
    Opt::new(
        "dialogkinds.info_keymap",
        "Info key map",
        &["default", "ctrl"],
    ),
    Opt::new("dialogkinds.choice_tones", "Choice tones", OFF_ON).default(1),
    Opt::new("dialogkinds.choice_stepper", "Choice stepper", OFF_ON).default(1),
    Opt::new(
        "dialogkinds.choice_buttons",
        "Choice buttons",
        &["two", "three"],
    ),
];

/// The Popups & tooltips page (SC19): the popover's placement, fit and
/// dismissal, the modal frame of its States grid, the tooltip policy and
/// the reveal policy.
pub static POPUPS_TOOLTIPS: &[Opt] = &[
    Opt::new(
        "popupstooltips.placement",
        "Popover placement",
        &["below", "above", "right", "left", "center"],
    ),
    Opt::new("popupstooltips.margin", "Edge margin", &["1", "0", "2"]),
    Opt::new(
        "popupstooltips.vertical_margin",
        "Vertical margin",
        &["as-margin", "0", "2"],
    ),
    Opt::new("popupstooltips.v_align", "Centred at", &["center", "third"]),
    Opt::new("popupstooltips.overflow", "No room", &["shrink", "slide"]),
    Opt::new("popupstooltips.text_inset", "Text inset", &["0", "2"]),
    Opt::new("popupstooltips.escape", "Esc closes", OFF_ON).default(1),
    Opt::new(
        "popupstooltips.outside_click",
        "Outside click closes",
        OFF_ON,
    )
    .default(1),
    Opt::new(
        "popupstooltips.pass_through",
        "Click passes through",
        OFF_ON,
    ),
    Opt::new(
        "popupstooltips.refuse_feedback",
        "Refused click",
        &["flash", "none"],
    ),
    Opt::new("popupstooltips.focus_out", "Focus out closes", OFF_ON).default(1),
    Opt::new(
        "popupstooltips.frame_placement",
        "Frame placement",
        &["center", "below", "right"],
    ),
    Opt::new(
        "popupstooltips.frame_fit",
        "Frame fit",
        &["default", "flush", "menu"],
    ),
    Opt::new(
        "popupstooltips.frame_margin",
        "Frame padding",
        &["3x1", "1x0"],
    ),
    Opt::new(
        "popupstooltips.frame_min",
        "Frame min size",
        &["20x5", "40x8"],
    ),
    Opt::new(
        "popupstooltips.frame_chrome",
        "Frame chrome rows",
        &["1", "0", "3"],
    ),
    Opt::new("popupstooltips.frame_backdrop", "Frame dims", OFF_ON).default(1),
    Opt::new(
        "popupstooltips.frame_dim",
        "Rows kept live",
        &["default", "full", "keep-top", "keep-bottom"],
    ),
    Opt::new(
        "popupstooltips.frame_dismiss",
        "Frame dismiss",
        &["escape", "never"],
    ),
    Opt::new("popupstooltips.frame_close", "Frame × button", OFF_ON).default(1),
    Opt::new(
        "popupstooltips.frame_close_button",
        "× look",
        &["default", "danger"],
    ),
    Opt::new("popupstooltips.tooltips", "Tooltips", OFF_ON).default(1),
    Opt::new(
        "popupstooltips.tip_delay",
        "Tooltip delay",
        &["600", "0", "1500"],
    ),
    Opt::new("popupstooltips.tip_width", "Tooltip width", &["48", "24"]),
    Opt::new("popupstooltips.reveal_hover", "Reveal on hover", OFF_ON).default(1),
    Opt::new(
        "popupstooltips.reveal_key",
        "Reveal key",
        &["right", "r", "none"],
    ),
    Opt::new(
        "popupstooltips.reveal_mode",
        "Reveal as",
        &["tooltip", "inline"],
    ),
    Opt::new("popupstooltips.reveal_rows", "Reveal rows", &["0", "3"]),
    Opt::new("popupstooltips.reveal_scroll", "Reveal scrolls", OFF_ON).default(1),
    Opt::new("popupstooltips.reveal_wheel", "Wheel lines", &["3", "1"]),
    Opt::new("popupstooltips.reveal_delay", "Reveal delay", &["300", "0"]),
    Opt::new(
        "popupstooltips.reveal_passes",
        "Wheel passes at ends",
        OFF_ON,
    )
    .default(1),
];

/// The Pickers page (SC20): the `Picker` builders, applied to every picker
/// the page opens and read back in its "Applied" card.
pub static PICKERS: &[Opt] = &[
    Opt::new("pickers.width", "Width", &["auto", "48", "96"]),
    Opt::new("pickers.max_rows", "Max rows", &["12", "6"]),
    Opt::new("pickers.placement", "Placement", &["center", "below"]),
    Opt::new("pickers.fit", "Fit", &["default", "third", "flush"]),
    Opt::new("pickers.backdrop", "Backdrop", &["default", "full"]),
    Opt::new("pickers.cursor_style", "Cursor row", &["fill", "bar"]),
    Opt::new("pickers.delete_is_secondary", "Delete closes a tab", OFF_ON).default(1),
    Opt::new("pickers.nav", "Nav keys", &["desktop", "vim"]),
    Opt::new("pickers.dismiss", "Dismiss", &["light", "escape"]),
    Opt::new("pickers.empty", "No matches", &["text", "state"]),
    Opt::new("pickers.placeholder", "Placeholder", &["default", "custom"]),
    Opt::new("pickers.hints", "Hint row", &["page", "short"]),
    Opt::new("pickers.remote", "Remote fetch", &["works", "fails"]),
];

/// The Tabs page (SC14): the editor strip's builders.
pub static TABS: &[Opt] = &[
    Opt::new(
        "tabs.rule",
        "Active rule",
        &["auto", "accent", "neutral", "focus-only"],
    ),
    Opt::new("tabs.close_reveal", "Show ×", &["always", "hover"]),
    Opt::new("tabs.close_w", "× target width", &["1", "3", "5"]).default(1),
    Opt::new("tabs.close_button", "× button", &["default", "danger"]),
    Opt::new("tabs.allow_new", "New-tab +", OFF_ON).default(1),
    Opt::new("tabs.new_key", "New-tab key", &["n", "t", "none"]),
    Opt::new("tabs.close_keys", "Close keys", &["x-delete", "delete"]),
    Opt::new("tabs.keymap", "Key map", &["default", "ctrl"]),
    Opt::new("tabs.digit_jump", "Digits jump", OFF_ON).default(1),
    Opt::new("tabs.nav", "Nav keys", &["desktop", "vim"]),
    Opt::new("tabs.wrap", "Wrap", OFF_ON),
    Opt::new("tabs.middle_click", "Middle-click closes", OFF_ON).default(1),
    Opt::new("tabs.overflow_menu", "Overflow +N", OFF_ON).default(1),
    Opt::new("tabs.more_button", "+N button", &["default", "tight"]),
    Opt::new("tabs.passive", "Passive", &["off", "on", "owner-focused"]),
    Opt::new("tabs.gap", "Body gap", &["1", "0"]),
    Opt::new("tabs.count", "Tabs", &["few", "many"]).default(1),
];

/// Progress (SC21): the bar layout every bar shares, and the meters'
/// visual and stale style.
pub static PROGRESS: &[Opt] = &[
    Opt::new(
        "progress.label_width",
        "Label column",
        &["16", "measured", "10"],
    ),
    Opt::new("progress.reserve_value", "Reserve the value column", OFF_ON).default(1),
    Opt::new("progress.meter_visual", "Meter visual", &["line", "block"]),
    Opt::new("progress.stale_style", "Stale style", &["dashed", "faint"]),
];

/// Meters (SC36): the columns every `MeterList` on the page shares, the
/// quota list's visual, and the offline look.
pub static METERS: &[Opt] = &[
    Opt::new(
        "meters.label_width",
        "Label column",
        &["measured", "12", "8"],
    ),
    Opt::new("meters.label_gap", "Label gap", &["1", "3"]),
    Opt::new("meters.meter_width", "Meter width", &["24", "fill", "14"]),
    Opt::new("meters.detail", "Details", OFF_ON).default(1),
    Opt::new("meters.detail_tone", "Detail tone", &["muted", "secondary"]),
    Opt::new("meters.visual", "Quota meter visual", &["block", "line"]),
    Opt::new("meters.offline", "Offline (last-good)", OFF_ON),
    Opt::new("meters.stale_style", "Stale style", &["dashed", "faint"]),
];

/// File browser (SC41): what Choose accepts, the optional parts (path
/// bar, filter, hidden-files check box, option check box, URL entry, step
/// line), type-ahead, the key map and the dismissal policy of the
/// `FileBrowser` the page opens.
pub static FILE_BROWSER: &[Opt] = &[
    Opt::new("filebrowser.choose", "Choose", &["folders", "files", "any"]),
    Opt::new("filebrowser.source", "Source", &["fixture", "real"]),
    Opt::new("filebrowser.path_bar", "Path bar", OFF_ON).default(1),
    Opt::new("filebrowser.filter", "Name filter", OFF_ON).default(1),
    Opt::new("filebrowser.hidden_toggle", "Hidden toggle", OFF_ON).default(1),
    Opt::new("filebrowser.show_hidden", "Show hidden", OFF_ON),
    Opt::new("filebrowser.type_ahead", "Type-ahead", OFF_ON).default(1),
    Opt::new("filebrowser.keys", "Keys", &["desktop", "vim"]),
    Opt::new("filebrowser.url_entry", "URL entry", OFF_ON),
    Opt::new("filebrowser.checkbox", "Option check box", OFF_ON).default(1),
    Opt::new("filebrowser.stepper", "Step line", OFF_ON),
    Opt::new(
        "filebrowser.dismiss",
        "Dismiss",
        &["escape", "light", "never"],
    ),
];

/// Notices (SC36): the kind of the query's error card, and the parts and
/// limits every `Notice` on the page shares.
pub static NOTICES: &[Opt] = &[
    Opt::new(
        "notices.kind",
        "Result notice kind",
        &["error", "warning", "info", "success"],
    ),
    Opt::new("notices.detail", "Detail", OFF_ON).default(1),
    Opt::new("notices.hint", "Hint", OFF_ON).default(1),
    Opt::new("notices.max_lines", "Detail lines", &["2", "1", "4"]),
    Opt::new("notices.message_lines", "Message lines", &["2", "1"]),
    Opt::new(
        "notices.detail_tone",
        "Detail tone",
        &["secondary", "muted"],
    ),
    Opt::new("notices.mark_cut", "Mark the cut …", OFF_ON).default(1),
    Opt::new("notices.width", "Width", &["wide", "narrow"]),
];

/// Disk cleanup (SC49): the cleanup mode, the typed gate, the checklist's
/// click policy, the tree's size column, the meters' visual and a held
/// scan.
pub static DISK_CLEANUP: &[Opt] = &[
    Opt::new(
        "diskcleanup.mode",
        "Mode",
        &["trash", "permanent", "dry-run"],
    ),
    Opt::new("diskcleanup.typed", "Typed phrase (gate 2)", OFF_ON).default(1),
    Opt::new("diskcleanup.check_click", "Check on click", &["box", "row"]).behaviour(),
    Opt::new("diskcleanup.sizes", "Tree sizes", OFF_ON).default(1),
    Opt::new(
        "diskcleanup.meter_visual",
        "Meter visual",
        &["line", "block"],
    ),
    Opt::new("diskcleanup.scan", "Scan", &["complete", "scanning"]),
    Opt::new(
        "diskcleanup.collapse_below",
        "Collapse below",
        &["72", "108"],
    ),
];

/// Activity (SC51): the run's state, the log's wrap, retention and find
/// query, and the stream chips' digit keys.
pub static ACTIVITY: &[Opt] = &[
    Opt::new(
        "activity.state",
        "Run state",
        &["running", "stopped", "failed"],
    ),
    Opt::new("activity.wrap", "Wrap long lines", OFF_ON).default(1),
    Opt::new("activity.retention", "Retained lines", &["2000", "20"]),
    Opt::new("activity.find", "Find", &["off", "error", "job"]),
    Opt::new("activity.digit_toggles", "Digits toggle streams", OFF_ON)
        .default(1)
        .behaviour(),
];

/// Workspaces (SC52): the master-detail's collapse width, seam and Esc,
/// the roster, and the purge's typed confirmation.
pub static WORKSPACES: &[Opt] = &[
    Opt::new(
        "workspaces.collapse_below",
        "Collapse below",
        &["80", "108", "150"],
    ),
    Opt::new("workspaces.size", "Detail size", &["resizable", "share"]),
    Opt::new("workspaces.roster", "Instance roster", &["auto", "off"]),
    Opt::new("workspaces.esc_returns", "Esc returns to the tree", OFF_ON)
        .default(1)
        .behaviour(),
    Opt::new("workspaces.typed", "Typed purge confirm", OFF_ON)
        .default(1)
        .behaviour(),
];

/// Accounts & usage (SC53): the demo frame's width (narrow shows the
/// collapsed master-detail), the quota meters' visual, offline usage and
/// the account warnings.
pub static ACCOUNTS_USAGE: &[Opt] = &[
    Opt::new("accountsusage.width", "Width", &["fill", "72"]),
    Opt::new(
        "accountsusage.visual",
        "Quota meter visual",
        &["block", "line"],
    ),
    Opt::new("accountsusage.offline", "Usage offline (last-good)", OFF_ON),
    Opt::new("accountsusage.warnings", "Account warnings", OFF_ON).default(1),
];

/// Launch cockpit (SC55): the path the launch takes, and the rail's and
/// the build log's knobs.
pub static LAUNCH_COCKPIT: &[Opt] = &[
    Opt::new(
        "launchcockpit.outcome",
        "Launch outcome",
        &["success", "credentials", "build-failure"],
    )
    .behaviour(),
    Opt::new("launchcockpit.numbered", "Numbered stages", OFF_ON),
    Opt::new("launchcockpit.selectable", "Stages take focus", OFF_ON),
    Opt::new("launchcockpit.wrap", "Wrap the log", OFF_ON).default(1),
];

/// Read-only report (SC56): the report's state, its meters' visual,
/// the attention notices and whether facts copy.
pub static READONLY_REPORT: &[Opt] = &[
    Opt::new("readonlyreport.offline", "Offline (last-good)", OFF_ON),
    Opt::new("readonlyreport.visual", "Meter visual", &["block", "line"]),
    Opt::new("readonlyreport.notices", "Attention notices", OFF_ON).default(1),
    Opt::new("readonlyreport.copyable", "Facts copy", OFF_ON).default(1),
];

/// Empty & error (SC21): the empty state's kind, hint and actions, and
/// the read-only reason.
pub static EMPTY_ERROR: &[Opt] = &[
    Opt::new("emptyerror.kind", "Kind", &["empty", "error"]),
    Opt::new("emptyerror.hint", "Hint", OFF_ON).default(1),
    Opt::new("emptyerror.actions", "Actions", &["none", "one", "two"]).default(2),
    Opt::new(
        "emptyerror.reason",
        "Read-only reason",
        &["view", "no-key", "result"],
    ),
];

/// Splits & stacks (SC23): the split, its seams and the stack.
pub static SPLITS_STACKS: &[Opt] = &[
    Opt::new(
        "splitsstacks.dir",
        "Split direction",
        &["horizontal", "vertical"],
    ),
    Opt::new("splitsstacks.percent", "First pane %", &["30", "50", "70"]),
    Opt::new("splitsstacks.min", "First pane minimum", &["16", "40"]),
    Opt::new(
        "splitsstacks.maximized",
        "Zoomed pane",
        &["none", "first", "second"],
    ),
    Opt::new(
        "splitsstacks.double_click",
        "Seam double-click",
        &["reset", "ignore"],
    ),
    Opt::new("splitsstacks.hit_pad", "Seam hit padding", &["1", "0", "3"]),
    Opt::new(
        "splitsstacks.overflow",
        "Stack overflow",
        &["scroll", "clip"],
    ),
    Opt::new(
        "splitsstacks.min_child_rows",
        "Stack child minimum",
        &["1", "3"],
    ),
    Opt::new("splitsstacks.gap", "Stack gap", &["0", "1"]),
    Opt::new(
        "splitsstacks.scrollbar_gap",
        "Stack scrollbar gap",
        &["1", "0", "2"],
    ),
    Opt::new(
        "splitsstacks.track_click",
        "Stack track click",
        &["page", "jump"],
    ),
];

/// Master-detail (SC38): the `MasterDetail` builders, its `SplitKeys`, and
/// the demo frame's width (so the collapsed layout shows on a wide
/// terminal).
pub static MASTER_DETAIL: &[Opt] = &[
    Opt::new("masterdetail.width", "Demo width", &["fill", "100", "72"]),
    Opt::new(
        "masterdetail.collapse",
        "Collapses below",
        &["80", "108", "140"],
    ),
    Opt::new("masterdetail.size", "Detail size", &["resizable", "share"]),
    Opt::new(
        "masterdetail.summary",
        "Narrow summary rows",
        &["1", "0", "3"],
    )
    .needs(&[("masterdetail.width", "72")]),
    Opt::new("masterdetail.esc_returns", "Esc returns to list", OFF_ON).default(1),
    Opt::new(
        "masterdetail.keys",
        "Resize keys",
        &["default", "custom", "none"],
    ),
    Opt::new("masterdetail.step", "Resize step", &["2", "1", "5"]),
    Opt::new("masterdetail.gap", "Gap between panes", &["2", "1", "4"]),
];

/// Pane tree (SC45): the `PaneTree` builders and its starting layout.
pub static PANE_TREE: &[Opt] = &[
    Opt::new(
        "panetree.layout",
        "Starting layout",
        &["three", "grid", "single"],
    ),
    Opt::new(
        "panetree.frames",
        "Pane frames",
        &["when-split", "always", "never"],
    ),
    Opt::new("panetree.min", "Pane minimum", &["12x3", "24x6"]),
    Opt::new("panetree.keys", "Pane keys", &["default", "custom", "none"]),
    Opt::new("panetree.step", "Resize step", &["2", "1", "5"]),
    Opt::new("panetree.hit_pad", "Seam hit padding", &["1", "0", "3"]),
];

/// Code editor (SC24): every `CodeEditor` knob, then the completion
/// popup's.
pub static CODE_EDITOR: &[Opt] = &[
    Opt::new("codeeditor.read_only", "Read-only", OFF_ON),
    Opt::new(
        "codeeditor.edit_entry",
        "Edit entry",
        &["theme", "immediate", "type-to-edit", "explicit"],
    ),
    Opt::new("codeeditor.tab_leaves", "Tab leaves the editor", OFF_ON),
    Opt::new("codeeditor.esc_reverts", "Esc reverts", OFF_ON),
    Opt::new(
        "codeeditor.field_keys",
        "Field keys",
        &["theme", "emacs", "desktop"],
    ),
    Opt::new(
        "codeeditor.text_keymap",
        "Text key map",
        &["preset", "custom"],
    ),
    Opt::new(
        "codeeditor.nav",
        "Reading keys",
        &["default", "vim", "no-page"],
    ),
    Opt::new("codeeditor.keymap", "Command keys", &["default", "custom"]),
    Opt::new("codeeditor.indent", "Indent", &["4", "2"]),
    Opt::new("codeeditor.highlighter", "Syntax tones", OFF_ON).default(1),
    Opt::new("codeeditor.segmenter", "Blocks", OFF_ON).default(1),
    Opt::new("codeeditor.document", "Document", &["sample", "empty"]),
    Opt::new(
        "codeeditor.horizontal",
        "Horizontal scroll",
        &["theme", "on", "off"],
    ),
    Opt::new(
        "codeeditor.scrollbar_gap",
        "Scrollbar gap",
        &["1", "0", "2"],
    ),
    Opt::new("codeeditor.track_click", "Track click", &["page", "jump"]),
    // completion
    Opt::new("codeeditor.completion_rows", "Completion rows", &["8", "4"]),
    Opt::new(
        "codeeditor.completion_width",
        "Completion width",
        &["default", "wide"],
    ),
    Opt::new(
        "codeeditor.completion_placement",
        "Completion placement",
        &["below", "above"],
    ),
    Opt::new(
        "codeeditor.completion_fit",
        "Completion fit",
        &["default", "flush"],
    ),
    Opt::new(
        "codeeditor.completion_cursor",
        "Completion cursor",
        &["fill", "bar"],
    ),
    Opt::new(
        "codeeditor.completion_dismiss",
        "Completion dismiss",
        &["light", "escape"],
    ),
    Opt::new(
        "codeeditor.completion_nav",
        "Completion keys",
        &["default", "vim"],
    ),
];

/// Diff (SC25): the view's mode, the no-file state and a narrow pane.
pub static DIFF: &[Opt] = &[
    Opt::new("diff.mode", "Mode", &["unified", "review"]),
    Opt::new("diff.file", "File", &["changes", "none"]),
    Opt::new("diff.width", "Page width", &["full", "narrow"]),
];

/// Terminal (SC25): every `TextViewport` knob, then the step rail's.
pub static TERMINAL: &[Opt] = &[
    Opt::new("terminal.wrap", "Wrap long lines", OFF_ON),
    Opt::new("terminal.max_lines", "Lines kept", &["2000", "20"]),
    Opt::new(
        "terminal.horizontal",
        "Horizontal scroll",
        &["theme", "on", "off"],
    ),
    Opt::new("terminal.scrollbar_gap", "Scrollbar gap", &["1", "0", "2"]),
    Opt::new("terminal.track_click", "Track click", &["page", "jump"]),
    Opt::new(
        "terminal.nav",
        "Viewport keys",
        &["default", "vim", "no-page"],
    ),
    Opt::new("terminal.keymap", "Command keys", &["default", "custom"]),
    Opt::new("terminal.mark", "Find marks", OFF_ON),
    // step rail
    Opt::new("terminal.selectable", "Selectable rail", OFF_ON),
    Opt::new("terminal.numbered", "Numbered steps", OFF_ON).default(1),
    Opt::new(
        "terminal.rail_nav",
        "Rail keys",
        &["default", "vim", "no-page"],
    ),
];

/// Every option `--opt` accepts: the shell-wide ones, then each page's.
pub fn all() -> impl Iterator<Item = &'static Opt> {
    SHELL.iter().chain(
        crate::app::NAV_ENTRIES
            .iter()
            .flat_map(|e| page_options(e.id).iter()),
    )
}

/// The option named `key`.
pub fn find(key: &str) -> Option<&'static Opt> {
    let key = key.trim();
    all().find(|o| o.key.eq_ignore_ascii_case(key))
}

/// The page that owns a page option's key (`None` for `shell.*`).
pub fn page_of(key: &str) -> Option<PageId> {
    let (scope, _) = key.split_once('.')?;
    if scope == "shell" {
        return None;
    }
    PageId::from_name(scope)
}

/// One validated `--opt` assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptSet {
    pub key: &'static str,
    /// Index into the option's values.
    pub value: usize,
}

/// Parse `KEY=VALUE` (`--opt`). An unknown key lists the valid keys; an
/// invalid value lists the option's values.
pub fn parse_opt(arg: &str) -> Result<OptSet, String> {
    let Some((key, value)) = arg.split_once('=') else {
        return Err(format!(
            "expected KEY=VALUE, got {arg:?}; valid keys: {}",
            keys()
        ));
    };
    let Some(opt) = find(key) else {
        return Err(format!(
            "unknown option {:?}; valid keys: {}",
            key.trim(),
            keys()
        ));
    };
    let Some(i) = opt.index_of(value) else {
        return Err(format!(
            "invalid value {:?} for {}; valid values: {}",
            value.trim(),
            opt.key,
            opt.values.join(", ")
        ));
    };
    Ok(OptSet {
        key: opt.key,
        value: i,
    })
}

fn keys() -> String {
    all().map(|o| o.key).collect::<Vec<_>>().join(", ")
}

/// The shell-wide option values of a session (indices into each
/// option's values, in [`SHELL`] order).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellOptions {
    values: Vec<usize>,
}

impl Default for ShellOptions {
    fn default() -> Self {
        Self {
            values: SHELL.iter().map(|o| o.default).collect(),
        }
    }
}

impl ShellOptions {
    fn index(key: &str) -> usize {
        SHELL
            .iter()
            .position(|o| o.key == key)
            .unwrap_or_else(|| panic!("no shell option {key}"))
    }
    /// The value index of `key`.
    pub fn get(&self, key: &str) -> usize {
        self.values[Self::index(key)]
    }
    /// The value of `key`, as `--opt` spells it.
    pub fn value(&self, key: &str) -> &'static str {
        let i = Self::index(key);
        SHELL[i].values[self.values[i]]
    }
    /// Set `key` to value index `value` (clamped to its values).
    pub fn set(&mut self, key: &str, value: usize) {
        let i = Self::index(key);
        self.values[i] = value.min(SHELL[i].values.len() - 1);
    }
    pub fn on(&self, key: &str) -> bool {
        self.value(key) == "on"
    }
}

// ---------------------------------------------------------------- controls

enum Control {
    Toggle(Toggle),
    Select(Box<Select>),
}

impl Control {
    fn id(&self) -> WidgetId {
        match self {
            Self::Toggle(t) => t.id,
            Self::Select(s) => s.id,
        }
    }
    fn height(&self) -> u16 {
        match self {
            Self::Toggle(_) => 1,
            // label row and field row
            Self::Select(_) => 2,
        }
    }
}

/// One row of an [`OptionsView`]: a group heading or an option's control.
enum Row {
    Heading(&'static str),
    /// A group with no options yet, and why.
    Note(&'static str),
    Option(&'static Opt, Control),
}

/// The controls for a list of option groups, stacked and scrolling by
/// row: the Inspector's Options tab, and a page's Options section.
///
/// The view holds no values: [`Self::sync`] reads them from their owner
/// before each frame, and an event that changes one returns it as an
/// [`OptSet`] for the owner to apply.
pub struct OptionsView {
    rows: Vec<Row>,
    sections: Sections,
}

/// A group of the view: its heading, its options, and what to show when it
/// has none.
pub struct Group {
    pub heading: &'static str,
    pub opts: &'static [Opt],
    pub empty: &'static str,
}

impl OptionsView {
    pub fn new(id: WidgetId, groups: &[Group]) -> Self {
        let mut rows = Vec::new();
        for g in groups {
            rows.push(Row::Heading(g.heading));
            if g.opts.is_empty() {
                rows.push(Row::Note(g.empty));
            }
            for opt in g.opts {
                let cid = id.sub(opt.key);
                let control = if opt.is_toggle() {
                    Control::Toggle(Toggle::new(cid, opt.label, opt.default == 1))
                } else {
                    Control::Select(Box::new(Select::new(
                        cid,
                        opt.label,
                        opt.values,
                        opt.default,
                    )))
                };
                rows.push(Row::Option(opt, control));
            }
        }
        let mut sections = Sections::new(id.sub("rows"));
        sections.stack = sections.stack.gap(0);
        Self { rows, sections }
    }

    fn controls(&self) -> impl Iterator<Item = (&'static Opt, &Control)> {
        self.rows.iter().filter_map(|r| match r {
            Row::Option(o, c) => Some((*o, c)),
            _ => None,
        })
    }

    fn controls_mut(&mut self) -> impl Iterator<Item = (&'static Opt, &mut Control)> {
        self.rows.iter_mut().filter_map(|r| match r {
            Row::Option(o, c) => Some((*o, c)),
            _ => None,
        })
    }

    /// The control id of option `key`.
    #[allow(dead_code)] // tests reach controls by key
    pub fn control_id(&self, key: &str) -> Option<WidgetId> {
        self.controls()
            .find(|(o, _)| o.key == key)
            .map(|(_, c)| c.id())
    }

    /// Load the current value of every option from its owner. An open
    /// select keeps its list.
    pub fn sync(&mut self, value: impl Fn(&Opt) -> usize) {
        for (opt, c) in self.controls_mut() {
            let v = value(opt);
            match c {
                Control::Toggle(t) => t.on = v == 1,
                Control::Select(s) if !s.open => {
                    s.selected = v;
                }
                Control::Select(_) => {}
            }
        }
    }

    /// Whether `id` is one of the view's option controls.
    pub fn has_control(&self, id: WidgetId) -> bool {
        self.controls().any(|(_, c)| c.id() == id)
    }

    /// Whether an option's select has its list open.
    pub fn popup_open(&self) -> bool {
        self.controls()
            .any(|(_, c)| matches!(c, Control::Select(s) if s.open))
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let t = ctx.theme;
        let children: Vec<StackChild> = self
            .rows
            .iter()
            .enumerate()
            .map(|(i, r)| match r {
                // a blank row above every group but the first
                Row::Heading(_) => StackChild::fixed(1 + u16::from(i > 0)),
                Row::Note(_) => StackChild::fixed(1),
                Row::Option(_, c) => StackChild::fixed(c.height()),
            })
            .collect();
        let focus = self.rows.iter().position(|r| match r {
            Row::Option(_, c) => ctx.interaction.has_focus(c.id()),
            _ => false,
        });
        let l = self.sections.layout(area, &children, focus, buf, ctx);
        for (i, row) in self.rows.iter_mut().enumerate() {
            let slot = l.slot(i);
            match (row, slot) {
                (Row::Heading(h), Some(r)) => {
                    let y = r.bottom() - 1;
                    buf.set_string(r.x, y, *h, t.title().bg(bg));
                }
                (Row::Note(n), Some(r)) => {
                    let text = junie_tui::ui::text::truncate(n, r.width as usize);
                    buf.set_string(r.x, r.y, text, t.faint().bg(bg));
                }
                (Row::Option(_, Control::Toggle(tg)), Some(r)) => tg.render(r, buf, ctx, bg),
                (Row::Option(_, Control::Select(s)), Some(r)) => s.render(r, buf, ctx, bg),
                // scrolled out: still a Tab stop, so focus scrolls it in
                (Row::Option(_, c), None) => ctx.focus_stop(c.id()),
                _ => {}
            }
        }
    }

    /// A focus change: an open list closes when its select loses focus.
    pub fn on_blur(&mut self, from: Option<WidgetId>) -> Outcome {
        let mut out = Outcome::Ignored;
        for (_, c) in self.controls_mut() {
            if let Control::Select(s) = c
                && Some(s.id) == from
            {
                out = out.or(s.on_blur());
            }
        }
        out
    }

    /// Route `ev` to the control it targets. Returns the outcome and the
    /// option it changed, if any; `None` when the view does not own it.
    pub fn handle(&mut self, ev: &PageEvent, cx: &mut Cx) -> Option<(Outcome, Option<OptSet>)> {
        if let Some(o) = self.sections.handle(ev) {
            return Some((o, None));
        }
        let changed = |opt: &'static Opt, i: usize| {
            Some(OptSet {
                key: opt.key,
                value: i,
            })
        };
        let focus = cx.focus.current();
        for (opt, c) in self.controls_mut() {
            match (c, ev) {
                (Control::Toggle(t), PageEvent::Key(key)) if focus == Some(t.id) => {
                    let o = t.on_key(key);
                    let set = (o == Outcome::Changed).then(|| changed(opt, usize::from(t.on)));
                    return Some((o, set.flatten()));
                }
                (Control::Toggle(t), PageEvent::Pointer(p)) => {
                    let Some(r) = t.route(p) else { continue };
                    if p.is(Phase::Click) {
                        cx.focus.focus(t.id);
                    }
                    let o = r.outcome();
                    let set = (o == Outcome::Changed).then(|| changed(opt, usize::from(t.on)));
                    return Some((o, set.flatten()));
                }
                (Control::Select(s), PageEvent::Key(key)) if focus == Some(s.id) => {
                    let (o, a) = s.on_key(key);
                    let set = a.and_then(|SelectAction::Changed(i)| changed(opt, i));
                    return Some((o, set));
                }
                (Control::Select(s), PageEvent::Pointer(p)) => {
                    let Some(r) = s.route(p) else { continue };
                    // a press opens the list, so it takes focus with it
                    if p.id == s.id && (p.is(Phase::Press) || p.is(Phase::Click)) {
                        cx.focus.focus(s.id);
                    }
                    let (o, a) = r.into_parts();
                    let set = a.and_then(|SelectAction::Changed(i)| changed(opt, i));
                    return Some((o, set));
                }
                _ => {}
            }
        }
        None
    }

    /// Footer hints for the focused control; `None` when focus is not in
    /// the view.
    pub fn hints(&self, focus: Option<WidgetId>) -> Option<Vec<Hint>> {
        let (_, c) = self.controls().find(|(_, c)| Some(c.id()) == focus)?;
        Some(match c {
            Control::Toggle(_) => vec![("Space", "Toggle")],
            Control::Select(s) if s.open => {
                vec![("↑↓", "Choose"), ("Enter", "Set"), ("Esc", "Close")]
            }
            Control::Select(_) => vec![("Enter", "Open"), ("↑↓", "Choose")],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_unique_and_scoped() {
        let mut seen = std::collections::HashSet::new();
        for o in all() {
            assert!(seen.insert(o.key), "duplicate option {}", o.key);
            let (scope, name) = o.key.split_once('.').expect("scoped key");
            assert!(!name.is_empty());
            assert!(
                scope == "shell" || PageId::from_name(scope).is_some(),
                "{}",
                o.key
            );
            assert!(o.default < o.values.len(), "{}", o.key);
            for (k, v) in o.needs {
                assert!(find(k).and_then(|n| n.index_of(v)).is_some(), "{}", o.key);
            }
        }
    }

    #[test]
    fn parse_opt_accepts_and_explains() {
        assert_eq!(
            parse_opt("shell.color=mono"),
            Ok(OptSet {
                key: "shell.color",
                value: 3
            })
        );
        assert_eq!(parse_opt(" Shell.Glyphs = ASCII ").map(|s| s.value), Ok(1));
        let unknown = parse_opt("shell.bogus=1").unwrap_err();
        assert!(
            unknown.contains("unknown option \"shell.bogus\""),
            "{unknown}"
        );
        assert!(
            unknown.contains("shell.color, shell.overrides"),
            "{unknown}"
        );
        let bad = parse_opt("shell.color=purple").unwrap_err();
        assert!(
            bad.contains("valid values: truecolor, 256, 16, mono"),
            "{bad}"
        );
        assert!(parse_opt("shell.color").unwrap_err().contains("KEY=VALUE"));
    }
}
