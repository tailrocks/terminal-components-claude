use std::{collections::BTreeMap, io::Write};

use crate::{
    dispatch::{self, Destination},
    domain::{
        action::Scope,
        activity::{Activity, ActivityState},
        fixtures,
        ranking::Pin,
    },
    scenario::{Motion, Scenario, Screen},
    screens::{
        actions::{Actions, Choice},
        activity::{self, Activities},
        args::TaskArgsEditor,
        cleanup,
        clone::ClonePicker,
        dialogs::{self, ProductDialog},
        disk, disk_scan, files,
        home::{self, HomeState},
        plan::{self, PlanState},
        plan_gate::PlanGate,
        port_args::PortArgsEditor,
        review, snapshot,
        task_run::{self, TaskRunScreen},
    },
    sim::{
        activity_supervisor::{
            ActivityExecutionId, ActivityExecutionPhase, ActivityExecutionStart,
            ActivityStopOutcome, ActivityStopProfile,
        },
        catalogue, pg,
        task_run::{
            PromptId, TaskRun, TaskRunEvent, TaskRunId, TaskRunState, TaskRunStore, TaskRunUpdate,
            TaskScriptId,
        },
        world::World,
    },
};
use termrock::{
    ActionKey, AppShell, Binding, Brand, Chord, Color, Constraints, Cx, Dialog, FgStep,
    Focusability, FrameRead, Hint, HintBar, HintKey, HintLayer, Id, Intent, ItemKey, KeyCode,
    KeyHint, KeyMap, KeyPhase, LayerOpening, Menu, MenuAction, MenuBar, MenuBarHoverScope,
    MenuItem, MenuState, Part, PartRef, Phase, Rect, Response, Role, StatusBar, StatusItem,
    StylePatch, Surface, TooSmall, Ui,
};

use crate::layout::{self, MIN_HEIGHT, MIN_WIDTH};

const BRAND: Id = Id::root("holla.brand");
const MENU: Id = Id::root("holla.menu");
const STRIP: Id = Id::root("holla.activities");
const FOOTER: Id = Id::root("holla.footer");
const HELP_LAYER: Id = Id::root("holla.help");
const HELP_SCROLL_HINT: Id = Id::root("holla.help.footer.scroll");
const HELP_CLOSE_HINT: Id = Id::root("holla.help.footer.close");
const PARITY_BROWSER_STATUS: Id = Id::root("holla.status.parity-browser");
const PARITY_FILES_STATUS: Id = Id::root("holla.status.parity-files");
const HELP: ActionKey = ActionKey::application("holla.help");
const ABOUT: ActionKey = ActionKey::application("holla.about");
const QUIT: ActionKey = ActionKey::application("holla.quit");
const HOME: ActionKey = ActionKey::application("holla.home");
const NEXT_ACTIVITY: ActionKey = ActionKey::application("holla.next-activity");
const SCOPE: ActionKey = ActionKey::application("holla.scope");
const PREVIEW: ActionKey = ActionKey::application("holla.preview");
const ACTIONS: ActionKey = ActionKey::application("holla.actions");
const TOP_FILES: ActionKey = ActionKey::application("holla.top-files");
const CLEANUP_REVIEW: ActionKey = ActionKey::application("holla.cleanup-review");
const DISK_SELECT: ActionKey = ActionKey::application("holla.disk.select-candidates");
const FOLDER_BROWSER: ActionKey = ActionKey::application("holla.folder-browser");
const FILES_OPEN: ActionKey = ActionKey::application("holla.files.open-directory");
const FILES_PARENT: ActionKey = ActionKey::application("holla.files.parent-directory");
const FILES_HIDDEN: ActionKey = ActionKey::application("holla.files.toggle-hidden");
const DISK_SCAN: ActionKey = ActionKey::application("holla.disk-scan.open");
const HOME_FIND: ActionKey = ActionKey::application("holla.home-find");
const CUSTOM_ACTION: ActionKey = ActionKey::application("holla.custom-action");
const SNAPSHOT_PAGE: ActionKey = ActionKey::application("holla.snapshot-page");
const DEFINITION_TRUST: ActionKey = ActionKey::application("holla.definition-trust");
const TOGGLE: ActionKey = ActionKey::application("holla.plan-toggle");
const CONTINUE: ActionKey = ActionKey::application("holla.plan-continue");
const OPEN_MENU: ActionKey = ActionKey::application("holla.open-menu");
const RESULTS: ActionKey = ActionKey::application("holla.results");
const QUERY_ESCAPE: ActionKey = ActionKey::application("holla.query-escape");
const PREVIOUS: ActionKey = ActionKey::application("holla.previous");
const INTERRUPT: ActionKey = ActionKey::application("holla.interrupt");
const SNAPSHOT_FOCUS_NEXT: ActionKey = ActionKey::application("holla.snapshot.focus-next");
const SNAPSHOT_FOCUS_PREVIOUS: ActionKey = ActionKey::application("holla.snapshot.focus-previous");
const SNAPSHOT_COPY: ActionKey = ActionKey::application("holla.snapshot.copy-resources");
const ACTIVITY_SHORTCUTS: &[(char, ActionKey)] = &[
    ('1', ActionKey::application("holla.activity-1")),
    ('2', ActionKey::application("holla.activity-2")),
    ('3', ActionKey::application("holla.activity-3")),
    ('4', ActionKey::application("holla.activity-4")),
    ('5', ActionKey::application("holla.activity-5")),
    ('6', ActionKey::application("holla.activity-6")),
    ('7', ActionKey::application("holla.activity-7")),
    ('8', ActionKey::application("holla.activity-8")),
    ('9', ActionKey::application("holla.activity-9")),
];
const GLOBAL: &[Binding<ActionKey>] = &[
    Binding {
        action: HELP,
        chord: Some(Chord::key(KeyCode::F(1))),
        cmd: HELP,
        label: "Help",
        priority: 50,
        visible: true,
    },
    Binding {
        action: OPEN_MENU,
        chord: Some(Chord::key(KeyCode::F(10))),
        cmd: OPEN_MENU,
        label: "Menu",
        priority: 40,
        visible: true,
    },
    Binding {
        action: QUIT,
        chord: Some(Chord::with(
            KeyCode::Char('q'),
            termrock::KeyModifiers::CONTROL,
        )),
        cmd: QUIT,
        label: "Quit",
        priority: 20,
        visible: true,
    },
    Binding {
        action: NEXT_ACTIVITY,
        chord: Some(Chord::with(
            KeyCode::Char('a'),
            termrock::KeyModifiers::CONTROL,
        )),
        cmd: NEXT_ACTIVITY,
        label: "Activities",
        priority: 30,
        visible: true,
    },
    Binding {
        action: SCOPE,
        chord: Some(Chord::with(
            KeyCode::Char('s'),
            termrock::KeyModifiers::CONTROL,
        )),
        cmd: SCOPE,
        label: "Scope",
        priority: 40,
        visible: true,
    },
    Binding {
        action: PREVIEW,
        chord: Some(Chord::with(
            KeyCode::Char('p'),
            termrock::KeyModifiers::CONTROL,
        )),
        cmd: PREVIEW,
        label: "Preview",
        priority: 60,
        visible: true,
    },
    Binding {
        action: ACTIONS,
        chord: Some(Chord::with(
            KeyCode::Char('o'),
            termrock::KeyModifiers::CONTROL,
        )),
        cmd: ACTIONS,
        label: "Actions",
        priority: 60,
        visible: true,
    },
];

const HELP_FINDER_ROWS: &[(&str, &str)] = &[
    (
        "Type",
        "search actions and resources together · @parent @children @system @all narrow the scope",
    ),
    ("↑↓", "move · PgUp PgDn Home End"),
    (
        "Enter",
        "primary action · destructive rows open a review instead",
    ),
    (
        "Alt+Enter",
        "alternatives · run, copy, insert, arguments, pin, alias, hide, why",
    ),
    ("Tab", "preview › tab strip › finder"),
    (
        "Ctrl+↑ / Ctrl+↓",
        "scope outward (parent, system) / inward (children)",
    ),
    ("Esc", "clear query › scope back to here › back a page"),
    (
        "Aliases",
        "gp pull · du disk usage · test the test task here · d then u · dc docker cleanup",
    ),
];
const HELP_PAGE_ROWS: &[(&str, &str)] = &[
    (
        "Disk",
        "↑↓ move · Space select · a all · Enter drill · c cleanup plan · p details",
    ),
    (
        "Plan review",
        "Space include or exclude · u undo · Enter step facts · c confirm",
    ),
    (
        "Gates",
        "Gate 1 reviews facts and the full sequence · Gate 2 types the target-bound phrase",
    ),
];
const HELP_ACTIVITY_ROWS: &[(&str, &str)] = &[
    ("↑↓ PgUp PgDn", "scroll · f follow tail · y copy selection"),
    ("s", "stop · r restart · x close tab"),
    (
        "Enter / i",
        "attach to an interactive program · Ctrl+] detaches",
    ),
    ("1–9", "show or hide a merged log stream"),
    ("Esc", "back to Here · the activity keeps running"),
];
const HELP_PLAN_ROWS: &[(&str, &str)] = &[
    ("↑↓", "select a step · Enter maximise its output · f follow"),
    ("r", "retry the failed step · k skip an optional failure"),
    ("Ctrl+C", "cancel the remaining work"),
    ("Esc", "back to Here · the plan keeps running"),
];
const HELP_MOUSE_ROWS: &[(&str, &str)] = &[
    ("Click", "select"),
    ("Double-click", "run"),
    ("Right-click", "alternatives"),
    ("Wheel", "scroll under the pointer"),
];
const HELP_EVERYWHERE_ROWS: &[(&str, &str)] = &[
    ("Alt+0", "Here"),
    ("Ctrl+W", "Close tab"),
    ("Ctrl+G", "Activities"),
    ("F1 ?", "This reference"),
    ("F10 Ctrl+F2", "Menu bar"),
    ("Ctrl+PgDn", "Next page"),
    ("Ctrl+PgUp", "Previous page"),
    ("Tab", "Next control"),
    ("Shift+Tab", "Previous control"),
    ("Esc", "Back"),
    ("Ctrl+Q", "Quit"),
    ("Ctrl+C", "Copy, else quit"),
    ("Alt+1–9", "Show that tab"),
];
static HELP_HOME_SECTIONS: [termrock::HelpSection<'static>; 4] = [
    termrock::HelpSection::plain("Finder", HELP_FINDER_ROWS),
    termrock::HelpSection::plain("Pages", HELP_PAGE_ROWS),
    termrock::HelpSection::plain("Mouse", HELP_MOUSE_ROWS),
    termrock::HelpSection::plain("Everywhere", HELP_EVERYWHERE_ROWS),
];
static HELP_ACTIVITY_SECTIONS: [termrock::HelpSection<'static>; 3] = [
    termrock::HelpSection::plain("Activity", HELP_ACTIVITY_ROWS),
    termrock::HelpSection::plain("Mouse", HELP_MOUSE_ROWS),
    termrock::HelpSection::plain("Everywhere", HELP_EVERYWHERE_ROWS),
];
static HELP_PLAN_SECTIONS: [termrock::HelpSection<'static>; 3] = [
    termrock::HelpSection::plain("Plan", HELP_PLAN_ROWS),
    termrock::HelpSection::plain("Mouse", HELP_MOUSE_ROWS),
    termrock::HelpSection::plain("Everywhere", HELP_EVERYWHERE_ROWS),
];
static HELP_PAGE_SECTIONS: [termrock::HelpSection<'static>; 3] = [
    termrock::HelpSection::plain("Pages", HELP_PAGE_ROWS),
    termrock::HelpSection::plain("Mouse", HELP_MOUSE_ROWS),
    termrock::HelpSection::plain("Everywhere", HELP_EVERYWHERE_ROWS),
];

const HELP_OVERLAY_PARTS: &[(Part, StylePatch)] = &[
    (
        Part::CONTAINER,
        StylePatch::new().set_bg(Role::Custom(Color::Rgb(0x18, 0x18, 0x1b))),
    ),
    (
        Part::BORDER,
        StylePatch::new()
            .set_fg(Role::Custom(Color::Rgb(0x4d, 0x4d, 0x4d)))
            .set_bg(Role::Custom(Color::Rgb(0x18, 0x18, 0x1b)))
            .remove(termrock::Modifier::BOLD),
    ),
    (
        Part::TITLE,
        StylePatch::new()
            .set_fg(Role::Custom(Color::Rgb(0xff, 0xff, 0xff)))
            .set_bg(Role::Custom(Color::Rgb(0x18, 0x18, 0x1b)))
            .add(termrock::Modifier::BOLD)
            .remove(termrock::Modifier::UNDERLINED),
    ),
    (
        Part::DETAIL,
        StylePatch::new()
            .set_fg(Role::Custom(Color::Rgb(0x73, 0x73, 0x73)))
            .set_bg(Role::Custom(Color::Rgb(0x18, 0x18, 0x1b)))
            .remove(termrock::Modifier::BOLD),
    ),
];
const HELP_FOOTER_PARTS: &[(Part, StylePatch)] = &[
    (
        Part::CONTAINER,
        StylePatch::new()
            .set_fg(Role::Custom(Color::Rgb(0xff, 0xff, 0xff)))
            .set_bg(Role::Custom(Color::Rgb(0x00, 0x00, 0x00))),
    ),
    (
        Part::KEY,
        StylePatch::new()
            .set_fg(Role::Custom(Color::Rgb(0xff, 0xff, 0xff)))
            .set_bg(Role::Custom(Color::Rgb(0x00, 0x00, 0x00)))
            .add(termrock::Modifier::BOLD),
    ),
    (
        Part::ACTION,
        StylePatch::new()
            .set_fg(Role::Custom(Color::Rgb(0x8a, 0x8a, 0x8a)))
            .set_bg(Role::Custom(Color::Rgb(0x00, 0x00, 0x00))),
    ),
];

fn help_sections(route: Route) -> &'static [termrock::HelpSection<'static>] {
    match route {
        Route::Home => &HELP_HOME_SECTIONS,
        Route::Plan => &HELP_PLAN_SECTIONS,
        Route::Activity | Route::TaskRun => &HELP_ACTIVITY_SECTIONS,
        Route::Disk
        | Route::Files
        | Route::Cleanup
        | Route::Review
        | Route::Snapshot
        | Route::DiskScan => &HELP_PAGE_SECTIONS,
    }
}

fn help_overlay(route: Route, scope: &str) -> termrock::HelpOverlay<'_> {
    termrock::HelpOverlay::new(HELP_LAYER, scope, help_sections(route))
        .title("Key reference")
        .patch_part(HELP_OVERLAY_PARTS)
}

fn help_footer_hints() -> HintLayer {
    HintLayer {
        hints: vec![
            Hint {
                key: HintKey::Label("↑↓"),
                label: "Scroll",
                priority: 50,
            },
            Hint {
                key: HintKey::Label("Esc"),
                label: "Close",
                priority: 50,
            },
        ],
        ..HintLayer::empty()
    }
}

fn paint_help_footer_hints(ui: &mut Ui<'_>, footer: Rect) {
    let hints = help_footer_hints();
    let mut x = footer.x.saturating_add(1);
    for (index, hint) in hints.hints.iter().take(2).enumerate() {
        let id = if index == 0 {
            HELP_SCROLL_HINT
        } else {
            HELP_CLOSE_HINT
        };
        let key_hint = KeyHint::from_hint(id, hint).patch_part(HELP_FOOTER_PARTS);
        let drawn = key_hint.draw(
            ui,
            Rect::new(x, footer.y, footer.right().saturating_sub(x), 1),
        );
        x = drawn.right().saturating_add(2);
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Route {
    Home,
    Plan,
    Activity,
    Disk,
    Files,
    Cleanup,
    Review,
    Snapshot,
    DiskScan,
    TaskRun,
}

fn opening_route(scenario: Scenario) -> Route {
    match scenario.screen() {
        Screen::Disk => Route::Disk,
        Screen::Activity => Route::Activity,
        Screen::Files => Route::Files,
        Screen::Cleanup => Route::Cleanup,
        Screen::Review => Route::Review,
        Screen::Snapshot => Route::Snapshot,
        Screen::Finder | Screen::Plan => Route::Home,
    }
}
#[derive(Clone, Copy)]
struct ReturnLocation {
    route: Route,
    focus: Option<Id>,
}

struct TaskRunOwner {
    route_id: u64,
    execution_id: ActivityExecutionId,
    scenario: Scenario,
    cwd: String,
    effect_revision: u64,
    action: crate::domain::action::Action,
    activity: Activity,
    origin: ReturnLocation,
}

struct TaskRunOverlayReturn {
    route_id: u64,
    run_id: TaskRunId,
    prompt_id: Option<PromptId>,
}

struct PortArgsOpening {
    token: LayerOpening,
    route: Route,
    origin_focus: Option<Id>,
}

struct PendingPortReview {
    opening: LayerOpening,
    route: Route,
    origin_focus: Option<Id>,
    producer: crate::sim::port_args::PortArgsProducer,
    plan: Box<crate::sim::port_args::PortInvocationPlan>,
}

enum Overlay {
    Dialog(Box<ProductDialog>),
    Help {
        state: termrock::HelpOverlayState,
        route: Route,
        scope: String,
    },
    Args(Box<TaskArgsEditor>),
    PortArgs(Box<PortArgsEditor>),
    Clone(ClonePicker),
    Actions(Actions),
    Gate(PlanGate),
    DiskSelection(disk::SelectionChooser),
}

pub struct App {
    world: World,
    source_theme: Option<ActivityThemeSource>,
    motion: Motion,
    route: Route,
    home: HomeState,
    plan: Option<PlanState>,
    activities: Activities,
    activity_executions: BTreeMap<u32, ActivityExecutionId>,
    task_runs: TaskRunStore,
    task_run_owners: BTreeMap<TaskRunId, TaskRunOwner>,
    task_run_screen: Option<TaskRunScreen>,
    retired_task_run_screens: Vec<TaskRunScreen>,
    next_task_run_route_id: Option<u64>,
    pending_task_run_return: Option<ReturnLocation>,
    disk_scan_task_run_return: Option<TaskRunOverlayReturn>,
    snapshot_kind: snapshot::Kind,
    snapshot_return: Option<ReturnLocation>,
    snapshot: snapshot::State,
    files: files::State,
    files_return: Option<ReturnLocation>,
    disk_scan: disk_scan::State,
    disk_scan_return: Option<ReturnLocation>,
    disk_scan_menu_origin: Option<ReturnLocation>,
    overlay: Option<Overlay>,
    retired_overlay: Option<Overlay>,
    port_args_opening: Option<PortArgsOpening>,
    pending_port_review: Option<PendingPortReview>,
    menu: MenuState,
    keymap: KeyMap,
    status: Option<String>,
    status_until_ms: Option<i64>,
    last_step_at: Option<termrock::Moment>,
    quit: bool,
}

#[derive(Clone)]
struct ActivityThemeSource {
    theme: termrock::Theme,
    projections: [(termrock::ColorLevel, termrock::Theme); 4],
    recipes_are_known: bool,
    preserve_theme_foreground: bool,
}

impl ActivityThemeSource {
    fn projection(&self, requested: termrock::ColorLevel) -> Option<&termrock::Theme> {
        self.projections
            .iter()
            .find_map(|(level, theme)| (*level == requested).then_some(theme))
    }

    fn matches_current(&self, current: &termrock::Theme) -> bool {
        self.projection(current.capability.color)
            .is_some_and(|expected| current == expected)
    }
}

struct ValidatedActivityTheme<'a> {
    source: &'a ActivityThemeSource,
    current: &'a termrock::Theme,
}
impl std::fmt::Debug for App {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("App")
            .field("scenario", &self.world.scenario)
            .field("motion", &self.motion)
            .finish_non_exhaustive()
    }
}
const BAR_LABELS: [&str; 3] = ["File", "Go", "Help"];

fn menu_bar() -> MenuBar<'static> {
    const FILE: &[MenuItem<'static>] = &[MenuItem::new(QUIT, "Quit").chord(Chord::with(
        KeyCode::Char('q'),
        termrock::KeyModifiers::CONTROL,
    ))];
    const GO: &[MenuItem<'static>] = &[
        MenuItem::new(HOME, "Home"),
        MenuItem::new(DISK_SCAN, "Modeled disk scan"),
    ];
    const HELP_MENU: &[MenuItem<'static>] = &[
        MenuItem::new(HELP, "Key reference").chord(Chord::key(KeyCode::F(1))),
        MenuItem::new(ABOUT, "About holla"),
    ];
    const MENUS: &[Menu<'static>] = &[
        Menu::new(BAR_LABELS[0], FILE),
        Menu::new(BAR_LABELS[1], GO),
        Menu::new(BAR_LABELS[2], HELP_MENU),
    ];

    MenuBar::new(MENU, MENUS)
        .chord_case(termrock::ChordCase::UppercaseAscii)
        .hover_scope(MenuBarHoverScope::Title)
}

fn brand() -> Brand<'static> {
    Brand::new(BRAND, "holla❯").clickable(true)
}

fn rest_tag_menu(world: &crate::sim::World) -> bool {
    use crate::sim::catalogue::{
        parity_browser_tag_frame, parity_cargo_tag_frame, parity_files_tag_frame,
        parity_git_batch_tag_frame, parity_git_current_tag_frame, parity_history_tag_frame,
    };
    world.scenario == Scenario::FirstUse
        || parity_git_current_tag_frame(world)
        || parity_cargo_tag_frame(world)
        || parity_git_batch_tag_frame(world)
        || parity_files_tag_frame(world)
        || parity_browser_tag_frame(world)
        || parity_history_tag_frame(world)
}

