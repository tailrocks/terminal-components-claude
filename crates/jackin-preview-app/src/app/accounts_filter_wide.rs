//! 120-column filtered accounts list.
//!
//! The list is the detail chrome with the filter title. The panel is the
//! overview inspector: health, comparable windows, and warnings from the
//! account registry.

use termrock::Ui;
use termrock::feedback::{
    AccountRole, AccountsDrawer, DetailAlert, DrawerButton, DrawerInk, DrawerLine, FilterHint,
    InspectLine, InspectSpan, InspectTone, PaneChrome, PaneRow, SettingsCluster,
};

use crate::domain::account::{Account, AccountOrigin, IssueCode};
use crate::domain::usage::{Freshness, HealthWord, OverallSummary, QuotaStatus};
use crate::sim::onepassword::OpSession;
use crate::sim::world::World;

pub(super) fn paint(ui: &mut Ui<'_>, world: &World) {
    let summary = OverallSummary::compute(&world.accounts.accounts);
    let scope = format!(
        "{} · {}",
        count(world.accounts.accounts.len(), "account", "accounts"),
        count(summary.counts.providers, "provider", "providers")
    );
    let rows = [
        pane(AccountRole::Root, None, "Overview", None, true),
        pane(
            AccountRole::Group,
            Some("▾"),
            "Claude",
            Some(("!", DetailAlert::Danger)),
            false,
        ),
        pane(
            AccountRole::Item,
            None,
            "Work",
            Some(("!", DetailAlert::Danger)),
            false,
        ),
    ];
    let buttons = [
        DrawerButton {
            label: "Refresh all ",
            danger: false,
        },
        DrawerButton {
            label: "Add account… ",
            danger: false,
        },
    ];
    let mut lines = vec![
        DrawerLine::Title {
            title: "Overview",
            meta: &scope,
        },
        DrawerLine::Rule,
    ];
    for _ in 0..32 {
        lines.push(DrawerLine::Blank);
    }
    lines.push(DrawerLine::Buttons(&buttons));
    lines.push(DrawerLine::Blank);
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
    termrock::feedback::paint_accounts_drawer(
        ui,
        ui.full(),
        &AccountsDrawer {
            brand: "jackin❯",
            menus: &["File", "Go", "Help"],
            crumb: "Accounts › Overview",
            status: "⠋ refreshing 1",
            cluster: SettingsCluster {
                note: "inside the Construct",
                mark: "●",
                value: " 2 running",
            },
            title: " Accounts · filt… ",
            summary: " 12 · 4 ▲ · 4 ! ",
            rows: &rows,
            lines: &lines,
            hints: &hints,
            hint_overflow: "",
            chrome: PaneChrome::Detail,
        },
    );
    termrock::feedback::paint_inspector_body(ui, ui.full(), &inspector_lines(world, &summary));
}

fn inspector_lines(world: &World, summary: &OverallSummary) -> Vec<InspectLine> {
    let mut lines = vec![
        InspectLine::Spans {
            label: "Health".into(),
            spans: health_spans(summary),
        },
        InspectLine::Prop {
            label: "Registry".into(),
            value: summary.counts_line(),
            tone: InspectTone::Plain,
        },
        InspectLine::Prop {
            label: "Sources".into(),
            value: sources_line(world),
            tone: InspectTone::Plain,
        },
        InspectLine::Blank,
        InspectLine::Heading {
            label: "Comparable windows".into(),
            meta: "identical provider, window and unit only".into(),
        },
    ];
    if summary.comparable.is_empty() {
        lines.push(InspectLine::Text {
            text: "Nothing comparable: no provider has two accounts with the same window".into(),
            tone: InspectTone::Muted,
        });
    }
    for window in &summary.comparable {
        lines.push(InspectLine::Text {
            text: comparable_line(window),
            tone: InspectTone::Secondary,
        });
    }
    if !summary.not_comparable.is_empty() {
        lines.push(InspectLine::Blank);
        lines.push(InspectLine::Heading {
            label: "Not comparable".into(),
            meta: String::new(),
        });
        let names: Vec<String> = summary
            .not_comparable
            .iter()
            .map(|note| format!("{} ({})", note.surface.label(), note.reason))
            .collect();
        lines.push(InspectLine::Prewrapped {
            text: names.join(" · "),
            tone: InspectTone::Muted,
        });
    }
    lines.push(InspectLine::Blank);
    lines.push(InspectLine::Heading {
        label: "Warnings".into(),
        meta: String::new(),
    });
    let mut any = false;
    for account in world.accounts.sorted() {
        if !account.enabled {
            continue;
        }
        for window in &account.usage.windows {
            match window.status {
                QuotaStatus::Warning => {
                    any = true;
                    lines.push(status(
                        false,
                        account.title(),
                        format!("{} {}% used", window.label, window.used_pct.unwrap_or(0)),
                        InspectTone::Warning,
                        window
                            .reset_secs
                            .map(|secs| format!(" · {}", world.clock.reset_label(secs)))
                            .unwrap_or_default(),
                    ));
                }
                QuotaStatus::Exhausted => {
                    any = true;
                    lines.push(status(
                        true,
                        account.title(),
                        format!("{} exhausted", window.label),
                        InspectTone::Danger,
                        window
                            .reset_secs
                            .map(|secs| format!(" · {}", world.clock.reset_label(secs)))
                            .unwrap_or_default(),
                    ));
                }
                _ => {}
            }
        }
        if let Some(issue) = &account.issue
            && !issue.is_informational()
            && issue.code != IssueCode::QuotaUnsupported
        {
            any = true;
            lines.push(status(
                true,
                account.title(),
                issue.message.clone(),
                InspectTone::Danger,
                String::new(),
            ));
        } else if account.usage.freshness.phase == Freshness::Stale {
            any = true;
            lines.push(status(
                false,
                account.title(),
                "stale".into(),
                InspectTone::Warning,
                format!(
                    " · last good {}",
                    account
                        .usage
                        .freshness
                        .last_good_secs
                        .map(|secs| world.clock.ago(secs))
                        .unwrap_or_else(|| "?".into())
                ),
            ));
        }
    }
    if !any {
        lines.push(InspectLine::Text {
            text: "No warnings".into(),
            tone: InspectTone::Muted,
        });
    }
    let unresolved: Vec<&Account> = world
        .accounts
        .accounts
        .iter()
        .filter(|account| account.enabled && account.identity.subject.is_none())
        .collect();
    if !unresolved.is_empty() {
        lines.push(InspectLine::Blank);
        lines.push(InspectLine::Heading {
            label: "Unresolved identity".into(),
            meta: String::new(),
        });
        for account in unresolved {
            lines.push(InspectLine::Text {
                text: format!(
                    "{} · {} · {} — not an authenticated account",
                    account.title(),
                    account.source.origin_label().to_lowercase(),
                    account.confidence.label()
                ),
                tone: InspectTone::Muted,
            });
        }
    }
    lines
}

