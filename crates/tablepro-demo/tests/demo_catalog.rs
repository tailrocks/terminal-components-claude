use tablepro_demo::db::{Engine, SafeMode, connections};

#[test]
fn shipped_connections_are_fully_labelled_and_consistent() {
    let connections = connections();
    assert!(
        !connections.is_empty(),
        "the demo ships at least one connection"
    );
    for connection in &connections {
        assert_ne!(connection.name, "");
        assert_ne!(connection.host, "");
        assert_ne!(connection.database, "");

        assert!(
            !connection.user.is_empty() || connection.ssh.is_some() || !connection.ssl,
            "{} stores no user only through a token-based path",
            connection.name
        );
        assert_ne!(connection.group, "");
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
