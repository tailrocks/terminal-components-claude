//! Application shell for the migrated showcase binary.

use tui_next::{
    ActionKey, App as TuiApp, Chord, ColorLevel, Cx, Dialog, DialogAction, DialogState, Id,
    ItemKey, KeyCode, KeyMap, KeyPhase, NavList, NavListAction, NavListState, Panel, PanelKind,
    Response, Size, Status, StatusBar, StatusItem, Theme, TooSmall, Ui, id, layout,
};

use crate::pages::forms::SUBMIT as FORM_SUBMIT;
use crate::pages::taskrunner::RUN_COMMAND;
use crate::pages::{
    Page, buttons::ButtonsPage, chips::ChipsPage, chrome::ChromePage, dialogs::DialogsPage,
    editable::EditablePage, editor::EditorPage, forms::FormsPage, grid::GridPage,
    inputs::InputsPage, lists::ListsPage, overview::OverviewPage, panels::PanelsPage,
    pickers::PickersPage, progress::ProgressPage, scrolling::ScrollingPage, settings::SettingsPage,
    sidebars::SidebarsPage, tables::TablesPage, taskrunner::TaskRunnerPage, terminal::TerminalPage,
    textareas::TextAreasPage, trees::TreesPage,
};

const NAV: Id = id!("navigation");
const BRAND: Id = id!("brand");
const STATUS: Id = id!("status");
const HELP: Id = id!("help");
const TOO_SMALL: Id = id!("too-small");
const QUIT: ActionKey = ActionKey::custom("showcase.quit");
const QUIT_CTRL: ActionKey = ActionKey::custom("showcase.quit.ctrl");
const HELP_COMMAND: ActionKey = ActionKey::custom("showcase.help");
const NEXT_PAGE: ActionKey = ActionKey::custom("showcase.page.next");
const PREV_PAGE: ActionKey = ActionKey::custom("showcase.page.previous");

const STATUS_LEFT: [StatusItem<'static>; 1] = [StatusItem::new("showcase")];
const STATUS_RIGHT: [StatusItem<'static>; 2] = [
    StatusItem::new("q quit").priority(10),
    StatusItem::new("? help").priority(5),
];

/// Stable page identity used by command-line selection and tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageId {
    /// Introductory page.
    Overview,
    /// Button variants.
    Buttons,
    /// Single-line input.
    Inputs,
    /// Multiline input.
    TextAreas,
    /// Form composition.
    Forms,
    /// Scrollable list.
    Lists,
    /// Hierarchical tree.
    Trees,
    /// Read-only table.
    Tables,
    /// Editable table.
    Editable,
    /// Panel containers.
    Panels,
    /// Sidebar navigation.
    Sidebars,
    /// Dialogs and layers.
    Dialogs,
    /// Progress indicators.
    Progress,
    /// Scrolling content.
    Scrolling,
    /// Terminal output.
    Terminal,
    /// Code editor preview.
    Editor,
    /// Grid preview.
    Grid,
    /// Chips and selectors.
    Chips,
    /// Picker controls.
    Pickers,
    /// Application chrome.
    Chrome,
    /// Settings controls.
    Settings,
    /// Animated task runner.
    TaskRunner,
}

impl PageId {
    /// Every page in navigation order.
    pub const ALL: [Self; 22] = [
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
    ];

    /// Human-readable title.
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
            Self::Editor => "Editor",
            Self::Grid => "Data grid",
            Self::Chips => "Chips & selects",
            Self::Pickers => "Pickers",
            Self::Chrome => "Chrome",
            Self::Settings => "Settings",
            Self::TaskRunner => "Task runner",
        }
    }

    /// Stable command-line spelling.
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
        }
    }

    /// Position in the stable navigation order.
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|page| *page == self).unwrap_or(0)
    }

    /// Parse a page slug or title without panicking.
    pub fn from_name(value: &str) -> Option<Self> {
        let normalized = |input: &str| {
            input
                .chars()
                .filter(char::is_ascii_alphanumeric)
                .map(|character| character.to_ascii_lowercase())
                .collect::<String>()
        };
        let value = normalized(value);
        Self::ALL
            .into_iter()
            .find(|page| normalized(page.slug()) == value || normalized(page.title()) == value)
    }

    /// Parse the keyed navigation value.
    pub fn from_key(key: ItemKey) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|page| ItemKey::text(page.slug()) == key)
    }
}

/// A sidebar item. The app owns these values; `NavList` only borrows them per phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NavEntry {
    /// Destination.
    pub id: PageId,
    /// Display label.
    pub label: &'static str,
    /// Visual section heading.
    pub section: &'static str,
    /// Stable navigation glyph.
    pub icon: &'static str,
}

