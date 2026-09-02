//! Account & Usage Center: register, inspect, edit, validate, enable,
//! set-default, remove and refresh provider accounts and API-key profiles;
//! 1Password is the primary credential path, masked plain text the
//! explicit alternative; an honest overall Usage summary sits on row zero.

use std::collections::HashSet;

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::core::scroll::ScrollState;
use junie_tui::theme::{ButtonKind, Tone};
use junie_tui::ui::ctx::{RenderCtx, fill};
use junie_tui::ui::layout::{Split, SplitDir};
use junie_tui::ui::text::{fit, truncate, width};
use junie_tui::widgets::button::{Button, row_layout};
use junie_tui::widgets::choice::RadioGroup;
use junie_tui::widgets::dialog::Dialog;
use junie_tui::widgets::empty::{self, EmptyState};
use junie_tui::widgets::input::TextInput;
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::progress::{MeterTone, render_meter, spinner_frame};
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::segments::Segment;
use junie_tui::widgets::select::Select;
use junie_tui::widgets::splitter::Splitter;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};

use super::modals::{BrowserResult, FileBrowser, FormDialog, FormField, FormValues, OpFlow};
use super::{Cx, Go, Modal, ModalResult, ModalTag, Screen, plural};
use crate::domain::account::{
    Account, AccountId, AccountOrigin, CredentialSource, DetectedKind, DuplicateProbe, IssueCode,
    Lifecycle, ValidationState, fingerprint, masked, tail_of,
};
use crate::domain::agent::{Provider, UsageSurface};
use crate::domain::onepassword::OpReference;
use crate::domain::usage::{Freshness, OverallSummary, QuotaStatus, QuotaWindow, WindowUnit};
use crate::sim::provider::{self, CheckRow, ValidationOutcome};
use crate::sim::world::{Msg, World};
use junie_tui::ui::text::thousands;

pub const TREE: WidgetId = WidgetId::of("accounts.tree");
pub const INSPECTOR: WidgetId = WidgetId::of("accounts.inspector");
pub const SEAM: WidgetId = WidgetId::of("accounts.seam");
pub const FORM: WidgetId = WidgetId::of("accounts.form");

