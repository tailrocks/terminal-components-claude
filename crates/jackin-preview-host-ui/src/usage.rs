//! Read-only usage route state and screen.
//!
//! The screen is one framed [`Panel`]. The list and the detail column are the
//! same projection at every size: a narrow body shows one column, a wide body
//! shows both. Nothing here is selected by terminal size or scenario id.

use ratatui::layout::Rect;
use termrock::author::{FgStep, Modifier, Part, Role, StateFlags, StylePatch, Ui};
use termrock::{
    Chord, Empty, EmptyState, Hint, HintKey, HintLayer, Id, ItemKey, KeyCode, List, ListState,
    Meter, MeterTone, MeterVisual, Panel, PanelKind, Props, PropsRow, RowUi, truncate, width,
};

use jackin_preview_domain::account::{Account, IssueCode, Lifecycle};
use jackin_preview_domain::agent::UsageSurface;
use jackin_preview_domain::usage::{
    Freshness, HealthWord, OverallSummary, QuotaStatus, QuotaWindow,
};
use jackin_preview_sim::world::World;

use crate::accounts::{meter_detail, refreshing_count, spinner_frame};

/// Usage root.
pub const ROOT: Id = Id::root("jackin.usage");
/// Usage tab strip.
pub const TABS: Id = ROOT.sub("tabs");
/// Usage panel.
pub const PANEL: Id = ROOT.sub("panel");
/// Usage list.
pub const LIST: Id = ROOT.sub("list");
/// Read-only detail column.
pub const DETAIL: Id = ROOT.sub("detail");
/// Detail meter.
const DETAIL_METER: Id = DETAIL.sub("meter");
/// Close action for the detail overlay.
pub const CLOSE: Id = DETAIL.sub("close");
/// Handoff action from usage detail to Accounts.
pub const MANAGE: Id = DETAIL.sub("manage");

/// Usage tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    /// Account health overview.
    #[default]
    Overview,
    /// Registration details.
    Registration,
    /// Provider quota.
    Quota,
}

impl Tab {
    /// Ordered tabs used by keyboard and click navigation.
    pub const ALL: [Self; 3] = [Self::Overview, Self::Registration, Self::Quota];

    /// Next tab, wrapping at the end of the strip.
    pub const fn next(self) -> Self {
        match self {
            Self::Overview => Self::Registration,
            Self::Registration => Self::Quota,
            Self::Quota => Self::Overview,
        }
    }

    /// Previous tab, wrapping at the beginning of the strip.
    pub const fn previous(self) -> Self {
        match self {
            Self::Overview => Self::Quota,
            Self::Registration => Self::Overview,
            Self::Quota => Self::Registration,
        }
    }
}

/// Read-only usage state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UsageState {
    /// Active usage tab.
    pub tab: Tab,
    /// Stable account id selected for the read-only detail projection.
    selected: Option<String>,
    /// Whether the detail overlay is visible.
    detail_open: bool,
}

impl UsageState {
    /// Select an account without copying or exposing usage material.
    pub fn select(&mut self, id: Option<impl Into<String>>) {
        self.selected = id.map(Into::into).filter(|id| !id.is_empty());
        if self.selected.is_none() {
            self.detail_open = false;
        }
    }