fn paint_rest_menu_titles(ui: &mut Ui<'_>, menu_x: u16, y: u16) {
    let plain = ui.paint_patch(
        &StylePatch::new()
            .set_fg(Role::Fg(FgStep::Secondary))
            .remove(termrock::Modifier::BOLD),
    );
    let gap = ui.paint_patch(
        &StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .remove(termrock::Modifier::BOLD),
    );
    let mut x = menu_x;
    ui.paint_str(Rect::new(x, y, 1, 1), " ", gap);
    x = x.saturating_add(1);
    for label in BAR_LABELS {
        let text = format!(" {label} ");
        let width = termrock::width(&text);
        ui.paint_str(Rect::new(x, y, width, 1), &text, plain);
        x = x.saturating_add(width);
        ui.paint_str(Rect::new(x, y, 1, 1), " ", gap);
        x = x.saturating_add(1);
    }
}

const MENU_CHROME_EDGE: u16 = 2;
const MENU_SEGMENT_GAP: u16 = 2;

const STATUS_EDGE: u16 = 1;
const STATUS_GAP: u16 = 3;
const STATUS_BG: Color = Color::Rgb(0x18, 0x18, 0x1b);
const STATUS_FG: Color = Color::Rgb(0xff, 0xff, 0xff);
const STATUS_GREY: Color = Color::Rgb(0x8a, 0x8a, 0x8a);
const STATUS_LINK: Color = Color::Rgb(0x79, 0xb8, 0xff);
const STATUS_DANGER: Color = Color::Rgb(0xf0, 0x5d, 0x5d);
const STATUS_AMBER: Color = Color::Rgb(0xf5, 0x9e, 0x09);
const STATUS_CENTER: &str = "! docker, github unavailable";
const STATUS_WARN: &str = "▲ 4 config warnings";
const PARITY_HINT_BG: Color = Color::Rgb(0, 0, 0);
const PARITY_HINT_PARTS: &[(Part, StylePatch)] = &[
    (
        Part::CONTAINER,
        StylePatch::new()
            .set_fg(Role::Custom(STATUS_FG))
            .set_bg(Role::Custom(PARITY_HINT_BG)),
    ),
    (
        Part::KEY,
        StylePatch::new()
            .set_fg(Role::Custom(STATUS_FG))
            .set_bg(Role::Custom(PARITY_HINT_BG))
            .add(termrock::Modifier::BOLD),
    ),
    (
        Part::ACTION,
        StylePatch::new()
            .set_fg(Role::Custom(STATUS_GREY))
            .set_bg(Role::Custom(PARITY_HINT_BG)),
    ),
    (
        Part::LABEL,
        StylePatch::new()
            .set_fg(Role::Custom(Color::Rgb(0xb3, 0xb3, 0xb3)))
            .set_bg(Role::Custom(PARITY_HINT_BG)),
    ),
];

const ACTIVITIES_HINT_PARTS: [(Part, StylePatch); 5] = [
    (
        Part::CONTAINER,
        StylePatch::new()
            .set_fg(Role::Custom(STATUS_FG))
            .set_bg(Role::Custom(PARITY_HINT_BG)),
    ),
    (
        Part::KEY,
        StylePatch::new()
            .set_fg(Role::Custom(STATUS_FG))
            .set_bg(Role::Custom(PARITY_HINT_BG))
            .add(termrock::Modifier::BOLD),
    ),
    (
        Part::ACTION,
        StylePatch::new()
            .set_fg(Role::Custom(STATUS_GREY))
            .set_bg(Role::Custom(PARITY_HINT_BG)),
    ),
    (
        Part::LABEL,
        StylePatch::new()
            .set_fg(Role::Custom(Color::Rgb(0xb3, 0xb3, 0xb3)))
            .set_bg(Role::Custom(PARITY_HINT_BG)),
    ),
    (
        Part::OVERFLOW,
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Faint))
            .set_bg(Role::Custom(PARITY_HINT_BG)),
    ),
];

fn status_ink(ui: &Ui<'_>, fg: Color, bold: bool) -> termrock::PaintStyle {
    let patch = StylePatch::new()
        .set_fg(Role::Custom(fg))
        .set_bg(Role::Custom(STATUS_BG));
    let patch = if bold {
        patch.add(termrock::Modifier::BOLD)
    } else {
        patch.remove(termrock::Modifier::BOLD)
    };
    ui.paint_patch(&patch)
}

fn paint_status_text(
    ui: &mut Ui<'_>,
    area: Rect,
    x: u16,
    text: &str,
    fg: Color,
    bold: bool,
) -> u16 {
    let width = termrock::width(text);
    let room = area.right().saturating_sub(x);
    if width == 0 || room == 0 {
        return x;
    }
    ui.paint_str(
        Rect::new(x, area.y, width.min(room), 1),
        text,
        status_ink(ui, fg, bold),
    );
    x.saturating_add(width.min(room))
}

fn status_ink_role(ui: &Ui<'_>, fg: Role, bold: bool) -> termrock::PaintStyle {
    let patch = StylePatch::new().set_fg(fg).set_bg(Role::Custom(STATUS_BG));
    let patch = if bold {
        patch.add(termrock::Modifier::BOLD)
    } else {
        patch.remove(termrock::Modifier::BOLD)
    };
    ui.paint_patch(&patch)
}

fn paint_status_text_role(
    ui: &mut Ui<'_>,
    area: Rect,
    x: u16,
    text: &str,
    fg: Role,
    bold: bool,
) -> u16 {
    let width = termrock::width(text);
    let room = area.right().saturating_sub(x);
    if width == 0 || room == 0 {
        return x;
    }
    let style = status_ink_role(ui, fg, bold);
    ui.paint_str(Rect::new(x, area.y, width.min(room), 1), text, style);
    x.saturating_add(width.min(room))
}

fn paint_size_label(ui: &mut Ui<'_>, area: Rect) -> u16 {
    let screen = ui.full();
    let size = format!(
        "{} · {}×{}",
        ui.theme_ref().capability.color.label(),
        screen.width,
        screen.height
    );
    let size_w = termrock::width(&size);
    let size_x = area
        .right()
        .saturating_sub(STATUS_EDGE)
        .saturating_sub(size_w);
    paint_status_text(ui, area, size_x, &size, STATUS_GREY, false);
    size_x
}

fn parity_history_tag_plate(world: &World, width: u16, _height: u16) -> bool {
    crate::sim::catalogue::parity_history_tag_frame(world) && (80..160).contains(&width)
}

fn parity_files_tag_plate(world: &World, width: u16, _height: u16) -> bool {
    crate::sim::catalogue::parity_files_tag_frame(world) && (80..160).contains(&width)
}

fn parity_browser_tag_plate(world: &World, width: u16, _height: u16) -> bool {
    crate::sim::catalogue::parity_browser_tag_frame(world) && (80..160).contains(&width)
}

fn parity_git_current_tag_plate(world: &World, width: u16, _height: u16) -> bool {
    crate::sim::catalogue::parity_git_current_tag_frame(world) && (80..160).contains(&width)
}

fn parity_cargo_tag_plate(world: &World, width: u16, _height: u16) -> bool {
    crate::sim::catalogue::parity_cargo_tag_frame(world) && (80..160).contains(&width)
}

fn parity_git_batch_tag_plate(world: &World, width: u16, _height: u16) -> bool {
    crate::sim::catalogue::parity_git_batch_tag_frame(world) && (80..160).contains(&width)
}

fn parity_task_sources_tag_plate(world: &World, width: u16, _height: u16) -> bool {
    crate::sim::catalogue::parity_task_sources_tag_frame(world)
        && layout::Band::of(width).is_narrow()
}

fn draw_parity_browser_status(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    let screen = ui.full();
    let size = format!(
        "{} · {}×{}",
        ui.theme_ref().capability.color.label(),
        screen.width,
        screen.height
    );
    let left = [StatusItem::new("~/work/site")
        .tone(Role::Custom(STATUS_FG))
        .strong()];
    let right = [StatusItem::new(&size).tone(Role::Custom(STATUS_GREY))];
    let patch = StylePatch::new()
        .set_fg(Role::Custom(STATUS_FG))
        .set_bg(Role::Custom(STATUS_BG))
        .remove(termrock::Modifier::BOLD);
    StatusBar::new(PARITY_BROWSER_STATUS)
        .left(&left)
        .right(&right)
        .patch(&patch)
        .draw(ui, area);
}

fn draw_parity_files_status(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    let screen = ui.full();
    let size = format!(
        "{} · {}×{}",
        ui.theme_ref().capability.color.label(),
        screen.width,
        screen.height
    );
    let left = [StatusItem::new("~/work/notes")
        .tone(Role::Custom(STATUS_FG))
        .strong()];
    let right = [StatusItem::new(&size).tone(Role::Custom(STATUS_GREY))];
    let patch = StylePatch::new()
        .set_fg(Role::Custom(STATUS_FG))
        .set_bg(Role::Custom(STATUS_BG))
        .remove(termrock::Modifier::BOLD);
    StatusBar::new(PARITY_FILES_STATUS)
        .left(&left)
        .right(&right)
        .patch(&patch)
        .draw(ui, area);
}

fn paint_parity_git_current_status(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    ui.fill(area, status_ink(ui, STATUS_FG, false));
    let mut x = area.x.saturating_add(STATUS_EDGE);
    for (text, fg, bold) in [
        ("~/work/svc/src/api", STATUS_FG, true),
        ("feature/x", STATUS_GREY, false),
        ("↓2 ↑1", STATUS_LINK, false),
        ("• 1", STATUS_AMBER, false),
    ] {
        x = paint_status_text(ui, area, x, text, fg, bold).saturating_add(STATUS_GAP);
    }
    let _ = x;
    let _ = paint_size_label(ui, area);
}

fn paint_parity_cargo_status(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    ui.fill(area, status_ink(ui, STATUS_FG, false));
    let cwd_end = paint_status_text(
        ui,
        area,
        area.x.saturating_add(STATUS_EDGE),
        "~/work/engine",
        STATUS_FG,
        true,
    );
    let _ = paint_status_text(
        ui,
        area,
        cwd_end.saturating_add(STATUS_GAP),
        "main",
        STATUS_GREY,
        false,
    );
    let _ = paint_size_label(ui, area);
}

fn paint_parity_task_sources_status(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    ui.fill(area, status_ink(ui, STATUS_FG, false));
    let _ = paint_status_text(
        ui,
        area,
        area.x.saturating_add(STATUS_EDGE),
        "~/work/poly",
        STATUS_FG,
        true,
    );
    let _ = paint_size_label(ui, area);
}

fn paint_parity_git_batch_status(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    ui.fill(area, status_ink(ui, STATUS_FG, false));
    let _ = paint_status_text(
        ui,
        area,
        area.x.saturating_add(STATUS_EDGE),
        "~/work/repos",
        STATUS_FG,
        true,
    );
    let _ = paint_size_label(ui, area);
}

fn paint_parity_history_status(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    ui.fill(area, status_ink(ui, STATUS_FG, false));
    let cwd_end = paint_status_text(
        ui,
        area,
        area.x.saturating_add(STATUS_EDGE),
        "~/work/holla",
        STATUS_FG,
        true,
    );
    let _ = paint_status_text(
        ui,
        area,
        cwd_end.saturating_add(STATUS_GAP),
        "main",
        STATUS_GREY,
        false,
    );
    let _ = paint_size_label(ui, area);
}

fn paint_first_use_status(
    ui: &mut Ui<'_>,
    area: Rect,
    world: &World,
    motion: Motion,
    center: Option<&str>,
) {
    if area.is_empty() {
        return;
    }
    ui.fill(area, status_ink(ui, STATUS_FG, false));
    let left_end = paint_status_text(
        ui,
        area,
        area.x.saturating_add(STATUS_EDGE),
        &world.cwd,
        STATUS_FG,
        true,
    );
    let size_x = paint_size_label(ui, area);
    let mut right_edge = size_x.saturating_sub(STATUS_GAP);
    if let Some(disk) = &world.disk {
        let percent = if disk.total_bytes == 0 {
            0
        } else {
            let numerator = u128::from(disk.used_bytes)
                .saturating_mul(100)
                .saturating_add(u128::from(disk.total_bytes) / 2);
            u32::try_from((numerator / u128::from(disk.total_bytes)).min(100)).unwrap_or(100)
        };
        let mark = format!("{percent}%{}", if percent >= 85 { " ▲" } else { "" });
        let mark_w = termrock::width(&mark);
        let bar_w = 11u16;
        let meter_w = 1u16
            .saturating_add(1)
            .saturating_add(bar_w)
            .saturating_add(1)
            .saturating_add(mark_w);
        let mut x = size_x.saturating_sub(STATUS_GAP).saturating_sub(meter_w);
        right_edge = x.saturating_sub(STATUS_GAP);
        x = paint_status_text(ui, area, x, "/", STATUS_GREY, false).saturating_add(1);
        let filled =
            u16::try_from(u32::from(bar_w).saturating_mul(percent).saturating_add(50) / 100)
                .unwrap_or(bar_w)
                .min(bar_w);
        x = paint_status_text(
            ui,
            area,
            x,
            &"━".repeat(usize::from(filled)),
            STATUS_AMBER,
            false,
        );
        let rest = bar_w.saturating_sub(filled);
        x = paint_status_text(
            ui,
            area,
            x,
            &"─".repeat(usize::from(rest)),
            Color::Rgb(0x3a, 0x3a, 0x3a),
            false,
        )
        .saturating_add(1);
        let _ = paint_status_text(ui, area, x, &mark, STATUS_AMBER, false);
    }
    if let Some(center) = center {
        let progress_start = left_end.saturating_add(STATUS_GAP);
        let available = right_edge.saturating_sub(progress_start);
        if available > 0 {
            let center = termrock::truncate(center, available);
            let center_width = termrock::width(&center);
            let x = progress_start.saturating_add(available.saturating_sub(center_width) / 2);
            let _ = paint_status_text(ui, area, x, &center, STATUS_GREY, false);
        }
        return;
    }
    let pending: Vec<_> = world
        .discovery
        .iter()
        .filter(|source| !source.done && !source.failed)
        .map(|source| source.domain.label())
        .collect();
    let failed: Vec<_> = world
        .discovery
        .iter()
        .filter(|source| source.failed)
        .map(|source| source.domain.label())
        .collect();
    let done = world.discovery.iter().filter(|source| source.done).count();
    let mut progress_spinner = None;
    let progress = if !pending.is_empty() {
        let frames = ui.design().motion.spinner_frames;
        let frame = if motion == Motion::Full {
            let tick_ms = ui.design().motion.tick_ms.max(1);
            let elapsed_ms = u64::try_from(world.clock.now_ms.max(0)).unwrap_or_default();
            let frame_count = u64::try_from(frames.len()).unwrap_or(1).max(1);
            usize::try_from((elapsed_ms / tick_ms) % frame_count).unwrap_or_default()
        } else {
            0
        };
        let spinner = frames.get(frame).copied().unwrap_or("⠋");
        progress_spinner = Some(spinner);
        let shown = pending
            .iter()
            .take(3)
            .copied()
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "{} discovering {}{} · {} of {}",
            spinner,
            shown,
            if pending.len() > 3 { ", …" } else { "" },
            done,
            world.discovery.len()
        )
    } else if !failed.is_empty() {
        format!("! {} unavailable", failed.join(", "))
    } else {
        "discovery complete".to_owned()
    };
    let progress_start = left_end.saturating_add(STATUS_GAP);
    let available = right_edge.saturating_sub(progress_start);
    if available > 0 {
        let progress = termrock::truncate(&progress, available);
        let progress_width = termrock::width(&progress);
        let x = progress_start.saturating_add(available.saturating_sub(progress_width) / 2);
        if let Some(spinner) = progress_spinner {
            let rest = progress.strip_prefix(spinner).unwrap_or("");
            let mut text_x = paint_status_text_role(ui, area, x, spinner, Role::Accent, false);
            if let Some(body) = rest.strip_prefix(" ") {
                text_x =
                    paint_status_text_role(ui, area, text_x, " ", Role::Fg(FgStep::Primary), false);
                let _ = paint_status_text_role(
                    ui,
                    area,
                    text_x,
                    body,
                    Role::Fg(FgStep::Secondary),
                    false,
                );
            } else {
                let _ = paint_status_text_role(
                    ui,
                    area,
                    text_x,
                    rest,
                    Role::Fg(FgStep::Secondary),
                    false,
                );
            }
        } else {
            let _ = paint_status_text(ui, area, x, &progress, STATUS_GREY, false);
        }
    }
}

fn paint_remote_host_status(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    ui.fill(area, status_ink(ui, STATUS_FG, false));
    let mut x = area.x.saturating_add(STATUS_EDGE);
    for (text, fg, bold) in [
        ("/srv/payments", STATUS_FG, true),
        ("detached @ v2.14.3", STATUS_GREY, false),
    ] {
        x = paint_status_text(ui, area, x, text, fg, bold).saturating_add(STATUS_GAP);
    }
    let _ = paint_size_label(ui, area);
}

fn paint_hard_cases_status(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    ui.fill(area, status_ink(ui, STATUS_FG, false));
    let path = termrock::truncate_middle(
        "~/work/clients/northwind-traders-platform-migration/services/inventory-reconciliation-service",
        44,
    );
    let mut x = area.x.saturating_add(STATUS_EDGE);
    if layout::Band::of(ui.full().width).is_narrow() {
        paint_status_text(ui, area, x, &path, STATUS_FG, true);
        let mark_x = area.right().saturating_sub(STATUS_EDGE).saturating_sub(1);
        paint_status_text(ui, area, mark_x, "…", STATUS_GREY, false);
        return;
    }
    let mid = layout::Band::of(ui.full().width).is_mid();
    for (text, fg, bold) in [
        (path.as_str(), STATUS_FG, true),
        ("↓14 ↑3", STATUS_LINK, false),
        ("• 7", STATUS_AMBER, false),
    ] {
        x = paint_status_text(ui, area, x, text, fg, bold).saturating_add(STATUS_GAP);
    }
    if !mid {
        for (text, fg, bold) in [
            ("2 conflicts", STATUS_DANGER, false),
            ("rebase", STATUS_AMBER, false),
        ] {
            x = paint_status_text(ui, area, x, text, fg, bold).saturating_add(STATUS_GAP);
        }
    }
    let warn = "! docker, github unavailable";
    let mark_x = area.right().saturating_sub(STATUS_EDGE).saturating_sub(1);
    let warn_x = if mid {
        let warn_w = termrock::width(warn);
        let hi = mark_x.saturating_sub(STATUS_GAP);
        let free = hi.saturating_sub(x);
        if free >= warn_w {
            x.saturating_add(free.saturating_sub(warn_w) / 2)
        } else {
            x
        }
    } else {
        x
    };
    paint_status_text(ui, area, warn_x, warn, STATUS_DANGER, false);
    paint_status_text(ui, area, mark_x, "…", STATUS_GREY, false);
}

fn paint_docker_cleanup_status(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    ui.fill(area, status_ink(ui, STATUS_FG, false));
    let _ = paint_status_text(
        ui,
        area,
        area.x.saturating_add(STATUS_EDGE),
        "/work/scratch",
        STATUS_FG,
        true,
    );
    let _ = paint_size_label(ui, area);
}

fn paint_disk_cleanup_status(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    ui.fill(area, status_ink(ui, STATUS_FG, false));
    let cwd_end = paint_status_text(
        ui,
        area,
        area.x.saturating_add(STATUS_EDGE),
        "~/work",
        STATUS_FG,
        true,
    );
    let size_x = paint_size_label(ui, area);
    let mark = "91% ▲";
    let mark_w = termrock::width(mark);
    let bar_w = 11u16;
    let meter_w = 1u16
        .saturating_add(1)
        .saturating_add(bar_w)
        .saturating_add(1)
        .saturating_add(mark_w);
    let meter_x = size_x.saturating_sub(STATUS_GAP).saturating_sub(meter_w);
    let center = "3 selected · 24.2 GiB";
    let center_w = termrock::width(center);
    let lo = cwd_end.saturating_add(STATUS_GAP);
    let hi = meter_x.saturating_sub(STATUS_GAP);
    let free = hi.saturating_sub(lo);
    if free >= center_w {
        let cx = lo.saturating_add(free.saturating_sub(center_w) / 2);
        paint_status_text(ui, area, cx, center, Color::Rgb(0xb3, 0xb3, 0xb3), false);
    }
    let mut x = paint_status_text(ui, area, meter_x, "/", STATUS_GREY, false).saturating_add(1);
    x = paint_status_text(ui, area, x, &"━".repeat(10), STATUS_AMBER, false);
    x = paint_status_text(ui, area, x, "─", Color::Rgb(0x3a, 0x3a, 0x3a), false).saturating_add(1);
    let _ = paint_status_text(ui, area, x, mark, STATUS_AMBER, false);
}

