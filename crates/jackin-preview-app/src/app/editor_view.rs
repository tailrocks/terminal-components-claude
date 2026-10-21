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
const DARK_MUTED: Color = Color::Rgb(0x73, 0x73, 0x73);
const SEAM: Color = Color::Rgb(0x3a, 0x3a, 0x3a);
const ACTIVE_BG: Color = Color::Rgb(0x27, 0x27, 0x2a);
const BADGE_BG: Color = Color::Rgb(0x17, 0x3f, 0x1c);
const AMBER: Color = Color::Rgb(0xf5, 0x9e, 0x09);
const BLACK: Color = Color::Rgb(0, 0, 0);
const WHITE: Color = Color::Rgb(255, 255, 255);

const TABS: &[(&str, u16, u16)] = &[
    ("General", 2, 11),
    ("Mounts", 13, 10),
    ("Roles", 23, 9),
    ("Environments", 32, 16),
    ("Accounts", 48, 12),
];

pub(crate) fn paint(app: &App, ui: &mut Ui<'_>, full: Rect) {
    if full.width == 0 || full.height == 0 {
        return;
    }
    let general = matches!(app.editor.tab, EditorTab::General);
    // The live shell is surface ink. The approved general page is canvas.
    if general {
        ui.fill(full, canvas());
    }

    // Row 0: Header
    paint_str(ui, 1, 0, " jackin❯ ", BRAND_FG, ACCENT, true);
    paint_str(ui, 12, 0, " File ", SECONDARY, BLACK, false);
    paint_str(ui, 19, 0, " Go ", SECONDARY, BLACK, false);
    paint_str(ui, 24, 0, " Help ", SECONDARY, BLACK, false);

    if general {
        let name = app
            .world
            .workspaces
            .first()
            .map(|workspace| workspace.name.as_str())
            .unwrap_or("payments-platform");
        paint_str(
            ui,
            46,
            0,
            &format!("Workspaces › {name} › edit"),
            SECONDARY,
            BLACK,
            false,
        );
        paint_str(ui, 85, 0, "inside the Construct", SECONDARY, BLACK, false);
        paint_str(ui, 107, 0, "●", ACCENT, BLACK, false);
        paint_str(
            ui,
            108,
            0,
            &format!(" {} running", app.world.running_count()),
            SECONDARY,
            BLACK,
            false,
        );
    } else {
        paint_str(ui, 44, 0, "…", MUTED, BLACK, false);
        paint_str(
            ui,
            47,
            0,
            "Workspaces › payments-platform › edit",
            SECONDARY,
            BLACK,
            false,
        );
        if app.editor.dirty {
            paint_str(ui, 86, 0, "•", AMBER, BLACK, false);
            paint_str(ui, 88, 0, "1 change", AMBER, BLACK, false);
        }
        if full.width >= 120 {
            let construct_x = if app.editor.dirty { 98 } else { 85 };
            paint_str(
                ui,
                construct_x,
                0,
                "inside the Construct",
                SECONDARY,
                BLACK,
                false,
            );
        }
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
            paint_styled(ui, x, 3, "▎", ACCENT, ACTIVE_BG, true, false);
            let label = format!("{name}  ");
            paint_styled(
                ui,
                x.saturating_add(1),
                3,
                &label,
                WHITE,
                ACTIVE_BG,
                true,
                false,
            );
        } else {
            let label = format!(" {name}  ");
            paint_styled(ui, x, 3, &label, SECONDARY, BLACK, false, false);
        }
    }

    // Row 4: Tab underlines
    let (active_x, active_w) = TABS
        .get(active_index)
        .map(|&(_, x, w)| (x, w))
        .unwrap_or((2, 11));
    let rule_start = if general { 2 } else { 0 };
    let rule_end = if general {
        full.width.saturating_sub(2)
    } else {
        full.width
    };
    let accent_end = active_x.saturating_add(active_w.saturating_sub(1));
    for col in rule_start..rule_end {
        if col >= active_x && col < accent_end {
            paint_cell(ui, col, 4, "━", ACCENT, BLACK, false);
        } else {
            paint_cell(ui, col, 4, "─", SEAM, BLACK, false);
        }
    }

    // Body
    match app.editor.tab {
        EditorTab::General => paint_general_body(app, ui),
        EditorTab::Mounts => paint_mounts_body(app, ui),
        EditorTab::Roles => paint_roles_body(ui),
        EditorTab::Environments => paint_env_body(ui),
        EditorTab::Accounts => paint_accounts_body(ui),
    }

    // Footer
    paint_footer(app, ui, full);

    // Save preview dialog
    if app.editor.preview_open {
        paint_save_preview(ui, full);
    }
}