    /// Selected account id, if the list has one.
    pub fn selected(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    /// Move to the next read-only tab.
    pub const fn next_tab(&mut self) {
        self.tab = self.tab.next();
    }

    /// Move to the previous read-only tab.
    pub const fn previous_tab(&mut self) {
        self.tab = self.tab.previous();
    }

    /// Open the detail overlay only when an account is selected.
    pub const fn open_detail(&mut self) -> bool {
        if self.selected.is_some() {
            self.detail_open = true;
        }
        self.detail_open
    }

    /// Close the detail overlay while retaining list selection.
    pub const fn close_detail(&mut self) {
        self.detail_open = false;
    }

    /// Whether the read-only detail overlay is visible.
    pub const fn detail_open(&self) -> bool {
        self.detail_open
    }

    /// Return the selected account for the Accounts handoff.
    pub fn manage_target(&self) -> Option<&str> {
        self.selected()
    }

    /// Restore the overview list and close any detail projection.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Menubar breadcrumb for the usage route (`Usage › {tab}`,
/// or `Usage › {provider} › {account}` while the detail is open).
pub fn crumb(world: &World, state: &UsageState) -> String {
    if state.detail_open()
        && let Some(account) = state.selected().and_then(|id| world.accounts.get(id))
    {
        return format!(
            "Usage › {} › {}",
            account.surface.surface_name(),
            account.display_name
        );
    }
    match state.tab {
        Tab::Overview => "Usage › Overview".into(),
        Tab::Registration => "Usage › Registration".into(),
        Tab::Quota => "Usage › Quota".into(),
    }
}

const FAINT_DETAIL: [(Part, StylePatch); 1] = [(
    Part::DETAIL,
    StylePatch::new().set_fg(Role::Fg(FgStep::Faint)),
)];

#[derive(Clone, PartialEq, Eq)]
enum Row {
    Overview,
    Heading(UsageSurface),
    Account(String),
}

enum Line {
    Text(String, Role),
    Meter {
        label: String,
        rest: String,
        pct: u8,
        tone: MeterTone,
    },
}

/// Read-only usage screen: one panel, list on the left, detail on the right.
pub struct UsageScreen;

impl UsageScreen {
    /// Draw the usage body. `focused` is the route's focus, not a size switch.
    pub fn draw(ui: &mut Ui<'_>, area: Rect, state: &UsageState, world: &World, focused: bool) {
        let rows = build_rows(world);
        let refreshing = refreshing_count(world);
        let meta = if refreshing > 0 {
            format!(
                "{} refreshing {refreshing}",
                spinner_frame(world.now_ms() as u64 / 80)
            )
        } else {
            format!("broker · {}", world.clock.ago(world.last_refresh_secs))
        };
        let cursor = cursor_index(&rows, state.selected());
        Panel::new(PANEL)
            .kind(PanelKind::Framed)
            .title("Usage · read-only")
            .meta(&meta)
            .focused(focused)
            .slot(Part::GUTTER, &|_ui, _cell| {})
            .patch_part(&FAINT_DETAIL)
            .draw(ui, area, |ui, inner| {
                if world.accounts.accounts.is_empty() {
                    Empty::new(
                        LIST,
                        EmptyState::Empty {
                            title: "No providers configured.",
                            hint: Some("Press R to refresh. · c registers an account"),
                        },
                    )
                    .draw(ui, inner);
                    return;
                }
                if area.width < 100 {
                    if state.detail_open() {
                        draw_detail(ui, inner, state, world);
                    } else {
                        draw_list(ui, inner, world, &rows, cursor, focused);
                    }
                    return;
                }
                let list_w = u16::try_from(u32::from(inner.width) * 34 / 100)
                    .unwrap_or(40)
                    .clamp(28, 40);
                let list = Rect::new(inner.x, inner.y, list_w, inner.height);
                let detail_x = inner.x.saturating_add(list_w).saturating_add(2);
                let detail = Rect::new(
                    detail_x,
                    inner.y,
                    inner.right().saturating_sub(detail_x),
                    inner.height,
                );
                draw_list(ui, list, world, &rows, cursor, focused);
                draw_detail(ui, detail, state, world);
            });
    }

    /// Footer hints for the list, or for the detail column when it is open.
    pub fn hints(detail_open: bool) -> HintLayer {
        let key = |ch: char| HintKey::Chord(Chord::key(KeyCode::Char(ch)));
        let hints = if detail_open {
            vec![
                Hint {
                    key: HintKey::Label("↑↓"),
                    label: "Scroll",
                    priority: 100,
                },
                Hint {
                    key: key('r'),
                    label: "Refresh",
                    priority: 90,
                },
                Hint {
                    key: key('m'),
                    label: "Manage in Accounts",
                    priority: 80,
                },
                Hint {
                    key: HintKey::Chord(Chord::key(KeyCode::Esc)),
                    label: "Back to list",
                    priority: 70,
                },
            ]
        } else {
            vec![
                Hint {
                    key: HintKey::Label("↑↓"),
                    label: "Move",
                    priority: 100,
                },
                Hint {
                    key: HintKey::Chord(Chord::key(KeyCode::Enter)),
                    label: "Detail",
                    priority: 90,
                },
                Hint {
                    key: key('r'),
                    label: "Refresh",
                    priority: 80,
                },
                Hint {
                    key: key('m'),
                    label: "Manage in Accounts",
                    priority: 70,
                },
                Hint {
                    key: HintKey::Chord(Chord::key(KeyCode::Esc)),
                    label: "Close",
                    priority: 60,
                },
            ]
        };
        HintLayer {
            hints,
            badge: None,
            status: None,
            centered: true,
        }
    }
}

fn build_rows(world: &World) -> Vec<Row> {
    let mut rows = vec![Row::Overview];
    for surface in UsageSurface::ALL {
        let accounts: Vec<&Account> = world
            .accounts
            .sorted()
            .into_iter()
            .filter(|account| account.surface == surface)
            .collect();
        if accounts.is_empty() && surface != UsageSurface::Unsupported {
            continue;
        }
        rows.push(Row::Heading(surface));
        for account in accounts {
            rows.push(Row::Account(account.id.clone()));
        }
    }
    rows
}

fn cursor_index(rows: &[Row], selected: Option<&str>) -> usize {
    match selected {
        Some(id) => rows
            .iter()
            .position(|row| matches!(row, Row::Account(account) if account == id))
            .unwrap_or(0),
        None => 0,
    }
}

const ROW_BOLD: StylePatch = StylePatch::new().add(Modifier::BOLD);
const ROW_FAINT: StylePatch = StylePatch::new()
    .set_fg(Role::Fg(FgStep::Faint))
    .remove(Modifier::BOLD);
const ROW_DANGER: StylePatch = StylePatch::new().set_fg(Role::Danger);
const ROW_WARNING: StylePatch = StylePatch::new().set_fg(Role::Warning);

fn draw_list(
    ui: &mut Ui<'_>,
    area: Rect,
    world: &World,
    rows: &[Row],
    cursor: usize,
    focused: bool,
) {
    if area.is_empty() || rows.is_empty() {
        return;
    }
    let items = usage_items(world, rows, area);
    let mut state = ListState::default();
    let key = ItemKey::index(cursor);
    state.set_cursor(cursor, key);
    state.choose(Some(key));
    List::new(LIST)
        .focused(focused)
        .bare_item(&is_heading)
        .row(paint_usage_row)
        .draw(ui, area, &state, &items);
}

fn is_heading(item: &UsageItem) -> bool {
    matches!(item.kind, ItemKind::Heading)
}

fn usage_items(world: &World, rows: &[Row], area: Rect) -> Vec<UsageItem> {
    let overflows = rows.len() > usize::from(area.height);
    let row_w = area.width.saturating_sub(u16::from(overflows));
    rows.iter()
        .map(|row| UsageItem::from_row(row, world, row_w))
        .collect()
}

struct UsageItem {
    kind: ItemKind,
    label: String,
    meta: String,
    health: &'static str,
    faint: bool,
}

enum ItemKind {
    Overview,
    Heading,
    Account,
}

impl UsageItem {
    fn from_row(row: &Row, world: &World, row_w: u16) -> Self {
        match row {
            Row::Overview => Self::plain(ItemKind::Overview, "Overview"),
            Row::Heading(surface) => Self::plain(ItemKind::Heading, surface.label()),
            Row::Account(id) => world
                .accounts
                .get(id)
                .map(|account| account_item(account, world, row_w))
                .unwrap_or_else(|| Self::plain(ItemKind::Account, "")),
        }
    }

    fn plain(kind: ItemKind, label: &str) -> Self {
        Self {
            kind,
            label: label.to_owned(),
            meta: String::new(),
            health: "",
            faint: false,
        }
    }
}

fn account_item(account: &Account, world: &World, row_w: u16) -> UsageItem {
    let mut label = account.display_name.clone();
    if account.default_for_provider {
        label.push_str("  ★");
    }
    let health = if !account.enabled {
        ""
    } else if account.is_error_state()
        || matches!(account.usage.worst_status(), Some(QuotaStatus::Exhausted))
    {
        "!"
    } else if matches!(account.usage.worst_status(), Some(QuotaStatus::Warning))
        || account.usage.freshness.phase == Freshness::Stale
    {
        "▲"
    } else {
        ""
    };
    let meta = if account.usage.freshness.phase == Freshness::Refreshing {
        spinner_frame(world.now_ms() as u64 / 80).to_owned()
    } else if !account.enabled {
        "disabled".to_owned()
    } else if account.lifecycle != Lifecycle::Available {
        account.lifecycle.label().to_owned()
    } else {
        String::new()
    };
    let faint = !account.enabled;
    let reserved = width(&meta).saturating_add(if health.is_empty() { 0 } else { 2 });
    let label_w = row_w
        .saturating_sub(5)
        .saturating_sub(reserved)
        .saturating_sub(2);
    if faint {
        label = fit(&label, label_w);
    }
    UsageItem {
        kind: ItemKind::Account,
        label,
        meta,
        health,
        faint,
    }
}

fn paint_usage_row(item: &UsageItem, ui: &mut RowUi<'_>) {
    match item.kind {
        ItemKind::Heading => {
            ui.indent(1);
            ui.label_patched(&item.label, &ROW_FAINT);
        }
        ItemKind::Overview => paint_named(ui, &item.label, false),
        ItemKind::Account => {
            if !item.meta.is_empty() {
                ui.meta(&item.meta);
            }
            if !item.health.is_empty() {
                let patch = if item.health == "!" {
                    &ROW_DANGER
                } else {
                    &ROW_WARNING
                };
                if item.meta.is_empty() {
                    let _ = ui.part(Part::META, 0);
                }
                ui.trailing(item.health, patch);
            }
            ui.indent(1);
            paint_named(ui, &item.label, item.faint);
        }
    }
}

fn paint_named(ui: &mut RowUi<'_>, label: &str, faint: bool) {
    if faint {
        ui.label_patched(label, &ROW_FAINT);
    } else if ui.flags().contains(StateFlags::FOCUSED) {
        ui.label_patched(label, &ROW_BOLD);
    } else {
        ui.label(label);
    }
}

fn draw_detail(ui: &mut Ui<'_>, area: Rect, state: &UsageState, world: &World) {
    if area.is_empty() {
        return;
    }
    let account = state.selected().and_then(|id| world.accounts.get(id));
    let (title, meta, lines) = match account {
        None => {
            let summary = OverallSummary::compute(&world.accounts.accounts);
            let meta = format!(
                "{} · {}",
                counted(summary.counts.accounts, "account", "accounts"),
                counted(summary.counts.providers, "provider", "providers")
            );
            ("Overview".to_owned(), meta, overview_lines(world))
        }
        Some(account) => (
            account.title(),
            account.status_word().to_owned(),
            account_lines(account, world),
        ),
    };
    draw_title(
        ui,
        Rect::new(area.x, area.y, area.width, 1.min(area.height)),
        &title,
        &meta,
    );
    let body = Rect::new(
        area.x,
        area.y.saturating_add(2),
        area.width,
        area.height.saturating_sub(2),
    );
    draw_detail_body(ui, body, &lines);
}

fn draw_title(ui: &mut Ui<'_>, area: Rect, title: &str, meta: &str) {
    if area.is_empty() {
        return;
    }
    let room = area
        .width
        .saturating_sub(width(meta).saturating_add(2))
        .max(1);
    let title = truncate(title, room);
    let item = [()];
    List::new(DETAIL.sub("title"))
        .bare(true)
        .focused(false)
        .row(move |_: &(), row: &mut RowUi<'_>| {
            row.trailing(meta, &ROW_FAINT);
            row.label_patched(&title, &ROW_BOLD);
        })
        .draw(ui, area, &ListState::default(), &item);
}

fn draw_detail_body(ui: &mut Ui<'_>, area: Rect, lines: &[Line]) {
    if area.is_empty() {
        return;
    }
    let mut y = area.y;
    let mut batch: Vec<(String, Role)> = Vec::new();
    let flush = |ui: &mut Ui<'_>, y: &mut u16, batch: &mut Vec<(String, Role)>| {
        if batch.is_empty() || *y >= area.bottom() {
            batch.clear();
            return;
        }
        let height = u16::try_from(batch.len())
            .unwrap_or(u16::MAX)
            .min(area.bottom().saturating_sub(*y));
        draw_props(ui, Rect::new(area.x, *y, area.width, height), batch);
        *y = y.saturating_add(height);
        batch.clear();
    };
    for (index, line) in lines.iter().enumerate() {
        if y >= area.bottom() {
            break;
        }
        match line {
            Line::Text(text, role) => batch.push((truncate(text, area.width), *role)),
            Line::Meter {
                label,
                rest,
                pct,
                tone,
            } => {
                flush(ui, &mut y, &mut batch);
                if y >= area.bottom() {
                    break;
                }
                draw_meter(ui, area, y, index, label, rest, *pct, *tone);
                y = y.saturating_add(1);
            }
        }
    }
    flush(ui, &mut y, &mut batch);
}

