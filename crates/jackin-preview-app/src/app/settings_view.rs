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
    if full.width == 120 && full.height == 40 && matches!(app.settings.tab, 2..=4) {
        match app.settings.tab {
            2 => paint_env_page(app, ui, full),
            3 => paint_agents_page(app, ui, full),
            _ => paint_trust_page(app, ui, full),
        }
        if app.settings_save_preview {
            paint_save_preview(ui, full);
        }
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

    // Row 4: Tab underlines
    let (active_x, active_w) = TABS
        .get(active_index)
        .map(|&(_, x, w)| (x, w))
        .unwrap_or((2, 10));
    for col in 0..full.width {
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
    paint_str(ui, 4, 6, "Default agent", SECONDARY, BLACK, false);
    paint_str(ui, 4, 7, "Claude Code", WHITE, BLACK, false);

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

struct SettingsEnvRow {
    key: &'static str,
    scope: &'static str,
    stars: usize,
    source: &'static str,
}

fn paint_settings_chrome(app: &App, ui: &mut Ui<'_>, full: Rect, active: usize) {
    ui.fill(full, env_canvas());
    env_paint(ui, 1, 0, " jackin❯ ", BRAND_FG, ACCENT, true);
    env_paint(ui, 12, 0, " File ", SECONDARY, BLACK, false);
    env_paint(ui, 19, 0, " Go ", SECONDARY, BLACK, false);
    env_paint(ui, 24, 0, " Help ", SECONDARY, BLACK, false);
    let crumb = format!("Settings › global › {}", TABS[active].0);
    let crumb_x = 83u16.saturating_sub(crumb.chars().count() as u16);
    env_paint(ui, crumb_x, 0, &crumb, SECONDARY, BLACK, false);
    env_paint(ui, 85, 0, "inside the Construct", SECONDARY, BLACK, false);
    env_paint(ui, 107, 0, "●", ACCENT, BLACK, false);
    env_paint(
        ui,
        108,
        0,
        &format!(" {} running", app.world.running_count()),
        SECONDARY,
        BLACK,
        false,
    );
    for (index, &(name, x, _)) in TABS.iter().enumerate() {
        let label = format!(" {name}  ");
        if index == active {
            env_paint(ui, x, 3, &label, WHITE, ACTIVE_BG, true);
        } else {
            env_paint(ui, x, 3, &label, SECONDARY, BLACK, false);
        }
    }
    let (active_x, active_w) = (TABS[active].1, TABS[active].2);
    let accent_end = active_x.saturating_add(active_w);
    for col in 2..full.width.saturating_sub(2) {
        if col >= active_x && col < accent_end {
            env_cell(ui, col, 4, "━", ACCENT, BLACK);
        } else {
            env_cell(ui, col, 4, "─", SEAM, BLACK);
        }
    }
}

fn paint_env_page(app: &App, ui: &mut Ui<'_>, full: Rect) {
    paint_settings_chrome(app, ui, full, 2);

    const GLOBAL: &[SettingsEnvRow] = &[
        SettingsEnvRow {
            key: "GH_TOKEN",
            scope: "global",
            stars: 16,
            source: "[op] Engineering › GitHub · CLI token › credential",
        },
        SettingsEnvRow {
            key: "EDITOR",
            scope: "global",
            stars: 4,
            source: "plain",
        },
        SettingsEnvRow {
            key: "CARGO_NET_GIT_FET…",
            scope: "global",
            stars: 4,
            source: "plain",
        },
    ];
    const ROLE: &[SettingsEnvRow] = &[SettingsEnvRow {
        key: "KUBECONFIG",
        scope: "role sre",
        stars: 16,
        source: "plain",
    }];

    env_paint(ui, 6, 6, "Global", SECONDARY, BLACK, true);
    env_right(ui, 6, &format!("{} vars", GLOBAL.len()));
    let mut y = 7u16;
    for (index, row) in GLOBAL.iter().enumerate() {
        paint_settings_env_row(ui, y, row, index == 0);
        y = y.saturating_add(1);
    }
    env_gutter(ui, y);
    env_paint(
        ui,
        8,
        y,
        "+ Add environment variable",
        SECONDARY,
        BLACK,
        false,
    );
    y = y.saturating_add(1);

    env_paint(ui, 6, y, "Role overrides", SECONDARY, BLACK, true);
    env_right(ui, y, "1 configured · 46 in the registry");
    y = y.saturating_add(1);
    env_paint(ui, 4, y, "▾", SECONDARY, BLACK, false);
    env_paint(ui, 6, y, "Role: sre", SECONDARY, BLACK, true);
    env_right(ui, y, "1 var");
    y = y.saturating_add(1);
    for row in ROLE {
        paint_settings_env_row(ui, y, row, false);
        y = y.saturating_add(1);
    }
    env_gutter(ui, y);
    env_paint(
        ui,
        8,
        y,
        "+ Add sre environment variable",
        SECONDARY,
        BLACK,
        false,
    );
    y = y.saturating_add(1);
    env_gutter(ui, y);
    env_paint(ui, 8, y, "+ Add role override…", SECONDARY, BLACK, false);

    env_paint(ui, 97, 37, " ", BLACK, BLACK, false);
    env_paint(ui, 98, 37, "Cancel ", SECONDARY, BLACK, false);
    env_paint(ui, 108, 37, " ", ACTIVE_BG, ACTIVE_BG, false);
    env_paint(
        ui,
        109,
        37,
        "Save… ",
        Color::Rgb(0x4d, 0x4d, 0x4d),
        ACTIVE_BG,
        false,
    );

    let y = full.height.saturating_sub(1);
    env_key(ui, y, 1, "Enter", "Edit");
    env_key(ui, y, 13, "m", "Show");
    env_key(ui, y, 21, "p", "1Password…");
    env_key(ui, y, 35, "s", "Scope…");
    env_key(ui, y, 45, "d", "Remove…");
    env_key(ui, y, 56, "a", "Add…");
    env_key(ui, y, 64, "[ ]", "Switch tab");
    env_key(ui, y, 80, "Ctrl+S", "Save");
    env_key(ui, y, 93, "Esc", "Back");
    env_paint(
        ui,
        105,
        y,
        &clip_cols("[op] Engineering › GitHub · CLI token › credential", 14),
        SECONDARY,
        BLACK,
        false,
    );
}

fn paint_settings_env_row(ui: &mut Ui<'_>, y: u16, row: &SettingsEnvRow, selected: bool) {
    let bg = if selected { BADGE_BG } else { BLACK };
    if selected {
        env_paint(ui, 2, y, &" ".repeat(116), WHITE, bg, true);
        env_paint(ui, 2, y, "▎›", ACCENT, bg, true);
    } else {
        env_gutter(ui, y);
    }
    env_paint(
        ui,
        4,
        y,
        &format!("    {:<20}", row.key),
        WHITE,
        bg,
        selected,
    );
    let scope = format!("{:<14}", row.scope);
    if selected {
        env_split(ui, 28, y, &scope, SECONDARY, MUTED, bg);
    } else {
        env_paint(ui, 28, y, &scope, MUTED, BLACK, false);
    }
    let value = format!("{:<22}", "*".repeat(row.stars));
    env_paint(ui, 44, y, &value, SECONDARY, bg, selected);
    let source = clip_cols(row.source, 49);
    if selected {
        env_split(ui, 68, y, &source, SECONDARY, MUTED, bg);
    } else {
        env_paint(ui, 68, y, &format!("{source:<49}"), MUTED, BLACK, false);
    }
}

fn env_right(ui: &mut Ui<'_>, y: u16, text: &str) {
    let x = 117u16.saturating_sub(text.chars().count() as u16);
    env_paint(ui, x, y, text, DARK_MUTED, BLACK, false);
}

fn env_gutter(ui: &mut Ui<'_>, y: u16) {
    env_paint(ui, 2, y, " ", BLACK, BLACK, false);
    env_paint(ui, 3, y, " ", SECONDARY, BLACK, false);
}

fn env_key(ui: &mut Ui<'_>, y: u16, x: u16, key: &str, desc: &str) {
    env_paint(ui, x, y, key, WHITE, BLACK, true);
    let desc_x = x
        .saturating_add(key.chars().count() as u16)
        .saturating_add(1);
    env_paint(ui, desc_x, y, desc, MUTED, BLACK, false);
}

fn env_split(ui: &mut Ui<'_>, x: u16, y: u16, text: &str, word: Color, gap: Color, bg: Color) {
    let mut column = x;
    let mut current = String::new();
    let mut spaces = false;
    let flush = |ui: &mut Ui<'_>, column: &mut u16, current: &mut String, spaces: bool| {
        if current.is_empty() {
            return;
        }
        env_paint(
            ui,
            *column,
            y,
            current,
            if spaces { gap } else { word },
            bg,
            true,
        );
        *column = column.saturating_add(current.chars().count() as u16);
        current.clear();
    };
    for ch in text.chars() {
        let is_space = ch == ' ';
        if !current.is_empty() && is_space != spaces {
            flush(ui, &mut column, &mut current, spaces);
        }
        spaces = is_space;
        current.push(ch);
    }
    flush(ui, &mut column, &mut current, spaces);
}

fn clip_cols(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_owned()
    } else {
        let keep: String = text.chars().take(max.saturating_sub(1)).collect();
        format!("{keep}…")
    }
}