const REGISTERABLE: [Provider; 4] = [
    Provider::Anthropic,
    Provider::OpenAi,
    Provider::XAi,
    Provider::OpenCode,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sel {
    Overview,
    Provider(UsageSurface),
    Account(AccountId),
    Add,
}

#[derive(Debug, Clone)]
struct Row {
    sel: Sel,
    depth: u16,
    star: bool,
    label: String,
    health: Option<(&'static str, Tone)>,
    meta: String,
    meta_tone: Tone,
    faint: bool,
    expandable: bool,
    expanded: bool,
}

/// Transient state of the add/edit form (never the key itself).
struct FormCtx {
    editing: Option<AccountId>,
    op_ref: Option<OpReference>,
    op_masked: Option<String>,
    validation: Option<ValidationOutcome>,
}

pub struct AccountsScreen {
    pub selected: Sel,
    folded: HashSet<UsageSurface>,
    pub filter: Option<String>,
    rows: Vec<Row>,
    scroll: ScrollState,
    inspector_scroll: ScrollState,
    tree_area: Rect,
    inspector_area: Rect,
    split: Split,
    seam: Splitter,
    seam_container: Rect,
    actions: Vec<Button>,
    form: Option<FormCtx>,
    narrow: bool,
    pub drawer_open: bool,
    pending_remove: Option<AccountId>,
}

impl Default for AccountsScreen {
    fn default() -> Self {
        Self::new()
    }
}

fn provider_label(p: Provider) -> &'static str {
    match p {
        Provider::Anthropic => "Claude Code · Anthropic / Claude",
        Provider::OpenAi => "Codex · OpenAI",
        Provider::XAi => "Grok Build · xAI / Grok",
        Provider::OpenCode => "OpenCode · OpenCode",
        _ => "",
    }
}

impl AccountsScreen {
    pub fn new() -> Self {
        Self {
            selected: Sel::Overview,
            folded: HashSet::new(),
            filter: None,
            rows: vec![],
            scroll: ScrollState::default(),
            inspector_scroll: ScrollState::default(),
            tree_area: Rect::ZERO,
            inspector_area: Rect::ZERO,
            split: Split::new(34, 30, 40),
            seam: Splitter::new(SEAM, SplitDir::Horizontal),
            seam_container: Rect::ZERO,
            actions: vec![],
            form: None,
            narrow: false,
            drawer_open: false,
            pending_remove: None,
        }
    }

    pub fn select(&mut self, id: Option<AccountId>) {
        if let Some(id) = id {
            self.selected = Sel::Account(id);
        }
    }

    fn build_rows(&mut self, w: &World) {
        let mut rows = vec![Row {
            sel: Sel::Overview,
            depth: 0,
            star: false,
            label: "Overview".into(),
            health: None,
            meta: String::new(),
            meta_tone: Tone::Muted,
            faint: false,
            expandable: false,
            expanded: false,
        }];
        let q = self
            .filter
            .as_ref()
            .map(|f| f.to_lowercase())
            .filter(|f| !f.is_empty());
        for surface in UsageSurface::ALL {
            let accounts: Vec<&Account> = w
                .accounts
                .sorted()
                .into_iter()
                .filter(|a| a.surface == surface)
                .filter(|a| match &q {
                    Some(q) => {
                        a.display_name.to_lowercase().contains(q)
                            || a.provider.label().to_lowercase().contains(q)
                            || a.identity.label().to_lowercase().contains(q)
                            || a.status_word().contains(q.as_str())
                            || a.identity
                                .plan
                                .as_ref()
                                .is_some_and(|p| p.to_lowercase().contains(q))
                    }
                    None => true,
                })
                .collect();
            if surface == UsageSurface::Unsupported {
                if q.is_none() {
                    rows.push(Row {
                        sel: Sel::Provider(surface),
                        depth: 0,
                        star: false,
                        label: "Unsupported".into(),
                        health: None,
                        meta: "sentinel".into(),
                        meta_tone: Tone::Faint,
                        faint: true,
                        expandable: false,
                        expanded: false,
                    });
                }
                continue;
            }
            if accounts.is_empty()
                && (q.is_some()
                    || !surface
                        .provider()
                        .is_some_and(|p| p.agent().is_some_and(|a| a.registerable())))
            {
                if q.is_none() {
                    rows.push(Row {
                        sel: Sel::Provider(surface),
                        depth: 0,
                        star: false,
                        label: surface.surface_name().into(),
                        health: None,
                        meta: "not discovered".into(),
                        meta_tone: Tone::Faint,
                        faint: true,
                        expandable: false,
                        expanded: false,
                    });
                }
                continue;
            }
            let expanded = !self.folded.contains(&surface);
            let warn = accounts
                .iter()
                .filter(|a| {
                    a.enabled && matches!(a.usage.worst_status(), Some(QuotaStatus::Warning))
                })
                .count();
            let err = accounts
                .iter()
                .filter(|a| {
                    a.enabled
                        && (a.is_error_state()
                            || matches!(a.usage.worst_status(), Some(QuotaStatus::Exhausted)))
                })
                .count();
            let health = if err > 0 {
                Some(("!", Tone::Error))
            } else if warn > 0 {
                Some(("▲", Tone::Warning))
            } else {
                None
            };
            rows.push(Row {
                sel: Sel::Provider(surface),
                depth: 0,
                star: false,
                label: surface.surface_name().into(),
                health,
                meta: if accounts.is_empty() {
                    "no accounts".into()
                } else {
                    accounts.len().to_string()
                },
                meta_tone: Tone::Muted,
                faint: accounts.is_empty(),
                expandable: !accounts.is_empty(),
                expanded,
            });
            if expanded {
                for a in accounts {
                    let (meta, tone) = account_meta(a, w);
                    let health = if !a.enabled {
                        None
                    } else if a.is_error_state()
                        || matches!(a.usage.worst_status(), Some(QuotaStatus::Exhausted))
                    {
                        Some(("!", Tone::Error))
                    } else if matches!(a.usage.worst_status(), Some(QuotaStatus::Warning))
                        || a.usage.freshness.phase == Freshness::Stale
                    {
                        Some(("▲", Tone::Warning))
                    } else {
                        None
                    };
                    rows.push(Row {
                        sel: Sel::Account(a.id.clone()),
                        depth: 1,
                        star: a.default_for_provider,
                        label: if a.origin == AccountOrigin::Discovered
                            && a.display_name != "discovered"
                        {
                            format!("{} · discovered", a.display_name)
                        } else {
                            a.display_name.clone()
                        },
                        health,
                        meta,
                        meta_tone: tone,
                        faint: !a.enabled,
                        expandable: false,
                        expanded: false,
                    });
                }
            }
        }
        if q.is_none() {
            rows.push(Row {
                sel: Sel::Add,
                depth: 0,
                star: false,
                label: "+ Add account…".into(),
                health: None,
                meta: String::new(),
                meta_tone: Tone::Muted,
                faint: false,
                expandable: false,
                expanded: false,
            });
        }
        self.rows = rows;
        if !self.rows.iter().any(|r| r.sel == self.selected) {
            self.selected = match &self.selected {
                Sel::Account(id) => w
                    .accounts
                    .get(id)
                    .map(|a| Sel::Provider(a.surface))
                    .unwrap_or(Sel::Overview),
                _ => Sel::Overview,
            };
            if !self.rows.iter().any(|r| r.sel == self.selected) {
                self.selected = Sel::Overview;
            }
        }
        self.scroll.set_content(self.rows.len());
        if let Some(i) = self.cursor() {
            self.scroll.ensure_visible(i);
        }
    }

    fn cursor(&self) -> Option<usize> {
        self.rows.iter().position(|r| r.sel == self.selected)
    }

    fn move_cursor(&mut self, delta: isize) {
        let n = self.rows.len();
        if n == 0 {
            return;
        }
        let cur = self.cursor().unwrap_or(0) as isize;
        let next = (cur + delta).clamp(0, n as isize - 1) as usize;
        self.selected = self.rows[next].sel.clone();
        self.scroll.ensure_visible(next);
        self.inspector_scroll.jump_start();
    }

    fn selected_account<'a>(&self, w: &'a World) -> Option<&'a Account> {
        match &self.selected {
            Sel::Account(id) => w.accounts.get(id),
            _ => None,
        }
    }

    // ------------------------------------------------------------ actions

    fn refresh_scope(&mut self, w: &mut World, cx: &mut Cx) {
        let ids: Vec<AccountId> = match &self.selected {
            Sel::Account(id) => vec![id.clone()],
            Sel::Provider(s) => w
                .accounts
                .accounts
                .iter()
                .filter(|a| a.surface == *s && a.enabled)
                .map(|a| a.id.clone())
                .collect(),
            _ => w
                .accounts
                .accounts
                .iter()
                .filter(|a| a.enabled)
                .map(|a| a.id.clone())
                .collect(),
        };
        if ids.is_empty() {
            cx.status("Nothing to refresh");
            return;
        }
        let mut started = 0;
        for (i, id) in ids.iter().enumerate() {
            let Some(a) = w.accounts.get_mut(id) else {
                continue;
            };
            if a.usage.freshness.phase == Freshness::Refreshing {
                continue;
            }
            a.usage.freshness.phase = Freshness::Refreshing;
            let dur = provider::refresh_duration_ms(a) + i as i64 * 160;
            w.schedule(
                dur,
                Msg::AccountRefreshed {
                    account: id.clone(),
                },
            );
            started += 1;
        }
        if started == 0 {
            cx.status("Refresh already running");
        } else {
            let scope = match &self.selected {
                Sel::Account(id) => w.accounts.get(id).map(|a| a.title()).unwrap_or_default(),
                Sel::Provider(s) => s.label().to_owned(),
                _ => "all".into(),
            };
            cx.status(format!(
                "Refreshing {scope} · {}",
                plural(started, "account", "accounts")
            ));
        }
    }

    fn apply_refresh(&mut self, id: &str, w: &mut World, cx: &mut Cx) {
        let now = w.now_secs();
        let locked = w.op.session != crate::sim::onepassword::OpSession::SignedIn;
        let broker_down = w.refresh_fails;
        let Some(a) = w.accounts.get_mut(id) else {
            return;
        };
        let title = a.title();
        a.last_refresh_secs = Some(now);
        let outcome: &str;
        match (&a.source, a.issue.as_ref().map(|i| i.code)) {
            _ if broker_down => {
                a.usage.freshness.phase = if a.usage.freshness.last_good_secs.is_some() {
                    Freshness::Stale
                } else {
                    Freshness::Failed
                };
                a.issue = Some(crate::domain::account::RecoverableIssue::new(
                    IssueCode::ProviderUnavailable,
                    "Usage broker unreachable: last good values kept",
                    crate::domain::account::Recoverability::Retryable,
                ));
                outcome = "broker unreachable · last good kept";
            }
            (CredentialSource::OnePassword(_), _) if locked => {
                a.usage.freshness.phase = Freshness::Failed;
                a.issue = Some(crate::domain::account::RecoverableIssue::new(
                    IssueCode::OpLocked,
                    "1Password locked: unlock the app and retry",
                    crate::domain::account::Recoverability::Retryable,
                ));
                outcome = "failed · 1Password locked";
            }
            (_, Some(IssueCode::RateLimited)) => {
                let retry = a.issue.as_ref().and_then(|i| i.retry_secs).unwrap_or(now);
                if now >= retry {
                    a.usage.freshness = crate::domain::usage::FreshnessInfo::current(now);
                    a.issue = None;
                    outcome = "current";
                } else {
                    a.usage.freshness.phase = Freshness::Failed;
                    outcome = "still rate limited";
                }
            }
            (_, Some(IssueCode::ProviderUnavailable)) => {
                a.usage.freshness.phase = Freshness::Failed;
                outcome = "provider unavailable";
            }
            (_, Some(IssueCode::CredentialFileMissing))
            | (_, Some(IssueCode::Unauthorized))
            | (_, Some(IssueCode::ApiKeyInvalid)) => {
                a.usage.freshness.phase = Freshness::Failed;
                outcome = "needs attention";
            }
            _ => {
                a.usage.freshness = crate::domain::usage::FreshnessInfo::current(now);
                if let Some(i) = &a.issue
                    && i.code == IssueCode::Stale
                {
                    a.issue = None;
                }
                if let Some(win) = a
                    .usage
                    .windows
                    .iter_mut()
                    .find(|w| w.used_pct.is_some() && w.status != QuotaStatus::Exhausted)
                {
                    let p = (win.used_pct.unwrap_or(0) + 1).min(100);
                    win.used_pct = Some(p);
                    win.status = QuotaStatus::from_pct(p);
                }
                outcome = "current";
            }
        }
        let n = a.usage.windows.iter().filter(|w| w.has_meter()).count();
        cx.status(format!(
            "Refreshed {title} · {outcome} · {}",
            plural(n, "window", "windows")
        ));
    }

    fn open_form(&mut self, editing: Option<&Account>, w: &World, cx: &mut Cx) {
        let title = match editing {
            Some(a) => format!("Edit account · {}", a.title()),
            None => "New account".into(),
        };
        let provider_idx = editing
            .map(|a| {
                REGISTERABLE
                    .iter()
                    .position(|p| *p == a.provider)
                    .unwrap_or(0)
            })
            .unwrap_or(0);
        let source_idx = match editing.map(|a| &a.source) {
            Some(CredentialSource::LocalFolder { .. }) => 1,
            Some(CredentialSource::PlainApiKey { .. }) => 2,
            _ => 0,
        };
        let labels: Vec<&str> = REGISTERABLE.iter().map(|p| provider_label(*p)).collect();
        let mut fields = vec![
            FormField::input(
                "name",
                TextInput::new(FORM.sub("name"), "Display name")
                    .required(true)
                    .placeholder("Personal, Work, Team…")
                    .value(editing.map(|a| a.display_name.as_str()).unwrap_or("")),
            ),
            FormField::input(
                "purpose",
                TextInput::new(FORM.sub("purpose"), "Purpose label")
                    .placeholder("personal · work · experiments")
                    .value(editing.and_then(|a| a.purpose.as_deref()).unwrap_or("")),
            ),
            FormField::select(
                "provider",
                Select::new(FORM.sub("provider"), "Provider", &labels, provider_idx),
            ),
            FormField::note("link", vec![]),
            FormField::radio(
                "source",
                RadioGroup::new(
                    FORM.sub("source"),
                    "Credential source",
                    &[
                        "1Password item / field  (recommended)",
                        "Local agent folder",
                        "Plain-text API key",
                    ],
                    source_idx,
                ),
            ),
            FormField::chooser(
                "op",
                FORM.sub("op"),
                "1Password reference",
                "not chosen",
                "Choose…",
            ),
            FormField::note("op_note", vec![]),
            FormField::input(
                "folder",
                TextInput::new(FORM.sub("folder"), "Local folder")
                    .placeholder("~/.claude")
                    .value(match editing.map(|a| &a.source) {
                        Some(CredentialSource::LocalFolder { path, .. }) => path.as_str(),
                        _ => "",
                    }),
            )
            .hidden(),
            FormField::chooser("browse", FORM.sub("browse"), "", "", "Browse…").hidden(),
            FormField::input(
                "key",
                TextInput::new(FORM.sub("key"), "API key")
                    .masked()
                    .reveal_tail(4)
                    .placeholder("paste the key · shown masked")
                    .help("Typed once; only a fingerprint and a four-character tail are stored"),
            )
            .hidden(),
            FormField::input(
                "endpoint",
                TextInput::new(FORM.sub("endpoint"), "Endpoint / deployment").value("api.x.ai"),
            )
            .hidden(),
            FormField::note("validation", vec![]),
        ];
        // editing keeps existing reference metadata
        let mut ctx = FormCtx {
            editing: editing.map(|a| a.id.clone()),
            op_ref: None,
            op_masked: None,
            validation: None,
        };
        if let Some(CredentialSource::OnePassword(r)) = editing.map(|a| &a.source) {
            ctx.op_ref = Some(r.clone());
            ctx.op_masked = w.op.describe(r).ok().map(|d| d.masked);
            if let Some(f) = fields.iter_mut().find(|f| f.name == "op")
                && let super::modals::FieldKindW::Chooser { value, .. } = &mut f.kind
            {
                *value = r.display_path();
            }
        }
        if let (Some(a), Some(e)) = (editing, editing.and_then(|a| a.endpoint.as_ref())) {
            let _ = a;
            if let Some(f) = fields.iter_mut().find(|f| f.name == "endpoint")
                && let super::modals::FieldKindW::Input(i) = &mut f.kind
            {
                *i = TextInput::new(i.id, "Endpoint / deployment").value(&e.host);
            }
        }
        self.form = Some(ctx);
        let mut form = FormDialog::new(FORM, &title, fields)
            .meta(if editing.is_some() {
                "form · edits"
            } else {
                "form · unsaved"
            })
            .action(
                "plain",
                Button::subtle(FORM.sub("plain"), "Enter plain text instead"),
            )
            .action(
                "validate",
                Button::secondary(FORM.sub("validate"), "Validate"),
            )
            .width(70)
            .keep_open_on_save();
        Self::reveal(&mut form, w);
        cx.open(Modal::Form(form), ModalTag::new("form"));
    }

    /// Progressive disclosure: show the fields the current choices need.
    fn reveal(form: &mut FormDialog, w: &World) {
        let provider = REGISTERABLE[form.choice("provider").min(3)];
        let source = form.choice("source");
        form.set_note(
            "link",
            vec![(
                format!(
                    "Agent runtime {} · provider {} · usage surface {}",
                    provider.agent().map(|a| a.label()).unwrap_or("—"),
                    provider.label(),
                    provider.usage_surface().surface_name()
                ),
                Tone::Muted,
            )],
        );
        form.set_visible("op", source == 0);
        form.set_visible("op_note", source == 0);
        form.set_visible("folder", source == 1);
        form.set_visible("browse", source == 1);
        form.set_visible("key", source == 2);
        form.set_visible("endpoint", provider.supports_endpoint() && source != 1);
        if let Some(f) = form.field_mut("folder")
            && let super::modals::FieldKindW::Input(i) = &mut f.kind
        {
            i.placeholder = match provider {
                Provider::Anthropic => "~/.claude".into(),
                Provider::OpenAi => "~/.codex  (CODEX_HOME)".into(),
                Provider::XAi => "~/.grok".into(),
                _ => "~/.local/share/opencode".into(),
            };
            i.label = provider.folder_label().to_owned();
        }
        if let Some(f) = form.field_mut("key")
            && let super::modals::FieldKindW::Input(i) = &mut f.kind
        {
            i.label = provider.plain_key_label().to_owned();
        }
        if let Some((_, b)) = form.actions.iter_mut().find(|(n, _)| n == "plain") {
            b.disabled = source != 0;
        }
        let _ = w;
    }

    fn run_validation(
        &mut self,
        values: &FormValues,
        w: &World,
    ) -> Result<ValidationOutcome, String> {
        let get = |name: &str| -> String {
            values
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| match v {
                    super::modals::FieldValue::Text(s) => s.clone(),
                    _ => String::new(),
                })
                .unwrap_or_default()
        };
        let choice = |name: &str| -> usize {
            values
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| match v {
                    super::modals::FieldValue::Choice(i) => *i,
                    _ => 0,
                })
                .unwrap_or(0)
        };
        let provider = REGISTERABLE[choice("provider").min(3)];
        let source = choice("source");
        let now = w.now_secs();
        let ctx = self.form.as_ref().ok_or("no form")?;
        match source {
            0 => {
                let r = ctx
                    .op_ref
                    .clone()
                    .ok_or("Choose a 1Password item and field first")?;
                Ok(provider::validate(
                    provider,
                    &CredentialSource::OnePassword(r),
                    None,
                    &w.op,
                    now,
                ))
            }
            1 => {
                let path = get("folder");
                if path.trim().is_empty() {
                    return Err("Enter or browse to the local agent folder".into());
                }
                Ok(provider::validate(
                    provider,
                    &CredentialSource::LocalFolder {
                        path: path.trim().to_owned(),
                        detected: DetectedKind::Unknown,
                    },
                    None,
                    &w.op,
                    now,
                ))
            }
            _ => {
                let key = get("key");
                if key.trim().is_empty() {
                    return Err("API key required: the field is empty".into());
                }
                Ok(provider::validate(
                    provider,
                    &CredentialSource::PlainApiKey {
                        fingerprint: fingerprint(&key),
                        tail: tail_of(&key),
                    },
                    Some(&key),
                    &w.op,
                    now,
                ))
            }
        }
    }

    fn validation_note(v: &ValidationOutcome) -> Vec<(String, Tone)> {
        let row = |label: &str, r: &CheckRow| match r {
            CheckRow::Ok(t) => (format!("✓ {label}   {t}"), Tone::Secondary),
            CheckRow::Failed(t) => (format!("! {label}   {t}"), Tone::Error),
            CheckRow::Skipped(t) => (format!("— {label}   {t}"), Tone::Muted),
        };
        let mut lines = vec![
            row("material", &v.material),
            row("identity", &v.identity_row),
            row("quota   ", &v.quota_row),
        ];
        if let Some(i) = &v.issue
            && !i.is_informational()
        {
            lines.push((
                i.message.clone(),
                if i.code == IssueCode::QuotaUnsupported {
                    Tone::Warning
                } else {
                    Tone::Error
                },
            ));
        }
        lines
    }

    fn save_form(&mut self, values: FormValues, w: &mut World, cx: &mut Cx) {
        let get = |name: &str| -> String {
            values
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| match v {
                    super::modals::FieldValue::Text(s) => s.clone(),
                    _ => String::new(),
                })
                .unwrap_or_default()
        };
        let choice = |name: &str| -> usize {
            values
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| match v {
                    super::modals::FieldValue::Choice(i) => *i,
                    _ => 0,
                })
                .unwrap_or(0)
        };
        let name = get("name").trim().to_owned();
        if name.is_empty() {
            cx.with_form(|f| f.error = Some("Display name is required".into()));
            return;
        }
        if name.chars().count() > 64 {
            cx.with_form(|f| f.error = Some("Display name is too long (64 max)".into()));
            return;
        }
        let provider = REGISTERABLE[choice("provider").min(3)];
        let source_idx = choice("source");
        // validate now (material level is the gate)
        let outcome = match self.run_validation(&values, w) {
            Ok(o) => o,
            Err(e) => {
                cx.with_form(move |f| f.error = Some(e));
                return;
            }
        };
        let Some(ctx) = self.form.as_ref() else {
            return;
        };
        let editing = ctx.editing.clone();
        let source = match source_idx {
            0 => CredentialSource::OnePassword(ctx.op_ref.clone().unwrap()),
            1 => {
                let path = get("folder").trim().to_owned();
                let detected = match provider::probe_folder(&path) {
                    provider::FolderProbe::Found(k) => k,
                    _ => DetectedKind::Unknown,
                };
                CredentialSource::LocalFolder { path, detected }
            }
            _ => {
                let key = get("key");
                CredentialSource::PlainApiKey {
                    fingerprint: fingerprint(&key),
                    tail: tail_of(&key),
                }
            }
        };
        if outcome.level.is_none() {
            let msg = outcome
                .issue
                .as_ref()
                .map(|i| i.message.clone())
                .unwrap_or("Validation failed".into());
            let note = Self::validation_note(&outcome);
            cx.with_form(move |f| {
                f.error = Some(msg);
                f.set_note("validation", note);
            });
            return;
        }
        // duplicate protection
        let probe = match &source {
            CredentialSource::OnePassword(r) => DuplicateProbe::OpReference {
                canonical: r.canonical(),
                account: r.account.clone(),
            },
            CredentialSource::LocalFolder { path, .. } => DuplicateProbe::Folder {
                provider,
                path: path.clone(),
            },
            CredentialSource::PlainApiKey { fingerprint, .. } => DuplicateProbe::KeyFingerprint {
                provider,
                fingerprint: fingerprint.clone(),
            },
            CredentialSource::HostEnv { .. } => DuplicateProbe::Folder {
                provider,
                path: String::new(),
            },
        };
        if let Some(dup) = w.accounts.find_duplicate(&probe)
            && Some(&dup.id) != editing.as_ref()
        {
            let msg = format!("Already registered: this source is used by {}", dup.title());
            cx.with_form(move |f| f.error = Some(msg));
            return;
        }
        if let Some(subject) = &outcome.identity.subject
            && let Some(dup) = w.accounts.find_duplicate(&DuplicateProbe::Identity {
                surface: provider.usage_surface(),
                subject: subject.clone(),
            })
            && Some(&dup.id) != editing.as_ref()
            && dup.origin == AccountOrigin::Registered
            && matches!(source, CredentialSource::LocalFolder { .. })
        {
            let msg = format!(
                "Already registered: {} authenticates the same identity",
                dup.title()
            );
            cx.with_form(move |f| f.error = Some(msg));
            return;
        }
        let name_taken = w.accounts.name_taken(provider, &name, editing.as_deref());
        let now = w.now_secs();
        let id = match &editing {
            Some(id) => id.clone(),
            None => format!(
                "acct-{}-{}",
                provider.short().to_lowercase().replace(['/', ' ', '.'], ""),
                name.to_lowercase()
                    .replace(|c: char| !c.is_ascii_alphanumeric(), "-")
            ),
        };
        let purpose = get("purpose").trim().to_owned();
        let endpoint = if provider.supports_endpoint() && source_idx != 1 {
            let e = get("endpoint").trim().to_owned();
            if e.is_empty() { None } else { Some(e) }
        } else {
            None
        };
        let first_for_provider = w
            .accounts
            .by_provider(provider)
            .filter(|a| a.origin == AccountOrigin::Registered)
            .count()
            == 0;
        // a discovered twin of the same folder is promoted rather than duplicated
        if let CredentialSource::LocalFolder { path, .. } = &source {
            let twin = w
                .accounts
                .accounts
                .iter()
                .find(|a| a.origin == AccountOrigin::Discovered && matches!(&a.source, CredentialSource::LocalFolder { path: p, .. } if p == path))
                .map(|a| a.id.clone());
            if let Some(t) = twin {
                w.accounts.remove(&t);
            }
        }
        let mut account = match editing.as_ref().and_then(|id| w.accounts.get(id).cloned()) {
            Some(mut a) => {
                a.display_name = name.clone();
                a.provider = provider;
                a.surface = provider.usage_surface();
                a.agent = provider.agent();
                a.source = source;
                a
            }
            None => Account::registered(&id, &name, provider, source),
        };
        account.purpose = if purpose.is_empty() {
            None
        } else {
            Some(purpose)
        };
        account.endpoint = endpoint.map(|host| crate::domain::account::Endpoint {
            label: "Grok Build".into(),
            host,
        });
        if account.endpoint.is_some() && !provider.supports_endpoint() {
            account.endpoint = None;
        }
        provider::apply_validation(&mut account, &outcome, now);
        if first_for_provider && editing.is_none() {
            account.default_for_provider = true;
        }
        let title = account.title();
        if editing.is_some() {
            if let Some(slot) = w.accounts.get_mut(&id) {
                *slot = account;
            }
            w.accounts.revision += 1;
        } else {
            w.accounts.insert(account);
        }
        self.form = None;
        cx.close();
        self.selected = Sel::Account(id.clone());
        self.folded.remove(&provider.usage_surface());
        if name_taken {
            cx.status(format!(
                "Saved {title} · name already used for {}",
                provider.short()
            ));
        } else {
            cx.status(format!("Saved {title}"));
        }
        cx.focus.focus(TREE);
    }

    fn remove(&mut self, id: &str, w: &World, cx: &mut Cx) {
        let Some(a) = w.accounts.get(id) else { return };
        if !a.mutations_allowed() {
            cx.status("Discovered accounts are read-only · nothing to remove");
            return;
        }
        let refs = w
            .workspaces
            .iter()
            .filter(|ws| {
                ws.account_overrides.values().any(|v| v == id)
                    || ws.role_account_overrides.values().any(|v| v == id)
            })
            .count();
        let body = format!(
            "{} is removed from the registry. {} Workspace or Role selections that point at it fall back to the provider default. The credential source itself is untouched.",
            a.title(),
            if refs == 0 {
                "No".to_owned()
            } else {
                refs.to_string()
            }
        );
        self.pending_remove = Some(id.to_owned());
        let d = Dialog::destructive(
            WidgetId::of("accounts.remove"),
            &format!("Remove account {}?", a.display_name),
            &body,
            "Remove",
        );
        cx.open(Modal::Dialog(d), ModalTag::new("remove").key(id));
    }

    fn toggle_enabled(&mut self, id: &str, w: &mut World, cx: &mut Cx) {
        let Some(a) = w.accounts.get_mut(id) else {
            return;
        };
        if !a.mutations_allowed() {
            cx.status("Discovered accounts are read-only");
            return;
        }
        a.enabled = !a.enabled;
        let (title, enabled, was_default) = (a.title(), a.enabled, a.default_for_provider);
        if !enabled && was_default {
            a.default_for_provider = false;
            cx.status(format!("Disabled {title} · provider default cleared"));
        } else {
            cx.status(format!(
                "{} {title}",
                if enabled { "Enabled" } else { "Disabled" }
            ));
        }
        w.accounts.revision += 1;
    }

    fn set_default(&mut self, id: &str, w: &mut World, cx: &mut Cx) {
        let title = w.accounts.get(id).map(|a| a.title()).unwrap_or_default();
        match w.accounts.set_default(id) {
            Ok(()) => {
                let p = w.accounts.get(id).map(|a| a.provider.short()).unwrap_or("");
                cx.status(format!("Default set: {p} → {title}"));
            }
            Err(e) => cx.error(format!("Cannot set default: {e}")),
        }
    }

    fn validate_account(&mut self, id: &str, w: &mut World, cx: &mut Cx) {
        let Some(a) = w.accounts.get_mut(id) else {
            return;
        };
        a.validation = ValidationState::Validating {
            started_tick: w.clock.now_ms as u64,
        };
        let title = a.title();
        w.schedule(
            900,
            Msg::AccountValidated {
                account: id.to_owned(),
            },
        );
        cx.status(format!("Validating {title}…"));
    }

    fn apply_validation_result(&mut self, id: &str, w: &mut World, cx: &mut Cx) {
        let now = w.now_secs();
        let Some(a) = w.accounts.get(id).cloned() else {
            return;
        };
        if let CredentialSource::PlainApiKey { fingerprint, .. } = &a.source {
            // no key material is stored, so re-validation can only confirm the
            // fingerprint and re-check the identity that the original entry produced
            let level = if a.usage.windows.iter().any(|w| w.has_meter()) {
                crate::domain::account::ValidationLevel::QuotaReadable
            } else if a.identity.subject.is_some() {
                crate::domain::account::ValidationLevel::IdentityAuthenticated
            } else {
                crate::domain::account::ValidationLevel::MaterialDiscovered
            };
            let short = fingerprint.chars().take(8).collect::<String>();
            if let Some(slot) = w.accounts.get_mut(id) {
                slot.validation = ValidationState::Valid(level);
                slot.last_refresh_secs = Some(now);
            }
            cx.status(format!(
                "{} · fingerprint {short} matches · {}",
                a.title(),
                level.label()
            ));
            return;
        }
        let outcome = provider::validate(a.provider, &a.source, None, &w.op, now);
        let Some(slot) = w.accounts.get_mut(id) else {
            return;
        };
        let keep_windows = slot.usage.windows.clone();
        provider::apply_validation(slot, &outcome, now);
        if slot.usage.windows.is_empty() {
            slot.usage.windows = keep_windows;
        }
        let word = match outcome.level {
            Some(l) => format!("Source validated · {}", l.label()),
            None => outcome
                .issue
                .as_ref()
                .map(|i| i.message.clone())
                .unwrap_or("Validation failed".into()),
        };
        if outcome.level.is_some() {
            cx.status(format!("{} · {word}", slot.title()));
        } else {
            cx.error(format!("{}: {word}", slot.title()));
        }
    }

    fn rebuild_actions(&mut self, w: &World) {
        let mk = |n: &str, l: &str, k: ButtonKind| Button::new(INSPECTOR.sub(n), l, k);
        self.actions = match &self.selected {
            Sel::Account(id) => match w.accounts.get(id) {
                Some(a) if a.mutations_allowed() => vec![
                    mk("refresh", "Refresh", ButtonKind::Secondary),
                    mk("validate", "Validate", ButtonKind::Secondary),
                    mk("edit", "Edit…", ButtonKind::Secondary),
                    mk("default", "Set default", ButtonKind::Secondary)
                        .disabled(a.default_for_provider || !a.enabled),
                    mk(
                        "toggle",
                        if a.enabled { "Disable" } else { "Enable" },
                        ButtonKind::Secondary,
                    ),
                    mk("remove", "Remove…", ButtonKind::Danger),
                ],
                Some(_) => vec![mk("refresh", "Refresh", ButtonKind::Secondary)],
                None => vec![],
            },
            Sel::Provider(_) => vec![
                mk("refresh", "Refresh provider", ButtonKind::Secondary),
                mk("add", "Add account…", ButtonKind::Secondary),
            ],
            Sel::Overview => vec![
                mk("refresh", "Refresh all", ButtonKind::Secondary),
                mk("add", "Add account…", ButtonKind::Secondary),
            ],
            Sel::Add => vec![mk("add", "Add account…", ButtonKind::Primary)],
        };
    }

    fn fire(&mut self, name: &str, w: &mut World, cx: &mut Cx) -> Outcome {
        let id = match &self.selected {
            Sel::Account(id) => Some(id.clone()),
            _ => None,
        };
        match (name, id) {
            ("refresh", _) => self.refresh_scope(w, cx),
            ("validate", Some(id)) => self.validate_account(&id, w, cx),
            ("edit", Some(id)) => {
                if let Some(a) = w.accounts.get(&id).cloned() {
                    self.open_form(Some(&a), w, cx);
                }
            }
            ("default", Some(id)) => self.set_default(&id, w, cx),
            ("toggle", Some(id)) => self.toggle_enabled(&id, w, cx),
            ("remove", Some(id)) => self.remove(&id, w, cx),
            ("add", _) => self.open_form(None, w, cx),
            _ => return Outcome::Consumed,
        }
        Outcome::Changed
    }

    // ------------------------------------------------------------- render

    fn draw_tree(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        let focused = ctx.interaction.focused(TREE);
        let summary = OverallSummary::compute(&w.accounts.accounts);
        let pos = scrollbar::position_label(&self.scroll);
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
        let title = match &self.filter {
            Some(f) if !f.is_empty() => format!("Accounts · filter {f}"),
            _ => "Accounts".into(),
        };
        let inner = Panel::framed(Some(&title))
            .focused(focused)
            .meta(&meta)
            .render(area, buf, t);
        self.tree_area = inner;
        let bg = t.canvas;
        self.scroll.set_viewport(inner.height as usize);
        if let Some(i) = self.cursor() {
            self.scroll.ensure_visible(i);
        }
        ctx.control(TREE, inner, false);
        ctx.scrollable(TREE, inner);
        let has_sb = self.scroll.overflows();
        let row_w = inner.width.saturating_sub(u16::from(has_sb));
        let cursor = self.cursor();
        let show_meta = row_w >= 40;
        for (k, i) in self.scroll.visible_range().enumerate() {
            let y = inner.y + k as u16;
            let row = &self.rows[i];
            let rid = TREE.child(i);
            let mut s = ctx.state(rid);
            s.focused = focused && cursor == Some(i);
            s.selected = cursor == Some(i);
            s.disabled = false;
            let st = t.row(s, bg);
            let rect = Rect::new(inner.x - 1, y, row_w + 1, 1);
            fill(buf, rect, st);
            buf.set_string(rect.x, y, "▎", t.gutter(s, st.bg.unwrap_or(bg), false));
            let mut x = rect.x + 2 + row.depth * 2;
            let glyph = if row.expandable {
                if row.expanded { "▾" } else { "▸" }
            } else if row.star {
                "★"
            } else {
                " "
            };
            let gs = if row.star {
                st.fg(if s.focused {
                    t.text_primary
                } else {
                    t.text_secondary
                })
            } else {
                st.fg(t.text_secondary).remove_modifier(Modifier::BOLD)
            };
            buf.set_string(x, y, glyph, gs);
            if row.expandable {
                ctx.clickable(TREE.child(i).sub("toggle"), Rect::new(x, y, 2, 1));
            }
            x += 2;
            let meta_w = if show_meta {
                width(&row.meta) as u16
            } else {
                0
            };
            let avail = rect.right().saturating_sub(x + 1);
            let hw: u16 = if row.health.is_some() { 2 } else { 0 };
            let lw = avail.saturating_sub(if meta_w > 0 { meta_w + 2 } else { 0 } + hw);
            let label_style = if row.faint {
                st.fg(t.text_faint)
            } else if row.sel == Sel::Add {
                st.fg(if s.focused {
                    t.text_primary
                } else {
                    t.text_secondary
                })
            } else if s.selected {
                st.fg(t.accent)
            } else {
                st
            };
            buf.set_string(
                x,
                y,
                fit(&truncate(&row.label, lw as usize), lw as usize),
                label_style,
            );
            if let Some((g, tone)) = row.health {
                buf.set_string(x + lw, y, g, st.fg(t.tone(tone)));
            }
            if meta_w > 0 && meta_w + 4 < avail {
                let ms = if row.faint {
                    st.fg(t.text_faint)
                } else {
                    st.fg(t.tone(row.meta_tone))
                };
                buf.set_string(
                    rect.right().saturating_sub(meta_w + 1),
                    y,
                    &row.meta,
                    ms.remove_modifier(Modifier::BOLD),
                );
            }
            ctx.clickable(rid, rect);
            if row.expandable {
                ctx.clickable(
                    TREE.child(i).sub("toggle"),
                    Rect::new(rect.x + 2 + row.depth * 2, y, 2, 1),
                );
            }
        }
        if has_sb {
            scrollbar::render_vertical(
                Rect::new(inner.right() - 1, inner.y, 1, inner.height),
                buf,
                ctx,
                TREE,
                &self.scroll,
                focused,
            );
        }
    }

    fn inspector_lines(&self, w: &World, width_: u16) -> (String, String, Vec<Line>) {
        let mut lines: Vec<Line> = vec![];
        match &self.selected {
            Sel::Overview => {
                let s = OverallSummary::compute(&w.accounts.accounts);
                lines.push(Line::Prop("Health".into(), format!("{} · {}", s.health.label(), s.issues_line()), match s.health {
                    crate::domain::usage::HealthWord::Blocked | crate::domain::usage::HealthWord::Degraded => Tone::Error,
                    crate::domain::usage::HealthWord::Attention => Tone::Warning,
                    _ => Tone::Normal,
                }));
                lines.push(Line::Prop("Registry".into(), s.counts_line(), Tone::Normal));
                let registered = w.accounts.accounts.iter().filter(|a| a.origin == AccountOrigin::Registered).count();
                lines.push(Line::Prop(
                    "Sources".into(),
                    format!(
                        "{registered} registered · {} discovered · 1Password {}",
                        w.accounts.accounts.len() - registered,
                        match w.op.session {
                            crate::sim::onepassword::OpSession::SignedIn => "available",
                            crate::sim::onepassword::OpSession::Locked => "locked",
                        }
                    ),
                    Tone::Normal,
                ));
                lines.push(Line::Blank);
                lines.push(Line::Heading("Comparable windows".into(), "identical provider, window and unit only".into()));
                if s.comparable.is_empty() {
                    lines.push(Line::Text("Nothing comparable: no provider has two accounts with the same window".into(), Tone::Muted));
                }
                for c in &s.comparable {
                    let mut text = format!("{} · {}   {} · {}–{}% remaining", c.surface.label(), c.label, plural(c.accounts, "account", "accounts"), c.min_remaining_pct, c.max_remaining_pct);
                    if let Some((u, l)) = c.summed {
                        text.push_str(&format!(" · {} / {} {}", junie_tui::ui::text::thousands(u as usize), junie_tui::ui::text::thousands(l as usize), c.unit.label()));
                    }
                    if c.last_good_count > 0 {
                        text.push_str(&format!(" ({} last good)", c.last_good_count));
                    }
                    lines.push(Line::Text(text, Tone::Secondary));
                }
                if !s.not_comparable.is_empty() {
                    lines.push(Line::Blank);
                    lines.push(Line::Heading("Not comparable".into(), String::new()));
                    let names: Vec<String> = s.not_comparable.iter().map(|n| format!("{} ({})", n.surface.label(), n.reason)).collect();
                    for l in junie_tui::ui::text::wrap(&names.join(" · "), width_ as usize) {
                        lines.push(Line::Text(l, Tone::Muted));
                    }
                }
                lines.push(Line::Blank);
                lines.push(Line::Heading("Warnings".into(), String::new()));
                let mut any = false;
                for a in w.accounts.sorted() {
                    if !a.enabled {
                        continue;
                    }
                    for win in &a.usage.windows {
                        match win.status {
                            QuotaStatus::Warning => {
                                any = true;
                                lines.push(Line::Text(format!("▲ {}   {} {}% used{}", a.title(), win.label, win.used_pct.unwrap_or(0), win.reset_secs.map(|r| format!(" · {}", w.clock.reset_label(r))).unwrap_or_default()), Tone::Warning));
                            }
                            QuotaStatus::Exhausted => {
                                any = true;
                                lines.push(Line::Text(format!("! {}   {} exhausted{}", a.title(), win.label, win.reset_secs.map(|r| format!(" · {}", w.clock.reset_label(r))).unwrap_or_default()), Tone::Error));
                            }
                            _ => {}
                        }
                    }
                    if let Some(i) = &a.issue
                        && !i.is_informational()
                        && i.code != IssueCode::QuotaUnsupported
                    {
                        any = true;
                        lines.push(Line::Text(format!("! {}   {}", a.title(), i.message), Tone::Error));
                    } else if a.usage.freshness.phase == Freshness::Stale {
                        any = true;
                        lines.push(Line::Text(format!("▲ {}   stale · last good {}", a.title(), a.usage.freshness.last_good_secs.map(|s| w.clock.ago(s)).unwrap_or("?".into())), Tone::Warning));
                    }
                }
                if !any {
                    lines.push(Line::Text("No warnings".into(), Tone::Muted));
                }
                let unresolved: Vec<&Account> = w.accounts.accounts.iter().filter(|a| a.enabled && a.identity.subject.is_none()).collect();
                if !unresolved.is_empty() {
                    lines.push(Line::Blank);
                    lines.push(Line::Heading("Unresolved identity".into(), String::new()));
                    for a in unresolved {
                        lines.push(Line::Text(format!("{} · {} · {} — not an authenticated account", a.title(), a.source.origin_label().to_lowercase(), a.confidence.label()), Tone::Muted));
                    }
                }
                ("Overview".into(), format!("{} · {}", plural(w.accounts.accounts.len(), "account", "accounts"), plural(s.counts.providers, "provider", "providers")), lines)
            }
            Sel::Provider(surface) => {
                let accounts: Vec<&Account> = w.accounts.sorted().into_iter().filter(|a| a.surface == *surface).collect();
                let p = surface.provider();
                lines.push(Line::Prop("Provider".into(), p.map(|p| p.label()).unwrap_or("—").into(), Tone::Normal));
                lines.push(Line::Prop("Agent runtime".into(), p.and_then(|p| p.agent()).map(|a| a.label()).unwrap_or("none").into(), Tone::Normal));
                lines.push(Line::Prop("Usage surface".into(), surface.surface_name().into(), Tone::Normal));
                lines.push(Line::Prop(
                    "Registration".into(),
                    if p.and_then(|p| p.agent()).is_some_and(|a| a.registerable()) {
                        "manual accounts allowed · 1Password, local folder or plain-text key".into()
                    } else {
                        "discovered only · read-only projection".into()
                    },
                    Tone::Secondary,
                ));
                if let Some(d) = accounts.iter().find(|a| a.default_for_provider) {
                    lines.push(Line::Prop("Default".into(), format!("★ {}", d.display_name), Tone::Normal));
                }
                lines.push(Line::Blank);
                if *surface == UsageSurface::Unsupported {
                    lines.push(Line::Text("Unsupported is the registry's explicit sentinel: a capability with no provider adapter. It is never synthesized as zero usage.".into(), Tone::Muted));
                } else if accounts.is_empty() {
                    lines.push(Line::Text("No accounts for this provider".into(), Tone::Muted));
                }
                for a in accounts {
                    lines.push(Line::Heading(a.display_name.clone(), account_meta(a, w).0));
                    for win in a.usage.windows.iter().take(2) {
                        if win.has_meter() {
                            lines.push(Line::Meter(win.label.clone(), win.used_pct.unwrap_or(0), meter_detail(win, w), meter_tone(win, a)));
                        } else {
                            lines.push(Line::Text(format!("{}   {}", win.label, win.value_label()), Tone::Muted));
                        }
                    }
                }
                (surface.surface_name().into(), format!("provider · {}", plural(w.accounts.by_provider(p.unwrap_or(Provider::Anthropic)).count(), "account", "accounts")), lines)
            }
            Sel::Account(id) => {
                let Some(a) = w.accounts.get(id) else {
                    return ("Account".into(), String::new(), lines);
                };
                let ws_users: Vec<String> = w
                    .workspaces
                    .iter()
                    .filter(|ws| ws.account_overrides.values().any(|v| v == id))
                    .map(|ws| ws.name.clone())
                    .collect();
                lines.push(Line::Prop("Provider".into(), a.provider.label().into(), Tone::Normal));
                lines.push(Line::Prop("Agent runtime".into(), a.agent.map(|x| x.label()).unwrap_or("none").into(), Tone::Normal));
                lines.push(Line::Prop("Usage surface".into(), a.surface.surface_name().into(), Tone::Normal));
                lines.push(Line::Prop("Identity".into(), a.identity.label(), if a.identity.subject.is_some() { Tone::Normal } else { Tone::Muted }));
                lines.push(Line::Prop("Plan".into(), a.identity.plan.clone().unwrap_or("unknown".into()), Tone::Normal));
                lines.push(Line::Prop("Credential".into(), format!("{} · {}", a.source.origin_label(), a.source.safe_detail()), Tone::Normal));
                match &a.source {
                    CredentialSource::OnePassword(r) => {
                        lines.push(Line::Prop(String::new(), format!("{} · {} · {}", r.account, r.canonical(), w.op.describe(r).map(|d| d.masked).unwrap_or("••••••••".into())), Tone::Muted));
                    }
                    CredentialSource::PlainApiKey { tail, .. } => lines.push(Line::Prop(String::new(), masked(tail), Tone::Muted)),
                    CredentialSource::LocalFolder { detected, .. } | CredentialSource::HostEnv { detected, .. } => {
                        lines.push(Line::Prop(String::new(), format!("detected {}", detected.label()), Tone::Muted));
                    }
                }
                if let Some(e) = &a.endpoint {
                    lines.push(Line::Prop("Endpoint".into(), format!("{} · {}", e.label, e.host), Tone::Normal));
                }
                lines.push(Line::Prop(
                    "Provenance".into(),
                    format!(
                        "{} · confidence {} · {}",
                        a.provenance.iter().map(|p| p.label()).collect::<Vec<_>>().join(", "),
                        a.confidence.label(),
                        a.origin_label()
                    ),
                    Tone::Secondary,
                ));
                lines.push(Line::Prop("Lifecycle".into(), a.lifecycle.label().into(), if a.lifecycle == Lifecycle::Available { Tone::Normal } else { Tone::Warning }));
                lines.push(Line::Prop(
                    "Default".into(),
                    if a.default_for_provider { format!("★ for {}", a.provider.short()) } else { "no".into() },
                    Tone::Normal,
                ));
                lines.push(Line::Prop("Enabled".into(), if a.enabled { "yes".into() } else { "no · disabled".into() }, if a.enabled { Tone::Normal } else { Tone::Muted }));
                if let Some(p) = &a.purpose {
                    lines.push(Line::Prop("Purpose".into(), p.clone(), Tone::Normal));
                }
                if !ws_users.is_empty() {
                    lines.push(Line::Prop("Used by".into(), format!("Workspace choice in {}", ws_users.join(", ")), Tone::Secondary));
                }
                lines.push(Line::Blank);
                let fresh = match a.usage.freshness.phase {
                    Freshness::Current => format!("current · refreshed {}", a.last_refresh_secs.map(|s| w.clock.ago(s)).unwrap_or("never".into())),
                    Freshness::Stale => format!("stale · last good {}", a.usage.freshness.last_good_secs.map(|s| w.clock.ago(s)).unwrap_or("?".into())),
                    Freshness::Refreshing => "refreshing…".into(),
                    Freshness::Failed => format!("failed · last good {}", a.usage.freshness.last_good_secs.map(|s| w.clock.ago(s)).unwrap_or("never".into())),
                };
                lines.push(Line::Heading("Quota".into(), fresh));
                if a.usage.windows.is_empty() {
                    lines.push(Line::Text("No quota windows · not started or unavailable".into(), Tone::Muted));
                }
                for win in &a.usage.windows {
                    if win.has_meter() {
                        lines.push(Line::Meter(win.label.clone(), win.used_pct.unwrap_or(0), meter_detail(win, w), meter_tone(win, a)));
                    } else {
                        let value = win.value_label();
                        let text = if value.to_lowercase().starts_with(&win.label.to_lowercase()) { value } else { format!("{}   {value}", win.label) };
                        lines.push(Line::Text(text, if win.status == QuotaStatus::Error { Tone::Error } else { Tone::Muted }));
                    }
                }
                lines.push(Line::Blank);
                let (m, i, q) = validation_marks(a);
                lines.push(Line::Prop("Validation".into(), format!("{m} material   {i} identity   {q} quota access · {}", a.validation.label()), Tone::Normal));
                if let Some(issue) = &a.issue {
                    let tone = if issue.is_informational() || issue.code == IssueCode::QuotaUnsupported { Tone::Warning } else { Tone::Error };
                    let mut text = issue.message.clone();
                    if let Some(d) = &issue.detail {
                        text.push_str(&format!(" · {d}"));
                    }
                    if let Some(r) = issue.retry_secs {
                        text.push_str(&format!(" · retry {}", w.clock.reset_label(r).replace("resets", "")));
                    }
                    lines.push(Line::Prop("Status".into(), format!("{} · {}", text, issue.recoverability.label()), tone));
                }
                if a.origin == AccountOrigin::Discovered {
                    lines.push(Line::Text("read-only: discovered on host · refresh only".into(), Tone::Muted));
                }
                (a.title(), format!("account · {}", a.status_word()), lines)
            }
            Sel::Add => (
                "Add account".into(),
                "form".into(),
                vec![
                    Line::Text("Register a Claude Code, Codex, Grok Build or OpenCode account.".into(), Tone::Secondary),
                    Line::Text("1Password item/field references are the primary credential path; a local agent folder or a masked plain-text key are the alternatives.".into(), Tone::Muted),
                ],
            ),
        }
    }

    fn draw_inspector(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        w: &World,
        as_drawer: bool,
    ) {
        let t = ctx.theme;
        let focused = ctx.interaction.focused(INSPECTOR)
            || self.actions.iter().any(|b| ctx.interaction.focused(b.id));
        self.rebuild_actions(w);
        let (title, scope, lines) = self.inspector_lines(w, area.width.saturating_sub(6));
        let inner = if as_drawer {
            Panel::framed(Some(&title))
                .focused(focused)
                .meta(&scope)
                .render(area, buf, t)
        } else {
            Panel::card(Some(&title))
                .focused(focused)
                .meta(&scope)
                .render(area, buf, t)
        };
        self.inspector_area = inner;
        let bg = if as_drawer { t.canvas } else { t.surface };
        let body_h = inner.height.saturating_sub(2);
        let body = Rect::new(inner.x, inner.y, inner.width, body_h);
        // wrap-aware line count
        let label_w: u16 = lines
            .iter()
            .map(|l| match l {
                Line::Prop(k, _, _) => width(k) as u16,
                Line::Meter(k, _, _, _) => width(k) as u16,
                _ => 0,
            })
            .max()
            .unwrap_or(13)
            .clamp(13, 22)
            + 1;
        let mut rendered: Vec<Line> = vec![];
        for l in lines {
            match l {
                Line::Prop(k, v, tone) => {
                    let vw = body.width.saturating_sub(label_w) as usize;
                    let parts = junie_tui::ui::text::wrap(&v, vw.max(8));
                    for (i, p) in parts.into_iter().enumerate() {
                        rendered.push(Line::Prop(
                            if i == 0 { k.clone() } else { String::new() },
                            p,
                            tone,
                        ));
                    }
                }
                Line::Text(v, tone) => {
                    for p in junie_tui::ui::text::wrap(&v, body.width as usize) {
                        rendered.push(Line::Text(p, tone));
                    }
                }
                other => rendered.push(other),
            }
        }
        self.inspector_scroll.set_content(rendered.len());
        self.inspector_scroll.set_viewport(body_h as usize);
        ctx.control(INSPECTOR, body, false);
        ctx.scrollable(INSPECTOR, body);
        let meter_w = if inner.width >= 70 {
            30
        } else if inner.width >= 50 {
            20
        } else {
            14
        };
        for (k, i) in self.inspector_scroll.visible_range().enumerate() {
            let y = body.y + k as u16;
            match &rendered[i] {
                Line::Prop(label, value, tone) => {
                    buf.set_string(body.x, y, label, t.muted().bg(bg));
                    buf.set_string(
                        body.x + label_w,
                        y,
                        truncate(value, body.width.saturating_sub(label_w) as usize),
                        Style::new().fg(t.tone(*tone)).bg(bg),
                    );
                }
                Line::Text(text, tone) => {
                    buf.set_string(
                        body.x,
                        y,
                        truncate(text, body.width as usize),
                        Style::new().fg(t.tone(*tone)).bg(bg),
                    );
                }
                Line::Heading(h, meta) => {
                    buf.set_string(
                        body.x,
                        y,
                        h,
                        t.secondary().bg(bg).add_modifier(Modifier::BOLD),
                    );
                    let room = body.width.saturating_sub(width(h) as u16 + 3) as usize;
                    let meta = truncate(meta, room);
                    let mw = width(&meta) as u16;
                    if mw > 0 && room >= 8 {
                        buf.set_string(body.right() - mw, y, &meta, t.faint().bg(bg));
                    }
                }
                Line::Meter(label, pct, value, tone) => {
                    buf.set_string(
                        body.x,
                        y,
                        truncate(label, (label_w - 1) as usize),
                        t.primary().bg(bg),
                    );
                    let mx = body.x + label_w;
                    let mw = meter_w.min(body.width.saturating_sub(label_w + 8));
                    let pct_text = format!("{pct:>3}%");
                    render_meter(
                        Rect::new(mx, y, mw + 6, 1),
                        buf,
                        ctx,
                        *pct,
                        &pct_text,
                        *tone,
                        bg,
                    );
                    let vx = mx + mw + 8;
                    if vx < body.right() {
                        buf.set_string(
                            vx,
                            y,
                            truncate(value, body.right().saturating_sub(vx) as usize),
                            t.muted().bg(bg),
                        );
                    }
                }
                Line::Blank => {}
            }
        }
        if self.inspector_scroll.overflows() {
            scrollbar::render_vertical(
                Rect::new(inner.right() - 1, body.y, 1, body_h),
                buf,
                ctx,
                INSPECTOR,
                &self.inspector_scroll,
                ctx.interaction.focused(INSPECTOR),
            );
        }
        let ay = inner.bottom().saturating_sub(1);
        let widths: Vec<u16> = self.actions.iter().map(|b| b.width()).collect();
        let rects = row_layout(Rect::new(inner.x - 1, ay, inner.width + 1, 1), &widths, 2);
        for (b, r) in self.actions.iter_mut().zip(rects) {
            b.render(r, buf, ctx, bg);
        }
    }

    fn draw_summary(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        let bg = t.surface;
        let (title, scope, lines) = self.inspector_lines(w, area.width.saturating_sub(4));
        let inner = Panel::card(Some(&title))
            .meta(&format!("{scope} · Tab details"))
            .render(area, buf, t);
        let mut y = inner.y;
        for l in lines.iter().take(inner.height as usize) {
            match l {
                Line::Prop(k, v, tone) if !k.is_empty() => {
                    buf.set_string(
                        inner.x,
                        y,
                        truncate(&format!("{k}  {v}"), inner.width as usize),
                        Style::new().fg(t.tone(*tone)).bg(bg),
                    );
                    y += 1;
                }
                Line::Meter(label, pct, value, tone) => {
                    buf.set_string(inner.x, y, truncate(label, 12), t.primary().bg(bg));
                    render_meter(
                        Rect::new(inner.x + 13, y, 22, 1),
                        buf,
                        ctx,
                        *pct,
                        &format!("{pct:>3}%"),
                        *tone,
                        bg,
                    );
                    buf.set_string(
                        inner.x + 37,
                        y,
                        truncate(value, inner.width.saturating_sub(37) as usize),
                        t.muted().bg(bg),
                    );
                    y += 1;
                }
                Line::Text(v, tone) => {
                    buf.set_string(
                        inner.x,
                        y,
                        truncate(v, inner.width as usize),
                        Style::new().fg(t.tone(*tone)).bg(bg),
                    );
                    y += 1;
                }
                _ => {}
            }
            if y >= inner.bottom() {
                break;
            }
        }
        ctx.clickable(INSPECTOR, area);
    }
}