fn draw_props(ui: &mut Ui<'_>, area: Rect, rows: &[(String, Role)]) {
    if area.is_empty() || rows.is_empty() {
        return;
    }
    let props: Vec<PropsRow<'_>> = rows
        .iter()
        .enumerate()
        .map(|(index, (text, tone))| PropsRow::new(ItemKey::index(index), "", text).tone(*tone))
        .collect();
    // Props values start two columns in. Shift the area left by that gap so
    // the text lands on the detail column's own origin.
    let origin = Rect::new(
        area.x.saturating_sub(2),
        area.y,
        area.width.saturating_add(2),
        area.height,
    );
    Props::rich(&props).draw(ui, origin);
}

fn draw_meter(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    index: usize,
    label: &str,
    rest: &str,
    pct: u8,
    tone: MeterTone,
) {
    let row = Rect::new(area.x, y, area.width, 1);
    let item = [()];
    List::new(DETAIL.item(ItemKey::index(index)))
        .bare(true)
        .focused(false)
        .row(|_: &(), row_ui: &mut RowUi<'_>| {
            row_ui.meta(rest);
            row_ui.label(label);
        })
        .draw(ui, row, &ListState::default(), &item);
    let meter_w: u16 = if area.width >= 70 { 24 } else { 16 };
    let mx = area.x.saturating_add(16);
    if mx < area.right() {
        let track = meter_w
            .saturating_add(6)
            .min(area.right().saturating_sub(mx));
        let pct_text = format!("{pct:>3}%");
        Meter::new(DETAIL_METER.item(ItemKey::index(index)))
            .ratio(f64::from(pct) / 100.0)
            .value(&pct_text)
            .tone(tone)
            .visual(MeterVisual::Block)
            .draw(ui, Rect::new(mx, y, track, 1));
    }
}