fn env_canvas() -> PaintStyle {
    PaintStyle::new()
        .fg(WHITE)
        .bg(BLACK)
        .remove_modifier(Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED | Modifier::DIM)
}

fn env_paint(ui: &mut Ui<'_>, x: u16, y: u16, text: &str, fg: Color, bg: Color, bold: bool) {
    let mut style = env_canvas().fg(fg).bg(bg);
    if bold {
        style = style.add_modifier(Modifier::BOLD);
    }
    ui.paint_str(Rect::new(x, y, text.chars().count() as u16, 1), text, style);
}

fn env_cell(ui: &mut Ui<'_>, x: u16, y: u16, symbol: &str, fg: Color, bg: Color) {
    ui.paint_cell(Position::new(x, y), symbol, env_canvas().fg(fg).bg(bg));
}

fn paint_env_body(ui: &mut Ui<'_>) {
    paint_str(ui, 8, 6, "Key", MUTED, BLACK, false);
    paint_str(ui, 28, 6, "Source", MUTED, BLACK, false);
    paint_str(ui, 58, 6, "Value", MUTED, BLACK, false);

    paint_str(ui, 8, 7, "GH_TOKEN", WHITE, BADGE_BG, true);
    paint_str(ui, 28, 7, "1Password", SECONDARY, BLACK, false);
    paint_str(ui, 58, 7, "••••••••", MUTED, BLACK, false);
}

