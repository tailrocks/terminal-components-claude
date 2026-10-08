//! Holla pending-roots slice 8A-H3 executable checks (VB Phase-8n).
//!
//! One ignored PTY test per H3 registry row (22 rows: the first 22 of the
//! plan's 25-root 8A-H3 slice in sorted root order — `upgrade/confirm`,
//! `upgrade/excluded`, and `upgrade/facts` stay pending for a later
//! tranche), over the real `holla` binary built from this worktree's
//! verified VB sources (`env!("CARGO_BIN_EXE_holla")`, the harness
//! `[[bin]]` compiled from `../../src/bin/holla/main.rs`). Each test
//! drives the row's specified inputs and asserts its visual (V), state
//! (S), action (A), and negative (N) checks in live-PTY executable form:
//!
//! - V: live needles plus same-line coexistence for args pages, snapshot
//!   pages, services rows, executor output, plan steps, picker rows, gate
//!   fact rows, and trust fact rows.
//! - S: live labels (cursor/focus/footer/status rows) plus field focus,
//!   exclusion counts, batch summaries, gate phases, and trust state.
//! - A: behavior probes — the specified gesture runs and the specified
//!   advance (or boundary refusal) is observed; `Changed`/`Consumed`/
//!   `Ignored` outcomes are proven by their screen correlates (moved,
//!   byte-identical), since PTY cannot see the enum.
//! - N: absence assertions, each paired with a presence proof in the same
//!   test (boot checkpoint, mid-flow checkpoint, or pre/post transition)
//!   so no absence passes vacuously.
//!
//! No new snapshots and no new static captures: all 22 roots already have
//! approved frames gated cell-exact by the ported matrices in `holla.rs`
//! (plan §Reconciliation: "rows plus checks, not recaptures"), so every
//! case here uses owned (dynamic) names and stays out of the `snapshots/`
//! inventory. No isolated-component captures: every row's `requires` set
//! (key-injection, glyph-capture) is PTY-level, and every assertion has a
//! live-PTY executable form.
//!
//! Per-test scratch lands under this harness crate's
//! `target/tuiscotti/journeys/<test>/` (gitignored): the row's named
//! checkpoint(s) as text frames, `00-boot.txt` where the row needs a boot
//! comparison, and `provenance.txt` (binary path, size, mtime, argv).
//! Typed input is synthetic and in-memory only (simulation data); no test
//! runs real commands, touches Git state, or performs destructive
//! operations: the executor rows run the fixture world's simulated scripts
//! only, every gate-2 dialog is cancelled, never executed, every typed
//! confirmation phrase is synthetic fixture text, and every world is the
//! fixture simulation.
//!
//! Row → test map (registry id → `h3_*` test):
//!
//! - DIALOG-PLATFORMS-001 → [`h3_platforms_linux_gate1`]
//! - DIALOG-PLATFORMS-002 → [`h3_platforms_linux_gate2`]
//! - DIALOG-REMOTE-001 → [`h3_remote_gate1`]
//! - DIALOG-REMOTE-002 → [`h3_remote_gate2`]
//! - DIALOG-REMOTE-003 → [`h3_remote_gate2_typed`]
//! - DIALOG-TRUST-001 → [`h3_trust_prompt`]
//! - PANEL-ARGS-001 → [`h3_args_snapshot`]
//! - PANEL-ARGS-002 → [`h3_args_clone`]
//! - PANEL-ARGS-003 → [`h3_args_page`]
//! - PANEL-BREW-001 → [`h3_brew_services`]
//! - PANEL-BREW-002 → [`h3_brew_services_stop_failed`]
//! - PANEL-BREW-003 → [`h3_brew_upgrade_batch`]
//! - PANEL-BREW-004 → [`h3_brew_upgrade_plan`]
//! - PANEL-GIT-001 → [`h3_git_batch`]
//! - PANEL-GIT-002 → [`h3_git_merge`]
//! - PANEL-GIT-003 → [`h3_git_pull_blocked`]
//! - PANEL-GIT-004 → [`h3_git_push_rejected`]
//! - PANEL-PLATFORMS-001 → [`h3_platforms_files`]
//! - PANEL-REMOTE-001 → [`h3_remote_query`]
//! - PANEL-TRUST-001 → [`h3_trust_accepted`]
//! - PANEL-TRUST-002 → [`h3_trust_deploy_failed`]
//! - PICKER-GIT-001 → [`h3_git_batch_picker`]

use std::path::{Path, PathBuf};
use std::time::Duration;

use tuiscotti::tui::Session;

use crate::support::state_waits as waits;
use crate::support::{self, Case, Color, HOLLA};

/// Boot needle on every holla route: the brand.
const BOOT: &str = "holla❯";
/// Empty finder query placeholder (boot/cleared states).
const PLACEHOLDER: &str = "Search actions and resources…";
/// Gate-phase labels shared by the two-gate reviews.
const GATE1: &str = "gate 1 of 2";
const GATE2: &str = "gate 2 of 2";