fn overview_lines(world: &World) -> Vec<Line> {
    let summary = OverallSummary::compute(&world.accounts.accounts);
    let health_tone = match summary.health {
        HealthWord::Degraded | HealthWord::Blocked => Role::Danger,
        HealthWord::Attention => Role::Warning,
        _ => Role::Fg(FgStep::Primary),
    };
    let mut lines = vec![
        Line::Text(
            format!(
                "Health       {} · {}",
                summary.health.label(),
                summary.issues_line()
            ),
            health_tone,
        ),
        Line::Text(
            format!(
                "Freshness    {} of {} current · broker projection {}",
                summary
                    .counts
                    .enabled
                    .saturating_sub(summary.counts.stale)
                    .saturating_sub(summary.counts.failed)
                    .saturating_sub(summary.counts.refreshing),
                summary.counts.enabled,
                world.clock.ago(world.last_refresh_secs)
            ),
            Role::Fg(FgStep::Primary),
        ),
        Line::Text(
            format!("Registry     {}", summary.counts_line()),
            Role::Fg(FgStep::Primary),
        ),
        Line::Text(String::new(), Role::Fg(FgStep::Primary)),
        Line::Text(
            format!(
                "{:<12} {:<9} {:<26} {}",
                "Provider", "Accounts", "Worst window", "Status"
            ),
            Role::Fg(FgStep::Muted),
        ),
    ];
    for surface in UsageSurface::ALL {
        lines.push(provider_line(world, surface));
    }
    let unresolved: Vec<&Account> = world
        .accounts
        .accounts
        .iter()
        .filter(|account| account.enabled && account.identity.subject.is_none())
        .collect();
    if !unresolved.is_empty() {
        lines.push(Line::Text(String::new(), Role::Fg(FgStep::Primary)));
        for account in unresolved {
            lines.push(Line::Text(
                format!(
                    "Unresolved   {} · {} ({}) — not an authenticated account",
                    account.surface.label(),
                    account.source.safe_detail(),
                    account.confidence.label()
                ),
                Role::Fg(FgStep::Muted),
            ));
        }
    }
    lines.push(Line::Text(String::new(), Role::Fg(FgStep::Primary)));
    lines.push(Line::Text(
        "Rollups sum identical windows and units only; a provider with mixed windows shows its worst window.".into(),
        Role::Fg(FgStep::Faint),
    ));
    if !summary.comparable.is_empty() {
        lines.push(Line::Text(String::new(), Role::Fg(FgStep::Primary)));
        for rollup in &summary.comparable {
            lines.push(Line::Text(
                format!(
                    "Comparable   {} · {} · {} · {}–{}% remaining",
                    rollup.surface.label(),
                    rollup.label,
                    counted(rollup.accounts, "account", "accounts"),
                    rollup.min_remaining_pct,
                    rollup.max_remaining_pct
                ),
                Role::Fg(FgStep::Secondary),
            ));
        }
    }
    lines
}

