//! Accounts route state and screen.
//!
//! The accounts route owns tree selection semantics while the shared
//! [`Panel`], [`SplitPane`], [`Meter`] and [`Button`] components own focus and
//! painting. Row construction, inspector lines, crumbs and hints are ported
//! from the frozen `screens/accounts.rs` screen so the live tree, inspector,
//! header crumb and footer match that authority.

use std::collections::HashSet;

use ratatui::layout::Rect;
use termrock::author::{
    Family, FgStep, Focusability, Id, Modifier, PaintStyle, Part, Role, StateFlags, StyleDefaults,
    StylePatch, Surface, Ui, Variant,
};
use termrock::{
    Button, Chord, Empty, EmptyState, Hint, HintKey, HintLayer, Insets, KeyCode, ListState, Meter,
    MeterTone, MeterVisual, Panel, PanelKind, ScrollState, SplitAxis, SplitPane, SplitPaneState,
    TextInputState, truncate, width, wrap,
};

use jackin_preview_domain::account::{
    Account, AccountOrigin, CredentialSource, IssueCode, Lifecycle, ValidationLevel,
    ValidationState, masked,
};
use jackin_preview_domain::agent::{Provider, UsageSurface};
use jackin_preview_domain::onepassword::OpReference;
use jackin_preview_domain::usage::{
    Freshness, HealthWord, OverallSummary, QuotaStatus, QuotaWindow, WindowUnit,
};
use jackin_preview_sim::onepassword::OpSession;
use jackin_preview_sim::world::World;

use crate::manager::{plural, position_label};

/// Accounts list control.
pub const LIST: Id = Id::root("jackin.accounts.list");
/// Accounts tree control.
pub const TREE: Id = Id::root("jackin.accounts.tree");
/// Accounts inspector control.
pub const INSPECTOR: Id = Id::root("jackin.accounts.inspector");
/// Split seam between tree and inspector.
pub const SEAM: Id = Id::root("jackin.accounts.seam");
/// Shared inspector meter readout.
pub const INSPECTOR_METER: Id = INSPECTOR.sub("meter");
/// Account form root.
pub const FORM: Id = Id::root("jackin.accounts.form");
/// Account form save action.
pub const SAVE: Id = FORM.sub("save");
/// Provider selection control.
pub const PROVIDER: Id = FORM.sub("provider");
/// Credential-source selection control.
pub const SOURCE: Id = FORM.sub("source");
/// 1Password source control.
pub const OP: Id = FORM.sub("op");
/// New-account opener.
pub const START: Id = FORM.sub("start");
/// Display-name field.
pub const NAME: Id = FORM.sub("name");
/// Agent/runtime selection control.
pub const AGENT: Id = FORM.sub("agent");
/// Local-folder field.
pub const FOLDER: Id = FORM.sub("folder");
/// Plain API-key field.
pub const SECRET: Id = FORM.sub("secret");

/// Inspector action buttons, one stable id per action.
pub const ACTION_REFRESH: Id = INSPECTOR.sub("refresh");
pub const ACTION_VALIDATE: Id = INSPECTOR.sub("validate");
pub const ACTION_EDIT: Id = INSPECTOR.sub("edit");
pub const ACTION_DEFAULT: Id = INSPECTOR.sub("default");
pub const ACTION_TOGGLE: Id = INSPECTOR.sub("toggle");
pub const ACTION_REMOVE: Id = INSPECTOR.sub("remove");
pub const ACTION_ADD: Id = INSPECTOR.sub("add");

/// Tree selection: the overview rollup, one provider surface, one account, or
/// the add-account action row.
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub enum AccountSel {
    /// The overview rollup row.
    #[default]
    Overview,
    /// One provider usage surface row.
    Provider(UsageSurface),
    /// One account row.
    Account(String),
    /// The add-account action row.
    Add,
}

/// One tree row: identity plus its display fragments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountRow {
    /// Selection identity of this row.
    pub sel: AccountSel,
    /// Tree depth (provider rows sit at 0, accounts at 1).
    pub depth: u16,
    /// Whether the row carries the provider-default star.
    pub star: bool,
    /// Display label.
    pub label: String,
    /// Trailing health glyph and tone, if any.
    pub health: Option<(&'static str, Role)>,
    /// Right-aligned meta text.
    pub meta: String,
    /// Tone of the meta text.
    pub meta_tone: Role,
    /// Whether the row renders faint (disabled or undiscovered).
    pub faint: bool,
    /// Whether the row can fold its children.
    pub expandable: bool,
    /// Whether an expandable row currently shows its children.
    pub expanded: bool,
}

/// Braille spinner frames shared by the refreshing badge and rows.
pub const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Return the spinner frame for a tick count.
pub const fn spinner_frame(tick: u64) -> &'static str {
    SPINNER[(tick % SPINNER.len() as u64) as usize]
}

/// Build the visible tree rows: Overview, one row per provider surface with
/// its accounts, and the Add row. A filter keeps only matching accounts (plus
/// their provider parents) and hides the Add row.
pub fn build_account_rows(
    world: &World,
    filter: Option<&str>,
    folded: &HashSet<UsageSurface>,
) -> Vec<AccountRow> {
    let mut rows = vec![AccountRow {
        sel: AccountSel::Overview,
        depth: 0,
        star: false,
        label: "Overview".into(),
        health: None,
        meta: String::new(),
        meta_tone: Role::Fg(FgStep::Muted),
        faint: false,
        expandable: false,
        expanded: false,
    }];
    let query = filter
        .map(str::to_lowercase)
        .filter(|query| !query.is_empty());
    for surface in UsageSurface::ALL {
        let accounts: Vec<&Account> = world
            .accounts
            .sorted()
            .into_iter()
            .filter(|account| account.surface == surface)
            .filter(|account| match &query {
                Some(query) => {
                    account.display_name.to_lowercase().contains(query)
                        || account.provider.label().to_lowercase().contains(query)
                        || account.identity.label().to_lowercase().contains(query)
                        || account.status_word().contains(query.as_str())
                        || account
                            .identity
                            .plan
                            .as_ref()
                            .is_some_and(|plan| plan.to_lowercase().contains(query))
                }
                None => true,
            })
            .collect();
        if surface == UsageSurface::Unsupported {
            if query.is_none() {
                rows.push(AccountRow {
                    sel: AccountSel::Provider(surface),
                    depth: 0,
                    star: false,
                    label: "Unsupported".into(),
                    health: None,
                    meta: "sentinel".into(),
                    meta_tone: Role::Fg(FgStep::Faint),
                    faint: true,
                    expandable: false,
                    expanded: false,
                });
            }
            continue;
        }
        if accounts.is_empty()
            && (query.is_some()
                || !surface.provider().is_some_and(|provider| {
                    provider.agent().is_some_and(|agent| agent.registerable())
                }))
        {
            if query.is_none() {
                rows.push(AccountRow {
                    sel: AccountSel::Provider(surface),
                    depth: 0,
                    star: false,
                    label: surface.surface_name().into(),
                    health: None,
                    meta: "not discovered".into(),
                    meta_tone: Role::Fg(FgStep::Faint),
                    faint: true,
                    expandable: false,
                    expanded: false,
                });
            }
            continue;
        }
        let expanded = !folded.contains(&surface);
        let warn = accounts
            .iter()
            .filter(|account| {
                account.enabled
                    && matches!(account.usage.worst_status(), Some(QuotaStatus::Warning))
            })
            .count();
        let err = accounts
            .iter()
            .filter(|account| {
                account.enabled
                    && (account.is_error_state()
                        || matches!(account.usage.worst_status(), Some(QuotaStatus::Exhausted)))
            })
            .count();
        let health = if err > 0 {
            Some(("!", Role::Danger))
        } else if warn > 0 {
            Some(("▲", Role::Warning))
        } else {
            None
        };
        rows.push(AccountRow {
            sel: AccountSel::Provider(surface),
            depth: 0,
            star: false,
            label: surface.surface_name().into(),
            health,
            meta: if accounts.is_empty() {
                "no accounts".into()
            } else {
                accounts.len().to_string()
            },
            meta_tone: Role::Fg(FgStep::Muted),
            faint: accounts.is_empty(),
            expandable: !accounts.is_empty(),
            expanded,
        });
        if expanded {
            for account in accounts {
                let (meta, tone) = account_meta(account, world);
                let health = if !account.enabled {
                    None
                } else if account.is_error_state()
                    || matches!(account.usage.worst_status(), Some(QuotaStatus::Exhausted))
                {
                    Some(("!", Role::Danger))
                } else if matches!(account.usage.worst_status(), Some(QuotaStatus::Warning))
                    || account.usage.freshness.phase == Freshness::Stale
                {
                    Some(("▲", Role::Warning))
                } else {
                    None
                };
                rows.push(AccountRow {
                    sel: AccountSel::Account(account.id.clone()),
                    depth: 1,
                    star: account.default_for_provider,
                    label: if account.origin == AccountOrigin::Discovered
                        && account.display_name != "discovered"
                    {
                        format!("{} · discovered", account.display_name)
                    } else {
                        account.display_name.clone()
                    },
                    health,
                    meta,
                    meta_tone: tone,
                    faint: !account.enabled,
                    expandable: false,
                    expanded: false,
                });
            }
        }
    }
    if query.is_none() {
        rows.push(AccountRow {
            sel: AccountSel::Add,
            depth: 0,
            star: false,
            label: "+ Add account…".into(),
            health: None,
            meta: String::new(),
            meta_tone: Role::Fg(FgStep::Muted),
            faint: false,
            expandable: false,
            expanded: false,
        });
    }
    rows
}

