//! Accounts page: the account tree, the registry inspector, and the drawer.
//!
//! Every value on this page comes from the live registry. The tree rows are
//! the accounts of the world, the inspector is the registry summary, and the
//! drawer shows the selected node. The page picks its layout from the area it
//! is given: a wide area gets the list beside the inspector, a narrow area
//! gets the list above the inspector band. Nothing here keys on a screen size.

use termrock::feedback::{
    AccountRole, AccountRow, AccountsDetail, AccountsDrawer, DetailAlert, DetailInk, DetailLine,
    DetailSpan, DetailTail, DrawerButton, DrawerInk, DrawerLine, DrawerMeter, DrawerTone, FilterHint,
    InspectLine, InspectSpan, InspectTone, PaneChrome, PaneRow, SettingsCluster,
};
use termrock::{FrameRead, Rect, Ui};

use crate::domain::account::{Account, AccountOrigin, CredentialSource, IssueCode, Lifecycle};
use crate::domain::agent::Provider;
use crate::domain::usage::{Freshness, HealthWord, OverallSummary, QuotaStatus};
use crate::sim::onepassword::OpSession;

use super::App;

/// The layout switches to the inspector beside the list when the area is wide
/// enough for both columns and tall enough for the shared painters.
///
/// The reference splits the accounts page when the body beside the list is
/// 100 columns wide, so the page needs 102 columns of its own.
const WIDE_MIN_WIDTH: u16 = 102;
const WIDE_MIN_HEIGHT: u16 = 24;

/// The width of the account list pane of the shared accounts painters.
///
/// The shared painters own this geometry; the page needs it to cut a row
/// name before the pane edge.
const LIST_PANE_W: u16 = 39;

/// The column of the panel values, measured from the panel content edge.
const PANEL_VALUE_AT: u16 = 22;

/// The width of the quota meter bar of the panel.
const METER_BAR_W: u16 = 34;

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
    // A filtered tree shows the matches only: the sentinel section and the
    // add-account row are not part of the query.
    if !app.filter_active() {
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
    }
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
    // A discovered account says so in the tree.
    let name = if account.origin == AccountOrigin::Discovered
        && account.display_name != "discovered"
    {
        format!("{} · discovered", account.display_name)
    } else {
        account.display_name.clone()
    };
    TreeRow {
        node: Node::Account(account.id.clone()),
        role,
        mark: account.default_for_provider.then(|| "★".to_owned()),
        name,
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
        Freshness::Failed => Some((failed_tail(account), DetailTail::Muted)),
        Freshness::Current => match account.lifecycle {
            Lifecycle::Unsupported => Some(("unsupported".to_owned(), DetailTail::Muted)),
            Lifecycle::NeedsLogin => Some(("needs login".to_owned(), DetailTail::Warning)),
            Lifecycle::NeedsSecret => Some(("needs secret".to_owned(), DetailTail::Warning)),
            _ => Some((
                format!("current {}", age_label(app, account)),
                DetailTail::Muted,
            )),
        },
    }
}