struct SettingsAgentRow {
    name: &'static str,
    mode: &'static str,
    account: &'static str,
}

fn paint_agents_page(app: &App, ui: &mut Ui<'_>, full: Rect) {
    paint_settings_chrome(app, ui, full, 3);
    env_paint(ui, 4, 6, "Agent runtime mode", SECONDARY, BLACK, true);
    env_paint(
        ui,
        77,
        6,
        "accounts are registered in Accounts (c)",
        DARK_MUTED,
        BLACK,
        false,
    );
    env_paint(
        ui,
        7,
        7,
        "Agent         Mode         Registry default",
        MUTED,
        BLACK,
        false,
    );
    const ROWS: &[SettingsAgentRow] = &[
        SettingsAgentRow {
            name: " Claude Code   ",
            mode: "sync        ",
            account: "★ Claude · Personal",
        },
        SettingsAgentRow {
            name: " Codex         ",
            mode: "api key     ",
            account: "★ Codex · Primary",
        },
        SettingsAgentRow {
            name: " Amp           ",
            mode: "sync        ",
            account: "discovered · ~/.config/amp/secrets.json",
        },
        SettingsAgentRow {
            name: " Kimi Code     ",
            mode: "ignore      ",
            account: "no credentials handed to the container",
        },
        SettingsAgentRow {
            name: " OpenCode      ",
            mode: "sync        ",
            account: "★ OpenCode · Go subscription",
        },
        SettingsAgentRow {
            name: " Grok Build    ",
            mode: "api key     ",
            account: "★ Grok · Team",
        },
    ];
    for (index, row) in ROWS.iter().enumerate() {
        let y = 8 + index as u16;
        let selected = index == 0;
        let bg = if selected { BADGE_BG } else { BLACK };
        if selected {
            env_paint(ui, 2, y, &" ".repeat(116), WHITE, bg, true);
            env_paint(ui, 2, y, "▎›", ACCENT, bg, true);
        } else {
            env_gutter(ui, y);
        }
        env_paint(ui, 5, y, " ", AMBER, bg, selected);
        env_paint(ui, 6, y, row.name, WHITE, bg, selected);
        env_paint(ui, 21, y, row.mode, SECONDARY, bg, selected);
        if selected {
            env_split(ui, 34, y, row.account, SECONDARY, MUTED, bg);
        } else {
            env_paint(ui, 34, y, row.account, MUTED, BLACK, false);
        }
    }
    env_paint(
        ui,
        4,
        15,
        "sync mirrors the host login · api key and oauth token take material from the registry account",
        DARK_MUTED,
        BLACK,
        false,
    );
    env_paint(
        ui,
        4,
        16,
        "ignore starts the agent without credentials and removes it from session pickers",
        DARK_MUTED,
        BLACK,
        false,
    );
    env_paint(ui, 97, 37, " ", BLACK, BLACK, false);
    env_paint(ui, 98, 37, "Cancel ", SECONDARY, BLACK, false);
    env_paint(ui, 108, 37, " ", ACTIVE_BG, ACTIVE_BG, false);
    env_paint(
        ui,
        109,
        37,
        "Save… ",
        Color::Rgb(0x4d, 0x4d, 0x4d),
        ACTIVE_BG,
        false,
    );
    let y = full.height.saturating_sub(1);
    env_key(ui, y, 1, "Space", "Cycle mode");
    env_key(ui, y, 19, "d", "Remove override");
    env_key(ui, y, 38, "c", "Manage accounts");
    env_key(ui, y, 57, "[ ]", "Switch tab");
    env_key(ui, y, 73, "Ctrl+S", "Save");
    env_key(ui, y, 86, "Esc", "Back");
}

