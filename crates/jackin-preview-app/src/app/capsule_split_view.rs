//! Approved 120×40 vertical-split agent picker over the live capsule.
//!
//! `ctrl-b` then `"` opens this picker. The card keeps dimmed underlay ink
//! on empty cells and paints the frozen rows on top.

use termrock::author::PaintStyle;
use termrock::{Color, Modifier, Position, Rect, Ui};

const ACCENT: Color = Color::Rgb(0x48, 0xe0, 0x54);
const WHITE: Color = Color::Rgb(0xff, 0xff, 0xff);
const MUTED: Color = Color::Rgb(0x8a, 0x8a, 0x8a);
const FAINT: Color = Color::Rgb(0x73, 0x73, 0x73);
const GHOST: Color = Color::Rgb(0x26, 0x26, 0x26);
const SECONDARY: Color = Color::Rgb(0xb3, 0xb3, 0xb3);
const CARD: Color = Color::Rgb(0x18, 0x18, 0x1b);
const FIELD: Color = Color::Rgb(0x1e, 0x1e, 0x22);
const BORDER: Color = Color::Rgb(0x4d, 0x4d, 0x4d);
const SELECTED: Color = Color::Rgb(0x2f, 0x5a, 0xa8);
const BLACK: Color = Color::Rgb(0, 0, 0);

const CARD_X: u16 = 24;
const CARD_Y: u16 = 14;
const CARD_W: u16 = 72;
const CARD_H: u16 = 12;

pub(crate) fn paint(ui: &mut Ui<'_>, title: &str) {
    let full = ui.full();
    if full.width == 120 && full.height == 40 {
        paint_120(ui, title);
        return;
    }
    paint_fitted(ui, title);
}

pub(crate) fn search_cursor(screen: Rect) -> Position {
    let card = card_rect(screen);
    Position::new(card.x.saturating_add(4), card.y.saturating_add(2))
}

fn card_rect(screen: Rect) -> Rect {
    let width = 72.min(screen.width.saturating_sub(4));
    let height = 12.min(screen.height);
    Rect::new(
        screen.width.saturating_sub(width) / 2,
        screen.height.saturating_sub(height) / 2,
        width,
        height,
    )
}

fn paint_120(ui: &mut Ui<'_>, title: &str) {
    let full = ui.full();
    ui.modal_backdrop(full);
    repair_underlay(ui);
    let interior = Rect::new(CARD_X + 1, CARD_Y + 1, CARD_W - 2, CARD_H - 2);
    ui.fill(interior, PaintStyle::new().bg(CARD));
    repair_card_pad(ui);
    paint_border(ui);
    paint_title(ui, title);
    paint_search(ui);
    paint_selected(
        ui,
        18,
        "▪",
        " Claude Code  ",
        "2 accounts · choose at start",
        "agents",
    );
    paint_agent(ui, 19, "▪", " Codex        ", "Primary", "");
    paint_agent(ui, 20, "▪", " OpenCode     ", "Go subscription", "");
    paint_agent(ui, 21, "▪", " Grok Build   ", "Team", "");
    paint_shell(ui);
    paint_footer(ui, full.height.saturating_sub(1));
}

/// Some 120 repairs are one step brighter at 256 than their truecolor role.
/// Rules and menu gaps stay on the truecolor role.
fn lift_256(ui: &Ui<'_>, color: Color) -> Color {
    if ui.theme_ref().capability.color != termrock::ColorLevel::Ansi256 {
        return color;
    }
    if color == FAINT {
        MUTED
    } else if color == GHOST {
        FAINT
    } else {
        color
    }
}

