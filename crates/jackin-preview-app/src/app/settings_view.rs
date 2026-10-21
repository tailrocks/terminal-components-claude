//! Live settings screen rendering with termrock widgets and components.
//! Replaces historical_settings_*.rs replay modules.

use termrock::author::PaintStyle;
use termrock::{Color, Modifier, Position, Rect, Style, Ui};

use super::App;

const ACCENT: Color = Color::Rgb(0x48, 0xe0, 0x54);
const BRAND_FG: Color = Color::Rgb(0x19, 0x19, 0x1c);
const SECONDARY: Color = Color::Rgb(0xb3, 0xb3, 0xb3);
const MUTED: Color = Color::Rgb(0x8a, 0x8a, 0x8a);
const DARK_MUTED: Color = Color::Rgb(0x73, 0x73, 0x73);
const SEAM: Color = Color::Rgb(0x3a, 0x3a, 0x3a);
const ACTIVE_BG: Color = Color::Rgb(0x27, 0x27, 0x2a);
const BADGE_BG: Color = Color::Rgb(0x17, 0x3f, 0x1c);
const AMBER: Color = Color::Rgb(0xf5, 0x9e, 0x09);
const BLACK: Color = Color::Rgb(0, 0, 0);
const WHITE: Color = Color::Rgb(255, 255, 255);

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
    paint_str(ui, 12, 0, " File ", SECONDARY, BLACK, false);
    paint_str(ui, 19, 0, " Go ", SECONDARY, BLACK, false);
    paint_str(ui, 24, 0, " Help ", SECONDARY, BLACK, false);

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
    paint_str(ui, crumb_x, 0, &crumb, SECONDARY, BLACK, false);

    if full.width >= 120 {
        paint_str(ui, 85, 0, "inside the Construct", SECONDARY, BLACK, false);
        paint_str(ui, 107, 0, "●", ACCENT, BLACK, false);
        paint_str(ui, 109, 0, "2 running", SECONDARY, BLACK, false);
    }

    // Row 3: Tabs
    let active_index = (app.settings.tab as usize).min(TABS.len().saturating_sub(1));
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
    paint_str(ui, 4, 6, "Runtime mode", SECONDARY, BLACK, false);
    paint_str(ui, 4, 7, "Sync host credentials", WHITE, BLACK, false);

    paint_str(ui, 4, 10, "Workspace", SECONDARY, BLACK, false);
    paint_str(ui, 4, 11, "payments-platform", WHITE, BLACK, false);

    paint_str(ui, 4, 14, "DCO signoff", SECONDARY, BLACK, false);
    paint_str(ui, 4, 15, "enabled", WHITE, BLACK, false);

    paint_str(ui, 4, 18, "Secret policy", SECONDARY, BLACK, false);
    paint_str(
        ui,
        4,
        19,
        "references only; resolved bytes are transient",
        WHITE,
        BLACK,
        false,
    );

    paint_str(ui, 4, 22, "Trust", SECONDARY, BLACK, false);
    let trust_label = if app.trusted {
        "trusted · sources verified"
    } else {
        "untrusted · explicit prompt"
    };
    paint_str(ui, 4, 23, trust_label, WHITE, BLACK, false);
}