fn paint_disk_cleanup_footer(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    let canvas = Color::Rgb(0, 0, 0);
    let key_style = status_ink(ui, STATUS_FG, true).bg(canvas);
    let label_style = status_ink(ui, STATUS_GREY, false).bg(canvas);
    let overflow = status_ink(ui, Color::Rgb(0x73, 0x73, 0x73), false).bg(canvas);
    let wide_hints = [
        ("↑↓", "Move"),
        ("←→", "Fold / open"),
        ("Space", "Select"),
        ("s", "Sort"),
        ("f", "Fold noise"),
        ("d", "Delete…"),
        ("c", "Artifact plan"),
        ("r", "Rescan"),
        ("Esc", "Back"),
        ("x", "Cancel scan"),
    ];
    let mid_hints = [
        ("↑↓", "Move"),
        ("←→", "Fold / open"),
        ("Space", "Select"),
        ("s", "Sort"),
        ("f", "Fold noise"),
        ("d", "Delete…"),
        ("Esc", "Back"),
        ("x", "Cancel scan"),
    ];
    let narrow_hints = [
        ("↑↓", "Move"),
        ("←→", "Fold / open"),
        ("Space", "Select"),
        ("s", "Sort"),
        ("f", "Fold noise"),
        ("Esc", "Back"),
        ("x", "Cancel scan"),
    ];
    let narrow = layout::Band::of(ui.full().width).is_narrow();
    let mid = layout::Band::of(ui.full().width).is_mid();
    let hints: &[(&str, &str)] = if narrow {
        &narrow_hints
    } else if mid {
        &mid_hints
    } else {
        &wide_hints
    };

    let room = u32::from(area.width.saturating_sub(4));
    let mut x = area.x.saturating_add(1);
    let mut truncated = false;
    for (index, (key, label)) in hints.iter().enumerate() {
        let key_w = termrock::width(key);
        let label_w = termrock::width(label);
        let need = key_w.saturating_add(1).saturating_add(label_w);
        let cut = if mid {
            disk_hint_cost(hints, index.saturating_add(1)) > room
        } else {
            x.saturating_add(need) > area.right()
        };
        if cut {
            truncated = true;
            break;
        }
        ui.paint_str(Rect::new(x, area.y, key_w, 1), key, key_style);
        x = x.saturating_add(key_w).saturating_add(1);
        ui.paint_str(Rect::new(x, area.y, label_w, 1), label, label_style);
        x = x.saturating_add(label_w);
        if index.saturating_add(1) != hints.len() {
            x = x.saturating_add(2);
        }
    }
    if truncated && x < area.right() {
        ui.paint_str(Rect::new(x, area.y, 1, 1), "…", overflow);
    }
}

fn disk_hint_cost(hints: &[(&str, &str)], keep: usize) -> u32 {
    let mut cost = 0u32;
    for (key, label) in hints.iter().take(keep) {
        let hint_w = u32::from(
            termrock::width(key)
                .saturating_add(1)
                .saturating_add(termrock::width(label)),
        );
        cost = cost.saturating_add(hint_w).saturating_add(2);
    }
    if keep < hints.len() {
        cost = cost.saturating_add(2);
    }
    cost
}

fn paint_monorepo_status(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    ui.fill(area, status_ink(ui, STATUS_FG, false));
    let mut x = area.x.saturating_add(STATUS_EDGE);
    for (text, fg, bold) in [
        ("~/work/acme", STATUS_FG, true),
        ("main", STATUS_GREY, false),
        ("↑1", STATUS_LINK, false),
    ] {
        x = paint_status_text(ui, area, x, text, fg, bold).saturating_add(STATUS_GAP);
    }
    let _ = paint_size_label(ui, area);
}

fn paint_activities_multi_status(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    paint_monorepo_status(ui, area);
    let size_x = paint_size_label(ui, area);
    let spin = " ⠋";
    let running = "4 running ";
    let group = termrock::width(spin)
        .saturating_add(1)
        .saturating_add(termrock::width(running));
    let x = size_x.saturating_sub(STATUS_GAP).saturating_sub(group);
    let label = activity_chip_style(ui, Color::Rgb(0xb3, 0xb3, 0xb3));
    let mark = activity_chip_style(ui, Color::Rgb(0x48, 0xe0, 0x54));
    ui.paint_str(Rect::new(x, area.y, 1, 1), " ", label);
    ui.paint_str(Rect::new(x.saturating_add(1), area.y, 1, 1), "⠋", mark);
    ui.paint_str(
        Rect::new(x.saturating_add(3), area.y, termrock::width(running), 1),
        running,
        label,
    );
}

fn activity_chip_style(ui: &Ui<'_>, fg: Color) -> termrock::PaintStyle {
    ui.paint_patch(
        &StylePatch::new()
            .set_fg(Role::Custom(fg))
            .set_bg(Role::Custom(Color::Rgb(0x27, 0x27, 0x2a)))
            .remove(termrock::Modifier::BOLD),
    )
}

fn launch_failure_activity_open(world: &World, ui: &Ui<'_>) -> bool {
    let full = ui.full();
    crate::sim::catalogue::launch_failure_activity_frame(world, full.width)
}

fn paint_launch_failure_status(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    ui.fill(area, status_ink(ui, STATUS_FG, false));
    let mut x = area.x.saturating_add(STATUS_EDGE);
    for (text, fg, bold) in [
        ("~/work/acme/apps/frontend", STATUS_FG, true),
        ("main", STATUS_GREY, false),
        ("↑1", STATUS_LINK, false),
        ("• 2", STATUS_AMBER, false),
    ] {
        x = paint_status_text(ui, area, x, text, fg, bold).saturating_add(STATUS_GAP);
    }
    let left_end = x.saturating_sub(STATUS_GAP);
    let narrow = layout::Band::of(ui.full().width).is_narrow();
    let size_x = if narrow {
        let mark_x = area.right().saturating_sub(STATUS_EDGE).saturating_sub(1);
        paint_status_text(ui, area, mark_x, "…", STATUS_GREY, false);
        mark_x
    } else {
        paint_size_label(ui, area)
    };
    let center = "! exit 1 · 1 s";
    let center_w = termrock::width(center);
    let lo = left_end.saturating_add(STATUS_GAP);
    let hi = size_x.saturating_sub(STATUS_GAP);
    let free = hi.saturating_sub(lo);
    if free >= center_w {
        let cx = lo.saturating_add(free.saturating_sub(center_w) / 2);
        paint_status_text(ui, area, cx, center, STATUS_DANGER, false);
    }
}

fn paint_launch_failure_footer(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    let canvas = Color::Rgb(0, 0, 0);
    let key_style = status_ink(ui, STATUS_FG, true).bg(canvas);
    let label_style = status_ink(ui, STATUS_GREY, false).bg(canvas);
    let overflow = status_ink(ui, Color::Rgb(0x73, 0x73, 0x73), false).bg(canvas);
    let danger = status_ink(ui, STATUS_DANGER, false).bg(canvas);
    let danger_bold = status_ink(ui, STATUS_DANGER, true).bg(canvas);
    let rest = " frontend dev failed · exit 1";
    let total = 1u16.saturating_add(termrock::width(rest));
    let status_x = area.right().saturating_sub(1).saturating_sub(total);
    let hints = [
        ("↑↓", "Scroll"),
        ("f", "Follow"),
        ("/", "Find"),
        ("y", "Copy"),
        ("r", "Restart"),
        ("x", "Close"),
        ("Esc", "Back to Here"),
        ("Shift+F10", "Menu"),
    ];
    let mut x = area.x.saturating_add(1);
    let mut truncated = false;
    let narrow = layout::Band::of(ui.full().width).is_narrow();
    for (index, (key, label)) in hints.iter().enumerate() {
        let key_w = termrock::width(key);
        let label_w = termrock::width(label);
        let need = key_w.saturating_add(1).saturating_add(label_w);

        let reserve = if narrow && index.saturating_add(1) != hints.len() {
            3
        } else {
            0
        };
        if x.saturating_add(need).saturating_add(reserve) > status_x {
            truncated = true;
            break;
        }
        ui.paint_str(Rect::new(x, area.y, key_w, 1), key, key_style);
        x = x.saturating_add(key_w).saturating_add(1);
        ui.paint_str(Rect::new(x, area.y, label_w, 1), label, label_style);
        x = x.saturating_add(label_w);
        if index.saturating_add(1) != hints.len() {
            x = x.saturating_add(2);
        }
    }
    if truncated && x < status_x {
        ui.paint_str(Rect::new(x, area.y, 1, 1), "…", overflow);
    }
    ui.paint_str(Rect::new(status_x, area.y, 1, 1), "!", danger_bold);
    ui.paint_str(
        Rect::new(status_x.saturating_add(1), area.y, termrock::width(rest), 1),
        rest,
        danger,
    );
}

fn paint_monorepo_child_status(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    ui.fill(area, status_ink(ui, STATUS_FG, false));
    let mut x = area.x.saturating_add(STATUS_EDGE);
    for (text, fg, bold) in [
        ("~/work/acme/apps/frontend", STATUS_FG, true),
        ("main", STATUS_GREY, false),
        ("↑1", STATUS_LINK, false),
        ("• 2", STATUS_AMBER, false),
    ] {
        x = paint_status_text(ui, area, x, text, fg, bold).saturating_add(STATUS_GAP);
    }
    let _ = paint_size_label(ui, area);
}

fn docker_hint(key: &'static str, label: &'static str) -> Hint {
    Hint {
        key: HintKey::Label(key),
        label,
        priority: 50,
    }
}

fn task_args_hints() -> HintLayer {
    let scope = Chord::with(KeyCode::Char('s'), termrock::KeyModifiers::CONTROL);
    let mut hints = HintLayer::from_bindings(GLOBAL);
    hints.hints.retain(|hint| {
        !matches!(
            hint.key,
            HintKey::Chord(chord) | HintKey::ChordWithCase { chord, .. } if chord == scope
        )
    });
    hints.hints.push(Hint {
        key: HintKey::ChordWithCase {
            chord: scope,
            case: termrock::ChordCase::UppercaseAscii,
        },
        label: "Review inputs",
        priority: 40,
    });
    hints
}

fn docker_cleanup_hints() -> HintLayer {
    HintLayer {
        hints: vec![
            docker_hint("Type", "Search"),
            docker_hint("↑↓", "Move"),
            docker_hint("Enter", "Review…"),
            docker_hint("Alt+Enter", "Alternatives…"),
            docker_hint("Tab", "Preview"),
            docker_hint("Ctrl+↑↓", "Scope"),
            docker_hint("Esc", "Clear"),
        ],
        status: Some(std::borrow::Cow::Borrowed("Clean Docker c…")),
        ..HintLayer::empty()
    }
}

fn docker_cleanup_100x30_hints() -> HintLayer {
    HintLayer {
        hints: vec![
            docker_hint("Type", "Search"),
            docker_hint("↑↓", "Move"),
            docker_hint("Enter", "Review…"),
            docker_hint("Alt+Enter", "Alternatives…"),
            docker_hint("Tab", "Preview"),
            docker_hint("Esc", "Clear"),
            docker_hint("Ctrl+↑↓", "Scope "),
        ],
        ..HintLayer::empty()
    }
}

fn docker_cleanup_80x24_hints() -> HintLayer {
    HintLayer {
        hints: vec![
            docker_hint("Type", "Search"),
            docker_hint("↑↓", "Move"),
            docker_hint("Enter", "Review…"),
            docker_hint("Alt+Enter", "Alternatives…"),
            docker_hint("Esc", "Clear"),
            docker_hint("Tab", "Preview"),
            docker_hint("Ctrl+↑↓", "Scope"),
        ],
        ..HintLayer::empty()
    }
}

fn tag_hints(confirm: &'static str, status: Option<&'static str>) -> HintLayer {
    fn hint(key: &'static str, label: &'static str) -> Hint {
        Hint {
            key: HintKey::Label(key),
            label,
            priority: 50,
        }
    }
    HintLayer {
        hints: vec![
            hint("Type", "Search"),
            hint("↑↓", "Move"),
            hint("Enter", confirm),
            hint("Alt+Enter", "Alternatives…"),
            hint("Tab", "Preview"),
            hint("Ctrl+↑↓", "Scope"),
            hint("Ctrl+G", "Activities"),
        ],
        status: status.map(std::borrow::Cow::Borrowed),
        ..HintLayer::empty()
    }
}

fn menu_project(world: &World) -> String {
    match world.scenario {
        Scenario::ParityDiscovery => "probe · Rust project".to_owned(),
        Scenario::ParityHistory => "holla · Rust project".to_owned(),
        Scenario::ParityGitCurrent => "svc · Rust project".to_owned(),
        Scenario::ParityCargo => "engine · Rust project".to_owned(),
        Scenario::ParityCustomActions => "team · Rust project".to_owned(),
        Scenario::ParityDiskNavigation => "big · Rust project".to_owned(),
        Scenario::ParityDeleteSafety | Scenario::ParityCleanupResults => {
            "safe · Rust project".to_owned()
        }
        Scenario::ParityGitBatch => "repos · project collection".to_owned(),
        Scenario::ParityExecutor => "batch · project collection".to_owned(),
        Scenario::ParityTaskSources => "poly · Node project · yarn".to_owned(),
        Scenario::ParityInsights => "web · Node project · pnpm".to_owned(),
        Scenario::ParityGradle => "android · Gradle build".to_owned(),
        Scenario::ParityTaskInput => "app · service checkout".to_owned(),
        Scenario::ParityFiles
        | Scenario::ParityBrowser
        | Scenario::ParityDocker
        | Scenario::ParityBrewServices
        | Scenario::ParityIdea
        | Scenario::ParityUpgradeManagers
        | Scenario::ParityDiskScan
        | Scenario::ParityPlatforms
        | Scenario::ParityPlatformsLinux => "no project".to_owned(),
        Scenario::FirstUse => "no project".to_owned(),
        Scenario::MonorepoRoot if crate::sim::catalogue::monorepo_root_tag_frame(world) => {
            "acme · mise monorepo".to_owned()
        }
        Scenario::ActivitiesMulti if crate::sim::catalogue::activities_multi_tag_frame(world) => {
            "acme · mise monorepo".to_owned()
        }
        Scenario::MonorepoChild if crate::sim::catalogue::monorepo_child_tag_frame(world) => {
            "acme › frontend · Node project · pnpm".to_owned()
        }
        Scenario::DockerCleanup if crate::sim::catalogue::docker_cleanup_tag_frame(world) => {
            "no project".to_owned()
        }
        Scenario::RemoteHost if crate::sim::catalogue::remote_host_tag_frame(world) => {
            "payments · service checkout".to_owned()
        }
        Scenario::HardCases if crate::sim::catalogue::hard_cases_tag_frame(world) => {
            "northwind-traders-platform-migration › inventory-reconciliation-service · Rust workspace · member inventory-reconciliation-core".to_owned()
        }
        Scenario::DiskCleanup if crate::sim::catalogue::disk_cleanup_tag_frame(world) => {
            "work · project collection".to_owned()
        }
        _ => world.cwd.clone(),
    }
}

fn paint_menu_tail(ui: &mut Ui<'_>, area: Rect, project: &str, host: &str) {
    if area.is_empty() {
        return;
    }
    let host_w = termrock::width(host);
    let budget = area.width.saturating_sub(
        host_w.saturating_add(MENU_SEGMENT_GAP.saturating_add(MENU_CHROME_EDGE.saturating_mul(2))),
    );
    let project = (budget >= 12).then(|| termrock::truncate_middle(project, budget));
    let secondary = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary)));
    let host_style = ui
        .paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Primary)))
        .add_modifier(termrock::Modifier::BOLD);
    let mut x = area.right().saturating_sub(MENU_CHROME_EDGE);
    x = x.saturating_sub(host_w);
    if x >= area.x && host_w > 0 {
        ui.paint_str(Rect::new(x, area.y, host_w, 1), host, host_style);
    }
    if let Some(project) = project.as_deref() {
        let project_w = termrock::width(project);
        x = x.saturating_sub(MENU_SEGMENT_GAP).saturating_sub(project_w);
        if x >= area.x && project_w > 0 {
            ui.paint_str(Rect::new(x, area.y, project_w, 1), project, secondary);
        }
    }
}

fn gate_dialog() -> Dialog<'static> {
    Dialog::new(crate::screens::plan_gate::GATE)
}

impl App {
    #[must_use]
    pub fn with_source_theme(mut self, source_theme: termrock::Theme) -> Self {
        let projections = [
            (
                termrock::ColorLevel::TrueColor,
                source_theme.for_level(termrock::ColorLevel::TrueColor),
            ),
            (
                termrock::ColorLevel::Ansi256,
                source_theme.for_level(termrock::ColorLevel::Ansi256),
            ),
            (
                termrock::ColorLevel::Ansi16,
                source_theme.for_level(termrock::ColorLevel::Ansi16),
            ),
            (
                termrock::ColorLevel::Mono,
                source_theme.for_level(termrock::ColorLevel::Mono),
            ),
        ];
        let token_recipes = termrock::Theme::from_tokens(source_theme.color).recipes;
        let paper_recipes = termrock::Theme::paper().recipes;
        let junie_recipes = termrock::Theme::junie().recipes;
        let recipes_are_known = source_theme.recipes == token_recipes
            || source_theme.recipes == paper_recipes
            || source_theme.recipes == junie_recipes;
        let source_foreground = source_theme
            .resolve(
                termrock::Family::HINTBAR,
                termrock::Variant::DEFAULT,
                Part::OVERFLOW,
                termrock::StateFlags::empty(),
                Surface::Canvas,
            )
            .style
            .fg_role;
        let preserve_theme_foreground = !recipes_are_known
            || matches!(source_foreground, Some((role, _)) if role != Role::Fg(FgStep::Faint));
        self.source_theme = Some(ActivityThemeSource {
            theme: source_theme,
            projections,
            recipes_are_known,
            preserve_theme_foreground,
        });
        self
    }

    #[must_use]
    pub fn with_files_snapshot(mut self, snapshot: crate::sim::FilesSnapshot) -> Self {
        self.files = files::State::new(snapshot, &self.world.cwd);
        self
    }

    pub fn effect_revision(&self) -> u64 {
        self.world.effect_revision
    }

    pub fn fixture_time_ms(&self) -> i64 {
        self.world.now_ms()
    }

    pub fn for_scenario_without_history(scenario: Scenario, motion: Motion, frame: u64) -> Self {
        let mut app = Self::for_scenario(scenario, motion, frame);
        app.home.history = false;
        app
    }

    pub fn for_scenario(scenario: Scenario, motion: Motion, frame: u64) -> Self {
        Self::from_fixture_world(fixtures::world_for(scenario), motion, frame)
    }

    pub fn for_nextest_args_fixture(motion: Motion, frame: u64) -> Self {
        let mut world = crate::sim::fixtures::nextest_args_world();
        if let Some(mise) = &mut world.mise
            && let Some(task) = mise.tasks.iter_mut().find(|task| task.id == "test-nextest")
        {
            task.trust = crate::domain::mise::Trust::Untrusted;
        }
        Self::from_fixture_world(world, motion, frame)
    }

