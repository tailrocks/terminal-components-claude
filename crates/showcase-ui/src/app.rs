use showcase_demos::render_number::RenderNumber;
use termrock::author::PaintStyle;
use termrock::{
    ActionKey, App as TuiApp, Brand, ByIndex, Chord, ColorLevel, Cx, Dialog, DialogAction,
    DialogState, FrameRead, GlyphRole, Id, Intent, ItemKey, KeyCode, KeyMap, KeyPhase, Moment,
    Panel, PanelKind, Part, PartRef, Phase, Props, Rect, Response, RowFn, Size, StateFlags, Status,
    StatusBar, StatusItem, Theme, TooSmall, Tree, TreeAction, TreeNode, TreeState, Ui, Variant, id,
    width,
};

use showcase_demos::pages::forms::SUBMIT as FORM_SUBMIT;
use showcase_demos::pages::taskrunner::RUN_COMMAND;
use showcase_demos::pages::{
    Page, PageStatus, buttons::ButtonsPage, celleditors::CellEditorsPage, chips::ChipsPage,
    chrome::ChromePage, dialogs::DialogsPage, editable::EditablePage, editor::EditorPage,
    forms::FormsPage, grid::GridPage, inputs::InputsPage, lists::ListsPage, overview::OverviewPage,
    panels::PanelsPage, pickers::PickersPage, progress::ProgressPage, scrolling::ScrollingPage,
    settings::SettingsPage, sidebars::SidebarsPage, tables::TablesPage, taskrunner::TaskRunnerPage,
    terminal::TerminalPage, textareas::TextAreasPage, trees::TreesPage,
};

const NAV: Id = id!("navigation");
const BRAND: Id = id!("brand");
const STATUS: Id = id!("status");
const HELP: Id = id!("help");
const TOO_SMALL: Id = id!("too-small");
const HEADER_HELP: Id = id!("header.help");
const HEADER_INSPECT: Id = id!("header.inspect");
const INSPECTOR: Id = id!("inspector");
const QUIT: ActionKey = ActionKey::application("showcase.quit");
const QUIT_CTRL: ActionKey = ActionKey::application("showcase.quit.ctrl");
const HELP_COMMAND: ActionKey = ActionKey::application("showcase.help");
const INSPECTOR_COMMAND: ActionKey = ActionKey::application("showcase.inspector");
const NEXT_PAGE: ActionKey = ActionKey::application("showcase.page.next");
const PREV_PAGE: ActionKey = ActionKey::application("showcase.page.previous");
const HELP_TEXT: &str = "Tab / Shift+Tab   move keyboard focus\n\
↑ ↓ ← →           move inside the focused control\n\
Enter / Space     activate · start editing\n\
Esc               cancel editing · back to navigation\n\
[ ]               previous / next page\n\
0                 jump to navigation\n\
i                 toggle state inspector\n\
q                 quit\n\n\
Mouse: hover to preview, click to focus and activate, wheel to scroll, drag the scrollbar thumb.";

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
    Chrome,
    Settings,
    TaskRunner,
    CellEditors,
}