/// Right-hand meta for a tree row: freshness word and age.
pub fn account_meta(account: &Account, world: &World) -> (String, Role) {
    if !account.enabled {
        return ("disabled".into(), Role::Fg(FgStep::Faint));
    }
    match account.usage.freshness.phase {
        Freshness::Refreshing => (
            format!("{} refreshing", spinner_frame(world.now_ms() as u64 / 80)),
            Role::Fg(FgStep::Secondary),
        ),
        Freshness::Stale => (
            format!(
                "stale {}",
                account
                    .usage
                    .freshness
                    .last_good_secs
                    .map(|secs| world.clock.ago(secs).replace(" ago", ""))
                    .unwrap_or("?".into())
            ),
            Role::Warning,
        ),
        Freshness::Failed => (
            match account.issue.as_ref().map(|issue| issue.code) {
                Some(IssueCode::RateLimited) => "rate limited".into(),
                Some(IssueCode::ProviderUnavailable) => "unavailable".into(),
                Some(IssueCode::CredentialFileMissing) => "needs secret".into(),
                Some(IssueCode::Unauthorized) | Some(IssueCode::ApiKeyInvalid) => {
                    "unauthorized".into()
                }
                Some(IssueCode::OpLocked) => "1Password locked".into(),
                _ => "error".into(),
            },
            Role::Danger,
        ),
        Freshness::Current => match account.lifecycle {
            Lifecycle::Unsupported => ("unsupported".into(), Role::Fg(FgStep::Muted)),
            Lifecycle::NeedsLogin => ("needs login".into(), Role::Warning),
            Lifecycle::NeedsSecret => ("needs secret".into(), Role::Warning),
            _ => (
                format!(
                    "current {}",
                    account
                        .last_refresh_secs
                        .map(|secs| world.clock.ago(secs).replace(" ago", ""))
                        .unwrap_or("now".into())
                ),
                Role::Fg(FgStep::Muted),
            ),
        },
    }
}

/// Format a count with thousands separators.
fn thousands(value: usize) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

/// One inspector body line before wrapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InspLine {
    /// Label plus value property row.
    Prop(String, String, Role),
    /// Plain wrapped text row.
    Text(String, Role),
    /// Bold heading with right-aligned faint meta.
    Heading(String, String),
    /// Quota meter row: label, used percent, detail, tone.
    Meter(String, u8, String, MeterTone),
    /// Blank row.
    Blank,
}

/// Quota detail fragments: counted usage, reset label, spend label, and the
/// exhaustion word.
pub fn meter_detail(win: &QuotaWindow, world: &World) -> String {
    let mut parts: Vec<String> = vec![];
    if let (Some(used), Some(limit)) = (win.used, win.limit)
        && win.unit != WindowUnit::Percent
    {
        parts.push(format!(
            "{} / {} {}",
            thousands(used as usize),
            thousands(limit as usize),
            win.unit.label()
        ));
    }
    if let Some(reset) = win.reset_secs {
        parts.push(world.clock.reset_label(reset));
    }
    if let Some(spend) = &win.spend_label {
        parts.push(spend.clone());
    }
    if win.status == QuotaStatus::Exhausted {
        parts.push("exhausted".into());
    }
    parts.join(" · ")
}

/// Meter tone: freshness dominates the window status.
fn meter_tone(win: &QuotaWindow, account: &Account) -> MeterTone {
    match account.usage.freshness.phase {
        Freshness::Refreshing => MeterTone::Medium,
        Freshness::Stale | Freshness::Failed => MeterTone::Stale,
        Freshness::Current => match win.status {
            QuotaStatus::Exhausted => MeterTone::High,
            QuotaStatus::Warning => MeterTone::Medium,
            _ => MeterTone::Low,
        },
    }
}

/// Validation marks: material, identity, quota access.
fn validation_marks(account: &Account) -> (&'static str, &'static str, &'static str) {
    match &account.validation {
        ValidationState::Valid(ValidationLevel::QuotaReadable) => ("✓", "✓", "✓"),
        ValidationState::Valid(ValidationLevel::IdentityAuthenticated) => (
            "✓",
            "✓",
            if account
                .issue
                .as_ref()
                .is_some_and(|issue| issue.code == IssueCode::QuotaUnsupported)
            {
                "—"
            } else {
                "▲"
            },
        ),
        ValidationState::Valid(ValidationLevel::MaterialDiscovered) => ("✓", "—", "—"),
        ValidationState::Invalid(_) => ("!", "—", "—"),
        ValidationState::Validating { .. } => ("⠋", "…", "…"),
        ValidationState::NeverValidated => ("—", "—", "—"),
    }
}

