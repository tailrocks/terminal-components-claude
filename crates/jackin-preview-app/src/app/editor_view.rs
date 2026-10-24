//! Live editor screen rendering with termrock widgets and components.
//! Replaces historical_editor*.rs replay modules.

use termrock::author::PaintStyle;
use termrock::feedback::{
    EditorEnvironmentBody, EditorEnvironmentRole, EditorEnvironmentRow, EditorEnvironmentValue,
    paint_editor_environment_body,
};
use termrock::{Color, Modifier, Position, Rect, Style, Ui, width};

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
    paint_with_environment_mode(app, ui, full, false);
}

#[cfg(test)]
fn paint_preimage(app: &App, ui: &mut Ui<'_>, full: Rect) {
    paint_with_environment_mode(app, ui, full, true);
}

fn paint_with_environment_mode(app: &App, ui: &mut Ui<'_>, full: Rect, legacy_environment: bool) {
    if full.is_empty() {
        return;
    }
    let general = matches!(app.editor.tab, EditorTab::General);
    let mounts = matches!(app.editor.tab, EditorTab::Mounts);
    let roles = matches!(app.editor.tab, EditorTab::Roles);
    let env = matches!(app.editor.tab, EditorTab::Environments);
    let accounts = matches!(app.editor.tab, EditorTab::Accounts);
    let chrome = general || mounts || roles || env || accounts;
    // The live shell is surface ink. These approved pages are canvas.
    if chrome {
        ui.fill(full, canvas());
    }

    // Row 0: Header
    paint_str(ui, 1, 0, " jackin❯ ", BRAND_FG, ACCENT, true);
    paint_str(ui, 12, 0, " File ", SECONDARY, BLACK, false);
    paint_str(ui, 19, 0, " Go ", SECONDARY, BLACK, false);
    paint_str(ui, 24, 0, " Help ", SECONDARY, BLACK, false);

    let preview = app.editor.preview_open;
    let changes = editor_changes(app);
    if chrome {
        let name = app
            .world
            .workspaces
            .first()
            .map(|workspace| workspace.name.as_str())
            .unwrap_or("payments-platform");
        let crumb = format!("Workspaces › {name} › edit");
        let mounts_dirty = mounts && app.editor.dirty && !preview;
        if mounts_dirty {
            paint_str(ui, 44, 0, "…", MUTED, BLACK, false);
            paint_str(ui, 47, 0, &crumb, SECONDARY, BLACK, false);
            paint_str(ui, 86, 0, "• 1 change", AMBER, BLACK, false);
            paint_str(ui, 98, 0, "inside the Construct", SECONDARY, BLACK, false);
        } else if preview && !changes.is_empty() {
            let note = format!("  • {}  ", change_label(changes.len()));
            let crumb_x =
                85u16.saturating_sub((crumb.chars().count() + note.chars().count()) as u16);
            paint_str(ui, crumb_x, 0, &crumb, SECONDARY, BLACK, false);
            let note_x = crumb_x.saturating_add(crumb.chars().count() as u16);
            paint_str(ui, note_x, 0, &note, WHITE, BLACK, false);
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
            paint_str(ui, 46, 0, &crumb, SECONDARY, BLACK, false);
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
        }
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

    let mounts_dirty = mounts && app.editor.dirty && !preview;
    let tab_shift = if (preview && !changes.is_empty()) || mounts_dirty {
        2
    } else {
        0
    };
    let mut active_w = TABS.get(active_index).map(|&(_, _, w)| w).unwrap_or(11);
    for (i, &(name, raw_x, _)) in TABS.iter().enumerate() {
        let x = if mounts_dirty {
            if i > 1 {
                raw_x.saturating_add(tab_shift)
            } else {
                raw_x
            }
        } else if i == 0 {
            raw_x
        } else {
            raw_x.saturating_add(tab_shift)
        };
        if mounts_dirty && i == active_index {
            paint_styled(ui, x, 3, " Mounts ", WHITE, ACTIVE_BG, true, false);
            paint_styled(
                ui,
                x.saturating_add(8),
                3,
                "•",
                AMBER,
                ACTIVE_BG,
                true,
                false,
            );
            paint_styled(
                ui,
                x.saturating_add(9),
                3,
                "  ",
                WHITE,
                ACTIVE_BG,
                true,
                false,
            );
            active_w = 11;
        } else if preview && !changes.is_empty() && i == active_index {
            let label = format!(" {name} •  ");
            active_w = label.chars().count() as u16;
            paint_styled(ui, x, 3, &label, WHITE, ACTIVE_BG, true, false);
        } else if i == active_index && general {
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
        } else if i == active_index {
            let label = format!(" {name}  ");
            paint_styled(ui, x, 3, &label, WHITE, ACTIVE_BG, true, false);
        } else {
            let label = format!(" {name}  ");
            paint_styled(ui, x, 3, &label, SECONDARY, BLACK, false, false);
        }
    }

    // Row 4: Tab underlines
    let active_x = TABS.get(active_index).map(|&(_, x, _)| x).unwrap_or(2);
    let rule_start = if chrome { 2 } else { 0 };
    let rule_end = if chrome {
        full.width.saturating_sub(2)
    } else {
        full.width
    };
    let accent_end = if (preview && !changes.is_empty()) || (mounts && app.editor.dirty && !preview)
    {
        active_x.saturating_add(active_w)
    } else {
        active_x.saturating_add(active_w.saturating_sub(1))
    };
    let heavy = if preview && !changes.is_empty() {
        SEAM
    } else {
        ACCENT
    };
    for col in rule_start..rule_end {
        if col >= active_x && col < accent_end {
            paint_cell(ui, col, 4, "━", heavy, BLACK, false);
        } else {
            paint_cell(ui, col, 4, "─", SEAM, BLACK, false);
        }
    }

    // Body
    match app.editor.tab {
        EditorTab::General => paint_general_body(app, ui),
        EditorTab::Mounts => paint_mounts_body(app, ui),
        EditorTab::Roles => paint_roles_body(app, ui),
        EditorTab::Environments => {
            if legacy_environment {
                paint_env_body_legacy(app, ui);
            } else {
                paint_env_body(app, ui, full);
            }
        }
        EditorTab::Accounts => paint_accounts_body(app, ui),
    }

    if preview {
        ui.modal_backdrop(full);
        paint_preview_footer(ui, full.height.saturating_sub(1));
        paint_save_preview(app, ui, full, &changes);
    } else {
        paint_footer(app, ui, full);
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
    let keep_awake = app.editor.pending.keep_awake;
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

    let preview = app.editor.preview_open;
    let keep_selected = preview && app.editor_general_cursor == 3;
    let keep_bg = if keep_selected { BADGE_BG } else { BLACK };
    if keep_selected {
        paint_str(ui, 4, 14, " ", keep_bg, keep_bg, false);
    } else {
        paint_str(ui, 4, 14, " ", BLACK, BLACK, false);
    }
    let keep_mark = if keep_awake { "[✓]" } else { "[ ]" };
    let keep_ink = if keep_awake && !preview {
        ACCENT
    } else if keep_awake {
        WHITE
    } else {
        SECONDARY
    };
    paint_str(ui, 5, 14, keep_mark, keep_ink, keep_bg, keep_selected);
    paint_str(
        ui,
        8,
        14,
        " Keep awake               ",
        WHITE,
        keep_bg,
        keep_selected,
    );
    if keep_awake && !preview {
        paint_str(ui, 34, 14, "macOS only", faint, BLACK, false);
    }
    if git_pull {
        let pull_ink = if preview { WHITE } else { ACCENT };
        paint_str(ui, 4, 15, " ", BLACK, BLACK, false);
        paint_str(ui, 5, 15, "[✓]", pull_ink, BLACK, false);
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
    let save_color = if app.editor.preview_open {
        SECONDARY
    } else {
        save_ink
    };
    paint_str(ui, 109, 37, "Save… ", save_color, ACTIVE_BG, false);
}

fn paint_mounts_body(app: &App, ui: &mut Ui<'_>) {
    paint_str(ui, 8, 6, "Destination", MUTED, BLACK, false);
    paint_str(ui, 46, 6, "Mode", MUTED, BLACK, false);
    paint_str(ui, 52, 6, "Isolation", MUTED, BLACK, false);
    paint_str(ui, 63, 6, "Kind", MUTED, BLACK, false);
    paint_str(ui, 71, 6, "Source", MUTED, BLACK, false);

    let rows: Vec<_> = app
        .world
        .workspaces
        .first()
        .map(|workspace| workspace.mounts.clone())
        .unwrap_or_default();
    if let Some(mount) = rows.first() {
        if !app.editor.dirty {
            paint_str(ui, 2, 7, &" ".repeat(116), WHITE, BADGE_BG, true);
            paint_str(ui, 2, 7, "▎›", ACCENT, BADGE_BG, true);
            paint_str(
                ui,
                4,
                7,
                &format!("    {:<38}", mount.destination),
                WHITE,
                BADGE_BG,
                true,
            );
            paint_str(
                ui,
                46,
                7,
                &format!("{:<4}", mount.mode_label()),
                SECONDARY,
                BADGE_BG,
                true,
            );
            paint_str(
                ui,
                52,
                7,
                &format!("{:<9}", isolation_word(mount.isolation)),
                SECONDARY,
                BADGE_BG,
                true,
            );
            paint_str(ui, 63, 7, "host", SECONDARY, BADGE_BG, true);
            paint_str(ui, 67, 7, "  ", MUTED, BADGE_BG, true);
            paint_str(ui, 71, 7, mount.source_label(), SECONDARY, BADGE_BG, true);
            paint_str(ui, 94, 7, &" ".repeat(23), MUTED, BADGE_BG, true);
        } else {
            paint_str(ui, 2, 7, &" ".repeat(116), WHITE, BADGE_BG, true);
            paint_str(ui, 2, 7, "▎›", ACCENT, BADGE_BG, true);
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
            paint_str(ui, 63, 7, "host", SECONDARY, BADGE_BG, true);
            paint_str(ui, 67, 7, "  ", MUTED, BADGE_BG, true);
            paint_str(
                ui,
                71,
                7,
                "~/src/payments-platform",
                SECONDARY,
                BADGE_BG,
                true,
            );
            paint_str(ui, 94, 7, &" ".repeat(23), MUTED, BADGE_BG, true);
        }
    }
    if let Some(mount) = rows.get(1) {
        paint_str(ui, 2, 8, " ", BLACK, BLACK, false);
        paint_str(ui, 3, 8, " ", SECONDARY, BLACK, false);
        paint_str(
            ui,
            4,
            8,
            &format!("    {:<38}", mount.destination),
            WHITE,
            BLACK,
            false,
        );
        paint_str(
            ui,
            46,
            8,
            &format!("{:<4}", mount.mode_label()),
            SECONDARY,
            BLACK,
            false,
        );
        paint_str(
            ui,
            52,
            8,
            &format!("{:<9}", isolation_word(mount.isolation)),
            SECONDARY,
            BLACK,
            false,
        );
        paint_str(ui, 63, 8, "host  ", MUTED, BLACK, false);
        paint_str(
            ui,
            71,
            8,
            &format!("{:<46}", mount.source_label()),
            MUTED,
            BLACK,
            false,
        );
    }
    paint_str(ui, 2, 9, " ", BLACK, BLACK, false);
    paint_str(ui, 3, 9, " ", SECONDARY, BLACK, false);
    paint_str(ui, 8, 9, "+ Add mount", SECONDARY, BLACK, false);
    paint_str(ui, 97, 37, " ", BLACK, BLACK, false);
    paint_str(ui, 98, 37, "Cancel ", SECONDARY, BLACK, false);
    if app.editor.dirty {
        paint_str(ui, 108, 37, " ", ACCENT, ACCENT, false);
        paint_str(ui, 109, 37, "Save… ", BRAND_FG, ACCENT, true);
    } else {
        paint_str(ui, 108, 37, " ", ACTIVE_BG, ACTIVE_BG, false);
        paint_str(
            ui,
            109,
            37,
            "Save… ",
            Color::Rgb(0x4d, 0x4d, 0x4d),
            ACTIVE_BG,
            false,
        );
    }
}

fn isolation_word(isolation: crate::domain::workspace::Isolation) -> &'static str {
    match isolation {
        crate::domain::workspace::Isolation::Shared => "shared",
        crate::domain::workspace::Isolation::Worktree => "worktree",
        crate::domain::workspace::Isolation::Clone => "clone",
    }
}

fn paint_roles_body(app: &App, ui: &mut Ui<'_>) {
    let workspace = app.world.workspaces.first();
    let default_leaf = workspace
        .and_then(|workspace| workspace.roles.default.as_deref())
        .map(role_leaf)
        .unwrap_or("");
    let allowed_count = match workspace.map(|workspace| &workspace.roles.allowed) {
        Some(crate::domain::workspace::AllowedRoles::Custom(list)) => list.len(),
        Some(crate::domain::workspace::AllowedRoles::All) => app.world.roles.len(),
        None => 0,
    };
    paint_str(
        ui,
        4,
        6,
        &format!(
            "Allowed roles  {allowed_count} of {}",
            app.world.roles.len()
        ),
        SECONDARY,
        BLACK,
        true,
    );
    paint_str(
        ui,
        93,
        6,
        &format!("default ★ {default_leaf}"),
        DARK_MUTED,
        BLACK,
        false,
    );

    let visible = 29usize;
    let thumb = 17usize;
    for (index, role) in app.world.roles.iter().take(visible).enumerate() {
        let y = 7 + index as u16;
        let selected = index == 0;
        let checked = workspace.is_some_and(|workspace| {
            workspace.roles.allows(&role.full_name()) || workspace.roles.allows(&role.name)
        });
        paint_role_row(ui, y, role, checked, selected, default_leaf);
        let bar = if index < thumb { "┃" } else { "┆" };
        let bar_ink = if index < thumb {
            Color::Rgb(0x80, 0x80, 0x80)
        } else {
            Color::Rgb(0x47, 0x47, 0x47)
        };
        paint_str(ui, 117, y, bar, bar_ink, BLACK, false);
    }
    // Overflow list: inner edge keeps 0.8, outer edge keeps 0.55. The bar
    // column stays out of the rect so the track glyph is not blended.
    let mut scroll = termrock::ScrollState::new(app.world.roles.len());
    scroll.set_viewport(visible);
    ui.scroll_edges(Rect::new(2, 7, 115, visible as u16), &scroll);

    paint_str(ui, 97, 37, " ", BLACK, BLACK, false);
    paint_str(ui, 98, 37, "Cancel ", SECONDARY, BLACK, false);
    paint_str(ui, 108, 37, " ", ACTIVE_BG, ACTIVE_BG, false);
    paint_str(
        ui,
        109,
        37,
        "Save… ",
        Color::Rgb(0x4d, 0x4d, 0x4d),
        ACTIVE_BG,
        false,
    );
}

fn paint_role_row(
    ui: &mut Ui<'_>,
    y: u16,
    role: &crate::domain::workspace::RoleEntry,
    checked: bool,
    selected: bool,
    default_leaf: &str,
) {
    let bg = if selected { BADGE_BG } else { BLACK };
    let bold = selected;
    if selected {
        paint_str(ui, 2, y, &" ".repeat(115), WHITE, bg, true);
        paint_str(ui, 2, y, "▎›", ACCENT, bg, true);
    } else {
        paint_str(ui, 2, y, " ", BLACK, BLACK, false);
        paint_str(ui, 3, y, " ", SECONDARY, BLACK, false);
    }
    paint_str(ui, 5, y, " ", AMBER, bg, bold);
    if checked {
        let star = if selected && role.name == default_leaf {
            " ★"
        } else {
            ""
        };
        let label = format!(" [{}] {}{star}", "✓", role.name);
        let width = if selected { 22 } else { 19 };
        paint_str(ui, 6, y, &format!("{label:<width$}"), WHITE, bg, bold);
    } else {
        paint_str(ui, 7, y, "[ ]", MUTED, bg, bold);
        paint_str(ui, 10, y, &format!(" {:<14}", role.name), WHITE, bg, bold);
    }
    if !selected {
        paint_str(ui, 25, y, " ", SECONDARY, bg, bold);
    }
    let (meta, tone) = role_meta(role);
    if selected {
        paint_role_meta(ui, 28, y, &meta, bg);
    } else {
        paint_str(ui, 28, y, &meta, tone, bg, bold);
    }
}

fn role_meta(role: &crate::domain::workspace::RoleEntry) -> (String, Color) {
    if let Some(error) = &role.load_error {
        return (
            format!("! load error · {error}"),
            Color::Rgb(0xf0, 0x5d, 0x5d),
        );
    }
    if !role.in_registry {
        let path = match &role.source {
            crate::domain::workspace::RoleSource::Local { path } => path.as_str(),
            crate::domain::workspace::RoleSource::Git { url, .. } => url.as_str(),
        };
        return (format!("{path} · not in registry"), AMBER);
    }
    let trust = if role.trusted { "trusted" } else { "untrusted" };
    (format!("registry · {trust} · {}", role.description), MUTED)
}

fn paint_role_meta(ui: &mut Ui<'_>, x: u16, y: u16, meta: &str, bg: Color) {
    let mut column = x;
    let mut word = String::new();
    let flush = |ui: &mut Ui<'_>, column: &mut u16, word: &mut String| {
        if word.is_empty() {
            return;
        }
        paint_str(ui, *column, y, word, SECONDARY, bg, true);
        *column = column.saturating_add(word.chars().count() as u16);
        word.clear();
    };
    for ch in meta.chars() {
        if ch == ' ' || ch == '·' {
            flush(ui, &mut column, &mut word);
            let tone = if ch == '·' { SECONDARY } else { MUTED };
            paint_str(ui, column, y, &ch.to_string(), tone, bg, true);
            column = column.saturating_add(1);
        } else {
            word.push(ch);
        }
    }
    flush(ui, &mut column, &mut word);
}

fn role_leaf(role: &str) -> &str {
    role.rsplit('/').next().unwrap_or(role)
}

fn paint_env_body(app: &App, ui: &mut Ui<'_>, full: Rect) {
    if paint_shared_env_body(app, ui, full) {
        paint_env_footer(ui);
    } else {
        paint_env_body_legacy(app, ui);
    }
}

fn paint_shared_env_body(app: &App, ui: &mut Ui<'_>, full: Rect) -> bool {
    // The App-level proof covers only the published 120x40 screen at origin.
    if full.x != 0 || full.y != 0 || full.width != 120 || full.height != 40 {
        return false;
    }
    let pending = &app.editor.pending;
    let workspace_paths = pending
        .env
        .iter()
        .map(|env| match &env.value {
            crate::domain::workspace::EnvValue::OnePassword(reference) => {
                Some(reference.display_path())
            }
            crate::domain::workspace::EnvValue::Plain(_)
            | crate::domain::workspace::EnvValue::HostEnv(_) => None,
        })
        .collect::<Vec<_>>();
    if !editor_env_rows_have_known_widths(&pending.env, &workspace_paths) {
        return false;
    }
    let Some(workspace_rows) = project_editor_env_rows(&pending.env, &workspace_paths, Some(0))
    else {
        return false;
    };

    let role_entries = pending
        .role_env
        .iter()
        .filter(|(_, envs)| !envs.is_empty())
        .map(|(role, envs)| (role.as_str(), envs.as_slice()))
        .collect::<Vec<_>>();
    let role_paths = role_entries
        .iter()
        .map(|(_, envs)| {
            envs.iter()
                .map(|env| match &env.value {
                    crate::domain::workspace::EnvValue::OnePassword(reference) => {
                        Some(reference.display_path())
                    }
                    crate::domain::workspace::EnvValue::Plain(_)
                    | crate::domain::workspace::EnvValue::HostEnv(_) => None,
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    if !role_entries
        .iter()
        .zip(&role_paths)
        .all(|((_, envs), paths)| editor_env_rows_have_known_widths(envs, paths))
        || !editor_env_headings_have_known_widths(app, &role_entries)
    {
        return false;
    }
    let mut role_rows = Vec::with_capacity(role_entries.len());
    for ((_, envs), paths) in role_entries.iter().zip(&role_paths) {
        let Some(rows) = project_editor_env_rows(envs, paths, None) else {
            return false;
        };
        role_rows.push(rows);
    }

    let role_rights = role_entries
        .iter()
        .map(|(_, envs)| format!("{} var", envs.len()))
        .collect::<Vec<_>>();
    let roles = role_entries
        .iter()
        .zip(&role_rows)
        .zip(&role_rights)
        .map(|(((role, _), rows), right)| EditorEnvironmentRole::new(role_leaf(role), right, rows))
        .collect::<Vec<_>>();
    let workspace_right = format!("{} vars", pending.env.len());
    let configured = role_entries.len();
    let overrides_right = format!(
        "{configured} configured · {} in the registry",
        app.world.roles.len()
    );
    let body =
        EditorEnvironmentBody::new(&workspace_right, &workspace_rows, &overrides_right, &roles);
    paint_editor_environment_body(ui, full, &body).is_ok()
}

fn editor_env_rows_have_known_widths(
    envs: &[crate::domain::workspace::EnvVar],
    paths: &[Option<String>],
) -> bool {
    // The legacy Editor body pads and advances these fields by character
    // count. Keep that painter for values outside the shared cell columns or
    // where character count does not match terminal-cell width.
    if envs.len() != paths.len() {
        return false;
    }
    envs.iter().zip(paths).all(|(env, path)| {
        editor_env_text_has_known_width(&env.key, 20)
            && match &env.value {
                crate::domain::workspace::EnvValue::Plain(_) => true,
                crate::domain::workspace::EnvValue::OnePassword(_) => {
                    path.as_deref().is_some_and(|path| {
                        editor_env_text_has_known_width(&format!("[op] {path}"), 63)
                    })
                }
                crate::domain::workspace::EnvValue::HostEnv(name) => {
                    editor_env_text_has_known_width(&format!("${name}"), 24)
                }
            }
    })
}

fn editor_env_headings_have_known_widths(
    app: &App,
    role_entries: &[(&str, &[crate::domain::workspace::EnvVar])],
) -> bool {
    let configured = role_entries.len();
    let workspace_right = format!("{} vars", app.editor.pending.env.len());
    let overrides_right = format!(
        "{configured} configured · {} in the registry",
        app.world.roles.len()
    );
    editor_env_text_has_known_width(&workspace_right, u16::MAX)
        && editor_env_text_has_known_width(&overrides_right, u16::MAX)
        && role_entries.iter().all(|(role, envs)| {
            let role = role_leaf(role);
            let title = format!("Role: {role}");
            let right = format!("{} var", envs.len());
            let add = format!("+ Add {role} environment variable");
            editor_env_text_has_known_width(&title, u16::MAX)
                && editor_env_text_has_known_width(&right, u16::MAX)
                && editor_env_text_has_known_width(&add, u16::MAX)
        })
}

fn editor_env_text_has_known_width(text: &str, maximum: u16) -> bool {
    let Ok(character_width) = u16::try_from(text.chars().count()) else {
        return false;
    };
    character_width <= maximum && width(text) == character_width
}

fn project_editor_env_rows<'a>(
    envs: &'a [crate::domain::workspace::EnvVar],
    paths: &'a [Option<String>],
    selected: Option<usize>,
) -> Option<Vec<EditorEnvironmentRow<'a>>> {
    if envs.len() != paths.len() {
        return None;
    }
    envs.iter()
        .zip(paths)
        .enumerate()
        .map(|(index, (env, path))| {
            let value = match &env.value {
                crate::domain::workspace::EnvValue::Plain(secret) => {
                    EditorEnvironmentValue::PlainMasked {
                        stars: secret.chars().count().min(16),
                    }
                }
                crate::domain::workspace::EnvValue::OnePassword(_) => {
                    EditorEnvironmentValue::OnePassword {
                        display_path: path.as_deref()?,
                    }
                }
                crate::domain::workspace::EnvValue::HostEnv(name) => {
                    EditorEnvironmentValue::HostEnv { name }
                }
            };
            Some(
                EditorEnvironmentRow::new(&env.key, value)
                    .selected(selected.is_some_and(|selected| selected == index)),
            )
        })
        .collect()
}

fn paint_env_footer(ui: &mut Ui<'_>) {
    paint_str(ui, 97, 37, " ", BLACK, BLACK, false);
    paint_str(ui, 98, 37, "Cancel ", SECONDARY, BLACK, false);
    paint_str(ui, 108, 37, " ", ACTIVE_BG, ACTIVE_BG, false);
    paint_str(
        ui,
        109,
        37,
        "Save… ",
        Color::Rgb(0x4d, 0x4d, 0x4d),
        ACTIVE_BG,
        false,
    );
}

fn paint_env_body_legacy(app: &App, ui: &mut Ui<'_>) {
    let pending = &app.editor.pending;
    let mut y = 6u16;
    paint_str(ui, 6, y, "Workspace", SECONDARY, BLACK, true);
    paint_right(ui, y, &format!("{} vars", pending.env.len()), DARK_MUTED);
    y = y.saturating_add(1);
    for (index, env) in pending.env.iter().enumerate() {
        paint_env_row(ui, y, &env.key, &env.value, index == 0);
        y = y.saturating_add(1);
    }
    paint_env_gutter(ui, y);
    paint_str(
        ui,
        8,
        y,
        "+ Add environment variable",
        SECONDARY,
        BLACK,
        false,
    );
    y = y.saturating_add(1);

    let configured = pending
        .role_env
        .values()
        .filter(|vars| !vars.is_empty())
        .count();
    paint_str(ui, 6, y, "Role overrides", SECONDARY, BLACK, true);
    paint_right(
        ui,
        y,
        &format!(
            "{configured} configured · {} in the registry",
            app.world.roles.len()
        ),
        DARK_MUTED,
    );
    y = y.saturating_add(1);
    for (role, vars) in &pending.role_env {
        if vars.is_empty() {
            continue;
        }
        let leaf = role_leaf(role);
        paint_str(ui, 4, y, "▾", SECONDARY, BLACK, false);
        paint_str(ui, 6, y, &format!("Role: {leaf}"), SECONDARY, BLACK, true);
        paint_right(ui, y, &format!("{} var", vars.len()), DARK_MUTED);
        y = y.saturating_add(1);
        for env in vars {
            paint_env_row(ui, y, &env.key, &env.value, false);
            y = y.saturating_add(1);
        }
        paint_env_gutter(ui, y);
        paint_str(
            ui,
            8,
            y,
            &format!("+ Add {leaf} environment variable"),
            SECONDARY,
            BLACK,
            false,
        );
        y = y.saturating_add(1);
    }
    paint_env_gutter(ui, y);
    paint_str(ui, 8, y, "+ Add role override…", SECONDARY, BLACK, false);
    paint_env_footer(ui);
}

fn paint_right(ui: &mut Ui<'_>, y: u16, text: &str, fg: Color) {
    let width = text.chars().count() as u16;
    paint_str(ui, 117u16.saturating_sub(width), y, text, fg, BLACK, false);
}

fn paint_env_gutter(ui: &mut Ui<'_>, y: u16) {
    paint_str(ui, 2, y, " ", BLACK, BLACK, false);
    paint_str(ui, 3, y, " ", SECONDARY, BLACK, false);
}

fn env_shown(value: &crate::domain::workspace::EnvValue) -> (String, String) {
    match value {
        crate::domain::workspace::EnvValue::Plain(secret) => (
            "*".repeat(secret.chars().count().min(16)),
            "plain".to_owned(),
        ),
        crate::domain::workspace::EnvValue::OnePassword(reference) => {
            ("*".repeat(16), format!("[op] {}", reference.display_path()))
        }
        crate::domain::workspace::EnvValue::HostEnv(name) => {
            (format!("${name}"), "host env".to_owned())
        }
    }
}

fn paint_env_row(
    ui: &mut Ui<'_>,
    y: u16,
    key: &str,
    value: &crate::domain::workspace::EnvValue,
    selected: bool,
) {
    let (shown, source) = env_shown(value);
    let key_cell = format!("    {key:<20}");
    let value_cell = format!("{shown:<24}");
    if selected {
        paint_str(ui, 2, y, &" ".repeat(116), WHITE, BADGE_BG, true);
        paint_str(ui, 2, y, "▎›", ACCENT, BADGE_BG, true);
        paint_str(ui, 4, y, &key_cell, WHITE, BADGE_BG, true);
        paint_str(ui, 28, y, &value_cell, SECONDARY, BADGE_BG, true);
        paint_str(ui, 54, y, &source, SECONDARY, BADGE_BG, true);
        let tail = 54u16.saturating_add(source.chars().count() as u16);
        if tail < 117 {
            paint_str(
                ui,
                tail,
                y,
                &" ".repeat(usize::from(117 - tail)),
                MUTED,
                BADGE_BG,
                true,
            );
        }
    } else {
        paint_env_gutter(ui, y);
        paint_str(ui, 4, y, &key_cell, WHITE, BLACK, false);
        paint_str(ui, 28, y, &value_cell, SECONDARY, BLACK, false);
        let source_cell = format!("{source:<63}");
        paint_str(ui, 54, y, &source_cell, MUTED, BLACK, false);
    }
}

fn paint_accounts_body(app: &App, ui: &mut Ui<'_>) {
    let registry = &app.world.accounts;
    let effective = app.editor.pending.effective_accounts(registry);
    let inherited = effective
        .iter()
        .filter(|entry| entry.origin == crate::domain::workspace::Effective::InheritedDefault)
        .count();
    let enabled_here = effective
        .iter()
        .filter(|entry| entry.origin == crate::domain::workspace::Effective::Enabled)
        .count();
    paint_str(
        ui,
        4,
        6,
        &format!(
            "Active accounts  {} effective · {inherited} inherited · {enabled_here} enabled here",
            effective.len()
        ),
        SECONDARY,
        BLACK,
        true,
    );
    paint_str(
        ui,
        92,
        6,
        "registry in Accounts (c)",
        DARK_MUTED,
        BLACK,
        false,
    );

    let mut groups: std::collections::BTreeMap<
        crate::domain::agent::Provider,
        Vec<&crate::domain::account::Account>,
    > = std::collections::BTreeMap::new();
    for account in registry.sorted() {
        groups.entry(account.provider).or_default().push(account);
    }

    let mut y = 7u16;
    let mut first = true;
    for (provider, accounts) in groups {
        let active = accounts
            .iter()
            .filter(|account| effective.iter().any(|entry| entry.id == account.id))
            .count();
        paint_str(ui, 4, y, provider.label(), SECONDARY, BLACK, true);
        let meta = if active > 1 {
            format!("{active} active · picker at session start")
        } else if active == 1 {
            "1 active".to_owned()
        } else {
            "none active".to_owned()
        };
        paint_right(ui, y, &meta, DARK_MUTED);
        y = y.saturating_add(1);
        for account in accounts {
            let entry = effective.iter().find(|entry| entry.id == account.id);
            paint_account_row(ui, y, account, entry, first);
            first = false;
            y = y.saturating_add(1);
        }
    }

    paint_str(ui, 97, 37, " ", BLACK, BLACK, false);
    paint_str(ui, 98, 37, "Cancel ", SECONDARY, BLACK, false);
    paint_str(ui, 108, 37, " ", ACTIVE_BG, ACTIVE_BG, false);
    paint_str(
        ui,
        109,
        37,
        "Save… ",
        Color::Rgb(0x4d, 0x4d, 0x4d),
        ACTIVE_BG,
        false,
    );
}

fn paint_account_row(
    ui: &mut Ui<'_>,
    y: u16,
    account: &crate::domain::account::Account,
    entry: Option<&crate::domain::workspace::EffectiveAccount>,
    selected: bool,
) {
    let bg = if selected { BADGE_BG } else { BLACK };
    let checked = entry.is_some();
    let preferred = entry.is_some_and(|entry| entry.preferred);
    let raw_name = if account.origin == crate::domain::account::AccountOrigin::Discovered
        && account.display_name == "discovered"
    {
        "host login"
    } else {
        account.display_name.as_str()
    };
    let name = clip_name(raw_name, if checked { 29 } else { 28 });
    let scope = account_scope(account, entry);
    let usable = crate::domain::workspace::usability_of(account);
    let ready = usable.is_ready();
    let status = usable.label();
    if selected {
        paint_str(ui, 2, y, &" ".repeat(116), WHITE, bg, true);
        paint_str(ui, 2, y, "▎›", ACCENT, bg, true);
    } else {
        paint_env_gutter(ui, y);
    }
    paint_str(ui, 5, y, " ", AMBER, bg, selected);
    if checked {
        paint_str(ui, 6, y, &format!(" [✓] {name:<29}"), WHITE, bg, selected);
    } else {
        paint_str(ui, 7, y, "[ ]", MUTED, bg, selected);
        let name_ink = if account.enabled { WHITE } else { DARK_MUTED };
        let name_x = if account.enabled { 10 } else { 11 };
        let shown = if account.enabled {
            format!(" {name:<29}")
        } else {
            name
        };
        paint_str(ui, name_x, y, &shown, name_ink, bg, selected);
    }
    let mark = if preferred { "★" } else { " " };
    let mark_ink = if selected { WHITE } else { SECONDARY };
    paint_str(ui, 40, y, mark, mark_ink, bg, selected);
    if selected {
        paint_split_field(ui, 43, y, &format!("{scope:<22}"), SECONDARY, MUTED, bg);
    } else {
        paint_str(ui, 43, y, &format!("{scope:<22}"), MUTED, BLACK, false);
    }
    let status_ink = if !ready {
        AMBER
    } else if selected {
        SECONDARY
    } else {
        MUTED
    };
    paint_str(ui, 67, y, &status, status_ink, bg, selected);
}

fn account_scope(
    account: &crate::domain::account::Account,
    entry: Option<&crate::domain::workspace::EffectiveAccount>,
) -> String {
    if let Some(entry) = entry {
        entry.origin.label().to_owned()
    } else if account.origin == crate::domain::account::AccountOrigin::Discovered {
        "discovered on host".to_owned()
    } else {
        "available".to_owned()
    }
}

fn clip_name(name: &str, max: usize) -> String {
    if name.chars().count() <= max {
        name.to_owned()
    } else {
        let keep: String = name.chars().take(max.saturating_sub(1)).collect();
        format!("{keep}…")
    }
}

fn paint_split_field(
    ui: &mut Ui<'_>,
    x: u16,
    y: u16,
    text: &str,
    word: Color,
    gap: Color,
    bg: Color,
) {
    let mut column = x;
    let mut current = String::new();
    let mut spaces = false;
    let flush = |ui: &mut Ui<'_>, column: &mut u16, current: &mut String, spaces: bool| {
        if current.is_empty() {
            return;
        }
        let ink = if spaces { gap } else { word };
        paint_str(ui, *column, y, current, ink, bg, true);
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

fn editor_changes(app: &App) -> Vec<String> {
    let Some(workspace) = app.world.workspaces.first() else {
        return Vec::new();
    };
    let mut lines = Vec::new();
    if app.editor.pending.keep_awake != workspace.keep_awake {
        lines.push(format!(
            "~ keep_awake {} → {}",
            if workspace.keep_awake {
                "true"
            } else {
                "false"
            },
            if app.editor.pending.keep_awake {
                "true"
            } else {
                "false"
            }
        ));
    }
    lines
}

fn change_label(count: usize) -> String {
    if count == 1 {
        "1 change".to_owned()
    } else {
        format!("{count} changes")
    }
}

fn clip_cols(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_owned()
    } else {
        let keep: String = text.chars().take(max.saturating_sub(1)).collect();
        format!("{keep}…")
    }
}

fn paint_preview_footer(ui: &mut Ui<'_>, y: u16) {
    paint_str(ui, 0, y, &" ".repeat(120), WHITE, BLACK, false);
    paint_str(ui, 1, y, "←→ ", WHITE, BLACK, true);
    paint_str(ui, 4, y, "Ch", MUTED, BLACK, true);
    paint_str(ui, 6, y, "oose", MUTED, BLACK, false);
    paint_str(ui, 12, y, "Enter ", WHITE, BLACK, true);
    paint_str(ui, 18, y, "Cancel", MUTED, BLACK, false);
    paint_str(ui, 25, y, " Esc", WHITE, BLACK, true);
    paint_str(ui, 30, y, "Cancel", MUTED, BLACK, false);
    paint_str(ui, 41, y, "      ", WHITE, BLACK, true);
    paint_str(ui, 54, y, "   ", WHITE, BLACK, true);
}

fn paint_field(ui: &mut Ui<'_>, x: u16, y: u16, bg: Color, label: &str, value: &str) {
    paint_styled(ui, x.saturating_add(3), y, label, MUTED, bg, false, false);
    paint_styled(ui, x.saturating_add(14), y, value, WHITE, bg, false, false);
}

fn paint_save_preview(app: &App, ui: &mut Ui<'_>, full: Rect, changes: &[String]) {
    let card_w = 66u16;
    let card_h = 13u16;
    let x = full
        .x
        .saturating_add((full.width.saturating_sub(card_w)) / 2);
    let y = 14u16;

    let bg = Color::Rgb(0x18, 0x18, 0x1b);
    let border = Color::Rgb(0x4d, 0x4d, 0x4d);

    ui.fill(
        Rect::new(x, y, card_w, card_h),
        Style::default().fg(MUTED).bg(bg),
    );
    let field = Style::default().fg(Color::Reset).bg(bg);
    for row in 4..9 {
        ui.fill(
            Rect::new(x.saturating_add(3), y.saturating_add(row), 60, 1),
            field,
        );
    }

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

    let name = app
        .world
        .workspaces
        .first()
        .map(|workspace| workspace.name.as_str())
        .unwrap_or("payments-platform");
    paint_field(ui, x, y.saturating_add(4), bg, "Workspace", name);
    let scope = clip_cols(
        &format!("workspace config · ~/.jackin/workspaces/{name}"),
        49,
    );
    paint_field(ui, x, y.saturating_add(5), bg, "Scope", &scope);
    paint_field(
        ui,
        x,
        y.saturating_add(6),
        bg,
        "Changes",
        &change_label(changes.len()),
    );
    if let Some(line) = changes.first() {
        paint_styled(
            ui,
            x.saturating_add(3),
            y.saturating_add(8),
            line,
            SECONDARY,
            bg,
            false,
            false,
        );
    }

    let buttons = y.saturating_add(10);
    paint_styled(
        ui,
        x.saturating_add(47),
        buttons,
        "▎",
        ACCENT,
        ACTIVE_BG,
        false,
        false,
    );
    paint_styled(
        ui,
        x.saturating_add(48),
        buttons,
        "C",
        WHITE,
        ACTIVE_BG,
        true,
        true,
    );
    paint_styled(
        ui,
        x.saturating_add(49),
        buttons,
        "ancel ",
        WHITE,
        ACTIVE_BG,
        true,
        false,
    );
    paint_styled(
        ui,
        x.saturating_add(57),
        buttons,
        " ",
        ACCENT,
        ACCENT,
        false,
        false,
    );
    paint_styled(
        ui,
        x.saturating_add(58),
        buttons,
        "S",
        BRAND_FG,
        ACCENT,
        true,
        true,
    );
    paint_styled(
        ui,
        x.saturating_add(59),
        buttons,
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

#[cfg(test)]
mod tests {
    use super::paint_preimage;
    use crate::domain::workspace::{EnvValue, EnvVar};
    use crate::{App, Motion, Scenario};
    use termrock::{Color, ColorLevel, KeyCode, StateFlags, Theme};
    use termrock_test_support::{Harness, Scene};

    const WIDTH: u16 = 120;
    const HEIGHT: u16 = 40;

    fn editor_environment_harness() -> Harness<App> {
        let theme = Theme::junie();
        assert_eq!(theme.capability.color, ColorLevel::TrueColor);
        assert_eq!(theme.color.accent, Color::Rgb(0x48, 0xe0, 0x54));
        let mut harness = Harness::new(
            App::for_scenario_at(Scenario::Returning, Motion::Paused, 0),
            theme,
            WIDTH,
            HEIGHT,
        );
        let _ = harness.key(KeyCode::Down);
        let _ = harness.key(KeyCode::Char('e'));
        let _ = harness.key(KeyCode::Char('4'));
        let _ = harness.key(KeyCode::Enter);
        assert_eq!(harness.app().route(), crate::Route::Editor);
        assert_eq!(
            harness.app().editor.tab,
            crate::screens::editor::Tab::Environments
        );
        harness
    }

    fn assert_actual_editor_gate(harness: &Harness<App>) {
        let app = harness.app();
        assert_eq!(app.world.scenario, Scenario::Returning);
        assert_eq!(app.motion, Motion::Paused);
        assert_eq!(app.route(), crate::Route::Editor);
        assert_eq!(app.editor.tab, crate::screens::editor::Tab::Environments);
        assert!(!app.editor.env_form_open);
        assert!(!app.editor_role_picker);
        assert!(!app.help_open);
        assert!(!app.quit_open);
        assert!(!app.cancel_open);
        assert!(!app.editor_exit_open);
        let editor_allowed = (!app.editor.preview_open && app.status.is_none())
            || (app.editor.preview_open
                && app
                    .status
                    .as_deref()
                    .is_some_and(|status| status.starts_with("Save workspace")));
        assert!(editor_allowed, "App Editor high-fidelity gate is closed");
        assert_eq!((WIDTH, HEIGHT), (120, 40));
    }

    fn assert_published_owner(harness: &Harness<App>, id: termrock::Id, label: &str) {
        let area = harness
            .area_of(id)
            .unwrap_or_else(|| panic!("published {label} owner has no area"));
        assert!(
            area.width > 0 && area.height > 0,
            "empty {label} area: {area:?}"
        );
        assert!(
            !harness.state_of(id).contains(StateFlags::PRESSED),
            "{label} retained a pressed/captured state"
        );
    }

    fn assert_save_owners(harness: &Harness<App>, confirm_visible: bool) {
        let save = crate::screens::editor::SAVE;
        let confirm = crate::app::EDITOR_SAVE_CONFIRM;
        assert_published_owner(harness, save, "Editor Save");
        assert_eq!(
            harness.area_of(confirm).is_some(),
            confirm_visible,
            "confirmation owner visibility"
        );
        if confirm_visible {
            assert_published_owner(harness, confirm, "Editor Save confirmation");
        } else {
            assert!(
                !harness.state_of(confirm).contains(StateFlags::PRESSED),
                "hidden confirmation retained a pressed/captured state"
            );
        }
        assert!(
            harness.diagnostics().is_empty(),
            "Editor input produced runtime diagnostics: {:?}",
            harness.diagnostics()
        );
    }

    fn assert_matches_preimage(harness: &Harness<App>, label: &'static str) {
        assert_actual_editor_gate(harness);
        let mut old_body = Scene::new(label, Theme::junie(), ColorLevel::TrueColor, WIDTH, HEIGHT);
        old_body.draw(|ui, area| paint_preimage(harness.app(), ui, area));
        for y in 6..37 {
            for x in 0..WIDTH {
                let actual = harness.buffer().cell((x, y)).expect("App cell");
                let expected = old_body.buffer().cell((x, y)).expect("preimage cell");
                assert_eq!(
                    format!("{actual:?}"),
                    format!("{expected:?}"),
                    "native App/preimage cell differs at ({x}, {y})"
                );
            }
        }
    }

    fn editor_environment_app() -> App {
        let mut app = App::for_scenario_at(Scenario::Returning, Motion::Paused, 0);
        let workspace = app
            .world
            .workspaces
            .first()
            .expect("fixture workspace")
            .clone();
        app.editor.load_workspace(&workspace);
        app.editor.tab = crate::screens::editor::Tab::Environments;
        app
    }

    fn assert_app_fallback_matches_legacy(app: &App, label: &'static str, width: u16, height: u16) {
        let mut routed = Scene::new(label, Theme::junie(), ColorLevel::TrueColor, width, height);
        routed.draw(|ui, area| super::paint_env_body(app, ui, area));

        let mut legacy = Scene::new(
            "editor_env_unproved_content_legacy_reference",
            Theme::junie(),
            ColorLevel::TrueColor,
            width,
            height,
        );
        legacy.draw(|ui, _| super::paint_env_body_legacy(app, ui));

        let mut shared_probe = Scene::new(
            "editor_env_unproved_content_shared_rejection",
            Theme::junie(),
            ColorLevel::TrueColor,
            width,
            height,
        );
        shared_probe.draw(|ui, area| {
            assert!(!super::paint_shared_env_body(app, ui, area));
        });

        for y in 0..height {
            for x in 0..width {
                let actual = routed.buffer().cell((x, y)).expect("routed cell");
                let expected = legacy.buffer().cell((x, y)).expect("legacy cell");
                assert_eq!(
                    format!("{actual:?}"),
                    format!("{expected:?}"),
                    "{label}: native cell differs at ({x}, {y})"
                );
            }
        }
    }

    fn assert_unproved_app_content_falls_back(app: &App, label: &'static str) {
        assert_app_fallback_matches_legacy(app, label, WIDTH, HEIGHT);
    }

    fn assert_env_projection_is_redacted(harness: &Harness<App>) {
        let text = harness.text();
        assert!(text.contains("DATABASE_URL"), "{text}");
        assert!(
            text.contains("$GH_TOKEN"),
            "HostEnv reference not literal: {text}"
        );
        assert!(text.contains("host env"), "HostEnv source missing: {text}");
        assert!(
            text.contains("[op] Engineering › Stripe · sandbox › credential"),
            "OnePassword display path missing: {text}"
        );
        assert!(
            text.contains("Role overrides"),
            "role section missing: {text}"
        );
        assert!(text.contains("Role: backend"), "role leaf missing: {text}");
        assert!(text.contains("OPENAI_API_KEY"), "role row missing: {text}");
        assert!(
            text.contains("[op] Engineering › OpenAI · Codex Primary › credential"),
            "role OnePassword path missing: {text}"
        );
        assert!(
            !text.contains("pw-fixture-only") && !text.contains("postgres://"),
            "plain value leaked: {text}"
        );
    }

    #[test]
    fn returning_editor_environment_matches_old_body_clean_changed_and_preview() {
        let mut harness = editor_environment_harness();
        harness.draw();
        assert_env_projection_is_redacted(&harness);
        assert_matches_preimage(&harness, "editor_env_old_clean_body_reference");

        let save = crate::screens::editor::SAVE;
        assert_save_owners(&harness, false);
        assert!(harness.tab_to(save), "clean Editor Save owner is focusable");
        assert_eq!(harness.focus(), Some(save));
        assert!(harness.state_of(save).contains(StateFlags::FOCUSED));
        assert_save_owners(&harness, false);
        // The fixture already carries a public HostEnv reference. Exercise
        // the normal add form, then prove the value stays pending until its
        // separate workspace Save confirmation is activated.
        let _ = harness.key(KeyCode::Char('a'));
        let _ = harness.key(KeyCode::Enter);
        assert!(harness.app().editor.env_form_open);
        assert_save_owners(&harness, false);
        let _ = harness.type_str("NEW_SECRET");
        let _ = harness.key(KeyCode::Tab);
        let _ = harness.key(KeyCode::Tab);
        let _ = harness.key(KeyCode::Enter);
        let _ = harness.type_str("sk-live-abcdefghijklmnop1234");
        assert!(harness.tab_to(save), "normal environment form Save focus");
        let _ = harness.key(KeyCode::Enter);
        assert!(!harness.app().editor.env_form_open);
        assert!(harness.app().editor.dirty);
        assert_save_owners(&harness, false);
        assert!(
            harness
                .app()
                .editor
                .pending
                .env
                .iter()
                .any(|env| env.key == "NEW_SECRET"),
            "valid value did not enter the pending editor"
        );
        assert!(
            harness
                .app()
                .world
                .workspace(1)
                .is_some_and(|workspace| workspace.env.iter().all(|env| env.key != "NEW_SECRET")),
            "draft environment value committed before workspace Save"
        );
        assert!(!harness.text().contains("abcdefghijklmnop"));

        // The successful add leaves a transient status that excludes this
        // approved full-frame painter. Clear only that message after the real
        // input path and pending/world ownership have been asserted. The next
        // native comparison is a controlled post-edit frame with the exact
        // unchanged status gate open, not the immediate status-bearing frame.
        harness.app_mut().status = None;
        harness.draw();
        assert_matches_preimage(&harness, "editor_env_old_changed_body_reference");
        assert_save_owners(&harness, false);

        let _ = harness.ctrl('s');
        assert!(harness.app().editor.preview_open);
        assert_eq!(harness.focus(), Some(crate::screens::editor::SAVE));
        assert!(harness.state_of(save).contains(StateFlags::FOCUSED));
        assert_save_owners(&harness, true);
        assert!(harness.text().contains("Save workspace"));
        assert!(harness.text().contains("NEW_SECRET"));
        assert!(!harness.text().contains("abcdefghijklmnop"));
        assert_matches_preimage(&harness, "editor_env_old_preview_body_reference");

        // The real first Escape leaves the focused Editor field and returns
        // focus to Save. The next Escape opens the leave confirmation; its
        // Cancel action must preserve the active preview and pending value.
        harness.app_mut().editor_leaf_focused = true;
        let _ = harness.key(KeyCode::Esc);
        assert!(!harness.app().editor_exit_open);
        assert_eq!(harness.app().route(), crate::Route::Editor);
        assert!(harness.app().editor.dirty);
        assert!(harness.app().editor.preview_open);
        assert_eq!(harness.focus(), Some(save));
        assert!(
            harness
                .app()
                .editor
                .pending
                .env
                .iter()
                .any(|env| env.key == "NEW_SECRET")
        );
        assert!(harness.diagnostics().is_empty());

        let _ = harness.key(KeyCode::Esc);
        assert!(harness.app().editor_exit_open);
        assert_eq!(harness.app().route(), crate::Route::Editor);
        let exit_cancel = App::editor_exit_dialog().action_id(0);
        assert!(
            harness.tab_to(exit_cancel),
            "editor exit Cancel is focusable"
        );
        assert_eq!(harness.focus(), Some(exit_cancel));
        assert!(harness.state_of(exit_cancel).contains(StateFlags::FOCUSED));
        assert_published_owner(&harness, exit_cancel, "Editor exit Cancel");
        assert!(harness.diagnostics().is_empty());
        let _ = harness.key(KeyCode::Enter);
        assert!(!harness.app().editor_exit_open);
        assert!(harness.app().editor.dirty);
        assert!(harness.app().editor.preview_open);
        assert!(
            harness
                .app()
                .editor
                .pending
                .env
                .iter()
                .any(|env| env.key == "NEW_SECRET"),
            "cancelling the exit confirmation discarded the pending value"
        );
        assert!(
            harness
                .app()
                .world
                .workspace(1)
                .is_some_and(|workspace| workspace.env.iter().all(|env| env.key != "NEW_SECRET")),
            "cancelling the exit confirmation committed the draft"
        );
        assert_eq!(harness.area_of(exit_cancel), None);
        assert_save_owners(&harness, true);

        // Reassert the existing preview owner through its real Ctrl+S command,
        // then use the native confirmation to commit the draft.
        let _ = harness.ctrl('s');
        assert!(harness.app().editor.preview_open);
        assert_eq!(harness.focus(), Some(crate::screens::editor::SAVE));
        assert!(harness.state_of(save).contains(StateFlags::FOCUSED));
        assert_save_owners(&harness, true);
        assert_matches_preimage(&harness, "editor_env_old_preview_after_cancel_reference");

        let confirm = crate::app::EDITOR_SAVE_CONFIRM;
        assert!(
            harness.tab_to(confirm),
            "preview confirmation owner is focusable"
        );
        assert_eq!(harness.focus(), Some(confirm));
        assert!(harness.state_of(confirm).contains(StateFlags::FOCUSED));
        assert_save_owners(&harness, true);
        let _ = harness.key(KeyCode::Enter);
        assert!(
            harness.app().editor.is_saving(),
            "native preview confirmation did not admit the save operation"
        );
        assert_eq!(harness.app().motion, Motion::Paused);
        harness.ticks(20);
        assert!(harness.app().editor.is_saving());
        assert!(
            harness
                .app()
                .world
                .workspace(1)
                .is_some_and(|workspace| workspace.env.iter().all(|env| env.key != "NEW_SECRET")),
            "paused product ticks committed the pending environment"
        );

        // All paused-frame parity checks are complete. Explicitly enable the
        // product and simulated-world clocks for the asynchronous save
        // completion; this phase does not claim paused-frame parity.
        harness.app_mut().motion = Motion::Full;
        harness.app_mut().world.clock.running = true;
        let _ = harness.tick();
        let mut save_clock_samples = vec![(
            "initial-tick",
            harness.now(),
            harness.next_deadline(),
            harness.app().world.now_ms(),
            harness.app().editor.is_saving(),
        )];
        for step in 0..20 {
            if !harness.app().editor.is_saving() {
                break;
            }
            let _ = harness.advance(core::time::Duration::from_millis(200));
            if step == 0 || step == 19 {
                save_clock_samples.push((
                    "advance",
                    harness.now(),
                    harness.next_deadline(),
                    harness.app().world.now_ms(),
                    harness.app().editor.is_saving(),
                ));
            }
        }
        let workspace_env_keys = harness
            .app()
            .world
            .workspaces
            .iter()
            .map(|workspace| {
                (
                    workspace.id,
                    workspace
                        .env
                        .iter()
                        .map(|env| env.key.as_str())
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        let pending_env_keys = harness
            .app()
            .editor
            .pending
            .env
            .iter()
            .map(|env| env.key.as_str())
            .collect::<Vec<_>>();
        assert!(
            harness
                .app()
                .world
                .workspace(1)
                .is_some_and(|workspace| workspace.env.iter().any(|env| env.key == "NEW_SECRET")),
            "enabled product ticks did not settle the admitted save: clock_samples={save_clock_samples:?}, workspace_id={:?}, saving={}, dirty={}, preview={}, route={:?}, motion={:?}, world_ms={}, world_clock_running={}, harness_now={:?}, status={:?}, diagnostics={:?}, workspace_env_keys={workspace_env_keys:?}, pending_env_keys={pending_env_keys:?}",
            harness.app().editor.workspace_id(),
            harness.app().editor.is_saving(),
            harness.app().editor.dirty,
            harness.app().editor.preview_open,
            harness.app().route(),
            harness.app().motion,
            harness.app().world.now_ms(),
            harness.app().world.clock.running,
            harness.now(),
            harness.app().status,
            harness.diagnostics()
        );
        assert!(!harness.app().editor.is_saving());
        assert!(!harness.app().editor.preview_open);
        assert_eq!(harness.app().route(), crate::Route::Manager);
        assert_eq!(harness.area_of(save), None);
        assert_eq!(harness.area_of(confirm), None);
        assert!(
            !harness.state_of(save).contains(StateFlags::PRESSED)
                && !harness.state_of(confirm).contains(StateFlags::PRESSED),
            "committed editor retained pressed/captured Save state"
        );
        assert!(
            harness.diagnostics().is_empty(),
            "commit produced runtime diagnostics: {:?}",
            harness.diagnostics()
        );
    }

    #[test]
    fn returning_editor_environment_invalid_key_never_stages() {
        let mut harness = editor_environment_harness();
        let before_pending_len = harness.app().editor.pending.env.len();
        let _ = harness.key(KeyCode::Char('a'));
        let _ = harness.key(KeyCode::Enter);
        let _ = harness.type_str("BAD-NAME");
        assert!(harness.tab_to(crate::screens::editor::SAVE));
        let _ = harness.key(KeyCode::Enter);
        assert!(harness.app().editor.env_form_open);
        assert_eq!(harness.app().editor.pending.env.len(), before_pending_len);
        assert!(
            harness
                .app()
                .editor
                .pending
                .env
                .iter()
                .all(|env| env.key != "BAD-NAME")
        );
        assert!(
            harness
                .app()
                .world
                .workspace(1)
                .is_some_and(|workspace| workspace.env.iter().all(|env| env.key != "BAD-NAME")),
            "invalid key reached the saved workspace"
        );
        assert!(harness.text().contains("letters, digits and underscores"));
        assert!(!harness.text().contains("pw-fixture-only"));
    }

    #[test]
    fn editor_environment_projection_falls_back_for_unproved_content() {
        let mut app = App::for_scenario_at(Scenario::Returning, Motion::Paused, 0);
        let workspace = app
            .world
            .workspaces
            .first()
            .expect("fixture workspace")
            .clone();
        app.editor.load_workspace(&workspace);
        app.editor.tab = crate::screens::editor::Tab::Environments;
        app.editor.pending.env = (0..40)
            .map(|index| EnvVar {
                key: format!("KEY_{index:02}"),
                value: EnvValue::HostEnv("TERM_PROGRAM".to_owned()),
            })
            .collect();
        let mut shared = Scene::new(
            "editor_env_overflow_fallback_actual_call",
            Theme::junie(),
            ColorLevel::TrueColor,
            WIDTH,
            HEIGHT,
        );
        shared.draw(|ui, area| super::paint_env_body(&app, ui, area));
        let mut legacy = Scene::new(
            "editor_env_overflow_fallback_preimage",
            Theme::junie(),
            ColorLevel::TrueColor,
            WIDTH,
            HEIGHT,
        );
        legacy.draw(|ui, _| super::paint_env_body_legacy(&app, ui));
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let actual = shared.buffer().cell((x, y)).expect("shared fallback cell");
                let expected = legacy.buffer().cell((x, y)).expect("legacy fallback cell");
                assert_eq!(
                    format!("{actual:?}"),
                    format!("{expected:?}"),
                    "overflow fallback changed cell ({x}, {y})"
                );
            }
        }
    }

    #[test]
    fn editor_environment_fixed_columns_fall_back_for_long_and_wide_fields() {
        let mut long_key = editor_environment_app();
        long_key.editor.pending.env[0].key = "K".repeat(21);

        let mut wide_key = editor_environment_app();
        wide_key.editor.pending.env[0].key = "界".to_owned();

        let mut long_host = editor_environment_app();
        let host = long_host
            .editor
            .pending
            .env
            .iter_mut()
            .find(|env| env.key == "GH_TOKEN")
            .expect("fixture HostEnv row");
        if let EnvValue::HostEnv(name) = &mut host.value {
            *name = "H".repeat(24);
        } else {
            panic!("GH_TOKEN fixture value is not HostEnv");
        }

        let mut wide_host = editor_environment_app();
        let host = wide_host
            .editor
            .pending
            .env
            .iter_mut()
            .find(|env| env.key == "GH_TOKEN")
            .expect("fixture HostEnv row");
        if let EnvValue::HostEnv(name) = &mut host.value {
            *name = "界".repeat(5);
        } else {
            panic!("GH_TOKEN fixture value is not HostEnv");
        }

        let mut long_source = editor_environment_app();
        let op = long_source
            .editor
            .pending
            .env
            .iter_mut()
            .find(|env| env.key == "STRIPE_KEY")
            .expect("fixture OnePassword row");
        if let EnvValue::OnePassword(reference) = &mut op.value {
            reference.item_title = "S".repeat(70);
        } else {
            panic!("STRIPE_KEY fixture value is not OnePassword");
        }

        let mut wide_source = editor_environment_app();
        let op = wide_source
            .editor
            .pending
            .env
            .iter_mut()
            .find(|env| env.key == "STRIPE_KEY")
            .expect("fixture OnePassword row");
        if let EnvValue::OnePassword(reference) = &mut op.value {
            reference.item_title = "界".repeat(10);
        } else {
            panic!("STRIPE_KEY fixture value is not OnePassword");
        }

        for (label, app) in [
            ("editor_env_long_key_fallback", &long_key),
            ("editor_env_wide_key_fallback", &wide_key),
            ("editor_env_long_host_fallback", &long_host),
            ("editor_env_wide_host_fallback", &wide_host),
            ("editor_env_long_source_fallback", &long_source),
            ("editor_env_wide_source_fallback", &wide_source),
        ] {
            assert_unproved_app_content_falls_back(app, label);
        }
    }

    #[test]
    fn editor_environment_supported_width_known_content_uses_shared_body() {
        let app = editor_environment_app();
        let mut shared = Scene::new(
            "editor_env_supported_shared_body",
            Theme::junie(),
            ColorLevel::TrueColor,
            WIDTH,
            HEIGHT,
        );
        shared.draw(|ui, area| {
            ui.fill(area, super::canvas());
            assert!(super::paint_shared_env_body(&app, ui, area));
        });

        let mut routed = Scene::new(
            "editor_env_supported_routed_body",
            Theme::junie(),
            ColorLevel::TrueColor,
            WIDTH,
            HEIGHT,
        );
        routed.draw(|ui, area| {
            ui.fill(area, super::canvas());
            super::paint_env_body(&app, ui, area);
        });

        let mut legacy = Scene::new(
            "editor_env_supported_legacy_reference",
            Theme::junie(),
            ColorLevel::TrueColor,
            WIDTH,
            HEIGHT,
        );
        legacy.draw(|ui, area| {
            ui.fill(area, super::canvas());
            super::paint_env_body_legacy(&app, ui);
        });

        for y in 6..37 {
            for x in 0..WIDTH {
                let routed_cell = routed.buffer().cell((x, y)).expect("routed cell");
                let shared_cell = shared.buffer().cell((x, y)).expect("shared cell");
                let legacy_cell = legacy.buffer().cell((x, y)).expect("legacy cell");
                assert_eq!(
                    format!("{routed_cell:?}"),
                    format!("{shared_cell:?}"),
                    "App did not route supported content through the shared body at ({x}, {y})"
                );
                assert_eq!(
                    format!("{routed_cell:?}"),
                    format!("{legacy_cell:?}"),
                    "supported fixed columns changed at ({x}, {y})"
                );
            }
        }
    }

    #[test]
    fn editor_environment_unproved_screen_geometry_uses_legacy_body() {
        let app = editor_environment_app();
        assert_app_fallback_matches_legacy(
            &app,
            "editor_env_wider_screen_legacy_fallback",
            WIDTH + 1,
            HEIGHT,
        );
        assert_app_fallback_matches_legacy(
            &app,
            "editor_env_taller_screen_legacy_fallback",
            WIDTH,
            HEIGHT + 1,
        );
    }
}