impl PageId {
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
        Self::Grid,
        Self::Chips,
        Self::Pickers,
        Self::Chrome,
        Self::Settings,
        Self::TaskRunner,
        Self::CellEditors,
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
            Self::Chrome => "Chrome",
            Self::Settings => "Settings",
            Self::TaskRunner => "Task runner",
            Self::CellEditors => "Cell editors",
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
            Self::Chrome => "chrome",
            Self::Settings => "settings",
            Self::TaskRunner => "taskrunner",
            Self::CellEditors => "celleditors",
        }
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|page| *page == self).unwrap_or(0)
    }

    pub fn from_name(value: &str) -> Option<Self> {
        nav_index_from_name(value).and_then(|index| {
            let entry = NAV_ENTRIES.get(index)?;
            entry_is_live(entry).then_some(entry.id)
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
        id: PageId::Overview,
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
        id: PageId::Overview,
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

fn entry_is_live(entry: &NavEntry) -> bool {
    entry.label == "Overview" || entry.id != PageId::Overview
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
    NAV_ENTRIES.iter().position(|entry| {
        entry_is_live(entry)
            && (normalized_name(entry.id.slug()) == value
                || normalized_name(entry.id.title()) == value)
    })
}

fn index_for_page(page: PageId) -> usize {
    NAV_ENTRIES
        .iter()
        .position(|entry| entry_is_live(entry) && entry.id == page)
        .unwrap_or(0)
}

struct NavNode {
    label: &'static str,
    depth: u16,
    parent: bool,
    entry: Option<usize>,
}

fn nav_nodes() -> Vec<NavNode> {
    let mut nodes = Vec::new();
    let mut section = "";
    for (index, entry) in NAV_ENTRIES.iter().enumerate() {
        if entry.section != section {
            section = entry.section;
            if !section.is_empty() {
                nodes.push(NavNode {
                    label: section,
                    depth: 0,
                    parent: true,
                    entry: None,
                });
            }
        }
        nodes.push(NavNode {
            label: entry.label,
            depth: u16::from(!entry.section.is_empty()),
            parent: false,
            entry: Some(index),
        });
    }
    nodes
}

fn node_of(node: &NavNode) -> TreeNode {
    if node.parent {
        TreeNode::parent(node.depth)
    } else {
        TreeNode::leaf(node.depth)
    }
}

const SIDEBAR_CHROME: [(Part, termrock::StylePatch); 1] = [(
    Part::CONTAINER,
    termrock::StylePatch::new().set_bg(termrock::Role::Surface(termrock::Surface::Surface)),
)];

fn nav_tree() -> Tree<'static, NavNode, ByIndex, impl RowFn<NavNode>> {
    Tree::new(NAV)
        .patch_part(&SIDEBAR_CHROME)
        .node(&node_of)
        .render_row(&paint_sidebar_row)
}

fn place_nav(state: &mut TreeState, entry: usize) {
    let nodes = nav_nodes();
    let Some(source) = nodes.iter().position(|node| node.entry == Some(entry)) else {
        return;
    };
    let active_parent = if nodes[source].depth > 0 {
        (0..source).rev().find(|&index| nodes[index].parent)
    } else {
        None
    };
    for (index, node) in nodes.iter().enumerate() {
        if node.parent {
            let key = ItemKey::index(index);
            if Some(index) == active_parent {
                state.expand(key);
            } else {
                state.collapse(key);
            }
        }
    }
    let key = ItemKey::index(source);
    let visible = visible_index(&nodes, source, state);
    state.choose(Some(key));
    state.set_cursor(visible, key);
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
        if node.parent && !state.is_expanded(ItemKey::index(index)) {
            collapsed_at = Some(node.depth);
        }
    }
    0
}

fn entry_for_key(key: ItemKey, nodes: &[NavNode]) -> Option<usize> {
    let source = (0..nodes.len()).find(|&index| ItemKey::index(index) == key)?;
    nodes.get(source)?.entry
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
        PageId::Chrome => Box::new(ChromePage::new()),
        PageId::Settings => Box::new(SettingsPage::new()),
        PageId::TaskRunner => Box::new(TaskRunnerPage::new()),
        PageId::CellEditors => Box::new(CellEditorsPage::new()),
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
    nav_index: usize,
    nav_state: TreeState,
    pages: Vec<Box<dyn Page>>,
    help_state: DialogState,
    selectable: showcase_demos::pages::selectable::Model,
    keymap: KeyMap,
    inspector: bool,
    shell_layout_cache: std::cell::Cell<Option<CachedShellLayout>>,
    quit: bool,
    status: Option<(PageStatus, Moment)>,
    motion_paused: bool,
    /// Paused-frame index. `--frame` seeks here, and a paused clock paste adds its milliseconds.
    paused_frame: usize,
}

impl core::fmt::Debug for App {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("App")
            .field("page", &self.page)
            .field("nav_index", &self.nav_index)
            .field("nav_state", &self.nav_state)
            .field("pages", &self.pages.len())
            .field("help_state", &self.help_state)
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
        let selected = NAV_ENTRIES.get(nav_index).copied();
        let initial = selected
            .filter(entry_is_live)
            .map(|entry| entry.id)
            .unwrap_or(PageId::Overview);
        let mut nav_state = TreeState::new();
        place_nav(&mut nav_state, nav_index);
        let mut pages: Vec<Box<dyn Page>> = PageId::ALL.into_iter().map(page).collect();
        if paused {
            for page in &mut pages {
                page.seek_paused(frame);
            }
        }
        Self {
            page: initial,
            nav_index,
            nav_state,
            pages,
            help_state: DialogState::default(),
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

    pub const fn page(&self) -> PageId {
        self.page
    }

    pub const fn quit(&self) -> bool {
        self.quit
    }

    fn goto_entry(&mut self, index: usize) {
        let Some(entry) = NAV_ENTRIES.get(index).copied() else {
            return;
        };
        self.nav_index = index;
        if entry_is_live(&entry) {
            self.page = entry.id;
        }
        place_nav(&mut self.nav_state, index);
    }

    fn goto_page(&mut self, page: PageId) {
        if let Some(index) = NAV_ENTRIES
            .iter()
            .position(|e| e.id == page && entry_is_live(e))
        {
            self.goto_entry(index);
        } else {
            self.page = page;
        }
    }

    fn resolved_shell_layout(&self, area: Rect) -> ShellLayout {
        if let Some(cached) = self.shell_layout_cache.get()
            && cached.area == area
            && cached.inspector == self.inspector
        {
            return cached.layout;
        }
        let layout = shell_layout(area, self.inspector);
        self.shell_layout_cache.set(Some(CachedShellLayout {
            area,
            inspector: self.inspector,
            layout,
        }));
        layout
    }

    fn show_live_page(&self) -> bool {
        NAV_ENTRIES.get(self.nav_index).is_some_and(entry_is_live)
    }

    fn nav_label(&self) -> &'static str {
        NAV_ENTRIES
            .get(self.nav_index)
            .map(|entry| entry.label)
            .unwrap_or("Overview")
    }

    fn active(&self) -> Option<&dyn Page> {
        self.pages.get(self.page.index()).map(Box::as_ref)
    }

    fn help_dialog() -> Dialog<'static> {
        Dialog::info(HELP, "Keyboard & mouse")
            .description(HELP_TEXT)
            .width(70)
    }

    fn update_help(&mut self, cx: &mut Cx<'_>, response: &mut Response<()>) {
        let help = Self::help_dialog().update(cx, &mut self.help_state);
        if let Some(action) = help.action_ref() {
            match action {
                DialogAction::Action(_) | DialogAction::Dismissed(_) => {
                    if cx.is_open(HELP) {
                        cx.close_layer(HELP, None);
                    }
                }
            }
        }
        *response |= help.erase();
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
                let layer = Self::help_dialog().layer(cx);
                cx.open_layer(HELP, layer);
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
            self.inspector = !self.inspector;
            *response |= Response::changed();
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

#[derive(Clone, Copy, Debug)]
struct CachedShellLayout {
    area: Rect,
    inspector: bool,
    layout: ShellLayout,
}

/// The panes the shell body is split into.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pane {
    Sidebar,
    Main,
    Spacer,
    Inspector,
}

/// The sidebar rail is wider on roomy terminals.
const SIDEBAR_WIDE: u16 = 25;
const SIDEBAR_NARROW: u16 = 20;
const SIDEBAR_WIDE_FROM: u16 = 110;
/// The inspector dock needs room to sit beside the main area.
const INSPECTOR_WIDTH: u16 = 30;
const INSPECTOR_MIN_WIDTH: u16 = 100;
/// The clear column between the main area and the inspector dock.
const SHELL_GAP: u16 = 1;

/// A fixed-width pane policy.
fn fixed_pane(width: u16) -> termrock::layout::ChildPolicy {
    termrock::layout::ChildPolicy::new(termrock::layout::Size::exact(width, 1))
        .min(width, 0)
        .max(Some(width), None)
}

/// A pane that takes the space the fixed panes leave.
fn flexible_pane() -> termrock::layout::ChildPolicy {
    termrock::layout::ChildPolicy::new(termrock::layout::Size::default()).flex(1)
}

/// Resolve the shell body from the shared layout plan.
///
/// The app states policy only: a fixed sidebar rail, a flexible main area and
/// an optional fixed inspector dock that keeps one clear column before it. The
/// shared plan owns the geometry.
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

fn shell_row_style(style: PaintStyle, flags: StateFlags) -> PaintStyle {
    let style = shell_compat_style(style);
    if flags.contains(StateFlags::FOCUSED) {
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

fn tag_accent_bg(theme: &termrock::Theme) -> termrock::Color {
    let drifted = termrock::Color::from_u32(0x000f_2e13);
    if theme.color.accent_tint == drifted {
        termrock::Color::from_u32(0x0017_3f1c)
    } else {
        theme.color.accent_tint
    }
}

fn sidebar_plane(ui: &Ui<'_>) -> PaintStyle {
    shell_compat_style(ui.surface_style()).bg(ui.theme().bg(termrock::Surface::Surface))
}

/// Flat catalog the frozen showcase frames paint. The grouped `NAV_ENTRIES`
/// tree stays the page index; this list is the visible shell.
const LEGACY_NAV: &[(&str, &str)] = &[
    ("Overview", "Foundations"),
    ("Buttons", "Components"),
    ("Inputs", "Components"),
    ("Text areas", "Components"),
    ("Forms", "Components"),
    ("Lists", "Components"),
    ("Trees", "Components"),
    ("Tables", "Components"),
    ("Editable tables", "Components"),
    ("Panels", "Components"),
    ("Sidebars", "Components"),
    ("Dialogs", "Components"),
    ("Progress", "Components"),
    ("Scrolling", "Components"),
    ("Terminal", "Components"),
    ("Code editor", "Components"),
    ("Diff", "Components"),
    ("Data grid", "Components"),
    ("Chips & selects", "Components"),
    ("Pickers", "Components"),
    ("Chrome", "Components"),
    ("Settings", "Screens"),
    ("Task runner", "Screens"),
];

fn legacy_crumb(label: &str) -> (&str, &str) {
    const ALIASES: &[(&str, &str)] = &[("Basic table", "Tables"), ("Editable", "Editable tables")];
    let shown = ALIASES
        .iter()
        .find(|(from, _)| *from == label)
        .map(|(_, to)| *to)
        .unwrap_or(label);
    LEGACY_NAV
        .iter()
        .find(|(row, _)| *row == shown)
        .copied()
        .unwrap_or((shown, "Components"))
}

fn legacy_row_at_y(y: u16, sidebar_y: u16, height: u16) -> Option<&'static str> {
    let mut cur_y = sidebar_y;
    let mut previous: Option<&str> = None;
    for (row_label, group) in LEGACY_NAV {
        if previous != Some(*group) {
            if previous.is_some() {
                if cur_y >= sidebar_y.saturating_add(height) {
                    break;
                }
                cur_y = cur_y.saturating_add(1);
            }
            if cur_y >= sidebar_y.saturating_add(height) {
                break;
            }
            cur_y = cur_y.saturating_add(1);
            previous = Some(*group);
        }
        if cur_y >= sidebar_y.saturating_add(height) {
            break;
        }
        if cur_y == y {
            return Some(*row_label);
        }
        cur_y = cur_y.saturating_add(1);
    }
    None
}

fn page_for_legacy_label(label: &str) -> Option<PageId> {
    match label {
        "Overview" => Some(PageId::Overview),
        "Buttons" => Some(PageId::Buttons),
        "Inputs" => Some(PageId::Inputs),
        "Text areas" => Some(PageId::TextAreas),
        "Forms" => Some(PageId::Forms),
        "Lists" => Some(PageId::Lists),
        "Trees" => Some(PageId::Trees),
        "Tables" => Some(PageId::Tables),
        "Editable tables" | "Editable" => Some(PageId::Editable),
        "Panels" => Some(PageId::Panels),
        "Sidebars" => Some(PageId::Sidebars),
        "Dialogs" => Some(PageId::Dialogs),
        "Progress" => Some(PageId::Progress),
        "Scrolling" => Some(PageId::Scrolling),
        "Terminal" => Some(PageId::Terminal),
        "Code editor" => Some(PageId::Editor),
        "Diff" => None,
        "Data grid" => Some(PageId::Grid),
        "Chips & selects" => Some(PageId::Chips),
        "Pickers" => Some(PageId::Pickers),
        "Chrome" => Some(PageId::Chrome),
        "Settings" => Some(PageId::Settings),
        "Task runner" => Some(PageId::TaskRunner),
        _ => None,
    }
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
    let muted = shell_part_style(
        ui,
        termrock::Family::PANEL,
        Part::DETAIL,
        StateFlags::empty(),
    );
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
    x = x.saturating_add(14);
    let _ = (section, faint);
    let budget = inspector_x.saturating_sub(x).saturating_sub(2);
    let (shown_label, group) = legacy_crumb(label);
    let crumb = format!("/ {group} / {shown_label}");
    let shown = clip_label(&crumb, budget);
    let muted = chrome_fg(
        ui,
        termrock::theme::baseline_muted(ui.theme().capability.color),
    );
    ui.paint_str(
        Rect::new(x, area.y, area.right().saturating_sub(x), 1),
        &shown,
        muted,
    );
    x.saturating_add(width(&shown))
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
    let level = ui.theme().capability.color;
    let muted = chrome_fg(ui, termrock::theme::baseline_muted(level));
    let faint = chrome_fg(ui, termrock::theme::baseline_faint(level));
    let _ = styles;
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

/// The navigation hit region keeps the sidebar rail inside its one-column
/// inset: the tree list starts two rows down and keeps one clear row below.
/// Painting still fills the whole rail; the shell-chrome pin owns this area.
fn nav_list_area(area: Rect) -> Rect {
    Rect::new(
        area.x.saturating_add(1),
        area.y.saturating_add(2),
        area.width.saturating_sub(2),
        area.height.saturating_sub(3),
    )
}

fn paint_sidebar(ui: &mut Ui<'_>, area: Rect, _state: &TreeState, label: &str, screen_width: u16) {
    if area.is_empty() {
        return;
    }
    if screen_width >= 110 {
        paint_wide_sidebar(ui, area, label);
        return;
    }
    let canvas = shell_compat_style(ui.surface_style());
    ui.fill(area, canvas);
    ui.register_part(NAV, PartRef::of(Part::CONTAINER), area);
    ui.register_control(NAV, nav_list_area(area), termrock::Focusability::Focusable);
    let secondary = shell_text_style(ui, termrock::FgStep::Secondary);
    let primary = shell_text_style(ui, termrock::FgStep::Primary);
    let chosen = primary.add_modifier(termrock::Modifier::BOLD);
    let gutter = shell_compat_style(ui.surface_style().fg(ui.theme().color.accent))
        .add_modifier(termrock::Modifier::BOLD);
    let black = if ui.theme().capability.color == termrock::ColorLevel::TrueColor
        || ui.theme().capability.color == termrock::ColorLevel::Ansi256
    {
        canvas
            .fg(termrock::Color::from_u32(0))
            .bg(termrock::Color::from_u32(0))
    } else {
        canvas
            .fg(termrock::Color::Indexed(0))
            .bg(termrock::Color::Indexed(0))
    };
    let (shown, _) = legacy_crumb(label);
    let focused = ui.state(NAV).contains(StateFlags::FOCUSED);
    for (index, (row_label, _)) in LEGACY_NAV.iter().enumerate() {
        let Ok(offset) = u16::try_from(index) else {
            break;
        };
        let y = area.y.saturating_add(offset);
        if y >= area.bottom() {
            break;
        }
        let selected = *row_label == shown;
        let row_label = if screen_width < 100 {
            clip_label(row_label, 14)
        } else {
            (*row_label).to_owned()
        };
        let row_label = row_label.as_str();
        let fade = if screen_width < 100 {
            nav_fade_steps(index, usize::from(area.height))
        } else {
            0
        };
        let secondary = fade_paint(secondary, fade);
        let primary = fade_paint(primary, fade);
        let chosen = fade_paint(chosen, fade);
        if selected {
            if focused {
                ui.paint_str(Rect::new(area.x, y, 2, 1), "▎›", gutter);
            } else {
                ui.paint_str(Rect::new(area.x, y, 1, 1), " ", black);
                ui.paint_str(Rect::new(area.x.saturating_add(1), y, 1, 1), " ", primary);
            }
            ui.paint_str(
                Rect::new(area.x.saturating_add(2), y, 17, 1),
                &" ".repeat(17),
                chosen,
            );
            ui.paint_str(
                Rect::new(area.x.saturating_add(3), y, width(row_label), 1),
                row_label,
                chosen,
            );
        } else {
            ui.paint_str(Rect::new(area.x, y, 1, 1), " ", black);
            ui.paint_str(Rect::new(area.x.saturating_add(1), y, 2, 1), "  ", primary);
            ui.paint_str(
                Rect::new(area.x.saturating_add(3), y, 15, 1),
                &" ".repeat(15),
                secondary,
            );
            ui.paint_str(
                Rect::new(area.x.saturating_add(3), y, width(row_label), 1),
                row_label,
                secondary,
            );
        }
        if screen_width < 100 && area.height < 23 {
            let mut pad = primary;
            if selected && focused {
                pad = pad.add_modifier(termrock::Modifier::BOLD);
            }
            ui.paint_str(Rect::new(area.x.saturating_add(17), y, 1, 1), " ", pad);
        }
    }
    if screen_width < 100 && area.height < 23 {
        paint_nav_seam(ui, area);
    }
}

fn nav_fade_steps(index: usize, visible: usize) -> u8 {
    if visible == 0 || visible >= 23 {
        return 0;
    }
    let depth = if visible >= 12 {
        2
    } else if visible >= 4 {
        1
    } else {
        0
    };
    let from_end = visible.saturating_sub(1).saturating_sub(index);
    if from_end == 0 && depth >= 1 {
        if depth == 1 { 1 } else { 2 }
    } else if from_end == 1 && depth >= 2 {
        1
    } else {
        0
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

fn paint_wide_sidebar(ui: &mut Ui<'_>, area: Rect, label: &str) {
    let canvas = shell_compat_style(ui.surface_style());
    ui.fill(area, canvas);
    ui.register_part(NAV, PartRef::of(Part::CONTAINER), area);
    ui.register_control(NAV, nav_list_area(area), termrock::Focusability::Focusable);
    let (shown, _) = legacy_crumb(label);
    let focused = ui.state(NAV).contains(StateFlags::FOCUSED);
    let mut y = area.y;
    let mut previous: Option<&str> = None;
    for (row_label, group) in LEGACY_NAV {
        if previous != Some(*group) {
            if previous.is_some() {
                if y >= area.bottom() {
                    break;
                }
                y = y.saturating_add(1);
            }
            if y >= area.bottom() {
                break;
            }
            paint_wide_group(ui, area, y, group);
            y = y.saturating_add(1);
            previous = Some(*group);
        }
        if y >= area.bottom() {
            break;
        }
        paint_wide_item(ui, area, y, row_label, *row_label == shown, focused);
        y = y.saturating_add(1);
    }
}

fn paint_wide_group(ui: &mut Ui<'_>, area: Rect, y: u16, group: &str) {
    let pad = shell_text_style(ui, termrock::FgStep::Primary);
    let faint = chrome_fg(
        ui,
        termrock::theme::baseline_faint(ui.theme().capability.color),
    );
    ui.paint_str(Rect::new(area.x, y, 3, 1), "   ", pad);
    ui.paint_str(
        Rect::new(area.x.saturating_add(3), y, width(group), 1),
        group,
        faint,
    );
}

fn paint_wide_item(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    row_label: &str,
    selected: bool,
    focused: bool,
) {
    let secondary = shell_text_style(ui, termrock::FgStep::Secondary);
    let primary = shell_text_style(ui, termrock::FgStep::Primary);
    let chosen = primary.add_modifier(termrock::Modifier::BOLD);
    let gutter = shell_compat_style(ui.surface_style().fg(ui.theme().color.accent))
        .add_modifier(termrock::Modifier::BOLD);
    let black = if ui.theme().capability.color == termrock::ColorLevel::TrueColor
        || ui.theme().capability.color == termrock::ColorLevel::Ansi256
    {
        shell_compat_style(ui.surface_style())
            .fg(termrock::Color::from_u32(0))
            .bg(termrock::Color::from_u32(0))
    } else {
        shell_compat_style(ui.surface_style())
            .fg(termrock::Color::Indexed(0))
            .bg(termrock::Color::Indexed(0))
    };
    let lane_right = area
        .x
        .saturating_add(if selected && focused { 24 } else { 23 });
    if selected && focused {
        ui.paint_str(Rect::new(area.x, y, 2, 1), "▎›", gutter);
        ui.paint_str(
            Rect::new(
                area.x.saturating_add(2),
                y,
                width(row_label).saturating_add(1),
                1,
            ),
            &format!(" {row_label}"),
            chosen,
        );
        let tail = area.x.saturating_add(3).saturating_add(width(row_label));
        if lane_right > tail {
            ui.paint_str(
                Rect::new(tail, y, lane_right.saturating_sub(tail), 1),
                &" ".repeat(usize::from(lane_right.saturating_sub(tail))),
                chosen,
            );
        }
    } else if selected {
        ui.paint_str(Rect::new(area.x, y, 1, 1), " ", black);
        ui.paint_str(Rect::new(area.x.saturating_add(1), y, 1, 1), " ", primary);
        ui.paint_str(
            Rect::new(area.x.saturating_add(3), y, width(row_label), 1),
            row_label,
            chosen,
        );
    } else {
        ui.paint_str(Rect::new(area.x, y, 1, 1), " ", black);
        ui.paint_str(Rect::new(area.x.saturating_add(1), y, 2, 1), "  ", primary);
        ui.paint_str(
            Rect::new(area.x.saturating_add(3), y, width(row_label), 1),
            row_label,
            secondary,
        );
        let tail = area.x.saturating_add(3).saturating_add(width(row_label));
        if lane_right > tail {
            ui.paint_str(
                Rect::new(tail, y, lane_right.saturating_sub(tail), 1),
                &" ".repeat(usize::from(lane_right.saturating_sub(tail))),
                secondary,
            );
        }
    }
}

fn paint_nav_seam(ui: &mut Ui<'_>, area: Rect) {
    let canvas = shell_compat_style(ui.surface_style());
    let level = ui.theme().capability.color;
    let (thumb_fg, track_fg) = match level {
        termrock::ColorLevel::TrueColor => (
            termrock::Color::from_u32(0x00ff_ffff),
            termrock::Color::from_u32(0x0026_2626),
        ),
        termrock::ColorLevel::Ansi256 => {
            (termrock::Color::Indexed(231), termrock::Color::Indexed(235))
        }
        termrock::ColorLevel::Ansi16 | termrock::ColorLevel::Mono => {
            (termrock::Color::White, termrock::Color::DarkGray)
        }
        _ => (termrock::Color::White, termrock::Color::DarkGray),
    };
    let bg = if matches!(
        level,
        termrock::ColorLevel::TrueColor | termrock::ColorLevel::Ansi256
    ) {
        termrock::Color::from_u32(0)
    } else {
        termrock::Color::Indexed(0)
    };
    let thumb = canvas.fg(thumb_fg).bg(bg);
    let track = canvas.fg(track_fg).bg(bg);
    let track_len = usize::from(area.height);
    let thumb_len = (track_len.saturating_mul(track_len) / 23)
        .max(1)
        .min(track_len);
    let x = area.x.saturating_add(18);
    for row in 0..area.height {
        let y = area.y.saturating_add(row);
        let on_thumb = usize::from(row) < thumb_len;
        ui.paint_str(
            Rect::new(x, y, 1, 1),
            if on_thumb { "┃" } else { "│" },
            if on_thumb { thumb } else { track },
        );
    }
}

fn chrome_fg(ui: &Ui<'_>, color: termrock::Color) -> PaintStyle {
    shell_compat_style(ui.surface_style().fg(color))
}

fn paint_sidebar_row(ui: &mut Ui<'_>, row: Rect, flags: StateFlags, _key: ItemKey, node: &NavNode) {
    if row.is_empty() {
        return;
    }
    let focused = flags.contains(StateFlags::FOCUSED);
    let selected = flags.contains(StateFlags::SELECTED);
    let plane = sidebar_plane(ui);
    let accent_bg = tag_accent_bg(ui.theme());
    let row_bg = if selected {
        accent_bg
    } else {
        plane.as_style().bg.unwrap_or(accent_bg)
    };
    let mut row_style = plane.bg(row_bg);
    if focused {
        row_style = row_style.add_modifier(termrock::Modifier::BOLD);
    }
    ui.fill(row, row_style);
    let mut label = shell_row_style(
        shell_part_style(ui, termrock::Family::LIST, Part::LABEL, flags),
        flags,
    )
    .bg(row_bg);
    if selected {
        label = label.fg(ui.theme().color.accent);
    }
    let secondary = shell_part_style(ui, termrock::Family::PANEL, Part::DETAIL, flags).bg(row_bg);
    let marker = shell_row_style(
        shell_part_style(
            ui,
            termrock::Family::LIST,
            Part::MARKER,
            if selected {
                flags | StateFlags::SELECTED
            } else {
                flags
            },
        ),
        flags,
    )
    .bg(row_bg);
    if focused {
        let gutter = shell_row_style(
            shell_part_style(ui, termrock::Family::LIST, Part::GUTTER, flags),
            flags,
        )
        .bg(row_bg);
        ui.glyph(Rect::new(row.x, row.y, 1, 1), GlyphRole::FocusBar, gutter);
    } else if let Some(color) = plane.as_style().bg {
        ui.paint_str(Rect::new(row.x, row.y, 1, 1), " ", plane.fg(color));
    }
    let indent = node.depth.saturating_mul(2);
    let mark_x = row.x.saturating_add(1).saturating_add(indent);
    if node.parent {
        let role = if flags.contains(StateFlags::EXPANDED) {
            GlyphRole::Expanded
        } else {
            GlyphRole::Collapsed
        };
        ui.glyph(Rect::new(mark_x, row.y, 1, 1), role, secondary);
    } else if selected {
        ui.glyph(Rect::new(mark_x, row.y, 1, 1), GlyphRole::Chosen, marker);
    }
    let label_x = mark_x.saturating_add(2);
    let room = row.right().saturating_sub(label_x).saturating_sub(1);
    let text = clip_label(node.label, room);
    if room > 0 {
        ui.fill(Rect::new(label_x, row.y, room, 1), label);
    }
    ui.paint_str(
        Rect::new(label_x, row.y, row.right().saturating_sub(label_x), 1),
        &text,
        label,
    );
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

fn paint_inspector(ui: &mut Ui<'_>, area: Rect, app: &App) {
    inspector_panel().draw(ui, area, |ui, inner| {
        let focus = if ui.state(NAV).contains(StateFlags::FOCUSED) {
            "navigation"
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
        Props::new(&rows).draw(ui, inner);
    });
}

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
    nav_focused: bool,
    page_hints: &[(&str, &str)],
    page_editing: bool,
    status: Option<&str>,
) {
    const TAB_NEXT: (&str, &str) = ("Tab", "Next");
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
    let action_style = chrome_fg(
        ui,
        termrock::theme::baseline_muted(ui.theme().capability.color),
    );
    let nav_hints: &[(&str, &str)] = if nav_focused {
        &[
            ("↑ ↓", "Move"),
            ("Enter", "Open"),
            ("Tab", "Into page"),
            ("q", "Quit"),
        ]
    } else {
        &[]
    };

    let tab_next = !nav_focused && !page_editing;
    let mut x = area.x.saturating_add(1);
    let reserved = status.map_or(14, |message| width(message).saturating_add(3));
    let page_hints: &[(&str, &str)] = if nav_focused { &[] } else { page_hints };
    for &(key, action) in nav_hints.iter().chain(page_hints.iter()) {
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
    if tab_next {
        paint_hint(
            ui,
            area,
            &mut x,
            reserved,
            key_style,
            action_style,
            TAB_NEXT.0,
            TAB_NEXT.1,
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
        self.update_header(cx, &mut response);
        match command {
            Some(QUIT | QUIT_CTRL) => {
                self.quit = true;
                cx.quit();
            }
            Some(NEXT_PAGE) => {
                let count = NAV_ENTRIES.len();
                if count > 0 {
                    self.goto_entry(self.nav_index.saturating_add(1) % count);
                }
            }
            Some(PREV_PAGE) => {
                let count = NAV_ENTRIES.len();
                if count > 0 {
                    let previous = self
                        .nav_index
                        .checked_sub(1)
                        .unwrap_or(count.saturating_sub(1));
                    self.goto_entry(previous);
                }
            }
            Some(HELP_COMMAND) if !cx.is_open(HELP) => {
                cx.open_layer(HELP, Self::help_dialog().layer(cx));
            }
            Some(INSPECTOR_COMMAND) => {
                self.inspector = !self.inspector;
            }
            _ => {}
        }
        if self.show_live_page()
            && let Some(action) = command
            && !matches!(
                action,
                QUIT | QUIT_CTRL | NEXT_PAGE | PREV_PAGE | HELP_COMMAND | INSPECTOR_COMMAND
            )
            && let Some(active) = self.pages.get_mut(self.page.index())
        {
            response |= active.command(cx, action);
        }
        if self.nav_label() == "Selection" {
            response |= self.selectable.update(cx);
        } else {
            self.selectable.blur();
        }
        let full = cx.viewport();
        let shell = self.resolved_shell_layout(full);
        let mut nav_navigated = false;
        for intent in cx.intents(NAV) {
            match intent {
                Intent::Key(key) if cx.state(NAV).contains(StateFlags::FOCUSED) => match key.code {
                    KeyCode::Down => {
                        let cur_index = LEGACY_NAV
                            .iter()
                            .position(|(label, _)| page_for_legacy_label(label) == Some(self.page))
                            .unwrap_or(0);
                        let next = (cur_index + 1).min(LEGACY_NAV.len().saturating_sub(1));
                        if let Some(page) = page_for_legacy_label(LEGACY_NAV[next].0) {
                            self.goto_page(page);
                            nav_navigated = true;
                            response = response.repaint();
                        }
                    }
                    KeyCode::Up => {
                        let cur_index = LEGACY_NAV
                            .iter()
                            .position(|(label, _)| page_for_legacy_label(label) == Some(self.page))
                            .unwrap_or(0);
                        let prev = cur_index.saturating_sub(1);
                        if let Some(page) = page_for_legacy_label(LEGACY_NAV[prev].0) {
                            self.goto_page(page);
                            nav_navigated = true;
                            response = response.repaint();
                        }
                    }
                    KeyCode::Enter => {
                        if self.show_live_page() {
                            cx.focus_next();
                        }
                        response = response.repaint();
                    }
                    _ => {}
                },
                Intent::Pointer {
                    phase: Phase::Release,
                    pos,
                    ..
                } if pos.x >= shell.sidebar.x
                    && pos.x < shell.sidebar.right()
                    && pos.y >= shell.sidebar.y
                    && pos.y < shell.sidebar.bottom() =>
                {
                    let label = if full.width >= 110 {
                        legacy_row_at_y(pos.y, shell.sidebar.y, shell.sidebar.height)
                    } else {
                        let idx = pos.y.saturating_sub(shell.sidebar.y) as usize;
                        LEGACY_NAV.get(idx).map(|(l, _)| *l)
                    };
                    if let Some(label) = label
                        && let Some(page) = page_for_legacy_label(label)
                    {
                        self.goto_page(page);
                        nav_navigated = true;
                        response = response.repaint();
                    }
                }
                _ => {}
            }
        }
        if !nav_navigated {
            let nodes = nav_nodes();
            let mut picked = None;
            response |= nav_tree()
                .update(cx, &mut self.nav_state, &nodes)
                .on_action(|action| {
                    let (key, activate) = match action {
                        TreeAction::Chose(key) => (key, false),
                        TreeAction::Activated(key) => (key, true),
                        TreeAction::Moved | TreeAction::Expanded(_) | TreeAction::Collapsed(_) => {
                            return;
                        }
                    };
                    picked = entry_for_key(key, &nodes).map(|index| (index, activate));
                });
            if let Some((index, activate)) = picked {
                self.goto_entry(index);
                if activate && self.show_live_page() {
                    cx.focus_next();
                }
            }
        }

        if self.show_live_page()
            && !(cx.update_cause() == termrock::UpdateCause::Tick
                && (cx.is_open(HELP) || self.motion_paused))
            && let Some(active) = self.pages.get_mut(self.page.index())
        {
            let update = active.update(cx);
            response |= update.response;
            if let Some(status) = update.status {
                self.status = Some((status, cx.now()));
                response = response.repaint();
            }
        }
        self.update_help(cx, &mut response);
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
        let entry = NAV_ENTRIES.get(self.nav_index).copied();
        let section = entry.map(|entry| entry.section).unwrap_or("");
        let label = entry.map(|entry| entry.label).unwrap_or("Overview");
        paint_sidebar(ui, shell.sidebar, &self.nav_state, label, full.width);
        paint_header(
            ui,
            shell.header,
            full.width,
            full.height,
            section,
            label,
            self.inspector,
        );
        if self.show_live_page() {
            if let Some(active) = self.active() {
                active.draw(ui, shell.main);
            } else {
                missing_page_panel().draw(ui, shell.main, |ui, area| {
                    let _ = ui.paint_str(area, "No page selected", ui.surface_style());
                });
            }
        } else if label == "Selection" {
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
        } else if label == "Selection" {
            (self.selectable.hints(), false)
        } else {
            (&[][..], false)
        };
        paint_footer(
            ui,
            shell.footer,
            ui.state(NAV).contains(StateFlags::FOCUSED),
            page_hints,
            page_editing,
            self.status.as_ref().map(|(status, _)| status.0.as_str()),
        );
        ui.layer(HELP, |ui, area| {
            Self::help_dialog().draw(ui, area, &self.help_state, |ui, body| {
                let _ = ui.paint_str(body, "q quit   ? help   Esc close", ui.surface_style());
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

#[cfg(test)]
mod paint_contract_tests {
    use super::*;
    use termrock::{Color, Family, Modifier, Role, StylePatch, Surface};

    fn collision_style(role: Role) -> PaintStyle {
        let mut theme = Theme::junie();
        theme.color.accent = Color::Rgb(100, 100, 100);
        theme.color.danger = theme.color.accent;
        let family = Family::custom("showcase.paint-contract");
        theme
            .define_family(family, |family| {
                family
                    .part(Part::LABEL)
                    .base(StylePatch::new().set_fg(role));
            })
            .resolve(
                family,
                Variant::DEFAULT,
                Part::LABEL,
                StateFlags::empty(),
                Surface::Canvas,
            )
            .style
    }

    #[test]
    fn shell_modifier_cleanup_preserves_colliding_semantic_roles() {
        let accent = collision_style(Role::Accent).add_modifier(Modifier::BOLD | Modifier::ITALIC);
        let danger = collision_style(Role::Danger).add_modifier(Modifier::BOLD | Modifier::ITALIC);
        assert_eq!(accent.as_style(), danger.as_style());
        let accent = shell_compat_style(accent);
        let danger = shell_compat_style(danger);
        assert_ne!(accent, danger, "equal RGB must not erase semantic identity");
        assert_eq!(accent.add_modifier, Modifier::BOLD);
        assert_eq!(accent.fg, Some(Color::Rgb(100, 100, 100)));
        assert_eq!(accent.as_style(), danger.as_style());
    }
}

#[cfg(test)]
mod app_tests {
    use super::*;

    #[test]
    fn application_commands_are_distinct_from_library_custom_keys() {
        let owned = [
            QUIT,
            QUIT_CTRL,
            HELP_COMMAND,
            INSPECTOR_COMMAND,
            NEXT_PAGE,
            PREV_PAGE,
            FORM_SUBMIT,
            RUN_COMMAND,
        ]
        .into_iter()
        .chain(showcase_demos::pages::pickers::action_keys())
        .collect::<Vec<_>>();
        let names = [
            "showcase.quit",
            "showcase.quit.ctrl",
            "showcase.help",
            "showcase.inspector",
            "showcase.page.next",
            "showcase.page.previous",
            "showcase.form.submit",
            "showcase.taskrunner.run",
            "showcase.menu.open",
            "showcase.menu.close",
            "showcase.context.inspect",
            "showcase.context.copy",
        ];
        assert_eq!(owned.len(), names.len());
        for (key, name) in owned.iter().zip(names) {
            assert_eq!(
                *key,
                ActionKey::application(name),
                "application owner: {name}"
            );
            assert_ne!(*key, ActionKey::custom(name), "library namespace: {name}");
            assert_eq!(
                owned.iter().filter(|other| *other == key).count(),
                1,
                "distinct command: {name}"
            );
        }
    }

    #[test]
    fn showcase_motion_flags_parse_and_bound_paused_frames() {
        let parsed = parse_args(
            ["--page", "progress", "--motion", "paused", "--frame", "80"]
                .into_iter()
                .map(str::to_owned),
        );
        assert!(parsed.is_ok(), "valid frame options: {parsed:?}");
        if let Ok((nav, _, paused, frame)) = parsed {
            assert_eq!(NAV_ENTRIES[nav].id, PageId::Progress);
            assert_eq!(NAV_ENTRIES[nav].label, "Progress");
            assert!(paused);
            assert_eq!(frame, 80);
        }

        let parsed = parse_args(["--frame", "10000"].into_iter().map(str::to_owned));
        assert!(parsed.is_ok(), "maximum frame is valid: {parsed:?}");
        if let Ok((_, _, paused, frame)) = parsed {
            assert!(paused);
            assert_eq!(frame, 10_000);
        }
        assert!(parse_args(["--frame", "10001"].into_iter().map(str::to_owned)).is_err());
        assert!(parse_args(["--motion"].into_iter().map(str::to_owned)).is_err());
        assert!(parse_args(["--motion", "reduced"].into_iter().map(str::to_owned)).is_err());
        assert!(
            parse_args(
                ["--motion", "full", "--frame", "80"]
                    .into_iter()
                    .map(str::to_owned)
            )
            .is_err()
        );
        assert!(
            parse_args(
                ["--frame", "80", "--motion", "full"]
                    .into_iter()
                    .map(str::to_owned)
            )
            .is_err()
        );
    }

    #[test]
    fn paused_clock_step_advances_frame_and_status_by_1100_ms() {
        let mut paused = App::with_page_motion(PageId::Progress, true, 0);
        let frame = paused.paused_frame;
        paused.status = Some((PageStatus("working".to_owned()), Moment::from_millis(5_000)));
        assert!(TuiApp::accept_paused_clock_step(&mut paused, 1_100));
        assert_eq!(paused.paused_frame, frame + 1_100);
        let since = paused
            .status
            .as_ref()
            .map(|(_, since)| since.as_duration().as_millis());
        assert_eq!(since, Some(3_900));

        let mut running = App::with_page_motion(PageId::Progress, false, 0);
        let frame = running.paused_frame;
        running.status = Some((PageStatus("working".to_owned()), Moment::from_millis(5_000)));
        assert!(!TuiApp::accept_paused_clock_step(&mut running, 1_100));
        assert_eq!(running.paused_frame, frame);
        let since = running
            .status
            .as_ref()
            .map(|(_, since)| since.as_duration().as_millis());
        assert_eq!(since, Some(5_000));
    }
}

#[cfg(test)]
mod shell_layout_contract {
    use super::*;

    fn legacy_shell_layout(area: Rect, inspector: bool) -> ShellLayout {
        let regions = termrock::AppShell::new()
            .header(1)
            .gap(1)
            .footer(1)
            .regions(area);
        let header = regions.header;
        let footer = regions.footer;
        let body = regions.body;
        let sidebar_span: u16 = if area.width >= 110 { 24 } else { 19 };
        let inspector_width = if inspector && area.width >= 100 {
            30
        } else {
            0
        };
        let sidebar = Rect::new(body.x, body.y, sidebar_span.saturating_add(1), body.height);
        let main_x = body.x.saturating_add(sidebar_span).saturating_add(2);
        let main_width = body.width.saturating_sub(
            sidebar_span
                .saturating_add(2)
                .saturating_add(inspector_width)
                .saturating_add(u16::from(inspector_width > 0).saturating_mul(2)),
        );
        let main = Rect::new(main_x, body.y, main_width, body.height);
        let inspector = (inspector_width > 0).then(|| {
            Rect::new(
                main.right().saturating_add(2),
                body.y,
                inspector_width,
                body.height,
            )
        });
        ShellLayout {
            header,
            sidebar,
            main,
            inspector,
            footer,
        }
    }

    #[test]
    fn shared_layout_keeps_all_visible_regions_at_width_boundaries() {
        for (x, y) in [(0, 0), (5, 3)] {
            for width in [
                72, 73, 79, 80, 99, 100, 101, 109, 110, 111, 119, 120, 121, 160,
            ] {
                for height in [20, 24, 40] {
                    for inspector in [false, true] {
                        let area = Rect::new(x, y, width, height);
                        let old = legacy_shell_layout(area, inspector);
                        let new = shell_layout(area, inspector);
                        assert_eq!(new.header, old.header, "header {area:?} {inspector}");
                        assert_eq!(new.sidebar, old.sidebar, "sidebar {area:?} {inspector}");
                        assert_eq!(new.main, old.main, "main {area:?} {inspector}");
                        assert_eq!(
                            new.inspector, old.inspector,
                            "inspector {area:?} {inspector}"
                        );
                        assert_eq!(new.footer, old.footer, "footer {area:?} {inspector}");
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod shell_cache_contract {
    use super::*;

    #[test]
    fn layout_cache_tracks_area_and_inspector_changes() {
        let mut app = App::new();
        let compact = Rect::new(0, 0, 80, 24);
        let wide = Rect::new(5, 3, 120, 40);
        assert!(app.resolved_shell_layout(compact).inspector.is_none());
        assert_eq!(
            app.resolved_shell_layout(compact).main,
            shell_layout(compact, false).main
        );
        app.inspector = true;
        assert_eq!(
            app.resolved_shell_layout(wide).inspector,
            shell_layout(wide, true).inspector
        );
        assert_eq!(
            app.resolved_shell_layout(wide).main,
            shell_layout(wide, true).main
        );
        assert!(app.resolved_shell_layout(compact).inspector.is_none());
        app.inspector = false;
        assert_eq!(
            app.resolved_shell_layout(wide).main,
            shell_layout(wide, false).main
        );
    }
}
