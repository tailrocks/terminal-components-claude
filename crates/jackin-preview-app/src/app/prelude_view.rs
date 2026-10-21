//! First-use new-workspace prelude at 120×40.

use termrock::author::{PaintStyle, put_cell, put_fill, put_fill_in, put_text};
use termrock::{Color, Modifier, Rect, Style, Ui};

use super::App;

const ACCENT: Color = Color::Rgb(0x48, 0xe0, 0x54);
const BRAND_FG: Color = Color::Rgb(0x19, 0x19, 0x1c);
const SECONDARY: Color = Color::Rgb(0xb3, 0xb3, 0xb3);
const MUTED: Color = Color::Rgb(0x8a, 0x8a, 0x8a);
const ACTIVE_BG: Color = Color::Rgb(0x27, 0x27, 0x2a);
const FIELD: Color = Color::Rgb(0x1e, 0x1e, 0x22);
const CARD: Color = Color::Rgb(0x18, 0x18, 0x1b);
const BORDER: Color = Color::Rgb(0x4d, 0x4d, 0x4d);
const BLACK: Color = Color::Rgb(0, 0, 0);
const WHITE: Color = Color::Rgb(255, 255, 255);

struct SourceRow {
    name: &'static str,
    meta: &'static str,
    file: bool,
}

const ROWS: &[SourceRow] = &[
    SourceRow {
        name: "..",
        meta: "parent",
        file: false,
    },
    SourceRow {
        name: "crates/",
        meta: "6 items",
        file: false,
    },
    SourceRow {
        name: "docs/",
        meta: "adr",
        file: false,
    },
    SourceRow {
        name: "scripts/",
        meta: "3 items",
        file: false,
    },
    SourceRow {
        name: "Cargo.toml",
        meta: "1 h",
        file: true,
    },
    SourceRow {
        name: "README.md",
        meta: "3 d",
        file: true,
    },
];

pub(crate) fn paint(app: &App, ui: &mut Ui<'_>, full: Rect) {
    put_fill_in(ui, full, canvas());
    paint_str(ui, 1, 0, " jackin❯ ", BRAND_FG, ACCENT, true);
    paint_str(ui, 12, 0, " File ", SECONDARY, BLACK, false);
    paint_str(ui, 19, 0, " Go ", SECONDARY, BLACK, false);
    paint_str(ui, 24, 0, " Help ", SECONDARY, BLACK, false);
    paint_str(
        ui,
        56,
        0,
        "Workspaces › new workspace",
        SECONDARY,
        BLACK,
        false,
    );
    paint_str(ui, 84, 0, "inside the Construct", SECONDARY, BLACK, false);
    paint_str(ui, 106, 0, "no instances", SECONDARY, BLACK, false);

    ui.modal_backdrop(full);
    paint_dialog(app, ui);
    paint_footer(ui, full.height.saturating_sub(1));
}