fn paint_mounts_body(ui: &mut Ui<'_>) {
    paint_str(ui, 8, 6, "Destination", MUTED, BLACK, false);
    paint_str(ui, 39, 6, "Scope", MUTED, BLACK, false);
    paint_str(ui, 55, 6, "Mode", MUTED, BLACK, false);
    paint_str(ui, 61, 6, "Isolation", MUTED, BLACK, false);
    paint_str(ui, 72, 6, "Kind", MUTED, BLACK, false);
    paint_str(ui, 80, 6, "Source", MUTED, BLACK, false);

    // Row 0 (selected)
    paint_str(ui, 2, 7, "▎›", ACCENT, BADGE_BG, true);
    paint_str(
        ui,
        4,
        7,
        "    /home/agent/.gitconfig         ",
        WHITE,
        BADGE_BG,
        true,
    );
    paint_str(ui, 39, 7, "global        ", SECONDARY, BADGE_BG, true);
    paint_str(ui, 55, 7, "ro  ", SECONDARY, BADGE_BG, true);
    paint_str(ui, 61, 7, "shared   ", SECONDARY, BADGE_BG, true);
    paint_str(ui, 72, 7, "host  ", SECONDARY, BADGE_BG, true);
    paint_str(
        ui,
        80,
        7,
        "~/.gitconfig                         ",
        SECONDARY,
        BADGE_BG,
        true,
    );

    // Row 1
    paint_str(
        ui,
        4,
        8,
        "    /home/agent/.cargo/registry    ",
        WHITE,
        BLACK,
        false,
    );
    paint_str(ui, 39, 8, "global        ", MUTED, BLACK, false);
    paint_str(ui, 55, 8, "rw  ", SECONDARY, BLACK, false);
    paint_str(ui, 61, 8, "shared   ", SECONDARY, BLACK, false);
    paint_str(ui, 72, 8, "host  ", MUTED, BLACK, false);
    paint_str(
        ui,
        80,
        8,
        "~/.cache/cargo-registry              ",
        MUTED,
        BLACK,
        false,
    );

    // Row 2
    paint_str(
        ui,
        4,
        9,
        "    /home/agent/.kube              ",
        WHITE,
        BLACK,
        false,
    );
    paint_str(ui, 39, 9, "role sre      ", MUTED, BLACK, false);
    paint_str(ui, 55, 9, "ro  ", SECONDARY, BLACK, false);
    paint_str(ui, 61, 9, "shared   ", SECONDARY, BLACK, false);
    paint_str(ui, 72, 9, "host  ", MUTED, BLACK, false);
    paint_str(
        ui,
        80,
        9,
        "~/roles/sre-kube                     ",
        MUTED,
        BLACK,
        false,
    );

    // Row 3
    paint_str(ui, 8, 10, "+ Add mount", SECONDARY, BLACK, false);
}

fn paint_env_body(ui: &mut Ui<'_>) {
    paint_str(ui, 8, 6, "Key", MUTED, BLACK, false);
    paint_str(ui, 28, 6, "Source", MUTED, BLACK, false);
    paint_str(ui, 58, 6, "Value", MUTED, BLACK, false);

    paint_str(ui, 8, 7, "GH_TOKEN", WHITE, BADGE_BG, true);
    paint_str(ui, 28, 7, "1Password", SECONDARY, BLACK, false);
    paint_str(ui, 58, 7, "••••••••", MUTED, BLACK, false);
}

fn paint_agents_body(ui: &mut Ui<'_>) {
    paint_str(ui, 4, 6, "Agent runtime mode", SECONDARY, BLACK, true);
    paint_str(
        ui,
        77,
        6,
        "accounts are registered in Accounts (c)",
        DARK_MUTED,
        BLACK,
        false,
    );
    paint_str(
        ui,
        7,
        7,
        "Agent         Mode         Registry default",
        MUTED,
        BLACK,
        false,
    );

    // Row 0 (selected)
    paint_str(ui, 2, 8, "▎›", ACCENT, BADGE_BG, true);
    paint_str(ui, 6, 8, " Claude Code   ", WHITE, BADGE_BG, true);
    paint_str(ui, 21, 8, "sync        ", SECONDARY, BADGE_BG, true);
    paint_str(ui, 34, 8, "★ Claude · Personal", SECONDARY, BADGE_BG, true);

    // Row 1
    paint_str(ui, 6, 9, " Codex         ", WHITE, BLACK, false);
    paint_str(ui, 21, 9, "api key     ", SECONDARY, BLACK, false);
    paint_str(ui, 34, 9, "★ Codex · Primary", MUTED, BLACK, false);

    // Row 2
    paint_str(ui, 6, 10, " Amp           ", WHITE, BLACK, false);
    paint_str(ui, 21, 10, "sync        ", SECONDARY, BLACK, false);
    paint_str(
        ui,
        34,
        10,
        "discovered · ~/.config/amp/secrets.json",
        MUTED,
        BLACK,
        false,
    );

    // Row 3
    paint_str(ui, 6, 11, " Kimi Code     ", WHITE, BLACK, false);
    paint_str(ui, 21, 11, "ignore      ", SECONDARY, BLACK, false);
    paint_str(
        ui,
        34,
        11,
        "no credentials handed to the container",
        MUTED,
        BLACK,
        false,
    );

    // Row 4
    paint_str(ui, 6, 12, " OpenCode      ", WHITE, BLACK, false);
    paint_str(ui, 21, 12, "sync        ", SECONDARY, BLACK, false);
    paint_str(
        ui,
        34,
        12,
        "★ OpenCode · Go subscription",
        MUTED,
        BLACK,
        false,
    );

    // Row 5
    paint_str(ui, 6, 13, " Grok Build    ", WHITE, BLACK, false);
    paint_str(ui, 21, 13, "api key     ", SECONDARY, BLACK, false);
    paint_str(ui, 34, 13, "★ Grok · Team", MUTED, BLACK, false);
}