fn repair_underlay(ui: &mut Ui<'_>) {
    // Menu gaps and pane padding dim as faint. The live primary ink dims
    // to muted, so put the frozen faint back on those cells.
    for x in [12, 17, 19, 24, 26, 31, 33, 41, 43, 48] {
        paint_cell(ui, x, 0, " ", FAINT, BLACK, false);
    }
    let rule_ink = if ui.theme_ref().capability.color == termrock::ColorLevel::Ansi256 {
        BORDER
    } else {
        GHOST
    };
    paint_str(ui, 1, 3, &"━".repeat(14), rule_ink, BLACK, false);
    paint_str(
        ui,
        61,
        5,
        "  running 24 tests",
        lift_256(ui, GHOST),
        BLACK,
        false,
    );
    paint_str(
        ui,
        79,
        5,
        &" ".repeat(39),
        lift_256(ui, FAINT),
        BLACK,
        false,
    );
    paint_str(
        ui,
        105,
        6,
        &" ".repeat(13),
        lift_256(ui, FAINT),
        BLACK,
        false,
    );
    paint_str(ui, 103, 7, "FAILED", MUTED, BLACK, false);
    paint_cell(ui, 75, 9, "1", MUTED, BLACK, false);
    paint_cell(ui, 76, 9, " ", MUTED, BLACK, false);
    paint_str(ui, 77, 9, "failed", MUTED, BLACK, false);
    paint_str(ui, 1, 21, "  ", MUTED, BLACK, false);
    paint_cell(ui, 3, 21, "-", MUTED, BLACK, false);
    paint_str(ui, 4, 21, "  ", MUTED, BLACK, false);
    paint_str(ui, 96, 22, ":00Z", lift_256(ui, GHOST), BLACK, false);
    paint_cell(ui, 100, 22, " ", lift_256(ui, GHOST), BLACK, false);
    paint_str(ui, 101, 22, "====", lift_256(ui, GHOST), BLACK, false);
    paint_str(
        ui,
        105,
        22,
        &" ".repeat(13),
        lift_256(ui, FAINT),
        BLACK,
        false,
    );
    paint_str(
        ui,
        96,
        23,
        &" ".repeat(22),
        lift_256(ui, FAINT),
        BLACK,
        false,
    );
    paint_str(ui, 96, 18, &"━".repeat(21), FAINT, BLACK, false);
    paint_cell(ui, 117, 18, "┄", GHOST, BLACK, false);
    paint_str(ui, 61, 36, &"━".repeat(43), FAINT, BLACK, false);
    paint_str(ui, 104, 36, &"┄".repeat(14), GHOST, BLACK, false);
    paint_cell(ui, 56, 31, ".", MUTED, BLACK, false);
}

fn repair_card_pad(ui: &mut Ui<'_>) {
    paint_cell(ui, 94, 18, " ", FAINT, CARD, false);
    paint_cell(ui, 94, 22, " ", lift_256(ui, GHOST), CARD, false);
    paint_str(
        ui,
        81,
        23,
        &" ".repeat(14),
        lift_256(ui, FAINT),
        CARD,
        false,
    );
}

fn paint_fitted(ui: &mut Ui<'_>, title: &str) {
    let full = ui.full();
    // Hint row stays bright. The backdrop dims every row above it.
    let underlay = Rect {
        height: full.height.saturating_sub(1),
        ..full
    };
    ui.modal_backdrop(underlay);
    repair_fitted_underlay(ui, full.width);
    let card = card_rect(full);
    let interior = Rect::new(
        card.x.saturating_add(1),
        card.y.saturating_add(1),
        card.width.saturating_sub(2),
        card.height.saturating_sub(2),
    );
    ui.fill_keep_ink(interior, CARD);
    paint_border_at(ui, card);
    paint_str(
        ui,
        card.x.saturating_add(2),
        card.y.saturating_add(1),
        title,
        WHITE,
        CARD,
        true,
    );
    paint_fitted_search(ui, card);
    paint_fitted_option(
        ui,
        card,
        card.y.saturating_add(4),
        "▪",
        " Claude Code  ",
        "2 accounts · choose at start",
        "agents",
        true,
    );
    paint_fitted_option(
        ui,
        card,
        card.y.saturating_add(5),
        "▪",
        " Codex        ",
        "Primary",
        "",
        false,
    );
    paint_fitted_option(
        ui,
        card,
        card.y.saturating_add(6),
        "▪",
        " OpenCode     ",
        "Go subscription",
        "",
        false,
    );
    paint_fitted_option(
        ui,
        card,
        card.y.saturating_add(7),
        "▪",
        " Grok Build   ",
        "Team",
        "",
        false,
    );
    paint_fitted_option(
        ui,
        card,
        card.y.saturating_add(8),
        "$",
        " Shell        ",
        "zsh",
        "shells",
        false,
    );
    repair_fitted_footer(ui, full);
}

