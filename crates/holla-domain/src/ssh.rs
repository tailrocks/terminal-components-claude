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
    pub alias: String,
    pub host_name: String,
    pub user: String,
    pub port: Option<u16>,
    pub identity_file: Option<String>,
    pub jump: Option<String>,
    pub host_key_policy: HostKeyPolicy,
    pub multiplexed: bool,
}

impl SshHost {
    pub fn chain(&self) -> String {
        match &self.jump {
            Some(j) => format!("{j} → {}", self.alias),
            None => self.alias.clone(),
        }
    }
}