    fn from_fixture_world(mut world: World, motion: Motion, frame: u64) -> Self {
        world.seek(frame);
        world.clock.running = motion != Motion::Paused;
        let scenario = world.scenario;
        let mut keymap = KeyMap::new();
        for binding in GLOBAL {
            if let Some(chord) = binding.chord {
                keymap = keymap.bind(KeyPhase::Capture, chord, binding.action);
            }
        }
        keymap = keymap
            .bind(KeyPhase::Bubble, Chord::key(KeyCode::Down), RESULTS)
            .bind(KeyPhase::Bubble, Chord::key(KeyCode::Up), PREVIOUS)
            .bind(
                KeyPhase::Bubble,
                Chord::key(KeyCode::Tab),
                SNAPSHOT_FOCUS_NEXT,
            )
            .bind(
                KeyPhase::Bubble,
                Chord::with(KeyCode::BackTab, termrock::KeyModifiers::SHIFT),
                SNAPSHOT_FOCUS_PREVIOUS,
            )
            .bind(
                KeyPhase::Bubble,
                Chord::key(KeyCode::BackTab),
                SNAPSHOT_FOCUS_PREVIOUS,
            )
            .bind(
                KeyPhase::Capture,
                Chord::with(KeyCode::Char('c'), termrock::KeyModifiers::CONTROL),
                INTERRUPT,
            )
            .bind(KeyPhase::Capture, Chord::key(KeyCode::F(2)), TOP_FILES)
            .bind(KeyPhase::Capture, Chord::key(KeyCode::F(3)), CLEANUP_REVIEW)
            .bind(KeyPhase::Capture, Chord::key(KeyCode::F(4)), FOLDER_BROWSER)
            .bind(KeyPhase::Capture, Chord::key(KeyCode::F(5)), HOME_FIND)
            .bind(KeyPhase::Capture, Chord::key(KeyCode::F(6)), CUSTOM_ACTION)
            .bind(KeyPhase::Capture, Chord::key(KeyCode::F(7)), SNAPSHOT_PAGE)
            .bind(
                KeyPhase::Capture,
                Chord::key(KeyCode::F(8)),
                DEFINITION_TRUST,
            );
        let route = opening_route(scenario);
        let mut activities = Activities::default();
        if route == Route::Activity
            && let Some(activity) = world.activities.first()
        {
            activities.show(activity.id);
        }
        let mut home = HomeState::default();
        home.history = crate::sim::catalogue::history_enabled();
        let file_snapshot = crate::sim::fixtures::files_for_cwd(&world.cwd);
        let files = files::State::new(file_snapshot, &world.cwd);
        let activity_executions = world
            .scenario_activity_execution_ids()
            .iter()
            .copied()
            .map(|execution_id| (execution_id.activity_id(), execution_id))
            .collect();
        Self {
            world,
            source_theme: None,
            motion,
            route,
            home,
            plan: None,
            activities,
            activity_executions,
            task_runs: TaskRunStore::default(),
            task_run_owners: BTreeMap::new(),
            task_run_screen: None,
            retired_task_run_screens: Vec::new(),
            next_task_run_route_id: Some(1),
            pending_task_run_return: None,
            disk_scan_task_run_return: None,
            snapshot_kind: snapshot::Kind::Resources,
            snapshot_return: None,
            snapshot: snapshot::State::default(),
            files,
            files_return: None,
            disk_scan: disk_scan::State::default(),
            disk_scan_return: None,
            disk_scan_menu_origin: None,
            overlay: None,
            retired_overlay: None,
            port_args_opening: None,
            pending_port_review: None,
            menu: MenuState::default(),
            keymap,
            status: None,
            status_until_ms: None,
            last_step_at: None,
            quit: false,
        }
    }
    fn open_dialog(&mut self, intent: dialogs::Intent, cx: &mut Cx<'_>) {
        let dialog = ProductDialog::new(intent, &self.world);
        dialog.open(cx);
        self.overlay = Some(Overlay::Dialog(Box::new(dialog)));
    }
    fn help_scope(&self) -> String {
        match self.route {
            Route::Home => "Here".to_owned(),
            Route::Plan => self
                .plan
                .as_ref()
                .map_or_else(|| "Plan".to_owned(), |plan| plan.plan().title().to_owned()),
            Route::Activity => self
                .activities
                .current(&self.world)
                .map_or_else(|| "Activity".to_owned(), |activity| activity.name.clone()),
            Route::TaskRun => self
                .task_run_screen
                .as_ref()
                .and_then(|screen| self.task_run_owners.get(&screen.run_id()))
                .map_or_else(
                    || "Task run".to_owned(),
                    |owner| owner.activity.name.clone(),
                ),
            Route::Disk => "Disk".to_owned(),
            Route::Files => "Files".to_owned(),
            Route::Cleanup => "Cleanup".to_owned(),
            Route::Review => "Plan review".to_owned(),
            Route::Snapshot => "System snapshot".to_owned(),
            Route::DiskScan => "Disk scan".to_owned(),
        }
    }
    fn open_help(&mut self, cx: &mut Cx<'_>) {
        if cx.is_open(HELP_LAYER) {
            return;
        }
        let route = self.route;
        let scope = self.help_scope();
        let layer = help_overlay(route, &scope).layer(cx);
        cx.open_layer(HELP_LAYER, layer);
        self.overlay = Some(Overlay::Help {
            state: termrock::HelpOverlayState::default(),
            route,
            scope,
        });
    }
    fn poll_pending_port_review(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let Some((opening, route, origin_focus)) = self
            .pending_port_review
            .as_ref()
            .map(|pending| (pending.opening, pending.route, pending.origin_focus))
        else {
            return Response::ignored();
        };

        if self.route != route {
            self.pending_port_review.take();
            self.set_status("Port action changed · review again".to_owned());
            return Response::changed();
        }
        match cx.layer_opening(crate::screens::port_args::DIALOG) {
            Some(current) if current == opening => return Response::ignored(),
            Some(_) => {
                self.pending_port_review.take();
                self.set_status("Port input changed · review again".to_owned());
                return Response::changed();
            }
            None => {}
        }

        if cx.area(crate::screens::port_args::DIALOG).is_some()
            || cx
                .state(crate::screens::port_args::DIALOG)
                .contains(termrock::StateFlags::FOCUSED)
            || cx.area(crate::screens::port_args::PORT).is_some()
            || cx
                .state(crate::screens::port_args::PORT)
                .contains(termrock::StateFlags::FOCUSED)
            || cx.top_layer() != termrock::LayerId::PAGE
        {
            return Response::ignored();
        }
        let Some(published_focus) = cx.focused_in_scope(termrock::ScopeId::ROOT) else {
            return Response::ignored();
        };
        if origin_focus.is_some_and(|origin| origin != published_focus) {
            self.pending_port_review.take();
            self.set_status("Port input focus changed · review again".to_owned());
            return Response::changed();
        }

        let Some(pending) = self.pending_port_review.take() else {
            return Response::ignored();
        };
        if self.route != pending.route {
            self.set_status("Port action changed · review again".to_owned());
            return Response::changed();
        }

        let destination = dispatch::review_port_args(&self.world, pending.producer, *pending.plan);
        self.destination(destination, cx);
        Response::changed()
    }
    fn destination(&mut self, destination: Destination, cx: &mut Cx<'_>) {
        match destination {
            Destination::Notice(message) => {
                self.pending_task_run_return = None;
                self.set_status(message);
            }
            Destination::ActivityNotice {
                id,
                message,
                execution_id,
            } => {
                self.pending_task_run_return = None;
                if let Some(snapshot) = self
                    .world
                    .activities
                    .iter()
                    .find(|activity| activity.id == id)
                    .cloned()
                {
                    self.bind_activity_execution(&snapshot, execution_id);
                }
                self.set_status(message);
            }
            Destination::Preview(action) => {
                self.remember_task_run_return(&action, cx);
                self.open_dialog(dialogs::Intent::Preview(action), cx);
            }
            Destination::Trust { action, file, args } => {
                self.remember_task_run_return(&action, cx);
                self.open_dialog(dialogs::Intent::Trust { action, file, args }, cx);
            }
            Destination::TaskRunStart { action, script } => self.start_task_run(action, script, cx),
            Destination::Snapshot(kind) => self.open_snapshot(kind, cx),
            Destination::DiskScan => self.open_disk_scan(cx, None),
            Destination::HostScope => {
                self.home.scope = Some(Scope::Host);
                self.set_status("Scope: host".to_owned());
            }
            Destination::PortArgs(start) => {
                if self.pending_port_review.is_some() || self.port_args_opening.is_some() {
                    self.set_status("Port review is still settling · try again".to_owned());
                    return;
                }
                let dispatch::PortArgsStart {
                    action,
                    producer,
                    session,
                } = *start;
                match PortArgsEditor::new(action, producer, session) {
                    Ok(editor) => {
                        let origin_focus = cx.focused_in_scope(termrock::ScopeId::ROOT);
                        if let Some(token) = editor.open(cx) {
                            self.port_args_opening = Some(PortArgsOpening {
                                token,
                                route: self.route,
                                origin_focus,
                            });
                            self.overlay = Some(Overlay::PortArgs(Box::new(editor)));
                        } else {
                            self.set_status("Port input could not open · try again".to_owned());
                        }
                    }
                    Err(message) => self.set_status(message),
                }
            }
            Destination::TaskArgs(start) => {
                let dispatch::TaskArgsStart {
                    action,
                    producer,
                    session,
                } = *start;
                match TaskArgsEditor::new(action, producer, session) {
                    Ok(editor) => {
                        editor.open(cx);
                        self.overlay = Some(Overlay::Args(Box::new(editor)));
                    }
                    Err(message) => self.set_status(message),
                }
            }
            Destination::PortArgsReview {
                producer,
                plan,
                facts,
            } => {
                self.open_dialog(
                    dialogs::Intent::PortArgsReview {
                        producer,
                        plan: Some(plan),
                        facts,
                    },
                    cx,
                );
            }
            Destination::TaskArgsReview {
                producer,
                plan,
                facts,
            } => {
                self.open_dialog(
                    dialogs::Intent::TaskArgsReview {
                        producer,
                        plan: Some(plan),
                        facts,
                    },
                    cx,
                );
            }
            Destination::Plan(review) => {
                self.plan = Some(PlanState::new(*review));
                self.route = Route::Plan;
                cx.focus(plan::STEPS);
            }
            Destination::Activity {
                id,
                message,
                execution_id,
            } => {
                self.activity_executions.insert(id, execution_id);
                self.activities.show(id);
                self.route = Route::Activity;
                self.set_status(message);
                cx.focus(activity::output_id(id));
            }
            Destination::Database => match (
                pg::Review::new(&self.world, pg::Operation::Cancel),
                pg::Review::new(&self.world, pg::Operation::Terminate),
            ) {
                (Ok(cancel), Ok(terminate)) => self.open_dialog(
                    dialogs::Intent::Database {
                        cancel: Some(cancel),
                        terminate: Some(terminate),
                    },
                    cx,
                ),
                (Err(error), _) | (_, Err(error)) => self.set_status(error.to_string()),
            },
            Destination::Monitor => self.open_dialog(dialogs::Intent::Monitor, cx),
            Destination::Clone => {
                let action = catalogue::catalogue(&self.world)
                    .into_iter()
                    .find(|action| action.id == "gh.clone")
                    .and_then(|action| catalogue::resolve_intent(&self.world, &action).ok())
                    .filter(|action| action.intent_id() == Some("gh.clone"));
                match (self.world.github.clone(), action) {
                    (Some(github), Some(action)) => {
                        let binding = dialogs::CloneBinding::of(&self.world);
                        let picker = ClonePicker::new(&github, action, binding);
                        picker.open(cx);
                        self.overlay = Some(Overlay::Clone(picker));
                    }
                    _ => self.set_status("Clone target changed · review it again".to_owned()),
                }
            }
        }
    }
    fn next_activity(&mut self, cx: &mut Cx<'_>) {
        let ordered = activity::ordered(&self.world);
        let next = (self.route == Route::Activity)
            .then(|| {
                ordered
                    .iter()
                    .position(|activity| Some(activity.id) == self.activities.current_id())
            })
            .flatten()
            .and_then(|index| ordered.get(index.saturating_add(1)))
            .or_else(|| ordered.first())
            .map(|activity| activity.id);
        if let Some(id) = next {
            self.activities.show(id);
            self.route = Route::Activity;
            cx.focus(activity::output_id(id));
        }
    }
    fn capture_return_location(&self, cx: &Cx<'_>) -> ReturnLocation {
        ReturnLocation {
            route: self.route,
            focus: cx.focused_in_scope(termrock::ScopeId::ROOT),
        }
    }

    fn remember_task_run_return(&mut self, action: &crate::domain::action::Action, cx: &Cx<'_>) {
        if action.id == "task:dev" {
            if self.pending_task_run_return.is_none() {
                self.pending_task_run_return = Some(self.capture_return_location(cx));
            }
        } else {
            self.pending_task_run_return = None;
        }
    }

    fn start_task_run(
        &mut self,
        action: crate::domain::action::Action,
        script: TaskScriptId,
        cx: &mut Cx<'_>,
    ) {
        let origin = self
            .pending_task_run_return
            .take()
            .unwrap_or_else(|| self.capture_return_location(cx));
        if script != TaskScriptId::DevServerDemo
            || !dispatch::task_run_source_is_current(&self.world, &action)
        {
            self.set_status("Task source changed · review it again".into());
            return;
        }
        let Some(activity_id) = dispatch::next_activity_id(&self.world) else {
            self.set_status("Activity identity exhausted · nothing started".into());
            return;
        };
        let Some(route_id) = self.next_task_run_route_id else {
            self.set_status("Task route identity exhausted · nothing started".into());
            return;
        };
        let next_route_id = route_id.checked_add(1);
        let started_ms = self.world.now_ms();
        let run_id = match self.task_runs.start(script, started_ms) {
            Ok(run_id) => run_id,
            Err(error) => {
                self.set_status(format!("Task run unavailable · {error}"));
                return;
            }
        };
        let activity = dispatch::task_run_activity(&self.world, &action, activity_id);
        let execution_id = match self.world.activity_supervisor.register(
            &activity,
            ActivityExecutionStart::Running,
            ActivityStopProfile::Resistant,
        ) {
            Ok(execution_id) => execution_id,
            Err(error) => {
                let compensation = self.task_runs.cancel(run_id, started_ms);
                let message = match compensation {
                    Ok(update) => {
                        let terminal = update.events().iter().any(|event| {
                            matches!(
                                event,
                                TaskRunEvent::Cancelled { .. } | TaskRunEvent::Completed { .. }
                            )
                        });
                        if terminal {
                            format!(
                                "Activity registration failed · {error} · TaskRun entry cancelled"
                            )
                        } else {
                            format!(
                                "Activity registration failed · {error} · cancellation incomplete"
                            )
                        }
                    }
                    Err(cancel_error) => format!(
                        "Activity registration failed · {error} · TaskRun compensation failed: {cancel_error}"
                    ),
                };
                self.set_status(message);
                return;
            }
        };
        let title = action.title.clone();
        self.activity_executions.insert(activity_id, execution_id);
        self.world.activities.push(activity.clone());
        self.activities.show(activity_id);
        self.task_run_owners.insert(
            run_id,
            TaskRunOwner {
                route_id,
                execution_id,
                scenario: self.world.scenario,
                cwd: self.world.cwd.clone(),
                effect_revision: self.world.effect_revision,
                action,
                activity,
                origin,
            },
        );
        self.next_task_run_route_id = next_route_id;
        self.retire_task_run_screen(cx);
        let mut screen = TaskRunScreen::new(route_id, run_id, activity_id);
        screen.bind_execution(execution_id);
        self.task_run_screen = Some(screen);
        self.route = Route::TaskRun;
        self.advance_task_runs_to_world();
        self.set_status(format!("Activity started: {title} · simulated"));
    }

    fn retire_task_run_screen(&mut self, cx: &mut Cx<'_>) {
        let Some(mut screen) = self.task_run_screen.take() else {
            return;
        };
        screen.retire(cx);
        self.retired_task_run_screens.push(screen);
    }

    fn poll_retired_task_run_screens(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        self.retired_task_run_screens.retain_mut(|screen| {
            let (screen_response, ready) = screen.poll_retired(cx);
            response |= screen_response;
            !ready
        });
        response
    }

    fn bind_activity_execution(&mut self, snapshot: &Activity, execution_id: ActivityExecutionId) {
        if self.activity_snapshot_is_unique(snapshot)
            && self
                .world
                .activity_supervisor
                .current_execution_id(snapshot.id)
                == Some(execution_id)
            && self
                .world
                .activity_supervisor
                .execution(execution_id)
                .is_some_and(|execution| {
                    execution.id() == execution_id && execution.snapshot() == snapshot
                })
        {
            self.activity_executions.insert(snapshot.id, execution_id);
        }
    }

    fn advance_task_runs_to_world(&mut self) {
        let now_ms = self.world.now_ms();
        let updates = self.task_runs.advance_to(now_ms);
        self.observe_task_run_updates(&updates, false);
    }

    fn observe_task_run_updates(&mut self, updates: &[TaskRunUpdate], observe_cancelled: bool) {
        for update in updates {
            let terminal = update.events().iter().any(|event| {
                matches!(event, TaskRunEvent::Completed { .. })
                    || observe_cancelled && matches!(event, TaskRunEvent::Cancelled { .. })
            });
            if terminal {
                self.observe_task_run_terminal(update.run_id());
            }
        }
    }

    fn observe_task_run_terminal(&mut self, run_id: TaskRunId) {
        let Some(owner) = self.task_run_owners.get(&run_id) else {
            return;
        };
        let snapshot = owner.activity.clone();
        let execution_id = owner.execution_id;
        if execution_id.activity_id() != snapshot.id || !self.activity_snapshot_is_unique(&snapshot)
        {
            return;
        }
        let _ = self
            .world
            .activity_supervisor
            .observe_terminal_exit(execution_id, &snapshot);
    }

    fn activity_snapshot_is_unique(&self, snapshot: &Activity) -> bool {
        let mut matching = self
            .world
            .activities
            .iter()
            .filter(|activity| activity.id == snapshot.id);
        matches!(matching.next(), Some(activity) if activity == snapshot)
            && matching.next().is_none()
    }

    fn activity_projections(&self) -> BTreeMap<u32, activity::ActivityProjection> {
        let mut projections = BTreeMap::new();
        for snapshot in &self.world.activities {
            let execution = self
                .activity_executions
                .get(&snapshot.id)
                .and_then(|id| self.world.activity_supervisor.execution(*id))
                .filter(|execution| {
                    self.world
                        .activity_supervisor
                        .current_execution_id(snapshot.id)
                        == Some(execution.id())
                        && execution.snapshot() == snapshot
                });
            let mut owners = self
                .task_run_owners
                .iter()
                .filter(|(_, owner)| owner.activity.id == snapshot.id);
            let first_owner = owners.next();
            let has_extra_owner = owners.next().is_some();
            let (run, link_valid) = match first_owner {
                None => (None, true),
                Some((run_id, owner))
                    if !has_extra_owner
                        && owner.activity == *snapshot
                        && self.task_run_owner_execution_is_current(owner) =>
                {
                    (
                        self.task_runs.get(*run_id),
                        self.task_run_owner_is_live(owner.route_id, *run_id),
                    )
                }
                _ => (None, false),
            };
            let mut projection = activity::project(snapshot, execution, run, self.world.now_ms());
            if !link_valid {
                projection.stop_enabled = false;
            }
            projections.insert(snapshot.id, projection);
        }
        projections
    }

    fn task_run_owner_is_live(&self, route_id: u64, run_id: TaskRunId) -> bool {
        let Some(owner) = self.task_run_owners.get(&run_id) else {
            return false;
        };
        if owner.route_id != route_id
            || owner.scenario != self.world.scenario
            || owner.cwd != self.world.cwd
            || owner.effect_revision != self.world.effect_revision
            || self.task_runs.get(run_id).is_none()
            || !dispatch::task_run_source_is_current(&self.world, &owner.action)
            || !self.task_run_owner_execution_is_current(owner)
        {
            return false;
        }
        true
    }

    fn task_run_owner_execution_is_current(&self, owner: &TaskRunOwner) -> bool {
        owner.execution_id.activity_id() == owner.activity.id
            && self.activity_executions.get(&owner.activity.id).copied() == Some(owner.execution_id)
            && self
                .world
                .activity_supervisor
                .current_execution_id(owner.activity.id)
                == Some(owner.execution_id)
            && self.activity_snapshot_is_unique(&owner.activity)
            && self
                .world
                .activity_supervisor
                .execution(owner.execution_id)
                .is_some_and(|execution| {
                    execution.id() == owner.execution_id && execution.snapshot() == &owner.activity
                })
    }

    fn task_run_screen_matches(&self, route_id: u64, run_id: TaskRunId) -> bool {
        self.task_run_screen
            .as_ref()
            .is_some_and(|screen| screen.route_id() == route_id && screen.run_id() == run_id)
    }

    fn task_run_owner_binding_is_current(&self, route_id: u64, run_id: TaskRunId) -> bool {
        self.task_run_screen_matches(route_id, run_id)
            && self.task_run_screen.as_ref().is_some_and(|screen| {
                self.task_run_owners.get(&run_id).is_some_and(|owner| {
                    screen.activity_id() == owner.activity.id
                        && screen.execution_id() == Some(owner.execution_id)
                })
            })
            && self.task_run_owner_is_live(route_id, run_id)
    }

    fn task_run_route_is_current(&self, route_id: u64, run_id: TaskRunId) -> bool {
        self.route == Route::TaskRun && self.task_run_owner_binding_is_current(route_id, run_id)
    }

    fn handle_activity_stop_event(&mut self, event: activity::StopEvent, controls_visible: bool) {
        if !controls_visible
            || self.route != Route::Activity
            || self.overlay.is_some()
            || self.retired_overlay.is_some()
            || self.pending_port_review.is_some()
            || self.port_args_opening.is_some()
            || self.menu.is_open()
            || self.activities.current_id() != Some(event.snapshot.id)
            || self.activities.current_view_token() != Some(event.view_token)
            || !self.activity_snapshot_is_unique(&event.snapshot)
            || self.activity_executions.get(&event.snapshot.id).copied() != Some(event.execution_id)
            || self
                .world
                .activity_supervisor
                .current_execution_id(event.snapshot.id)
                != Some(event.execution_id)
        {
            self.set_status("Activity view changed · stop ignored".into());
            return;
        }
        let linked_run = match self.linked_task_run_for_activity(&event.snapshot) {
            Ok(run_id) => run_id,
            Err(message) => {
                self.set_status(message);
                return;
            }
        };
        self.request_activity_stop(event.execution_id, &event.snapshot, linked_run);
    }

    fn handle_task_run_stop_event(
        &mut self,
        route_id: u64,
        run_id: TaskRunId,
        execution_id: ActivityExecutionId,
        controls_visible: bool,
    ) {
        let Some(owner) = self.task_run_owners.get(&run_id) else {
            self.set_status("Task run owner changed · stop ignored".into());
            return;
        };
        let snapshot = owner.activity.clone();
        let screen_matches = self.task_run_screen.as_ref().is_some_and(|screen| {
            screen.route_id() == route_id
                && screen.run_id() == run_id
                && screen.activity_id() == snapshot.id
                && screen.execution_id() == Some(execution_id)
        });
        if !controls_visible
            || self.route != Route::TaskRun
            || !screen_matches
            || !self.task_run_owner_binding_is_current(route_id, run_id)
            || self.activity_executions.get(&snapshot.id).copied() != Some(execution_id)
            || !self.activity_snapshot_is_unique(&snapshot)
        {
            self.set_status("Task run view changed · stop ignored".into());
            return;
        }
        self.request_activity_stop(execution_id, &snapshot, Some(run_id));
    }

    fn linked_task_run_for_activity(
        &self,
        snapshot: &Activity,
    ) -> Result<Option<TaskRunId>, String> {
        let mut owners = self
            .task_run_owners
            .iter()
            .filter(|(_, owner)| owner.activity.id == snapshot.id);
        let first_owner = owners.next();
        let has_extra_owner = owners.next().is_some();
        match first_owner {
            None => Ok(None),
            Some((run_id, owner))
                if !has_extra_owner
                    && owner.activity == *snapshot
                    && owner.execution_id.activity_id() == snapshot.id
                    && self.task_run_owner_is_live(owner.route_id, *run_id) =>
            {
                Ok(Some(*run_id))
            }
            _ => Err("Task run owner changed · stop unavailable".into()),
        }
    }

