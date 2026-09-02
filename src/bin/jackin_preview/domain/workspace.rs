//! Persisted Workspace configuration: workdir, mounts, Roles, environments,
//! Auth, policies. Everything here is durable host data, distinct from
//! instance records and live daemon snapshots.

use std::collections::BTreeMap;

use super::agent::{Agent, AuthMode, Provider};

pub type WorkspaceId = u32;
pub type RoleName = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub name: String,
    pub workdir: String,
    pub mounts: Vec<Mount>,
    pub roles: RolePolicy,
    /// Workspace-scope environment variables.
    pub env: Vec<EnvVar>,
    /// Role-scope environment variables keyed by Role name.
    pub role_env: BTreeMap<RoleName, Vec<EnvVar>>,
    /// Workspace-scope Auth overrides per Agent.
    pub auth: Vec<AuthEntry>,
    /// Role-scope Auth overrides.
    pub role_auth: BTreeMap<RoleName, Vec<AuthEntry>>,
    pub keep_awake: bool,
    pub git_pull: bool,
    /// Workspace-level provider account choice (overrides provider default).
    pub account_overrides: BTreeMap<Provider, super::account::AccountId>,
    /// Role-level provider account choice (overrides the Workspace choice).
    pub role_account_overrides: BTreeMap<(RoleName, Provider), super::account::AccountId>,
    pub dirty_policy: DirtyExitPolicy,
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
            auth: vec![],
            role_auth: BTreeMap::new(),
            keep_awake: false,
            git_pull: true,
            account_overrides: BTreeMap::new(),
            role_account_overrides: BTreeMap::new(),
            dirty_policy: DirtyExitPolicy::Ask,
        }
    }

    /// Number of fields that differ from `other` (dirty count).
    pub fn change_count(&self, other: &Workspace) -> usize {
        let mut n = 0;
        n += usize::from(self.name != other.name);
        n += usize::from(self.workdir != other.workdir);
        n += usize::from(self.keep_awake != other.keep_awake);
        n += usize::from(self.git_pull != other.git_pull);
        n += usize::from(self.roles != other.roles);
        n += usize::from(self.dirty_policy != other.dirty_policy);
        n += diff_len(&self.mounts, &other.mounts);
        n += diff_len(&self.env, &other.env);
        n += diff_len(&self.auth, &other.auth);
        n += usize::from(self.role_env != other.role_env);
        n += usize::from(self.role_auth != other.role_auth);
        n += usize::from(self.account_overrides != other.account_overrides);
        n += usize::from(self.role_account_overrides != other.role_account_overrides);
        n
    }

    pub fn env_count(&self) -> usize {
        self.env.len() + self.role_env.values().map(Vec::len).sum::<usize>()
    }
}

fn diff_len<T: PartialEq>(a: &[T], b: &[T]) -> usize {
    let added = a.iter().filter(|x| !b.contains(x)).count();
    let removed = b.iter().filter(|x| !a.contains(x)).count();
    added + removed
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

// ---------------------------------------------------------------- mounts

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mount {
    pub source: MountSource,
    pub destination: String,
    pub readonly: bool,
    pub isolation: Isolation,
    pub kind: MountKind,
    /// Scope for global mounts (Settings); Workspace mounts are `Workspace`.
    pub scope: MountScope,
    /// Source path moved or vanished since the mount was saved.
    pub drift: Option<String>,
    /// An isolated (worktree/clone) copy is in use by a running instance;
    /// changing isolation requires cleanup first.
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

    /// `Shared → Worktree → Clone → Shared`.
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

impl MountKind {
    pub fn label(self) -> &'static str {
        match self {
            MountKind::Directory => "directory",
            MountKind::Repository => "repository",
        }
    }
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

// ----------------------------------------------------------------- roles

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

/// A Role as the host registry knows it: namespace/name, source, trust.
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

// ------------------------------------------------------------ environments

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvValue {
    /// Plain text; rendered masked by default.
    Plain(String),
    /// 1Password item/field reference, resolved on demand (`[op]`).
    OnePassword(super::onepassword::OpReference),
    /// Forward a host environment variable by name.
    HostEnv(String),
}

impl EnvValue {
    pub fn source_label(&self) -> &'static str {
        match self {
            EnvValue::Plain(_) => "plain text",
            EnvValue::OnePassword(_) => "1Password",
            EnvValue::HostEnv(_) => "host env",
        }
    }

    /// Masked presentation: never the plain value unless unmasked.
    pub fn display(&self, unmasked: bool) -> String {
        match self {
            EnvValue::Plain(v) => {
                if unmasked {
                    v.clone()
                } else {
                    mask(v)
                }
            }
            EnvValue::OnePassword(r) => format!("[op] {}", r.display()),
            EnvValue::HostEnv(name) => format!("${name}"),
        }
    }
}

/// `********` plus a short synthetic tail so two masked values differ.
/// Asterisk runs, never `•` (which marks modified rows in the tables).
pub fn mask(v: &str) -> String {
    if v.is_empty() {
        return "(empty)".into();
    }
    let tail: String = v.chars().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect();
    if v.chars().count() <= 6 {
        "*".repeat(v.chars().count())
    } else {
        format!("************{tail}")
    }
}

/// Reserved and forbidden environment keys (validation before save).
pub fn env_key_error(key: &str) -> Option<String> {
    let k = key.trim();
    if k.is_empty() {
        return Some("Key is required".into());
    }
    if !k
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
        || k.chars().next().is_some_and(|c| c.is_ascii_digit())
    {
        return Some("Key must be letters, digits and underscores, not starting with a digit".into());
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

// ------------------------------------------------------------------- auth

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthEntry {
    pub agent: Agent,
    pub mode: AuthMode,
    pub source: AuthSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthSource {
    /// The host's own agent profile (sync mode).
    HostProfile,
    /// A specific local profile folder.
    Folder(String),
    /// A registered account from the Account & Usage Center.
    Account(super::account::AccountId),
    /// A direct 1Password reference (API key / token).
    OnePassword(super::onepassword::OpReference),
    /// Masked plain-text secret material (fingerprint only).
    Plain { fingerprint: String },
    None,
}

impl AuthSource {
    pub fn label(&self) -> String {
        match self {
            AuthSource::HostProfile => "host profile".into(),
            AuthSource::Folder(p) => p.clone(),
            AuthSource::Account(id) => format!("account #{id}"),
            AuthSource::OnePassword(r) => format!("[op] {}", r.display()),
            AuthSource::Plain { fingerprint } => format!("plain text · {fingerprint}"),
            AuthSource::None => "none".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masking_never_reveals_the_value() {
        assert_eq!(mask("sk-ant-api03-verysecretvalue"), "************alue");
        assert_eq!(mask("abc"), "***");
        assert_eq!(mask(""), "(empty)");
        assert!(env_key_error("PATH").is_some());
        assert!(env_key_error("1ABC").is_some());
        assert!(env_key_error("DATABASE_URL").is_none());
    }

    #[test]
    fn change_count_tracks_fields_and_rows() {
        let a = Workspace::new(1, "payments", "/workspace");
        let mut b = a.clone();
        assert_eq!(a.change_count(&b), 0);
        b.name = "payments-platform".into();
        b.mounts.push(Mount::host("/Users/op/src/x", "/workspace/x"));
        b.git_pull = false;
        assert_eq!(a.change_count(&b), 3);
        assert_eq!(Isolation::Clone.next(), Isolation::Shared);
    }
}
