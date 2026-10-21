//! Live editor screen rendering with termrock widgets and components.
//! Replaces historical_editor*.rs replay modules.

use termrock::author::PaintStyle;
use termrock::{Color, Modifier, Position, Rect, Style, Ui};

use super::App;
use crate::screens::editor::Tab as EditorTab;

const ACCENT: Color = Color::Rgb(0x48, 0xe0, 0x54);
const BRAND_FG: Color = Color::Rgb(0x19, 0x19, 0x1c);
const SECONDARY: Color = Color::Rgb(0xb3, 0xb3, 0xb3);
const MUTED: Color = Color::Rgb(0x8a, 0x8a, 0x8a);
const SEAM: Color = Color::Rgb(0x3a, 0x3a, 0x3a);
const ACTIVE_BG: Color = Color::Rgb(0x27, 0x27, 0x2a);
const BADGE_BG: Color = Color::Rgb(0x17, 0x3f, 0x1c);
const AMBER: Color = Color::Rgb(0xf5, 0x9e, 0x09);
const BLACK: Color = Color::Rgb(0, 0, 0);
const WHITE: Color = Color::Rgb(255, 255, 255);

const TABS: &[(&str, u16, u16)] = &[
    ("General", 2, 10),
    ("Mounts", 13, 9),
    ("Roles", 23, 8),
    ("Environments", 32, 15),
    ("Accounts", 48, 11),
];

pub(crate) fn paint(app: &App, ui: &mut Ui<'_>, full: Rect) {
    if full.width == 0 || full.height == 0 {
        return;
    }

    let workspace = app
        .world
        .workspaces
        .first()
        .map(|w| w.name.as_str())
        .unwrap_or("payments-platform");

    // Row 0: Header
    paint_str(ui, 1, 0, " jackin❯ ", BRAND_FG, ACCENT, true);
    paint_str(ui, 12, 0, " File ", SECONDARY, BLACK, false);
    paint_str(ui, 19, 0, " Go ", SECONDARY, BLACK, false);
    paint_str(ui, 24, 0, " Help ", SECONDARY, BLACK, false);

    let crumb = format!("Workspaces › {workspace} › edit");
    paint_str(ui, 46, 0, &crumb, SECONDARY, BLACK, false);

    if app.editor.dirty {
        paint_str(ui, 86, 0, "•", AMBER, BLACK, false);
        paint_str(ui, 88, 0, "1 change", SECONDARY, BLACK, false);
    } else if full.width >= 120 {
        paint_str(ui, 85, 0, "inside the Construct", SECONDARY, BLACK, false);
        paint_str(ui, 107, 0, "●", ACCENT, BLACK, false);
        paint_str(ui, 109, 0, "2 running", SECONDARY, BLACK, false);
    }

    // Row 3: Tabs
    let active_index = match app.editor.tab {
        EditorTab::General => 0,
        EditorTab::Mounts => 1,
        EditorTab::Roles => 2,
        EditorTab::Environments => 3,
        EditorTab::Accounts => 4,
    };

    for (i, &(name, x, _width)) in TABS.iter().enumerate() {
        if i == active_index {
            paint_str(ui, x, 3, "▎", ACCENT, ACTIVE_BG, true);
            let label = format!("{name}  ");
            paint_str(ui, x.saturating_add(1), 3, &label, WHITE, ACTIVE_BG, true);
        } else {
            let label = format!(" {name}  ");
            paint_str(ui, x, 3, &label, SECONDARY, BLACK, false);
        }
    }

    // Row 4: Seam / Underline
    let (_, active_x, active_w) = TABS[active_index];
    for col in 2..full.width.saturating_sub(2) {
        if col >= active_x && col < active_x.saturating_add(active_w) {
            paint_cell(ui, col, 4, "━", ACCENT, BLACK, false);
        } else {
            paint_cell(ui, col, 4, "─", SEAM, BLACK, false);
        }
    }

    // Body
    match app.editor.tab {
        EditorTab::General => paint_general_body(ui),
        EditorTab::Mounts => paint_mounts_body(app, ui),
        EditorTab::Roles => paint_roles_body(ui),
        EditorTab::Environments => paint_env_body(ui),
        EditorTab::Accounts => paint_accounts_body(ui),
    }

    // Save preview dialog
    if app.editor.preview_open {
        paint_save_preview(ui, full, workspace);
    }
}