/// Inspector title, scope meta, and unwrapped body lines for a selection.
pub fn inspector_lines(
    world: &World,
    sel: &AccountSel,
    width: u16,
) -> (String, String, Vec<InspLine>) {
    let mut lines: Vec<InspLine> = vec![];
    match sel {
        AccountSel::Overview => {
            let summary = OverallSummary::compute(&world.accounts.accounts);
            lines.push(InspLine::Prop(
                "Health".into(),
                format!(
                    "{} · {}",
                    summary.health.label(),
                    summary.issues_line()
                ),
                match summary.health {
                    HealthWord::Blocked | HealthWord::Degraded => Role::Danger,
                    HealthWord::Attention => Role::Warning,
                    _ => Role::Fg(FgStep::Primary),
                },
            ));
            lines.push(InspLine::Prop(
                "Registry".into(),
                summary.counts_line(),
                Role::Fg(FgStep::Primary),
            ));
            let registered = world
                .accounts
                .accounts
                .iter()
                .filter(|account| account.origin == AccountOrigin::Registered)
                .count();
            lines.push(InspLine::Prop(
                "Sources".into(),
                format!(
                    "{registered} registered · {} discovered · 1Password {}",
                    world.accounts.accounts.len() - registered,
                    match world.op.session {
                        OpSession::SignedIn => "available",
                        OpSession::Locked => "locked",
                    }
                ),
                Role::Fg(FgStep::Primary),
            ));
            lines.push(InspLine::Blank);
            lines.push(InspLine::Heading(
                "Comparable windows".into(),
                "identical provider, window and unit only".into(),
            ));
            if summary.comparable.is_empty() {
                lines.push(InspLine::Text(
                    "Nothing comparable: no provider has two accounts with the same window".into(),
                    Role::Fg(FgStep::Muted),
                ));
            }
            for rollup in &summary.comparable {
                let mut text = format!(
                    "{} · {}   {} · {}–{}% remaining",
                    rollup.surface.label(),
                    rollup.label,
                    plural(rollup.accounts, "account", "accounts"),
                    rollup.min_remaining_pct,
                    rollup.max_remaining_pct
                );
                if let Some((used, limit)) = rollup.summed {
                    text.push_str(&format!(
                        " · {} / {} {}",
                        thousands(used as usize),
                        thousands(limit as usize),
                        rollup.unit.label()
                    ));
                }
                if rollup.last_good_count > 0 {
                    text.push_str(&format!(" ({} last good)", rollup.last_good_count));
                }
                lines.push(InspLine::Text(text, Role::Fg(FgStep::Secondary)));
            }
            if !summary.not_comparable.is_empty() {
                lines.push(InspLine::Blank);
                lines.push(InspLine::Heading("Not comparable".into(), String::new()));
                let names: Vec<String> = summary
                    .not_comparable
                    .iter()
                    .map(|note| format!("{} ({})", note.surface.label(), note.reason))
                    .collect();
                for line in wrap(&names.join(" · "), width) {
                    lines.push(InspLine::Text(line, Role::Fg(FgStep::Muted)));
                }
            }
            lines.push(InspLine::Blank);
            lines.push(InspLine::Heading("Warnings".into(), String::new()));
            let mut any = false;
            for account in world.accounts.sorted() {
                if !account.enabled {
                    continue;
                }
                for win in &account.usage.windows {
                    match win.status {
                        QuotaStatus::Warning => {
                            any = true;
                            lines.push(InspLine::Text(
                                format!(
                                    "▲ {}   {} {}% used{}",
                                    account.title(),
                                    win.label,
                                    win.used_pct.unwrap_or(0),
                                    win.reset_secs
                                        .map(|reset| format!(
                                            " · {}",
                                            world.clock.reset_label(reset)
                                        ))
                                        .unwrap_or_default()
                                ),
                                Role::Warning,
                            ));
                        }
                        QuotaStatus::Exhausted => {
                            any = true;
                            lines.push(InspLine::Text(
                                format!(
                                    "! {}   {} exhausted{}",
                                    account.title(),
                                    win.label,
                                    win.reset_secs
                                        .map(|reset| format!(
                                            " · {}",
                                            world.clock.reset_label(reset)
                                        ))
                                        .unwrap_or_default()
                                ),
                                Role::Danger,
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
                    lines.push(InspLine::Text(
                        format!("! {}   {}", account.title(), issue.message),
                        Role::Danger,
                    ));
                } else if account.usage.freshness.phase == Freshness::Stale {
                    any = true;
                    lines.push(InspLine::Text(
                        format!(
                            "▲ {}   stale · last good {}",
                            account.title(),
                            account
                                .usage
                                .freshness
                                .last_good_secs
                                .map(|secs| world.clock.ago(secs))
                                .unwrap_or("?".into())
                        ),
                        Role::Warning,
                    ));
                }
            }
            if !any {
                lines.push(InspLine::Text("No warnings".into(), Role::Fg(FgStep::Muted)));
            }
            let unresolved: Vec<&Account> = world
                .accounts
                .accounts
                .iter()
                .filter(|account| account.enabled && account.identity.subject.is_none())
                .collect();
            if !unresolved.is_empty() {
                lines.push(InspLine::Blank);
                lines.push(InspLine::Heading(
                    "Unresolved identity".into(),
                    String::new(),
                ));
                for account in unresolved {
                    lines.push(InspLine::Text(
                        format!(
                            "{} · {} · {} — not an authenticated account",
                            account.title(),
                            account.source.origin_label().to_lowercase(),
                            account.confidence.label()
                        ),
                        Role::Fg(FgStep::Muted),
                    ));
                }
            }
            (
                "Overview".into(),
                format!(
                    "{} · {}",
                    plural(world.accounts.accounts.len(), "account", "accounts"),
                    plural(summary.counts.providers, "provider", "providers")
                ),
                lines,
            )
        }
        AccountSel::Provider(surface) => {
            let accounts: Vec<&Account> = world
                .accounts
                .sorted()
                .into_iter()
                .filter(|account| account.surface == *surface)
                .collect();
            let provider = surface.provider();
            lines.push(InspLine::Prop(
                "Provider".into(),
                provider.map(|p| p.label()).unwrap_or("—").into(),
                Role::Fg(FgStep::Primary),
            ));
            lines.push(InspLine::Prop(
                "Agent runtime".into(),
                provider
                    .and_then(|p| p.agent())
                    .map(|agent| agent.label())
                    .unwrap_or("none")
                    .into(),
                Role::Fg(FgStep::Primary),
            ));
            lines.push(InspLine::Prop(
                "Usage surface".into(),
                surface.surface_name().into(),
                Role::Fg(FgStep::Primary),
            ));
            lines.push(InspLine::Prop(
                "Registration".into(),
                if provider
                    .and_then(|p| p.agent())
                    .is_some_and(|agent| agent.registerable())
                {
                    "manual accounts allowed · 1Password, local folder or plain-text key".into()
                } else {
                    "discovered only · read-only projection".into()
                },
                Role::Fg(FgStep::Secondary),
            ));
            if let Some(default) = accounts.iter().find(|account| account.default_for_provider) {
                lines.push(InspLine::Prop(
                    "Default".into(),
                    format!("★ {}", default.display_name),
                    Role::Fg(FgStep::Primary),
                ));
            }
            lines.push(InspLine::Blank);
            if *surface == UsageSurface::Unsupported {
                lines.push(InspLine::Text(
                    "Unsupported is the registry's explicit sentinel: a capability with no provider adapter. It is never synthesized as zero usage.".into(),
                    Role::Fg(FgStep::Muted),
                ));
            } else if accounts.is_empty() {
                lines.push(InspLine::Text(
                    "No accounts for this provider".into(),
                    Role::Fg(FgStep::Muted),
                ));
            }
            for account in accounts {
                lines.push(InspLine::Heading(
                    account.display_name.clone(),
                    account_meta(account, world).0,
                ));
                for win in account.usage.windows.iter().take(2) {
                    if win.has_meter() {
                        lines.push(InspLine::Meter(
                            win.label.clone(),
                            win.used_pct.unwrap_or(0),
                            meter_detail(win, world),
                            meter_tone(win, account),
                        ));
                    } else {
                        lines.push(InspLine::Text(
                            format!("{}   {}", win.label, win.value_label()),
                            Role::Fg(FgStep::Muted),
                        ));
                    }
                }
            }
            (
                surface.surface_name().into(),
                format!(
                    "provider · {}",
                    plural(
                        world
                            .accounts
                            .by_provider(provider.unwrap_or(Provider::Anthropic))
                            .count(),
                        "account",
                        "accounts"
                    )
                ),
                lines,
            )
        }
        AccountSel::Account(id) => {
            let Some(account) = world.accounts.get(id) else {
                return ("Account".into(), String::new(), lines);
            };
            let ws_users: Vec<String> = world
                .workspaces
                .iter()
                .filter(|workspace| {
                    workspace
                        .effective_accounts(&world.accounts)
                        .iter()
                        .any(|entry| &entry.id == id)
                })
                .map(|workspace| workspace.name.clone())
                .collect();
            lines.push(InspLine::Prop(
                "Provider".into(),
                account.provider.label().into(),
                Role::Fg(FgStep::Primary),
            ));
            lines.push(InspLine::Prop(
                "Agent runtime".into(),
                account
                    .agent
                    .map(|agent| agent.label())
                    .unwrap_or("none")
                    .into(),
                Role::Fg(FgStep::Primary),
            ));
            lines.push(InspLine::Prop(
                "Usage surface".into(),
                account.surface.surface_name().into(),
                Role::Fg(FgStep::Primary),
            ));
            lines.push(InspLine::Prop(
                "Identity".into(),
                account.identity.label(),
                if account.identity.subject.is_some() {
                    Role::Fg(FgStep::Primary)
                } else {
                    Role::Fg(FgStep::Muted)
                },
            ));
            lines.push(InspLine::Prop(
                "Plan".into(),
                account.identity.plan.clone().unwrap_or("unknown".into()),
                Role::Fg(FgStep::Primary),
            ));
            lines.push(InspLine::Prop(
                "Credential".into(),
                format!(
                    "{} · {}",
                    account.source.origin_label(),
                    account.source.safe_detail()
                ),
                Role::Fg(FgStep::Primary),
            ));
            match &account.source {
                CredentialSource::OnePassword(reference) => {
                    lines.push(InspLine::Prop(
                        String::new(),
                        format!(
                            "{} · {} · {}",
                            reference.account,
                            reference.canonical(),
                            world
                                .op
                                .describe(reference)
                                .map(|descriptor| descriptor.masked)
                                .unwrap_or("••••••••".into())
                        ),
                        Role::Fg(FgStep::Muted),
                    ));
                }
                CredentialSource::PlainApiKey { tail, .. } => lines.push(InspLine::Prop(
                    String::new(),
                    masked(tail),
                    Role::Fg(FgStep::Muted),
                )),
                CredentialSource::LocalFolder { detected, .. }
                | CredentialSource::HostEnv { detected, .. } => {
                    lines.push(InspLine::Prop(
                        String::new(),
                        format!("detected {}", detected.label()),
                        Role::Fg(FgStep::Muted),
                    ));
                }
            }
            if let Some(endpoint) = &account.endpoint {
                lines.push(InspLine::Prop(
                    "Endpoint".into(),
                    format!("{} · {}", endpoint.label, endpoint.host),
                    Role::Fg(FgStep::Primary),
                ));
            }
            lines.push(InspLine::Prop(
                "Provenance".into(),
                format!(
                    "{} · confidence {} · {}",
                    account
                        .provenance
                        .iter()
                        .map(|provenance| provenance.label())
                        .collect::<Vec<_>>()
                        .join(", "),
                    account.confidence.label(),
                    match account.origin {
                        AccountOrigin::Registered => "registered",
                        AccountOrigin::Discovered => "discovered",
                    }
                ),
                Role::Fg(FgStep::Secondary),
            ));
            lines.push(InspLine::Prop(
                "Lifecycle".into(),
                account.lifecycle.label().into(),
                if account.lifecycle == Lifecycle::Available {
                    Role::Fg(FgStep::Primary)
                } else {
                    Role::Warning
                },
            ));
            lines.push(InspLine::Prop(
                "Default".into(),
                if account.default_for_provider {
                    format!("★ for {}", account.provider.short())
                } else {
                    "no".into()
                },
                Role::Fg(FgStep::Primary),
            ));
            lines.push(InspLine::Prop(
                "Enabled".into(),
                if account.enabled {
                    "yes".into()
                } else {
                    "no · disabled".into()
                },
                if account.enabled {
                    Role::Fg(FgStep::Primary)
                } else {
                    Role::Fg(FgStep::Muted)
                },
            ));
            if let Some(purpose) = &account.purpose {
                lines.push(InspLine::Prop(
                    "Purpose".into(),
                    purpose.clone(),
                    Role::Fg(FgStep::Primary),
                ));
            }
            if !ws_users.is_empty() {
                lines.push(InspLine::Prop(
                    "Used by".into(),
                    format!("Workspace choice in {}", ws_users.join(", ")),
                    Role::Fg(FgStep::Secondary),
                ));
            }
            lines.push(InspLine::Blank);
            let fresh = match account.usage.freshness.phase {
                Freshness::Current => format!(
                    "current · refreshed {}",
                    account
                        .last_refresh_secs
                        .map(|secs| world.clock.ago(secs))
                        .unwrap_or("never".into())
                ),
                Freshness::Stale => format!(
                    "stale · last good {}",
                    account
                        .usage
                        .freshness
                        .last_good_secs
                        .map(|secs| world.clock.ago(secs))
                        .unwrap_or("?".into())
                ),
                Freshness::Refreshing => "refreshing…".into(),
                Freshness::Failed => format!(
                    "failed · last good {}",
                    account
                        .usage
                        .freshness
                        .last_good_secs
                        .map(|secs| world.clock.ago(secs))
                        .unwrap_or("never".into())
                ),
            };
            lines.push(InspLine::Heading("Quota".into(), fresh));
            if account.usage.windows.is_empty() {
                lines.push(InspLine::Text(
                    "No quota windows · not started or unavailable".into(),
                    Role::Fg(FgStep::Muted),
                ));
            }
            for win in &account.usage.windows {
                if win.has_meter() {
                    lines.push(InspLine::Meter(
                        win.label.clone(),
                        win.used_pct.unwrap_or(0),
                        meter_detail(win, world),
                        meter_tone(win, account),
                    ));
                } else {
                    let value = win.value_label();
                    let text = if value
                        .to_lowercase()
                        .starts_with(&win.label.to_lowercase())
                    {
                        value
                    } else {
                        format!("{}   {value}", win.label)
                    };
                    lines.push(InspLine::Text(
                        text,
                        if win.status == QuotaStatus::Error {
                            Role::Danger
                        } else {
                            Role::Fg(FgStep::Muted)
                        },
                    ));
                }
            }
            lines.push(InspLine::Blank);
            let (material, identity, quota) = validation_marks(account);
            lines.push(InspLine::Prop(
                "Validation".into(),
                format!(
                    "{material} material   {identity} identity   {quota} quota access · {}",
                    account.validation.label()
                ),
                Role::Fg(FgStep::Primary),
            ));
            if let Some(issue) = &account.issue {
                let tone =
                    if issue.is_informational() || issue.code == IssueCode::QuotaUnsupported {
                        Role::Warning
                    } else {
                        Role::Danger
                    };
                let mut text = issue.message.clone();
                if let Some(detail) = &issue.detail {
                    text.push_str(&format!(" · {detail}"));
                }
                if let Some(retry) = issue.retry_secs {
                    text.push_str(&format!(
                        " · retry {}",
                        world.clock.reset_label(retry).replace("resets", "")
                    ));
                }
                lines.push(InspLine::Prop(
                    "Status".into(),
                    format!("{} · {}", text, issue.recoverability.label()),
                    tone,
                ));
            }
            if account.origin == AccountOrigin::Discovered {
                lines.push(InspLine::Text(
                    "read-only: discovered on host · refresh only".into(),
                    Role::Fg(FgStep::Muted),
                ));
            }
            (
                account.title(),
                format!("account · {}", account.status_word()),
                lines,
            )
        }
        AccountSel::Add => (
            "Add account".into(),
            "form".into(),
            vec![
                InspLine::Text(
                    "Register a Claude Code, Codex, Grok Build or OpenCode account.".into(),
                    Role::Fg(FgStep::Secondary),
                ),
                InspLine::Text(
                    "1Password item/field references are the primary credential path; a local agent folder or a masked plain-text key are the alternatives.".into(),
                    Role::Fg(FgStep::Muted),
                ),
            ],
        ),
    }
}

/// Header crumb naming the current selection.
pub fn crumb(world: &World, sel: &AccountSel) -> String {
    match sel {
        AccountSel::Overview => "Accounts › Overview".into(),
        AccountSel::Provider(surface) => format!("Accounts › {}", surface.surface_name()),
        AccountSel::Account(id) => match world.accounts.get(id) {
            Some(account) => format!(
                "Accounts › {} › {}",
                account.surface.surface_name(),
                account.display_name
            ),
            None => "Accounts".into(),
        },
        AccountSel::Add => "Accounts › new account".into(),
    }
}

/// Number of accounts currently refreshing (the header badge count).
pub fn refreshing_count(world: &World) -> usize {
    world
        .accounts
        .accounts
        .iter()
        .filter(|account| account.usage.freshness.phase == Freshness::Refreshing)
        .count()
}

fn resolve_style(ui: &Ui<'_>, fg: Role, bg: Role, bold: bool) -> PaintStyle {
    let mut patch = StylePatch::new().set_fg(fg).set_bg(bg);
    if bold {
        patch = patch.add(Modifier::BOLD);
    } else {
        patch = patch.remove(Modifier::BOLD);
    }
    ui.style_defaults(
        Family::PANEL,
        Variant::DEFAULT,
        Part::custom("jackin.accounts.style"),
        StateFlags::empty(),
        StyleDefaults::new(patch),
        None,
    )
    .style
}

/// Resolved paint styles for the accounts route.
pub struct AccountsPalette {
    /// Canvas background style.
    pub canvas: PaintStyle,
    /// Unfocused gutter style.
    pub gutter_unfocused: PaintStyle,
    /// Border on canvas.
    pub border_on_canvas: PaintStyle,
    /// Primary text on canvas.
    pub primary_on_canvas: PaintStyle,
    /// Bold primary text on canvas.
    pub primary_on_canvas_bold: PaintStyle,
    /// Secondary text on canvas.
    pub secondary_on_canvas: PaintStyle,
    /// Muted text on canvas.
    pub muted_on_canvas: PaintStyle,
    /// Faint text on canvas.
    pub faint_on_canvas: PaintStyle,
    /// Accent text on canvas.
    pub accent_on_canvas: PaintStyle,
    /// Danger text on canvas.
    pub danger_on_canvas: PaintStyle,
    /// Warning text on canvas.
    pub warning_on_canvas: PaintStyle,
    /// Bold primary text on the focused-row tint.
    pub primary_on_accent_tint_bold: PaintStyle,
    /// Accent text on the focused-row tint.
    pub accent_on_accent_tint_bold: PaintStyle,
    /// Secondary text on the focused-row tint.
    pub secondary_on_accent_tint: PaintStyle,
    /// Muted text on the focused-row tint.
    pub muted_on_accent_tint: PaintStyle,
    /// Faint text on the focused-row tint.
    pub faint_on_accent_tint: PaintStyle,
    /// Danger text on the focused-row tint.
    pub danger_on_accent_tint: PaintStyle,
    /// Warning text on the focused-row tint.
    pub warning_on_accent_tint: PaintStyle,
    /// Card background style.
    pub card_bg: PaintStyle,
    /// Card border style.
    pub card_border: PaintStyle,
    /// Primary text on cards.
    pub card_primary: PaintStyle,
    /// Bold primary text on cards.
    pub card_primary_bold: PaintStyle,
    /// Secondary text on cards.
    pub card_secondary: PaintStyle,
    /// Bold secondary text on cards.
    pub card_secondary_bold: PaintStyle,
    /// Muted text on cards.
    pub card_muted: PaintStyle,
    /// Faint text on cards.
    pub card_faint: PaintStyle,
    /// Danger text on cards.
    pub card_danger: PaintStyle,
    /// Warning text on cards.
    pub card_warning: PaintStyle,
}

impl AccountsPalette {
    /// Resolve every accounts style against the current theme.
    pub fn new(ui: &Ui<'_>) -> Self {
        let canvas = Role::Surface(Surface::Canvas);
        let surface = Role::Surface(Surface::Surface);
        Self {
            canvas: resolve_style(ui, Role::Fg(FgStep::Primary), canvas, false),
            gutter_unfocused: resolve_style(ui, canvas, canvas, false),
            border_on_canvas: resolve_style(ui, Role::BorderSubtle, canvas, false),
            primary_on_canvas: resolve_style(ui, Role::Fg(FgStep::Primary), canvas, false),
            primary_on_canvas_bold: resolve_style(ui, Role::Fg(FgStep::Primary), canvas, true),
            secondary_on_canvas: resolve_style(ui, Role::Fg(FgStep::Secondary), canvas, false),
            muted_on_canvas: resolve_style(ui, Role::Fg(FgStep::Muted), canvas, false),
            faint_on_canvas: resolve_style(ui, Role::Fg(FgStep::Faint), canvas, false),
            accent_on_canvas: resolve_style(ui, Role::Accent, canvas, false),
            danger_on_canvas: resolve_style(ui, Role::Danger, canvas, false),
            warning_on_canvas: resolve_style(ui, Role::Warning, canvas, false),
            primary_on_accent_tint_bold: resolve_style(
                ui,
                Role::Fg(FgStep::Primary),
                Role::AccentTint,
                true,
            ),
            accent_on_accent_tint_bold: resolve_style(ui, Role::Accent, Role::AccentTint, true),
            secondary_on_accent_tint: resolve_style(
                ui,
                Role::Fg(FgStep::Secondary),
                Role::AccentTint,
                false,
            ),
            muted_on_accent_tint: resolve_style(
                ui,
                Role::Fg(FgStep::Muted),
                Role::AccentTint,
                false,
            ),
            faint_on_accent_tint: resolve_style(
                ui,
                Role::Fg(FgStep::Faint),
                Role::AccentTint,
                false,
            ),
            danger_on_accent_tint: resolve_style(ui, Role::Danger, Role::AccentTint, false),
            warning_on_accent_tint: resolve_style(ui, Role::Warning, Role::AccentTint, false),
            card_bg: resolve_style(ui, Role::Fg(FgStep::Primary), surface, false),
            card_border: resolve_style(ui, Role::BorderSubtle, surface, false),
            card_primary: resolve_style(ui, Role::Fg(FgStep::Primary), surface, false),
            card_primary_bold: resolve_style(ui, Role::Fg(FgStep::Primary), surface, true),
            card_secondary: resolve_style(ui, Role::Fg(FgStep::Secondary), surface, false),
            card_secondary_bold: resolve_style(ui, Role::Fg(FgStep::Secondary), surface, true),
            card_muted: resolve_style(ui, Role::Fg(FgStep::Muted), surface, false),
            card_faint: resolve_style(ui, Role::Fg(FgStep::Faint), surface, false),
            card_danger: resolve_style(ui, Role::Danger, surface, false),
            card_warning: resolve_style(ui, Role::Warning, surface, false),
        }
    }

    fn card_tone(&self, tone: Role) -> PaintStyle {
        match tone {
            Role::Danger => self.card_danger,
            Role::Warning => self.card_warning,
            Role::Accent => self.card_primary,
            Role::Fg(FgStep::Secondary) => self.card_secondary,
            Role::Fg(FgStep::Muted) => self.card_muted,
            Role::Fg(FgStep::Faint) => self.card_faint,
            _ => self.card_primary,
        }
    }

    fn tree_tone(&self, tone: Role, focused: bool) -> PaintStyle {
        match tone {
            Role::Danger => {
                if focused {
                    self.danger_on_accent_tint
                } else {
                    self.danger_on_canvas
                }
            }
            Role::Warning => {
                if focused {
                    self.warning_on_accent_tint
                } else {
                    self.warning_on_canvas
                }
            }
            Role::Fg(FgStep::Secondary) => {
                if focused {
                    self.secondary_on_accent_tint
                } else {
                    self.secondary_on_canvas
                }
            }
            Role::Fg(FgStep::Faint) => {
                if focused {
                    self.faint_on_accent_tint
                } else {
                    self.faint_on_canvas
                }
            }
            _ => {
                if focused {
                    self.muted_on_accent_tint
                } else {
                    self.muted_on_canvas
                }
            }
        }
    }
}

const FAINT_DETAIL_PATCH: [(Part, StylePatch); 1] = [(
    Part::DETAIL,
    StylePatch::new().set_fg(Role::Fg(FgStep::Faint)),
)];

fn fit(text: &str, width: u16) -> String {
    let truncated = truncate(text, width);
    let pad = width.saturating_sub(termrock::width(&truncated));
    format!("{truncated}{}", " ".repeat(pad as usize))
}

/// The accounts route screen: tree plus inspector.
pub struct AccountsScreen;

impl AccountsScreen {
    /// Draw the tree and inspector panes for the route body area.
    pub fn draw(ui: &mut Ui<'_>, area: Rect, state: &AccountsState, world: &World, focused: bool) {
        let palette = AccountsPalette::new(ui);
        let full = ui.full();
        let stage = Rect::new(
            full.x,
            full.y.saturating_add(1),
            full.width,
            full.height.saturating_sub(2),
        );
        ui.fill(stage, palette.canvas);
        let rows = build_account_rows(world, state.filter.as_deref(), &state.folded);
        let is_narrow = area.width < 100;

        if is_narrow {
            let drawer = state.drawer_open;
            let summary_h = 6u16.min(area.height / 3);
            let tree_area = Rect::new(
                area.x,
                area.y,
                area.width,
                area.height.saturating_sub(summary_h + 1),
            );
            Self::draw_tree(
                ui,
                tree_area,
                state,
                world,
                focused && !drawer,
                &rows,
                &palette,
            );
            if drawer {
                Self::draw_inspector(ui, area, state, world, focused, true, &palette);
            } else {
                let summary_area = Rect::new(area.x, tree_area.bottom() + 1, area.width, summary_h);
                Self::draw_summary(ui, summary_area, state, world, &palette);
            }
            return;
        }

        let split = SplitPane::new(SEAM, SplitAxis::Horizontal)
            .gap(2)
            .seam_end(1)
            .min_first(30)
            .min_second(40);
        let split_state = SplitPaneState::new(34);

        split.draw(ui, area, &split_state, |ui, left, right| {
            Self::draw_tree(
                ui,
                left,
                state,
                world,
                focused && !state.drawer_open,
                &rows,
                &palette,
            );
            Self::draw_inspector(
                ui,
                right,
                state,
                world,
                focused && state.drawer_open,
                false,
                &palette,
            );
            if world.accounts.accounts.is_empty()
                && matches!(state.selected, AccountSel::Overview)
            {
                let empty_state = EmptyState::Empty {
                    title: "No accounts registered",
                    hint: Some(
                        "a adds one · 1Password is the default credential source · discovered sources appear after a refresh",
                    ),
                };
                ui.fill(
                    Rect::new(
                        right.x.saturating_add(2),
                        right.y.saturating_add(3),
                        right.width.saturating_sub(4),
                        right.height.saturating_sub(6),
                    ),
                    palette.card_bg,
                );
                Empty::new(INSPECTOR.sub("empty"), empty_state).draw(
                    ui,
                    Rect::new(
                        right.x.saturating_add(2),
                        right.y.saturating_add(3),
                        right.width.saturating_sub(4),
                        right.height.saturating_sub(6),
                    ),
                );
            }
        });
    }

    /// Footer hints for the current selection and drawer state.
    pub fn hints(state: &AccountsState, world: &World) -> HintLayer {
        if state.drawer_open {
            return HintLayer {
                hints: vec![
                    Hint {
                        key: HintKey::Label("↑↓"),
                        label: "Scroll",
                        priority: 100,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Tab)),
                        label: "Actions",
                        priority: 90,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Esc)),
                        label: "Back",
                        priority: 80,
                    },
                ],
                badge: None,
                status: None,
                centered: true,
            };
        }
        let mut hints = vec![];
        let mut priority = 100u8;
        let mut push = |key: HintKey, label: &'static str| {
            hints.push(Hint {
                key,
                label,
                priority,
            });
            priority = priority.saturating_sub(5);
        };
        let enter = || HintKey::Chord(Chord::key(KeyCode::Enter));
        let key = |ch: char| HintKey::Chord(Chord::key(KeyCode::Char(ch)));
        match &state.selected {
            AccountSel::Account(id) => {
                let registered = world
                    .accounts
                    .get(id)
                    .is_some_and(|account| account.mutations_allowed());
                push(enter(), "Details");
                push(key('r'), "Refresh");
                if registered {
                    push(key('e'), "Edit…");
                    push(key(' '), "Default");
                    push(key('d'), "Disable");
                    push(key('v'), "Validate");
                    push(key('x'), "Remove…");
                }
            }
            AccountSel::Provider(_) => {
                push(key(' '), "Fold");
                push(key('r'), "Refresh provider");
            }
            AccountSel::Overview => {
                push(enter(), "Details");
                push(key('r'), "Refresh all");
            }
            AccountSel::Add => push(enter(), "Add account…"),
        }
        push(key('a'), "Add…");
        push(key('/'), "Filter");
        push(key('m'), "Usage");
        push(HintKey::Chord(Chord::key(KeyCode::Esc)), "Back");
        HintLayer {
            hints,
            badge: None,
            status: None,
            centered: true,
        }
    }

    fn draw_tree(
        ui: &mut Ui<'_>,
        area: Rect,
        state: &AccountsState,
        world: &World,
        focused: bool,
        rows: &[AccountRow],
        palette: &AccountsPalette,
    ) {
        let summary = OverallSummary::compute(&world.accounts.accounts);
        let cursor = rows.iter().position(|row| row.sel == state.selected);
        let mut scroll = ScrollState::new(rows.len());
        scroll.set_viewport(area.height.saturating_sub(2) as usize);
        if let Some(index) = cursor {
            scroll.ensure_visible(index);
        }
        let pos = position_label(&scroll);
        let mut meta = format!("{}", summary.counts.accounts);
        if summary.counts.warnings > 0 {
            meta.push_str(&format!(" · {} ▲", summary.counts.warnings));
        }
        if summary.counts.exhausted + summary.counts.failed > 0 {
            meta.push_str(&format!(
                " · {} !",
                summary.counts.exhausted + summary.counts.failed
            ));
        }
        if !pos.is_empty() {
            meta.push_str(&format!(" · {pos}"));
        }
        let title = match state.filter.as_deref() {
            Some(active) if !active.is_empty() => format!("Accounts · filter {active}"),
            _ => "Accounts".into(),
        };

        ui.register_control(TREE, area, Focusability::Focusable);

        Panel::new(TREE)
            .kind(PanelKind::Framed)
            .title(&title)
            .meta(&meta)
            .focused(focused)
            .slot(Part::GUTTER, &|_ui, _cell| {})
            .patch_part(&FAINT_DETAIL_PATCH)
            .inner_inset(Insets {
                l: 2,
                t: 1,
                r: 2,
                b: 1,
            })
            .draw(ui, area, |ui, inner| {
                let has_sb = scroll.overflows();
                let row_w = inner.width.saturating_sub(u16::from(has_sb));
                let show_meta = row_w >= 40;
                for (k, i) in scroll.visible_range().enumerate() {
                    let y = inner.y + k as u16;
                    if i >= rows.len() {
                        break;
                    }
                    let row = &rows[i];
                    let is_selected = cursor == Some(i);
                    let is_focused = focused && is_selected;
                    let rect = Rect::new(inner.x, y, row_w, 1);
                    if is_focused {
                        ui.fill(rect, palette.primary_on_accent_tint_bold);
                        ui.paint_str(
                            Rect::new(rect.x, y, 1, 1),
                            "▎",
                            palette.accent_on_accent_tint_bold,
                        );
                    } else {
                        ui.fill(rect, palette.canvas);
                        ui.paint_str(Rect::new(rect.x, y, 1, 1), " ", palette.gutter_unfocused);
                    }
                    let mut x = rect.x + 2 + row.depth * 2;
                    let glyph = if row.expandable {
                        if row.expanded { "▾" } else { "▸" }
                    } else if row.star {
                        "★"
                    } else {
                        " "
                    };
                    let gs = if row.star {
                        if is_focused {
                            palette.primary_on_accent_tint_bold
                        } else {
                            palette.secondary_on_canvas
                        }
                    } else if is_focused {
                        palette.secondary_on_accent_tint
                    } else {
                        palette.secondary_on_canvas
                    };
                    ui.paint_str(Rect::new(x, y, 1, 1), glyph, gs);
                    x += 2;
                    let meta_w = if show_meta { width(&row.meta) } else { 0 };
                    let avail = rect.right().saturating_sub(x + 1);
                    let hw: u16 = if row.health.is_some() { 2 } else { 0 };
                    let lw = avail.saturating_sub(if meta_w > 0 { meta_w + 2 } else { 0 } + hw);
                    let label_style = if row.faint {
                        palette.tree_tone(Role::Fg(FgStep::Faint), is_focused)
                    } else if row.sel == AccountSel::Add {
                        if is_focused {
                            palette.primary_on_accent_tint_bold
                        } else {
                            palette.secondary_on_canvas
                        }
                    } else if is_focused {
                        palette.accent_on_accent_tint_bold
                    } else if is_selected {
                        palette.accent_on_canvas
                    } else {
                        palette.primary_on_canvas
                    };
                    ui.paint_str(
                        Rect::new(x, y, lw, 1),
                        &fit(&truncate(&row.label, lw), lw),
                        label_style,
                    );
                    if let Some((glyph, tone)) = row.health {
                        ui.paint_str(
                            Rect::new(x + lw, y, 1, 1),
                            glyph,
                            palette.tree_tone(tone, is_focused),
                        );
                    }
                    if meta_w > 0 && meta_w + 4 < avail {
                        ui.paint_str(
                            Rect::new(rect.right().saturating_sub(meta_w + 1), y, meta_w, 1),
                            &row.meta,
                            palette.tree_tone(row.meta_tone, is_focused),
                        );
                    }
                }
                if has_sb {
                    ui.scroll_edges(
                        Rect::new(
                            inner.x,
                            inner.y,
                            (inner.right() - 1).saturating_sub(inner.x),
                            inner.height,
                        ),
                        &scroll,
                    );
                    let sb_rect =
                        Rect::new(inner.right().saturating_sub(1), inner.y, 1, inner.height);
                    let track_len = usize::from(sb_rect.height);
                    let (thumb_start, thumb_len) = scroll.thumb(track_len);
                    for row in 0..track_len {
                        let y = sb_rect.y + row as u16;
                        let pos = Rect::new(sb_rect.x, y, 1, 1);
                        if row >= thumb_start && row < thumb_start + thumb_len {
                            ui.paint_str(pos, "┃", palette.primary_on_canvas);
                        } else {
                            ui.paint_str(pos, "│", palette.border_on_canvas);
                        }
                    }
                }
            });
    }

    fn draw_inspector(
        ui: &mut Ui<'_>,
        area: Rect,
        state: &AccountsState,
        world: &World,
        focused: bool,
        as_drawer: bool,
        palette: &AccountsPalette,
    ) {
        let actions = inspector_actions(world, &state.selected);
        let (title, scope, lines) =
            inspector_lines(world, &state.selected, area.width.saturating_sub(6));
        let kind = if as_drawer {
            PanelKind::Framed
        } else {
            PanelKind::Card
        };
        let mut panel = Panel::new(INSPECTOR)
            .kind(kind)
            .title(&title)
            .meta(&scope)
            .focused(focused)
            .patch_part(&FAINT_DETAIL_PATCH);
        if kind == PanelKind::Card {
            panel = panel.inner_inset(Insets {
                l: 2,
                t: 2,
                r: 2,
                b: 1,
            });
        } else {
            panel = panel.inner_inset(Insets {
                l: 2,
                t: 1,
                r: 2,
                b: 1,
            });
        }
        panel.draw(ui, area, |ui, inner| {
            let body_h = inner.height.saturating_sub(2);
            let body = Rect::new(inner.x, inner.y, inner.width, body_h);
            let label_w: u16 = lines
                .iter()
                .map(|line| match line {
                    InspLine::Prop(label, _, _) => width(label),
                    InspLine::Meter(label, _, _, _) => width(label),
                    _ => 0,
                })
                .max()
                .unwrap_or(13)
                .clamp(13, 22)
                + 1;
            let mut rendered: Vec<InspLine> = vec![];
            for line in lines {
                match line {
                    InspLine::Prop(label, value, tone) => {
                        let value_w = body.width.saturating_sub(label_w) as usize;
                        for (i, part) in wrap(&value, value_w.max(8) as u16).into_iter().enumerate()
                        {
                            rendered.push(InspLine::Prop(
                                if i == 0 { label.clone() } else { String::new() },
                                part,
                                tone,
                            ));
                        }
                    }
                    InspLine::Text(value, tone) => {
                        for part in wrap(&value, body.width) {
                            rendered.push(InspLine::Text(part, tone));
                        }
                    }
                    other => rendered.push(other),
                }
            }
            let mut scroll = ScrollState::new(rendered.len());
            scroll.set_viewport(body_h as usize);
            ui.register_control(INSPECTOR, body, Focusability::Focusable);
            let meter_w = if inner.width >= 70 {
                30
            } else if inner.width >= 50 {
                20
            } else {
                14
            };
            let (label_style, heading_style, faint_style) = if as_drawer {
                (
                    palette.muted_on_canvas,
                    palette.secondary_on_canvas,
                    palette.faint_on_canvas,
                )
            } else {
                (
                    palette.card_muted,
                    palette.card_secondary_bold,
                    palette.card_faint,
                )
            };
            for (k, i) in scroll.visible_range().enumerate() {
                let y = body.y + k as u16;
                match &rendered[i] {
                    InspLine::Prop(label, value, tone) => {
                        ui.paint_str(
                            Rect::new(body.x, y, label_w.min(body.width), 1),
                            &truncate(label, label_w.min(body.width)),
                            label_style,
                        );
                        let tone_style = if as_drawer {
                            palette.tree_tone(*tone, false)
                        } else {
                            palette.card_tone(*tone)
                        };
                        ui.paint_str(
                            Rect::new(body.x + label_w, y, body.width.saturating_sub(label_w), 1),
                            &truncate(value, body.width.saturating_sub(label_w)),
                            tone_style,
                        );
                    }
                    InspLine::Text(text, tone) => {
                        let tone_style = if as_drawer {
                            palette.tree_tone(*tone, false)
                        } else {
                            palette.card_tone(*tone)
                        };
                        ui.paint_str(
                            Rect::new(body.x, y, body.width, 1),
                            &truncate(text, body.width),
                            tone_style,
                        );
                    }
                    InspLine::Heading(heading, meta) => {
                        ui.paint_str(
                            Rect::new(body.x, y, width(heading).min(body.width), 1),
                            &truncate(heading, body.width),
                            heading_style,
                        );
                        let room = body.width.saturating_sub(width(heading) + 3);
                        let meta = truncate(meta, room);
                        let meta_w = width(&meta);
                        if meta_w > 0 && room >= 8 {
                            ui.paint_str(
                                Rect::new(body.right() - meta_w, y, meta_w, 1),
                                &meta,
                                faint_style,
                            );
                        }
                    }
                    InspLine::Meter(label, pct, value, tone) => {
                        ui.paint_str(
                            Rect::new(body.x, y, label_w.saturating_sub(1), 1),
                            &truncate(label, label_w - 1),
                            if as_drawer {
                                palette.primary_on_canvas
                            } else {
                                palette.card_primary
                            },
                        );
                        let mx = body.x + label_w;
                        let mw = meter_w.min(body.width.saturating_sub(label_w + 8));
                        let pct_text = format!("{pct:>3}%");
                        Meter::new(INSPECTOR_METER)
                            .ratio(f64::from(*pct) / 100.0)
                            .value(&pct_text)
                            .tone(*tone)
                            .visual(MeterVisual::Block)
                            .draw(ui, Rect::new(mx, y, mw + 6, 1));
                        let vx = mx + mw + 8;
                        if vx < body.right() {
                            ui.paint_str(
                                Rect::new(vx, y, body.right().saturating_sub(vx), 1),
                                &truncate(value, body.right().saturating_sub(vx)),
                                if as_drawer {
                                    palette.muted_on_canvas
                                } else {
                                    palette.card_muted
                                },
                            );
                        }
                    }
                    InspLine::Blank => {}
                }
            }
            if scroll.overflows() {
                ui.scroll_edges(
                    Rect::new(
                        inner.x,
                        body.y,
                        (inner.right() - 1).saturating_sub(inner.x),
                        body_h,
                    ),
                    &scroll,
                );
                let sb_rect = Rect::new(inner.right() - 1, body.y, 1, body_h);
                let track_len = usize::from(sb_rect.height);
                let (thumb_start, thumb_len) = scroll.thumb(track_len);
                for row in 0..track_len {
                    let y = sb_rect.y + row as u16;
                    let pos = Rect::new(sb_rect.x, y, 1, 1);
                    if row >= thumb_start && row < thumb_start + thumb_len {
                        ui.paint_str(
                            pos,
                            "┃",
                            if as_drawer {
                                palette.primary_on_canvas
                            } else {
                                palette.card_primary
                            },
                        );
                    } else {
                        ui.paint_str(
                            pos,
                            "│",
                            if as_drawer {
                                palette.border_on_canvas
                            } else {
                                palette.card_border
                            },
                        );
                    }
                }
            }
            let ay = inner.bottom().saturating_sub(1);
            let mut x = inner.x.saturating_sub(1);
            for (id, label, danger, disabled) in &actions {
                let button_w = width(label) + 2;
                let avail = inner.right().saturating_add(1).saturating_sub(x);
                let button_w = button_w.min(avail);
                if button_w == 0 {
                    break;
                }
                let mut button = Button::new(*id, label);
                if *danger {
                    button = button.variant(Variant::DANGER);
                }
                if *disabled {
                    button = button.disabled(true);
                }
                button.draw(ui, Rect::new(x, ay, button_w, 1));
                x = x.saturating_add(button_w).saturating_add(2);
            }
        });
    }

    fn draw_summary(
        ui: &mut Ui<'_>,
        area: Rect,
        state: &AccountsState,
        world: &World,
        palette: &AccountsPalette,
    ) {
        let (title, scope, lines) =
            inspector_lines(world, &state.selected, area.width.saturating_sub(4));
        Panel::new(INSPECTOR)
            .kind(PanelKind::Card)
            .title(&title)
            .meta(&format!("{scope} · Tab details"))
            .draw(ui, area, |ui, inner| {
                let mut y = inner.y;
                for line in lines.iter().take(inner.height as usize) {
                    match line {
                        InspLine::Prop(label, value, tone) if !label.is_empty() => {
                            ui.paint_str(
                                Rect::new(inner.x, y, inner.width, 1),
                                &truncate(&format!("{label}  {value}"), inner.width),
                                palette.card_tone(*tone),
                            );
                            y += 1;
                        }
                        InspLine::Meter(label, pct, value, tone) => {
                            ui.paint_str(
                                Rect::new(inner.x, y, 12.min(inner.width), 1),
                                &truncate(label, 12),
                                palette.card_primary,
                            );
                            let pct_text = format!("{pct:>3}%");
                            Meter::new(INSPECTOR_METER)
                                .ratio(f64::from(*pct) / 100.0)
                                .value(&pct_text)
                                .tone(*tone)
                                .visual(MeterVisual::Block)
                                .draw(ui, Rect::new(inner.x + 13, y, 22, 1));
                            ui.paint_str(
                                Rect::new(inner.x + 37, y, inner.width.saturating_sub(37), 1),
                                &truncate(value, inner.width.saturating_sub(37)),
                                palette.card_muted,
                            );
                            y += 1;
                        }
                        InspLine::Text(value, tone) => {
                            ui.paint_str(
                                Rect::new(inner.x, y, inner.width, 1),
                                &truncate(value, inner.width),
                                palette.card_tone(*tone),
                            );
                            y += 1;
                        }
                        _ => {}
                    }
                    if y >= inner.bottom() {
                        break;
                    }
                }
            });
    }
}

