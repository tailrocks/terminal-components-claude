use termrock::Ui;
use termrock::feedback::{
    AccountRole, AccountRow, AccountsDetail, DetailAlert, DetailHint, DetailTail,
};

pub(super) fn paint(ui: &mut Ui<'_>) {
    let rows = [
        row(
            AccountRole::Root,
            None,
            "Overview",
            None,
            None,
            false,
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
            true,
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
    let facts = [
        "  Provider  Anthropic / Claude",
        "  Agent runtime  Claude Code",
        "  Usage surface  Claude",
    ];
    termrock::feedback::paint_accounts_detail(
        ui,
        ui.full(),
        &AccountsDetail {
            brand: "jackin❯",
            menus: &["File", "Go", "Help"],
            crumb: "Accounts › Claude › Work",
            status: "⠋ refreshing 1",
            title: " Accounts ",
            summary: " 12 · 4 ▲ · 4 ! · 1–11 of 23 ",
            rows: &rows,
            total: 23,
            offset: 0,
            heading: "Claude · Work",
            heading_meta: "account · exhausted · Tab details",
            facts: &facts,
            hints: &hints,
            hint_overflow: "…",
        },
    );
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