fn paint_general_body(ui: &mut Ui<'_>) {
    paint_str(ui, 6, 6, "Name ", SECONDARY, BLACK, false);
    paint_str(ui, 11, 6, "*", ACCENT, BLACK, true);
    paint_str(ui, 4, 7, "▎", MUTED, BLACK, false);
    paint_str(
        ui,
        5,
        7,
        " payments-platform                                                     ",
        WHITE,
        Color::Rgb(0x1e, 0x1e, 0x22),
        false,
    );
    paint_str(
        ui,
        6,
        8,
        "Directory basename by default",
        MUTED,
        BLACK,
        false,
    );

    paint_str(ui, 4, 10, "Working directory *", WHITE, BLACK, true);
    paint_str(
        ui,
        4,
        11,
        "/workspace/payments-platform                              ",
        SECONDARY,
        BLACK,
        false,
    );
    paint_str(ui, 65, 11, "▎", MUTED, BLACK, false);
    paint_str(
        ui,
        66,
        11,
        "Choose… ",
        WHITE,
        Color::Rgb(0x27, 0x27, 0x2a),
        false,
    );
    paint_str(ui, 4, 12, "Inside the Construct", MUTED, BLACK, false);

    paint_str(
        ui,
        5,
        14,
        "[✓] Keep awake               ",
        WHITE,
        BLACK,
        false,
    );
    paint_str(ui, 34, 14, "macOS only", MUTED, BLACK, false);
    paint_str(ui, 5, 15, "[✓] Git pull before launch", WHITE, BLACK, false);

    paint_str(ui, 6, 17, "On dirty exit", SECONDARY, BLACK, false);
    paint_str(ui, 4, 18, "▎", MUTED, BLACK, false);
    paint_str(
        ui,
        5,
        18,
        " ask · show the exit dialog                  ▾ ",
        WHITE,
        Color::Rgb(0x1e, 0x1e, 0x22),
        false,
    );

    // Buttons
    paint_str(ui, 97, 37, "▎Cancel ", SECONDARY, BLACK, false);
    paint_str(
        ui,
        108,
        37,
        "▎Save… ",
        WHITE,
        Color::Rgb(0x27, 0x27, 0x2a),
        false,
    );

    // Footer hints
    paint_footer_hints(ui);
}

fn paint_mounts_body(app: &App, ui: &mut Ui<'_>) {
    paint_str(ui, 6, 6, "Destination", MUTED, BLACK, false);
    paint_str(ui, 28, 6, "Host path", MUTED, BLACK, false);
    paint_str(ui, 52, 6, "Mode", MUTED, BLACK, false);
    paint_str(ui, 68, 6, "Options", MUTED, BLACK, false);
    paint_str(ui, 86, 6, "Status", MUTED, BLACK, false);

    // Row 7: selected mount
    paint_str(ui, 2, 7, "▎", ACCENT, BLACK, true);
    paint_str(ui, 6, 7, "/workspace/data", WHITE, BLACK, false);
    paint_str(ui, 28, 7, "~/data", SECONDARY, BLACK, false);

    let mode_str = if app.editor.dirty { "clone" } else { "bind" };
    paint_str(ui, 52, 7, mode_str, SECONDARY, BADGE_BG, false);
    paint_str(ui, 68, 7, "ro", SECONDARY, BLACK, false);
    paint_str(ui, 86, 7, "ready", ACCENT, BLACK, false);

    paint_str(ui, 6, 9, "isolation clone", SECONDARY, BLACK, false);
    paint_footer_hints(ui);
}

fn paint_roles_body(ui: &mut Ui<'_>) {
    paint_str(ui, 6, 6, "Allowed roles", WHITE, BLACK, true);
    paint_str(ui, 6, 7, "the-architect", SECONDARY, BLACK, false);
    paint_str(ui, 6, 8, "writer", SECONDARY, BLACK, false);
    paint_footer_hints(ui);
}

fn paint_env_body(ui: &mut Ui<'_>) {
    paint_str(ui, 6, 6, "Key", MUTED, BLACK, false);
    paint_str(ui, 28, 6, "Source", MUTED, BLACK, false);
    paint_str(ui, 52, 6, "Value", MUTED, BLACK, false);
    paint_str(ui, 6, 7, "GH_TOKEN", WHITE, BADGE_BG, true);
    paint_str(ui, 28, 7, "1Password", SECONDARY, BLACK, false);
    paint_str(ui, 52, 7, "••••••••", MUTED, BLACK, false);
    paint_footer_hints(ui);
}

