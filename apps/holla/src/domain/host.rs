//! Host identity: where this session runs. Remote and sensitive hosts must be
//! unmistakable in persistent chrome (CONCEPT.md §5.5.2, §8.12).

/// Environment role of a host; drives identity glyphs and confirmation
/// strength. `◆` production / `◇` staging per the design-system glyph table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Environment {
    /// Local developer machine.
    Local,
    /// Remote development host.
    Dev,
    /// Remote staging host.
    #[allow(dead_code)] // P5 hard-cases host
    Staging,
    /// Remote production host: strongest confirmation treatment.
    Production,
}

impl Environment {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Dev => "dev",
            Environment::Staging => "staging",
            Environment::Production => "production",
        }
    }

    /// Identity marker for remote environments; `None` for a local machine.
    pub(crate) fn glyph(self) -> Option<&'static str> {
        match self {
            Environment::Local | Environment::Dev => None,
            Environment::Staging => Some("◇"),
            Environment::Production => Some("◆"),
        }
    }

    /// Whether destructive confirmation gets the strongest treatment.
    #[allow(dead_code)] // P3 safety gates
    pub(crate) fn sensitive(self) -> bool {
        matches!(self, Environment::Staging | Environment::Production)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HostKind {
    /// The machine holla runs on.
    Local,
    /// Reached over SSH; the session itself is remote.
    Remote,
}

impl HostKind {
    pub(crate) fn label(self) -> &'static str {
        match self {
            HostKind::Local => "local",
            HostKind::Remote => "ssh",
        }
    }
}

/// Deterministic host vitals for the system snapshot (§8.6). Integers keep
/// `Host` `Eq`; load is ×100.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HostMetrics {
    pub(crate) load_x100: (u32, u32, u32),
    pub(crate) mem_used_mb: u32,
    pub(crate) mem_total_mb: u32,
    pub(crate) uptime_days: u32,
}

impl HostMetrics {
    fn dev() -> Self {
        Self {
            load_x100: (42, 38, 31),
            mem_used_mb: 6_300,
            mem_total_mb: 16_384,
            uptime_days: 16,
        }
    }

    fn production() -> Self {
        Self {
            load_x100: (310, 280, 250),
            mem_used_mb: 14_200,
            mem_total_mb: 16_384,
            uptime_days: 212,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Host {
    pub(crate) name: String,
    pub(crate) kind: HostKind,
    pub(crate) env: Environment,
    pub(crate) metrics: HostMetrics,
}

impl Host {
    pub(crate) fn local(name: &str) -> Self {
        Self {
            name: name.into(),
            kind: HostKind::Local,
            env: Environment::Local,
            metrics: HostMetrics::dev(),
        }
    }

    pub(crate) fn remote(name: &str, env: Environment) -> Self {
        let metrics = match env {
            Environment::Production | Environment::Staging => HostMetrics::production(),
            _ => HostMetrics::dev(),
        };
        Self {
            name: name.into(),
            kind: HostKind::Remote,
            env,
            metrics,
        }
    }

    /// Right-hand identity for chrome: `devbox · local`, `◆ prod-eu-1 · production`.
    pub(crate) fn identity(&self) -> String {
        let role = match (self.kind, self.env) {
            (HostKind::Local, Environment::Local) => "local".to_owned(),
            _ => format!("{} · {}", self.kind.label(), self.env.label()),
        };
        match self.env.glyph() {
            Some(g) => format!("{g} {} · {role}", self.name),
            None => format!("{} · {role}", self.name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_marks_sensitive_hosts() {
        assert_eq!(Host::local("devbox").identity(), "devbox · local");
        assert_eq!(
            Host::remote("prod-eu-1", Environment::Production).identity(),
            "◆ prod-eu-1 · ssh · production"
        );
        assert!(Environment::Production.sensitive());
        assert!(!Environment::Local.sensitive());
    }
}
