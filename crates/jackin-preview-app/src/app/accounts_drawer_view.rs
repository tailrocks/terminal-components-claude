use termrock::feedback::{
    AccountRole, AccountsDrawer, DetailAlert, DetailHint, DetailInk, DetailLine, DetailSpan,
    DrawerButton, DrawerInk, DrawerLine, DrawerMeter, DrawerTone, PaneChrome, PaneRow,
    SettingsCluster,
};
use termrock::{ColorLevel, ItemKey, Ui};

use crate::scenario::{Motion, Scenario};

use super::{App, Route};

pub(super) fn paint(app: &App, ui: &mut Ui<'_>) {
    if !approved_drawer(app, ui) {
        return;
    }
    paint_page(ui, PaneChrome::Drawer);
}

pub(super) fn paint_detail(app: &App, ui: &mut Ui<'_>) {
    if !approved_detail(app, ui) {
        return;
    }
    paint_page(ui, PaneChrome::Detail);
}

/// Four Downs on the paused 120×40 accounts list land on Claude · Work.
/// The live list never paints `Accounts › Claude › Work`.
fn approved_detail(app: &App, ui: &Ui<'_>) -> bool {
    let theme = ui.theme_ref();
    let screen = ui.full();
    let accounts = &app.accounts;
    screen.width == 120
        && screen.height == 40
        && theme.capability.color == ColorLevel::TrueColor
        && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
        && app.route == Route::Accounts
        && app.world.scenario == Scenario::AccountsMixed
        && app.motion == Motion::Paused
        && !accounts.form_open
        && !accounts.started
        && accounts.editing.is_none()
        && accounts.draft_name.is_empty()
        && accounts.masked_input.is_empty()
        && accounts.selected_op.is_none()
        && accounts.op_stage == 0
        && accounts.provider_index == 0
        && accounts.source_index == 0
        && accounts.selected_id.as_deref() == Some("acct-claude-work")
        && accounts.list.cursor() == Some(ItemKey::index(4))
        && accounts.pending_refresh.is_none()
        && accounts.remove_confirmation.is_none()
        && !accounts.filtering
        && accounts.filter_query.is_empty()
        && !accounts.drawer_open
        && app.status.as_deref() == Some("Accounts › Claude › Work")
        && !app.help_open
        && !app.quit_open
        && !app.cancel_open
        && !app.about_open
        && app.picker_mode.is_none()
}

/// Enter after four Downs on the paused 120×40 accounts list.
/// The detail frame is the same row with the drawer still closed.
fn approved_drawer(app: &App, ui: &Ui<'_>) -> bool {
    let theme = ui.theme_ref();
    let screen = ui.full();
    let accounts = &app.accounts;
    screen.width == 120
        && screen.height == 40
        && theme.capability.color == ColorLevel::TrueColor
        && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
        && app.route == Route::Accounts
        && app.world.scenario == Scenario::AccountsMixed
        && app.motion == Motion::Paused
        && !accounts.form_open
        && !accounts.started
        && accounts.editing.is_none()
        && accounts.draft_name.is_empty()
        && accounts.masked_input.is_empty()
        && accounts.selected_op.is_none()
        && accounts.op_stage == 0
        && accounts.provider_index == 0
        && accounts.source_index == 0
        && accounts.selected_id.as_deref() == Some("acct-claude-work")
        && accounts.list.cursor() == Some(ItemKey::index(4))
        && accounts.pending_refresh.is_none()
        && accounts.remove_confirmation.is_none()
        && !accounts.filtering
        && accounts.filter_query.is_empty()
        && accounts.drawer_open
        && app.status.as_deref() == Some("Accounts › Claude › Work")
        && !app.help_open
        && !app.quit_open
        && !app.cancel_open
        && !app.about_open
        && app.picker_mode.is_none()
}