fn provider_line(world: &World, surface: UsageSurface) -> Line {
    if surface == UsageSurface::Unsupported {
        return Line::Text(
            format!(
                "{:<12} {:<9} {:<26} {}",
                "—", "—", "—", "unsupported sentinel"
            ),
            Role::Fg(FgStep::Muted),
        );
    }
    let accounts: Vec<&Account> = world
        .accounts
        .accounts
        .iter()
        .filter(|account| account.surface == surface && account.enabled)
        .collect();
    if accounts.is_empty() {
        return Line::Text(
            format!(
                "{:<12} {:<9} {:<26} {}",
                surface.label(),
                "—",
                "—",
                "not discovered"
            ),
            Role::Fg(FgStep::Faint),
        );
    }
    let worst = accounts
        .iter()
        .flat_map(|account| account.usage.windows.iter().map(move |win| (account, win)))
        .filter(|(_, win)| win.used_pct.is_some())
        .max_by_key(|(_, win)| win.used_pct.unwrap_or(0));
    let (window, status, tone) = match worst {
        Some((account, win)) => {
            let (status, tone) = match (win.status, account.usage.freshness.phase) {
                (QuotaStatus::Exhausted, _) => ("! exhausted".to_owned(), Role::Danger),
                (_, Freshness::Failed) => (
                    format!(
                        "! {}",
                        account
                            .issue
                            .as_ref()
                            .map(|issue| {
                                issue
                                    .message
                                    .split(':')
                                    .next()
                                    .unwrap_or("error")
                                    .to_lowercase()
                            })
                            .unwrap_or_else(|| "error".into())
                    ),
                    Role::Danger,
                ),
                (QuotaStatus::Warning, _) => ("▲ warning".to_owned(), Role::Warning),
                (_, Freshness::Stale) => ("▲ stale".to_owned(), Role::Warning),
                _ => ("current".to_owned(), Role::Fg(FgStep::Primary)),
            };
            (
                format!("{} {}%", win.label, win.used_pct.unwrap_or(0)),
                status,
                tone,
            )
        }
        None => (
            "—".to_owned(),
            accounts[0].status_word().to_owned(),
            Role::Fg(FgStep::Muted),
        ),
    };
    Line::Text(
        format!(
            "{:<12} {:<9} {:<26} {}",
            surface.label(),
            accounts.len(),
            truncate(&window, 26),
            status
        ),
        tone,
    )
}

