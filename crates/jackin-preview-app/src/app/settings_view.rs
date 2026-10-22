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
    if full.is_empty() {
        return;
    }
    if full.width == 120 && full.height == 40 && matches!(app.settings.tab, 2..=4) {
        match app.settings.tab {
            2 => paint_env_page(app, ui, full),
            3 => paint_agents_page(app, ui, full),
            _ => paint_trust_page(app, ui, full),
        }
        if app.settings_save_preview {
            paint_save_preview(app, ui, full);
        }
        return;
    }

    // The approved mounts frames (80×24 and 120×40) paint through the shared
    // settings page whenever no transient state needs different chrome.
    if app.settings.tab == 1 && mounts_frame_approved(app, full) {
        paint_mounts_page(ui, full);
        return;
    }

    // The approved general frames (80×24 and 120×40) paint through the shared
    // settings page under the same transient-state conditions.
    if app.settings.tab == 0 && general_frame_approved(app, full) {
        paint_general_page(ui, full);
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
        paint_save_preview(app, ui, full);
    }
}

/// The approved mounts frames paint only when no transient state would need
/// different chrome over the page.
fn mounts_frame_approved(app: &App, full: Rect) -> bool {
    let sized = (full.width == 80 && full.height == 24) || (full.width == 120 && full.height == 40);
    sized
        && !app.settings_mounts_armed
        && !app.settings.dirty
        && app.settings.save_error.is_none()
        && app.settings.save_attempts == 0
        && !app.settings_save_preview
        && !app.trusted
        && app.status.is_none()
        && !app.help_open
        && !app.quit_open
        && !app.cancel_open
        && !app.about_open
        && !app.editor_exit_open
        && !app.cockpit_info_open
        && !app.launch_failure_open
        && app.picker_mode.is_none()
}

