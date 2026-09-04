//! Small, deterministic Jackin fixture data.
//!
//! Fixture constructors keep display metadata separate from credential
//! material. The only material-bearing values live inside
//! `sim::onepassword` and are exposed to provider code through its
//! secret-free closure.

use std::collections::BTreeSet;

use crate::domain::account::{
    Account, AccountId, AccountIdentity, AccountRegistry, Confidence, CredentialSource,
    DetectedKind, IdentitySubject, Lifecycle, Provenance, ValidationLevel, ValidationState,
};
use crate::domain::agent::{Agent, Provider};
use crate::domain::instance::{
    AgentState, DaemonSnapshot, Instance, InstanceId, InstanceStatus, PaneSnapshot, RunId,
    SessionRecord, SessionStatus, TabSnapshot,
};
use crate::domain::onepassword::OpReference;
use crate::domain::usage::{AccountUsage, FreshnessInfo};
use crate::domain::workspace::{
    AllowedRoles, DirtyExitPolicy, EnvVar, Mount, MountScope, RoleEntry, RolePolicy, RoleSource,
    Workspace, WorkspaceId,
};
use crate::sim::onepassword::SimOnePassword;
use crate::sim::provider;

pub const HOME: &str = "/Users/alexey";
pub const PAYMENTS_WORKSPACE: WorkspaceId = 1;
pub const PAYMENTS_WORKSPACE_NAME: &str = "payments-platform";
pub const PAYMENTS_WORKDIR: &str = "/Users/alexey/src/payments-platform";

/// Precedence used when selecting the account for one agent session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrecedenceLevel {
    Session,
    Role,
    Workspace,
    Global,
    Discovered,
}

/// A resolved account plus the source of the decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedAccount {
    pub account: Option<AccountId>,
    pub level: Option<PrecedenceLevel>,
    pub reason: String,
}

/// Build an id-only 1Password reference. It names metadata; it never embeds
/// or derives a credential value.
pub fn op_reference(
    vault_id: &str,
    vault_name: &str,
    item_id: &str,
    item_title: &str,
) -> OpReference {
    OpReference {
        account: "chainargos.1password.com".into(),
        vault_id: vault_id.into(),
        vault_name: vault_name.into(),
        item_id: item_id.into(),
        item_title: item_title.into(),
        section: None,
        field_id: "credential".into(),
        field_label: "credential".into(),
    }
}

/// Resolve the account with explicit session > role > workspace > global >
/// discovered precedence. Invalid or disabled choices are not silently
/// treated as usable selections.
pub fn resolve_account(
    provider: Provider,
    workspace: Option<&Workspace>,
    role: Option<&str>,
    session: Option<&AccountId>,
    registry: &AccountRegistry,
) -> ResolvedAccount {
    if let Some(id) = session
        && ready_for(id, provider, registry)
    {
        return ResolvedAccount {
            account: Some(id.clone()),
            level: Some(PrecedenceLevel::Session),
            reason: "session choice".into(),
        };
    }

    if let (Some(workspace), Some(role)) = (workspace, role)
        && let Some(id) = workspace.role_preference(role, provider, registry)
        && ready_for(&id, provider, registry)
    {
        return ResolvedAccount {
            account: Some(id),
            level: Some(PrecedenceLevel::Role),
            reason: format!("role preference · {role}"),
        };
    }

    if let Some(workspace) = workspace
        && let Some(id) = workspace
            .accounts
            .preferred
            .get(&provider)
            .filter(|id| ready_for(id, provider, registry))
    {
        return ResolvedAccount {
            account: Some(id.clone()),
            level: Some(PrecedenceLevel::Workspace),
            reason: format!("Workspace preference · {}", workspace.name),
        };
    }

    if let Some(account) = registry.default_for(provider) {
        return ResolvedAccount {
            account: Some(account.id.clone()),
            level: Some(PrecedenceLevel::Global),
            reason: "global provider default".into(),
        };
    }

    if let Some(account) = registry.discovered_current(provider) {
        return ResolvedAccount {
            account: Some(account.id.clone()),
            level: Some(PrecedenceLevel::Discovered),
            reason: "discovered host login".into(),
        };
    }

    ResolvedAccount {
        account: None,
        level: None,
        reason: "no ready account".into(),
    }
}

fn ready_for(id: &str, provider: Provider, registry: &AccountRegistry) -> bool {
    registry.get(id).is_some_and(|account| {
        account.provider == provider
            && account.enabled
            && account.lifecycle == Lifecycle::Available
    })
}

fn validated(
    id: &str,
    name: &str,
    provider: Provider,
    source: CredentialSource,
    op: &SimOnePassword,
    now: i64,
) -> Account {
    let mut account = Account::registered(id, name, provider, source);
    let outcome = provider::validate(provider, &account.source, None, op, now);
    provider::apply_validation(&mut account, &outcome, now);
    account
}