fn account_lines(account: &Account, world: &World) -> Vec<Line> {
    let mut lines = vec![
        Line::Text(
            format!(
                "Provider     {} · surface {}",
                account.provider.label(),
                account.surface.surface_name()
            ),
            Role::Fg(FgStep::Primary),
        ),
        Line::Text(
            format!(
                "Account      {} · {}",
                account.identity.label(),
                account
                    .identity
                    .plan
                    .clone()
                    .unwrap_or_else(|| "plan unknown".into())
            ),
            if account.identity.subject.is_some() {
                Role::Fg(FgStep::Primary)
            } else {
                Role::Fg(FgStep::Muted)
            },
        ),
        Line::Text(
            format!(
                "Credential   {} · {}",
                account.source.origin_label(),
                account.source.safe_detail()
            ),
            Role::Fg(FgStep::Primary),
        ),
    ];
    let (status, tone) = match account.usage.freshness.phase {
        Freshness::Current => (
            format!(
                "{} · current · refreshed {}",
                account.lifecycle.label(),
                account
                    .last_refresh_secs
                    .map(|secs| world.clock.ago(secs))
                    .unwrap_or_else(|| "never".into())
            ),
            Role::Fg(FgStep::Primary),
        ),
        Freshness::Stale => (
            format!(
                "{} · stale · last good {}",
                account.lifecycle.label(),
                account
                    .usage
                    .freshness
                    .last_good_secs
                    .map(|secs| world.clock.ago(secs))
                    .unwrap_or_else(|| "?".into())
            ),
            Role::Warning,
        ),
        Freshness::Refreshing => (
            format!("{} · refreshing…", account.lifecycle.label()),
            Role::Fg(FgStep::Primary),
        ),
        Freshness::Failed => (
            format!(
                "{} · error: {}",
                account.lifecycle.label(),
                account
                    .issue
                    .as_ref()
                    .map(|issue| issue.message.clone())
                    .unwrap_or_else(|| "refresh failed".into())
            ),
            Role::Danger,
        ),
    };
    lines.push(Line::Text(format!("Status       {status}"), tone));
    lines.push(Line::Text(String::new(), Role::Fg(FgStep::Primary)));
    lines.push(Line::Text("Limits".into(), Role::Fg(FgStep::Secondary)));
    if account.usage.windows.is_empty() {
        let text = match account.lifecycle {
            Lifecycle::NeedsLogin => "  needs login · no quota until the agent signs in",
            Lifecycle::NeedsSecret => "  needs secret · no quota until a key is present",
            Lifecycle::Unsupported => "  unsupported · this provider exposes no usage",
            _ => "  not started",
        };
        lines.push(Line::Text(text.into(), Role::Fg(FgStep::Muted)));
    }
    for win in &account.usage.windows {
        lines.push(window_line(account, win, world));
    }
    if account.usage.freshness.phase != Freshness::Current && !account.usage.windows.is_empty() {
        lines.push(Line::Text(String::new(), Role::Fg(FgStep::Primary)));
        let quota = account
            .issue
            .as_ref()
            .filter(|issue| issue.code == IssueCode::QuotaUnsupported)
            .map(|_| " · quota not visible")
            .unwrap_or("");
        lines.push(Line::Text(
            format!(
                "Last good    kept from {}{quota}",
                account
                    .usage
                    .freshness
                    .last_good_secs
                    .map(|secs| world.clock.ago(secs))
                    .unwrap_or_else(|| "?".into())
            ),
            Role::Fg(FgStep::Muted),
        ));
    }
    lines
}