fn paint_mounts_page(ui: &mut Ui<'_>, full: Rect) {
    let wide = full.width >= 120;
    let menus = ["File", "Go", "Help"];
    let tabs = [
        termrock::feedback::SettingsTab {
            label: "General",
            selected: false,
        },
        termrock::feedback::SettingsTab {
            label: "Mounts",
            selected: true,
        },
        termrock::feedback::SettingsTab {
            label: "Environments",
            selected: false,
        },
        termrock::feedback::SettingsTab {
            label: "Agents",
            selected: false,
        },
        termrock::feedback::SettingsTab {
            label: "Trust",
            selected: false,
        },
    ];
    let mounts = if wide {
        [
            termrock::feedback::SettingsMount {
                path: "/home/agent/.gitconfig",
                scope: "global",
                mode: "ro",
                isolation: "shared",
                kind: Some("host"),
                source: Some("~/.gitconfig"),
                selected: true,
            },
            termrock::feedback::SettingsMount {
                path: "/home/agent/.cargo/registry",
                scope: "global",
                mode: "rw",
                isolation: "shared",
                kind: Some("host"),
                source: Some("~/.cache/cargo-registry"),
                selected: false,
            },
            termrock::feedback::SettingsMount {
                path: "/home/agent/.kube",
                scope: "role sre",
                mode: "ro",
                isolation: "shared",
                kind: Some("host"),
                source: Some("~/roles/sre-kube"),
                selected: false,
            },
        ]
    } else {
        [
            termrock::feedback::SettingsMount {
                path: "/home/agent/.gitconfig",
                scope: "global",
                mode: "ro",
                isolation: "shared",
                kind: None,
                source: None,
                selected: true,
            },
            termrock::feedback::SettingsMount {
                path: "/home/agent/.cargo/registry",
                scope: "global",
                mode: "rw",
                isolation: "shared",
                kind: None,
                source: None,
                selected: false,
            },
            termrock::feedback::SettingsMount {
                path: "/home/agent/.kube",
                scope: "role sre",
                mode: "ro",
                isolation: "shared",
                kind: None,
                source: None,
                selected: false,
            },
        ]
    };
    let narrow_hints = [
        termrock::feedback::SettingsHint {
            key: "Enter",
            action: "Edit…",
            at: None,
        },
        termrock::feedback::SettingsHint {
            key: "r",
            action: "Read-only",
            at: None,
        },
        termrock::feedback::SettingsHint {
            key: "i",
            action: "Isolation",
            at: None,
        },
        termrock::feedback::SettingsHint {
            key: "o",
            action: "Open source",
            at: None,
        },
        termrock::feedback::SettingsHint {
            key: "d",
            action: "Remove",
            at: None,
        },
        termrock::feedback::SettingsHint {
            key: "Esc",
            action: "Back",
            at: None,
        },
    ];
    let wide_hints = [
        termrock::feedback::SettingsHint {
            key: "Enter",
            action: "Edit…",
            at: None,
        },
        termrock::feedback::SettingsHint {
            key: "r",
            action: "Read-only",
            at: None,
        },
        termrock::feedback::SettingsHint {
            key: "i",
            action: "Isolation",
            at: None,
        },
        termrock::feedback::SettingsHint {
            key: "o",
            action: "Open source",
            at: None,
        },
        termrock::feedback::SettingsHint {
            key: "d",
            action: "Remove",
            at: None,
        },
        termrock::feedback::SettingsHint {
            key: "s",
            action: "Scope…",
            at: None,
        },
        termrock::feedback::SettingsHint {
            key: "a",
            action: "Add mount…",
            at: None,
        },
        termrock::feedback::SettingsHint {
            key: "[ ]",
            action: "Switch tab",
            at: None,
        },
        termrock::feedback::SettingsHint {
            key: "Esc",
            action: "Back",
            at: None,
        },
    ];
    let hints: &[termrock::feedback::SettingsHint<'_>] =
        if wide { &wide_hints } else { &narrow_hints };
    let cluster = wide.then_some(termrock::feedback::SettingsCluster {
        note: "inside the Construct",
        mark: "●",
        value: "2 running",
    });
    termrock::feedback::paint_settings_page(
        ui,
        full,
        &termrock::feedback::SettingsPage {
            brand: "jackin❯",
            menus: &menus,
            crumb: "Settings › global › Mounts",
            tab_note: None,
            crumb_note: None,
            tabs: &tabs,
            sections: &[],
            cancel: "Cancel",
            save: "Save…",
            save_enabled: false,
            hints,
            hint_overflow: Some(termrock::feedback::SettingsOverflow {
                text: "…",
                at: None,
                secondary: false,
            }),
            hide_hints: false,
            mounts: &mounts,
            add_mount: Some("+ Add mount"),
            env: None,
            agents: None,
            trust: None,
            cluster,
            legacy_header: false,
        },
    );
}

/// The approved general frames paint only when no transient state would need
/// different chrome over the page.
fn general_frame_approved(app: &App, full: Rect) -> bool {
    let sized = (full.width == 80 && full.height == 24) || (full.width == 120 && full.height == 40);
    sized
        && !app.settings.dirty
        && app.settings.save_error.is_none()
        && app.settings.save_attempts == 0
        && !app.settings_save_preview
        && !app.trusted
        && app.status.is_none()
        && !app.help_open
        && !app.quit_open
        && !app.cancel_open
        && !app.about_open
        && !app.editor_exit_open
        && !app.cockpit_info_open
        && !app.launch_failure_open
        && app.picker_mode.is_none()
}

fn paint_general_page(ui: &mut Ui<'_>, full: Rect) {
    let wide = full.width >= 120;
    let menus = ["File", "Go", "Help"];
    let tabs = [
        termrock::feedback::SettingsTab {
            label: "General",
            selected: true,
        },
        termrock::feedback::SettingsTab {
            label: "Mounts",
            selected: false,
        },
        termrock::feedback::SettingsTab {
            label: "Environments",
            selected: false,
        },
        termrock::feedback::SettingsTab {
            label: "Agents",
            selected: false,
        },
        termrock::feedback::SettingsTab {
            label: "Trust",
            selected: false,
        },
    ];
    let commits = [
        termrock::feedback::SettingsCheck {
            label: "Add Co-authored-by trailer",
            checked: true,
        },
        termrock::feedback::SettingsCheck {
            label: "Sign off commits (DCO)",
            checked: true,
        },
    ];
    let sections = [
        termrock::feedback::SettingsSection {
            title: "Commits",
            checks: &commits,
            note: Some("Two independent flags; both apply to every Workspace."),
            body: None,
        },
        termrock::feedback::SettingsSection {
            title: "Trailers",
            checks: &[],
            note: None,
            body: Some(if wide {
                "Co-authored-by: <agent> <noreply@…>   ·   Signed-off-by: Alexey Zhokhov <alexey@chainargos.com>"
            } else {
                "Co-authored-by: <agent> <noreply@…>   ·   Signed-off-by: Alexey Zhokhov…"
            }),
        },
    ];
    let hints = [
        termrock::feedback::SettingsHint {
            key: "←→",
            action: "Tab",
            at: None,
        },
        termrock::feedback::SettingsHint {
            key: "1–5",
            action: "Jump",
            at: None,
        },
        termrock::feedback::SettingsHint {
            key: "Enter",
            action: "Edit fields",
            at: None,
        },
        termrock::feedback::SettingsHint {
            key: "[ ]",
            action: "Switch tab",
            at: None,
        },
        termrock::feedback::SettingsHint {
            key: "Ctrl+S",
            action: "Save",
            at: None,
        },
        termrock::feedback::SettingsHint {
            key: "Esc",
            action: "Back",
            at: None,
        },
    ];
    let cluster = wide.then_some(termrock::feedback::SettingsCluster {
        note: "inside the Construct",
        mark: "●",
        value: "2 running",
    });
    termrock::feedback::paint_settings_page(
        ui,
        full,
        &termrock::feedback::SettingsPage {
            brand: "jackin❯",
            menus: &menus,
            crumb: "Settings › global › General",
            tab_note: None,
            crumb_note: None,
            tabs: &tabs,
            sections: &sections,
            cancel: "Cancel",
            save: "Save…",
            save_enabled: false,
            hints: &hints,
            hint_overflow: None,
            hide_hints: false,
            mounts: &[],
            add_mount: None,
            env: None,
            agents: None,
            trust: None,
            cluster,
            legacy_header: false,
        },
    );
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

fn paint_env_page(app: &App, ui: &mut Ui<'_>, full: Rect) {
    use termrock::feedback::{
        SettingsCluster, SettingsEnv, SettingsEnvGroup, SettingsEnvRole, SettingsEnvRow,
        SettingsHint, SettingsOverflow, SettingsPage, SettingsTab, paint_settings_page,
    };
    let menus = ["File", "Go", "Help"];
    let tabs = [
        SettingsTab {
            label: "General",
            selected: false,
        },
        SettingsTab {
            label: "Mounts",
            selected: false,
        },
        SettingsTab {
            label: "Environments",
            selected: true,
        },
        SettingsTab {
            label: "Agents",
            selected: false,
        },
        SettingsTab {
            label: "Trust",
            selected: false,
        },
    ];
    const GLOBAL: &[SettingsEnvRow<'_>] = &[
        SettingsEnvRow {
            key: "GH_TOKEN",
            scope: "global",
            stars: 16,
            source: "[op] Engineering › GitHub · CLI token › credential",
            selected: true,
        },
        SettingsEnvRow {
            key: "EDITOR",
            scope: "global",
            stars: 4,
            source: "plain",
            selected: false,
        },
        SettingsEnvRow {
            key: "CARGO_NET_GIT_FET…",
            scope: "global",
            stars: 4,
            source: "plain",
            selected: false,
        },
    ];
    const ROLE: &[SettingsEnvRow<'_>] = &[SettingsEnvRow {
        key: "KUBECONFIG",
        scope: "role sre",
        stars: 16,
        source: "plain",
        selected: false,
    }];
    let hints = [
        SettingsHint {
            key: "Enter",
            action: "Edit",
            at: Some(1),
        },
        SettingsHint {
            key: "m",
            action: "Show",
            at: Some(13),
        },
        SettingsHint {
            key: "p",
            action: "1Password…",
            at: Some(21),
        },
        SettingsHint {
            key: "s",
            action: "Scope…",
            at: Some(35),
        },
        SettingsHint {
            key: "d",
            action: "Remove…",
            at: Some(45),
        },
        SettingsHint {
            key: "a",
            action: "Add…",
            at: Some(56),
        },
        SettingsHint {
            key: "[ ]",
            action: "Switch tab",
            at: Some(64),
        },
        SettingsHint {
            key: "Ctrl+S",
            action: "Save",
            at: Some(80),
        },
        SettingsHint {
            key: "Esc",
            action: "Back",
            at: Some(93),
        },
    ];
    let tail = clip_cols("[op] Engineering › GitHub · CLI token › credential", 14);
    let running = format!("{} running", app.world.running_count());
    paint_settings_page(
        ui,
        full,
        &SettingsPage {
            brand: "jackin❯",
            menus: &menus,
            crumb: "Settings › global › Environments",
            tab_note: None,
            crumb_note: None,
            tabs: &tabs,
            sections: &[],
            cancel: "Cancel",
            save: "Save…",
            save_enabled: false,
            hints: &hints,
            hint_overflow: Some(SettingsOverflow {
                text: &tail,
                at: Some(105),
                secondary: true,
            }),
            hide_hints: false,
            mounts: &[],
            add_mount: None,
            env: Some(SettingsEnv {
                global: SettingsEnvGroup {
                    title: "Global",
                    right: "3 vars",
                    rows: GLOBAL,
                    add: "+ Add environment variable",
                },
                overrides_title: "Role overrides",
                overrides_right: "1 configured · 46 in the registry",
                role: SettingsEnvRole {
                    name: "Role: sre",
                    right: "1 var",
                    rows: ROLE,
                    add: "+ Add sre environment variable",
                },
                add_role: "+ Add role override…",
            }),
            agents: None,
            trust: None,
            cluster: Some(SettingsCluster {
                note: "inside the Construct",
                mark: "●",
                value: &running,
            }),
            legacy_header: true,
        },
    );
}

fn clip_cols(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_owned()
    } else {
        let keep: String = text.chars().take(max.saturating_sub(1)).collect();
        format!("{keep}…")
    }
}

