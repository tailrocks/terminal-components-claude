//! TablePro app-journey suite, slice 1 (VB Phase-7g, §15) — impl port.
//!
//! Ported verbatim from VB commit `04121e4bfa2aeeda3893e81f6bb9e84a04d34a29`
//! (`tests/harness/tests/visual_baseline/tablepro_journeys.rs`), adapting
//! only paths, binary resolution, and harness API:
//!
//! - `Session` is the impl [`support::Session`] wrapper (live samples via
//!   `s.inner.observe_now()`), not `tuiscotti::tui::Session` directly.
//! - The subject is the `tablepro` binary built from this impl
//!   worktree's sources, resolved via [`support::try_resolve_bin`]
//!   (name → executed path + sha256); [`write_provenance`] records path,
//!   digest, size, mtime, and argv behind each journey.
//! - `support::drive` (3-arg) becomes [`support::drive_with_timeout`]
//!   (timeout in ms); single steps use [`support::press_step`].
//! - The `--help` enumeration shells the resolved binary path (impl
//!   `TABLEPRO` is a name, not a `CARGO_BIN_EXE_*` absolute path).
//!
//! Multi-step PTY journeys assert needle state across steps — they never
//! gate snapshots, so every case here uses owned (dynamic) names: the
//! approval-inventory parser in [`crate::support::suite_capture_names`]
//! only expands the vendored case registry (never suite sources), and
//! journey names stay out of the committed `snapshots/` inventory.
//!
//! Per-test scratch (checkpoints + provenance) lands under this
//! conformance crate's `target/tuiscotti/journeys/<test>/` (gitignored,
//! unique per test fn). Checkpoints are text frames (`.txt`);
//! `provenance.txt` records the binary identity and argv behind each
//! journey. Typed input is synthetic and in-memory only (simulation data).
//! TablePro is a deterministic demo database (no drivers, a small
//! in-memory engine), so journeys never touch a real database: the
//! connection form is filled but never submitted, and the second session
//! in journey 2 proves the typed name never persisted anywhere.

use std::path::{Path, PathBuf};

use crate::support::Session;
use crate::support::state_waits as waits;
use crate::support::{self, Case, Color, TABLEPRO};

/// Boot needle on the connections screen: the Production tree row.
const BOOT_CONN: &str = "Production";
/// Boot needle once connected: the first query tab.
const BOOT_WB: &str = "Query 1";
const COLS: u16 = 120;
const ROWS: u16 = 40;

/// Owned-name journey case at the canonical 120x40 truecolor geometry.
fn journey_case(name: &str, args: &'static [&'static str], boot: &'static str) -> Case {
    Case::dynamic(
        format!("journeys/tablepro/{name}"),
        TABLEPRO,
        args,
        COLS,
        ROWS,
        Color::Truecolor,
        boot,
    )
}

/// Unique scratch dir for one journey test (created, never shared).
fn journey_dir(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/tuiscotti/journeys")
        .join(test);
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("mkdir {}: {e}", dir.display()));
    dir
}

/// Record the binary identity behind a journey: resolved absolute path,
/// sha256 digest, byte size, mtime, and argv. Written to
/// `provenance.txt`, echoed too.
fn write_provenance(dir: &Path, case: &Case) {
    write_provenance_named(dir, case, "provenance.txt");
}

/// [`write_provenance`] with an explicit filename, for multi-session
/// journeys where each session's argv deserves its own record.
fn write_provenance_named(dir: &Path, case: &Case, file: &str) {
    let subject =
        support::try_resolve_bin(case.bin).unwrap_or_else(|e| panic!("resolve {}: {e}", case.bin));
    let modified = subject
        .mtime_unix
        .map_or_else(|| "unknown".to_string(), |m| m.to_string());
    let argv = support::argv_for(case);
    let body = format!(
        "profile: tuiscotti-default\ntransport: pty\nport_of: 04121e4bfa2aeeda3893e81f6bb9e84a04d34a29\nbinary: {}\nsha256: {}\nsize_bytes: {}\nmodified_unix: {modified}\nargv: {}\n",
        subject.path.display(),
        subject.sha256,
        subject.len,
        argv.join(" "),
    );
    std::fs::write(dir.join(file), &body)
        .unwrap_or_else(|e| panic!("write {}: {e}", dir.join(file).display()));
    eprintln!("journey provenance ({file}): {body}");
}

