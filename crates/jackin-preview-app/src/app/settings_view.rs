//! Live settings screen rendering with termrock widgets and components.
//! Replaces historical_settings_*.rs replay modules.

use termrock::author::PaintStyle;
use termrock::{Color, Modifier, Position, Rect, Style, Ui};

use super::App;

const ACCENT: Color = Color::Rgb(0x48, 0xe0, 0x54);
const BRAND_FG: Color = Color::Rgb(0x19, 0x19, 0x1c);
const SECONDARY: Color = Color::Rgb(0xb3, 0xb3, 0xb3);
const MUTED: Color = Color::Rgb(0x8a, 0x8a, 0x8a);
const SEAM: Color = Color::Rgb(0x3a, 0x3a, 0x3a);
const ACTIVE_BG: Color = Color::Rgb(0x27, 0x27, 0x2a);
const BADGE_BG: Color = Color::Rgb(0x17, 0x3f, 0x1c);
const AMBER: Color = Color::Rgb(0xf5, 0x9e, 0x09);

const TABS: &[(&str, u16, u16)] = &[
    ("General", 2, 10),
    ("Mounts", 13, 9),
    ("Environments", 23, 15),
    ("Agents", 39, 9),
    ("Trust", 49, 8),
];

pub(crate) fn paint(app: &App, ui: &mut Ui<'_>, full: Rect) {
    if full.width == 0 || full.height == 0 {
        return;
    }

    // Row 0: Header
    paint_str(ui, 1, 0, " jackin❯ ", BRAND_FG, ACCENT, true);
    paint_str(ui, 12, 0, " File ", SECONDARY, Color::Black, false);
    paint_str(ui, 19, 0, " Go ", SECONDARY, Color::Black, false);
    paint_str(ui, 24, 0, " Help ", SECONDARY, Color::Black, false);

    let tab_name = match app.settings.tab {
        0 => "General",
        1 => "Mounts",
        2 => "Environments",
        3 => "Agents",
        4 => "Trust",
        _ => "General",
    };

    let crumb_x = if full.width >= 120 { 56 } else { 51 };
    let crumb = format!("Settings › global › {tab_name}");
    paint_str(ui, crumb_x, 0, &crumb, SECONDARY, Color::Black, false);

    if full.width >= 120 {
        paint_str(ui, 85, 0, "inside the Construct", SECONDARY, Color::Black, false);
        paint_str(ui, 107, 0, "●", ACCENT, Color::Black, false);
        paint_str(ui, 109, 0, "2 running", SECONDARY, Color::Black, false);
    }

    // Row 3: Tabs
    let active_index = (app.settings.tab as usize).min(TABS.len().saturating_sub(1));
    for (i, &(name, x, _width)) in TABS.iter().enumerate() {
        if i == active_index {
            paint_str(ui, x, 3, "▎", ACCENT, ACTIVE_BG, true);
            let label = format!("{name}  ");
            paint_str(ui, x.saturating_add(1), 3, &label, Color::White, ACTIVE_BG, true);
        } else {
            let label = format!(" {name}  ");
            paint_str(ui, x, 3, &label, SECONDARY, Color::Black, false);
        }
    }

    // Row 4: Seam / Underline
    let (_, active_x, active_w) = TABS[active_index];
    for col in 2..full.width.saturating_sub(2) {
        if col >= active_x && col < active_x.saturating_add(active_w) {
            paint_cell(ui, col, 4, "━", ACCENT, Color::Black, false);
        } else {
            paint_cell(ui, col, 4, "─", SEAM, Color::Black, false);
        }
    }

    // Body
    match app.settings.tab {
        0 => paint_general_body(app, ui),
        1 => paint_mounts_body(ui),
        2 => paint_env_body(ui),
        3 => paint_agents_body(ui),
        4 => paint_trust_body(ui),
        _ => paint_general_body(app, ui),
    }

    // Save preview dialog
    if app.settings_save_preview {
        paint_save_preview(ui, full);
    }
}