    fn request_activity_stop(
        &mut self,
        execution_id: ActivityExecutionId,
        snapshot: &Activity,
        linked_run: Option<TaskRunId>,
    ) {
        if !self.activity_snapshot_is_unique(snapshot)
            || self.activity_executions.get(&snapshot.id).copied() != Some(execution_id)
            || self
                .world
                .activity_supervisor
                .current_execution_id(snapshot.id)
                != Some(execution_id)
        {
            self.set_status("Activity changed · stop ignored".into());
            return;
        }
        let Some(execution) = self.world.activity_supervisor.execution(execution_id) else {
            self.set_status("Activity execution changed · stop ignored".into());
            return;
        };
        if execution.snapshot() != snapshot {
            self.set_status("Activity changed · stop ignored".into());
            return;
        }
        if execution.phase() == ActivityExecutionPhase::Cancelling {
            self.set_status("Activity is stopping · its grace period is still active".into());
            return;
        }
        if let Some(run_id) = linked_run {
            if !self
                .task_run_owners
                .get(&run_id)
                .is_some_and(|owner| owner.execution_id == execution_id)
            {
                self.set_status("Task run owner changed · stop ignored".into());
                return;
            }
            let Some(run) = self.task_runs.get(run_id) else {
                self.set_status("Task run owner changed · stop ignored".into());
                return;
            };
            if matches!(
                run.state(),
                TaskRunState::Completed | TaskRunState::Cancelled
            ) {
                let _ = self
                    .world
                    .activity_supervisor
                    .observe_terminal_exit(execution_id, snapshot);
                self.set_status("Task run already ended · no stop grace started".into());
                return;
            }
        }

        let now_ms = self.world.now_ms();
        let admission =
            match self
                .world
                .activity_supervisor
                .admit_stop(execution_id, snapshot, now_ms)
            {
                Ok(admission) => admission,
                Err(error) => {
                    self.set_status(format!("Activity stop unavailable · {error}"));
                    return;
                }
            };

        if let Some(run_id) = linked_run {
            match self.task_runs.cancel(run_id, now_ms) {
                Ok(update) => {
                    let completed = update
                        .events()
                        .iter()
                        .any(|event| matches!(event, TaskRunEvent::Completed { .. }));
                    let cancelled = update
                        .events()
                        .iter()
                        .any(|event| matches!(event, TaskRunEvent::Cancelled { .. }));
                    if completed {
                        let _ = admission;
                        self.observe_task_run_updates(&[update], false);
                        self.set_status(
                            "Task run completed before Stop · execution ownership released".into(),
                        );
                        return;
                    }
                    if !cancelled {
                        let _ = admission;
                        self.set_status(
                            "Task cancellation did not complete · stop not committed".into(),
                        );
                        return;
                    }
                }
                Err(error) => {
                    let _ = admission;
                    self.set_status(format!("Task cancellation failed · {error}"));
                    return;
                }
            }
        }

        match admission.commit() {
            ActivityStopOutcome::CancelledBeforeStart => {
                self.set_status("Activity cancelled before start · simulated".into());
            }
            ActivityStopOutcome::TermSent => {
                self.set_status(
                    "Activity stopping · TaskRun output frozen · execution remains owned".into(),
                );
            }
            _ => self.set_status("Activity stop outcome changed · refresh the view".into()),
        }
    }

    fn task_run_prompt_event_is_current(
        &self,
        route_id: u64,
        run_id: TaskRunId,
        prompt_id: PromptId,
    ) -> bool {
        self.task_run_route_is_current(route_id, run_id)
            && self
                .task_run_screen
                .as_ref()
                .is_some_and(|screen| screen.current_prompt_id() == Some(prompt_id))
            && self
                .task_runs
                .get(run_id)
                .and_then(TaskRun::active_prompt)
                .is_some_and(|prompt| prompt.id() == prompt_id)
    }

    fn invalidate_task_run_route(&mut self, cx: &mut Cx<'_>) {
        self.retire_task_run_screen(cx);
        self.route = Route::Home;
        self.pending_task_run_return = None;
        self.disk_scan_task_run_return = None;
        cx.focus(home::QUERY);
        self.set_status("Task run context changed · reopen it".into());
    }

    fn update_task_run_screen(
        &mut self,
        cx: &mut Cx<'_>,
        allow_input: bool,
    ) -> (Response<()>, Option<task_run::Event>) {
        let Some((route_id, run_id)) = self
            .task_run_screen
            .as_ref()
            .map(|screen| (screen.route_id(), screen.run_id()))
        else {
            self.invalidate_task_run_route(cx);
            return (Response::changed(), None);
        };
        if !self.task_run_route_is_current(route_id, run_id) {
            self.invalidate_task_run_route(cx);
            return (Response::changed(), None);
        }
        let Some(run) = self.task_runs.get(run_id) else {
            self.invalidate_task_run_route(cx);
            return (Response::changed(), None);
        };
        let projections = self.activity_projections();
        let projection = self
            .task_run_owners
            .get(&run_id)
            .and_then(|owner| projections.get(&owner.activity.id))
            .cloned()
            .unwrap_or_else(|| {
                self.task_run_owners.get(&run_id).map_or_else(
                    || activity::ActivityProjection {
                        execution_id: None,
                        state_label: "unavailable".into(),
                        stop_grace_ms: None,
                        simulated_stop_history: None,
                        stop_enabled: false,
                        lines: Vec::new(),
                    },
                    |owner| {
                        activity::project(&owner.activity, None, Some(run), self.world.now_ms())
                    },
                )
            });
        let stop_enabled = projection.stop_enabled
            && self
                .task_run_screen
                .as_ref()
                .is_some_and(|screen| screen.execution_id() == projection.execution_id);
        let stop_label = if stop_enabled {
            "Stop"
        } else if projection.state_label == "stopping" {
            "Stopping…"
        } else {
            "Stopped"
        };
        let viewport = cx.viewport();
        let body = if too_small(viewport) {
            Rect::default()
        } else {
            shell_areas(viewport).0
        };
        self.task_run_screen
            .as_mut()
            .map_or((Response::ignored(), None), |screen| {
                screen.update(cx, run, body, allow_input, stop_enabled, stop_label)
            })
    }

    fn poll_covered_task_run_screen(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.task_run_screen
            .as_mut()
            .map_or_else(Response::ignored, |screen| screen.poll_covered(cx))
    }

    fn handle_task_run_event(
        &mut self,
        event: task_run::Event,
        cx: &mut Cx<'_>,
        controls_visible: bool,
    ) {
        match event {
            task_run::Event::Answer {
                route_id,
                run_id,
                prompt_id,
                answer,
            } => {
                if self.route != Route::TaskRun || !self.task_run_screen_matches(route_id, run_id) {
                    return;
                }
                if !self.task_run_owner_binding_is_current(route_id, run_id) {
                    self.invalidate_task_run_route(cx);
                    return;
                }
                if !self.task_run_prompt_event_is_current(route_id, run_id, prompt_id) {
                    self.set_status("Task prompt changed · answer ignored".into());
                    return;
                }
                match self
                    .task_runs
                    .answer_prompt(run_id, prompt_id, &answer, self.world.now_ms())
                {
                    Ok(update) => self.observe_task_run_updates(&[update], false),
                    Err(error) => {
                        self.set_status(format!("Task prompt changed · {error}"));
                    }
                }
            }
            task_run::Event::Cancel {
                route_id,
                run_id,
                prompt_id,
            } => {
                self.cancel_task_run_prompt(route_id, run_id, prompt_id, cx);
            }
            task_run::Event::Stop {
                route_id,
                run_id,
                execution_id,
            } => self.handle_task_run_stop_event(route_id, run_id, execution_id, controls_visible),
        }
    }

    fn cancel_task_run_prompt(
        &mut self,
        route_id: u64,
        run_id: TaskRunId,
        prompt_id: crate::sim::task_run::PromptId,
        cx: &mut Cx<'_>,
    ) {
        if self.route != Route::TaskRun || !self.task_run_screen_matches(route_id, run_id) {
            return;
        }
        if !self.task_run_owner_binding_is_current(route_id, run_id) {
            self.invalidate_task_run_route(cx);
            return;
        }
        if !self.task_run_prompt_event_is_current(route_id, run_id, prompt_id) {
            self.set_status("Task prompt changed · cancellation ignored".into());
            return;
        }
        match self.task_runs.cancel(run_id, self.world.now_ms()) {
            Ok(update) => {
                let completed = update
                    .events()
                    .iter()
                    .any(|event| matches!(event, TaskRunEvent::Completed { .. }));
                self.observe_task_run_updates(&[update], true);
                if completed {
                    self.set_status("Task run completed · simulated · no process executed".into());
                } else {
                    self.set_status("Task run cancelled · simulated · no process executed".into());
                }
            }
            Err(error) => self.set_status(format!("Task cancellation failed · {error}")),
        }
    }

    fn cancel_current_task_run_prompt(&mut self, cx: &mut Cx<'_>) -> bool {
        if self.route != Route::TaskRun
            || cx.top_layer() != termrock::LayerId::PAGE
            || self.overlay.is_some()
            || self.retired_overlay.is_some()
            || self.pending_port_review.is_some()
            || self.port_args_opening.is_some()
            || self.menu.is_open()
        {
            return false;
        }
        let Some((route_id, run_id, input_id)) = self.task_run_screen.as_ref().and_then(|screen| {
            screen
                .current_input_id()
                .map(|input_id| (screen.route_id(), screen.run_id(), input_id))
        }) else {
            return false;
        };
        let viewport = cx.viewport();
        let body = if too_small(viewport) {
            Rect::default()
        } else {
            shell_areas(viewport).0
        };
        let input_visible = self
            .task_run_screen
            .as_ref()
            .and_then(|screen| screen.input_area(body))
            .is_some_and(|expected| cx.area(input_id) == Some(expected));
        if !input_visible || !cx.state(input_id).contains(termrock::StateFlags::FOCUSED) {
            return false;
        }
        if !self.task_run_route_is_current(route_id, run_id) {
            self.invalidate_task_run_route(cx);
            return true;
        }
        let Some(prompt_id) = self
            .task_runs
            .get(run_id)
            .and_then(TaskRun::active_prompt)
            .map(|prompt| prompt.id())
        else {
            return false;
        };
        self.cancel_task_run_prompt(route_id, run_id, prompt_id, cx);
        true
    }

    fn leave_task_run(&mut self, cx: &mut Cx<'_>) {
        let Some((route_id, run_id)) = self
            .task_run_screen
            .as_ref()
            .map(|screen| (screen.route_id(), screen.run_id()))
        else {
            self.invalidate_task_run_route(cx);
            return;
        };
        if !self.task_run_route_is_current(route_id, run_id) {
            self.invalidate_task_run_route(cx);
            return;
        }
        let origin = self.task_run_owners.get(&run_id).map(|owner| owner.origin);
        self.retire_task_run_screen(cx);
        self.disk_scan_task_run_return = None;
        if let Some(location) = origin
            && let Some(focus) = self.valid_disk_scan_return_focus(location)
        {
            self.route = location.route;
            cx.focus(focus);
        } else {
            self.route = Route::Home;
            cx.focus(home::QUERY);
        }
    }

    fn valid_task_run_return_focus(&self, location: ReturnLocation) -> Option<Id> {
        if location.route != Route::TaskRun {
            return None;
        }
        let binding = self.disk_scan_task_run_return.as_ref()?;
        if !self.task_run_owner_binding_is_current(binding.route_id, binding.run_id) {
            return None;
        }
        let prompt_id = self
            .task_runs
            .get(binding.run_id)
            .and_then(TaskRun::active_prompt)
            .map(|prompt| prompt.id());
        if prompt_id != binding.prompt_id {
            return None;
        }
        let focus = location.focus?;
        if focus == MENU {
            return Some(focus);
        }
        let prompt_id = prompt_id?;
        let expected =
            TaskRunScreen::input_id_for(binding.route_id, binding.run_id.get(), prompt_id.get());
        (focus == expected).then_some(focus)
    }

    fn open_snapshot(&mut self, kind: snapshot::Kind, cx: &mut Cx<'_>) {
        if self.route == Route::Snapshot || self.overlay.is_some() || self.retired_overlay.is_some()
        {
            return;
        }
        self.snapshot_return = Some(self.capture_return_location(cx));
        self.snapshot_kind = kind;
        self.route = Route::Snapshot;
        cx.focus(if kind == snapshot::Kind::Processes {
            snapshot::PROCESSES
        } else if self.world.scenario == Scenario::FirstUse {
            snapshot::RESOURCES_BODY
        } else {
            MENU
        });
    }

    fn open_disk_scan(&mut self, cx: &mut Cx<'_>, menu_origin: Option<ReturnLocation>) {
        if self.route == Route::DiskScan
            || self.overlay.is_some()
            || self.retired_overlay.is_some()
            || self.pending_port_review.is_some()
            || self.port_args_opening.is_some()
        {
            return;
        }
        let return_location = menu_origin
            .filter(|location| location.route == self.route && location.focus.is_some())
            .unwrap_or_else(|| self.capture_return_location(cx));
        let task_run_return = if return_location.route == Route::TaskRun {
            self.task_run_screen.as_ref().and_then(|screen| {
                let route_id = screen.route_id();
                let run_id = screen.run_id();
                let prompt_id = self
                    .task_runs
                    .get(run_id)
                    .and_then(TaskRun::active_prompt)
                    .map(|prompt| prompt.id());
                self.task_run_owner_binding_is_current(route_id, run_id)
                    .then_some(TaskRunOverlayReturn {
                        route_id,
                        run_id,
                        prompt_id,
                    })
            })
        } else {
            None
        };
        match self.disk_scan.open(&self.world) {
            Ok(_) => {
                self.disk_scan_return = Some(return_location);
                self.disk_scan_task_run_return = task_run_return;
                self.route = Route::DiskScan;
                if let Some(tree) = self.disk_scan.tree_owner() {
                    cx.focus(tree);
                }
            }
            Err(error) => self.set_status(format!("Modeled disk scan unavailable · {error}")),
        }
    }

    fn return_from_disk_scan(&mut self, cx: &mut Cx<'_>) {
        self.disk_scan.leave(&self.world);
        if let Some(location) = self.disk_scan_return.take() {
            self.route = location.route;
            if let Some(focus) = self.valid_disk_scan_return_focus(location) {
                cx.focus(focus);
            } else {
                self.route = Route::Home;
                cx.focus(home::QUERY);
            }
        } else {
            self.route = Route::Home;
            cx.focus(home::QUERY);
        }
        self.disk_scan_task_run_return = None;
    }

    fn valid_disk_scan_return_focus(&self, location: ReturnLocation) -> Option<Id> {
        match location.route {
            Route::Home => location
                .focus
                .filter(|focus| *focus == home::QUERY || *focus == home::ROWS || *focus == MENU),
            Route::Plan if self.plan.is_some() => location
                .focus
                .filter(|focus| *focus == plan::STEPS || *focus == MENU),
            Route::Plan => None,
            Route::Activity => {
                let activity_id = self.activities.current_id()?;
                let current = self
                    .activities
                    .current(&self.world)
                    .is_some_and(|activity| activity.id == activity_id);
                let focus = location.focus?;
                (current && (focus == activity::output_id(activity_id) || focus == MENU))
                    .then_some(focus)
            }
            Route::Files => Some(files::LIST),

            Route::Disk | Route::Cleanup | Route::Review | Route::Snapshot => {
                location.focus.filter(|focus| *focus == MENU)
            }
            Route::DiskScan => None,
            Route::TaskRun => self.valid_task_run_return_focus(location),
        }
    }

    fn has_running_work(&self) -> bool {
        let live_activity = self.world.activities.iter().any(|activity| {
            match self
                .world
                .activity_supervisor
                .current_execution_id(activity.id)
            {
                Some(execution_id) => self
                    .world
                    .activity_supervisor
                    .execution(execution_id)
                    .is_none_or(|execution| execution.is_owned()),
                None => matches!(
                    activity.state,
                    ActivityState::Running | ActivityState::Waiting | ActivityState::Detached
                ),
            }
        });
        let live_task_run = self.task_run_owners.keys().any(|run_id| {
            self.task_runs.get(*run_id).is_some_and(|run| {
                matches!(run.state(), TaskRunState::Running | TaskRunState::Waiting)
            })
        });
        live_activity || live_task_run
    }

