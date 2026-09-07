//! Application shell for the migrated showcase binary.

use junie_tui::{
    ActionKey, App as TuiApp, Brand, Chord, ColorLevel, Cx, Dialog, DialogAction, DialogState,
    FrameRead, Id, Intent, ItemKey, KeyCode, KeyMap, KeyPhase, NavList, NavListAction,
    NavListState, Panel, PanelKind, Part, PartRef, Phase, Props, Rect, Response, Size, StateFlags,
    Status, StatusBar, StatusItem, Style, Theme, TooSmall, Ui, Variant, id, width,
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
const HEADER_HELP: Id = id!("header.help");
const HEADER_INSPECT: Id = id!("header.inspect");
const INSPECTOR: Id = id!("inspector");
const QUIT: ActionKey = ActionKey::custom("showcase.quit");
const QUIT_CTRL: ActionKey = ActionKey::custom("showcase.quit.ctrl");
const HELP_COMMAND: ActionKey = ActionKey::custom("showcase.help");
const INSPECTOR_COMMAND: ActionKey = ActionKey::custom("showcase.inspector");
const NEXT_PAGE: ActionKey = ActionKey::custom("showcase.page.next");
const PREV_PAGE: ActionKey = ActionKey::custom("showcase.page.previous");
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
            Self::Editor => "Code editor",
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

impl std::fmt::Display for NavEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label)
    }
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

fn nav() -> NavList<'static, NavEntry, impl Fn(&NavEntry) -> ItemKey> {
    NavList::new(NAV)
        .key(nav_key)
        .section(&nav_section)
        .compact_when_clipped()
        .header_indent(3)
        .render_row(&paint_nav_row)
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
    }
}

