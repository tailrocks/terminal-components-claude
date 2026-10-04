//! Connection and safety mode domain types.

/// Database engine type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Engine {
    Postgres,
    MySql,
    Sqlite,
}

impl Engine {
    pub fn label(self) -> &'static str {
        match self {
            Engine::Postgres => "PostgreSQL",
            Engine::MySql => "MySQL",
            Engine::Sqlite => "SQLite",
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            Engine::Postgres => "pg",
            Engine::MySql => "mysql",
            Engine::Sqlite => "sqlite",
        }
    }
}

/// `TablePro`'s per-connection Safe Mode level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SafeMode {
    /// Default: writes and reads run; only dangerous statements ask.
    Silent,
    /// Writes ask for confirmation.
    Alert,
    /// Every statement asks for confirmation.
    AlertFull,
    /// Writes ask and require deliberate confirmation.
    Safe,
    /// Every statement asks and requires deliberate confirmation.
    SafeFull,
    /// Writes are refused.
    ReadOnly,
}

impl SafeMode {
    pub const ALL: [SafeMode; 6] = [
        SafeMode::Silent,
        SafeMode::Alert,
        SafeMode::AlertFull,
        SafeMode::Safe,
        SafeMode::SafeFull,
        SafeMode::ReadOnly,
    ];

    pub fn label(self) -> &'static str {
        match self {
            SafeMode::Silent => "Silent",
            SafeMode::Alert => "Alert",
            SafeMode::AlertFull => "Alert (Full)",
            SafeMode::Safe => "Safe Mode",
            SafeMode::SafeFull => "Safe Mode (Full)",
            SafeMode::ReadOnly => "Read-Only",
        }
    }

    /// Short token for identity strip.
    pub fn token(self) -> &'static str {
        match self {
            SafeMode::Silent => "silent",
            SafeMode::Alert => "alert",
            SafeMode::AlertFull => "alert+",
            SafeMode::Safe => "safe",
            SafeMode::SafeFull => "safe+",
            SafeMode::ReadOnly => "read-only",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            SafeMode::Silent => "Writes run without asking. Destructive statements still confirm.",
            SafeMode::Alert => "Every write asks for confirmation before it runs.",
            SafeMode::AlertFull => "Every statement, reads included, asks for confirmation.",
            SafeMode::Safe => "Writes ask for confirmation and a deliberate acknowledgement.",
            SafeMode::SafeFull => {
                "Every statement asks for confirmation and a deliberate acknowledgement."
            }
            SafeMode::ReadOnly => "Writes are refused. Reads and exports still work.",
        }
    }

    pub fn requires_confirmation(self) -> bool {
        !matches!(self, SafeMode::Silent | SafeMode::ReadOnly)
    }

    pub fn requires_authentication(self) -> bool {
        matches!(self, SafeMode::Safe | SafeMode::SafeFull)
    }

    pub fn applies_to_all_queries(self) -> bool {
        matches!(self, SafeMode::AlertFull | SafeMode::SafeFull)
    }
}

/// Deployment environment tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Local,
    Development,
    Staging,
    Production,
}

impl Environment {
    pub fn label(self) -> &'static str {
        match self {
            Environment::Local => "Local",
            Environment::Development => "Development",
            Environment::Staging => "Staging",
            Environment::Production => "Production",
        }
    }
}

/// Outcome of a connection test attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectOutcome {
    Ok,
    AuthFailed,
    Unreachable,
}

/// Connection profile definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connection {
    pub name: String,
    pub engine: Engine,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub environment: Environment,
    pub safe_mode: SafeMode,
    pub ssl: bool,
    pub ssh: Option<String>,
    pub group: String,
    pub last_used: String,
    pub outcome: ConnectOutcome,
}