fn paint_general_body(app: &App, ui: &mut Ui<'_>) {
    paint_str(ui, 4, 6, "Runtime mode", SECONDARY, Color::Black, false);
    paint_str(ui, 4, 7, "Sync host credentials", Color::White, Color::Black, false);

    paint_str(ui, 4, 10, "Workspace", SECONDARY, Color::Black, false);
    paint_str(ui, 4, 11, "payments-platform", Color::White, Color::Black, false);

    paint_str(ui, 4, 14, "DCO signoff", SECONDARY, Color::Black, false);
    paint_str(ui, 4, 15, "enabled", Color::White, Color::Black, false);

    paint_str(ui, 4, 18, "Secret policy", SECONDARY, Color::Black, false);
    paint_str(
        ui,
        4,
        19,
        "references only; resolved bytes are transient",
        Color::White,
        Color::Black,
        false,
    );

    paint_str(ui, 4, 22, "Trust", SECONDARY, Color::Black, false);
    let trust_label = if app.trusted {
        "trusted · sources verified"
    } else {
        "untrusted · explicit prompt"
    };
    paint_str(ui, 4, 23, trust_label, Color::White, Color::Black, false);
}

fn paint_mounts_body(ui: &mut Ui<'_>) {
    paint_str(ui, 8, 6, "Destination", MUTED, Color::Black, false);
    paint_str(ui, 28, 6, "Host path", MUTED, Color::Black, false);
    paint_str(ui, 58, 6, "Mode", MUTED, Color::Black, false);
    paint_str(ui, 74, 6, "Status", MUTED, Color::Black, false);

    paint_str(ui, 8, 7, "/workspace/data", Color::White, Color::Black, false);
    paint_str(ui, 28, 7, "~/data", SECONDARY, Color::Black, false);
    paint_str(ui, 58, 7, "bind", SECONDARY, BADGE_BG, false);
    paint_str(ui, 74, 7, "ready", ACCENT, Color::Black, false);
}

fn paint_env_body(ui: &mut Ui<'_>) {
    paint_str(ui, 8, 6, "Key", MUTED, Color::Black, false);
    paint_str(ui, 28, 6, "Source", MUTED, Color::Black, false);
    paint_str(ui, 58, 6, "Value", MUTED, Color::Black, false);

    paint_str(ui, 8, 7, "GH_TOKEN", Color::White, BADGE_BG, true);
    paint_str(ui, 28, 7, "1Password", SECONDARY, Color::Black, false);
    paint_str(ui, 58, 7, "••••••••", MUTED, Color::Black, false);
}

fn paint_agents_body(ui: &mut Ui<'_>) {
    paint_str(ui, 8, 6, "Configured agents", Color::White, Color::Black, true);
    paint_str(ui, 8, 7, "Claude Code", SECONDARY, Color::Black, false);
    paint_str(ui, 8, 8, "Codex", SECONDARY, Color::Black, false);
}

fn paint_trust_body(ui: &mut Ui<'_>) {
    paint_str(ui, 8, 6, "Trusted role sources", Color::White, Color::Black, true);
    paint_str(ui, 8, 7, "github.com/chainargos/roles", SECONDARY, Color::Black, false);
}

fn paint_save_preview(ui: &mut Ui<'_>, full: Rect) {
    let card_w = 66u16;
    let card_h = 11u16;
    let x = full.x.saturating_add(full.width.saturating_sub(card_w) / 2);
    let y = full.y.saturating_add(full.height.saturating_sub(card_h) / 2);
    let card = Rect::new(x, y, card_w, card_h);

    ui.add_dim(full);

    let bg = Color::Rgb(0x18, 0x18, 0x1b);
    let border = Color::Rgb(0x4d, 0x4d, 0x4d);
    ui.fill(card, Style::default().bg(bg));

    paint_str(ui, x.saturating_add(3), y.saturating_add(1), "Save settings", Color::White, bg, true);
    paint_str(
        ui,
        x.saturating_add(3),
        y.saturating_add(3),
        "Scope · global config",
        SECONDARY,
        bg,
        false,
    );
    paint_str(
        ui,
        x.saturating_add(3),
        y.saturating_add(5),
        "Changes · trust draft",
        AMBER,
        bg,
        false,
    );

    paint_str(ui, x.saturating_add(card_w.saturating_sub(22)), y.saturating_add(8), "Cancel", SECONDARY, bg, false);
    paint_str(
        ui,
        x.saturating_add(card_w.saturating_sub(12)),
        y.saturating_add(8),
        "Save",
        Color::White,
        Color::Rgb(0x27, 0x27, 0x2a),
        true,
    );

    for col in x..x.saturating_add(card_w) {
        paint_cell(ui, col, y, "─", border, bg, false);
        paint_cell(ui, col, y.saturating_add(card_h).saturating_sub(1), "─", border, bg, false);
    }
    for row in y..y.saturating_add(card_h) {
        paint_cell(ui, x, row, "│", border, bg, false);
        paint_cell(ui, x.saturating_add(card_w).saturating_sub(1), row, "│", border, bg, false);
    }
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
