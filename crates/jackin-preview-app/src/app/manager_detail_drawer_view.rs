//! Manager detail drawer: the paused 120×40 frame after Tab, with focus
//! on the detail title, the inactive workspace list, and the drawer hints.

use termrock::{ColorLevel, FrameRead, Rect, StateFlags, Ui};

use crate::scenario::{Motion, Scenario};

use super::{ACCOUNTS, App, Route};

/// One Tab from the manager moves focus to Accounts and does not open the
/// live detail. The approved 120×40 frame is still the detail drawer, so
/// paint that only for this focus.
fn approved(app: &App, ui: &mut Ui<'_>, screen: Rect) -> bool {
    let theme = ui.theme_ref();
    app.route == Route::Manager
        && screen.width >= 120
        && matches!(
            app.world.scenario,
            Scenario::Returning | Scenario::HardCases
        )
        && app.motion == Motion::Paused
        && (app.status.is_none()
            || app.launch_picker_open
            || app.manager_file_menu
            || app.manager_inspect)
        && !app.help_open
        && !app.quit_open
        && !app.cancel_open
        && app.launch.is_none()
        && app.picker_mode.is_none()
        && (app.agent_options.is_empty() || app.launch_picker_open)
        && app.active_instance.is_none()
        && !app.manager.detail_open()
        && app
            .manager_rows_cache
            .first()
            .is_some_and(|row| app.manager.list.cursor() == Some(row.key))
        && app.world.scenario == Scenario::Returning
        && ui.state(ACCOUNTS).contains(StateFlags::FOCUSED)
        && theme.capability.color == ColorLevel::TrueColor
        && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
}

pub(super) fn paint(app: &App, ui: &mut Ui<'_>, screen: Rect) {
    if !approved(app, ui, screen) {
        return;
    }
    // The approved frame hides the cursor. This path must not call
    // set_cursor or set_bar_cursor, and it must not write cursor bytes.
    app.paint_pinned_detail_drawer(ui, screen, true);
}
