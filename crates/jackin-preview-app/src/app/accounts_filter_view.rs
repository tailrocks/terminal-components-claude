use termrock::Ui;
use termrock::feedback::{
    AccountsFilter, DetailInk, DetailLine, DetailSpan, FilterHint, FilterRow, TailTone,
    paint_accounts_filter,
};

pub(super) fn paint(ui: &mut Ui<'_>) {
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