enum Line {
    Prop(String, String, Tone),
    Text(String, Tone),
    Heading(String, String),
    Meter(String, u8, String, MeterTone),
    Blank,
}

/// Everything a meter row says besides the percentage the bar already shows.
pub fn meter_detail(win: &QuotaWindow, w: &World) -> String {
    let mut parts: Vec<String> = vec![];
    if let (Some(u), Some(l)) = (win.used, win.limit)
        && win.unit != WindowUnit::Percent
    {
        parts.push(format!(
            "{} / {} {}",
            thousands(u as usize),
            thousands(l as usize),
            win.unit.label()
        ));
    }
    if let Some(r) = win.reset_secs {
        parts.push(w.clock.reset_label(r));
    }
    if let Some(s) = &win.spend_label {
        parts.push(s.clone());
    }
    if win.status == QuotaStatus::Exhausted {
        parts.push("exhausted".into());
    }
    parts.join(" · ")
}

fn meter_tone(win: &QuotaWindow, a: &Account) -> MeterTone {
    if a.usage.freshness.phase != Freshness::Current {
        return MeterTone::Stale;
    }
    match win.status {
        QuotaStatus::Exhausted => MeterTone::Exhausted,
        QuotaStatus::Warning => MeterTone::Warning,
        _ => MeterTone::Normal,
    }
}