    fn command(&mut self, command: ActionKey, cx: &mut Cx<'_>) {
        match command {
            HELP => self.open_help(cx),
            TOP_FILES => self.open_dialog(dialogs::Intent::TopFiles, cx),
            CLEANUP_REVIEW => self.open_dialog(dialogs::Intent::CleanupReview, cx),
            FOLDER_BROWSER => {
                if self.route != Route::Files {
                    self.files_return = Some(self.capture_return_location(cx));
                }
                self.open_dialog(dialogs::Intent::FolderBrowser, cx);
            }
            DISK_SELECT if self.route == Route::Disk => {
                if let Some(selection) = disk::SelectionChooser::open(&self.world, cx) {
                    self.overlay = Some(Overlay::DiskSelection(selection));
                } else {
                    self.set_status(
                        "Disk candidates are unavailable · review the current inventory".into(),
                    );
                }
            }
            FILES_OPEN if self.route == Route::Files => self.files.open_selected(cx),
            FILES_PARENT if self.route == Route::Files => {
                if !self.files.close_preview(cx) {
                    self.files.go_parent();
                }
            }
            FILES_HIDDEN if self.route == Route::Files => self.files.toggle_hidden(),
            HOME_FIND => self.open_dialog(dialogs::Intent::HomeFind, cx),
            CUSTOM_ACTION => self.open_dialog(dialogs::Intent::CustomAction, cx),
            SNAPSHOT_PAGE => self.open_dialog(dialogs::Intent::SnapshotPage, cx),
            DEFINITION_TRUST => self.open_dialog(dialogs::Intent::DefinitionTrust, cx),
            ABOUT => self.open_dialog(dialogs::Intent::About, cx),
            QUIT if self.has_running_work() => {
                self.open_dialog(dialogs::Intent::Quit, cx);
            }
            QUIT => self.quit = true,
            INTERRUPT => {
                if !self.cancel_current_task_run_prompt(cx) {
                    self.quit = true;
                }
            }
            HOME => {
                self.snapshot_return = None;
                if self.route == Route::TaskRun {
                    self.retire_task_run_screen(cx);
                    self.pending_task_run_return = None;
                    self.disk_scan_task_run_return = None;
                }
                if self.route == Route::DiskScan {
                    self.disk_scan.leave(&self.world);
                    self.disk_scan_return = None;
                }
                self.route = Route::Home;
                cx.focus(home::QUERY);
            }
            DISK_SCAN => self.open_disk_scan(cx, None),
            OPEN_MENU => {
                let _ = menu_bar().open_menu(cx, &mut self.menu, 0);
            }
            NEXT_ACTIVITY => self.next_activity(cx),
            SCOPE if self.route == Route::Home => {
                self.home.scope = self.home.scope.map_or_else(
                    || Scope::ORDER.first().copied(),
                    |scope| {
                        Scope::ORDER
                            .iter()
                            .position(|current| *current == scope)
                            .and_then(|index| Scope::ORDER.get(index.saturating_add(1)).copied())
                    },
                );
                self.set_status(format!(
                    "Scope: {}",
                    self.home.scope.map_or("all", Scope::label)
                ));
            }
            PREVIEW if self.route == Route::Home => {
                if let Some(action) = self.home.subject(&self.world, cx) {
                    let destination = dispatch::preview(&mut self.world, &action);
                    self.destination(destination, cx);
                }
            }
            ACTIONS if self.route == Route::Home => {
                if let Some(action) = self.home.subject(&self.world, cx) {
                    self.pending_task_run_return = Some(self.capture_return_location(cx));
                    let picker = Actions::new(action, &self.world);
                    picker.open(cx);
                    self.overlay = Some(Overlay::Actions(picker));
                }
            }
            TOGGLE
                if self.route == Route::Plan
                    && cx
                        .state(plan::STEPS)
                        .contains(termrock::StateFlags::FOCUSED) =>
            {
                if let Some(plan) = &mut self.plan {
                    let message = plan.toggle_selected();
                    self.set_status(message);
                }
            }
            CONTINUE if self.route == Route::Plan => {
                if let Some(plan) = &self.plan {
                    self.plan_event(plan.continuation(), cx);
                }
            }
            RESULTS if self.route == Route::Home => self.home.focus_first(&self.world, cx),
            PREVIOUS if self.route == Route::Home => cx.focus_prev(),
            SNAPSHOT_FOCUS_NEXT
                if self.route == Route::Snapshot
                    && self.world.scenario == Scenario::FirstUse
                    && self.snapshot_kind == snapshot::Kind::Resources =>
            {
                cx.focus_next();
            }
            SNAPSHOT_FOCUS_PREVIOUS
                if self.route == Route::Snapshot
                    && self.world.scenario == Scenario::FirstUse
                    && self.snapshot_kind == snapshot::Kind::Resources =>
            {
                cx.focus_prev();
            }
            SNAPSHOT_COPY if self.snapshot_copy_allowed(cx) => {
                if let Some(event) = snapshot::copy_resource_body(&self.world) {
                    self.snapshot_event(event, cx);
                }
            }
            QUERY_ESCAPE if self.route == Route::Home => {
                if self.home.scope.take().is_some() {
                    self.set_status("Scope: all".into());
                } else {
                    self.home.clear_query(&self.world);
                }
            }
            command if self.route == Route::Activity => {
                if let Some(index) = ACTIVITY_SHORTCUTS
                    .iter()
                    .position(|(_, key)| *key == command)
                {
                    if let Some(activity) = activity::ordered(&self.world).get(index) {
                        let id = activity.id;
                        self.activities.show(id);
                        cx.focus(activity::output_id(id));
                    } else {
                        self.set_status(format!("No activity {}", index.saturating_add(1)));
                    }
                }
            }
            _ => {}
        }
    }
    fn action_choice(
        &mut self,
        choice: Choice,
        action: crate::domain::action::Action,
        cx: &mut Cx<'_>,
    ) {
        if !matches!(choice, Choice::Run | Choice::Preview) {
            self.pending_task_run_return = None;
        }
        match choice {
            Choice::Run => {
                let destination = dispatch::invoke(&mut self.world, &action);
                self.destination(destination, cx);
            }
            Choice::Preview => {
                let destination = dispatch::preview(&mut self.world, &action);
                self.destination(destination, cx);
            }
            Choice::Copy => self.copy_osc52(&action.command),
            Choice::Alias => self.open_dialog(dialogs::Intent::Alias(action), cx),
            Choice::Pin => {
                if self.world.memory.pin_at(&self.world.cwd, &action.command) {
                    self.world
                        .memory
                        .pins
                        .retain(|pin| pin.path != self.world.cwd || pin.command != action.command);
                    self.set_status(format!("Unpinned {}", action.command));
                } else {
                    self.world.memory.pins.push(Pin {
                        path: self.world.cwd.clone(),
                        command: action.command.clone(),
                    });
                    self.set_status(format!("Pinned {} here", action.command));
                }
            }
            Choice::Hide => {
                if self
                    .world
                    .memory
                    .hidden_at(&self.world.cwd, &action.command)
                {
                    self.world
                        .memory
                        .hides
                        .retain(|pin| pin.path != self.world.cwd || pin.command != action.command);
                    self.set_status(format!("Unhidden {} here", action.command));
                } else {
                    self.world.memory.hides.push(Pin {
                        path: self.world.cwd.clone(),
                        command: action.command.clone(),
                    });
                    self.set_status(format!(
                        "Hidden {} here · Reset ranking restores",
                        action.command
                    ));
                }
            }
            Choice::Reset => {
                self.world.memory.reset_at(&self.world.cwd, &action.command);
                self.set_status(format!(
                    "Reset ranking for {} · pins, aliases, hides cleared",
                    action.command
                ));
            }
        }
    }
    fn update_overlay(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let Some(mut overlay) = self.overlay.take() else {
            return Response::ignored();
        };
        let (response, closed) = match &mut overlay {
            Overlay::Dialog(dialog) => {
                let is_folder_browser = dialog.is_folder_browser();
                let (response, event) = dialog.update(cx, &mut self.world);
                let closed = event.is_some();
                if let Some(event) = event {
                    match event {
                        dialogs::Event::Close => {
                            self.pending_task_run_return = None;
                            if is_folder_browser && self.route != Route::Files {
                                self.files_return = None;
                            }
                        }
                        dialogs::Event::BrowseFiles => {
                            self.route = Route::Files;
                            cx.focus(files::LIST);
                        }
                        dialogs::Event::Quit => self.quit = true,
                        dialogs::Event::Destination(destination) => {
                            self.destination(*destination, cx);
                        }
                        dialogs::Event::OpenCloneOptions(request) => {
                            self.open_dialog(dialogs::Intent::CloneOptions(request), cx);
                        }
                        dialogs::Event::ReturnToCloneReview(request) => {
                            self.open_dialog(dialogs::Intent::CloneReview(request), cx);
                        }
                    }
                }
                (response, closed)
            }
            Overlay::Help {
                state,
                route,
                scope,
            } => {
                let response = help_overlay(*route, scope).update(cx, state);
                let closed = matches!(response.action_ref(), Some(termrock::HelpAction::Closed(_)));
                (response.erase(), closed)
            }
            Overlay::PortArgs(editor) => {
                let (response, event) = editor.update(cx, &self.world);
                let closed = event.is_some();
                if let Some(event) = event {
                    match event {
                        crate::screens::port_args::Event::Cancelled => {
                            self.port_args_opening.take();
                        }
                        crate::screens::port_args::Event::Rejected(message) => {
                            self.port_args_opening.take();
                            self.set_status(message);
                        }
                        crate::screens::port_args::Event::Accepted { producer, plan } => {
                            if let Some(opening) = self.port_args_opening.take() {
                                if opening.route != self.route || self.pending_port_review.is_some()
                                {
                                    self.set_status("Port input changed · review again".to_owned());
                                } else {
                                    self.pending_port_review = Some(PendingPortReview {
                                        opening: opening.token,
                                        route: opening.route,
                                        origin_focus: opening.origin_focus,
                                        producer,
                                        plan,
                                    });
                                }
                            } else {
                                self.set_status("Port input changed · review again".to_owned());
                            }
                        }
                    }
                }
                (response, closed)
            }
            Overlay::Args(editor) => {
                let (response, event) = editor.update(cx, &self.world);
                let closed = event.is_some();
                if let Some(event) = event {
                    match event {
                        crate::screens::args::Event::Cancelled => {}
                        crate::screens::args::Event::Rejected(message) => {
                            self.set_status(message);
                        }
                        crate::screens::args::Event::Accepted { producer, plan } => {
                            let destination =
                                dispatch::review_task_args(&self.world, producer, *plan);
                            self.destination(destination, cx);
                        }
                    }
                }
                (response, closed)
            }
            Overlay::Clone(picker) => {
                let (response, chosen, closed) = picker.update(cx);
                let done = chosen.is_some() || closed;
                if let Some((repo, login, action, github, binding)) = chosen {
                    let request = dialogs::CloneRequest::new(repo, login, action, github, binding);
                    if request.target_is_current(&self.world) {
                        self.open_dialog(dialogs::Intent::CloneReview(request), cx);
                    } else {
                        self.set_status("Clone target changed · review it again".to_owned());
                    }
                }
                (response, done)
            }
            Overlay::DiskSelection(selection) => {
                let (response, event, closed) = selection.update(cx, &self.world);
                let done = event.is_some() || closed;
                if let Some(event) = event {
                    match event {
                        disk::SelectionEvent::Cancelled => {}
                        disk::SelectionEvent::Stale => self.set_status(
                            "Disk inventory changed · reopen candidate selection".to_owned(),
                        ),
                        disk::SelectionEvent::Invalid => self
                            .set_status("Disk selection is invalid · nothing changed".to_owned()),
                        disk::SelectionEvent::Review(review) => {
                            self.destination(Destination::Plan(Box::new(review)), cx);
                        }
                    }
                }
                (response, done)
            }
            Overlay::Actions(picker) => {
                let (response, choice, closed) = picker.update(cx);
                let done = choice.is_some() || closed;
                if let Some(choice) = choice {
                    match picker.reviewed_action(&self.world) {
                        Ok(action) => self.action_choice(choice, action, cx),
                        Err(reason) => {
                            self.pending_task_run_return = None;
                            self.set_status(reason.into());
                        }
                    }
                } else if closed {
                    self.pending_task_run_return = None;
                }
                (response, done)
            }
            Overlay::Gate(gate) => {
                if let Some(plan) = &mut self.plan {
                    let (response, result) = gate.update(cx, &mut plan.review, &mut self.world);
                    let done = result.is_some();
                    if let Some(message) = result {
                        self.set_status(message);
                    }
                    (response, done)
                } else {
                    (Response::ignored(), true)
                }
            }
        };
        if !closed && self.overlay.is_none() {
            self.overlay = Some(overlay);
        } else {
            self.retired_overlay = Some(overlay);
        }
        response
    }
    fn strip_update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        for intent in cx.intents(STRIP) {
            if let Intent::Pointer {
                phase: Phase::Click | Phase::DoubleClick,
                part: PartRef {
                    item: Some(key), ..
                },
                ..
            } = intent
                && let Some(id) = self
                    .world
                    .activities
                    .iter()
                    .find(|activity| ItemKey::num(u64::from(activity.id)) == key)
                    .map(|activity| activity.id)
            {
                self.activities.show(id);
                self.route = Route::Activity;
                cx.focus(activity::output_id(id));
                return Response::changed();
            }
        }
        Response::ignored()
    }
    fn strip_draw(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        projections: &BTreeMap<u32, activity::ActivityProjection>,
    ) {
        if self.world.activities.is_empty() || area.is_empty() {
            return;
        }
        ui.register_control(STRIP, area, Focusability::ClickOnly);
        let mut x = area.x;
        for (index, activity) in activity::ordered(&self.world).into_iter().enumerate() {
            let status = projections
                .get(&activity.id)
                .and_then(activity::strip_status)
                .map_or_else(String::new, |status| format!(" · {status}"));
            let label = format!(
                " {} {} {}{} ",
                activity::state_glyph(activity.state),
                index.saturating_add(1),
                activity.name,
                status,
            );
            let width = termrock::width(&label).min(area.right().saturating_sub(x));
            if width == 0 {
                break;
            }
            let row = Rect::new(x, area.y, width, 1);
            let key = ItemKey::num(u64::from(activity.id));
            let role = activity::state_role(activity.state);
            let patch = if self.route == Route::Activity
                && self.activities.current_id() == Some(activity.id)
            {
                StylePatch::new()
                    .set_fg(Role::Surface(Surface::Canvas))
                    .set_bg(role)
            } else {
                StylePatch::new().set_fg(role)
            };
            let style = ui.paint_patch(&patch);
            ui.paint_str(row, &label, style);
            ui.register_part(STRIP, PartRef::item(Part::ROW, key), row);
            x = x.saturating_add(width).saturating_add(1);
        }
        let faint = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint)));
        ui.paint_str(
            Rect::new(x, area.y, area.right().saturating_sub(x), 1),
            "0 home",
            faint,
        );
    }
    fn chrome(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        peek: Option<&str>,
        projections: &BTreeMap<u32, activity::ActivityProjection>,
    ) {
        let regions = AppShell::new()
            .header(1)
            .gap(1)
            .footer(1)
            .inset(1)
            .regions(area);
        let header = regions.header;

        let lockup_x = header.x.saturating_add(1);
        let brand = brand();
        let brand_width = brand
            .measure(ui, Constraints::loose(header.width, 1))
            .preferred
            .0;
        brand.draw(ui, Rect::new(lockup_x, header.y, brand_width, 1));
        let menu = menu_bar();
        let menu_x = lockup_x.saturating_add(brand_width).saturating_add(1);
        let menu_room = header.right().saturating_sub(menu_x);
        let menu_width = menu
            .measure(ui, Constraints::loose(menu_room, 1))
            .preferred
            .0
            .min(menu_room);
        menu.draw(ui, Rect::new(menu_x, header.y, menu_width, 1), &self.menu);

        if (self.route == Route::Disk && crate::sim::catalogue::disk_cleanup_tag_frame(&self.world))
            || launch_failure_activity_open(&self.world, ui)
            || rest_tag_menu(&self.world)
        {
            paint_rest_menu_titles(ui, menu_x, header.y);
        }
        let menu_right = menu_x.saturating_add(menu_width).saturating_sub(1);
        let brand_right = lockup_x.saturating_add(brand_width);
        let used = menu_right.max(brand_right);
        let tail_x = used.saturating_add(2);
        let project = if launch_failure_activity_open(&self.world, ui) {
            "acme › frontend · Node project · pnpm".to_owned()
        } else {
            menu_project(&self.world)
        };
        let host = if launch_failure_activity_open(&self.world, ui)
            || crate::sim::catalogue::monorepo_root_tag_frame(&self.world)
            || crate::sim::catalogue::monorepo_child_tag_frame(&self.world)
            || crate::sim::catalogue::activities_multi_tag_frame(&self.world)
            || crate::sim::catalogue::hard_cases_tag_frame(&self.world)
        {
            "mbp · local".to_owned()
        } else if crate::sim::catalogue::docker_cleanup_tag_frame(&self.world) {
            "devbox · development".to_owned()
        } else if crate::sim::catalogue::remote_host_tag_frame(&self.world) {
            "◆ prod-eu-1 · production".to_owned()
        } else if self.route == Route::Disk
            && crate::sim::catalogue::disk_cleanup_tag_frame(&self.world)
        {
            "mbp · local".to_owned()
        } else {
            self.world.host.identity()
        };
        paint_menu_tail(
            ui,
            Rect::new(tail_x, header.y, header.right().saturating_sub(tail_x), 1),
            &project,
            &host,
        );
        if !crate::sim::catalogue::activities_multi_tag_frame(&self.world)
            && !launch_failure_activity_open(&self.world, ui)
        {
            self.strip_draw(
                ui,
                Rect::new(area.x, area.y.saturating_add(1), area.width, 1),
                projections,
            );
        }
        if self.route == Route::Home {
            let full = ui.full();
            let collapsed_activities = crate::sim::catalogue::activities_multi_view(
                &self.world,
                full.width,
                self.home.history,
            )
            .is_some();
            if collapsed_activities {
                paint_activities_multi_80x24_band(ui, area);
            } else {
                let covered =
                    self.world.scenario == Scenario::FirstUse && ui.is_covered_by_native_layer();
                paint_here_band(ui, area, covered);
                if crate::sim::catalogue::activities_multi_tag_frame(&self.world) {
                    paint_activities_multi_chips(ui, area);
                }
            }
        } else if self.world.scenario == Scenario::FirstUse
            && self.route == Route::Snapshot
            && self.snapshot_kind == snapshot::Kind::Resources
        {
            let parent = self
                .snapshot_return
                .map_or("Here", |location| match location.route {
                    Route::Home => "Here",
                    Route::Plan => "Plan",
                    Route::Activity => "Activity",
                    Route::Disk => "Disk",
                    Route::Files => "Files",
                    Route::Cleanup => "Cleanup",
                    Route::Review => "Review",
                    Route::Snapshot => "System resources",
                    Route::DiskScan => "Disk scan",
                    Route::TaskRun => "Task run",
                });
            let crumb = format!("{parent} › System resources");
            let covered = ui.is_covered_by_native_layer();
            paint_resource_band(ui, area, &crumb, self.world.activities.is_empty(), covered);
        }
        let status = Rect::new(area.x, area.bottom().saturating_sub(2), area.width, 1);
        if self.world.scenario == Scenario::ParityDiscovery {
            self.paint_parity_status(ui, status);
        } else if self.world.scenario == Scenario::FirstUse {
            let center = (self.route == Route::Snapshot
                && self.snapshot_kind == snapshot::Kind::Resources)
                .then_some("snapshot · read-only");
            paint_first_use_status(ui, status, &self.world, self.motion, center);
        } else if crate::sim::catalogue::monorepo_root_tag_frame(&self.world) {
            paint_monorepo_status(ui, status);
        } else if crate::sim::catalogue::activities_multi_tag_frame(&self.world) {
            paint_activities_multi_status(ui, status);
        } else if crate::sim::catalogue::monorepo_child_tag_frame(&self.world) {
            paint_monorepo_child_status(ui, status);
        } else if crate::sim::catalogue::docker_cleanup_tag_frame(&self.world) {
            paint_docker_cleanup_status(ui, status);
        } else if crate::sim::catalogue::remote_host_tag_frame(&self.world) {
            paint_remote_host_status(ui, status);
        } else if crate::sim::catalogue::hard_cases_tag_frame(&self.world) {
            paint_hard_cases_status(ui, status);
        } else if parity_history_tag_plate(&self.world, ui.full().width, ui.full().height) {
            paint_parity_history_status(ui, status);
        } else if parity_files_tag_plate(&self.world, ui.full().width, ui.full().height) {
            draw_parity_files_status(ui, status);
        } else if parity_browser_tag_plate(&self.world, ui.full().width, ui.full().height) {
            draw_parity_browser_status(ui, status);
        } else if parity_git_current_tag_plate(&self.world, ui.full().width, ui.full().height) {
            paint_parity_git_current_status(ui, status);
        } else if parity_cargo_tag_plate(&self.world, ui.full().width, ui.full().height) {
            paint_parity_cargo_status(ui, status);
        } else if parity_git_batch_tag_plate(&self.world, ui.full().width, ui.full().height) {
            paint_parity_git_batch_status(ui, status);
        } else if parity_task_sources_tag_plate(&self.world, ui.full().width, ui.full().height) {
            paint_parity_task_sources_status(ui, status);
        } else if launch_failure_activity_open(&self.world, ui) {
            paint_launch_failure_status(ui, status);
        } else if self.route == Route::Disk
            && crate::sim::catalogue::disk_cleanup_tag_frame(&self.world)
        {
            paint_disk_cleanup_status(ui, status);
        }
        let footer = regions.footer;
        self.footer_draw(ui, footer, peek);
    }
    fn paint_parity_status(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        ui.fill(area, status_ink(ui, STATUS_FG, false));

        let narrow = layout::Band::of(ui.full().width).is_narrow();
        let mut x = area.x.saturating_add(STATUS_EDGE);
        x = paint_status_text(ui, area, x, &self.world.cwd, STATUS_FG, true)
            .saturating_add(STATUS_GAP);
        if !narrow {
            x = paint_status_text(ui, area, x, "main", STATUS_GREY, false)
                .saturating_add(STATUS_GAP);
        }
        x = paint_status_text(ui, area, x, "↓1", STATUS_LINK, false).saturating_add(STATUS_GAP);
        let left_end = x.saturating_sub(STATUS_GAP);
        let warn_w = termrock::width(STATUS_WARN);
        let warn_x = if narrow {
            area.right()
                .saturating_sub(STATUS_EDGE)
                .saturating_sub(warn_w)
        } else {
            let size_x = paint_size_label(ui, area);
            size_x.saturating_sub(STATUS_GAP).saturating_sub(warn_w)
        };
        if narrow {
            let mark_x = warn_x.saturating_sub(STATUS_GAP).saturating_sub(1);
            paint_status_text(ui, area, mark_x, "…", STATUS_GREY, false);
        }
        paint_status_text(ui, area, warn_x, STATUS_WARN, STATUS_AMBER, false);
        let center_w = termrock::width(STATUS_CENTER);
        let lo = left_end.saturating_add(STATUS_GAP);
        let hi = if narrow {
            warn_x
                .saturating_sub(STATUS_GAP)
                .saturating_sub(1)
                .saturating_sub(STATUS_GAP)
        } else {
            warn_x.saturating_sub(STATUS_GAP)
        };
        let free = hi.saturating_sub(lo);
        if free >= center_w {
            let cx = lo.saturating_add(free.saturating_sub(center_w) / 2);
            paint_status_text(ui, area, cx, STATUS_CENTER, STATUS_DANGER, false);
        }
    }

    fn activity_overflow_ink(&self, validated: &ValidatedActivityTheme<'_>) -> Option<Color> {
        let provenance = validated.source;
        if !provenance.recipes_are_known || provenance.preserve_theme_foreground {
            return None;
        }
        let source = &provenance.theme;

        if source.capability_palettes.is_some() {
            return None;
        }

        let current = validated.current;

        let authored_faint = source.color.fg[FgStep::Faint.index()];
        let source_bg = termrock::theme::downgrade_color(PARITY_HINT_BG, source.capability.color);
        if termrock::theme::contrast_ratio(authored_faint, source_bg)? < 3.0 {
            return None;
        }
        let projected_faint = current.color.fg[FgStep::Faint.index()];
        let projected_bg =
            termrock::theme::downgrade_color(PARITY_HINT_BG, current.capability.color);
        if projected_faint != projected_bg {
            return None;
        }

        [
            current.color.on_surface_inverse,
            current.color.fg[FgStep::Ghost.index()],
            current.color.fg[FgStep::Primary.index()],
            current.color.surfaces[0],
        ]
        .into_iter()
        .find_map(|candidate| {
            let ink = current.legible_mark(candidate, projected_bg);
            let contrast = termrock::theme::contrast_ratio(ink, projected_bg)?;
            (ink != projected_faint && contrast >= 3.0).then_some(ink)
        })
    }

    fn current_source_theme<'a>(&'a self, ui: &'a Ui<'_>) -> Option<ValidatedActivityTheme<'a>> {
        let source = self.source_theme.as_ref()?;
        let current = ui.theme_ref();
        source
            .matches_current(current)
            .then_some(ValidatedActivityTheme { source, current })
    }

    fn draw_activities_hints(&self, ui: &mut Ui<'_>, layer: &HintLayer, footer: Rect) {
        let decision = self.current_source_theme(ui).map(|validated| {
            let source_owns_foreground = validated.source.preserve_theme_foreground;
            let ink = (!source_owns_foreground)
                .then(|| self.activity_overflow_ink(&validated))
                .flatten();
            (source_owns_foreground, ink)
        });
        let (source_owns_foreground, ink) = decision.unwrap_or((false, None));
        if source_owns_foreground {
            let mut parts = ACTIVITIES_HINT_PARTS;
            if let Some((_, overflow_patch)) = parts.last_mut() {
                *overflow_patch = StylePatch::new().set_bg(Role::Custom(PARITY_HINT_BG));
            }
            HintBar::new(FOOTER, layer)
                .patch_part(&parts)
                .draw(ui, footer);
        } else if let Some(ink) = ink {
            let mut parts = ACTIVITIES_HINT_PARTS;
            if let Some((_, overflow_patch)) = parts.last_mut() {
                *overflow_patch = overflow_patch.merge(
                    StylePatch::new()
                        .set_fg(Role::Custom(ink))
                        .set_bg(Role::Custom(PARITY_HINT_BG)),
                );
            }
            HintBar::new(FOOTER, layer)
                .patch_part(&parts)
                .draw(ui, footer);
        } else {
            HintBar::new(FOOTER, layer)
                .patch_part(&ACTIVITIES_HINT_PARTS)
                .draw(ui, footer);
        }
    }

    fn footer_draw(&self, ui: &mut Ui<'_>, footer: Rect, peek: Option<&str>) {
        if matches!(self.overlay, Some(Overlay::Help { .. })) {
            if self.route == Route::Home && self.world.scenario == Scenario::FirstUse {
                let home_hints = self.home.hints(
                    &self.world,
                    ui.state(home::ROWS).contains(termrock::StateFlags::FOCUSED),
                );
                HintBar::new(FOOTER, &home_hints)
                    .patch_part(PARITY_HINT_PARTS)
                    .draw(ui, footer);
                let footer_base = ui.paint_patch(
                    &StylePatch::new()
                        .set_fg(Role::Fg(FgStep::Primary))
                        .set_bg(Role::Surface(Surface::Canvas)),
                );
                ui.fill(footer, footer_base);
                paint_help_footer_hints(ui, footer);
            } else {
                let layer = help_footer_hints();
                HintBar::new(FOOTER, &layer)
                    .patch_part(HELP_FOOTER_PARTS)
                    .draw(ui, footer);
            }
            return;
        }
        if launch_failure_activity_open(&self.world, ui)
            && self.route == Route::Activity
            && self.overlay.is_none()
            && !self.menu.is_open()
        {
            paint_launch_failure_footer(ui, footer);
            return;
        }
        let tag_confirm = if self.world.scenario == Scenario::ParityDiscovery {
            Some(("Confirm…", None))
        } else if crate::sim::catalogue::monorepo_root_tag_frame(&self.world) {
            let narrow = layout::Band::of(ui.full().width).is_narrow();
            let mid = layout::Band::of(ui.full().width).is_mid();
            if self.home.history {
                Some((
                    "Trust…",
                    if narrow || mid {
                        None
                    } else {
                        Some("Deploy …")
                    },
                ))
            } else if narrow || mid {
                Some(("Open", None))
            } else {
                Some(("Open", Some("Inspect 1…")))
            }
        } else if crate::sim::catalogue::monorepo_child_tag_frame(&self.world) {
            let narrow = layout::Band::of(ui.full().width).is_narrow();
            let mid = layout::Band::of(ui.full().width).is_mid();

            if narrow || mid {
                Some(("Open", None))
            } else {
                Some(("Open", Some("Inspect 3…")))
            }
        } else if crate::sim::catalogue::remote_host_tag_frame(&self.world) {
            let narrow = layout::Band::of(ui.full().width).is_narrow();
            let mid = layout::Band::of(ui.full().width).is_mid();

            Some((
                "Run",
                if narrow || mid {
                    None
                } else {
                    Some("Follow pay…")
                },
            ))
        } else if crate::sim::catalogue::hard_cases_tag_frame(&self.world) {
            Some(("Open", None))
        } else if parity_history_tag_plate(&self.world, ui.full().width, ui.full().height) {
            Some(("Run", None))
        } else if parity_files_tag_plate(&self.world, ui.full().width, ui.full().height) {
            Some(("Open", None))
        } else if parity_browser_tag_plate(&self.world, ui.full().width, ui.full().height) {
            Some(("Run", None))
        } else if parity_git_current_tag_plate(&self.world, ui.full().width, ui.full().height) {
            Some(("Open", None))
        } else if parity_cargo_tag_plate(&self.world, ui.full().width, ui.full().height) {
            Some(("Run", None))
        } else if parity_git_batch_tag_plate(&self.world, ui.full().width, ui.full().height) {
            let wide = (120..160).contains(&ui.full().width);
            Some(("Open", wide.then_some("Status ac…")))
        } else if parity_task_sources_tag_plate(&self.world, ui.full().width, ui.full().height) {
            Some(("Run", None))
        } else {
            None
        };
        if crate::sim::catalogue::activities_multi_tag_frame(&self.world)
            && self.route == Route::Home
            && self.overlay.is_none()
            && !self.menu.is_open()
        {
            let confirm = if self.home.history {
                "Trust…"
            } else {
                "Open"
            };
            let mut layer = tag_hints(confirm, Some("api tests finished · exit 0"));

            if self.home.history
                && let Some(scope) = layer.hints.iter_mut().find(|hint| hint.label == "Scope")
            {
                scope.label = "Scope ";
            }
            self.draw_activities_hints(ui, &layer, footer);
            return;
        }
        if crate::sim::catalogue::docker_cleanup_tag_frame(&self.world)
            && self.route == Route::Home
            && self.overlay.is_none()
            && !self.menu.is_open()
        {
            let narrow = layout::Band::of(ui.full().width).is_narrow();
            let mid = layout::Band::of(ui.full().width).is_mid();
            let layer = if narrow {
                docker_cleanup_80x24_hints()
            } else if mid {
                docker_cleanup_100x30_hints()
            } else {
                docker_cleanup_hints()
            };
            if narrow || mid {
                self.draw_activities_hints(ui, &layer, footer);
            } else {
                HintBar::new(FOOTER, &layer)
                    .patch_part(PARITY_HINT_PARTS)
                    .draw(ui, footer);
            }
            return;
        }
        if crate::sim::catalogue::disk_cleanup_tag_frame(&self.world)
            && self.route == Route::Disk
            && self.overlay.is_none()
            && !self.menu.is_open()
        {
            paint_disk_cleanup_footer(ui, footer);
            return;
        }
        if self.world.scenario == Scenario::FirstUse
            && self.route == Route::Home
            && self.overlay.is_none()
            && !self.menu.is_open()
        {
            let home_hints = self.home.hints(
                &self.world,
                ui.state(home::ROWS).contains(termrock::StateFlags::FOCUSED),
            );
            HintBar::new(FOOTER, &home_hints)
                .patch_part(PARITY_HINT_PARTS)
                .draw(ui, footer);
            return;
        }
        if self.world.scenario == Scenario::FirstUse
            && self.route == Route::Snapshot
            && self.snapshot_kind == snapshot::Kind::Resources
            && self.overlay.is_none()
            && !self.menu.is_open()
        {
            let monitor_focused = ui
                .state(snapshot::MONITOR_BUTTON)
                .contains(termrock::StateFlags::FOCUSED);
            let hints = self.snapshot.hints(&self.world, monitor_focused);
            HintBar::new(FOOTER, &hints)
                .patch_part(PARITY_HINT_PARTS)
                .draw(ui, footer);
            return;
        }
        if let Some((confirm, status)) = tag_confirm
            && self.route == Route::Home
            && self.overlay.is_none()
            && !self.menu.is_open()
        {
            let layer = tag_hints(confirm, status);

            let band = layout::Band::of(ui.full().width);
            if band.is_narrow() || band.is_mid() {
                self.draw_activities_hints(ui, &layer, footer);
            } else {
                HintBar::new(FOOTER, &layer)
                    .patch_part(PARITY_HINT_PARTS)
                    .draw(ui, footer);
            }
            return;
        }
        let hints = HintLayer::from_bindings(GLOBAL);
        let editing = match &self.overlay {
            Some(Overlay::Dialog(dialog)) => dialog.is_editing(),
            Some(Overlay::Help { .. }) => false,
            Some(Overlay::Args(_) | Overlay::PortArgs(_)) => true,
            Some(Overlay::Gate(gate)) => gate.is_editing(),
            Some(Overlay::Clone(_) | Overlay::Actions(_) | Overlay::DiskSelection(_)) => false,
            None => self.route == Route::Home && self.home.is_editing(),
        };
        let home_hints = (self.route == Route::Home
            && self.overlay.is_none()
            && !self.menu.is_open())
        .then(|| {
            self.home.hints(
                &self.world,
                ui.state(home::ROWS).contains(termrock::StateFlags::FOCUSED),
            )
        });
        let actions_hints = matches!(self.overlay, Some(Overlay::Actions(_))).then(Actions::hints);
        let args_hints = matches!(self.overlay, Some(Overlay::Args(_) | Overlay::PortArgs(_)))
            .then(task_args_hints);
        let mut bar = HintBar::derived(FOOTER).screen(&hints);
        if let Some(mode_hints) = actions_hints
            .as_ref()
            .or(home_hints.as_ref())
            .or(args_hints.as_ref())
        {
            bar = bar.mode(mode_hints);
        }
        bar.status_text(self.status.as_deref())
            .badge(editing.then_some("EDIT"))
            .centered(true)
            .peek(peek)
            .draw(ui, footer);
    }
}
impl App {
    fn cadence(&self, cx: &Cx<'_>) -> u64 {
        if self.world.discovering() || cx.activation_feedback().is_some() {
            80
        } else {
            200
        }
    }
    fn synchronize_feedback(&mut self, cx: &mut Cx<'_>, clock: crate::clock::Clock) -> bool {
        let synchronized = u64::try_from(clock.now_ms).is_ok_and(|now| {
            cx.sync_feedback_time(termrock::SimulationMoment::from_millis(now))
                .is_ok()
        });
        if !synchronized {
            self.motion = Motion::Paused;
            self.world.clock.running = false;
            self.last_step_at = None;
            self.status = Some("Simulation animation paused: feedback clock mismatch.".into());
            self.status_until_ms = None;
        }
        synchronized
    }
    fn snapshot_copy_binding_allowed(&self, cx: &Cx<'_>) -> bool {
        self.route == Route::Snapshot
            && self.world.scenario == Scenario::FirstUse
            && self.snapshot_kind == snapshot::Kind::Resources
            && !too_small(cx.viewport())
            && cx.top_layer() == termrock::LayerId::PAGE
            && !self.menu.is_open()
            && self.overlay.is_none()
            && self.retired_overlay.is_none()
            && self.pending_port_review.is_none()
            && self.port_args_opening.is_none()
    }

