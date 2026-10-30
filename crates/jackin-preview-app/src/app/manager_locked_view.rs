//! Locked hard-launch manager chrome: the paused capture state paints the
//! session header and hint bar through the shared feedback painter.

use termrock::feedback::{SessionChrome, SessionHint, SessionMenu, paint_session_chrome};
use termrock::{ColorLevel, Rect, Ui};

use crate::scenario::{Motion, Scenario};
use crate::screens::manager::ManagerRowKey;

use super::{App, Route};

/// The captured launch keeps the manager on the approved chrome while the
/// picker, menus, help, and every transient dialog stay closed.
pub(super) fn approved(app: &App, ui: &Ui<'_>, screen: Rect) -> bool {
    let theme = ui.theme_ref();
    screen.width == 120
        && screen.height == 40
        && theme.capability.color == ColorLevel::TrueColor
        && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
        && app.route == Route::Manager
        && app.world.scenario == Scenario::HardCases
        && app.motion == Motion::Paused
        && !app.manager_file_menu
        && !app.manager_inspect
        && !app.help_open
        && app.status.is_none()
        && !app.quit_open
        && !app.cancel_open
        && !app.about_open
        && app.launch.is_none()
        && app.picker_mode.is_none()
        && !app.launch_picker_open
        && app.agent_options.is_empty()
        && app.active_instance.is_none()
        && !app.manager.detail_open()
        && app.manager.selected_row() == &ManagerRowKey::Workspace(12)
}

pub(super) fn paint(app: &App, ui: &mut Ui<'_>, screen: Rect) {
    if !approved(app, ui, screen) {
        return;
    }
    // Approved frame hides the cursor. This path must not call set_cursor
    // or set_bar_cursor, and it must not write cursor bytes.
    let menus = [
        SessionMenu {
            label: "File",
            focused: true,
        },
        SessionMenu {
            label: "Edit",
            focused: false,
        },
        SessionMenu {
            label: "View",
            focused: false,
        },
        SessionMenu {
            label: "Session",
            focused: false,
        },
        SessionMenu {
            label: "Help",
            focused: false,
        },
    ];
    let hints = [
        SessionHint {
            key: "Ctrl+B",
            action: "Pane commands…",
        },
        SessionHint {
            key: "F10",
            action: "Menu",
        },
        SessionHint {
            key: "Ctrl+\\",
            action: "Palette",
        },
        SessionHint {
            key: "Alt+Shift+↑↓←→",
            action: "Resize",
        },
    ];
    paint_session_chrome(
        ui,
        screen,
        &SessionChrome {
            brand: "jackin❯",
            menus: &menus,
            crumb: "› backend",
            instance: "jackin-data-pipeline-e0e0",
            instances: "3 instances",
            branch: "main",
            state: "clean",
            account: "no account · shell",
            hints: &hints,
            overflow: "…",
            attached: "Attached to  · tabs and panes restored",
        },
    );
}