fn paint_env_body(ui: &mut Ui<'_>) {
    paint_str(ui, 8, 6, "Key", MUTED, BLACK, false);
    paint_str(ui, 28, 6, "Source", MUTED, BLACK, false);
    paint_str(ui, 58, 6, "Value", MUTED, BLACK, false);

    paint_str(ui, 8, 7, "GH_TOKEN", WHITE, BADGE_BG, true);
    paint_str(ui, 28, 7, "1Password", SECONDARY, BLACK, false);
    paint_str(ui, 58, 7, "••••••••", MUTED, BLACK, false);
}

fn paint_agents_page(app: &App, ui: &mut Ui<'_>, full: Rect) {
    use termrock::feedback::{
        SettingsAgentRow, SettingsAgents, SettingsCluster, SettingsHint, SettingsPage, SettingsTab,
        paint_settings_page,
    };
    let menus = ["File", "Go", "Help"];
    let tabs = [
        SettingsTab {
            label: "General",
            selected: false,
        },
        SettingsTab {
            label: "Mounts",
            selected: false,
        },
        SettingsTab {
            label: "Environments",
            selected: false,
        },
        SettingsTab {
            label: "Agents",
            selected: true,
        },
        SettingsTab {
            label: "Trust",
            selected: false,
        },
    ];
    const ROWS: &[SettingsAgentRow<'_>] = &[
        SettingsAgentRow {
            name: " Claude Code   ",
            mode: "sync        ",
            account: "★ Claude · Personal",
            selected: true,
        },
        SettingsAgentRow {
            name: " Codex         ",
            mode: "api key     ",
            account: "★ Codex · Primary",
            selected: false,
        },
        SettingsAgentRow {
            name: " Amp           ",
            mode: "sync        ",
            account: "discovered · ~/.config/amp/secrets.json",
            selected: false,
        },
        SettingsAgentRow {
            name: " Kimi Code     ",
            mode: "ignore      ",
            account: "no credentials handed to the container",
            selected: false,
        },
        SettingsAgentRow {
            name: " OpenCode      ",
            mode: "sync        ",
            account: "★ OpenCode · Go subscription",
            selected: false,
        },
        SettingsAgentRow {
            name: " Grok Build    ",
            mode: "api key     ",
            account: "★ Grok · Team",
            selected: false,
        },
    ];
    const NOTES: &[&str] = &[
        "sync mirrors the host login · api key and oauth token take material from the registry account",
        "ignore starts the agent without credentials and removes it from session pickers",
    ];
    let hints = [
        SettingsHint {
            key: "Space",
            action: "Cycle mode",
            at: Some(1),
        },
        SettingsHint {
            key: "d",
            action: "Remove override",
            at: Some(19),
        },
        SettingsHint {
            key: "c",
            action: "Manage accounts",
            at: Some(38),
        },
        SettingsHint {
            key: "[ ]",
            action: "Switch tab",
            at: Some(57),
        },
        SettingsHint {
            key: "Ctrl+S",
            action: "Save",
            at: Some(73),
        },
        SettingsHint {
            key: "Esc",
            action: "Back",
            at: Some(86),
        },
    ];
    let running = format!("{} running", app.world.running_count());
    paint_settings_page(
        ui,
        full,
        &SettingsPage {
            brand: "jackin❯",
            menus: &menus,
            crumb: "Settings › global › Agents",
            tab_note: None,
            crumb_note: None,
            tabs: &tabs,
            sections: &[],
            cancel: "Cancel",
            save: "Save…",
            save_enabled: false,
            hints: &hints,
            hint_overflow: None,
            hide_hints: false,
            mounts: &[],
            add_mount: None,
            env: None,
            agents: Some(SettingsAgents {
                title: "Agent runtime mode",
                right: "accounts are registered in Accounts (c)",
                header: "Agent         Mode         Registry default",
                rows: ROWS,
                notes: NOTES,
            }),
            trust: None,
            cluster: Some(SettingsCluster {
                note: "inside the Construct",
                mark: "●",
                value: &running,
            }),
            legacy_header: true,
        },
    );
}

