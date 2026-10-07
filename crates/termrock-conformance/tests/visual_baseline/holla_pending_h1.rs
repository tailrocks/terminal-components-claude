//! Holla pending-roots slice 8A-H1 executable checks — impl port.
//!
//! Ported verbatim from VB commit `e740009d378f2ace27b8df20870c3177005a9230`
//! (`tests/harness/tests/visual_baseline/holla_pending_h1.rs`), adapting
//! only paths, binary resolution, and harness API:
//!
//! - `Session` is the impl [`support::Session`] wrapper (live samples via
//!   `s.inner.observe_now()`), not `tuiscotti::tui::Session` directly.
//! - The subject is the `holla` binary built from this impl
//!   worktree's sources, resolved via [`support::try_resolve_bin`]
//!   (name -> executed path + sha256); [`write_provenance`] records path,
//!   digest, size, mtime, and argv behind each check.
//! - `support::drive` (3-arg) becomes [`support::drive_with_timeout`]
//!   (timeout in ms).
//!
//! One ignored PTY test per H1 registry row (29 rows), over the real
//! `holla` binary built from this worktree's impl sources
//! (resolved via [`support::try_resolve_bin`]). Each test drives the row's
//! inputs and asserts its visual (V), state (S), action (A), and
//! negative (N) checks in live-PTY executable form:
//!
//! - V: live needles plus same-line coexistence for finder rows,
//!   cleanup review rows, gate fact rows, dialog titles, disk tree rows,
//!   and report outcome rows.
//! - S: live labels (cursor/focus/footer/status rows) plus sort, fold,
//!   selection, and gate-phase state.
//! - A: behavior probes — the specified gesture runs and the specified
//!   advance (or boundary refusal) is observed; `Changed`/`Consumed`/
//!   `Ignored` outcomes are proven by their screen correlates (moved,
//!   byte-identical), since PTY cannot see the enum.
//! - N: absence assertions, each paired with a presence proof in the same
//!   test (boot checkpoint, mid-flow checkpoint, or pre/post transition)
//!   so no absence passes vacuously.
//!
//! No new snapshots and no new static captures: all 29 roots already have
//! approved frames gated cell-exact by the ported matrices in `holla.rs`
//! (plan §Reconciliation: "rows plus checks, not recaptures"), so every
//! case here uses owned (dynamic) names and stays out of the `snapshots/`
//! inventory. No isolated-component captures: every row's `requires` set
//! (key-injection, glyph-capture) is PTY-level, and every assertion has a
//! live-PTY executable form.
//!
//! Per-test scratch lands under this conformance crate's
//! `target/tuiscotti/journeys/<test>/` (gitignored): the row's named
//! checkpoint(s) as text frames, `00-boot.txt` where the row needs a boot
//! comparison, and `provenance.txt` (binary path, digest, size, mtime, argv).
//! Typed input is synthetic and in-memory only (simulation data); no test
//! runs real cleanup, touches Git state, or performs destructive
//! operations: every gate-2 dialog is cancelled, never executed, every
//! typed confirmation phrase is synthetic fixture text, and every world
//! is the fixture simulation. The two report rows assert the simulated
//! post-commit pages; they never commit anything themselves.
//!
//! Row → test map (registry id → `h1_*` test):
//!
//! - PANEL-CARGO-001 → [`h1_cargo_clean`]
//! - DIALOG-CARGO-001 → [`h1_cargo_clean_confirm`]
//! - PANEL-CLEANUP-006 → [`h1_cleanup_artifacts`]
//! - PANEL-CLEANUP-003 → [`h1_cleanup_categories`]
//! - PANEL-CLEANUP-004 → [`h1_cleanup_derived_data`]
//! - DIALOG-CLEANUP-001 → [`h1_cleanup_gate_seq`]
//! - DIALOG-CLEANUP-002 → [`h1_cleanup_gate1`]
//! - DIALOG-CLEANUP-003 → [`h1_cleanup_gate2`]
//! - DIALOG-CLEANUP-004 → [`h1_cleanup_gate2_typed`]
//! - PANEL-CLEANUP-005 → [`h1_cleanup_history`]
//! - PANEL-CLEANUP-007 → [`h1_cleanup_linux_report`]
//! - PANEL-CLEANUP-001 → [`h1_cleanup_plan`]
//! - PANEL-CLEANUP-002 → [`h1_cleanup_report`]
//! - PANEL-DISK-001 → [`h1_disk_scan_complete`]
//! - PANEL-DISK-002 → [`h1_disk_top_files`]
//! - TREE-DISK-001 → [`h1_disk_tree`]
//! - TREE-DISK-002 → [`h1_disk_tree_apparent`]
//! - TREE-DISK-004 → [`h1_disk_tree_selected`]
//! - TREE-DISK-003 → [`h1_disk_tree_unfolded`]
//! - PANEL-DOCKER-001 → [`h1_docker_done`]
//! - DIALOG-DOCKER-001 → [`h1_docker_drift`]
//! - DIALOG-DOCKER-002 → [`h1_docker_gate2_typed`]
//! - PANEL-DOCKER-002 → [`h1_docker_remove_failed`]
//! - DIALOG-DOCKER-003 → [`h1_docker_remove_gate1`]
//! - DIALOG-DOCKER-004 → [`h1_docker_remove_gate2`]
//! - PANEL-GRADLE-001 → [`h1_gradle_cleanup`]
//! - DIALOG-GRADLE-001 → [`h1_gradle_gate1`]
//! - PANEL-IDEA-001 → [`h1_idea_cleanup`]
//! - DIALOG-IDEA-001 → [`h1_idea_gate2`]

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::support::Session;

use crate::support::state_waits as waits;
use crate::support::{self, Case, Color, HOLLA};

/// Boot needle on every holla route: the brand.
const BOOT: &str = "holla❯";

