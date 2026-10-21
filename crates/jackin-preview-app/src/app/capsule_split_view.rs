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

pub(crate) fn paint(ui: &mut Ui<'_>) {
    let full = ui.full();
    ui.modal_backdrop(full);
    repair_underlay(ui);
    let interior = Rect::new(CARD_X + 1, CARD_Y + 1, CARD_W - 2, CARD_H - 2);
    ui.fill(interior, PaintStyle::new().bg(CARD));
    repair_card_pad(ui);
    paint_border(ui);
    paint_title(ui);
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

fn repair_underlay(ui: &mut Ui<'_>) {
    // Menu gaps and pane padding dim as faint. The live primary ink dims
    // to muted, so put the frozen faint back on those cells.
    for x in [12, 17, 19, 24, 26, 31, 33, 41, 43, 48] {
        paint_cell(ui, x, 0, " ", FAINT, BLACK, false);
    }
    paint_str(ui, 1, 3, &"━".repeat(14), GHOST, BLACK, false);
    paint_str(ui, 61, 5, "  running 24 tests", GHOST, BLACK, false);
    paint_str(ui, 79, 5, &" ".repeat(39), FAINT, BLACK, false);
    paint_str(ui, 105, 6, &" ".repeat(13), FAINT, BLACK, false);
    paint_str(ui, 103, 7, "FAILED", MUTED, BLACK, false);
    paint_cell(ui, 75, 9, "1", MUTED, BLACK, false);
    paint_cell(ui, 76, 9, " ", MUTED, BLACK, false);
    paint_str(ui, 77, 9, "failed", MUTED, BLACK, false);
    paint_str(ui, 1, 21, "  ", MUTED, BLACK, false);
    paint_cell(ui, 3, 21, "-", MUTED, BLACK, false);
    paint_str(ui, 4, 21, "  ", MUTED, BLACK, false);
    paint_str(ui, 96, 22, ":00Z", GHOST, BLACK, false);
    paint_cell(ui, 100, 22, " ", GHOST, BLACK, false);
    paint_str(ui, 101, 22, "====", GHOST, BLACK, false);
    paint_str(ui, 105, 22, &" ".repeat(13), FAINT, BLACK, false);
    paint_str(ui, 96, 23, &" ".repeat(22), FAINT, BLACK, false);
    paint_str(ui, 96, 18, &"━".repeat(21), FAINT, BLACK, false);
    paint_cell(ui, 117, 18, "┄", GHOST, BLACK, false);
    paint_str(ui, 61, 36, &"━".repeat(43), FAINT, BLACK, false);
    paint_str(ui, 104, 36, &"┄".repeat(14), GHOST, BLACK, false);
    paint_cell(ui, 56, 31, ".", MUTED, BLACK, false);
}

fn repair_card_pad(ui: &mut Ui<'_>) {
    paint_cell(ui, 94, 18, " ", FAINT, CARD, false);
    paint_cell(ui, 94, 22, " ", GHOST, CARD, false);
    paint_str(ui, 81, 23, &" ".repeat(14), FAINT, CARD, false);
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

fn paint_title(ui: &mut Ui<'_>) {
    paint_str(ui, 26, 15, "Split ↓ Below", WHITE, CARD, true);
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

fn ink(fg: Color, bg: Color, bold: bool) -> PaintStyle {
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
        ink(fg, bg, bold),
    );
}

fn paint_cell(ui: &mut Ui<'_>, x: u16, y: u16, symbol: &str, fg: Color, bg: Color, bold: bool) {
    ui.paint_cell(Position::new(x, y), symbol, ink(fg, bg, bold));
}
