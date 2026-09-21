//! Direct-lane launch through the same constructors the production CLI uses.

use junie_tui::{ColorLevel, KeyCode, Theme};
use junie_tui_testing::Harness;
use tablepro_app::TableProApp;

use crate::input::{ctrl, enter, esc, type_text};
use crate::scenario::{ColorProfile, FIXTURE_LOCAL_POSTGRESQL, FIXTURE_PRODUCTION};

/// Default oracle theme resolved for `level`, matching `run_with`'s Junie choice.
#[must_use]
pub fn theme_for(level: ColorLevel) -> Theme {
    Theme::junie().for_level(level)
}

fn harness(
    app: TableProApp,
    width: u16,
    height: u16,
    profile: ColorProfile,
) -> Harness<TableProApp> {
    let level = profile.color_level();
    Harness::new(app, theme_for(level), width, height).with_color(level)
}

/// Preset C: no CLI arguments, so `run_with(theme, None)` → `TableProApp::default()`.
///
/// This is the production Connections landing path. It does not call
/// `set_surface`.
#[must_use]
pub fn launch_preset_c(width: u16, height: u16, profile: ColorProfile) -> Harness<TableProApp> {
    harness(TableProApp::default(), width, height, profile)
}

/// Apply `--connect NAME` the same way `run_with` does: fixture lookup then `connect`.
pub fn connect_fixture(app: &mut TableProApp, index: usize) -> bool {
    app.connect(index)
}

/// Preset W: `--connect Production`.
#[must_use]
pub fn launch_preset_w(width: u16, height: u16, profile: ColorProfile) -> Harness<TableProApp> {
    let mut app = TableProApp::default();
    let _ = connect_fixture(&mut app, FIXTURE_PRODUCTION);
    harness(app, width, height, profile)
}

/// Preset L: `--connect "Local PostgreSQL"`.
#[must_use]
pub fn launch_preset_l(width: u16, height: u16, profile: ColorProfile) -> Harness<TableProApp> {
    let mut app = TableProApp::default();
    let _ = connect_fixture(&mut app, FIXTURE_LOCAL_POSTGRESQL);
    harness(app, width, height, profile)
}

/// Preset T: W then Ctrl+O, literal `orders`, Enter.
#[must_use]
pub fn launch_preset_t(width: u16, height: u16, profile: ColorProfile) -> Harness<TableProApp> {
    let mut harness = launch_preset_w(width, height, profile);
    ctrl(&mut harness, 'o');
    type_text(&mut harness, "orders");
    enter(&mut harness);
    harness
}

/// Preset Q(`sql`): W, Ctrl+T, `i`, literal SQL, Esc.
#[must_use]
pub fn launch_preset_q(
    sql: &str,
    width: u16,
    height: u16,
    profile: ColorProfile,
) -> Harness<TableProApp> {
    query_from_workbench(launch_preset_w(width, height, profile), sql)
}

/// Preset QL(`sql`): L, Ctrl+T, `i`, literal SQL, Esc.
#[must_use]
pub fn launch_preset_ql(
    sql: &str,
    width: u16,
    height: u16,
    profile: ColorProfile,
) -> Harness<TableProApp> {
    query_from_workbench(launch_preset_l(width, height, profile), sql)
}

fn query_from_workbench(mut harness: Harness<TableProApp>, sql: &str) -> Harness<TableProApp> {
    ctrl(&mut harness, 't');
    let _ = harness.key(KeyCode::Char('i'));
    type_text(&mut harness, sql);
    esc(&mut harness);
    harness
}
