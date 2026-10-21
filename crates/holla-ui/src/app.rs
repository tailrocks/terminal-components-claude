use std::io::Write;

use crate::{
    dispatch::{self, Destination},
    domain::{action::Scope, fixtures, ranking::Pin},
    scenario::{Motion, Scenario, Screen},
    screens::{
        actions::{Actions, Choice},
        activity::{self, Activities},
        cleanup,
        clone::ClonePicker,
        dialogs::{self, ProductDialog},
        disk, files,
        home::{self, HomeState},
        plan::{self, PlanState},
        plan_gate::PlanGate,
        review, snapshot,
    },
    sim::{pg, world::World},
};
use termrock::{
    ActionKey, AppShell, Binding, Brand, Chord, Color, Constraints, Cx, Dialog, FgStep,
    Focusability, FrameRead, Hint, HintBar, HintKey, HintLayer, Id, Intent, ItemKey, KeyCode,
    KeyMap, KeyPhase, Menu, MenuAction, MenuBar, MenuBarHoverScope, MenuItem, MenuState, Part,
    PartRef, Phase, Rect, Response, Role, StylePatch, Surface, TooSmall, Ui,
};

use crate::layout::{self, MIN_HEIGHT, MIN_WIDTH};

const BRAND: Id = Id::root("holla.brand");
const MENU: Id = Id::root("holla.menu");
const STRIP: Id = Id::root("holla.activities");
const FOOTER: Id = Id::root("holla.footer");
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
const FOLDER_BROWSER: ActionKey = ActionKey::application("holla.folder-browser");
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
enum Overlay {
    Dialog(Box<ProductDialog>),
    Clone(ClonePicker),
    Actions(Actions),
    Gate(PlanGate),
}

pub struct App {
    world: World,
    motion: Motion,
    route: Route,
    home: HomeState,
    plan: Option<PlanState>,
    activities: Activities,
    overlay: Option<Overlay>,
    retired_overlay: Option<Overlay>,
    menu: MenuState,
    keymap: KeyMap,
    status: Option<String>,
    status_until_ms: Option<i64>,
    last_step_at: Option<termrock::Moment>,
    quit: bool,
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
    const GO: &[MenuItem<'static>] = &[MenuItem::new(HOME, "Home")];
    const HELP_MENU: &[MenuItem<'static>] = &[
        MenuItem::new(HELP, "Key reference").chord(Chord::key(KeyCode::F(1))),
        MenuItem::new(ABOUT, "About holla"),
    ];
    const MENUS: &[Menu<'static>] = &[
        Menu::new(BAR_LABELS[0], FILE),
        Menu::new(BAR_LABELS[1], GO),
        Menu::new(BAR_LABELS[2], HELP_MENU),
    ];
    // MenuBar styles the label and the tag's " {title} " pads as one title.
    MenuBar::new(MENU, MENUS)
        .chord_case(termrock::ChordCase::UppercaseAscii)
        .hover_scope(MenuBarHoverScope::Title)
}

fn brand() -> Brand<'static> {
    Brand::new(BRAND, "holla❯").clickable(true)
}