/// The tail of an account whose usage failed: the issue decides the word.
fn failed_tail(account: &Account) -> String {
    match account.issue.as_ref().map(|issue| issue.code) {
        Some(IssueCode::RateLimited) => "rate limited".to_owned(),
        Some(IssueCode::ProviderUnavailable) => "unavailable".to_owned(),
        Some(IssueCode::CredentialFileMissing) => "needs secret".to_owned(),
        Some(IssueCode::Unauthorized | IssueCode::ApiKeyInvalid) => "unauthorized".to_owned(),
        Some(IssueCode::OpLocked) => "1Password locked".to_owned(),
        _ => "error".to_owned(),
    }
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
    let visible = visible_rows(area, true);
    let total = tree.rows.len();
    let offset = list_offset(app, total, visible);
    draw_list_owner(app, ui, list_area(area, true, visible));
    let names = pane_names(tree);
    let rows = pane_rows(app, tree, &names, offset, visible);
    let summary = list_summary(app);
    let title = list_title(app, &selection, &summary, area, true);
    let (crumb, status, running, note) = header_parts(app, area, &selection, true);
    let room = panel_value_room(area);
    let note_room = panel_note_room(area);
    let panel = panel_for(app, &selection, room, note_room);
    let buttons: Vec<DrawerButton<'_>> = panel
        .buttons
        .iter()
        .map(|(label, danger)| DrawerButton {
            label,
            danger: *danger,
        })
        .collect();
    let lines = panel_lines(&panel, area, &buttons);
    let hints: &[FilterHint<'_>] = match &selection {
        Selection::Account(_) if app.accounts.drawer_open => &DRAWER_HINTS,
        Selection::Account(_) => &ACCOUNT_HINTS,
        Selection::Provider(_) => &PROVIDER_HINTS,
        Selection::Overview => &OVERVIEW_HINTS,
    };
    let (hints, overflow) = fit_hints(hints, area.width);
    let page = AccountsDrawer {
        brand: "jackin❯",
        menus: &["File", "Go", "Help"],
        crumb: &crumb,
        status: &status,
        cluster: SettingsCluster {
            note: &note,
            mark: "●",
            value: &running,
        },
        title: &title,
        summary: &summary,
        rows: &rows,
        lines: &lines,
        hints: &hints,
        hint_overflow: &overflow,
        chrome: if app.accounts.drawer_open {
            PaneChrome::Drawer
        } else {
            PaneChrome::Detail
        },
    };
    termrock::feedback::paint_accounts_drawer(ui, area, &page);
    if !matches!(selection, Selection::Account(_)) {
        termrock::feedback::paint_inspector_body(ui, area, &panel.body);
    }
    register_add_row(ui, area, tree, offset);
}

/// The list above the inspector band.
fn paint_narrow(app: &App, ui: &mut Ui<'_>, area: Rect, tree: &Tree) {
    let selection = selection(app, tree);
    let visible = visible_rows(area, false);
    let total = tree.rows.len();
    let offset = list_offset(app, total, visible);
    draw_list_owner(app, ui, list_area(area, false, visible));
    let names = detail_names(area.width, tree);
    let rows = detail_rows(app, tree, &names, offset, visible);
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
    let shown = offset.saturating_add(visible).min(total);
    // The range shows only when the tree does not fit on the screen.
    let range = if total > visible {
        format!(" · {}\u{2013}{shown} of {total}", offset.saturating_add(1))
    } else {
        String::new()
    };
    let summary = format!(
        " {} · {} ▲ · {} !{range} ",
        app.world.accounts.accounts.len(),
        warning_count(app),
        danger_count(app),
    );
    let title = list_title(app, &selection, &summary, area, false);
    let hints: &[FilterHint<'_>] = match &selection {
        Selection::Account(_) if app.accounts.drawer_open => &DRAWER_HINTS,
        Selection::Account(_) => &ACCOUNT_HINTS,
        Selection::Provider(_) => &PROVIDER_HINTS,
        Selection::Overview => &OVERVIEW_HINTS,
    };
    let (hints, overflow) = fit_hints(hints, area.width);
    let page = AccountsDetail {
        brand: "jackin❯",
        menus: &["File", "Go", "Help"],
        crumb: &crumb,
        status: &status,
        title: &title,
        summary: &summary,
        rows: &rows,
        total: u16::try_from(total).unwrap_or(u16::MAX),
        offset: u16::try_from(offset).unwrap_or(u16::MAX),
        heading: &heading,
        heading_meta: &heading_meta,
        facts: &fact_lines,
        hints: &hints,
        hint_overflow: &overflow,
    };
    termrock::feedback::paint_accounts_detail(ui, area, &page);
    register_add_row(ui, area, tree, offset);
}