fn validation_marks(a: &Account) -> (&'static str, &'static str, &'static str) {
    use crate::domain::account::ValidationLevel as L;
    match &a.validation {
        ValidationState::Valid(L::QuotaReadable) => ("✓", "✓", "✓"),
        ValidationState::Valid(L::IdentityAuthenticated) => (
            "✓",
            "✓",
            if a.issue
                .as_ref()
                .is_some_and(|i| i.code == IssueCode::QuotaUnsupported)
            {
                "—"
            } else {
                "▲"
            },
        ),
        ValidationState::Valid(L::MaterialDiscovered) => ("✓", "—", "—"),
        ValidationState::Invalid(_) => ("!", "—", "—"),
        ValidationState::Validating { .. } => ("⠋", "…", "…"),
        ValidationState::NeverValidated => ("—", "—", "—"),
    }
}

/// Right-hand meta for a tree row: freshness word and age.
fn account_meta(a: &Account, w: &World) -> (String, Tone) {
    if !a.enabled {
        return ("disabled".into(), Tone::Faint);
    }
    match a.usage.freshness.phase {
        Freshness::Refreshing => (
            format!("{} refreshing", spinner_frame(w.now_ms() as u64 / 80)),
            Tone::Secondary,
        ),
        Freshness::Stale => (
            format!(
                "stale {}",
                a.usage
                    .freshness
                    .last_good_secs
                    .map(|s| w.clock.ago(s).replace(" ago", ""))
                    .unwrap_or("?".into())
            ),
            Tone::Warning,
        ),
        Freshness::Failed => (
            match a.issue.as_ref().map(|i| i.code) {
                Some(IssueCode::RateLimited) => "rate limited".into(),
                Some(IssueCode::ProviderUnavailable) => "unavailable".into(),
                Some(IssueCode::CredentialFileMissing) => "needs secret".into(),
                Some(IssueCode::Unauthorized) | Some(IssueCode::ApiKeyInvalid) => {
                    "unauthorized".into()
                }
                Some(IssueCode::OpLocked) => "1Password locked".into(),
                _ => "error".into(),
            },
            Tone::Error,
        ),
        Freshness::Current => match a.lifecycle {
            Lifecycle::Unsupported => ("unsupported".into(), Tone::Muted),
            Lifecycle::NeedsLogin => ("needs login".into(), Tone::Warning),
            Lifecycle::NeedsSecret => ("needs secret".into(), Tone::Warning),
            _ => (
                format!(
                    "current {}",
                    a.last_refresh_secs
                        .map(|s| w.clock.ago(s).replace(" ago", ""))
                        .unwrap_or("now".into())
                ),
                Tone::Muted,
            ),
        },
    }
}

