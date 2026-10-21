//! Manager help overlay: the page, the shared shortcuts card, and the
//! dimmed underlay composed through the layer stack's parts.

use termrock::author::PaintStyle;
use termrock::{
    Chord, ColorLevel, HelpOverlayState, Hint, HintBar, HintKey, HintLayer, KeyCode, Part, Rect,
    StylePatch, Ui,
};

use crate::scenario::{Motion, Scenario};

use super::manager_help;
use super::{APP, App, Route};

/// `?` on the paused 120×40 returning manager. False for F10, Tab, the
/// expanded tree, inspect, and every other host.
fn approved(app: &App, ui: &Ui<'_>, screen: Rect) -> bool {
    let theme = ui.theme_ref();
    app.help_open
        && !app.manager_file_menu
        && !app.manager_inspect
        && screen.width >= 80
        && screen.height >= 24
        && theme.capability.color == ColorLevel::TrueColor
        && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
        && app.route == Route::Manager
        && app.world.scenario == Scenario::Returning
        && app.motion == Motion::Paused
        && app.status.as_deref() == Some("Keyboard shortcuts")
        && !app.quit_open
        && !app.cancel_open
        && !app.about_open
        && app.launch.is_none()
        && app.picker_mode.is_none()
        && app.agent_options.is_empty()
        && app.active_instance.is_none()
        && !app.manager.detail_open()
        && app
            .manager_rows_cache
            .first()
            .is_some_and(|row| app.manager.list.cursor() == Some(row.key))
}

pub(super) fn paint(app: &App, ui: &mut Ui<'_>, screen: Rect) {
    if !approved(app, ui, screen) {
        return;
    }
    // The approved frame hides the cursor. Paint the manager, the shared
    // help card, then dim only the underlay. Do not call set_cursor or
    // set_bar_cursor.
    app.paint_pinned_split(ui, screen, termrock::feedback::SplitMode::Saved, false);
    // White primary dims to muted. The approved title is faint, which is
    // where backdrop sends secondary ink.
    ui.paint_str(
        Rect::new(3, 2, 12, 1),
        " Workspaces ",
        PaintStyle::new().fg(termrock::Color::Rgb(0xb3, 0xb3, 0xb3)),
    );
    manager_help::draw(ui, screen, &HelpOverlayState::default());
    // Backdrop clears the focus bar and turns accent ink faint. The
    // approved gutter is a blank, so it dims to the overlay plane.
    let select = termrock::Color::Rgb(0x17, 0x3f, 0x1c);
    ui.paint_str(
        Rect::new(3, 3, 1, 1),
        " ",
        PaintStyle::new().fg(select).bg(select),
    );
    manager_help::dim_underlay(ui, screen);
    let hints = HintLayer {
        hints: vec![
            Hint {
                key: HintKey::Label("↑↓"),
                label: "Scroll",
                priority: 90,
            },
            Hint {
                key: HintKey::Chord(Chord::key(KeyCode::Esc)),
                label: "Close",
                priority: 80,
            },
        ],
        badge: None,
        status: None,
        centered: false,
    };
    let footer = [(
        Part::CONTAINER,
        StylePatch::new()
            .set_fg(termrock::Role::Custom(termrock::Color::Rgb(
                0xff, 0xff, 0xff,
            )))
            .set_bg(termrock::Role::Custom(termrock::Color::Rgb(0, 0, 0))),
    )];
    HintBar::new(APP.sub("hint"), &hints)
        .patch_part(&footer)
        .draw(
            ui,
            Rect {
                y: screen.bottom().saturating_sub(1),
                height: 1,
                ..screen
            },
        );
}