/// Inspector action buttons for a selection: id, label, danger, disabled.
fn inspector_actions(world: &World, sel: &AccountSel) -> Vec<(Id, &'static str, bool, bool)> {
    match sel {
        AccountSel::Account(id) => match world.accounts.get(id) {
            Some(account) if account.mutations_allowed() => vec![
                (ACTION_REFRESH, "Refresh", false, false),
                (ACTION_VALIDATE, "Validate", false, false),
                (ACTION_EDIT, "Edit…", false, false),
                (
                    ACTION_DEFAULT,
                    "Set default",
                    false,
                    account.default_for_provider || !account.enabled,
                ),
                (
                    ACTION_TOGGLE,
                    if account.enabled { "Disable" } else { "Enable" },
                    false,
                    false,
                ),
                (ACTION_REMOVE, "Remove…", true, false),
            ],
            Some(_) => vec![(ACTION_REFRESH, "Refresh", false, false)],
            None => vec![],
        },
        AccountSel::Provider(_) => vec![
            (ACTION_REFRESH, "Refresh provider", false, false),
            (ACTION_ADD, "Add account…", false, false),
        ],
        AccountSel::Overview => vec![
            (ACTION_REFRESH, "Refresh all", false, false),
            (ACTION_ADD, "Add account…", false, false),
        ],
        AccountSel::Add => vec![(ACTION_ADD, "Add account…", false, false)],
    }
}

