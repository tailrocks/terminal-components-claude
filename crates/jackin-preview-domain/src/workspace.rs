use std::collections::{BTreeMap, BTreeSet};
use std::{fmt, mem};

use super::account::{AccountId, AccountRegistry, Lifecycle};
use super::agent::Provider;

pub type WorkspaceId = u32;

pub type RoleName = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub id: WorkspaceId,

    pub name: String,

    pub workdir: String,

    pub mounts: Vec<Mount>,

    pub roles: RolePolicy,

    pub env: Vec<EnvVar>,

    pub role_env: BTreeMap<RoleName, Vec<EnvVar>>,

    pub keep_awake: bool,

    pub git_pull: bool,

    pub accounts: AccountPolicy,

    pub dirty_policy: DirtyExitPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AccountPolicy {
    pub disabled_defaults: BTreeSet<AccountId>,

    pub enabled: BTreeSet<AccountId>,

    pub preferred: BTreeMap<Provider, AccountId>,

    pub role_preferred: BTreeMap<(RoleName, Provider), AccountId>,
}

impl AccountPolicy {
    pub fn change_count(&self, other: &AccountPolicy) -> usize {
        self.disabled_defaults
            .symmetric_difference(&other.disabled_defaults)
            .count()
            + self.enabled.symmetric_difference(&other.enabled).count()
            + usize::from(self.preferred != other.preferred)
            + usize::from(self.role_preferred != other.role_preferred)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effective {
    InheritedDefault,

    Enabled,
}

impl Effective {
    pub fn label(self) -> &'static str {
        match self {
            Effective::InheritedDefault => "inherited default",
            Effective::Enabled => "enabled here",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Usability {
    Ready,

    Disabled(String),

    NeedsLogin,

    Invalid(String),
}

impl Usability {
    pub fn label(&self) -> String {
        match self {
            Usability::Ready => "ready".into(),
            Usability::Disabled(r) => r.clone(),
            Usability::NeedsLogin => "needs login".into(),
            Usability::Invalid(r) => r.clone(),
        }
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, Usability::Ready)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveAccount {
    pub id: AccountId,

    pub provider: Provider,

    pub origin: Effective,

    pub preferred: bool,

    pub usable: Usability,
}

pub fn usability_of(a: &super::account::Account) -> Usability {
    if !a.enabled {
        return Usability::Disabled("disabled globally".into());
    }
    match a.lifecycle {
        Lifecycle::Available => Usability::Ready,
        Lifecycle::NeedsLogin => Usability::NeedsLogin,
        Lifecycle::NeedsSecret => Usability::Invalid("needs secret".into()),
        Lifecycle::AgentUninitialized => Usability::Invalid("agent uninitialized".into()),
        Lifecycle::Unsupported => Usability::Invalid("unsupported".into()),
        Lifecycle::Unavailable => Usability::Invalid("unavailable".into()),
        Lifecycle::Error => Usability::Invalid(
            a.issue
                .as_ref()
                .map(|i| i.message.clone())
                .unwrap_or("error".into()),
        ),
    }
}

impl Workspace {
    pub fn new(id: WorkspaceId, name: &str, workdir: &str) -> Self {
        Self {
            id,
            name: name.to_owned(),
            workdir: workdir.to_owned(),
            mounts: vec![],
            roles: RolePolicy::default(),
            env: vec![],
            role_env: BTreeMap::new(),
            keep_awake: false,
            git_pull: true,
            accounts: AccountPolicy::default(),
            dirty_policy: DirtyExitPolicy::Ask,
        }
    }

    pub fn effective_accounts(&self, registry: &AccountRegistry) -> Vec<EffectiveAccount> {
        let mut out: Vec<EffectiveAccount> = vec![];
        for a in &registry.accounts {
            let inherited =
                a.default_for_provider && !self.accounts.disabled_defaults.contains(&a.id);
            let enabled = self.accounts.enabled.contains(&a.id);
            let origin = if inherited {
                Effective::InheritedDefault
            } else if enabled {
                Effective::Enabled
            } else {
                continue;
            };
            out.push(EffectiveAccount {
                id: a.id.clone(),
                provider: a.provider,
                origin,
                preferred: false,
                usable: usability_of(a),
            });
        }

        let providers: Vec<Provider> = {
            let mut v: Vec<Provider> = out.iter().map(|e| e.provider).collect();
            v.dedup();
            v
        };
        for p in providers {
            let pick = self
                .accounts
                .preferred
                .get(&p)
                .filter(|id| out.iter().any(|e| &e.id == *id))
                .cloned()
                .or_else(|| {
                    out.iter()
                        .find(|e| e.provider == p && e.origin == Effective::InheritedDefault)
                        .map(|e| e.id.clone())
                })
                .or_else(|| {
                    out.iter()
                        .find(|e| e.provider == p && e.usable.is_ready())
                        .map(|e| e.id.clone())
                });
            if let Some(id) = pick
                && let Some(e) = out.iter_mut().find(|e| e.id == id)
            {
                e.preferred = true;
            }
        }
        out.sort_by(|a, b| {
            a.provider
                .cmp(&b.provider)
                .then(b.preferred.cmp(&a.preferred))
        });
        out
    }

    pub fn role_preference(
        &self,
        role: &str,
        provider: Provider,
        registry: &AccountRegistry,
    ) -> Option<AccountId> {
        let id = self
            .accounts
            .role_preferred
            .get(&(role.to_owned(), provider))?;
        self.effective_accounts(registry)
            .iter()
            .find(|e| &e.id == id)
            .map(|e| e.id.clone())
    }

    pub fn change_count(&self, other: &Workspace) -> usize {
        let mut n = 0;
        n += usize::from(self.name != other.name);
        n += usize::from(self.workdir != other.workdir);
        n += usize::from(self.keep_awake != other.keep_awake);
        n += usize::from(self.git_pull != other.git_pull);
        n += usize::from(self.roles != other.roles);
        n += usize::from(self.dirty_policy != other.dirty_policy);
        n += keyed(&self.mounts, &other.mounts, |m| m.destination.clone());
        n += keyed(&self.env, &other.env, |e| e.key.clone());
        n += usize::from(self.role_env != other.role_env);
        n += self.accounts.change_count(&other.accounts);
        n
    }

    pub fn env_count(&self) -> usize {
        self.env.len() + self.role_env.values().map(Vec::len).sum::<usize>()
    }
}

fn keyed<T: PartialEq>(a: &[T], b: &[T], key: impl Fn(&T) -> String) -> usize {
    let mut n = 0;
    for x in a {
        match b.iter().find(|y| key(y) == key(x)) {
            None => n += 1,
            Some(y) if y != x => n += 1,
            _ => {}
        }
    }
    n += b
        .iter()
        .filter(|y| !a.iter().any(|x| key(x) == key(y)))
        .count();
    n
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirtyExitPolicy {
    Ask,

    Keep,

    Discard,
}

impl DirtyExitPolicy {
    pub fn label(self) -> &'static str {
        match self {
            DirtyExitPolicy::Ask => "ask",
            DirtyExitPolicy::Keep => "keep",
            DirtyExitPolicy::Discard => "discard",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mount {
    pub source: MountSource,

    pub destination: String,

    pub readonly: bool,

    pub isolation: Isolation,

    pub kind: MountKind,

    pub scope: MountScope,

    pub drift: Option<String>,

    pub running_isolated: bool,
}

impl Mount {
    pub fn host(source: &str, destination: &str) -> Self {
        Self {
            source: MountSource::Host(source.to_owned()),
            destination: destination.to_owned(),
            readonly: false,
            isolation: Isolation::Shared,
            kind: MountKind::Directory,
            scope: MountScope::Workspace,
            drift: None,
            running_isolated: false,
        }
    }

    pub fn git(url: &str, destination: &str) -> Self {
        Self {
            source: MountSource::Git(url.to_owned()),
            destination: destination.to_owned(),
            readonly: false,
            isolation: Isolation::Clone,
            kind: MountKind::Repository,
            scope: MountScope::Workspace,
            drift: None,
            running_isolated: false,
        }
    }

    pub fn readonly(mut self, ro: bool) -> Self {
        self.readonly = ro;
        self
    }

    pub fn isolation(mut self, iso: Isolation) -> Self {
        self.isolation = iso;
        self
    }

    pub fn repository(mut self) -> Self {
        self.kind = MountKind::Repository;
        self
    }

    pub fn scope(mut self, scope: MountScope) -> Self {
        self.scope = scope;
        self
    }

    pub fn source_label(&self) -> &str {
        match &self.source {
            MountSource::Host(p) | MountSource::Git(p) => p,
        }
    }

    pub fn mode_label(&self) -> &'static str {
        if self.readonly { "ro" } else { "rw" }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MountSource {
    Host(String),

    Git(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Isolation {
    Shared,

    Worktree,

    Clone,
}

impl Isolation {
    pub fn label(self) -> &'static str {
        match self {
            Isolation::Shared => "Shared",
            Isolation::Worktree => "Worktree",
            Isolation::Clone => "Clone",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Isolation::Shared => Isolation::Worktree,
            Isolation::Worktree => Isolation::Clone,
            Isolation::Clone => Isolation::Shared,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountKind {
    Directory,

    Repository,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MountScope {
    Global,

    Workspace,

    Role(RoleName),
}

impl MountScope {
    pub fn label(&self) -> String {
        match self {
            MountScope::Global => "global".into(),
            MountScope::Workspace => "workspace".into(),
            MountScope::Role(r) => format!("role {r}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RolePolicy {
    pub allowed: AllowedRoles,

    pub default: Option<RoleName>,

    pub last: Option<RoleName>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AllowedRoles {
    #[default]
    All,

    Custom(Vec<RoleName>),
}

impl RolePolicy {
    pub fn allows(&self, role: &str) -> bool {
        match &self.allowed {
            AllowedRoles::All => true,
            AllowedRoles::Custom(list) => list.iter().any(|r| r == role),
        }
    }

    pub fn summary(&self) -> String {
        match &self.allowed {
            AllowedRoles::All => "Allowed roles: all".into(),
            AllowedRoles::Custom(list) => format!("Allowed roles: {}", list.len()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleEntry {
    pub namespace: String,

    pub name: String,

    pub source: RoleSource,

    pub trusted: bool,

    pub in_registry: bool,

    pub description: String,

    pub load_error: Option<String>,
}

impl RoleEntry {
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.namespace, self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoleSource {
    Git { url: String, branch: String },

    Local { path: String },
}

impl RoleSource {
    pub fn label(&self) -> String {
        match self {
            RoleSource::Git { url, branch } => format!("{url} @ {branch}"),
            RoleSource::Local { path } => path.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvVar {
    pub key: String,

    pub value: EnvValue,
}

impl EnvVar {
    pub fn plain(key: &str, value: &str) -> Self {
        Self {
            key: key.to_owned(),
            value: EnvValue::Plain(value.to_owned()),
        }
    }

    pub fn op(key: &str, reference: super::onepassword::OpReference) -> Self {
        Self {
            key: key.to_owned(),
            value: EnvValue::OnePassword(reference),
        }
    }

    pub fn host(key: &str, host_var: &str) -> Self {
        Self {
            key: key.to_owned(),
            value: EnvValue::HostEnv(host_var.to_owned()),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum EnvValue {
    Plain(String),

    OnePassword(super::onepassword::OpReference),

    HostEnv(String),
}

impl fmt::Debug for EnvValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plain(_) => formatter.write_str("Plain([redacted])"),
            Self::OnePassword(_) => formatter.write_str("OnePassword([redacted])"),
            Self::HostEnv(_) => formatter.write_str("HostEnv([redacted])"),
        }
    }
}

impl Drop for EnvValue {
    fn drop(&mut self) {
        if let Self::Plain(value) = self {
            let mut secret = termrock::Secret::new(mem::take(value));
            secret.zeroize();
        }
    }
}

impl EnvValue {
    pub fn source_label(&self) -> &'static str {
        match self {
            EnvValue::Plain(_) => "plain text",
            EnvValue::OnePassword(_) => "1Password",
            EnvValue::HostEnv(_) => "host env",
        }
    }
}

pub fn mask(v: &str) -> String {
    if v.is_empty() {
        return "(empty)".into();
    }
    let n = v.chars().count();
    let key_shaped = n >= 16 && !v.contains("://") && !v.contains('/') && !v.contains(' ');
    if key_shaped {
        let tail: String = v
            .chars()
            .rev()
            .take(4)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        format!("************{tail}")
    } else {
        "*".repeat(n.min(16))
    }
}

pub fn env_key_error(key: &str) -> Option<String> {
    let k = key.trim();
    if k.is_empty() {
        return Some("Key is required".into());
    }
    if !k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        || k.chars().next().is_some_and(|c| c.is_ascii_digit())
    {
        return Some(
            "Key must be letters, digits and underscores, not starting with a digit".into(),
        );
    }
    const RESERVED: [&str; 6] = [
        "PATH",
        "HOME",
        "JACKIN_INSTANCE",
        "JACKIN_ROLE",
        "JACKIN_SOCKET",
        "TERM",
    ];
    if RESERVED.contains(&k) {
        return Some(format!("{k} is reserved by the runtime"));
    }
    None
}
