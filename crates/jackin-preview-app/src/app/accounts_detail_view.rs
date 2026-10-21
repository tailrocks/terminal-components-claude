use termrock::Ui;
use termrock::feedback::{
    AccountRole, AccountRow, AccountsDetail, DetailAlert, DetailHint, DetailInk, DetailLine,
    DetailSpan, DetailTail,
};

pub(super) fn paint(ui: &mut Ui<'_>) {
    paint_page(ui, false);
}

pub(super) fn paint_audit(ui: &mut Ui<'_>) {
    paint_page(ui, true);
}

fn paint_page(ui: &mut Ui<'_>, overview: bool) {
    let rows = [
        row(
            AccountRole::Root,
            None,
            "Overview",
            None,
            None,
            overview,
            false,
        ),
        row(
            AccountRole::Group,
            Some("▾"),
            "Claude",
            Some(("!", DetailAlert::Danger)),
            Some(("3", DetailTail::Muted)),
            false,
            false,
        ),
        row(
            AccountRole::Item,
            Some("★"),
            "Personal",
            None,
            Some(("current 4 min", DetailTail::Muted)),
            false,
            false,
        ),
        row(
            AccountRole::Note,
            None,
            "Archived contractor laptop profile — do not use for pro…",
            None,
            Some(("disabled", DetailTail::Muted)),
            false,
            false,
        ),
        row(
            AccountRole::Item,
            None,
            "Work",
            Some(("!", DetailAlert::Danger)),
            Some(("stale 47 min", DetailTail::Warning)),
            !overview,
            false,
        ),
        row(
            AccountRole::Group,
            Some("▾"),
            "Codex",
            None,
            Some(("2", DetailTail::Muted)),
            false,
            false,
        ),
        row(
            AccountRole::Item,
            Some("★"),
            "Primary",
            None,
            Some(("current 2 min", DetailTail::Muted)),
            false,
            false,
        ),
        row(
            AccountRole::Item,
            None,
            "Experiments",
            None,
            Some(("⠋ refreshing", DetailTail::Secondary)),
            false,
            false,
        ),
        row(
            AccountRole::Group,
            Some("▾"),
            "Amp",
            Some(("▲", DetailAlert::Warning)),
            Some(("1", DetailTail::Muted)),
            false,
            false,
        ),
        row(
            AccountRole::Item,
            None,
            "discovered",
            Some(("▲", DetailAlert::Warning)),
            Some(("current 1 min", DetailTail::Muted)),
            false,
            false,
        ),
        row(
            AccountRole::Group,
            Some("▾"),
            "Grok",
            None,
            Some(("1", DetailTail::Muted)),
            false,
            true,
        ),
    ];
    let hints = [
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
            key: "Esc",
            action: "Back",
        },
    ];
    let provider = [span("  Provider  Anthropic / Claude", DetailInk::Plain)];
    let runtime = [span("  Agent runtime  Claude Code", DetailInk::Plain)];
    let surface = [span("  Usage surface  Claude", DetailInk::Plain)];
    let health = [
        span("  Health  ", DetailInk::Plain),
        span("degraded", DetailInk::DangerBold),
        span(" · ", DetailInk::Muted),
        span("4 warnings", DetailInk::Warning),
        span(" · ", DetailInk::Muted),
        span("1 exhausted", DetailInk::Danger),
        span(" · ", DetailInk::Muted),
        span("2 stale", DetailInk::Warning),
        span(" · ", DetailInk::Muted),
        span("1 refreshing", DetailInk::Info),
        span(" · ", DetailInk::Muted),
        span("3 …", DetailInk::Danger),
    ];
    let registry = [span(
        "  Registry  12 accounts · 11 enabled · 1 disabled · 8 providers",
        DetailInk::Plain,
    )];
    let sources = [span(
        "  Sources  7 registered · 5 discovered · 1Password available",
        DetailInk::Plain,
    )];
    let work_facts = [
        DetailLine { spans: &provider },
        DetailLine { spans: &runtime },
        DetailLine { spans: &surface },
    ];
    let overview_facts = [
        DetailLine { spans: &health },
        DetailLine { spans: &registry },
        DetailLine { spans: &sources },
    ];
    let overview_hints = [
        DetailHint {
            key: "Enter",
            action: "Details",
        },
        DetailHint {
            key: "r",
            action: "Refresh all",
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
            key: "m",
            action: "Usage",
        },
        DetailHint {
            key: "Esc",
            action: "Back",
        },
    ];
    termrock::feedback::paint_accounts_detail(
        ui,
        ui.full(),
        &AccountsDetail {
            brand: "jackin❯",
            menus: &["File", "Go", "Help"],
            crumb: if overview {
                "Accounts › Overview"
            } else {
                "Accounts › Claude › Work"
            },
            status: "⠋ refreshing 1",
            title: " Accounts ",
            summary: " 12 · 4 ▲ · 4 ! · 1–11 of 23 ",
            rows: &rows,
            total: 23,
            offset: 0,
            heading: if overview {
                "Overview"
            } else {
                "Claude · Work"
            },
            heading_meta: if overview {
                "12 accounts · 8 providers · Tab details"
            } else {
                "account · exhausted · Tab details"
            },
            facts: if overview {
                &overview_facts
            } else {
                &work_facts
            },
            hints: if overview { &overview_hints } else { &hints },
            hint_overflow: if overview { "" } else { "…" },
        },
    );
}

fn span(text: &str, ink: DetailInk) -> DetailSpan<'_> {
    DetailSpan { text, ink }
}

fn row<'a>(
    role: AccountRole,
    mark: Option<&'a str>,
    name: &'a str,
    alert: Option<(&'a str, DetailAlert)>,
    tail: Option<(&'a str, DetailTail)>,
    selected: bool,
    faded: bool,
) -> AccountRow<'a> {
    let (alert, alert_tone) = alert.unwrap_or(("", DetailAlert::Danger));
    let (tail, tail_tone) = tail.unwrap_or(("", DetailTail::Muted));
    AccountRow {
        role,
        mark,
        name,
        alert: if alert.is_empty() { None } else { Some(alert) },
        alert_tone,
        tail: if tail.is_empty() { None } else { Some(tail) },
        tail_tone,
        selected,
        faded,
    }
}