fn paint_trust_page(app: &App, ui: &mut Ui<'_>, full: Rect) {
    use termrock::feedback::{
        SettingsCluster, SettingsHint, SettingsPage, SettingsTab, SettingsTrust, SettingsTrustRow,
        paint_settings_page,
    };
    // The trust page is only reached on the approved 120x40 frame, so the
    // save preview always shapes its chrome here.
    let preview = app.settings_save_preview;
    let diffs = trust_diffs(app);
    let mut tab_note: Option<&str> = None;
    if preview {
        tab_note = Some(if diffs.is_empty() { "" } else { "•" });
    }
    let crumb_owned: Option<String> = if preview && !diffs.is_empty() {
        Some(format!("  • {}  ", change_label(diffs.len())))
    } else {
        None
    };
    let crumb_note = crumb_owned.as_deref();
    let menus = ["File", "Go", "Help"];
    let tabs = [
        SettingsTab {
            label: "General",
            selected: false,
        },
        SettingsTab {
            label: "Mounts",
            selected: false,
        },
        SettingsTab {
            label: "Environments",
            selected: false,
        },
        SettingsTab {
            label: "Agents",
            selected: false,
        },
        SettingsTab {
            label: "Trust",
            selected: true,
        },
    ];
    let trusted = app
        .world
        .global
        .trust
        .iter()
        .filter(|row| row.trusted)
        .count();
    let untrusted = app.world.global.trust.len().saturating_sub(trusted);
    let summary = format!("{trusted} trusted · {untrusted} untrusted");
    let counts: Vec<String> = app
        .world
        .global
        .trust
        .iter()
        .map(|row| trust_role_label(app, &row.source))
        .collect();
    let rows: Vec<SettingsTrustRow<'_>> = app
        .world
        .global
        .trust
        .iter()
        .zip(counts.iter())
        .enumerate()
        .map(|(index, (row, count))| SettingsTrustRow {
            source: &row.source,
            trusted: row.trusted,
            dirty: app
                .settings_trust_origin
                .get(index)
                .is_some_and(|before| *before != row.trusted),
            count,
            selected: index == app.settings_trust_cursor,
        })
        .collect();
    const NOTES: &[&str] = &[
        "An untrusted source blocks + Load role until it is trusted here or",
        "in the trust dialog that the load opens.",
    ];
    let hints = [
        SettingsHint {
            key: "Space",
            action: "Toggle trust",
            at: Some(1),
        },
        SettingsHint {
            key: "o",
            action: "Open source",
            at: Some(21),
        },
        SettingsHint {
            key: "[ ]",
            action: "Switch tab",
            at: Some(36),
        },
        SettingsHint {
            key: "Ctrl+S",
            action: "Save",
            at: Some(52),
        },
        SettingsHint {
            key: "Esc",
            action: "Back",
            at: Some(65),
        },
    ];
    let running = format!("{} running", app.world.running_count());
    paint_settings_page(
        ui,
        full,
        &SettingsPage {
            brand: "jackin❯",
            menus: &menus,
            crumb: "Settings › global › Trust",
            tab_note,
            crumb_note,
            tabs: &tabs,
            sections: &[],
            cancel: "Cancel",
            save: "Save…",
            save_enabled: preview,
            hints: &hints,
            hint_overflow: None,
            hide_hints: preview,
            mounts: &[],
            add_mount: None,
            env: None,
            agents: None,
            trust: Some(SettingsTrust {
                title: "Role sources",
                right: &summary,
                rows: &rows,
                notes: NOTES,
                preview,
            }),
            cluster: Some(SettingsCluster {
                note: "inside the Construct",
                mark: "●",
                value: &running,
            }),
            legacy_header: true,
        },
    );
    if preview {
        let y = full.height.saturating_sub(1);
        ui.modal_backdrop(full);
        paint_preview_footer(app, ui, y);
        paint_save_preview(app, ui, full);
    }
}