/// The complete migrated navigation surface.
pub const NAV_ENTRIES: &[NavEntry] = &[
    NavEntry {
        id: PageId::Overview,
        label: "Overview",
        section: "Foundations",
        icon: "•",
    },
    NavEntry {
        id: PageId::Buttons,
        label: "Buttons",
        section: "Components",
        icon: "•",
    },
    NavEntry {
        id: PageId::Inputs,
        label: "Inputs",
        section: "Components",
        icon: "•",
    },
    NavEntry {
        id: PageId::TextAreas,
        label: "Text areas",
        section: "Components",
        icon: "•",
    },
    NavEntry {
        id: PageId::Forms,
        label: "Forms",
        section: "Components",
        icon: "•",
    },
    NavEntry {
        id: PageId::Lists,
        label: "Lists",
        section: "Components",
        icon: "•",
    },
    NavEntry {
        id: PageId::Trees,
        label: "Trees",
        section: "Components",
        icon: "•",
    },
    NavEntry {
        id: PageId::Tables,
        label: "Tables",
        section: "Components",
        icon: "•",
    },
    NavEntry {
        id: PageId::Editable,
        label: "Editable tables",
        section: "Components",
        icon: "•",
    },
    NavEntry {
        id: PageId::Panels,
        label: "Panels",
        section: "Components",
        icon: "•",
    },
    NavEntry {
        id: PageId::Sidebars,
        label: "Sidebars",
        section: "Components",
        icon: "•",
    },
    NavEntry {
        id: PageId::Dialogs,
        label: "Dialogs",
        section: "Components",
        icon: "•",
    },
    NavEntry {
        id: PageId::Progress,
        label: "Progress",
        section: "Components",
        icon: "•",
    },
    NavEntry {
        id: PageId::Scrolling,
        label: "Scrolling",
        section: "Components",
        icon: "•",
    },
    NavEntry {
        id: PageId::Terminal,
        label: "Terminal",
        section: "Components",
        icon: "•",
    },
    NavEntry {
        id: PageId::Editor,
        label: "Code editor",
        section: "Components",
        icon: "•",
    },
    NavEntry {
        id: PageId::Grid,
        label: "Data grid",
        section: "Components",
        icon: "•",
    },
    NavEntry {
        id: PageId::Chips,
        label: "Chips & selects",
        section: "Components",
        icon: "•",
    },
    NavEntry {
        id: PageId::Pickers,
        label: "Pickers",
        section: "Components",
        icon: "•",
    },
    NavEntry {
        id: PageId::Chrome,
        label: "Chrome",
        section: "Components",
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
];

fn nav_key(entry: &NavEntry) -> ItemKey {
    ItemKey::text(entry.id.slug())
}

fn nav_section(entry: &NavEntry) -> &str {
    entry.section
}

fn nav_row(entry: &NavEntry, row: &mut tui_next::RowUi<'_>) {
    row.label(entry.label);
}

fn nav_icon(entry: &NavEntry) -> &str {
    entry.icon
}

fn nav() -> NavList<
    'static,
    NavEntry,
    impl Fn(&NavEntry) -> ItemKey,
    impl Fn(&NavEntry, &mut tui_next::RowUi<'_>),
> {
    NavList::new(NAV)
        .key(nav_key)
        .section(&nav_section)
        .icon(&nav_icon)
        .row(nav_row)
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
    }
}

fn keymap() -> KeyMap {
    KeyMap::new()
        .bind(KeyPhase::Bubble, Chord::key(KeyCode::Char('q')), QUIT)
        // Capture keeps the global interrupt available while a text control
        // owns printable-key handling.
        .bind(
            KeyPhase::Capture,
            Chord::with(KeyCode::Char('c'), tui_next::KeyModifiers::CONTROL),
            QUIT_CTRL,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('c'), tui_next::KeyModifiers::CONTROL),
            QUIT_CTRL,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('?')),
            HELP_COMMAND,
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
            Chord::with(KeyCode::Char('s'), tui_next::KeyModifiers::CONTROL),
            FORM_SUBMIT,
        )
}

/// The complete showcase app state.
pub struct App {
    page: PageId,
    nav_state: NavListState,
    pages: Vec<Box<dyn Page>>,
    help_state: DialogState,
    keymap: KeyMap,
    quit: bool,
}

impl core::fmt::Debug for App {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("App")
            .field("page", &self.page)
            .field("nav_state", &self.nav_state)
            .field("pages", &self.pages.len())
            .field("help_state", &self.help_state)
            .field("keymap", &self.keymap)
            .field("quit", &self.quit)
            .finish()
    }
}

impl App {
    /// Construct the overview page.
    pub fn new() -> Self {
        Self::with_page(PageId::Overview)
    }