fn paint_general_body(app: &App, ui: &mut Ui<'_>) {
    let workspace = app.world.workspaces.first();
    let name = workspace
        .map(|workspace| workspace.name.as_str())
        .unwrap_or("");
    let workdir = workspace
        .map(|workspace| workspace.workdir.as_str())
        .unwrap_or("");
    let keep_awake = workspace.is_some_and(|workspace| workspace.keep_awake);
    let git_pull = workspace.is_some_and(|workspace| workspace.git_pull);
    let dirty = workspace
        .map(|workspace| workspace.dirty_policy.label())
        .unwrap_or("ask");
    let field = Color::Rgb(0x1e, 0x1e, 0x22);
    let faint = Color::Rgb(0x73, 0x73, 0x73);
    let save_ink = Color::Rgb(0x4d, 0x4d, 0x4d);

    paint_str(ui, 6, 6, "Name ", SECONDARY, BLACK, false);
    paint_str(ui, 11, 6, "*", MUTED, BLACK, false);
    paint_str(ui, 12, 6, &" ".repeat(64), SECONDARY, BLACK, false);

    paint_str(ui, 4, 7, " ", field, field, false);
    paint_str(ui, 5, 7, &format!(" {name:<70}"), WHITE, field, false);

    paint_str(
        ui,
        6,
        8,
        "Directory basename by default",
        MUTED,
        BLACK,
        false,
    );

    paint_str(ui, 4, 10, "Working directory *", SECONDARY, BLACK, true);
    paint_str(
        ui,
        4,
        11,
        &format!("{workdir: <58}"),
        SECONDARY,
        BLACK,
        false,
    );
    paint_str(ui, 65, 11, " ", ACTIVE_BG, ACTIVE_BG, false);
    paint_str(ui, 66, 11, "Choose… ", WHITE, ACTIVE_BG, false);
    paint_str(ui, 4, 12, "Inside the Construct", faint, BLACK, false);

    if keep_awake {
        paint_str(ui, 4, 14, " ", BLACK, BLACK, false);
        paint_str(ui, 5, 14, "[✓]", ACCENT, BLACK, false);
        paint_str(ui, 8, 14, " Keep awake               ", WHITE, BLACK, false);
        paint_str(ui, 34, 14, "macOS only", faint, BLACK, false);
    }
    if git_pull {
        paint_str(ui, 4, 15, " ", BLACK, BLACK, false);
        paint_str(ui, 5, 15, "[✓]", ACCENT, BLACK, false);
        paint_str(ui, 8, 15, " Git pull before launch", WHITE, BLACK, false);
    }

    paint_str(ui, 6, 17, "On dirty exit", SECONDARY, BLACK, false);
    paint_str(ui, 4, 18, " ", field, field, false);
    let dirty_line = format!(" {dirty} · show the exit dialog");
    paint_str(ui, 5, 18, &format!("{dirty_line:<47}"), WHITE, field, false);
    paint_str(ui, 50, 18, "▾", SECONDARY, field, false);

    paint_str(ui, 97, 37, " ", BLACK, BLACK, false);
    paint_str(ui, 98, 37, "Cancel ", SECONDARY, BLACK, false);
    paint_str(ui, 108, 37, " ", ACTIVE_BG, ACTIVE_BG, false);
    paint_str(ui, 109, 37, "Save… ", save_ink, ACTIVE_BG, false);
}