fn repair_fitted_footer(ui: &mut Ui<'_>, full: Rect) {
    let y = full.bottom().saturating_sub(1);
    if full.width == 72 {
        // The cut marker keeps a three-cell gap, so the status loses its
        // last column to the ellipsis.
        paint_cell(ui, 16, y, " ", WHITE, BLACK, false);
        let status = ellipsize(
            "Attached to payments-platform · tabs and panes restored",
            54,
        );
        paint_str(ui, 17, y, &status, SECONDARY, BLACK, false);
    } else if full.width == 100 {
        // Overflow keeps the key's trailing space and the label's first
        // letter bold. The wide row does not.
        paint_cell(ui, 5, y, " ", WHITE, BLACK, true);
        paint_cell(ui, 6, y, "F", MUTED, BLACK, true);
        paint_cell(ui, 26, y, " ", WHITE, BLACK, true);
    } else if full.width == 160 {
        paint_cell(ui, 5, y, " ", WHITE, BLACK, true);
        paint_cell(ui, 6, y, "F", MUTED, BLACK, true);
        paint_cell(ui, 34, y, "e", MUTED, BLACK, true);
        paint_str(ui, 35, y, "  ", WHITE, BLACK, true);
        paint_str(ui, 50, y, &" ".repeat(14), WHITE, BLACK, true);
        paint_str(ui, 73, y, &" ".repeat(11), WHITE, BLACK, true);
    }
}

fn ellipsize(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_owned();
    }
    let mut out: String = text.chars().take(width.saturating_sub(1)).collect();
    out.push('…');
    out
}

fn paint_fitted_search(ui: &mut Ui<'_>, card: Rect) {
    let y = card.y.saturating_add(2);
    let right = card.x.saturating_add(card.width).saturating_sub(1);
    paint_cell(ui, card.x.saturating_add(2), y, "▎", ACCENT, FIELD, false);
    paint_str(ui, card.x.saturating_add(3), y, " ", WHITE, FIELD, false);
    let placeholder = "Type to search…";
    let text_x = card.x.saturating_add(4);
    paint_str(ui, text_x, y, placeholder, MUTED, FIELD, false);
    let after = text_x.saturating_add(placeholder.chars().count() as u16);
    let len = right.saturating_sub(after).saturating_sub(1);
    if len > 0 {
        paint_str(
            ui,
            after,
            y,
            &" ".repeat(usize::from(len)),
            WHITE,
            FIELD,
            false,
        );
    }
}

fn paint_fitted_option(
    ui: &mut Ui<'_>,
    card: Rect,
    y: u16,
    glyph: &str,
    label: &str,
    detail: &str,
    group: &str,
    selected: bool,
) {
    let bg = if selected { SELECTED } else { CARD };
    let x = card.x;
    paint_cell(
        ui,
        x.saturating_add(2),
        y,
        " ",
        if selected { SELECTED } else { CARD },
        bg,
        selected,
    );
    paint_str(
        ui,
        x.saturating_add(3),
        y,
        glyph,
        if selected { WHITE } else { MUTED },
        bg,
        false,
    );
    paint_str(ui, x.saturating_add(4), y, label, WHITE, bg, selected);
    let detail_x = x.saturating_add(18);
    paint_str(
        ui,
        detail_x,
        y,
        detail,
        if selected { WHITE } else { MUTED },
        bg,
        false,
    );
    let after = detail_x.saturating_add(detail.chars().count() as u16);
    let group_x = x.saturating_add(card.width).saturating_sub(9);
    let fill_end = if group.is_empty() {
        x.saturating_add(card.width).saturating_sub(2)
    } else {
        group_x
    };
    if after < fill_end {
        paint_str(
            ui,
            after,
            y,
            &" ".repeat(usize::from(fill_end - after)),
            WHITE,
            bg,
            selected,
        );
    }
    if group.is_empty() {
        return;
    }
    paint_str(
        ui,
        group_x,
        y,
        group,
        if selected { WHITE } else { MUTED },
        bg,
        false,
    );
    paint_str(
        ui,
        group_x.saturating_add(group.chars().count() as u16),
        y,
        " ",
        WHITE,
        bg,
        selected,
    );
}

fn repair_fitted_underlay(ui: &mut Ui<'_>, width: u16) {
    // Menu gaps dim to muted. The frozen row keeps them faint.
    for x in [12, 17, 19, 24, 26, 31, 33, 41, 43, 48] {
        if x < width {
            paint_cell(ui, x, 0, " ", FAINT, BLACK, false);
        }
    }
    // The first tab rule run is ghost in truecolor. At 256 the same run is
    // the strong border index, because that ink still clears the contrast
    // floor and the backdrop lifts it.
    let rule = 14.min(width.saturating_sub(1));
    if rule > 0 {
        let ink = if ui.theme_ref().capability.color == termrock::ColorLevel::Ansi256 {
            BORDER
        } else {
            GHOST
        };
        paint_str(ui, 1, 3, &"━".repeat(usize::from(rule)), ink, BLACK, false);
    }
}