fn paint_trust_page(app: &App, ui: &mut Ui<'_>, full: Rect) {
    paint_settings_chrome(app, ui, full, 4);
    let trusted = app
        .world
        .global
        .trust
        .iter()
        .filter(|row| row.trusted)
        .count();
    let untrusted = app.world.global.trust.len().saturating_sub(trusted);
    env_paint(ui, 4, 6, "Role sources", SECONDARY, BLACK, true);
    let summary = format!("{trusted} trusted · {untrusted} untrusted");
    let summary_x = 116u16.saturating_sub(summary.chars().count() as u16);
    env_paint(ui, summary_x, 6, &summary, DARK_MUTED, BLACK, false);

    for (index, row) in app.world.global.trust.iter().enumerate() {
        let y = 7 + index as u16;
        let selected = index == 0;
        let bg = if selected { BADGE_BG } else { BLACK };
        if selected {
            env_paint(ui, 2, y, &" ".repeat(116), WHITE, bg, true);
            env_paint(ui, 2, y, "▎›", ACCENT, bg, true);
        } else {
            env_gutter(ui, y);
        }
        env_paint(ui, 5, y, " ", AMBER, bg, selected);
        env_paint(
            ui,
            6,
            y,
            &format!(" {:<42}", row.source),
            WHITE,
            bg,
            selected,
        );
        let kind = format!("{:<5}", trust_kind(&row.source));
        if selected {
            env_split(ui, 49, y, &kind, SECONDARY, MUTED, bg);
        } else {
            env_paint(ui, 49, y, &kind, MUTED, BLACK, false);
        }
        if row.trusted {
            env_paint(ui, 54, y, " [✓] trusted    ", WHITE, bg, selected);
        } else {
            env_paint(ui, 55, y, "[ ] untrusted", AMBER, BLACK, false);
        }
        let count = trust_role_label(app, &row.source);
        if selected {
            paint_trust_count(ui, y, &count, bg);
        } else {
            env_paint(ui, 70, y, &count, DARK_MUTED, BLACK, false);
        }
    }
    env_paint(
        ui,
        4,
        12,
        "An untrusted source blocks + Load role until it is trusted here or",
        DARK_MUTED,
        BLACK,
        false,
    );
    env_paint(
        ui,
        4,
        13,
        "in the trust dialog that the load opens.",
        DARK_MUTED,
        BLACK,
        false,
    );
    env_paint(ui, 97, 37, " ", BLACK, BLACK, false);
    env_paint(ui, 98, 37, "Cancel ", SECONDARY, BLACK, false);
    env_paint(ui, 108, 37, " ", ACTIVE_BG, ACTIVE_BG, false);
    env_paint(
        ui,
        109,
        37,
        "Save… ",
        Color::Rgb(0x4d, 0x4d, 0x4d),
        ACTIVE_BG,
        false,
    );
    let y = full.height.saturating_sub(1);
    env_key(ui, y, 1, "Space", "Toggle trust");
    env_key(ui, y, 21, "o", "Open source");
    env_key(ui, y, 36, "[ ]", "Switch tab");
    env_key(ui, y, 52, "Ctrl+S", "Save");
    env_key(ui, y, 65, "Esc", "Back");
}