/// Save the live screen text as `<name>.txt` in the journey dir.
fn checkpoint(s: &mut Session, dir: &Path, name: &str) {
    let obs = s
        .inner
        .observe_now()
        .unwrap_or_else(|e| panic!("checkpoint `{name}` sample failed: {e:#}"));
    let text = support::screen_text(&obs.screen);
    let body = format!(
        "# checkpoint {name} {}x{}\n{text}\n",
        obs.screen.cols(),
        obs.screen.rows()
    );
    let path = dir.join(format!("{name}.txt"));
    std::fs::write(&path, body).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
    eprintln!("checkpoint {name} ({} bytes visible text)", text.len());
}

/// Slice 1, journey 1: enumerate the connection entry point (the binary's
/// own `--help` must document `--connect`), then visit the connection
/// tree — seven downs from the boot cursor reach Production
/// (`tablepro.rs`: rows 0 Personal … 7 Production), proven by the detail
/// card following the cursor (Production host appears, Local host
/// non-vacuously leaves).
#[test]
#[ignore = "tablepro journey; run with --ignored"]
fn journey_tablepro_connections_tree() {
    let dir = journey_dir("journey_tablepro_connections_tree");

    let tablepro =
        support::try_resolve_bin(TABLEPRO).unwrap_or_else(|e| panic!("resolve {TABLEPRO}: {e}"));
    let help = std::process::Command::new(&tablepro.path)
        .arg("--help")
        .output()
        .unwrap_or_else(|e| panic!("tablepro --help failed: {e}"));
    assert!(
        help.status.success(),
        "tablepro --help exits 0 (got {})",
        help.status
    );
    let text = String::from_utf8_lossy(&help.stdout);
    std::fs::write(dir.join("connect-help.txt"), text.as_bytes())
        .unwrap_or_else(|e| panic!("write connect-help.txt: {e}"));
    assert!(
        text.contains("--connect"),
        "help documents `--connect` ({}/connect-help.txt)",
        dir.display()
    );
    eprintln!("enumeration: --connect documented");

    let case = journey_case("connections_tree", &[], BOOT_CONN);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    // Boot selection is the Local connection: its host anchors the
    // detail-follows-cursor proof (and the wait_gone below).
    waits::wait_state(&mut s, "localhost:5432", "boot detail is Local");
    checkpoint(&mut s, &dir, "00-connections");
    support::drive_with_timeout(
        &mut s,
        &["down", "down", "down", "down", "down", "down", "down"],
        case.timeout_ms,
    );
    waits::wait_state(&mut s, "prod-db-1.acme.io", "Production detail card");
    waits::wait_state(&mut s, "acme_prod", "Production database");
    // The boot checkpoint above proves `localhost:5432` was present, so
    // this absence poll is non-vacuous across checkpoints (the holla
    // jump-recovery pattern — `wait_gone` cannot apply here: its inline
    // presence sample would land after the multi-step drive flipped the
    // card already).
    support::wait_screen(
        &mut s,
        support::DEFAULT_WAIT,
        "Local detail left",
        |screen| !support::screen_text(screen).contains("localhost:5432"),
    );
    checkpoint(&mut s, &dir, "01-production-detail");
    eprintln!("journey connections_tree: tree visited, Production detail proven");
}