fn health_spans(summary: &OverallSummary) -> Vec<InspectSpan> {
    let tone = match summary.health {
        HealthWord::Blocked | HealthWord::Degraded => InspectTone::DangerBold,
        HealthWord::Attention => InspectTone::Warning,
        _ => InspectTone::Plain,
    };
    let mut spans = vec![InspectSpan {
        text: summary.health.label().into(),
        tone,
    }];
    let counts = &summary.counts;
    if counts.warnings > 0 {
        push_part(
            &mut spans,
            count(counts.warnings, "warning", "warnings"),
            InspectTone::Warning,
        );
    }
    if counts.exhausted > 0 {
        push_part(
            &mut spans,
            format!("{} exhausted", counts.exhausted),
            InspectTone::Danger,
        );
    }
    if counts.stale > 0 {
        push_part(
            &mut spans,
            format!("{} stale", counts.stale),
            InspectTone::Warning,
        );
    }
    if counts.refreshing > 0 {
        push_part(
            &mut spans,
            format!("{} refreshing", counts.refreshing),
            InspectTone::Info,
        );
    }
    if counts.failed > 0 {
        push_part(
            &mut spans,
            format!("{} failed", counts.failed),
            InspectTone::Danger,
        );
    }
    if counts.unsupported > 0 {
        push_part(
            &mut spans,
            format!("{} quota not visible", counts.unsupported),
            InspectTone::Plain,
        );
    }
    if counts.unresolved_identity > 0 {
        let word = if counts.unresolved_identity == 1 {
            "y"
        } else {
            "ies"
        };
        push_part(
            &mut spans,
            format!("{} identit{word} unresolved", counts.unresolved_identity),
            InspectTone::Plain,
        );
    }
    spans
}

fn push_part(spans: &mut Vec<InspectSpan>, text: String, tone: InspectTone) {
    spans.push(InspectSpan {
        text: " · ".into(),
        tone: InspectTone::Muted,
    });
    spans.push(InspectSpan { text, tone });
}

fn sources_line(world: &World) -> String {
    let registered = world
        .accounts
        .accounts
        .iter()
        .filter(|account| account.origin == AccountOrigin::Registered)
        .count();
    let discovered = world.accounts.accounts.len() - registered;
    let session = match world.op.session {
        OpSession::SignedIn => "available",
        OpSession::Locked => "locked",
    };
    format!("{registered} registered · {discovered} discovered · 1Password {session}")
}

fn comparable_line(window: &crate::domain::usage::ComparableRollup) -> String {
    let mut text = format!(
        "{} · {}   {} · {}\u{2013}{}% remaining",
        window.surface.label(),
        window.label,
        count(window.accounts, "account", "accounts"),
        window.min_remaining_pct,
        window.max_remaining_pct
    );
    if let Some((used, limit)) = window.summed {
        text.push_str(&format!(
            " · {} / {} {}",
            thousands(used as usize),
            thousands(limit as usize),
            window.unit.label()
        ));
    }
    if window.last_good_count > 0 {
        text.push_str(&format!(" ({} last good)", window.last_good_count));
    }
    text
}

fn status(
    fail: bool,
    title: String,
    message: String,
    tone: InspectTone,
    suffix: String,
) -> InspectLine {
    InspectLine::Status {
        fail,
        spans: vec![
            InspectSpan {
                text: format!("{title}   "),
                tone: InspectTone::Plain,
            },
            InspectSpan {
                text: message,
                tone,
            },
            InspectSpan {
                text: suffix,
                tone: InspectTone::Muted,
            },
        ],
    }
}

fn count(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

fn thousands(value: usize) -> String {
    let digits = value.to_string();
    let mut out = String::new();
    for (index, ch) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

fn pane<'a>(
    role: AccountRole,
    mark: Option<&'a str>,
    name: &'a str,
    alert: Option<(&'a str, DetailAlert)>,
    selected: bool,
) -> PaneRow<'a> {
    let (alert, alert_tone) = alert.unwrap_or(("", DetailAlert::Danger));
    PaneRow {
        role,
        mark,
        name,
        ink: DrawerInk::Plain,
        alert: if alert.is_empty() { None } else { Some(alert) },
        alert_tone,
        selected,
        empty: false,
    }
}
