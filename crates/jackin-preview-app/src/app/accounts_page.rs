//! Accounts page: the account tree, the registry inspector, and the drawer.
//!
//! Every value on this page comes from the live registry. The tree rows are
//! the accounts of the world, the inspector is the registry summary, and the
//! drawer shows the selected node. The page picks its layout from the area it
//! is given: a wide area gets the list beside the inspector, a narrow area
//! gets the list above the inspector band. Nothing here keys on a screen size.

use termrock::feedback::{
    AccountRole, AccountRow, AccountsDetail, AccountsDrawer, DetailAlert, DetailInk, DetailLine,
    DetailSpan, DetailTail, DrawerButton, DrawerInk, DrawerLine, FilterHint, InspectLine,
    InspectSpan, InspectTone, PaneChrome, PaneRow, SettingsCluster,
};
use termrock::{FrameRead, Rect, Ui};

use crate::domain::account::{Account, AccountOrigin, CredentialSource, IssueCode, Lifecycle};
use crate::domain::agent::Provider;
use crate::domain::usage::{Freshness, HealthWord, OverallSummary, QuotaStatus};
use crate::sim::onepassword::OpSession;

use super::App;

/// The layout switches to the inspector beside the list when the area is wide
/// enough for both columns and tall enough for the shared painters.
const WIDE_MIN_WIDTH: u16 = 100;
const WIDE_MIN_HEIGHT: u16 = 24;

/// The width of the account list pane of the shared accounts painters.
///
/// The shared painters own this geometry; the page needs it to cut a row
/// name before the pane edge.
const LIST_PANE_W: u16 = 39;

/// The registration policy of the accounts page.
const REGISTRATION_POLICY: &str =
    "manual accounts allowed · 1Password, local folder or plain-text key";