/// Register the add-account row of the tree as the add control.
///
/// The region follows the painted row: the tree scrolls, so the row index and
/// the row on the screen are not the same thing.
fn register_add_row(ui: &mut Ui<'_>, area: Rect, tree: &Tree, offset: usize) {
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
    // The painted row of the tree, counted from the first row of the box.
    let Some(painted) = index.checked_sub(offset) else {
        return;
    };
    let y = area
        .y
        .saturating_add(3)
        .saturating_add(u16::try_from(painted).unwrap_or(u16::MAX));
    if y >= area.bottom() {
        return;
    }
    ui.register_control(
        crate::ACCOUNT_ADD,
        Rect::new(area.x.saturating_add(name_x), y, 20, 1),
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
    // The value carries the gap behind the running mark.
    let value = format!(" {} running", app.world.running_count());
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
///
/// The rows of the shared painters start under the box title, so the list
/// owner starts there too: the hit region of a row and the row on the screen
/// are then the same row.
fn list_area(area: Rect, wide: bool, visible: usize) -> Rect {
    let width = if wide {
        LIST_PANE_W
    } else {
        area.width.saturating_sub(2)
    };
    Rect::new(
        area.x.saturating_add(1),
        area.y.saturating_add(3),
        width,
        u16::try_from(visible).unwrap_or(u16::MAX),
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
    /// The fields of the panel.
    fields: Vec<PanelField>,
    /// The heading of the quota block and its meta.
    quota: Option<(String, String)>,
    /// The quota meters of the panel.
    meters: Vec<PanelMeter>,
    /// The validation line of the panel.
    validation: Option<PanelField>,
    /// The status line of the panel.
    status: Option<PanelField>,
    /// The inspector body of the registry summary.
    body: Vec<InspectLine>,
    /// The buttons at the foot of the panel.
    buttons: Vec<(String, bool)>,
}

/// One field of the panel: the label, the wrapped value and the ink.
struct PanelField {
    /// The label of the field.
    label: String,
    /// The value of the field, cut on the value column.
    parts: Vec<String>,
    /// The ink of the value.
    ink: DrawerInk,
}

/// One quota meter of the panel.
struct PanelMeter {
    /// The label of the meter.
    label: String,
    /// The readout inside the bar.
    readout: String,
    /// The filled columns of the bar.
    fill: u16,
    /// The empty columns of the bar.
    track: u16,
    /// The tone of the bar.
    tone: DrawerTone,
    /// The mark after the bar.
    mark: String,
    /// Whether the mark is bold.
    mark_bold: bool,
    /// The note after the mark, cut to the room of the note.
    note: String,
}

/// The panel of the selected node.
fn panel_for(app: &App, selection: &Selection, room: usize, note_room: usize) -> Panel {
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
                fields: Vec::new(),
                quota: None,
                meters: Vec::new(),
                validation: None,
                status: None,
                body: registry_lines(app, &summary),
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
                fields: Vec::new(),
                quota: None,
                meters: Vec::new(),
                validation: None,
                status: None,
                body: provider_lines(app, *provider, &members),
                buttons: vec![
                    ("Refresh provider ".to_owned(), false),
                    ("Add account… ".to_owned(), false),
                ],
            }
        }
        Selection::Account(id) => {
            let Some(account) = app.world.accounts.get(id) else {
                return panel_for(app, &Selection::Overview, room, note_room);
            };
            let mut fields = account_fields(app, account);
            let validation = validation_line(account);
            let status = account_status(app, account);
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
                fields: fields
                    .drain(..)
                    .map(|(label, value, ink)| field(label, value, ink, room))
                    .collect(),
                quota: Some(("Quota".to_owned(), freshness_meta(app, account))),
                meters: account_meters(app, account, note_room),
                validation: Some(field(
                    validation.0,
                    validation.1,
                    DrawerInk::Plain,
                    room,
                )),
                status: status.map(|(value, ink)| field("Status".to_owned(), value, ink, room)),
                body: Vec::new(),
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

/// One field of the panel, cut on the value column.
fn field(label: String, value: String, ink: DrawerInk, room: usize) -> PanelField {
    PanelField {
        label,
        parts: wrap_value(&value, room),
        ink,
    }
}

/// The fields of the account panel.
fn account_fields(app: &App, account: &Account) -> Vec<(String, String, DrawerInk)> {
    let mut fields: Vec<(String, String, DrawerInk)> = vec![
        (
            "Provider".to_owned(),
            account.provider.label().to_owned(),
            DrawerInk::Plain,
        ),
        (
            "Agent runtime".to_owned(),
            account.agent.map_or("—", |agent| agent.label()).to_owned(),
            DrawerInk::Plain,
        ),
        (
            "Usage surface".to_owned(),
            account.surface.surface_name().to_owned(),
            DrawerInk::Plain,
        ),
        (
            "Identity".to_owned(),
            account.identity.label(),
            DrawerInk::Plain,
        ),
        (
            "Plan".to_owned(),
            account
                .identity
                .plan
                .clone()
                .unwrap_or_else(|| "unknown".to_owned()),
            DrawerInk::Plain,
        ),
        (
            "Credential".to_owned(),
            credential_line(app, account),
            DrawerInk::Plain,
        ),
    ];
    if let CredentialSource::OnePassword(reference) = &account.source {
        let masked = app
            .world
            .op
            .describe(reference)
            .map(|descriptor| descriptor.masked)
            .unwrap_or_default();
        fields.push((
            String::new(),
            format!("{} · {} · {masked}", reference.account, reference.canonical()),
            DrawerInk::Muted,
        ));
    }
    fields.push((
        "Provenance".to_owned(),
        format!(
            "{} · confidence {} · {}",
            account
                .provenance
                .first()
                .map_or("unknown", |provenance| provenance.label()),
            account.confidence.label(),
            match account.origin {
                AccountOrigin::Registered => "registered",
                AccountOrigin::Discovered => "discovered",
            }
        ),
        DrawerInk::Secondary,
    ));
    fields.push((
        "Lifecycle".to_owned(),
        account.lifecycle.label().to_owned(),
        DrawerInk::Plain,
    ));
    fields.push((
        "Default".to_owned(),
        if account.default_for_provider {
            "yes".to_owned()
        } else {
            "no".to_owned()
        },
        DrawerInk::Plain,
    ));
    fields.push((
        "Enabled".to_owned(),
        if account.enabled {
            "yes".to_owned()
        } else {
            "no · disabled".to_owned()
        },
        DrawerInk::Plain,
    ));
    if let Some(purpose) = &account.purpose {
        fields.push(("Purpose".to_owned(), purpose.clone(), DrawerInk::Plain));
    }
    let users = workspace_users(app, account);
    if !users.is_empty() {
        fields.push((
            "Used by".to_owned(),
            format!("Workspace choice in {}", users.join(", ")),
            DrawerInk::Secondary,
        ));
    }
    fields
}

/// The workspaces that use the account.
fn workspace_users(app: &App, account: &Account) -> Vec<String> {
    app.world
        .workspaces
        .iter()
        .filter(|workspace| {
            workspace
                .effective_accounts(&app.world.accounts)
                .iter()
                .any(|entry| entry.id == account.id)
        })
        .map(|workspace| workspace.name.clone())
        .collect()
}

/// The quota meters of the account panel.
fn account_meters(app: &App, account: &Account, note_room: usize) -> Vec<PanelMeter> {
    account
        .usage
        .windows
        .iter()
        .map(|window| {
            let pct = window.used_pct.unwrap_or(0);
            // The bar is filled to the rounded share of its width.
            let fill = u16::from(pct)
                .saturating_mul(METER_BAR_W)
                .saturating_add(50)
                .saturating_div(100)
                .min(METER_BAR_W);
            let track = METER_BAR_W.saturating_sub(fill);
            let (tone, mark, mark_bold) = match window.status {
                QuotaStatus::Exhausted => (DrawerTone::Danger, " !", true),
                QuotaStatus::Warning => (DrawerTone::Warning, " ▲", false),
                _ => (DrawerTone::Info, "  ", false),
            };
            PanelMeter {
                label: format!("  {:<18}", window.label),
                readout: format!("{pct:>4}%"),
                fill,
                track,
                tone,
                mark: mark.to_owned(),
                mark_bold,
                note: truncate(&meter_detail(app, window), note_room),
            }
        })
        .collect()
}

/// The note of a quota meter: the spend, the reset and the state.
fn meter_detail(app: &App, window: &crate::domain::usage::QuotaWindow) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let (Some(used), Some(limit)) = (window.used, window.limit)
        && window.unit != crate::domain::usage::WindowUnit::Percent
    {
        parts.push(format!(
            "{} / {} {}",
            thousands(used as usize),
            thousands(limit as usize),
            window.unit.label()
        ));
    }
    if let Some(reset) = window.reset_secs {
        parts.push(app.world.clock.reset_label(reset));
    }
    if let Some(spend) = &window.spend_label {
        parts.push(spend.clone());
    }
    if window.status == QuotaStatus::Exhausted {
        parts.push("exhausted".to_owned());
    }
    parts.join(" · ")
}

/// The status line of the account panel.
fn account_status(app: &App, account: &Account) -> Option<(String, DrawerInk)> {
    let issue = account.issue.as_ref()?;
    let ink = if issue.is_informational() || issue.code == IssueCode::QuotaUnsupported {
        DrawerInk::Warning
    } else {
        DrawerInk::Danger
    };
    let mut text = issue.message.clone();
    if let Some(detail) = &issue.detail {
        text.push_str(&format!(" · {detail}"));
    }
    if let Some(retry) = issue.retry_secs {
        text.push_str(&format!(
            " · retry {}",
            app.world.clock.reset_label(retry).replace("resets", "")
        ));
    }
    text.push_str(&format!(" · {}", issue.recoverability.label()));
    Some((text, ink))
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
                ("  Provider  ".into(), DetailInk::Plain),
                (account.provider.label().to_owned(), DetailInk::Plain),
            ]);
            facts.push(vec![
                ("  Agent runtime  ".into(), DetailInk::Plain),
                (
                    account.agent.map_or("—", |agent| agent.label()).to_owned(),
                    DetailInk::Plain,
                ),
            ]);
            facts.push(vec![
                ("  Usage surface  ".into(), DetailInk::Plain),
                (account.surface.surface_name().to_owned(), DetailInk::Plain),
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
        push_part(&mut spans, format!("{} stale", counts.stale), InspectTone::Warning);
    }
    if counts.refreshing > 0 {
        push_part(
            &mut spans,
            format!("{} refreshing", counts.refreshing),
            InspectTone::Info,
        );
    }
    if counts.failed > 0 {
        push_part(&mut spans, format!("{} failed", counts.failed), InspectTone::Danger);
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
fn pane_rows<'a>(
    app: &App,
    tree: &'a Tree,
    names: &'a [String],
    offset: usize,
    visible: usize,
) -> Vec<PaneRow<'a>> {
    let selected = cursor_index(app);
    tree.rows
        .iter()
        .enumerate()
        .skip(offset)
        .take(visible)
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
fn detail_rows<'a>(
    app: &App,
    tree: &'a Tree,
    names: &'a [String],
    offset: usize,
    visible: usize,
) -> Vec<AccountRow<'a>> {
    let selected = cursor_index(app);
    let total = tree.rows.len();
    let shown = visible.min(total.saturating_sub(offset));
    tree.rows
        .iter()
        .enumerate()
        .skip(offset)
        .take(visible)
        .map(|(index, row)| AccountRow {
            role: row.role,
            mark: row.mark.as_deref(),
            name: &names[index],
            alert: row.alert.map(|(glyph, _)| glyph),
            alert_tone: row.alert.map_or(DetailAlert::Danger, |(_, tone)| tone),
            tail: row.tail.as_deref(),
            tail_tone: row.tail_tone,
            selected: Some(index) == selected,
            // The last row on the screen fades into the edge of the list when
            // the list carries more rows below.
            faded: index == offset.saturating_add(shown).saturating_sub(1) && total > visible,
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

/// How many rows the list shows in one screenful.
fn visible_rows(area: Rect, wide: bool) -> usize {
    let rows = if wide {
        area.height.saturating_sub(6)
    } else {
        area.height.saturating_sub(13)
    };
    usize::from(rows.max(1))
}

/// The first visible row of the list.
///
/// The list owns the scroll: its offset keeps the cursor on the screen, and
/// the rows the painter draws are the rows the list shows.
fn list_offset(app: &App, total: usize, visible: usize) -> usize {
    let offset = app.accounts.list.scroll().offset();
    offset.min(total.saturating_sub(visible))
}

/// The title of the list box, cut to the room left by the summary.
fn list_title(app: &App, selection: &Selection, summary: &str, area: Rect, wide: bool) -> String {
    let inner = if app.filter_active() {
        format!("Accounts · filter {}", app.accounts.filter_query.trim())
    } else {
        match selection {
            Selection::Provider(provider) => provider.usage_surface().surface_name().to_owned(),
            Selection::Overview | Selection::Account(_) => "Accounts".to_owned(),
        }
    };
    // The title stops one column before the summary, so the dashes between
    // them always have a column to draw.
    let title_x = area.x.saturating_add(3);
    let summary_x = if wide {
        area.x
            .saturating_add(LIST_PANE_W)
            .saturating_sub(1)
            .saturating_sub(u16::try_from(text_width(summary)).unwrap_or(u16::MAX))
    } else {
        area.right()
            .saturating_sub(3)
            .saturating_sub(u16::try_from(text_width(summary)).unwrap_or(u16::MAX))
    };
    let room = summary_x.saturating_sub(title_x).saturating_sub(1);
    let inner = truncate(&inner, usize::from(room.saturating_sub(2)));
    format!(" {inner} ")
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
        CredentialSource::PlainApiKey { fingerprint, tail } => {
            format!(
                "{} · {} · fingerprint {fingerprint}",
                account.source.origin_label(),
                crate::domain::account::masked(tail)
            )
        }
        CredentialSource::HostEnv { var, detected } => {
            format!(
                "{} · ${var} detected {}",
                account.source.origin_label(),
                detected.label()
            )
        }
    }
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

/// The validation line of an account.
fn validation_line(account: &Account) -> (String, String) {
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
    (
        "Validation".to_owned(),
        format!("{material}   {identity}   {quota} · {}", account.validation.label()),
    )
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

/// The room of the panel values, measured in columns.
///
/// The values sit in the value column of the panel and stop at the panel
/// edge, so a long value wraps on the same columns as the reference.
fn panel_value_room(area: Rect) -> usize {
    let content_x = area.x.saturating_add(LIST_PANE_W).saturating_add(3);
    let body_right = area.right().saturating_sub(3);
    let value_x = content_x.saturating_add(PANEL_VALUE_AT);
    usize::from(body_right.saturating_sub(value_x))
}

/// The room of the meter notes, measured in columns.
fn panel_note_room(area: Rect) -> usize {
    let content_x = area.x.saturating_add(LIST_PANE_W).saturating_add(3);
    let body_right = area.right().saturating_sub(3);
    let note_x = content_x
        .saturating_add(PANEL_VALUE_AT)
        .saturating_add(METER_BAR_W)
        .saturating_add(4);
    usize::from(body_right.saturating_sub(note_x))
}

/// The lines of the panel, in the order the shared painter draws them.
fn panel_lines<'a>(
    panel: &'a Panel,
    area: Rect,
    buttons: &'a [DrawerButton<'a>],
) -> Vec<DrawerLine<'a>> {
    let mut lines = vec![
        DrawerLine::Title {
            title: &panel.title,
            meta: &panel.meta,
        },
        DrawerLine::Rule,
    ];
    for field in &panel.fields {
        push_field(&mut lines, field);
    }
    if let Some((label, meta)) = &panel.quota {
        // The quota block sits under a blank row.
        lines.push(DrawerLine::Blank);
        lines.push(DrawerLine::Heading { label, meta });
    }
    for meter in &panel.meters {
        lines.push(DrawerLine::Meter(DrawerMeter {
            label: &meter.label,
            readout: &meter.readout,
            fill_cols: meter.fill,
            track_cols: meter.track,
            tone: meter.tone,
            mark: &meter.mark,
            mark_bold: meter.mark_bold,
            note: &meter.note,
        }));
    }
    if let Some(validation) = &panel.validation {
        lines.push(DrawerLine::Blank);
        push_field(&mut lines, validation);
    }
    if let Some(status) = &panel.status {
        push_field(&mut lines, status);
    }
    // The buttons sit at the foot of the panel.
    let panel_rows = usize::from(area.height.saturating_sub(4));
    while lines.len() + 2 < panel_rows {
        lines.push(DrawerLine::Blank);
    }
    lines.push(DrawerLine::Buttons(buttons));
    lines.push(DrawerLine::Blank);
    lines
}

/// Push one field of the panel: the first part under the label, the rest
/// under the value column.
fn push_field<'a>(lines: &mut Vec<DrawerLine<'a>>, field: &'a PanelField) {
    for (index, part) in field.parts.iter().enumerate() {
        if index == 0 {
            lines.push(DrawerLine::Field {
                label: &field.label,
                value: part,
                ink: field.ink,
            });
        } else {
            lines.push(DrawerLine::Field {
                label: "",
                value: part,
                ink: field.ink,
            });
        }
    }
}