/// Slice 1, journey 2: the connection form. Ctrl+N opens a *new* form
/// (title proves it — an edit form would say `Edit connection`), Enter
/// begins editing the Name field (typing alone does not: `TextInput`
/// only commits Enter/F2 while idle, and a stray `e` would re-open the
/// form as an edit — the lesson baked into the `form_new-filled`
/// capture), then a synthetic name is typed and proven on screen. The
/// form is never submitted: a second session boots the untouched tree
/// and asserts the typed name persisted nowhere — in-memory only.
#[test]
#[ignore = "tablepro journey; run with --ignored"]
fn journey_tablepro_connection_form() {
    const TYPED: &str = "Journey staging";

    let dir = journey_dir("journey_tablepro_connection_form");
    let case = journey_case("connection_form", &[], BOOT_CONN);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-connections");

    support::drive_with_timeout(&mut s, &["ctrl-n", "wait:New connection"], case.timeout_ms);
    waits::wait_state(&mut s, "Name", "name field label");
    checkpoint(&mut s, &dir, "01-form-open");
    support::drive_with_timeout(
        &mut s,
        &["enter", &format!("type:{TYPED}"), &format!("wait:{TYPED}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-form-filled");

    // Second session: the unsubmitted form left no trace.
    let case2 = journey_case("connection_form_clean", &[], BOOT_CONN);
    write_provenance_named(&dir, &case2, "provenance-clean.txt");
    let mut s2 = support::spawn_boot(&case2);
    let after = support::screen_text(
        &s2.inner
            .observe_now()
            .unwrap_or_else(|e| panic!("clean-boot sample failed: {e:#}"))
            .screen,
    );
    assert!(
        !after.contains(TYPED),
        "unsubmitted form left no `{TYPED}` on a fresh boot"
    );
    checkpoint(&mut s2, &dir, "03-clean-boot");
    eprintln!("journey connection_form: filled, proven, persisted nowhere");
}

/// Slice 1, journey 3: workbench layout plus the table data/structure
/// views. `--connect Production` boots the workbench (checkpointed with
/// its Explorer + Query 1 layout); five downs + Enter open the orders
/// table, proven by the `public › orders` tab plus a data cell; Ctrl+D
/// flips to Structure, proven non-vacuously by the structure-only
/// sub-tabs appearing while the data cell leaves.
#[test]
#[ignore = "tablepro journey; run with --ignored"]
fn journey_tablepro_workbench_table_views() {
    let dir = journey_dir("journey_tablepro_workbench_table_views");
    let case = journey_case(
        "workbench_table_views",
        &["--connect", "Production"],
        BOOT_WB,
    );
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    waits::wait_state(&mut s, "Explorer", "explorer pane");
    checkpoint(&mut s, &dir, "00-workbench");

    support::drive_with_timeout(
        &mut s,
        &[
            "down",
            "down",
            "down",
            "down",
            "down",
            "enter",
            "wait:public › orders",
        ],
        case.timeout_ms,
    );
    waits::wait_state(&mut s, "10000", "first data cell");
    checkpoint(&mut s, &dir, "01-data");
    support::press_step(&s, "ctrl-d");
    // `wait_gone` first: `press_step` (unlike `drive_with_timeout`) has
    // no trailing sleep, so its presence sample still sees the data grid
    // before the app processes the chord (the jackin save-cancel pattern).
    waits::wait_gone(&mut s, "10000", "data grid left");
    waits::wait_state(&mut s, "Foreign keys", "structure sub-tabs");
    checkpoint(&mut s, &dir, "02-structure");
    eprintln!("journey workbench_table_views: data then structure, both proven");
}

/// Slice 1, journey 4: the query editor with assertion-proven states.
/// Tab + `i` enter the editor, typing a partial FROM triggers
/// completion (proven by the `order_items` popup row), Enter accepts,
/// a partial WHERE triggers column completion (Tab accepts `status`),
/// and Ctrl+R runs the finished statement — proven by the `25 rows`
/// result header plus the `Ctrl+X Explain` results footer. The query
/// is a read over the in-memory demo engine: nothing is written.
#[test]
#[ignore = "tablepro journey; run with --ignored"]
fn journey_tablepro_query_completion_results() {
    let dir = journey_dir("journey_tablepro_query_completion_results");
    let case = journey_case(
        "query_completion_results",
        &["--connect", "Production"],
        BOOT_WB,
    )
    .timeout(15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-workbench");

    support::drive_with_timeout(
        &mut s,
        &["tab", "i", "type:SELECT * FROM ord", "wait:order_items"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-completion");
    support::drive_with_timeout(
        &mut s,
        &[
            "enter",
            "type: WHERE st",
            "tab",
            "type: = 'pending' ORDER BY created_at DESC LIMIT 25",
            "escape",
            "ctrl-r",
            "wait:25 rows",
            "wait:Ctrl+X Explain",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-results");
    eprintln!("journey query_completion_results: completion triggered, 25 rows proven");
}