const OVERVIEW_HINTS: [FilterHint<'static>; 6] = [
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

const PROVIDER_HINTS: [FilterHint<'static>; 6] = [
    FilterHint {
        key: "Space",
        action: "Fold",
    },
    FilterHint {
        key: "r",
        action: "Refresh provider",
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

const ACCOUNT_HINTS: [FilterHint<'static>; 11] = [
    FilterHint {
        key: "Enter",
        action: "Details",
    },
    FilterHint {
        key: "r",
        action: "Refresh",
    },
    FilterHint {
        key: "e",
        action: "Edit…",
    },
    FilterHint {
        key: "Space",
        action: "Default",
    },
    FilterHint {
        key: "t",
        action: "Disable",
    },
    FilterHint {
        key: "v",
        action: "Validate",
    },
    FilterHint {
        key: "d",
        action: "Remove…",
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

const DRAWER_HINTS: [FilterHint<'static>; 3] = [
    FilterHint {
        key: "↑↓",
        action: "Scroll",
    },
    FilterHint {
        key: "Tab",
        action: "Actions",
    },
    FilterHint {
        key: "Esc",
        action: "Back",
    },
];

/// What a tree row stands for.
#[derive(Clone, PartialEq, Eq)]
pub(super) enum Node {
    /// The registry overview.
    Overview,
    /// A provider group.
    Provider(Provider),
    /// One account, by registry id.
    Account(String),
    /// The section of accounts whose quota is not visible.
    Unsupported,
    /// The add-account action row.
    AddAccount,
}

/// One row of the account tree.
pub(super) struct TreeRow {
    pub(super) node: Node,
    pub(super) role: AccountRole,
    pub(super) mark: Option<String>,
    pub(super) name: String,
    pub(super) ink: DrawerInk,
    pub(super) alert: Option<(&'static str, DetailAlert)>,
    pub(super) tail: Option<String>,
    pub(super) tail_tone: DetailTail,
}

/// The account tree of the live registry.
pub(super) struct Tree {
    pub(super) rows: Vec<TreeRow>,
}

impl Tree {
    /// The account id of the row at `index`, if that row is an account.
    pub(super) fn account_at(&self, index: usize) -> Option<&str> {
        match self.rows.get(index)?.node {
            Node::Account(ref id) => Some(id.as_str()),
            _ => None,
        }
    }
}

/// The account tree for the app: the registry, filtered by the live query.
pub(super) fn tree(app: &App) -> Tree {
    let mut rows = vec![TreeRow {
        node: Node::Overview,
        role: AccountRole::Root,
        mark: None,
        name: "Overview".to_owned(),
        ink: DrawerInk::Plain,
        alert: None,
        tail: None,
        tail_tone: DetailTail::Muted,
    }];
    let mut groups: Vec<(Provider, Vec<&Account>)> = Vec::new();
    for account in app.world.accounts.sorted() {
        if !app.account_matches_filter(account) {
            continue;
        }
        match groups.last_mut() {
            Some((provider, members)) if *provider == account.provider => {
                members.push(account);
            }
            _ => groups.push((account.provider, vec![&account])),
        }
    }
    for (provider, members) in &groups {
        rows.push(group_row(*provider, members));
        for account in members {
            rows.push(account_row(app, account));
        }
    }
    if app
        .world
        .accounts
        .accounts
        .iter()
        .any(|account| account.enabled && quota_is_hidden(account))
    {
        rows.push(TreeRow {
            node: Node::Unsupported,
            role: AccountRole::Root,
            mark: None,
            name: "Unsupported".to_owned(),
            ink: DrawerInk::Muted,
            alert: None,
            tail: None,
            tail_tone: DetailTail::Muted,
        });
    }
    rows.push(TreeRow {
        node: Node::AddAccount,
        role: AccountRole::Root,
        mark: None,
        name: "+ Add account…".to_owned(),
        ink: DrawerInk::Secondary,
        alert: None,
        tail: None,
        tail_tone: DetailTail::Muted,
    });
    Tree { rows }
}

fn group_row(provider: Provider, members: &[&Account]) -> TreeRow {
    let mut alert: Option<(&'static str, DetailAlert)> = None;
    for account in members {
        let Some(candidate) = account_alert(account) else {
            continue;
        };
        alert = match (alert, candidate) {
            (None, candidate) => Some(candidate),
            (Some((_, DetailAlert::Warning)), (_, DetailAlert::Danger)) => Some(candidate),
            (current, _) => current,
        };
    }
    TreeRow {
        node: Node::Provider(provider),
        role: AccountRole::Group,
        mark: Some("▾".to_owned()),
        name: provider.usage_surface().surface_name().to_owned(),
        ink: DrawerInk::Plain,
        alert,
        tail: Some(members.len().to_string()),
        tail_tone: DetailTail::Muted,
    }
}

fn account_row(app: &App, account: &Account) -> TreeRow {
    let role = if account.enabled {
        AccountRole::Item
    } else {
        AccountRole::Note
    };
    let tail = account_tail(app, account);
    TreeRow {
        node: Node::Account(account.id.clone()),
        role,
        mark: account.default_for_provider.then(|| "★".to_owned()),
        name: account.display_name.clone(),
        ink: if account.enabled {
            DrawerInk::Plain
        } else {
            DrawerInk::Muted
        },
        alert: account_alert(account),
        tail: tail.as_ref().map(|text| text.0.clone()),
        tail_tone: tail.as_ref().map_or(DetailTail::Muted, |text| text.1),
    }
}

/// The alert mark of an account: `!` for a blocked account, `▲` for a warning.
fn account_alert(account: &Account) -> Option<(&'static str, DetailAlert)> {
    if !account.enabled {
        return None;
    }
    if account.status_word() == "exhausted" || account.is_error_state() {
        return Some(("!", DetailAlert::Danger));
    }
    if account.status_word() == "warning" {
        return Some(("▲", DetailAlert::Warning));
    }
    None
}

/// The state tail of an account: freshness, lifecycle, or issue.
fn account_tail(app: &App, account: &Account) -> Option<(String, DetailTail)> {
    if !account.enabled {
        return Some(("disabled".to_owned(), DetailTail::Muted));
    }
    match account.usage.freshness.phase {
        Freshness::Refreshing => Some(("⠋ refreshing".to_owned(), DetailTail::Secondary)),
        Freshness::Stale => Some((
            format!("stale {}", age_label(app, account)),
            DetailTail::Warning,
        )),
        Freshness::Failed => Some((
            issue_tail(account).unwrap_or_else(|| "failed".to_owned()),
            DetailTail::Muted,
        )),
        Freshness::Current => {
            if account.is_error_state() {
                return Some((
                    issue_tail(account).unwrap_or_else(|| account.status_word().to_owned()),
                    DetailTail::Muted,
                ));
            }
            Some((
                format!("current {}", age_label(app, account)),
                DetailTail::Muted,
            ))
        }
    }
}

/// The short issue label of an account, when it has one.
fn issue_tail(account: &Account) -> Option<String> {
    let issue = account.issue.as_ref()?;
    if issue.is_informational() || issue.code == IssueCode::QuotaUnsupported {
        return None;
    }
    Some(match issue.code {
        IssueCode::RateLimited => "rate limited".to_owned(),
        IssueCode::ProviderUnavailable => "unavailable".to_owned(),
        IssueCode::Stale => "stale".to_owned(),
        IssueCode::IdentityUnresolved => "identity unresolved".to_owned(),
        _ => issue
            .message
            .split(':')
            .next()
            .unwrap_or("issue")
            .to_lowercase(),
    })
}

/// The age of the last good usage sample, without the "ago" suffix.
fn age_label(app: &App, account: &Account) -> String {
    let Some(last_good) = account.usage.freshness.last_good_secs else {
        return "unknown".to_owned();
    };
    let ago = app.world.clock.ago(last_good);
    ago.trim_end_matches(" ago").to_owned()
}

/// True when the account cannot show a quota window.
fn quota_is_hidden(account: &Account) -> bool {
    account.lifecycle == Lifecycle::Unsupported
        || (!account.usage.windows.is_empty()
            && account
                .usage
                .windows
                .iter()
                .all(|window| window.status == QuotaStatus::Unsupported))
}

/// Paint the accounts page into `area`.
pub(super) fn paint(app: &App, ui: &mut Ui<'_>, area: Rect) {
    let tree = tree(app);
    if area.width >= WIDE_MIN_WIDTH && area.height >= WIDE_MIN_HEIGHT {
        paint_wide(app, ui, area, &tree);
    } else {
        paint_narrow(app, ui, area, &tree);
    }
    paint_filter_field(app, ui, area);
}

/// The filter field, while it holds the caret.
///
/// The field lives on the blank row under the header, so the committed frame
/// keeps that row empty and the caret is the only sign of the filter.
fn paint_filter_field(app: &App, ui: &mut Ui<'_>, area: Rect) {
    if !app.accounts.filtering {
        return;
    }
    if !ui
        .state(crate::screens::accounts::FILTER)
        .contains(termrock::StateFlags::FOCUSED)
    {
        return;
    }
    let field = Rect::new(
        area.x.saturating_add(1),
        area.y.saturating_add(1),
        area.width.saturating_sub(2),
        1,
    );
    App::account_filter_input()
        .value(&app.accounts.filter_query)
        .draw(ui, field, &app.accounts.filter_input);
}

/// The list beside the inspector.
fn paint_wide(app: &App, ui: &mut Ui<'_>, area: Rect, tree: &Tree) {
    let selection = selection(app, tree);
    draw_list_owner(app, ui, list_area(area, true));
    let names = pane_names(tree);
    let rows = pane_rows(app, tree, &names);
    let summary = list_summary(app);
    let title = list_title(app, &selection, &summary);
    let (crumb, status, running, note) = header_parts(app, area, &selection, true);
    let panel = panel_for(app, &selection);
    let buttons: Vec<DrawerButton<'_>> = panel
        .buttons
        .iter()
        .map(|(label, danger)| DrawerButton {
            label,
            danger: *danger,
        })
        .collect();
    let mut lines: Vec<DrawerLine<'_>> = vec![
        DrawerLine::Title {
            title: &panel.title,
            meta: &panel.meta,
        },
        DrawerLine::Blank,
    ];
    for _ in 0..area.height.saturating_sub(8) {
        lines.push(DrawerLine::Blank);
    }
    lines.push(DrawerLine::Buttons(&buttons));
    lines.push(DrawerLine::Blank);
    let hints: &[FilterHint<'_>] = match &selection {
        Selection::Account(_) if app.accounts.drawer_open => &DRAWER_HINTS,
        Selection::Account(_) => &ACCOUNT_HINTS,
        Selection::Provider(_) => &PROVIDER_HINTS,
        Selection::Overview => &OVERVIEW_HINTS,
    };
    let overflow = if app.accounts.drawer_open { "" } else { "…" };
    let page = AccountsDrawer {
        brand: "jackin❯",
        menus: &["File", "Go", "Help"],
        crumb: &crumb,
        status: &status,
        cluster: SettingsCluster {
            note: &note,
            mark: "",
            value: &running,
        },
        title: &title,
        summary: &summary,
        rows: &rows,
        lines: &lines,
        hints,
        hint_overflow: overflow,
        chrome: if app.accounts.drawer_open {
            PaneChrome::Drawer
        } else {
            PaneChrome::Detail
        },
    };
    termrock::feedback::paint_accounts_drawer(ui, area, &page);
    termrock::feedback::paint_inspector_body(ui, area, &panel.lines);
    register_add_row(ui, area, tree);
}

/// The list above the inspector band.
fn paint_narrow(app: &App, ui: &mut Ui<'_>, area: Rect, tree: &Tree) {
    let selection = selection(app, tree);
    draw_list_owner(app, ui, list_area(area, false));
    let names = detail_names(area.width, tree);
    let rows = detail_rows(app, tree, &names);
    let visible = visible_rows(area, rows.len());
    let total = u16::try_from(tree.rows.len()).unwrap_or(u16::MAX);
    let offset = list_offset(app, tree, visible);
    let facts = narrow_facts(app, &selection, area.width);
    let spans: Vec<Vec<DetailSpan<'_>>> = facts
        .iter()
        .map(|fact| {
            fact.iter()
                .map(|(text, ink)| DetailSpan { text, ink: *ink })
                .collect()
        })
        .collect();
    let fact_lines: Vec<DetailLine<'_>> = spans.iter().map(|spans| DetailLine { spans }).collect();
    let (heading, heading_meta) = narrow_heading(app, &selection);
    let (crumb, status, _running, _note) = header_parts(app, area, &selection, false);
    let summary = format!(
        " {} · {} ▲ · {} ! · {}\u{2013}{} of {} ",
        app.world.accounts.accounts.len(),
        warning_count(app),
        danger_count(app),
        offset.saturating_add(1),
        offset.saturating_add(u16::try_from(visible).unwrap_or(0)),
        total
    );
    let title = list_title(app, &selection, &summary);
    let hints: &[FilterHint<'_>] = match &selection {
        Selection::Account(_) => &ACCOUNT_HINTS,
        Selection::Provider(_) => &PROVIDER_HINTS,
        Selection::Overview => &OVERVIEW_HINTS,
    };
    let page = AccountsDetail {
        brand: "jackin❯",
        menus: &["File", "Go", "Help"],
        crumb: &crumb,
        status: &status,
        title: &title,
        summary: &summary,
        rows: &rows,
        total,
        offset,
        heading: &heading,
        heading_meta: &heading_meta,
        facts: &fact_lines,
        hints,
        hint_overflow: "…",
    };
    termrock::feedback::paint_accounts_detail(ui, area, &page);
}

/// Register the add-account row of the tree as the add control.
fn register_add_row(ui: &mut Ui<'_>, area: Rect, tree: &Tree) {
    let Some(index) = tree
        .rows
        .iter()
        .position(|row| row.node == Node::AddAccount)
    else {
        return;
    };
    let Some(row) = tree.rows.get(index) else {
        return;
    };
    let name_x = match row.role {
        AccountRole::Root | AccountRole::Group => 7,
        AccountRole::Item | AccountRole::Note => 9,
    };
    let top = area.y.saturating_add(2).saturating_add(1);
    let y = top.saturating_add(u16::try_from(index).unwrap_or(u16::MAX));
    if y >= area.bottom() {
        return;
    }
    ui.register_control(
        crate::ACCOUNT_ADD,
        Rect::new(area.x.saturating_add(1).saturating_add(name_x), y, 20, 1),
        termrock::Focusability::Focusable,
    );
}

/// The header of the page: the crumb, the status, and the cluster.
///
/// The crumb carries the app message, so a saved account or a help note stays
/// visible. The status and the cluster follow it when they still fit; a crumb
/// that does not fit the header at all drops the whole right side.
fn header_parts(
    app: &App,
    area: Rect,
    selection: &Selection,
    cluster: bool,
) -> (String, String, String, String) {
    let tree_crumb = crumb(app, selection);
    let crumb = app.status.clone().unwrap_or(tree_crumb);
    let status = header_status(app);
    let note = "inside the Construct".to_owned();
    let value = format!("{} running", app.world.running_count());
    // The brand and the menus take the left of the header.
    let left = 30;
    let crumb_w = text_width(&crumb);
    if left + crumb_w + 2 > usize::from(area.width) {
        return (String::new(), String::new(), String::new(), String::new());
    }
    let mut rest = text_width(&status) + 2;
    if cluster {
        rest += text_width(&note) + text_width(&value) + 4;
    }
    if left + crumb_w + rest > usize::from(area.width) {
        return (crumb, String::new(), String::new(), String::new());
    }
    (crumb, status, value, note)
}

/// The column width of a text.
fn text_width(text: &str) -> usize {
    text.chars().count()
}

/// The area of the list pane.
fn list_area(area: Rect, wide: bool) -> Rect {
    let width = if wide {
        LIST_PANE_W
    } else {
        area.width.saturating_sub(2)
    };
    Rect::new(
        area.x.saturating_add(1),
        area.y.saturating_add(2),
        width,
        area.height.saturating_sub(5),
    )
}

/// Give the list the keys and the hit region.
///
/// The shared painter draws the rows, so the list only has to own the
/// focus: its bindings and its control are published here and the pane is
/// painted over by the shared component.
fn draw_list_owner(app: &App, ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    let rows = app.account_rows();
    termrock::List::new(crate::ACCOUNTS_LIST).draw(ui, area, &app.accounts.list, &rows);
}

/// The inspector content of the panel next to the list.
struct Panel {
    /// The title of the panel.
    title: String,
    /// The meta of the title.
    meta: String,
    /// The inspector lines.
    lines: Vec<InspectLine>,
    /// The buttons at the foot of the panel.
    buttons: Vec<(String, bool)>,
}

/// The panel of the selected node.
fn panel_for(app: &App, selection: &Selection) -> Panel {
    match selection {
        Selection::Overview => {
            let summary = OverallSummary::compute(&app.world.accounts.accounts);
            let scope = format!(
                "{} · {}",
                count(app.world.accounts.accounts.len(), "account", "accounts"),
                count(summary.counts.providers, "provider", "providers")
            );
            Panel {
                title: "Overview".to_owned(),
                meta: scope,
                lines: registry_lines(app, &summary),
                buttons: vec![
                    ("Refresh all ".to_owned(), false),
                    ("Add account… ".to_owned(), false),
                ],
            }
        }
        Selection::Provider(provider) => {
            let members = provider_accounts(app, *provider);
            Panel {
                title: provider.label().to_owned(),
                meta: format!("provider · {}", count(members.len(), "account", "accounts")),
                lines: provider_lines(app, *provider, &members),
                buttons: vec![
                    ("Refresh provider ".to_owned(), false),
                    ("Add account… ".to_owned(), false),
                ],
            }
        }
        Selection::Account(id) => {
            let Some(account) = app.world.accounts.get(id) else {
                return panel_for(app, &Selection::Overview);
            };
            Panel {
                title: truncate(
                    &format!(
                        "{} · {}",
                        account.surface.surface_name(),
                        account.display_name
                    ),
                    72,
                ),
                meta: format!("account · {}", account.status_word()),
                lines: account_lines(app, account),
                buttons: vec![
                    ("Refresh ".to_owned(), false),
                    ("Validate ".to_owned(), false),
                    ("Edit… ".to_owned(), false),
                    ("Set default ".to_owned(), false),
                    (
                        if account.enabled {
                            "Disable ".to_owned()
                        } else {
                            "Enable ".to_owned()
                        },
                        false,
                    ),
                    ("Remove… ".to_owned(), true),
                ],
            }
        }
    }
}

/// The accounts of one provider, in registry order.
fn provider_accounts(app: &App, provider: Provider) -> Vec<&Account> {
    app.world
        .accounts
        .sorted()
        .into_iter()
        .filter(|account| account.provider == provider)
        .collect()
}

/// The inspector lines of the registry overview.
fn registry_lines(app: &App, summary: &OverallSummary) -> Vec<InspectLine> {
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
            value: sources_line(app),
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
    for account in app.world.accounts.sorted() {
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
                            .map(|secs| format!(" · {}", app.world.clock.reset_label(secs)))
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
                            .map(|secs| format!(" · {}", app.world.clock.reset_label(secs)))
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
                        .map(|secs| app.world.clock.ago(secs))
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
    let unresolved: Vec<&Account> = app
        .world
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

/// The inspector lines of one provider.
fn provider_lines(app: &App, provider: Provider, members: &[&Account]) -> Vec<InspectLine> {
    let mut lines = vec![
        InspectLine::Prop {
            label: "Provider".into(),
            value: provider.label().to_owned(),
            tone: InspectTone::Plain,
        },
        InspectLine::Prop {
            label: "Agent runtime".into(),
            value: provider
                .agent()
                .map_or("—", |agent| agent.label())
                .to_owned(),
            tone: InspectTone::Plain,
        },
        InspectLine::Prop {
            label: "Usage surface".into(),
            value: provider.usage_surface().surface_name().to_owned(),
            tone: InspectTone::Plain,
        },
        InspectLine::Prop {
            label: "Registration".into(),
            value: REGISTRATION_POLICY.to_owned(),
            tone: InspectTone::Plain,
        },
        InspectLine::Prop {
            label: "Default".into(),
            value: members
                .iter()
                .find(|account| account.default_for_provider)
                .map_or("none".to_owned(), |account| {
                    format!("★ {}", account.display_name)
                }),
            tone: InspectTone::Plain,
        },
        InspectLine::Blank,
    ];
    for account in members {
        lines.push(InspectLine::Heading {
            label: account.display_name.clone(),
            meta: account
                .tail_label(app)
                .unwrap_or_else(|| account.status_word().to_owned()),
        });
        for window in &account.usage.windows {
            lines.push(InspectLine::Prop {
                label: window.label.clone(),
                value: format!("{}% used", window.used_pct.unwrap_or(0)),
                tone: match window.status {
                    QuotaStatus::Exhausted => InspectTone::Danger,
                    QuotaStatus::Warning => InspectTone::Warning,
                    _ => InspectTone::Plain,
                },
            });
        }
    }
    lines
}

/// The inspector lines of one account.
fn account_lines(app: &App, account: &Account) -> Vec<InspectLine> {
    let mut lines = vec![
        InspectLine::Prop {
            label: "Provider".into(),
            value: account.provider.label().to_owned(),
            tone: InspectTone::Plain,
        },
        InspectLine::Prop {
            label: "Agent runtime".into(),
            value: account.agent.map_or("—", |agent| agent.label()).to_owned(),
            tone: InspectTone::Plain,
        },
        InspectLine::Prop {
            label: "Usage surface".into(),
            value: account.surface.surface_name().to_owned(),
            tone: InspectTone::Plain,
        },
        InspectLine::Prop {
            label: "Identity".into(),
            value: account.identity.label(),
            tone: InspectTone::Plain,
        },
        InspectLine::Prop {
            label: "Plan".into(),
            value: account
                .identity
                .plan
                .clone()
                .unwrap_or_else(|| "unknown".into()),
            tone: InspectTone::Plain,
        },
        InspectLine::Prop {
            label: "Credential".into(),
            value: credential_line(app, account),
            tone: InspectTone::Plain,
        },
        InspectLine::Text {
            text: credential_locator(app, account),
            tone: InspectTone::Muted,
        },
        InspectLine::Prop {
            label: "Provenance".into(),
            value: format!(
                "{} · confidence {} · {}",
                account.provenance.first().map_or("unknown", |p| p.label()),
                account.confidence.label(),
                account.source.origin_label().to_lowercase()
            ),
            tone: InspectTone::Plain,
        },
        InspectLine::Prop {
            label: "Lifecycle".into(),
            value: account.lifecycle.label().to_owned(),
            tone: InspectTone::Plain,
        },
        InspectLine::Prop {
            label: "Default".into(),
            value: if account.default_for_provider {
                "yes".to_owned()
            } else {
                "no".to_owned()
            },
            tone: InspectTone::Plain,
        },
        InspectLine::Prop {
            label: "Enabled".into(),
            value: if account.enabled {
                "yes".to_owned()
            } else {
                "no · disabled".to_owned()
            },
            tone: InspectTone::Plain,
        },
        InspectLine::Blank,
    ];
    if account.usage.windows.is_empty() {
        lines.push(InspectLine::Heading {
            label: "Quota".into(),
            meta: freshness_meta(app, account),
        });
        lines.push(InspectLine::Text {
            text: "No quota windows · not started or unavailable".into(),
            tone: InspectTone::Muted,
        });
    } else {
        lines.push(InspectLine::Heading {
            label: "Quota".into(),
            meta: freshness_meta(app, account),
        });
        for window in &account.usage.windows {
            lines.push(InspectLine::Prop {
                label: window.label.clone(),
                value: format!("{}% used", window.used_pct.unwrap_or(0)),
                tone: match window.status {
                    QuotaStatus::Exhausted => InspectTone::Danger,
                    QuotaStatus::Warning => InspectTone::Warning,
                    _ => InspectTone::Plain,
                },
            });
        }
    }
    lines.push(InspectLine::Blank);
    lines.push(InspectLine::Prop {
        label: "Validation".into(),
        value: validation_line(account),
        tone: InspectTone::Plain,
    });
    lines.push(InspectLine::Prop {
        label: "Status".into(),
        value: account.issue.as_ref().map_or_else(
            || account.status_word().to_owned(),
            |issue| issue.message.clone(),
        ),
        tone: InspectTone::Plain,
    });
    lines
}

/// The freshness line of an account.
fn freshness_meta(app: &App, account: &Account) -> String {
    let phase = account.usage.freshness.phase;
    let age = account
        .usage
        .freshness
        .last_good_secs
        .map(|secs| app.world.clock.ago(secs))
        .unwrap_or_else(|| "never".to_owned());
    match phase {
        Freshness::Current => format!("current · refreshed {age}"),
        Freshness::Stale => format!("stale · last good {age}"),
        Freshness::Refreshing => format!("refreshing · last good {age}"),
        Freshness::Failed => format!("failed · last good {age}"),
    }
}

/// The credential line of an account: the reference and its masked tail.
fn credential_line(_app: &App, account: &Account) -> String {
    match &account.source {
        CredentialSource::OnePassword(reference) => {
            format!(
                "{} · {}",
                account.source.origin_label(),
                reference.display_path()
            )
        }
        CredentialSource::LocalFolder { path, detected } => {
            format!(
                "{} · {path} detected {}",
                account.source.origin_label(),
                detected.label()
            )
        }
        CredentialSource::PlainApiKey { fingerprint, tail } => format!(
            "{} · {} · fingerprint {fingerprint}",
            account.source.origin_label(),
            crate::domain::account::masked(tail)
        ),
        CredentialSource::HostEnv { var, detected } => {
            format!(
                "{} · ${var} detected {}",
                account.source.origin_label(),
                detected.label()
            )
        }
    }
}

/// The canonical locator of the credential and its masked tail.
fn credential_locator(app: &App, account: &Account) -> String {
    let CredentialSource::OnePassword(reference) = &account.source else {
        return String::new();
    };
    let masked = app
        .world
        .op
        .describe(reference)
        .map(|descriptor| descriptor.masked)
        .unwrap_or_default();
    format!("{} · {masked}", reference.canonical())
}

/// The validation line of an account.
fn validation_line(account: &Account) -> String {
    let material = if account.validation.level().is_some() {
        "✓ material"
    } else {
        "— material"
    };
    let identity = if account.identity.subject.is_some() {
        "✓ identity"
    } else {
        "— identity"
    };
    let quota = if quota_is_hidden(account) {
        "— quota access"
    } else {
        "✓ quota access"
    };
    format!("{material}   {identity}   {quota}")
}

/// The tail label of an account, for the provider inspector.
trait TailLabel {
    fn tail_label(&self, app: &App) -> Option<String>;
}

impl TailLabel for Account {
    fn tail_label(&self, app: &App) -> Option<String> {
        account_tail(app, self).map(|tail| tail.0)
    }
}

/// The narrow inspector band of the selected node.
///
/// Each entry is one band line: the label, then the value.
fn narrow_facts(
    app: &App,
    selection: &Selection,
    area_width: u16,
) -> Vec<Vec<(String, DetailInk)>> {
    let summary = OverallSummary::compute(&app.world.accounts.accounts);
    let health = health_text(app, &summary);
    let mut facts = Vec::new();
    match selection {
        Selection::Account(id) => {
            let Some(account) = app.world.accounts.get(id) else {
                return facts;
            };
            facts.push(vec![
                ("  Provider  ".into(), DetailInk::Muted),
                (account.provider.label().to_owned(), DetailInk::Plain),
            ]);
            facts.push(vec![
                ("  Agent runtime  ".into(), DetailInk::Muted),
                (
                    account.agent.map_or("—", |agent| agent.label()).to_owned(),
                    DetailInk::Plain,
                ),
            ]);
            facts.push(vec![
                ("  Usage surface  ".into(), DetailInk::Muted),
                (account.surface.surface_name().to_owned(), DetailInk::Plain),
            ]);
            facts.push(vec![
                ("  Quota  ".into(), DetailInk::Muted),
                (freshness_meta(app, account), DetailInk::Plain),
            ]);
        }
        Selection::Provider(provider) => {
            facts.push(vec![
                ("  Provider  ".into(), DetailInk::Muted),
                (provider.label().to_owned(), DetailInk::Plain),
            ]);
            facts.push(vec![
                ("  Accounts  ".into(), DetailInk::Muted),
                (
                    count(
                        provider_accounts(app, *provider).len(),
                        "account",
                        "accounts",
                    ),
                    DetailInk::Plain,
                ),
            ]);
        }
        Selection::Overview => {
            facts.push(vec![
                ("  Health  ".into(), DetailInk::Muted),
                (
                    truncate(&health, usize::from(area_width.saturating_sub(14))),
                    DetailInk::DangerBold,
                ),
            ]);
            facts.push(vec![
                ("  Registry  ".into(), DetailInk::Muted),
                (summary.counts_line(), DetailInk::Plain),
            ]);
            facts.push(vec![
                ("  Sources  ".into(), DetailInk::Muted),
                (sources_line(app), DetailInk::Plain),
            ]);
        }
    }
    facts
}

/// The heading of the narrow inspector band.
fn narrow_heading(app: &App, selection: &Selection) -> (String, String) {
    let summary = OverallSummary::compute(&app.world.accounts.accounts);
    match selection {
        Selection::Overview => (
            "Overview".to_owned(),
            format!(
                "{} · {} · Tab details",
                count(summary.counts.accounts, "account", "accounts"),
                count(summary.counts.providers, "provider", "providers")
            ),
        ),
        Selection::Provider(provider) => (
            provider.label().to_owned(),
            format!(
                "provider · {} · Tab details",
                count(
                    provider_accounts(app, *provider).len(),
                    "account",
                    "accounts"
                )
            ),
        ),
        Selection::Account(id) => {
            let Some(account) = app.world.accounts.get(id) else {
                return (String::new(), String::new());
            };
            (
                format!(
                    "{} · {}",
                    account.surface.surface_name(),
                    account.display_name
                ),
                format!("account · {} · Tab details", account.status_word()),
            )
        }
    }
}

/// The health text of the registry.
fn health_text(app: &App, summary: &OverallSummary) -> String {
    let _ = app;
    let mut text = summary.health.label().to_owned();
    let counts = &summary.counts;
    if counts.warnings > 0 {
        text.push_str(&format!(
            " · {}",
            count(counts.warnings, "warning", "warnings")
        ));
    }
    if counts.exhausted > 0 {
        text.push_str(&format!(" · {} exhausted", counts.exhausted));
    }
    if counts.stale > 0 {
        text.push_str(&format!(" · {} stale", counts.stale));
    }
    if counts.refreshing > 0 {
        text.push_str(&format!(" · {} refreshing", counts.refreshing));
    }
    if counts.failed > 0 {
        text.push_str(&format!(" · {} failed", counts.failed));
    }
    if counts.unsupported > 0 {
        text.push_str(&format!(" · {} quota not visible", counts.unsupported));
    }
    if counts.unresolved_identity > 0 {
        let word = if counts.unresolved_identity == 1 {
            "y"
        } else {
            "ies"
        };
        text.push_str(&format!(
            " · {} identit{word} unresolved",
            counts.unresolved_identity
        ));
    }
    text
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

fn sources_line(app: &App) -> String {
    let registered = app
        .world
        .accounts
        .accounts
        .iter()
        .filter(|account| account.origin == AccountOrigin::Registered)
        .count();
    let discovered = app.world.accounts.accounts.len() - registered;
    let session = match app.world.op.session {
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

/// Cut `text` to `width` columns, ending on a word, and mark the cut.
fn truncate(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_owned();
    }
    let mut cut: String = text.chars().take(width.saturating_sub(1)).collect();
    cut.push('…');
    cut
}

/// What the tree cursor stands for.
enum Selection {
    Overview,
    Provider(Provider),
    Account(String),
}

fn selection(app: &App, tree: &Tree) -> Selection {
    let Some(index) = cursor_index(app) else {
        return Selection::Overview;
    };
    match tree.rows.get(index).map(|row| &row.node) {
        Some(Node::Provider(provider)) => Selection::Provider(*provider),
        Some(Node::Account(id)) => Selection::Account(id.clone()),
        _ => Selection::Overview,
    }
}

fn cursor_index(app: &App) -> Option<usize> {
    match app.accounts.list.cursor() {
        Some(termrock::ItemKey::Index(index)) => Some(index),
        _ => None,
    }
}

/// The cut names of the wide list.
fn pane_names(tree: &Tree) -> Vec<String> {
    tree.rows
        .iter()
        .map(|row| truncate(&row.name, pane_name_width(row)))
        .collect()
}

/// The cut names of the narrow list.
fn detail_names(area_width: u16, tree: &Tree) -> Vec<String> {
    tree.rows
        .iter()
        .map(|row| truncate(&row.name, narrow_name_width(area_width, row)))
        .collect()
}

/// The rows of the wide list.
fn pane_rows<'a>(app: &App, tree: &'a Tree, names: &'a [String]) -> Vec<PaneRow<'a>> {
    let selected = cursor_index(app);
    tree.rows
        .iter()
        .enumerate()
        .map(|(index, row)| PaneRow {
            role: row.role,
            mark: row.mark.as_deref(),
            name: &names[index],
            ink: row.ink,
            alert: row.alert.map(|(glyph, _)| glyph),
            alert_tone: row.alert.map_or(DetailAlert::Danger, |(_, tone)| tone),
            selected: Some(index) == selected,
            empty: false,
        })
        .collect()
}

/// The rows of the narrow list.
fn detail_rows<'a>(app: &App, tree: &'a Tree, names: &'a [String]) -> Vec<AccountRow<'a>> {
    let selected = cursor_index(app);
    tree.rows
        .iter()
        .enumerate()
        .map(|(index, row)| AccountRow {
            role: row.role,
            mark: row.mark.as_deref(),
            name: &names[index],
            alert: row.alert.map(|(glyph, _)| glyph),
            alert_tone: row.alert.map_or(DetailAlert::Danger, |(_, tone)| tone),
            tail: row.tail.as_deref(),
            tail_tone: row.tail_tone,
            selected: Some(index) == selected,
            faded: false,
        })
        .collect()
}

/// The name column of a row of the wide list.
fn pane_name_width(row: &TreeRow) -> usize {
    let name_x = name_column(row);
    let right = if row.alert.is_some() {
        LIST_PANE_W.saturating_sub(5)
    } else {
        LIST_PANE_W.saturating_sub(2)
    };
    usize::from(right.saturating_sub(name_x))
}

/// The name column of a row of the narrow list.
fn narrow_name_width(area_width: u16, row: &TreeRow) -> usize {
    let name_x = name_column(row);
    let tail =
        u16::try_from(row.tail.as_ref().map_or(0, |tail| tail.chars().count())).unwrap_or(u16::MAX);
    // The tail sits against the scrollbar, three columns clear of the name.
    let right = area_width
        .saturating_sub(4)
        .saturating_sub(1)
        .saturating_sub(tail)
        .saturating_sub(2);
    usize::from(right.saturating_sub(name_x))
}

/// The column of the name of a row, from the left edge of the screen.
fn name_column(row: &TreeRow) -> u16 {
    match row.role {
        AccountRole::Root | AccountRole::Group => 7,
        AccountRole::Item | AccountRole::Note => 9,
    }
}

/// How many rows the narrow list shows.
fn visible_rows(area: Rect, total: usize) -> usize {
    let band = area.height.saturating_sub(8);
    usize::from(band.saturating_sub(5)).max(1).min(total)
}

/// The first visible row of the narrow list.
fn list_offset(app: &App, tree: &Tree, visible: usize) -> u16 {
    let total = tree.rows.len();
    if total <= visible {
        return 0;
    }
    let cursor = cursor_index(app).unwrap_or(0);
    if cursor < visible {
        return 0;
    }
    u16::try_from(cursor.saturating_sub(visible / 2)).unwrap_or(u16::MAX)
}

/// The title of the list box, cut to the room left by the summary.
fn list_title(app: &App, selection: &Selection, summary: &str) -> String {
    let title = if app.accounts.filtering && !app.accounts.filter_query.is_empty() {
        format!(" Accounts · filter {} ", app.accounts.filter_query)
    } else {
        match selection {
            Selection::Overview => " Accounts ".to_owned(),
            Selection::Provider(provider) => {
                format!(" {} ", provider.usage_surface().surface_name())
            }
            Selection::Account(_) => " Accounts ".to_owned(),
        }
    };
    let room = LIST_PANE_W
        .saturating_sub(4)
        .saturating_sub(u16::try_from(summary.chars().count()).unwrap_or(u16::MAX));
    truncate(&title, usize::from(room))
}

/// The registry counts of the list box.
fn list_summary(app: &App) -> String {
    let summary = OverallSummary::compute(&app.world.accounts.accounts);
    format!(
        " {} · {} ▲ · {} ! ",
        summary.counts.accounts,
        summary.counts.warnings,
        summary.counts.exhausted + summary.counts.failed
    )
}

/// The breadcrumb of the header.
fn crumb(app: &App, selection: &Selection) -> String {
    match selection {
        Selection::Overview => "Accounts › Overview".to_owned(),
        Selection::Provider(provider) => format!("Accounts › {}", provider.label()),
        Selection::Account(id) => {
            let Some(account) = app.world.accounts.get(id) else {
                return "Accounts › Overview".to_owned();
            };
            format!(
                "Accounts › {} › {}",
                account.surface.surface_name(),
                account.display_name
            )
        }
    }
}

/// The status of the header.
///
/// A transient app message takes the slot; otherwise the slot shows how many
/// accounts are refreshing.
fn header_status(app: &App) -> String {
    let refreshing = app
        .world
        .accounts
        .accounts
        .iter()
        .filter(|account| account.enabled && account.usage.freshness.phase == Freshness::Refreshing)
        .count();
    format!("⠋ refreshing {refreshing}")
}

/// The accounts that carry a warning mark.
fn warning_count(app: &App) -> usize {
    let summary = OverallSummary::compute(&app.world.accounts.accounts);
    summary.counts.warnings
}

/// The accounts that carry a danger mark.
fn danger_count(app: &App) -> usize {
    let summary = OverallSummary::compute(&app.world.accounts.accounts);
    summary.counts.exhausted + summary.counts.failed
}