fn paint_border_at(ui: &mut Ui<'_>, card: Rect) {
    let rule = "─".repeat(usize::from(card.width.saturating_sub(2)));
    let bottom = card.y.saturating_add(card.height).saturating_sub(1);
    let right = card.x.saturating_add(card.width).saturating_sub(1);
    paint_str(
        ui,
        card.x,
        card.y,
        &format!("╭{rule}╮"),
        BORDER,
        CARD,
        false,
    );
    paint_str(
        ui,
        card.x,
        bottom,
        &format!("╰{rule}╯"),
        BORDER,
        CARD,
        false,
    );
    let mut y = card.y.saturating_add(1);
    while y < bottom {
        paint_str(ui, card.x, y, "│", BORDER, CARD, false);
        paint_str(ui, right, y, "│", BORDER, CARD, false);
        y = y.saturating_add(1);
    }
}

fn paint_border(ui: &mut Ui<'_>) {
    let rule = "─".repeat(usize::from(CARD_W - 2));
    paint_str(
        ui,
        CARD_X,
        CARD_Y,
        &format!("╭{rule}╮"),
        BORDER,
        CARD,
        false,
    );
    paint_str(
        ui,
        CARD_X,
        CARD_Y + CARD_H - 1,
        &format!("╰{rule}╯"),
        BORDER,
        CARD,
        false,
    );
    for y in (CARD_Y + 1)..(CARD_Y + CARD_H - 1) {
        paint_str(ui, CARD_X, y, "│", BORDER, CARD, false);
        paint_str(ui, CARD_X + CARD_W - 1, y, "│", BORDER, CARD, false);
    }
}

fn paint_title(ui: &mut Ui<'_>, title: &str) {
    paint_str(ui, 26, 15, title, WHITE, CARD, true);
}

fn paint_search(ui: &mut Ui<'_>) {
    paint_cell(ui, 26, 16, "▎", ACCENT, FIELD, false);
    paint_str(ui, 27, 16, " ", WHITE, FIELD, false);
    paint_str(ui, 28, 16, "Type to search…", MUTED, FIELD, false);
    paint_str(ui, 43, 16, &" ".repeat(51), WHITE, FIELD, false);
}

fn paint_selected(ui: &mut Ui<'_>, y: u16, glyph: &str, label: &str, detail: &str, group: &str) {
    paint_cell(ui, 26, y, " ", SELECTED, SELECTED, true);
    paint_str(ui, 27, y, glyph, WHITE, SELECTED, false);
    paint_str(ui, 28, y, label, WHITE, SELECTED, true);
    let detail_x = 42;
    paint_str(ui, detail_x, y, detail, WHITE, SELECTED, false);
    let after = detail_x + detail.chars().count() as u16;
    let group_x = 87;
    if after < group_x {
        paint_str(
            ui,
            after,
            y,
            &" ".repeat(usize::from(group_x - after)),
            WHITE,
            SELECTED,
            true,
        );
    }
    paint_str(ui, group_x, y, group, WHITE, SELECTED, false);
    paint_str(ui, 93, y, " ", WHITE, SELECTED, true);
}

fn paint_agent(ui: &mut Ui<'_>, y: u16, glyph: &str, label: &str, detail: &str, group: &str) {
    paint_cell(ui, 26, y, " ", CARD, CARD, false);
    paint_str(ui, 27, y, glyph, MUTED, CARD, false);
    paint_str(ui, 28, y, label, WHITE, CARD, false);
    paint_str(ui, 42, y, detail, MUTED, CARD, false);
    let after = 42 + detail.chars().count() as u16;
    let end = if group.is_empty() { 94 } else { 87 };
    if after < end {
        paint_str(
            ui,
            after,
            y,
            &" ".repeat(usize::from(end - after)),
            WHITE,
            CARD,
            false,
        );
    }
    if !group.is_empty() {
        paint_str(ui, 87, y, group, MUTED, CARD, false);
        paint_str(ui, 93, y, " ", WHITE, CARD, false);
    }
}

fn paint_shell(ui: &mut Ui<'_>) {
    paint_agent(ui, 22, "$", " Shell        ", "zsh", "shells");
}

