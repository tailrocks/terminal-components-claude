use crate::pane_tree::PaneTreePage;
use showcase_demos::render_number::RenderNumber;
use termrock::author::PaintStyle;
use termrock::overlays::NoticeReveal;
use termrock::{
    ActionKey, Anchor, App as TuiApp, Brand, Chord, ColorLevel, CrossAlign, Cx, Dialog,
    DialogAction, DialogState, Dismiss, EmptyState, FrameRead, GlyphRole, HelpOverlay,
    HelpOverlayState, HelpSection, Id, Intent, ItemKey, KeyCode, KeyMap, KeyPhase, LayerOpening,
    Moment, Panel, PanelKind, Part, PartRef, Phase, Position, Props, Rect, Response, RowFn, Side,
    Size, StateFlags, Status, StatusBar, StatusItem, Theme, TooSmall, Tree, TreeAction, TreeNode,
    TreeState, Ui, Variant, id, width,
};

use showcase_demos::pages::forms::SUBMIT as FORM_SUBMIT;
use showcase_demos::pages::taskrunner::RUN_COMMAND;
use showcase_demos::pages::{
    Page, PageStatus,
    buttons::ButtonsPage,
    celleditors::CellEditorsPage,
    checks_radios::ChecksRadiosPage,
    chips::ChipsPage,
    chrome::ChromePage,
    dialog_kinds::DialogKindsPage,
    dialogs::DialogsPage,
    diff::DiffPage,
    editable::EditablePage,
    editor::EditorPage,
    empty_error::EmptyErrorPage,
    forms::FormsPage,
    grid::GridPage,
    inputs::InputsPage,
    lists::ListsPage,
    master_detail::MasterDetailPage,
    notices::{NOTICE_REVEAL_OWNERS, NoticeRevealTarget, NoticesPage},
    overview::OverviewPage,
    panels::PanelsPage,
    pickers::PickersPage,
    progress::ProgressPage,
    scrolling::ScrollingPage,
    settings::SettingsPage,
    sidebars::SidebarsPage,
    tables::TablesPage,
    taskrunner::TaskRunnerPage,
    terminal::TerminalPage,
    textareas::TextAreasPage,
    toolbars::ToolbarsPage,
    trees::TreesPage,
};

const NAV: Id = id!("navigation");
const CHECKS_RADIOS_ENTRY: usize = 12;
const EMPTY_ERROR_ENTRY: usize = 31;
const NOTICES_ENTRY: usize = 33;
const DIFF_ENTRY: usize = 40;
const MASTER_DETAIL_ENTRY: usize = 37;
const PANE_TREE_ENTRY: usize = 38;
const BRAND: Id = id!("brand");
const STATUS: Id = id!("status");
const HELP: Id = id!("help");
const TOO_SMALL: Id = id!("too-small");
const HEADER_HELP: Id = id!("header.help");
const HEADER_INSPECT: Id = id!("header.inspect");
const INSPECTOR: Id = id!("inspector");
const INSPECTOR_CLOSE: Id = INSPECTOR.part(Part::ACTIONS).index(0);
const QUIT: ActionKey = ActionKey::application("showcase.quit");
const QUIT_CTRL: ActionKey = ActionKey::application("showcase.quit.ctrl");
const HELP_COMMAND: ActionKey = ActionKey::application("showcase.help");
const INSPECTOR_COMMAND: ActionKey = ActionKey::application("showcase.inspector");
const NEXT_PAGE: ActionKey = ActionKey::application("showcase.page.next");
const PREV_PAGE: ActionKey = ActionKey::application("showcase.page.previous");
const NAV_PANEL_PARTS: [(Part, termrock::StylePatch); 1] = [(
    Part::DETAIL,
    termrock::StylePatch::new().set_fg(termrock::Role::Fg(termrock::FgStep::Faint)),
)];
const HELP_SHELL_ROWS: &[(&str, &str)] = &[
    ("F1 ?", "This reference"),
    ("F10 Ctrl+F2", "Menu bar"),
    ("Ctrl+PgDn ]", "Next page"),
    ("Ctrl+PgUp [", "Previous page"),
    ("0", "Jump to navigation"),
    ("i", "The Inspector"),
    ("Tab", "Next control"),
    ("Shift+Tab", "Previous control"),
    ("Esc", "Back"),
    ("Ctrl+Q q", "Quit"),
    ("Ctrl+C", "Copy, else quit"),
];
const HELP_SIDEBAR_ROWS: &[(&str, &str)] = &[
    ("↑↓", "Move"),
    ("PgUp PgDn", "Page"),
    ("Home End", "First / last"),
    ("a–z", "Jump to a row by name"),
    ("←→", "Fold / unfold"),
    ("Enter", "Activate"),
    ("*", "Expand all"),
    ("-", "Collapse all"),
    ("/", "Filter (Esc clears)"),
];
const HELP_MOUSE_ROWS: &[(&str, &str)] = &[
    ("Hover", "Preview"),
    ("Click", "Focus and activate"),
    ("Wheel", "Scroll"),
    ("Drag", "A scrollbar thumb"),
];
const HELP_SECTIONS: [HelpSection<'static>; 3] = [
    HelpSection::plain("Shell", HELP_SHELL_ROWS),
    HelpSection::plain("Sidebar", HELP_SIDEBAR_ROWS),
    HelpSection::plain("Mouse", HELP_MOUSE_ROWS),
];

const STATUS_LEFT: [StatusItem<'static>; 1] = [StatusItem::new("showcase")];
const STATUS_RIGHT: [StatusItem<'static>; 2] = [
    StatusItem::new("q quit").priority(10),
    StatusItem::new("? help").priority(5),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageId {
    Overview,
    Buttons,
    Inputs,
    TextAreas,
    Forms,
    Lists,
    Trees,
    Tables,
    Editable,
    Panels,
    Sidebars,
    Dialogs,
    Progress,
    Scrolling,
    Terminal,
    Editor,
    Grid,
    Chips,
    Pickers,
    DialogKinds,
    Chrome,
    Settings,
    TaskRunner,
    CellEditors,
    Toolbars,
}

impl PageId {
    pub const ALL: [Self; 25] = [
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
        Self::Grid,
        Self::Chips,
        Self::Pickers,
        Self::DialogKinds,
        Self::Chrome,
        Self::Settings,
        Self::TaskRunner,
        Self::CellEditors,
        Self::Toolbars,
    ];

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
            Self::Grid => "Data grid",
            Self::Chips => "Chips & selects",
            Self::Pickers => "Pickers",
            Self::DialogKinds => "Dialog kinds",
            Self::Chrome => "Chrome",
            Self::Settings => "Settings",
            Self::TaskRunner => "Task runner",
            Self::CellEditors => "Cell editors",
            Self::Toolbars => "Toolbars",
        }
    }

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
            Self::Editable => "editable",
            Self::Panels => "panels",
            Self::Sidebars => "sidebars",
            Self::Dialogs => "dialogs",
            Self::Progress => "progress",
            Self::Scrolling => "scrolling",
            Self::Terminal => "terminal",
            Self::Editor => "editor",
            Self::Grid => "grid",
            Self::Chips => "chips",
            Self::Pickers => "pickers",
            Self::DialogKinds => "dialog-kinds",
            Self::Chrome => "chrome",
            Self::Settings => "settings",
            Self::TaskRunner => "taskrunner",
            Self::CellEditors => "celleditors",
            Self::Toolbars => "toolbars",
        }
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|page| *page == self).unwrap_or(0)
    }

    pub fn from_name(value: &str) -> Option<Self> {
        nav_index_from_name(value).and_then(|index| match nav_target_at(index)? {
            NavTarget::Live(page) => Some(page),
            NavTarget::Selection | NavTarget::Placeholder(_) => None,
        })
    }

    pub fn from_key(key: ItemKey) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|page| ItemKey::text(page.slug()) == key)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NavEntry {
    pub id: PageId,
    pub label: &'static str,
    pub section: &'static str,
    pub icon: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NavTarget {
    Live(PageId),
    Selection,
    Placeholder(usize),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavExpand {
    #[default]
    All,

    Accordion,

    CurrentGroup,
}

impl NavTarget {
    const fn page(self) -> Option<PageId> {
        match self {
            Self::Live(page) => Some(page),
            Self::Selection | Self::Placeholder(_) => None,
        }
    }

    const fn has_content(self) -> bool {
        matches!(
            self,
            Self::Live(_)
                | Self::Selection
                | Self::Placeholder(CHECKS_RADIOS_ENTRY)
                | Self::Placeholder(EMPTY_ERROR_ENTRY)
                | Self::Placeholder(NOTICES_ENTRY)
                | Self::Placeholder(DIFF_ENTRY)
                | Self::Placeholder(MASTER_DETAIL_ENTRY)
                | Self::Placeholder(PANE_TREE_ENTRY)
        )
    }
}

impl std::fmt::Display for NavEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label)
    }
}

pub const NAV_ENTRIES: &[NavEntry] = &[
    NavEntry {
        id: PageId::Overview,
        label: "Overview",
        section: "",
        icon: "•",
    },
    NavEntry {
        id: PageId::Chrome,
        label: "Chrome",
        section: "App shell",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Status & hints",
        section: "App shell",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Breadcrumbs & rails",
        section: "App shell",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Help & keys",
        section: "App shell",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Quit & leave",
        section: "App shell",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Focus & navigation",
        section: "App shell",
        icon: "•",
    },
    NavEntry {
        id: PageId::Buttons,
        label: "Buttons",
        section: "Inputs & forms",
        icon: "•",
    },
    NavEntry {
        id: PageId::Inputs,
        label: "Inputs",
        section: "Inputs & forms",
        icon: "•",
    },
    NavEntry {
        id: PageId::TextAreas,
        label: "Text areas",
        section: "Inputs & forms",
        icon: "•",
    },
    NavEntry {
        id: PageId::Chips,
        label: "Chips & selects",
        section: "Inputs & forms",
        icon: "•",
    },
    NavEntry {
        id: PageId::Forms,
        label: "Forms",
        section: "Inputs & forms",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Checks & radios",
        section: "Inputs & forms",
        icon: "•",
    },
    NavEntry {
        id: PageId::Toolbars,
        label: "Toolbars",
        section: "Inputs & forms",
        icon: "•",
    },
    NavEntry {
        id: PageId::Tables,
        label: "Basic table",
        section: "Tables",
        icon: "•",
    },
    NavEntry {
        id: PageId::Editable,
        label: "Editable",
        section: "Tables",
        icon: "•",
    },
    NavEntry {
        id: PageId::Grid,
        label: "Data grid",
        section: "Tables",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Selection",
        section: "Tables",
        icon: "•",
    },
    NavEntry {
        id: PageId::CellEditors,
        label: "Cell editors",
        section: "Tables",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Columns",
        section: "Tables",
        icon: "•",
    },
    NavEntry {
        id: PageId::Lists,
        label: "Lists",
        section: "Navigation",
        icon: "•",
    },
    NavEntry {
        id: PageId::Trees,
        label: "Trees",
        section: "Navigation",
        icon: "•",
    },
    NavEntry {
        id: PageId::Sidebars,
        label: "Sidebars",
        section: "Navigation",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Tabs",
        section: "Navigation",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Menus",
        section: "Navigation",
        icon: "•",
    },
    NavEntry {
        id: PageId::Dialogs,
        label: "Dialogs",
        section: "Overlays",
        icon: "•",
    },
    NavEntry {
        id: PageId::Pickers,
        label: "Pickers",
        section: "Overlays",
        icon: "•",
    },
    NavEntry {
        id: PageId::DialogKinds,
        label: "Dialog kinds",
        section: "Overlays",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Popups & tooltips",
        section: "Overlays",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "File browser",
        section: "Overlays",
        icon: "•",
    },
    NavEntry {
        id: PageId::Progress,
        label: "Progress",
        section: "Feedback",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Empty & error",
        section: "Feedback",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Meters",
        section: "Feedback",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Notices",
        section: "Feedback",
        icon: "•",
    },
    NavEntry {
        id: PageId::Panels,
        label: "Panels",
        section: "Layout",
        icon: "•",
    },
    NavEntry {
        id: PageId::Scrolling,
        label: "Scrolling",
        section: "Layout",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Splits & stacks",
        section: "Layout",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Master-detail",
        section: "Layout",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Pane tree",
        section: "Layout",
        icon: "•",
    },
    NavEntry {
        id: PageId::Editor,
        label: "Code editor",
        section: "Text & code",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Diff",
        section: "Text & code",
        icon: "•",
    },
    NavEntry {
        id: PageId::Terminal,
        label: "Terminal",
        section: "Text & code",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Text utilities",
        section: "Text & code",
        icon: "•",
    },
    NavEntry {
        id: PageId::Settings,
        label: "Settings",
        section: "Screens",
        icon: "•",
    },
    NavEntry {
        id: PageId::TaskRunner,
        label: "Task runner",
        section: "Screens",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Query workbench",
        section: "Screens",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Connections",
        section: "Screens",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Finder",
        section: "Screens",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Disk cleanup",
        section: "Screens",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Activity",
        section: "Screens",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Workspaces",
        section: "Screens",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Accounts & usage",
        section: "Screens",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Launch cockpit",
        section: "Screens",
        icon: "•",
    },
    NavEntry {
        id: PageId::Overview,
        label: "Read-only report",
        section: "Screens",
        icon: "•",
    },
];