impl Account {
    fn origin_label(&self) -> &'static str {
        match self.origin {
            AccountOrigin::Registered => "registered",
            AccountOrigin::Discovered => "discovered",
        }
    }
}

impl Screen for AccountsScreen {
    fn enter(&mut self, w: &mut World, cx: &mut Cx) {
        self.build_rows(w);
        cx.focus.focus(TREE);
        self.drawer_open = false;
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Some(TREE)
    }

    fn animating(&self, w: &World) -> bool {
        w.accounts.accounts.iter().any(|a| {
            a.usage.freshness.phase == Freshness::Refreshing
                || matches!(a.validation, ValidationState::Validating { .. })
        })
    }

    fn on_tick(&mut self, w: &mut World, _cx: &mut Cx) -> Outcome {
        self.build_rows(w);
        if self.animating(w) {
            Outcome::Changed
        } else {
            Outcome::Ignored
        }
    }

    fn on_msg(&mut self, msg: &Msg, w: &mut World, cx: &mut Cx) -> Outcome {
        match msg {
            Msg::AccountRefreshed { account } => {
                self.apply_refresh(account, w, cx);
                self.build_rows(w);
                Outcome::Changed
            }
            Msg::AccountValidated { account } => {
                self.apply_validation_result(account, w, cx);
                self.build_rows(w);
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    fn on_key(&mut self, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome {
        self.build_rows(w);
        let focus = cx.focus.current();
        let in_tree = focus == Some(TREE);
        let in_insp = focus == Some(INSPECTOR);
        let action_focus = self.actions.iter().position(|b| Some(b.id) == focus);
        if action_focus.is_none() {
            match key.code {
                KeyCode::Char('a') if key.plain() => {
                    self.open_form(None, w, cx);
                    return Outcome::Changed;
                }
                KeyCode::Char('r') if key.plain() => {
                    self.refresh_scope(w, cx);
                    return Outcome::Changed;
                }
                KeyCode::F(5) => {
                    let prev = self.selected.clone();
                    self.selected = Sel::Overview;
                    self.refresh_scope(w, cx);
                    self.selected = prev;
                    return Outcome::Changed;
                }
                KeyCode::Char('/') if key.plain() => {
                    let input = TextInput::new(WidgetId::of("accounts.filter.input"), "Filter")
                        .placeholder("name, provider, identity, status, plan")
                        .value(self.filter.as_deref().unwrap_or(""))
                        .plain_label();
                    let d = Dialog::prompt(
                        WidgetId::of("accounts.filter"),
                        "Filter accounts",
                        input,
                        "Apply",
                    );
                    cx.open(Modal::Dialog(d), ModalTag::new("filter"));
                    return Outcome::Changed;
                }
                KeyCode::Char('e') if key.plain() => {
                    if let Some(a) = self.selected_account(w).cloned() {
                        if a.mutations_allowed() {
                            self.open_form(Some(&a), w, cx);
                        } else {
                            cx.status("Discovered accounts are read-only · register a copy with a");
                        }
                        return Outcome::Changed;
                    }
                }
                KeyCode::Char('d') if key.plain() => {
                    if let Sel::Account(id) = self.selected.clone() {
                        self.toggle_enabled(&id, w, cx);
                        return Outcome::Changed;
                    }
                }
                KeyCode::Char('x') | KeyCode::Delete if key.plain() => {
                    if let Sel::Account(id) = self.selected.clone() {
                        self.remove(&id, w, cx);
                        return Outcome::Changed;
                    }
                }
                KeyCode::Char('v') if key.plain() => {
                    if let Sel::Account(id) = self.selected.clone() {
                        self.validate_account(&id, w, cx);
                        return Outcome::Changed;
                    }
                }
                KeyCode::Char('m') if key.plain() => {
                    let sel = match &self.selected {
                        Sel::Account(id) => Some(id.clone()),
                        _ => None,
                    };
                    cx.go(Go::Usage { select: sel });
                    return Outcome::Changed;
                }
                _ => {}
            }
        }
        if in_tree {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.move_cursor(-1);
                    return Outcome::Changed;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.move_cursor(1);
                    return Outcome::Changed;
                }
                KeyCode::PageUp => {
                    self.move_cursor(-(self.scroll.viewport_len.max(1) as isize));
                    return Outcome::Changed;
                }
                KeyCode::PageDown => {
                    self.move_cursor(self.scroll.viewport_len.max(1) as isize);
                    return Outcome::Changed;
                }
                KeyCode::Home | KeyCode::Char('g') => {
                    self.move_cursor(-(self.rows.len() as isize));
                    return Outcome::Changed;
                }
                KeyCode::End | KeyCode::Char('G') => {
                    self.move_cursor(self.rows.len() as isize);
                    return Outcome::Changed;
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    if let Sel::Provider(s) = self.selected {
                        if self.folded.remove(&s) {
                        } else {
                            self.move_cursor(1);
                        }
                        self.build_rows(w);
                    }
                    return Outcome::Changed;
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    match self.selected.clone() {
                        Sel::Provider(s) => {
                            self.folded.insert(s);
                        }
                        Sel::Account(id) => {
                            if let Some(a) = w.accounts.get(&id) {
                                self.selected = Sel::Provider(a.surface);
                            }
                        }
                        _ => {}
                    }
                    self.build_rows(w);
                    return Outcome::Changed;
                }
                KeyCode::Char('*') => {
                    self.folded.clear();
                    self.build_rows(w);
                    return Outcome::Changed;
                }
                KeyCode::Char('-') => {
                    for s in UsageSurface::ALL {
                        self.folded.insert(s);
                    }
                    self.build_rows(w);
                    return Outcome::Changed;
                }
                KeyCode::Char(' ') => {
                    match self.selected.clone() {
                        Sel::Provider(s) => {
                            if !self.folded.remove(&s) {
                                self.folded.insert(s);
                            }
                            self.build_rows(w);
                        }
                        Sel::Account(id) => self.set_default(&id, w, cx),
                        _ => {}
                    }
                    return Outcome::Changed;
                }
                KeyCode::Enter => {
                    match self.selected.clone() {
                        Sel::Add => self.open_form(None, w, cx),
                        Sel::Provider(s) => {
                            if !self.folded.remove(&s) {
                                self.folded.insert(s);
                            }
                            self.build_rows(w);
                        }
                        _ => {
                            self.drawer_open = true;
                            cx.focus.focus(INSPECTOR);
                        }
                    }
                    return Outcome::Changed;
                }
                KeyCode::Tab => {
                    self.drawer_open = true;
                    cx.focus.focus(INSPECTOR);
                    return Outcome::Changed;
                }
                KeyCode::Esc => {
                    if self.filter.is_some() {
                        self.filter = None;
                        self.build_rows(w);
                        cx.status("Filter cleared");
                        return Outcome::Changed;
                    }
                    cx.go(Go::Manager);
                    return Outcome::Changed;
                }
                _ => {}
            }
        }
        if in_insp {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.inspector_scroll.scroll_by(-1);
                    return Outcome::Changed;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.inspector_scroll.scroll_by(1);
                    return Outcome::Changed;
                }
                KeyCode::PageUp => {
                    self.inspector_scroll.page_up();
                    return Outcome::Changed;
                }
                KeyCode::PageDown => {
                    self.inspector_scroll.page_down();
                    return Outcome::Changed;
                }
                KeyCode::Esc | KeyCode::Left | KeyCode::BackTab => {
                    self.drawer_open = false;
                    cx.focus.focus(TREE);
                    return Outcome::Changed;
                }
                KeyCode::Tab | KeyCode::Enter => {
                    self.drawer_open = true;
                    if let Some(b) = self.actions.first() {
                        cx.focus.focus(b.id);
                    }
                    return Outcome::Changed;
                }
                _ => {}
            }
        }
        if let Some(i) = action_focus {
            let (o, fired) = self.actions[i].on_key(key);
            if fired {
                let id = self.actions[i].id;
                for n in [
                    "refresh", "validate", "edit", "default", "toggle", "remove", "add",
                ] {
                    if INSPECTOR.sub(n) == id {
                        return self.fire(n, w, cx);
                    }
                }
            }
            if o.consumed() {
                return o;
            }
            match key.code {
                KeyCode::Left | KeyCode::Char('h') => {
                    if i > 0 {
                        cx.focus.focus(self.actions[i - 1].id);
                    } else {
                        cx.focus.focus(INSPECTOR);
                    }
                    return Outcome::Changed;
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    if i + 1 < self.actions.len() {
                        cx.focus.focus(self.actions[i + 1].id);
                    }
                    return Outcome::Changed;
                }
                KeyCode::Esc => {
                    self.drawer_open = false;
                    cx.focus.focus(TREE);
                    return Outcome::Changed;
                }
                _ => {}
            }
        }
        Outcome::Ignored
    }