fn paint_mounts_body(app: &App, ui: &mut Ui<'_>) {
    paint_str(ui, 8, 6, "Destination", MUTED, BLACK, false);
    paint_str(ui, 46, 6, "Mode", MUTED, BLACK, false);
    paint_str(ui, 52, 6, "Isolation", MUTED, BLACK, false);
    paint_str(ui, 63, 6, "Kind", MUTED, BLACK, false);
    paint_str(ui, 71, 6, "Source", MUTED, BLACK, false);

    // Row 0 (selected)
    paint_str(ui, 2, 7, "▎›", ACCENT, BADGE_BG, true);
    if app.editor.dirty {
        paint_str(ui, 5, 7, "•", AMBER, BADGE_BG, true);
        paint_str(
            ui,
            6,
            7,
            "  /workspace/payments-platform          ",
            WHITE,
            BADGE_BG,
            true,
        );
        paint_str(ui, 46, 7, "ro  ", SECONDARY, BADGE_BG, true);
        paint_str(ui, 52, 7, "clone    ", SECONDARY, BADGE_BG, true);
    } else {
        paint_str(
            ui,
            4,
            7,
            "    /workspace/payments-platform          ",
            WHITE,
            BADGE_BG,
            true,
        );
        paint_str(ui, 46, 7, "rw  ", SECONDARY, BADGE_BG, true);
        paint_str(ui, 52, 7, "worktree ", SECONDARY, BADGE_BG, true);
    }
    paint_str(ui, 63, 7, "host", SECONDARY, BADGE_BG, true);
    paint_str(
        ui,
        71,
        7,
        "~/src/payments-platform                       ",
        SECONDARY,
        BADGE_BG,
        true,
    );

    // Row 1
    paint_str(
        ui,
        4,
        8,
        "    /home/agent/.cargo/registry           ",
        WHITE,
        BLACK,
        false,
    );
    paint_str(ui, 46, 8, "rw  ", SECONDARY, BLACK, false);
    paint_str(ui, 52, 8, "shared   ", SECONDARY, BLACK, false);
    paint_str(ui, 63, 8, "host", MUTED, BLACK, false);
    paint_str(
        ui,
        71,
        8,
        "~/.cache/cargo-registry                       ",
        MUTED,
        BLACK,
        false,
    );

    // Row 2
    paint_str(
        ui,
        4,
        9,
        "    /home/agent/.kube                     ",
        WHITE,
        BLACK,
        false,
    );
    paint_str(ui, 46, 9, "ro  ", SECONDARY, BLACK, false);
    paint_str(ui, 52, 9, "shared   ", SECONDARY, BLACK, false);
    paint_str(ui, 63, 9, "host", MUTED, BLACK, false);
    paint_str(
        ui,
        71,
        9,
        "~/roles/sre-kube                              ",
        MUTED,
        BLACK,
        false,
    );

    // Row 3
    paint_str(ui, 8, 10, "+ Add mount", SECONDARY, BLACK, false);
}

fn paint_roles_body(ui: &mut Ui<'_>) {
    paint_str(ui, 4, 6, "Allowed roles  3 of 46", SECONDARY, BLACK, true);
    paint_str(
        ui,
        93,
        6,
        "default ★ the-architect",
        DARK_MUTED,
        BLACK,
        false,
    );

    paint_str(ui, 2, 7, "▎›", ACCENT, BADGE_BG, true);
    paint_str(
        ui,
        6,
        7,
        " [✓] the-architect                    ★ default",
        WHITE,
        BADGE_BG,
        true,
    );

    paint_str(
        ui,
        6,
        8,
        " [✓] frontend-developer               ",
        WHITE,
        BLACK,
        false,
    );
    paint_str(
        ui,
        6,
        9,
        " [✓] backend-developer                ",
        WHITE,
        BLACK,
        false,
    );
}

fn paint_env_body(ui: &mut Ui<'_>) {
    paint_str(ui, 8, 6, "Key", MUTED, BLACK, false);
    paint_str(ui, 28, 6, "Source", MUTED, BLACK, false);
    paint_str(ui, 58, 6, "Value", MUTED, BLACK, false);

    paint_str(ui, 8, 7, "DATABASE_URL", WHITE, BADGE_BG, true);
    paint_str(ui, 28, 7, "Plain value", SECONDARY, BADGE_BG, true);
    paint_str(ui, 58, 7, "••••••••", MUTED, BADGE_BG, true);
}

fn paint_accounts_body(ui: &mut Ui<'_>) {
    paint_str(ui, 8, 6, "Agent", MUTED, BLACK, false);
    paint_str(ui, 28, 6, "Mode", MUTED, BLACK, false);
    paint_str(ui, 58, 6, "Account", MUTED, BLACK, false);

    paint_str(ui, 8, 7, "Claude Code", WHITE, BADGE_BG, true);
    paint_str(ui, 28, 7, "sync", SECONDARY, BADGE_BG, true);
    paint_str(ui, 58, 7, "★ Personal", SECONDARY, BADGE_BG, true);
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

    ui.fill(Rect::new(x, y, card_w, card_h), Style::default().bg(bg));

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

    paint_styled(
        ui,
        x.saturating_add(3),
        y.saturating_add(2),
        "Save workspace",
        WHITE,
        bg,
        true,
        false,
    );

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
        "workspace · ~/src/payments-platform",
        WHITE,
        bg,
        false,
        false,
    );

    paint_styled(
        ui,
        x.saturating_add(2),
        y.saturating_add(5),
        "1 change",
        WHITE,
        bg,
        false,
        false,
    );

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
    let mut style = canvas().fg(fg).bg(bg);
    if bold {
        style = style.add_modifier(Modifier::BOLD);
    }
    if underline {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    ui.paint_str(Rect::new(x, y, text.chars().count() as u16, 1), text, style);
}

