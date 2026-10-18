//! Ownership-check admission: the check passes on the workspace and
//! fails closed on violations.
//!
//! This integration test is the CI execution path for the FIX-008
//! ownership check: the generated CI runs `cargo nextest` for every
//! crate lane (including `termrock-xtask`), and the Velnor generator
//! schema offers no supported input for a bespoke step, so the check
//! runs here instead of as its own workflow step.

use std::path::{Path, PathBuf};

use termrock_xtask::ownership::{check_workspace, exceptions, rules};

/// The workspace root above this crate's manifest dir.
fn workspace_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|crates| crates.parent())
        .expect("xtask crate lives two levels below the workspace root")
        .to_path_buf()
}

/// The ownership check passes on the current tree: every finding is
/// admitted by the versioned exceptions file and no exception is stale.
#[test]
fn ownership_check_passes_on_workspace() {
    let root = workspace_root();
    let exceptions = root
        .join("crates")
        .join("termrock-xtask")
        .join("exceptions")
        .join("v1.json");
    let report = check_workspace(&root, &exceptions);
    assert!(
        report.files_scanned > 300,
        "the check must scan the whole tree, got {} files",
        report.files_scanned
    );
    assert!(
        report.passed(),
        "ownership check failed:\n{}",
        termrock_xtask::ownership::render_human(&report)
    );
}

/// Write a scratch mini-workspace with one preview member.
fn scratch_workspace(member_src: &str, exceptions_json: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("scratch dir");
    let root = dir.path();
    std::fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/demo-preview\"]\n",
    )
    .expect("write manifest");
    let src = root.join("crates").join("demo-preview").join("src");
    std::fs::create_dir_all(&src).expect("create src");
    std::fs::write(src.join("lib.rs"), member_src).expect("write lib.rs");
    std::fs::write(root.join("exceptions.json"), exceptions_json).expect("write exceptions");
    dir
}

fn empty_exceptions() -> String {
    format!(
        r#"{{"schema":"{}","exceptions":[]}}"#,
        exceptions::SCHEMA_VERSION
    )
}

/// A forbidden pattern in a scratch tree fails the check: raw buffer
/// import through an alias, a buffer paint call, and a production expect.
#[test]
fn negative_probe_forbidden_pattern_fails() {
    let dir = scratch_workspace(
        "use ratatui::buffer::Buffer as Buf;\n\
         fn paint(buf: &mut Buf) {\n\
         \x20   buf.set_string(0, 0, \"x\", s);\n\
         }\n\
         fn load(o: Option<u32>) -> u32 {\n\
         \x20   o.expect(\"present\")\n\
         }\n",
        &empty_exceptions(),
    );
    let report = check_workspace(dir.path(), &dir.path().join("exceptions.json"));
    assert!(!report.passed(), "a forbidden pattern must fail the check");
    let kinds: Vec<String> = report.failures.iter().map(|f| f.message.clone()).collect();
    assert!(
        kinds.iter().any(|m| m.contains("OWN-01")),
        "raw buffer import must fail: {kinds:?}"
    );
    assert!(
        kinds.iter().any(|m| m.contains("OWN-02")),
        "buffer paint call must fail: {kinds:?}"
    );
    assert!(
        kinds.iter().any(|m| m.contains("OWN-14")),
        "production expect must fail: {kinds:?}"
    );
    // Alias resolution: the finding names the canonical buffer path.
    let import = report
        .findings
        .iter()
        .find(|f| f.rule == "OWN-01")
        .expect("an OWN-01 finding");
    assert!(
        import.detail.contains("ratatui::buffer::Buffer"),
        "alias must resolve to the canonical path: {}",
        import.detail
    );
}

/// Growth beyond a pinned ratchet maximum fails the check.
#[test]
fn negative_probe_growth_beyond_max_fails() {
    let capped = format!(
        r#"{{"schema":"{}","exceptions":[{{"path":"crates/demo-preview/src/lib.rs","rule":"OWN-02","max":1,"reason":"probe","review":"probe"}}]}}"#,
        exceptions::SCHEMA_VERSION
    );
    let dir = scratch_workspace(
        "use termrock::Ui;\n\
         fn a(ui: &mut Ui) {\n\
         \x20   ui.fill(area, s);\n\
         \x20   ui.fill(area, s);\n\
         }\n",
        &capped,
    );
    let report = check_workspace(dir.path(), &dir.path().join("exceptions.json"));
    assert!(!report.passed(), "growth beyond max must fail");
    assert!(
        report
            .failures
            .iter()
            .any(|f| f.kind == exceptions::FailureKind::Growth),
        "expected a growth failure: {:?}",
        report.failures
    );
}

/// An exception that matches nothing is stale and fails the check.
#[test]
fn negative_probe_stale_exception_fails() {
    let stale = format!(
        r#"{{"schema":"{}","exceptions":[{{"path":"crates/demo-preview/src/lib.rs","rule":"OWN-02","max":3,"reason":"probe","review":"probe"}}]}}"#,
        exceptions::SCHEMA_VERSION
    );
    let dir = scratch_workspace("fn clean() {}\n", &stale);
    let report = check_workspace(dir.path(), &dir.path().join("exceptions.json"));
    assert!(!report.passed(), "a stale exception must fail");
    assert!(
        report
            .failures
            .iter()
            .any(|f| f.kind == exceptions::FailureKind::Stale),
        "expected a stale failure: {:?}",
        report.failures
    );
}

/// An unknown exceptions schema fails closed.
#[test]
fn negative_probe_unknown_schema_fails() {
    let dir = scratch_workspace("fn clean() {}\n", r#"{"schema":"nope/v9","exceptions":[]}"#);
    let report = check_workspace(dir.path(), &dir.path().join("exceptions.json"));
    assert!(!report.passed(), "an unknown schema must fail");
}

/// The rule catalog is stable: 14 rules with unique ids in order.
#[test]
fn rule_catalog_is_stable() {
    let ids: Vec<&str> = rules::RULES.iter().map(|r| r.id).collect();
    assert_eq!(
        ids,
        [
            "OWN-01", "OWN-02", "OWN-03", "OWN-04", "OWN-05", "OWN-06", "OWN-07", "OWN-08",
            "OWN-09", "OWN-10", "OWN-11", "OWN-12", "OWN-13", "OWN-14",
        ]
    );
}

/// Every exception entry in the shipped file is well-formed.
#[test]
fn shipped_exceptions_parse() {
    let root = workspace_root();
    let path: PathBuf = root
        .join("crates")
        .join("termrock-xtask")
        .join("exceptions")
        .join("v1.json");
    let text = std::fs::read_to_string(&path).expect("read shipped exceptions");
    let parsed = exceptions::load_exceptions(&text).expect("shipped exceptions parse");
    assert!(
        !parsed.exceptions.is_empty(),
        "the G1 corpus is non-empty by construction"
    );
    for exception in &parsed.exceptions {
        assert!(
            Path::new(&root).join(&exception.path).exists(),
            "exception path must exist: {}",
            exception.path
        );
        assert!(!exception.reason.is_empty(), "reason required");
        assert!(!exception.review.is_empty(), "review required");
    }
}
