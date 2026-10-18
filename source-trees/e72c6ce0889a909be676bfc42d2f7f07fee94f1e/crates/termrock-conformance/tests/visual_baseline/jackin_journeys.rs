//! Jackin-preview app-journey suite, slice 1 (VB Phase-7e, §15) — impl port.
//!
//! Ported verbatim from VB commit `41ef28cca744998c8af936e758856ecb2d4b48e3`
//! (`tests/harness/tests/visual_baseline/jackin_journeys.rs`), adapting
//! only paths, binary resolution, and harness API:
//!
//! - `Session` is the impl [`support::Session`] wrapper (live samples via
//!   `s.inner.observe_now()`), not `tuiscotti::tui::Session` directly.
//! - The subject is the `jackin-preview` binary built from this impl
//!   worktree's sources, resolved via [`support::try_resolve_bin`]
//!   (name → executed path + sha256); [`write_provenance`] records path,
//!   digest, size, mtime, and argv behind each journey.
//! - `support::drive` (3-arg) becomes [`support::drive_with_timeout`]
//!   (timeout in ms); single steps use [`support::press_step`].
//! - The `--help` enumeration shells the resolved binary path (impl
//!   `JACKIN` is a name, not a `CARGO_BIN_EXE_*` absolute path).
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
//! Journeys never launch a real container, never log into an account, and
//! never perform destructive operations: the save flow cancels its preview
//! dialog (`Not saved · keep editing`), the failure flow only closes the
//! frozen-failure dialog, and every world is the fixture simulation.

use std::path::{Path, PathBuf};

use crate::support::Session;
use crate::support::state_waits as waits;
use crate::support::{self, Case, Color, JACKIN};

const BOOT: &str = "jackin❯";
const BOOT_FAILURE: &str = "Launch failed";
const COLS: u16 = 120;
const ROWS: u16 = 40;

/// Core routes visited in journey 1: argv, checkpoint slug, and the route
/// needles that prove each boot landed in its own fixture world (header
/// plus one content line, both read off the approved `jackin/scenarios/`
/// frames these argvs reproduce).
const ROUTES: [(&[&str], &str, &[&str]); 2] = [
    (
        &[
            "--scenario",
            "returning",
            "--motion",
            "paused",
            "--frame",
            "40",
        ],
        "manager",
        &["Current directory · payments-platform", "Working dir"],
    ),
    (
        &[
            "--scenario",
            "launch-running",
            "--motion",
            "paused",
            "--frame",
            "40",
        ],
        "cockpit",
        &[
            "Launch › payments-platform › the-architect",
            "stage 3 of 11 · Credentials",
        ],
    ),
];

