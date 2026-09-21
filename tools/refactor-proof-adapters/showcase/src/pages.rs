//! Twenty-three oracle Showcase pages, including Diff.
//!
//! Candidate production currently exposes 22 routes and no Diff module.
//! Mapping is observation-only: a missing production route is recorded, not
//! repaired.

use showcase_app::PageId;

/// Oracle page identity from the pinned 23-entry navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OraclePageId {
    /// Overview / overview.
    Overview,
    /// Buttons / buttons.
    Buttons,
    /// Inputs / inputs.
    Inputs,
    /// Text areas / textareas.
    TextAreas,
    /// Forms / forms.
    Forms,
    /// Lists / lists.
    Lists,
    /// Trees / trees.
    Trees,
    /// Tables / tables.
    Tables,
    /// Editable tables / editabletables.
    Editable,
    /// Panels / panels.
    Panels,
    /// Sidebars / sidebars.
    Sidebars,
    /// Dialogs / dialogs.
    Dialogs,
    /// Progress / progress.
    Progress,
    /// Scrolling / scrolling.
    Scrolling,
    /// Terminal / terminal.
    Terminal,
    /// Code editor / codeeditor.
    Editor,
    /// Diff / diff (oracle-only; production has no route).
    Diff,
    /// Data grid / datagrid.
    Grid,
    /// Chips & selects / chipsselects.
    Chips,
    /// Pickers / pickers.
    Pickers,
    /// Chrome / chrome.
    Chrome,
    /// Settings / settings.
    Settings,
    /// Task runner / taskrunner.
    TaskRunner,
}

impl OraclePageId {
    /// Oracle navigation order, 23 pages including Diff after the editor.
    pub const ALL: [Self; 23] = [
        Self::Overview,
        Self::Buttons,
        Self::Inputs,
        Self::TextAreas,
        Self::Forms,
        Self::Lists,
        Self::Trees,
        Self::Tables,
        Self::Editable,
        Self::Panels,
        Self::Sidebars,
        Self::Dialogs,
        Self::Progress,
        Self::Scrolling,
        Self::Terminal,
        Self::Editor,
        Self::Diff,
        Self::Grid,
        Self::Chips,
        Self::Pickers,
        Self::Chrome,
        Self::Settings,
        Self::TaskRunner,
    ];

    /// Scenario TSV `page` column / oracle CLI normalized name.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Buttons => "buttons",
            Self::Inputs => "inputs",
            Self::TextAreas => "textareas",
            Self::Forms => "forms",
            Self::Lists => "lists",
            Self::Trees => "trees",
            Self::Tables => "tables",
            Self::Editable => "editabletables",
            Self::Panels => "panels",
            Self::Sidebars => "sidebars",
            Self::Dialogs => "dialogs",
            Self::Progress => "progress",
            Self::Scrolling => "scrolling",
            Self::Terminal => "terminal",
            Self::Editor => "codeeditor",
            Self::Diff => "diff",
            Self::Grid => "datagrid",
            Self::Chips => "chipsselects",
            Self::Pickers => "pickers",
            Self::Chrome => "chrome",
            Self::Settings => "settings",
            Self::TaskRunner => "taskrunner",
        }
    }

    /// Oracle navigation label.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Buttons => "Buttons",
            Self::Inputs => "Inputs",
            Self::TextAreas => "Text areas",
            Self::Forms => "Forms",
            Self::Lists => "Lists",
            Self::Trees => "Trees",
            Self::Tables => "Tables",
            Self::Editable => "Editable tables",
            Self::Panels => "Panels",
            Self::Sidebars => "Sidebars",
            Self::Dialogs => "Dialogs",
            Self::Progress => "Progress",
            Self::Scrolling => "Scrolling",
            Self::Terminal => "Terminal",
            Self::Editor => "Code editor",
            Self::Diff => "Diff",
            Self::Grid => "Data grid",
            Self::Chips => "Chips & selects",
            Self::Pickers => "Pickers",
            Self::Chrome => "Chrome",
            Self::Settings => "Settings",
            Self::TaskRunner => "Task runner",
        }
    }

    /// Parse a TSV page token (`overview`, `all23` is not a page).
    #[must_use]
    pub fn from_slug(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|page| page.slug() == value || page.title() == value)
    }

    /// Production route, if the candidate App currently exposes it.
    ///
    /// Uses the same `PageId::from_name` parser as the production `--page`
    /// CLI route; its alphanumeric normalization resolves oracle slugs such
    /// as `codeeditor` through production titles.
    #[must_use]
    pub fn production(self) -> Option<PageId> {
        PageId::from_name(self.slug()).or_else(|| PageId::from_name(self.title()))
    }

    /// Whether production currently constructs this page.
    #[must_use]
    pub fn production_present(self) -> bool {
        self.production().is_some()
    }

    /// Map a production page onto the oracle identity (never Diff).
    #[must_use]
    pub const fn from_production(page: PageId) -> Self {
        match page {
            PageId::Overview => Self::Overview,
            PageId::Buttons => Self::Buttons,
            PageId::Inputs => Self::Inputs,
            PageId::TextAreas => Self::TextAreas,
            PageId::Forms => Self::Forms,
            PageId::Lists => Self::Lists,
            PageId::Trees => Self::Trees,
            PageId::Tables => Self::Tables,
            PageId::Editable => Self::Editable,
            PageId::Panels => Self::Panels,
            PageId::Sidebars => Self::Sidebars,
            PageId::Dialogs => Self::Dialogs,
            PageId::Progress => Self::Progress,
            PageId::Scrolling => Self::Scrolling,
            PageId::Terminal => Self::Terminal,
            PageId::Editor => Self::Editor,
            PageId::Grid => Self::Grid,
            PageId::Chips => Self::Chips,
            PageId::Pickers => Self::Pickers,
            PageId::Chrome => Self::Chrome,
            PageId::Settings => Self::Settings,
            PageId::TaskRunner => Self::TaskRunner,
        }
    }
}

/// Production navigation length observed from the candidate App.
#[must_use]
pub fn production_nav_len() -> usize {
    showcase_app::NAV_ENTRIES.len()
}

/// Production `PageId::ALL` length.
#[must_use]
pub fn production_page_len() -> usize {
    PageId::ALL.len()
}