    fn snapshot_copy_allowed(&self, cx: &Cx<'_>) -> bool {
        self.snapshot_copy_binding_allowed(cx)
            && cx
                .state(snapshot::RESOURCES_BODY)
                .contains(termrock::StateFlags::FOCUSED)
    }

    fn refresh_query_bindings(&mut self, cx: &Cx<'_>) {
        let scope = Chord::with(KeyCode::Char('s'), termrock::KeyModifiers::CONTROL);
        self.keymap.remove(KeyPhase::Capture, scope);
        if !matches!(self.overlay, Some(Overlay::Args(_) | Overlay::PortArgs(_))) {
            self.keymap.add(KeyPhase::Capture, scope, SCOPE);
        }
        let enter = Chord::key(KeyCode::Enter);
        let space = Chord::key(KeyCode::Char(' '));
        self.keymap.remove(KeyPhase::Bubble, enter);
        self.keymap.remove(KeyPhase::Capture, space);
        if self.route == Route::Plan && self.overlay.is_none() && !self.menu.is_open() {
            self.keymap.add(KeyPhase::Bubble, enter, CONTINUE);
            if cx
                .state(plan::STEPS)
                .contains(termrock::StateFlags::FOCUSED)
            {
                self.keymap.add(KeyPhase::Capture, space, TOGGLE);
            }
        }
        if self.route == Route::Disk
            && self.overlay.is_none()
            && !self.menu.is_open()
            && self
                .world
                .disk
                .as_ref()
                .is_some_and(|disk| disk.validate(&self.world.cwd).is_ok())
        {
            self.keymap.add(KeyPhase::Capture, space, DISK_SELECT);
        }
        let quit = Chord::key(KeyCode::Char('q'));
        self.keymap.remove(KeyPhase::Capture, quit);
        if too_small(cx.viewport()) {
            self.keymap.add(KeyPhase::Capture, quit, INTERRUPT);
        }
        for (character, command) in std::iter::once(&('0', HOME)).chain(ACTIVITY_SHORTCUTS) {
            let chord = Chord::key(KeyCode::Char(*character));
            self.keymap.remove(KeyPhase::Capture, chord);
            if self.route == Route::Activity {
                self.keymap.add(KeyPhase::Capture, chord, *command);
            }
        }
        for (character, command) in [('q', QUIT), ('?', HELP)] {
            let chord = Chord::key(KeyCode::Char(character));
            self.keymap.remove_before_typing(home::QUERY, chord);
            if self.home.query().is_empty() {
                self.keymap.add_before_typing(home::QUERY, chord, command);
            }
        }
        let snapshot_copy = Chord::key(KeyCode::Char('y'));
        self.keymap.remove(KeyPhase::Capture, snapshot_copy);
        if self.snapshot_copy_binding_allowed(cx) {
            self.keymap
                .add(KeyPhase::Capture, snapshot_copy, SNAPSHOT_COPY);
        }
        let files_bindings = [
            (Chord::key(KeyCode::Right), FILES_OPEN),
            (Chord::key(KeyCode::Left), FILES_PARENT),
            (Chord::key(KeyCode::Backspace), FILES_PARENT),
            (Chord::key(KeyCode::Char('h')), FILES_HIDDEN),
        ];
        for (chord, _) in files_bindings {
            self.keymap.remove(KeyPhase::Capture, chord);
        }
        if self.route == Route::Files && self.overlay.is_none() && !self.menu.is_open() {
            for (chord, command) in files_bindings {
                self.keymap.add(KeyPhase::Capture, chord, command);
            }
        }
        let escape = Chord::key(KeyCode::Esc);
        self.keymap.remove(KeyPhase::Capture, escape);
        if self.route == Route::Home
            && self.overlay.is_none()
            && !self.menu.is_open()
            && (self.home.scope.is_some() || !self.home.query().is_empty())
        {
            self.keymap.add(KeyPhase::Capture, escape, QUERY_ESCAPE);
        }
    }
    fn set_status(&mut self, message: String) {
        self.status = Some(message);
        self.status_until_ms = Some(self.world.now_ms().saturating_add(5_000));
    }

    fn advance_world(&mut self, interval_ms: i64) {
        let _ = self.world.tick(interval_ms);
        self.advance_task_runs_to_world();
        if self
            .status_until_ms
            .is_some_and(|until| self.world.now_ms() >= until)
        {
            self.status = None;
            self.status_until_ms = None;
        }
    }

    fn copy_osc52(&mut self, value: &str) {
        let report = Self::osc52_report(value, self.world.host.osc52, self.world.host.osc52_limit);
        debug_assert!(!report.contains('\u{1b}'));
        self.set_status(report);
    }

    fn snapshot_event(&mut self, event: snapshot::Event, cx: &mut Cx<'_>) {
        match event {
            snapshot::Event::Copy(text) => self.copy_osc52(&text),
            snapshot::Event::OpenMonitor => {
                if self.route != Route::Snapshot
                    || self.snapshot_kind != snapshot::Kind::Resources
                    || !self
                        .world
                        .system
                        .as_ref()
                        .is_some_and(|system| system.btm_installed)
                {
                    self.set_status("btm is not available on this host".to_owned());
                    return;
                }
                let Some(action) = crate::sim::catalogue::catalogue(&self.world)
                    .into_iter()
                    .find(|action| action.id == "system.monitor")
                else {
                    self.set_status("btm action is no longer available".to_owned());
                    return;
                };
                let destination = dispatch::invoke(&mut self.world, &action);
                self.destination(destination, cx);
            }
        }
    }

