#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Local,
    Dev,
    Staging,
    Production,
}

impl Environment {
    pub fn label(self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Dev => "dev",
            Environment::Staging => "staging",
            Environment::Production => "production",
        }
    }

    pub fn glyph(self) -> Option<&'static str> {
        match self {
            Environment::Local | Environment::Dev => None,
            Environment::Staging => Some("◇"),
            Environment::Production => Some("◆"),
        }
    }

    pub fn sensitive(self) -> bool {
        matches!(self, Environment::Staging | Environment::Production)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostKind {
    Local,
    Remote,
}

impl HostKind {
    pub fn label(self) -> &'static str {
        match self {
            HostKind::Local => "local",
            HostKind::Remote => "ssh",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostMetrics {
    pub load_x100: (u32, u32, u32),
    pub mem_used_mb: u32,
    pub mem_total_mb: u32,
    pub uptime_days: u32,
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
pub struct Host {
    pub name: String,
    pub kind: HostKind,
    pub env: Environment,
    pub metrics: HostMetrics,
    pub osc52: bool,
    pub osc52_limit: usize,
}

impl Host {
    pub fn local(name: &str) -> Self {
        Self {
            name: name.into(),
            kind: HostKind::Local,
            env: Environment::Local,
            metrics: HostMetrics::dev(),
            osc52: false,
            osc52_limit: 100_000,
        }
    }

    pub fn remote(name: &str, env: Environment) -> Self {
        let metrics = match env {
            Environment::Production | Environment::Staging => HostMetrics::production(),
            _ => HostMetrics::dev(),
        };
        Self {
            name: name.into(),
            kind: HostKind::Remote,
            env,
            metrics,
            osc52: false,
            osc52_limit: 100_000,
        }
    }

    pub fn identity(&self) -> String {
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
