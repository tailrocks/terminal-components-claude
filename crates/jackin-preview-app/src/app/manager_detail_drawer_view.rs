//! Manager detail drawer: the paused 120×40 frame after Tab, with focus
//! on the detail title, the inactive workspace list, and the drawer hints.

use termrock::author::PaintStyle;
use termrock::{ColorLevel, FrameRead, Modifier, Rect, StateFlags, Ui};

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
    let legacy = ui.theme_ref().capability.color == ColorLevel::TrueColor
        && ui.theme_ref().color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54);
    app.paint_pinned_split(ui, screen, termrock::feedback::SplitMode::Saved, true);
    if !legacy {
        return;
    }
    let patch = |ui: &mut Ui<'_>, x: u16, y: u16, text: &str, fg: u32, bg: u32, bold: bool| {
        let mut style = PaintStyle::new()
            .fg(termrock::Color::from_u32(fg))
            .bg(termrock::Color::from_u32(bg));
        style = if bold {
            style.add_modifier(Modifier::BOLD)
        } else {
            style.remove_modifier(Modifier::BOLD)
        };
        let width = u16::try_from(text.chars().count()).unwrap_or(0);
        if y < screen.bottom() && x < screen.right() && width > 0 {
            ui.paint_str(Rect::new(x, y, width, 1), text, style);
        }
    };
    let rule = 0x3a_3a_3a;
    let black = 0x00_00_00;
    patch(ui, 1, 2, "╭─", rule, black, false);
    patch(ui, 3, 2, " Workspaces ", 0xb3_b3_b3, black, false);
    patch(ui, 15, 2, "──────────", rule, black, false);
    patch(ui, 36, 2, "─╮", rule, black, false);
    patch(ui, 41, 2, "▎", 0x48_e0_54, 0x11_11_11, false);
    patch(
        ui,
        42,
        2,
        "Current directory · payments-platform",
        0xff_ff_ff,
        0x11_11_11,
        true,
    );
    patch(ui, 3, 3, " ", 0x17_3f_1c, 0x17_3f_1c, true);
    patch(ui, 5, 3, " ", 0xff_ff_ff, 0x17_3f_1c, false);
    for y in 3..37 {
        patch(ui, 1, y, "│", rule, black, false);
        patch(ui, 37, y, "│", rule, black, false);
    }
    patch(
        ui,
        1,
        37,
        "╰───────────────────────────────────╯",
        rule,
        black,
        false,
    );
    let white = 0xff_ff_ff;
    let hint = 0x8a_8a_8a;
    patch(ui, 1, 39, "↑↓", white, black, true);
    patch(ui, 3, 39, " ", white, black, false);
    patch(ui, 4, 39, "Move", hint, black, false);
    patch(ui, 8, 39, "  ", white, black, false);
    patch(ui, 10, 39, "Tab", white, black, true);
    patch(ui, 14, 39, "Actions", hint, black, false);
    patch(ui, 22, 39, " ", white, black, false);
    patch(ui, 23, 39, "Esc", white, black, true);
    patch(ui, 26, 39, " ", white, black, false);
    patch(ui, 27, 39, "Back", hint, black, false);
    for (x, text) in [
        (31, "  "),
        (34, "       "),
        (43, " "),
        (45, "        "),
        (55, " "),
        (57, "     "),
        (64, " "),
        (66, "        "),
        (76, " "),
        (78, "    "),
        (84, " "),
        (86, "    "),
    ] {
        patch(ui, x, 39, text, white, black, false);
    }
}