const RUST_PAUSED: &[&str] = &[
    "--scenario",
    "rust-dirty",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const RUST: &[&str] = &["--scenario", "rust-dirty", "--motion", "reduced"];
const BREW: &[&str] = &[
    "--scenario",
    "parity-brew-services",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const BREW_RUN: &[&str] = &["--scenario", "parity-brew-services", "--motion", "reduced"];
const UPGRADERS: &[&str] = &[
    "--scenario",
    "parity-upgrade-managers",
    "--motion",
    "reduced",
];
const GITBATCH: &[&str] = &["--scenario", "parity-git-batch", "--motion", "reduced"];
const GITCURRENT: &[&str] = &[
    "--scenario",
    "parity-git-current",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const GITCURRENT_RUN: &[&str] = &["--scenario", "parity-git-current", "--motion", "reduced"];
const PLATFORMS: &[&str] = &[
    "--scenario",
    "parity-platforms",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const PLATFORMS_LINUX: &[&str] = &[
    "--scenario",
    "parity-platforms-linux",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const REMOTE: &[&str] = &[
    "--scenario",
    "remote-host",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const REMOTE_RUN: &[&str] = &["--scenario", "remote-host", "--motion", "reduced"];
const CHILD: &[&str] = &["--scenario", "monorepo-child", "--motion", "reduced"];
const CUSTOM: &[&str] = &["--scenario", "parity-custom-actions", "--motion", "reduced"];

/// Owned-name H3 case at the canonical 120x40 truecolor geometry.
fn h3_case(slug: &str, args: &'static [&'static str], boot: &'static str) -> Case {
    Case::dynamic(
        format!("journeys/holla/h3/{slug}"),
        HOLLA,
        args,
        120,
        40,
        Color::Truecolor,
        boot,
    )
}

/// [`h3_case`] with an explicit per-step timeout for tick-driven flows.
fn h3_case_t(
    slug: &str,
    args: &'static [&'static str],
    boot: &'static str,
    timeout_ms: u64,
) -> Case {
    h3_case(slug, args, boot).timeout(timeout_ms)
}

/// Unique scratch dir for one H3 test (created, never shared).
fn h3_dir(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/tuiscotti/journeys")
        .join(test);
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("mkdir {}: {e}", dir.display()));
    dir
}

/// Record the binary path + identity: absolute path, byte size, mtime, argv.
/// Written to `provenance.txt`, echoed too (the build-identity record).
fn write_provenance(dir: &Path, case: &Case) {
    let meta = std::fs::metadata(case.bin).unwrap_or_else(|e| panic!("stat {}: {e}", case.bin));
    let modified = meta
        .modified()
        .map(|t| format!("{t:?}"))
        .unwrap_or_else(|_| "unknown".to_string());
    let argv = support::argv_for(case);
    let body = format!(
        "profile: tuiscotti-default\ntransport: pty\nbinary: {}\nsize_bytes: {}\nmodified: {modified}\nargv: {}\n",
        case.bin,
        meta.len(),
        argv.join(" "),
    );
    std::fs::write(dir.join("provenance.txt"), &body)
        .unwrap_or_else(|e| panic!("write {}: {e}", dir.join("provenance.txt").display()));
    eprintln!("h3 provenance: {body}");
}

/// Save the live screen text as `<name>.txt` in the test dir.
fn checkpoint(s: &mut Session, dir: &Path, name: &str) {
    let obs = s
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
        .observe_now()
        .unwrap_or_else(|e| panic!("live sample failed: {e:#}"));
    support::screen_text(&obs.screen)
}

/// Absence wait for a just-pressed key, without the vacuous-absence
/// guard. Every call site proves presence in its pre-keypress sample
/// (a V/S/pre assert or a drive `wait:`), so the guard would only add
/// a race: a descheduled test thread can take its guard snapshot after
/// the app already redrew, failing a correct transition. The predicate
/// form passes at once when the transition already landed, and also
/// drains one-frame lingers (a gate-2 title can vanish a frame before
/// its prompt text).
fn wait_absent(s: &mut Session, needle: &str, what: &str) {
    support::wait_screen(
        s,
        support::DEFAULT_WAIT,
        &format!("{what}: `{needle}` never expired"),
        |screen| !support::screen_text(screen).contains(needle),
    );
}

/// Some line must contain both `needle` and `also` (the same-row
/// coexistence proof: args rows, snapshot rows, dialog rows, titles).
fn assert_line_has(text: &str, needle: &str, also: &str, what: &str) {
    assert!(
        text.lines().any(|l| l.contains(needle) && l.contains(also)),
        "{what}: no line contains both `{needle}` and `{also}`"
    );
}

/// PANEL-ARGS-002 (`holla/flows/args/clone`): the clone-repository
/// Arguments page.
///
/// Flow: boot, type `clone`, Enter (page), Esc (back), Esc (query
/// cleared), type + Enter (page again).
#[test]
#[ignore = "holla h3 check; run with --ignored"]
fn h3_args_clone() {
    const TITLE: &str = "Clone a GitHub repository… · arguments";
    let dir = h3_dir("h3_args_clone");
    let case = h3_case("args_clone", RUST, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // A1: Enter opens the clone Arguments page.
    support::drive(
        &mut s,
        &["type:clone", "enter", &format!("wait:{TITLE}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-page");
    let page = live_text(&mut s);

    // V1: the page titles the clone action with its arguments class.
    assert!(page.contains(TITLE), "V1: no clone title");
    // V2: the rendered command names the owner placeholder and protocol.
    assert_line_has(&page, "gh repo clone", "--protocol ssh", "V2 command");
    // S1: the owner field carries the focused default.
    assert!(page.contains("alex-dev"), "S1: no owner default");
    // S2: the footer offers the run gesture.
    assert!(page.contains("Ctrl+S Run"), "S2: no run footer");
    // N1: the empty-query placeholder is gone on the page.
    assert!(!page.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: Esc backs to the finder, Esc clears the retained query.
    support::press_step(&s, "escape");
    wait_absent(&mut s, TITLE, "page closed");
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-finder");
    let finder = live_text(&mut s);
    // N2: the arguments class is gone after backing out.
    assert!(!finder.contains("· arguments"), "N2: arguments class stuck");
    support::drive(
        &mut s,
        &["type:clone", "enter", &format!("wait:{TITLE}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-page-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "gh repo clone", "--protocol ssh", "page intact");
    eprintln!("h3 args_clone: page, back roundtrip");
}

/// PANEL-ARGS-003 (`holla/flows/args/page`): the find-the-process
/// Arguments page.
///
/// Flow: boot, type `port`, Enter (page), Esc (back), Esc (query
/// cleared), type + Enter (page again).
#[test]
#[ignore = "holla h3 check; run with --ignored"]
fn h3_args_page() {
    const TITLE: &str = "Find the process on a port… · arguments";
    let dir = h3_dir("h3_args_page");
    let case = h3_case("args_page", RUST, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // A1: Enter opens the port Arguments page.
    support::drive(
        &mut s,
        &["type:port", "enter", &format!("wait:{TITLE}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-page");
    let page = live_text(&mut s);

    // V1: the page titles the port action with its arguments class.
    assert!(page.contains(TITLE), "V1: no port title");
    // V2: the rendered command pins the default port.
    assert!(
        page.contains("lsof -nP '-iTCP:5173' -sTCP:LISTEN"),
        "V2: no lsof command"
    );
    // S1: the port field carries the focused default.
    assert!(page.contains("▎ 5173"), "S1: no port default");
    // S2: the footer offers the run gesture.
    assert!(page.contains("Ctrl+S Run"), "S2: no run footer");
    // N1: the empty-query placeholder is gone on the page.
    assert!(!page.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: Esc backs to the finder, Esc clears the retained query.
    support::press_step(&s, "escape");
    wait_absent(&mut s, TITLE, "page closed");
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-finder");
    let finder = live_text(&mut s);
    // N2: the arguments class is gone after backing out.
    assert!(!finder.contains("· arguments"), "N2: arguments class stuck");
    support::drive(
        &mut s,
        &["type:port", "enter", &format!("wait:{TITLE}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-page-again");
    let back = live_text(&mut s);
    assert!(back.contains(TITLE), "page intact");
    eprintln!("h3 args_page: page, back roundtrip");
}

/// PANEL-ARGS-001 (`holla/flows/args`): the port listeners snapshot
/// after running the Arguments page with its defaults.
///
/// Flow: boot, type `port`, Enter (args page), Ctrl+S (snapshot), Esc
/// (finder back), Enter (args again), Ctrl+S (snapshot again).
#[test]
#[ignore = "holla h3 check; run with --ignored"]
fn h3_args_snapshot() {
    const TITLE: &str = "Find the process on a port… · arguments";
    const SNAP: &str = "Port 5173";
    let dir = h3_dir("h3_args_snapshot");
    let case = h3_case("args_snapshot", RUST_PAUSED, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
        &mut s,
        &["type:port", "enter", &format!("wait:{TITLE}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-args");
    let args = live_text(&mut s);
    assert!(args.contains(TITLE), "pre: no args page");

    // A1: Ctrl+S runs the defaults and opens the listeners snapshot.
    support::drive(
        &mut s,
        &["ctrl-s", &format!("wait:{SNAP}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-snapshot");
    let snap = live_text(&mut s);

    // V1: the process line names the listener with its state.
    assert_line_has(&snap, "node (vite)", "sleeping", "V1 process");
    // V2: the listeners row pairs the pid with the bound port.
    assert_line_has(&snap, "*:5173 (LISTEN)", "7731", "V2 listener");
    // S1: the snapshot meta carries the live host age.
    assert_line_has(&snap, SNAP, "live ·", "S1 meta");
    // S2: the status marks the read-only snapshot.
    assert!(
        snap.contains("snapshot · read-only"),
        "S2: no snapshot status"
    );
    // N1: the arguments class is gone on the snapshot.
    assert!(!snap.contains("· arguments"), "N1: args page stuck");

    // A2: Esc backs to the finder (the run consumes the args page,
    // so Back skips it); Enter re-opens the args page over the
    // retained query and Ctrl+S re-runs the snapshot.
    support::press_step(&s, "escape");
    wait_absent(&mut s, SNAP, "snapshot closed");
    waits::wait_state(&mut s, "scope ‹ here ›", "finder back");
    checkpoint(&mut s, &dir, "03-finder");
    let finder = live_text(&mut s);
    // N2: the snapshot title is gone after backing out.
    assert!(!finder.contains(SNAP), "N2: snapshot title stuck");
    support::drive(
        &mut s,
        &[
            "enter",
            &format!("wait:{TITLE}"),
            "ctrl-s",
            &format!("wait:{SNAP}"),
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "04-snapshot-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "*:5173 (LISTEN)", "7731", "snapshot intact");
    eprintln!("h3 args_snapshot: snapshot, back roundtrip");
}

/// PANEL-BREW-001 (`holla/flows/brew/services`): the Homebrew services
/// listing snapshot.
///
/// Flow: boot, type `Homebrew services`, Enter (listing), Esc (back),
/// Esc (query cleared), type + Enter (listing again).
#[test]
#[ignore = "holla h3 check; run with --ignored"]
fn h3_brew_services() {
    const DETAIL: &str = "12 lines · y copies";
    let dir = h3_dir("h3_brew_services");
    let case = h3_case("brew_services", BREW, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // A1: Enter opens the services listing.
    support::drive(
        &mut s,
        &["type:Homebrew services", "enter", "wait:svc02"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-services");
    let list = live_text(&mut s);
    // Pre: the detail panel (the A2 close signal) is open.
    assert!(list.contains(DETAIL), "pre: no detail panel");

    // V1: the errored service row pairs the name with its status.
    assert_line_has(&list, "svc02", "error", "V1 row");
    // V2: the services fact counts the listing.
    assert_line_has(&list, "Services", "11", "V2 count");
    // S1: the page meta carries the live host age.
    assert_line_has(&list, "Homebrew services", "live ·", "S1 meta");
    // S2: the cache-write fact names the degraded host behavior.
    assert_line_has(&list, "Cache write", "fails on this host", "S2 cache");
    // N1: the empty-query placeholder is gone on the listing.
    assert!(!list.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: Esc backs to the finder, Esc clears the retained query.
    support::press_step(&s, "escape");
    wait_absent(&mut s, DETAIL, "listing closed");
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-finder");
    let finder = live_text(&mut s);
    // N2: the detail panel is gone after backing out.
    assert!(!finder.contains(DETAIL), "N2: detail panel stuck");
    support::drive(
        &mut s,
        &["type:Homebrew services", "enter", "wait:svc02"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-services-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "svc02", "error", "listing intact");
    eprintln!("h3 brew_services: listing, back roundtrip");
}

/// PANEL-BREW-002 (`holla/flows/brew/services_stop_failed`): the
/// executor output for the failed service stop.
///
/// Flow: boot, run `Stop svc02` (exit 1), r (restart, fails again),
/// Alt+0 (back to Here). The script is the fixture world's simulation;
/// nothing real executes.
#[test]
#[ignore = "holla h3 check; run with --ignored"]
fn h3_brew_services_stop_failed() {
    const FAILED: &str = "Bootstrap failed";
    let dir = h3_dir("h3_brew_services_stop_failed");
    let case = h3_case("brew_services_stop_failed", BREW_RUN, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // A1: running the stop lands on the launchctl failure. The output
    // line arrives while the run is still `running`; the exit marker is
    // the completion proof, so the drive waits for both.
    support::drive(
        &mut s,
        &[
            "type:Stop svc02",
            "enter",
            "right",
            "enter",
            &format!("wait:{FAILED}"),
            "wait:exit 1",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-failed");
    let failed = live_text(&mut s);

    // V1: the output records the bootstrap failure.
    assert!(
        failed.contains("Bootstrap failed: 5: Input/output error"),
        "V1: no bootstrap line"
    );
    // V2: the title pairs the stop with its failed exit.
    assert_line_has(&failed, "stop svc02 · failed", "exit 1", "V2 title");
    // S1: the output names the started-by command.
    assert!(
        failed.contains("$ brew services stop svc02 · started by Stop svc02"),
        "S1: no started-by line"
    );
    // S2: the footer records the failed stop.
    assert!(
        failed.contains("! stop svc02 failed · exit 1"),
        "S2: no failed footer"
    );
    // N1: the empty-query placeholder is gone on the output.
    assert!(!failed.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: r restarts the script, the stop fails again.
    // press_step (no pacing sleep): the restart clears the output before
    // wait_gone's first sample would otherwise miss the presence proof.
    support::press_step(&s, "r");
    wait_absent(&mut s, FAILED, "output restarted");
    support::drive(
        &mut s,
        &[&format!("wait:{FAILED}"), "wait:exit 1"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-failed-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "stop svc02 · failed", "exit 1", "title intact");
    support::press_step(&s, "alt-0");
    wait_absent(&mut s, FAILED, "back to Here");
    waits::wait_state(&mut s, "scope ‹ here ›", "finder back");
    checkpoint(&mut s, &dir, "03-here");
    let here = live_text(&mut s);
    // N2: the failure line is gone after leaving the executor.
    assert!(!here.contains(FAILED), "N2: failure line stuck");
    eprintln!("h3 brew_services_stop_failed: fail, restart roundtrip");
}

/// PANEL-BREW-003 (`holla/flows/brew/upgrade_batch`): the executor
/// output for the Homebrew batch upgrade.
///
/// Flow: boot, run `Upgrade Homebrew packages` (succeeds), r
/// (restart, succeeds again), Alt+0 (back to Here). The script is the
/// fixture world's simulation; nothing real executes.
#[test]
#[ignore = "holla h3 check; run with --ignored"]
fn h3_brew_upgrade_batch() {
    const OUTDATED: &str = "Outdated Formulae · 2";
    let dir = h3_dir("h3_brew_upgrade_batch");
    let case = h3_case_t("brew_upgrade_batch", UPGRADERS, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // A1: running the upgrade lands on the outdated-formulae report.
    // The report line arrives while the run is still `running`, and a
    // sibling member can finish first, so the drive gates the member
    // title and then the settled batch footer — not the bare exit.
    support::drive(
        &mut s,
        &[
            "type:Upgrade Homebrew packages",
            "enter",
            "right",
            "enter",
            &format!("wait:{OUTDATED}"),
            "wait:brew update · succeeded",
            "wait:4 ok · 1 failed · 0 cancelled",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-batch");
    let batch = live_text(&mut s);

    // V1: the output reports the outdated formulae.
    assert!(
        batch.contains("==> Outdated Formulae · 2"),
        "V1: no outdated line"
    );
    // V2: the title pairs the update with its clean exit.
    assert_line_has(&batch, "brew update · succeeded", "exit 0", "V2 title");
    // S1: the output names the batch upgrade class.
    assert!(
        batch.contains("started by Upgrade Homebrew packages · batch upgrade.brew-packages"),
        "S1: no batch class"
    );
    // S2: the footer summarizes the batch outcome.
    assert!(
        batch.contains("! Upgrade Homebrew packages · 4 ok · 1 failed · 0 cancelled"),
        "S2: no batch footer"
    );
    // N1: the empty-query placeholder is gone on the output.
    assert!(!batch.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: r restarts the script, the report returns.
    // press_step (no pacing sleep): the restart clears the output before
    // wait_gone's first sample would otherwise miss the presence proof.
    support::press_step(&s, "r");
    wait_absent(&mut s, OUTDATED, "output restarted");
    support::drive(
        &mut s,
        &[&format!("wait:{OUTDATED}"), "wait:brew update · succeeded"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-batch-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "brew update · succeeded", "exit 0", "title intact");
    support::press_step(&s, "alt-0");
    wait_absent(&mut s, OUTDATED, "back to Here");
    waits::wait_state(&mut s, "scope ‹ here ›", "finder back");
    checkpoint(&mut s, &dir, "03-here");
    let here = live_text(&mut s);
    // N2: the report line is gone after leaving the executor.
    assert!(!here.contains(OUTDATED), "N2: report line stuck");
    eprintln!("h3 brew_upgrade_batch: batch, restart roundtrip");
}

/// PANEL-BREW-004 (`holla/flows/brew/upgrade_plan`): the Upgrade
/// everything plan review over the finished batch.
///
/// Flow: boot, run the batch, Alt+0, run `Upgrade everything` (plan),
/// Down x2, Space (step excluded), Space (included back).
#[test]
#[ignore = "holla h3 check; run with --ignored"]
fn h3_brew_upgrade_plan() {
    const EXCLUDED_ONE: &str = "1 excluded";
    const EXCLUDED_NONE: &str = "0 excluded";
    let dir = h3_dir("h3_brew_upgrade_plan");
    let case = h3_case_t("brew_upgrade_plan", UPGRADERS, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    // A1: the batch plus the everything query open the plan review.
    support::drive(
        &mut s,
        &[
            "type:Upgrade Homebrew packages",
            "enter",
            "right",
            "enter",
            "wait:Outdated Formulae · 2",
            "alt-0",
            "ctrl-u",
            "type:Upgrade everything",
            "enter",
            "wait:9 steps",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-plan");
    let plan = live_text(&mut s);

    // V1: the plan title pairs the scope with the step count.
    assert_line_has(&plan, "Upgrade everything", "9 included", "V1 title");
    // V2: the plan foot counts required, optional, and parallel lanes.
    assert!(
        plan.contains("7 required · 2 optional · 4 branches run in parallel where independent"),
        "V2: no plan foot"
    );
    // S1: every step starts included.
    assert!(plan.contains(EXCLUDED_NONE), "S1: no included count");
    // S2: the dialog offers the confirm choice.
    assert_line_has(&plan, "Cancel", "Confirm plan", "S2 choice");
    // N1 (paired below): the none-excluded count is present here and
    // gone once a step is excluded.

    // A2: Space on the optional step 03 excludes it, Space includes
    // it back. (Required steps refuse exclusion with a status line, so
    // the roundtrip targets an optional step.)
    support::drive(
        &mut s,
        &["down", "down", "space", &format!("wait:{EXCLUDED_ONE}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-excluded");
    let excluded = live_text(&mut s);
    assert!(excluded.contains(EXCLUDED_ONE), "excluded: no 1 excluded");
    // N1: the none-excluded count is gone with the exclusion.
    assert!(
        !excluded.contains(EXCLUDED_NONE),
        "N1: none-excluded count stuck"
    );
    support::drive(
        &mut s,
        &["space", &format!("wait:{EXCLUDED_NONE}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-included");
    let back = live_text(&mut s);
    // N2: the excluded count is gone after including the step back.
    assert!(!back.contains(EXCLUDED_ONE), "N2: excluded count stuck");
    assert_line_has(&back, "Upgrade everything", "9 included", "plan intact");
    eprintln!("h3 brew_upgrade_plan: plan, exclude roundtrip");
}

/// PANEL-GIT-001 (`holla/flows/git/batch`): the executor output for
/// the sibling batch pull.
///
/// Flow: boot, run `Pull 4 sibling repositories` (2 ok, 2 failed), r
/// (restart, succeeds again), Alt+0 (back to Here). The script is the
/// fixture world's simulation; nothing real executes.
#[test]
#[ignore = "holla h3 check; run with --ignored"]
fn h3_git_batch() {
    const FF: &str = "Fast-forward";
    let dir = h3_dir("h3_git_batch");
    let case = h3_case_t("git_batch", GITBATCH, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // A1: running the batch lands on the fast-forward report. The
    // report line arrives while the run is still `running`, and a
    // sibling member can finish first, so the drive gates the member
    // title and then the settled batch footer — not the bare exit.
    support::drive(
        &mut s,
        &[
            "type:Pull 4 sibling",
            "enter",
            "right",
            "enter",
            &format!("wait:{FF}"),
            "wait:pull alpha · succeeded",
            "wait:2 ok · 2 failed · 0 cancelled",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-batch");
    let batch = live_text(&mut s);

    // V1: the output reports the fast-forward.
    assert!(batch.contains(FF), "V1: no fast-forward line");
    // V2: the title pairs the pull with its clean exit.
    assert_line_has(&batch, "pull alpha · succeeded", "exit 0", "V2 title");
    // S1: the output names the batch pull class.
    assert!(
        batch.contains("started by Pull 4 sibling repositories · batch git.pull-all"),
        "S1: no batch class"
    );
    // S2: the footer summarizes the batch outcome.
    assert!(
        batch.contains("! Pull 4 sibling repositories · 2 ok · 2 failed · 0 cancelled"),
        "S2: no batch footer"
    );
    // N1: the empty-query placeholder is gone on the output.
    assert!(!batch.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: r restarts the script, the report returns.
    // press_step (no pacing sleep): the restart clears the output before
    // wait_gone's first sample would otherwise miss the presence proof.
    support::press_step(&s, "r");
    wait_absent(&mut s, FF, "output restarted");
    support::drive(
        &mut s,
        &[&format!("wait:{FF}"), "wait:pull alpha · succeeded"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-batch-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "pull alpha · succeeded", "exit 0", "title intact");
    support::press_step(&s, "alt-0");
    wait_absent(&mut s, FF, "back to Here");
    waits::wait_state(&mut s, "scope ‹ here ›", "finder back");
    checkpoint(&mut s, &dir, "03-here");
    let here = live_text(&mut s);
    // N2: the report line is gone after leaving the executor.
    assert!(!here.contains(FF), "N2: report line stuck");
    eprintln!("h3 git_batch: batch, restart roundtrip");
}

/// PICKER-GIT-001 (`holla/flows/git/batch_picker`): the Activities
/// picker over the finished sibling batch.
///
/// Flow: boot, run the batch, Ctrl+G (picker), Esc (cancelled),
/// Ctrl+G (picker again). The output footer also lives behind the
/// picker, so the footer swap (`Enter Switch` vs `f Follow`) is the
/// open/close signal, not the title.
#[test]
#[ignore = "holla h3 check; run with --ignored"]
fn h3_git_batch_picker() {
    let dir = h3_dir("h3_git_batch_picker");
    let case = h3_case_t("git_batch_picker", GITBATCH, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
        &mut s,
        &[
            "type:Pull 4 sibling",
            "enter",
            "right",
            "enter",
            "wait:Fast-forward",
            "wait:2 ok · 2 failed",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-output");
    let output = live_text(&mut s);
    assert!(output.contains("f Follow"), "pre: no output footer");

    // A1: Ctrl+G opens the Activities picker over the batch.
    support::drive(&mut s, &["ctrl-g", "wait:0 live"], case_timeout(&case));
    checkpoint(&mut s, &dir, "02-picker");
    let picker = live_text(&mut s);

    // V1: the picker titles the settled activities.
    assert_line_has(&picker, "Activities", "0 live", "V1 title");
    // V2: the top row pairs the pull with its outcome.
    assert_line_has(&picker, "pull alpha", "succeeded · child", "V2 row");
    // S1: the picker footer offers the switch gesture.
    assert!(picker.contains("Enter Switch"), "S1: no switch footer");
    // S2: the failed sibling stays listed with its outcome.
    assert_line_has(&picker, "pull delta", "failed", "S2 failed row");
    // N1: the output footer is gone under the picker footer.
    assert!(!picker.contains("f Follow"), "N1: output footer stuck");

    // A2: Esc cancels the picker back to the output.
    support::press_step(&s, "escape");
    wait_absent(&mut s, "Enter Switch", "picker cancelled");
    waits::wait_state(&mut s, "f Follow", "output footer back");
    checkpoint(&mut s, &dir, "03-cancelled");
    let cancelled = live_text(&mut s);
    // N2: the picker footer is gone after the cancel.
    assert!(!cancelled.contains("Enter Switch"), "N2: picker stuck");
    support::drive(&mut s, &["ctrl-g", "wait:0 live"], case_timeout(&case));
    checkpoint(&mut s, &dir, "04-picker-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "Activities", "0 live", "picker intact");
    eprintln!("h3 git_batch_picker: picker, cancel roundtrip");
}

/// PANEL-GIT-002 (`holla/flows/git/merge`): the executor output for
/// the blocked merge pull.
///
/// Flow: boot, run `Pull with merge` (exit 1), r (restart, fails
/// again), Alt+0 (back to Here). The script is the fixture world's
/// simulation; nothing real executes.
#[test]
#[ignore = "holla h3 check; run with --ignored"]
fn h3_git_merge() {
    const BLOCKED: &str = "Your local changes";
    let dir = h3_dir("h3_git_merge");
    let case = h3_case("git_merge", GITCURRENT_RUN, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // A1: running the merge lands on the local-changes error. The
    // error line arrives while the run is still `running`; the exit
    // marker is the completion proof, so the drive waits for both.
    support::drive(
        &mut s,
        &[
            "type:Pull with merge",
            "enter",
            "right",
            "enter",
            "wait:error: Your local changes",
            "wait:exit 1",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-merge");
    let merge = live_text(&mut s);

    // V1: the output records the overwrite refusal.
    assert!(
        merge.contains(
            "error: Your local changes to the following files would be overwritten by merge:"
        ),
        "V1: no refusal line"
    );
    // V2: the title pairs the merge with its failed exit.
    assert_line_has(&merge, "pull with merge · failed", "exit 1", "V2 title");
    // S1: the output advises the commit-or-stash recovery.
    assert!(
        merge.contains("Please commit your changes or stash them before you merge."),
        "S1: no recovery line"
    );
    // S2: the footer records the failed merge.
    assert!(
        merge.contains("! pull with merge failed · exit 1"),
        "S2: no failed footer"
    );
    // N1: the empty-query placeholder is gone on the output.
    assert!(!merge.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: r restarts the script, the merge fails again.
    // press_step (no pacing sleep): the restart clears the output before
    // wait_gone's first sample would otherwise miss the presence proof.
    support::press_step(&s, "r");
    wait_absent(&mut s, BLOCKED, "output restarted");
    support::drive(
        &mut s,
        &[&format!("wait:{BLOCKED}"), "wait:exit 1"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-merge-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "pull with merge · failed", "exit 1", "title intact");
    support::press_step(&s, "alt-0");
    wait_absent(&mut s, BLOCKED, "back to Here");
    waits::wait_state(&mut s, "scope ‹ here ›", "finder back");
    checkpoint(&mut s, &dir, "03-here");
    let here = live_text(&mut s);
    // N2: the refusal line is gone after leaving the executor.
    assert!(!here.contains(BLOCKED), "N2: refusal line stuck");
    eprintln!("h3 git_merge: merge, restart roundtrip");
}

/// PANEL-GIT-003 (`holla/flows/git/pull_blocked`): the finder query
/// for the blocked pull.
///
/// Flow: boot, type `Pull`, Esc (cleared), type `Pull` again.
#[test]
#[ignore = "holla h3 check; run with --ignored"]
fn h3_git_pull_blocked() {
    let dir = h3_dir("h3_git_pull_blocked");
    let case = h3_case("git_pull_blocked", GITCURRENT, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // A1: typing filters to the pull actions.
    support::drive(
        &mut s,
        &["type:Pull", "wait:blocked · 1 modified"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    // V1: the top row pairs the pull with its blocked reason.
    assert_line_has(&res, "› Pull", "blocked · 1 modified", "V1 row");
    // V2: the result count.
    assert!(res.contains("Results · 3"), "V2: no Results · 3");
    // S1: the preview explains the block.
    assert_line_has(&res, "Why", "blocked · 1 modified", "S1 why");
    // S2: the scope readout stays here.
    assert!(res.contains("scope ‹ here ›"), "S2: no here scope");
    // N1: the empty-query placeholder is gone with the results.
    assert!(!res.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: Esc clears the query back to the placeholder.
    support::press_step(&s, "escape");
    wait_absent(&mut s, "Results · 3", "results cleared");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);
    // N2: the result count is gone after the clear.
    assert!(!cleared.contains("Results · 3"), "N2: results stuck");
    support::drive(
        &mut s,
        &["type:Pull", "wait:blocked · 1 modified"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-results-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "› Pull", "blocked · 1 modified", "row intact");
    eprintln!("h3 git_pull_blocked: results, clear roundtrip");
}

/// PANEL-GIT-004 (`holla/flows/git/push_rejected`): the executor
/// output for the rejected push, over the failed merge tab.
///
/// Flow: boot, fail `Pull with merge`, Alt+0, run `Push` (rejected),
/// r (restart, rejected again), Alt+0 (back to Here). The scripts are
/// the fixture world's simulation; nothing real executes.
#[test]
#[ignore = "holla h3 check; run with --ignored"]
fn h3_git_push_rejected() {
    const REJECTED: &str = "rejected";
    let dir = h3_dir("h3_git_push_rejected");
    let case = h3_case("git_push_rejected", GITCURRENT_RUN, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    support::drive(
        &mut s,
        &[
            "type:Pull with merge",
            "enter",
            "right",
            "enter",
            "wait:error: Your local changes",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-merge");
    let merge = live_text(&mut s);
    assert!(
        merge.contains("error: Your local changes"),
        "mid: no merge error"
    );

    // A1: the push lands on the fetch-first rejection. The rejection
    // line arrives while the run is still `running`; the push footer is
    // the completion proof (the merge tab already shows `exit 1`, so the
    // bare exit marker cannot gate the push).
    support::drive(
        &mut s,
        &[
            "alt-0",
            "ctrl-u",
            "type:Push",
            "enter",
            "right",
            "enter",
            &format!("wait:{REJECTED}"),
            "wait:! push failed · exit 1",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-push");
    let push = live_text(&mut s);

    // V1: the output records the rejected ref with its reason.
    assert_line_has(&push, "[rejected]", "fetch first", "V1 rejected");
    // V2: the title pairs the push with its failed exit.
    assert_line_has(&push, "push · failed", "exit 1", "V2 title");
    // S1: the output reports the failed ref update.
    assert!(
        push.contains("error: failed to push some refs to 'origin'"),
        "S1: no refs error"
    );
    // S2: the footer records the failed push.
    assert!(
        push.contains("! push failed · exit 1"),
        "S2: no push footer"
    );
    // N1: the empty-query placeholder is gone on the output.
    assert!(!push.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: r restarts the script, the push is rejected again.
    // press_step (no pacing sleep): the restart clears the output before
    // wait_gone's first sample would otherwise miss the presence proof.
    support::press_step(&s, "r");
    wait_absent(&mut s, REJECTED, "output restarted");
    support::drive(
        &mut s,
        &[&format!("wait:{REJECTED}"), "wait:! push failed · exit 1"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-push-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "push · failed", "exit 1", "title intact");
    support::press_step(&s, "alt-0");
    wait_absent(&mut s, REJECTED, "back to Here");
    waits::wait_state(&mut s, "scope ‹ here ›", "finder back");
    checkpoint(&mut s, &dir, "04-here");
    let here = live_text(&mut s);
    // N2: the rejection is gone after leaving the executor.
    assert!(!here.contains(REJECTED), "N2: rejection stuck");
    eprintln!("h3 git_push_rejected: push, restart roundtrip");
}

/// PANEL-PLATFORMS-001 (`holla/flows/platforms/files`): the Top
/// files page over the timed-out Spotlight query.
///
/// Flow: boot, type `Top files on this Mac`, Enter (page), Esc
/// (back), Esc (query cleared), type + Enter (page again). The page
/// title also lives in the finder results, so the footer swap
/// (`Esc Back` vs `Esc Clear`) is the open/close signal, not the
/// title.
#[test]
#[ignore = "holla h3 check; run with --ignored"]
fn h3_platforms_files() {
    const TIMEOUT: &str = "did not finish within 5 s";
    let dir = h3_dir("h3_platforms_files");
    let case = h3_case("platforms_files", PLATFORMS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // A1: Enter opens the Top files page. The body renders a frame
    // before the status bar, so the drive gates the status line too.
    support::drive(
        &mut s,
        &[
            "type:Top files on this Mac",
            "enter",
            &format!("wait:{TIMEOUT}"),
            "wait:0 files · 0 selected",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-files");
    let files = live_text(&mut s);

    // V1: the page titles the Spotlight view.
    assert!(files.contains("Top files · Spotlight"), "V1: no page title");
    // V2: the page reports the query timeout with its fallback.
    assert!(
        files.contains(
            "unavailable · the Spotlight query did not finish within 5 s · use the tree scan"
        ),
        "V2: no timeout line"
    );
    // S1: the status counts the empty selection.
    assert!(
        files.contains("0 files · 0 selected"),
        "S1: no empty status"
    );
    // S2: the footer offers the back gesture.
    assert!(files.contains("Esc Back"), "S2: no back footer");
    // N1: the empty-query placeholder is gone on the page.
    assert!(!files.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: Esc backs to the finder, Esc clears the retained query.
    support::press_step(&s, "escape");
    wait_absent(&mut s, "Esc Back", "page closed");
    waits::wait_state(&mut s, "Esc Clear", "finder back");
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-finder");
    let finder = live_text(&mut s);
    // N2: the timeout report is gone after backing out.
    assert!(!finder.contains(TIMEOUT), "N2: timeout line stuck");
    support::drive(
        &mut s,
        &[
            "type:Top files on this Mac",
            "enter",
            &format!("wait:{TIMEOUT}"),
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-files-again");
    let back = live_text(&mut s);
    assert!(back.contains("Top files · Spotlight"), "page intact");
    eprintln!("h3 platforms_files: page, back roundtrip");
}

/// DIALOG-PLATFORMS-001 (`holla/flows/platforms/linux_gate1`): the
/// Linux trash gate-1 review without a Trash backend.
///
/// Flow: boot, open the review, select the row, d (gate 1),
/// Right+Enter (gate 2 opens), Esc (cancelled), Right+Enter (again),
/// Esc (cancelled again). Never armed, never executed.
#[test]
#[ignore = "holla h3 check; run with --ignored"]
fn h3_platforms_linux_gate1() {
    let dir = h3_dir("h3_platforms_linux_gate1");
    let case = h3_case("platforms_linux_gate1", PLATFORMS_LINUX, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
        &mut s,
        &[
            "type:Review cleanup candidates",
            "enter",
            "right",
            "down",
            "space",
            "d",
            "wait:no Trash backend",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-gate1");
    let gate = live_text(&mut s);

    // V1: the gate-1 title names the trash target and the phase.
    assert_line_has(&gate, "Review · trash 1 item", GATE1, "V1 title");
    // V2: the mode fact names the missing backend.
    assert_line_has(&gate, "Mode", "no Trash backend", "V2 mode");
    // S1: the recovery fact.
    assert_line_has(
        &gate,
        "Recoverable",
        "yes · until the Trash is emptied",
        "S1 recovery",
    );
    // S2: the selection size survives into the review.
    assert!(
        gate.contains("1 item selected · 124.0 KiB"),
        "S2: no selection size"
    );
    // N1: the gate-2 dialog is closed at the gate-1 state.
    assert!(!gate.contains(GATE2), "N1: gate 2 open before Right+Enter");

    // A1/N1 presence: Right+Enter opens gate 2, Esc cancels it.
    support::drive(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-gate2-presence");
    support::press_step(&s, "escape");
    wait_absent(&mut s, GATE2, "gate 2 cancelled");
    wait_absent(&mut s, "to confirm", "prompt fully closed");
    waits::wait_state(&mut s, GATE1, "gate 1 back");

    // A2: the open/cancel roundtrip repeats.
    support::drive(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case_timeout(&case),
    );
    support::press_step(&s, "escape");
    wait_absent(&mut s, GATE2, "gate 2 cancelled again");
    wait_absent(&mut s, "to confirm", "prompt fully closed");
    let back = live_text(&mut s);
    // N2: the gate-2 chrome is gone after the second cancel.
    assert!(!back.contains(GATE2), "N2: gate 2 stuck");
    assert!(!back.contains("to confirm"), "N2: phrase prompt stuck");
    assert_line_has(&back, "Review · trash 1 item", GATE1, "gate 1 intact");
    checkpoint(&mut s, &dir, "03-gate1-again");
    eprintln!("h3 platforms_linux_gate1: gate 1, gate-2 roundtrip x2");
}

/// DIALOG-PLATFORMS-002 (`holla/flows/platforms/linux_gate2`): the
/// Linux trash gate-2 typed-phrase dialog, unarmed.
///
/// Flow: boot, open the review, select the row, d (gate 1),
/// Right+Enter (gate 2), Esc (cancelled), Right+Enter (again), Esc
/// (cancelled again). Never armed, never executed.
#[test]
#[ignore = "holla h3 check; run with --ignored"]
fn h3_platforms_linux_gate2() {
    const PHRASE: &str = "Type TRASH 1 UNDER /home/alex ON devbox to confirm";
    let dir = h3_dir("h3_platforms_linux_gate2");
    let case = h3_case("platforms_linux_gate2", PLATFORMS_LINUX, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
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
            &format!("wait:{GATE2}"),
        ],
        case_timeout(&case),
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
    wait_absent(&mut s, GATE2, "gate 2 cancelled");
    wait_absent(&mut s, "to confirm", "prompt fully closed");
    waits::wait_state(&mut s, GATE1, "gate 1 back");
    checkpoint(&mut s, &dir, "02-cancelled");
    let cancelled = live_text(&mut s);
    // N1: the gate-2 chrome is gone with the cancel.
    assert!(!cancelled.contains(GATE2), "N1: gate 2 stuck");
    assert!(!cancelled.contains("to confirm"), "N1: phrase prompt stuck");

    // A2: Right+Enter re-opens gate 2, Esc cancels it again.
    support::drive(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case_timeout(&case),
    );
    support::press_step(&s, "escape");
    wait_absent(&mut s, GATE2, "gate 2 cancelled again");
    wait_absent(&mut s, "to confirm", "prompt fully closed");
    let back = live_text(&mut s);
    // N2: nothing stuck open after the second cancel either.
    assert!(!back.contains(GATE2), "N2: gate 2 stuck again");
    assert_line_has(&back, "Review · trash 1 item", GATE1, "gate 1 intact");
    checkpoint(&mut s, &dir, "03-cancelled-again");
    eprintln!("h3 platforms_linux_gate2: gate 2, cancel roundtrip x2");
}

/// DIALOG-REMOTE-001 (`holla/flows/remote/gate-1`): the remote
/// restart gate-1 review.
///
/// Flow: boot, type `restart payments`, Enter (gate 1),
/// Right+Enter (gate 2 opens), Esc (cancelled), Right+Enter (again),
/// Esc (cancelled again). Never armed, never executed.
#[test]
#[ignore = "holla h3 check; run with --ignored"]
fn h3_remote_gate1() {
    let dir = h3_dir("h3_remote_gate1");
    let case = h3_case("remote_gate1", REMOTE_RUN, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
        &mut s,
        &["type:restart payments", "enter", &format!("wait:{GATE1}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-gate1");
    let gate = live_text(&mut s);

    // V1: the gate-1 title names the restart and the phase.
    assert_line_has(&gate, "Review · Restart payments", GATE1, "V1 title");
    // V2: the action fact names the remote restart.
    assert_line_has(
        &gate,
        "Action",
        "systemctl restart payments on prod-eu-1",
        "V2 action",
    );
    // S1: the gate fact names the second-gate contract.
    assert_line_has(
        &gate,
        "Gate",
        "second gate types the target-bound phrase",
        "S1 gate",
    );
    // S2: the status holds the gate-1 phase.
    assert!(
        gate.contains("review · gate 1 of 2"),
        "S2: no gate-1 status"
    );
    // N1: the gate-2 dialog is closed at the gate-1 state.
    assert!(!gate.contains(GATE2), "N1: gate 2 open before Right+Enter");

    // A1/N1 presence: Right+Enter opens gate 2, Esc cancels it.
    support::drive(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-gate2-presence");
    support::press_step(&s, "escape");
    wait_absent(&mut s, GATE2, "gate 2 cancelled");
    wait_absent(&mut s, "to confirm", "prompt fully closed");
    waits::wait_state(&mut s, GATE1, "gate 1 back");

    // A2: the open/cancel roundtrip repeats.
    support::drive(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case_timeout(&case),
    );
    support::press_step(&s, "escape");
    wait_absent(&mut s, GATE2, "gate 2 cancelled again");
    wait_absent(&mut s, "to confirm", "prompt fully closed");
    let back = live_text(&mut s);
    // N2: the gate-2 chrome is gone after the second cancel.
    assert!(!back.contains(GATE2), "N2: gate 2 stuck");
    assert!(!back.contains("to confirm"), "N2: phrase prompt stuck");
    assert_line_has(&back, "Review · Restart payments", GATE1, "gate 1 intact");
    checkpoint(&mut s, &dir, "03-gate1-again");
    eprintln!("h3 remote_gate1: gate 1, gate-2 roundtrip x2");
}

/// DIALOG-REMOTE-002 (`holla/flows/remote/gate2`): the remote
/// restart gate-2 typed-phrase dialog, unarmed.
///
/// Flow: boot, type `restart payments`, Enter (gate 1), Right+Enter
/// (gate 2), Esc (cancelled), Right+Enter (again), Esc (cancelled
/// again). Never armed, never executed.
#[test]
#[ignore = "holla h3 check; run with --ignored"]
fn h3_remote_gate2() {
    const PHRASE: &str = "Type RESTART PAYMENTS ON prod-eu-1 to confirm";
    let dir = h3_dir("h3_remote_gate2");
    let case = h3_case("remote_gate2", REMOTE, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
        &mut s,
        &[
            "type:restart payments",
            "enter",
            &format!("wait:{GATE1}"),
            "right",
            "enter",
            &format!("wait:{GATE2}"),
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-gate2");
    let gate = live_text(&mut s);

    // V1: the gate-2 dialog title.
    assert!(
        gate.contains("Restart payments · gate 2 of 2"),
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
    wait_absent(&mut s, GATE2, "gate 2 cancelled");
    wait_absent(&mut s, "to confirm", "prompt fully closed");
    waits::wait_state(&mut s, GATE1, "gate 1 back");
    checkpoint(&mut s, &dir, "02-cancelled");
    let cancelled = live_text(&mut s);
    // N1: the gate-2 chrome is gone with the cancel.
    assert!(!cancelled.contains(GATE2), "N1: gate 2 stuck");
    assert!(!cancelled.contains("to confirm"), "N1: phrase prompt stuck");

    // A2: Right+Enter re-opens gate 2, Esc cancels it again.
    support::drive(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case_timeout(&case),
    );
    support::press_step(&s, "escape");
    wait_absent(&mut s, GATE2, "gate 2 cancelled again");
    wait_absent(&mut s, "to confirm", "prompt fully closed");
    let back = live_text(&mut s);
    // N2: nothing stuck open after the second cancel either.
    assert!(!back.contains(GATE2), "N2: gate 2 stuck again");
    assert_line_has(&back, "Review · Restart payments", GATE1, "gate 1 intact");
    checkpoint(&mut s, &dir, "03-cancelled-again");
    eprintln!("h3 remote_gate2: gate 2, cancel roundtrip x2");
}

/// DIALOG-REMOTE-003 (`holla/flows/remote/gate2_typed`): the remote
/// restart gate-2 dialog with the target-bound phrase typed.
///
/// Flow: boot, type `restart payments`, Enter (gate 1), Right+Enter
/// (gate 2), Enter (arm), type the phrase, Esc (text cleared), Esc
/// (dialog cancelled). The phrase is synthetic fixture text; the
/// dialog cancels and never executes.
#[test]
#[ignore = "holla h3 check; run with --ignored"]
fn h3_remote_gate2_typed() {
    const PHRASE: &str = "RESTART PAYMENTS ON prod-eu-1";
    let dir = h3_dir("h3_remote_gate2_typed");
    let case = h3_case("remote_gate2_typed", REMOTE, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
        &mut s,
        &[
            "type:restart payments",
            "enter",
            &format!("wait:{GATE1}"),
            "right",
            "enter",
            &format!("wait:{GATE2}"),
            "enter",
            &format!("type:{PHRASE}"),
        ],
        case_timeout(&case),
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
        typed.contains("Restart payments · gate 2 of 2"),
        "V2: no gate-2 title"
    );
    // S1: the edit-mode footer (also the N1 presence proof).
    assert_line_has(&typed, "EDIT", "Type Phrase", "S1 edit footer");
    // S2: the gate-1 page survives below the dialog.
    assert!(typed.contains(GATE1), "S2: no gate-1 status");

    // A1: the first Esc clears the typed text (the dialog stays open).
    support::press_step(&s, "escape");
    wait_absent(&mut s, &format!("▎ {PHRASE}"), "phrase cleared");
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
    wait_absent(&mut s, GATE2, "dialog cancelled");
    wait_absent(&mut s, "to confirm", "prompt fully closed");
    waits::wait_state(&mut s, GATE1, "gate 1 back");
    checkpoint(&mut s, &dir, "03-cancelled");
    let back = live_text(&mut s);
    // N2: the gate-2 chrome is gone with the cancel.
    assert!(!back.contains(GATE2), "N2: gate 2 stuck");
    assert!(!back.contains("to confirm"), "N2: phrase prompt stuck");
    eprintln!("h3 remote_gate2_typed: typed, cleared, cancelled");
}

/// PANEL-REMOTE-001 (`holla/flows/remote/query`): the finder query
/// for the remote restart.
///
/// Flow: boot, type `restart payments`, Esc (cleared), type again.
#[test]
#[ignore = "holla h3 check; run with --ignored"]
fn h3_remote_query() {
    let dir = h3_dir("h3_remote_query");
    let case = h3_case("remote_query", REMOTE, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // A1: typing filters to the remote restart actions.
    support::drive(
        &mut s,
        &["type:restart payments", "wait:Results · 2"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    // V1: the top row pairs the restart with its privilege.
    assert_line_has(&res, "› Restart payments", "privileged", "V1 row");
    // V2: the result count.
    assert!(res.contains("Results · 2"), "V2: no Results · 2");
    // S1: the preview names the two-gate contract.
    assert_line_has(&res, "Gate", "two gates", "S1 gate");
    // S2: the scope readout stays here.
    assert!(res.contains("scope ‹ here ›"), "S2: no here scope");
    // N1: the empty-query placeholder is gone with the results.
    assert!(!res.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: Esc clears the query back to the placeholder.
    support::press_step(&s, "escape");
    wait_absent(&mut s, "Results · 2", "results cleared");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);
    // N2: the result count is gone after the clear.
    assert!(!cleared.contains("Results · 2"), "N2: results stuck");
    support::drive(
        &mut s,
        &["type:restart payments", "wait:Results · 2"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-results-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "› Restart payments", "privileged", "row intact");
    eprintln!("h3 remote_query: results, clear roundtrip");
}

/// PANEL-TRUST-001 (`holla/flows/trust/accepted`): the task Arguments
/// page after accepting the trust prompt.
///
/// Flow: boot, type `test`, Enter (prompt), Right+Enter (accepted,
/// args page), Esc (finder back), Enter (args again, no re-prompt).
#[test]
#[ignore = "holla h3 check; run with --ignored"]
fn h3_trust_accepted() {
    const PROMPT: &str = "Trust ~/work/acme/apps/frontend/mise.toml?";
    const ARGS: &str = "Run tests · arguments";
    const TRUSTED: &str = "Trusted ~/work/acme/apps/frontend/mise.toml";
    let dir = h3_dir("h3_trust_accepted");
    let case = h3_case("trust_accepted", CHILD, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
        &mut s,
        &["type:test", "enter", &format!("wait:{PROMPT}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-prompt");
    let prompt = live_text(&mut s);
    assert!(prompt.contains(PROMPT), "pre: no trust prompt");
    assert!(prompt.contains("not trusted yet"), "pre: no untrusted fact");

    // A1: Right+Enter trusts the file and opens the Arguments page.
    support::drive(
        &mut s,
        &["right", "enter", &format!("wait:{TRUSTED}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-args");
    let args = live_text(&mut s);

    // V1: the page titles the accepted task with its arguments class.
    assert!(args.contains(ARGS), "V1: no args title");
    // V2: the rendered command pins the profile default.
    assert!(
        args.contains("mise run //apps/frontend:test --profile default"),
        "V2: no mise command"
    );
    // S1: the status records the trusted file.
    assert!(args.contains(TRUSTED), "S1: no trusted status");
    // S2: the footer offers the run gesture.
    assert!(args.contains("Ctrl+S Run"), "S2: no run footer");
    // N1: the trust prompt is gone on the arguments page.
    assert!(!args.contains(PROMPT), "N1: prompt stuck");

    // A2: Esc backs to the finder, Enter re-opens the args page
    // without a second prompt (the file stays trusted).
    support::press_step(&s, "escape");
    wait_absent(&mut s, ARGS, "args closed");
    waits::wait_state(&mut s, "Type Search", "finder back");
    support::drive(
        &mut s,
        &["enter", &format!("wait:{ARGS}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-args-again");
    let back = live_text(&mut s);
    // N2: no second trust review after re-running the trusted task.
    assert!(!back.contains("not trusted yet"), "N2: re-prompted");
    assert_line_has(&back, "mise run", "--profile default", "args intact");
    eprintln!("h3 trust_accepted: accepted, no re-prompt");
}

/// PANEL-TRUST-002 (`holla/flows/trust/deploy_failed`): the executor
/// output for the trusted deploy without a simulated outcome.
///
/// Flow: boot, type `Deploy preview`, Enter (prompt), Right+Enter
/// (accepted, exit 127), r (restart, fails again), Alt+0 (back to
/// Here). The script is the fixture world's simulation; nothing real
/// executes.
#[test]
#[ignore = "holla h3 check; run with --ignored"]
fn h3_trust_deploy_failed() {
    const EXIT: &str = "exit 127";
    let dir = h3_dir("h3_trust_deploy_failed");
    let case = h3_case("trust_deploy_failed", CUSTOM, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // A1: trusting the deploy runs it to the missing-outcome failure.
    support::drive(
        &mut s,
        &[
            "type:Deploy preview",
            "enter",
            "wait:Trust ~/work/team/.holla.toml?",
            "right",
            "enter",
            &format!("wait:{EXIT}"),
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-failed");
    let failed = live_text(&mut s);

    // V1: the output records the missing simulated outcome.
    assert!(
        failed.contains("holla: no simulated outcome for `tools/deploy-preview.sh --env 'preview stack' '$(whoami)'` in this preview"),
        "V1: no outcome line"
    );
    // V2: the title pairs the deploy with its failed exit.
    assert_line_has(&failed, "deploy preview · failed", EXIT, "V2 title");
    // S1: the output names the started-by action.
    assert!(
        failed.contains("started by Deploy preview"),
        "S1: no started-by line"
    );
    // S2: the footer records the failed deploy.
    assert!(
        failed.contains("! deploy preview failed · exit 127"),
        "S2: no failed footer"
    );
    // N1: the empty-query placeholder is gone on the output.
    assert!(!failed.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: r restarts the script, the deploy fails again.
    // press_step (no pacing sleep): the restart clears the output before
    // wait_gone's first sample would otherwise miss the presence proof.
    support::press_step(&s, "r");
    wait_absent(&mut s, EXIT, "output restarted");
    waits::wait_state(&mut s, EXIT, "deploy failed again");
    checkpoint(&mut s, &dir, "02-failed-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "deploy preview · failed", EXIT, "title intact");
    // The Here footer keeps the `! deploy preview failed · exit 127`
    // summary, so the output body (not the exit marker) is the
    // left-the-executor signal.
    support::press_step(&s, "alt-0");
    wait_absent(&mut s, "no simulated outcome", "back to Here");
    waits::wait_state(&mut s, "scope ‹ here ›", "finder back");
    checkpoint(&mut s, &dir, "03-here");
    let here = live_text(&mut s);
    // N2: the output body is gone after leaving the executor.
    assert!(
        !here.contains("no simulated outcome"),
        "N2: output body stuck"
    );
    eprintln!("h3 trust_deploy_failed: fail, restart roundtrip");
}

/// DIALOG-TRUST-001 (`holla/flows/trust/prompt`): the trust review
/// prompt for the frontend mise file.
///
/// Flow: boot, type `test`, Enter (prompt), Esc (back), Esc (query
/// cleared), type + Enter (prompt again).
#[test]
#[ignore = "holla h3 check; run with --ignored"]
fn h3_trust_prompt() {
    const PROMPT: &str = "Trust ~/work/acme/apps/frontend/mise.toml?";
    let dir = h3_dir("h3_trust_prompt");
    let case = h3_case("trust_prompt", CHILD, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // A1: Enter opens the trust review.
    support::drive(
        &mut s,
        &["type:test", "enter", &format!("wait:{PROMPT}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-prompt");
    let prompt = live_text(&mut s);

    // V1: the page titles the exact untrusted file.
    assert!(prompt.contains(PROMPT), "V1: no prompt title");
    // V2: the defines fact counts the reviewed content.
    assert_line_has(&prompt, "Defines", "4 tasks · 2 tools", "V2 defines");
    // S1: the page states the untrusted precondition.
    assert!(
        prompt.contains(
            "not trusted yet · review the exact definition, commands and environment before it runs"
        ),
        "S1: no untrusted fact"
    );
    // S2: the dialog offers the trust choice.
    assert_line_has(&prompt, "Cancel", "Trust this file", "S2 choice");
    // N1: the empty-query placeholder is gone on the review.
    assert!(!prompt.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: Esc backs to the finder, Esc clears the retained query.
    support::press_step(&s, "escape");
    wait_absent(&mut s, PROMPT, "review closed");
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-finder");
    let finder = live_text(&mut s);
    // N2: the trust choice is gone after backing out.
    assert!(
        !finder.contains("Trust this file"),
        "N2: trust choice stuck"
    );
    support::drive(
        &mut s,
        &["type:test", "enter", &format!("wait:{PROMPT}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-prompt-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "Defines", "4 tasks · 2 tools", "review intact");
    eprintln!("h3 trust_prompt: prompt, back roundtrip");
}