const NAV_ALIASES: &[(&str, &str)] = &[
    ("tables", "Basic table"),
    ("editabletables", "Editable"),
    ("selectabletable", "Selection"),
    ("optioneditors", "Cell editors"),
    ("options", "Cell editors"),
    ("enums", "Cell editors"),
    ("choices", "Cell editors"),
    ("selection", "Selection"),
    ("bulkedit", "Selection"),
    ("selectable", "Selection"),
    ("hints", "Status & hints"),
    ("breadcrumbs", "Breadcrumbs & rails"),
    ("rails", "Breadcrumbs & rails"),
    ("help", "Help & keys"),
    ("quit", "Quit & leave"),
    ("leave", "Quit & leave"),
    ("focus", "Focus & navigation"),
    ("checks", "Checks & radios"),
    ("radios", "Checks & radios"),
    ("popups", "Popups & tooltips"),
    ("tooltips", "Popups & tooltips"),
    ("empty", "Empty & error"),
    ("splits", "Splits & stacks"),
    ("text", "Text utilities"),
    ("datagrid", "Data grid"),
    ("codeeditor", "Code editor"),
    ("chipsselects", "Chips & selects"),
    ("queryworkbench", "Query workbench"),
    ("celleditors", "Cell editors"),
];

fn normalized_name(input: &str) -> String {
    input
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|character| character.to_ascii_lowercase())
        .collect()
}

fn nav_index_from_name(value: &str) -> Option<usize> {
    let value = normalized_name(value);
    if let Some(label) = NAV_ALIASES
        .iter()
        .find(|(alias, _)| *alias == value)
        .map(|(_, label)| *label)
    {
        return NAV_ENTRIES.iter().position(|entry| entry.label == label);
    }
    if let Some(index) = NAV_ENTRIES
        .iter()
        .position(|entry| normalized_name(entry.label) == value)
    {
        return Some(index);
    }
    NAV_ENTRIES
        .iter()
        .enumerate()
        .find(|(index, entry)| {
            matches!(nav_target_for_entry(entry, *index), NavTarget::Live(_))
                && (normalized_name(entry.id.slug()) == value
                    || normalized_name(entry.id.title()) == value)
        })
        .map(|(index, _)| index)
}

fn index_for_page(page: PageId) -> usize {
    NAV_ENTRIES
        .iter()
        .enumerate()
        .position(|(index, entry)| nav_target_for_entry(entry, index) == NavTarget::Live(page))
        .unwrap_or(0)
}

const NAV_ENTRY_KEYSPACE: u64 = 0x4e41_5645_4e54_5259;
const NAV_GROUP_KEYSPACE: u64 = 0x4e41_5647_524f_5550;

#[derive(Clone, Debug, PartialEq, Eq)]
struct NavQuery {
    text: String,
    revision: u64,
}

fn nav_target_for_entry(entry: &NavEntry, index: usize) -> NavTarget {
    if entry.label == "Selection" {
        NavTarget::Selection
    } else if entry.label == "Overview" || entry.id != PageId::Overview {
        NavTarget::Live(entry.id)
    } else {
        NavTarget::Placeholder(index)
    }
}

fn nav_entry_key(index: usize) -> ItemKey {
    ItemKey::pair(NAV_ENTRY_KEYSPACE, index as u64)
}

fn nav_group_key(index: usize) -> ItemKey {
    ItemKey::pair(NAV_GROUP_KEYSPACE, index as u64)
}

fn nav_index_for_target(target: NavTarget) -> Option<usize> {
    match target {
        NavTarget::Placeholder(index) => NAV_ENTRIES
            .get(index)
            .filter(|entry| nav_target_for_entry(entry, index) == target)
            .map(|_| index),
        _ => NAV_ENTRIES
            .iter()
            .enumerate()
            .find(|(index, entry)| nav_target_for_entry(entry, *index) == target)
            .map(|(index, _)| index),
    }
}

fn nav_display_label(label: &str) -> &str {
    match label {
        "Basic table" => "Tables",
        "Editable" => "Editable tables",
        _ => label,
    }
}

#[derive(Clone, Copy, Debug)]
struct NavNode {
    label: &'static str,
    depth: u16,
    parent: bool,
    entry: Option<usize>,
    target: Option<NavTarget>,
    key: ItemKey,
    query_match: bool,
    fade: u8,
}

fn nav_nodes(state: &TreeState, viewport_height: u16) -> Vec<NavNode> {
    nav_nodes_for_query(state, viewport_height, None)
}

fn nav_nodes_for_query(
    state: &TreeState,
    viewport_height: u16,
    query: Option<&NavQuery>,
) -> Vec<NavNode> {
    let mut nodes = Vec::new();
    let mut section = "";
    let mut section_index = 0usize;
    for (index, entry) in NAV_ENTRIES.iter().enumerate() {
        if entry.section != section {
            section = entry.section;
            if !section.is_empty() {
                nodes.push(NavNode {
                    label: section,
                    depth: 0,
                    parent: true,
                    entry: None,
                    target: None,
                    key: nav_group_key(section_index),
                    query_match: true,
                    fade: 0,
                });
                section_index = section_index.saturating_add(1);
            }
        }
        nodes.push(NavNode {
            label: nav_display_label(entry.label),
            depth: u16::from(!entry.section.is_empty()),
            parent: false,
            entry: Some(index),
            target: Some(nav_target_for_entry(entry, index)),
            key: nav_entry_key(index),
            query_match: true,
            fade: 0,
        });
    }

    let query_matches = query.map(|query| nav_query_projection(&nodes, state, query));
    if let Some(matches) = &query_matches {
        for (node, included) in nodes.iter_mut().zip(matches.iter().copied()) {
            node.query_match = included;
        }
    }

    let visible_sources = if let Some(matches) = &query_matches {
        matches
            .iter()
            .enumerate()
            .filter_map(|(source, included)| included.then_some(source))
            .collect::<Vec<_>>()
    } else {
        nav_expanded_sources(&nodes, state)
    };

    let viewport = usize::from(viewport_height);
    let mut scroll = *state.scroll();
    scroll.apply_layout(viewport, visible_sources.len());
    let offset = scroll.offset();
    let end = offset.saturating_add(viewport);
    let after_hidden = visible_sources.len() > end;
    for (visible, source) in visible_sources.iter().copied().enumerate() {
        if viewport > 0 && visible >= offset && visible < end {
            let from_start = visible.saturating_sub(offset);
            let from_end = end.saturating_sub(visible.saturating_add(1));
            if offset > 0 && from_start < 2 {
                nodes[source].fade = if from_start == 0 { 2 } else { 1 };
            } else if after_hidden && from_end < 2 {
                nodes[source].fade = if from_end == 0 { 2 } else { 1 };
            }
        }
    }
    nodes
}

fn nav_expanded_sources(nodes: &[NavNode], state: &TreeState) -> Vec<usize> {
    let mut visible_sources = Vec::with_capacity(nodes.len());
    let mut collapsed_at = None;
    for (source, node) in nodes.iter().enumerate() {
        if collapsed_at.is_some_and(|depth| node.depth > depth) {
            continue;
        }
        collapsed_at = None;
        visible_sources.push(source);
        if node.parent && !state.is_expanded(node.key) {
            collapsed_at = Some(node.depth);
        }
    }
    visible_sources
}

fn nav_query_projection(nodes: &[NavNode], state: &TreeState, query: &NavQuery) -> Vec<bool> {
    let mut included = vec![false; nodes.len()];
    if query.text.is_empty() {
        for source in nav_expanded_sources(nodes, state) {
            if let Some(row) = included.get_mut(source) {
                *row = true;
            }
        }
        return included;
    }

    let needle = query.text.to_lowercase();
    let matching_groups = nodes
        .iter()
        .map(|node| node.parent && node.label.to_lowercase().contains(&needle))
        .collect::<Vec<_>>();
    for (source, node) in nodes.iter().enumerate() {
        if node.parent {
            included[source] = matching_groups[source];
            continue;
        }
        let Some(entry_index) = node.entry else {
            continue;
        };
        let Some(entry) = NAV_ENTRIES.get(entry_index) else {
            continue;
        };
        let parent = (0..source)
            .rev()
            .find(|parent| nodes[*parent].parent && nodes[*parent].depth < node.depth);
        let parent_matches = parent.is_some_and(|parent| matching_groups[parent]);
        let parent_expanded = parent.is_some_and(|parent| state.is_expanded(nodes[parent].key));

        if (parent_matches && parent_expanded)
            || (!parent_matches && nav_entry_matches(entry, entry_index, node.label, &needle))
        {
            included[source] = true;
        }
    }

    for source in 0..nodes.len() {
        if !included[source] {
            continue;
        }
        let mut ancestor_depth = nodes[source].depth;
        for ancestor in (0..source).rev() {
            let candidate = &nodes[ancestor];
            if candidate.parent && candidate.depth < ancestor_depth {
                included[ancestor] = true;
                ancestor_depth = candidate.depth;
                if ancestor_depth == 0 {
                    break;
                }
            }
        }
    }
    included
}

fn nav_entry_matches(entry: &NavEntry, index: usize, display_label: &str, needle: &str) -> bool {
    [display_label, entry.label]
        .into_iter()
        .any(|label| query_contains(label, needle))
        || (matches!(nav_target_for_entry(entry, index), NavTarget::Live(_))
            && [entry.id.title(), entry.id.slug()]
                .into_iter()
                .any(|label| query_contains(label, needle)))
        || NAV_ALIASES
            .iter()
            .filter(|(_, label)| *label == entry.label)
            .any(|(alias, _)| query_contains(alias, needle))
}

fn query_contains(label: &str, needle: &str) -> bool {
    label.to_lowercase().contains(needle)
}

fn nav_query_match(node: &NavNode) -> bool {
    node.query_match
}

fn nav_node_key(node: &NavNode) -> ItemKey {
    node.key
}

fn node_of(node: &NavNode) -> TreeNode {
    if node.parent {
        TreeNode::parent(node.depth)
    } else {
        TreeNode::leaf(node.depth)
    }
}

fn nav_tree(
    query_revision: Option<u64>,
) -> Tree<'static, NavNode, fn(&NavNode) -> ItemKey, impl RowFn<NavNode>> {
    let tree = Tree::new(NAV)
        .key(nav_node_key as fn(&NavNode) -> ItemKey)
        .node(&node_of)
        .render_row(&paint_sidebar_row)
        .empty(EmptyState::Empty {
            title: "No matching pages",
            hint: None,
        });
    if let Some(revision) = query_revision {
        tree.query(revision, &nav_query_match)
    } else {
        tree
    }
}