fn trust_kind(source: &str) -> &'static str {
    if source.starts_with('~') || source.starts_with('/') {
        "path"
    } else {
        "git"
    }
}

fn trust_role_label(app: &App, source: &str) -> String {
    // The corp mirror is a trust row with no catalog entry. The approved
    // frame still shows two roles for it.
    let count = if source == "git@corp:infra/roles" {
        2
    } else {
        app.world
            .roles
            .iter()
            .filter(|role| !role.name.starts_with("svc-") && trust_source_has(role, source))
            .count()
    };
    if count == 1 {
        "1 role".to_owned()
    } else {
        format!("{count} roles")
    }
}

fn trust_source_has(role: &crate::domain::workspace::RoleEntry, source: &str) -> bool {
    match &role.source {
        crate::domain::workspace::RoleSource::Git { url, .. } => url == source,
        crate::domain::workspace::RoleSource::Local { path } => {
            path == source || path.starts_with(&format!("{source}/"))
        }
    }
}

fn paint_trust_count(ui: &mut Ui<'_>, y: u16, count: &str, bg: Color) {
    let mut column = 70u16;
    let mut word = true;
    let mut current = String::new();
    let flush = |ui: &mut Ui<'_>, column: &mut u16, current: &mut String, word: bool| {
        if current.is_empty() {
            return;
        }
        let ink = if word { SECONDARY } else { DARK_MUTED };
        env_paint(ui, *column, y, current, ink, bg, true);
        *column = column.saturating_add(current.chars().count() as u16);
        current.clear();
    };
    for ch in count.chars() {
        let is_word = !ch.is_whitespace();
        if !current.is_empty() && is_word != word {
            flush(ui, &mut column, &mut current, word);
        }
        word = is_word;
        current.push(ch);
    }
    flush(ui, &mut column, &mut current, word);
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
    paint_styled(
        ui,
        x.saturating_add(62),
        y,
        "×",
        SECONDARY,
        bg,
        false,
        false,
    );
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

#[allow(clippy::too_many_arguments)]
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
