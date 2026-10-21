//! Shared debug-info card, plus the reduced-color recolor.
//!
//! The info card paints through the shared `termrock` overlays info card.
//! Empty padding keeps the dimmed cockpit ink. This module only remaps those
//! inks for 256, 16, and mono until the shared capability ladders reproduce
//! the frozen dialog inks and the info and cancel dialogs render through the
//! real layer flow at every level.
//!
//! The 256-color remap uses the shared theme ladder
//! (`termrock::theme::downgrade_color`). The 16-color and mono remap below is
//! still an app-local shim: the theme crate has no per-ink ladder that follows
//! the token ramp, so the accent would land on gray instead of the bright
//! green the frozen frames carry.

use termrock::{Color, ColorLevel, Rect, Ui};

pub(super) fn paint_info(ui: &mut Ui<'_>, facts: &[termrock::overlays::info_card::Fact<'_>]) {
    termrock::overlays::info_card::paint(ui, facts);
    recolor(ui, 66, 11, 0);
}

pub(super) fn paint_cancel(ui: &mut Ui<'_>) {
    termrock::overlays::cancel_card::paint(
        ui,
        "Cancel the launch?",
        "The pipeline stops at its current stage and the partially prepared instance is marked failed setup. Nothing is attached.",
        "Keep launching",
        "Cancel launch",
    );
    recolor(ui, 54, 11, 1);
}

fn recolor(ui: &mut Ui<'_>, width: u16, height: u16, y_bias: u16) {
    let screen = ui.full();
    let origin_x = screen
        .x
        .saturating_add(screen.width.saturating_sub(width) / 2);
    let origin_y = screen
        .y
        .saturating_add(screen.height.saturating_sub(height) / 2)
        .saturating_add(y_bias);
    // The block is truecolor. A 256-color theme swaps those inks for the
    // baseline index table after the dim, and leaves indexed cockpit cells.
    if ui.theme_ref().capability.color == ColorLevel::Ansi256 {
        let card = Rect::new(origin_x, origin_y, width, height);
        ui.recolor_rgb(card, ansi256_ink);
        // Backdrop maps rain index 238 (`#444444`) to overlay 235 (`#262626`).
        // On black that ink is the ghost ramp, index 238.
        ui.recolor_fg(screen, |fg, bg| {
            let rain = matches!(fg, Color::Indexed(235) | Color::Rgb(0x26, 0x26, 0x26));
            let plane = on_black(bg) || matches!(bg, Color::Indexed(233));
            if rain && plane {
                Some(Color::Indexed(238))
            } else {
                None
            }
        });
    }
    // 16-color and mono dim the whole card, including its title. `NO_COLOR`
    // keeps the stripped inks and only sets `dim`.
    if matches!(
        ui.theme_ref().capability.color,
        ColorLevel::Ansi16 | ColorLevel::Mono
    ) {
        let card = Rect::new(origin_x, origin_y, width, height);
        if std::env::var_os("NO_COLOR").is_none() {
            let ansi16 = ui.theme_ref().capability.color == ColorLevel::Ansi16;
            ui.recolor_pair(card, move |fg, bg| reduced_pair(fg, bg, ansi16));
        }
        ui.add_dim(card);
    }
}

fn reduced_pair(fg: Color, bg: Color, ansi16: bool) -> (Color, Color) {
    let button = matches!(
        bg,
        Color::Rgb(0x27, 0x27, 0x2a) | Color::Rgb(0x3f, 0x3f, 0x46)
    );
    let bg = if button {
        Color::DarkGray
    } else if matches!(bg, Color::Rgb(..)) {
        Color::Black
    } else {
        bg
    };
    let fg = if button {
        if matches!(fg, Color::Rgb(0x48, 0xe0, 0x54)) {
            Color::White
        } else if matches!(
            fg,
            Color::Rgb(0x18, 0x18, 0x1b) | Color::Rgb(0x27, 0x27, 0x2a) | Color::Rgb(0, 0, 0)
        ) {
            Color::DarkGray
        } else {
            Color::Black
        }
    } else if matches!(fg, Color::Rgb(0xff, 0xff, 0xff) | Color::White) {
        Color::White
    } else if matches!(fg, Color::Rgb(0x48, 0xe0, 0x54)) {
        if ansi16 {
            Color::Indexed(10)
        } else {
            Color::White
        }
    } else if matches!(
        fg,
        Color::Rgb(0x00, 0x00, 0x00)
            | Color::Black
            | Color::Rgb(0x18, 0x18, 0x1b)
            | Color::Rgb(0x27, 0x27, 0x2a)
    ) {
        Color::Black
    } else if matches!(fg, Color::Rgb(..)) {
        Color::Gray
    } else {
        fg
    };
    (fg, bg)
}

fn on_black(color: Color) -> bool {
    matches!(
        color,
        Color::Black | Color::Indexed(0) | Color::Indexed(16) | Color::Rgb(0, 0, 0)
    )
}

/// One truecolor ink → the 256-color ink of the shared ladder.
///
/// The downgrade comes from the theme crate, not from an app-local table.
fn ansi256_ink(color: Color) -> Color {
    termrock::theme::downgrade_color(color, ColorLevel::Ansi256)
}