const TREE_QUERY_BINDINGS: [(ActionKey, Chord); 7] = [
    (
        ActionKey::custom("tree.up-vim"),
        Chord::key(KeyCode::Char('k')),
    ),
    (
        ActionKey::custom("tree.down-vim"),
        Chord::key(KeyCode::Char('j')),
    ),
    (
        ActionKey::custom("tree.home-vim"),
        Chord::key(KeyCode::Char('g')),
    ),
    (
        ActionKey::custom("tree.end-vim"),
        Chord::key(KeyCode::Char('G')),
    ),
    (
        ActionKey::custom("tree.expand-vim"),
        Chord::key(KeyCode::Char('l')),
    ),
    (
        ActionKey::custom("tree.collapse-vim"),
        Chord::key(KeyCode::Char('h')),
    ),
    (
        ActionKey::custom("tree.choose"),
        Chord::key(KeyCode::Char(' ')),
    ),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NavQueryInput {
    Start,
    Insert(char),
    Backspace,
    Escape,
    FocusOut,
    Consume,
}

fn nav_query_input(cx: &Cx<'_>, active: bool) -> Option<NavQueryInput> {
    let mut raw_key = None;
    let mut binding = None;
    let mut focus_out = false;
    for intent in cx.intents(NAV) {
        match intent {
            Intent::Key(key) => raw_key = Some(key),
            Intent::Binding(action) => binding = Some(action),
            Intent::FocusOut { .. } => focus_out = true,
            _ => {}
        }
    }

    if active && focus_out {
        return Some(NavQueryInput::FocusOut);
    }
    if active
        && raw_key.is_some_and(|key| {
            matches!(
                key.code,
                KeyCode::Left | KeyCode::Right | KeyCode::PageUp | KeyCode::PageDown
            )
        })
    {
        return Some(NavQueryInput::Consume);
    }
    if let Some(key) = raw_key
        && key
            .mods
            .difference(termrock::KeyModifiers::SHIFT)
            .is_empty()
    {
        match key.code {
            KeyCode::Esc if active => return Some(NavQueryInput::Escape),
            KeyCode::Backspace if active => return Some(NavQueryInput::Backspace),
            KeyCode::Char('/') if !active => return Some(NavQueryInput::Start),
            KeyCode::Char(character) if active && key.bare_char().is_some() => {
                return Some(NavQueryInput::Insert(character));
            }
            _ => {}
        }
    }

    if active
        && binding.is_some_and(|action| {
            [
                "tree.expand",
                "tree.collapse",
                "tree.page-up",
                "tree.page-down",
            ]
            .into_iter()
            .any(|name| action == ActionKey::custom(name))
        })
    {
        return Some(NavQueryInput::Consume);
    }

    let action = binding?;
    for (candidate, default) in TREE_QUERY_BINDINGS {
        if action != candidate {
            continue;
        }
        let chord = cx.effective_chord(NAV, action, Some(default))?;
        if !chord.is_bare_char() {
            return None;
        }
        let KeyCode::Char(character) = chord.code else {
            return None;
        };
        if active || character == '/' {
            return cx.claim_binding_chord(NAV, chord).map(|_| {
                if active {
                    NavQueryInput::Insert(character)
                } else {
                    NavQueryInput::Start
                }
            });
        }
    }
    None
}

fn nav_target_at(index: usize) -> Option<NavTarget> {
    let entry = NAV_ENTRIES.get(index)?;
    Some(nav_target_for_entry(entry, index))
}

fn nav_group_for_entry(nodes: &[NavNode], entry: usize) -> Option<ItemKey> {
    let source = nodes.iter().position(|node| node.entry == Some(entry))?;
    let depth = nodes[source].depth;
    nodes[..source]
        .iter()
        .rev()
        .find(|node| node.parent && node.depth < depth)
        .map(|node| node.key)
}

fn reveal_nav(state: &mut TreeState, nodes: &[NavNode], entry: usize) {
    let Some(source) = nodes.iter().position(|node| node.entry == Some(entry)) else {
        return;
    };
    if nodes[source].depth > 0
        && let Some(parent) = nodes[..source].iter().rev().find(|node| node.parent)
    {
        state.expand(parent.key);
    }
    let key = nodes[source].key;
    let visible = visible_index(nodes, source, state);
    state.choose(Some(key));
    state.set_cursor(visible, key);
}

fn query_entry_position(nodes: &[NavNode], entry: usize) -> Option<(usize, ItemKey)> {
    let source = nodes
        .iter()
        .position(|node| node.entry == Some(entry) && node.query_match)?;
    let visible = nodes[..source]
        .iter()
        .filter(|node| node.query_match)
        .count();
    Some((visible, nodes[source].key))
}

fn reveal_query_entry(state: &mut TreeState, nodes: &[NavNode], entry: usize) -> bool {
    let Some((visible, key)) = query_entry_position(nodes, entry) else {
        return false;
    };
    state.choose(Some(key));
    state.set_cursor(visible, key);
    true
}

fn cursor_to_first_query_entry(state: &mut TreeState, nodes: &[NavNode]) {
    for (source, node) in nodes.iter().enumerate() {
        if node.entry.is_some() && node.query_match {
            let visible = nodes[..source]
                .iter()
                .filter(|candidate| candidate.query_match)
                .count();
            state.set_cursor(visible, node.key);
            return;
        }
    }
}

fn reconcile_query_cursor(state: &mut TreeState, nodes: &[NavNode], previous: Option<ItemKey>) {
    let visible_position = |key: ItemKey| {
        let source = nodes
            .iter()
            .position(|node| node.key == key && node.query_match)?;
        let visible = nodes[..source]
            .iter()
            .filter(|node| node.query_match)
            .count();
        Some((visible, key))
    };
    let next = previous.and_then(visible_position).or_else(|| {
        nodes
            .iter()
            .enumerate()
            .find(|(_, node)| node.query_match)
            .map(|(source, node)| {
                let visible = nodes[..source]
                    .iter()
                    .filter(|candidate| candidate.query_match)
                    .count();
                (visible, node.key)
            })
    });
    if let Some((visible, key)) = next {
        state.set_cursor(visible, key);
    }
}

fn visible_index(nodes: &[NavNode], source: usize, state: &TreeState) -> usize {
    let mut collapsed_at = None;
    let mut seen = 0usize;
    for (index, node) in nodes.iter().enumerate() {
        if collapsed_at.is_some_and(|depth| node.depth > depth) {
            continue;
        }
        collapsed_at = None;
        if index == source {
            return seen;
        }
        seen = seen.saturating_add(1);
        if node.parent && !state.is_expanded(node.key) {
            collapsed_at = Some(node.depth);
        }
    }
    0
}

fn target_for_key(key: ItemKey, nodes: &[NavNode]) -> Option<(usize, NavTarget)> {
    let node = nodes.iter().find(|node| node.key == key)?;
    Some((node.entry?, node.target?))
}

fn shell_brand() -> Brand<'static> {
    Brand::new(BRAND, "Junie").tagline("Design system")
}

fn shell_status() -> StatusBar<'static> {
    StatusBar::new(STATUS)
        .left(&STATUS_LEFT)
        .right(&STATUS_RIGHT)
        .status(Status::Ready)
}

fn missing_page_panel() -> Panel<'static> {
    Panel::new(BRAND)
        .kind(PanelKind::Framed)
        .title("Missing page")
}

fn inspector_panel() -> Panel<'static> {
    Panel::new(INSPECTOR).kind(PanelKind::Card).title("State")
}

fn too_small_notice() -> TooSmall<'static> {
    TooSmall::new(TOO_SMALL, "showcase").minimum(72, 20)
}

fn page(kind: PageId) -> Box<dyn Page> {
    match kind {
        PageId::Overview => Box::new(OverviewPage::new()),
        PageId::Buttons => Box::new(ButtonsPage::new()),
        PageId::Inputs => Box::new(InputsPage::new()),
        PageId::TextAreas => Box::new(TextAreasPage::new()),
        PageId::Forms => Box::new(FormsPage::new()),
        PageId::Lists => Box::new(ListsPage::new()),
        PageId::Trees => Box::new(TreesPage::new()),
        PageId::Tables => Box::new(TablesPage::new()),
        PageId::Editable => Box::new(EditablePage::new()),
        PageId::Panels => Box::new(PanelsPage::new()),
        PageId::Sidebars => Box::new(SidebarsPage::new()),
        PageId::Dialogs => Box::new(DialogsPage::new()),
        PageId::Progress => Box::new(ProgressPage::new()),
        PageId::Scrolling => Box::new(ScrollingPage::new()),
        PageId::Terminal => Box::new(TerminalPage::new()),
        PageId::Editor => Box::new(EditorPage::new()),
        PageId::Grid => Box::new(GridPage::new()),
        PageId::Chips => Box::new(ChipsPage::new()),
        PageId::Pickers => Box::new(PickersPage::new()),
        PageId::DialogKinds => Box::new(DialogKindsPage::new()),
        PageId::Chrome => Box::new(ChromePage::new()),
        PageId::Settings => Box::new(SettingsPage::new()),
        PageId::TaskRunner => Box::new(TaskRunnerPage::new()),
        PageId::CellEditors => Box::new(CellEditorsPage::new()),
        PageId::Toolbars => Box::new(ToolbarsPage::new()),
    }
}

fn keymap() -> KeyMap {
    KeyMap::new()
        .bind(KeyPhase::Bubble, Chord::key(KeyCode::Char('q')), QUIT)
        .bind(
            KeyPhase::Capture,
            Chord::with(KeyCode::Char('c'), termrock::KeyModifiers::CONTROL),
            QUIT_CTRL,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('c'), termrock::KeyModifiers::CONTROL),
            QUIT_CTRL,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('?')),
            HELP_COMMAND,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('i')),
            INSPECTOR_COMMAND,
        )
        .bind(KeyPhase::Bubble, Chord::key(KeyCode::Char(']')), NEXT_PAGE)
        .bind(KeyPhase::Bubble, Chord::key(KeyCode::Char('[')), PREV_PAGE)
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('r')),
            RUN_COMMAND,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('s'), termrock::KeyModifiers::CONTROL),
            FORM_SUBMIT,
        )
}

pub struct App {
    page: PageId,
    nav_target: NavTarget,
    nav_state: TreeState,
    nav_expand: NavExpand,
    nav_auto_group: Option<ItemKey>,
    nav_query: Option<NavQuery>,
    nav_presentation: std::cell::Cell<Option<NavTreePresentation>>,
    pages: Vec<Box<dyn Page>>,
    checks_radios: ChecksRadiosPage,
    empty_error: EmptyErrorPage,
    notices: NoticesPage,
    diff: DiffPage,
    master_detail: MasterDetailPage,
    pane_tree: PaneTreePage,
    notice_reveals: std::cell::RefCell<[NoticeReveal; 6]>,
    notice_reveal_snapshot: Option<NoticeRevealSnapshot>,
    notice_reveal_revision: u64,
    help_state: HelpOverlayState,
    help_open: bool,
    overlay_open: bool,
    inspector_state: DialogState,
    inspector_opening: Option<LayerOpening>,
    inspector_programmatic_close: Option<LayerOpening>,
    selectable: showcase_demos::pages::selectable::Model,
    keymap: KeyMap,
    inspector: bool,
    shell_layout_cache: std::cell::Cell<Option<CachedShellLayout>>,
    quit: bool,
    status: Option<(PageStatus, Moment)>,
    motion_paused: bool,

    paused_frame: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NoticeRevealSnapshot {
    content_revision: u64,
    play_scroll_offset: usize,
    viewport: Rect,
    main: Rect,
}

impl core::fmt::Debug for App {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("App")
            .field("page", &self.page)
            .field("nav_target", &self.nav_target)
            .field("nav_state", &self.nav_state)
            .field("nav_expand", &self.nav_expand)
            .field("nav_auto_group", &self.nav_auto_group)
            .field("nav_query", &self.nav_query)
            .field("nav_presentation", &self.nav_presentation.get())
            .field("pages", &self.pages.len())
            .field("checks_radios", &self.checks_radios)
            .field("empty_error", &self.empty_error)
            .field("notices", &self.notices)
            .field("master_detail", &self.master_detail)
            .field("pane_tree", &self.pane_tree)
            .field("notice_reveal_snapshot", &self.notice_reveal_snapshot)
            .field("notice_reveal_revision", &self.notice_reveal_revision)
            .field("help_state", &self.help_state)
            .field("help_open", &self.help_open)
            .field("overlay_open", &self.overlay_open)
            .field("inspector_state", &self.inspector_state)
            .field("inspector_opening", &self.inspector_opening)
            .field(
                "inspector_programmatic_close",
                &self.inspector_programmatic_close,
            )
            .field("selectable", &self.selectable)
            .field("keymap", &self.keymap)
            .field("inspector", &self.inspector)
            .field("motion_paused", &self.motion_paused)
            .field("paused_frame", &self.paused_frame)
            .field("quit", &self.quit)
            .field("status", &self.status.as_ref().map(|(_, since)| since))
            .finish()
    }
}