fn paint_trust_body(ui: &mut Ui<'_>) {
    paint_str(ui, 4, 6, "Role sources", SECONDARY, BLACK, true);
    paint_str(
        ui,
        93,
        6,
        "3 trusted · 1 untrusted",
        DARK_MUTED,
        BLACK,
        false,
    );

    // Row 0 (selected)
    paint_str(ui, 2, 7, "▎›", ACCENT, BADGE_BG, true);
    paint_str(
        ui,
        6,
        7,
        " github.com/chainargos/roles               ",
        WHITE,
        BADGE_BG,
        true,
    );
    paint_str(ui, 49, 7, "git", SECONDARY, BADGE_BG, true);
    paint_str(ui, 54, 7, " [✓] trusted    ", WHITE, BADGE_BG, true);
    paint_str(ui, 70, 7, "4", SECONDARY, BADGE_BG, true);
    paint_str(ui, 72, 7, "roles", SECONDARY, BADGE_BG, true);

    // Row 1
    paint_str(
        ui,
        6,
        8,
        " github.com/acme-labs/roles-experimental   ",
        WHITE,
        BLACK,
        false,
    );
    paint_str(ui, 49, 8, "git  ", MUTED, BLACK, false);
    paint_str(ui, 55, 8, "[ ] untrusted", AMBER, BLACK, false);
    paint_str(ui, 70, 8, "1 role", DARK_MUTED, BLACK, false);

    // Row 2
    paint_str(
        ui,
        6,
        9,
        " ~/roles                                   ",
        WHITE,
        BLACK,
        false,
    );
    paint_str(ui, 49, 9, "path ", MUTED, BLACK, false);
    paint_str(ui, 54, 9, " [✓] trusted    ", WHITE, BLACK, false);
    paint_str(ui, 70, 9, "1 role", DARK_MUTED, BLACK, false);

    // Row 3
    paint_str(
        ui,
        6,
        10,
        " git@corp:infra/roles                      ",
        WHITE,
        BLACK,
        false,
    );
    paint_str(ui, 49, 10, "git  ", MUTED, BLACK, false);
    paint_str(ui, 54, 10, " [✓] trusted    ", WHITE, BLACK, false);
    paint_str(ui, 70, 10, "2 roles", DARK_MUTED, BLACK, false);
}

