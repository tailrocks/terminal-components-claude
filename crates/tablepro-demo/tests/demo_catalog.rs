//! The deterministic demo catalog stays self-consistent: every shipped
//! connection names a real engine and environment, and the safe-mode rules
//! agree with the documented confirmation behaviour.

use tablepro_demo::db::{Engine, SafeMode, connections};

#[test]
fn shipped_connections_are_fully_labelled_and_consistent() {
    let connections = connections();
    assert!(
        !connections.is_empty(),
        "the demo ships at least one connection"
    );
    for connection in &connections {
        assert!(!connection.name.is_empty());
        assert!(!connection.host.is_empty());
        assert!(!connection.database.is_empty());
        // IAM/SSO connections authenticate without a stored user.
        assert!(
            !connection.user.is_empty() || connection.ssh.is_some() || !connection.ssl,
            "{} stores no user only through a token-based path",
            connection.name
        );
        assert!(!connection.group.is_empty());
        let _ = match connection.engine {
            Engine::Postgres | Engine::MySql | Engine::Sqlite => connection.engine.label(),
        };
        let _ = match connection.safe_mode {
            SafeMode::Silent
            | SafeMode::Alert
            | SafeMode::AlertFull
            | SafeMode::Safe
            | SafeMode::SafeFull
            | SafeMode::ReadOnly => connection.safe_mode.label(),
        };
        // Silent and read-only never prompt; everything in between does.
        let silent_or_readonly =
            matches!(connection.safe_mode, SafeMode::Silent | SafeMode::ReadOnly);
        assert_eq!(
            connection.safe_mode.requires_confirmation(),
            !silent_or_readonly,
            "confirmation tracks the safe mode for {}",
            connection.name
        );
    }
}