impl App {
    pub fn new() -> Self {
        Self::with_page(PageId::Overview)
    }

    pub fn with_page(initial: PageId) -> Self {
        Self::with_page_motion(initial, false, 0)
    }

    pub fn for_page_name(name: &str) -> Self {
        Self::boot(nav_index_from_name(name).unwrap_or(0), false, 0)
    }

    pub fn with_page_motion(initial: PageId, paused: bool, frame: usize) -> Self {
        Self::boot(index_for_page(initial), paused, frame)
    }

    fn boot(nav_index: usize, paused: bool, frame: usize) -> Self {
        let nav_index = nav_index.min(NAV_ENTRIES.len().saturating_sub(1));
        let nav_target = nav_target_at(nav_index).unwrap_or(NavTarget::Live(PageId::Overview));
        let initial = nav_target.page().unwrap_or(PageId::Overview);
        let mut nav_state = TreeState::new();
        let nodes = nav_nodes(&nav_state, 0);
        reveal_nav(&mut nav_state, &nodes, nav_index);
        let mut pages: Vec<Box<dyn Page>> = PageId::ALL.into_iter().map(page).collect();
        if paused {
            for page in &mut pages {
                page.seek_paused(frame);
            }
        }
        let mut checks_radios = ChecksRadiosPage::new();
        if paused {
            checks_radios.seek_paused(frame);
        }
        let mut diff = DiffPage::new();
        if paused {
            diff.seek_paused(frame);
        }
        Self {
            page: initial,
            nav_target,
            nav_state,
            nav_expand: NavExpand::All,
            nav_auto_group: None,
            nav_query: None,
            nav_presentation: std::cell::Cell::new(None),
            pages,
            checks_radios,
            empty_error: EmptyErrorPage::new(),
            notices: NoticesPage::new(),
            diff,
            master_detail: MasterDetailPage::new(),
            pane_tree: PaneTreePage::new(),
            notice_reveals: std::cell::RefCell::new(std::array::from_fn(|index| {
                NoticeReveal::new(NOTICE_REVEAL_OWNERS[index])
            })),
            notice_reveal_snapshot: None,
            notice_reveal_revision: 0,
            help_state: HelpOverlayState::default(),
            help_open: false,
            overlay_open: false,
            inspector_state: DialogState::default(),
            inspector_opening: None,
            inspector_programmatic_close: None,
            selectable: showcase_demos::pages::selectable::Model::default(),
            keymap: keymap(),
            inspector: false,
            shell_layout_cache: std::cell::Cell::new(None),
            quit: false,
            status: None,
            motion_paused: paused,
            paused_frame: frame,
        }
    }

    pub fn set_nav_expand(&mut self, policy: NavExpand) {
        self.nav_expand = policy;
        self.nav_auto_group = None;
        let route = nav_index_for_target(self.nav_target).unwrap_or(0);
        let previous_cursor = self.nav_state.cursor();
        self.sync_nav_expansion(route, true);
        let nodes = nav_nodes(&self.nav_state, 0);
        reveal_nav(&mut self.nav_state, &nodes, route);
        if self.nav_query.is_some() {
            self.bump_nav_query_revision();
            let query_nodes = nav_nodes_for_query(&self.nav_state, 0, self.nav_query.as_ref());
            reconcile_query_cursor(&mut self.nav_state, &query_nodes, previous_cursor);
        }
    }

    pub const fn page(&self) -> PageId {
        self.page
    }

    pub const fn quit(&self) -> bool {
        self.quit
    }

    fn goto_entry(&mut self, index: usize) {
        let query_preview = self.nav_query.is_some();
        let Some(target) = nav_target_at(index) else {
            return;
        };
        self.nav_target = target;
        if let Some(page) = target.page() {
            self.page = page;
        }
        if let Some(query) = self.nav_query.as_ref() {
            let nodes = nav_nodes_for_query(&self.nav_state, 0, Some(query));
            if reveal_query_entry(&mut self.nav_state, &nodes, index) {
                return;
            }
            self.nav_query = None;
        }
        if !query_preview {
            self.sync_nav_expansion(index, false);
        }
        let nodes = nav_nodes(&self.nav_state, 0);
        reveal_nav(&mut self.nav_state, &nodes, index);
    }

    fn goto_entry_with_cx(&mut self, cx: &mut Cx<'_>, index: usize) -> Response<()> {
        let next = nav_target_at(index);
        let response = if self.nav_target == NavTarget::Placeholder(PANE_TREE_ENTRY)
            && next.is_some_and(|target| target != self.nav_target)
        {
            self.pane_tree.prepare_for_route_leave(cx)
        } else {
            Response::ignored()
        };
        self.goto_entry(index);
        response
    }

    fn sync_nav_expansion(&mut self, index: usize, reset: bool) {
        let nodes = nav_nodes(&self.nav_state, 0);
        let destination = nav_group_for_entry(&nodes, index);
        match self.nav_expand {
            NavExpand::All => {
                self.nav_auto_group = None;
                if reset {
                    self.nav_state.expand_all();
                }
            }
            NavExpand::Accordion => {
                self.nav_state.collapse_all();
                self.nav_auto_group = None;
            }
            NavExpand::CurrentGroup if reset => {
                self.nav_state.collapse_all();
                if let Some(group) = destination {
                    self.nav_state.expand(group);
                    self.nav_auto_group = Some(group);
                } else {
                    self.nav_auto_group = None;
                }
            }
            NavExpand::CurrentGroup => {
                let previous_auto = self.nav_auto_group.take();
                if let Some(previous) = previous_auto
                    && Some(previous) != destination
                {
                    self.nav_state.collapse(previous);
                }
                if let Some(group) = destination {
                    let opened_now = !self.nav_state.is_expanded(group);
                    self.nav_state.expand(group);
                    if opened_now || previous_auto == Some(group) {
                        self.nav_auto_group = Some(group);
                    }
                }
            }
        }
    }

    fn apply_nav_query_input(&mut self, input: NavQueryInput) {
        match input {
            NavQueryInput::Start => {
                if self.nav_query.is_none() {
                    self.nav_query = Some(NavQuery {
                        text: String::new(),
                        revision: 1,
                    });
                }
            }
            NavQueryInput::Insert(character) => {
                if self.nav_query.is_some() {
                    if let Some(query) = &mut self.nav_query {
                        query.text.push(character);
                    }
                    self.bump_nav_query_revision();
                    self.refresh_nav_query_cursor();
                }
            }
            NavQueryInput::Backspace => {
                let changed = self
                    .nav_query
                    .as_mut()
                    .is_some_and(|query| query.text.pop().is_some());
                if changed {
                    self.bump_nav_query_revision();
                    self.refresh_nav_query_cursor();
                }
            }
            NavQueryInput::Escape | NavQueryInput::FocusOut => self.end_nav_query(),
            NavQueryInput::Consume => {}
        }
    }

    fn bump_nav_query_revision(&mut self) {
        let saturated = if let Some(query) = &mut self.nav_query {
            match query.revision.checked_add(1) {
                Some(revision) => {
                    query.revision = revision;
                    false
                }
                None => {
                    query.revision = 1;
                    true
                }
            }
        } else {
            false
        };
        if saturated {
            self.nav_state.invalidate();
        }
    }

    fn refresh_nav_query_cursor(&mut self) {
        let Some(query) = self.nav_query.as_ref() else {
            return;
        };
        let nodes = nav_nodes_for_query(&self.nav_state, 0, Some(query));
        if query.text.is_empty() {
            if let Some(entry) = nav_index_for_target(self.nav_target) {
                reveal_nav(&mut self.nav_state, &nodes, entry);
            }
        } else {
            cursor_to_first_query_entry(&mut self.nav_state, &nodes);
        }
    }

    fn end_nav_query(&mut self) {
        if self.nav_query.take().is_none() {
            return;
        }
        let nodes = nav_nodes(&self.nav_state, 0);
        if let Some(entry) = nav_index_for_target(self.nav_target) {
            reveal_nav(&mut self.nav_state, &nodes, entry);
        }
    }

    fn end_nav_query_for_activation(&mut self) {
        self.nav_query = None;
    }

    fn resolved_shell_layout(&self, area: Rect) -> ShellLayout {
        let inspector_dock = self.inspector && self.inspector_opening.is_none();
        if let Some(cached) = self.shell_layout_cache.get()
            && cached.area == area
            && cached.inspector == inspector_dock
        {
            return cached.layout;
        }
        let layout = shell_layout(area, inspector_dock);
        self.shell_layout_cache.set(Some(CachedShellLayout {
            area,
            inspector: inspector_dock,
            layout,
        }));
        layout
    }

    fn update_notice_reveals(&mut self, cx: &mut Cx<'_>, response: &mut Response<()>) {
        let full = cx.viewport();
        let snapshot = if self.nav_target == NavTarget::Placeholder(NOTICES_ENTRY)
            && self.nav_query.is_none()
            && !self.overlay_open
            && cx.top_layer() == termrock::LayerId::PAGE
            && full.width >= 72
            && full.height >= 20
        {
            self.notices
                .reveal_state()
                .map(|page| NoticeRevealSnapshot {
                    content_revision: page.content_revision,
                    play_scroll_offset: page.play_scroll_offset,
                    viewport: full,
                    main: self.resolved_shell_layout(full).main,
                })
        } else {
            None
        };

        if let Some(snapshot) = snapshot {
            if self.notice_reveal_snapshot != Some(snapshot) {
                self.notice_reveal_revision = self.notice_reveal_revision.wrapping_add(1).max(1);
                self.notice_reveal_snapshot = Some(snapshot);
                *response |= Response::changed();
            }
            for reveal in self.notice_reveals.get_mut() {
                *response |= reveal.update(cx, self.notice_reveal_revision);
            }
        } else {
            if self.notice_reveal_snapshot.take().is_some() {
                *response |= Response::changed();
            }
            for reveal in self.notice_reveals.get_mut() {
                reveal.reset();
            }
        }
    }

    fn show_live_page(&self) -> bool {
        matches!(
            self.nav_target,
            NavTarget::Live(_)
                | NavTarget::Placeholder(CHECKS_RADIOS_ENTRY)
                | NavTarget::Placeholder(EMPTY_ERROR_ENTRY)
                | NavTarget::Placeholder(NOTICES_ENTRY)
                | NavTarget::Placeholder(DIFF_ENTRY)
                | NavTarget::Placeholder(MASTER_DETAIL_ENTRY)
                | NavTarget::Placeholder(PANE_TREE_ENTRY)
        )
    }