/// Durable accounts route state.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct AccountsState {
    /// Public-list selection state owned by this route.
    pub list: ListState,
    /// Current tree selection.
    pub selected: AccountSel,
    /// Provider surfaces folded shut in the tree.
    pub folded: HashSet<UsageSurface>,
    /// Applied tree filter text, if any.
    pub filter: Option<String>,
    /// Whether the inspector drawer holds focus.
    pub drawer_open: bool,
    /// Whether the account form is visible.
    pub form_open: bool,
    /// Whether the opener has handed focus to the editable form.
    pub started: bool,
    /// Existing account name being edited, if any.
    pub editing: Option<String>,
    /// Draft account display name.
    pub draft_name: String,
    /// Masked transient credential input.
    pub masked_input: String,
    /// Pending account refresh message, if any.
    pub pending_refresh: Option<String>,
    /// Pending account-removal confirmation, if any.
    pub remove_confirmation: Option<String>,
    /// Controlled public text field state for the display name.
    pub name_input: TextInputState,
    /// Controlled public text field state for a local folder.
    pub folder_input: TextInputState,
    /// Controlled public text field state for a transient API key.
    pub secret_input: TextInputState,
    /// Provider selector list state.
    pub provider_list: ListState,
    /// Credential-source selector list state.
    pub source_list: ListState,
    /// Selected registerable provider (Anthropic, OpenAI, xAI, OpenCode).
    pub provider_index: u8,
    /// Credential source choice (1Password, folder, API key).
    pub source_index: u8,
    /// Selected 1Password item metadata.
    pub selected_op: Option<OpReference>,
    /// Selected item label in the 1Password browser.
    pub op_item: String,
    /// Current 1Password browser level.
    pub op_stage: u8,
    /// Stable selected account id for semantic commands.
    pub selected_id: Option<String>,
}