/// Accounts shared by the populated scenarios.
pub fn fixture_accounts(op: &SimOnePassword, now: i64) -> AccountRegistry {
    let mut registry = AccountRegistry::default();

    let mut anthropic = validated(
        "anthropic-work",
        "Work",
        Provider::Anthropic,
        CredentialSource::OnePassword(op_reference(
            "v_eng01",
            "Engineering",
            "it_ant01",
            "Anthropic · Work",
        )),
        op,
        now,
    );
    anthropic.default_for_provider = true;
    registry.insert(anthropic);

    let mut openai = validated(
        "openai-primary",
        "Primary",
        Provider::OpenAi,
        CredentialSource::OnePassword(op_reference(
            "v_eng01",
            "Engineering",
            "it_cdx01",
            "OpenAI · Codex Primary",
        )),
        op,
        now,
    );
    openai.default_for_provider = true;
    registry.insert(openai);

    let mut grok = validated(
        "grok-team",
        "Team",
        Provider::XAi,
        CredentialSource::OnePassword(op_reference(
            "v_eng01",
            "Engineering",
            "it_grk01",
            "xAI · Grok Team",
        )),
        op,
        now,
    )
    .with_endpoint("xAI production", "https://api.x.ai/v1");
    grok.default_for_provider = true;
    registry.insert(grok);

    registry.insert(Account::registered(
        "anthropic-personal",
        "Personal",
        Provider::Anthropic,
        CredentialSource::OnePassword(op_reference(
            "v_per01",
            "Personal",
            "it_ant02",
            "Anthropic · Personal API key",
        )),
    ));

    let mut discovered = Account::discovered(
        "grok-host",
        "Host profile",
        Provider::XAi,
        CredentialSource::HostEnv {
            var: "XAI_API_KEY".into(),
            detected: DetectedKind::GrokAuthJson,
        },
    );
    discovered.identity = AccountIdentity {
        subject: Some(IdentitySubject::Handle("team_chainargos".into())),
        plan: Some("Team · prepaid".into()),
    };
    discovered.provenance = vec![Provenance::LiveHost];
    discovered.confidence = Confidence::PresenceOnly;
    discovered.lifecycle = Lifecycle::Available;
    discovered.validation = ValidationState::Valid(ValidationLevel::MaterialDiscovered);
    discovered.last_refresh_secs = Some(now);
    discovered.usage = AccountUsage {
        freshness: FreshnessInfo::current(now),
        windows: provider::windows_for(Provider::XAi, now, false),
    };
    registry.insert(discovered);

    registry
}

/// Roles visible in the role picker.
pub fn fixture_roles() -> Vec<RoleEntry> {
    vec![
        RoleEntry {
            namespace: "chainargos".into(),
            name: "the-architect".into(),
            source: RoleSource::Git {
                url: "https://github.com/chainargos/roles".into(),
                branch: "main".into(),
            },
            trusted: true,
            in_registry: true,
            description: "Plan and review repository-scale changes".into(),
            load_error: None,
        },
        RoleEntry {
            namespace: "chainargos".into(),
            name: "reviewer".into(),
            source: RoleSource::Git {
                url: "https://github.com/chainargos/roles".into(),
                branch: "main".into(),
            },
            trusted: true,
            in_registry: true,
            description: "Review diffs and preserve invariants".into(),
            load_error: None,
        },
        RoleEntry {
            namespace: "chainargos".into(),
            name: "incident".into(),
            source: RoleSource::Local {
                path: "~/.jackin/roles/incident".into(),
            },
            trusted: false,
            in_registry: false,
            description: "Triage a production incident with restricted mounts".into(),
            load_error: None,
        },
    ]
}

/// The saved Workspace used by the manager and launch fixtures.
pub fn fixture_workspace() -> Workspace {
    let mut workspace = Workspace::new(
        PAYMENTS_WORKSPACE,
        PAYMENTS_WORKSPACE_NAME,
        PAYMENTS_WORKDIR,
    );
    workspace.mounts = vec![
        Mount::host(PAYMENTS_WORKDIR, "/workspace").scope(MountScope::Workspace),
        Mount::git(
            "git@github.com:chainargos/payments-platform.git",
            "/workspace",
        )
        .scope(MountScope::Workspace),
    ];
    workspace.roles = RolePolicy {
        allowed: AllowedRoles::Custom(vec![
            "chainargos/the-architect".into(),
            "chainargos/reviewer".into(),
            "chainargos/incident".into(),
        ]),
        default: Some("chainargos/the-architect".into()),
        last: Some("chainargos/reviewer".into()),
    };
    workspace.env = vec![
        EnvVar::plain("APP_ENV", "staging"),
        EnvVar::host("TERM_PROGRAM", "TERM_PROGRAM"),
        EnvVar::op(
            "DEPLOY_TOKEN",
            op_reference(
                "v_eng01",
                "Engineering",
                "it_dep01",
                "Prod · Deploy token",
            ),
        ),
    ];
    workspace.keep_awake = true;
    workspace.git_pull = true;
    workspace.accounts.enabled = BTreeSet::from(["grok-host".into()]);
    workspace
        .accounts
        .preferred
        .insert(Provider::Anthropic, "anthropic-work".into());
    workspace
        .accounts
        .preferred
        .insert(Provider::OpenAi, "openai-primary".into());
    workspace.accounts.role_preferred.insert(
        ("chainargos/reviewer".into(), Provider::Anthropic),
        "anthropic-work".into(),
    );
    workspace.dirty_policy = DirtyExitPolicy::Keep;
    workspace
}