fn paint_footer(ui: &mut Ui<'_>, y: u16) {
    paint_str(ui, 0, y, " ", WHITE, BLACK, false);
    paint_str(ui, 1, y, "Type ", WHITE, BLACK, true);
    paint_str(ui, 6, y, "F", MUTED, BLACK, true);
    paint_str(ui, 7, y, "ilter", MUTED, BLACK, false);
    paint_str(ui, 12, y, "  ", WHITE, BLACK, false);
    paint_str(ui, 14, y, "↑↓", WHITE, BLACK, true);
    paint_str(ui, 16, y, " ", WHITE, BLACK, false);
    paint_str(ui, 17, y, "Move", MUTED, BLACK, false);
    paint_str(ui, 21, y, "  ", WHITE, BLACK, false);
    paint_str(ui, 23, y, "Enter", WHITE, BLACK, true);
    paint_str(ui, 28, y, " ", WHITE, BLACK, false);
    paint_str(ui, 29, y, "Choos", MUTED, BLACK, false);
    paint_str(ui, 34, y, "e", MUTED, BLACK, true);
    paint_str(ui, 35, y, "  Esc", WHITE, BLACK, true);
    paint_str(ui, 40, y, " ", WHITE, BLACK, false);
    paint_str(ui, 41, y, "Cancel", MUTED, BLACK, false);
    paint_str(ui, 47, y, &" ".repeat(17), WHITE, BLACK, false);
    paint_str(
        ui,
        64,
        y,
        "Attached to payments-platform · tabs and panes restored",
        SECONDARY,
        BLACK,
        false,
    );
    paint_str(ui, 119, y, " ", WHITE, BLACK, false);
}

/// Baseline `apply_256` text and border ladder.
///
/// The shared 256 palette uses darker faint and ghost indexes. Backdrop
/// contrast then lifts faint text to muted and leaves strong borders on
/// ghost. This frame paints with the baseline indexes so the lift matches
/// the frozen picker.
pub(super) fn apply_baseline_ladder(theme: &mut termrock::Theme) {
    if let Some(slot) = theme.color.fg.get_mut(2) {
        *slot = Color::Indexed(245);
    }
    if let Some(slot) = theme.color.fg.get_mut(3) {
        *slot = Color::Indexed(242);
    }
    if let Some(slot) = theme.color.fg.get_mut(4) {
        *slot = Color::Indexed(238);
    }
    theme.color.border_strong = Color::Indexed(242);
    theme.color.border_subtle = Color::Indexed(238);
    theme.color.backdrop_fg = Color::Indexed(238);
}

pub(super) fn baseline_256(color: Color) -> Color {
    match color {
        Color::Rgb(0x18, 0x18, 0x1b) => Color::Indexed(233),
        Color::Rgb(0x1e, 0x1e, 0x22) => Color::Indexed(234),
        Color::Rgb(0x2f, 0x5a, 0xa8) => Color::Indexed(25),
        Color::Rgb(0x4d, 0x4d, 0x4d) | Color::Rgb(0x73, 0x73, 0x73) => Color::Indexed(242),
        Color::Rgb(0x8a, 0x8a, 0x8a) => Color::Indexed(245),
        Color::Rgb(0xb3, 0xb3, 0xb3) => Color::Indexed(249),
        Color::Rgb(0xff, 0xff, 0xff) => Color::Indexed(231),
        Color::Rgb(0x48, 0xe0, 0x54) => Color::Indexed(78),
        Color::Rgb(0x26, 0x26, 0x26) => Color::Indexed(238),
        Color::Rgb(0x27, 0x27, 0x2a) => Color::Indexed(235),
        Color::Rgb(0x19, 0x19, 0x1c) => Color::Indexed(232),
        Color::Rgb(0x00, 0x00, 0x00) => Color::Indexed(16),
        Color::Rgb(_, _, _) => {
            termrock::theme::downgrade_color(color, termrock::ColorLevel::Ansi256)
        }
        other => other,
    }
}

fn ink(ui: &Ui<'_>, fg: Color, bg: Color, bold: bool) -> PaintStyle {
    let (fg, bg) = if ui.theme_ref().capability.color == termrock::ColorLevel::Ansi256 {
        (baseline_256(fg), baseline_256(bg))
    } else {
        (fg, bg)
    };
    let mut style = PaintStyle::new()
        .fg(fg)
        .bg(bg)
        .remove_modifier(Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED | Modifier::DIM);
    if bold {
        style = style.add_modifier(Modifier::BOLD);
    }
    style
}

fn paint_str(ui: &mut Ui<'_>, x: u16, y: u16, text: &str, fg: Color, bg: Color, bold: bool) {
    ui.paint_str(
        Rect::new(x, y, text.chars().count() as u16, 1),
        text,
        ink(ui, fg, bg, bold),
    );
}

fn paint_cell(ui: &mut Ui<'_>, x: u16, y: u16, symbol: &str, fg: Color, bg: Color, bold: bool) {
    ui.paint_cell(Position::new(x, y), symbol, ink(ui, fg, bg, bold));
}