fn paint_save_preview(ui: &mut Ui<'_>, full: Rect) {
    let card_w = 66u16;
    let card_h = 14u16;
    let x = full.x.saturating_add(full.width.saturating_sub(card_w) / 2);
    let y = full
        .y
        .saturating_add(full.height.saturating_sub(card_h) / 2);

    let bg = Color::Rgb(0x18, 0x18, 0x1b);
    let border = Color::Rgb(0x4d, 0x4d, 0x4d);

    // Card fill
    ui.fill(Rect::new(x, y, card_w, card_h), Style::default().bg(bg));

    // Top border (y)
    paint_str(
        ui,
        x,
        y,
        "╭──────────────────────────────────────────────────────────── ",
        border,
        bg,
        false,
    );
    paint_styled(ui, x.saturating_add(62), y, "×", SECONDARY, bg, false, false);
    paint_str(ui, x.saturating_add(63), y, " ─╮", border, bg, false);

    // Left and right borders for content rows
    for row in 1..card_h.saturating_sub(1) {
        paint_cell(ui, x, y.saturating_add(row), "│", border, bg, false);
        paint_cell(
            ui,
            x.saturating_add(card_w).saturating_sub(1),
            y.saturating_add(row),
            "│",
            border,
            bg,
            false,
        );
    }

    // Row y + 2: Header
    paint_styled(
        ui,
        x.saturating_add(3),
        y.saturating_add(2),
        "Save settings",
        WHITE,
        bg,
        true,
        false,
    );

    // Row y + 4: Scope
    paint_styled(
        ui,
        x.saturating_add(2),
        y.saturating_add(4),
        "Scope",
        MUTED,
        bg,
        false,
        false,
    );
    paint_styled(
        ui,
        x.saturating_add(11),
        y.saturating_add(4),
        "global config · ~/.jackin/config.toml",
        WHITE,
        bg,
        false,
        false,
    );

    // Row y + 5: 2 changes
    paint_styled(
        ui,
        x.saturating_add(2),
        y.saturating_add(5),
        "2 changes",
        WHITE,
        bg,
        false,
        false,
    );

    // Row y + 6: Trust
    paint_styled(
        ui,
        x.saturating_add(2),
        y.saturating_add(6),
        "Trust",
        MUTED,
        bg,
        false,
        false,
    );
    paint_styled(
        ui,
        x.saturating_add(11),
        y.saturating_add(6),
        "github.com/chainargos/roles → untrusted · github.c…",
        WHITE,
        bg,
        false,
        false,
    );

    // Row y + 8 & y + 9: Details
    paint_styled(
        ui,
        x.saturating_add(3),
        y.saturating_add(8),
        "~ trust github.com/chainargos/roles true → false",
        SECONDARY,
        bg,
        false,
        false,
    );
    paint_styled(
        ui,
        x.saturating_add(3),
        y.saturating_add(9),
        "~ trust github.com/acme-labs/roles-experimental false → true",
        SECONDARY,
        bg,
        false,
        false,
    );

    // Row y + 11: Buttons
    paint_styled(
        ui,
        x.saturating_add(47),
        y.saturating_add(11),
        "▎",
        ACCENT,
        ACTIVE_BG,
        false,
        false,
    );
    paint_styled(
        ui,
        x.saturating_add(48),
        y.saturating_add(11),
        "C",
        WHITE,
        ACTIVE_BG,
        true,
        true,
    );
    paint_styled(
        ui,
        x.saturating_add(49),
        y.saturating_add(11),
        "ancel ",
        WHITE,
        ACTIVE_BG,
        true,
        false,
    );

    paint_styled(
        ui,
        x.saturating_add(57),
        y.saturating_add(11),
        " ",
        ACCENT,
        ACCENT,
        false,
        false,
    );
    paint_styled(
        ui,
        x.saturating_add(58),
        y.saturating_add(11),
        "S",
        BRAND_FG,
        ACCENT,
        true,
        true,
    );
    paint_styled(
        ui,
        x.saturating_add(59),
        y.saturating_add(11),
        "ave ",
        BRAND_FG,
        ACCENT,
        true,
        false,
    );

    // Bottom border (y + 13)
    paint_str(
        ui,
        x,
        y.saturating_add(card_h).saturating_sub(1),
        "╰────────────────────────────────────────────────────────────────╯",
        border,
        bg,
        false,
    );
}

fn paint_str(ui: &mut Ui<'_>, x: u16, y: u16, text: &str, fg: Color, bg: Color, bold: bool) {
    let mut style = PaintStyle::new().fg(fg).bg(bg);
    if bold {
        style = style.add_modifier(Modifier::BOLD);
    }
    ui.paint_str(Rect::new(x, y, text.chars().count() as u16, 1), text, style);
}

fn paint_styled(
    ui: &mut Ui<'_>,
    x: u16,
    y: u16,
    text: &str,
    fg: Color,
    bg: Color,
    bold: bool,
    underline: bool,
) {
    let mut style = PaintStyle::new().fg(fg).bg(bg);
    let mut modifier = Modifier::empty();
    if bold {
        modifier |= Modifier::BOLD;
    }
    if underline {
        modifier |= Modifier::UNDERLINED;
    }
    if !modifier.is_empty() {
        style = style.add_modifier(modifier);
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
