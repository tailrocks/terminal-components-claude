//! Deterministic fixture worlds, one per scenario, and the account
//! precedence resolver that every surface shares.

use std::collections::BTreeMap;

use crate::arbiter::{Arbiter, DiscoveryError};
use crate::clock::Clock;
use crate::domain::account::{
    Account, AccountId, AccountIdentity, AccountRegistry, Confidence, CredentialSource, DetectedKind,
    IdentitySubject, IssueCode, Lifecycle, Provenance, Recoverability, RecoverableIssue, ValidationLevel,
    ValidationState,
};
use crate::domain::agent::{Agent, AuthMode, Provider};
use crate::domain::instance::{DaemonSnapshot, Instance, InstanceStatus, ManifestError, SessionRecord, SessionStatus};
use crate::domain::onepassword::OpReference;
use crate::domain::usage::{AccountUsage, FreshnessInfo, QuotaStatus, QuotaWindow, WindowCategory, WindowUnit};
use crate::domain::workspace::{
    AllowedRoles, AuthEntry, AuthSource, EnvVar, Isolation, Mount, MountScope, RoleEntry, RolePolicy, RoleSource,
    Workspace,
};
use crate::scenario::Scenario;
use crate::sim::onepassword::{OpSession, SimOnePassword};
use crate::sim::pty::Daemon;
use crate::sim::world::{DaemonHealth, FsEntry, GithubRepo, GlobalConfig, TrustRow, World};
use junie_tui::ui::layout::SplitDir;

pub const HOME: &str = "/Users/alexey";

// ------------------------------------------------------------ precedence

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrecedenceLevel {
    Session,
    Role,
    Workspace,
    ProviderDefault,
    Discovered,
    None,
}