fn keymap() -> KeyMap {
    KeyMap::new()
        .bind(KeyPhase::Bubble, Chord::key(KeyCode::Char('q')), QUIT)
        // Capture keeps the global interrupt available while a text control
        // owns printable-key handling.
        .bind(
            KeyPhase::Capture,
            Chord::with(KeyCode::Char('c'), junie_tui::KeyModifiers::CONTROL),
            QUIT_CTRL,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('c'), junie_tui::KeyModifiers::CONTROL),
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
            Chord::with(KeyCode::Char('s'), junie_tui::KeyModifiers::CONTROL),
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
    inspector: bool,
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
            .field("inspector", &self.inspector)
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
        let initial_key = ItemKey::text(initial.slug());
        nav_state.set_current(Some(initial_key));
        nav_state.set_cursor(initial.index(), initial_key);
        let pages = PageId::ALL.into_iter().map(|kind| page(kind)).collect();
        Self {
            page: initial,
            nav_state,
            pages,
            help_state: DialogState::default(),
            keymap: keymap(),
            inspector: false,
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
        let key = ItemKey::text(page.slug());
        self.nav_state.set_current(Some(key));
        self.nav_state.set_cursor(page.index(), key);
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

fn shell_layout(area: Rect, inspector: bool) -> ShellLayout {
    let header = Rect::new(area.x, area.y, area.width, 1);
    let footer = Rect::new(area.x, area.bottom().saturating_sub(1), area.width, 1);
    let body = Rect::new(
        area.x,
        area.y.saturating_add(2),
        area.width,
        area.height.saturating_sub(4),
    );
    let sidebar_width = if area.width >= 110 { 24 } else { 19 };
    let inspector_width = if inspector && area.width >= 100 {
        30
    } else {
        0
    };
    let sidebar = Rect::new(body.x, body.y, sidebar_width, body.height);
    let main_x = body.x.saturating_add(sidebar_width).saturating_add(2);
    let main_width = body.width.saturating_sub(
        sidebar_width
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

fn shell_part_style(
    ui: &mut Ui<'_>,
    family: junie_tui::Family,
    part: Part,
    flags: StateFlags,
) -> Style {
    let background = ui.bg();
    shell_compat_style(
        ui.style(family, Variant::DEFAULT, part, flags)
            .style
            .bg(background),
    )
}

/// Clear component-first modifiers before the compatibility paint pass.
///
/// `Buffer::set_style` patches a cell, so a public component's bold state is
/// otherwise retained by a later shell style that only changes colours. The
/// historical shell starts from plain cells and adds bold only where its old
/// renderer did.
fn shell_compat_style(style: Style) -> Style {
    let bold = style.add_modifier.contains(junie_tui::Modifier::BOLD);
    let style = style.remove_modifier(junie_tui::Modifier::all());
    if bold {
        style.add_modifier(junie_tui::Modifier::BOLD)
    } else {
        style
    }
}

fn shell_row_style(style: Style, flags: StateFlags) -> Style {
    let style = shell_compat_style(style);
    if flags.contains(StateFlags::FOCUSED) {
        style.add_modifier(junie_tui::Modifier::BOLD)
    } else {
        style
    }
}

fn shell_text_style(ui: &Ui<'_>, step: usize) -> Style {
    shell_compat_style(ui.surface_style().fg(ui.theme().color.fg[step]))
}

fn paint_header(
    ui: &mut Ui<'_>,
    area: Rect,
    screen_width: u16,
    screen_height: u16,
    page: PageId,
    inspector: bool,
) {
    if area.is_empty() {
        return;
    }
    let canvas = shell_compat_style(ui.surface_style());
    ui.fill(area, canvas);

    let (title, secondary, muted, faint, marker) = header_styles(ui);
    let left = paint_header_breadcrumb(ui, area, page, title, secondary, muted, marker);
    paint_header_actions(
        ui,
        area,
        (screen_width, screen_height),
        left,
        (muted, faint),
        inspector,
    );
}

fn header_styles(ui: &mut Ui<'_>) -> (Style, Style, Style, Style, Style) {
    let title = shell_part_style(
        ui,
        junie_tui::Family::PANEL,
        Part::TITLE,
        StateFlags::empty(),
    );
    let secondary = shell_part_style(
        ui,
        junie_tui::Family::PANEL,
        Part::DETAIL,
        StateFlags::empty(),
    );
    let muted = shell_part_style(ui, junie_tui::Family::LIST, Part::META, StateFlags::empty());
    let faint = shell_text_style(ui, 3);
    let marker = shell_part_style(
        ui,
        junie_tui::Family::LIST,
        Part::MARKER,
        StateFlags::SELECTED,
    );
    (title, secondary, muted, faint, marker)
}

fn paint_header_breadcrumb(
    ui: &mut Ui<'_>,
    area: Rect,
    page: PageId,
    title: Style,
    secondary: Style,
    muted: Style,
    marker: Style,
) -> u16 {
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

    let Some(entry) = NAV_ENTRIES
        .get(page.index())
        .copied()
        .or_else(|| NAV_ENTRIES.first().copied())
    else {
        return x;
    };
    let crumb = format!("/ {} / {}", entry.section, entry.label);
    ui.paint_str(
        Rect::new(x, area.y, area.right().saturating_sub(x), 1),
        &crumb,
        muted,
    );
    x.saturating_add(width(&crumb))
}

fn paint_header_actions(
    ui: &mut Ui<'_>,
    area: Rect,
    screen: (u16, u16),
    left: u16,
    styles: (Style, Style),
    inspector: bool,
) {
    let (screen_width, screen_height) = screen;
    let (muted, faint) = styles;
    let capability = ui.theme().capability.color.label();
    let dimensions = format!("{screen_width}×{screen_height}");
    let capability_width = width(capability)
        .saturating_add(3)
        .saturating_add(width(&dimensions));
    let help_text = " ? Help ";
    let inspector_text = if inspector {
        " i Inspector · on "
    } else {
        " i Inspector "
    };
    let help_width = width(help_text);
    let inspector_width = width(inspector_text);
    let mut right = area.right().saturating_sub(1);
    let help_x = right.saturating_sub(help_width);
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
    right = help_x.saturating_sub(1);

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
    if right > left.saturating_add(capability_width) {
        let cap_x = right.saturating_sub(capability_width);
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
        ui.paint_str(
            Rect::new(
                cap_x.saturating_add(width(capability)).saturating_add(3),
                area.y,
                width(&dimensions),
                1,
            ),
            &dimensions,
            faint,
        );
    }
}

fn header_action_style(ui: &mut Ui<'_>, id: Id, muted: Style) -> Style {
    if ui.state(id).contains(StateFlags::HOVERED) {
        shell_part_style(
            ui,
            junie_tui::Family::LIST,
            Part::CONTAINER,
            StateFlags::HOVERED,
        )
    } else {
        muted
    }
}

fn paint_nav_row(ui: &mut Ui<'_>, row: Rect, flags: StateFlags, _key: ItemKey, entry: &NavEntry) {
    let current = flags.contains(StateFlags::SELECTED);
    // Current destination is a marker, not row selection. Keyboard cursor
    // and hover remain independent, as in the pinned product reference.
    let flags = flags.difference(StateFlags::SELECTED);
    let emphasized = current || flags.intersects(StateFlags::FOCUSED | StateFlags::HOVERED);
    let container = shell_compat_style(
        ui.style(
            junie_tui::Family::LIST,
            Variant::DEFAULT,
            Part::CONTAINER,
            flags,
        )
        .style,
    );
    let row_background = container.bg.unwrap_or(ui.bg());
    ui.fill(row, container);
    let gutter = ui
        .style(
            junie_tui::Family::LIST,
            Variant::DEFAULT,
            Part::GUTTER,
            flags,
        )
        .style
        .bg(row_background);
    let gutter = if flags.contains(StateFlags::FOCUSED) {
        gutter
    } else {
        gutter.fg(row_background)
    };
    let marker = ui
        .style(
            junie_tui::Family::LIST,
            Variant::DEFAULT,
            Part::MARKER,
            if current {
                flags | StateFlags::SELECTED
            } else {
                flags
            },
        )
        .style
        .bg(row_background);
    let label = ui
        .style(
            junie_tui::Family::LIST,
            Variant::DEFAULT,
            Part::LABEL,
            flags,
        )
        .style
        .bg(row_background);
    let secondary =
        shell_part_style(ui, junie_tui::Family::PANEL, Part::DETAIL, flags).bg(row_background);
    let gutter = shell_row_style(gutter, flags);
    let marker = shell_row_style(marker, flags);
    let label = shell_row_style(label, flags);
    let secondary = shell_row_style(secondary, flags);
    ui.paint_str(Rect::new(row.x, row.y, 1, 1), "▎", gutter);
    ui.paint_str(
        Rect::new(row.x.saturating_add(1), row.y, 1, 1),
        if current { "›" } else { " " },
        marker,
    );
    ui.paint_str(Rect::new(row.x.saturating_add(2), row.y, 1, 1), " ", label);
    ui.paint_str(
        Rect::new(
            row.x.saturating_add(3),
            row.y,
            row.width.saturating_sub(4),
            1,
        ),
        entry.label,
        if emphasized { label } else { secondary },
    );
    let label_style = if emphasized { label } else { secondary };
    let label_area = Rect::new(
        row.x.saturating_add(3),
        row.y,
        row.width.saturating_sub(4),
        1,
    );
    let used = width(entry.label).min(label_area.width);
    if used < label_area.width {
        ui.fill(
            Rect::new(
                label_area.x.saturating_add(used),
                label_area.y,
                label_area.width.saturating_sub(used),
                1,
            ),
            label_style,
        );
    }
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
            ("page", app.page.title()),
            ("focus", focus),
            ("hover", hover),
            ("pressed", pressed),
            ("colors", ui.theme().capability.color.label()),
        ];
        Props::new(&rows).draw(ui, inner);
    });
}

fn paint_footer(ui: &mut Ui<'_>, area: Rect, nav_focused: bool) {
    if area.is_empty() {
        return;
    }
    let canvas = shell_compat_style(ui.surface_style());
    ui.fill(area, canvas);
    let key_style = shell_part_style(
        ui,
        junie_tui::Family::KEYHINT,
        Part::KEY,
        StateFlags::empty(),
    );
    let action_style = shell_part_style(
        ui,
        junie_tui::Family::KEYHINT,
        Part::ACTION,
        StateFlags::empty(),
    );
    let hints: &[(&str, &str)] = if nav_focused {
        &[
            ("↑ ↓", "Move"),
            ("Enter", "Open"),
            ("Tab", "Into page"),
            ("q", "Quit"),
        ]
    } else {
        &[("Tab", "Next"), ("Esc", "Navigation"), ("q", "Quit")]
    };
    let mut x = area.x.saturating_add(1);
    for (key, action) in hints {
        let key_width = width(key);
        let action_width = width(action);
        ui.paint_str(Rect::new(x, area.y, key_width, 1), key, key_style);
        x = x.saturating_add(key_width.saturating_add(1));
        ui.paint_str(Rect::new(x, area.y, action_width, 1), action, action_style);
        x = x.saturating_add(action_width.saturating_add(2));
        if x >= area.right() {
            break;
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
        if cx.update_cause() == junie_tui::UpdateCause::Bootstrap {
            cx.focus(NAV);
        }
        let mut response = Response::ignored();
        response |= shell_brand().update(cx).erase();
        response |= shell_status().update(cx).erase();
        // These stateless shell props have no update phase of their own, but
        // the same constructors must remain the source of truth in both
        // runtime phases.  Calling them here also keeps the minimum-size and
        // fallback branches construction-safe under the props guard.
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
            Some(INSPECTOR_COMMAND) => {
                self.inspector = !self.inspector;
            }
            _ => {}
        }
        if let Some(action) = command
            && !matches!(
                action,
                QUIT | QUIT_CTRL | NEXT_PAGE | PREV_PAGE | HELP_COMMAND | INSPECTOR_COMMAND
            )
            && let Some(active) = self.pages.get_mut(self.page.index())
        {
            response |= active.command(cx, action);
        }
        response |= nav()
            .update(cx, &mut self.nav_state, NAV_ENTRIES)
            .on_action(|action| match action {
                NavListAction::Chose(key) => {
                    if let Some(page) = PageId::from_key(key) {
                        self.goto(page);
                    }
                }
                NavListAction::EnterContent(key) => {
                    if let Some(page) = PageId::from_key(key) {
                        self.goto(page);
                        cx.focus_next();
                    }
                }
                NavListAction::Moved(_) => {}
            });
        if let Some(active) = self.pages.get_mut(self.page.index()) {
            response |= active.update(cx);
        }
        self.update_help(cx, &mut response);
        response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let full = ui.full();
        // The historical renderer establishes a complete canvas before any
        // component paints. This clears cells that Brand/Nav/Status do not
        // touch and gives the compatibility pass deterministic write state.
        ui.fill(full, shell_compat_style(ui.surface_style()));
        if full.width < 72 || full.height < 20 {
            too_small_notice().draw(ui, full);
            return;
        }
        let shell = shell_layout(full, self.inspector);

        // Navigation owns both row painting and registration. Header/footer
        // compatibility painting remains separate shell migration work.
        shell_brand().draw(ui, shell.header);
        nav().draw(ui, shell.sidebar, &self.nav_state, NAV_ENTRIES);
        shell_status().draw(ui, shell.footer);
        paint_header(
            ui,
            shell.header,
            full.width,
            full.height,
            self.page,
            self.inspector,
        );
        if let Some(active) = self.active() {
            active.draw(ui, shell.main);
        } else {
            missing_page_panel().draw(ui, shell.main, |ui, area| {
                let _ = ui.paint_str(area, "No page selected", ui.surface_style());
            });
        }
        if let Some(inspector) = shell.inspector {
            paint_inspector(ui, inspector, self);
        }
        paint_footer(
            ui,
            shell.footer,
            ui.state(NAV).contains(StateFlags::FOCUSED),
        );
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
    junie_tui::run(App::with_page(page), theme)
}