    fn nav_label(&self) -> &'static str {
        nav_index_for_target(self.nav_target)
            .and_then(|index| NAV_ENTRIES.get(index))
            .map(|entry| entry.label)
            .unwrap_or("Overview")
    }

    fn active(&self) -> Option<&dyn Page> {
        if self.nav_target == NavTarget::Placeholder(CHECKS_RADIOS_ENTRY) {
            Some(&self.checks_radios)
        } else if self.nav_target == NavTarget::Placeholder(EMPTY_ERROR_ENTRY) {
            Some(&self.empty_error)
        } else if self.nav_target == NavTarget::Placeholder(NOTICES_ENTRY) {
            Some(&self.notices)
        } else if self.nav_target == NavTarget::Placeholder(DIFF_ENTRY) {
            Some(&self.diff)
        } else if self.nav_target == NavTarget::Placeholder(MASTER_DETAIL_ENTRY) {
            Some(&self.master_detail)
        } else if self.nav_target == NavTarget::Placeholder(PANE_TREE_ENTRY) {
            Some(&self.pane_tree)
        } else {
            self.pages.get(self.page.index()).map(Box::as_ref)
        }
    }

    fn active_mut(&mut self) -> Option<&mut (dyn Page + 'static)> {
        if self.nav_target == NavTarget::Placeholder(CHECKS_RADIOS_ENTRY) {
            Some(&mut self.checks_radios)
        } else if self.nav_target == NavTarget::Placeholder(EMPTY_ERROR_ENTRY) {
            Some(&mut self.empty_error)
        } else if self.nav_target == NavTarget::Placeholder(NOTICES_ENTRY) {
            Some(&mut self.notices)
        } else if self.nav_target == NavTarget::Placeholder(DIFF_ENTRY) {
            Some(&mut self.diff)
        } else if self.nav_target == NavTarget::Placeholder(MASTER_DETAIL_ENTRY) {
            Some(&mut self.master_detail)
        } else if self.nav_target == NavTarget::Placeholder(PANE_TREE_ENTRY) {
            Some(&mut self.pane_tree)
        } else {
            self.pages.get_mut(self.page.index()).map(Box::as_mut)
        }
    }

    fn help_overlay() -> HelpOverlay<'static> {
        HelpOverlay::new(HELP, "Showcase", &HELP_SECTIONS).title("Keyboard & mouse")
    }

    fn open_help(&mut self, cx: &mut Cx<'_>) {
        self.help_state = HelpOverlayState::default();
        cx.open_layer(HELP, Self::help_overlay().layer(cx));
    }

    fn inspector_dialog(viewport: Rect) -> Dialog<'static> {
        Dialog::info(INSPECTOR, "Inspector")
            .description("State")
            .body_rows(viewport.height.saturating_sub(11))
            .width(34)
    }

    fn inspector_anchor(viewport: Rect) -> Anchor {
        Anchor::Rect {
            rect: Rect::new(
                viewport.right().saturating_sub(2),
                viewport.y.saturating_add(1),
                1,
                1,
            ),
            side: Side::Left,
            align: CrossAlign::Center,
        }
    }

    fn inspector_layer_spec(cx: &Cx<'_>) -> termrock::LayerSpec {
        let viewport = cx.viewport();
        let dialog = Self::inspector_dialog(viewport);
        dialog
            .layer(cx)
            .anchor(Self::inspector_anchor(viewport))
            .dismiss(Dismiss::ESC_AND_OUTSIDE)
            .initial_focus(dialog.action_id(0))
            .viewport_margin(2, 1)
    }

    fn inspector_modal_supported(viewport: Rect) -> bool {
        (72..100).contains(&viewport.width) && viewport.height >= 20
    }

    fn inspector_modal_active(&self, cx: &Cx<'_>) -> bool {
        self.inspector
            && self.inspector_opening.is_some_and(|opening| {
                cx.layer_opening(INSPECTOR) == Some(opening) && cx.top_layer() == opening.layer_id()
            })
    }

    fn toggle_inspector(&mut self, response: &mut Response<()>) {
        self.inspector = !self.inspector;
        *response |= Response::changed();
    }

    fn update_inspector(&mut self, cx: &mut Cx<'_>, response: &mut Response<()>) {
        let viewport = cx.viewport();
        let modal_expected = self.inspector && Self::inspector_modal_supported(viewport);

        if modal_expected
            && self
                .inspector_opening
                .is_some_and(|opening| cx.layer_opening(INSPECTOR) == Some(opening))
        {
            cx.reanchor_layer(INSPECTOR, Self::inspector_anchor(viewport));
        }

        if let Some(opening) = self.inspector_opening
            && cx.layer_opening(INSPECTOR) == Some(opening)
            && !modal_expected
            && cx.top_layer() == opening.layer_id()
        {
            self.inspector_programmatic_close = Some(opening);
            cx.close_layer(INSPECTOR, None);
            *response |= Response::changed();
        }

        let dialog = Self::inspector_dialog(viewport);
        let update = dialog.update(cx, &mut self.inspector_state);
        let action = update.action_ref().copied();
        *response |= update.erase();
        if matches!(
            action,
            Some(DialogAction::Action(ActionKey::CLOSE) | DialogAction::Dismissed(_))
        ) {
            let opening = self.inspector_opening;
            let app_closed =
                opening.is_some_and(|opening| self.inspector_programmatic_close == Some(opening));
            if !app_closed {
                self.inspector = false;
            }
            if let Some(opening) = opening
                && cx.layer_opening(INSPECTOR) == Some(opening)
                && cx.top_layer() == opening.layer_id()
            {
                cx.close_layer(INSPECTOR, None);
            }
            self.inspector_opening = None;
            self.inspector_programmatic_close = None;
            *response |= Response::changed();
        }

        if self.inspector_opening.is_none()
            && cx.layer_opening(INSPECTOR).is_none()
            && self.inspector
            && Self::inspector_modal_supported(viewport)
        {
            let spec = Self::inspector_layer_spec(cx);
            if let Some(opening) = cx.open_layer_with_token(INSPECTOR, spec) {
                self.inspector_opening = Some(opening);
                *response |= Response::changed();
            }
        }
    }

    fn update_help(&mut self, cx: &mut Cx<'_>, response: &mut Response<()>) {
        *response |= Self::help_overlay()
            .update(cx, &mut self.help_state)
            .erase();
    }

    fn update_header(&mut self, cx: &mut Cx<'_>, response: &mut Response<()>) {
        let help_clicked = cx.intents(HEADER_HELP).any(|intent| {
            matches!(
                intent,
                Intent::Pointer {
                    phase: Phase::Click,
                    ..
                }
            )
        });
        if help_clicked {
            if !cx.is_open(HELP) {
                self.open_help(cx);
            }
            *response |= Response::changed();
        }
        let inspector_clicked = cx.intents(HEADER_INSPECT).any(|intent| {
            matches!(
                intent,
                Intent::Pointer {
                    phase: Phase::Click,
                    ..
                }
            )
        });
        if inspector_clicked {
            self.toggle_inspector(response);
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct ShellLayout {
    header: Rect,
    sidebar: Rect,
    main: Rect,
    inspector: Option<Rect>,
    footer: Rect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SidebarPresentation {
    query: Option<Rect>,
    tree: Rect,
}

impl SidebarPresentation {
    fn new(sidebar: Rect, panel_inner: Rect, gutter: u16, query_active: bool) -> Self {
        if sidebar.is_empty() || panel_inner.is_empty() {
            return Self {
                query: None,
                tree: Rect::default(),
            };
        }
        let content = Rect {
            x: sidebar.x.saturating_add(gutter),
            y: panel_inner.y,
            width: sidebar.width.saturating_sub(gutter.saturating_mul(2)),
            height: panel_inner.height,
        };
        if !query_active || content.is_empty() {
            return Self {
                query: None,
                tree: content,
            };
        }
        let query = Rect::new(content.x, content.y, content.width, 1);
        let tree = Rect {
            y: content.y.saturating_add(1),
            height: content.height.saturating_sub(1),
            ..content
        };
        Self {
            query: Some(query),
            tree,
        }
    }
}

fn sidebar_panel_inner(sidebar: Rect, card_inset: u16) -> Rect {
    Rect {
        x: sidebar.x.saturating_add(card_inset),
        y: sidebar.y.saturating_add(2),
        width: sidebar.width.saturating_sub(card_inset.saturating_mul(2)),
        height: sidebar.height.saturating_sub(3),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NavTreePresentation {
    area: Rect,
    query_revision: Option<u64>,
}

impl NavTreePresentation {
    fn new(area: Rect, query: Option<&NavQuery>) -> Self {
        Self {
            area,
            query_revision: query.map(|query| query.revision),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct CachedShellLayout {
    area: Rect,
    inspector: bool,
    layout: ShellLayout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pane {
    Sidebar,
    Main,
    Spacer,
    Inspector,
}

const SIDEBAR_WIDE: u16 = 25;
const SIDEBAR_NARROW: u16 = 20;
const SIDEBAR_WIDE_FROM: u16 = 110;

const INSPECTOR_WIDTH: u16 = 30;
const INSPECTOR_MIN_WIDTH: u16 = 100;

const SHELL_GAP: u16 = 1;

fn fixed_pane(width: u16) -> termrock::layout::ChildPolicy {
    termrock::layout::ChildPolicy::new(termrock::layout::Size::exact(width, 1))
        .min(width, 0)
        .max(Some(width), None)
}

fn flexible_pane() -> termrock::layout::ChildPolicy {
    termrock::layout::ChildPolicy::new(termrock::layout::Size::default()).flex(1)
}

fn shell_layout(area: Rect, inspector: bool) -> ShellLayout {
    let regions = termrock::AppShell::new()
        .header(1)
        .gap(1)
        .footer(1)
        .regions(area);
    let body = regions.body;
    let sidebar_width = if body.width >= SIDEBAR_WIDE_FROM {
        SIDEBAR_WIDE
    } else {
        SIDEBAR_NARROW
    };
    let inspector_width = if inspector && body.width >= INSPECTOR_MIN_WIDTH {
        INSPECTOR_WIDTH
    } else {
        0
    };
    let panes = [Pane::Sidebar, Pane::Main, Pane::Spacer, Pane::Inspector];
    let children = [
        termrock::layout::Child::new(&Pane::Sidebar, fixed_pane(sidebar_width)),
        termrock::layout::Child::new(&Pane::Main, flexible_pane()),
        termrock::layout::Child::new(
            &Pane::Spacer,
            termrock::layout::ChildPolicy::new(termrock::layout::Size::default())
                .max(Some(0), None),
        ),
        termrock::layout::Child::new(&Pane::Inspector, fixed_pane(inspector_width)),
    ];
    let count = if inspector_width > 0 { 4 } else { 2 };
    let plan = termrock::layout::row(
        body,
        children.get(..count).unwrap_or(&[]),
        termrock::layout::Composition::new().gap(SHELL_GAP),
        |_, _| termrock::layout::Size::default(),
    );
    let mut layout = ShellLayout {
        header: regions.header,
        sidebar: Rect::default(),
        main: Rect::default(),
        inspector: None,
        footer: regions.footer,
    };
    for child in &plan.children {
        match panes.get(child.index).copied() {
            Some(Pane::Sidebar) => layout.sidebar = child.logical,
            Some(Pane::Main) => layout.main = child.logical,
            Some(Pane::Inspector) => layout.inspector = Some(child.logical),
            Some(Pane::Spacer) | None => {}
        }
    }
    layout
}

fn shell_part_style(
    ui: &mut Ui<'_>,
    family: termrock::Family,
    part: Part,
    flags: StateFlags,
) -> PaintStyle {
    let background = ui.surface_style();
    shell_compat_style(
        ui.style(family, Variant::DEFAULT, part, flags)
            .style
            .with_bg_from(background),
    )
}

fn shell_compat_style(style: PaintStyle) -> PaintStyle {
    let bold = style.add_modifier.contains(termrock::Modifier::BOLD);
    let style = style.remove_modifier(termrock::Modifier::all());
    if bold {
        style.add_modifier(termrock::Modifier::BOLD)
    } else {
        style
    }
}

fn shell_text_style(ui: &Ui<'_>, step: termrock::FgStep) -> PaintStyle {
    shell_compat_style(
        ui.surface_style()
            .patch(ui.paint_patch(&termrock::StylePatch::new().set_fg(termrock::Role::Fg(step)))),
    )
}

fn paint_header(
    ui: &mut Ui<'_>,
    area: Rect,
    screen_width: u16,
    screen_height: u16,
    section: &str,
    label: &str,
    inspector: bool,
) {
    if area.is_empty() {
        return;
    }
    let canvas = shell_compat_style(ui.surface_style());
    ui.fill(area, canvas);

    let styles = header_styles(ui);
    let (title, secondary, muted, faint, marker) = styles;
    let (_, inspector_x) = header_action_origins(area, inspector);
    let left = paint_header_breadcrumb(
        ui,
        area,
        section,
        label,
        (title, secondary, faint, marker),
        inspector_x,
    );
    paint_header_actions(
        ui,
        area,
        (screen_width, screen_height),
        left,
        (muted, faint),
        inspector,
    );
}

fn header_styles(ui: &mut Ui<'_>) -> (PaintStyle, PaintStyle, PaintStyle, PaintStyle, PaintStyle) {
    let title = shell_part_style(
        ui,
        termrock::Family::PANEL,
        Part::TITLE,
        StateFlags::empty(),
    );
    let secondary = shell_part_style(
        ui,
        termrock::Family::PANEL,
        Part::DETAIL,
        StateFlags::empty(),
    );
    let muted = secondary;
    let faint = shell_text_style(ui, termrock::FgStep::Faint);
    let marker = shell_part_style(
        ui,
        termrock::Family::LIST,
        Part::MARKER,
        StateFlags::SELECTED,
    );
    (title, secondary, muted, faint, marker)
}

fn paint_header_breadcrumb(
    ui: &mut Ui<'_>,
    area: Rect,
    section: &str,
    label: &str,
    styles: (PaintStyle, PaintStyle, PaintStyle, PaintStyle),
    inspector_x: u16,
) -> u16 {
    let (title, secondary, faint, marker) = styles;
    let mut x = area.x.saturating_add(1);
    ui.paint_str(Rect::new(x, area.y, 1, 1), "▪", marker);
    x = x.saturating_add(2);
    ui.paint_str(
        Rect::new(x, area.y, area.right().saturating_sub(x), 1),
        "Junie",
        title,
    );
    x = x.saturating_add(6);
    ui.paint_str(
        Rect::new(x, area.y, area.right().saturating_sub(x), 1),
        "Design system",
        secondary,
    );
    x = x.saturating_add(15);
    let budget = inspector_x.saturating_sub(x).saturating_sub(2);
    let label = nav_display_label(label);
    let current = title.add_modifier(termrock::Modifier::BOLD);
    if section.is_empty() {
        let shown = clip_label(label, budget);
        ui.paint_str(Rect::new(x, area.y, budget, 1), &shown, current);
        return x.saturating_add(width(&shown));
    }

    let separator_width = width("/ ").saturating_add(width(" / "));
    if budget <= separator_width {
        let crumb = format!("/ {section} / {label}");
        let shown = clip_label(&crumb, budget);
        ui.paint_str(Rect::new(x, area.y, budget, 1), &shown, secondary);
        return x.saturating_add(width(&shown));
    }
    let label_budget = width(label).min(budget.saturating_sub(separator_width));
    let section_budget = budget
        .saturating_sub(separator_width)
        .saturating_sub(label_budget);
    let shown_section = clip_label(section, section_budget);
    let shown_label_budget = budget
        .saturating_sub(separator_width)
        .saturating_sub(width(&shown_section));
    let shown_label = clip_label(label, shown_label_budget);

    ui.paint_str(Rect::new(x, area.y, width("/ "), 1), "/ ", faint);
    x = x.saturating_add(width("/ "));
    ui.paint_str(
        Rect::new(x, area.y, width(&shown_section), 1),
        &shown_section,
        secondary,
    );
    x = x.saturating_add(width(&shown_section));
    ui.paint_str(Rect::new(x, area.y, width(" / "), 1), " / ", faint);
    x = x.saturating_add(width(" / "));
    ui.paint_str(
        Rect::new(x, area.y, width(&shown_label), 1),
        &shown_label,
        current,
    );
    x.saturating_add(width(&shown_label))
}

fn header_action_origins(area: Rect, inspector: bool) -> (u16, u16) {
    let help_width = width(" ? Help ");
    let inspector_width = width(if inspector {
        " i Inspector · on "
    } else {
        " i Inspector "
    });
    let right = area.right().saturating_sub(1);
    let help_x = right.saturating_sub(help_width);
    let inspector_x = help_x.saturating_sub(1).saturating_sub(inspector_width);
    (help_x, inspector_x)
}

fn paint_header_actions(
    ui: &mut Ui<'_>,
    area: Rect,
    screen: (u16, u16),
    left: u16,
    styles: (PaintStyle, PaintStyle),
    inspector: bool,
) {
    let (screen_width, screen_height) = screen;
    let (muted, faint) = styles;
    let capability = ui.theme().capability.color.label();
    let width_text = RenderNumber::new(usize::from(screen_width));
    let height_text = RenderNumber::new(usize::from(screen_height));
    let dimensions_width = width(width_text.as_str())
        .saturating_add(1)
        .saturating_add(width(height_text.as_str()));
    let capability_width = width(capability)
        .saturating_add(3)
        .saturating_add(dimensions_width);
    let help_text = " ? Help ";
    let inspector_text = if inspector {
        " i Inspector · on "
    } else {
        " i Inspector "
    };
    let help_width = width(help_text);
    let inspector_width = width(inspector_text);
    let (help_x, _) = header_action_origins(area, inspector);
    let help_style = header_action_style(ui, HEADER_HELP, muted);
    ui.paint_str(
        Rect::new(help_x, area.y, help_width, 1),
        help_text,
        help_style,
    );
    ui.register_part(
        HEADER_HELP,
        PartRef::of(Part::LABEL),
        Rect::new(help_x, area.y, help_width, 1),
    );
    let mut right = help_x.saturating_sub(1);

    let inspector_x = right.saturating_sub(inspector_width);
    let inspector_style = header_action_style(ui, HEADER_INSPECT, muted);
    ui.paint_str(
        Rect::new(inspector_x, area.y, inspector_width, 1),
        inspector_text,
        inspector_style,
    );
    ui.register_part(
        HEADER_INSPECT,
        PartRef::of(Part::LABEL),
        Rect::new(inspector_x, area.y, inspector_width, 1),
    );
    right = inspector_x.saturating_sub(1);

    if right > left.saturating_add(capability_width).saturating_add(2) {
        let cap_x = right.saturating_sub(capability_width.saturating_add(1));
        ui.paint_str(
            Rect::new(cap_x, area.y, width(capability), 1),
            capability,
            faint,
        );
        ui.paint_str(
            Rect::new(cap_x.saturating_add(width(capability)), area.y, 3, 1),
            " · ",
            faint,
        );
        let mut dimension_x = cap_x.saturating_add(width(capability)).saturating_add(3);
        for fragment in [width_text.as_str(), "×", height_text.as_str()] {
            let columns = width(fragment);
            ui.paint_str(Rect::new(dimension_x, area.y, columns, 1), fragment, faint);
            dimension_x = dimension_x.saturating_add(columns);
        }
    }
}

fn header_action_style(ui: &mut Ui<'_>, id: Id, muted: PaintStyle) -> PaintStyle {
    if ui.state(id).contains(StateFlags::HOVERED) {
        shell_part_style(
            ui,
            termrock::Family::LIST,
            Part::CONTAINER,
            StateFlags::HOVERED,
        )
    } else {
        muted
    }
}

fn fade_paint(style: PaintStyle, steps: u8) -> PaintStyle {
    if steps == 0 {
        return style;
    }
    let Some(termrock::Color::Rgb(red, green, blue)) = style.as_style().fg else {
        return style;
    };
    let keep = if steps == 1 { 0.8 } else { 0.55 };
    let mix = |channel: u8| (f32::from(channel) * keep).round() as u8;
    style.fg(termrock::Color::Rgb(mix(red), mix(green), mix(blue)))
}

fn paint_sidebar_row(ui: &mut Ui<'_>, row: Rect, flags: StateFlags, _key: ItemKey, node: &NavNode) {
    if row.is_empty() {
        return;
    }
    let focused = flags.contains(StateFlags::FOCUSED);
    let selected = flags.contains(StateFlags::SELECTED);
    let fade = node.fade;
    let canvas = shell_compat_style(ui.surface_style());
    let label_ink = fade_paint(canvas, fade);
    let group_ink = fade_paint(shell_text_style(ui, termrock::FgStep::Secondary), fade);
    let selected_patch =
        ui.paint_patch(&termrock::StylePatch::new().set_bg(termrock::Role::AccentTint));
    let selected_surface =
        shell_compat_style(canvas.patch(selected_patch)).add_modifier(termrock::Modifier::BOLD);
    let selected_ink = shell_compat_style(
        selected_surface
            .patch(ui.paint_patch(&termrock::StylePatch::new().set_fg(termrock::Role::Accent))),
    );
    let accent = ui.paint_patch(&termrock::StylePatch::new().set_fg(termrock::Role::Accent));
    let marker_surface = if selected { selected_surface } else { canvas };
    let marker = fade_paint(shell_compat_style(marker_surface.patch(accent)), fade)
        .add_modifier(termrock::Modifier::BOLD);

    ui.fill(row, if selected { selected_surface } else { canvas });
    if !focused && !selected {
        let gutter = shell_compat_style(canvas.patch(
            ui.paint_patch(&termrock::StylePatch::new().set_fg(termrock::Role::CurrentSurface)),
        ));
        ui.fill(Rect::new(row.x, row.y, 1, 1), gutter);
    }
    if focused {
        ui.glyph(Rect::new(row.x, row.y, 1, 1), GlyphRole::FocusBar, marker);
    }
    let indent = node.depth.saturating_mul(2);
    let mark_x = row.x.saturating_add(1).saturating_add(indent);
    if node.parent {
        let role = if flags.contains(StateFlags::EXPANDED) {
            GlyphRole::Expanded
        } else {
            GlyphRole::Collapsed
        };
        ui.glyph(Rect::new(mark_x, row.y, 1, 1), role, group_ink);
    } else if selected {
        ui.glyph(Rect::new(mark_x, row.y, 1, 1), GlyphRole::Chosen, marker);
    }
    let label_x = mark_x.saturating_add(2);
    let room = row.right().saturating_sub(label_x).saturating_sub(1);
    let text = clip_label(node.label, room);
    let label = if node.parent {
        label_ink
    } else if selected {
        selected_ink
    } else {
        label_ink
    };
    if room > 0 {
        ui.fill(Rect::new(label_x, row.y, room, 1), label);
    }
    ui.paint_str(
        Rect::new(label_x, row.y, row.right().saturating_sub(label_x), 1),
        &text,
        label,
    );
}

fn paint_sidebar_heading(ui: &mut Ui<'_>, area: Rect, gutter: u16, focused: bool) {
    if area.is_empty() {
        return;
    }
    let (title, secondary, _, _, _) = header_styles(ui);
    let style = if focused { title } else { secondary };
    let title_area = Rect::new(
        area.x.saturating_add(gutter),
        area.y,
        area.width.saturating_sub(gutter.saturating_mul(2)),
        1,
    );
    ui.paint_str(title_area, "Components", style);
    let rule_area = Rect::new(title_area.x, area.y.saturating_add(1), title_area.width, 1);
    let glyph = ui.glyph_str(GlyphRole::RuleQuiet);
    let rule = glyph.repeat(usize::from(rule_area.width));
    let style = shell_compat_style(
        ui.surface_style()
            .patch(ui.paint_patch(&termrock::StylePatch::new().set_fg(termrock::Role::ChromeRule))),
    );
    ui.paint_str(rule_area, &rule, style);
}

fn paint_placeholder(ui: &mut Ui<'_>, area: Rect, label: &str) {
    if area.is_empty() {
        return;
    }
    let title = shell_part_style(
        ui,
        termrock::Family::PANEL,
        Part::TITLE,
        StateFlags::empty(),
    );
    ui.paint_str(Rect::new(area.x, area.y, area.width, 1), label, title);
}

fn clip_label(label: &str, max: u16) -> String {
    let max = usize::from(max);
    if usize::from(width(label)) <= max {
        return label.to_owned();
    }
    let room = max.saturating_sub(1);
    let mut end = 0;
    let mut used = 0usize;
    for (index, character) in label.char_indices() {
        let mut buf = [0u8; 4];
        let piece = character.encode_utf8(&mut buf);
        let piece_width = usize::from(width(piece));
        if used.saturating_add(piece_width) > room {
            break;
        }
        used = used.saturating_add(piece_width);
        end = index.saturating_add(character.len_utf8());
    }
    let mut out = label[..end].to_owned();
    out.push('…');
    out
}

fn paint_inspector_props(ui: &mut Ui<'_>, area: Rect, app: &App) {
    let focus = if ui.state(NAV).contains(StateFlags::FOCUSED) {
        "navigation"
    } else if ui.state(INSPECTOR_CLOSE).contains(StateFlags::FOCUSED) {
        "inspector"
    } else {
        "page"
    };
    let hover = if ui.hovered_part(NAV).is_some() {
        "navigation"
    } else {
        "—"
    };
    let pressed = if ui.pressed_part(NAV).is_some() {
        "navigation"
    } else {
        "—"
    };
    let rows = [
        ("page", app.nav_label()),
        ("focus", focus),
        ("hover", hover),
        ("pressed", pressed),
        ("colors", ui.theme().capability.color.label()),
    ];
    Props::new(&rows).draw(ui, area);
}

fn paint_inspector(ui: &mut Ui<'_>, area: Rect, app: &App) {
    inspector_panel().draw(ui, area, |ui, inner| {
        paint_inspector_props(ui, inner, app);
    });
}

fn nav_query_display(query: &str, row_width: u16) -> String {
    match row_width {
        0 => String::new(),
        1 => "/".to_owned(),
        width => format!("/ {}", termrock::truncate(query, width.saturating_sub(3))),
    }
}

fn nav_query_cursor(area: Rect, displayed: &str) -> Option<Position> {
    if area.width < 3 {
        return None;
    }
    let x = area.x.saturating_add(width(displayed));
    (x < area.right()).then(|| Position::new(x, area.y))
}

fn paint_nav_query(ui: &mut Ui<'_>, area: Rect, query: &NavQuery, focused: bool) {
    if area.is_empty() {
        return;
    }
    let field = shell_compat_style(ui.surface_style()).patch(ui.paint_patch(
        &termrock::StylePatch::new().set_bg(termrock::Role::Surface(termrock::Surface::Field)),
    ));
    ui.fill(area, field);

    let prompt = field.patch(ui.paint_patch(
        &termrock::StylePatch::new().set_fg(termrock::Role::Fg(termrock::FgStep::Faint)),
    ));
    let input = field.patch(ui.paint_patch(
        &termrock::StylePatch::new().set_fg(termrock::Role::Fg(termrock::FgStep::Primary)),
    ));
    if area.width > 0 {
        ui.paint_str(Rect::new(area.x, area.y, 1, 1), "/", prompt);
    }

    let display = nav_query_display(&query.text, area.width);
    if area.width > 2 && display.len() > 2 {
        let text = &display[2..];
        ui.paint_str(
            Rect::new(
                area.x.saturating_add(2),
                area.y,
                area.width.saturating_sub(3),
                1,
            ),
            text,
            input,
        );
    }
    if focused && let Some(cursor) = nav_query_cursor(area, &display) {
        ui.set_bar_cursor(NAV, cursor);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NavCursorKind {
    None,
    Group,
    Leaf,
}

fn nav_cursor_kind(state: &TreeState, nodes: &[NavNode], query_active: bool) -> NavCursorKind {
    let Some(key) = state.cursor() else {
        return NavCursorKind::None;
    };
    let Some(node) = nodes
        .iter()
        .find(|node| node.key == key && (!query_active || node.query_match))
    else {
        return NavCursorKind::None;
    };
    if node.parent {
        NavCursorKind::Group
    } else if node.target.is_some() {
        NavCursorKind::Leaf
    } else {
        NavCursorKind::None
    }
}

const NAV_QUERY_MATCH_HINTS: &[(&str, &str)] = &[
    ("type", "Filter"),
    ("↑↓", "Move"),
    ("Backspace", "Edit"),
    ("Esc", "Clear"),
    ("Enter", "Open"),
];
const NAV_QUERY_EMPTY_HINTS: &[(&str, &str)] = &[
    ("type", "Filter"),
    ("↑↓", "Move"),
    ("Backspace", "Edit"),
    ("Esc", "Clear"),
];
const NAV_GROUP_HINTS: &[(&str, &str)] = &[
    ("↑↓", "Move"),
    ("← →", "Fold"),
    ("Enter", "Toggle"),
    ("/", "Filter"),
    ("q", "Quit"),
];
const NAV_LEAF_HINTS: &[(&str, &str)] = &[
    ("↑↓", "Move"),
    ("Enter", "Open"),
    ("/", "Filter"),
    ("q", "Quit"),
];
const NAV_EMPTY_HINTS: &[(&str, &str)] = &[("↑↓", "Move"), ("/", "Filter"), ("q", "Quit")];
const HELP_HINTS: &[(&str, &str)] = &[("↑↓", "Scroll"), ("Esc", "Close")];

#[expect(
    clippy::too_many_arguments,
    reason = "the footer paints pre-resolved part styles without re-resolving per hint"
)]
fn paint_hint(
    ui: &mut Ui<'_>,
    area: Rect,
    x: &mut u16,
    reserved: u16,
    key_style: PaintStyle,
    action_style: PaintStyle,
    key: &str,
    action: &str,
) {
    let key_width = width(key);
    let action_width = width(action);
    let hint_width = key_width.saturating_add(action_width).saturating_add(3);
    if x.saturating_add(hint_width).saturating_add(reserved) > area.right() {
        return;
    }
    ui.paint_str(Rect::new(*x, area.y, key_width, 1), key, key_style);
    *x = x.saturating_add(key_width.saturating_add(1));
    ui.paint_str(Rect::new(*x, area.y, action_width, 1), action, action_style);
    *x = x.saturating_add(action_width.saturating_add(2));
}

fn paint_footer(
    ui: &mut Ui<'_>,
    area: Rect,
    hints: &[(&str, &str)],
    tab_hint: Option<(&str, &str)>,
    status: Option<&str>,
) {
    if area.is_empty() {
        return;
    }
    let canvas = shell_compat_style(ui.surface_style());
    ui.fill(area, canvas);
    let key_style = shell_part_style(
        ui,
        termrock::Family::KEYHINT,
        Part::KEY,
        StateFlags::empty(),
    );
    let action_style = shell_text_style(ui, termrock::FgStep::Muted);
    let mut x = area.x.saturating_add(1);
    let reserved = status.map_or(14, |message| width(message).saturating_add(3));
    let place_tab_before_quit = tab_hint.is_some()
        && hints
            .iter()
            .any(|(key, action)| *key == "q" && *action == "Quit");
    let mut tab_placed = false;
    for &(key, action) in hints {
        if place_tab_before_quit
            && key == "q"
            && action == "Quit"
            && let Some((tab, action)) = tab_hint
        {
            paint_hint(
                ui,
                area,
                &mut x,
                reserved,
                key_style,
                action_style,
                tab,
                action,
            );
            tab_placed = true;
        }
        paint_hint(
            ui,
            area,
            &mut x,
            reserved,
            key_style,
            action_style,
            key,
            action,
        );
    }
    if !tab_placed && let Some((key, action)) = tab_hint {
        paint_hint(
            ui,
            area,
            &mut x,
            reserved,
            key_style,
            action_style,
            key,
            action,
        );
    }
    if let Some(message) = status {
        let message_width = width(message);
        if area.right() > message_width.saturating_add(1) {
            let style = canvas.patch(
                ui.paint_patch(
                    &termrock::StylePatch::new()
                        .set_fg(termrock::Role::Fg(termrock::FgStep::Secondary)),
                ),
            );
            ui.paint_str(
                Rect::new(
                    area.right().saturating_sub(message_width).saturating_sub(1),
                    area.y,
                    message_width,
                    1,
                ),
                message,
                style,
            );
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl TuiApp for App {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == termrock::UpdateCause::Bootstrap {
            cx.focus(NAV);
        }
        let mut response = Response::ignored();
        if !self.motion_paused
            && cx.update_cause() == termrock::UpdateCause::Tick
            && self.status.as_ref().is_some_and(|(_, since)| {
                cx.now().saturating_duration_since(*since) > std::time::Duration::from_secs(4)
            })
        {
            self.status = None;
            response = response.repaint();
        }
        response |= shell_brand().update(cx).erase();
        response |= shell_status().update(cx).erase();

        let _ = too_small_notice();
        let _ = missing_page_panel();
        let _ = inspector_panel();
        let command = cx.command();
        let inspector_modal_active = self.inspector_modal_active(cx);
        let consume_nav_tree_key = match nav_query_input(cx, self.nav_query.is_some()) {
            Some(NavQueryInput::Consume) => {
                response |= Response::consumed();
                true
            }
            Some(input) => {
                self.apply_nav_query_input(input);
                response |= Response::changed();
                false
            }
            None => false,
        };
        self.update_header(cx, &mut response);
        match command {
            Some(QUIT | QUIT_CTRL) => {
                self.quit = true;
                cx.quit();
            }
            Some(NEXT_PAGE) if !inspector_modal_active => {
                let count = NAV_ENTRIES.len();
                if count > 0 {
                    let current = nav_index_for_target(self.nav_target).unwrap_or(0);
                    response |= self.goto_entry_with_cx(cx, current.saturating_add(1) % count);
                }
            }
            Some(PREV_PAGE) if !inspector_modal_active => {
                let count = NAV_ENTRIES.len();
                if count > 0 {
                    let current = nav_index_for_target(self.nav_target).unwrap_or(0);
                    let previous = current.checked_sub(1).unwrap_or(count.saturating_sub(1));
                    response |= self.goto_entry_with_cx(cx, previous);
                }
            }
            Some(NEXT_PAGE | PREV_PAGE) => {}
            Some(HELP_COMMAND) if !cx.is_open(HELP) => {
                self.open_help(cx);
            }
            Some(INSPECTOR_COMMAND) if cx.update_cause() == termrock::UpdateCause::Event => {
                self.toggle_inspector(&mut response);
            }
            _ => {}
        }
        if self.show_live_page()
            && let Some(action) = command
            && !matches!(
                action,
                QUIT | QUIT_CTRL | NEXT_PAGE | PREV_PAGE | HELP_COMMAND | INSPECTOR_COMMAND
            )
            && let Some(active) = self.active_mut()
        {
            response |= active.command(cx, action);
        }
        let sidebar_area = self.resolved_shell_layout(cx.viewport()).sidebar;
        let sidebar = SidebarPresentation::new(
            sidebar_area,
            sidebar_panel_inner(sidebar_area, cx.design().space.card_inset),
            cx.design().space.gutter,
            self.nav_query.is_some(),
        );
        let presentation = NavTreePresentation::new(sidebar.tree, self.nav_query.as_ref());
        let nodes = nav_nodes_for_query(
            &self.nav_state,
            sidebar.tree.height,
            self.nav_query.as_ref(),
        );
        let query_disclosure = self.nav_query.as_ref().and_then(|_| {
            cx.intents(NAV).find_map(|intent| match intent {
                Intent::Pointer {
                    phase: Phase::Click | Phase::DoubleClick,
                    part,
                    ..
                } if matches!(part.part, Part::ICON | Part::ROW) => part.item,
                _ => None,
            })
        });
        let query_disclosure =
            query_disclosure.filter(|key| nodes.iter().any(|node| node.key == *key && node.parent));
        let auto_group_expanded_before = self
            .nav_auto_group
            .map(|key| self.nav_state.is_expanded(key));
        let nav_response =
            if !consume_nav_tree_key && self.nav_presentation.get() == Some(presentation) {
                nav_tree(presentation.query_revision).update(cx, &mut self.nav_state, &nodes)
            } else {
                Response::ignored().repaint()
            };
        let nav_action = nav_response.action_ref().copied();
        response |= nav_response.erase();
        if let Some(key) = query_disclosure {
            self.nav_state.toggle(key);
            self.bump_nav_query_revision();
        }
        if let (Some(owner), Some(was_expanded)) = (self.nav_auto_group, auto_group_expanded_before)
            && self.nav_state.is_expanded(owner) != was_expanded
        {
            self.nav_auto_group = None;
        }
        let mut enter_content = false;
        let picked = match nav_action {
            Some(TreeAction::Chose(key)) => target_for_key(key, &nodes),
            Some(TreeAction::Activated(key)) => {
                let picked = target_for_key(key, &nodes);
                enter_content = picked.is_some_and(|(_, target)| target.has_content());
                if picked.is_some() {
                    self.end_nav_query_for_activation();
                }
                picked
            }
            Some(TreeAction::Moved) => self
                .nav_state
                .cursor()
                .and_then(|key| target_for_key(key, &nodes)),
            Some(TreeAction::Expanded(_) | TreeAction::Collapsed(_)) | None => None,
        };
        if let Some((index, _target)) = picked {
            response |= self.goto_entry_with_cx(cx, index);
        }

        if self.nav_target == NavTarget::Selection {
            response |= self.selectable.update(cx);
        } else {
            self.selectable.blur();
        }

        if self.show_live_page()
            && !(cx.update_cause() == termrock::UpdateCause::Tick
                && (cx.is_open(HELP) || self.motion_paused))
            && let Some(active) = self.active_mut()
        {
            let update = active.update(cx);
            response |= update.response;
            if let Some(status) = update.status {
                self.status = Some((status, cx.now()));
                response = response.repaint();
            }
        }
        if enter_content {
            cx.focus_next();
        }
        self.update_help(cx, &mut response);
        self.update_inspector(cx, &mut response);
        self.help_open = cx.is_open(HELP);
        self.overlay_open = cx.top_layer() != termrock::LayerId::PAGE;
        if self.nav_target == NavTarget::Placeholder(PANE_TREE_ENTRY) {
            let viewport = cx.viewport();
            let page_area = (viewport.width >= 72 && viewport.height >= 20 && !self.overlay_open)
                .then(|| self.resolved_shell_layout(viewport).main);
            response |= self.pane_tree.prepare_for_presentation(cx, page_area);
        }
        self.update_notice_reveals(cx, &mut response);
        if !self.motion_paused
            && let Some((_, since)) = &self.status
        {
            let deadline = since
                .saturating_add(std::time::Duration::from_secs(4))
                .saturating_add(std::time::Duration::from_nanos(1));
            cx.request_repaint_at(deadline);
        }
        response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let full = ui.full();

        ui.fill(full, shell_compat_style(ui.surface_style()));
        if full.width < 72 || full.height < 20 {
            too_small_notice().draw(ui, full);
            return;
        }
        let shell = self.resolved_shell_layout(full);

        shell_brand().draw(ui, shell.header);
        shell_status().draw(ui, shell.footer);
        let entry = nav_index_for_target(self.nav_target)
            .and_then(|index| NAV_ENTRIES.get(index))
            .copied();
        let section = entry.map(|entry| entry.section).unwrap_or("");
        let label = entry.map(|entry| entry.label).unwrap_or("Overview");
        let nav_flags = ui.state(NAV);
        let nav_focused = nav_flags.contains(StateFlags::FOCUSED)
            && !nav_flags.contains(StateFlags::INACTIVE_LAYER)
            && !self.overlay_open;
        let route_count = NAV_ENTRIES.len().to_string();
        let panel = Panel::new(NAV)
            .title("")
            .meta(&route_count)
            .focused(nav_focused)
            .patch_part(&NAV_PANEL_PARTS);
        let panel_inner = panel.inner(ui, shell.sidebar);
        panel.draw(ui, shell.sidebar, |_, _| ());
        let gutter = ui.design().space.gutter;
        let sidebar =
            SidebarPresentation::new(shell.sidebar, panel_inner, gutter, self.nav_query.is_some());
        let presentation = NavTreePresentation::new(sidebar.tree, self.nav_query.as_ref());
        let nodes = nav_nodes_for_query(
            &self.nav_state,
            sidebar.tree.height,
            self.nav_query.as_ref(),
        );
        let sidebar_surface = ui.theme().raise(ui.surface());
        ui.with_surface(sidebar_surface, |ui| {
            paint_sidebar_heading(ui, shell.sidebar, gutter, nav_focused);
            nav_tree(presentation.query_revision).draw(ui, sidebar.tree, &self.nav_state, &nodes);
            if let (Some(query_area), Some(query)) = (sidebar.query, self.nav_query.as_ref()) {
                paint_nav_query(ui, query_area, query, nav_focused);
            }
        });
        self.nav_presentation.set(Some(presentation));
        paint_header(
            ui,
            shell.header,
            full.width,
            full.height,
            section,
            label,
            self.inspector,
        );
        let mut notice_reveal_targets: Option<[Option<NoticeRevealTarget>; 6]> = None;
        if self.show_live_page() {
            if self.nav_target == NavTarget::Placeholder(NOTICES_ENTRY) {
                if self.nav_query.is_none() && !self.overlay_open {
                    notice_reveal_targets = Some(self.notices.draw_with_reveal_targets(
                        ui,
                        shell.main,
                        self.notice_reveal_revision,
                    ));
                } else {
                    self.notices.draw(ui, shell.main);
                }
            } else if let Some(active) = self.active() {
                active.draw(ui, shell.main);
            } else {
                missing_page_panel().draw(ui, shell.main, |ui, area| {
                    let _ = ui.paint_str(area, "No page selected", ui.surface_style());
                });
            }
        } else if self.nav_target == NavTarget::Selection {
            showcase_demos::pages::selectable::paint(ui, shell.main, &self.selectable);
        } else {
            paint_placeholder(ui, shell.main, label);
        }
        if let Some(inspector) = shell.inspector {
            paint_inspector(ui, inspector, self);
        }
        let (page_hints, page_editing) = if self.show_live_page() {
            self.active()
                .map(|page| (page.hints(ui), page.editing(ui)))
                .unwrap_or_default()
        } else if self.nav_target == NavTarget::Selection {
            (self.selectable.hints(), false)
        } else {
            (&[][..], false)
        };
        let cursor_kind = nav_cursor_kind(&self.nav_state, &nodes, self.nav_query.is_some());
        let modal_page_hints = match self.nav_target {
            NavTarget::Selection => self.selectable.hints(),
            NavTarget::Placeholder(
                CHECKS_RADIOS_ENTRY | EMPTY_ERROR_ENTRY | NOTICES_ENTRY | DIFF_ENTRY
                | MASTER_DETAIL_ENTRY | PANE_TREE_ENTRY,
            ) => page_hints,
            NavTarget::Live(PageId::Pickers | PageId::DialogKinds | PageId::Toolbars) => page_hints,
            NavTarget::Live(_) | NavTarget::Placeholder(_) => &[][..],
        };
        let (footer_hints, tab_hint) = if self.overlay_open {
            if self.help_open {
                (HELP_HINTS, None)
            } else {
                (modal_page_hints, None)
            }
        } else if nav_focused {
            let hints = if self.nav_query.is_some() {
                if cursor_kind == NavCursorKind::Leaf {
                    NAV_QUERY_MATCH_HINTS
                } else {
                    NAV_QUERY_EMPTY_HINTS
                }
            } else {
                match cursor_kind {
                    NavCursorKind::Group => NAV_GROUP_HINTS,
                    NavCursorKind::Leaf => NAV_LEAF_HINTS,
                    NavCursorKind::None => NAV_EMPTY_HINTS,
                }
            };
            let tab = if self.nav_target.has_content() {
                ("Tab", "Into page")
            } else {
                ("Tab", "Next")
            };
            (hints, Some(tab))
        } else {
            let tab = (!page_editing).then_some(("Tab", "Next"));
            (page_hints, tab)
        };
        paint_footer(
            ui,
            shell.footer,
            footer_hints,
            tab_hint,
            self.status.as_ref().map(|(status, _)| status.0.as_str()),
        );
        if let Some(targets) = notice_reveal_targets
            && self.nav_target == NavTarget::Placeholder(NOTICES_ENTRY)
            && self.nav_query.is_none()
            && !self.overlay_open
        {
            let mut reveals = self.notice_reveals.borrow_mut();
            for (reveal, target) in reveals.iter_mut().zip(targets) {
                if let Some(target) = target {
                    let _ = reveal.draw(
                        ui,
                        &target.notice,
                        target.logical_area,
                        target.window,
                        target.revision,
                    );
                } else {
                    reveal.reset();
                }
            }
        }
        ui.layer(HELP, |ui, area| {
            Self::help_overlay().draw(ui, area, &self.help_state);
        });
        ui.layer(INSPECTOR, |ui, area| {
            Self::inspector_dialog(full).draw(ui, area, &self.inspector_state, |ui, body| {
                paint_inspector_props(ui, body, self)
            });
        });
        if label == "Selection" {
            showcase_demos::pages::selectable::Model::draw_dialog(ui, &self.selectable);
        }
    }

    fn should_quit(&self) -> bool {
        self.quit
    }

    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }

    fn min_size(&self) -> Size {
        Size {
            min: (72, 20),
            preferred: (120, 40),
        }
    }

    fn on_esc(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        cx.focus(NAV);
        Response::changed()
    }

    fn accept_paused_clock_step(&mut self, step_ms: u64) -> bool {
        if !self.motion_paused {
            return false;
        }
        self.advance_paused_clock(step_ms);
        true
    }
}

impl App {
    fn advance_paused_clock(&mut self, step_ms: u64) {
        if let Some((_, since)) = &mut self.status {
            *since = Moment::from_duration(
                since
                    .as_duration()
                    .saturating_sub(std::time::Duration::from_millis(step_ms)),
            );
        }
        let frames = usize::try_from(step_ms).unwrap_or(usize::MAX);
        self.paused_frame = self.paused_frame.saturating_add(frames);
        for page in &mut self.pages {
            page.seek_paused(self.paused_frame);
        }
        self.checks_radios.seek_paused(self.paused_frame);
        self.diff.seek_paused(self.paused_frame);
    }
}

pub fn run() -> std::io::Result<()> {
    let (nav, theme, paused, frame) = parse_args(std::env::args().skip(1))?;
    termrock::run(App::boot(nav, paused, frame), theme)
}

fn parse_args(
    mut args: impl Iterator<Item = String>,
) -> std::io::Result<(usize, Theme, bool, usize)> {
    let mut theme = Theme::junie();
    let mut nav = 0usize;
    let mut paused = false;
    let mut frame = 0usize;
    let mut frame_seen = false;
    let mut full_seen = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--theme" => {
                if let Some(value) = args.next() {
                    theme = if value.eq_ignore_ascii_case("paper") {
                        Theme::paper()
                    } else {
                        Theme::junie()
                    };
                }
            }
            "--color" => {
                if let Some(value) = args.next() {
                    let level = match value.to_ascii_lowercase().as_str() {
                        "truecolor" | "24bit" => Some(ColorLevel::TrueColor),
                        "256" | "ansi256" => Some(ColorLevel::Ansi256),
                        "16" | "ansi16" => Some(ColorLevel::Ansi16),
                        "none" | "mono" => Some(ColorLevel::Mono),
                        _ => None,
                    };
                    if let Some(level) = level {
                        theme = theme.downgrade(level);
                    }
                }
            }
            "--page" => {
                if let Some(value) = args.next()
                    && let Some(index) = nav_index_from_name(&value)
                {
                    nav = index;
                }
            }
            "--motion" => {
                let Some(value) = args.next() else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "--motion requires full or paused",
                    ));
                };
                match value.as_str() {
                    "full" => {
                        full_seen = true;
                        paused = false;
                    }
                    "paused" => paused = true,
                    _ => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "--motion must be full or paused",
                        ));
                    }
                }
            }
            "--frame" => {
                let Some(value) = args.next() else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "--frame requires an integer",
                    ));
                };
                frame = value
                    .parse::<usize>()
                    .ok()
                    .filter(|n| *n <= 10_000)
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "--frame must be between 0 and 10000",
                        )
                    })?;
                frame_seen = true;
                paused = true;
            }
            _ => {}
        }
    }
    if frame_seen && full_seen {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "--frame cannot be combined with --motion full",
        ));
    }
    Ok((nav, theme, paused, frame))
}