impl core::fmt::Debug for AccountsState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AccountsState")
            .field("list", &self.list)
            .field("selected", &self.selected)
            .field("folded", &self.folded)
            .field("filter", &self.filter)
            .field("drawer_open", &self.drawer_open)
            .field("form_open", &self.form_open)
            .field("started", &self.started)
            .field("editing", &self.editing)
            .field("draft_name", &self.draft_name)
            .field("masked_input", &"[redacted]")
            .field("pending_refresh", &self.pending_refresh)
            .field("remove_confirmation", &self.remove_confirmation)
            .field("name_input", &self.name_input)
            .field("folder_input", &self.folder_input)
            .field("secret_input", &self.secret_input)
            .field("provider_list", &self.provider_list)
            .field("source_list", &self.source_list)
            .field("provider_index", &self.provider_index)
            .field("source_index", &self.source_index)
            .field("selected_op", &self.selected_op)
            .field("op_item", &self.op_item)
            .field("op_stage", &self.op_stage)
            .field("selected_id", &self.selected_id)
            .finish()
    }
}

impl AccountsState {
    /// Open a new account form.
    pub fn open_new(&mut self) {
        self.form_open = true;
        self.started = false;
        self.editing = None;
        self.draft_name.clear();
        self.masked_input.clear();
        self.name_input = TextInputState::default();
        self.folder_input = TextInputState::default();
        self.secret_input = TextInputState::default();
        self.provider_list = ListState::default();
        self.source_list = ListState::default();
        self.provider_index = 0;
        self.source_index = 0;
        self.selected_op = None;
        self.op_item.clear();
        self.op_stage = 0;
        self.selected_id = None;
        self.pending_refresh = None;
        self.remove_confirmation = None;
    }

