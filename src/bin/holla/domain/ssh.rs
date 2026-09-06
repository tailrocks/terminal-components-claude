//! ssh: literal aliases from `~/.ssh/config`. Identity files are filenames
//! only — key material never enters fixtures, UI or plans.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostKeyPolicy {
    Strict,
    Ask,
    TrustOnFirstUse,
}

impl HostKeyPolicy {
    pub fn label(self) -> &'static str {
        match self {
            HostKeyPolicy::Strict => "strict",
            HostKeyPolicy::Ask => "ask",
            HostKeyPolicy::TrustOnFirstUse => "trust-on-first-use",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SshHost {
    /// The literal alias line (`Host prod-eu-1`).
    pub alias: String,
    pub host_name: String,
    pub user: String,
    pub port: Option<u16>,
    /// Identity FILENAME (`id_ed25519_prod`); contents are never modeled.
    pub identity_file: Option<String>,
    /// ProxyJump alias, when the host is reached through another.
    pub jump: Option<String>,
    pub host_key_policy: HostKeyPolicy,
    /// ControlMaster multiplexing currently active.
    pub multiplexed: bool,
}

impl SshHost {
    /// Jump chain as shown to the user: `bastion → prod-eu-1`.
    pub fn chain(&self) -> String {
        match &self.jump {
            Some(j) => format!("{j} → {}", self.alias),
            None => self.alias.clone(),
        }
    }
}