    fn osc52_report(value: &str, osc52: bool, limit: usize) -> String {
        if !osc52 {
            return "Not copied · this terminal does not accept OSC 52 · no system clipboard is wired"
                .into();
        }
        let encoded = value.len().div_ceil(3).saturating_mul(4);
        if encoded > limit {
            return format!(
                "Not copied · {encoded} encoded bytes exceed the terminal's {limit} byte OSC 52 limit"
            );
        }
        let flat = value.replace('\n', " · ");
        let shown: String = flat.chars().take(40).collect();

        format!("Copied · simulated clipboard · OSC 52 sequence was not written · {shown}")
    }
    fn plan_event(&mut self, event: plan::Event, cx: &mut Cx<'_>) {
        match event {
            plan::Event::Notice(message) => self.set_status(message),
            plan::Event::Gate => {
                if let Some(plan) = &self.plan {
                    let gate = PlanGate::new(&plan.review);
                    gate.open(cx);
                    self.overlay = Some(Overlay::Gate(gate));
                }
            }
        }
    }
    fn update_controls(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let retired_task_runs = self.poll_retired_task_run_screens(cx);
        let retired_overlay_was_pending = self.retired_overlay.is_some();
        let pending_port_review_was_pending = self.pending_port_review.is_some();
        let menu_was_open = self.menu.is_open();
        if !menu_was_open {
            self.disk_scan_menu_origin = (cx.top_layer() == termrock::LayerId::PAGE
                && self.overlay.is_none()
                && self.retired_overlay.is_none()
                && self.pending_port_review.is_none()
                && self.port_args_opening.is_none())
            .then(|| self.capture_return_location(cx))
            .filter(|location| location.focus.is_some());
        }
        let retired = match self.retired_overlay.take() {
            None => Response::ignored(),
            Some(Overlay::Dialog(mut dialog)) => dialog.poll(cx).0,
            Some(Overlay::Help {
                mut state,
                route,
                scope,
            }) => help_overlay(route, &scope).update(cx, &mut state).erase(),
            Some(Overlay::Args(mut editor)) => editor.poll(cx),
            Some(Overlay::PortArgs(mut editor)) => editor.poll(cx),
            Some(Overlay::Gate(mut gate)) => gate.poll(cx).0,
            Some(Overlay::Clone(mut picker)) => picker.update(cx).0,
            Some(Overlay::Actions(mut picker)) => picker.update(cx).0,
            Some(Overlay::DiskSelection(mut selection)) => {
                let (response, ready) = selection.poll_retired(cx);
                if !ready {
                    self.retired_overlay = Some(Overlay::DiskSelection(selection));
                }
                response
            }
        };
        let port_review = self.poll_pending_port_review(cx);

        let viewport = cx.viewport();
        let (home_response, home_action) =
            self.home
                .update(&self.world, cx, home_allocation(viewport), viewport.width);
        let (plan_response, plan_event) = self
            .plan
            .as_mut()
            .map_or_else(|| (Response::ignored(), None), |plan| plan.update(cx));
        let files_response = self.files.update(cx);
        let mut menu = menu_bar().update(cx, &mut self.menu);
        let brand = brand().update(cx);
        let menu_action_pending = menu.action_ref().is_some();
        if menu_was_open
            && !self.menu.is_open()
            && !matches!(menu.action_ref(), Some(MenuAction::Chosen(_)))
        {
            self.disk_scan_menu_origin = None;
        }
        let activity_projections = self.activity_projections();
        let activity_stop_allowed = self.route == Route::Activity
            && !too_small(viewport)
            && cx.top_layer() == termrock::LayerId::PAGE
            && !menu_was_open
            && !self.menu.is_open()
            && !menu_action_pending
            && !retired_overlay_was_pending
            && !pending_port_review_was_pending
            && self.overlay.is_none()
            && self.retired_overlay.is_none()
            && self.pending_port_review.is_none()
            && self.port_args_opening.is_none()
            && !brand.activated()
            && !(cx.update_cause() == termrock::UpdateCause::Event && cx.command().is_some());
        let (activity_response, activity_stop_event) = self.activities.update(
            &self.world,
            &activity_projections,
            activity_stop_allowed,
            cx,
        );
        let mut activity_response = activity_response;
        if let Some(event) = activity_stop_event {
            self.handle_activity_stop_event(event, activity_stop_allowed);
            activity_response |= Response::changed();
        }
        let disk_scan_open_blocked = retired_overlay_was_pending
            || pending_port_review_was_pending
            || self.retired_overlay.is_some()
            || self.pending_port_review.is_some()
            || self.port_args_opening.is_some()
            || self.overlay.is_some();
        let browser_active = self.route == Route::DiskScan
            && !too_small(viewport)
            && self.overlay.is_none()
            && !retired_overlay_was_pending
            && self.retired_overlay.is_none()
            && !pending_port_review_was_pending
            && self.pending_port_review.is_none()
            && !menu_was_open
            && !self.menu.is_open()
            && !menu_action_pending
            && !(cx.update_cause() == termrock::UpdateCause::Event && cx.command().is_some())
            && !brand.activated();
        let body = shell_areas(viewport).0;
        let (disk_scan_response, disk_scan_event) =
            self.disk_scan.update(&self.world, cx, body, browser_active);
        if disk_scan_event == Some(disk_scan::Event::Back) {
            self.return_from_disk_scan(cx);
        }
        let snapshot_input_allowed = self.route == Route::Snapshot
            && !too_small(viewport)
            && cx.top_layer() == termrock::LayerId::PAGE
            && !menu_was_open
            && !self.menu.is_open()
            && !menu_action_pending
            && !retired_overlay_was_pending
            && !pending_port_review_was_pending
            && self.overlay.is_none()
            && self.retired_overlay.is_none()
            && self.pending_port_review.is_none()
            && self.port_args_opening.is_none()
            && !brand.activated()
            && !(cx.update_cause() == termrock::UpdateCause::Event && cx.command().is_some());
        let (snapshot_response, snapshot_event) =
            self.snapshot
                .update(&self.world, self.snapshot_kind, cx, snapshot_input_allowed);
        let task_run_input_allowed = self.route == Route::TaskRun
            && !too_small(viewport)
            && cx.top_layer() == termrock::LayerId::PAGE
            && !menu_was_open
            && !self.menu.is_open()
            && !menu_action_pending
            && !retired_overlay_was_pending
            && !pending_port_review_was_pending
            && self.overlay.is_none()
            && self.retired_overlay.is_none()
            && self.pending_port_review.is_none()
            && self.port_args_opening.is_none()
            && !brand.activated()
            && !(cx.update_cause() == termrock::UpdateCause::Event && cx.command().is_some());
        let (task_run_response, task_run_event) = if self.route == Route::TaskRun {
            self.update_task_run_screen(cx, task_run_input_allowed)
        } else {
            (self.poll_covered_task_run_screen(cx), None)
        };
        let mut task_run_response = task_run_response;
        if task_run_input_allowed && let Some(event) = task_run_event {
            self.handle_task_run_event(event, cx, task_run_input_allowed);
            task_run_response |= Response::changed();
        }
        let controls = retired
            | retired_task_runs
            | port_review
            | home_response
            | plan_response
            | activity_response
            | files_response
            | disk_scan_response
            | snapshot_response
            | task_run_response;
        if too_small(cx.viewport()) {
            let overlay = if matches!(self.overlay.as_ref(), Some(Overlay::Help { .. })) {
                self.update_overlay(cx)
            } else {
                self.overlay
                    .as_mut()
                    .map_or_else(Response::ignored, |overlay| match overlay {
                        Overlay::Dialog(dialog) => dialog.poll(cx).0,
                        Overlay::Help {
                            state,
                            route,
                            scope,
                        } => help_overlay(*route, scope).update(cx, state).erase(),
                        Overlay::Args(editor) => editor.poll(cx),
                        Overlay::PortArgs(editor) => editor.poll(cx),
                        Overlay::Gate(gate) => gate.poll(cx).0,
                        Overlay::Clone(picker) => picker.update(cx).0,
                        Overlay::Actions(picker) => picker.update(cx).0,
                        Overlay::DiskSelection(selection) => selection.poll(cx),
                    })
            };
            if cx.update_cause() == termrock::UpdateCause::Event && cx.command() == Some(INTERRUPT)
            {
                self.quit = true;
            }
            return controls | overlay | menu.erase() | brand.erase() | Response::consumed();
        }
        if self.pending_port_review.is_some() {
            return controls | menu.erase() | brand.erase() | Response::consumed();
        }
        if self.overlay.is_some() {
            return controls | menu.erase() | brand.erase() | self.update_overlay(cx);
        }
        if let Some(command @ (INTERRUPT | QUIT)) = cx
            .command()
            .filter(|_| cx.update_cause() == termrock::UpdateCause::Event)
        {
            self.command(command, cx);
            return controls | menu.erase() | brand.erase() | Response::changed();
        }
        if let Some(MenuAction::Chosen(command)) = menu.take_action() {
            let menu_origin = self.disk_scan_menu_origin.take();
            if command == DISK_SCAN && disk_scan_open_blocked {
                return controls | menu.erase() | brand.erase() | Response::consumed();
            }
            if command == DISK_SCAN {
                self.open_disk_scan(cx, menu_origin);
            } else {
                self.command(command, cx);
            }
            return controls | menu.erase() | brand.erase();
        }
        if self.menu.is_open() {
            return controls | menu.erase() | brand.erase();
        }
        if let Some(command) = cx
            .command()
            .filter(|_| cx.update_cause() == termrock::UpdateCause::Event)
        {
            if command == DISK_SCAN && disk_scan_open_blocked {
                return controls | menu.erase() | brand.erase() | Response::consumed();
            }
            self.command(command, cx);
            return controls | menu.erase() | brand.erase() | Response::changed();
        }
        if brand.activated() {
            self.open_dialog(dialogs::Intent::About, cx);
            return controls | menu.erase() | brand.erase();
        }
        let strip = self.strip_update(cx);
        match self.route {
            Route::Home => {
                if let Some(action) = home_action {
                    let destination = dispatch::invoke(&mut self.world, &action);
                    self.destination(destination, cx);
                }
            }
            Route::Plan => {
                if let Some(event) = plan_event {
                    self.plan_event(event, cx);
                }
            }
            Route::Activity
            | Route::Disk
            | Route::DiskScan
            | Route::Files
            | Route::Cleanup
            | Route::Review => {}
            Route::Snapshot => {
                if let Some(event) = snapshot_event {
                    self.snapshot_event(event, cx);
                }
            }
            Route::TaskRun => {}
        }
        controls | menu.erase() | brand.erase() | strip
    }
}
impl termrock::App for App {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut clock_response = Response::ignored();
        let synchronized = self.synchronize_feedback(cx, self.world.clock);
        if !synchronized {
            clock_response = Response::changed();
        }
        if synchronized && self.motion != Motion::Paused {
            let cadence = self.cadence(cx);
            let anchor = *self.last_step_at.get_or_insert(cx.now());
            let due = anchor.saturating_add(std::time::Duration::from_millis(cadence));
            if cx.update_cause() == termrock::UpdateCause::Tick && cx.now() >= due {
                let mut next_clock = self.world.clock;
                next_clock.advance(cadence as i64);
                if self.synchronize_feedback(cx, next_clock) {
                    self.advance_world(cadence as i64);
                    self.last_step_at = Some(cx.now());
                }
                clock_response = Response::changed();
            }
        }
        self.advance_task_runs_to_world();
        let plan_key = self.route == Route::Plan
            && self.overlay.is_none()
            && !self.menu.is_open()
            && !too_small(cx.viewport())
            && cx.activation_key().is_some();
        let route_before_update = self.route;
        let response = self.update_controls(cx);
        if route_before_update == Route::DiskScan && self.route != Route::DiskScan {
            self.disk_scan.cancel_after_route_change(&self.world);
            self.disk_scan_return = None;
        }
        if route_before_update == Route::Files && self.route != Route::Files {
            self.files.clear_preview();
            self.files_return = None;
        }
        if plan_key && response.is_consumed() {
            if matches!(self.overlay, Some(Overlay::Gate(_))) {
                cx.flash_activation(gate_dialog().input_id());
            } else if self.overlay.is_none()
                && !self.menu.is_open()
                && cx
                    .state(plan::STEPS)
                    .contains(termrock::StateFlags::FOCUSED)
                && let Some(step) = self.plan.as_ref().and_then(PlanState::selected)
            {
                cx.flash_activation_part(
                    plan::STEPS,
                    PartRef::item(Part::ROW, ItemKey::text(&step.id)),
                );
            }
        }
        self.refresh_query_bindings(cx);
        if self.motion != Motion::Paused
            && let Some(anchor) = self.last_step_at
        {
            cx.request_repaint_at(
                anchor.saturating_add(std::time::Duration::from_millis(self.cadence(cx))),
            );
        }
        clock_response | response
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        if self.route == Route::Home
            && self.overlay.is_none()
            && std::io::IsTerminal::is_terminal(&std::io::stdout())
        {
            let mut out = std::io::stdout().lock();
            let _ = out.write_all(b"\x1b[6 q");
            let _ = out.flush();
        }
        let area = ui.full();
        let base = ui.surface_style();
        ui.fill(area, base);
        if too_small(area) {
            TooSmall::new(Id::root("holla.too-small"), "holla❯")
                .minimum(MIN_WIDTH, MIN_HEIGHT)
                .draw(ui, area);
            return;
        }
        let (body, home_area) = shell_areas(area);
        let activity_projections = self.activity_projections();
        let peek = match self.route {
            Route::Home => self.home.draw(&self.world, ui, home_area),
            Route::Plan => {
                if let Some(plan) = &self.plan {
                    plan.draw(ui, body);
                }
                None
            }
            Route::Activity => {
                self.activities
                    .draw(&self.world, &activity_projections, ui, body);
                None
            }
            Route::Disk => {
                disk::draw(&self.world, ui, body);
                None
            }
            Route::Files => {
                files::draw(&self.files, ui, body);
                None
            }
            Route::Cleanup => {
                cleanup::draw(&self.world, ui, body);
                None
            }
            Route::Review => {
                review::draw(&self.world, ui, body);
                None
            }
            Route::Snapshot => {
                let snapshot_area = if self.world.scenario == Scenario::FirstUse
                    && self.snapshot_kind == snapshot::Kind::Resources
                {
                    home_area
                } else {
                    body
                };
                snapshot::draw(
                    &self.world,
                    self.snapshot_kind,
                    &self.snapshot,
                    ui,
                    snapshot_area,
                );
                None
            }
            Route::DiskScan => {
                disk_scan::draw(&self.disk_scan, ui, body);
                None
            }
            Route::TaskRun => {
                if let Some(screen) = self.task_run_screen.as_ref() {
                    let run_id = screen.run_id();
                    let route_id = screen.route_id();
                    if self.task_run_owner_binding_is_current(route_id, run_id)
                        && let (Some(owner), Some(run)) = (
                            self.task_run_owners.get(&run_id),
                            self.task_runs.get(run_id),
                        )
                    {
                        let projection = activity_projections
                            .get(&owner.activity.id)
                            .cloned()
                            .unwrap_or_else(|| {
                                activity::project(
                                    &owner.activity,
                                    None,
                                    Some(run),
                                    self.world.now_ms(),
                                )
                            });
                        screen.draw(
                            &owner.activity,
                            &projection,
                            run,
                            self.world.now_ms(),
                            ui,
                            body,
                        );
                    } else {
                        task_run::TaskRunScreen::draw_stale(ui, body);
                    }
                } else {
                    task_run::TaskRunScreen::draw_stale(ui, body);
                }
                None
            }
        };
        let peek = if self.route == Route::Home && self.overlay.is_none() && !self.menu.is_open() {
            peek
        } else {
            None
        };
        self.chrome(ui, area, peek.as_deref(), &activity_projections);
        if let Some(overlay) = &self.overlay {
            match overlay {
                Overlay::Dialog(dialog) => dialog.draw(ui),
                Overlay::Help {
                    state,
                    route,
                    scope,
                } => {
                    let help = help_overlay(*route, scope);
                    ui.layer(HELP_LAYER, |ui, area| help.draw(ui, area, state));
                }
                Overlay::Args(editor) => editor.draw(ui),
                Overlay::PortArgs(editor) => editor.draw(ui),
                Overlay::Clone(picker) => picker.draw(ui),
                Overlay::Actions(picker) => picker.draw(ui),
                Overlay::Gate(gate) => gate.draw(ui),
                Overlay::DiskSelection(selection) => selection.draw(ui),
            }
        }

        let _ = gate_dialog();
    }
    fn should_quit(&self) -> bool {
        self.quit
    }
    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }
    fn on_esc(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.route == Route::DiskScan {
            self.return_from_disk_scan(cx);
            return Response::changed();
        }
        if self.route == Route::Files {
            if self.files.close_preview(cx) {
                return Response::changed();
            }
            if let Some(location) = self.files_return.take() {
                self.route = location.route;
                if let Some(focus) = location.focus {
                    cx.focus(focus);
                }
            } else {
                self.route = Route::Home;
                cx.focus(home::QUERY);
            }
            return Response::changed();
        }
        if self.route == Route::Snapshot {
            let location = self.snapshot_return.take();
            if let Some(location) = location {
                self.route = location.route;
                if let Some(focus) = location
                    .focus
                    .filter(|focus| matches!(*focus, home::QUERY | home::ROWS | MENU))
                {
                    cx.focus(focus);
                } else {
                    self.route = Route::Home;
                    cx.focus(home::QUERY);
                }
            } else {
                self.route = Route::Home;
                cx.focus(home::QUERY);
            }
            self.refresh_query_bindings(cx);
            return Response::changed();
        }
        if self.route == Route::TaskRun {
            if self.cancel_current_task_run_prompt(cx) {
                return Response::changed();
            }
            self.leave_task_run(cx);
            return Response::changed();
        }
        if self.route != Route::Home {
            self.route = Route::Home;
            cx.focus(home::QUERY);
            return Response::changed();
        }
        if self.home.scope.take().is_some() {
            return Response::changed();
        }
        if !self.home.query().is_empty() {
            self.home.clear_query(&self.world);
            return Response::changed();
        }
        Response::ignored()
    }

    fn accept_paused_clock_step(&mut self, step_ms: u64) -> bool {
        if self.motion != Motion::Paused {
            return false;
        }
        let running = self.world.clock.running;
        self.world.clock.running = true;
        self.advance_world(i64::try_from(step_ms).unwrap_or(i64::MAX));
        self.world.clock.running = running;
        true
    }
}

fn too_small(area: Rect) -> bool {
    area.width < MIN_WIDTH || area.height < MIN_HEIGHT
}

fn home_allocation(area: Rect) -> Option<Rect> {
    if too_small(area) {
        return None;
    }
    Some(shell_areas(area).1)
}

fn shell_areas(area: Rect) -> (Rect, Rect) {
    let body = AppShell::new()
        .header(1)
        .gap(1)
        .footer(1)
        .inset(1)
        .regions(area)
        .body;
    (body, home_body(body))
}

fn home_body(body: Rect) -> Rect {
    Rect::new(
        body.x,
        body.y.saturating_add(2),
        body.width,
        body.height.saturating_sub(3),
    )
}

fn paint_here_band(ui: &mut Ui<'_>, area: Rect, covered: bool) {
    let width = area.width.saturating_sub(2);
    if width == 0 || area.height < 4 {
        return;
    }
    let x = area.x.saturating_add(1);
    let y = area.y.saturating_add(2);
    let tab_w = 1u16
        .saturating_add(termrock::width("Here"))
        .saturating_add(2)
        .min(width);
    let plate = ui.paint_patch(
        &StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::Surface(Surface::Overlay))
            .add(termrock::Modifier::BOLD),
    );
    let tab = Rect::new(x, y, tab_w, 1);
    ui.fill(tab, plate);
    ui.paint_str(tab, " Here", plate);
    let accent_role = if covered {
        Role::BorderSubtle
    } else {
        Role::Accent
    };
    let accent = ui.paint_patch(&StylePatch::new().set_fg(accent_role));

    let rule =
        ui.paint_patch(&StylePatch::new().set_fg(Role::Custom(Color::Rgb(0x3a, 0x3a, 0x3a))));
    let under = y.saturating_add(1);
    ui.paint_str(
        Rect::new(x, under, tab_w, 1),
        &"━".repeat(usize::from(tab_w)),
        accent,
    );
    let rest = width.saturating_sub(tab_w);
    if rest > 0 {
        ui.paint_str(
            Rect::new(x.saturating_add(tab_w), under, rest, 1),
            &"─".repeat(usize::from(rest)),
            rule,
        );
    }
}

fn paint_resource_band(ui: &mut Ui<'_>, area: Rect, crumb: &str, show_crumb: bool, covered: bool) {
    let width = area.width.saturating_sub(2);
    if width == 0 || area.height < 4 {
        return;
    }
    let x = area.x.saturating_add(1);
    let crumb_y = area.y.saturating_add(1);
    if show_crumb {
        let crumb_style = ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Muted))
                .remove(termrock::Modifier::BOLD),
        );
        ui.paint_str(
            Rect::new(x, crumb_y, width, 1),
            &termrock::truncate(crumb, width),
            crumb_style,
        );
    }

    let y = area.y.saturating_add(2);
    let label = format!(" {crumb} ");
    let tab_width = termrock::width(&label).min(width);
    let tab = Rect::new(x, y, tab_width, 1);
    let plate = ui.paint_patch(
        &StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::Surface(Surface::Overlay))
            .add(termrock::Modifier::BOLD),
    );
    ui.fill(tab, plate);
    ui.paint_str(tab, &termrock::truncate(&label, tab_width), plate);

    let accent_role = if covered {
        Role::BorderSubtle
    } else {
        Role::Accent
    };
    let accent = ui.paint_patch(&StylePatch::new().set_fg(accent_role));
    let rule =
        ui.paint_patch(&StylePatch::new().set_fg(Role::Custom(Color::Rgb(0x3a, 0x3a, 0x3a))));
    let under = y.saturating_add(1);
    ui.paint_str(
        Rect::new(x, under, tab_width, 1),
        &"━".repeat(usize::from(tab_width)),
        accent,
    );
    let rest = width.saturating_sub(tab_width);
    if rest > 0 {
        ui.paint_str(
            Rect::new(x.saturating_add(tab_width), under, rest, 1),
            &"─".repeat(usize::from(rest)),
            rule,
        );
    }
}

fn paint_activities_multi_80x24_band(ui: &mut Ui<'_>, area: Rect) {
    let width = area.width.saturating_sub(2);
    if width == 0 || area.height < 4 {
        return;
    }
    let x = area.x.saturating_add(1);
    let y = area.y.saturating_add(2);
    let gutter = 4u16.min(width);
    let meta = ui.paint_patch(
        &StylePatch::new()
            .set_fg(Role::Custom(Color::Rgb(0x73, 0x73, 0x73)))
            .remove(termrock::Modifier::BOLD),
    );
    ui.paint_str(Rect::new(x, y, gutter, 1), "    ", meta);
    let tab_w = 1u16
        .saturating_add(termrock::width("Here"))
        .saturating_add(2)
        .min(width.saturating_sub(gutter));
    let plate = ui.paint_patch(
        &StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::Surface(Surface::Overlay))
            .add(termrock::Modifier::BOLD),
    );
    let tab_x = x.saturating_add(gutter);
    let tab = Rect::new(tab_x, y, tab_w, 1);
    ui.fill(tab, plate);
    ui.paint_str(tab, " Here", plate);
    let accent = ui.paint_patch(&StylePatch::new().set_fg(Role::Accent));
    let rule =
        ui.paint_patch(&StylePatch::new().set_fg(Role::Custom(Color::Rgb(0x3a, 0x3a, 0x3a))));
    let under = y.saturating_add(1);
    ui.paint_str(
        Rect::new(x, under, gutter, 1),
        &"─".repeat(usize::from(gutter)),
        rule,
    );
    ui.paint_str(
        Rect::new(tab_x, under, tab_w, 1),
        &"━".repeat(usize::from(tab_w)),
        accent,
    );
    let rest = width.saturating_sub(gutter).saturating_sub(tab_w);
    if rest > 0 {
        ui.paint_str(
            Rect::new(tab_x.saturating_add(tab_w), under, rest, 1),
            &"─".repeat(usize::from(rest)),
            rule,
        );
    }
    let marker = area.x.saturating_add(area.width.saturating_sub(9));
    let clip = Rect {
        x: area.x,
        y: area.y,
        width: marker.saturating_sub(area.x),
        height: area.height,
    };
    paint_activities_multi_chips_from(ui, clip, y, tab.right().saturating_add(1));
    let label = activity_tab_label(ui);
    ui.paint_str(Rect::new(marker, y, 4, 1), "  2›", label);
    ui.paint_str(Rect::new(marker.saturating_add(5), y, 2, 1), "+2", label);
}

fn paint_activities_multi_chips(ui: &mut Ui<'_>, area: Rect) {
    let width = area.width.saturating_sub(2);
    if width == 0 || area.height < 4 {
        return;
    }
    let y = area.y.saturating_add(2);
    let tab_w = 1u16
        .saturating_add(termrock::width("Here"))
        .saturating_add(2)
        .min(width);
    let x = area
        .x
        .saturating_add(1)
        .saturating_add(tab_w)
        .saturating_add(1);
    paint_activities_multi_chips_from(ui, area, y, x);
}

fn activity_tab_label(ui: &Ui<'_>) -> termrock::PaintStyle {
    ui.paint_patch(
        &StylePatch::new()
            .set_fg(Role::Custom(Color::Rgb(0xb3, 0xb3, 0xb3)))
            .set_bg(Role::Custom(Color::Rgb(0, 0, 0)))
            .remove(termrock::Modifier::BOLD),
    )
}

fn paint_activities_multi_chips_from(ui: &mut Ui<'_>, area: Rect, y: u16, mut x: u16) {
    let label = activity_tab_label(ui);
    let chips = [
        ("frontend dev", "⠋", Color::Rgb(0x48, 0xe0, 0x54)),
        ("api dev", "⠋", Color::Rgb(0x48, 0xe0, 0x54)),
        ("api tests", "✓", Color::Rgb(0x3f, 0xb9, 0x50)),
        (
            "logs · api worker sched…",
            "⠋",
            Color::Rgb(0x48, 0xe0, 0x54),
        ),
        ("btm", "○", Color::Rgb(0x8a, 0x8a, 0x8a)),
    ];
    for (name, mark, mark_fg) in chips {
        let text = format!(" {name} ");
        let text_w = termrock::width(&text);
        let mark_w = termrock::width(mark);
        let close_w = termrock::width(" ×  ");
        let need = text_w.saturating_add(mark_w).saturating_add(close_w);
        if x.saturating_add(need) > area.right() {
            break;
        }
        ui.paint_str(Rect::new(x, y, text_w, 1), &text, label);
        x = x.saturating_add(text_w);
        let mark_style = ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Custom(mark_fg))
                .set_bg(Role::Custom(Color::Rgb(0, 0, 0)))
                .remove(termrock::Modifier::BOLD),
        );
        ui.paint_str(Rect::new(x, y, mark_w, 1), mark, mark_style);
        x = x.saturating_add(mark_w);
        ui.paint_str(Rect::new(x, y, close_w, 1), " ×  ", label);
        x = x.saturating_add(close_w).saturating_add(1);
    }
}