    /// Close the form without writing a credential.
    pub const fn close(&mut self) {
        self.form_open = false;
    }

    /// Synchronize the stable account id with the tree selection.
    pub fn sync_selected_id(&mut self) {
        self.selected_id = match &self.selected {
            AccountSel::Account(id) => Some(id.clone()),
            _ => None,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jackin_preview_domain::account::AccountRegistry;
    use jackin_preview_domain::scenario::Scenario;

    fn accounts_mixed_world() -> World {
        jackin_preview_sim::world::world_for(Scenario::AccountsMixed)
    }

    #[test]
    fn spinner_cycles_ten_braille_frames() {
        assert_eq!(spinner_frame(0), "⠋");
        assert_eq!(spinner_frame(9), "⠏");
        assert_eq!(spinner_frame(10), "⠋");
    }

    #[test]
    fn boot_rows_open_on_overview_with_expanded_providers() {
        let world = accounts_mixed_world();
        let rows = build_account_rows(&world, None, &HashSet::new());
        assert_eq!(rows[0].sel, AccountSel::Overview);
        assert_eq!(rows[0].label, "Overview");
        // Overview, Claude provider, Personal, Archived, Work.
        assert_eq!(rows[1].sel, AccountSel::Provider(UsageSurface::Claude));
        assert!(rows[1].expandable && rows[1].expanded);
        assert!(rows[2].star, "Personal is the provider default");
        assert_eq!(rows[2].depth, 1);
        assert!(
            matches!(rows[4].sel, AccountSel::Account(_)),
            "fourth row down is the Work account"
        );
        assert_eq!(rows[4].label, "Work");
        assert_eq!(rows.last().unwrap().sel, AccountSel::Add);
    }

    #[test]
    fn work_filter_keeps_parents_and_drops_the_add_row() {
        let world = accounts_mixed_world();
        let rows = build_account_rows(&world, Some("work"), &HashSet::new());
        assert!(rows.iter().any(|row| row.label == "Work"));
        assert!(
            rows.iter()
                .any(|row| row.sel == AccountSel::Provider(UsageSurface::Claude)),
            "the Claude parent survives the filter"
        );
        assert!(
            !rows
                .iter()
                .any(|row| row.label == "★ Personal" || row.label == "Personal"),
            "Personal is filtered out"
        );
        assert!(
            !rows.iter().any(|row| row.sel == AccountSel::Add),
            "the Add row hides while filtered"
        );
    }

    #[test]
    fn crumbs_name_every_selection_kind() {
        let world = accounts_mixed_world();
        assert_eq!(crumb(&world, &AccountSel::Overview), "Accounts › Overview");
        assert_eq!(
            crumb(&world, &AccountSel::Provider(UsageSurface::Codex)),
            "Accounts › Codex"
        );
        assert_eq!(crumb(&world, &AccountSel::Add), "Accounts › new account");
        let rows = build_account_rows(&world, None, &HashSet::new());
        let work = rows
            .iter()
            .find(|row| row.label == "Work")
            .expect("Work row exists");
        assert_eq!(crumb(&world, &work.sel), "Accounts › Claude › Work");
    }

    #[test]
    fn overview_inspector_reports_registry_totals() {
        let world = accounts_mixed_world();
        let (title, scope, lines) = inspector_lines(&world, &AccountSel::Overview, 73);
        assert_eq!(title, "Overview");
        assert_eq!(scope, "12 accounts · 8 providers");
        let health = lines
            .iter()
            .find_map(|line| match line {
                InspLine::Prop(label, value, _) if label == "Health" => Some(value.clone()),
                _ => None,
            })
            .expect("Health prop exists");
        assert!(
            health.contains("degraded · 4 warnings"),
            "unexpected health line: {health}"
        );
    }

    #[test]
    fn overview_hints_match_the_tree_footer() {
        let world = accounts_mixed_world();
        let state = AccountsState::default();
        let layer = AccountsScreen::hints(&state, &world);
        let labels: Vec<&str> = layer.hints.iter().map(|hint| hint.label).collect();
        assert_eq!(
            labels,
            vec!["Details", "Refresh all", "Add…", "Filter", "Usage", "Back"]
        );
    }

    #[test]
    fn registry_is_empty_without_accounts() {
        let mut world = accounts_mixed_world();
        world.accounts = AccountRegistry::default();
        let rows = build_account_rows(&world, None, &HashSet::new());
        assert_eq!(rows[0].sel, AccountSel::Overview);
        assert_eq!(rows.last().unwrap().sel, AccountSel::Add);
        let (_, scope, _) = inspector_lines(&world, &AccountSel::Overview, 73);
        assert_eq!(scope, "0 accounts · 0 providers");
    }
}