const CARGO: &[&str] = &[
    "--scenario",
    "parity-cargo",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const INSIGHTS: &[&str] = &[
    "--scenario",
    "parity-insights",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const PLAN80: &[&str] = &[
    "--scenario",
    "disk-cleanup",
    "--motion",
    "paused",
    "--frame",
    "80",
];
const DELSAFE: &[&str] = &["--scenario", "parity-delete-safety", "--motion", "reduced"];
const HIST_PAUSED: &[&str] = &[
    "--scenario",
    "parity-cleanup-results",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const HIST_REDUCED: &[&str] = &[
    "--scenario",
    "parity-cleanup-results",
    "--motion",
    "reduced",
];
const LINUX: &[&str] = &[
    "--scenario",
    "parity-platforms-linux",
    "--motion",
    "reduced",
];
const SCAN: &[&str] = &["--scenario", "parity-disk-scan", "--motion", "reduced"];
const NAV: &[&str] = &[
    "--scenario",
    "parity-disk-navigation",
    "--motion",
    "reduced",
];
const DOCKER_PAUSED: &[&str] = &[
    "--scenario",
    "docker-cleanup",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const DOCKER_REDUCED: &[&str] = &["--scenario", "docker-cleanup", "--motion", "reduced"];
const PDOCKER_RED: &[&str] = &["--scenario", "parity-docker", "--motion", "reduced"];
const PDOCKER_PAUSED: &[&str] = &[
    "--scenario",
    "parity-docker",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const GRADLE: &[&str] = &[
    "--scenario",
    "parity-gradle",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const IDEA: &[&str] = &[
    "--scenario",
    "parity-idea",
    "--motion",
    "paused",
    "--frame",
    "40",
];

/// Gate-2 dialog title suffix shared by every two-gate flow.
const GATE2: &str = "gate 2 of 2";
/// Gate-1 page title suffix shared by every two-gate flow.
const GATE1: &str = "gate 1 of 2";

/// Owned-name H1 case at the canonical 120x40 truecolor geometry.
fn h1_case(slug: &str, args: &'static [&'static str], boot: &'static str) -> Case {
    Case::dynamic(
        format!("journeys/holla/h1/{slug}"),
        HOLLA,
        args,
        120,
        40,
        Color::Truecolor,
        boot,
    )
}

/// [`h1_case`] with an explicit per-step timeout for tick-driven flows.
fn h1_case_t(
    slug: &str,
    args: &'static [&'static str],
    boot: &'static str,
    timeout_ms: u64,
) -> Case {
    h1_case(slug, args, boot).timeout(timeout_ms)
}

/// Unique scratch dir for one H1 test (created, never shared).
fn h1_dir(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/tuiscotti/journeys")
        .join(test);
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("mkdir {}: {e}", dir.display()));
    dir
}

/// Record the binary identity: resolved absolute path, sha256 digest, byte size, mtime, argv.
/// Written to `provenance.txt`, echoed too (the build-identity record).
fn write_provenance(dir: &Path, case: &Case) {
    let subject =
        support::try_resolve_bin(case.bin).unwrap_or_else(|e| panic!("resolve {}: {e}", case.bin));
    let modified = subject
        .mtime_unix
        .map_or_else(|| "unknown".to_string(), |m| m.to_string());
    let argv = support::argv_for(case);
    let body = format!(
        "profile: tuiscotti-default\ntransport: pty\nport_of: e740009d378f2ace27b8df20870c3177005a9230\nbinary: {}\nsha256: {}\nsize_bytes: {}\nmodified_unix: {modified}\nargv: {}\n",
        subject.path.display(),
        subject.sha256,
        subject.len,
        argv.join(" "),
    );
    std::fs::write(dir.join("provenance.txt"), &body)
        .unwrap_or_else(|e| panic!("write {}: {e}", dir.join("provenance.txt").display()));
    eprintln!("h1 provenance: {body}");
}

/// Save the live screen text as `<name>.txt` in the test dir.
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

fn case_timeout(case: &Case) -> Duration {
    Duration::from_millis(case.timeout_ms)
}

/// One fresh live text sample (no wait).
fn live_text(s: &mut Session) -> String {
    let obs = s
        .inner
        .observe_now()
        .unwrap_or_else(|e| panic!("live sample failed: {e:#}"));
    support::screen_text(&obs.screen)
}

/// Some line must contain both `needle` and `also` (the same-row
/// coexistence proof: finder rows, review rows, dialog rows, tree rows).
fn assert_line_has(text: &str, needle: &str, also: &str, what: &str) {
    assert!(
        text.lines().any(|l| l.contains(needle) && l.contains(also)),
        "{what}: no line contains both `{needle}` and `{also}`"
    );
}

/// PANEL-CARGO-001 (`holla/flows/cargo/clean`): the cargo-clean finder
/// row with its preview facts.
///
/// Flow: boot, type `cargo clean`, Enter (dialog presence proof), Esc
/// (cancelled), Enter (dialog again), Esc (cancelled again).
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cargo_clean() {
    const DIALOG_FOOTER: &str = "← → Choose  Enter Confirm  Esc Cancel";
    let dir = h1_dir("h1_cargo_clean");
    let case = h1_case("cargo_clean", CARGO, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["type:cargo clean", "wait:900.0 MiB"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-clean");
    let clean = live_text(&mut s);

    // V1: the ranked row pairs the action with the destructive badge.
    assert_line_has(&clean, "› cargo clean · 900.0 MiB", "destructive", "V1 row");
    assert!(clean.contains("Results · 3"), "V1: no Results · 3");
    // V2: the preview resolves the target directory.
    assert!(
        clean.contains("remove the resolved"),
        "V2: no resolved-target fact"
    );
    assert!(clean.contains("~/work/engine/target"), "V2: no target path");
    // S1: the finder footer offers the confirm gesture.
    assert_line_has(&clean, "Enter Confirm…", "Alt+Enter More", "S1 footer");
    // S2: the gate fact names the one-confirmation flow.
    assert_line_has(&clean, "Gate", "one confirmation", "S2 gate");
    // N1: the confirm dialog is closed at the clean state.
    assert!(
        !clean.contains(DIALOG_FOOTER),
        "N1: dialog open before Enter"
    );

    // A1/N1 presence: Enter opens the one-confirmation dialog …
    support::drive_with_timeout(
        &mut s,
        &["enter", &format!("wait:{DIALOG_FOOTER}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-dialog-presence");
    // … and Esc cancels it back to the clean state.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DIALOG_FOOTER, "dialog cancelled");
    waits::wait_state(&mut s, "900.0 MiB", "clean state back");
    // A2: the open/cancel roundtrip repeats.
    support::drive_with_timeout(
        &mut s,
        &["enter", &format!("wait:{DIALOG_FOOTER}")],
        case.timeout_ms,
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DIALOG_FOOTER, "dialog cancelled again");
    let back = live_text(&mut s);
    // N2: the dialog is gone after the second cancel, the row intact.
    assert!(!back.contains(DIALOG_FOOTER), "N2: dialog stuck after Esc");
    assert_line_has(
        &back,
        "› cargo clean · 900.0 MiB",
        "destructive",
        "row intact",
    );
    checkpoint(&mut s, &dir, "03-clean-again");
    eprintln!("h1 cargo_clean: row, dialog roundtrip x2");
}

/// DIALOG-CARGO-001 (`holla/flows/cargo/clean_confirm`): the cargo-clean
/// one-confirmation dialog.
///
/// Flow: boot, type `cargo clean`, Enter (dialog), Esc (cancel), Enter
/// (dialog again), Esc (cancel again). Never Run: the dialog only ever
/// cancels.
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cargo_clean_confirm() {
    const DIALOG_FOOTER: &str = "← → Choose  Enter Confirm  Esc Cancel";
    let dir = h1_dir("h1_cargo_clean_confirm");
    let case = h1_case("cargo_clean_confirm", CARGO, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:cargo clean",
            "enter",
            &format!("wait:{DIALOG_FOOTER}"),
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-dialog");
    let dialog = live_text(&mut s);

    // V1: the dialog restates the action and the resolved target.
    assert_line_has(
        &dialog,
        "Action",
        "remove the resolved target directory",
        "V1 action",
    );
    // V2: the Cancel/Run choice row.
    assert_line_has(&dialog, "▎Cancel", "Run", "V2 choice");
    // S1: the dialog footer (also the N1 presence proof).
    assert!(dialog.contains(DIALOG_FOOTER), "S1: no dialog footer");
    // S2: the one-explicit-confirmation fact.
    assert_line_has(
        &dialog,
        "Confirmation",
        "one explicit confirmation",
        "S2 confirmation",
    );

    // A1: Esc cancels the dialog back to the clean row.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DIALOG_FOOTER, "dialog cancelled");
    waits::wait_state(&mut s, "900.0 MiB", "clean row back");
    checkpoint(&mut s, &dir, "02-cancelled");
    let cancelled = live_text(&mut s);
    // N1: the dialog chrome is gone with the cancel.
    assert!(
        !cancelled.contains(DIALOG_FOOTER),
        "N1: dialog footer stuck"
    );
    assert!(
        !cancelled.contains("one explicit confirmation"),
        "N1: confirmation fact stuck"
    );

    // A2: Enter re-opens the dialog, Esc cancels it again.
    support::drive_with_timeout(
        &mut s,
        &["enter", &format!("wait:{DIALOG_FOOTER}")],
        case.timeout_ms,
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DIALOG_FOOTER, "dialog cancelled again");
    let back = live_text(&mut s);
    // N2: nothing stuck open after the second cancel either.
    assert!(!back.contains(DIALOG_FOOTER), "N2: dialog stuck again");
    assert_line_has(
        &back,
        "› cargo clean · 900.0 MiB",
        "destructive",
        "row intact",
    );
    checkpoint(&mut s, &dir, "03-cancelled-again");
    eprintln!("h1 cargo_clean_confirm: dialog, cancel roundtrip x2");
}

/// PANEL-CLEANUP-006 (`holla/flows/cleanup/artifacts`): the cleanup
/// review with the cursor jumped to the last category.
///
/// Flow: boot, query the review, End (last row), Up/Down (cursor
/// roundtrip), Right (fold opens), Left (fold closes).
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cleanup_artifacts() {
    let dir = h1_dir("h1_cleanup_artifacts");
    let case = h1_case("cleanup_artifacts", INSIGHTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Review cleanup candidates",
            "enter",
            "wait:18 categories",
            "end",
            "wait:Project artifacts",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-artifacts");
    let arts = live_text(&mut s);

    // V1: the cursor sits on the last category row.
    assert_line_has(&arts, "▎[ ] ▸ Project artifacts", "2.5 GiB", "V1 cursor");
    // V2: the detail panel scopes to project artifacts.
    assert!(
        arts.contains("14 of 14 candidates eligible"),
        "V2: no eligibility fact"
    );
    // S1: the review header counts hold.
    assert_line_has(
        &arts,
        "18 categories",
        "19 selected (12.4 GiB)",
        "S1 header",
    );
    // S2: the review footer offers deletion review.
    assert!(arts.contains("d Review deletion"), "S2: no d hint");
    // N1: the artifacts fold is closed at the jumped state.
    assert!(
        !arts.contains("▾ Project artifacts"),
        "N1: fold open before Right"
    );

    // A1: Up moves the cursor off the last row, Down returns it.
    support::press_step(&s, "up");
    waits::wait_state(&mut s, "▎[✓] ▸ JetBrains logs", "cursor up");
    support::press_step(&s, "down");
    waits::wait_state(&mut s, "▎[ ] ▸ Project artifacts", "cursor back");
    // A2/N1 presence: Right opens the fold, Left closes it again.
    support::press_step(&s, "right");
    waits::wait_state(&mut s, "▾ Project artifacts", "fold opened");
    checkpoint(&mut s, &dir, "02-fold-presence");
    support::press_step(&s, "left");
    waits::wait_gone(&mut s, "▾ Project artifacts", "fold closed");
    waits::wait_state(&mut s, "▎[ ] ▸ Project artifacts", "cursor intact");
    let back = live_text(&mut s);
    // N2: the fold is closed again after the roundtrip.
    assert!(!back.contains("▾ Project artifacts"), "N2: fold stuck open");
    checkpoint(&mut s, &dir, "03-fold-closed");
    eprintln!("h1 cleanup_artifacts: last row, cursor + fold roundtrips");
}

/// PANEL-CLEANUP-003 (`holla/flows/cleanup/categories`): the cleanup
/// review landing page.
///
/// Flow: boot, query the review, Down/Up (cursor roundtrip), Right
/// (fold opens), Left (fold closes).
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cleanup_categories() {
    let dir = h1_dir("h1_cleanup_categories");
    let case = h1_case("cleanup_categories", INSIGHTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Review cleanup candidates",
            "enter",
            "wait:18 categories",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-categories");
    let cats = live_text(&mut s);

    // V1: the review header with counts.
    assert_line_has(&cats, "18 categories", "18.8 GiB", "V1 header");
    // V2: the first category row.
    assert_line_has(
        &cats,
        "▎[–] ▸ Xcode DerivedData",
        "810.0 MiB",
        "V2 first row",
    );
    // S1: the detail panel scopes to the cursor row.
    assert_line_has(&cats, "Safety", "rebuilt on demand", "S1 detail");
    // S2: the review footer hints.
    assert_line_has(&cats, "m Permanent", "n Dry run", "S2 footer");
    // N1: the first fold is closed at the landing state.
    assert!(
        !cats.contains("▾ Xcode DerivedData"),
        "N1: fold open before Right"
    );

    // A1: Down moves the cursor, Up returns it.
    support::press_step(&s, "down");
    waits::wait_state(&mut s, "▎[✓] ▸ Xcode device support", "cursor down");
    support::press_step(&s, "up");
    waits::wait_state(&mut s, "▎[–] ▸ Xcode DerivedData", "cursor back");
    // A2/N1 presence: Right opens the fold, Left closes it again.
    support::press_step(&s, "right");
    waits::wait_state(&mut s, "▾ Xcode DerivedData", "fold opened");
    checkpoint(&mut s, &dir, "02-fold-presence");
    support::press_step(&s, "left");
    waits::wait_gone(&mut s, "▾ Xcode DerivedData", "fold closed");
    waits::wait_state(&mut s, "▎[–] ▸ Xcode DerivedData", "cursor intact");
    let back = live_text(&mut s);
    // N2: the fold is closed again after the roundtrip.
    assert!(!back.contains("▾ Xcode DerivedData"), "N2: fold stuck open");
    checkpoint(&mut s, &dir, "03-fold-closed");
    eprintln!("h1 cleanup_categories: landing, cursor + fold roundtrips");
}

/// PANEL-CLEANUP-004 (`holla/flows/cleanup/derived_data`): the cleanup
/// review with the first category fold opened.
///
/// Flow: boot, query the review, Right (fold opens), Left (closes),
/// Right (opens again).
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cleanup_derived_data() {
    let dir = h1_dir("h1_cleanup_derived_data");
    let case = h1_case("cleanup_derived_data", INSIGHTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Review cleanup candidates",
            "enter",
            "wait:18 categories",
            "right",
            "wait:▾ Xcode DerivedData",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-derived");
    let open = live_text(&mut s);

    // V1: the opened fold header.
    assert_line_has(&open, "▾ Xcode DerivedData", "810.0 MiB", "V1 fold");
    // V2: the folded candidate row.
    assert_line_has(
        &open,
        "~/Library/Developer/Xcode/D…",
        "30 d",
        "V2 candidate",
    );
    // S1: the guard fact explains the skip.
    assert_line_has(&open, "Guard", "Xcode is running", "S1 guard");
    // S2: the review footer survives the fold.
    assert!(open.contains("d Review deletion"), "S2: no d hint");

    // A1: Left closes the fold …
    support::press_step(&s, "left");
    waits::wait_gone(&mut s, "▾ Xcode DerivedData", "fold closed");
    waits::wait_state(&mut s, "▸ Xcode DerivedData", "closed marker back");
    checkpoint(&mut s, &dir, "02-closed");
    let closed = live_text(&mut s);
    // N1: the opened fold is gone with the close.
    assert!(
        !closed.contains("▾ Xcode DerivedData"),
        "N1: fold stuck open"
    );
    assert!(
        !closed.contains("~/Library/Developer/Xcode/D…"),
        "N1: candidate row stuck"
    );

    // A2: Right re-opens the same fold.
    support::press_step(&s, "right");
    waits::wait_state(&mut s, "▾ Xcode DerivedData", "fold reopened");
    let again = live_text(&mut s);
    // N2: the closed marker is gone while the fold is open.
    assert!(
        !again.contains("▸ Xcode DerivedData"),
        "N2: closed marker stuck"
    );
    assert_line_has(&again, "▾ Xcode DerivedData", "810.0 MiB", "fold intact");
    checkpoint(&mut s, &dir, "03-reopened");
    eprintln!("h1 cleanup_derived_data: fold, close roundtrip, reopen");
}

/// DIALOG-CLEANUP-001 (`holla/flows/cleanup/gate-1`): the plan gate-1
/// review with its command sequence.
///
/// Flow: boot (disk-cleanup plan), c (plan), c (gate 1), Right+Enter
/// (gate 2 presence proof), Esc (gate 2 cancelled), Esc (plan back).
/// This is the plan-family gate-1: the `gate1` twin below is the
/// delete-safety-family gate-1 over a different fixture.
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cleanup_gate_seq() {
    let dir = h1_dir("h1_cleanup_gate_seq");
    let case = h1_case("cleanup_gate_seq", PLAN80, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(&mut s, &["c", "wait:4 steps · 4 included"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-plan");
    support::drive_with_timeout(&mut s, &["c", &format!("wait:{GATE1}")], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-gate1");
    let gate = live_text(&mut s);

    // V1: the gate-1 title names the plan and the phase.
    assert_line_has(
        &gate,
        "Review · Clean developer artifacts under ~/work",
        GATE1,
        "V1 title",
    );
    // V2: the command sequence box.
    assert_line_has(&gate, "Sequence · 4 commands", "y copies", "V2 sequence");
    assert_line_has(
        &gate,
        "cargo clean",
        "/Users/alex/work/backend",
        "V2 sequence row",
    );
    // S1: the gate footer (also the N1 presence proof anchor).
    assert_line_has(&gate, "y Copy sequence", "Esc Cancel", "S1 footer");
    // S2: the review status line.
    assert!(
        gate.contains("review · gate 1 of 2"),
        "S2: no review status"
    );
    // N1: the gate-2 dialog is closed at the gate-1 state.
    assert!(!gate.contains(GATE2), "N1: gate 2 open before Right+Enter");

    // A1/N1 presence: Right+Enter opens gate 2, Esc cancels it.
    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "03-gate2-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled");
    waits::wait_state(&mut s, GATE1, "gate 1 back");

    // A2: Esc leaves gate 1 back to the plan page.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE1, "gate 1 left");
    waits::wait_state(&mut s, "4 steps · 4 included", "plan back");
    checkpoint(&mut s, &dir, "04-plan");
    let back = live_text(&mut s);
    // N2: the gate chrome is gone with the leave.
    assert!(!back.contains(GATE1), "N2: gate 1 stuck");
    assert!(!back.contains(GATE2), "N2: gate 2 stuck");
    assert_line_has(
        &back,
        "▎[✓] 01 Trash frontend › node_modules",
        "optional",
        "plan intact",
    );
    eprintln!("h1 cleanup_gate_seq: gate 1, gate-2 probe, plan back");
}

/// DIALOG-CLEANUP-002 (`holla/flows/cleanup/gate1`): the delete-safety
/// gate-1 review over the finished disk scan.
///
/// Flow: boot, run the scan, select the row, d (gate 1), Right+Enter
/// (gate 2 presence proof), Esc (cancelled), Right+Enter (again), Esc
/// (cancelled again).
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cleanup_gate1() {
    let dir = h1_dir("h1_cleanup_gate1");
    let case = h1_case_t("cleanup_gate1", DELSAFE, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Analyze disk usage",
            "enter",
            "wait:1 unreadable",
            "down",
            "space",
            "d",
            &format!("wait:{GATE1}"),
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-gate1");
    let gate = live_text(&mut s);

    // V1: the gate-1 title names the trash target and the phase.
    assert_line_has(&gate, "Review · trash 1 item", GATE1, "V1 title");
    // V2: the review revision pins the resolved facts.
    assert_line_has(&gate, "Revision", "68c21e8eb8ed0a71", "V2 revision");
    // S1: the recovery fact.
    assert_line_has(
        &gate,
        "Recoverable",
        "yes · until the Trash is emptied",
        "S1 recovery",
    );
    // S2: the selection estimate survives into the review.
    assert!(
        gate.contains("1 item selected · 1000.0 MiB estimated"),
        "S2: no selection estimate"
    );
    // N1: the gate-2 dialog is closed at the gate-1 state.
    assert!(!gate.contains(GATE2), "N1: gate 2 open before Right+Enter");

    // A1/N1 presence: Right+Enter opens gate 2, Esc cancels it.
    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-gate2-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled");
    waits::wait_state(&mut s, GATE1, "gate 1 back");

    // A2: the open/cancel roundtrip repeats.
    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled again");
    let back = live_text(&mut s);
    // N2: the gate-2 chrome is gone after the second cancel.
    assert!(!back.contains(GATE2), "N2: gate 2 stuck");
    assert!(!back.contains("to confirm"), "N2: phrase prompt stuck");
    assert_line_has(&back, "Review · trash 1 item", GATE1, "gate 1 intact");
    checkpoint(&mut s, &dir, "03-gate1-again");
    eprintln!("h1 cleanup_gate1: gate 1, gate-2 roundtrip x2");
}

/// DIALOG-CLEANUP-003 (`holla/flows/cleanup/gate2`): the delete-safety
/// gate-2 typed-phrase dialog, unarmed.
///
/// Flow: boot, run the scan, select the row, d (gate 1), Right+Enter
/// (gate 2), Esc (cancelled), Right+Enter (again), Esc (cancelled
/// again). Never armed, never executed.
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cleanup_gate2() {
    const PHRASE: &str = "Type TRASH 1 UNDER /Users/alex/Projects/safe ON mbp to confirm";
    let dir = h1_dir("h1_cleanup_gate2");
    let case = h1_case_t("cleanup_gate2", DELSAFE, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Analyze disk usage",
            "enter",
            "wait:1 unreadable",
            "down",
            "space",
            "d",
            &format!("wait:{GATE1}"),
            "right",
            "enter",
            &format!("wait:{GATE2}"),
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-gate2");
    let gate = live_text(&mut s);

    // V1: the gate-2 dialog title.
    assert!(
        gate.contains("Trash 1 items · gate 2 of 2"),
        "V1: no gate-2 title"
    );
    // V2: the target-bound phrase prompt.
    assert!(gate.contains(PHRASE), "V2: no phrase prompt");
    // S1: the gate-2 footer (also the N1 presence proof).
    assert_line_has(&gate, "Enter Confirm", "Esc Cancel", "S1 footer");
    // S2: the Execute choice is present but untouched.
    assert_line_has(&gate, "Cancel", "Execute", "S2 choice");

    // A1: Esc cancels the dialog back to gate 1.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled");
    waits::wait_state(&mut s, GATE1, "gate 1 back");
    checkpoint(&mut s, &dir, "02-cancelled");
    let cancelled = live_text(&mut s);
    // N1: the gate-2 chrome is gone with the cancel.
    assert!(!cancelled.contains(GATE2), "N1: gate 2 stuck");
    assert!(!cancelled.contains("to confirm"), "N1: phrase prompt stuck");

    // A2: Right+Enter re-opens gate 2, Esc cancels it again.
    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled again");
    let back = live_text(&mut s);
    // N2: nothing stuck open after the second cancel either.
    assert!(!back.contains(GATE2), "N2: gate 2 stuck again");
    assert_line_has(&back, "Review · trash 1 item", GATE1, "gate 1 intact");
    checkpoint(&mut s, &dir, "03-cancelled-again");
    eprintln!("h1 cleanup_gate2: gate 2, cancel roundtrip x2");
}

/// DIALOG-CLEANUP-004 (`holla/flows/cleanup/gate2_typed`): the
/// delete-safety gate-2 dialog with the target-bound phrase typed.
///
/// Flow: boot, run the scan, select the row, d (gate 1), Right+Enter
/// (gate 2), Enter (arm), type the phrase, Esc (text cleared), Esc
/// (dialog cancelled). The phrase is synthetic fixture text; the
/// dialog cancels and never executes.
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cleanup_gate2_typed() {
    const PHRASE: &str = "TRASH 1 UNDER /Users/alex/Projects/safe ON mbp";
    let dir = h1_dir("h1_cleanup_gate2_typed");
    let case = h1_case_t("cleanup_gate2_typed", DELSAFE, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Analyze disk usage",
            "enter",
            "wait:1 unreadable",
            "down",
            "space",
            "d",
            &format!("wait:{GATE1}"),
            "right",
            "enter",
            &format!("wait:{GATE2}"),
            "enter",
            &format!("type:{PHRASE}"),
        ],
        case.timeout_ms,
    );
    waits::wait_state(&mut s, &format!("▎ {PHRASE}"), "phrase typed");
    checkpoint(&mut s, &dir, "01-typed");
    let typed = live_text(&mut s);

    // V1: the armed ack field shows the typed phrase.
    assert!(
        typed.contains(&format!("▎ {PHRASE}")),
        "V1: no typed phrase"
    );
    // V2: the gate-2 dialog title.
    assert!(
        typed.contains("Trash 1 items · gate 2 of 2"),
        "V2: no gate-2 title"
    );
    // S1: the edit-mode footer (also the N1 presence proof).
    assert_line_has(&typed, "EDIT", "Type Phrase", "S1 edit footer");
    // S2: the gate-1 page survives below the dialog.
    assert!(typed.contains(GATE1), "S2: no gate-1 status");

    // A1: the first Esc clears the typed text (the dialog stays open).
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, &format!("▎ {PHRASE}"), "phrase cleared");
    waits::wait_state(&mut s, GATE2, "dialog still open");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);
    // N1: the typed phrase is gone, the prompt (which names the same
    // words) is not the typed line.
    assert!(
        !cleared.contains(&format!("▎ {PHRASE}")),
        "N1: typed phrase stuck"
    );
    assert!(cleared.contains("to confirm"), "N1: prompt lost");

    // A2: the second Esc cancels the dialog back to gate 1.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "dialog cancelled");
    waits::wait_state(&mut s, GATE1, "gate 1 back");
    checkpoint(&mut s, &dir, "03-cancelled");
    let back = live_text(&mut s);
    // N2: the gate-2 chrome is gone with the cancel.
    assert!(!back.contains(GATE2), "N2: gate 2 stuck");
    assert!(!back.contains("to confirm"), "N2: phrase prompt stuck");
    eprintln!("h1 cleanup_gate2_typed: typed, cleared, cancelled");
}

/// PANEL-CLEANUP-005 (`holla/flows/cleanup/history`): the cleanup
/// history page over one prior record.
///
/// Flow: boot, query the history, Esc (finder back, query intact),
/// Enter (history back).
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cleanup_history() {
    let dir = h1_dir("h1_cleanup_history");
    let case = h1_case("cleanup_history", HIST_PAUSED, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Show cleanup history",
            "enter",
            "wait:operation log records",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-history");
    let hist = live_text(&mut s);

    // V1: the history header counts the operation log.
    assert_line_has(&hist, "Entries", "1 operation log records", "V1 header");
    // V2: the prior record row.
    assert_line_has(&hist, "2026-08-29 08:41", "trashed", "V2 record");
    // S1: the append-only log locator.
    assert_line_has(&hist, "Log", "~/.cache/holla/ops.log", "S1 log");
    // S2: the read-only snapshot footer.
    assert_line_has(&hist, "↑↓ Scroll", "Esc Back", "S2 footer");
    // N1: the history page is not the boot state.
    assert!(
        !boot.contains("operation log records"),
        "N1: history visible at boot"
    );

    // A1: Esc leaves the history back to the finder.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "operation log records", "history left");
    checkpoint(&mut s, &dir, "02-finder");
    let left = live_text(&mut s);
    // N2: the history chrome is gone with the leave.
    assert!(!left.contains("operation log records"), "N2: history stuck");
    assert!(!left.contains("Cleanup history"), "N2: title stuck");

    // A2: the intact query re-opens the history with Enter alone.
    support::drive_with_timeout(
        &mut s,
        &["enter", "wait:operation log records"],
        case.timeout_ms,
    );
    waits::wait_state(&mut s, "2026-08-29 08:41", "record back");
    checkpoint(&mut s, &dir, "03-history-again");
    eprintln!("h1 cleanup_history: history, leave, reopen");
}

/// PANEL-CLEANUP-007 (`holla/flows/cleanup/linux_report`): the cleanup
/// report where every item failed — Linux without a Trash backend.
///
/// Flow: boot, open the review, select one candidate, d (gate 1),
/// Right+Enter (gate 2), Enter (arm), type the phrase, Enter, Tab,
/// Enter (commit the simulated trash), Esc (report left). The commit
/// runs only the fixture world's simulated trash; the failure is the
/// simulated missing backend.
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cleanup_linux_report() {
    const PHRASE: &str = "TRASH 1 UNDER /home/alex ON devbox";
    let dir = h1_dir("h1_cleanup_linux_report");
    let case = h1_case_t("cleanup_linux_report", LINUX, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Review cleanup candidates",
            "enter",
            "right",
            "down",
            "space",
            "d",
            "wait:no Trash backend",
            "right",
            "enter",
            &format!("wait:Type {PHRASE}"),
            "enter",
            &format!("type:{PHRASE}"),
            "enter",
            "tab",
            "enter",
            "wait:Cleanup report",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-report");
    let rep = live_text(&mut s);

    // V1: the failed outcome row.
    assert_line_has(&rep, "Outcome", "1 failed", "V1 outcome");
    // V2: the backend failure reason.
    assert!(
        rep.contains("Trash unavailable: no Trash backend on this host"),
        "V2: no backend reason"
    );
    // S1: the failure status (also the N1 presence proof).
    assert!(rep.contains("! 1 failed"), "S1: no failure status");
    // S2: the report revision pins the run.
    assert_line_has(&rep, "Revision", "c4d729e20ca0805b", "S2 revision");
    // N1: the report page is not the boot state.
    assert!(
        !boot.contains("Cleanup report"),
        "N1: report visible at boot"
    );

    // A1: Esc leaves the report.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Cleanup report", "report left");
    checkpoint(&mut s, &dir, "02-left");
    let left = live_text(&mut s);
    // N2: the report page is gone with the leave; the finder is back.
    // (The `! 1 failed` status is sticky by design and outlives the
    // leave, so it is not asserted absent.)
    assert!(!left.contains("Cleanup report"), "N2: report stuck");
    assert!(
        left.contains("› Review cleanup candidates"),
        "N2: finder not back"
    );
    checkpoint(&mut s, &dir, "03-left");
    eprintln!("h1 cleanup_linux_report: failed report, left");
}

/// PANEL-CLEANUP-001 (`holla/flows/cleanup/plan`): the cleanup plan
/// page with its four steps.
///
/// Flow: boot (disk-cleanup plan), c (plan), Down/Up (step roundtrip),
/// c (gate-1 presence proof), Esc (plan back).
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cleanup_plan() {
    let dir = h1_dir("h1_cleanup_plan");
    let case = h1_case("cleanup_plan", PLAN80, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(&mut s, &["c", "wait:4 steps · 4 included"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-plan");
    let plan = live_text(&mut s);

    // V1: the plan header counts.
    assert!(
        plan.contains("4 steps · 4 included · 0 excluded · ~9 s"),
        "V1: no plan header"
    );
    // V2: the cursor on the first step.
    assert_line_has(
        &plan,
        "▎[✓] 01 Trash frontend › node_modules",
        "optional",
        "V2 cursor",
    );
    // S1: the plan footer offers confirm and facts.
    assert_line_has(&plan, "c Confirm", "p Facts", "S1 footer");
    // S2: the selected step is ready.
    assert_line_has(&plan, "State", "ready", "S2 state");
    // N1: the gate review is not open at the plan state.
    assert!(!plan.contains(GATE1), "N1: gate 1 open before c");

    // A1: Down moves the step cursor, Up returns it.
    support::press_step(&s, "down");
    waits::wait_state(&mut s, "▎[✓] 02 cargo clean · backend", "cursor down");
    support::press_step(&s, "up");
    waits::wait_state(
        &mut s,
        "▎[✓] 01 Trash frontend › node_modules",
        "cursor back",
    );
    // A2/N1 presence: c opens gate 1, Esc returns to the plan.
    support::press_step(&s, "c");
    waits::wait_state(&mut s, GATE1, "gate 1 opened");
    checkpoint(&mut s, &dir, "02-gate1-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE1, "gate 1 left");
    waits::wait_state(&mut s, "4 steps · 4 included", "plan back");
    let back = live_text(&mut s);
    // N2: the gate chrome is gone after the leave, the cursor intact.
    assert!(!back.contains(GATE1), "N2: gate 1 stuck");
    assert_line_has(
        &back,
        "▎[✓] 01 Trash frontend › node_modules",
        "optional",
        "cursor intact",
    );
    checkpoint(&mut s, &dir, "03-plan-again");
    eprintln!("h1 cleanup_plan: plan, cursor + gate roundtrips");
}

/// PANEL-CLEANUP-002 (`holla/flows/cleanup/report`): the cleanup report
/// after the simulated trash commits one item.
///
/// Flow: boot, run the scan, select the row, d (gate 1), Right+Enter
/// (gate 2), Enter (arm), type the phrase, Enter, Tab, Enter (commit),
/// Esc (report left). The commit runs only the fixture world's
/// simulated trash.
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cleanup_report() {
    const PHRASE: &str = "TRASH 1 UNDER /Users/alex/Projects/safe ON mbp";
    let dir = h1_dir("h1_cleanup_report");
    let case = h1_case_t("cleanup_report", HIST_REDUCED, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Analyze disk usage",
            "enter",
            "wait:1 unreadable",
            "down",
            "space",
            "d",
            &format!("wait:{GATE1}"),
            "right",
            "enter",
            &format!("wait:{GATE2}"),
            "enter",
            &format!("type:{PHRASE}"),
            "enter",
            "tab",
            "enter",
            "wait:Cleanup report",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-report");
    let rep = live_text(&mut s);

    // V1: the trashed outcome row.
    assert_line_has(&rep, "Outcome", "1 trashed", "V1 outcome");
    // V2: the detail row names the path and the bytes.
    assert_line_has(&rep, "trashed", "~/Projects/safe/big", "V2 detail");
    assert!(rep.contains("5.9 GiB left the tree"), "V2: no bytes fact");
    // S1: the trashed status (also the N1 presence proof).
    assert!(rep.contains("1 trashed"), "S1: no trashed status");
    // S2: the report links the history page.
    assert_line_has(&rep, "Next", "Cleanup history", "S2 next");
    // N1: the report page is not the boot state.
    assert!(
        !boot.contains("Cleanup report"),
        "N1: report visible at boot"
    );

    // A1: Esc leaves the report.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Cleanup report", "report left");
    checkpoint(&mut s, &dir, "02-left");
    let left = live_text(&mut s);
    // N2: the report chrome is gone with the leave.
    assert!(!left.contains("Cleanup report"), "N2: report stuck");
    assert!(
        !left.contains("~/Projects/safe/big"),
        "N2: detail row stuck"
    );
    checkpoint(&mut s, &dir, "03-left");
    eprintln!("h1 cleanup_report: trashed report, left");
}

/// PANEL-DISK-001 (`holla/flows/disk/scan_complete`): the finished disk
/// scan with its unreadable warning.
///
/// Flow: boot, run the scan, Down/Up (cursor roundtrip), s (apparent
/// presence proof), s (allocated back).
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_disk_scan_complete() {
    let dir = h1_dir("h1_disk_scan_complete");
    let case = h1_case_t("disk_scan_complete", SCAN, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["type:Analyze disk usage", "enter", "wait:2 unreadable"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-scan");
    let scan = live_text(&mut s);

    // V1: the completion summary.
    assert!(
        scan.contains("scan complete · 852.0 KiB allocated"),
        "V1: no completion summary"
    );
    // V2: the unreadable warning.
    assert_line_has(&scan, "2 unreadable", "Full Disk Access", "V2 warning");
    // S1: the freshness fact.
    assert_line_has(&scan, "Freshness", "live · complete", "S1 freshness");
    // S2: the empty selection.
    assert!(scan.contains("0 selected · 0 B"), "S2: no empty selection");
    // N1: the apparent sort is not active at the scan state.
    assert!(
        !scan.contains("largest first · apparent"),
        "N1: apparent sort active"
    );

    // A1: Down moves the cursor into the tree, Up returns it. The
    // waits predicate on the cursor row itself: the row names are
    // always on screen, so a plain needle wait would pass vacuously.
    support::press_step(&s, "down");
    support::wait_screen(&mut s, case_timeout(&case), "cursor on many", |screen| {
        support::screen_text(screen)
            .lines()
            .any(|l| l.contains("▎") && l.contains("many"))
    });
    let moved = live_text(&mut s);
    assert_line_has(&moved, "▎", "many", "A1 cursor on many");
    support::press_step(&s, "up");
    waits::wait_state(&mut s, "▎▾   data", "cursor back");
    // A2/N1 presence: s switches to apparent sort, s switches back.
    support::press_step(&s, "s");
    waits::wait_state(&mut s, "largest first · apparent", "apparent on");
    checkpoint(&mut s, &dir, "02-apparent-presence");
    support::press_step(&s, "s");
    waits::wait_state(&mut s, "largest first · allocated", "allocated back");
    let back = live_text(&mut s);
    // N2: the apparent sort is gone after the toggle-back.
    assert!(
        !back.contains("largest first · apparent"),
        "N2: apparent stuck"
    );
    assert!(back.contains("scan complete"), "N2: scan state lost");
    checkpoint(&mut s, &dir, "03-allocated-again");
    eprintln!("h1 disk_scan_complete: scan, cursor + sort roundtrips");
}

/// PANEL-DISK-002 (`holla/flows/disk/top_files`): the Spotlight
/// top-files page over the finished scan.
///
/// Flow: boot, run the scan, t (top files), Esc (tree back), t (top
/// files again).
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_disk_top_files() {
    let dir = h1_dir("h1_disk_top_files");
    let case = h1_case_t("disk_top_files", NAV, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Analyze disk usage",
            "enter",
            "wait:scan complete",
            "t",
            "wait:Top files · Spotlight",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-top");
    let top = live_text(&mut s);

    // V1: the top-files title.
    assert!(top.contains("Top files · Spotlight"), "V1: no title");
    // V2: the first file row.
    assert_line_has(&top, "iso.iso", "700.0 MiB", "V2 first row");
    assert!(top.contains("stat failed"), "V2: no stat-failed row");
    // S1: the file counts.
    assert!(top.contains("3 files · 0 selected"), "S1: no counts");
    // S2: the top-files footer.
    assert_line_has(&top, "y Copy path", "Esc Back", "S2 footer");
    // N2 (order: asserted at state, paired below): the tree sort label
    // is gone while the top page is open.
    assert!(!top.contains("largest first"), "N2: tree sort label stuck");

    // A1: Esc returns to the tree …
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Top files · Spotlight", "top page left");
    waits::wait_state(&mut s, "largest first", "tree back");
    checkpoint(&mut s, &dir, "02-tree");
    let tree = live_text(&mut s);
    // N1: the top page is gone with the leave.
    assert!(
        !tree.contains("Top files · Spotlight"),
        "N1: top page stuck"
    );

    // A2: t re-opens the top page over the same scan.
    support::press_step(&s, "t");
    waits::wait_state(&mut s, "Top files · Spotlight", "top reopened");
    waits::wait_state(&mut s, "3 files · 0 selected", "counts back");
    checkpoint(&mut s, &dir, "03-top-again");
    eprintln!("h1 disk_top_files: top page, tree roundtrip, reopen");
}

/// TREE-DISK-001 (`holla/flows/disk/tree`): the finished disk tree.
///
/// Flow: boot, run the scan, Down/Up (cursor roundtrip), s (apparent
/// presence proof), s (allocated back).
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_disk_tree() {
    let dir = h1_dir("h1_disk_tree");
    let case = h1_case_t("disk_tree", NAV, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["type:Analyze disk usage", "enter", "wait:scan complete"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-tree");
    let tree = live_text(&mut s);

    // V1: the completion summary.
    assert!(
        tree.contains("scan complete · 2.3 MiB allocated · 21 entries"),
        "V1: no completion summary"
    );
    // V2: the root row with its share.
    assert_line_has(&tree, "▾   big", "2.3 MiB · 100%", "V2 root");
    // S1: the sort/noise state.
    assert!(
        tree.contains("largest first · allocated · noise folded"),
        "S1: no sort state"
    );
    // S2: the unselected cursor row.
    assert_line_has(
        &tree,
        "Selection",
        "not selected · Space selects",
        "S2 selection",
    );
    // N1: the apparent sort is not active at the tree state.
    assert!(
        !tree.contains("largest first · apparent"),
        "N1: apparent sort active"
    );

    // A1: Down moves the cursor into the tree, Up returns it. The
    // wait predicates on the cursor row itself: the row names are
    // always on screen, so a plain needle wait would pass vacuously.
    support::press_step(&s, "down");
    support::wait_screen(&mut s, case_timeout(&case), "cursor on target", |screen| {
        support::screen_text(screen)
            .lines()
            .any(|l| l.contains("▎") && l.contains("target"))
    });
    let moved = live_text(&mut s);
    assert_line_has(&moved, "▎", "target", "A1 cursor on target");
    support::press_step(&s, "up");
    waits::wait_state(&mut s, "▎▾   big", "cursor back");
    // A2/N1 presence: s switches to apparent sort, s switches back.
    support::press_step(&s, "s");
    waits::wait_state(&mut s, "largest first · apparent", "apparent on");
    checkpoint(&mut s, &dir, "02-apparent-presence");
    support::press_step(&s, "s");
    waits::wait_state(&mut s, "largest first · allocated", "allocated back");
    let back = live_text(&mut s);
    // N2: the apparent sort is gone after the toggle-back.
    assert!(
        !back.contains("largest first · apparent"),
        "N2: apparent stuck"
    );
    assert!(back.contains("noise folded"), "N2: fold state lost");
    checkpoint(&mut s, &dir, "03-allocated-again");
    eprintln!("h1 disk_tree: tree, cursor + sort roundtrips");
}

/// TREE-DISK-002 (`holla/flows/disk/tree_apparent`): the finished disk
/// tree sorted by apparent size.
///
/// Flow: boot, run the scan, s (apparent), s (allocated presence
/// proof), s (apparent back).
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_disk_tree_apparent() {
    let dir = h1_dir("h1_disk_tree_apparent");
    let case = h1_case_t("disk_tree_apparent", NAV, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Analyze disk usage",
            "enter",
            "wait:scan complete",
            "s",
            "wait:largest first · apparent",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-apparent");
    let app = live_text(&mut s);

    // V1: the apparent sort label.
    assert!(
        app.contains("largest first · apparent · noise folded"),
        "V1: no apparent label"
    );
    // V2: the apparent-first row order (vm.img outranks target).
    assert_line_has(&app, "vm.img", "3.5 MiB", "V2 apparent row");
    // S1: the sort footer explains the rows.
    assert!(
        app.contains("Sorted by apparent size · rows still show allocated bytes"),
        "S1: no sort footer"
    );
    // S2: the apparent total in the detail panel.
    assert_line_has(&app, "Apparent", "5.6 MiB · sparse", "S2 apparent total");

    // A1: s toggles back to allocated …
    support::press_step(&s, "s");
    waits::wait_state(&mut s, "largest first · allocated", "allocated on");
    checkpoint(&mut s, &dir, "02-allocated");
    let alloc = live_text(&mut s);
    // N1: the apparent label is gone with the toggle.
    assert!(
        !alloc.contains("largest first · apparent"),
        "N1: apparent stuck"
    );

    // A2: s toggles forward to apparent again.
    support::press_step(&s, "s");
    waits::wait_state(&mut s, "largest first · apparent", "apparent back");
    let again = live_text(&mut s);
    // N2: the allocated label is gone while apparent is active.
    assert!(
        !again.contains("largest first · allocated"),
        "N2: allocated stuck"
    );
    assert_line_has(&again, "vm.img", "3.5 MiB", "order intact");
    checkpoint(&mut s, &dir, "03-apparent-again");
    eprintln!("h1 disk_tree_apparent: apparent, toggle roundtrip");
}

/// TREE-DISK-004 (`holla/flows/disk/tree_selected`): the finished disk
/// tree with one entry selected.
///
/// Flow: boot, run the scan, s, s (allocated), Down, Down (noise),
/// f, f (fold roundtrip), Space (select), Space (unselect presence
/// proof), Space (select again).
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_disk_tree_selected() {
    let dir = h1_dir("h1_disk_tree_selected");
    let case = h1_case_t("disk_tree_selected", NAV, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Analyze disk usage",
            "enter",
            "wait:scan complete",
            "s",
            "s",
            "down",
            "down",
            "f",
            "f",
            "space",
            "wait:1 selected (176.0",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-selected");
    let sel = live_text(&mut s);

    // V1: the selection summary.
    assert!(
        sel.contains("1 selected (176.0 KiB)"),
        "V1: no selection summary"
    );
    // V2: the checked noise row.
    assert_line_has(&sel, "✓ noise", "176.0 KiB", "V2 checked row");
    // S1: the partial parent marker.
    assert_line_has(&sel, "◐ big", "2.3 MiB · 100%", "S1 partial parent");
    // S2: the selected-row detail fact.
    assert_line_has(&sel, "Selection", "selected · Space unselects", "S2 detail");

    // A1: Space unselects the row …
    support::press_step(&s, "space");
    waits::wait_gone(&mut s, "✓ noise", "row unchecked");
    waits::wait_state(&mut s, "0 selected", "selection empty");
    checkpoint(&mut s, &dir, "02-unselected");
    let unsel = live_text(&mut s);
    // N1: the selection marks are gone with the unselect.
    assert!(!unsel.contains("✓ noise"), "N1: check stuck");
    assert!(!unsel.contains("◐ big"), "N1: partial marker stuck");

    // A2: Space re-selects the same row.
    support::press_step(&s, "space");
    waits::wait_state(&mut s, "1 selected (176.0 KiB)", "selected again");
    let again = live_text(&mut s);
    // N2: the empty-selection state is gone while selected.
    assert!(!again.contains("0 selected"), "N2: empty state stuck");
    assert_line_has(&again, "✓ noise", "176.0 KiB", "check intact");
    checkpoint(&mut s, &dir, "03-selected-again");
    eprintln!("h1 disk_tree_selected: selected, unselect roundtrip, reselect");
}

/// TREE-DISK-003 (`holla/flows/disk/tree_unfolded`): the finished disk
/// tree with noise folders unfolded.
///
/// Flow: boot, run the scan, s, s (allocated), Down, Down (noise),
/// f (unfold), f (fold presence proof), f (unfold again).
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_disk_tree_unfolded() {
    let dir = h1_dir("h1_disk_tree_unfolded");
    let case = h1_case_t("disk_tree_unfolded", NAV, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Analyze disk usage",
            "enter",
            "wait:scan complete",
            "s",
            "s",
            "down",
            "down",
            "f",
            "wait:allocated · unfolded",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-unfolded");
    let unf = live_text(&mut s);

    // V1: the unfolded sort label.
    assert!(
        unf.contains("largest first · allocated · unfolded"),
        "V1: no unfolded label"
    );
    // V2: the cursor on the noise row with its detail path.
    assert_line_has(&unf, "▎", "noise", "V2 cursor");
    assert_line_has(&unf, "Path", "~/work/big/noise", "V2 detail path");
    // S1: the unfold footer.
    assert!(
        unf.contains("Noise folders unfolded"),
        "S1: no unfold footer"
    );
    // S2: the noise entries fact.
    assert_line_has(&unf, "Entries", "15 · hardlinks counted once", "S2 entries");

    // A1: f folds the noise back …
    support::press_step(&s, "f");
    waits::wait_state(&mut s, "noise folded", "folded again");
    checkpoint(&mut s, &dir, "02-folded");
    let folded = live_text(&mut s);
    // N1: the unfolded label is gone with the fold.
    assert!(!folded.contains("unfolded"), "N1: unfolded stuck");
    assert!(
        !folded.contains("Noise folders unfolded"),
        "N1: unfold footer stuck"
    );

    // A2: f unfolds the noise once more.
    support::press_step(&s, "f");
    waits::wait_state(&mut s, "allocated · unfolded", "unfolded again");
    let again = live_text(&mut s);
    // N2: the folded label is gone while unfolded.
    assert!(!again.contains("noise folded"), "N2: folded stuck");
    assert_line_has(&again, "▎", "noise", "cursor intact");
    checkpoint(&mut s, &dir, "03-unfolded-again");
    eprintln!("h1 disk_tree_unfolded: unfolded, fold roundtrip, unfold");
}

/// PANEL-DOCKER-001 (`holla/flows/docker/done`): the finished Docker
/// cleanup run, 7 of 7 steps done.
///
/// Flow: boot, open the plan, both gates, typed phrase, Execute (the
/// drift bounces back to gate 1 — checkpointed), both gates again,
/// Execute (the simulated run completes), Up/Down (step roundtrip).
/// Every command runs only inside the fixture world's simulation.
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_docker_done() {
    const PHRASE: &str = "I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox";
    let dir = h1_dir("h1_docker_done");
    let case = h1_case_t("docker_done", DOCKER_REDUCED, BOOT, 60_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    // First pass: the drift bounces the first Execute back to gate 1.
    support::drive_with_timeout(
        &mut s,
        &[
            "enter",
            "c",
            &format!("wait:{GATE1}"),
            "right",
            "enter",
            &format!("wait:{GATE2}"),
            "enter",
            &format!("type:{PHRASE}"),
            "enter",
            "tab",
            "enter",
            "wait:Plan changed",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-drift");
    // Second pass: the same gates execute and the run completes.
    support::drive_with_timeout(
        &mut s,
        &[
            "right",
            "enter",
            "enter",
            &format!("type:{PHRASE}"),
            "enter",
            "tab",
            "enter",
            "wait:7 succeeded",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-done");
    let done = live_text(&mut s);

    // V1: the finished run header.
    assert!(
        done.contains("Clean Docker completely · 7 of 7 done"),
        "V1: no done header"
    );
    // V2: the reclaimed-bytes fact in the rescan output.
    assert!(
        done.contains("reclaimed 18.4 GB on devbox"),
        "V2: no reclaimed fact"
    );
    // S1: the finished status.
    assert!(done.contains("finished · 7 ok"), "S1: no finished status");
    // S2: the success tail links the follow-up page.
    assert!(done.contains("7 succeeded"), "S2: no success count");
    assert_line_has(&done, "Next", "Docker disk usage", "S2 next");
    // N1: the drift bounce is consumed by the second pass.
    assert!(!done.contains("Plan changed"), "N1: drift bounce stuck");

    // A1: Up moves the step cursor, Down returns it to the last step.
    support::press_step(&s, "up");
    waits::wait_state(&mut s, "▎ ✓ 06 Prune builder", "cursor up");
    support::press_step(&s, "down");
    waits::wait_state(&mut s, "▎ ✓ 07 Rescan and report", "cursor back");
    let back = live_text(&mut s);
    // N2: the gate chrome never returns once the run is done.
    assert!(!back.contains(GATE1), "N2: gate 1 stuck");
    assert!(!back.contains(GATE2), "N2: gate 2 stuck");
    checkpoint(&mut s, &dir, "03-cursor-back");
    eprintln!("h1 docker_done: drift bounce, 7/7 done, cursor roundtrip");
}

/// DIALOG-DOCKER-001 (`holla/flows/docker/drift`): the docker gate-1
/// review after the drift bounce, with its Changed row.
///
/// Flow: boot, open the plan, both gates, typed phrase, Execute (the
/// drift bounces back to gate 1), Right+Enter (gate-2 presence proof),
/// Esc (cancelled), Right+Enter (again), Esc (cancelled again).
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_docker_drift() {
    const PHRASE: &str = "I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox";
    let dir = h1_dir("h1_docker_drift");
    let case = h1_case("docker_drift", DOCKER_PAUSED, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "enter",
            "c",
            &format!("wait:{GATE1}"),
            "right",
            "enter",
            &format!("wait:{GATE2}"),
            "enter",
            &format!("type:{PHRASE}"),
            "enter",
            "tab",
            "enter",
            "wait:a new container",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-drift");
    let drift = live_text(&mut s);

    // V1: the Changed row names the drifted container.
    assert_line_has(&drift, "Changed", "tmp-shell-3", "V1 changed row");
    // V2: the invalidated-confirmation footer.
    assert!(
        drift.contains("! Plan changed · a new container (tmp-shell-3) appeared"),
        "V2: no drift footer"
    );
    // S1: the gate-1 phase survives the bounce.
    assert_line_has(
        &drift,
        "Review · Clean Docker completely",
        GATE1,
        "S1 title",
    );
    // S2: the reclaimable fact.
    assert_line_has(&drift, "Reclaimable", "~18.4 GB", "S2 reclaimable");
    // N1: the gate-2 dialog is closed at the drifted gate 1.
    assert!(!drift.contains(GATE2), "N1: gate 2 open after bounce");

    // A1/N1 presence: Right+Enter re-opens gate 2, Esc cancels it.
    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-gate2-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled");
    waits::wait_state(&mut s, "a new container", "drift intact");

    // A2: the open/cancel roundtrip repeats.
    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled again");
    let back = live_text(&mut s);
    // N2: the gate-2 chrome is gone after the second cancel.
    assert!(!back.contains(GATE2), "N2: gate 2 stuck");
    assert_line_has(&back, "Changed", "tmp-shell-3", "drift intact");
    checkpoint(&mut s, &dir, "03-drift-again");
    eprintln!("h1 docker_drift: drift bounce, gate-2 roundtrip x2");
}

/// DIALOG-DOCKER-002 (`holla/flows/docker/gate2_typed`): the docker
/// gate-2 dialog with the target-bound phrase typed.
///
/// Flow: boot, open the plan, both gates, Enter (arm), type the
/// phrase, Esc (text cleared), Esc (dialog cancelled). The phrase is
/// synthetic fixture text; the dialog cancels and never executes.
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_docker_gate2_typed() {
    const PHRASE: &str = "I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox";
    let dir = h1_dir("h1_docker_gate2_typed");
    let case = h1_case("docker_gate2_typed", DOCKER_PAUSED, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "enter",
            "c",
            &format!("wait:{GATE1}"),
            "right",
            "enter",
            &format!("wait:{GATE2}"),
            "enter",
            &format!("type:{PHRASE}"),
        ],
        case.timeout_ms,
    );
    waits::wait_state(&mut s, &format!("▎ {PHRASE}"), "phrase typed");
    checkpoint(&mut s, &dir, "01-typed");
    let typed = live_text(&mut s);

    // V1: the armed ack field shows the typed phrase.
    assert!(
        typed.contains(&format!("▎ {PHRASE}")),
        "V1: no typed phrase"
    );
    // V2: the gate-2 dialog title.
    assert!(
        typed.contains("Clean Docker completely · gate 2 of 2"),
        "V2: no gate-2 title"
    );
    // S1: the edit-mode footer (also the N1 presence proof).
    assert_line_has(&typed, "EDIT", "Type Phrase", "S1 edit footer");
    // S2: the reclaimable fact inside the dialog.
    assert_line_has(&typed, "Reclaimable", "~18.4 GB", "S2 reclaimable");

    // A1: the first Esc clears the typed text (the dialog stays open).
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, &format!("▎ {PHRASE}"), "phrase cleared");
    waits::wait_state(&mut s, GATE2, "dialog still open");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);
    // N1: the typed phrase is gone, the prompt is not the typed line.
    assert!(
        !cleared.contains(&format!("▎ {PHRASE}")),
        "N1: typed phrase stuck"
    );
    assert!(cleared.contains("to confirm"), "N1: prompt lost");

    // A2: the second Esc cancels the dialog back to gate 1.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "dialog cancelled");
    waits::wait_state(&mut s, GATE1, "gate 1 back");
    checkpoint(&mut s, &dir, "03-cancelled");
    let back = live_text(&mut s);
    // N2: the gate-2 chrome is gone with the cancel.
    assert!(!back.contains(GATE2), "N2: gate 2 stuck");
    assert!(!back.contains("to confirm"), "N2: phrase prompt stuck");
    eprintln!("h1 docker_gate2_typed: typed, cleared, cancelled");
}

/// PANEL-DOCKER-002 (`holla/flows/docker/remove_failed`): the failed
/// stop-all-containers activity tab.
///
/// Flow: boot, run Stop-all (the simulated daemon refuses one
/// container), Esc (Here, query intact), run Stop-all again (the
/// simulated failure repeats deterministically). The failure status
/// is sticky by design, so leave/absence needles are tab-local.
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_docker_remove_failed() {
    const HEADER: &str = "stop all containers · failed · 0 s · exit 1";
    let dir = h1_dir("h1_docker_remove_failed");
    let case = h1_case_t("docker_remove_failed", PDOCKER_RED, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Stop all containers",
            "enter",
            "right",
            "enter",
            &format!("wait:{HEADER}"),
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-failed");
    let failed = live_text(&mut s);

    // V1: the failed activity header.
    assert!(failed.contains(HEADER), "V1: no failed header");
    // V2: the daemon refusal in the output.
    assert!(
        failed.contains("cannot stop container"),
        "V2: no daemon refusal"
    );
    assert!(failed.contains("acme-scheduler-1"), "V2: no container id");
    // S1: the failure status (also the N1 presence proof).
    assert!(
        failed.contains("! stop all containers failed · exit 1"),
        "S1: no failure status"
    );
    // S2: the activity footer offers restart and close.
    assert_line_has(&failed, "r Restart", "x Close", "S2 footer");
    // N1: the failure tab is not the boot state.
    assert!(!boot.contains("exit 1"), "N1: failure visible at boot");

    // A1: Esc leaves the failed tab for Here …
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, HEADER, "failed tab left");
    checkpoint(&mut s, &dir, "02-here");
    let here = live_text(&mut s);
    // N2: the tab-local failure chrome is gone with the leave.
    assert!(!here.contains(HEADER), "N2: failed header stuck");
    assert!(
        !here.contains("cannot stop container"),
        "N2: daemon output stuck"
    );

    // A2: running Stop-all again fails the same simulated way.
    support::drive_with_timeout(
        &mut s,
        &["enter", "right", "enter", &format!("wait:{HEADER}")],
        case.timeout_ms,
    );
    waits::wait_state(&mut s, "cannot stop container", "failed again");
    checkpoint(&mut s, &dir, "03-failed-again");
    eprintln!("h1 docker_remove_failed: failed, left, failed again");
}

/// DIALOG-DOCKER-003 (`holla/flows/docker/remove_gate1`): the
/// stop-and-remove gate-1 review.
///
/// Flow: boot, query the remove-all action, Enter (gate 1),
/// Right+Enter (gate-2 presence proof), Esc (cancelled), Right+Enter
/// (again), Esc (cancelled again).
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_docker_remove_gate1() {
    let dir = h1_dir("h1_docker_remove_gate1");
    let case = h1_case("docker_remove_gate1", PDOCKER_PAUSED, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Stop and remove all",
            "enter",
            &format!("wait:{GATE1}"),
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-gate1");
    let gate = live_text(&mut s);

    // V1: the gate-1 title names the action and the phase.
    assert_line_has(
        &gate,
        "Review · Stop and remove all containers",
        GATE1,
        "V1 title",
    );
    // V2: the destructive changes fact.
    assert_line_has(&gate, "Changes", "6 containers removed", "V2 changes");
    // S1: the risk fact.
    assert_line_has(&gate, "Risk", "destructive", "S1 risk");
    // S2: the two-command sequence.
    assert_line_has(&gate, "Sequence · 2 commands", "y copies", "S2 sequence");
    // N1: the gate-2 dialog is closed at the gate-1 state.
    assert!(!gate.contains(GATE2), "N1: gate 2 open before Right+Enter");

    // A1/N1 presence: Right+Enter opens gate 2, Esc cancels it.
    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-gate2-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled");
    waits::wait_state(&mut s, GATE1, "gate 1 back");

    // A2: the open/cancel roundtrip repeats.
    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled again");
    let back = live_text(&mut s);
    // N2: the gate-2 chrome is gone after the second cancel.
    assert!(!back.contains(GATE2), "N2: gate 2 stuck");
    assert!(!back.contains("to confirm"), "N2: phrase prompt stuck");
    assert_line_has(
        &back,
        "Review · Stop and remove all containers",
        GATE1,
        "gate 1 intact",
    );
    checkpoint(&mut s, &dir, "03-gate1-again");
    eprintln!("h1 docker_remove_gate1: gate 1, gate-2 roundtrip x2");
}

/// DIALOG-DOCKER-004 (`holla/flows/docker/remove_gate2`): the
/// stop-and-remove gate-2 typed-phrase dialog, unarmed.
///
/// Flow: boot, query the remove-all action, Enter (gate 1),
/// Right+Enter (gate 2), Esc (cancelled), Right+Enter (again), Esc
/// (cancelled again). Never armed, never executed.
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_docker_remove_gate2() {
    const PHRASE: &str = "REMOVE ALL CONTAINERS ON mbp";
    let dir = h1_dir("h1_docker_remove_gate2");
    let case = h1_case("docker_remove_gate2", PDOCKER_PAUSED, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Stop and remove all",
            "enter",
            &format!("wait:{GATE1}"),
            "right",
            "enter",
            &format!("wait:{GATE2}"),
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-gate2");
    let gate = live_text(&mut s);

    // V1: the gate-2 dialog title.
    assert!(
        gate.contains("Stop and remove all containers · gate 2 of 2"),
        "V1: no gate-2 title"
    );
    // V2: the target-bound phrase prompt.
    assert!(
        gate.contains(&format!("Type {PHRASE} to confirm")),
        "V2: no phrase prompt"
    );
    // S1: the gate-2 footer (also the N1 presence proof).
    assert_line_has(&gate, "Enter Confirm", "Esc Cancel", "S1 footer");
    // S2: the destructive risk fact inside the dialog.
    assert_line_has(&gate, "Risk", "destructive", "S2 risk");

    // A1: Esc cancels the dialog back to gate 1.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled");
    waits::wait_state(&mut s, GATE1, "gate 1 back");
    checkpoint(&mut s, &dir, "02-cancelled");
    let cancelled = live_text(&mut s);
    // N1: the gate-2 chrome is gone with the cancel.
    assert!(!cancelled.contains(GATE2), "N1: gate 2 stuck");
    assert!(!cancelled.contains("to confirm"), "N1: phrase prompt stuck");

    // A2: Right+Enter re-opens gate 2, Esc cancels it again.
    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled again");
    let back = live_text(&mut s);
    // N2: nothing stuck open after the second cancel either.
    assert!(!back.contains(GATE2), "N2: gate 2 stuck again");
    assert_line_has(
        &back,
        "Review · Stop and remove all containers",
        GATE1,
        "gate 1 intact",
    );
    checkpoint(&mut s, &dir, "03-cancelled-again");
    eprintln!("h1 docker_remove_gate2: gate 2, cancel roundtrip x2");
}

/// PANEL-GRADLE-001 (`holla/flows/gradle/cleanup`): the Gradle cleanup
/// review with its fold opened.
///
/// Flow: boot, query the Gradle cleanup, Right (fold opens), Left
/// (closes), Right (opens again), d (gate-1 presence proof), Esc
/// (review back).
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_gradle_cleanup() {
    let dir = h1_dir("h1_gradle_cleanup");
    let case = h1_case("gradle_cleanup", GRADLE, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Clean Gradle outputs",
            "enter",
            "wait:Cleanup · Gradle outputs",
            "right",
            "wait:▾ Gradle outputs",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-cleanup");
    let gradle = live_text(&mut s);

    // V1: the Gradle cleanup title.
    assert!(
        gradle.contains("Cleanup · Gradle outputs"),
        "V1: no cleanup title"
    );
    // V2: the opened fold with its bytes.
    assert_line_has(&gradle, "▾ Gradle outputs", "49.0 MiB", "V2 fold");
    // S1: the daemon prerequisite.
    assert_line_has(&gradle, "Prerequisite", "gradle --stop", "S1 prerequisite");
    // S2: the full eligibility.
    assert_line_has(
        &gradle,
        "Selection",
        "4 of 4 candidates eligible",
        "S2 selection",
    );
    // N1: the gate review is not open at the cleanup state.
    assert!(!gradle.contains(GATE1), "N1: gate 1 open before d");

    // A1: Left closes the fold, Right re-opens it.
    support::press_step(&s, "left");
    waits::wait_gone(&mut s, "▾ Gradle outputs", "fold closed");
    support::press_step(&s, "right");
    waits::wait_state(&mut s, "▾ Gradle outputs", "fold reopened");
    // A2/N1 presence: d opens gate 1, Esc returns to the review.
    support::press_step(&s, "d");
    waits::wait_state(&mut s, GATE1, "gate 1 opened");
    checkpoint(&mut s, &dir, "02-gate1-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE1, "gate 1 left");
    waits::wait_state(&mut s, "Cleanup · Gradle outputs", "review back");
    let back = live_text(&mut s);
    // N2: the gate chrome is gone after the leave.
    assert!(!back.contains(GATE1), "N2: gate 1 stuck");
    assert!(!back.contains(GATE2), "N2: gate 2 stuck");
    checkpoint(&mut s, &dir, "03-review-again");
    eprintln!("h1 gradle_cleanup: review, fold + gate roundtrips");
}

/// DIALOG-GRADLE-001 (`holla/flows/gradle/gate1`): the Gradle trash
/// gate-1 review.
///
/// Flow: boot, query the Gradle cleanup, Right (fold opens), d
/// (gate 1), Right+Enter (gate-2 presence proof), Esc (cancelled),
/// Right+Enter (again), Esc (cancelled again).
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_gradle_gate1() {
    let dir = h1_dir("h1_gradle_gate1");
    let case = h1_case("gradle_gate1", GRADLE, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Clean Gradle outputs",
            "enter",
            "wait:Cleanup · Gradle outputs",
            "right",
            "wait:▾ Gradle outputs",
            "d",
            &format!("wait:{GATE1}"),
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-gate1");
    let gate = live_text(&mut s);

    // V1: the gate-1 title names the trash count and the phase.
    assert_line_has(&gate, "Review · trash 4 items", GATE1, "V1 title");
    // V2: the daemon prerequisite survives into the review.
    assert_line_has(
        &gate,
        "Prerequisite",
        "gradle --stop first",
        "V2 prerequisite",
    );
    // S1: the four-command sequence.
    assert_line_has(&gate, "Sequence · 4 commands", "y copies", "S1 sequence");
    // S2: the trashed paths.
    assert_line_has(&gate, "Paths", ".gradle", "S2 paths");
    // N1: the gate-2 dialog is closed at the gate-1 state.
    assert!(!gate.contains(GATE2), "N1: gate 2 open before Right+Enter");

    // A1/N1 presence: Right+Enter opens gate 2, Esc cancels it.
    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-gate2-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled");
    waits::wait_state(&mut s, GATE1, "gate 1 back");

    // A2: the open/cancel roundtrip repeats.
    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled again");
    let back = live_text(&mut s);
    // N2: the gate-2 chrome is gone after the second cancel.
    assert!(!back.contains(GATE2), "N2: gate 2 stuck");
    assert!(!back.contains("to confirm"), "N2: phrase prompt stuck");
    assert_line_has(&back, "Review · trash 4 items", GATE1, "gate 1 intact");
    checkpoint(&mut s, &dir, "03-gate1-again");
    eprintln!("h1 gradle_gate1: gate 1, gate-2 roundtrip x2");
}

/// PANEL-IDEA-001 (`holla/flows/idea/cleanup`): the IntelliJ cleanup
/// review with its fold opened.
///
/// Flow: boot, query the IntelliJ cleanup, Right (fold opens), Left
/// (closes), Right (opens again), d (gate-1 presence proof), Esc
/// (review back).
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_idea_cleanup() {
    let dir = h1_dir("h1_idea_cleanup");
    let case = h1_case("idea_cleanup", IDEA, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["type:Clean IntelliJ", "enter", "right", "wait:idea.clean"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-cleanup");
    let idea = live_text(&mut s);

    // V1: the IntelliJ cleanup title.
    assert!(
        idea.contains("Cleanup · IntelliJ metadata"),
        "V1: no cleanup title"
    );
    // V2: the opened fold with its bytes.
    assert_line_has(&idea, "▾ IntelliJ metadata", "44.0 KiB", "V2 fold");
    // S1: the unreadable lower-bound fact (wrapped over three rows).
    assert!(idea.contains("Unreadable"), "S1: no unreadable label");
    assert!(idea.contains("~/work/ide/locked"), "S1: no locked path");
    // S2: the full eligibility.
    assert_line_has(
        &idea,
        "Selection",
        "4 of 4 candidates eligible",
        "S2 selection",
    );
    // N1: the gate review is not open at the cleanup state.
    assert!(!idea.contains(GATE1), "N1: gate 1 open before d");

    // A1: Left closes the fold, Right re-opens it.
    support::press_step(&s, "left");
    waits::wait_gone(&mut s, "▾ IntelliJ metadata", "fold closed");
    support::press_step(&s, "right");
    waits::wait_state(&mut s, "▾ IntelliJ metadata", "fold reopened");
    // A2/N1 presence: d opens gate 1, Esc returns to the review.
    support::press_step(&s, "d");
    waits::wait_state(&mut s, GATE1, "gate 1 opened");
    checkpoint(&mut s, &dir, "02-gate1-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE1, "gate 1 left");
    waits::wait_state(&mut s, "Cleanup · IntelliJ metadata", "review back");
    let back = live_text(&mut s);
    // N2: the gate chrome is gone after the leave.
    assert!(!back.contains(GATE1), "N2: gate 1 stuck");
    assert!(!back.contains(GATE2), "N2: gate 2 stuck");
    checkpoint(&mut s, &dir, "03-review-again");
    eprintln!("h1 idea_cleanup: review, fold + gate roundtrips");
}

/// DIALOG-IDEA-001 (`holla/flows/idea/gate2`): the IntelliJ trash
/// gate-2 typed-phrase dialog, unarmed.
///
/// Flow: boot, query the IntelliJ cleanup, Right (fold opens), d
/// (gate 1), Right+Enter (gate 2), Esc (cancelled), Right+Enter
/// (again), Esc (cancelled again). Never armed, never executed.
#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_idea_gate2() {
    const PHRASE: &str = "TRASH 4 UNDER /Users/alex/work/ide ON mbp";
    let dir = h1_dir("h1_idea_gate2");
    let case = h1_case("idea_gate2", IDEA, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Clean IntelliJ",
            "enter",
            "right",
            "wait:idea.clean",
            "d",
            &format!("wait:{GATE1}"),
            "right",
            "enter",
            &format!("wait:{GATE2}"),
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-gate2");
    let gate = live_text(&mut s);

    // V1: the gate-2 dialog title.
    assert!(
        gate.contains("Trash 4 items · gate 2 of 2"),
        "V1: no gate-2 title"
    );
    // V2: the target-bound phrase prompt.
    assert!(
        gate.contains(&format!("Type {PHRASE} to confirm")),
        "V2: no phrase prompt"
    );
    // S1: the four trashed paths.
    assert_line_has(&gate, "Paths", "~/work/ide/.idea", "S1 paths");
    assert!(gate.contains("mod/mod.iml"), "S1: no fourth path");
    // S2: the gate-2 footer (also the N1 presence proof).
    assert_line_has(&gate, "Enter Confirm", "Esc Cancel", "S2 footer");

    // A1: Esc cancels the dialog back to gate 1.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled");
    waits::wait_state(&mut s, GATE1, "gate 1 back");
    checkpoint(&mut s, &dir, "02-cancelled");
    let cancelled = live_text(&mut s);
    // N1: the gate-2 chrome is gone with the cancel.
    assert!(!cancelled.contains(GATE2), "N1: gate 2 stuck");
    assert!(!cancelled.contains("to confirm"), "N1: phrase prompt stuck");

    // A2: Right+Enter re-opens gate 2, Esc cancels it again.
    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled again");
    let back = live_text(&mut s);
    // N2: nothing stuck open after the second cancel either.
    assert!(!back.contains(GATE2), "N2: gate 2 stuck again");
    assert_line_has(&back, "Review · trash 4 items", GATE1, "gate 1 intact");
    checkpoint(&mut s, &dir, "03-cancelled-again");
    eprintln!("h1 idea_gate2: gate 2, cancel roundtrip x2");
}
