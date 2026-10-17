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
//! - [`Shown::StateRow`]: a row of a page's States grid (SC2);
//! - [`Shown::Via`]: a function covered by the demo of its owning type
//!   ([`VIA`]), valid only while that owner is itself declared directly
//!   (`Demo`, `Option` or `StateRow`).
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
    /// A function covered by the demo of its owning type (the type's
    /// capability id). Only from [`VIA`], through [`declarations`].
    Via(&'static str),
}

use Shown::{Demo, Option, StateRow};
use ViaRole::{Operation, Plumbing, Protocol};

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
    // SC25: Diff and Terminal
    (
        "widgets::diff::DiffLineKind",
        Demo("diff", "+ attempts = 5"),
    ),
    (
        "widgets::diff::DiffLineKind::Context",
        Demo("diff", "[service.worker_0]"),
    ),
    (
        "widgets::diff::DiffLineKind::Add",
        Demo("diff", "+ attempts = 5"),
    ),
    (
        "widgets::diff::DiffLineKind::Remove",
        Demo("diff", "- attempts = 3"),
    ),
    ("widgets::diff::DiffLine", Demo("diff", "- attempts = 3")),
    ("widgets::diff::DiffHunk", Demo("diff", "@@ -1,6 +1,7 @@")),
    (
        "widgets::diff::DiffStatus",
        Demo("diff", "M config/service.toml"),
    ),
    (
        "widgets::diff::DiffStatus::Added",
        Demo("diff", "A docs/retries.md"),
    ),
    (
        "widgets::diff::DiffStatus::Modified",
        Demo("diff", "M config/service.toml"),
    ),
    (
        "widgets::diff::DiffStatus::Deleted",
        Demo("diff", "D src/legacy_backoff.rs"),
    ),
    (
        "widgets::diff::DiffStatus::Renamed",
        Demo("diff", "R src/retry.rs →"),
    ),
    ("widgets::diff::DiffFile", Demo("diff", "+10 −5")),
    (
        "widgets::diff::DiffMode::toggled",
        Option("diff", "diff.mode"),
    ),
    (
        "widgets::steps::StepState::Done",
        StateRow("terminal", "rest"),
    ),
    (
        "widgets::steps::StepState::Skipped",
        StateRow("terminal", "rest"),
    ),
    (
        "widgets::steps::StepState::Failed",
        StateRow("terminal", "rest"),
    ),
    (
        "widgets::steps::StepState::Blocked",
        StateRow("terminal", "rest"),
    ),
    (
        "widgets::steps::StepRail::nav",
        Option("terminal", "terminal.rail_nav"),
    ),
    (
        "widgets::steps::StepRail::track_click",
        Option("terminal", "terminal.track_click"),
    ),
    (
        "widgets::steps::StepRail::selectable",
        Option("terminal", "terminal.selectable"),
    ),
    (
        "widgets::viewport::Span",
        Demo("terminal", "profile dev · config jackin.toml"),
    ),
    (
        "widgets::viewport::Span::bold",
        Demo("terminal", "payments-platform ❯ jackin launch"),
    ),
    (
        "widgets::viewport::Span::italic",
        Demo("terminal", "profile dev"),
    ),
    (
        "widgets::viewport::Span::underline",
        Demo("terminal", "config jackin.toml"),
    ),
    (
        "widgets::viewport::Span::reversed",
        Demo("terminal", "LIVE"),
    ),
    (
        "widgets::viewport::CellPos",
        StateRow("terminal", "selected"),
    ),
    (
        "widgets::viewport::Mark",
        Option("terminal", "terminal.mark"),
    ),
    (
        "widgets::viewport::WorkCounters",
        Demo("terminal", "viewport work:"),
    ),
    (
        "widgets::viewport::ViewportCmd",
        Option("terminal", "terminal.keymap"),
    ),
    (
        "widgets::viewport::ViewportCmd::ToggleFollow",
        Option("terminal", "terminal.keymap"),
    ),
    (
        "widgets::viewport::ViewportCmd::Copy",
        Option("terminal", "terminal.keymap"),
    ),
    (
        "widgets::viewport::ViewportCmd::SelectAll",
        Option("terminal", "terminal.keymap"),
    ),
    (
        "widgets::viewport::TextViewport::nav",
        Option("terminal", "terminal.nav"),
    ),
    (
        "widgets::viewport::TextViewport::keymap",
        Option("terminal", "terminal.keymap"),
    ),
    (
        "widgets::viewport::TextViewport::horizontal",
        Option("terminal", "terminal.horizontal"),
    ),
    (
        "widgets::viewport::TextViewport::track_click",
        Option("terminal", "terminal.track_click"),
    ),
    (
        "widgets::viewport::TextViewport::scrollbar_gap",
        Option("terminal", "terminal.scrollbar_gap"),
    ),
    (
        "widgets::viewport::TextViewport::wrap",
        Option("terminal", "terminal.wrap"),
    ),
    (
        "widgets::viewport::TextViewport::max_lines",
        Option("terminal", "terminal.max_lines"),
    ),
    // SC24: Code editor
    ("widgets::code::Severity", StateRow("codeeditor", "error")),
    (
        "widgets::code::Severity::Error",
        StateRow("codeeditor", "error"),
    ),
    (
        "widgets::code::Severity::Warning",
        Demo("codeeditor", "unused variable"),
    ),
    ("widgets::code::Diagnostic", StateRow("codeeditor", "error")),
    (
        "widgets::code::FindState",
        Demo("codeeditor", "Find closed"),
    ),
    (
        "widgets::code::CodeCmd",
        Option("codeeditor", "codeeditor.keymap"),
    ),
    (
        "widgets::code::CodeCmd::Edit",
        Option("codeeditor", "codeeditor.keymap"),
    ),
    (
        "widgets::code::CodeCmd::Append",
        Option("codeeditor", "codeeditor.keymap"),
    ),
    (
        "widgets::code::CodeCmd::PrevBlock",
        Option("codeeditor", "codeeditor.keymap"),
    ),
    (
        "widgets::code::CodeCmd::NextBlock",
        Option("codeeditor", "codeeditor.keymap"),
    ),
    (
        "widgets::code::CodeCmd::Find",
        Option("codeeditor", "codeeditor.keymap"),
    ),
    (
        "widgets::code::CodeCmd::NextMatch",
        Option("codeeditor", "codeeditor.keymap"),
    ),
    (
        "widgets::code::CodeCmd::PrevMatch",
        Option("codeeditor", "codeeditor.keymap"),
    ),
    (
        "widgets::code::CodeEditor::edit_entry",
        Option("codeeditor", "codeeditor.edit_entry"),
    ),
    (
        "widgets::code::CodeEditor::esc_reverts",
        Option("codeeditor", "codeeditor.esc_reverts"),
    ),
    (
        "widgets::code::CodeEditor::tab_leaves",
        Option("codeeditor", "codeeditor.tab_leaves"),
    ),
    (
        "widgets::code::CodeEditor::text_keymap",
        Option("codeeditor", "codeeditor.text_keymap"),
    ),
    (
        "widgets::code::CodeEditor::field_keys",
        Option("codeeditor", "codeeditor.field_keys"),
    ),
    (
        "widgets::code::CodeEditor::nav",
        Option("codeeditor", "codeeditor.nav"),
    ),
    (
        "widgets::code::CodeEditor::keymap",
        Option("codeeditor", "codeeditor.keymap"),
    ),
    (
        "widgets::code::CodeEditor::scrollbar_gap",
        Option("codeeditor", "codeeditor.scrollbar_gap"),
    ),
    (
        "widgets::code::CodeEditor::horizontal",
        Option("codeeditor", "codeeditor.horizontal"),
    ),
    (
        "widgets::code::CodeEditor::highlighter",
        Option("codeeditor", "codeeditor.highlighter"),
    ),
    (
        "widgets::code::CodeEditor::segmenter",
        Option("codeeditor", "codeeditor.segmenter"),
    ),
    (
        "widgets::code::CodeEditor::read_only",
        Option("codeeditor", "codeeditor.read_only"),
    ),
    (
        "widgets::code::CodeEditor::placeholder",
        Option("codeeditor", "codeeditor.document=empty"),
    ),
    (
        "widgets::code::CodeEditor::track_click",
        Option("codeeditor", "codeeditor.track_click"),
    ),
    (
        "widgets::option_list::Item",
        Option("codeeditor", "codeeditor.completion_rows"),
    ),
    (
        "widgets::completion::Completion",
        Option("codeeditor", "codeeditor.completion_rows"),
    ),
    (
        "widgets::completion::Completion::nav",
        Option("codeeditor", "codeeditor.completion_nav"),
    ),
    (
        "widgets::completion::Completion::max_rows",
        Option("codeeditor", "codeeditor.completion_rows"),
    ),
    (
        "widgets::completion::Completion::popup_width",
        Option("codeeditor", "codeeditor.completion_width"),
    ),
    (
        "widgets::completion::Completion::placement",
        Option("codeeditor", "codeeditor.completion_placement"),
    ),
    (
        "widgets::completion::Completion::fit",
        Option("codeeditor", "codeeditor.completion_fit"),
    ),
    (
        "widgets::completion::Completion::cursor_style",
        Option("codeeditor", "codeeditor.completion_cursor"),
    ),
    (
        "widgets::completion::Completion::dismiss",
        Option("codeeditor", "codeeditor.completion_dismiss"),
    ),
    // SC23: Splits & stacks
    (
        "ui::layout::SplitDir",
        Option("splitsstacks", "splitsstacks.dir"),
    ),
    (
        "ui::layout::SplitDir::Horizontal",
        Option("splitsstacks", "splitsstacks.dir=horizontal"),
    ),
    (
        "ui::layout::SplitDir::Vertical",
        Option("splitsstacks", "splitsstacks.dir=vertical"),
    ),
    (
        "ui::layout::Maximized",
        Option("splitsstacks", "splitsstacks.maximized"),
    ),
    (
        "ui::layout::Maximized::None",
        Option("splitsstacks", "splitsstacks.maximized=none"),
    ),
    (
        "ui::layout::Maximized::First",
        Option("splitsstacks", "splitsstacks.maximized=first"),
    ),
    (
        "ui::layout::Maximized::Second",
        Option("splitsstacks", "splitsstacks.maximized=second"),
    ),
    (
        "ui::layout::Split",
        Option("splitsstacks", "splitsstacks.percent"),
    ),
    (
        "ui::layout::Overflow",
        Option("splitsstacks", "splitsstacks.overflow"),
    ),
    (
        "ui::layout::Overflow::Scroll",
        Option("splitsstacks", "splitsstacks.overflow=scroll"),
    ),
    (
        "ui::layout::Overflow::Clip",
        Option("splitsstacks", "splitsstacks.overflow=clip"),
    ),
    (
        "ui::layout::StackChild",
        Demo("splitsstacks", "15 rows wanted, 8 given"),
    ),
    (
        "ui::layout::Stack",
        Demo("splitsstacks", "▸ Connection · 3 rows"),
    ),
    (
        "ui::layout::StackScroll",
        Option("splitsstacks", "splitsstacks.min_child_rows=3"),
    ),
    (
        "ui::layout::StackLayout",
        Demo("splitsstacks", "▸ Danger zone · 1 row"),
    ),
    (
        "ui::layout::Stack::gap",
        Option("splitsstacks", "splitsstacks.gap"),
    ),
    (
        "ui::layout::Stack::overflow",
        Option("splitsstacks", "splitsstacks.overflow"),
    ),
    (
        "ui::layout::Stack::min_child_rows",
        Option("splitsstacks", "splitsstacks.min_child_rows"),
    ),
    (
        "ui::layout::Stack::scrollbar_gap",
        Option("splitsstacks", "splitsstacks.scrollbar_gap"),
    ),
    (
        "ui::layout::Stack::track_click",
        Option("splitsstacks", "splitsstacks.track_click"),
    ),
    (
        "widgets::splitter::SeamDoubleClick",
        Option("splitsstacks", "splitsstacks.double_click"),
    ),
    (
        "widgets::splitter::SeamDoubleClick::Reset",
        Option("splitsstacks", "splitsstacks.double_click=reset"),
    ),
    (
        "widgets::splitter::SeamDoubleClick::Ignore",
        Option("splitsstacks", "splitsstacks.double_click=ignore"),
    ),
    (
        "widgets::splitter::Splitter::hit_pad",
        Option("splitsstacks", "splitsstacks.hit_pad"),
    ),
    (
        "widgets::splitter::Splitter::double_click",
        Option("splitsstacks", "splitsstacks.double_click"),
    ),
    // SC45: Pane tree
    (
        "ui::layout::SplitCmd::Up",
        Demo("panetree", "Alt+Shift+arrows resize"),
    ),
    (
        "ui::layout::SplitCmd::Down",
        Demo("panetree", "Alt+Shift+arrows resize"),
    ),
    (
        "ui::pane_tree::PaneNode",
        Option("panetree", "panetree.layout"),
    ),
    (
        "ui::pane_tree::PaneNode::Leaf",
        Option("panetree", "panetree.layout=single"),
    ),
    (
        "ui::pane_tree::PaneNode::Split",
        Option("panetree", "panetree.layout=grid"),
    ),
    (
        "ui::pane_tree::Seam",
        Option("panetree", "panetree.hit_pad"),
    ),
    (
        "ui::pane_tree::Direction",
        Demo("panetree", "Alt+arrows move"),
    ),
    (
        "ui::pane_tree::Direction::Left",
        Demo("panetree", "Alt+arrows move"),
    ),
    (
        "ui::pane_tree::Direction::Right",
        Demo("panetree", "Alt+arrows move"),
    ),
    (
        "ui::pane_tree::Direction::Up",
        Demo("panetree", "Alt+arrows move"),
    ),
    (
        "ui::pane_tree::Direction::Down",
        Demo("panetree", "Alt+arrows move"),
    ),
    (
        "ui::pane_tree::PaneCmd",
        Option("panetree", "panetree.keys"),
    ),
    (
        "ui::pane_tree::PaneCmd::Next",
        Demo("panetree", "F6 next pane"),
    ),
    (
        "ui::pane_tree::PaneCmd::Prev",
        Option("panetree", "panetree.keys=custom"),
    ),
    (
        "ui::pane_tree::PaneCmd::FocusLeft",
        Demo("panetree", "Alt+arrows move"),
    ),
    (
        "ui::pane_tree::PaneCmd::FocusRight",
        Demo("panetree", "Alt+arrows move"),
    ),
    (
        "ui::pane_tree::PaneCmd::FocusUp",
        Demo("panetree", "Alt+arrows move"),
    ),
    (
        "ui::pane_tree::PaneCmd::FocusDown",
        Demo("panetree", "Alt+arrows move"),
    ),
    (
        "ui::pane_tree::PaneCmd::Zoom",
        Demo("panetree", "Alt+Z zoom"),
    ),
    (
        "ui::pane_tree::Frames",
        Option("panetree", "panetree.frames"),
    ),
    (
        "ui::pane_tree::Frames::WhenSplit",
        Option("panetree", "panetree.frames=when-split"),
    ),
    (
        "ui::pane_tree::Frames::Always",
        Option("panetree", "panetree.frames=always"),
    ),
    (
        "ui::pane_tree::Frames::Never",
        Option("panetree", "panetree.frames=never"),
    ),
    ("ui::pane_tree::PaneTree", StateRow("panetree", "focused")),
    (
        "ui::pane_tree::PaneTree::keys",
        Option("panetree", "panetree.keys"),
    ),
    (
        "ui::pane_tree::PaneTree::split_keys",
        Option("panetree", "panetree.step"),
    ),
    (
        "ui::pane_tree::PaneTree::min_size",
        Option("panetree", "panetree.min"),
    ),
    (
        "ui::pane_tree::PaneTree::frames",
        Option("panetree", "panetree.frames"),
    ),
    (
        "ui::pane_tree::PaneTree::hit_pad",
        Option("panetree", "panetree.hit_pad"),
    ),
    // SC38: Master-detail
    (
        "ui::layout::SplitCmd",
        Option("masterdetail", "masterdetail.keys"),
    ),
    (
        "ui::layout::SplitCmd::Left",
        Option("masterdetail", "masterdetail.keys=custom"),
    ),
    (
        "ui::layout::SplitCmd::Right",
        Option("masterdetail", "masterdetail.keys=custom"),
    ),
    (
        "ui::layout::SplitKeys",
        Option("masterdetail", "masterdetail.keys"),
    ),
    (
        "ui::layout::SplitKeys::keymap",
        Option("masterdetail", "masterdetail.keys"),
    ),
    (
        "ui::layout::SplitKeys::step",
        Option("masterdetail", "masterdetail.step"),
    ),
    (
        "ui::master_detail::DetailSize",
        Option("masterdetail", "masterdetail.size"),
    ),
    (
        "ui::master_detail::DetailSize::Share",
        Option("masterdetail", "masterdetail.size=share"),
    ),
    (
        "ui::master_detail::DetailSize::Resizable",
        Option("masterdetail", "masterdetail.size=resizable"),
    ),
    (
        "ui::master_detail::Mode",
        Option("masterdetail", "masterdetail.width"),
    ),
    (
        "ui::master_detail::Mode::Split",
        Option("masterdetail", "masterdetail.width=fill"),
    ),
    (
        "ui::master_detail::Mode::Master",
        Option("masterdetail", "masterdetail.width=72"),
    ),
    (
        "ui::master_detail::Mode::Drawer",
        Demo("masterdetail", "Show details"),
    ),
    (
        "ui::master_detail::MdLayout",
        Demo("masterdetail", "╭─ Tables"),
    ),
    (
        "ui::master_detail::MasterDetail",
        StateRow("masterdetail", "narrow"),
    ),
    (
        "ui::master_detail::MasterDetail::collapse_below",
        Option("masterdetail", "masterdetail.collapse"),
    ),
    (
        "ui::master_detail::MasterDetail::detail_share",
        Option("masterdetail", "masterdetail.size=share"),
    ),
    (
        "ui::master_detail::MasterDetail::resizable",
        Option("masterdetail", "masterdetail.size=resizable"),
    ),
    (
        "ui::master_detail::MasterDetail::seam",
        Demo("masterdetail", "╭─ orders"),
    ),
    (
        "ui::master_detail::MasterDetail::keys",
        Option("masterdetail", "masterdetail.keys"),
    ),
    (
        "ui::master_detail::MasterDetail::gap",
        Option("masterdetail", "masterdetail.gap"),
    ),
    (
        "ui::master_detail::MasterDetail::summary_rows",
        Option("masterdetail", "masterdetail.summary"),
    ),
    (
        "ui::master_detail::MasterDetail::esc_returns",
        Option("masterdetail", "masterdetail.esc_returns"),
    ),
    (
        "ui::master_detail::MasterDetail::detail_owns",
        Demo("masterdetail", "Open table"),
    ),
    // SC21: Progress, Empty & error
    (
        "widgets::progress::BarLayout",
        Option("progress", "progress.label_width"),
    ),
    (
        "widgets::progress::BarLayout::label_width",
        Option("progress", "progress.label_width"),
    ),
    (
        "widgets::progress::BarLayout::reserve_value",
        Option("progress", "progress.reserve_value"),
    ),
    (
        "widgets::progress::ProgressBar",
        StateRow("progress", "loading"),
    ),
    (
        "widgets::progress::ProgressBar::status",
        Demo("progress", "Integration ━"),
    ),
    (
        "widgets::progress::ProgressBar::layout",
        Option("progress", "progress.label_width"),
    ),
    (
        "widgets::progress::Tally",
        Demo("progress", "2 of 7 done · 3 running · 1 paused · 1 failed"),
    ),
    (
        "widgets::empty::EmptyKind",
        Option("emptyerror", "emptyerror.kind"),
    ),
    (
        "widgets::empty::EmptyKind::Empty",
        Option("emptyerror", "emptyerror.kind=empty"),
    ),
    (
        "widgets::empty::EmptyKind::Error",
        Option("emptyerror", "emptyerror.kind=error"),
    ),
    (
        "widgets::empty::EmptyAction",
        Option("emptyerror", "emptyerror.actions"),
    ),
    (
        "widgets::empty::EmptyState::actions",
        Option("emptyerror", "emptyerror.actions"),
    ),
    (
        "widgets::empty::EmptyState::hint",
        Option("emptyerror", "emptyerror.hint"),
    ),
    // SC26: Text utilities
    (
        "core::text::TextBuffer",
        Demo(
            "textutilities",
            "TextBuffer · the model under every text control",
        ),
    ),
    (
        "core::text::CursorPos",
        Demo("textutilities", "CursorPos { line: 1, col: 10 }"),
    ),
    (
        "ui::text::WrapOpts",
        Option("textutilities", "textutilities.wrap"),
    ),
    (
        "ui::text::DurationStyle",
        Option("textutilities", "textutilities.duration"),
    ),
    (
        "ui::text::DurationStyle::Compact",
        Option("textutilities", "textutilities.duration=compact"),
    ),
    (
        "ui::text::DurationStyle::Spoken",
        Option("textutilities", "textutilities.duration=spoken"),
    ),
    (
        "widgets::props::Prop::copyable",
        StateRow("textutilities", "focused"),
    ),
    (
        "widgets::props::Prop::tone",
        Demo("textutilities", "Health healthy"),
    ),
    (
        "widgets::props::Prop::wrap",
        Demo("textutilities", "Prop::wrap a value marked wrap"),
    ),
    (
        "widgets::props::Sheet",
        Option("textutilities", "textutilities.sheet"),
    ),
    (
        "widgets::props::PropsAction",
        Demo("textutilities", "last PropsAction: —"),
    ),
    (
        "widgets::props::PropsAction::Copy",
        Demo("textutilities", "Copy(row) on the copy key"),
    ),
    (
        "widgets::props::PropsAction::Activate",
        Demo("textutilities", "(Activate(row) on Enter"),
    ),
    (
        "widgets::props::PropsList",
        Demo("textutilities", "PropsList · nav default"),
    ),
    (
        "widgets::props::PropsCmd",
        Option("textutilities", "textutilities.copy_key"),
    ),
    (
        "widgets::props::PropsCmd::Copy",
        Option("textutilities", "textutilities.copy_key"),
    ),
    (
        "widgets::props::PropsList::keymap",
        Option("textutilities", "textutilities.copy_key"),
    ),
    (
        "widgets::props::PropsList::nav",
        Option("textutilities", "textutilities.nav"),
    ),
    (
        "widgets::props::PropsList::track_click",
        Option("textutilities", "textutilities.track_click"),
    ),
    (
        "widgets::props::PropsList::scrollbar_gap",
        Option("textutilities", "textutilities.scrollbar_gap"),
    ),
    (
        "widgets::props::PropsList::sheet",
        Option("textutilities", "textutilities.sheet"),
    ),
    (
        "widgets::props::PropsList::reveal",
        Option("textutilities", "textutilities.reveal"),
    ),
    // SC4: Chrome, Status & hints
    (
        "widgets::hintbar::HintLayer",
        Demo("statushints", "screen layer"),
    ),
    (
        "widgets::hintbar::HintLayer::align",
        Option("statushints", "statushints.align"),
    ),
    (
        "widgets::hintbar::HintLayer::badge",
        Option("statushints", "statushints.badge"),
    ),
    (
        "widgets::hintbar::HintLayer::status",
        Option("statushints", "statushints.status"),
    ),
    (
        "widgets::keyhint::Hint",
        Demo("statushints", "Priorities · HintPriority"),
    ),
    (
        "widgets::keyhint::HintPriority",
        Option("statushints", "statushints.priority"),
    ),
    (
        "widgets::keyhint::HintPriority::Auto",
        Option("statushints", "statushints.priority=auto"),
    ),
    (
        "widgets::keyhint::HintPriority::Normal",
        Option("statushints", "statushints.priority=normal"),
    ),
    (
        "widgets::keyhint::HintPriority::Pinned",
        Option("statushints", "statushints.priority=pinned"),
    ),
    (
        "widgets::keyhint::Hint::priority",
        Option("statushints", "statushints.priority"),
    ),
    (
        "widgets::keyhint::Hint::pinned",
        Demo("statushints", "Hint::pinned keeps F5"),
    ),
    (
        "theme::HintAlign",
        Option("statushints", "statushints.align"),
    ),
    (
        "theme::HintAlign::Start",
        Demo("statushints", "HintAlign::Start"),
    ),
    (
        "theme::HintAlign::Center",
        Demo("statushints", "HintAlign::Center"),
    ),
    (
        "theme::HintBarStyle",
        Demo("statushints", "HintBarStyle · align Start"),
    ),
    // the shell pieces on Status & hints (SC30)
    (
        "theme::Theme::centred_hints",
        Demo("statushints", "Theme::centred_hints"),
    ),
    (
        "widgets::too_small::TooSmall",
        Demo("statushints", "│ Terminal too small │"),
    ),
    (
        "widgets::too_small::TooSmall::min",
        Option("statushints", "statushints.too_small_min"),
    ),
    (
        "widgets::too_small::TooSmall::quit_hint",
        Option("statushints", "statushints.quit_hint"),
    ),
    (
        "widgets::too_small::TooSmall::brand_style",
        Option("statushints", "statushints.brand"),
    ),
    (
        "widgets::menu::MenuBarAction",
        Demo("chrome", "last MenuBarAction: —"),
    ),
    (
        "widgets::menu::MenuBarAction::Opened",
        Demo("chrome", "(Opened(menu)"),
    ),
    (
        "widgets::menu::MenuBarAction::Chosen",
        Demo("chrome", "Chosen(menu, item)"),
    ),
    (
        "widgets::menu::MenuBarAction::Closed",
        Demo("chrome", "· Closed ·"),
    ),
    (
        "widgets::menu::MenuBarAction::Brand",
        Demo("chrome", "Closed · Brand)"),
    ),
    ("widgets::menu::OpenOn", Option("chrome", "chrome.open_on")),
    (
        "widgets::menu::OpenOn::Press",
        Option("chrome", "chrome.open_on=press"),
    ),
    (
        "widgets::menu::OpenOn::Release",
        Option("chrome", "chrome.open_on=release"),
    ),
    (
        "widgets::menu::Overflow",
        Option("chrome", "chrome.overflow"),
    ),
    (
        "widgets::menu::Overflow::Hide",
        Option("chrome", "chrome.overflow=hide"),
    ),
    (
        "widgets::menu::Overflow::Chevron",
        Option("chrome", "chrome.overflow=chevron"),
    ),
    (
        "widgets::menu::MenuBar::nav",
        Option("chrome", "chrome.nav"),
    ),
    (
        "widgets::menu::MenuBar::open_on",
        Option("chrome", "chrome.open_on"),
    ),
    (
        "widgets::menu::MenuBar::press_drag",
        Option("chrome", "chrome.press_drag"),
    ),
    (
        "widgets::menu::MenuBar::type_ahead",
        Option("chrome", "chrome.type_ahead"),
    ),
    (
        "widgets::menu::MenuBar::overflow",
        Option("chrome", "chrome.overflow"),
    ),
    (
        "widgets::menu::MenuBar::hide_owner_focus",
        Option("chrome", "chrome.hide_owner_focus"),
    ),
    (
        "widgets::menu::MenuBar::dismiss",
        Option("chrome", "chrome.dismiss"),
    ),
    (
        "widgets::menu::MenuBar::wrap",
        Option("chrome", "chrome.wrap"),
    ),
    (
        "widgets::menu::MenuBar::hover_switch",
        Option("chrome", "chrome.hover_switch"),
    ),
    (
        "widgets::menu::MenuBar::brand",
        Option("chrome", "chrome.brand"),
    ),
    (
        "widgets::segments::Segment::bold",
        Demo("chrome", "payments-platform ◆ production"),
    ),
    (
        "widgets::segments::Segment::clickable",
        Demo("chrome", "Clickable segment"),
    ),
    (
        "widgets::segments::Segment::priority",
        Demo("chrome", "identity strip · segments drop by priority"),
    ),
    (
        "widgets::segments::Segment::lead",
        Demo("chrome", "◆ production ◇ staging"),
    ),
    (
        "widgets::statusbar::Emphasis",
        Demo("chrome", "Status chip"),
    ),
    (
        "widgets::statusbar::Emphasis::Plain",
        Demo("chrome", "PR #482 · settlement backoff"),
    ),
    (
        "widgets::statusbar::Emphasis::Strong",
        Demo("chrome", "payments-platform PR #482"),
    ),
    (
        "widgets::statusbar::Emphasis::Chip",
        Demo("chrome", "Weekly 59%"),
    ),
    (
        "widgets::statusbar::StatusItem::busy",
        StateRow("chrome", "busy"),
    ),
    (
        "widgets::statusbar::StatusItem::meter",
        Demo("chrome", "disk ━"),
    ),
    (
        "widgets::statusbar::StatusItem::meter_stale",
        Demo("chrome", "quota ╍"),
    ),
    (
        "widgets::statusbar::StatusItem::priority",
        Demo("chrome", "StatusBar::layout places 8 of 8"),
    ),
    (
        "widgets::statusbar::StatusItem::clickable",
        StateRow("chrome", "hover"),
    ),
    (
        "widgets::statusbar::StatusItem::strong",
        Demo("chrome", "payments-platform PR #482"),
    ),
    (
        "widgets::statusbar::StatusItem::chip",
        Demo("chrome", "Weekly 59%"),
    ),
    (
        "widgets::statusbar::Group",
        Demo("chrome", "places 8 of 8 · Left"),
    ),
    ("widgets::statusbar::Group::Left", Demo("chrome", "Left 2,")),
    (
        "widgets::statusbar::Group::Center",
        Demo("chrome", "Center 1,"),
    ),
    (
        "widgets::statusbar::Group::Right",
        Demo("chrome", "Right 5"),
    ),
    (
        "widgets::statusbar::Placed",
        Demo("chrome", "StatusBar::layout places"),
    ),
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
        Demo("chrome", "app❯ File Edit View Session Help"),
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
    // the shell footer (M1): every hint naming one key is a click target
    // that replays it (`HintBar::render_ctx`)
    (
        "widgets::hintbar::HintKeys",
        Demo("overview", "Tab Into page"),
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
    // SC10 Chips & selects
    (
        "widgets::chips::ChipCmd",
        Option("chipsselects", "chipsselects.chip_keymap"),
    ),
    (
        "widgets::chips::ChipCmd::Remove",
        Option("chipsselects", "chipsselects.chip_keymap=custom"),
    ),
    (
        "widgets::chips::ChipCmd::Add",
        Option("chipsselects", "chipsselects.chip_keymap=custom"),
    ),
    (
        "widgets::chips::ChipCmd::ClearAll",
        Option("chipsselects", "chipsselects.chip_keymap=custom"),
    ),
    (
        "widgets::chips::ChipCmd::Lead",
        Option("chipsselects", "chipsselects.chip_keymap=custom"),
    ),
    (
        "widgets::chips::ChipBar::close_target_w",
        Option("chipsselects", "chipsselects.close_target"),
    ),
    (
        "widgets::chips::ChipBar::close_button",
        Option("chipsselects", "chipsselects.close_button"),
    ),
    (
        "widgets::chips::ChipBar::overflow_menu",
        Option("chipsselects", "chipsselects.overflow_menu"),
    ),
    (
        "widgets::chips::ChipBar::more_button",
        Option("chipsselects", "chipsselects.more_button"),
    ),
    (
        "widgets::chips::ChipBar::nav",
        Option("chipsselects", "chipsselects.chip_nav"),
    ),
    (
        "widgets::chips::ChipBar::keymap",
        Option("chipsselects", "chipsselects.chip_keymap"),
    ),
    (
        "widgets::chips::ChipBar::overflow_marker",
        Option("chipsselects", "chipsselects.overflow_marker"),
    ),
    (
        "widgets::chips::ChipBar::digit_toggles",
        Option("chipsselects", "chipsselects.digit_toggles"),
    ),
    (
        "widgets::option_list::SelectArrows",
        Option("chipsselects", "chipsselects.arrows"),
    ),
    (
        "widgets::option_list::SelectArrows::Step",
        Option("chipsselects", "chipsselects.arrows=step"),
    ),
    (
        "widgets::option_list::SelectArrows::Open",
        Option("chipsselects", "chipsselects.arrows=open"),
    ),
    (
        "widgets::option_list::WheelMode",
        Option("chipsselects", "chipsselects.wheel"),
    ),
    (
        "widgets::option_list::WheelMode::Viewport",
        Option("chipsselects", "chipsselects.wheel=viewport"),
    ),
    (
        "widgets::option_list::WheelMode::Cursor",
        Option("chipsselects", "chipsselects.wheel=cursor"),
    ),
    (
        "widgets::select::TabInOpen",
        Option("chipsselects", "chipsselects.tab_in_open"),
    ),
    (
        "widgets::select::TabInOpen::Choose",
        Option("chipsselects", "chipsselects.tab_in_open=choose"),
    ),
    (
        "widgets::select::TabInOpen::Close",
        Option("chipsselects", "chipsselects.tab_in_open=close"),
    ),
    (
        "widgets::select::TabInOpen::Swallow",
        Option("chipsselects", "chipsselects.tab_in_open=swallow"),
    ),
    (
        "widgets::select::Select::wheel",
        Option("chipsselects", "chipsselects.wheel"),
    ),
    (
        "widgets::select::Select::hover_moves_cursor",
        Option("chipsselects", "chipsselects.hover_moves_cursor"),
    ),
    (
        "widgets::select::Select::press_drag",
        Option("chipsselects", "chipsselects.press_drag"),
    ),
    (
        "widgets::select::Select::label_hit",
        Option("chipsselects", "chipsselects.label_hit"),
    ),
    (
        "widgets::select::Select::track_click",
        Option("chipsselects", "chipsselects.track_click"),
    ),
    (
        "widgets::select::Select::nav",
        Option("chipsselects", "chipsselects.select_nav"),
    ),
    (
        "widgets::select::Select::dismiss",
        Option("chipsselects", "chipsselects.dismiss"),
    ),
    (
        "widgets::select::Select::max_rows",
        Option("chipsselects", "chipsselects.max_rows"),
    ),
    (
        "widgets::select::Select::placement",
        Option("chipsselects", "chipsselects.placement"),
    ),
    (
        "widgets::select::Select::popup_fit",
        Option("chipsselects", "chipsselects.popup_fit"),
    ),
    (
        "widgets::select::Select::popup_width",
        Option("chipsselects", "chipsselects.popup_width"),
    ),
    (
        "widgets::select::Select::cursor_style",
        Option("chipsselects", "chipsselects.cursor_style"),
    ),
    (
        "widgets::select::Select::mixed",
        Option("chipsselects", "chipsselects.mixed"),
    ),
    (
        "widgets::select::Select::wrap",
        Option("chipsselects", "chipsselects.wrap"),
    ),
    (
        "widgets::select::Select::type_ahead",
        Option("chipsselects", "chipsselects.type_ahead"),
    ),
    (
        "widgets::select::Select::arrows",
        Option("chipsselects", "chipsselects.arrows"),
    ),
    (
        "widgets::select::Select::close_on_blur",
        Option("chipsselects", "chipsselects.close_on_blur"),
    ),
    (
        "widgets::select::Select::tab_in_open",
        Option("chipsselects", "chipsselects.tab_in_open"),
    ),
    (
        "widgets::select::Select::help",
        Demo("chipsselects", "Applies to the next query"),
    ),
    (
        "widgets::select::Select::disabled",
        Demo("chipsselects", "Fixed by the connection"),
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
        "widgets::edit::Validator",
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
    // S17 (SC36): the readout inside the field
    (
        "widgets::input::TextInput::readout",
        Option("inputs", "inputs.readout"),
    ),
    (
        "widgets::input::TextInput::readout_link",
        Demo("finder", "scope ‹ here ›"),
    ),
    (
        "widgets::input::TextInput::set_readout",
        Option("finder", "finder.scope"),
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
    // Forms on the template (Priority 0): its option table
    (
        "widgets::input::TextInput::validate_when",
        Option("forms", "forms.validate_when"),
    ),
    (
        "widgets::input::TextInput::required",
        Option("forms", "forms.required"),
    ),
    (
        "widgets::textarea::TextArea::blur_action",
        Option("forms", "forms.blur"),
    ),
    (
        "widgets::choice::Toggle::disabled",
        Option("forms", "forms.managed"),
    ),
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
        Demo("editable", "Cancel the edit · Cancelled"),
    ),
    (
        "widgets::table::TableAction::Committed",
        Demo("editable", "Edit · Enter again Committed"),
    ),
    (
        "widgets::table::TableAction::Cancelled",
        Demo("editable", "Cancel the edit · Cancelled"),
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
        "widgets::table::DataTable::click",
        Option("selection", "selection.activate"),
    ),
    (
        "core::selection::ClickPolicy",
        Option("selection", "selection.activate"),
    ),
    (
        "core::selection::ClickPolicy::activate",
        Option("selection", "selection.activate"),
    ),
    (
        "core::selection::Activate",
        Option("selection", "selection.activate"),
    ),
    (
        "core::selection::Activate::Double",
        Option("selection", "selection.activate=double"),
    ),
    (
        "core::selection::Activate::ClickAgain",
        Option("selection", "selection.activate=again"),
    ),
    (
        "core::selection::Activate::Single",
        Option("selection", "selection.activate=single"),
    ),
    (
        "core::selection::Activate::Never",
        Option("selection", "selection.activate=never"),
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
        "widgets::grid::cell_validator",
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
    // Settings on the template (Priority 0): its option table and States
    (
        "widgets::table::DataTable::cell_nav",
        Option("settings", "settings.cell_nav"),
    ),
    (
        "widgets::list::SelectMode::Single",
        Option("settings", "settings.env_select=single"),
    ),
    (
        "widgets::tabs::TabItem::dirty",
        Option("settings", "settings.tab_dirty"),
    ),
    ("widgets::tabs::Tabs", StateRow("settings", "focused")),
    (
        "widgets::table::DataTable",
        StateRow("settings", "selected"),
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
        Demo("progress", "Build image ━"),
    ),
    (
        "widgets::progress::ProgressStatus::Done",
        Demo("progress", "Build image ━"),
    ),
    (
        "widgets::progress::ProgressStatus::Error",
        Demo("progress", "Integration ━"),
    ),
    (
        "widgets::progress::ProgressStatus::Paused",
        Demo("progress", "Migrate schema ━"),
    ),
    // Priority 0 Screens (SC49, SC51, SC52): what the screens show of
    // functions whose own pages come later
    (
        "widgets::progress::spinner_frame",
        Demo("activity", "running · 12 min"),
    ),
    (
        "widgets::steps::StepRail::counts",
        Demo("activity", "4 of 5"),
    ),
    (
        "widgets::viewport::TextViewport::retention",
        Option("activity", "activity.retention"),
    ),
    (
        "widgets::viewport::TextViewport::set_marks",
        Option("activity", "activity.find"),
    ),
    (
        "widgets::tree::TreeView::cursor_path",
        Demo("workspaces", "/workspace/payments-platform"),
    ),
    (
        "widgets::notice::Notice",
        Demo("notices", "Authentication failed"),
    ),
    (
        "widgets::notice::NoticeKind",
        Demo("notices", "Saved 12 rows in 84 ms"),
    ),
    (
        "widgets::notice::NoticeKind::Error",
        Demo("notices", "! Port must be a number"),
    ),
    (
        "widgets::notice::NoticeKind::Warning",
        Demo("notices", "3 rows changed outside this session"),
    ),
    (
        "widgets::notice::NoticeKind::Info",
        Demo("notices", "Read-only replica"),
    ),
    (
        "widgets::notice::NoticeKind::Success",
        Demo("notices", "✓ Saved 12 rows"),
    ),
    (
        "widgets::notice::NoticeKind::message_tone",
        Demo("notices", "the glyph carries it"),
    ),
    (
        "widgets::notice::Notice::kind",
        Option("notices", "notices.kind"),
    ),
    (
        "widgets::notice::Notice::detail",
        Option("notices", "notices.detail"),
    ),
    (
        "widgets::notice::Notice::hint",
        Option("notices", "notices.hint"),
    ),
    (
        "widgets::notice::Notice::max_lines",
        Option("notices", "notices.max_lines"),
    ),
    (
        "widgets::notice::Notice::message_lines",
        Option("notices", "notices.message_lines"),
    ),
    (
        "widgets::notice::Notice::detail_tone",
        Option("notices", "notices.detail_tone"),
    ),
    (
        "widgets::notice::Notice::mark_cut",
        Option("notices", "notices.mark_cut"),
    ),
    (
        "ui::layers::LayerResult::File",
        Demo("filebrowser", "LayerResult::File"),
    ),
    (
        "widgets::file_browser::FileEntry",
        Demo("filebrowser", "FileEntry rows of ~"),
    ),
    (
        "widgets::file_browser::FileEntry::meta",
        Demo("filebrowser", "src folder 4 items"),
    ),
    (
        "widgets::file_browser::FileEntry::hidden",
        Demo("filebrowser", ".ssh folder keys hidden"),
    ),
    (
        "widgets::file_browser::MemorySource",
        Demo("filebrowser", "MemorySource · home"),
    ),
    (
        "widgets::file_browser::MemorySource::home",
        Demo("filebrowser", "home /Users/ada"),
    ),
    (
        "widgets::file_browser::FsSource",
        Option("filebrowser", "filebrowser.source"),
    ),
    (
        "widgets::file_browser::Choose",
        Option("filebrowser", "filebrowser.choose"),
    ),
    (
        "widgets::file_browser::Choose::Dirs",
        Option("filebrowser", "filebrowser.choose"),
    ),
    (
        "widgets::file_browser::Choose::Files",
        Option("filebrowser", "filebrowser.choose"),
    ),
    (
        "widgets::file_browser::Choose::Any",
        Option("filebrowser", "filebrowser.choose"),
    ),
    (
        "widgets::file_browser::FileCmd",
        Demo("filebrowser", "FileCmd · desktop key map"),
    ),
    (
        "widgets::file_browser::FileCmd::Open",
        Demo("filebrowser", "Enter Open"),
    ),
    (
        "widgets::file_browser::FileCmd::Parent",
        Demo("filebrowser", "Parent folder"),
    ),
    (
        "widgets::file_browser::FileCmd::Choose",
        Demo("filebrowser", "Space Choose"),
    ),
    (
        "widgets::file_browser::FileCmd::ToggleHidden",
        Demo("filebrowser", ". Hidden files"),
    ),
    (
        "widgets::file_browser::FileCmd::GoTo",
        Demo("filebrowser", "/ Go to folder"),
    ),
    (
        "widgets::file_browser::FileCmd::Filter",
        Demo("filebrowser", "Ctrl+F Filter"),
    ),
    (
        "widgets::file_browser::FileCmd::Url",
        Demo("filebrowser", "unbound URL entry"),
    ),
    (
        "widgets::file_browser::FileCmd::vim_keymap",
        Option("filebrowser", "filebrowser.keys"),
    ),
    (
        "widgets::file_browser::UrlEntry",
        Option("filebrowser", "filebrowser.url_entry"),
    ),
    (
        "widgets::file_browser::UrlEntry::back",
        Option("filebrowser", "filebrowser.url_entry"),
    ),
    (
        "widgets::file_browser::UrlEntry::placeholder",
        Option("filebrowser", "filebrowser.url_entry"),
    ),
    (
        "widgets::file_browser::UrlEntry::help",
        Option("filebrowser", "filebrowser.url_entry"),
    ),
    (
        "widgets::file_browser::UrlEntry::hint",
        Option("filebrowser", "filebrowser.url_entry"),
    ),
    (
        "widgets::file_browser::UrlEntry::empty_error",
        Option("filebrowser", "filebrowser.url_entry"),
    ),
    (
        "widgets::file_browser::UrlEntry::validate",
        Option("filebrowser", "filebrowser.url_entry"),
    ),
    (
        "widgets::file_browser::FileBrowserResult",
        Demo("filebrowser", "FileBrowserResult: Chosen"),
    ),
    (
        "widgets::file_browser::FileBrowserResult::Chosen",
        Demo("filebrowser", "Chosen { path, checked }"),
    ),
    (
        "widgets::file_browser::FileBrowserResult::Url",
        Demo("filebrowser", "checked } · Url ·"),
    ),
    (
        "widgets::file_browser::FileBrowserResult::Cancelled",
        Demo("filebrowser", "Url · Cancelled"),
    ),
    (
        "widgets::file_browser::FileBrowser",
        Demo("filebrowser", "FileBrowser on the layer stack"),
    ),
    (
        "widgets::file_browser::FileBrowser::dismiss",
        Option("filebrowser", "filebrowser.dismiss"),
    ),
    (
        "widgets::file_browser::FileBrowser::choose",
        Option("filebrowser", "filebrowser.choose"),
    ),
    (
        "widgets::file_browser::FileBrowser::checkbox",
        Option("filebrowser", "filebrowser.checkbox"),
    ),
    (
        "widgets::file_browser::FileBrowser::show_hidden",
        Option("filebrowser", "filebrowser.show_hidden"),
    ),
    (
        "widgets::file_browser::FileBrowser::hidden_toggle",
        Option("filebrowser", "filebrowser.hidden_toggle"),
    ),
    (
        "widgets::file_browser::FileBrowser::path_bar",
        Option("filebrowser", "filebrowser.path_bar"),
    ),
    (
        "widgets::file_browser::FileBrowser::filter",
        Option("filebrowser", "filebrowser.filter"),
    ),
    (
        "widgets::file_browser::FileBrowser::type_ahead",
        Option("filebrowser", "filebrowser.type_ahead"),
    ),
    (
        "widgets::file_browser::FileBrowser::keys",
        Option("filebrowser", "filebrowser.keys"),
    ),
    (
        "widgets::file_browser::FileBrowser::url_entry",
        Option("filebrowser", "filebrowser.url_entry"),
    ),
    (
        "widgets::file_browser::FileBrowser::stepper",
        Option("filebrowser", "filebrowser.stepper"),
    ),
    (
        "widgets::meter_list::MeterList",
        Demo("meters", "Session (5 h)"),
    ),
    (
        "widgets::meter_list::MeterRow",
        Demo("meters", "Extra credits"),
    ),
    (
        "widgets::meter_list::MeterRow::detail",
        Option("meters", "meters.detail"),
    ),
    (
        "widgets::meter_list::MeterList::label_width",
        Option("meters", "meters.label_width"),
    ),
    (
        "widgets::meter_list::MeterList::label_gap",
        Option("meters", "meters.label_gap"),
    ),
    (
        "widgets::meter_list::MeterList::meter_width",
        Option("meters", "meters.meter_width"),
    ),
    (
        "widgets::meter_list::MeterList::detail_tone",
        Option("meters", "meters.detail_tone"),
    ),
    (
        "widgets::meter_list::MeterList::label_tone",
        Demo("meters", "/System/Volumes/Data"),
    ),
    (
        "widgets::meter_list::MeterList::pad_label",
        Demo("meters", "/Volumes/Scratch"),
    ),
    (
        "widgets::meter_list::MeterList::reserve",
        Demo("meters", "reserve 4"),
    ),
    (
        "widgets::meter_list::MeterList::min_meter",
        Demo("meters", "min 8"),
    ),
    (
        "widgets::meter_list::MeterList::detail_gap",
        Demo("meters", "detail gap 3"),
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
        Demo("progress", "Push to registry ━"),
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
    // Task runner on the template (Priority 0): its option table and States
    (
        "widgets::panel::ScrollPanel::tail",
        Option("taskrunner", "taskrunner.follow"),
    ),
    (
        "widgets::panel::ScrollPanel::wrap",
        Option("taskrunner", "taskrunner.wrap"),
    ),
    (
        "widgets::tree::TreeView::indent",
        Option("taskrunner", "taskrunner.indent"),
    ),
    (
        "widgets::tree::TreeView::show_meta",
        Option("taskrunner", "taskrunner.tree_meta"),
    ),
    ("widgets::tree::TreeView", StateRow("taskrunner", "focused")),
    (
        "widgets::panel::ScrollPanel",
        StateRow("taskrunner", "overflow"),
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
    ("theme::Charset", Option("overview", "shell.glyphs")),
    (
        "theme::Charset::Unicode",
        Option("overview", "shell.glyphs=unicode"),
    ),
    (
        "theme::Charset::Ascii",
        Option("overview", "shell.glyphs=ascii"),
    ),
    // `CommonArgs::theme` applies the detected charset; the options seed
    // from it and draw with its glyphs and copy (`App::apply_shell`)
    ("theme::Charset::glyphs", Option("overview", "shell.glyphs")),
    (
        "theme::Charset::strings",
        Option("overview", "shell.strings"),
    ),
    ("theme::Charset::apply", Option("overview", "shell.glyphs")),
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
    // ActionRow (S17, SC36): the Buttons page's Next row
    (
        "widgets::action_row::ActionRow",
        Demo("buttons", "Open logs"),
    ),
    (
        "widgets::action_row::ActionRow::align",
        Demo("buttons", "Retry failed"),
    ),
    (
        "widgets::action_row::ActionRow::gap",
        Demo("buttons", "Share"),
    ),
    (
        "widgets::action_row::ActionRow::message",
        Option("buttons", "buttons.next_message"),
    ),
    (
        "widgets::action_row::ActionRow::wrap",
        Option("buttons", "buttons.row_wrap"),
    ),
    (
        "widgets::action_row::ActionRow::layout",
        Demo("buttons", "Copy run ID"),
    ),
    (
        "widgets::action_row::ActionRow::render",
        Demo("buttons", "2 not available yet"),
    ),
    (
        "widgets::action_row::ActionRow::step",
        Demo("buttons", "Retry failed"),
    ),
    // DetailCard (S17, SC36): the Connections screen's detail pane
    (
        "widgets::detail_card::DetailCard",
        Demo("connections", "Engine"),
    ),
    (
        "widgets::detail_card::DetailLayout",
        Demo("connections", "Connect"),
    ),
    (
        "widgets::detail_card::DetailCard::meta",
        Demo("connections", "production"),
    ),
    (
        "widgets::detail_card::DetailCard::props",
        Demo("connections", "Last used"),
    ),
    (
        "widgets::detail_card::DetailCard::actions",
        Demo("connections", "Duplicate"),
    ),
    (
        "widgets::detail_card::DetailCard::max_size",
        Demo("connections", "Database"),
    ),
    (
        "widgets::detail_card::DetailCard::reserve",
        Option("connections", "connections.state"),
    ),
    (
        "widgets::detail_card::DetailCard::focused",
        Demo("connections", "Edit…"),
    ),
    (
        "widgets::detail_card::DetailCard::fit",
        Demo("connections", "SSL"),
    ),
    (
        "widgets::detail_card::DetailCard::render",
        Demo("connections", "Delete…"),
    ),
    // PageHeader and Heading (S17, SC36): the Meters page's header
    (
        "widgets::page_header::PageHeader",
        Demo("meters", "Usage and capacity"),
    ),
    (
        "widgets::page_header::PageHeader::status",
        Option("meters", "meters.offline"),
    ),
    (
        "widgets::page_header::PageHeader::right",
        Demo("meters", "refreshes every 2 s"),
    ),
    (
        "widgets::page_header::PageHeader::right_gap",
        Demo("meters", "refreshes every 2 s"),
    ),
    (
        "widgets::page_header::PageHeader::subtitle",
        Demo("meters", "Plan quotas, filesystems and host load"),
    ),
    (
        "widgets::page_header::PageHeader::inset",
        Demo("meters", "Usage and capacity · live"),
    ),
    (
        "widgets::page_header::PageHeader::height",
        Demo("meters", "in aligned columns"),
    ),
    (
        "widgets::page_header::PageHeader::body",
        Demo("meters", "Sections"),
    ),
    (
        "widgets::page_header::PageHeader::render",
        Demo("meters", "Usage and capacity"),
    ),
    ("widgets::page_header::Heading", Demo("meters", "Sections")),
    (
        "widgets::page_header::Heading::meta",
        Demo("meters", "3 cards"),
    ),
    (
        "widgets::page_header::Heading::tone",
        Demo("meters", "Sections"),
    ),
    (
        "widgets::page_header::Heading::render",
        Demo("meters", "Sections"),
    ),
    // NavList (S17, SC36): the Sidebars page's list
    ("widgets::nav_list::NavList", Demo("sidebars", "Workspace")),
    // the list's States rows (its `Specimen`)
    (
        "widgets::nav_list::NavList",
        StateRow("sidebars", "focused"),
    ),
    (
        "widgets::nav_list::NavItem",
        Demo("sidebars", "Environment"),
    ),
    (
        "widgets::nav_list::NavAction",
        Demo("sidebars", "Last: none yet"),
    ),
    (
        "widgets::nav_list::NavAction::Activated",
        Demo("sidebars", "Enter opens"),
    ),
    (
        "widgets::nav_list::NavItem::icon",
        Demo("sidebars", "T Tasks"),
    ),
    (
        "widgets::nav_list::NavItem::section",
        Demo("sidebars", "Preferences"),
    ),
    (
        "widgets::nav_list::NavItem::badge",
        Option("sidebars", "sidebars.badges"),
    ),
    (
        "widgets::nav_list::NavItem::disabled",
        Demo("sidebars", "Disabled items are skipped"),
    ),
    (
        "widgets::nav_list::NavList::current",
        Demo("sidebars", "› T Tasks"),
    ),
    (
        "widgets::nav_list::NavList::collapsed",
        Option("sidebars", "sidebars.collapsed"),
    ),
    (
        "widgets::nav_list::NavList::nav",
        Option("sidebars", "sidebars.keys"),
    ),
    (
        "widgets::nav_list::NavList::widths",
        Option("sidebars", "sidebars.width"),
    ),
    (
        "widgets::nav_list::NavList::item_id",
        Demo("sidebars", "Runs"),
    ),
    (
        "widgets::nav_list::NavList::width",
        Demo("sidebars", "Branches"),
    ),
    ("widgets::nav_list::NavList::on_key", Demo("sidebars", "↑↓")),
    (
        "widgets::nav_list::NavList::on_click",
        Demo("sidebars", "Members"),
    ),
    (
        "widgets::nav_list::NavList::render",
        Demo("sidebars", "Keyboard"),
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
        "widgets::tabs::TabCmd::ShowHidden",
        Demo("tabs", "↓ hidden tabs"),
    ),
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
    // S17 (SC36): the passive strip and the strip/body split
    (
        "widgets::tabs::Tabs::passive",
        Option("tabs", "tabs.passive"),
    ),
    (
        "widgets::tabs::Tabs::passive_focused",
        Option("tabs", "tabs.passive"),
    ),
    ("widgets::tabs::Tabs::layout", Demo("tabs", "Tabs::layout")),
    (
        "widgets::tabs::Tabs::layout_gap",
        Option("tabs", "tabs.gap"),
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
    ("widgets::menu::MenuItem::disabled", Demo("menus", "Share…")),
    ("widgets::menu::MenuItem::reason", Demo("menus", "╰─ Off ─")),
    ("widgets::menu::MenuItem::danger", Demo("menus", "Delete")),
    ("widgets::menu::MenuItem::danger_if", Demo("menus", "Trash")),
    (
        "widgets::menu::MenuItem::separator",
        Demo("menus", "Copy path"),
    ),
    (
        "widgets::menu::MenuItem::separator_if",
        Demo("menus", "Share…"),
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
    // Focus & navigation (SC28)
    (
        "core::event::InputConfig",
        Option("focusnavigation", "shell.shift_wheel_horizontal"),
    ),
    (
        "core::focus::FocusRing",
        Demo("focusnavigation", "Ring reading order"),
    ),
    (
        "core::focus::FocusFallback",
        Option("focusnavigation", "shell.focus_fallback"),
    ),
    (
        "core::focus::FocusFallback::Nearest",
        Option("focusnavigation", "shell.focus_fallback=nearest"),
    ),
    (
        "core::focus::FocusFallback::First",
        Option("focusnavigation", "shell.focus_fallback=first"),
    ),
    (
        "core::focus::FocusChange",
        Demo("focusnavigation", "Change none yet"),
    ),
    ("core::focus::Focus", Demo("focusnavigation", "Focus ring")),
    (
        "core::keyboard::KeyboardProtocol",
        Option("focusnavigation", "shell.keyboard_protocol"),
    ),
    (
        "core::keyboard::KeyboardProtocol::Auto",
        Option("focusnavigation", "shell.keyboard_protocol=auto"),
    ),
    (
        "core::keyboard::KeyboardProtocol::On",
        Option("focusnavigation", "shell.keyboard_protocol=on"),
    ),
    (
        "core::keyboard::KeyboardProtocol::Off",
        Option("focusnavigation", "shell.keyboard_protocol=off"),
    ),
    (
        "core::keyboard::ProtocolState",
        Demo("focusnavigation", "On pushes ESC[>5u"),
    ),
    (
        "core::keyboard::AltKeys",
        Option("focusnavigation", "shell.alt_keys"),
    ),
    (
        "core::keyboard::AltKeys::Delivered",
        Option("focusnavigation", "shell.alt_keys=delivered"),
    ),
    (
        "core::keyboard::AltKeys::Refused",
        Option("focusnavigation", "shell.alt_keys=refused"),
    ),
    (
        "core::keyboard::AltKeys::UntilSeen",
        Option("focusnavigation", "shell.alt_keys=until-seen"),
    ),
    (
        "core::keyboard::AltKeys::WithProtocol",
        Option("focusnavigation", "shell.alt_keys=with-protocol"),
    ),
    (
        "core::keymap::KeyBinding::alt_stand_in",
        Demo("focusnavigation", "stand-in Ctrl+Shift+←"),
    ),
    ("core::keymap::Nav", Demo("focusnavigation", "Nav map")),
    ("core::keymap::Nav::Up", Demo("focusnavigation", "Up ↑")),
    ("core::keymap::Nav::Down", Demo("focusnavigation", "Down ↓")),
    ("core::keymap::Nav::Left", Demo("focusnavigation", "Left ←")),
    (
        "core::keymap::Nav::Right",
        Demo("focusnavigation", "Right →"),
    ),
    (
        "core::keymap::Nav::PageUp",
        Demo("focusnavigation", "PageUp PgUp"),
    ),
    (
        "core::keymap::Nav::PageDown",
        Demo("focusnavigation", "PageDown PgDn"),
    ),
    (
        "core::keymap::Nav::Home",
        Demo("focusnavigation", "Home Home"),
    ),
    ("core::keymap::Nav::End", Demo("focusnavigation", "End End")),
    (
        "core::keymap::NavKeys",
        Option("focusnavigation", "focusnavigation.nav"),
    ),
    (
        "core::keymap::NavKeys::vim",
        Option("focusnavigation", "focusnavigation.nav=vim"),
    ),
    (
        "core::keymap::NavKeys::page",
        Option("focusnavigation", "focusnavigation.nav=no-page"),
    ),
    (
        "core::keymap::NavKeys::home_end",
        Option("focusnavigation", "focusnavigation.nav=no-home-end"),
    ),
    (
        "core::keymap::NavKeys::arrows",
        Option("focusnavigation", "focusnavigation.nav=vim-no-arrows"),
    ),
    (
        "core::keymap::KeyBinding",
        Demo("focusnavigation", "palette Ctrl+K"),
    ),
    (
        "core::keymap::KeyBinding::shift",
        Demo("focusnavigation", "back Shift+Tab"),
    ),
    // Toolbars (SC28)
    (
        "widgets::toolbar::Toolbar",
        Demo("toolbars", "Query toolbar"),
    ),
    ("widgets::toolbar::ToolItem", Demo("toolbars", "Explain")),
    (
        "widgets::toolbar::ToolItem::shortcut",
        Demo("toolbars", "Run F5"),
    ),
    (
        "widgets::toolbar::ToolItem::disabled",
        Demo("toolbars", "Stop is disabled until a run starts"),
    ),
    (
        "widgets::toolbar::ToolItem::toggle",
        Demo("toolbars", "Line numbers"),
    ),
    (
        "widgets::toolbar::ToolItem::separator",
        Demo("toolbars", "Formatting toolbar"),
    ),
    (
        "widgets::toolbar::ToolItem::tip",
        Demo("toolbars", "Run all Shift+F5"),
    ),
    (
        "widgets::toolbar::ToolbarAction",
        Demo("toolbars", "Activated(i) or Toggled(i, on)"),
    ),
    (
        "widgets::toolbar::ToolbarAction::Activated",
        Demo("toolbars", "Activated(i)"),
    ),
    (
        "widgets::toolbar::ToolbarAction::Toggled",
        Demo("toolbars", "Toggled(i, on)"),
    ),
    (
        "widgets::toolbar::Toolbar::nav",
        Option("toolbars", "toolbars.nav"),
    ),
    (
        "widgets::toolbar::Toolbar::wrap",
        Option("toolbars", "toolbars.wrap"),
    ),
    (
        "widgets::toolbar::Toolbar::skip_disabled",
        Option("toolbars", "toolbars.skip_disabled"),
    ),
    (
        "widgets::toolbar::Toolbar::gap",
        Option("toolbars", "toolbars.gap"),
    ),
    (
        "widgets::toolbar::Toolbar::overflow",
        Option("toolbars", "toolbars.overflow=off"),
    ),
    (
        "widgets::toolbar::Toolbar::more_glyph",
        Option("toolbars", "toolbars.overflow=…"),
    ),
    (
        "widgets::toolbar::Toolbar::more_button",
        Demo("toolbars", "Narrow toolbar"),
    ),
    (
        "widgets::toolbar::ToolItem::reason",
        Demo("toolbars", "No query is running"),
    ),
    // ---- Breadcrumbs & rails (SC5)
    (
        "widgets::breadcrumb::Breadcrumb",
        StateRow("breadcrumbsrails", "overflow"),
    ),
    (
        "widgets::breadcrumb::Breadcrumb::clickable",
        Option("breadcrumbsrails", "breadcrumbsrails.clickable"),
    ),
    (
        "widgets::rail::SidebarRail",
        StateRow("breadcrumbsrails", "pressed"),
    ),
    (
        "widgets::rail::SidebarRail::collapsed_rail",
        Option("breadcrumbsrails", "breadcrumbsrails.collapsed_rail"),
    ),
    (
        "widgets::rail::RailAction",
        Demo("breadcrumbsrails", "Last RailAction"),
    ),
    (
        "widgets::rail::RailAction::Expand",
        Demo("breadcrumbsrails", "RailAction::Expand"),
    ),
    (
        "widgets::rail::RailAction::Collapse",
        Demo("breadcrumbsrails", "RailAction::Collapse"),
    ),
    // ---- Help & keys, Quit & leave (SC6)
    (
        "widgets::help::HelpOverlay",
        Demo("helpkeys", "Open help overlay"),
    ),
    ("widgets::help::HelpCmd", Demo("helpkeys", "HelpCmd::Close")),
    (
        "widgets::help::HelpCmd::Close",
        Option("helpkeys", "helpkeys.keymap"),
    ),
    (
        "widgets::help::HelpOverlay::nav",
        Option("helpkeys", "helpkeys.nav"),
    ),
    (
        "widgets::help::HelpOverlay::keymap",
        Option("helpkeys", "helpkeys.keymap"),
    ),
    (
        "widgets::help::HelpOverlay::max_percent",
        Option("helpkeys", "helpkeys.max_percent"),
    ),
    (
        "widgets::help::HelpOverlay::title",
        Option("helpkeys", "helpkeys.title"),
    ),
    (
        "widgets::help::HelpOverlay::dismiss",
        Option("helpkeys", "helpkeys.dismiss"),
    ),
    (
        "widgets::keyref::KeyReference",
        StateRow("helpkeys", "overflow"),
    ),
    (
        "widgets::keyref::KeyReference::section",
        Demo("helpkeys", "Results"),
    ),
    (
        "widgets::keyref::KeyReference::key",
        Demo("helpkeys", "Ctrl+Shift+Enter"),
    ),
    (
        "widgets::keyref::KeyReference::gap",
        Option("helpkeys", "helpkeys.gap"),
    ),
    (
        "widgets::keyref::KeyReference::key_max_share",
        Option("helpkeys", "helpkeys.key_max_share"),
    ),
    ("ui::leave::Stakes", Demo("quitleave", "Stakes::max")),
    (
        "ui::leave::Stakes::Clean",
        Option("quitleave", "quitleave.stakes=clean"),
    ),
    (
        "ui::leave::Stakes::Unsaved",
        Option("quitleave", "quitleave.stakes=unsaved"),
    ),
    (
        "ui::leave::Stakes::Running",
        Option("quitleave", "quitleave.stakes=running"),
    ),
    ("ui::leave::Stakes::max", Demo("quitleave", "Stakes::max")),
    ("ui::leave::LeaveKey", Demo("quitleave", "→ LeaveKey")),
    ("ui::leave::LeaveKey::Copy", Demo("quitleave", "Copy")),
    ("ui::leave::LeaveKey::Quit", Demo("quitleave", "Quit")),
    ("ui::leave::KeyContext", Demo("quitleave", "text focus")),
    (
        "ui::leave::QuitKeys",
        Demo("quitleave", "QuitKeys::classify"),
    ),
    (
        "ui::leave::QuitKeys::letter",
        Option("quitleave", "quitleave.letter"),
    ),
    (
        "ui::leave::QuitKeys::esc_quits",
        Option("quitleave", "quitleave.esc_quits"),
    ),
    ("ui::leave::Leave", Demo("quitleave", "Leave::Quit")),
    ("ui::leave::Leave::Quit", Demo("quitleave", "Leave::Quit")),
    (
        "ui::leave::Leave::To",
        Demo("quitleave", "Leave::To(Overview)"),
    ),
    ("ui::leave::Resolved", Demo("quitleave", "Resolved::Stay")),
    (
        "ui::leave::Resolved::Stay",
        Demo("quitleave", "Resolved::Stay"),
    ),
    (
        "ui::leave::Resolved::Save",
        Demo("quitleave", "Resolved::Save(to)"),
    ),
    (
        "ui::leave::Resolved::Discard",
        Demo("quitleave", "Resolved::Discard(to)"),
    ),
    ("ui::leave::LeaveGuard", Demo("quitleave", "LeaveGuard")),
    (
        "ui::leave::LeaveGuard::save",
        Option("quitleave", "quitleave.save"),
    ),
    // ---- Lists & trees (SC15)
    ("core::typeahead::TypeAhead", Demo("lists", "(TypeAhead,")),
    (
        "widgets::list::ListItem",
        Demo("lists", "tests/checkout.rs"),
    ),
    ("widgets::list::ListItem::meta", Demo("lists", "generated")),
    (
        "widgets::list::ListItem::disabled",
        StateRow("lists", "disabled"),
    ),
    (
        "widgets::list::ListCmd",
        Demo("lists", "ListCmd::ToggleAll"),
    ),
    (
        "widgets::list::ListCmd::ToggleAll",
        Demo("lists", "a ListCmd::ToggleAll"),
    ),
    (
        "widgets::list::ListCmd::SelectAll",
        Demo("lists", "Ctrl+A ListCmd::SelectAll"),
    ),
    ("widgets::list::ListBox::nav", Option("lists", "lists.nav")),
    (
        "widgets::list::ListBox::keymap",
        Option("lists", "lists.keymap"),
    ),
    (
        "widgets::list::ListBox::wrap",
        Option("lists", "lists.wrap"),
    ),
    (
        "widgets::list::ListBox::type_ahead",
        Option("lists", "lists.type_ahead"),
    ),
    (
        "widgets::list::ListBox::skip_disabled",
        Option("lists", "lists.skip_disabled"),
    ),
    (
        "widgets::list::ListBox::click",
        Option("lists", "lists.click"),
    ),
    (
        "widgets::list::ListBox::toggle_mods",
        Option("lists", "lists.toggle_mods"),
    ),
    (
        "core::selection::ClickPolicy::check",
        Option("lists", "lists.check"),
    ),
    (
        "core::selection::CheckClick",
        Option("lists", "lists.check"),
    ),
    (
        "core::selection::CheckClick::BoxOnly",
        Option("lists", "lists.check=box"),
    ),
    (
        "core::selection::CheckClick::Row",
        Option("lists", "lists.check=row"),
    ),
    (
        "widgets::list::ListBox::scrollbar_gap",
        Option("lists", "lists.scrollbar_gap"),
    ),
    (
        "widgets::list::ListBox::reveal",
        Option("lists", "lists.reveal"),
    ),
    (
        "widgets::list::ListBox::empty",
        Option("lists", "lists.empty"),
    ),
    (
        "widgets::list::ListBox::content",
        Option("lists", "lists.content"),
    ),
    ("ui::content::ContentState", StateRow("lists", "loading")),
    (
        "ui::content::ContentState::Ready",
        StateRow("lists", "rest"),
    ),
    (
        "ui::content::ContentState::Loading",
        StateRow("lists", "loading"),
    ),
    (
        "ui::content::ContentState::Error",
        StateRow("lists", "error"),
    ),
    ("ui::list_core::ListCore", StateRow("lists", "rest")),
    (
        "widgets::list::ListBox::track_click",
        Option("lists", "lists.track_click"),
    ),
    (
        "widgets::tree::TreeNode::glyph",
        Demo("trees", "◆ production"),
    ),
    ("widgets::tree::TreeNode::meta", Demo("trees", "lag 4.2 s")),
    ("widgets::tree::FlatRow", Demo("trees", "visible")),
    (
        "widgets::tree::TreeAction",
        Demo("trees", "last TreeAction:"),
    ),
    (
        "widgets::tree::TreeAction::Expand",
        Demo("trees", "TreeAction::Expand on a lazy node"),
    ),
    (
        "widgets::tree::TreeAction::Activate",
        Demo("trees", "TreeAction::Activate on a leaf"),
    ),
    (
        "widgets::tree::BranchAction",
        Option("trees", "trees.branch"),
    ),
    (
        "widgets::tree::BranchAction::Toggle",
        Option("trees", "trees.branch=toggle"),
    ),
    (
        "widgets::tree::BranchAction::Activate",
        Option("trees", "trees.branch=activate"),
    ),
    (
        "widgets::tree::BranchAction::Both",
        Option("trees", "trees.branch=both"),
    ),
    (
        "widgets::tree::TreeCmd",
        Demo("trees", "TreeCmd::ExpandAll"),
    ),
    (
        "widgets::tree::TreeCmd::ExpandAll",
        Demo("trees", "* TreeCmd::ExpandAll"),
    ),
    (
        "widgets::tree::TreeCmd::CollapseAll",
        Demo("trees", "- TreeCmd::CollapseAll"),
    ),
    (
        "widgets::tree::TreeView::branch_activate",
        Option("trees", "trees.branch"),
    ),
    (
        "widgets::tree::TreeView::click",
        Option("trees", "trees.click"),
    ),
    (
        "widgets::tree::TreeView::wrap",
        Option("trees", "trees.wrap"),
    ),
    ("widgets::tree::TreeView::nav", Option("trees", "trees.nav")),
    (
        "widgets::tree::TreeView::keymap",
        Option("trees", "trees.keymap"),
    ),
    (
        "widgets::tree::TreeView::indent",
        Option("trees", "trees.indent"),
    ),
    (
        "widgets::tree::TreeView::type_ahead",
        Option("trees", "trees.type_ahead"),
    ),
    (
        "widgets::tree::TreeView::divider",
        Option("trees", "trees.divider"),
    ),
    (
        "widgets::tree::TreeView::selected_marker",
        Option("trees", "trees.marker"),
    ),
    (
        "widgets::tree::TreeView::alt_click_recursive",
        Option("trees", "trees.alt_click"),
    ),
    (
        "widgets::tree::TreeView::reserve_right",
        Option("trees", "trees.reserve_right"),
    ),
    (
        "widgets::tree::TreeView::show_meta",
        Option("trees", "trees.show_meta"),
    ),
    (
        "widgets::tree::TreeView::track_click",
        Option("trees", "trees.track_click"),
    ),
    (
        "core::scroll::ScrollState",
        Option("scrolling", "scrolling.context"),
    ),
    (
        "core::scroll::ScrollState::with_follow_cursor",
        Option("scrolling", "scrolling.follow"),
    ),
    (
        "core::scroll::ScrollState::with_context",
        Option("scrolling", "scrolling.context"),
    ),
    (
        "core::scroll::FollowCursor",
        Option("scrolling", "scrolling.follow"),
    ),
    (
        "core::scroll::FollowCursor::OnMove",
        Option("scrolling", "scrolling.follow=on-move"),
    ),
    (
        "core::scroll::FollowCursor::Always",
        Option("scrolling", "scrolling.follow=always"),
    ),
    (
        "core::scroll::ScrollbarVisibility",
        Option("scrolling", "scrolling.hbar"),
    ),
    (
        "core::scroll::ScrollbarVisibility::Auto",
        Option("scrolling", "scrolling.hbar=auto"),
    ),
    (
        "core::scroll::ScrollbarVisibility::Always",
        Option("scrolling", "scrolling.hbar=always"),
    ),
    (
        "core::scroll::ScrollbarVisibility::Never",
        Option("scrolling", "scrolling.hbar=never"),
    ),
    (
        "core::scroll::TrackClick",
        Option("scrolling", "scrolling.track_click"),
    ),
    (
        "core::scroll::TrackClick::Jump",
        Option("scrolling", "scrolling.track_click=jump"),
    ),
    (
        "core::scroll::TrackClick::Page",
        Option("scrolling", "scrolling.track_click=page"),
    ),
    ("core::scroll::ScrollPane", Demo("scrolling", "Hand-drawn")),
    (
        "core::scroll::ScrollPane::horizontal",
        Option("scrolling", "scrolling.hbar"),
    ),
    (
        "core::scroll::ScrollPane::track_click",
        Option("scrolling", "scrolling.track_click"),
    ),
    (
        "core::scroll::ScrollPane::columns",
        Option("scrolling", "scrolling.columns"),
    ),
    ("widgets::scrollbar::Bars", Demo("scrolling", "Hand-drawn")),
    (
        "widgets::scrollbar::VSplit",
        Demo("scrolling", "Divider or bar"),
    ),
    ("core::scroll::HScroll", Option("panels", "panels.hscroll")),
    (
        "core::scroll::HScroll::bar",
        Option("panels", "panels.hscroll=bar-always"),
    ),
    (
        "core::scroll::HScroll::step",
        Option("panels", "panels.hscroll=step-1"),
    ),
    (
        "core::scroll::HScroll::track_click",
        Option("panels", "panels.hscroll=jump"),
    ),
    (
        "widgets::panel::PanelFocus",
        Option("panels", "panels.focus_mark"),
    ),
    (
        "widgets::panel::PanelFocus::Title",
        Option("panels", "panels.focus_mark=title"),
    ),
    (
        "widgets::panel::PanelFocus::Bar",
        Option("panels", "panels.focus_mark=bar"),
    ),
    (
        "widgets::panel::Panel::focused",
        StateRow("panels", "focused"),
    ),
    (
        "widgets::panel::Panel::focus_mark",
        Option("panels", "panels.focus_mark"),
    ),
    ("widgets::panel::Panel::meta", Demo("panels", "surface")),
    (
        "widgets::panel::Panel::meta_short",
        Option("panels", "panels.meta_short"),
    ),
    (
        "widgets::panel::Panel::pad_x",
        Option("panels", "panels.pad_x"),
    ),
    (
        "widgets::panel::Panel::title_min",
        Option("panels", "panels.title_min"),
    ),
    (
        "widgets::panel::PanelCmd",
        Option("panels", "panels.keymap"),
    ),
    (
        "widgets::panel::PanelCmd::Follow",
        Option("panels", "panels.keymap"),
    ),
    (
        "widgets::panel::PanelCmd::Copy",
        Option("panels", "panels.keymap"),
    ),
    (
        "widgets::panel::PanelCmd::SelectAll",
        Option("panels", "panels.keymap"),
    ),
    (
        "widgets::panel::ScrollPanel::nav",
        Option("panels", "panels.nav"),
    ),
    (
        "widgets::panel::ScrollPanel::keymap",
        Option("panels", "panels.keymap"),
    ),
    (
        "widgets::panel::ScrollPanel::horizontal",
        Option("panels", "panels.hscroll"),
    ),
    (
        "widgets::panel::ScrollPanel::wrap",
        Option("panels", "panels.wrap"),
    ),
    (
        "widgets::panel::ScrollPanel::tail",
        Option("panels", "panels.tail"),
    ),
    (
        "widgets::panel::ScrollPanel::track_click",
        Option("panels", "panels.track_click"),
    ),
    ("core::list::Cursor", Demo("scrolling", "Cursor")),
    (
        "core::list::Cursor::wrap",
        Option("scrolling", "scrolling.wrap"),
    ),
    ("core::list::Cursor::axis", Demo("scrolling", "· vertical")),
    (
        "core::list::Cursor::skip_disabled",
        Option("scrolling", "scrolling.skip_disabled"),
    ),
    (
        "core::list::Moved",
        Demo("scrolling", "Moved: To · Stayed · Ignored"),
    ),
    (
        "core::list::Moved::Ignored",
        Demo("scrolling", "Moved: To · Stayed · Ignored"),
    ),
    (
        "core::list::Moved::Stayed",
        Demo("scrolling", "Moved: To · Stayed · Ignored"),
    ),
    (
        "core::list::Moved::To",
        Demo("scrolling", "Moved: To · Stayed · Ignored"),
    ),
    ("core::list::RowFn", Demo("scrolling", "RowFn")),
    (
        "core::list::KeyStep",
        Demo("scrolling", "KeyStep: Nav · Type"),
    ),
    (
        "core::list::KeyStep::Nav",
        Demo("scrolling", "KeyStep: Nav · Type"),
    ),
    (
        "core::list::KeyStep::Type",
        Demo("scrolling", "KeyStep: Nav · Type"),
    ),
    (
        "core::list::NavProfile",
        Option("scrolling", "scrolling.profile"),
    ),
    (
        "core::list::NavProfile::Desktop",
        Option("scrolling", "scrolling.profile=desktop"),
    ),
    (
        "core::list::NavProfile::Vim",
        Option("scrolling", "scrolling.profile=vim"),
    ),
    (
        "core::list::TypeAheadPrecedence",
        Option("scrolling", "scrolling.precedence"),
    ),
    (
        "core::list::TypeAheadPrecedence::Shortcuts",
        Option("scrolling", "scrolling.precedence=shortcuts"),
    ),
    (
        "core::list::TypeAheadPrecedence::TypeAhead",
        Option("scrolling", "scrolling.precedence=type-ahead"),
    ),
    (
        "widgets::grid::DataGrid::type_ahead_precedence",
        Option("scrolling", "scrolling.precedence"),
    ),
    (
        "widgets::grid::DataGrid::type_ahead_timeout",
        Option("scrolling", "scrolling.timeout"),
    ),
    (
        "widgets::list::ListBox::type_ahead_precedence",
        Option("scrolling", "scrolling.precedence"),
    ),
    (
        "widgets::list::ListBox::type_ahead_timeout",
        Option("scrolling", "scrolling.timeout"),
    ),
    (
        "widgets::select::Select::type_ahead_precedence",
        Option("scrolling", "scrolling.precedence"),
    ),
    (
        "widgets::select::Select::type_ahead_timeout",
        Option("scrolling", "scrolling.timeout"),
    ),
    (
        "widgets::table::DataTable::type_ahead_precedence",
        Option("scrolling", "scrolling.precedence"),
    ),
    (
        "widgets::table::DataTable::type_ahead_timeout",
        Option("scrolling", "scrolling.timeout"),
    ),
    (
        "widgets::tree::TreeView::type_ahead_precedence",
        Option("scrolling", "scrolling.precedence"),
    ),
    (
        "widgets::tree::TreeView::type_ahead_timeout",
        Option("scrolling", "scrolling.timeout"),
    ),
    // SC17: Dialogs
    (
        "widgets::dialog::DialogBody",
        Demo("dialogs", "Text · Input · Facts"),
    ),
    (
        "widgets::dialog::DialogBody::Text",
        Demo("dialogs", "Text body"),
    ),
    (
        "widgets::dialog::DialogBody::Input",
        Demo("dialogs", "Input body"),
    ),
    (
        "widgets::dialog::DialogBody::Facts",
        Demo("dialogs", "Facts · Close only"),
    ),
    ("widgets::dialog::AckInput", StateRow("dialogs", "editing")),
    (
        "widgets::dialog::DialogResult",
        Demo("dialogs", "DialogResult"),
    ),
    (
        "widgets::dialog::DialogResult::Action",
        Demo("dialogs", "Action(i) or Cancelled"),
    ),
    (
        "widgets::dialog::DialogResult::Cancelled",
        Demo("dialogs", "Action(i) or Cancelled"),
    ),
    (
        "widgets::dialog::AckEnter",
        Option("dialogs", "dialogs.ack_enter"),
    ),
    (
        "widgets::dialog::AckEnter::Submit",
        Option("dialogs", "dialogs.ack_enter=submit"),
    ),
    (
        "widgets::dialog::AckEnter::NextField",
        Option("dialogs", "dialogs.ack_enter=next-field"),
    ),
    (
        "widgets::dialog::DialogCmd",
        Option("dialogs", "dialogs.keymap"),
    ),
    (
        "widgets::dialog::DialogCmd::Confirm",
        Option("dialogs", "dialogs.keymap=ctrl"),
    ),
    (
        "widgets::dialog::DialogCmd::Cancel",
        Option("dialogs", "dialogs.keymap=ctrl"),
    ),
    (
        "widgets::dialog::Dialog::width",
        Option("dialogs", "dialogs.width"),
    ),
    (
        "widgets::dialog::Dialog::accelerators",
        Option("dialogs", "dialogs.accelerators"),
    ),
    (
        "widgets::dialog::Dialog::keymap",
        Option("dialogs", "dialogs.keymap"),
    ),
    (
        "widgets::dialog::Dialog::nav",
        Option("dialogs", "dialogs.nav"),
    ),
    (
        "widgets::dialog::Dialog::ack_enter",
        Option("dialogs", "dialogs.ack_enter"),
    ),
    (
        "widgets::dialog::Dialog::mnemonics",
        Option("dialogs", "dialogs.mnemonics"),
    ),
    (
        "widgets::dialog::Dialog::guard_dirty_prompt",
        Option("dialogs", "dialogs.guard_dirty"),
    ),
    (
        "widgets::dialog::Dialog::guard_destructive_dismiss",
        Option("dialogs", "dialogs.guard_destructive"),
    ),
    (
        "widgets::dialog::Dialog::cancel_label",
        Option("dialogs", "dialogs.cancel_label"),
    ),
    (
        "widgets::dialog::Dialog::dismiss",
        Option("dialogs", "dialogs.dismiss"),
    ),
    (
        "widgets::dialog::Dialog::close_button",
        Option("dialogs", "dialogs.close_button"),
    ),
    (
        "widgets::dialog::Dialog::edit_entry",
        Option("dialogs", "dialogs.edit_entry"),
    ),
    (
        "widgets::dialog::Dialog::actions",
        Demo("dialogs", "Retry · Force push"),
    ),
    (
        "widgets::dialog::Dialog::button_row",
        Option("dialogs", "dialogs.button_row"),
    ),
    (
        "widgets::dialog::Dialog::body_scroll",
        Option("dialogs", "dialogs.body_scroll"),
    ),
    (
        "widgets::dialog::Dialog::max_height",
        Option("dialogs", "dialogs.max_height"),
    ),
    (
        "widgets::dialog::Dialog::outer_margin",
        Option("dialogs", "dialogs.outer_margin"),
    ),
    (
        "widgets::dialog::Dialog::backdrop",
        Option("dialogs", "dialogs.backdrop"),
    ),
    (
        "widgets::dialog::Dialog::chrome",
        Option("dialogs", "dialogs.chrome"),
    ),
    (
        "widgets::dialog::Dialog::placement",
        Option("dialogs", "dialogs.placement=center"),
    ),
    (
        "widgets::dialog::Dialog::anchor",
        Option("dialogs", "dialogs.placement=below"),
    ),
    (
        "widgets::dialog::Dialog::confirm_index",
        Option("dialogs", "dialogs.confirm_index"),
    ),
    // SC18: Dialog kinds
    (
        "widgets::choice_dialog::ChoiceDialog",
        StateRow("dialogkinds", "rest"),
    ),
    (
        "widgets::choice_dialog::ChoiceDialog::nav",
        Option("dialogkinds", "dialogkinds.nav"),
    ),
    (
        "widgets::choice_dialog::ChoiceDialog::buttons",
        Option("dialogkinds", "dialogkinds.choice_buttons"),
    ),
    (
        "widgets::choice_dialog::ChoiceDialog::line",
        Demo("dialogkinds", "width 54 · line · stepper"),
    ),
    (
        "widgets::choice_dialog::ChoiceDialog::stepper",
        Option("dialogkinds", "dialogkinds.choice_stepper"),
    ),
    (
        "widgets::choice_dialog::ChoiceDialog::width",
        Option("dialogkinds", "dialogkinds.width"),
    ),
    (
        "widgets::choice_dialog::ChoiceDialog::option_tones",
        Option("dialogkinds", "dialogkinds.choice_tones"),
    ),
    (
        "widgets::choice_dialog::ChoiceDialog::close_button",
        Option("dialogkinds", "dialogkinds.close_button"),
    ),
    (
        "widgets::choice_dialog::ChoiceDialog::button_row",
        Option("dialogkinds", "dialogkinds.button_wrap"),
    ),
    (
        "widgets::form_dialog::FieldValue",
        Demo("dialogkinds", "values Text · Choice · Bool"),
    ),
    (
        "widgets::form_dialog::FieldValue::Text",
        Demo("dialogkinds", "values Text · Choice · Bool"),
    ),
    (
        "widgets::form_dialog::FieldValue::Choice",
        Demo("dialogkinds", "values Text · Choice · Bool"),
    ),
    (
        "widgets::form_dialog::FieldValue::Bool",
        Demo("dialogkinds", "values Text · Choice · Bool"),
    ),
    (
        "widgets::form_dialog::FormFieldKind",
        Demo("dialogkinds", "Form fields Input · Select"),
    ),
    (
        "widgets::form_dialog::FormFieldKind::Input",
        Demo("dialogkinds", "Form fields Input"),
    ),
    (
        "widgets::form_dialog::FormFieldKind::Select",
        Demo("dialogkinds", "Input · Select"),
    ),
    (
        "widgets::form_dialog::FormFieldKind::Check",
        Demo("dialogkinds", "Radio · Check"),
    ),
    (
        "widgets::form_dialog::FormFieldKind::Radio",
        Demo("dialogkinds", "Radio · Check"),
    ),
    (
        "widgets::form_dialog::FormFieldKind::Chooser",
        Demo("dialogkinds", "Check · Chooser · Note"),
    ),
    (
        "widgets::form_dialog::FormFieldKind::Note",
        Demo("dialogkinds", "Chooser · Note · Note"),
    ),
    (
        "widgets::form_dialog::FormField",
        Demo("dialogkinds", "Form fields"),
    ),
    (
        "widgets::form_dialog::FormField::hidden",
        Demo("dialogkinds", "hidden key"),
    ),
    (
        "widgets::form_dialog::FormAction",
        Demo(
            "dialogkinds",
            "Form(Save | Cancel | Changed | Choose | Action)",
        ),
    ),
    (
        "widgets::form_dialog::FormAction::Changed",
        Demo(
            "dialogkinds",
            "Form(Save | Cancel | Changed | Choose | Action)",
        ),
    ),
    (
        "widgets::form_dialog::FormAction::Choose",
        Demo(
            "dialogkinds",
            "Form(Save | Cancel | Changed | Choose | Action)",
        ),
    ),
    (
        "widgets::form_dialog::FormAction::Action",
        Demo(
            "dialogkinds",
            "Form(Save | Cancel | Changed | Choose | Action)",
        ),
    ),
    (
        "widgets::form_dialog::FormAction::Save",
        Demo(
            "dialogkinds",
            "Form(Save | Cancel | Changed | Choose | Action)",
        ),
    ),
    (
        "widgets::form_dialog::FormAction::Cancel",
        Demo(
            "dialogkinds",
            "Form(Save | Cancel | Changed | Choose | Action)",
        ),
    ),
    (
        "widgets::form_dialog::FormDialog",
        Demo("dialogkinds", "Add connection…"),
    ),
    (
        "widgets::form_dialog::FormDialog::keep_open_on_save",
        Option("dialogkinds", "dialogkinds.keep_open"),
    ),
    (
        "widgets::form_dialog::FormDialog::action",
        Demo("dialogkinds", "actions test"),
    ),
    (
        "widgets::form_dialog::FormDialog::width",
        Option("dialogkinds", "dialogkinds.width"),
    ),
    (
        "widgets::form_dialog::FormDialog::meta",
        Option("dialogkinds", "dialogkinds.form_meta"),
    ),
    (
        "widgets::form_dialog::FormDialog::nav",
        Option("dialogkinds", "dialogkinds.nav"),
    ),
    (
        "widgets::form_dialog::FormDialog::close_button",
        Option("dialogkinds", "dialogkinds.close_button"),
    ),
    (
        "widgets::form_dialog::FormDialog::button_row",
        Option("dialogkinds", "dialogkinds.button_wrap"),
    ),
    (
        "widgets::help::HelpOverlay::close_button",
        Option("dialogkinds", "dialogkinds.close_button"),
    ),
    (
        "widgets::info_dialog::InfoResult",
        Demo("dialogkinds", "Info(Closed | Copy | Action)"),
    ),
    (
        "widgets::info_dialog::InfoResult::Closed",
        Demo("dialogkinds", "Info(Closed | Copy | Action)"),
    ),
    (
        "widgets::info_dialog::InfoResult::Copy",
        Demo("dialogkinds", "Info(Closed | Copy | Action)"),
    ),
    (
        "widgets::info_dialog::InfoResult::Action",
        Demo("dialogkinds", "Info(Closed | Copy | Action)"),
    ),
    (
        "widgets::info_dialog::InfoDialog",
        Demo("dialogkinds", "Server info"),
    ),
    (
        "widgets::info_dialog::InfoCmd",
        Option("dialogkinds", "dialogkinds.info_keymap"),
    ),
    (
        "widgets::info_dialog::InfoCmd::Close",
        Option("dialogkinds", "dialogkinds.info_keymap"),
    ),
    (
        "widgets::info_dialog::InfoCmd::Copy",
        Option("dialogkinds", "dialogkinds.info_keymap"),
    ),
    (
        "widgets::info_dialog::InfoDialog::nav",
        Option("dialogkinds", "dialogkinds.nav"),
    ),
    (
        "widgets::info_dialog::InfoDialog::keymap",
        Option("dialogkinds", "dialogkinds.info_keymap"),
    ),
    (
        "widgets::info_dialog::InfoDialog::intro",
        Demo("dialogkinds", "read-only · intro · detail 3 rows"),
    ),
    (
        "widgets::info_dialog::InfoDialog::detail",
        Demo("dialogkinds", "detail 3 rows"),
    ),
    (
        "widgets::info_dialog::InfoDialog::action",
        Demo("dialogkinds", "detail 3 rows · actions 1"),
    ),
    (
        "widgets::info_dialog::InfoDialog::width",
        Option("dialogkinds", "dialogkinds.width"),
    ),
    (
        "widgets::info_dialog::InfoDialog::meta",
        Demo("dialogkinds", "meta read-only"),
    ),
    (
        "widgets::info_dialog::InfoDialog::error",
        Option("dialogkinds", "dialogkinds.info_error"),
    ),
    (
        "widgets::info_dialog::InfoDialog::close_button",
        Option("dialogkinds", "dialogkinds.close_button"),
    ),
    // SC19: Popups & tooltips
    (
        "ui::ctx::Backdrop",
        Option("popupstooltips", "popupstooltips.frame_dim"),
    ),
    (
        "ui::ctx::Backdrop::keep_bottom",
        Option("popupstooltips", "popupstooltips.frame_dim=keep-bottom"),
    ),
    (
        "ui::ctx::Backdrop::keep_top",
        Option("popupstooltips", "popupstooltips.frame_dim=keep-top"),
    ),
    ("ui::modal::Frame", StateRow("popupstooltips", "rest")),
    (
        "ui::modal::Frame::close_button",
        Option("popupstooltips", "popupstooltips.frame_close_button"),
    ),
    (
        "ui::modal::Frame::dismiss",
        Option("popupstooltips", "popupstooltips.frame_dismiss"),
    ),
    (
        "ui::modal::Frame::title",
        StateRow("popupstooltips", "rest"),
    ),
    ("ui::modal::Frame::meta", StateRow("popupstooltips", "rest")),
    (
        "ui::modal::Frame::placement",
        Option("popupstooltips", "popupstooltips.frame_placement=center"),
    ),
    (
        "ui::modal::Frame::anchor",
        Option("popupstooltips", "popupstooltips.frame_placement=below"),
    ),
    (
        "ui::modal::Frame::fit",
        Option("popupstooltips", "popupstooltips.frame_fit"),
    ),
    (
        "ui::modal::Frame::margin",
        Option("popupstooltips", "popupstooltips.frame_margin"),
    ),
    (
        "ui::modal::Frame::min_size",
        Option("popupstooltips", "popupstooltips.frame_min"),
    ),
    (
        "ui::modal::Frame::chrome",
        Option("popupstooltips", "popupstooltips.frame_chrome"),
    ),
    (
        "ui::modal::Frame::backdrop",
        Option("popupstooltips", "popupstooltips.frame_backdrop"),
    ),
    (
        "ui::modal::Frame::dim",
        Option("popupstooltips", "popupstooltips.frame_dim"),
    ),
    (
        "ui::popup::Dismiss",
        Option("popupstooltips", "popupstooltips.escape"),
    ),
    (
        "ui::popup::RefuseFeedback",
        Option("popupstooltips", "popupstooltips.refuse_feedback"),
    ),
    (
        "ui::popup::RefuseFeedback::Flash",
        Option("popupstooltips", "popupstooltips.refuse_feedback=flash"),
    ),
    (
        "ui::popup::RefuseFeedback::None",
        Option("popupstooltips", "popupstooltips.refuse_feedback=none"),
    ),
    (
        "ui::popup::Dismiss::pass_through",
        Option("popupstooltips", "popupstooltips.pass_through"),
    ),
    (
        "ui::popup::Dismiss::close_button",
        Option("popupstooltips", "popupstooltips.frame_close"),
    ),
    (
        "ui::popup::Dismiss::refuse_feedback",
        Option("popupstooltips", "popupstooltips.refuse_feedback"),
    ),
    (
        "ui::popup::Dismiss::focus_out",
        Option("popupstooltips", "popupstooltips.focus_out"),
    ),
    (
        "ui::popup::Placement",
        Option("popupstooltips", "popupstooltips.placement"),
    ),
    (
        "ui::popup::Placement::Below",
        Option("popupstooltips", "popupstooltips.placement=below"),
    ),
    (
        "ui::popup::Placement::Above",
        Option("popupstooltips", "popupstooltips.placement=above"),
    ),
    (
        "ui::popup::Placement::Right",
        Option("popupstooltips", "popupstooltips.placement=right"),
    ),
    (
        "ui::popup::Placement::Left",
        Option("popupstooltips", "popupstooltips.placement=left"),
    ),
    (
        "ui::popup::Placement::Center",
        Option("popupstooltips", "popupstooltips.placement=center"),
    ),
    (
        "ui::popup::Overflow",
        Option("popupstooltips", "popupstooltips.overflow"),
    ),
    (
        "ui::popup::Overflow::Shrink",
        Option("popupstooltips", "popupstooltips.overflow=shrink"),
    ),
    (
        "ui::popup::Overflow::Slide",
        Option("popupstooltips", "popupstooltips.overflow=slide"),
    ),
    (
        "ui::popup::VAlign",
        Option("popupstooltips", "popupstooltips.v_align"),
    ),
    (
        "ui::popup::VAlign::Center",
        Option("popupstooltips", "popupstooltips.v_align=center"),
    ),
    (
        "ui::popup::VAlign::Third",
        Option("popupstooltips", "popupstooltips.v_align=third"),
    ),
    (
        "ui::popup::Place",
        Option("popupstooltips", "popupstooltips.margin"),
    ),
    (
        "ui::popup::Place::vertical_margin",
        Option("popupstooltips", "popupstooltips.vertical_margin"),
    ),
    (
        "ui::popup::Place::overflow",
        Option("popupstooltips", "popupstooltips.overflow"),
    ),
    (
        "ui::popup::Place::margin",
        Option("popupstooltips", "popupstooltips.margin"),
    ),
    (
        "ui::popup::Place::v_align",
        Option("popupstooltips", "popupstooltips.v_align"),
    ),
    (
        "ui::popup::Place::text_inset",
        Option("popupstooltips", "popupstooltips.text_inset"),
    ),
    (
        "ui::reveal::RevealMode",
        Option("popupstooltips", "popupstooltips.reveal_mode"),
    ),
    (
        "ui::reveal::RevealMode::Tooltip",
        Option("popupstooltips", "popupstooltips.reveal_mode=tooltip"),
    ),
    (
        "ui::reveal::RevealMode::InlineWrap",
        Option("popupstooltips", "popupstooltips.reveal_mode=inline"),
    ),
    (
        "ui::reveal::Reveal",
        Option("popupstooltips", "popupstooltips.reveal_hover"),
    ),
    (
        "ui::reveal::Reveal::hover",
        Option("popupstooltips", "popupstooltips.reveal_hover"),
    ),
    (
        "ui::reveal::Reveal::keyboard",
        Option("popupstooltips", "popupstooltips.reveal_key"),
    ),
    (
        "ui::reveal::Reveal::mode",
        Option("popupstooltips", "popupstooltips.reveal_mode"),
    ),
    (
        "ui::reveal::Reveal::max_rows",
        Option("popupstooltips", "popupstooltips.reveal_rows"),
    ),
    (
        "ui::reveal::Reveal::scroll",
        Option("popupstooltips", "popupstooltips.reveal_scroll"),
    ),
    (
        "ui::reveal::Reveal::wheel_lines",
        Option("popupstooltips", "popupstooltips.reveal_wheel"),
    ),
    (
        "ui::reveal::Reveal::delay_ms",
        Option("popupstooltips", "popupstooltips.reveal_delay"),
    ),
    (
        "ui::reveal::Reveal::wheel_passes_at_ends",
        Option("popupstooltips", "popupstooltips.reveal_passes"),
    ),
    (
        "ui::reveal::RevealState",
        Demo("popupstooltips", "RevealState per-widget policy set"),
    ),
    (
        "ui::tooltip::Tooltips",
        Option("popupstooltips", "popupstooltips.tooltips"),
    ),
    (
        "ui::tooltip::Tooltips::enabled",
        Option("popupstooltips", "popupstooltips.tooltips"),
    ),
    (
        "ui::tooltip::Tooltips::delay_ms",
        Option("popupstooltips", "popupstooltips.tip_delay"),
    ),
    (
        "ui::tooltip::Tooltips::max_width",
        Option("popupstooltips", "popupstooltips.tip_width"),
    ),
    (
        "ui::tooltip::Tip",
        Demo("popupstooltips", "Save the draft to this device ⌘S"),
    ),
    (
        "ui::tooltip::Tip::shortcut",
        Demo("popupstooltips", "Open an empty query tab ⌘T"),
    ),
    (
        "widgets::statusbar::StatusItem::tooltip",
        Demo("popupstooltips", "Region of the connection ⌘R"),
    ),
    // SC20: Pickers
    (
        "widgets::option_list::Item::detail",
        Demo("pickers", "disabled: Run a query first"),
    ),
    (
        "widgets::option_list::Item::glyph",
        Demo("pickers", "▶ Run query"),
    ),
    (
        "widgets::option_list::Item::group",
        Demo("pickers", "Commands"),
    ),
    (
        "widgets::option_list::Item::key",
        Demo("pickers", "(run.query)"),
    ),
    (
        "widgets::option_list::Item::disabled",
        Demo("pickers", "disabled: No open transaction"),
    ),
    (
        "widgets::picker::PickerStatus",
        StateRow("pickers", "loading"),
    ),
    (
        "widgets::picker::PickerStatus::Ready",
        StateRow("pickers", "rest"),
    ),
    (
        "widgets::picker::PickerStatus::Loading",
        StateRow("pickers", "loading"),
    ),
    (
        "widgets::picker::PickerStatus::Error",
        StateRow("pickers", "error"),
    ),
    (
        "widgets::picker::PickerAction",
        Demo(
            "pickers",
            "QueryChanged · Chosen(i) · ChosenAlt(i) · Submit · Secondary(i) · NextScope · PrevScope · Back · Cancelled",
        ),
    ),
    (
        "widgets::picker::PickerAction::QueryChanged",
        Demo(
            "pickers",
            "QueryChanged · Chosen(i) · ChosenAlt(i) · Submit · Secondary(i) · NextScope · PrevScope · Back · Cancelled",
        ),
    ),
    (
        "widgets::picker::PickerAction::Chosen",
        Demo(
            "pickers",
            "QueryChanged · Chosen(i) · ChosenAlt(i) · Submit · Secondary(i) · NextScope · PrevScope · Back · Cancelled",
        ),
    ),
    (
        "widgets::picker::PickerAction::ChosenAlt",
        Demo(
            "pickers",
            "QueryChanged · Chosen(i) · ChosenAlt(i) · Submit · Secondary(i) · NextScope · PrevScope · Back · Cancelled",
        ),
    ),
    (
        "widgets::picker::PickerAction::Submit",
        Demo(
            "pickers",
            "QueryChanged · Chosen(i) · ChosenAlt(i) · Submit · Secondary(i) · NextScope · PrevScope · Back · Cancelled",
        ),
    ),
    (
        "widgets::picker::PickerAction::Secondary",
        Demo(
            "pickers",
            "QueryChanged · Chosen(i) · ChosenAlt(i) · Submit · Secondary(i) · NextScope · PrevScope · Back · Cancelled",
        ),
    ),
    (
        "widgets::picker::PickerAction::NextScope",
        Demo(
            "pickers",
            "QueryChanged · Chosen(i) · ChosenAlt(i) · Submit · Secondary(i) · NextScope · PrevScope · Back · Cancelled",
        ),
    ),
    (
        "widgets::picker::PickerAction::Cancelled",
        Demo(
            "pickers",
            "QueryChanged · Chosen(i) · ChosenAlt(i) · Submit · Secondary(i) · NextScope · PrevScope · Back · Cancelled",
        ),
    ),
    (
        "widgets::picker::PickerAction::Back",
        Demo(
            "pickers",
            "QueryChanged · Chosen(i) · ChosenAlt(i) · Submit · Secondary(i) · NextScope · PrevScope · Back · Cancelled",
        ),
    ),
    (
        "widgets::picker::PickerAction::PrevScope",
        Demo(
            "pickers",
            "QueryChanged · Chosen(i) · ChosenAlt(i) · Submit · Secondary(i) · NextScope · PrevScope · Back · Cancelled",
        ),
    ),
    (
        "widgets::picker::Picker::nav",
        Option("pickers", "pickers.nav"),
    ),
    (
        "widgets::picker::Picker::delete_is_secondary",
        Option("pickers", "pickers.delete_is_secondary"),
    ),
    (
        "widgets::picker::Picker::width",
        Option("pickers", "pickers.width"),
    ),
    (
        "widgets::picker::Picker::placement",
        Option("pickers", "pickers.placement=center"),
    ),
    (
        "widgets::picker::Picker::anchor",
        Option("pickers", "pickers.placement=below"),
    ),
    (
        "widgets::picker::Picker::fit",
        Option("pickers", "pickers.fit"),
    ),
    (
        "widgets::picker::Picker::backdrop",
        Option("pickers", "pickers.backdrop"),
    ),
    (
        "widgets::picker::Picker::hints",
        Option("pickers", "pickers.hints"),
    ),
    (
        "widgets::picker::Picker::max_rows",
        Option("pickers", "pickers.max_rows"),
    ),
    (
        "widgets::picker::Picker::placeholder",
        Option("pickers", "pickers.placeholder"),
    ),
    (
        "widgets::picker::Picker::empty",
        Option("pickers", "pickers.empty=state"),
    ),
    (
        "widgets::picker::Picker::empty_text",
        Option("pickers", "pickers.empty=text"),
    ),
    (
        "widgets::picker::Picker::dismiss",
        Option("pickers", "pickers.dismiss"),
    ),
    (
        "widgets::picker::Picker::cursor_style",
        Option("pickers", "pickers.cursor_style"),
    ),
    (
        "ui::specimen::SpecimenState::Loading",
        StateRow("pickers", "loading"),
    ),
    ("state::loading", StateRow("pickers", "loading")),
    // public functions the showcase already shows (SC-G1)
    (
        "core::focus::request_focus_next",
        Demo("focusnavigation", "request_focus_next"),
    ),
    ("ui::columns::solve", Demo("columns", "columns::solve")),
    (
        "ui::leave::QuitKeys::classify",
        Demo("quitleave", "QuitKeys::classify"),
    ),
    (
        "ui::popup::place_with",
        Demo("popupstooltips", "place_with →"),
    ),
    (
        "ui::text::width",
        Demo("textutilities", "width 10 cells for"),
    ),
    ("ui::text::wider_than", Demo("textutilities", "wider_than")),
    (
        "ui::text::width_capped",
        Demo("textutilities", "width_capped"),
    ),
    (
        "ui::text::truncate",
        Demo("textutilities", "truncate payments"),
    ),
    (
        "ui::text::slice_cells",
        Demo("textutilities", "slice_cells"),
    ),
    (
        "ui::text::truncate_middle",
        Demo("textutilities", "truncate_middle"),
    ),
    (
        "ui::text::thousands",
        Demo("textutilities", "thousands 1,234,567"),
    ),
    ("ui::text::fit", Demo("textutilities", "fit [ok")),
    ("ui::text::fit_right", Demo("textutilities", "fit_right")),
    ("ui::text::center_x", Demo("textutilities", "center_x")),
    (
        "ui::text::plural",
        Demo("textutilities", "plural 1 row · 3 rows"),
    ),
    (
        "ui::text::truncate_sep",
        Demo("textutilities", "truncate_sep"),
    ),
    ("ui::text::crumb_fit", Demo("textutilities", "crumb_fit")),
    ("ui::text::clip", Demo("textutilities", "clip")),
    (
        "ui::text::duration",
        Demo("textutilities", "duration Spoken"),
    ),
    ("ui::text::bytes", Demo("textutilities", "bytes 512")),
    ("ui::text::wrap_with", Demo("textutilities", "wrap_with at")),
    (
        "ui::text::find_ranges",
        Demo("textutilities", "find_ranges"),
    ),
    (
        "widgets::hintbar::HintBar::resolve",
        Demo("statushints", "HintBar::resolve"),
    ),
    (
        "widgets::keyhint::toggle_hint",
        Demo("statushints", "toggle_hint Ctrl+H"),
    ),
    (
        "widgets::keyhint::render_toned",
        Demo("statushints", "render_toned"),
    ),
    (
        "widgets::keyhint::render_status",
        Demo("statushints", "render_status"),
    ),
    (
        "widgets::props::render_with",
        Demo("textutilities", "props::render_with"),
    ),
    (
        "widgets::statusbar::StatusBar::layout",
        Demo("chrome", "StatusBar::layout places"),
    ),
    (
        "theme::Theme::interactive",
        Demo("overview", "Theme::interactive(kind, state, plane)"),
    ),
    (
        "theme::Theme::legible",
        Demo("overview", "Theme::legible steps it up"),
    ),
    ("theme::contrast_ratio", Demo("overview", "contrast_ratio")),
];

/// Why a function counts as shown by the demo of its owning type. The one
/// place these categories are defined: nothing else of a type is covered
/// this way. Queries, labels and render helpers with something of their
/// own to show get a direct declaration or a pending allowlist line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViaRole {
    /// The widget protocol: `render*`, input handlers (`on_*`, `route*`),
    /// hit-region ids and measurement. Every demo of the owner runs them.
    Protocol,
    /// An operation an app calls on an event (open, select, scroll, edit,
    /// commit, ...). The owner's playground drives it.
    Operation,
    /// Plumbing of a shared `core`/`ui` type behind the widgets. Every
    /// demo of a widget built on the owner runs it.
    Plumbing,
}

impl ViaRole {
    /// The reason a function of this role needs no surface of its own.
    pub const fn reason(self) -> &'static str {
        match self {
            Self::Protocol => "widget protocol: every demo of its owner runs it",
            Self::Operation => {
                "operation an app calls on an event: the owner's playground drives it"
            }
            Self::Plumbing => "plumbing behind the widgets: every demo built on its owner runs it",
        }
    }
}

/// Functions covered by the demo of their owner: `(owner type id, role,
/// functions)`. A bare name is a method of the owner (`owner::name`); a
/// path is a free function, filed under the type whose demo runs it.
///
/// The gate (`app_tests_catalog.rs`) fails a `Via` whose owner is not a
/// type declared directly in [`SHOWN`] (a `Via` never counts: the owner
/// must be demonstrated itself), a name that is not a function of the
/// catalog, a bare name of another type, a path that is a method, and a
/// function that is also declared directly or allowlisted.
pub static VIA: &[(&str, ViaRole, &[&str])] = &[
    (
        "widgets::file_browser::FileBrowser",
        Protocol,
        &[
            "load",
            "on_key",
            "pointer",
            "on_paste",
            "is_editing",
            "on_blur",
            "render",
            "initial_focus",
            "hints",
            "url_button",
        ],
    ),
    ("widgets::file_browser::FileEntry", Operation, &["name"]),
    (
        "ui::content::ContentState",
        Protocol,
        &[
            "is_ready",
            "actions",
            "ui::content::draw_line",
            "ui::content::draw_loading",
            "ui::content::draw_empty",
            "ui::content::specimen",
        ],
    ),
    (
        "ui::list_core::ListCore",
        Plumbing,
        &[
            "cursor_model",
            "key_step",
            "has_rows",
            "is_focus_stop",
            "draw_content",
        ],
    ),
    (
        "widgets::notice::Notice",
        Protocol,
        &["height", "render", "render_theme"],
    ),
    (
        "widgets::meter_list::MeterList",
        Protocol,
        &["height", "label_column", "meter_column", "render"],
    ),
    ("core::edit::FieldKeys", Plumbing, &["text_keymap"]),
    ("core::edit::FocusSelect", Plumbing, &["apply"]),
    (
        "core::edit::UndoStack",
        Plumbing,
        &[
            "record", "seal", "clear", "can_undo", "can_redo", "undo", "redo",
        ],
    ),
    ("core::event::ClientPlatform", Plumbing, &["is_macos"]),
    (
        "core::focus::Focus",
        Plumbing,
        &[
            "core::focus::forward_blur",
            "core::focus::cancel_focus_request",
            "current",
            "is",
            "set",
            "focus",
            "next",
            "prev",
            "moves",
            "apply_request",
            "settle",
            "end_edit",
            "take_changes",
            "ensure_valid",
            "ensure_valid_with",
        ],
    ),
    (
        "core::focus::FocusRing",
        Plumbing,
        &[
            "register",
            "mark_selection",
            "has_selection",
            "len",
            "is_empty",
            "sort_from",
            "push_barrier",
            "reachable",
            "contains",
            "first",
            "next",
            "prev",
        ],
    ),
    (
        "core::keyboard::KeyboardProtocol",
        Plumbing,
        &[
            "core::keyboard::push",
            "core::keyboard::parse_flags_reply",
            "core::keyboard::modifiers",
            "core::keyboard::modifier_param",
            "core::keyboard::encode",
            "core::keyboard::decode",
            "core::keyboard::active",
            "core::keyboard::super_delivered",
            "core::keyboard::set_super_delivered",
            "core::keyboard::observe",
            "core::keyboard::alt_delivered",
            "core::keyboard::set_alt_delivered",
            "core::keyboard::set_alt_keys",
            "core::keyboard::label_deliverable",
        ],
    ),
    (
        "core::keyboard::ProtocolState",
        Plumbing,
        &["enter", "leave", "resume"],
    ),
    (
        "core::list::Cursor",
        Plumbing,
        &[
            "apply",
            "step",
            "first",
            "last",
            "select",
            "type_ahead",
            "core::list::route_key",
            "core::nav::step_index",
        ],
    ),
    ("core::list::KeyStep", Plumbing, &["of", "ordered"]),
    (
        "core::list::Moved",
        Plumbing,
        &["index", "moved", "outcome"],
    ),
    (
        "core::list::NavProfile",
        Plumbing,
        &["nav", "type_ahead", "install"],
    ),
    ("core::list::TypeAheadPrecedence", Plumbing, &["install"]),
    ("core::scroll::HScroll", Plumbing, &["shows_bar"]),
    (
        "core::scroll::ScrollPane",
        Plumbing,
        &[
            "set_content",
            "set_viewport",
            "scroll_by",
            "scroll_to",
            "ensure_visible",
            "follow",
            "begin",
            "v_click",
            "v_track",
            "h_track",
            "set_track",
            "clear_track",
            "held",
            "axis",
            "layout_columns",
            "reveal_column",
            "h_step",
            "h_scrolls",
            "wheel",
            "press",
            "press_on",
            "drag",
            "drag_on",
            "release",
            "axis_of",
            "bar_of",
            "bar",
            "draw_v",
            "draw_divider",
            "draw_h",
            "draw_with_fade",
        ],
    ),
    (
        "core::scroll::ScrollState",
        Plumbing,
        &[
            "core::pointer::scroll_input",
            "max_offset",
            "overflows",
            "at_end",
            "set_viewport",
            "set_content",
            "clamp",
            "scroll_by",
            "scroll_to",
            "page_up",
            "page_down",
            "jump_start",
            "jump_end",
            "pointer_move",
            "ensure_visible",
            "reveal_span",
            "follow",
            "visible_range",
            "thumb",
            "offset_for_thumb_start",
            "offset_for_track_pos",
            "press_track",
            "drag_track",
            "release_track",
            "press_track_with",
            "ui::fade::scroll_edges",
            "ui::fade::scroll_edges_except",
        ],
    ),
    ("core::scroll::ScrollbarVisibility", Plumbing, &["shows"]),
    ("core::selection::Activate", Plumbing, &["fires"]),
    (
        "core::selection::ClickPolicy",
        Plumbing,
        &["activates", "edits", "toggles"],
    ),
    (
        "core::selection::RowSelection",
        Plumbing,
        &[
            "len",
            "is_empty",
            "contains",
            "iter",
            "to_vec",
            "anchor",
            "clear",
            "select_only",
            "toggle",
            "apply",
            "context_select",
            "set",
            "extend",
            "select_all",
            "toggle_all",
            "tri_state",
            "retain",
            "remap",
        ],
    ),
    (
        "core::selection::TriState",
        Plumbing,
        &["glyph", "glyph_in"],
    ),
    ("core::shape::CaretShape", Plumbing, &["decscusr"]),
    (
        "core::text::TextBuffer",
        Plumbing,
        &[
            "text",
            "is_empty",
            "cursor_offset",
            "select_range",
            "selection_lines",
            "has_selection_lines",
            "has_selection",
            "insert_at",
            "remove_range",
            "set_text",
            "selection",
            "select_all",
            "clear_selection",
            "select_to",
            "word_range_at",
            "select_word_at",
            "select_line_at",
            "selected_text",
            "move_left",
            "move_right",
            "move_word_left",
            "move_word_right",
            "move_home",
            "move_end",
            "move_doc_start",
            "move_doc_end",
            "move_up",
            "move_down",
            "set_cursor_line_col",
            "insert_char",
            "insert_str",
            "backspace",
            "delete",
            "delete_word_left",
            "delete_to_line_end",
            "delete_to_line_start",
            "line_count",
            "cursor_pos",
            "pos_of",
            "offset_at",
            "width",
        ],
    ),
    (
        "core::typeahead::TypeAhead",
        Plumbing,
        &["char_of", "feed", "feed_at", "reset", "find"],
    ),
    ("theme::Ladder", Operation, &["step"]),
    ("theme::Strings", Operation, &["follow"]),
    (
        "ui::columns::ColumnSizing",
        Operation,
        &[
            "user",
            "set",
            "clear",
            "reset",
            "is_default",
            "press",
            "dragging",
            "drag_to",
            "release",
            "apply",
            "seam_rect",
        ],
    ),
    (
        "ui::columns::ColumnSizing",
        Protocol,
        &[
            "width",
            "ui::columns::render_seam",
            "ui::columns::seam_part",
        ],
    ),
    ("ui::context::ContextAnchor", Plumbing, &["apply"]),
    (
        "ui::context::ContextConfig",
        Plumbing,
        &["on_secondary", "on_key"],
    ),
    ("ui::context::ContextKeys", Plumbing, &["matches"]),
    ("ui::context::ContextRequest", Plumbing, &["menu"]),
    ("ui::layout::Split", Protocol, &["layout", "handle"]),
    ("ui::layout::Split", Protocol, &["vertical", "horizontal"]),
    (
        "ui::layout::Split",
        Operation,
        &["reset", "toggle_max", "grow", "drag_to", "nudge"],
    ),
    (
        "ui::layout::SplitKeys",
        Protocol,
        &["delta", "apply", "hint"],
    ),
    (
        "ui::pane_tree::PaneNode",
        Protocol,
        &["leaves", "layout", "node_at_mut", "path_to"],
    ),
    (
        "ui::pane_tree::PaneNode",
        Operation,
        &["split_leaf", "remove_leaf"],
    ),
    (
        "ui::pane_tree::PaneTree",
        Protocol,
        &[
            "ui::pane_tree::nearest",
            "render",
            "on_key",
            "on_pointer",
            "hints",
            "layout",
            "seam_id",
            "panes",
            "seams",
            "leaves",
            "zoomed",
            "framed",
        ],
    ),
    (
        "ui::pane_tree::PaneTree",
        Operation,
        &[
            "split",
            "close",
            "toggle_zoom",
            "focus_toward",
            "focus_next",
            "resize",
        ],
    ),
    (
        "ui::master_detail::MasterDetail",
        Protocol,
        &[
            "layout",
            "finish",
            "on_key",
            "on_pointer",
            "hints",
            "in_detail",
            "last",
        ],
    ),
    (
        "ui::master_detail::MasterDetail",
        Operation,
        &[
            "toggle_drawer",
            "open_drawer",
            "close_drawer",
            "drawer_open",
            "menu_item",
        ],
    ),
    ("ui::layout::Stack", Protocol, &["measure", "layout"]),
    ("ui::layout::StackScroll", Operation, &["reset"]),
    (
        "ui::layout::StackScroll",
        Protocol,
        &["on_wheel", "on_scrollbar", "on_scrollbar_drag"],
    ),
    (
        "ui::leave::LeaveGuard",
        Operation,
        &["request", "request_quit", "resolve"],
    ),
    ("ui::modal::Frame", Protocol, &["render"]),
    (
        "ui::popup::Dismiss",
        Plumbing,
        &["ui::popup::backdrop_id", "ui::popup::backdrop"],
    ),
    (
        "ui::popup::Placement",
        Plumbing,
        &[
            "ui::popup::place",
            "ui::popup::surface",
            "ui::popup::surface_for",
        ],
    ),
    (
        "ui::reveal::Reveal",
        Plumbing,
        &[
            "key_label",
            "ui::reveal::truncated",
            "ui::reveal::hover_ready",
            "ui::reveal::hover_wait",
            "ui::reveal::on_pointer_wheel",
            "ui::reveal::advertise",
            "ui::reveal::peek",
            "ui::reveal::queue_cut",
            "ui::reveal::cut_id",
        ],
    ),
    (
        "ui::reveal::RevealState",
        Plumbing,
        &[
            "effective",
            "available",
            "hint",
            "scrolls",
            "begin",
            "close",
            "on_key",
            "on_wheel",
            "inline",
            "cell",
        ],
    ),
    ("ui::specimen::KindSample", Plumbing, &["draw"]),
    (
        "ui::specimen::SpecimenState",
        Plumbing,
        &["slug", "label", "from_slug", "visual"],
    ),
    (
        "ui::specimen::StateGrid",
        Plumbing,
        &[
            "column_labels",
            "rendered_rows",
            "size",
            "mark",
            "render",
            "row_heights",
            "render_rows",
            "levels_height",
            "render_levels",
            "render_level",
        ],
    ),
    (
        "ui::tooltip::Tip",
        Plumbing,
        &[
            "ui::tooltip::tip_id",
            "ui::tooltip::show",
            "ui::tooltip::on_input",
        ],
    ),
    (
        "widgets::brand::Lockup",
        Protocol,
        &["width", "render", "render_clickable"],
    ),
    (
        "widgets::breadcrumb::Breadcrumb",
        Protocol,
        &["segment_id", "on_click", "render"],
    ),
    (
        "widgets::button::Button",
        Protocol,
        &["width", "on_key", "on_click", "render"],
    ),
    ("widgets::button::ButtonRow", Operation, &["step"]),
    (
        "widgets::button::ButtonRow",
        Protocol,
        &["more_id", "layout", "render"],
    ),
    (
        "widgets::chips::ChipBar",
        Protocol,
        &[
            "more_id", "menu_id", "chip_id", "close_id", "add_id", "lead_id", "on_key", "on_click",
            "render",
        ],
    ),
    (
        "widgets::choice::Checkbox",
        Protocol,
        &["on_key", "on_click", "render"],
    ),
    (
        "widgets::choice::RadioGroup",
        Protocol,
        &[
            "height",
            "on_key",
            "on_click",
            "option_id",
            "option_part",
            "render",
        ],
    ),
    (
        "widgets::choice::Toggle",
        Protocol,
        &["on_key", "on_click", "render"],
    ),
    (
        "widgets::choice_dialog::ChoiceDialog",
        Protocol,
        &[
            "initial_focus",
            "height",
            "on_key",
            "on_click",
            "render",
            "pointer_in",
        ],
    ),
    (
        "widgets::code::CodeEditor",
        Operation,
        &[
            "set_text",
            "jump_to",
            "begin_edit",
            "set_running",
            "commit",
            "open_find",
        ],
    ),
    (
        "widgets::code::CodeEditor",
        Protocol,
        &[
            "entry",
            "hints",
            "wants_text_input",
            "on_key",
            "on_click",
            "on_drag",
            "on_paste",
            "render",
        ],
    ),
    (
        "widgets::completion::Completion",
        Operation,
        &["open", "close", "apply"],
    ),
    (
        "widgets::completion::Completion",
        Protocol,
        &["row_id", "on_key", "on_click", "render"],
    ),
    (
        "widgets::dialog::Dialog",
        Protocol,
        &[
            "hints",
            "is_editing",
            "on_drag",
            "height",
            "on_key",
            "on_blur",
            "on_paste",
            "on_press",
            "on_click",
            "on_click_count",
            "on_click_outside",
            "render",
            "render_in",
            "pointer_in",
        ],
    ),
    (
        "widgets::diff::DiffView",
        Operation,
        &["set_file", "set_mode", "toggle_mode"],
    ),
    (
        "widgets::diff::DiffView",
        Protocol,
        &["id", "layout", "on_key", "on_click", "on_drag", "render"],
    ),
    (
        "widgets::empty::EmptyState",
        Protocol,
        &[
            "on_click",
            "on_key",
            "widgets::empty::render_ctx",
            "widgets::empty::render",
            "widgets::empty::height",
        ],
    ),
    (
        "widgets::field_common::PointerSelect",
        Protocol,
        &[
            "widgets::field_common::route_press",
            "widgets::field_common::route_drag",
            "widgets::field_common::route_click",
        ],
    ),
    (
        "widgets::form_dialog::FormDialog",
        Operation,
        &[
            "field_mut",
            "set_visible",
            "set_text",
            "set_chooser",
            "set_note",
        ],
    ),
    (
        "widgets::form_dialog::FormDialog",
        Protocol,
        &[
            "initial_focus",
            "is_editing",
            "on_key",
            "on_press",
            "on_drag",
            "on_release",
            "on_click",
            "on_click_count",
            "on_blur",
            "on_paste",
            "render",
            "pointer_in",
        ],
    ),
    (
        "widgets::grid::DataGrid",
        Operation,
        &[
            "set_rows",
            "append_rows",
            "set_loading",
            "begin_edit",
            "commit_edit",
            "cancel_edit",
            "record_cell",
            "toggle_delete",
            "insert_row",
            "duplicate_row",
            "undo",
            "apply_commit_result",
            "discard",
            "select_rows",
            "context_selection",
            "has_selection",
        ],
    ),
    (
        "widgets::grid::DataGrid",
        Protocol,
        &[
            "choice_id",
            "open_id",
            "on_header_action",
            "is_editing",
            "on_blur",
            "on_key",
            "header_id",
            "cell_id",
            "seam_id",
            "rownum_id",
            "more_id",
            "left_id",
            "right_id",
            "on_press",
            "on_click",
            "on_click_with",
            "on_drag",
            "on_paste",
            "render",
            "bar_ids",
            "on_bar_key",
        ],
    ),
    (
        "widgets::help::HelpOverlay",
        Protocol,
        &["on_click", "on_key", "render"],
    ),
    (
        "widgets::hintbar::HintBar",
        Protocol,
        &["render_ctx", "render_at", "render"],
    ),
    (
        "widgets::icon_button::IconButton",
        Protocol,
        &["width", "render", "render_in"],
    ),
    (
        "widgets::info_dialog::InfoDialog",
        Protocol,
        &[
            "initial_focus",
            "on_key",
            "on_click",
            "on_drag",
            "render",
            "pointer_in",
        ],
    ),
    (
        "widgets::input::TextInput",
        Operation,
        &[
            "set_edit_entry",
            "clear",
            "begin_edit",
            "commit",
            "cancel",
            "resume",
            "validate",
        ],
    ),
    (
        "widgets::input::TextInput",
        Protocol,
        &[
            "entry",
            "wants_text_input",
            "hints",
            "on_blur",
            "height",
            "height_for",
            "on_key",
            "on_paste",
            "on_click",
            "on_click_count",
            "on_press",
            "on_press_count",
            "on_drag",
            "render",
        ],
    ),
    (
        "widgets::keyhint::Hint",
        Protocol,
        &[
            "widgets::keyhint::render",
            "widgets::keyhint::render_aligned",
            "widgets::keyhint::render_status_at",
            "widgets::keyhint::render_status_styled_at",
            "widgets::keyhint::render_styled",
            "widgets::keyhint::render_parts",
            "widgets::keyhint::render_parts_at",
        ],
    ),
    (
        "widgets::list::ListBox",
        Operation,
        &[
            "activate",
            "select",
            "remove_rows",
            "is_checked",
            "set_checked",
        ],
    ),
    (
        "widgets::list::ListBox",
        Protocol,
        &[
            "row_id",
            "on_key",
            "on_click",
            "on_click_count",
            "on_click_at",
            "mark_id",
            "render",
        ],
    ),
    (
        "widgets::menu::ContextMenu",
        Protocol,
        &[
            "more_above_id",
            "more_below_id",
            "hides_page_focus",
            "row_id",
            "size",
            "on_key",
            "on_click",
            "on_click_outside",
            "render",
        ],
    ),
    (
        "widgets::menu::MenuBar",
        Protocol,
        &[
            "on_press",
            "more_id",
            "hides_page_focus",
            "label_id",
            "brand_id",
            "on_key",
            "on_click",
            "on_hover",
            "render",
        ],
    ),
    ("widgets::option_editor::ChoiceEditor", Operation, &["open"]),
    (
        "widgets::option_editor::ChoiceEditor",
        Protocol,
        &[
            "option_id",
            "on_key",
            "on_paste",
            "on_click",
            "wheel",
            "render",
        ],
    ),
    (
        "widgets::option_editor::ColumnEditor",
        Operation,
        &["toggled"],
    ),
    ("widgets::panel::Panel", Operation, &["mark_clipped"]),
    (
        "widgets::panel::Panel",
        Protocol,
        &["draw_meta", "try_render", "render"],
    ),
    ("widgets::panel::ScrollPanel", Operation, &["push"]),
    (
        "widgets::panel::ScrollPanel",
        Protocol,
        &["on_key", "measure", "render"],
    ),
    (
        "widgets::picker::Picker",
        Operation,
        &[
            "set_items",
            "refresh_items",
            "set_cursor",
            "query",
            "set_query",
            "current",
            "is_searchable",
            "set_searchable",
        ],
    ),
    (
        "widgets::picker::Picker",
        Protocol,
        &[
            "on_click_outside",
            "height",
            "on_paste",
            "row_id",
            "on_key",
            "on_click",
            "draw",
            "render",
        ],
    ),
    (
        "widgets::progress::Meter",
        Protocol,
        &["render", "widgets::progress::render_meter"],
    ),
    (
        "widgets::progress::ProgressBar",
        Protocol,
        &[
            "widgets::progress::render_bar",
            "widgets::progress::render_bar_with",
            "widgets::progress::render_indeterminate",
            "widgets::progress::render_indeterminate_with",
            "widgets::progress::render_spinner",
            "render",
        ],
    ),
    ("widgets::progress::Tally", Protocol, &["render"]),
    (
        "widgets::props::Prop",
        Protocol,
        &["widgets::props::measure", "widgets::props::render"],
    ),
    ("widgets::props::PropsList", Operation, &["set_props"]),
    (
        "widgets::props::PropsList",
        Protocol,
        &["row_id", "on_key", "on_click", "on_click_count", "render"],
    ),
    (
        "widgets::rail::SidebarRail",
        Protocol,
        &[
            "collapse_id",
            "width",
            "on_click",
            "render_rail",
            "render_collapse",
        ],
    ),
    (
        "widgets::scrollbar::Bars",
        Operation,
        &[
            "widgets::scrollbar::id_for",
            "widgets::scrollbar::h_id_for",
            "widgets::scrollbar::is_bar_of",
            "widgets::scrollbar::reserve",
            "widgets::scrollbar::offset_for_click",
            "widgets::scrollbar::press",
            "widgets::scrollbar::press_with",
            "widgets::scrollbar::drag",
            "widgets::scrollbar::press_h",
            "widgets::scrollbar::drag_h",
            "widgets::scrollbar::apply",
            "widgets::scrollbar::apply_with",
            "widgets::scrollbar::position_label",
        ],
    ),
    (
        "widgets::scrollbar::Bars",
        Protocol,
        &[
            "widgets::scrollbar::render_divider",
            "widgets::scrollbar::render_vertical",
            "widgets::scrollbar::render_horizontal",
            "widgets::scrollbar::render_horizontal_with",
            "widgets::scrollbar::render_with_fade",
            "widgets::scrollbar::render_with_fade_except",
        ],
    ),
    (
        "widgets::scrollbar::VSplit",
        Operation,
        &["widgets::scrollbar::split"],
    ),
    (
        "widgets::segments::Segment",
        Protocol,
        &["widgets::segments::render"],
    ),
    (
        "widgets::select::Select",
        Operation,
        &["set_mixed", "close"],
    ),
    (
        "widgets::select::Select",
        Protocol,
        &[
            "bar_id",
            "option_id",
            "on_key",
            "on_click",
            "on_blur",
            "on_click_outside",
            "on_press",
            "on_drag",
            "on_release",
            "render",
        ],
    ),
    (
        "widgets::splitter::Splitter",
        Protocol,
        &["render", "render_held", "on_press", "on_drag", "on_click"],
    ),
    (
        "widgets::statusbar::StatusBar",
        Protocol,
        &["widgets::statusbar::reveal_id", "render"],
    ),
    ("widgets::statusbar::StatusItem", Protocol, &["width"]),
    (
        "widgets::steps::StepRail",
        Operation,
        &["set_state", "set_meta"],
    ),
    (
        "widgets::steps::StepRail",
        Protocol,
        &["row_id", "on_key", "on_click", "render"],
    ),
    (
        "widgets::table::DataTable",
        Operation,
        &[
            "toggle_mods_for",
            "set_row_disabled",
            "select_all",
            "clear_selection",
            "toggle_row",
            "set_row",
            "remove_rows",
            "set_rows",
            "sort_by",
            "begin_edit",
            "commit_edit",
            "cancel_edit",
            "context_selection",
        ],
    ),
    (
        "widgets::table::DataTable",
        Protocol,
        &[
            "choice_id",
            "open_id",
            "check_id",
            "check_all_id",
            "on_click_check",
            "on_click_check_all",
            "seam_id",
            "on_press",
            "on_drag",
            "on_header_action",
            "is_editing",
            "header_id",
            "row_id",
            "cell_id",
            "on_click_editor",
            "on_blur",
            "on_key",
            "on_paste",
            "on_click_header",
            "on_click_cell",
            "on_click_cell_with",
            "on_click_cell_count",
            "render",
        ],
    ),
    (
        "widgets::tabs::Tabs",
        Operation,
        &["set_items", "set_active", "remove"],
    ),
    (
        "widgets::tabs::Tabs",
        Protocol,
        &[
            "more_id",
            "menu_id",
            "on_wheel",
            "on_middle_click",
            "tab_id",
            "close_id",
            "new_id",
            "left_id",
            "right_id",
            "on_key",
            "on_click",
            "render",
        ],
    ),
    (
        "widgets::textarea::TextArea",
        Operation,
        &["begin_edit", "commit"],
    ),
    (
        "widgets::textarea::TextArea",
        Protocol,
        &[
            "entry",
            "wants_text_input",
            "hints",
            "on_blur",
            "height",
            "on_key",
            "on_paste",
            "on_click",
            "on_click_count",
            "on_press",
            "on_press_count",
            "on_drag",
            "render",
        ],
    ),
    ("widgets::too_small::TooSmall", Protocol, &["render"]),
    (
        "widgets::toolbar::Toolbar",
        Protocol,
        &[
            "more_id", "menu_id", "on_more", "item_id", "width", "on_key", "on_click", "render",
        ],
    ),
    (
        "widgets::tree::TreeView",
        Operation,
        &[
            "node_mut",
            "set_children",
            "set_busy",
            "set_filter",
            "reveal",
            "toggle",
            "expand_all",
            "collapse_all",
            "toggle_recursive",
        ],
    ),
    (
        "widgets::tree::TreeView",
        Protocol,
        &[
            "row_id",
            "toggle_id",
            "on_key",
            "on_click_row",
            "on_click_toggle",
            "on_click_with",
            "on_click_at",
            "on_click_row_count",
            "row_at",
            "glyph_id",
            "render",
        ],
    ),
    (
        "widgets::viewport::TextViewport",
        Protocol,
        &[
            "widgets::viewport::render_spans",
            "on_click",
            "on_drag",
            "on_key",
            "render",
        ],
    ),
];

/// [`SHOWN`] plus every [`VIA`] entry as a [`Shown::Via`] declaration:
/// what the coverage gate counts as shown.
pub fn declarations() -> &'static [(&'static str, Shown)] {
    static ALL: std::sync::OnceLock<Vec<(&'static str, Shown)>> = std::sync::OnceLock::new();
    ALL.get_or_init(|| expand(SHOWN, VIA))
}

/// `shown` plus each function of `via` as a [`Shown::Via`] of its owner.
pub fn expand(
    shown: &[(&'static str, Shown)],
    via: &[(&'static str, ViaRole, &'static [&'static str])],
) -> Vec<(&'static str, Shown)> {
    let mut all = shown.to_vec();
    for (owner, _, names) in via {
        for name in *names {
            let id: &'static str = if name.contains("::") {
                name
            } else {
                Box::leak(format!("{owner}::{name}").into_boxed_str())
            };
            all.push((id, Shown::Via(owner)));
        }
    }
    all
}
