//! ssh: literal aliases from `~/.ssh/config`. Identity files are filenames
//! only — key material never enters fixtures, UI or plans.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HostKeyPolicy {
    Strict,
    Ask,
    TrustOnFirstUse,
}

impl HostKeyPolicy {
    pub(crate) fn label(self) -> &'static str {
        match self {
            HostKeyPolicy::Strict => "strict",
            HostKeyPolicy::Ask => "ask",
            HostKeyPolicy::TrustOnFirstUse => "trust-on-first-use",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SshHost {
    /// The literal alias line (`Host prod-eu-1`).
    pub(crate) alias: String,
    pub(crate) host_name: String,
    pub(crate) user: String,
    pub(crate) port: Option<u16>,
    /// Identity FILENAME (`id_ed25519_prod`); contents are never modeled.
    pub(crate) identity_file: Option<String>,
    /// ProxyJump alias, when the host is reached through another.
    pub(crate) jump: Option<String>,
    pub(crate) host_key_policy: HostKeyPolicy,
    /// ControlMaster multiplexing currently active.
    pub(crate) multiplexed: bool,
}

impl SshHost {
    /// Jump chain as shown to the user: `bastion → prod-eu-1`.
    pub(crate) fn chain(&self) -> String {
        match &self.jump {
            Some(j) => format!("{j} → {}", self.alias),
            None => self.alias.clone(),
        }
    }
}