fn paint_dialog(app: &App, ui: &mut Ui<'_>) {
    let x = 18u16;
    let y = 9u16;
    let w = 84u16;
    let h = 22u16;
    put_fill(ui, x, y, w, h, Style::default().fg(MUTED).bg(CARD));
    paint_str(ui, x, y, "╭─", BORDER, CARD, false);
    paint_str(
        ui,
        x.saturating_add(2),
        y,
        " New workspace · step 1 of 5 · Source ",
        WHITE,
        CARD,
        true,
    );
    for col in 58..97 {
        paint_cell(ui, col, y, "─", BORDER, CARD);
    }
    paint_cell(ui, 97, y, " ", BORDER, CARD);
    paint_str(ui, 98, y, "×", SECONDARY, CARD, false);
    paint_str(ui, 99, y, " ─╮", BORDER, CARD, false);
    for row in 1..h.saturating_sub(1) {
        paint_cell(ui, x, y.saturating_add(row), "│", BORDER, CARD);
        paint_cell(
            ui,
            x.saturating_add(w).saturating_sub(1),
            y.saturating_add(row),
            "│",
            BORDER,
            CARD,
        );
    }
    paint_str(
        ui,
        x,
        y.saturating_add(h).saturating_sub(1),
        &format!("╰{}╯", "─".repeat(82)),
        BORDER,
        CARD,
        false,
    );

    paint_str(
        ui,
        21,
        10,
        "Source · Destination · Edit · Working dir · Name",
        MUTED,
        CARD,
        false,
    );
    paint_str(
        ui,
        22,
        12,
        &format!("Path{:<73}", ""),
        SECONDARY,
        CARD,
        false,
    );
    paint_str(ui, 20, 13, " ", FIELD, FIELD, false);
    paint_str(
        ui,
        21,
        13,
        &format!(" {:<77}", app.prelude.source()),
        WHITE,
        FIELD,
        false,
    );

    for (index, row) in ROWS.iter().enumerate() {
        let line = 15 + index as u16;
        if index == 0 {
            paint_str(ui, 20, line, "▎", ACCENT, CARD, true);
            paint_str(
                ui,
                21,
                line,
                &format!("  {:<69}", row.name),
                WHITE,
                CARD,
                true,
            );
            paint_str(ui, 92, line, row.meta, MUTED, CARD, true);
            paint_str(ui, 98, line, " ", WHITE, CARD, true);
        } else if row.file {
            paint_cell(ui, 20, line, " ", CARD, CARD);
            let body = format!("  {:<68}{:>7} ", row.name, row.meta);
            paint_str(ui, 21, line, &body, BORDER, CARD, false);
        } else {
            paint_cell(ui, 20, line, " ", CARD, CARD);
            paint_str(ui, 21, line, &" ".repeat(78), WHITE, CARD, false);
            paint_str(ui, 21, line, &format!("  {}", row.name), WHITE, CARD, false);
            let meta_x = 98u16.saturating_sub(row.meta.chars().count() as u16);
            paint_str(ui, meta_x, line, row.meta, MUTED, CARD, false);
        }
    }

    paint_cell(ui, 20, 26, " ", CARD, CARD);
    paint_str(ui, 21, 26, "[ ]", MUTED, CARD, false);
    paint_str(
        ui,
        24,
        26,
        &format!(" Mount read-only{:<59}", ""),
        WHITE,
        CARD,
        false,
    );

    paint_cell(ui, 71, 29, " ", ACTIVE_BG, ACTIVE_BG);
    paint_str(ui, 72, 29, "Git URL… ", WHITE, ACTIVE_BG, false);
    paint_cell(ui, 82, 29, " ", CARD, CARD);
    paint_str(ui, 83, 29, "Cancel ", SECONDARY, CARD, false);
    paint_cell(ui, 91, 29, " ", ACCENT, ACCENT);
    paint_str(ui, 92, 29, "Choose ", BRAND_FG, ACCENT, true);
}

fn paint_footer(ui: &mut Ui<'_>, y: u16) {
    paint_str(ui, 0, y, &" ".repeat(120), WHITE, BLACK, false);
    paint_str(ui, 1, y, "Enter", WHITE, BLACK, true);
    paint_str(ui, 7, y, "Open", MUTED, BLACK, false);
    paint_str(ui, 13, y, "Space ", WHITE, BLACK, true);
    paint_str(ui, 19, y, "Cho", MUTED, BLACK, true);
    paint_str(ui, 22, y, "ose", MUTED, BLACK, false);
    paint_str(ui, 27, y, "g ", WHITE, BLACK, true);
    paint_str(ui, 29, y, "Git", MUTED, BLACK, true);
    paint_str(ui, 32, y, " URL", MUTED, BLACK, false);
    paint_str(ui, 38, y, "Tab ", WHITE, BLACK, true);
    paint_str(ui, 42, y, "Next", MUTED, BLACK, false);
    paint_str(ui, 48, y, "Esc", WHITE, BLACK, true);
    paint_str(ui, 52, y, "Can", MUTED, BLACK, true);
    paint_str(ui, 55, y, "cel", MUTED, BLACK, false);
    paint_str(ui, 62, y, "   ", WHITE, BLACK, true);
}

fn canvas() -> PaintStyle {
    PaintStyle::new()
        .fg(WHITE)
        .bg(BLACK)
        .remove_modifier(Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED | Modifier::DIM)
}

fn paint_str(ui: &mut Ui<'_>, x: u16, y: u16, text: &str, fg: Color, bg: Color, bold: bool) {
    let mut style = canvas().fg(fg).bg(bg);
    if bold {
        style = style.add_modifier(Modifier::BOLD);
    }
    put_text(ui, x, y, text, style);
}

fn paint_cell(ui: &mut Ui<'_>, x: u16, y: u16, symbol: &str, fg: Color, bg: Color) {
    put_cell(ui, x, y, symbol, canvas().fg(fg).bg(bg));
}