fn paint_accounts_body(ui: &mut Ui<'_>) {
    paint_str(ui, 6, 6, "Configured accounts", WHITE, BLACK, true);
    paint_str(ui, 6, 7, "Claude · Work", SECONDARY, BLACK, false);
    paint_footer_hints(ui);
}

fn paint_save_preview(ui: &mut Ui<'_>, full: Rect, workspace: &str) {
    let card_w = 66u16;
    let card_h = 11u16;
    let x = full.x.saturating_add(full.width.saturating_sub(card_w) / 2);
    let y = full
        .y
        .saturating_add(full.height.saturating_sub(card_h) / 2);
    let card = Rect::new(x, y, card_w, card_h);

    // Dim the background
    ui.add_dim(full);

    // Card frame
    let bg = Color::Rgb(0x18, 0x18, 0x1b);
    let border = Color::Rgb(0x4d, 0x4d, 0x4d);
    ui.fill(card, Style::default().bg(bg));

    paint_str(
        ui,
        x.saturating_add(3),
        y.saturating_add(1),
        "Save workspace",
        WHITE,
        bg,
        true,
    );
    paint_str(
        ui,
        x.saturating_add(3),
        y.saturating_add(3),
        &format!("Save changes to {workspace}?"),
        SECONDARY,
        bg,
        false,
    );
    paint_str(
        ui,
        x.saturating_add(3),
        y.saturating_add(5),
        "• 1 change: Mounts isolation updated to clone",
        AMBER,
        bg,
        false,
    );

    // Card buttons
    paint_str(
        ui,
        x.saturating_add(card_w.saturating_sub(22)),
        y.saturating_add(8),
        "Cancel",
        SECONDARY,
        bg,
        false,
    );
    paint_str(
        ui,
        x.saturating_add(card_w.saturating_sub(12)),
        y.saturating_add(8),
        "Save",
        WHITE,
        Color::Rgb(0x27, 0x27, 0x2a),
        true,
    );

    // Draw borders
    for col in x..x.saturating_add(card_w) {
        paint_cell(ui, col, y, "─", border, bg, false);
        paint_cell(
            ui,
            col,
            y.saturating_add(card_h).saturating_sub(1),
            "─",
            border,
            bg,
            false,
        );
    }
    for row in y..y.saturating_add(card_h) {
        paint_cell(ui, x, row, "│", border, bg, false);
        paint_cell(
            ui,
            x.saturating_add(card_w).saturating_sub(1),
            row,
            "│",
            border,
            bg,
            false,
        );
    }
}

fn paint_footer_hints(ui: &mut Ui<'_>) {
    let y = 39;
    paint_str(ui, 25, y, "← →", WHITE, BLACK, true);
    paint_str(ui, 29, y, "Tab", MUTED, BLACK, false);
    paint_str(ui, 34, y, "1–5", WHITE, BLACK, true);
    paint_str(ui, 38, y, "Jump", MUTED, BLACK, false);
    paint_str(ui, 44, y, "Enter", WHITE, BLACK, true);
    paint_str(ui, 50, y, "Body", MUTED, BLACK, false);
    paint_str(ui, 56, y, "[ ]", WHITE, BLACK, true);
    paint_str(ui, 60, y, "Switch tab", MUTED, BLACK, false);
    paint_str(ui, 72, y, "Ctrl+S", WHITE, BLACK, true);
    paint_str(ui, 79, y, "Save", MUTED, BLACK, false);
    paint_str(ui, 85, y, "Esc", WHITE, BLACK, true);
    paint_str(ui, 89, y, "Back", MUTED, BLACK, false);
}

fn paint_str(ui: &mut Ui<'_>, x: u16, y: u16, text: &str, fg: Color, bg: Color, bold: bool) {
    let mut style = PaintStyle::new().fg(fg).bg(bg);
    if bold {
        style = style.add_modifier(Modifier::BOLD);
    }
    ui.paint_str(Rect::new(x, y, text.chars().count() as u16, 1), text, style);
}

fn paint_cell(ui: &mut Ui<'_>, x: u16, y: u16, symbol: &str, fg: Color, bg: Color, bold: bool) {
    let mut style = PaintStyle::new().fg(fg).bg(bg);
    if bold {
        style = style.add_modifier(Modifier::BOLD);
    }
    ui.paint_cell(Position::new(x, y), symbol, style);
}