impl PrecedenceLevel {
    pub fn label(self) -> &'static str {
        match self {
            PrecedenceLevel::Session => "session choice",
            PrecedenceLevel::Role => "Role choice",
            PrecedenceLevel::Workspace => "Workspace choice",
            PrecedenceLevel::ProviderDefault => "provider default",
            PrecedenceLevel::Discovered => "discovered on host",
            PrecedenceLevel::None => "no account",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedAccount {
    pub account: Option<AccountId>,
    pub level: PrecedenceLevel,
    pub why: String,
    /// Lower levels that would have applied.
    pub shadowed: Vec<(PrecedenceLevel, AccountId)>,
}

impl ResolvedAccount {
    pub fn label(&self, registry: &AccountRegistry) -> String {
        match &self.account {
            Some(id) => registry
                .get(id)
                .map(|a| a.title())
                .unwrap_or_else(|| id.clone()),
            None => "no account".into(),
        }
    }
}

/// session choice › Role choice › Workspace choice › provider default ›
/// discovered current source. Disabled hits are skipped and recorded.
pub fn resolve_account(
    provider: Provider,
    ws: Option<&Workspace>,
    role: Option<&str>,
    session: Option<&AccountId>,
    registry: &AccountRegistry,
) -> ResolvedAccount {
    let mut candidates: Vec<(PrecedenceLevel, AccountId, String)> = vec![];
    if let Some(s) = session {
        candidates.push((
            PrecedenceLevel::Session,
            s.clone(),
            "Session choice · picked for this session in Capsule".into(),
        ));
    }
    if let (Some(w), Some(r)) = (ws, role)
        && let Some(id) = w.role_account_overrides.get(&(r.to_owned(), provider))
    {
        candidates.push((
            PrecedenceLevel::Role,
            id.clone(),
            format!("Role choice · set on Role {r} in {}", w.name),
        ));
    }
    if let Some(w) = ws
        && let Some(id) = w.account_overrides.get(&provider)
    {
        candidates.push((
            PrecedenceLevel::Workspace,
            id.clone(),
            format!("Workspace choice · set in {} › Auth", w.name),
        ));
    }
    if let Some(a) = registry.default_for(provider) {
        candidates.push((
            PrecedenceLevel::ProviderDefault,
            a.id.clone(),
            "Provider default · set in Account & Usage Center".into(),
        ));
    }
    if let Some(a) = registry.discovered_current(provider) {
        candidates.push((
            PrecedenceLevel::Discovered,
            a.id.clone(),
            format!(
                "Discovered on host · no explicit choice for {}",
                provider.short()
            ),
        ));
    }
    let mut shadowed = vec![];
    let mut chosen: Option<(PrecedenceLevel, AccountId, String)> = None;
    for (level, id, why) in candidates {
        let usable = registry.get(&id).is_some_and(|a| a.enabled);
        if chosen.is_none() {
            if usable {
                chosen = Some((level, id, why));
            } else {
                shadowed.push((level, format!("{id} (disabled)")));
            }
        } else {
            shadowed.push((level, id));
        }
    }
    match chosen {
        Some((level, id, mut why)) => {
            if let Some((l, other)) = shadowed.first() {
                let other_label = registry
                    .get(other.trim_end_matches(" (disabled)"))
                    .map(|a| a.display_name.clone())
                    .unwrap_or_else(|| other.clone());
                if other.ends_with("(disabled)") {
                    why.push_str(&format!(" · {} {other_label} skipped: disabled", l.label()));
                } else {
                    why.push_str(&format!(" · overrides {} {other_label}", l.label()));
                }
            }
            ResolvedAccount {
                account: Some(id),
                level,
                why,
                shadowed,
            }
        }
        None => ResolvedAccount {
            account: None,
            level: PrecedenceLevel::None,
            why: "No account: register one in Account & Usage Center".into(),
            shadowed,
        },
    }
}

// --------------------------------------------------------------- helpers

fn op_ref(account: &str, vault: (&str, &str), item: (&str, &str), field: &str) -> OpReference {
    OpReference {
        account: account.into(),
        vault_id: vault.0.into(),
        vault_name: vault.1.into(),
        item_id: item.0.into(),
        item_title: item.1.into(),
        section: None,
        field_id: field.into(),
        field_label: field.into(),
    }
}

fn handle(s: &str) -> Option<IdentitySubject> {
    Some(IdentitySubject::Handle(s.into()))
}

fn roles() -> Vec<RoleEntry> {
    let git = |name: &str, desc: &str, trusted: bool| RoleEntry {
        namespace: "chainargos".into(),
        name: name.into(),
        source: RoleSource::Git {
            url: "github.com/chainargos/roles".into(),
            branch: "main".into(),
        },
        trusted,
        in_registry: true,
        description: desc.into(),
        load_error: None,
    };
    vec![
        git("the-architect", "Full-stack design and refactoring; Claude Code default", true),
        git("backend", "Rust and Postgres services", true),
        git("reviewer", "Read-mostly code review with limited write scope", true),
        git("sre", "Infrastructure, Terraform, Kubernetes", true),
        RoleEntry {
            namespace: "acme-labs".into(),
            name: "data-eng".into(),
            source: RoleSource::Git {
                url: "github.com/acme-labs/roles-experimental".into(),
                branch: "next".into(),
            },
            trusted: false,
            in_registry: true,
            description: "Notebook and pipeline tooling (untrusted source)".into(),
            load_error: Some("trust required".into()),
        },
        RoleEntry {
            namespace: "local".into(),
            name: "writer".into(),
            source: RoleSource::Local {
                path: "~/roles/writer".into(),
            },
            trusted: true,
            in_registry: false,
            description: "Docs and release notes".into(),
            load_error: None,
        },
    ]
}

fn workspaces(rich: bool) -> Vec<Workspace> {
    let mut v = vec![];
    let mut w = Workspace::new(1, "payments-platform", "/workspace/payments-platform");
    w.mounts = vec![
        Mount::host("~/src/payments-platform", "/workspace/payments-platform")
            .repository()
            .isolation(Isolation::Worktree),
        Mount::host("~/src/shared-libs", "/workspace/libs").readonly(true),
    ];
    w.roles = RolePolicy {
        allowed: AllowedRoles::Custom(vec!["the-architect".into(), "backend".into(), "reviewer".into()]),
        default: Some("the-architect".into()),
        last: Some("the-architect".into()),
    };
    w.env = vec![
        EnvVar::plain("DATABASE_URL", "postgres://payments:pw-fixture-only@db.internal:5432/payments"),
        EnvVar::op(
            "STRIPE_KEY",
            op_ref("chainargos.1password.com", ("v_eng01", "Engineering"), ("it_str01", "Stripe · sandbox"), "credential"),
        ),
        EnvVar::plain("LOG_LEVEL", "debug"),
        EnvVar::host("GH_TOKEN", "GH_TOKEN"),
    ];
    w.role_env.insert(
        "backend".into(),
        vec![EnvVar::op(
            "OPENAI_API_KEY",
            op_ref("chainargos.1password.com", ("v_eng01", "Engineering"), ("it_cdx01", "OpenAI · Codex Primary"), "credential"),
        )],
    );
    w.auth = vec![AuthEntry {
        agent: Agent::ClaudeCode,
        mode: AuthMode::ApiKey,
        source: AuthSource::Account("acct-claude-work".into()),
    }];
    w.account_overrides.insert(Provider::Anthropic, "acct-claude-work".into());
    w.keep_awake = true;
    v.push(w);

    let mut w = Workspace::new(2, "infra-control-plane", "/workspace/infra-control-plane");
    w.mounts = vec![Mount::host("~/src/infra-control-plane", "/workspace/infra-control-plane").repository()];
    w.roles = RolePolicy {
        allowed: AllowedRoles::All,
        default: Some("sre".into()),
        last: Some("sre".into()),
    };
    w.env = vec![
        EnvVar::op(
            "CLOUDFLARE_API_TOKEN",
            op_ref("chainargos.1password.com", ("v_eng01", "Engineering"), ("it_cf01", "Cloudflare · infra"), "credential"),
        ),
        EnvVar::plain("TF_LOG", "WARN"),
    ];
    w.account_overrides.insert(Provider::OpenAi, "acct-codex-experiments".into());
    v.push(w);

    let mut w = Workspace::new(3, "release-automation", "/workspace/release-automation");
    w.mounts = vec![Mount::git("github.com/chainargos/release-automation", "/workspace/release-automation")];
    w.roles = RolePolicy {
        allowed: AllowedRoles::Custom(vec!["backend".into()]),
        default: Some("backend".into()),
        last: None,
    };
    w.git_pull = false;
    v.push(w);

    let mut w = Workspace::new(4, "customer-portal", "/workspace/customer-portal");
    w.mounts = vec![
        Mount::host("~/src/customer-portal", "/workspace/customer-portal").repository(),
        Mount::host("~/design/portal-assets", "/workspace/assets").readonly(true),
    ];
    w.roles = RolePolicy {
        allowed: AllowedRoles::All,
        default: None,
        last: Some("writer".into()),
    };
    w.env = vec![EnvVar::plain("NEXT_PUBLIC_API", "https://api.portal.local")];
    v.push(w);

    if rich {
        let names = [
            "data-pipeline",
            "docs-site",
            "auth-service",
            "gateway",
            "mobile-app",
            "ml-notebooks",
            "search-indexer",
            "staging-env",
            "billing-reconciliation-service-with-a-very-long-name-for-truncation",
            "legacy-monolith",
        ];
        for (i, n) in names.iter().enumerate() {
            let mut w = Workspace::new(10 + i as u32, n, &format!("/workspace/{n}"));
            let src = if i == 8 {
                "~/src/enterprise/platform/services/billing/reconciliation/billing-reconciliation-service-with-a-very-long-name-for-truncation".to_owned()
            } else {
                format!("~/src/{n}")
            };
            w.mounts = vec![Mount::host(&src, &format!("/workspace/{n}")).repository()];
            v.push(w);
        }
    }
    v
}

fn instance(
    id: &str,
    ws: Option<u32>,
    wsname: &str,
    role: &str,
    agent: Agent,
    status: InstanceStatus,
    created: i64,
    last_seen: i64,
) -> Instance {
    Instance {
        id: id.into(),
        container: format!("jackin-{wsname}"),
        workspace: ws,
        workdir: format!("/workspace/{wsname}"),
        role: role.into(),
        agent,
        status,
        created_secs: created,
        last_seen_secs: last_seen,
        run_id: format!("run-{}", &id[3..]),
        sessions: Ok(vec![]),
        daemon: DaemonSnapshot::Unavailable,
        branch: None,
        pr: None,
        default_branch: "main".into(),
        uncommitted: 0,
        unpushed: 0,
    }
}

fn global(rich: bool) -> GlobalConfig {
    GlobalConfig {
        coauthor_trailer: true,
        dco_signoff: true,
        mounts: if rich {
            vec![
                Mount::host("~/.gitconfig", "/home/agent/.gitconfig")
                    .readonly(true)
                    .scope(MountScope::Global),
                Mount::host("~/.cache/cargo-registry", "/home/agent/.cargo/registry").scope(MountScope::Global),
                Mount::host("~/roles/sre-kube", "/home/agent/.kube")
                    .readonly(true)
                    .scope(MountScope::Role("sre".into())),
            ]
        } else {
            vec![]
        },
        env: if rich {
            vec![
                EnvVar::op(
                    "GH_TOKEN",
                    op_ref("chainargos.1password.com", ("v_eng01", "Engineering"), ("it_gh01", "GitHub · CLI token"), "credential"),
                ),
                EnvVar::plain("EDITOR", "nvim"),
                EnvVar::plain("CARGO_NET_GIT_FETCH_WITH_CLI", "true"),
            ]
        } else {
            vec![]
        },
        role_env: if rich {
            let mut m = BTreeMap::new();
            m.insert("sre".to_owned(), vec![EnvVar::plain("KUBECONFIG", "/home/agent/.kube/config")]);
            m
        } else {
            BTreeMap::new()
        },
        auth: if rich {
            vec![
                AuthEntry {
                    agent: Agent::ClaudeCode,
                    mode: AuthMode::Sync,
                    source: AuthSource::Account("acct-claude-personal".into()),
                },
                AuthEntry {
                    agent: Agent::Codex,
                    mode: AuthMode::ApiKey,
                    source: AuthSource::Account("acct-codex-primary".into()),
                },
                AuthEntry {
                    agent: Agent::GrokBuild,
                    mode: AuthMode::ApiKey,
                    source: AuthSource::Account("acct-grok-team".into()),
                },
                AuthEntry {
                    agent: Agent::OpenCode,
                    mode: AuthMode::Sync,
                    source: AuthSource::Folder("~/.local/share/opencode".into()),
                },
                AuthEntry {
                    agent: Agent::Amp,
                    mode: AuthMode::Sync,
                    source: AuthSource::HostProfile,
                },
                AuthEntry {
                    agent: Agent::KimiCode,
                    mode: AuthMode::Ignore,
                    source: AuthSource::None,
                },
            ]
        } else {
            vec![]
        },
        role_auth: if rich {
            let mut m = BTreeMap::new();
            m.insert(
                "backend".to_owned(),
                vec![AuthEntry {
                    agent: Agent::ClaudeCode,
                    mode: AuthMode::ApiKey,
                    source: AuthSource::Account("acct-claude-work".into()),
                }],
            );
            m
        } else {
            BTreeMap::new()
        },
        trust: vec![
            TrustRow {
                source: "github.com/chainargos/roles".into(),
                kind: "git",
                trusted: true,
                roles: 4,
            },
            TrustRow {
                source: "github.com/acme-labs/roles-experimental".into(),
                kind: "git",
                trusted: false,
                roles: 1,
            },
            TrustRow {
                source: "~/roles".into(),
                kind: "path",
                trusted: true,
                roles: 1,
            },
            TrustRow {
                source: "git@corp:infra/roles".into(),
                kind: "git",
                trusted: true,
                roles: 2,
            },
        ],
    }
}

fn fs() -> Vec<FsEntry> {
    let d = |p: &str, git: Option<&str>, meta: &str| FsEntry {
        path: p.into(),
        dir: true,
        git: git.map(str::to_owned),
        meta: meta.into(),
    };
    let f = |p: &str, meta: &str| FsEntry {
        path: p.into(),
        dir: false,
        git: None,
        meta: meta.into(),
    };
    vec![
        d("/Users/alexey", None, ""),
        d("/Users/alexey/src", None, "12 items"),
        d("/Users/alexey/design", None, "2 items"),
        d("/Users/alexey/roles", None, "1 item"),
        d("/Users/alexey/notes", None, "9 items"),
        d("/Users/alexey/.claude", None, "profile"),
        d("/Users/alexey/.codex", None, "profile"),
        d("/Users/alexey/.local/share/opencode", None, "profile"),
        d("/Users/alexey/.grok", None, "empty"),
        d("/Users/alexey/.codex-locked", None, "no access"),
        d("/Users/alexey/.opencode-broken", None, "profile"),
        f("/Users/alexey/notes.md", "2 d"),
        d("/Users/alexey/src/payments-platform", Some("feature/settlement-backoff"), "git"),
        d("/Users/alexey/src/payments-platform/crates", None, "6 items"),
        d("/Users/alexey/src/payments-platform/crates/settlement", None, "rust"),
        d("/Users/alexey/src/payments-platform/crates/ledger", None, "rust"),
        d("/Users/alexey/src/payments-platform/docs", None, "adr"),
        d("/Users/alexey/src/payments-platform/scripts", None, "3 items"),
        f("/Users/alexey/src/payments-platform/Cargo.toml", "1 h"),
        f("/Users/alexey/src/payments-platform/README.md", "3 d"),
        d("/Users/alexey/src/infra-control-plane", Some("main"), "git"),
        d("/Users/alexey/src/infra-control-plane/modules", None, "terraform"),
        d("/Users/alexey/src/infra-control-plane/kube", None, "go"),
        d("/Users/alexey/src/release-automation", Some("main"), "git"),
        d("/Users/alexey/src/customer-portal", Some("develop"), "git"),
        d("/Users/alexey/src/customer-portal/web", None, "next.js"),
        d("/Users/alexey/src/customer-portal/services", None, "5 items"),
        d("/Users/alexey/src/shared-libs", Some("main"), "git"),
        d("/Users/alexey/src/data-pipeline", Some("main"), "git"),
        d("/Users/alexey/src/docs-site", Some("gh-pages"), "git"),
        d("/Users/alexey/src/scratch", None, "4 items"),
        d("/Users/alexey/src/enterprise", None, "1 item"),
        d("/Users/alexey/design/portal-assets", None, "assets"),
        d("/Users/alexey/design/brand", None, "assets"),
        d("/Users/alexey/roles/writer", None, "manifest"),
    ]
}

fn github() -> Vec<GithubRepo> {
    let r = |n: &str, b: &str, extra: &[&str], upd: &str| GithubRepo {
        full_name: n.into(),
        default_branch: b.into(),
        branches: std::iter::once(b.to_owned())
            .chain(extra.iter().map(|s| (*s).to_owned()))
            .collect(),
        updated: upd.into(),
        url: format!("https://github.com/{n}"),
    };
    vec![
        r("chainargos/payments-platform", "main", &["feature/settlement-backoff", "release/2026.09"], "1 h ago"),
        r("chainargos/infra-control-plane", "main", &["sre/node-pools"], "3 h ago"),
        r("chainargos/release-automation", "main", &["node-22"], "2 d ago"),
        r("chainargos/customer-portal", "develop", &["main", "feature/skeletons"], "5 h ago"),
        r("chainargos/roles", "main", &[], "6 d ago"),
        r("chainargos/docs", "main", &["gh-pages"], "3 d ago"),
        r("acme-labs/roles-experimental", "next", &[], "2 mo ago"),
    ]
}

// --------------------------------------------------------------- accounts

fn accounts_mixed(now: i64) -> AccountRegistry {
    let h = 3600;
    let d = 86_400;
    let mut r = AccountRegistry::default();

    let mut a = Account::registered(
        "acct-claude-personal",
        "Personal",
        Provider::Anthropic,
        CredentialSource::LocalFolder {
            path: "~/.claude".into(),
            detected: DetectedKind::ClaudeOAuthProfile,
        },
    );
    a.identity = AccountIdentity {
        subject: handle("alexey@donbeave.dev"),
        plan: Some("Max 5x".into()),
    };
    a.confidence = Confidence::Authoritative;
    a.lifecycle = Lifecycle::Available;
    a.purpose = Some("personal".into());
    a.default_for_provider = true;
    a.validation = ValidationState::Valid(ValidationLevel::QuotaReadable);
    a.last_refresh_secs = Some(now - 4 * 60);
    a.usage = AccountUsage {
        freshness: FreshnessInfo::current(now - 4 * 60),
        windows: vec![
            QuotaWindow::pct("session", "Session · 5-hour", WindowCategory::Session, 38).reset(now + 3 * h + 12 * 60),
            QuotaWindow::pct("weekly", "Weekly · all models", WindowCategory::LongRange, 33).reset(now + 3 * d + 10 * h),
            QuotaWindow::pct("weekly_sonnet", "Weekly · Sonnet", WindowCategory::Model, 21).reset(now + 3 * d + 10 * h),
            QuotaWindow::pct("weekly_opus", "Weekly · Opus", WindowCategory::Model, 54).reset(now + 3 * d + 10 * h),
        ],
    };
    r.insert(a);

    let mut a = Account::registered(
        "acct-claude-work",
        "Work",
        Provider::Anthropic,
        CredentialSource::OnePassword(op_ref(
            "chainargos.1password.com",
            ("v_eng01", "Engineering"),
            ("it_ant01", "Anthropic · Work"),
            "credential",
        )),
    );
    a.identity = AccountIdentity {
        subject: handle("alexey@chainargos.com"),
        plan: Some("Team".into()),
    };
    a.confidence = Confidence::Authoritative;
    a.lifecycle = Lifecycle::Available;
    a.purpose = Some("work".into());
    a.validation = ValidationState::Valid(ValidationLevel::QuotaReadable);
    a.last_refresh_secs = Some(now - 47 * 60);
    a.issue = Some(
        RecoverableIssue::new(IssueCode::Stale, "Usage stale · last good 47 min ago", Recoverability::Retryable)
            .retry(now + 13 * 60),
    );
    a.usage = AccountUsage {
        freshness: FreshnessInfo::stale(now - 47 * 60, now + 13 * 60),
        windows: vec![
            QuotaWindow::pct("session", "Session · 5-hour", WindowCategory::Session, 76).reset(now + h + 5 * 60),
            QuotaWindow::pct("weekly", "Weekly · all models", WindowCategory::LongRange, 88).reset(now + 3 * d + 10 * h),
            QuotaWindow::pct("weekly_opus", "Weekly · Opus", WindowCategory::Model, 100).reset(now + 3 * d + 10 * h),
            QuotaWindow::counted("credits", "Extra usage credits", WindowCategory::Other, WindowUnit::Usd, 1_420, 5_000)
                .spend("$14.20 of $50.00"),
        ],
    };
    r.insert(a);

    let mut a = Account::registered(
        "acct-codex-primary",
        "Primary",
        Provider::OpenAi,
        CredentialSource::OnePassword(op_ref(
            "chainargos.1password.com",
            ("v_eng01", "Engineering"),
            ("it_cdx01", "OpenAI · Codex Primary"),
            "credential",
        )),
    );
    a.identity = AccountIdentity {
        subject: handle("ChatGPT account org_7Hq2"),
        plan: Some("Pro 20x".into()),
    };
    a.confidence = Confidence::Authoritative;
    a.lifecycle = Lifecycle::Available;
    a.purpose = Some("work".into());
    a.default_for_provider = true;
    a.validation = ValidationState::Valid(ValidationLevel::QuotaReadable);
    a.last_refresh_secs = Some(now - 2 * 60);
    a.usage = AccountUsage {
        freshness: FreshnessInfo::current(now - 2 * 60),
        windows: vec![
            QuotaWindow::pct("session", "Session · 5-hour", WindowCategory::Session, 12).reset(now + 4 * h + 40 * 60),
            QuotaWindow::pct("weekly", "Weekly · 7-day", WindowCategory::LongRange, 59).reset(now + 2 * d + 19 * h),
            QuotaWindow::not_started("spark", "Codex Spark · 5-hour", WindowCategory::Model),
            QuotaWindow::counted("credits", "Credits", WindowCategory::Other, WindowUnit::Credits, 1_240, 5_000),
        ],
    };
    r.insert(a);

    let mut a = Account::registered(
        "acct-codex-experiments",
        "Experiments",
        Provider::OpenAi,
        CredentialSource::PlainApiKey {
            fingerprint: "7f3a91c2".into(),
            tail: "k7Qz".into(),
        },
    );
    a.identity = AccountIdentity {
        subject: handle("ChatGPT account org_7Hq2"),
        plan: Some("Plus".into()),
    };
    a.confidence = Confidence::Estimated;
    a.lifecycle = Lifecycle::Available;
    a.purpose = Some("experiments".into());
    a.validation = ValidationState::Valid(ValidationLevel::IdentityAuthenticated);
    a.last_refresh_secs = Some(now - 20 * 60);
    a.issue = Some(RecoverableIssue::new(
        IssueCode::QuotaUnsupported,
        "Quota not visible: OpenAI does not expose usage for API keys",
        Recoverability::Unsupported,
    ));
    a.usage = AccountUsage {
        freshness: FreshnessInfo::refreshing(Some(now - 20 * 60)),
        windows: vec![
            QuotaWindow::pct("session", "Session · 5-hour", WindowCategory::Session, 4).reset(now + 4 * h),
            QuotaWindow::pct("weekly", "Weekly · 7-day", WindowCategory::LongRange, 12).reset(now + 2 * d + 19 * h),
            QuotaWindow::counted("credits", "Credits", WindowCategory::Other, WindowUnit::Credits, 90, 500),
            QuotaWindow::sentinel("quota", "Quota", QuotaStatus::Unsupported, "Quota not visible for API keys"),
        ],
    };
    r.insert(a);

    let mut a = Account::registered(
        "acct-grok-team",
        "Team",
        Provider::XAi,
        CredentialSource::OnePassword(op_ref(
            "chainargos.1password.com",
            ("v_eng01", "Engineering"),
            ("it_grk01", "xAI · Grok Team"),
            "credential",
        )),
    )
    .with_endpoint("Grok Build (default)", "api.x.ai");
    a.identity = AccountIdentity {
        subject: handle("team_chainargos"),
        plan: Some("Team · prepaid".into()),
    };
    a.confidence = Confidence::Authoritative;
    a.lifecycle = Lifecycle::Available;
    a.purpose = Some("shared".into());
    a.default_for_provider = true;
    a.validation = ValidationState::Valid(ValidationLevel::QuotaReadable);
    a.last_refresh_secs = Some(now - 9 * 60);
    a.usage = AccountUsage {
        freshness: FreshnessInfo::current(now - 9 * 60),
        windows: vec![
            QuotaWindow::pct("monthly", "Monthly", WindowCategory::LongRange, 31).reset(now + 27 * d + 10 * h),
            QuotaWindow::pct("weekly", "Weekly", WindowCategory::LongRange, 68).reset(now + 3 * d + 10 * h),
            QuotaWindow::counted("credits", "Credits", WindowCategory::Other, WindowUnit::Usd, 3_140, 10_000)
                .spend("$68.60 remaining of $100.00"),
            QuotaWindow::counted("ondemand", "On-demand usage", WindowCategory::Other, WindowUnit::Usd, 315, 315)
                .status(QuotaStatus::Available)
                .spend("$3.15 this month"),
        ],
    };
    r.insert(a);

    let mut a = Account::registered(
        "acct-opencode-go",
        "Go subscription",
        Provider::OpenCode,
        CredentialSource::PlainApiKey {
            fingerprint: "c41d0be9".into(),
            tail: "m2Xa".into(),
        },
    );
    a.identity = AccountIdentity {
        subject: handle("donbeave"),
        plan: Some("OpenCode Go".into()),
    };
    a.confidence = Confidence::Authoritative;
    a.lifecycle = Lifecycle::Available;
    a.default_for_provider = true;
    a.validation = ValidationState::Valid(ValidationLevel::IdentityAuthenticated);
    a.last_refresh_secs = Some(now - 3 * h - 2 * 60);
    a.issue = Some(
        RecoverableIssue::new(IssueCode::RateLimited, "Rate limited: retry after 25 min", Recoverability::Retryable)
            .detail("Last-good data kept from 3 h ago")
            .retry(now + 25 * 60),
    );
    a.usage = AccountUsage {
        freshness: FreshnessInfo::failed(Some(now - 3 * h - 2 * 60), Some(now + 25 * 60)),
        windows: vec![
            QuotaWindow::pct("rolling", "Rolling", WindowCategory::Session, 57).reset(now + h + 50 * 60),
            QuotaWindow::pct("weekly", "Weekly", WindowCategory::LongRange, 45).reset(now + 3 * d),
            QuotaWindow::pct("monthly", "Monthly", WindowCategory::LongRange, 22).reset(now + 27 * d),
        ],
    };
    r.insert(a);

    let mut a = Account::registered(
        "acct-claude-archive",
        "Archived contractor laptop profile — do not use for production launches",
        Provider::Anthropic,
        CredentialSource::LocalFolder {
            path: "~/Library/Application Support/jackin/profiles/claude-contractor-2025-archive".into(),
            detected: DetectedKind::ClaudeApiKeyEnv,
        },
    );
    a.confidence = Confidence::PresenceOnly;
    a.lifecycle = Lifecycle::NeedsLogin;
    a.enabled = false;
    a.validation = ValidationState::Valid(ValidationLevel::MaterialDiscovered);
    a.last_refresh_secs = Some(now - 30 * d);
    a.issue = Some(RecoverableIssue::new(
        IssueCode::IdentityUnresolved,
        "Identity unresolved · showing usage without a public handle",
        Recoverability::Unsupported,
    ));
    a.usage = AccountUsage {
        freshness: FreshnessInfo::current(now - 30 * d),
        windows: vec![],
    };
    r.insert(a);

    // discovered, read-only
    let mut a = Account::discovered(
        "disc-amp",
        "discovered",
        Provider::Amp,
        CredentialSource::LocalFolder {
            path: "~/.config/amp/secrets.json".into(),
            detected: DetectedKind::AmpSecrets,
        },
    );
    a.identity = AccountIdentity {
        subject: None,
        plan: Some("Free".into()),
    };
    a.confidence = Confidence::PresenceOnly;
    a.lifecycle = Lifecycle::Available;
    a.validation = ValidationState::Valid(ValidationLevel::QuotaReadable);
    a.last_refresh_secs = Some(now - 60);
    a.usage = AccountUsage {
        freshness: FreshnessInfo::current(now - 60),
        windows: vec![
            QuotaWindow::pct("daily_free", "Amp Free · daily", WindowCategory::Session, 91).reset(now + 10 * h),
            QuotaWindow::counted("credits", "Individual credits", WindowCategory::Other, WindowUnit::Usd, 0, 0)
                .spend("$0.00"),
        ],
    };
    r.insert(a);

    let mut a = Account::discovered(
        "disc-zai",
        "discovered",
        Provider::Zai,
        CredentialSource::HostEnv {
            var: "ZAI_API_KEY".into(),
            detected: DetectedKind::ZaiApiKeyEnv,
        },
    );
    a.identity = AccountIdentity {
        subject: handle("zai_9f1c"),
        plan: Some("GLM Coding Plan Pro".into()),
    };
    a.confidence = Confidence::Authoritative;
    a.lifecycle = Lifecycle::Available;
    a.validation = ValidationState::Valid(ValidationLevel::QuotaReadable);
    a.last_refresh_secs = Some(now - 2 * h - 15 * 60);
    a.issue = Some(RecoverableIssue::new(IssueCode::Stale, "Usage stale · last good 2 h ago", Recoverability::Retryable));
    a.usage = AccountUsage {
        freshness: FreshnessInfo::stale(now - 2 * h - 15 * 60, now + 10 * 60),
        windows: vec![
            QuotaWindow::counted("session", "Session", WindowCategory::Session, WindowUnit::Tokens, 4_200_000, 10_000_000)
                .reset(now + 2 * h),
            QuotaWindow::counted("weekly", "Weekly", WindowCategory::LongRange, WindowUnit::Tokens, 61_000_000, 80_000_000)
                .reset(now + 4 * d),
            QuotaWindow::counted("credits", "Credits", WindowCategory::Other, WindowUnit::Credits, 310, 1_000),
        ],
    };
    r.insert(a);

    let mut a = Account::discovered(
        "disc-kimi",
        "discovered",
        Provider::Moonshot,
        CredentialSource::LocalFolder {
            path: "~/.kimi".into(),
            detected: DetectedKind::KimiApiKeyEnv,
        },
    );
    a.identity = AccountIdentity {
        subject: None,
        plan: Some("Kimi Code".into()),
    };
    a.confidence = Confidence::PresenceOnly;
    a.lifecycle = Lifecycle::NeedsSecret;
    a.validation = ValidationState::Valid(ValidationLevel::MaterialDiscovered);
    a.issue = Some(RecoverableIssue::new(
        IssueCode::CredentialFileMissing,
        "No credential found: ~/.kimi has no api key",
        Recoverability::ActionRequired,
    ));
    a.usage = AccountUsage {
        freshness: FreshnessInfo::failed(None, None),
        windows: vec![QuotaWindow::sentinel(
            "quota",
            "Quota",
            QuotaStatus::Unavailable,
            "Quota unavailable until a key is present",
        )],
    };
    r.insert(a);

    let mut a = Account::discovered(
        "disc-minimax",
        "discovered",
        Provider::MiniMax,
        CredentialSource::HostEnv {
            var: "MINIMAX_API_TOKEN".into(),
            detected: DetectedKind::MinimaxTokenEnv,
        },
    );
    a.identity = AccountIdentity {
        subject: handle("mm_4471"),
        plan: Some("Coding Plan".into()),
    };
    a.confidence = Confidence::Estimated;
    a.lifecycle = Lifecycle::Unavailable;
    a.validation = ValidationState::Valid(ValidationLevel::IdentityAuthenticated);
    a.last_refresh_secs = Some(now - 6 * h);
    a.issue = Some(
        RecoverableIssue::new(
            IssueCode::ProviderUnavailable,
            "Provider unavailable: MiniMax did not respond",
            Recoverability::Retryable,
        )
        .detail("Timed out after 8 s"),
    );
    a.usage = AccountUsage {
        freshness: FreshnessInfo::failed(Some(now - 6 * h), Some(now + 15 * 60)),
        windows: vec![
            QuotaWindow::pct("general_session", "General · Session", WindowCategory::Session, 8).reset(now + 3 * h),
            QuotaWindow::pct("general_weekly", "General · Weekly", WindowCategory::LongRange, 34).reset(now + 5 * d),
            QuotaWindow::pct("m2_weekly", "MiniMax-M2 · Weekly", WindowCategory::Model, 47).reset(now + 5 * d),
        ],
    };
    r.insert(a);

    let mut a = Account::discovered(
        "disc-opencode-ci",
        "ci-bot",
        Provider::OpenCode,
        CredentialSource::LocalFolder {
            path: "~/.local/share/opencode/auth.json".into(),
            detected: DetectedKind::OpenCodeGoAuthJson,
        },
    );
    a.identity = AccountIdentity {
        subject: handle("ci-bot"),
        plan: None,
    };
    a.confidence = Confidence::Authoritative;
    a.lifecycle = Lifecycle::Unsupported;
    a.validation = ValidationState::Valid(ValidationLevel::IdentityAuthenticated);
    a.last_refresh_secs = Some(now - 5 * 60);
    a.issue = Some(RecoverableIssue::new(
        IssueCode::QuotaUnsupported,
        "Quota not visible: OpenCode returned 403",
        Recoverability::Unsupported,
    ));
    a.usage = AccountUsage {
        freshness: FreshnessInfo::current(now - 5 * 60),
        windows: vec![QuotaWindow::sentinel(
            "quota",
            "Quota",
            QuotaStatus::Unsupported,
            "Quota not visible: OpenCode returned 403",
        )],
    };
    r.insert(a);
    r
}

fn accounts_hard(now: i64) -> AccountRegistry {
    let mut r = accounts_mixed(now);
    let mut a = Account::registered(
        "acct-grok-revoked",
        "Revoked key",
        Provider::XAi,
        CredentialSource::OnePassword(op_ref(
            "chainargos.1password.com",
            ("v_eng01", "Engineering"),
            ("it_leg01", "Legacy · Rotated key"),
            "credential",
        )),
    );
    a.lifecycle = Lifecycle::NeedsLogin;
    a.validation = ValidationState::Invalid(RecoverableIssue::new(
        IssueCode::Unauthorized,
        "Not authorized: xAI rejected the credential",
        Recoverability::ActionRequired,
    ));
    a.issue = Some(
        RecoverableIssue::new(IssueCode::Unauthorized, "Not authorized: xAI rejected the credential", Recoverability::ActionRequired)
            .detail("HTTP 401 · re-login required"),
    );
    a.last_refresh_secs = Some(now - 40 * 60);
    a.usage = AccountUsage {
        freshness: FreshnessInfo::failed(None, None),
        windows: vec![],
    };
    r.insert(a);
    // long labels
    if let Some(w) = r.get_mut("acct-claude-work")
        && let Some(win) = w.usage.windows.iter_mut().find(|w| w.id == "weekly")
    {
        win.label = "Weekly · all models · includes Claude Code, desktop and API usage on the Team plan".into();
    }
    r
}

// -------------------------------------------------------------- worlds

fn base_world(scenario: Scenario) -> World {
    let clock = Clock::new();
    let now = clock.now_secs();
    let mut w = World {
        scenario,
        clock,
        arbiter: Arbiter::new(0),
        home: HOME.into(),
        cwd: format!("{HOME}/src/payments-platform"),
        workspaces: vec![],
        next_workspace_id: 100,
        roles: roles(),
        instances: vec![],
        daemons: BTreeMap::new(),
        global: global(false),
        accounts: AccountRegistry::default(),
        op: SimOnePassword::fixture(now),
        fs: fs(),
        github: github(),
        clipboard: None,
        jobs: vec![],
        daemon_health: DaemonHealth::Healthy,
        last_refresh_secs: now - 3,
        refresh_fails: false,
        save_fails_once: false,
        takeover_at_ms: None,
        session_accounts: BTreeMap::new(),
        discovery_pending: false,
    };
    w.sync_arbiter();
    w
}

/// Populated world shared by returning / accounts / launch / capsule.
fn populated(scenario: Scenario, rich: bool) -> World {
    let mut w = base_world(scenario);
    let now = w.now_secs();
    let h = 3600;
    let d = 86_400;
    w.workspaces = workspaces(rich);
    w.global = global(true);
    w.accounts = if rich { accounts_hard(now) } else { accounts_mixed(now) };
    // instances
    let mut i1 = instance("jk-7f3a", Some(1), "payments-platform", "the-architect", Agent::ClaudeCode, InstanceStatus::Running, now - 2 * h - 14 * 60, now - 3);
    i1.branch = Some("feature/settlement-backoff".into());
    i1.pr = Some((482, "Settlement retry backoff".into()));
    i1.uncommitted = 2;
    i1.unpushed = 1;
    i1.sessions = Ok(vec![
        SessionRecord {
            id: "s-01".into(),
            agent: Some(Agent::ClaudeCode),
            label: "claude · the-architect".into(),
            status: SessionStatus::Active,
            started_secs: now - 2 * h - 14 * 60,
        },
        SessionRecord {
            id: "s-02".into(),
            agent: Some(Agent::Codex),
            label: "codex · ledger tests".into(),
            status: SessionStatus::Exited(0),
            started_secs: now - d,
        },
    ]);
    let mut i2 = instance("jk-c41e", Some(1), "payments-platform", "reviewer", Agent::Codex, InstanceStatus::PreservedDirty, now - d - 3 * h, now - d);
    i2.uncommitted = 3;
    i2.sessions = Ok(vec![SessionRecord {
        id: "s-03".into(),
        agent: Some(Agent::Codex),
        label: "codex · review".into(),
        status: SessionStatus::Exited(0),
        started_secs: now - d - 3 * h,
    }]);
    let mut i3 = instance("jk-9b02", Some(2), "infra-control-plane", "sre", Agent::Codex, InstanceStatus::Running, now - 40 * 60, now - 2);
    i3.branch = Some("sre/node-pools".into());
    i3.sessions = Ok(vec![SessionRecord {
        id: "s-04".into(),
        agent: Some(Agent::Codex),
        label: "codex · sre".into(),
        status: SessionStatus::Active,
        started_secs: now - 40 * 60,
    }]);
    let mut i4 = instance("jk-12ee", Some(4), "customer-portal", "writer", Agent::Amp, InstanceStatus::Crashed, now - 5 * h, now - 4 * h);
    i4.sessions = Ok(vec![SessionRecord {
        id: "s-05".into(),
        agent: Some(Agent::Amp),
        label: "amp · docs".into(),
        status: SessionStatus::Crashed,
        started_secs: now - 5 * h,
    }]);
    let i5 = instance("jk-a1c0", Some(3), "release-automation", "backend", Agent::OpenCode, InstanceStatus::RestoreAvailable, now - 3 * d, now - 3 * d);
    let mut i6 = instance("jk-04d7", Some(2), "infra-control-plane", "sre", Agent::GrokBuild, InstanceStatus::PreservedUnpushed, now - 2 * d, now - 2 * d);
    i6.unpushed = 2;
    let i7 = instance("jk-77aa", Some(1), "payments-platform", "backend", Agent::Codex, InstanceStatus::Superseded, now - 6 * d, now - 6 * d);
    let i8 = instance("jk-88bb", Some(4), "customer-portal", "writer", Agent::KimiCode, InstanceStatus::Purged, now - 9 * d, now - 9 * d);
    let mut i9 = instance("jk-5e5e", Some(3), "release-automation", "backend", Agent::Codex, InstanceStatus::FailedSetup, now - 30 * 60, now - 30 * 60);
    i9.sessions = Ok(vec![]);
    w.instances = vec![i1, i2, i3, i4, i5, i6, i7, i8, i9];
    if rich {
        // many instances, missing daemon data, manifest error
        let mut i10 = instance("jk-e0e0", Some(10), "data-pipeline", "backend", Agent::ClaudeCode, InstanceStatus::Running, now - 10 * 60, now - 90);
        i10.sessions = Err(ManifestError::ReadError);
        w.instances.push(i10);
        let i11 = instance("jk-f1f1", Some(11), "docs-site", "writer", Agent::KimiCode, InstanceStatus::CleanExited, now - 4 * d, now - 4 * d);
        w.instances.push(i11);
        for k in 0..6 {
            let id = format!("jk-b{k}{k}{k}");
            let st = if k % 2 == 0 { InstanceStatus::CleanExited } else { InstanceStatus::RestoreAvailable };
            w.instances.push(instance(&id, Some(1), "payments-platform", "reviewer", Agent::Codex, st, now - (k + 2) * d, now - (k + 2) * d));
        }
    }
    // daemons for running instances
    let now_ms = w.now_ms();
    let mut d1 = Daemon::new("jk-7f3a", "payments-platform", now_ms - 2 * 3600 * 1000);
    d1.new_tab(Some(Agent::ClaudeCode), Some("acct-claude-work".into()), now_ms - 60_000, true);
    d1.split(SplitDir::Horizontal, false, Some(Agent::Codex), Some("acct-codex-primary".into()), now_ms - 50_000, true);
    d1.split(SplitDir::Vertical, false, None, None, now_ms - 40_000, true);
    if let Some(t) = d1.active_tab_mut() {
        t.focused = 1;
    }
    d1.new_tab(None, None, now_ms - 30_000, true);
    d1.new_tab(Some(Agent::Amp), None, now_ms - 20_000, true);
    if let Some(t) = d1.tabs.get_mut(2) {
        t.custom_label = Some("docs".into());
    }
    d1.active = 0;
    // long shell log for scrollback
    if let Some(p) = d1.pane_mut(3) {
        for n in 0..1_960u32 {
            let status = match n % 7 {
                0 => "retrying attempt=2",
                3 => "retrying attempt=1",
                _ => "settled",
            };
            let items = 9 + (n * 37) % 50;
            let l = if n % 60 == 0 {
                vec![junie_tui::widgets::viewport::Span::muted(format!("==== settlement.batch.2026-09-03T{:02}:{:02}Z ====", 9 + n / 360, (n / 6) % 60))]
            } else if n % 23 == 0 {
                vec![junie_tui::widgets::viewport::Span::new(format!("warn: batch {} backoff 500 ms", 4000 + n), junie_tui::theme::Tone::Warning)]
            } else if n % 97 == 0 {
                vec![junie_tui::widgets::viewport::Span::new(format!("error: batch {} gave up after 3 attempts", 4000 + n), junie_tui::theme::Tone::Error)]
            } else {
                vec![junie_tui::widgets::viewport::Span::plain(format!("batch {}  {:>2} items   status={status}", 4000 + n, items))]
            };
            p.term.lines.insert(0, l);
        }
        p.term.set_lines(p.term.lines.clone());
    }
    w.daemons.insert("jk-7f3a".into(), d1);
    let mut d3 = Daemon::new("jk-9b02", "infra-control-plane", now_ms - 40 * 60 * 1000);
    d3.new_tab(Some(Agent::Codex), Some("acct-codex-experiments".into()), now_ms - 30_000, true);
    w.daemons.insert("jk-9b02".into(), d3);
    if rich {
        let d10 = Daemon::new("jk-e0e0", "data-pipeline", now_ms);
        w.daemons.insert("jk-e0e0".into(), d10);
        // its daemon never answers
        if let Some(i) = w.instance_mut("jk-e0e0") {
            i.daemon = DaemonSnapshot::Unavailable;
        }
        w.daemons.remove("jk-e0e0");
    }
    refresh_snapshots(&mut w);
    w.sync_arbiter();
    w
}

/// Copy live daemon topology into the instance records' snapshots.
pub fn refresh_snapshots(w: &mut World) {
    let snaps: Vec<(String, DaemonSnapshot)> = w
        .daemons
        .iter()
        .map(|(id, d)| (id.clone(), d.snapshot()))
        .collect();
    for (id, s) in snaps {
        if let Some(i) = w.instance_mut(&id) {
            i.daemon = s;
        }
    }
    for i in w.instances.iter_mut() {
        if i.status != InstanceStatus::Running {
            i.daemon = DaemonSnapshot::Unavailable;
        }
    }
}

pub fn world_for(scenario: Scenario) -> World {
    match scenario {
        Scenario::FirstUse => {
            let mut w = base_world(scenario);
            w.discovery_pending = true;
            w
        }
        Scenario::Returning | Scenario::AccountsMixed | Scenario::LaunchRunning | Scenario::LaunchFailure | Scenario::CapsuleMulti => {
            populated(scenario, false)
        }
        Scenario::OutroLast => {
            let mut w = populated(scenario, false);
            // only one instance runs; entered 2 h 14 min ago
            for i in w.instances.iter_mut() {
                if i.id == "jk-9b02" {
                    i.status = InstanceStatus::CleanExited;
                }
            }
            w.daemons.remove("jk-9b02");
            refresh_snapshots(&mut w);
            w.sync_arbiter();
            w.arbiter.entered_at_ms = Some(-8_040_000);
            w
        }
        Scenario::HardCases => {
            let mut w = populated(scenario, true);
            w.op.session = OpSession::Locked;
            w.daemon_health = DaemonHealth::Stale;
            w.refresh_fails = true;
            w.save_fails_once = true;
            w.arbiter.discovery = Err(DiscoveryError::IndexUnreadable);
            w.arbiter.entered_at_ms = None;
            w.takeover_at_ms = Some(45_000);
            w
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precedence_order_and_why() {
        let w = populated(Scenario::Returning, false);
        let ws = w.workspace(1).unwrap();
        let r = resolve_account(Provider::Anthropic, Some(ws), None, None, &w.accounts);
        assert_eq!(r.account.as_deref(), Some("acct-claude-work"));
        assert_eq!(r.level, PrecedenceLevel::Workspace);
        assert!(r.why.contains("overrides provider default"));
        let s = "acct-claude-personal".to_owned();
        let r2 = resolve_account(Provider::Anthropic, Some(ws), None, Some(&s), &w.accounts);
        assert_eq!(r2.level, PrecedenceLevel::Session);
        let r3 = resolve_account(Provider::OpenAi, None, None, None, &w.accounts);
        assert_eq!(r3.level, PrecedenceLevel::ProviderDefault);
        let r4 = resolve_account(Provider::Amp, None, None, None, &w.accounts);
        assert_eq!(r4.level, PrecedenceLevel::Discovered);
        let r5 = resolve_account(Provider::Moonshot, None, None, None, &w.accounts);
        assert_eq!(r5.level, PrecedenceLevel::None);
    }

    #[test]
    fn every_scenario_builds() {
        for s in Scenario::ALL {
            let w = world_for(s);
            assert!(w.instances.iter().all(|i| !i.id.is_empty()));
        }
        let w = world_for(Scenario::CapsuleMulti);
        let d = &w.daemons["jk-7f3a"];
        assert_eq!(d.tabs.len(), 3);
        assert_eq!(d.tabs[0].leaves().len(), 3);
        assert!(d.pane(3).unwrap().term.len() >= 1_900);
        assert_eq!(w.running_count(), 2);
        let o = world_for(Scenario::OutroLast);
        assert_eq!(o.running_count(), 1);
    }
}