/// A live instance with independent persisted manifest and daemon snapshot.
pub fn fixture_instance(
    status: InstanceStatus,
    run_id: RunId,
    now: i64,
    daemon: DaemonSnapshot,
) -> Instance {
    Instance {
        id: instance_id_for(status),
        container: "jackin-payments-platform".into(),
        workspace: Some(PAYMENTS_WORKSPACE),
        workdir: PAYMENTS_WORKDIR.into(),
        role: "chainargos/the-architect".into(),
        agent: Agent::ClaudeCode,
        status,
        created_secs: now - 12 * 60,
        last_seen_secs: now,
        run_id,
        sessions: Ok(vec![SessionRecord {
            id: "sess-01".into(),
            agent: Some(Agent::ClaudeCode),
            label: "Architecture review".into(),
            status: if status == InstanceStatus::Running {
                SessionStatus::Active
            } else {
                SessionStatus::Exited(0)
            },
            started_secs: now - 10 * 60,
        }]),
        daemon,
        branch: Some("feat/settlement-retry".into()),
        pr: Some((184, "Retry settlement after a provider timeout".into())),
        default_branch: "main".into(),
        uncommitted: usize::from(status == InstanceStatus::PreservedDirty),
        unpushed: usize::from(status == InstanceStatus::PreservedUnpushed),
        accounts: vec!["anthropic-work".into(), "openai-primary".into()],
    }
}

fn instance_id_for(status: InstanceStatus) -> InstanceId {
    match status {
        InstanceStatus::Running => "jk-7f3a".into(),
        InstanceStatus::Crashed => "jk-crash".into(),
        InstanceStatus::PreservedDirty => "jk-dirty".into(),
        InstanceStatus::PreservedUnpushed => "jk-unpushed".into(),
        _ => "jk-history".into(),
    }
}

/// Daemon state for the primary Capsule fixture.
pub fn live_capsule() -> DaemonSnapshot {
    DaemonSnapshot::Tabs(vec![
        TabSnapshot {
            label: "payments review".into(),
            active: true,
            panes: vec![
                PaneSnapshot {
                    label: "Claude Code".into(),
                    agent: Some(Agent::ClaudeCode),
                    state: AgentState::Working,
                    focused: true,
                },
                PaneSnapshot {
                    label: "Codex".into(),
                    agent: Some(Agent::Codex),
                    state: AgentState::Idle,
                    focused: false,
                },
            ],
        },
        TabSnapshot {
            label: "shell".into(),
            active: false,
            panes: vec![PaneSnapshot {
                label: "terminal".into(),
                agent: None,
                state: AgentState::Idle,
                focused: false,
            }],
        },
    ])
}

/// Useful only for tests and diagnostics: all populated fixture records are
/// deterministic and contain no resolved credential material.
pub fn fixture_metadata(op: &SimOnePassword) -> (usize, usize) {
    let accounts = op.accounts.len();
    let fields = op
        .accounts
        .iter()
        .flat_map(|account| account.vaults.iter())
        .flat_map(|vault| vault.items.iter())
        .map(|item| item.fields.len())
        .sum();
    (accounts, fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_precedence_is_explicit_and_deterministic() {
        let op = SimOnePassword::fixture(0);
        let registry = fixture_accounts(&op, 0);
        let workspace = fixture_workspace();

        let session = "openai-primary".to_owned();
        let resolved = resolve_account(
            Provider::OpenAi,
            Some(&workspace),
            Some("chainargos/the-architect"),
            Some(&session),
            &registry,
        );
        assert_eq!(resolved.account.as_deref(), Some("openai-primary"));
        assert_eq!(resolved.level, Some(PrecedenceLevel::Session));

        let resolved = resolve_account(
            Provider::Anthropic,
            Some(&workspace),
            Some("chainargos/reviewer"),
            None,
            &registry,
        );
        assert_eq!(resolved.account.as_deref(), Some("anthropic-work"));
        assert_eq!(resolved.level, Some(PrecedenceLevel::Role));
    }

    #[test]
    fn references_and_debug_never_contain_fixture_material() {
        let op = SimOnePassword::fixture(0);
        let reference = op_reference(
            "v_eng01",
            "Engineering",
            "it_cdx01",
            "OpenAI · Codex Primary",
        );
        assert_eq!(reference.canonical(), "op://v_eng01/it_cdx01/credential");
        assert!(!format!("{reference:?}").contains("openai:valid-cdx01"));
        assert!(!format!("{op:?}").contains("openai:valid-cdx01"));

        let account = Account::registered(
            "safe",
            "Safe",
            Provider::OpenAi,
            CredentialSource::OnePassword(reference),
        );
        assert!(!format!("{account:?}").contains("valid-cdx01"));
    }

    #[test]
    fn run_id_is_not_an_arbitrary_display_string() {
        let id = RunId::new(0x9c41_e2f0);
        assert_eq!(id.value(), 0x9c41_e2f0);
        assert_eq!(id.short(), "9c41e2f0");
    }
}