    /// Construct with a selected initial page.
    pub fn with_page(initial: PageId) -> Self {
        let mut nav_state = NavListState::new();
        nav_state.set_current(Some(ItemKey::text(initial.slug())));
        let pages = PageId::ALL.into_iter().map(|kind| page(kind)).collect();
        Self {
            page: initial,
            nav_state,
            pages,
            help_state: DialogState::default(),
            keymap: keymap(),
            quit: false,
        }
    }

    /// Current page.
    pub const fn page(&self) -> PageId {
        self.page
    }

    /// Whether quit was requested.
    pub const fn quit(&self) -> bool {
        self.quit
    }

    fn goto(&mut self, page: PageId) {
        self.page = page;
        self.nav_state.set_current(Some(ItemKey::text(page.slug())));
    }

    fn active(&self) -> Option<&dyn Page> {
        self.pages
            .iter()
            .find(|candidate| candidate.title() == self.page.title())
            .map(Box::as_ref)
    }

    fn help_dialog() -> Dialog<'static> {
        Dialog::info(HELP, "Showcase help")
            .description("Tab moves through controls. Enter activates. Esc closes layers or returns to Overview.")
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
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl TuiApp for App {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        let command = cx.command();
        match command {
            Some(QUIT | QUIT_CTRL) => {
                self.quit = true;
                cx.quit();
            }
            Some(NEXT_PAGE) => {
                let next = self
                    .page
                    .index()
                    .checked_add(1)
                    .and_then(|index| PageId::ALL.get(index).copied())
                    .or_else(|| PageId::ALL.first().copied());
                if let Some(page) = next {
                    self.goto(page);
                }
            }
            Some(PREV_PAGE) => {
                let previous = self
                    .page
                    .index()
                    .checked_sub(1)
                    .and_then(|index| PageId::ALL.get(index).copied())
                    .or_else(|| PageId::ALL.last().copied());
                if let Some(page) = previous {
                    self.goto(page);
                }
            }
            Some(HELP_COMMAND) if !cx.is_open(HELP) => {
                cx.open_layer(HELP, Self::help_dialog().layer(cx));
            }
            _ => {}
        }
        if let Some(action) = command
            && !matches!(
                action,
                QUIT | QUIT_CTRL | NEXT_PAGE | PREV_PAGE | HELP_COMMAND
            )
            && let Some(active) = self
                .pages
                .iter_mut()
                .find(|candidate| candidate.title() == self.page.title())
        {
            response |= active.command(cx, action);
        }
        response |= nav()
            .update(cx, &mut self.nav_state, NAV_ENTRIES)
            .on_action(|action| {
                if let NavListAction::Chose(key) | NavListAction::EnterContent(key) = action
                    && let Some(page) = PageId::from_key(key)
                {
                    self.goto(page);
                }
            });
        let title = self.page.title();
        if let Some(active) = self
            .pages
            .iter_mut()
            .find(|candidate| candidate.title() == title)
        {
            response |= active.update(cx);
        }
        self.update_help(cx, &mut response);
        response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let full = ui.full();
        if full.width < 72 || full.height < 20 {
            TooSmall::new(TOO_SMALL, "showcase")
                .minimum(72, 20)
                .draw(ui, full);
            return;
        }
        let (without_status, status_area) = layout::split_v(full, full.height.saturating_sub(1));
        let (header, body) = layout::split_v(without_status, 3);
        let (sidebar, content) = layout::split_h(body, 24);
        tui_next::Brand::new(BRAND, "SHOWCASE")
            .tagline("public tui-next API")
            .draw(ui, header);
        nav().draw(ui, sidebar, &self.nav_state, NAV_ENTRIES);
        if let Some(active) = self.active() {
            active.draw(ui, content);
        } else {
            Panel::new(BRAND)
                .kind(PanelKind::Framed)
                .title("Missing page")
                .draw(ui, content, |ui, area| {
                    let _ = ui.paint_str(area, "No page selected", ui.surface_style());
                });
        }
        StatusBar::new(STATUS)
            .left(&STATUS_LEFT)
            .right(&STATUS_RIGHT)
            .status(Status::Ready)
            .draw(ui, status_area);
        ui.layer(HELP, |ui, area| {
            Self::help_dialog().draw(ui, area, &self.help_state, |ui, body| {
                let _ = ui.paint_str(body, "q quit   ? help   Esc close", ui.surface_style());
            });
        });
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

    fn on_esc(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        if self.page == PageId::Overview {
            self.quit = true;
            Response::changed()
        } else {
            self.goto(PageId::Overview);
            Response::changed()
        }
    }
}

/// Parse CLI options and run the migrated binary.
pub(crate) fn run() -> std::io::Result<()> {
    let mut theme = Theme::junie();
    let mut page = PageId::Overview;
    let mut args = std::env::args().skip(1);
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
                    && let Some(selected) = PageId::from_name(&value)
                {
                    page = selected;
                }
            }
            _ => {}
        }
    }
    tui_next::run(App::with_page(page), theme)
}
