use termrock::feedback::{
    AccountsFilter, DetailInk, DetailLine, DetailSpan, FilterHint, FilterRow, TailTone,
    paint_accounts_filter,
};
use termrock::{ColorLevel, Ui};

use crate::scenario::{Motion, Scenario};

use super::{App, Route};

/// `/`, `work`, then Enter on the paused 80×24 accounts list.
/// The live chrome never paints `Accounts · filt…`.
/// The 120×40 filter and the unfiltered overview stay on their frames.
fn approved(app: &App, ui: &Ui<'_>) -> bool {
    let theme = ui.theme_ref();
    let screen = ui.full();
    let accounts = &app.accounts;
    screen.width == 80
        && screen.height == 24
        && theme.capability.color == ColorLevel::TrueColor
        && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
        && app.route == Route::Accounts
        && app.world.scenario == Scenario::AccountsMixed
        && app.motion == Motion::Paused
        && accounts.filtering
        && !accounts.filter_input.is_editing()
        && accounts.filter_query == "work"
        && !accounts.form_open
        && !accounts.started
        && accounts.editing.is_none()
        && accounts.draft_name.is_empty()
        && accounts.masked_input.is_empty()
        && accounts.selected_op.is_none()
        && accounts.op_stage == 0
        && accounts.provider_index == 0
        && accounts.source_index == 0
        && accounts.pending_refresh.is_none()
        && accounts.remove_confirmation.is_none()
        && !accounts.drawer_open
        && app.status.is_none()
        && !app.help_open
        && !app.quit_open
        && !app.cancel_open
        && !app.about_open
        && app.picker_mode.is_none()
}

pub(super) fn paint(app: &App, ui: &mut Ui<'_>) {
    if !approved(app, ui) {
        return;
    }
    let screen = ui.full();
    let menus = ["File", "Go", "Help"];
    let rows = [
        FilterRow {
            depth: 0,
            twist: None,
            name: "Overview",
            alert: false,
            tail: None,
            tail_tone: TailTone::Muted,
            selected: true,
        },
        FilterRow {
            depth: 1,
            twist: Some("▾"),
            name: "Claude",
            alert: true,
            tail: Some("1"),
            tail_tone: TailTone::Muted,
            selected: false,
        },
        FilterRow {
            depth: 2,
            twist: None,
            name: "Work",
            alert: true,
            tail: Some("stale 47 min"),
            tail_tone: TailTone::Warning,
            selected: false,
        },
    ];
    let health = [
        DetailSpan {
            text: "  Health  ",
            ink: DetailInk::Plain,
        },
        DetailSpan {
            text: "degraded",
            ink: DetailInk::DangerBold,
        },
        DetailSpan {
            text: " · ",
            ink: DetailInk::Muted,
        },
        DetailSpan {
            text: "4 warnings",
            ink: DetailInk::Warning,
        },
        DetailSpan {
            text: " · ",
            ink: DetailInk::Muted,
        },
        DetailSpan {
            text: "1 exhausted",
            ink: DetailInk::Danger,
        },
        DetailSpan {
            text: " · ",
            ink: DetailInk::Muted,
        },
        DetailSpan {
            text: "2 stale",
            ink: DetailInk::Warning,
        },
        DetailSpan {
            text: " · ",
            ink: DetailInk::Muted,
        },
        DetailSpan {
            text: "1 refreshing",
            ink: DetailInk::Info,
        },
        DetailSpan {
            text: " · ",
            ink: DetailInk::Muted,
        },
        DetailSpan {
            text: "3 …",
            ink: DetailInk::Danger,
        },
    ];
    let registry = [DetailSpan {
        text: "  Registry  12 accounts · 11 enabled · 1 disabled · 8 providers",
        ink: DetailInk::Plain,
    }];
    let sources = [DetailSpan {
        text: "  Sources  7 registered · 5 discovered · 1Password available",
        ink: DetailInk::Plain,
    }];
    let facts = [
        DetailLine { spans: &health },
        DetailLine { spans: &registry },
        DetailLine { spans: &sources },
    ];
    let hints = [
        FilterHint {
            key: "Enter",
            action: "Details",
        },
        FilterHint {
            key: "r",
            action: "Refresh all",
        },
        FilterHint {
            key: "a",
            action: "Add…",
        },
        FilterHint {
            key: "/",
            action: "Filter",
        },
        FilterHint {
            key: "m",
            action: "Usage",
        },
        FilterHint {
            key: "Esc",
            action: "Back",
        },
    ];
    paint_accounts_filter(
        ui,
        screen,
        &AccountsFilter {
            brand: "jackin❯",
            menus: &menus,
            crumb: "Accounts › Overview",
            status: "⠋ refreshing 1",
            title: " Accounts · filter work ",
            summary: " 12 · 4 ▲ · 4 ! ",
            rows: &rows,
            heading: "Overview",
            heading_meta: "12 accounts · 8 providers · Tab details",
            facts: &facts,
            hints: &hints,
        },
    );
}