    fn on_click(&mut self, id: WidgetId, pos: Position, w: &mut World, cx: &mut Cx) -> Outcome {
        self.build_rows(w);
        for i in self.scroll.visible_range() {
            if TREE.child(i).sub("toggle") == id {
                if let Sel::Provider(s) = self.rows[i].sel {
                    self.selected = Sel::Provider(s);
                    if !self.folded.remove(&s) {
                        self.folded.insert(s);
                    }
                    self.build_rows(w);
                }
                cx.focus.focus(TREE);
                return Outcome::Changed;
            }
            if TREE.child(i) == id {
                let same = self.selected == self.rows[i].sel;
                self.selected = self.rows[i].sel.clone();
                self.inspector_scroll.jump_start();
                cx.focus.focus(TREE);
                self.drawer_open = false;
                if same && self.selected == Sel::Add {
                    self.open_form(None, w, cx);
                }
                return Outcome::Changed;
            }
        }
        if id == scrollbar::id_for(TREE) {
            let track = Rect::new(
                self.tree_area.right() - 1,
                self.tree_area.y,
                1,
                self.tree_area.height,
            );
            self.scroll
                .scroll_to(scrollbar::offset_for_click(track, pos, &self.scroll));
            return Outcome::Changed;
        }
        if id == scrollbar::id_for(INSPECTOR) {
            let track = Rect::new(
                self.inspector_area.right() - 1,
                self.inspector_area.y,
                1,
                self.inspector_area.height,
            );
            self.inspector_scroll.scroll_to(scrollbar::offset_for_click(
                track,
                pos,
                &self.inspector_scroll,
            ));
            return Outcome::Changed;
        }
        if id == INSPECTOR {
            cx.focus.focus(INSPECTOR);
            self.drawer_open = true;
            return Outcome::Changed;
        }
        for i in 0..self.actions.len() {
            if self.actions[i].id == id {
                cx.focus.focus(id);
                if self.actions[i].on_click() {
                    for n in [
                        "refresh", "validate", "edit", "default", "toggle", "remove", "add",
                    ] {
                        if INSPECTOR.sub(n) == id {
                            return self.fire(n, w, cx);
                        }
                    }
                }
                return Outcome::Changed;
            }
        }
        Outcome::Ignored
    }