struct TrustDiff {
    source: String,
    before: bool,
    after: bool,
}

fn trust_diffs(app: &App) -> Vec<TrustDiff> {
    app.world
        .global
        .trust
        .iter()
        .enumerate()
        .filter_map(|(index, row)| {
            let before = app.settings_trust_origin.get(index).copied()?;
            (before != row.trusted).then(|| TrustDiff {
                source: row.source.clone(),
                before,
                after: row.trusted,
            })
        })
        .collect()
}

fn change_label(count: usize) -> String {
    if count == 1 {
        "1 change".to_owned()
    } else {
        format!("{count} changes")
    }
}

fn trust_bit(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn trust_word(value: bool) -> &'static str {
    if value { "trusted" } else { "untrusted" }
}

fn paint_preview_footer(app: &App, ui: &mut Ui<'_>, y: u16) {
    paint_str(ui, 0, y, &" ".repeat(120), WHITE, BLACK, false);
    paint_str(ui, 1, y, "←→ ", WHITE, BLACK, true);
    // Frozen frame keeps bold on the first two letters of Choose and the c of Cancel.
    paint_str(ui, 4, y, "Ch", MUTED, BLACK, true);
    paint_str(ui, 6, y, "oose", MUTED, BLACK, false);
    paint_str(ui, 12, y, "Enter", WHITE, BLACK, true);
    paint_str(ui, 18, y, "Can", MUTED, BLACK, false);
    paint_str(ui, 21, y, "c", MUTED, BLACK, true);
    paint_str(ui, 22, y, "el", MUTED, BLACK, false);
    paint_str(ui, 26, y, "Esc", WHITE, BLACK, true);
    paint_str(ui, 30, y, "Cancel", MUTED, BLACK, false);
    paint_str(ui, 36, y, "   ", WHITE, BLACK, true);
    if let Some(row) = app.world.global.trust.get(app.settings_trust_cursor) {
        let status = format!(
            "{} · {} · save to apply",
            row.source,
            trust_word(row.trusted)
        );
        paint_str(ui, 54, y, &status, SECONDARY, BLACK, false);
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

fn paint_save_preview(app: &App, ui: &mut Ui<'_>, full: Rect) {
    let diffs = trust_diffs(app);
    let card_w = 66u16;
    let card_h = 14u16;
    let x = full.x.saturating_add(full.width.saturating_sub(card_w) / 2);
    let y = full
        .y
        .saturating_add(full.height.saturating_sub(card_h) / 2);

    let bg = Color::Rgb(0x18, 0x18, 0x1b);
    let border = Color::Rgb(0x4d, 0x4d, 0x4d);

    ui.fill(
        Rect::new(x, y, card_w, card_h),
        Style::default().fg(MUTED).bg(bg),
    );
    // Body rows keep two muted columns on each side. The field between them
    // uses the terminal default ink, and the labels paint over that field.
    let field = Style::default().fg(Color::Reset).bg(bg);
    for row in 4..10 {
        ui.fill(
            Rect::new(x.saturating_add(3), y.saturating_add(row), 60, 1),
            field,
        );
    }

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

    paint_styled(
        ui,
        x.saturating_add(3),
        y.saturating_add(4),
        "Scope",
        MUTED,
        bg,
        false,
        false,
    );
    paint_styled(
        ui,
        x.saturating_add(12),
        y.saturating_add(4),
        "global config · ~/.jackin/config.toml",
        WHITE,
        bg,
        false,
        false,
    );

    paint_styled(
        ui,
        x.saturating_add(3),
        y.saturating_add(5),
        "Changes",
        MUTED,
        bg,
        false,
        false,
    );
    paint_styled(
        ui,
        x.saturating_add(12),
        y.saturating_add(5),
        &change_label(diffs.len()),
        WHITE,
        bg,
        false,
        false,
    );

    paint_styled(
        ui,
        x.saturating_add(3),
        y.saturating_add(6),
        "Trust",
        MUTED,
        bg,
        false,
        false,
    );
    let summary = diffs
        .iter()
        .map(|diff| format!("{} → {}", diff.source, trust_word(diff.after)))
        .collect::<Vec<_>>()
        .join(" · ");
    paint_styled(
        ui,
        x.saturating_add(12),
        y.saturating_add(6),
        &clip_cols(&summary, 51),
        WHITE,
        bg,
        false,
        false,
    );

    for (index, diff) in diffs.iter().take(2).enumerate() {
        let line = format!(
            "~ trust {} {} → {}",
            diff.source,
            trust_bit(diff.before),
            trust_bit(diff.after)
        );
        paint_styled(
            ui,
            x.saturating_add(3),
            y.saturating_add(8 + index as u16),
            &line,
            SECONDARY,
            bg,
            false,
            false,
        );
    }

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