fn paint_page(ui: &mut Ui<'_>, chrome: PaneChrome) {
    let mut rows = vec![
        pane(
            AccountRole::Root,
            None,
            "Overview",
            DrawerInk::Plain,
            None,
            false,
        ),
        pane(
            AccountRole::Group,
            Some("▾"),
            "Claude",
            DrawerInk::Plain,
            Some(("!", DetailAlert::Danger)),
            false,
        ),
        pane(
            AccountRole::Item,
            Some("★"),
            "Personal",
            DrawerInk::Plain,
            None,
            false,
        ),
        pane(
            AccountRole::Note,
            None,
            "Archived contractor laptop …",
            DrawerInk::Muted,
            None,
            false,
        ),
        pane(
            AccountRole::Item,
            None,
            "Work",
            DrawerInk::Plain,
            Some(("!", DetailAlert::Danger)),
            true,
        ),
        pane(
            AccountRole::Group,
            Some("▾"),
            "Codex",
            DrawerInk::Plain,
            None,
            false,
        ),
        pane(
            AccountRole::Item,
            Some("★"),
            "Primary",
            DrawerInk::Plain,
            None,
            false,
        ),
        pane(
            AccountRole::Item,
            None,
            "Experiments",
            DrawerInk::Plain,
            None,
            false,
        ),
        pane(
            AccountRole::Group,
            Some("▾"),
            "Amp",
            DrawerInk::Plain,
            Some(("▲", DetailAlert::Warning)),
            false,
        ),
        pane(
            AccountRole::Item,
            None,
            "discovered",
            DrawerInk::Plain,
            Some(("▲", DetailAlert::Warning)),
            false,
        ),
        pane(
            AccountRole::Group,
            Some("▾"),
            "Grok",
            DrawerInk::Plain,
            None,
            false,
        ),
        pane(
            AccountRole::Item,
            Some("★"),
            "Team",
            DrawerInk::Plain,
            None,
            false,
        ),
        pane(
            AccountRole::Group,
            Some("▾"),
            "Z.AI",
            DrawerInk::Plain,
            Some(("▲", DetailAlert::Warning)),
            false,
        ),
        pane(
            AccountRole::Item,
            None,
            "discovered",
            DrawerInk::Plain,
            Some(("▲", DetailAlert::Warning)),
            false,
        ),
        pane(
            AccountRole::Group,
            Some("▾"),
            "Kimi",
            DrawerInk::Plain,
            Some(("!", DetailAlert::Danger)),
            false,
        ),
        pane(
            AccountRole::Item,
            None,
            "discovered",
            DrawerInk::Plain,
            Some(("!", DetailAlert::Danger)),
            false,
        ),
        pane(
            AccountRole::Group,
            Some("▾"),
            "MiniMax",
            DrawerInk::Plain,
            Some(("!", DetailAlert::Danger)),
            false,
        ),
        pane(
            AccountRole::Item,
            None,
            "discovered",
            DrawerInk::Plain,
            Some(("!", DetailAlert::Danger)),
            false,
        ),
        pane(
            AccountRole::Group,
            Some("▾"),
            "OpenCode",
            DrawerInk::Plain,
            Some(("!", DetailAlert::Danger)),
            false,
        ),
        pane(
            AccountRole::Item,
            Some("★"),
            "Go subscription",
            DrawerInk::Plain,
            Some(("!", DetailAlert::Danger)),
            false,
        ),
        pane(
            AccountRole::Item,
            None,
            "ci-bot · discovered",
            DrawerInk::Plain,
            None,
            false,
        ),
        pane(
            AccountRole::Root,
            None,
            "Unsupported",
            DrawerInk::Muted,
            None,
            false,
        ),
        pane(
            AccountRole::Root,
            None,
            "+ Add account…",
            DrawerInk::Secondary,
            None,
            false,
        ),
    ];
    for _ in 0..11 {
        rows.push(PaneRow {
            role: AccountRole::Root,
            mark: None,
            name: "",
            ink: DrawerInk::Plain,
            alert: None,
            alert_tone: DetailAlert::Danger,
            selected: false,
            empty: true,
        });
    }
    let validation = [
        DetailSpan {
            text: "  ",
            ink: DetailInk::Plain,
        },
        DetailSpan {
            text: "Validation",
            ink: DetailInk::Muted,
        },
        DetailSpan {
            text: "          ✓ material   ✓ identity   ✓ quota access · quota",
            ink: DetailInk::Plain,
        },
    ];
    let validation_line = DetailLine { spans: &validation };
    let lines = [
        DrawerLine::Title {
            title: "Claude · Work",
            meta: "account · exhausted",
        },
        DrawerLine::Rule,
        field("Provider", "Anthropic / Claude", DrawerInk::Plain),
        field("Agent runtime", "Claude Code", DrawerInk::Plain),
        field("Usage surface", "Claude", DrawerInk::Plain),
        field("Identity", "alexey@chainargos.com", DrawerInk::Plain),
        field("Plan", "Team", DrawerInk::Plain),
        field(
            "Credential",
            "1Password · Engineering › Anthropic · Work ›",
            DrawerInk::Plain,
        ),
        DrawerLine::Value {
            text: "credential",
            ink: DrawerInk::Plain,
        },
        DrawerLine::Value {
            text: "chainargos.1password.com ·",
            ink: DrawerInk::Muted,
        },
        DrawerLine::Value {
            text: "op://v_eng01/it_ant01/credential · ••••••••…3c9e",
            ink: DrawerInk::Muted,
        },
        field(
            "Provenance",
            "configured source · confidence authoritative ·",
            DrawerInk::Secondary,
        ),
        DrawerLine::Value {
            text: "registered",
            ink: DrawerInk::Secondary,
        },
        field("Lifecycle", "available", DrawerInk::Plain),
        field("Default", "no", DrawerInk::Plain),
        field("Enabled", "yes", DrawerInk::Plain),
        field("Purpose", "work", DrawerInk::Plain),
        field(
            "Used by",
            "Workspace choice in payments-platform",
            DrawerInk::Secondary,
        ),
        DrawerLine::Blank,
        DrawerLine::Heading {
            label: "Quota",
            meta: "stale · last good 47 min ago",
        },
        meter(
            "  Session · 5-hour    ",
            "  76%",
            26,
            8,
            DrawerTone::Warning,
            " ▲",
            false,
            "resets in 1 h …",
        ),
        meter(
            "  Weekly · all models ",
            "  88%",
            30,
            4,
            DrawerTone::Warning,
            " ▲",
            false,
            "resets Sun 19:…",
        ),
        meter(
            "  Weekly · Opus       ",
            " 100%",
            34,
            0,
            DrawerTone::Danger,
            " !",
            true,
            "resets Sun 19:…",
        ),
        meter(
            "  Extra usage credits ",
            "  28%",
            10,
            24,
            DrawerTone::Info,
            "  ",
            false,
            "1,420 / 5,000 …",
        ),
        DrawerLine::Blank,
        DrawerLine::Spans(&validation_line),
        DrawerLine::Text(
            "                                   readable                                  ",
        ),
        field(
            "Status",
            "Usage stale · last good 47 min ago · retry  in 13 min",
            DrawerInk::Warning,
        ),
        DrawerLine::Value {
            text: "· retryable",
            ink: DrawerInk::Warning,
        },
        DrawerLine::Blank,
        DrawerLine::Blank,
        DrawerLine::Blank,
        DrawerLine::Blank,
        DrawerLine::Blank,
        DrawerLine::Buttons(&[
            DrawerButton {
                label: "Refresh ",
                danger: false,
            },
            DrawerButton {
                label: "Validate ",
                danger: false,
            },
            DrawerButton {
                label: "Edit… ",
                danger: false,
            },
            DrawerButton {
                label: "Set default ",
                danger: false,
            },
            DrawerButton {
                label: "Disable ",
                danger: false,
            },
            DrawerButton {
                label: "Remove… ",
                danger: true,
            },
        ]),
        DrawerLine::Blank,
    ];
    let drawer_hints = [
        DetailHint {
            key: "↑↓",
            action: "Scroll",
        },
        DetailHint {
            key: "Tab",
            action: "Actions",
        },
        DetailHint {
            key: "Esc",
            action: "Back",
        },
    ];
    let detail_hints = [
        DetailHint {
            key: "Enter",
            action: "Details",
        },
        DetailHint {
            key: "r",
            action: "Refresh",
        },
        DetailHint {
            key: "e",
            action: "Edit…",
        },
        DetailHint {
            key: "Space",
            action: "Default",
        },
        DetailHint {
            key: "t",
            action: "Disable",
        },
        DetailHint {
            key: "v",
            action: "Validate",
        },
        DetailHint {
            key: "d",
            action: "Remove…",
        },
        DetailHint {
            key: "a",
            action: "Add…",
        },
        DetailHint {
            key: "/",
            action: "Filter",
        },
        DetailHint {
            key: "Esc",
            action: "Back",
        },
    ];
    termrock::feedback::paint_accounts_drawer(
        ui,
        ui.full(),
        &AccountsDrawer {
            brand: "jackin❯",
            menus: &["File", "Go", "Help"],
            crumb: "Accounts › Claude › Work",
            status: "⠋ refreshing 1",
            cluster: SettingsCluster {
                note: "inside the Construct",
                mark: "●",
                value: " 2 running",
            },
            title: " Accounts ",
            summary: " 12 · 4 ▲ · 4 ! ",
            rows: &rows,
            lines: &lines,
            hints: if chrome == PaneChrome::Detail {
                &detail_hints
            } else {
                &drawer_hints
            },
            hint_overflow: if chrome == PaneChrome::Detail {
                "…"
            } else {
                ""
            },
            chrome,
        },
    );
}

fn pane<'a>(
    role: AccountRole,
    mark: Option<&'a str>,
    name: &'a str,
    ink: DrawerInk,
    alert: Option<(&'a str, DetailAlert)>,
    selected: bool,
) -> PaneRow<'a> {
    let (alert, alert_tone) = alert.unwrap_or(("", DetailAlert::Danger));
    PaneRow {
        role,
        mark,
        name,
        ink,
        alert: if alert.is_empty() { None } else { Some(alert) },
        alert_tone,
        selected,
        empty: false,
    }
}

fn field<'a>(label: &'a str, value: &'a str, ink: DrawerInk) -> DrawerLine<'a> {
    DrawerLine::Field { label, value, ink }
}

#[allow(clippy::too_many_arguments)]
fn meter<'a>(
    label: &'a str,
    readout: &'a str,
    fill_cols: u16,
    track_cols: u16,
    tone: DrawerTone,
    mark: &'a str,
    mark_bold: bool,
    note: &'a str,
) -> DrawerLine<'a> {
    DrawerLine::Meter(DrawerMeter {
        label,
        readout,
        fill_cols,
        track_cols,
        tone,
        mark,
        mark_bold,
        note,
    })
}