    fn on_drag(&mut self, pressed: WidgetId, pos: Position, _w: &mut World) -> Outcome {
        if pressed == SEAM {
            let c = self.seam_container;
            return self.seam.on_drag(&mut self.split, c, 2, pos);
        }
        if pressed == scrollbar::id_for(TREE) {
            let track = Rect::new(
                self.tree_area.right() - 1,
                self.tree_area.y,
                1,
                self.tree_area.height,
            );
            self.scroll
                .scroll_to(scrollbar::offset_for_click(track, pos, &self.scroll));
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn on_wheel(&mut self, id: WidgetId, delta: i32, _pos: Position, _w: &mut World) -> Outcome {
        if id == TREE || id == scrollbar::id_for(TREE) {
            self.scroll.scroll_by(delta as isize);
            return Outcome::Changed;
        }
        if id == INSPECTOR || id == scrollbar::id_for(INSPECTOR) {
            self.inspector_scroll.scroll_by(delta as isize);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn form_changed(&mut self, tag: &ModalTag, form: &mut FormDialog, w: &World) {
        if tag.kind == "form" {
            Self::reveal(form, w);
            form.error = None;
            if let Some(ctx) = self.form.as_mut() {
                ctx.validation = None;
            }
            form.set_note("validation", vec![]);
        }
    }

    fn on_modal(
        &mut self,
        tag: &ModalTag,
        result: ModalResult,
        w: &mut World,
        cx: &mut Cx,
    ) -> Outcome {
        match (tag.kind, result) {
            ("form", ModalResult::FormAction(name, values)) => {
                match name.as_str() {
                    "choose:op" => {
                        let flow = OpFlow::new(WidgetId::of("accounts.op"), &w.op, w.now_ms());
                        cx.open(Modal::Op(flow), ModalTag::new("op"));
                    }
                    "choose:browse" => {
                        let b = FileBrowser::new(
                            WidgetId::of("accounts.browser"),
                            "Choose the local agent folder",
                            &w.home,
                            false,
                            true,
                            w,
                        );
                        cx.open(Modal::Browser(b), ModalTag::new("browse"));
                    }
                    "plain" => {
                        let w2 = w.op.clone();
                        cx.with_form(move |f| {
                            if let Some(field) = f.field_mut("source")
                                && let super::modals::FieldKindW::Radio(r) = &mut field.kind
                            {
                                r.selected = 2;
                                r.cursor = 2;
                            }
                            let _ = &w2;
                        });
                        cx.with_form(|f| {
                            // re-reveal with the changed radio
                            f.set_visible("op", false);
                            f.set_visible("op_note", false);
                            f.set_visible("key", true);
                            f.set_visible("folder", false);
                            f.set_visible("browse", false);
                            if let Some((_, b)) = f.actions.iter_mut().find(|(n, _)| n == "plain") {
                                b.disabled = true;
                            }
                        });
                        cx.status("Plain text chosen · the key is masked and never stored");
                    }
                    "validate" => match self.run_validation(&values, w) {
                        Ok(o) => {
                            let note = Self::validation_note(&o);
                            let ok = o.level.is_some();
                            if let Some(ctx) = self.form.as_mut() {
                                ctx.validation = Some(o);
                            }
                            cx.with_form(move |f| {
                                f.set_note("validation", note);
                                f.error = None;
                            });
                            cx.status(if ok {
                                "Validated · Save stores the reference, never the secret"
                            } else {
                                "Validation failed · fix the source and try again"
                            });
                        }
                        Err(e) => {
                            cx.with_form(move |f| f.error = Some(e));
                        }
                    },
                    "save" => self.save_form(values, w, cx),
                    _ => {}
                }
                Outcome::Changed
            }
            ("form", ModalResult::Form(None)) => {
                self.form = None;
                cx.status("Cancelled · nothing saved");
                cx.focus.focus(TREE);
                Outcome::Changed
            }
            ("form", ModalResult::Form(Some(values))) => {
                self.save_form(values, w, cx);
                Outcome::Changed
            }
            ("op", ModalResult::Op(Some(r))) => {
                let masked =
                    w.op.describe(&r)
                        .map(|d| d.masked)
                        .unwrap_or("••••••••".into());
                let path = r.display_path();
                let meta = format!("{} · {} · {}", r.account, r.canonical(), masked);
                if let Some(ctx) = self.form.as_mut() {
                    ctx.op_ref = Some(r.clone());
                    ctx.op_masked = Some(masked.clone());
                    ctx.validation = None;
                }
                let endpoint = w.op.endpoint_of(&r);
                cx.with_form(move |f| {
                    f.set_chooser("op", &path, None);
                    f.set_note(
                        "op_note",
                        vec![
                            (meta, Tone::Muted),
                            (
                                "only the reference is saved · the value is resolved at launch"
                                    .into(),
                                Tone::Faint,
                            ),
                        ],
                    );
                    f.set_note("validation", vec![]);
                    if let Some(e) = endpoint {
                        f.set_text(
                            "endpoint",
                            e.trim_start_matches("https://").trim_end_matches("/v1"),
                        );
                    }
                });
                cx.status("Reference chosen · Validate checks provider compatibility");
                Outcome::Changed
            }
            ("op", ModalResult::Op(None)) => Outcome::Changed,
            ("browse", ModalResult::Browser(BrowserResult::Chosen { path, .. })) => {
                let tilde = w.tilde(&path);
                cx.with_form(move |f| f.set_text("folder", &tilde));
                Outcome::Changed
            }
            (
                "remove",
                ModalResult::Dialog {
                    action: Some(1), ..
                },
            ) => {
                if let Some(a) = w.accounts.remove(&tag.key) {
                    for ws in w.workspaces.iter_mut() {
                        ws.account_overrides.retain(|_, v| *v != a.id);
                        ws.role_account_overrides.retain(|_, v| *v != a.id);
                    }
                    self.selected = Sel::Provider(a.surface);
                    cx.status(format!("Removed {}", a.title()));
                }
                self.build_rows(w);
                Outcome::Changed
            }
            (
                "filter",
                ModalResult::Dialog {
                    action: Some(1),
                    text,
                },
            ) => {
                self.filter = text.filter(|t| !t.trim().is_empty());
                self.build_rows(w);
                Outcome::Changed
            }
            (_, ModalResult::Cancelled)
            | (
                _,
                ModalResult::Dialog {
                    action: Some(0), ..
                },
            ) => {
                if tag.kind == "remove" {
                    cx.status("Cancelled · account kept");
                }
                Outcome::Changed
            }
            _ => Outcome::Changed,
        }
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        self.build_rows(w);
        let t = ctx.theme;
        self.narrow = area.width < 100;
        let focus = ctx.interaction.focus;
        if w.accounts.accounts.is_empty() && self.filter.is_none() {
            // empty state still lists Overview and Add
        }
        if self.narrow {
            let drawer = self.drawer_open
                || focus.is_some_and(|f| f == INSPECTOR || self.actions.iter().any(|b| b.id == f));
            let summary_h = 6u16.min(area.height / 3);
            let tree = Rect::new(
                area.x,
                area.y,
                area.width,
                area.height.saturating_sub(summary_h + 1),
            );
            self.draw_tree(tree, buf, ctx, w);
            if drawer {
                self.draw_inspector(area, buf, ctx, w, true);
            } else {
                let summary = Rect::new(area.x, tree.bottom() + 1, area.width, summary_h);
                self.draw_summary(summary, buf, ctx, w);
                ctx.control(INSPECTOR, Rect::ZERO, false);
            }
            return;
        }
        self.seam_container = area;
        let (left, right) = self.split.horizontal(area, 2);
        let handle = self.split.handle(SplitDir::Horizontal, area, 2);
        self.draw_tree(left, buf, ctx, w);
        self.seam.render(
            Rect::new(handle.x + 1, handle.y, 1, handle.height),
            buf,
            ctx,
            t.canvas,
        );
        self.draw_inspector(right, buf, ctx, w, false);
        if w.accounts.accounts.is_empty() && matches!(self.selected, Sel::Overview) {
            let e = EmptyState::new("No accounts registered").hint("a adds one · 1Password is the default credential source · discovered sources appear after a refresh");
            let inner = Rect::new(
                right.x + 2,
                right.y + 3,
                right.width.saturating_sub(4),
                right.height.saturating_sub(6),
            );
            fill(buf, inner, Style::new().bg(t.surface));
            empty::render(inner, buf, t, &e, t.surface);
        }
    }

    fn hints(&self, focus: Option<WidgetId>, w: &World) -> Vec<Hint> {
        if focus == Some(INSPECTOR) {
            return vec![
                hint("↑↓", "Scroll"),
                hint("Tab", "Actions"),
                hint("Esc", "Back"),
            ];
        }
        if self.actions.iter().any(|b| Some(b.id) == focus) {
            return vec![
                hint("← →", "Choose"),
                hint("Enter", "Run"),
                hint("Esc", "Back"),
            ];
        }
        let mut v = vec![];
        match &self.selected {
            Sel::Account(id) => {
                let reg = w.accounts.get(id).is_some_and(|a| a.mutations_allowed());
                v.push(hint("Enter", "Details"));
                v.push(hint("r", "Refresh"));
                if reg {
                    v.push(hint("e", "Edit…"));
                    v.push(hint("Space", "Default"));
                    v.push(hint("d", "Disable"));
                    v.push(hint("v", "Validate"));
                    v.push(hint("x", "Remove…"));
                }
            }
            Sel::Provider(_) => {
                v.push(hint("Space", "Fold"));
                v.push(hint("r", "Refresh provider"));
            }
            Sel::Overview => {
                v.push(hint("Enter", "Details"));
                v.push(hint("r", "Refresh all"));
            }
            Sel::Add => v.push(hint("Enter", "Add account…")),
        }
        v.push(hint("a", "Add…"));
        v.push(hint("/", "Filter"));
        v.push(hint("m", "Usage"));
        v.push(hint("Esc", "Back"));
        v
    }

    fn crumb(&self, w: &World) -> String {
        match &self.selected {
            Sel::Overview => "Accounts › Overview".into(),
            Sel::Provider(s) => format!("Accounts › {}", s.surface_name()),
            Sel::Account(id) => match w.accounts.get(id) {
                Some(a) => format!(
                    "Accounts › {} › {}",
                    a.surface.surface_name(),
                    a.display_name
                ),
                None => "Accounts".into(),
            },
            Sel::Add => "Accounts › new account".into(),
        }
    }

    fn strip_right(&self, w: &World) -> Vec<Segment> {
        let n = w
            .accounts
            .accounts
            .iter()
            .filter(|a| a.usage.freshness.phase == Freshness::Refreshing)
            .count();
        if n > 0 {
            vec![
                Segment::new(
                    format!("{} refreshing {n}", spinner_frame(w.now_ms() as u64 / 80)),
                    Tone::Secondary,
                )
                .priority(6),
            ]
        } else {
            vec![]
        }
    }

    fn is_editing(&self) -> bool {
        false
    }
}