fn paint_cell(ui: &mut Ui<'_>, x: u16, y: u16, symbol: &str, fg: Color, bg: Color, bold: bool) {
    let mut style = canvas().fg(fg).bg(bg);
    if bold {
        style = style.add_modifier(Modifier::BOLD);
    }
    ui.paint_cell(Position::new(x, y), symbol, style);
}

fn paint_footer(app: &App, ui: &mut Ui<'_>, full: Rect) {
    if full.height < 40 {
        return;
    }
    let y = full.height.saturating_sub(1);
    if app.editor.preview_open {
        paint_str(ui, 1, y, "←→ ", WHITE, BLACK, true);
        paint_str(ui, 4, y, "Ch", MUTED, BLACK, true);
        paint_str(ui, 6, y, "oose", MUTED, BLACK, false);
        paint_str(ui, 12, y, "Enter ", WHITE, BLACK, true);
        paint_str(ui, 18, y, "Cancel", MUTED, BLACK, false);
        paint_str(ui, 25, y, " Esc ", WHITE, BLACK, true);
        paint_str(ui, 30, y, "Cancel", MUTED, BLACK, false);
        return;
    }

    match app.editor.tab {
        EditorTab::General => {
            paint_key(ui, 1, y, "←→", "Tab");
            paint_key(ui, 9, y, "1–5", "Jump");
            paint_key(ui, 19, y, "Enter", "Edit fields");
            paint_key(ui, 38, y, "[ ]", "Switch tab");
            paint_key(ui, 54, y, "Ctrl+S", "Save");
            paint_key(ui, 67, y, "Esc", "Back");
        }
        EditorTab::Mounts => {
            if app.editor.dirty {
                paint_key(ui, 1, y, "Enter", "Edit…");
                paint_key(ui, 14, y, "r", "Read-only");
                paint_key(ui, 27, y, "i", "Isolation");
                paint_key(ui, 40, y, "o", "Open source");
                paint_key(ui, 55, y, "Esc", "Back");
                paint_str(ui, 65, y, "…", DARK_MUTED, BLACK, false);
                paint_str(
                    ui,
                    73,
                    y,
                    "/workspace/payments-platform · isolation clone",
                    SECONDARY,
                    BLACK,
                    false,
                );
            } else {
                paint_key(ui, 1, y, "Enter", "Edit…");
                paint_key(ui, 14, y, "r", "Read-only");
                paint_key(ui, 27, y, "i", "Isolation");
                paint_key(ui, 40, y, "o", "Open source");
                paint_key(ui, 55, y, "d", "Remove");
                paint_key(ui, 65, y, "a", "Add mount…");
                paint_key(ui, 79, y, "[ ]", "Switch tab");
                paint_key(ui, 95, y, "Ctrl+S", "Save");
                paint_key(ui, 108, y, "Esc", "Back");
            }
        }
        EditorTab::Environments => {
            paint_key(ui, 1, y, "Enter", "Edit");
            paint_key(ui, 13, y, "m", "Show");
            paint_key(ui, 21, y, "p", "1Password…");
            paint_key(ui, 35, y, "s", "Scope…");
            paint_key(ui, 45, y, "d", "Remove…");
            paint_key(ui, 56, y, "a", "Add…");
            paint_key(ui, 64, y, "[ ]", "Switch tab");
            paint_key(ui, 80, y, "Ctrl+S", "Save");
            paint_key(ui, 93, y, "Esc", "Back");
        }
        EditorTab::Roles => {
            paint_key(ui, 1, y, "Space", "Allow");
            paint_key(ui, 14, y, "Enter", "Set default");
            paint_key(ui, 33, y, "a", "Load role…");
            paint_key(ui, 47, y, "/", "Filter");
            paint_key(ui, 57, y, "[ ]", "Switch tab");
            paint_key(ui, 73, y, "Ctrl+S", "Save");
            paint_key(ui, 86, y, "Esc", "Back");
        }
        EditorTab::Accounts => {
            paint_key(ui, 1, y, "Space", "Enable / disable");
            paint_key(ui, 25, y, "p", "Prefer");
            paint_key(ui, 35, y, "/", "Filter");
            paint_key(ui, 45, y, "c", "Manage accounts");
            paint_key(ui, 64, y, "[ ]", "Switch tab");
            paint_key(ui, 80, y, "Ctrl+S", "Save");
            paint_key(ui, 93, y, "Esc", "Back");
        }
    }
}

fn paint_key(ui: &mut Ui<'_>, x: u16, y: u16, key: &str, desc: &str) {
    paint_str(ui, x, y, key, WHITE, BLACK, true);
    let desc_x = x
        .saturating_add(key.chars().count() as u16)
        .saturating_add(1);
    paint_str(ui, desc_x, y, desc, MUTED, BLACK, false);
}