/// Wrap a panel value on the value column.
///
/// The wrap is the reference wrap: a greedy word wrap on the value column,
/// with a hanging indent after the first run of three spaces.
fn wrap_value(text: &str, room: usize) -> Vec<String> {
    let room = room.max(8);
    let hang = hang_column(text, room);
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    let mut used = 0usize;
    let mut base = 0usize;
    for word in text.split(' ') {
        let word_w = text_width(word);
        if used == base {
            if used + word_w <= room {
                line.push_str(word);
                used += word_w;
            } else {
                hard_wrap(&mut line, &mut used, &mut base, word, room, &mut lines);
            }
        } else if used + 1 + word_w <= room {
            line.push(' ');
            line.push_str(word);
            used += 1 + word_w;
        } else {
            lines.push(std::mem::take(&mut line));
            line = " ".repeat(hang);
            used = hang;
            base = hang;
            if used + word_w <= room {
                line.push_str(word);
                used += word_w;
            } else {
                hard_wrap(&mut line, &mut used, &mut base, word, room, &mut lines);
            }
        }
    }
    lines.push(line);
    lines
}

/// Split a word that does not fit the room.
fn hard_wrap(
    line: &mut String,
    used: &mut usize,
    base: &mut usize,
    word: &str,
    room: usize,
    lines: &mut Vec<String>,
) {
    for ch in word.chars() {
        let piece_w = text_width(&ch.to_string()).max(1);
        if *used + piece_w > room && *used != *base {
            lines.push(std::mem::take(line));
            *line = " ".repeat(*base);
            *used = *base;
        }
        line.push(ch);
        *used += piece_w;
    }
}