fn window_line(account: &Account, win: &QuotaWindow, world: &World) -> Line {
    if win.has_meter() {
        let tone = match account.usage.freshness.phase {
            Freshness::Refreshing => MeterTone::Medium,
            Freshness::Stale | Freshness::Failed => MeterTone::Stale,
            Freshness::Current => match win.status {
                QuotaStatus::Exhausted => MeterTone::High,
                QuotaStatus::Warning => MeterTone::Medium,
                _ => MeterTone::Low,
            },
        };
        Line::Meter {
            label: format!("  {}", win.label),
            rest: meter_detail(win, world),
            pct: win.used_pct.unwrap_or(0),
            tone,
        }
    } else {
        let value = win.value_label();
        let text = if value.to_lowercase().starts_with(&win.label.to_lowercase()) {
            format!("  {value}")
        } else {
            format!("  {}   {value}", win.label)
        };
        let tone = if win.status == QuotaStatus::Error {
            Role::Danger
        } else {
            Role::Fg(FgStep::Muted)
        };
        Line::Text(text, tone)
    }
}

fn counted(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

fn fit(text: &str, columns: u16) -> String {
    let shown = truncate(text, columns);
    let pad = columns.saturating_sub(width(&shown));
    format!("{shown}{}", " ".repeat(usize::from(pad)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_detail_is_read_only_and_requires_selection() {
        let mut state = UsageState::default();
        assert!(!state.open_detail());
        state.select(Some("acct-claude"));
        assert!(state.open_detail());
        assert_eq!(state.manage_target(), Some("acct-claude"));
        state.close_detail();
        assert_eq!(state.selected(), Some("acct-claude"));
        assert!(!state.detail_open());
    }

    #[test]
    fn usage_tabs_wrap_without_mutating_selection() {
        let mut state = UsageState::default();
        state.select(Some("acct"));
        state.previous_tab();
        assert_eq!(state.tab, Tab::Quota);
        state.next_tab();
        assert_eq!(state.tab, Tab::Overview);
        assert_eq!(state.selected(), Some("acct"));
    }
}
