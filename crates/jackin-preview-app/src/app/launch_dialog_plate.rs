//! Shared debug-info card, plus the reduced-color recolor.
//!
//! The info card paints through the shared `termrock` overlays info card.
//! Empty padding keeps the dimmed cockpit ink. This module only remaps those
//! inks for 256, 16, and mono until the shared capability ladders reproduce
//! the frozen dialog inks and the info and cancel dialogs render through the
//! real layer flow at every level.
//!
//! The 256-color remap uses the shared theme ladder
//! (`termrock::theme::downgrade_color`). The 16-color remap uses the shared
//! pair ladder (`termrock::theme::downgrade_pair`), and mono prefers the
//! frozen launch pair table (`termrock::theme::legacy_mono`) with the pair
//! ladder as fallback.

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
    // keeps the stripped inks and only sets `dim`. Mono prefers the frozen
    // launch pair table; anything it does not list falls back to the pair
    // ladder. Only colors are remapped here: no card pair needs the table's
    // underline/reverse marks, and `add_dim` below owns the dim.
    if matches!(
        ui.theme_ref().capability.color,
        ColorLevel::Ansi16 | ColorLevel::Mono
    ) {
        let card = Rect::new(origin_x, origin_y, width, height);
        if std::env::var_os("NO_COLOR").is_none() {
            let mono = ui.theme_ref().capability.color == ColorLevel::Mono;
            if mono {
                ui.recolor_pair(card, |fg, bg| {
                    // Two card pairs miss the frozen table and land on
                    // Black-on-Black through the pair ladder, which is
                    // unreadable (danger label) or invisible (border). Keep
                    // the shipped readable inks for exactly these pairs.
                    if matches!(
                        (fg, bg),
                        (Color::Rgb(0x4d, 0x4d, 0x4d), Color::Rgb(0x18, 0x18, 0x1b))
                    ) {
                        return (Color::Gray, Color::Black);
                    }
                    if matches!(
                        (fg, bg),
                        (Color::Rgb(0xff, 0x6b, 0x6b), Color::Rgb(0x27, 0x27, 0x2a))
                    ) {
                        return (Color::Gray, Color::DarkGray);
                    }
                    termrock::theme::legacy_mono(fg, bg, false)
                        .map(|ink| (ink.fg, ink.bg))
                        .unwrap_or_else(|| {
                            termrock::theme::downgrade_pair(fg, bg, ColorLevel::Mono)
                        })
                });
            } else {
                ui.recolor_pair(card, |fg, bg| {
                    termrock::theme::downgrade_pair(fg, bg, ColorLevel::Ansi16)
                });
            }
        }
        ui.add_dim(card);
    }
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
    if let Color::Rgb(red, green, blue) = color {
        match (red, green, blue) {
            // The card border and the ghost ramp keep their corpus index: the
            // approved frames carry 242 and 238, and the shared ladder lands
            // on 239 and 235.
            (0x4d, 0x4d, 0x4d) => return Color::Indexed(242),
            (0x26, 0x26, 0x26) => return Color::Indexed(238),
            _ => {}
        }
    }
    termrock::theme::downgrade_color(color, ColorLevel::Ansi256)
}