/// The column of the hanging indent of a value.
///
/// The indent is the text before the first run of three spaces, so a value
/// with a gap hangs its continuation under the text that follows the gap.
fn hang_column(text: &str, room: usize) -> usize {
    let Some(at) = text.find("   ") else {
        return 0;
    };
    let rest = text[at..].trim_start_matches(' ');
    if rest.is_empty() {
        return 0;
    }
    let column = text_width(&text[..text.len() - rest.len()]);
    if column * 2 > room { 0 } else { column }
}

/// The hints of the page, cut to the room the hint bar has.
///
/// The bar keeps the pinned keys and drops the rest from the end, and marks
/// the cut with an ellipsis.
fn fit_hints<'a>(
    hints: &[FilterHint<'a>],
    width: u16,
) -> (Vec<FilterHint<'a>>, String) {
    let room = usize::from(width.saturating_sub(1));
    let cost = |keep: &[usize]| -> usize {
        let mut total = 0usize;
        for &index in keep {
            total += hint_cost(&hints[index]);
        }
        total
    };
    let mut keep: Vec<usize> = (0..hints.len()).collect();
    if cost(&keep) <= room {
        return (kept_hints(hints, &keep), String::new());
    }
    let mut order: Vec<usize> = (0..hints.len())
        .rev()
        .filter(|&index| !hint_pinned(&hints[index].key))
        .collect();
    order.extend((0..hints.len()).rev().filter(|&index| hint_pinned(&hints[index].key)));
    for gone in order {
        if cost(&keep) + 3 <= room {
            break;
        }
        keep.retain(|&index| index != gone);
    }
    let cut = keep.len() < hints.len();
    (
        kept_hints(hints, &keep),
        if cut { "…".to_owned() } else { String::new() },
    )
}

/// The hints the bar keeps.
fn kept_hints<'a>(hints: &[FilterHint<'a>], keep: &[usize]) -> Vec<FilterHint<'a>> {
    keep.iter()
        .map(|&index| FilterHint {
            key: hints[index].key,
            action: hints[index].action,
        })
        .collect()
}

/// The columns one hint takes, with the gap behind it.
fn hint_cost(hint: &FilterHint<'_>) -> usize {
    text_width(hint.key) + 1 + text_width(hint.action) + 2
}

/// The keys the hint bar never drops.
fn hint_pinned(key: &str) -> bool {
    matches!(key, "Esc" | "?" | "F1")
}