/// Rest titles for the disk plate. Focus would bold them; the tag bar is not bold.
fn paint_disk_menu_titles(ui: &mut Ui<'_>, menu_x: u16, y: u16) {
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

/// Tag `segments::render` inset. Not the status-bar gutter: that bar fills `#111111`.
const MENU_CHROME_EDGE: u16 = 2;
const MENU_SEGMENT_GAP: u16 = 2;

/// Tag status row. Gaps are the container ink, so item text does not include them.
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

/// Activities footer clips `Ctrl+G` to `…` in meta ink. Other plates keep the theme overflow.
const ACTIVITIES_HINT_PARTS: &[(Part, StylePatch)] = &[
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
            .set_fg(Role::Custom(Color::Rgb(0x73, 0x73, 0x73)))
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

/// The `parity-history` bands with their own body: single-column below 160.
fn parity_history_tag_plate(world: &World, width: u16, _height: u16) -> bool {
    crate::sim::catalogue::parity_history_tag_frame(world) && (80..160).contains(&width)
}

/// Below 120 the single-column history clips the last hint. 120 and up keep the wide parts.
fn parity_history_clipped_plate(world: &World, width: u16, _height: u16) -> bool {
    crate::sim::catalogue::parity_history_tag_frame(world) && (80..120).contains(&width)
}

/// 80–119 clip the last hint. 120 and up fits every hint, so it keeps the wide parts.
fn parity_files_clipped_plate(world: &World, width: u16, _height: u16) -> bool {
    crate::sim::catalogue::parity_files_tag_frame(world) && (80..120).contains(&width)
}

/// 80–159 `parity-files` tag frames. Other sizes stay off.
fn parity_files_tag_plate(world: &World, width: u16, _height: u16) -> bool {
    crate::sim::catalogue::parity_files_tag_frame(world) && (80..160).contains(&width)
}

/// Below 120 the single-column browser clips the last hint. 120 and up keep the wide parts.
fn parity_browser_clipped_plate(world: &World, width: u16, _height: u16) -> bool {
    crate::sim::catalogue::parity_browser_tag_frame(world) && (80..120).contains(&width)
}

/// The `parity-browser` bands with their own body: single-column below 120, the
/// wide card from 120 up to 160. Files plates and every other size stay off.
fn parity_browser_tag_plate(world: &World, width: u16, _height: u16) -> bool {
    crate::sim::catalogue::parity_browser_tag_frame(world) && (80..160).contains(&width)
}

/// 80×24 and 100×30 clip the last hint. 120×40 fits every hint, so it keeps the wide parts.
fn parity_git_current_clipped_plate(world: &World, width: u16, _height: u16) -> bool {
    crate::sim::catalogue::parity_git_current_tag_frame(world) && (80..120).contains(&width)
}

fn parity_git_current_tag_plate(world: &World, width: u16, _height: u16) -> bool {
    crate::sim::catalogue::parity_git_current_tag_frame(world) && (80..160).contains(&width)
}

fn parity_cargo_clipped_plate(world: &World, width: u16, _height: u16) -> bool {
    crate::sim::catalogue::parity_cargo_tag_frame(world) && (80..120).contains(&width)
}

fn parity_cargo_tag_plate(world: &World, width: u16, _height: u16) -> bool {
    crate::sim::catalogue::parity_cargo_tag_frame(world) && (80..160).contains(&width)
}

fn parity_git_batch_clipped_plate(world: &World, width: u16, _height: u16) -> bool {
    crate::sim::catalogue::parity_git_batch_tag_frame(world) && (80..120).contains(&width)
}

fn parity_git_batch_tag_plate(world: &World, width: u16, _height: u16) -> bool {
    crate::sim::catalogue::parity_git_batch_tag_frame(world) && (80..160).contains(&width)
}

fn parity_task_sources_tag_plate(world: &World, width: u16, _height: u16) -> bool {
    // The task-sources chrome lives in the narrow band; the mid band keeps the
    // generic adaptive status and hints.
    crate::sim::catalogue::parity_task_sources_tag_frame(world)
        && layout::Band::of(width).is_narrow()
}

/// Tag parity-browser status at 80×24, 100×30, and 120×40: cwd, then the size label. No branch.
fn paint_parity_browser_status(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    ui.fill(area, status_ink(ui, STATUS_FG, false));
    let _ = paint_status_text(
        ui,
        area,
        area.x.saturating_add(STATUS_EDGE),
        "~/work/site",
        STATUS_FG,
        true,
    );
    let _ = paint_size_label(ui, area);
}

/// Tag parity-git-current status at 80×24, 100×30, and 120×40: cwd, branch, behind/ahead, dirty mark, then the size label.
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

/// Tag parity-cargo status at 80×24, 100×30, and 120×40: cwd, branch, then the size label. No ahead count.
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

/// Tag parity-task-sources status at 80×24: cwd, then the size label. No branch.
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

/// Tag parity-git-batch status at 80×24, 100×30, and 120×40: cwd, then the size label. No branch.
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

/// Tag parity-files status at 80×24, 100×30, and 120×40: cwd, then the size label. No branch.
fn paint_parity_files_status(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    ui.fill(area, status_ink(ui, STATUS_FG, false));
    let _ = paint_status_text(
        ui,
        area,
        area.x.saturating_add(STATUS_EDGE),
        "~/work/notes",
        STATUS_FG,
        true,
    );
    let _ = paint_size_label(ui, area);
}

/// Tag parity-history status at 80×24, 100×30, and 120×40: cwd, branch, then the size label. No ahead count.
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

/// Tag first-use status: cwd, an 11-cell `/` meter at 91%, then the size label.
fn paint_first_use_status(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    ui.fill(area, status_ink(ui, STATUS_FG, false));
    let _ = paint_status_text(
        ui,
        area,
        area.x.saturating_add(STATUS_EDGE),
        "~/scratch",
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
    let mut x = size_x.saturating_sub(STATUS_GAP).saturating_sub(meter_w);
    x = paint_status_text(ui, area, x, "/", STATUS_GREY, false).saturating_add(1);
    x = paint_status_text(ui, area, x, &"━".repeat(10), STATUS_AMBER, false);
    x = paint_status_text(ui, area, x, "─", Color::Rgb(0x3a, 0x3a, 0x3a), false).saturating_add(1);
    let _ = paint_status_text(ui, area, x, mark, STATUS_AMBER, false);
}

/// Tag remote-host status: service cwd, detached tag, then the size label.
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

/// Tag hard-cases status. The size label does not fit, so the row ends in `…`.
/// 80×24 keeps only the path. 100×30 keeps the git marks and centers the docker warning.
/// Wider frames also show the conflict count and the rebase mark.
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

/// Tag docker-cleanup status: cwd without the tilde, then the size label.
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

/// Tag disk-cleanup status: cwd, centered selection, then the 91% meter.
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

/// Tag disk-cleanup footer. The last hint does not fit, so the row ends in `…`.
/// 80×24 drops Delete, the artifact plan, and Rescan.
/// 100×30 keeps Delete and drops the artifact plan and Rescan. Cancel scan misses the tag fit budget.
/// Wider frames keep the artifact plan and Rescan.
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
    // Tag hint `fit` room is the width minus the left inset, the right inset, and the overflow gap.
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

/// Tag hint `fit` cost. Each kept hint pays its width plus the gap. A cut reserves one more gap.
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

/// Tag monorepo-root status: cwd, branch, ahead count, then the size label.
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

/// Tag activities-multi status: the monorepo cluster, then `⠋` and `4 running` chips.
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

/// Tag launch-failure status: child cwd, branch, ahead, dirty mark, failure, then the size label.
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

/// Tag launch-failure footer. `Shift+F10 Menu` misses the status, so the row shows `…`.
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
        // 80×24 keeps the ellipsis in the budget, so `Restart` drops. 120×40 does not.
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

/// Tag monorepo-child status: child cwd, branch, ahead count, dirty mark, then the size label.
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

/// 100×30 docker footer. `Esc Clear` stays. `Ctrl+↑↓ Scope` misses by one column, so the meta `…` paints.
/// No status. The wide plate keeps `Clean Docker c…`.
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

/// 80×24 docker footer. `Esc Clear` stays and `Tab Preview` becomes the meta `…`.
/// The wide plate keeps list order and the `Clean Docker c…` status.
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
    // Parity worlds have no `location.project`. These strings are the tag `draw_menu` labels.
    // Concept screens still show cwd; that path is the crumb those screens document.
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
        Scenario::FirstUse if crate::sim::catalogue::first_use_tag_frame(world) => {
            "no project".to_owned()
        }
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
    pub fn effect_revision(&self) -> u64 {
        self.world.effect_revision
    }

    pub fn fixture_time_ms(&self) -> i64 {
        self.world.now_ms()
    }

    /// The app without the usage history.
    ///
    /// The usage history is on by default, the way the frozen reference runs
    /// it. The no-history projection is a setting, not the default.
    pub fn for_scenario_without_history(scenario: Scenario, motion: Motion, frame: u64) -> Self {
        let mut app = Self::for_scenario(scenario, motion, frame);
        app.home.history = false;
        app
    }

    pub fn for_scenario(scenario: Scenario, motion: Motion, frame: u64) -> Self {
        let mut world = fixtures::world_for(scenario);
        world.seek(frame);
        world.clock.running = motion != Motion::Paused;
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
        if crate::sim::catalogue::docker_cleanup_tag_frame(&world) {
            home.seed_query(crate::sim::catalogue::DOCKER_CLEANUP_QUERY);
        }
        Self {
            world,
            motion,
            route,
            home,
            plan: None,
            activities,
            overlay: None,
            retired_overlay: None,
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
    fn destination(&mut self, destination: Destination, cx: &mut Cx<'_>) {
        match destination {
            Destination::Notice(message) => self.set_status(message),
            Destination::Preview(action) => self.open_dialog(dialogs::Intent::Preview(action), cx),
            Destination::Trust { action, file } => {
                self.open_dialog(dialogs::Intent::Trust { action, file }, cx);
            }
            Destination::Plan(review) => {
                self.plan = Some(PlanState::new(*review));
                self.route = Route::Plan;
                cx.focus(plan::STEPS);
            }
            Destination::Activity(id, message) => {
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
                if let Some(github) = &self.world.github {
                    let picker = ClonePicker::new(github);
                    picker.open(cx);
                    self.overlay = Some(Overlay::Clone(picker));
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
    fn command(&mut self, command: ActionKey, cx: &mut Cx<'_>) {
        match command {
            HELP => self.open_dialog(dialogs::Intent::Help, cx),
            TOP_FILES => self.open_dialog(dialogs::Intent::TopFiles, cx),
            CLEANUP_REVIEW => self.open_dialog(dialogs::Intent::CleanupReview, cx),
            FOLDER_BROWSER => self.open_dialog(dialogs::Intent::FolderBrowser, cx),
            HOME_FIND => self.open_dialog(dialogs::Intent::HomeFind, cx),
            CUSTOM_ACTION => self.open_dialog(dialogs::Intent::CustomAction, cx),
            SNAPSHOT_PAGE => self.open_dialog(dialogs::Intent::SnapshotPage, cx),
            DEFINITION_TRUST => self.open_dialog(dialogs::Intent::DefinitionTrust, cx),
            ABOUT => self.open_dialog(dialogs::Intent::About, cx),
            QUIT => self.open_dialog(dialogs::Intent::Quit, cx),
            INTERRUPT => self.quit = true,
            HOME => {
                self.route = Route::Home;
                cx.focus(home::QUERY);
            }
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
                    if let crate::domain::action::Availability::NeedsTrust(file) =
                        &action.availability
                    {
                        self.open_dialog(
                            dialogs::Intent::Trust {
                                file: file.clone(),
                                action,
                            },
                            cx,
                        );
                    } else {
                        self.open_dialog(dialogs::Intent::Preview(action), cx);
                    }
                }
            }
            ACTIONS if self.route == Route::Home => {
                if let Some(action) = self.home.subject(&self.world, cx) {
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
            QUERY_ESCAPE if self.route == Route::Home => {
                if self.home.scope.take().is_some() {
                    self.set_status("Scope: all".into());
                } else {
                    self.home.clear_query();
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
        match choice {
            Choice::Run => {
                let destination = dispatch::invoke(&mut self.world, &action);
                self.destination(destination, cx);
            }
            Choice::Preview => self.open_dialog(dialogs::Intent::Preview(action), cx),
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
                let (response, event) = dialog.update(cx, &mut self.world);
                let closed = event.is_some();
                if let Some(event) = event {
                    match event {
                        dialogs::Event::Close => {}
                        dialogs::Event::Quit => self.quit = true,
                        dialogs::Event::Destination(destination) => {
                            self.destination(*destination, cx);
                        }
                    }
                }
                (response, closed)
            }
            Overlay::Clone(picker) => {
                let (response, chosen, closed) = picker.update(cx);
                let done = chosen.is_some() || closed;
                if let Some((repo, login)) = chosen {
                    self.open_dialog(dialogs::Intent::Clone { repo, login }, cx);
                }
                (response, done)
            }
            Overlay::Actions(picker) => {
                let (response, choice, closed) = picker.update(cx);
                let done = choice.is_some() || closed;
                if let Some(choice) = choice {
                    match picker.reviewed_action(&self.world) {
                        Ok(action) => self.action_choice(choice, action, cx),
                        Err(reason) => self.set_status(reason.into()),
                    }
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
    fn strip_draw(&self, ui: &mut Ui<'_>, area: Rect) {
        if self.world.activities.is_empty() || area.is_empty() {
            return;
        }
        ui.register_control(STRIP, area, Focusability::ClickOnly);
        let mut x = area.x;
        for (index, activity) in activity::ordered(&self.world).into_iter().enumerate() {
            let label = format!(
                " {} {} {} ",
                activity::state_glyph(activity.state),
                index.saturating_add(1),
                activity.name
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
    fn chrome(&self, ui: &mut Ui<'_>, area: Rect) {
        let regions = AppShell::new()
            .header(1)
            .gap(1)
            .footer(1)
            .inset(1)
            .regions(area);
        let header = regions.header;
        // Tag lockup starts at x + chrome_edge - 1. One more column before this bar matches the tag's post-lockup gap (its title pad is the other).
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
        // A disk plate has no query, so the bar would keep focus and bold its titles.
        if (self.route == Route::Disk && crate::sim::catalogue::disk_cleanup_tag_frame(&self.world))
            || launch_failure_activity_open(&self.world, ui)
        {
            paint_disk_menu_titles(ui, menu_x, header.y);
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
            || crate::sim::catalogue::first_use_tag_frame(&self.world)
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
                paint_here_band(ui, area);
                if crate::sim::catalogue::activities_multi_tag_frame(&self.world) {
                    paint_activities_multi_chips(ui, area);
                }
            }
        }
        let status = Rect::new(area.x, area.bottom().saturating_sub(2), area.width, 1);
        if self.world.scenario == Scenario::ParityDiscovery {
            self.paint_parity_status(ui, status);
        } else if crate::sim::catalogue::first_use_tag_frame(&self.world) {
            paint_first_use_status(ui, status);
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
            paint_parity_files_status(ui, status);
        } else if parity_browser_tag_plate(&self.world, ui.full().width, ui.full().height) {
            paint_parity_browser_status(ui, status);
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
        self.footer_draw(ui, footer);
    }
    fn paint_parity_status(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        ui.fill(area, status_ink(ui, STATUS_FG, false));
        // 80×24 has no room for the branch or the size label. The warning sits on the right, with `…` before it.
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
    fn footer_draw(&self, ui: &mut Ui<'_>, footer: Rect) {
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
        } else if crate::sim::catalogue::first_use_tag_frame(&self.world) {
            Some(("Open", None))
        } else if crate::sim::catalogue::monorepo_root_tag_frame(&self.world) {
            let narrow = layout::Band::of(ui.full().width).is_narrow();
            let mid = layout::Band::of(ui.full().width).is_mid();
            if self.home.history {
                // Wide history keeps the deploy status. 80×24 and 100×30 have no room for it.
                Some((
                    "Trust…",
                    if narrow || mid {
                        None
                    } else {
                        Some("Deploy …")
                    },
                ))
            } else if narrow || mid {
                // 100×30 fits `Ctrl+↑↓ Scope` and clips `Ctrl+G` to the meta `…`. No status.
                Some(("Open", None))
            } else {
                Some(("Open", Some("Inspect 1…")))
            }
        } else if crate::sim::catalogue::monorepo_child_tag_frame(&self.world) {
            let narrow = layout::Band::of(ui.full().width).is_narrow();
            let mid = layout::Band::of(ui.full().width).is_mid();
            // 100×30 fits `Ctrl+↑↓ Scope` and clips `Ctrl+G` to the meta `…`. No status.
            if narrow || mid {
                Some(("Open", None))
            } else {
                Some(("Open", Some("Inspect 3…")))
            }
        } else if crate::sim::catalogue::remote_host_tag_frame(&self.world) {
            let narrow = layout::Band::of(ui.full().width).is_narrow();
            let mid = layout::Band::of(ui.full().width).is_mid();
            // 100×30 fits `Ctrl+↑↓ Scope` and clips `Ctrl+G` to the meta `…`. No status.
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
            // The wide card keeps every hint. The title is cut to the ten cells left of the edge.
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
            // `Trust…` is two columns wider than `Open`. HintBar keeps the gap
            // before a later hint but not the overflow glyph, so `Ctrl+↑↓ Scope`
            // still fits and the `…` never paints. One undisplayed column makes
            // that hint miss the budget. The no-history plate stays on `Open`.
            if self.home.history
                && let Some(scope) = layer.hints.iter_mut().find(|hint| hint.label == "Scope")
            {
                scope.label = "Scope ";
            }
            HintBar::new(FOOTER, &layer)
                .patch_part(ACTIVITIES_HINT_PARTS)
                .draw(ui, footer);
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
            let parts = if narrow || mid {
                ACTIVITIES_HINT_PARTS
            } else {
                PARITY_HINT_PARTS
            };
            HintBar::new(FOOTER, &layer)
                .patch_part(parts)
                .draw(ui, footer);
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
        if let Some((confirm, status)) = tag_confirm
            && self.route == Route::Home
            && self.overlay.is_none()
            && !self.menu.is_open()
        {
            let layer = tag_hints(confirm, status);
            // 80×24 first-use, monorepo-root, monorepo-child, remote-host, hard-cases, and parity-discovery clip the last hint to a meta `…`. The 100×30 first-use, monorepo-root, monorepo-child, remote-host, hard-cases, and parity-discovery plates do the same. Wider plates keep the theme overflow.
            let narrow = layout::Band::of(ui.full().width).is_narrow();
            let mid = layout::Band::of(ui.full().width).is_mid();
            let monorepo_child_mid =
                mid && crate::sim::catalogue::monorepo_child_tag_frame(&self.world);
            let remote_host_mid = mid && crate::sim::catalogue::remote_host_tag_frame(&self.world);
            let first_use_mid = mid && crate::sim::catalogue::first_use_tag_frame(&self.world);
            let monorepo_root_mid =
                mid && crate::sim::catalogue::monorepo_root_tag_frame(&self.world);
            let hard_cases_mid = mid && crate::sim::catalogue::hard_cases_tag_frame(&self.world);
            let discovery_mid =
                mid && crate::sim::catalogue::parity_discovery_tag_frame(&self.world);
            let history_plate =
                parity_history_clipped_plate(&self.world, ui.full().width, ui.full().height);
            let files_plate =
                parity_files_clipped_plate(&self.world, ui.full().width, ui.full().height);
            let browser_plate =
                parity_browser_clipped_plate(&self.world, ui.full().width, ui.full().height);
            let git_current_plate =
                parity_git_current_clipped_plate(&self.world, ui.full().width, ui.full().height);
            let cargo_plate =
                parity_cargo_clipped_plate(&self.world, ui.full().width, ui.full().height);
            let git_batch_plate =
                parity_git_batch_clipped_plate(&self.world, ui.full().width, ui.full().height);
            let task_sources_plate =
                parity_task_sources_tag_plate(&self.world, ui.full().width, ui.full().height);
            let parts = if monorepo_child_mid
                || remote_host_mid
                || first_use_mid
                || monorepo_root_mid
                || hard_cases_mid
                || discovery_mid
                || history_plate
                || files_plate
                || browser_plate
                || git_current_plate
                || cargo_plate
                || git_batch_plate
                || task_sources_plate
                || (narrow
                    && (crate::sim::catalogue::first_use_tag_frame(&self.world)
                        || crate::sim::catalogue::monorepo_root_tag_frame(&self.world)
                        || crate::sim::catalogue::monorepo_child_tag_frame(&self.world)
                        || crate::sim::catalogue::remote_host_tag_frame(&self.world)
                        || crate::sim::catalogue::hard_cases_tag_frame(&self.world)
                        || self.world.scenario == Scenario::ParityDiscovery))
            {
                ACTIVITIES_HINT_PARTS
            } else {
                PARITY_HINT_PARTS
            };
            HintBar::new(FOOTER, &layer)
                .patch_part(parts)
                .draw(ui, footer);
            return;
        }
        let hints = HintLayer::from_bindings(GLOBAL);
        let editing = match &self.overlay {
            Some(Overlay::Dialog(dialog)) => dialog.is_editing(),
            Some(Overlay::Gate(gate)) => gate.is_editing(),
            Some(Overlay::Clone(_) | Overlay::Actions(_)) => false,
            None => self.route == Route::Home && self.home.is_editing(),
        };
        let home_hints = (self.route == Route::Home
            && self.overlay.is_none()
            && !self.menu.is_open())
        .then(|| {
            HomeState::hints(
                &self.world,
                ui.state(home::ROWS).contains(termrock::StateFlags::FOCUSED),
            )
        });
        let actions_hints = matches!(self.overlay, Some(Overlay::Actions(_))).then(Actions::hints);
        let mut bar = HintBar::derived(FOOTER).screen(&hints);
        if let Some(mode_hints) = actions_hints.as_ref().or(home_hints.as_ref()) {
            bar = bar.mode(mode_hints);
        }
        bar.status_text(self.status.as_deref())
            .badge(editing.then_some("EDIT"))
            .centered(true)
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
    fn refresh_query_bindings(&mut self, cx: &Cx<'_>) {
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

    /// One sim step of `world.tick`. That call moves `now_ms` only while the clock is running.
    fn advance_world(&mut self, interval_ms: i64) {
        let _ = self.world.tick(interval_ms);
        self.home.release_tag_query(&self.world);
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
        let retired = self
            .retired_overlay
            .take()
            .map_or_else(Response::ignored, |mut overlay| match &mut overlay {
                Overlay::Dialog(dialog) => dialog.poll(cx).0,
                Overlay::Gate(gate) => gate.poll(cx).0,
                Overlay::Clone(picker) => picker.update(cx).0,
                Overlay::Actions(picker) => picker.update(cx).0,
            });

        let (home_response, home_action) = self.home.update(&self.world, cx);
        let (plan_response, plan_event) = self
            .plan
            .as_mut()
            .map_or_else(|| (Response::ignored(), None), |plan| plan.update(cx));
        let activity_response = self.activities.update(&self.world, cx);
        let mut menu = menu_bar().update(cx, &mut self.menu);
        let brand = brand().update(cx);
        let controls = retired | home_response | plan_response | activity_response;
        if too_small(cx.viewport()) {
            let overlay = self
                .overlay
                .as_mut()
                .map_or_else(Response::ignored, |overlay| match overlay {
                    Overlay::Dialog(dialog) => dialog.poll(cx).0,
                    Overlay::Gate(gate) => gate.poll(cx).0,
                    Overlay::Clone(picker) => picker.update(cx).0,
                    Overlay::Actions(picker) => picker.update(cx).0,
                });
            if cx.update_cause() == termrock::UpdateCause::Event && cx.command() == Some(INTERRUPT)
            {
                self.quit = true;
            }
            return controls | overlay | menu.erase() | brand.erase() | Response::consumed();
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
            self.command(command, cx);
            return controls | menu.erase() | brand.erase();
        }
        if self.menu.is_open() {
            return controls | menu.erase() | brand.erase();
        }
        if let Some(command) = cx
            .command()
            .filter(|_| cx.update_cause() == termrock::UpdateCause::Event)
        {
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
            | Route::Files
            | Route::Cleanup
            | Route::Review
            | Route::Snapshot => {}
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
        let plan_key = self.route == Route::Plan
            && self.overlay.is_none()
            && !self.menu.is_open()
            && !too_small(cx.viewport())
            && cx.activation_key().is_some();
        let response = self.update_controls(cx);
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
        let regions = AppShell::new()
            .header(1)
            .gap(1)
            .footer(1)
            .inset(1)
            .regions(area);
        let body = regions.body;
        match self.route {
            Route::Home => self.home.draw(&self.world, ui, home_body(body)),
            Route::Plan => {
                if let Some(plan) = &self.plan {
                    plan.draw(ui, body);
                }
            }
            Route::Activity => {
                let plate = launch_failure_activity_open(&self.world, ui)
                    && home::paint_launch_failure_plate(ui);
                if !plate {
                    self.activities.draw(&self.world, ui, body);
                }
            }
            Route::Disk => {
                // The 100×30 fixture composes the mid preview; every other size only
                // needs the tag frame gate (the old 100×30 plate wrapper was a no-op
                // over it, so the size check is gone).
                let allow = crate::sim::catalogue::disk_cleanup_tag_frame(&self.world);
                let plate = allow && home::paint_disk_cleanup_plate(ui);
                if !plate {
                    disk::draw(&self.world, ui, body);
                }
            }
            Route::Files => files::draw(&self.world, ui, body),
            Route::Cleanup => cleanup::draw(&self.world, ui, body),
            Route::Review => review::draw(&self.world, ui, body),
            Route::Snapshot => snapshot::draw(&self.world, ui, body),
        }
        self.chrome(ui, area);
        if let Some(overlay) = &self.overlay {
            match overlay {
                Overlay::Dialog(dialog) => dialog.draw(ui),
                Overlay::Clone(picker) => picker.draw(ui),
                Overlay::Actions(picker) => picker.draw(ui),
                Overlay::Gate(gate) => gate.draw(ui),
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
        if self.route != Route::Home {
            self.route = Route::Home;
            cx.focus(home::QUERY);
            return Response::changed();
        }
        if self.home.scope.take().is_some() {
            return Response::changed();
        }
        if !self.home.query().is_empty() {
            self.home.clear_query();
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

/// Tag body starts under a 4-row header. `body` is the port's y+2 frame, so Home drops two more rows for the band and one for the tag footer gap.
fn home_body(body: Rect) -> Rect {
    Rect::new(
        body.x,
        body.y.saturating_add(2),
        body.width,
        body.height.saturating_sub(3),
    )
}

/// Tag `Tabs` active item for this boot: width `1 + label + 2`, bold primary on `#27272a`, accent `━`, then `#3a3a3a` `─`.
fn paint_here_band(ui: &mut Ui<'_>, area: Rect) {
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
    let accent = ui.paint_patch(&StylePatch::new().set_fg(Role::Accent));
    // Tag `border_subtle` is `#3a3a3a`. The port `BorderSubtle` token is `#262626`.
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

/// Collapsed activities band. Here sits four columns in. Two chips stay off-screen as `2›` and `+2`.
/// 80×24 and 100×30 share the three visible chips. The markers sit nine columns from the right.
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

/// Tag activity chips sit on the Here row. The live strip stays on the row above for later seeks.
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

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "Deterministic product routes and named fixtures must be present"
)]
mod tests {
    use super::*;
    use crate::sim::catalogue;
    use termrock::{Dialog, Theme};
    use termrock_test_support::Harness;
    fn harness(app: App, theme: Theme, width: u16, height: u16) -> Harness<App> {
        let initial =
            termrock::SimulationMoment::from_millis(u64::try_from(app.fixture_time_ms()).unwrap());
        Harness::new_with_feedback_clock(
            app,
            theme,
            width,
            height,
            termrock::FeedbackClock::Simulation { initial },
        )
    }
    fn app(scenario: Scenario) -> Harness<App> {
        harness(
            App::for_scenario_without_history(scenario, Motion::Paused, 4_000),
            Theme::junie(),
            120,
            40,
        )
    }
    #[test]
    fn menu_bar_pointer_hover_changes_only_the_file_title_and_clears_on_exit() {
        let mut scene = app(Scenario::FirstUse);
        let before = scene.snapshot();
        let (file_x, file_y) = scene.find("File").expect("menu bar File title");
        assert_eq!(file_y, 0);
        assert!(file_x > 0);
        let x = file_x - 1;
        let _ = scene.mouse(termrock::MouseKind::Move, file_x, file_y);

        for column in x..x + 6 {
            let cell = scene.cell(column, file_y);
            assert_eq!(cell.bg, Color::Rgb(0x1c, 0x1c, 0x1f), "column {column}");
            assert_eq!(cell.fg, Color::Rgb(255, 255, 255), "column {column}");
        }
        for column in 0..120 {
            if !(x..x + 6).contains(&column) {
                let expected = before.buffer().cell((column, file_y)).unwrap();
                assert_eq!(scene.cell(column, file_y), expected, "column {column}");
            }
        }

        let cursor = scene.cursor();
        let _ = scene.mouse(termrock::MouseKind::Move, 119, 39);
        assert_eq!(scene.snapshot().buffer(), before.buffer());
        assert_eq!(scene.cursor(), cursor);
    }
    #[test]
    fn discovery_boot_paints_the_here_band_above_the_filter() {
        let harness = harness(
            App::for_scenario_without_history(Scenario::ParityDiscovery, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        let text = harness.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines[2].contains("Here"), "{}", lines[2]);
        assert!(!lines[2].contains('▎'), "{}", lines[2]);
        assert!(lines[3].contains('━'), "{}", lines[3]);
        assert!(lines[4].contains("Search actions"), "{}", lines[4]);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn monorepo_root_frame_40_paints_the_tag_plate() {
        let harness = harness(
            App::for_scenario_without_history(Scenario::MonorepoRoot, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        let text = harness.text();
        assert!(text.contains("acme · mise monorepo"), "{text}");
        assert!(text.contains("Follow acme-api-1 logs"), "{text}");
        assert!(text.contains("Inspect 10 GB of generat"), "{text}");
        assert!(text.contains("~/work/acme"), "{text}");
        assert!(text.contains("↑1"), "{text}");
        assert!(text.contains("Inspect 1…"), "{text}");
        assert!(text.contains("2 rebuildable candidates"), "{text}");
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn monorepo_root_80x24_frame_40_paints_the_selected_narrow_row_and_later_seeks_keep_home() {
        let plate = harness(
            App::for_scenario_without_history(Scenario::MonorepoRoot, Motion::Paused, 40),
            Theme::junie(),
            80,
            24,
        );
        let text = plate.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(
            lines[8].contains("› Inspect 10 GB of generate…"),
            "{}",
            lines[8]
        );
        assert!(
            lines[8].contains("10 GB generated artifacts"),
            "{}",
            lines[8]
        );
        assert!(
            lines[9].contains("Install missing tools · s…"),
            "{}",
            lines[9]
        );
        assert!(lines[18].contains("Children · 3 projects"), "{}", lines[18]);
        assert!(
            lines[20].contains("→ 'holla disk analyze --artifacts' · here · read-only"),
            "{}",
            lines[20]
        );
        assert!(!text.contains("System · mbp"), "{text}");
        assert!(!text.contains("Inspect 1…"), "{text}");
        let selected = plate.cell(4, 8);
        assert_eq!(selected.symbol(), "I");
        assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
        assert!(selected.modifier.contains(termrock::Modifier::BOLD));
        let kind = plate.cell(32, 8);
        assert_eq!(kind.symbol(), "d");
        assert!(!kind.modifier.contains(termrock::Modifier::BOLD));
        assert_eq!(kind.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
        let thumb = plate.cell(78, 8);
        assert_eq!(thumb.symbol(), "┃");
        assert_eq!(thumb.fg, Color::Rgb(0x80, 0x80, 0x80));
        assert_eq!(thumb.bg, Color::Rgb(0, 0, 0));
        let track = plate.cell(78, 18);
        assert_eq!(track.symbol(), "┆");
        assert_eq!(track.fg, Color::Rgb(0x47, 0x47, 0x47));
        let faded = plate.cell(4, 18);
        assert_eq!(faded.symbol(), "C");
        assert_eq!(faded.fg, Color::Rgb(140, 140, 140));
        let ellipsis = plate.cell(73, 23);
        assert_eq!(ellipsis.symbol(), "…");
        assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
        assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
        let world = &plate.app().world;
        assert!(catalogue::monorepo_root_view(world, 120, false).is_none());
        assert_ne!(
            catalogue::monorepo_root_view(world, 80, true),
            catalogue::monorepo_root_view(world, 80, false)
        );
        let view = catalogue::monorepo_root_view(world, 80, false).expect("narrow view");
        assert!(view.lines.iter().any(|line| matches!(
            line,
            catalogue::FinderLine::Row(row) if row.selected && row.title.starts_with("Inspect 10 GB")
        )));
        let wide = harness(
            App::for_scenario_without_history(Scenario::MonorepoRoot, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        assert!(
            wide.text().contains("2 rebuildable candidates"),
            "{}",
            wide.text()
        );
        assert!(!wide.text().contains("holla disk analyze --artifacts"));
        assert!(wide.text().contains("Inspect 1…"), "{}", wide.text());
        let later = harness(
            App::for_scenario_without_history(Scenario::MonorepoRoot, Motion::Paused, 4_000),
            Theme::junie(),
            80,
            24,
        );
        assert!(!later.text().contains("holla disk analyze --artifacts"));
        assert!(!later.text().contains("› Inspect 10 GB"));
        assert!(later.text().contains("Search actions"), "{}", later.text());
        assert!(catalogue::monorepo_root_view(&later.app().world, 80, false).is_none());
        assert!(!catalogue::monorepo_root_tag_frame(&later.app().world));
    }
    #[test]
    fn monorepo_root_100x30_frame_40_paints_the_mid_plate_and_later_seeks_keep_home() {
        let plate = harness(
            App::for_scenario_without_history(Scenario::MonorepoRoot, Motion::Paused, 40),
            Theme::junie(),
            100,
            30,
        );
        let text = plate.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(
            lines[8].contains("› Inspect 10 GB of generated artif…"),
            "{}",
            lines[8]
        );
        assert!(
            lines[8].contains("10 GB generated artifacts"),
            "{}",
            lines[8]
        );
        assert!(
            lines[9].contains("Install missing tools · sqlx-cli"),
            "{}",
            lines[9]
        );
        assert!(!lines[9].contains("node"), "{}", lines[9]);
        assert!(lines[19].contains("System · mbp"), "{}", lines[19]);
        assert!(
            lines[26].contains("→ 'holla disk analyze --artifacts' · here · read-only"),
            "{}",
            lines[26]
        );
        assert!(!text.contains("2 rebuildable candidates"), "{text}");
        assert!(lines[29].contains("Enter Open"), "{}", lines[29]);
        assert!(lines[29].contains("Ctrl+↑↓ Scope"), "{}", lines[29]);
        assert!(!lines[29].contains("Ctrl+G"), "{}", lines[29]);
        assert!(!lines[29].contains("Inspect 1…"), "{}", lines[29]);
        let selected = plate.cell(4, 8);
        assert_eq!(selected.symbol(), "I");
        assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
        assert!(selected.modifier.contains(termrock::Modifier::BOLD));
        let kind = plate.cell(39, 8);
        assert_eq!(kind.symbol(), "d");
        assert!(!kind.modifier.contains(termrock::Modifier::BOLD));
        assert_eq!(kind.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
        let ellipsis = plate.cell(88, 29);
        assert_eq!(ellipsis.symbol(), "…");
        assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
        assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
        let world = &plate.app().world;
        assert!(catalogue::monorepo_root_view(world, 100, false).is_some());
        assert!(catalogue::monorepo_root_view(world, 120, false).is_none());
        assert_ne!(
            catalogue::monorepo_root_view(world, 100, true),
            catalogue::monorepo_root_view(world, 100, false)
        );
        let wide = harness(
            App::for_scenario_without_history(Scenario::MonorepoRoot, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        assert!(wide.text().contains("Inspect 1…"), "{}", wide.text());
        assert!(
            wide.text().contains("2 rebuildable candidates"),
            "{}",
            wide.text()
        );
        let narrow = harness(
            App::for_scenario_without_history(Scenario::MonorepoRoot, Motion::Paused, 40),
            Theme::junie(),
            80,
            24,
        );
        assert!(
            narrow.text().contains("Install missing tools · s…"),
            "{}",
            narrow.text()
        );
        assert!(!narrow.text().contains("System · mbp"), "{}", narrow.text());
        let later = harness(
            App::for_scenario_without_history(Scenario::MonorepoRoot, Motion::Paused, 4_000),
            Theme::junie(),
            100,
            30,
        );
        assert!(!later.text().contains("holla disk analyze --artifacts"));
        assert!(catalogue::monorepo_root_view(&later.app().world, 100, false).is_none());
        assert!(!catalogue::monorepo_root_tag_frame(&later.app().world));
        let other = App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 40);
        assert!(catalogue::monorepo_root_view(&other.world, 100, false).is_none());
        let mut history =
            App::for_scenario_without_history(Scenario::MonorepoRoot, Motion::Paused, 40);
        history.home.history = true;
        assert!(catalogue::monorepo_root_view(&history.world, 100, true).is_some());
        let history_screen = harness(history, Theme::junie(), 100, 30);
        assert!(
            history_screen
                .text()
                .contains("→ tools/deploy-preview.sh --env preview · here · mutating"),
            "{}",
            history_screen.text()
        );
        assert!(
            !history_screen.text().contains("Deploy …"),
            "{}",
            history_screen.text()
        );
        assert!(
            !history_screen
                .text()
                .contains("→ 'holla disk analyze --artifacts' · here · read-only"),
            "{}",
            history_screen.text()
        );
    }
    #[test]
    fn monorepo_root_history_frame_40_paints_the_deploy_tag_and_later_seeks_keep_home() {
        let mut app = App::for_scenario_without_history(Scenario::MonorepoRoot, Motion::Paused, 40);
        app.home.history = true;
        let screen = harness(app, Theme::junie(), 120, 40);
        let text = screen.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines[7].contains("› Deploy preview"), "{}", lines[7]);
        assert!(lines[7].contains("Current folder…"), "{}", lines[7]);
        assert!(lines[8].contains("Follow acme-api-1 logs"), "{}", lines[8]);
        assert!(lines[8].contains("push the current branch"), "{}", lines[8]);
        assert!(
            lines[10].contains("tools/deploy-pre…preview"),
            "{}",
            lines[10]
        );
        assert!(lines[14].contains("Pull"), "{}", lines[14]);
        assert!(lines[14].contains("used 20 times on mbp"), "{}", lines[14]);
        assert!(lines[18].contains("mutating"), "{}", lines[18]);
        assert!(lines[20].contains("trust required"), "{}", lines[20]);
        assert!(lines[39].contains("Enter Trust…"), "{}", lines[39]);
        assert!(lines[39].contains("Deploy …"), "{}", lines[39]);
        assert!(!text.contains("nothing used here yet"), "{text}");
        assert!(!text.contains("holla disk anal"), "{text}");
        let selected = screen.cell(2, 7);
        assert_eq!(selected.symbol(), "›");
        assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
        assert!(selected.modifier.contains(termrock::Modifier::BOLD));
        let runs = screen.cell(92, 10);
        assert_eq!(runs.symbol(), "t");
        assert_eq!(runs.fg, Color::Rgb(0xb3, 0xb3, 0xb3));
        let risk = screen.cell(92, 18);
        assert_eq!(risk.symbol(), "m");
        assert_eq!(risk.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        assert!(
            screen.diagnostics().is_empty(),
            "{:?}",
            screen.diagnostics()
        );
        let mut later_app =
            App::for_scenario_without_history(Scenario::MonorepoRoot, Motion::Paused, 4_000);
        later_app.home.history = true;
        let later = harness(later_app, Theme::junie(), 120, 40);
        assert!(!later.text().contains("tools/deploy-pre"));
        assert!(!later.text().contains("used 20 times on mbp"));
        assert!(later.text().contains("Search actions"), "{}", later.text());
        let mut stepped =
            App::for_scenario_without_history(Scenario::MonorepoRoot, Motion::Paused, 40);
        stepped.home.history = true;
        assert!(catalogue::monorepo_root_history_tag(&stepped.world).is_some());
        assert!(termrock::App::accept_paused_clock_step(&mut stepped, 1_100));
        assert!(catalogue::monorepo_root_history_tag(&stepped.world).is_none());
        assert!(!catalogue::monorepo_root_tag_frame(&stepped.world));
    }
    #[test]
    fn monorepo_child_frame_40_paints_the_tag_plate_and_later_seeks_keep_home() {
        let plate = harness(
            App::for_scenario_without_history(Scenario::MonorepoChild, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        let text = plate.text();
        assert!(
            text.contains("acme › frontend · Node project · pnpm"),
            "{text}"
        );
        assert!(text.contains("From acme · the parent root"), "{text}");
        assert!(text.contains("Inspect 3 GB of generate"), "{text}");
        assert!(text.contains("• 2"), "{text}");
        assert!(text.contains("Inspect 3…"), "{text}");
        assert!(text.contains("~/work/acme/apps…rontend"), "{text}");
        assert!(text.contains("1 rebuildable candidate"), "{text}");
        assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
        let later = harness(
            App::for_scenario_without_history(Scenario::MonorepoChild, Motion::Paused, 4_000),
            Theme::junie(),
            120,
            40,
        );
        assert!(!later.text().contains("From acme · the parent root"));
        assert!(later.text().contains("Run dev"), "{}", later.text());
    }
    #[test]
    fn monorepo_child_80x24_frame_40_paints_the_narrow_plate_and_later_seeks_keep_home() {
        let plate = harness(
            App::for_scenario_without_history(Scenario::MonorepoChild, Motion::Paused, 40),
            Theme::junie(),
            80,
            24,
        );
        let text = plate.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(
            lines[7].contains("› Inspect 3 GB of generated…"),
            "{}",
            lines[7]
        );
        assert!(
            lines[7].contains("3 GB generated artifacts"),
            "{}",
            lines[7]
        );
        assert!(
            lines[8].contains("Install missing tools · s…"),
            "{}",
            lines[8]
        );
        assert!(
            lines[16].contains("From acme · the parent root"),
            "{}",
            lines[16]
        );
        assert!(
            lines[20].contains("→ 'holla disk analyze --artifacts' · here · read-only"),
            "{}",
            lines[20]
        );
        assert!(lines[23].contains("Tab Preview"), "{}", lines[23]);
        assert!(!text.contains("Inspect 3…"), "{text}");
        let reason_end = plate.cell(76, 9);
        assert_eq!(reason_end.symbol(), "…");
        assert_eq!(reason_end.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        let before_thumb = plate.cell(77, 9);
        assert_eq!(before_thumb.symbol(), " ");
        assert!(!text.contains("System · mbp"), "{text}");
        let selected = plate.cell(4, 7);
        assert_eq!(selected.symbol(), "I");
        assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
        assert!(selected.modifier.contains(termrock::Modifier::BOLD));
        let thumb = plate.cell(78, 7);
        assert_eq!(thumb.symbol(), "┃");
        assert_eq!(thumb.fg, Color::Rgb(0x80, 0x80, 0x80));
        let track = plate.cell(78, 18);
        assert_eq!(track.symbol(), "┆");
        assert_eq!(track.fg, Color::Rgb(0x47, 0x47, 0x47));
        let faded = plate.cell(4, 18);
        assert_eq!(faded.symbol(), "S");
        assert_eq!(faded.fg, Color::Rgb(140, 140, 140));
        let ellipsis = plate.cell(73, 23);
        assert_eq!(ellipsis.symbol(), "…");
        assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
        assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
        let world = &plate.app().world;
        assert!(catalogue::monorepo_child_view(world, 120).is_none());
        let wide = harness(
            App::for_scenario_without_history(Scenario::MonorepoChild, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        assert!(
            wide.text().contains("1 rebuildable candidate"),
            "{}",
            wide.text()
        );
        assert!(!wide.text().contains("holla disk analyze --artifacts"));
        assert!(wide.text().contains("Inspect 3…"), "{}", wide.text());
        let later = harness(
            App::for_scenario_without_history(Scenario::MonorepoChild, Motion::Paused, 4_000),
            Theme::junie(),
            80,
            24,
        );
        assert!(!later.text().contains("holla disk analyze --artifacts"));
        assert!(!later.text().contains("› Inspect 3 GB"));
        assert!(later.text().contains("Search actions"), "{}", later.text());
        assert!(catalogue::monorepo_child_view(&later.app().world, 80).is_none());
        assert!(!catalogue::monorepo_child_tag_frame(&later.app().world));
    }
    #[test]
    fn monorepo_child_100x30_frame_40_paints_the_mid_plate_and_later_seeks_keep_home() {
        let plate = harness(
            App::for_scenario_without_history(Scenario::MonorepoChild, Motion::Paused, 40),
            Theme::junie(),
            100,
            30,
        );
        let text = plate.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(
            lines[7].contains("› Inspect 3 GB of generated artifa…"),
            "{}",
            lines[7]
        );
        assert!(
            lines[7].contains("3 GB generated artifacts"),
            "{}",
            lines[7]
        );
        assert!(
            lines[8].contains("Install missing tools · sqlx-cli"),
            "{}",
            lines[8]
        );
        assert!(
            lines[16].contains("From acme · the parent root"),
            "{}",
            lines[16]
        );
        assert!(
            lines[26].contains("→ 'holla disk analyze --artifacts' · here · read-only"),
            "{}",
            lines[26]
        );
        assert!(!lines[26].contains("Inspect 3"), "{}", lines[26]);
        assert!(lines[29].contains("Ctrl+↑↓ Scope"), "{}", lines[29]);
        assert!(!text.contains("Inspect 3…"), "{text}");
        assert!(!text.contains("1 rebuildable candidate"), "{text}");
        let ellipsis = plate.cell(88, 29);
        assert_eq!(ellipsis.symbol(), "…");
        assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
        assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
        assert!(catalogue::monorepo_child_view(&plate.app().world, 120).is_none());
        let wide = harness(
            App::for_scenario_without_history(Scenario::MonorepoChild, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        assert!(
            wide.text().contains("1 rebuildable candidate"),
            "{}",
            wide.text()
        );
        assert!(wide.text().contains("Inspect 3…"), "{}", wide.text());
        assert!(!wide.text().contains("'holla disk analyze --artifacts'"));
        let narrow = harness(
            App::for_scenario_without_history(Scenario::MonorepoChild, Motion::Paused, 40),
            Theme::junie(),
            80,
            24,
        );
        assert!(
            narrow.text().contains("Inspect 3 GB of generated…"),
            "{}",
            narrow.text()
        );
        assert!(!narrow.text().contains("generated artifa…"));
        let later = harness(
            App::for_scenario_without_history(Scenario::MonorepoChild, Motion::Paused, 4_000),
            Theme::junie(),
            100,
            30,
        );
        assert!(!later.text().contains("From acme · the parent root"));
        assert!(later.text().contains("Search actions"), "{}", later.text());
        assert!(catalogue::monorepo_child_view(&later.app().world, 100).is_none());
    }
    #[test]
    fn docker_cleanup_frame_40_paints_the_tag_plate_and_later_seeks_keep_home() {
        let plate = harness(
            App::for_scenario_without_history(Scenario::DockerCleanup, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        let text = plate.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines[0].contains("no project"), "{}", lines[0]);
        assert!(lines[0].contains("devbox · development"), "{}", lines[0]);
        assert!(lines[4].contains("docker clean"), "{}", lines[4]);
        assert!(lines[6].contains("Results · 7"), "{}", lines[6]);
        assert!(
            lines[7].contains("› Clean Docker completely"),
            "{}",
            lines[7]
        );
        assert!(lines[8].contains("Prune builder cache"), "{}", lines[8]);
        assert!(
            lines[15].contains("docker stop acme…redis-1"),
            "{}",
            lines[15]
        );
        assert!(lines[38].contains("/work/scratch"), "{}", lines[38]);
        assert!(lines[39].contains("Enter Review…"), "{}", lines[39]);
        assert!(lines[39].contains("Esc Clear"), "{}", lines[39]);
        assert!(lines[39].contains("Clean Docker c…"), "{}", lines[39]);
        let query = plate.cell(3, 4);
        assert_eq!(query.symbol(), "d");
        assert!(query.modifier.contains(termrock::Modifier::UNDERLINED));
        let thumb = plate.cell(116, 8);
        assert_eq!(thumb.symbol(), "┃");
        assert_eq!(thumb.fg, Color::Rgb(0x80, 0x80, 0x80));
        let track = plate.cell(116, 34);
        assert_eq!(track.symbol(), "┆");
        assert_eq!(track.fg, Color::Rgb(0x47, 0x47, 0x47));
        let gate = plate.cell(92, 34);
        assert_eq!(gate.fg, Color::Rgb(207, 207, 207));
        assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
        let later = harness(
            App::for_scenario_without_history(Scenario::DockerCleanup, Motion::Paused, 4_000),
            Theme::junie(),
            120,
            40,
        );
        assert!(!later.text().contains("Results · 7"));
        assert!(later.text().contains("Suggested here"), "{}", later.text());
        assert!(later.app().home.query().is_empty());
        let mut stepped =
            App::for_scenario_without_history(Scenario::DockerCleanup, Motion::Paused, 40);
        assert!(termrock::App::accept_paused_clock_step(&mut stepped, 1_100));
        assert!(stepped.home.query().is_empty());
        assert!(!crate::sim::catalogue::docker_cleanup_tag_frame(
            &stepped.world
        ));
    }
    #[test]
    fn docker_cleanup_80x24_frame_40_paints_the_narrow_plate_and_later_seeks_keep_home() {
        let plate = harness(
            App::for_scenario_without_history(Scenario::DockerCleanup, Motion::Paused, 40),
            Theme::junie(),
            80,
            24,
        );
        let text = plate.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines[6].contains("Results · 7"), "{}", lines[6]);
        assert!(
            lines[7].contains("› Clean Docker completely"),
            "{}",
            lines[7]
        );
        assert!(lines[7].contains("18.4 GB recla…"), "{}", lines[7]);
        assert!(
            lines[13].contains("Stop and remove all conta…"),
            "{}",
            lines[13]
        );
        assert!(
            lines[20].contains(
                "→ docker stop acme-api-1 acme-wor…-1 acme-redis-1 · on devbox · destructive"
            ),
            "{}",
            lines[20]
        );
        let selected = plate.cell(4, 7);
        assert_eq!(selected.symbol(), "C");
        assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
        assert!(selected.modifier.contains(termrock::Modifier::BOLD));
        let kind = plate.cell(32, 7);
        assert_eq!(kind.symbol(), "p");
        assert!(!kind.modifier.contains(termrock::Modifier::BOLD));
        assert_eq!(kind.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
        let margin = plate.cell(79, 7);
        assert_eq!(margin.symbol(), " ");
        assert_eq!(margin.bg, Color::Rgb(0, 0, 0));
        let esc = plate.cell(63, 23);
        assert_eq!(esc.symbol(), "E");
        assert!(esc.modifier.contains(termrock::Modifier::BOLD));
        let ellipsis = plate.cell(74, 23);
        assert_eq!(ellipsis.symbol(), "…");
        assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
        assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
        let wide = harness(
            App::for_scenario_without_history(Scenario::DockerCleanup, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        assert!(
            wide.text()
                .lines()
                .nth(15)
                .unwrap()
                .contains("docker stop acme…redis-1"),
            "{}",
            wide.text()
        );
        let later = harness(
            App::for_scenario_without_history(Scenario::DockerCleanup, Motion::Paused, 4_000),
            Theme::junie(),
            80,
            24,
        );
        assert!(!later.text().contains("Results · 7"));
        assert!(later.text().contains("Suggested here"), "{}", later.text());
        assert!(!crate::sim::catalogue::docker_cleanup_tag_frame(
            &later.app().world
        ));
    }
    #[test]
    fn docker_cleanup_100x30_frame_40_paints_the_mid_plate_and_later_seeks_keep_home() {
        let plate = harness(
            App::for_scenario_without_history(Scenario::DockerCleanup, Motion::Paused, 40),
            Theme::junie(),
            100,
            30,
        );
        let text = plate.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines[6].contains("Results · 7"), "{}", lines[6]);
        assert!(
            lines[7].contains("› Clean Docker completely"),
            "{}",
            lines[7]
        );
        assert!(
            lines[7].contains("18.4 GB reclaimable on devbox"),
            "{}",
            lines[7]
        );
        assert!(
            lines[13].contains("Stop and remove all containers"),
            "{}",
            lines[13]
        );
        assert!(
            lines[26].contains(
                "→ docker stop acme-api-1 acme-worker-1 acme-sc…acme-db-1 acme-redis-1 · on devbox · destructive"
            ),
            "{}",
            lines[26]
        );
        assert!(!lines[26].contains("Clean Docker"), "{}", lines[26]);
        let ellipsis = plate.cell(87, 29);
        assert_eq!(ellipsis.symbol(), "…");
        assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
        assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
        assert!(catalogue::docker_cleanup_view(&plate.app().world, 120).is_none());
        let wide = harness(
            App::for_scenario_without_history(Scenario::DockerCleanup, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        assert!(
            wide.text()
                .lines()
                .nth(15)
                .unwrap()
                .contains("docker stop acme…redis-1"),
            "{}",
            wide.text()
        );
        assert!(!wide.text().contains("acme-worker-1 acme-sc…acme-db-1"));
        let narrow = harness(
            App::for_scenario_without_history(Scenario::DockerCleanup, Motion::Paused, 40),
            Theme::junie(),
            80,
            24,
        );
        assert!(!narrow.text().contains("acme-worker-1 acme-sc…acme-db-1"));
        let later = harness(
            App::for_scenario_without_history(Scenario::DockerCleanup, Motion::Paused, 4_000),
            Theme::junie(),
            100,
            30,
        );
        assert!(!later.text().contains("Results · 7"));
        assert!(later.text().contains("Suggested here"), "{}", later.text());
        assert!(catalogue::docker_cleanup_view(&later.app().world, 100).is_none());
    }
    #[test]
    fn disk_cleanup_frame_40_paints_the_tag_plate_and_later_seeks_keep_home() {
        let plate = harness(
            App::for_scenario_without_history(Scenario::DiskCleanup, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        let text = plate.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(plate.app().route == Route::Disk);
        assert!(
            lines[0].contains("work · project collection"),
            "{}",
            lines[0]
        );
        assert!(lines[0].contains("mbp · local"), "{}", lines[0]);
        assert!(lines[1].contains("Here › Disk › Usage"), "{}", lines[1]);
        assert!(lines[2].contains("Here › Disk › Usage"), "{}", lines[2]);
        assert!(lines[4].contains("Disk usage · ~/work"), "{}", lines[4]);
        assert!(lines[5].contains("scan complete"), "{}", lines[5]);
        assert!(lines[8].contains("842 of 926 GB"), "{}", lines[8]);
        assert!(lines[12].contains("work"), "{}", lines[12]);
        assert!(lines[13].contains("~/work"), "{}", lines[13]);
        assert!(lines[38].contains("3 selected · 24.2 GiB"), "{}", lines[38]);
        assert!(lines[39].contains("Esc Back"), "{}", lines[39]);
        assert!(lines[39].contains('…'), "{}", lines[39]);
        let file = plate.cell(12, 0);
        assert_eq!(file.symbol(), "F");
        assert!(!file.modifier.contains(termrock::Modifier::BOLD));
        let gutter = plate.cell(1, 12);
        assert_eq!(gutter.symbol(), "▎");
        assert_eq!(gutter.fg, Color::Rgb(0x48, 0xe0, 0x54));
        assert!(gutter.modifier.contains(termrock::Modifier::BOLD));
        let hidden = plate.cell(1, 13);
        assert_eq!(hidden.symbol(), " ");
        assert_eq!(hidden.fg, Color::Rgb(0, 0, 0));
        let card = plate.cell(75, 20);
        assert_eq!(card.bg, Color::Rgb(0x11, 0x11, 0x11));
        let tab = plate.cell(2, 2);
        assert_eq!(tab.bg, Color::Rgb(0x27, 0x27, 0x2a));
        let meter = plate.cell(81, 38);
        assert_eq!(meter.symbol(), "━");
        assert_eq!(meter.fg, Color::Rgb(0xf5, 0x9e, 0x09));
        let ellipsis = plate.cell(110, 39);
        assert_eq!(ellipsis.symbol(), "…");
        assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
        assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
        let later = harness(
            App::for_scenario_without_history(Scenario::DiskCleanup, Motion::Paused, 4_000),
            Theme::junie(),
            120,
            40,
        );
        assert!(!later.text().contains("scan complete"));
        assert!(later.text().contains("Disk usage"), "{}", later.text());
        assert!(
            later.text().contains("~/work/forge/target"),
            "{}",
            later.text()
        );
        assert!(later.app().route == Route::Disk);
        let mut stepped =
            App::for_scenario_without_history(Scenario::DiskCleanup, Motion::Paused, 40);
        assert!(termrock::App::accept_paused_clock_step(&mut stepped, 1_100));
        assert!(!crate::sim::catalogue::disk_cleanup_tag_frame(
            &stepped.world
        ));
    }
    #[test]
    fn disk_cleanup_80x24_frame_40_paints_the_narrow_plate_and_later_seeks_keep_home() {
        let plate = harness(
            App::for_scenario_without_history(Scenario::DiskCleanup, Motion::Paused, 40),
            Theme::junie(),
            80,
            24,
        );
        let text = plate.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines[5].contains("scan complete"), "{}", lines[5]);
        assert!(lines[11].contains("largest first"), "{}", lines[11]);
        assert!(!lines[11].contains("directory"), "{}", lines[11]);
        assert!(lines[12].contains("31.8 GiB"), "{}", lines[12]);
        assert!(!lines[12].contains("Path"), "{}", lines[12]);
        assert!(lines[22].contains("3 selected · 24.2 GiB"), "{}", lines[22]);
        assert!(lines[23].contains("Esc Back"), "{}", lines[23]);
        assert!(lines[23].contains('…'), "{}", lines[23]);
        assert!(!lines[23].contains("Delete"), "{}", lines[23]);
        let gap = plate.cell(3, 12);
        assert_eq!(gap.symbol(), " ");
        assert!(gap.modifier.contains(termrock::Modifier::BOLD));
        let tail = plate.cell(78, 12);
        assert_eq!(tail.symbol(), " ");
        assert!(tail.modifier.contains(termrock::Modifier::BOLD));
        let digit = plate.cell(63, 12);
        assert_eq!(digit.symbol(), "3");
        assert_eq!(digit.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        assert!(digit.modifier.contains(termrock::Modifier::BOLD));
        let ellipsis = plate.cell(72, 23);
        assert_eq!(ellipsis.symbol(), "…");
        assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
        assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
        for (width, height) in [(120_u16, 40_u16), (80, 40), (81, 24), (100, 30)] {
            let other = harness(
                App::for_scenario_without_history(Scenario::DiskCleanup, Motion::Paused, 40),
                Theme::junie(),
                width,
                height,
            );
            let text = other.text();
            if width == 120 && height == 40 {
                assert!(text.contains("directory"), "{text}");
                assert!(text.lines().nth(39).unwrap().contains("Delete"), "{text}");
            } else if width == 100 && height == 30 {
                assert!(text.contains("scan complete"), "{text}");
                assert!(!text.contains("directory"), "{text}");
            } else {
                assert!(
                    !text.contains("scan complete"),
                    "{width}x{height} used the narrow plate\n{text}"
                );
            }
        }
        let later = harness(
            App::for_scenario_without_history(Scenario::DiskCleanup, Motion::Paused, 4_000),
            Theme::junie(),
            80,
            24,
        );
        assert!(!later.text().contains("scan complete"));
        assert!(!crate::sim::catalogue::disk_cleanup_tag_frame(
            &later.app().world
        ));
    }
    #[test]
    fn disk_cleanup_100x30_frame_40_paints_the_mid_plate_and_later_seeks_keep_home() {
        let plate = harness(
            App::for_scenario_without_history(Scenario::DiskCleanup, Motion::Paused, 40),
            Theme::junie(),
            100,
            30,
        );
        let text = plate.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines[5].contains("scan complete"), "{}", lines[5]);
        assert!(lines[12].contains("31.8 GiB · 100%"), "{}", lines[12]);
        assert!(!text.contains("directory"), "{text}");
        assert!(lines[28].contains("3 selected · 24.2 GiB"), "{}", lines[28]);
        assert!(lines[29].contains("d Delete…"), "{}", lines[29]);
        assert!(lines[29].contains("Esc Back"), "{}", lines[29]);
        assert!(!lines[29].contains("Artifact"), "{}", lines[29]);
        assert!(!lines[29].contains("Cancel"), "{}", lines[29]);
        let ellipsis = plate.cell(83, 29);
        assert_eq!(ellipsis.symbol(), "…");
        assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
        let size = plate.cell(83, 12);
        assert_eq!(size.symbol(), "3");
        assert!(size.modifier.contains(termrock::Modifier::BOLD));
        let tail = plate.cell(98, 12);
        assert_eq!(tail.symbol(), " ");
        assert!(tail.modifier.contains(termrock::Modifier::BOLD));
        let edge = plate.cell(99, 12);
        assert_eq!(edge.symbol(), " ");
        assert!(!edge.modifier.contains(termrock::Modifier::BOLD));
        assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
        let world = &plate.app().world;
        assert!(catalogue::disk_cleanup_tag_frame(world));
        let wide = harness(
            App::for_scenario_without_history(Scenario::DiskCleanup, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        assert!(wide.text().contains("directory"), "{}", wide.text());
        assert_eq!(wide.cell(110, 39).symbol(), "…");
        let narrow = harness(
            App::for_scenario_without_history(Scenario::DiskCleanup, Motion::Paused, 40),
            Theme::junie(),
            80,
            24,
        );
        assert!(!narrow.text().contains("Delete"), "{}", narrow.text());
        assert_eq!(narrow.cell(72, 23).symbol(), "…");
        let later = harness(
            App::for_scenario_without_history(Scenario::DiskCleanup, Motion::Paused, 4_000),
            Theme::junie(),
            100,
            30,
        );
        assert!(!later.text().contains("scan complete"));
        assert!(!catalogue::disk_cleanup_tag_frame(&later.app().world));
        let other = harness(
            App::for_scenario_without_history(Scenario::DockerCleanup, Motion::Paused, 40),
            Theme::junie(),
            100,
            30,
        );
        assert!(!catalogue::disk_cleanup_tag_frame(&other.app().world));
        assert!(other.text().contains("Results · 7"), "{}", other.text());
    }
    #[test]
    fn remote_host_frame_40_paints_the_tag_plate_and_later_seeks_keep_home() {
        let plate = harness(
            App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        let text = plate.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(
            lines[0].contains("payments · service checkout"),
            "{}",
            lines[0]
        );
        assert!(
            lines[0].contains("◆ prod-eu-1 · production"),
            "{}",
            lines[0]
        );
        assert!(!lines[0].contains("ssh"), "{}", lines[0]);
        assert!(
            lines[8].contains("› Follow payments-worker j…"),
            "{}",
            lines[8]
        );
        assert!(lines[8].contains("unit is restarting"), "{}", lines[8]);
        assert!(lines[8].contains("journalctl -u"), "{}", lines[8]);
        assert!(
            lines[10].contains("journalctl -u pa… -n 200"),
            "{}",
            lines[10]
        );
        assert!(lines[14].contains("nothing used here yet"), "{}", lines[14]);
        assert!(lines[18].contains("System · prod-eu-1"), "{}", lines[18]);
        assert!(lines[38].contains("/srv/payments"), "{}", lines[38]);
        assert!(lines[38].contains("detached @ v2.14.3"), "{}", lines[38]);
        assert!(lines[39].contains("Enter Run"), "{}", lines[39]);
        assert!(lines[39].contains("Follow pay…"), "{}", lines[39]);
        assert!(lines[39].contains("Ctrl+G"), "{}", lines[39]);
        let selected = plate.cell(4, 8);
        assert_eq!(selected.symbol(), "F");
        assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
        assert!(selected.modifier.contains(termrock::Modifier::BOLD));
        let kind = plate.cell(31, 8);
        assert_eq!(kind.symbol(), "s");
        assert!(!kind.modifier.contains(termrock::Modifier::BOLD));
        let risk = plate.cell(92, 17);
        assert_eq!(risk.symbol(), "r");
        assert_eq!(risk.fg, Color::Rgb(0x73, 0x73, 0x73));
        assert_eq!(risk.bg, Color::Rgb(0x11, 0x11, 0x11));
        assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
        let later = harness(
            App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 4_000),
            Theme::junie(),
            120,
            40,
        );
        assert!(!later.text().contains("Follow payments-worker"));
        assert!(!later.text().contains("detached @ v2.14.3"));
        assert!(later.app().route == Route::Home);
        let mut stepped =
            App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 40);
        assert!(termrock::App::accept_paused_clock_step(&mut stepped, 1_100));
        assert!(!crate::sim::catalogue::remote_host_tag_frame(
            &stepped.world
        ));
    }
    #[test]
    fn remote_host_80x24_frame_40_paints_the_narrow_plate_and_later_seeks_keep_home() {
        let plate = harness(
            App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 40),
            Theme::junie(),
            80,
            24,
        );
        let text = plate.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(
            lines[8].contains("› Follow payments-worker jo…"),
            "{}",
            lines[8]
        );
        assert!(lines[8].contains("unit is restarting"), "{}", lines[8]);
        assert!(
            lines[20]
                .contains("→ journalctl -u payments-worker -f -n 200 · on prod-eu-1 · read-only"),
            "{}",
            lines[20]
        );
        assert!(lines[23].contains("Enter Run"), "{}", lines[23]);
        assert!(lines[23].contains('…'), "{}", lines[23]);
        assert!(!lines[23].contains("Follow pay…"), "{}", lines[23]);
        assert!(!text.contains("journalctl -u pa…"), "{text}");
        let selected = plate.cell(4, 8);
        assert_eq!(selected.symbol(), "F");
        assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
        assert!(selected.modifier.contains(termrock::Modifier::BOLD));
        let kind = plate.cell(32, 8);
        assert_eq!(kind.symbol(), "s");
        assert!(!kind.modifier.contains(termrock::Modifier::BOLD));
        assert_eq!(kind.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
        let edge = plate.cell(78, 8);
        assert_eq!(edge.symbol(), " ");
        assert_eq!(edge.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
        let ellipsis = plate.cell(72, 23);
        assert_eq!(ellipsis.symbol(), "…");
        assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
        assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
        let world = &plate.app().world;
        assert!(catalogue::remote_host_view(world, 120).is_none());
        let wide = harness(
            App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        assert!(
            !wide
                .text()
                .contains("journalctl -u payments-worker -f -n 200"),
            "120x40 used the narrow plate\n{}",
            wide.text()
        );
        // The 80-column band is height-free: 80x40 keeps the narrow list.
        let tall = harness(
            App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 40),
            Theme::junie(),
            80,
            40,
        );
        assert!(
            tall.text()
                .contains("journalctl -u payments-worker -f -n 200"),
            "80x40 lost the narrow band\n{}",
            tall.text()
        );
        let later = harness(
            App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 4_000),
            Theme::junie(),
            80,
            24,
        );
        assert!(!later.text().contains("Follow payments-worker"));
        assert!(!later.text().contains("journalctl -u payments-worker"));
        assert!(catalogue::remote_host_view(&later.app().world, 80).is_none());
        assert!(!catalogue::remote_host_tag_frame(&later.app().world));
    }
    #[test]
    fn remote_host_100x30_frame_40_paints_the_mid_plate_and_later_seeks_keep_home() {
        let plate = harness(
            App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 40),
            Theme::junie(),
            100,
            30,
        );
        let text = plate.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(
            lines[7].contains("Who is blocking the database?"),
            "{}",
            lines[7]
        );
        assert!(
            lines[7].contains("2 sessions blocked for 4 min"),
            "{}",
            lines[7]
        );
        assert!(!lines[7].contains("41 min"), "{}", lines[7]);
        assert!(
            lines[8].contains("› Follow payments-worker journal"),
            "{}",
            lines[8]
        );
        assert!(lines[8].contains("unit is restarting"), "{}", lines[8]);
        assert!(lines[9].contains("sustained pressure…"), "{}", lines[9]);
        assert!(lines[18].contains("System · prod-eu-1"), "{}", lines[18]);
        assert!(
            lines[26]
                .contains("→ journalctl -u payments-worker -f -n 200 · on prod-eu-1 · read-only"),
            "{}",
            lines[26]
        );
        assert!(!text.contains("journalctl -u pa…"), "{text}");
        assert!(lines[28].contains("/srv/payments"), "{}", lines[28]);
        assert!(lines[28].contains("detached @ v2.14.3"), "{}", lines[28]);
        assert!(lines[29].contains("Enter Run"), "{}", lines[29]);
        assert!(lines[29].contains("Ctrl+↑↓ Scope"), "{}", lines[29]);
        assert!(lines[29].contains('…'), "{}", lines[29]);
        assert!(!lines[29].contains("Follow pay…"), "{}", lines[29]);
        assert!(!lines[29].contains("Ctrl+G"), "{}", lines[29]);
        let selected = plate.cell(4, 8);
        assert_eq!(selected.symbol(), "F");
        assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
        assert!(selected.modifier.contains(termrock::Modifier::BOLD));
        let kind = plate.cell(36, 8);
        assert_eq!(kind.symbol(), "s");
        assert!(!kind.modifier.contains(termrock::Modifier::BOLD));
        assert_eq!(kind.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
        let edge = plate.cell(98, 8);
        assert_eq!(edge.symbol(), " ");
        assert_eq!(edge.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
        let age = plate.cell(83, 7);
        assert_eq!(age.symbol(), "4");
        assert_eq!(age.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        let ellipsis = plate.cell(87, 29);
        assert_eq!(ellipsis.symbol(), "…");
        assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
        assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
        let world = &plate.app().world;
        assert!(catalogue::remote_host_view(world, 100).is_some());
        assert!(catalogue::remote_host_view(world, 120).is_none());
        let wide = harness(
            App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        assert!(wide.text().contains("Follow pay…"), "{}", wide.text());
        assert!(wide.text().contains("Ctrl+G"), "{}", wide.text());
        assert!(!wide.text().contains("for 4 min"), "{}", wide.text());
        assert!(
            wide.text().contains("journalctl -u pa… -n 200"),
            "{}",
            wide.text()
        );
        let narrow = harness(
            App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 40),
            Theme::junie(),
            80,
            24,
        );
        assert!(
            narrow.text().contains("Follow payments-worker jo…"),
            "{}",
            narrow.text()
        );
        assert!(!narrow.text().contains("for 4 min"), "{}", narrow.text());
        assert!(!narrow.text().contains("Follow pay…"), "{}", narrow.text());
        let later = harness(
            App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 4_000),
            Theme::junie(),
            100,
            30,
        );
        assert!(!later.text().contains("Follow payments-worker"));
        assert!(!later.text().contains("journalctl -u payments-worker"));
        assert!(catalogue::remote_host_view(&later.app().world, 100).is_none());
        assert!(!catalogue::remote_host_tag_frame(&later.app().world));
        let other =
            App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 40);
        assert!(catalogue::remote_host_view(&other.world, 100).is_none());
    }
    #[test]
    fn launch_failure_frame_40_paints_the_tag_plate_and_later_seeks_keep_the_activity() {
        let plate = harness(
            App::for_scenario_without_history(Scenario::LaunchFailure, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        let text = plate.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(plate.app().route == Route::Activity);
        assert!(
            lines[0].contains("acme › frontend · Node project · pnpm"),
            "{}",
            lines[0]
        );
        assert!(lines[0].contains("mbp · local"), "{}", lines[0]);
        assert!(lines[1].trim().is_empty(), "{}", lines[1]);
        assert!(lines[2].contains("frontend dev"), "{}", lines[2]);
        assert!(lines[4].contains("failed · 1 s · exit 1"), "{}", lines[4]);
        assert!(
            lines[8].contains("$ mise run dev  (apps/frontend)"),
            "{}",
            lines[8]
        );
        assert!(lines[13].contains("EADDRINUSE"), "{}", lines[13]);
        assert!(
            lines[36].contains("Find the process on port 5173"),
            "{}",
            lines[36]
        );
        assert!(
            lines[38].contains("~/work/acme/apps/frontend"),
            "{}",
            lines[38]
        );
        assert!(lines[38].contains("! exit 1 · 1 s"), "{}", lines[38]);
        assert!(lines[39].contains("Esc Back to Here"), "{}", lines[39]);
        assert!(
            lines[39].contains("frontend dev failed · exit 1"),
            "{}",
            lines[39]
        );
        let file = plate.cell(12, 0);
        assert_eq!(file.symbol(), "F");
        assert!(!file.modifier.contains(termrock::Modifier::BOLD));
        let tab = plate.cell(23, 2);
        assert_eq!(tab.symbol(), "!");
        assert_eq!(tab.fg, Color::Rgb(0xf0, 0x5d, 0x5d));
        assert_eq!(tab.bg, Color::Rgb(0x27, 0x27, 0x2a));
        let gutter = plate.cell(8, 36);
        assert_eq!(gutter.symbol(), " ");
        assert_eq!(gutter.fg, Color::Rgb(0, 0, 0));
        let bang = plate.cell(89, 39);
        assert_eq!(bang.symbol(), "!");
        assert!(bang.modifier.contains(termrock::Modifier::BOLD));
        assert_eq!(bang.fg, Color::Rgb(0xf0, 0x5d, 0x5d));
        assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
        let later = harness(
            App::for_scenario_without_history(Scenario::LaunchFailure, Motion::Paused, 4_000),
            Theme::junie(),
            120,
            40,
        );
        assert!(!later.text().contains("EADDRINUSE"));
        assert!(later.text().contains("seed db"), "{}", later.text());
        assert!(later.app().route == Route::Activity);
        assert!(!crate::sim::catalogue::launch_failure_tag_frame(
            &later.app().world
        ));
        let mut stepped =
            App::for_scenario_without_history(Scenario::LaunchFailure, Motion::Paused, 40);
        assert!(termrock::App::accept_paused_clock_step(&mut stepped, 1_100));
        assert!(!crate::sim::catalogue::launch_failure_tag_frame(
            &stepped.world
        ));
        for scenario in [
            Scenario::RemoteHost,
            Scenario::ActivitiesMulti,
            Scenario::MonorepoRoot,
            Scenario::FirstUse,
            Scenario::DockerCleanup,
            Scenario::DiskCleanup,
        ] {
            let other = harness(
                App::for_scenario_without_history(scenario, Motion::Paused, 40),
                Theme::junie(),
                120,
                40,
            );
            assert!(!other.text().contains("EADDRINUSE"), "{}", scenario.name());
            assert!(!crate::sim::catalogue::launch_failure_tag_frame(
                &other.app().world
            ));
        }
    }
    #[test]
    fn launch_failure_80x24_frame_40_paints_the_narrow_plate_and_later_seeks_keep_the_activity() {
        let plate = harness(
            App::for_scenario_without_history(Scenario::LaunchFailure, Motion::Paused, 40),
            Theme::junie(),
            80,
            24,
        );
        let text = plate.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(
            lines[0].contains("acme › frontend · Nod…ect · pnpm"),
            "{}",
            lines[0]
        );
        assert!(lines[0].contains("mbp · local"), "{}", lines[0]);
        assert!(lines[1].trim().is_empty(), "{}", lines[1]);
        assert!(lines[2].contains("frontend dev"), "{}", lines[2]);
        assert!(lines[4].contains("failed · 1 s · exit 1"), "{}", lines[4]);
        assert!(lines[8].contains("> vite --host"), "{}", lines[8]);
        assert!(!lines[8].contains("mise run dev"), "{}", lines[8]);
        assert!(lines[11].contains("EADDRINUSE"), "{}", lines[11]);
        assert!(
            lines[19].contains("Find the process on port 5173"),
            "{}",
            lines[19]
        );
        assert!(lines[19].contains("Retry frontend dev"), "{}", lines[19]);
        assert!(
            lines[20].contains("Open the mise task definition"),
            "{}",
            lines[20]
        );
        assert!(
            lines[22].contains("~/work/acme/apps/frontend"),
            "{}",
            lines[22]
        );
        assert!(lines[22].contains("! exit 1 · 1 s"), "{}", lines[22]);
        assert!(lines[22].contains('…'), "{}", lines[22]);
        assert!(!lines[22].contains("truecolor"), "{}", lines[22]);
        assert!(lines[23].contains("y Copy"), "{}", lines[23]);
        assert!(lines[23].contains('…'), "{}", lines[23]);
        assert!(!lines[23].contains("Restart"), "{}", lines[23]);
        assert!(
            lines[23].contains("frontend dev failed · exit 1"),
            "{}",
            lines[23]
        );
        assert!(!text.contains("seed db"), "{text}");
        let faded = plate.cell(4, 8);
        assert_eq!(faded.symbol(), ">");
        assert_eq!(faded.fg, Color::Rgb(0x8c, 0x8c, 0x8c));
        let track = plate.cell(76, 8);
        assert_eq!(track.symbol(), "┆");
        assert_eq!(track.fg, Color::Rgb(0x47, 0x47, 0x47));
        let thumb = plate.cell(76, 10);
        assert_eq!(thumb.symbol(), "┃");
        assert_eq!(thumb.fg, Color::Rgb(0x80, 0x80, 0x80));
        let status_mark = plate.cell(78, 22);
        assert_eq!(status_mark.symbol(), "…");
        assert_eq!(status_mark.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        assert_eq!(status_mark.bg, Color::Rgb(0x18, 0x18, 0x1b));
        let footer_mark = plate.cell(38, 23);
        assert_eq!(footer_mark.symbol(), "…");
        assert_eq!(footer_mark.fg, Color::Rgb(0x73, 0x73, 0x73));
        assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
        let world = &plate.app().world;
        // The width band carries the failure card at every viewport, not only at
        // the captured frame sizes.
        assert!(catalogue::launch_failure_activity_frame(world, 160));
        assert!(catalogue::launch_failure_activity_frame(world, 80));
        assert!(catalogue::launch_failure_activity_frame(world, 81));
        assert!(!catalogue::launch_failure_activity_frame(world, 60));
        let wide = harness(
            App::for_scenario_without_history(Scenario::LaunchFailure, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        assert!(wide.text().contains("$ mise run dev  (apps/frontend)"));
        assert!(wide.text().contains("Node project · pnpm"));
        assert!(!wide.text().contains("Nod…ect"));
        assert!(catalogue::launch_failure_activity_frame(
            &wide.app().world,
            120
        ));
        for (width, height) in [(80_u16, 40_u16), (160, 50)] {
            let other = harness(
                App::for_scenario_without_history(Scenario::LaunchFailure, Motion::Paused, 40),
                Theme::junie(),
                width,
                height,
            );
            assert!(
                other.text().contains("seed db"),
                "{width}x{height}\n{}",
                other.text()
            );
            assert!(!other.text().contains("EADDRINUSE"), "{width}x{height}");
            assert!(catalogue::launch_failure_activity_frame(
                &other.app().world,
                width
            ));
        }
        let later = harness(
            App::for_scenario_without_history(Scenario::LaunchFailure, Motion::Paused, 4_000),
            Theme::junie(),
            80,
            24,
        );
        assert!(later.text().contains("seed db"), "{}", later.text());
        assert!(!later.text().contains("EADDRINUSE"));
        assert!(!catalogue::launch_failure_activity_frame(
            &later.app().world,
            80
        ));
        assert!(!catalogue::launch_failure_tag_frame(&later.app().world));
    }
    #[test]
    fn launch_failure_100x30_frame_40_paints_the_mid_plate_and_later_seeks_keep_the_activity() {
        let plate = harness(
            App::for_scenario_without_history(Scenario::LaunchFailure, Motion::Paused, 40),
            Theme::junie(),
            100,
            30,
        );
        let text = plate.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(
            lines[0].contains("acme › frontend · Node project · pnpm"),
            "{}",
            lines[0]
        );
        assert!(lines[0].contains("mbp · local"), "{}", lines[0]);
        assert!(lines[1].trim().is_empty(), "{}", lines[1]);
        assert!(lines[2].contains("frontend dev"), "{}", lines[2]);
        assert!(lines[4].contains("failed · 1 s · exit 1"), "{}", lines[4]);
        assert!(
            lines[4].contains("here · ~/work/acme/apps/frontend · mbp"),
            "{}",
            lines[4]
        );
        assert!(
            lines[8].contains("$ mise run dev  (apps/frontend)"),
            "{}",
            lines[8]
        );
        assert!(lines[13].contains("EADDRINUSE"), "{}", lines[13]);
        assert!(lines[24].contains('╰'), "{}", lines[24]);
        assert!(
            lines[26].contains("Find the process on port 5173"),
            "{}",
            lines[26]
        );
        assert!(lines[26].contains("Retry frontend dev"), "{}", lines[26]);
        assert!(
            lines[26].contains("Open the mise task definition"),
            "{}",
            lines[26]
        );
        assert!(
            lines[28].contains("~/work/acme/apps/frontend"),
            "{}",
            lines[28]
        );
        assert!(lines[28].contains("! exit 1 · 1 s"), "{}", lines[28]);
        assert!(lines[28].contains("truecolor · 100×30"), "{}", lines[28]);
        assert!(lines[29].contains("r Restart"), "{}", lines[29]);
        assert!(lines[29].contains("x Close"), "{}", lines[29]);
        assert!(lines[29].contains('…'), "{}", lines[29]);
        assert!(!lines[29].contains("Back to Here"), "{}", lines[29]);
        assert!(
            lines[29].contains("frontend dev failed · exit 1"),
            "{}",
            lines[29]
        );
        assert!(!text.contains("seed db"), "{text}");
        assert!(!text.contains('┆'), "{text}");
        let here = plate.cell(60, 4);
        assert_eq!(here.symbol(), "h");
        assert_eq!(here.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        let bang = plate.cell(4, 12);
        assert_eq!(bang.symbol(), "!");
        assert!(bang.modifier.contains(termrock::Modifier::BOLD));
        assert_eq!(bang.fg, Color::Rgb(0xf0, 0x5d, 0x5d));
        let edge = plate.cell(98, 8);
        assert_eq!(edge.symbol(), "│");
        assert_eq!(edge.fg, Color::Rgb(0x4d, 0x4d, 0x4d));
        let footer_mark = plate.cell(58, 29);
        assert_eq!(footer_mark.symbol(), "…");
        assert_eq!(footer_mark.fg, Color::Rgb(0x73, 0x73, 0x73));
        let size = plate.cell(81, 28);
        assert_eq!(size.symbol(), "t");
        assert_eq!(size.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        assert_eq!(size.bg, Color::Rgb(0x18, 0x18, 0x1b));
        assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
        let world = &plate.app().world;
        assert!(catalogue::launch_failure_activity_frame(world, 100));
        assert!(catalogue::launch_failure_activity_frame(world, 100));
        assert!(catalogue::launch_failure_activity_frame(world, 99));
        let wide = harness(
            App::for_scenario_without_history(Scenario::LaunchFailure, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        assert!(wide.text().contains("Esc Back to Here"));
        assert!(catalogue::launch_failure_activity_frame(
            &wide.app().world,
            120
        ));
        let narrow = harness(
            App::for_scenario_without_history(Scenario::LaunchFailure, Motion::Paused, 40),
            Theme::junie(),
            80,
            24,
        );
        assert!(narrow.text().contains("> vite --host"));
        assert!(!narrow.text().contains("$ mise run dev"));
        assert!(catalogue::launch_failure_activity_frame(
            &narrow.app().world,
            80
        ));
        let later = harness(
            App::for_scenario_without_history(Scenario::LaunchFailure, Motion::Paused, 4_000),
            Theme::junie(),
            100,
            30,
        );
        assert!(later.text().contains("seed db"), "{}", later.text());
        assert!(!later.text().contains("EADDRINUSE"));
        assert!(!catalogue::launch_failure_activity_frame(
            &later.app().world,
            100
        ));
        assert!(!catalogue::launch_failure_tag_frame(&later.app().world));
        let other = App::for_scenario_without_history(Scenario::HardCases, Motion::Paused, 40);
        assert!(!catalogue::launch_failure_activity_frame(&other.world, 100));
    }
    #[test]
    fn hard_cases_frame_40_paints_the_tag_plate_and_later_seeks_keep_home() {
        let plate = harness(
            App::for_scenario_without_history(Scenario::HardCases, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        let text = plate.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(plate.app().route == Route::Home);
        assert!(
            lines[0].contains("northwind-traders-platform-migration › inventory"),
            "{}",
            lines[0]
        );
        assert!(lines[0].contains("mbp · local"), "{}", lines[0]);
        assert!(lines[0].contains("ory-reconciliation-core"), "{}", lines[0]);
        assert!(lines[7].contains("Analyze disk usage"), "{}", lines[7]);
        assert!(lines[7].contains("/ is 91% full"), "{}", lines[7]);
        assert!(
            lines[8].contains("Resolve the rebase in la…"),
            "{}",
            lines[8]
        );
        assert!(lines[14].contains("nothing used here yet"), "{}", lines[14]);
        assert!(lines[14].contains("'holla disk analyze'"), "{}", lines[14]);
        assert!(
            lines[18].contains("Children · 14 projects"),
            "{}",
            lines[18]
        );
        assert!(
            lines[38].contains("~/work/clients/northwind-trad…iation-service"),
            "{}",
            lines[38]
        );
        assert!(lines[38].contains("↓14 ↑3"), "{}", lines[38]);
        assert!(lines[38].contains("2 conflicts"), "{}", lines[38]);
        assert!(
            lines[38].contains("! docker, github unavailable"),
            "{}",
            lines[38]
        );
        assert!(lines[39].contains("Enter Open"), "{}", lines[39]);
        assert!(!lines[39].contains("Follow"), "{}", lines[39]);
        let selected = plate.cell(4, 7);
        assert_eq!(selected.symbol(), "A");
        assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
        assert!(selected.modifier.contains(termrock::Modifier::BOLD));
        let risk = plate.cell(92, 20);
        assert_eq!(risk.symbol(), "r");
        assert_eq!(risk.fg, Color::Rgb(0x73, 0x73, 0x73));
        assert_eq!(risk.bg, Color::Rgb(0x11, 0x11, 0x11));
        let overflow = plate.cell(118, 38);
        assert_eq!(overflow.symbol(), "…");
        assert_eq!(overflow.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
        let later = harness(
            App::for_scenario_without_history(Scenario::HardCases, Motion::Paused, 4_000),
            Theme::junie(),
            120,
            40,
        );
        assert!(!later.text().contains("northwind"));
        assert!(!later.text().contains("! docker, github unavailable"));
        assert!(
            later.text().contains("docker: discovery failed"),
            "{}",
            later.text()
        );
        assert!(later.app().route == Route::Home);
        assert!(!crate::sim::catalogue::hard_cases_tag_frame(
            &later.app().world
        ));
        let mut stepped =
            App::for_scenario_without_history(Scenario::HardCases, Motion::Paused, 40);
        assert!(termrock::App::accept_paused_clock_step(&mut stepped, 1_100));
        assert!(!crate::sim::catalogue::hard_cases_tag_frame(&stepped.world));
        for scenario in [
            Scenario::LaunchFailure,
            Scenario::RemoteHost,
            Scenario::ActivitiesMulti,
            Scenario::MonorepoRoot,
            Scenario::FirstUse,
            Scenario::DockerCleanup,
            Scenario::DiskCleanup,
        ] {
            let other = harness(
                App::for_scenario_without_history(scenario, Motion::Paused, 40),
                Theme::junie(),
                120,
                40,
            );
            assert!(!other.text().contains("northwind"), "{}", scenario.name());
            assert!(!crate::sim::catalogue::hard_cases_tag_frame(
                &other.app().world
            ));
        }
    }
    #[test]
    fn hard_cases_80x24_frame_40_paints_the_narrow_tag_list_and_later_seeks_keep_home() {
        let plate = harness(
            App::for_scenario_without_history(Scenario::HardCases, Motion::Paused, 40),
            Theme::junie(),
            80,
            24,
        );
        let text = plate.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines[7].contains("› Analyze disk usage"), "{}", lines[7]);
        assert!(
            lines[8].contains("Resolve the rebase in laz…"),
            "{}",
            lines[8]
        );
        assert!(lines[8].contains("rebase in progress…"), "{}", lines[8]);
        assert!(
            lines[18].contains("Children · 14 projects"),
            "{}",
            lines[18]
        );
        assert!(
            lines[20].contains("→ 'holla disk analyze' · here · read-only"),
            "{}",
            lines[20]
        );
        assert!(
            lines[22].contains("~/work/clients/northwind-trad…iation-service"),
            "{}",
            lines[22]
        );
        assert!(!lines[22].contains("↓14"), "{}", lines[22]);
        assert!(!lines[22].contains("docker"), "{}", lines[22]);
        assert!(lines[23].contains("Enter Open"), "{}", lines[23]);
        assert!(lines[23].contains("Tab"), "{}", lines[23]);
        let thumb = plate.cell(78, 6);
        assert_eq!(thumb.symbol(), "┃");
        assert_eq!(thumb.fg, Color::Rgb(0x80, 0x80, 0x80));
        let track = plate.cell(78, 17);
        assert_eq!(track.symbol(), "┆");
        assert_eq!(track.fg, Color::Rgb(0x47, 0x47, 0x47));
        let mut footer_overflow = None;
        for x in 0..80 {
            let cell = plate.cell(x, 23);
            if cell.symbol() == "…" {
                footer_overflow = Some((x, cell.fg));
            }
        }
        assert_eq!(
            footer_overflow,
            Some((73, Color::Rgb(0x73, 0x73, 0x73))),
            "{}",
            lines[23]
        );
        let world = &plate.app().world;
        assert!(catalogue::hard_cases_tag_frame(world));
        let wide = harness(
            App::for_scenario_without_history(Scenario::HardCases, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        assert!(
            wide.text().contains("! docker, github unavailable"),
            "{}",
            wide.text()
        );
        assert!(wide.text().contains("↓14 ↑3"), "{}", wide.text());
        let later = harness(
            App::for_scenario_without_history(Scenario::HardCases, Motion::Paused, 4_000),
            Theme::junie(),
            80,
            24,
        );
        assert!(
            !later.text().contains("holla disk analyze"),
            "{}",
            later.text()
        );
        assert!(!catalogue::hard_cases_tag_frame(&later.app().world));
    }
    #[test]
    fn hard_cases_100x30_frame_40_paints_the_mid_tag_list_and_later_seeks_keep_home() {
        let plate = harness(
            App::for_scenario_without_history(Scenario::HardCases, Motion::Paused, 40),
            Theme::junie(),
            100,
            30,
        );
        let text = plate.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(
            lines[0].contains("northwind-traders-platform-migrati…conciliation-core"),
            "{}",
            lines[0]
        );
        assert!(lines[0].contains("mbp · local"), "{}", lines[0]);
        assert!(lines[7].contains("› Analyze disk usage"), "{}", lines[7]);
        assert!(lines[7].contains("/ is 91% full"), "{}", lines[7]);
        assert!(
            lines[8].contains("Resolve the rebase in lazygit"),
            "{}",
            lines[8]
        );
        assert!(
            lines[8].contains("rebase in progress · 2 conflicts"),
            "{}",
            lines[8]
        );
        assert!(
            lines[19].contains("Parent · northwind-traders-platf…"),
            "{}",
            lines[19]
        );
        assert!(lines[20].contains("System · mbp"), "{}", lines[20]);
        assert!(
            lines[26].contains("→ 'holla disk analyze' · here · read-only"),
            "{}",
            lines[26]
        );
        assert!(
            lines[28].contains("~/work/clients/northwind-trad…iation-service"),
            "{}",
            lines[28]
        );
        assert!(lines[28].contains("↓14 ↑3"), "{}", lines[28]);
        assert!(lines[28].contains("• 7"), "{}", lines[28]);
        assert!(
            lines[28].contains("! docker, github unavailable"),
            "{}",
            lines[28]
        );
        assert!(!lines[28].contains("2 conflicts"), "{}", lines[28]);
        assert!(!lines[28].contains("rebase"), "{}", lines[28]);
        assert!(lines[29].contains("Enter Open"), "{}", lines[29]);
        assert!(lines[29].contains("Ctrl+↑↓ Scope"), "{}", lines[29]);
        assert!(!lines[29].contains("Ctrl+G"), "{}", lines[29]);
        let selected = plate.cell(4, 7);
        assert_eq!(selected.symbol(), "A");
        assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
        assert!(selected.modifier.contains(termrock::Modifier::BOLD));
        let kind = plate.cell(39, 7);
        assert_eq!(kind.symbol(), "d");
        assert!(!kind.modifier.contains(termrock::Modifier::BOLD));
        assert_eq!(kind.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
        let warn = plate.cell(65, 28);
        assert_eq!(warn.symbol(), "!");
        assert_eq!(warn.fg, Color::Rgb(0xf0, 0x5d, 0x5d));
        let status_mark = plate.cell(98, 28);
        assert_eq!(status_mark.symbol(), "…");
        assert_eq!(status_mark.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        let footer_mark = plate.cell(88, 29);
        assert_eq!(footer_mark.symbol(), "…");
        assert_eq!(footer_mark.fg, Color::Rgb(0x73, 0x73, 0x73));
        assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
        let world = &plate.app().world;
        assert!(catalogue::hard_cases_tag_frame(world));
        let wide = harness(
            App::for_scenario_without_history(Scenario::HardCases, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        let wide_text = wide.text();
        let wide_lines: Vec<&str> = wide_text.lines().collect();
        assert!(wide_lines[38].contains("2 conflicts"), "{}", wide_lines[38]);
        assert!(wide_lines[38].contains("rebase"), "{}", wide_lines[38]);
        assert!(
            wide_lines[39].contains("Ctrl+G Activities"),
            "{}",
            wide_lines[39]
        );
        let narrow = harness(
            App::for_scenario_without_history(Scenario::HardCases, Motion::Paused, 40),
            Theme::junie(),
            80,
            24,
        );
        let narrow_text = narrow.text();
        let narrow_lines: Vec<&str> = narrow_text.lines().collect();
        assert!(!narrow_lines[22].contains("↓14"), "{}", narrow_lines[22]);
        assert!(catalogue::hard_cases_tag_frame(&narrow.app().world));
        let later = harness(
            App::for_scenario_without_history(Scenario::HardCases, Motion::Paused, 4_000),
            Theme::junie(),
            100,
            30,
        );
        assert!(
            !later.text().contains("holla disk analyze"),
            "{}",
            later.text()
        );
        assert!(!catalogue::hard_cases_tag_frame(&later.app().world));
        let other = App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 40);
        assert!(!catalogue::hard_cases_tag_frame(&other.world));
    }
    #[test]
    fn activities_multi_frame_40_paints_the_tag_plate_and_later_seeks_keep_home() {
        let plate = harness(
            App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        let text = plate.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines[0].contains("acme · mise monorepo"), "{}", lines[0]);
        assert!(lines[0].contains("mbp · local"), "{}", lines[0]);
        assert!(lines[1].trim().is_empty(), "{}", lines[1]);
        assert!(lines[2].contains("frontend dev"), "{}", lines[2]);
        assert!(lines[2].contains("api tests"), "{}", lines[2]);
        assert!(lines[2].contains("btm"), "{}", lines[2]);
        assert!(
            lines[8].contains("Inspect 10 GB of generat"),
            "{}",
            lines[8]
        );
        assert!(lines[11].contains("Current folder…"), "{}", lines[11]);
        assert!(lines[16].contains("Running"), "{}", lines[16]);
        assert!(
            lines[17].contains("logs · api worker schedu…"),
            "{}",
            lines[17]
        );
        assert!(lines[19].contains("⠋ detached · 3 s"), "{}", lines[19]);
        assert!(lines[22].contains("Explore"), "{}", lines[22]);
        assert!(lines[38].contains("~/work/acme"), "{}", lines[38]);
        assert!(lines[38].contains("↑1"), "{}", lines[38]);
        assert!(lines[38].contains("4 running"), "{}", lines[38]);
        assert!(lines[39].contains("Enter Open"), "{}", lines[39]);
        assert!(
            lines[39].contains("api tests finished · exit 0"),
            "{}",
            lines[39]
        );
        assert!(!lines[39].contains("Ctrl+G"), "{}", lines[39]);
        let spin = plate.cell(23, 2);
        assert_eq!(spin.symbol(), "⠋");
        assert_eq!(spin.fg, Color::Rgb(0x48, 0xe0, 0x54));
        let done = plate.cell(55, 2);
        assert_eq!(done.symbol(), "✓");
        assert_eq!(done.fg, Color::Rgb(0x3f, 0xb9, 0x50));
        let idle = plate.cell(98, 2);
        assert_eq!(idle.symbol(), "○");
        assert_eq!(idle.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        let badge = plate.cell(86, 38);
        assert_eq!(badge.symbol(), "⠋");
        assert_eq!(badge.fg, Color::Rgb(0x48, 0xe0, 0x54));
        assert_eq!(badge.bg, Color::Rgb(0x27, 0x27, 0x2a));
        let gap = plate.cell(87, 38);
        assert_eq!(gap.symbol(), " ");
        assert_eq!(gap.bg, Color::Rgb(0x18, 0x18, 0x1b));
        let count = plate.cell(88, 38);
        assert_eq!(count.symbol(), "4");
        assert_eq!(count.fg, Color::Rgb(0xb3, 0xb3, 0xb3));
        assert_eq!(count.bg, Color::Rgb(0x27, 0x27, 0x2a));
        let ellipsis = plate.cell(88, 39);
        assert_eq!(ellipsis.symbol(), "…");
        assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
        assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
        let later = harness(
            App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 4_000),
            Theme::junie(),
            120,
            40,
        );
        assert!(!later.text().contains("Follow acme-api-1 logs"));
        assert!(!later.text().contains("api tests finished"));
        assert!(later.text().contains("dev server"), "{}", later.text());
        assert!(later.app().route == Route::Home);
        let mut stepped =
            App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 40);
        assert!(termrock::App::accept_paused_clock_step(&mut stepped, 1_100));
        assert!(!crate::sim::catalogue::activities_multi_tag_frame(
            &stepped.world
        ));
    }
    #[test]
    fn activities_multi_80x24_frame_40_paints_the_narrow_plate_and_later_seeks_keep_home() {
        let plate = harness(
            App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 40),
            Theme::junie(),
            80,
            24,
        );
        let text = plate.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(
            lines[8].contains("› Inspect 10 GB of generate…"),
            "{}",
            lines[8]
        );
        assert!(
            lines[9].contains("Install missing tools · s…"),
            "{}",
            lines[9]
        );
        assert!(
            lines[17].contains("logs · api worker schedul…"),
            "{}",
            lines[17]
        );
        assert!(
            lines[20].contains("→ 'holla disk analyze --artifacts' · here · read-only"),
            "{}",
            lines[20]
        );
        assert!(lines[2].contains("2›"), "{}", lines[2]);
        assert!(lines[2].contains("+2"), "{}", lines[2]);
        assert!(!text.contains("sqlx-cli"), "{text}");
        let here = plate.cell(6, 2);
        assert_eq!(here.symbol(), "H");
        assert_eq!(here.bg, Color::Rgb(0x27, 0x27, 0x2a));
        assert!(here.modifier.contains(termrock::Modifier::BOLD));
        let gutter = plate.cell(1, 2);
        assert_eq!(gutter.symbol(), " ");
        assert_eq!(gutter.fg, Color::Rgb(0x73, 0x73, 0x73));
        let hidden = plate.cell(73, 2);
        assert_eq!(hidden.symbol(), "2");
        assert_eq!(hidden.fg, Color::Rgb(0xb3, 0xb3, 0xb3));
        let more = plate.cell(76, 2);
        assert_eq!(more.symbol(), "+");
        assert_eq!(more.fg, Color::Rgb(0xb3, 0xb3, 0xb3));
        let rule = plate.cell(1, 3);
        assert_eq!(rule.symbol(), "─");
        assert_eq!(rule.fg, Color::Rgb(0x3a, 0x3a, 0x3a));
        let accent = plate.cell(5, 3);
        assert_eq!(accent.symbol(), "━");
        let selected = plate.cell(4, 8);
        assert_eq!(selected.symbol(), "I");
        assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
        let thumb = plate.cell(78, 8);
        assert_eq!(thumb.symbol(), "┃");
        assert_eq!(thumb.fg, Color::Rgb(0x80, 0x80, 0x80));
        assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
        let world = &plate.app().world;
        assert!(catalogue::activities_multi_view(world, 120, false).is_none());
        assert!(catalogue::activities_multi_view(world, 80, true).is_none());
        let wide = harness(
            App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        assert!(wide.text().contains("btm"), "{}", wide.text());
        assert!(!wide.text().contains("sqlx-cli"), "{}", wide.text());
        assert!(
            !wide.text().contains("'holla disk analyze --artifacts'"),
            "{}",
            wide.text()
        );
        let later = harness(
            App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 4_000),
            Theme::junie(),
            80,
            24,
        );
        assert!(!later.text().contains("Follow acme-api-1 logs"));
        assert!(!later.text().contains("holla disk analyze"));
        assert!(catalogue::activities_multi_view(&later.app().world, 80, false).is_none());
        assert!(!catalogue::activities_multi_tag_frame(&later.app().world));
    }
    #[test]
    fn activities_multi_100x30_frame_40_paints_the_mid_plate_and_later_seeks_keep_home() {
        let plate = harness(
            App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 40),
            Theme::junie(),
            100,
            30,
        );
        let text = plate.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines[2].contains("2›"), "{}", lines[2]);
        assert!(lines[2].contains("+2"), "{}", lines[2]);
        assert!(!lines[2].contains("btm"), "{}", lines[2]);
        assert!(
            lines[8].contains("› Inspect 10 GB of generated artif…"),
            "{}",
            lines[8]
        );
        assert!(
            lines[9].contains("Install missing tools · sqlx-cli"),
            "{}",
            lines[9]
        );
        assert!(
            lines[17].contains("logs · api worker scheduler"),
            "{}",
            lines[17]
        );
        assert!(lines[24].contains("Children · 3 projects"), "{}", lines[24]);
        assert!(
            lines[26].contains("→ 'holla disk analyze --artifacts' · here · read-only"),
            "{}",
            lines[26]
        );
        assert!(lines[28].contains("~/work/acme"), "{}", lines[28]);
        assert!(lines[28].contains("4 running"), "{}", lines[28]);
        assert!(lines[29].contains("Enter Open"), "{}", lines[29]);
        assert!(
            lines[29].contains("Alt+Enter Alternatives…"),
            "{}",
            lines[29]
        );
        assert!(
            lines[29].contains("api tests finished · exit 0"),
            "{}",
            lines[29]
        );
        assert!(!lines[29].contains("Tab Preview"), "{}", lines[29]);
        assert!(!lines[29].contains("Ctrl+G"), "{}", lines[29]);
        let hidden = plate.cell(93, 2);
        assert_eq!(hidden.symbol(), "2");
        assert_eq!(hidden.fg, Color::Rgb(0xb3, 0xb3, 0xb3));
        let more = plate.cell(96, 2);
        assert_eq!(more.symbol(), "+");
        assert_eq!(more.fg, Color::Rgb(0xb3, 0xb3, 0xb3));
        let ellipsis = plate.cell(60, 29);
        assert_eq!(ellipsis.symbol(), "…");
        assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
        let thumb = plate.cell(98, 8);
        assert_eq!(thumb.symbol(), "┃");
        assert_eq!(thumb.fg, Color::Rgb(0x80, 0x80, 0x80));
        let track = plate.cell(98, 24);
        assert_eq!(track.symbol(), "┆");
        assert_eq!(track.fg, Color::Rgb(0x47, 0x47, 0x47));
        assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
        let world = &plate.app().world;
        assert!(catalogue::activities_multi_view(world, 100, false).is_some());
        assert!(catalogue::activities_multi_view(world, 120, false).is_none());
        assert_ne!(
            catalogue::activities_multi_view(world, 100, true),
            catalogue::activities_multi_view(world, 100, false)
        );
        let other = App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 40);
        assert!(catalogue::activities_multi_view(&other.world, 100, false).is_none());
        let wide = harness(
            App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        assert!(wide.text().contains("btm"), "{}", wide.text());
        assert!(!wide.text().contains("sqlx-cli"), "{}", wide.text());
        let narrow = harness(
            App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 40),
            Theme::junie(),
            80,
            24,
        );
        assert!(
            narrow.text().contains("Install missing tools · s…"),
            "{}",
            narrow.text()
        );
        assert!(!narrow.text().contains("sqlx-cli"), "{}", narrow.text());
        let later = harness(
            App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 4_000),
            Theme::junie(),
            100,
            30,
        );
        assert!(!later.text().contains("Follow acme-api-1 logs"));
        assert!(!later.text().contains("holla disk analyze"));
        assert!(catalogue::activities_multi_view(&later.app().world, 100, false).is_none());
        assert!(!catalogue::activities_multi_tag_frame(&later.app().world));
    }
    #[test]
    fn activities_multi_history_frame_40_paints_the_deploy_tag_and_later_seeks_keep_home() {
        let mut app =
            App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 40);
        app.home.history = true;
        let screen = harness(app, Theme::junie(), 120, 40);
        let text = screen.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines[7].contains("› Deploy preview"), "{}", lines[7]);
        assert!(lines[7].contains("Current folder…"), "{}", lines[7]);
        assert!(lines[8].contains("Follow acme-api-1 logs"), "{}", lines[8]);
        assert!(
            lines[10].contains("tools/deploy-pre…preview"),
            "{}",
            lines[10]
        );
        assert!(lines[14].contains("Pull"), "{}", lines[14]);
        assert!(lines[14].contains("used 20 times on mbp"), "{}", lines[14]);
        assert!(lines[16].contains("Running"), "{}", lines[16]);
        assert!(
            lines[17].contains("logs · api worker schedu…"),
            "{}",
            lines[17]
        );
        assert!(lines[18].contains("mutating"), "{}", lines[18]);
        assert!(lines[20].contains("trust required"), "{}", lines[20]);
        assert!(lines[39].contains("Enter Trust…"), "{}", lines[39]);
        assert!(
            lines[39].contains("api tests finished · exit 0"),
            "{}",
            lines[39]
        );
        assert!(!lines[39].contains("Enter Open"), "{}", lines[39]);
        assert!(!lines[39].contains("Ctrl+↑↓"), "{}", lines[39]);
        assert!(!lines[39].contains("Deploy"), "{}", lines[39]);
        let ellipsis = screen.cell(75, 39);
        assert_eq!(ellipsis.symbol(), "…");
        assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
        assert!(!text.contains("nothing used here yet"), "{text}");
        assert!(!text.contains("holla disk anal"), "{text}");
        let selected = screen.cell(2, 7);
        assert_eq!(selected.symbol(), "›");
        assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
        assert!(selected.modifier.contains(termrock::Modifier::BOLD));
        let runs = screen.cell(92, 10);
        assert_eq!(runs.symbol(), "t");
        assert_eq!(runs.fg, Color::Rgb(0xb3, 0xb3, 0xb3));
        let risk = screen.cell(92, 18);
        assert_eq!(risk.symbol(), "m");
        assert_eq!(risk.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        assert!(
            screen.diagnostics().is_empty(),
            "{:?}",
            screen.diagnostics()
        );
        let mut later_app =
            App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 4_000);
        later_app.home.history = true;
        let later = harness(later_app, Theme::junie(), 120, 40);
        assert!(!later.text().contains("tools/deploy-pre"));
        assert!(!later.text().contains("used 20 times on mbp"));
        assert!(later.text().contains("Search actions"), "{}", later.text());
        let mut stepped =
            App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 40);
        stepped.home.history = true;
        assert!(catalogue::activities_multi_history_tag(&stepped.world).is_some());
        assert!(termrock::App::accept_paused_clock_step(&mut stepped, 1_100));
        assert!(catalogue::activities_multi_history_tag(&stepped.world).is_none());
        assert!(!catalogue::activities_multi_tag_frame(&stepped.world));
    }
    #[test]
    fn discovery_results_header_is_tag_grey_at_column_4() {
        let harness = harness(
            App::for_scenario_without_history(Scenario::ParityDiscovery, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        let (x, y) = harness.find("Suggested here").expect("results header");
        assert_eq!((x, y), (4, 6));
        let title = harness.cell(x, y);
        assert_eq!(title.symbol(), "S");
        assert_eq!(title.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        assert!(!title.modifier.contains(termrock::Modifier::BOLD));
        let pad = harness.cell(1, y);
        assert_eq!(pad.symbol(), " ");
        assert_eq!(pad.fg, Color::Rgb(0xff, 0xff, 0xff));
        assert_eq!(pad.bg, Color::Rgb(0, 0, 0));
        assert!(!pad.modifier.contains(termrock::Modifier::BOLD));
        let before = harness.cell(3, y);
        assert_eq!(before.symbol(), " ");
        assert_eq!(before.fg, Color::Rgb(0xff, 0xff, 0xff));
    }
    #[test]
    fn discovery_frame_40_shows_pull_and_the_preview_plate() {
        let harness = harness(
            App::for_scenario_without_history(Scenario::ParityDiscovery, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        let text = harness.text();
        assert!(text.contains("Pull"), "{text}");
        assert!(text.contains("move every .idea"), "{text}");
        assert!(!text.contains("Monitor this host"), "{text}");
        let (x, y) = harness.find("Pull").expect("pull row");
        assert_eq!((x, y), (4, 7));
        let plate = harness.cell(79, 6);
        assert_eq!(plate.bg, Color::Rgb(0x11, 0x11, 0x11));
        assert_eq!(plate.symbol(), " ");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(
            lines[38].trim(),
            "~/work/probe   main   ↓1             ! docker, github unavailable             ▲ 4 config warnings   truecolor · 120×40"
        );
        assert!(lines[39].contains("Type Search"));
        assert!(lines[39].contains("Ctrl+G Activities"));
        let origin = harness.cell(0, 38);
        assert_eq!(origin.symbol(), " ");
        assert_eq!(origin.fg, Color::Rgb(0xff, 0xff, 0xff));
        assert_eq!(origin.bg, Color::Rgb(0x18, 0x18, 0x1b));
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn first_use_frame_40_shows_the_disk_tag() {
        let screen = harness(
            App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        let text = screen.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines[0].contains("no project"), "{}", lines[0]);
        assert!(lines[0].contains("mbp · local"), "{}", lines[0]);
        assert!(lines[7].contains("› Analyze disk usage"), "{}", lines[7]);
        assert!(lines[7].contains("/ is 91% full"), "{}", lines[7]);
        assert!(lines[8].contains("sustained pressure…"), "{}", lines[8]);
        assert!(lines[14].contains("'holla disk analyze'"), "{}", lines[14]);
        assert!(lines[27].contains("custom path…"), "{}", lines[27]);
        assert_eq!(
            lines[38].trim(),
            "~/scratch                                                                     / ━━━━━━━━━━─ 91% ▲   truecolor · 120×40"
        );
        assert!(lines[39].contains("Enter Open"), "{}", lines[39]);
        assert!(!lines[39].contains("Confirm"), "{}", lines[39]);
        let runs = screen.cell(92, 14);
        assert_eq!(runs.fg, Color::Rgb(0xb3, 0xb3, 0xb3));
        let risk = screen.cell(92, 20);
        assert_eq!(risk.symbol(), "r");
        assert_eq!(risk.fg, Color::Rgb(0x73, 0x73, 0x73));
        let gate = screen.cell(92, 22);
        assert_eq!(gate.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        let bar = screen.cell(81, 38);
        assert_eq!(bar.symbol(), "━");
        assert_eq!(bar.fg, Color::Rgb(0xf5, 0x9e, 0x09));
        let rest = screen.cell(91, 38);
        assert_eq!(rest.symbol(), "─");
        assert_eq!(rest.fg, Color::Rgb(0x3a, 0x3a, 0x3a));
        let early = harness(
            App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 0),
            Theme::junie(),
            120,
            40,
        );
        assert!(!early.text().contains("holla disk analyze"));
        assert!(
            screen.diagnostics().is_empty(),
            "{:?}",
            screen.diagnostics()
        );
    }
    #[test]
    fn first_use_80x24_frame_40_paints_the_narrow_plate_and_later_seeks_keep_home() {
        let plate = harness(
            App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 40),
            Theme::junie(),
            80,
            24,
        );
        let text = plate.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines[7].contains("› Analyze disk usage"), "{}", lines[7]);
        assert!(lines[7].contains("/ is 91% full"), "{}", lines[7]);
        assert!(lines[8].contains("sustained pressure…"), "{}", lines[8]);
        assert!(
            lines[20].contains("→ 'holla disk analyze' · here · read-only"),
            "{}",
            lines[20]
        );
        assert!(!text.contains("Monitor this host"), "{text}");
        let selected = plate.cell(4, 7);
        assert_eq!(selected.symbol(), "A");
        assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
        assert!(selected.modifier.contains(termrock::Modifier::BOLD));
        let kind = plate.cell(27, 7);
        assert_eq!(kind.symbol(), "d");
        assert!(!kind.modifier.contains(termrock::Modifier::BOLD));
        assert_eq!(kind.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
        let ellipsis = plate.cell(73, 23);
        assert_eq!(ellipsis.symbol(), "…");
        assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
        assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
        let wide = harness(
            App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        assert!(
            wide.text()
                .lines()
                .nth(14)
                .unwrap()
                .contains("'holla disk analyze'")
        );
        let runs = wide.cell(92, 14);
        assert_eq!(runs.fg, Color::Rgb(0xb3, 0xb3, 0xb3));
        let later = harness(
            App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 4_000),
            Theme::junie(),
            80,
            24,
        );
        assert!(!later.text().contains("holla disk analyze"));
        assert!(
            later.text().contains("Monitor this host"),
            "{}",
            later.text()
        );
        assert!(!catalogue::first_use_tag_frame(&later.app().world));
    }
    #[test]
    fn first_use_100x30_frame_40_paints_the_mid_plate_and_later_seeks_keep_home() {
        let plate = harness(
            App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 40),
            Theme::junie(),
            100,
            30,
        );
        let text = plate.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines[7].contains("› Analyze disk usage"), "{}", lines[7]);
        assert!(lines[7].contains("/ is 91% full"), "{}", lines[7]);
        assert!(
            lines[8].contains("sustained pressure · continuous view fits"),
            "{}",
            lines[8]
        );
        assert!(!lines[8].contains('…'), "{}", lines[8]);
        assert!(
            lines[26].contains("→ 'holla disk analyze' · here · read-only"),
            "{}",
            lines[26]
        );
        assert!(!text.contains("largest-first"), "{text}");
        assert!(lines[29].contains("Enter Open"), "{}", lines[29]);
        assert!(lines[29].contains("Ctrl+↑↓ Scope"), "{}", lines[29]);
        assert!(!lines[29].contains("Ctrl+G"), "{}", lines[29]);
        let selected = plate.cell(4, 7);
        assert_eq!(selected.symbol(), "A");
        assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
        assert!(selected.modifier.contains(termrock::Modifier::BOLD));
        let kind = plate.cell(27, 7);
        assert_eq!(kind.symbol(), "d");
        assert!(!kind.modifier.contains(termrock::Modifier::BOLD));
        assert_eq!(kind.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
        let ellipsis = plate.cell(88, 29);
        assert_eq!(ellipsis.symbol(), "…");
        assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
        assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
        let world = &plate.app().world;
        assert!(catalogue::first_use_view(world, 100, false).is_some());
        assert!(catalogue::first_use_view(world, 120, false).is_none());
        assert_ne!(
            catalogue::first_use_view(world, 100, true),
            catalogue::first_use_view(world, 100, false)
        );
        let wide = harness(
            App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        assert!(wide.text().contains("Ctrl+G Activities"), "{}", wide.text());
        assert!(wide.text().contains("largest-first"), "{}", wide.text());
        let narrow = harness(
            App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 40),
            Theme::junie(),
            80,
            24,
        );
        assert!(
            narrow.text().contains("sustained pressure…"),
            "{}",
            narrow.text()
        );
        assert!(
            !narrow
                .text()
                .contains("sustained pressure · continuous view fits"),
            "{}",
            narrow.text()
        );
        let later = harness(
            App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 4_000),
            Theme::junie(),
            100,
            30,
        );
        assert!(!later.text().contains("holla disk analyze"));
        assert!(catalogue::first_use_view(&later.app().world, 100, false).is_none());
        assert!(!catalogue::first_use_tag_frame(&later.app().world));
        let other = App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 40);
        assert!(catalogue::first_use_view(&other.world, 100, false).is_none());
        let mut history = App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 40);
        history.home.history = true;
        assert!(catalogue::first_use_view(&history.world, 100, true).is_some());
        let history_screen = harness(history, Theme::junie(), 100, 30);
        assert!(
            history_screen.text().contains("holla snapshot"),
            "{}",
            history_screen.text()
        );
        assert!(
            !history_screen
                .text()
                .contains("→ 'holla disk analyze' · here · read-only"),
            "{}",
            history_screen.text()
        );
    }
    #[test]
    fn first_use_history_frame_40_paints_the_system_tag_and_later_seeks_keep_home() {
        let mut app = App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 40);
        app.home.history = true;
        let screen = harness(app, Theme::junie(), 120, 40);
        let text = screen.text();
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines[7].contains("› Show system resources"), "{}", lines[7]);
        assert!(lines[7].contains("CPU 78% for 4 min"), "{}", lines[7]);
        assert!(
            lines[12].contains("everything used here is suggested above"),
            "{}",
            lines[12]
        );
        assert!(
            lines[13].contains("'holla snapshot system'"),
            "{}",
            lines[13]
        );
        assert!(lines[22].contains("runs directly"), "{}", lines[22]);
        assert!(!text.contains("nothing used here yet"), "{text}");
        assert!(!text.contains("holla disk analyze"), "{text}");
        let selected = screen.cell(2, 7);
        assert_eq!(selected.symbol(), "›");
        assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
        assert!(selected.modifier.contains(termrock::Modifier::BOLD));
        let runs = screen.cell(92, 13);
        assert_eq!(runs.symbol(), "'");
        assert_eq!(runs.fg, Color::Rgb(0xb3, 0xb3, 0xb3));
        let risk = screen.cell(92, 20);
        assert_eq!(risk.symbol(), "r");
        assert_eq!(risk.fg, Color::Rgb(0x73, 0x73, 0x73));
        let gate = screen.cell(92, 22);
        assert_eq!(gate.symbol(), "r");
        assert_eq!(gate.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        assert_eq!(
            lines[38].trim(),
            "~/scratch                                                                     / ━━━━━━━━━━─ 91% ▲   truecolor · 120×40"
        );
        assert!(
            screen.diagnostics().is_empty(),
            "{:?}",
            screen.diagnostics()
        );
        let mut later_app =
            App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 4_000);
        later_app.home.history = true;
        let later = harness(later_app, Theme::junie(), 120, 40);
        assert!(!later.text().contains("holla snapshot system"));
        assert!(
            !later
                .text()
                .contains("everything used here is suggested above")
        );
        assert!(later.text().contains("Search actions"), "{}", later.text());
        let mut stepped = App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 40);
        stepped.home.history = true;
        assert!(catalogue::first_use_history_tag(&stepped.world).is_some());
        assert!(termrock::App::accept_paused_clock_step(&mut stepped, 1_100));
        assert!(catalogue::first_use_history_tag(&stepped.world).is_none());
        assert!(!catalogue::first_use_tag_frame(&stepped.world));
    }
    #[test]
    fn paused_clock_step_advances_world_time_by_1100_ms() {
        let mut paused = App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 0);
        let start = paused.world.now_ms();
        assert!(termrock::App::accept_paused_clock_step(&mut paused, 1_100));
        assert_eq!(paused.world.now_ms(), start + 1_100);
        assert!(!paused.world.clock.running);

        let mut running = App::for_scenario_without_history(Scenario::FirstUse, Motion::Full, 0);
        let start = running.world.now_ms();
        assert!(!termrock::App::accept_paused_clock_step(
            &mut running,
            1_100
        ));
        assert_eq!(running.world.now_ms(), start);
    }

    #[test]
    fn home_query_stays_armed_under_action_focus_with_shared_caret_and_paste() {
        let mut harness = app(Scenario::HardCases);
        assert!(harness.tab_to(home::ROWS));
        let _ = harness.type_str("git");
        assert_eq!(harness.focus(), Some(home::ROWS));
        assert_eq!(harness.app().home.query(), "git");
        assert!(harness.cursor().is_some());
        let _ = harness.paste(" q?0 ");
        assert_eq!(harness.app().home.query(), "git q?0 ");
        assert!(harness.app().overlay.is_none());
        let _ = harness.key(KeyCode::Backspace);
        assert_eq!(harness.app().home.query(), "git q?0");
        assert!(harness.app().home.rows(&harness.app().world).is_empty());
        assert_ne!(harness.focus(), Some(home::ROWS));
        assert!(harness.cursor().is_some());
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn empty_query_chrome_exceptions_and_modal_typing_do_not_leak() {
        for (character, title) in [('?', "Key reference"), ('q', "Quit holla?")] {
            let mut harness = app(Scenario::HardCases);
            assert!(harness.tab_to(home::ROWS));
            let _ = harness.key(KeyCode::Char(character));
            assert!(harness.text().contains(title));
            assert_eq!(harness.app().home.query(), "");
            let _ = harness.type_str("q?0text");
            assert_eq!(harness.app().home.query(), "");
            assert!(harness.app().overlay.is_some());
            let _ = harness.key(KeyCode::Esc);
            assert!(harness.app().overlay.is_none());
            let _ = harness.paste("q?");
            assert_eq!(harness.app().home.query(), "q?");
            assert!(harness.app().overlay.is_none());
            assert!(
                harness.diagnostics().is_empty(),
                "{:?}",
                harness.diagnostics()
            );
        }
    }
    #[test]
    fn unmatched_home_query_has_source_notice_and_no_empty_row_focus_stop() {
        let mut harness = app(Scenario::HardCases);
        assert!(harness.tab_to(home::QUERY));
        let _ = harness.paste("  impossible-no-fixture-match  ");
        assert!(harness.app().home.rows(&harness.app().world).is_empty());
        assert!(
            harness
                .text()
                .contains("Nothing matches “impossible-no-fixture-match” here")
        );
        let revision = harness.app().effect_revision();
        let _ = harness.key(KeyCode::Down);
        assert_ne!(harness.focus(), Some(home::ROWS));
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(harness.tab_to(home::QUERY));
        let _ = harness.key(KeyCode::Esc);
        assert!(harness.app().home.query().is_empty());
        let _ = harness.key(KeyCode::Down);
        assert_eq!(harness.focus(), Some(home::ROWS));
        assert_eq!(harness.app().effect_revision(), revision);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn frame_seek_lands_on_discovered_state() {
        let settled = harness(
            App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 4_000),
            Theme::junie(),
            120,
            40,
        );
        assert!(!settled.app().world.discovering());
        assert!(!settled.text().contains("discovering…"));
        let early = harness(
            App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 100),
            Theme::junie(),
            120,
            40,
        );
        assert!(early.app().world.discovering());
        assert!(!early.text().lines().next().unwrap().contains("discovering"));
        assert!(early.diagnostics().is_empty(), "{:?}", early.diagnostics());
    }
    #[test]
    fn actions_picker_preserves_source_footer_height_and_query_isolation() {
        let mut h = app(Scenario::RustDirty);
        let _ = h.type_str("cargo build");
        let _ = h.ctrl('o');
        assert_eq!(
            h.layer_area(crate::screens::actions::PICKER)
                .map(|area| area.height),
            Some(12)
        );
        assert_eq!(h.cursor(), None);
        let text = h.text();
        let footer = text.lines().last().unwrap();
        assert!(
            footer.contains("↑↓ Move")
                && footer.contains("Enter Choose")
                && footer.contains("Esc Cancel")
        );
        assert!(!footer.contains("EDIT"));
        let _ = h.type_str("ignored");
        let _ = h.paste("ignored paste");
        assert_eq!(h.app().home.query(), "cargo build");
        assert!(h.text().contains("Set alias…"));
        let _ = h.key(KeyCode::Esc);
        assert!(h.app().overlay.is_none());
        assert_eq!(h.app().home.query(), "cargo build");
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
    #[test]
    fn home_hint_casing_preserves_lowercase_physical_shortcuts() {
        let mut h = app(Scenario::RustDirty);
        let text = h.text();
        let footer = text.lines().last().unwrap();
        assert!(footer.contains("Ctrl+S Scope"));
        assert!(footer.contains("q Quit"));
        let _ = h.ctrl('s');
        assert!(h.text().contains("scope ‹ here ›"));
        assert!(h.tab_to(home::ROWS));
        let text = h.text();
        let footer = text.lines().last().unwrap();
        assert!(footer.contains("Ctrl+P Preview"));
        assert!(footer.contains("Ctrl+O Actions"));
        assert!(h.tab_to(home::QUERY));
        let _ = h.key(KeyCode::Char('q'));
        assert!(h.text().contains("Quit holla?"));
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
    #[test]
    fn home_product_hints_follow_query_rows_and_yield_to_modal_context() {
        let mut h = app(Scenario::RustDirty);
        assert!(h.text().lines().last().unwrap().contains("Run top match"));
        assert!(h.tab_to(home::ROWS));
        let footer = h.text().lines().last().unwrap().to_owned();
        assert!(footer.contains("Run") && footer.contains("Preview"));
        assert!(!footer.contains("Run top match"));
        let _ = h.key(KeyCode::F(1));
        assert!(!h.text().lines().last().unwrap().contains("Run top match"));
        assert!(!h.text().lines().last().unwrap().contains("Filter"));
        let _ = h.key(KeyCode::Esc);
        assert!(h.tab_to(home::QUERY));
        assert!(h.text().lines().last().unwrap().contains("Type Filter"));
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
    #[test]
    fn home_scope_stays_off_the_menu_row_and_clears_with_escape() {
        let mut harness = app(Scenario::HardCases);
        let _ = harness.ctrl('s');
        assert!(harness.app().home.scope.is_some());
        assert!(!harness.text().lines().next().unwrap().contains("scope:"));
        let _ = harness.key(KeyCode::Esc);
        assert!(harness.app().home.scope.is_none());
        assert!(!harness.text().lines().next().unwrap().contains("scope:"));
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn home_escape_clears_scope_before_canonical_query() {
        let mut harness = app(Scenario::HardCases);
        assert!(harness.tab_to(home::ROWS));
        let _ = harness.type_str("git");
        let _ = harness.ctrl('s');
        assert!(harness.app().home.scope.is_some());
        let _ = harness.key(KeyCode::Esc);
        assert!(harness.app().home.scope.is_none());
        assert_eq!(harness.app().home.query(), "git");
        let _ = harness.key(KeyCode::Esc);
        assert_eq!(harness.app().home.query(), "");
        let _ = harness.type_str("docker");
        assert_eq!(harness.app().home.query(), "docker");
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn home_navigation_uses_shared_boundaries_and_reveals_selected_rows() {
        let mut harness = harness(
            App::for_scenario_without_history(Scenario::HardCases, Motion::Paused, 4_000),
            Theme::junie(),
            120,
            20,
        );
        assert!(harness.tab_to(home::QUERY));
        let count = harness.app().home.rows(&harness.app().world).len();
        let _ = harness.key(KeyCode::Down);
        assert_eq!(harness.focus(), Some(home::ROWS));
        let first = harness
            .app()
            .home
            .selected(&harness.app().world)
            .unwrap()
            .id;
        let _ = harness.key(KeyCode::Up);
        assert_eq!(harness.focus(), Some(home::QUERY));
        let _ = harness.key(KeyCode::Down);
        for _ in 1..count {
            let _ = harness.key(KeyCode::Down);
        }
        assert_eq!(harness.focus(), Some(home::ROWS));
        let last = harness.app().home.selected(&harness.app().world).unwrap();
        assert_ne!(last.id, first);
        assert!(harness.text().contains(&last.title), "{}", harness.text());
        let _ = harness.key(KeyCode::Down);
        assert_ne!(harness.focus(), Some(home::ROWS));
        assert!(harness.tab_to(home::QUERY));
        let _ = harness.key(KeyCode::Down);
        assert_eq!(
            harness
                .app()
                .home
                .selected(&harness.app().world)
                .unwrap()
                .id,
            first
        );
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn activity_digits_are_route_scoped_and_cycle_from_home_starts_first() {
        let mut harness = app(Scenario::ActivitiesMulti);
        let ids: Vec<_> = activity::ordered(&harness.app().world)
            .iter()
            .map(|a| a.id)
            .collect();
        assert!(ids.len() >= 2);
        let _ = harness.type_str("12");
        assert_eq!(harness.app().home.query(), "12");
        let _ = harness.ctrl('a');
        assert_eq!(harness.app().activities.current_id(), Some(ids[0]));
        let _ = harness.key(KeyCode::Char('2'));
        assert_eq!(harness.app().activities.current_id(), Some(ids[1]));
        let _ = harness.key(KeyCode::Char('9'));
        assert_eq!(harness.app().status.as_deref(), Some("No activity 9"));
        let _ = harness.key(KeyCode::Char('0'));
        assert!(harness.app().route == Route::Home);
        let _ = harness.ctrl('a');
        assert_eq!(harness.app().activities.current_id(), Some(ids[0]));
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn interrupt_quits_outside_modals_and_quit_chord_precedes_open_menu() {
        let mut harness = app(Scenario::HardCases);
        let _ = harness.key(KeyCode::F(1));
        let _ = harness.ctrl('c');
        assert!(!harness.app().quit);
        let _ = harness.key(KeyCode::Esc);
        let _ = harness.key(KeyCode::F(10));
        assert!(
            harness.app().menu.is_open(),
            "F10 must open the real menu before testing precedence"
        );
        let _ = harness.ctrl('q');
        assert!(harness.text().contains("Quit holla?"));
        assert!(!harness.app().quit);
        let mut harness = app(Scenario::HardCases);
        let _ = harness.key(KeyCode::F(10));
        let _ = harness.ctrl('c');
        assert!(harness.app().quit);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn menu_opens_before_publication_and_survives_focus_settlement() {
        let mut harness = app(Scenario::HardCases);
        assert!(harness.tab_to(home::QUERY));
        let _ = harness.type_str("git");
        let mut harness = harness.with_auto_draw(false);
        let _ = harness.key(KeyCode::F(10));
        assert!(harness.app().menu.is_open(), "event must open menu");
        harness.draw();
        assert!(
            harness.app().menu.is_open(),
            "publication must preserve menu"
        );
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn too_small_surface_consumes_commands_and_quits_without_query_or_modal_effects() {
        for key in [KeyCode::Char('q'), KeyCode::Char('c')] {
            let mut harness = app(Scenario::HardCases);
            let _ = harness.type_str("git");
            let _ = harness.key(KeyCode::F(1));
            let revision = harness.app().effect_revision();
            let _ = harness.resize(40, 10);
            assert!(
                harness.diagnostics().is_empty(),
                "resize: {:?}",
                harness.diagnostics()
            );
            let _ = harness.key(KeyCode::F(1));
            assert!(!harness.app().quit);
            let _ = harness.resize(120, 40);
            assert!(harness.text().contains("Key reference"));
            assert!(
                harness.diagnostics().is_empty(),
                "restore: {:?}",
                harness.diagnostics()
            );
            let _ = harness.resize(40, 10);
            assert!(
                harness.diagnostics().is_empty(),
                "second resize: {:?}",
                harness.diagnostics()
            );
            if key == KeyCode::Char('c') {
                let _ = harness.ctrl('c');
            } else {
                let _ = harness.key(key);
            }
            assert!(harness.app().quit);
            assert_eq!(harness.app().home.query(), "git");
            assert_eq!(harness.app().effect_revision(), revision);
            assert!(
                harness.diagnostics().is_empty(),
                "{:?}",
                harness.diagnostics()
            );
        }
    }
    #[test]
    fn plan_physical_edges_leave_steps_without_changing_inclusion_or_effects() {
        let mut harness = app(Scenario::DockerCleanup);
        let _ = harness.type_str("docker system prune");
        let _ = harness.key(KeyCode::Enter);
        assert!(harness.app().route == Route::Plan);
        assert!(harness.tab_to(plan::STEPS));
        let _ = harness.key(KeyCode::Home);
        let included = harness.app().plan.as_ref().unwrap().plan().included_count();
        let revision = harness.app().effect_revision();
        let _ = harness.key(KeyCode::Up);
        assert_ne!(harness.focus(), Some(plan::STEPS));
        assert!(harness.tab_to(plan::STEPS));
        let _ = harness.key(KeyCode::End);
        let _ = harness.key(KeyCode::Down);
        assert_ne!(harness.focus(), Some(plan::STEPS));
        assert_eq!(
            harness.app().plan.as_ref().unwrap().plan().included_count(),
            included
        );
        assert_eq!(harness.app().effect_revision(), revision);
        assert!(harness.app().overlay.is_none());
        assert!(harness.activation_feedback().is_none());
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn plan_space_is_focus_bound_and_preserves_chrome_activation() {
        let mut harness = app(Scenario::DockerCleanup);
        let _ = harness.type_str("docker system prune");
        let _ = harness.key(KeyCode::Enter);
        let index = harness
            .app()
            .plan
            .as_ref()
            .unwrap()
            .plan()
            .steps()
            .iter()
            .position(|step| step.optional)
            .unwrap();
        for _ in 0..index {
            let _ = harness.key(KeyCode::Down);
        }
        let included = harness.app().plan.as_ref().unwrap().plan().included_count();
        assert!(harness.tab_to(MENU));
        let _ = harness.key(KeyCode::Char(' '));
        assert!(harness.app().menu.is_open());
        assert_eq!(
            harness.app().plan.as_ref().unwrap().plan().included_count(),
            included
        );
        let _ = harness.key(KeyCode::Esc);
        assert!(harness.tab_to(MENU));
        let _ = harness.key(KeyCode::Enter);
        assert!(harness.app().menu.is_open());
        let _ = harness.key(KeyCode::Esc);
        assert!(harness.tab_to(plan::STEPS));
        let _ = harness.key(KeyCode::Char(' '));
        assert!(harness.app().plan.as_ref().unwrap().plan().included_count() < included);
        assert!(harness.app().overlay.is_none());
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn plan_keyboard_feedback_uses_post_request_owner_and_double_click_cannot_open_gate() {
        let mut harness = harness(
            App::for_scenario_without_history(Scenario::DockerCleanup, Motion::Full, 4_000),
            Theme::junie(),
            120,
            40,
        );
        let _ = harness.type_str("docker system prune");
        let _ = harness.key(KeyCode::Enter);
        assert!(
            harness.activation_feedback().is_none(),
            "Home query submission does not flash"
        );
        let step = harness
            .app()
            .plan
            .as_ref()
            .unwrap()
            .selected()
            .unwrap()
            .id
            .clone();
        let part = PartRef::item(Part::ROW, ItemKey::text(&step));
        let _ = harness.click_part(plan::STEPS, part);
        let _ = harness.click_part(plan::STEPS, part);
        assert!(harness.app().overlay.is_none());
        let _ = harness.advance(std::time::Duration::from_millis(80));
        assert_eq!(
            harness.activation_feedback().unwrap().remaining,
            std::time::Duration::from_millis(60)
        );
        let _ = harness.key(KeyCode::Char(' '));
        let feedback = harness.activation_feedback().unwrap();
        assert_eq!((feedback.owner, feedback.part), (plan::STEPS, part));
        assert_eq!(feedback.remaining, std::time::Duration::from_millis(140));
        let _ = harness.key(KeyCode::Enter);
        let gate = Dialog::new(crate::screens::plan_gate::GATE);
        assert_eq!(
            harness.activation_feedback().unwrap().owner,
            gate.input_id()
        );
        assert_eq!(harness.focus(), Some(gate.input_id()));
        let _ = harness.advance(std::time::Duration::from_millis(80));
        let _ = harness.advance(std::time::Duration::from_millis(80));
        assert!(harness.activation_feedback().is_none());
        let phrase = harness
            .app()
            .plan
            .as_ref()
            .unwrap()
            .plan()
            .phrase()
            .to_owned();
        let _ = harness.type_str(&phrase);
        let _ = harness.key(KeyCode::Enter);
        assert_eq!(harness.app().effect_revision(), 0, "editor Enter only arms");
        assert!(
            harness.activation_feedback().is_none(),
            "modal editing has no keyboard flash"
        );
        assert!(harness.tab_to(gate.action_id(1)));
        let _ = harness.key(KeyCode::Char(' '));
        assert_eq!(harness.app().effect_revision(), 1);
        assert!(harness.app().plan.as_ref().unwrap().plan().ran());
        assert!(
            harness.activation_feedback().is_none(),
            "modal confirmation preserves source feedback policy"
        );
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn copy_reports_osc52_status_without_emitting_a_sequence() {
        let mut harness = app(Scenario::RustDirty);
        let _ = harness.type_str("cargo build");
        let _ = harness.ctrl('o');
        let _ = harness.key(KeyCode::Down);
        let _ = harness.key(KeyCode::Down);
        let _ = harness.key(KeyCode::Enter);
        let refused = harness.text();
        assert!(refused.contains("does not accept OSC 52"), "{refused}");
        assert!(!refused.contains('\u{1b}'), "{refused}");
        assert!(harness.app().route == Route::Home);

        harness.app_mut().world.host.osc52 = true;
        harness.app_mut().world.host.osc52_limit = 4;
        harness.app_mut().copy_osc52("cargo build");
        harness.draw();
        let limited = harness.text();
        assert!(
            limited.contains("exceed the terminal's 4 byte OSC 52 limit"),
            "{limited}"
        );
        assert!(!limited.contains('\u{1b}'));

        harness.app_mut().world.host.osc52_limit = 100_000;
        harness.app_mut().copy_osc52("cargo build");
        harness.draw();
        let flushed = harness.text();
        assert!(
            flushed.contains(
                "Copied · simulated clipboard · OSC 52 sequence was not written · cargo build"
            ),
            "{flushed}"
        );
        assert!(!flushed.contains("flushed"), "{flushed}");
        assert!(!flushed.contains('\u{1b}'));
        assert!(harness.app().route == Route::Home);
    }
    #[test]
    fn closed_pages_open_from_commands_and_leave_the_home_frame_unchanged() {
        let mut closed = app(Scenario::DiskCleanup);
        assert!(closed.app().route == Route::Disk);
        let _ = closed.key(KeyCode::Esc);
        assert!(closed.app().route == Route::Home);
        assert!(closed.app().overlay.is_none());
        let before = closed.text();
        assert!(!before.contains("Largest files"), "{before}");
        assert!(!before.contains("Review cleanup"), "{before}");
        assert!(!before.contains('\u{1b}'));
        let candidates = closed
            .app()
            .world
            .disk
            .as_ref()
            .map(|disk| disk.candidates.clone());
        let pages = [
            (KeyCode::F(2), "Largest files"),
            (KeyCode::F(3), "Review cleanup"),
            (KeyCode::F(4), "Folder browser"),
            (KeyCode::F(5), "Find in home"),
            (KeyCode::F(6), "Custom action"),
            (KeyCode::F(7), "Snapshot page"),
            (KeyCode::F(8), "Trust this definition"),
        ];
        for (key, title) in pages {
            let mut harness = app(Scenario::DiskCleanup);
            let _ = harness.key(KeyCode::Esc);
            assert!(harness.app().route == Route::Home, "{title}");
            let _ = harness.key(key);
            let text = harness.text();
            assert!(text.contains(title), "{title} missing from {text}");
            assert!(harness.app().route == Route::Home, "{title}");
            assert!(matches!(harness.app().overlay, Some(Overlay::Dialog(_))));
            assert!(!text.contains('\u{1b}'), "{title}");
            let _ = harness.key(KeyCode::Esc);
            assert!(harness.app().overlay.is_none(), "{title}");
            assert!(harness.app().route == Route::Home, "{title}");
            assert_eq!(
                harness
                    .app()
                    .world
                    .disk
                    .as_ref()
                    .map(|disk| disk.candidates.clone()),
                candidates,
                "{title} changed disk candidates"
            );
        }
    }
    #[test]
    fn paused_scenarios_stay_on_home_without_a_finder_stack_or_osc52() {
        for scenario in Scenario::ALL {
            let harness = app(scenario);
            let expected = match scenario.screen() {
                Screen::Disk => Route::Disk,
                Screen::Activity => Route::Activity,
                Screen::Finder | Screen::Plan => Route::Home,
                Screen::Files | Screen::Cleanup | Screen::Review | Screen::Snapshot => Route::Home,
            };
            assert!(
                harness.app().route == expected,
                "{} opened {:?}",
                scenario.name(),
                harness.app().route as u8
            );
            assert!(harness.app().overlay.is_none(), "{}", scenario.name());
            assert!(
                harness.text().contains("holla❯"),
                "{} missing brand",
                scenario.name()
            );
            assert!(
                harness.text().contains(scenario.boot_title()),
                "{} missing {}",
                scenario.name(),
                scenario.boot_title()
            );
            assert!(
                !harness.text().contains('\u{1b}'),
                "frame 0 must not emit an OSC 52 clipboard sequence: {}",
                scenario.name()
            );
        }
        assert_eq!(Scenario::ALL.len(), 34);
    }
    #[test]
    fn every_real_app_scenario_renders_purely_with_persistent_identity() {
        for scenario in Scenario::ALL {
            let mut harness = app(scenario);
            assert!(harness.text().contains("holla❯"));
            assert!(harness.text().contains(&harness.app().world.host.name));
            let catalogue = catalogue::catalogue(&harness.app().world);
            let time = harness.app().world.now_ms();
            let text = harness.text();
            harness.draw();
            harness.draw();
            assert_eq!(harness.text(), text);
            assert_eq!(catalogue::catalogue(&harness.app().world), catalogue);
            assert_eq!(harness.app().world.now_ms(), time);
        }
    }
    #[test]
    fn help_escape_and_quit_use_actual_dialog_routes() {
        let mut harness = app(Scenario::FirstUse);
        let _ = harness.key(KeyCode::F(1));
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(matches!(harness.app().overlay, Some(Overlay::Dialog(_))));
        assert!(harness.text().contains("Key reference"));
        let _ = harness.key(KeyCode::Esc);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(harness.app().overlay.is_none());
        let _ = harness.ctrl('q');
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(harness.text().contains("Quit holla?"));
        let _ = harness.click_id(Dialog::new(dialogs::DIALOG).action_id(1));
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(harness.app().quit);
    }
    #[test]
    fn filtered_cleanup_enters_review_and_exact_gate_applies_once() {
        let mut harness = app(Scenario::DockerCleanup);
        assert!(harness.tab_to(home::QUERY));
        let _ = harness.type_str("docker system prune");
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        let _ = harness.key(KeyCode::Enter);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(matches!(harness.app().route, Route::Plan));
        assert_eq!(harness.app().world.effect_revision, 0);
        assert!(harness.tab_to(plan::STEPS));
        let _ = harness.key(KeyCode::Enter);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(matches!(harness.app().overlay, Some(Overlay::Gate(_))));
        let phrase = harness
            .app()
            .plan
            .as_ref()
            .unwrap()
            .plan()
            .phrase()
            .to_owned();
        let _ = harness.type_str(&phrase);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        let _ = harness.click_id(Dialog::new(crate::screens::plan_gate::GATE).action_id(1));
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert_eq!(harness.app().world.effect_revision, 1);
        assert!(harness.app().plan.as_ref().unwrap().plan().ran());
        assert!(harness.app().overlay.is_none());
        let _ = harness.key(KeyCode::Enter);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert_eq!(harness.app().world.effect_revision, 1);
    }
    #[test]
    fn clone_picker_is_reachable_from_the_real_action_row() {
        let mut harness = app(Scenario::RustDirty);
        assert!(harness.tab_to(home::QUERY));
        let _ = harness.type_str("gh repo clone");
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        let _ = harness.key(KeyCode::Enter);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(matches!(harness.app().overlay, Some(Overlay::Clone(_))));
        let _ = harness.key(KeyCode::Enter);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(matches!(harness.app().overlay, Some(Overlay::Dialog(_))));
        assert!(harness.text().contains("Primary branch"));
    }
    #[test]
    fn action_picker_refuses_memory_changes_after_context_moves() {
        let mut harness = app(Scenario::DockerCleanup);
        assert!(harness.tab_to(home::QUERY));
        let _ = harness.type_str("docker system prune");
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        let _ = harness.ctrl('o');
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(matches!(harness.app().overlay, Some(Overlay::Actions(_))));
        let pins = harness.app().world.memory.pins.clone();
        harness.app_mut().world.cwd = "/different-folder".into();
        let _ = harness.type_str("Pin here");
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        let _ = harness.key(KeyCode::Enter);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(harness.app().overlay.is_none());
        assert_eq!(harness.app().world.memory.pins, pins);
        assert!(harness.app().status.as_deref().unwrap().contains("changed"));
    }
    #[test]
    fn ssh_preview_shows_literal_resolution_and_refuses_changed_target() {
        for changed in [false, true] {
            let mut harness = app(Scenario::RemoteHost);
            let host = harness.app().world.ssh[0].clone();
            assert!(harness.tab_to(home::QUERY));
            let _ = harness.type_str(&format!("ssh {}", host.alias));
            assert!(
                harness.diagnostics().is_empty(),
                "{:?}",
                harness.diagnostics()
            );
            let _ = harness.key(KeyCode::Enter);
            assert!(
                harness.diagnostics().is_empty(),
                "{:?}",
                harness.diagnostics()
            );
            assert!(matches!(harness.app().overlay, Some(Overlay::Dialog(_))));
            let text = harness.text();
            for expected in [
                &host.host_name,
                &host.user,
                &host.chain(),
                "Host key policy",
                "Multiplexing",
            ] {
                assert!(text.contains(expected), "missing {expected}");
            }
            if changed {
                harness.app_mut().world.ssh[0].host_name = "different.example".into();
            }
            let _ = harness.click_id(Dialog::new(dialogs::DIALOG).action_id(1));
            assert!(
                harness.diagnostics().is_empty(),
                "{:?}",
                harness.diagnostics()
            );
            let status = harness.app().status.as_deref().unwrap();
            if changed {
                assert!(status.contains("SSH target changed"));
            } else {
                assert!(status.contains(&format!("Would run: ssh {}", host.alias)));
            }
            assert_eq!(harness.app().world.effect_revision, 0);
        }
    }
    #[test]
    fn routes_and_gate_deliver_owner_lifecycle() {
        let mut harness = app(Scenario::DockerCleanup);
        assert!(harness.diagnostics().is_empty());
        assert!(harness.tab_to(home::QUERY));
        let _ = harness.type_str("docker system prune");
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        let _ = harness.key(KeyCode::Enter);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(
            harness.diagnostics().is_empty(),
            "plan {:?}",
            harness.diagnostics()
        );
        assert!(harness.tab_to(plan::STEPS));
        let _ = harness.key(KeyCode::Enter);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(
            harness.diagnostics().is_empty(),
            "gate {:?}",
            harness.diagnostics()
        );
        let _ = harness.key(KeyCode::Esc);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(
            harness.diagnostics().is_empty(),
            "cancel {:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn activity_switching_and_home_keep_viewport_lifecycle_clean() {
        let mut harness = app(Scenario::ActivitiesMulti);
        for _ in 0..3 {
            let _ = harness.ctrl('a');
            assert!(matches!(harness.app().route, Route::Activity));
            assert!(
                harness.diagnostics().is_empty(),
                "{:?}",
                harness.diagnostics()
            );
        }
        let _ = harness.key(KeyCode::Char('0'));
        assert!(matches!(harness.app().route, Route::Home));
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }

    #[test]
    fn coalesced_deadlines_preserve_fixed_steps_and_pause() {
        for motion in [Motion::Full, Motion::Reduced, Motion::Paused] {
            let mut harness = harness(
                App::for_scenario_without_history(Scenario::FirstUse, motion, 0),
                Theme::junie(),
                120,
                40,
            );
            harness.ticks(20);
            assert_eq!(harness.app().world.now_ms(), 0);
            let _ = harness.advance(std::time::Duration::from_millis(1_800));
            let step = if motion == Motion::Paused { 0 } else { 80 };
            assert_eq!(harness.app().world.now_ms(), step);
            harness.ticks(20);
            harness.draw();
            assert_eq!(harness.app().world.now_ms(), step);
            let _ = harness.advance(std::time::Duration::from_millis(80));
            assert_eq!(harness.app().world.now_ms(), step * 2);
            assert!(
                harness.diagnostics().is_empty(),
                "{:?}",
                harness.diagnostics()
            );
        }
    }
    #[test]
    fn pointer_feedback_changes_cadence_from_last_wake_and_expires_in_fixture_time() {
        for input_at in [5_u64, 100] {
            let mut harness = harness(
                App::for_scenario_without_history(Scenario::HardCases, Motion::Full, 4_000),
                Theme::junie(),
                120,
                40,
            );
            let _ = harness.advance(std::time::Duration::from_millis(input_at));
            let _ = harness.click_id(BRAND);
            assert_eq!(harness.app().fixture_time_ms(), 4_000);
            let _ = harness.advance(std::time::Duration::from_millis(
                80_u64.saturating_sub(input_at),
            ));
            assert_eq!(harness.app().fixture_time_ms(), 4_080);
            assert_eq!(
                harness.activation_feedback().unwrap().remaining,
                std::time::Duration::from_millis(60)
            );
            harness.ticks(5);
            let _ = harness.resize(121, 40);
            assert_eq!(harness.app().fixture_time_ms(), 4_080);
            let _ = harness.advance(std::time::Duration::from_millis(80));
            assert_eq!(harness.app().fixture_time_ms(), 4_160);
            assert!(harness.activation_feedback().is_none());
            let _ = harness.advance(std::time::Duration::from_millis(199));
            assert_eq!(harness.app().fixture_time_ms(), 4_160);
            let _ = harness.advance(std::time::Duration::from_millis(1));
            assert_eq!(harness.app().fixture_time_ms(), 4_360);
            assert!(
                harness.diagnostics().is_empty(),
                "{:?}",
                harness.diagnostics()
            );
        }
    }
    #[test]
    fn delayed_pointer_feedback_coalesces_and_paused_feedback_keeps_fixture_epoch() {
        for motion in [Motion::Full, Motion::Paused] {
            let mut harness = harness(
                App::for_scenario_without_history(Scenario::HardCases, motion, 4_000),
                Theme::junie(),
                120,
                40,
            );
            let _ = harness.advance(std::time::Duration::from_millis(5));
            let _ = harness.click_id(BRAND);
            let _ = harness.advance(std::time::Duration::from_millis(1_800));
            let step = if motion == Motion::Paused { 0 } else { 80 };
            assert_eq!(harness.app().fixture_time_ms(), 4_000 + step);
            assert_eq!(
                harness.activation_feedback().unwrap().remaining,
                std::time::Duration::from_millis(if motion == Motion::Paused { 140 } else { 60 })
            );
            harness.ticks(10);
            assert_eq!(harness.app().fixture_time_ms(), 4_000 + step);
            assert!(
                harness.diagnostics().is_empty(),
                "{:?}",
                harness.diagnostics()
            );
        }
    }
    #[test]
    fn feedback_clock_mismatch_and_backwards_epoch_refuse_all_fixture_progress() {
        let snapshot = |app: &App| {
            (
                app.fixture_time_ms(),
                app.effect_revision(),
                app.world
                    .discovery
                    .iter()
                    .map(|mark| (mark.domain, mark.at_ms, mark.fails, mark.done, mark.failed))
                    .collect::<Vec<_>>(),
                catalogue::catalogue(&app.world),
            )
        };
        for clock in [
            termrock::FeedbackClock::Elapsed,
            termrock::FeedbackClock::Simulation {
                initial: termrock::SimulationMoment::from_millis(5_000),
            },
        ] {
            let app = App::for_scenario_without_history(Scenario::HardCases, Motion::Full, 4_000);
            let expected = snapshot(&app);
            let mut harness = Harness::new_with_feedback_clock(app, Theme::junie(), 120, 40, clock);
            assert_eq!(snapshot(harness.app()), expected);
            assert!(!harness.app().world.clock.running);
            assert_eq!(harness.app().motion, Motion::Paused);
            assert!(harness.text().contains("feedback clock mismatch"));
            let _ = harness.advance(std::time::Duration::from_millis(1_800));
            assert_eq!(snapshot(harness.app()), expected);
            assert!(
                harness.diagnostics().is_empty(),
                "{:?}",
                harness.diagnostics()
            );
        }
    }

    #[test]
    fn modal_status_ages_only_with_admitted_virtual_steps() {
        let mut harness = harness(
            App::for_scenario_without_history(Scenario::FirstUse, Motion::Full, 4_000),
            Theme::junie(),
            120,
            40,
        );
        let _ = harness.ctrl('s');
        let _ = harness.key(KeyCode::F(1));
        for _ in 0..24 {
            let _ = harness.advance(std::time::Duration::from_millis(200));
            assert!(harness.app().status.is_some());
        }
        let _ = harness.advance(std::time::Duration::from_millis(200));
        assert_eq!(harness.app().world.now_ms(), 9_000);
        assert!(harness.app().status.is_none());
        assert!(harness.app().overlay.is_some());
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }

    #[test]
    fn early_input_and_unrelated_ticks_cannot_admit_simulation_time() {
        let mut harness = harness(
            App::for_scenario_without_history(Scenario::FirstUse, Motion::Full, 0),
            Theme::junie(),
            120,
            40,
        );
        for _ in 0..10 {
            let _ = harness.advance(std::time::Duration::from_micros(600));
            let _ = harness.tick();
        }
        let _ = harness.key(KeyCode::F(1));
        assert_eq!(harness.app().world.now_ms(), 0);
        let _ = harness.advance(std::time::Duration::from_millis(74));
        assert_eq!(harness.app().world.now_ms(), 80);
    }
    #[test]
    fn footer_status_survives_focused_hints_and_edit_badge_uses_top_context() {
        let mut harness = app(Scenario::DockerCleanup);
        assert!(harness.tab_to(home::QUERY));
        let _ = harness.type_str("docker system prune");
        let _ = harness.ctrl('s');
        assert!(harness.row(39).contains("EDIT"));
        assert!(harness.row(39).contains("Scope: here"));
        let _ = harness.key(KeyCode::F(1));
        assert!(!harness.row(39).contains("EDIT"));
        assert!(harness.row(39).contains("Scope: here"));
        let _ = harness.key(KeyCode::Esc);
        assert!(harness.row(39).contains("EDIT"));
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );

        let mut harness = app(Scenario::DockerCleanup);
        assert!(harness.tab_to(home::QUERY));
        let _ = harness.type_str("docker system prune");
        let _ = harness.key(KeyCode::Enter);
        assert!(harness.tab_to(plan::STEPS));
        let _ = harness.key(KeyCode::Enter);
        assert!(harness.row(39).contains("EDIT"));
        let _ = harness.key(KeyCode::Enter);
        assert!(!harness.row(39).contains("EDIT"));
        assert_eq!(harness.app().world.effect_revision, 0);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
}

#[cfg(test)]
mod historical_tests;

#[cfg(test)]
mod independent_draw_review {
    use super::*;

    struct Borrowed<'a>(&'a App);

    impl termrock::App for Borrowed<'_> {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            termrock::App::draw(self.0, ui);
        }
    }

    fn fingerprint(a: &App) -> String {
        format!(
            "{:?}{:?}{:?}{:?}{:?}{:?}",
            (
                a.world.effect_revision,
                a.world.now_ms(),
                &a.world.cwd,
                &a.world.discovery
            ),
            (&a.world.mise, &a.world.git, &a.world.ssh, &a.world.github),
            (&a.world.docker, &a.world.pg, &a.world.disk, &a.world.debian),
            (&a.world.memory, &a.world.activities),
            (&a.status, a.quit, a.activities.current_id()),
            crate::sim::catalogue::catalogue(&a.world)
                .iter()
                .map(|action| action.id.clone())
                .collect::<Vec<_>>(),
        )
    }

    #[test]
    fn all_scenarios_available_routes_repeat_full_cells_without_model_effects() {
        let mut count = 0;
        for scenario in Scenario::ALL {
            for route in [
                Route::Home,
                Route::Plan,
                Route::Activity,
                Route::Disk,
                Route::Files,
                Route::Cleanup,
                Route::Review,
                Route::Snapshot,
            ] {
                let mut app = App::for_scenario_without_history(scenario, Motion::Paused, 4000);
                if route == Route::Plan {
                    let plan = crate::sim::catalogue::catalogue(&app.world)
                        .iter()
                        .find_map(|action| crate::sim::plans::plan_for(&app.world, &action.id));
                    let Some(plan) = plan else { continue };
                    app.plan = Some(PlanState::new(plan));
                }
                if let (Route::Activity, Some(activity)) = (route, app.world.activities.first()) {
                    app.activities.show(activity.id);
                }
                app.route = route;
                let before = fingerprint(&app);
                for (cols, rows) in [(71, 19), (72, 20), (120, 40)] {
                    for color in [
                        termrock::ColorLevel::TrueColor,
                        termrock::ColorLevel::Ansi256,
                        termrock::ColorLevel::Ansi16,
                        termrock::ColorLevel::Mono,
                    ] {
                        let mut scene = termrock_test_support::Harness::new(
                            Borrowed(&app),
                            termrock::Theme::junie().downgrade(color),
                            cols,
                            rows,
                        );
                        scene.draw();
                        let cells = scene.buffer().clone();
                        let cursor = scene.runtime().cursor_position();
                        scene.draw();
                        assert_eq!(
                            scene.buffer(),
                            &cells,
                            "{scenario:?} route {} {cols}x{rows}",
                            route as u8
                        );
                        assert_eq!(
                            scene.runtime().cursor_position(),
                            cursor,
                            "{scenario:?} route {} {cols}x{rows}",
                            route as u8
                        );
                        assert_eq!(
                            fingerprint(&app),
                            before,
                            "{scenario:?} route {}",
                            route as u8
                        );
                        count += 1;
                    }
                }
            }
        }
        debug_assert!(count > 0, "no cases drawn");
        let _ = count;
    }
}