/// Owned-name journey case at the canonical 120x40 truecolor geometry.
fn journey_case(name: &str, args: &'static [&'static str], boot: &'static str) -> Case {
    Case::dynamic(
        format!("journeys/jackin/{name}"),
        JACKIN,
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
        "profile: tuiscotti-default\ntransport: pty\nport_of: 41ef28cca744998c8af936e758856ecb2d4b48e3\nbinary: {}\nsha256: {}\nsize_bytes: {}\nmodified_unix: {modified}\nargv: {}\n",
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

/// Slice 1, journey 1: enumerate the scenario definitions (the binary's
/// own `--help` must list the scenarios this slice boots), then visit the
/// core routes — Manager, Cockpit — one session per scenario, proving
/// each with its header plus a content line.
#[test]
#[ignore = "jackin journey; run with --ignored"]
fn journey_jackin_core_routes() {
    let dir = journey_dir("journey_jackin_core_routes");

    let jackin =
        support::try_resolve_bin(JACKIN).unwrap_or_else(|e| panic!("resolve {JACKIN}: {e}"));
    let help = std::process::Command::new(&jackin.path)
        .arg("--help")
        .output()
        .unwrap_or_else(|e| panic!("jackin-preview --help failed: {e}"));
    assert!(
        help.status.success(),
        "jackin-preview --help exits 0 (got {})",
        help.status
    );
    let text = String::from_utf8_lossy(&help.stdout);
    std::fs::write(dir.join("scenario-help.txt"), text.as_bytes())
        .unwrap_or_else(|e| panic!("write scenario-help.txt: {e}"));
    for scenario in [
        "returning",
        "accounts-mixed",
        "launch-running",
        "launch-failure",
    ] {
        assert!(
            text.contains(scenario),
            "scenario registry lists `{scenario}` ({}/scenario-help.txt)",
            dir.display()
        );
    }
    eprintln!("enumeration: returning, accounts-mixed, launch-running, launch-failure all listed");

    // By-value iteration copies each `&[&str]` out of the const's promoted
    // statics, so `args` already is `&'static [&'static str]`.
    for (i, (args, slug, needles)) in ROUTES.into_iter().enumerate() {
        let case = journey_case(&format!("core_routes_{slug}"), args, BOOT);
        write_provenance_named(&dir, &case, &format!("provenance-{slug}.txt"));
        let mut s = support::spawn_boot(&case);
        for needle in needles.iter() {
            waits::wait_state(&mut s, needle, &format!("route {slug}"));
        }
        checkpoint(&mut s, &dir, &format!("{i:02}-{slug}"));
    }
    eprintln!("journey core_routes: visited {} routes", ROUTES.len());
}

/// Slice 1, journey 2: one editor tab visit. `e` on the Manager opens the
/// workspace editor on General (crumb plus the `Keep awake` body row
/// prove it), then `2` + Enter jumps to the Mounts tab while TABS is
/// focused — proven by the mounts body (`+ Add mount`) and its footer
/// (`r Read-only`), neither of which exists on any other tab.
#[test]
#[ignore = "jackin journey; run with --ignored"]
fn journey_jackin_editor_mounts_tab() {
    const CRUMB: &str = "Workspaces › payments-platform › edit";

    let dir = journey_dir("journey_jackin_editor_mounts_tab");
    let case = journey_case(
        "editor_mounts_tab",
        &[
            "--scenario",
            "returning",
            "--motion",
            "paused",
            "--frame",
            "40",
        ],
        BOOT,
    );
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-manager");

    support::drive_with_timeout(&mut s, &["e", &format!("wait:{CRUMB}")], case.timeout_ms);
    waits::wait_state(&mut s, "Keep awake", "general tab body");
    checkpoint(&mut s, &dir, "01-general");
    support::drive_with_timeout(&mut s, &["2", "enter", "wait:+ Add mount"], case.timeout_ms);
    waits::wait_state(&mut s, "r Read-only", "mounts tab footer");
    waits::wait_state(&mut s, CRUMB, "editor crumb still up");
    checkpoint(&mut s, &dir, "02-mounts");
    eprintln!("journey editor_mounts_tab: general then mounts, both proven");
}

/// Slice 1, journey 3: one settings tab visit. `accounts-mixed` boots
/// straight into the Accounts route, `s` opens global Settings on General
/// (crumb plus the DCO body row prove it), then `3` + Enter jumps to the
/// Environments tab while TABS is focused — proven by the `GH_TOKEN`
/// global row plus the Environments crumb.
#[test]
#[ignore = "jackin journey; run with --ignored"]
fn journey_jackin_settings_env_tab() {
    const CRUMB_ENV: &str = "Settings › global › Environments";

    let dir = journey_dir("journey_jackin_settings_env_tab");
    let case = journey_case(
        "settings_env_tab",
        &[
            "--scenario",
            "accounts-mixed",
            "--motion",
            "paused",
            "--frame",
            "40",
        ],
        BOOT,
    );
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-accounts");

    support::drive_with_timeout(&mut s, &["s", "wait:Settings › global"], case.timeout_ms);
    waits::wait_state(&mut s, "Sign off commits (DCO)", "general tab body");
    checkpoint(&mut s, &dir, "01-general");
    support::drive_with_timeout(&mut s, &["3", "enter", "wait:GH_TOKEN"], case.timeout_ms);
    waits::wait_state(&mut s, CRUMB_ENV, "environments crumb");
    checkpoint(&mut s, &dir, "02-env");
    eprintln!("journey settings_env_tab: general then environments, both proven");
}

/// Slice 1, journey 4: the save-boundary case with assertion-proven
/// states. In the editor, three downs reach the `Keep awake` checkbox
/// (TABS → NAME → WORKDIR → KEEP_AWAKE) and Space toggles it off —
/// proven by the `• 1 change` header badge. Ctrl+S opens the `Save
/// workspace` preview (the deterministic boundary: the +900 ms
/// virtual-time completion job never fires under paused motion), proven
/// by the dialog title plus the `~ keep_awake true → false` diff line.
/// Escape cancels — proven non-vacuously by the dialog's disappearance
/// plus the `Not saved · keep editing` status and the intact `• 1 change`
/// badge. Nothing is written: the preview is cancelled, not confirmed.
#[test]
#[ignore = "jackin journey; run with --ignored"]
fn journey_jackin_editor_save_cancel() {
    const CRUMB: &str = "Workspaces › payments-platform › edit";
    const DIALOG: &str = "Save workspace";

    let dir = journey_dir("journey_jackin_editor_save_cancel");
    let case = journey_case(
        "editor_save_cancel",
        &[
            "--scenario",
            "returning",
            "--motion",
            "paused",
            "--frame",
            "40",
        ],
        BOOT,
    );
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-manager");

    support::drive_with_timeout(&mut s, &["e", &format!("wait:{CRUMB}")], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-editor");
    support::drive_with_timeout(
        &mut s,
        &["down", "down", "down", "space", "wait:• 1 change"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-dirty");
    support::press_step(&s, "ctrl-s");
    waits::wait_state(&mut s, DIALOG, "save preview open");
    waits::wait_state(&mut s, "~ keep_awake true → false", "preview diff line");
    checkpoint(&mut s, &dir, "03-save-preview");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DIALOG, "preview cancelled");
    waits::wait_state(&mut s, "Not saved · keep editing", "cancel status");
    waits::wait_state(&mut s, "• 1 change", "edits intact");
    waits::wait_state(&mut s, CRUMB, "editor still up");
    checkpoint(&mut s, &dir, "04-cancelled");
    eprintln!("journey editor_save_cancel: dirty, preview, cancelled, edits intact");
}

/// Slice 1, journey 5: the failed-launch case with assertion-proven
/// states. `launch-failure` at frame 240 is the frozen FailNetwork
/// failure with the `Launch failed` dialog open — proven by the title
/// plus the `The Construct network could not be attached` summary and the
/// `Check that Docker's jackin-net bridge exists` next step. Escape
/// acknowledges the failure: the cockpit is torn down and the Manager
/// returns with the `Launch failed · … still running in the Construct`
/// status (app.rs `Go::LaunchFailedAck`). The ack is proven
/// non-vacuously by the dialog summary's disappearance (the title text
/// itself survives into the status line, so it proves nothing in either
/// direction) plus the ack status and the Manager footer. No container is
/// touched: the failure and its retries are the fixture simulation's
/// recorded log lines.
#[test]
#[ignore = "jackin journey; run with --ignored"]
fn journey_jackin_launch_failure_dialog() {
    const SUMMARY: &str = "The Construct network could not be attached";

    let dir = journey_dir("journey_jackin_launch_failure_dialog");
    let case = journey_case(
        "launch_failure_dialog",
        &[
            "--scenario",
            "launch-failure",
            "--motion",
            "paused",
            "--frame",
            "240",
        ],
        BOOT_FAILURE,
    );
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    waits::wait_state(&mut s, SUMMARY, "failure summary");
    waits::wait_state(
        &mut s,
        "Check that Docker's jackin-net bridge exists",
        "failure next step",
    );
    checkpoint(&mut s, &dir, "00-failure-dialog");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, SUMMARY, "failure dialog closed");
    waits::wait_state(
        &mut s,
        "still running in the Construct",
        "ack status on manager",
    );
    // The ack selects the failed instance row, so the footer carries the
    // instance hints (no `Enter Launch`); the tree root plus the manager
    // footer prove the route is back.
    waits::wait_state(&mut s, "Current directory", "manager tree back");
    waits::wait_state(&mut s, "Tab Details", "manager footer back");
    checkpoint(&mut s, &dir, "01-acknowledged");
    eprintln!("journey launch_failure_dialog: failure proven, acknowledged, manager back");
}
