//! Holla pending-roots slice 8A-H4 executable checks (VB Phase-8p).
//!
//! One ignored PTY test per H4 registry row (22 rows: the deferred H3
//! `upgrade` triplet — `upgrade/confirm`, `upgrade/excluded`,
//! `upgrade/facts` — plus the first 19 of the plan's 21-root 8A-H4
//! parity slice in sorted root order, through `parity/task-input`;
//! `parity/task-sources` and `parity/upgrade-managers` stay pending for
//! a later tranche), over the real `holla` binary built from this
//! worktree's verified VB sources (`env!("CARGO_BIN_EXE_holla")`, the
//! harness `[[bin]]` compiled from `../../src/bin/holla/main.rs`).
//! Each test drives the row's specified inputs and asserts its visual
//! (V), state (S), action (A), and negative (N) checks in live-PTY
//! executable form:
//!
//! - V: live needles from the row's boot frame (suggestion rows,
//!   counts, badges) plus same-line coexistence proofs.
//! - S: live labels (finder scope, facts, footers, status rows) from
//!   the boot frame (S1) and the flow landing state (S2).
//! - A: behavior probes — the specified gesture runs and the specified
//!   advance (toggle, dialog, cancel, restart, or boundary clear) is
//!   observed; `Changed`/`Consumed`/`Ignored` outcomes are proven by
//!   their screen correlates (moved, byte-identical), since PTY cannot
//!   see the enum.
//! - N: absence assertions, each paired with a presence proof in the
//!   same test (boot checkpoint, mid-flow checkpoint, or pre/post
//!   transition) so no absence passes vacuously.
//!
//! No new snapshots and no new static captures: all 22 roots already
//! have approved frames gated cell-exact by the ported matrices in
//! `holla.rs` (plan §Reconciliation: "rows plus checks, not
//! recaptures"), so every case here uses owned (dynamic) names and
//! stays out of the `snapshots/` inventory. No isolated-component
//! captures: every row's `requires` set (key-injection,
//! glyph-capture) is PTY-level, and every assertion has a live-PTY
//! executable form.
//!
//! Per-test scratch lands under this harness crate's
//! `target/tuiscotti/journeys/<test>/` (gitignored): the row's named
//! checkpoint(s) as text frames, `00-boot.txt` where the row needs a
//! boot comparison, and `provenance.txt` (binary path, size, mtime,
//! argv). Typed input is synthetic and in-memory only (simulation
//! data); no test runs real commands, touches Git state, or performs
//! destructive operations: the executor rows run the fixture world's
//! simulated scripts only, every gate-2 dialog is cancelled, never
//! executed, every plan confirm dialog is cancelled (never confirmed),
//! and every world is the fixture simulation.
//!
//! Row → test map (registry id → `h4_*` test):
//!
//! - DIALOG-UPGRADE-001 → [`h4_upgrade_confirm`]
//! - PANEL-UPGRADE-001 → [`h4_upgrade_excluded`]
//! - PANEL-UPGRADE-002 → [`h4_upgrade_facts`]
//! - PARITY-BREW-SERVICES-001 → [`h4_parity_brew_services`]
//! - PARITY-BROWSER-001 → [`h4_parity_browser`]
//! - PARITY-CARGO-001 → [`h4_parity_cargo`]
//! - PARITY-CLEANUP-RESULTS-001 → [`h4_parity_cleanup_results`]
//! - PARITY-CUSTOM-ACTIONS-001 → [`h4_parity_custom_actions`]
//! - PARITY-DELETE-SAFETY-001 → [`h4_parity_delete_safety`]
//! - PARITY-DISK-NAVIGATION-001 → [`h4_parity_disk_navigation`]
//! - PARITY-DOCKER-001 → [`h4_parity_docker`]
//! - PARITY-EXECUTOR-001 → [`h4_parity_executor`]
//! - PARITY-FILES-001 → [`h4_parity_files`]
//! - PARITY-GIT-BATCH-001 → [`h4_parity_git_batch`]
//! - PARITY-GIT-CURRENT-001 → [`h4_parity_git_current`]
//! - PARITY-GRADLE-001 → [`h4_parity_gradle`]
//! - PARITY-HISTORY-001 → [`h4_parity_history`]
//! - PARITY-IDEA-001 → [`h4_parity_idea`]
//! - PARITY-INSIGHTS-001 → [`h4_parity_insights`]
//! - PARITY-PLATFORMS-001 → [`h4_parity_platforms`]
//! - PARITY-PLATFORMS-LINUX-001 → [`h4_parity_platforms_linux`]
//! - PARITY-TASK-INPUT-001 → [`h4_parity_task_input`]

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

const UPGRADE: &[&str] = &["--scenario", "upgrade-plan", "--motion", "reduced"];
const UPGRADE_PAUSED: &[&str] = &[
    "--scenario",
    "upgrade-plan",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const BREW: &[&str] = &[
    "--scenario",
    "parity-brew-services",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const BROWSER: &[&str] = &[
    "--scenario",
    "parity-browser",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const CARGO: &[&str] = &[
    "--scenario",
    "parity-cargo",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const CLEANUP_RESULTS: &[&str] = &[
    "--scenario",
    "parity-cleanup-results",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const CUSTOM: &[&str] = &[
    "--scenario",
    "parity-custom-actions",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const DELSAFE: &[&str] = &["--scenario", "parity-delete-safety", "--motion", "reduced"];
const NAV: &[&str] = &[
    "--scenario",
    "parity-disk-navigation",
    "--motion",
    "reduced",
];
const PDOCKER: &[&str] = &[
    "--scenario",
    "parity-docker",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const EXECUTOR: &[&str] = &["--scenario", "parity-executor", "--motion", "reduced"];
const FILES: &[&str] = &[
    "--scenario",
    "parity-files",
    "--motion",
    "paused",
    "--frame",
    "40",
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
const GRADLE: &[&str] = &[
    "--scenario",
    "parity-gradle",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const HISTORY: &[&str] = &[
    "--scenario",
    "parity-history",
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
const INSIGHTS: &[&str] = &[
    "--scenario",
    "parity-insights",
    "--motion",
    "paused",
    "--frame",
    "40",
];
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
const TASKINPUT: &[&str] = &["--scenario", "parity-task-input", "--motion", "reduced"];

/// Owned-name H4 case at the canonical 120x40 truecolor geometry.
fn h4_case(slug: &str, args: &'static [&'static str], boot: &'static str) -> Case {
    Case::dynamic(
        format!("journeys/holla/h4/{slug}"),
        HOLLA,
        args,
        120,
        40,
        Color::Truecolor,
        boot,
    )
}

/// [`h4_case`] with an explicit per-step timeout for tick-driven flows.
fn h4_case_t(
    slug: &str,
    args: &'static [&'static str],
    boot: &'static str,
    timeout_ms: u64,
) -> Case {
    h4_case(slug, args, boot).timeout(timeout_ms)
}

/// Unique scratch dir for one H4 test (created, never shared).
fn h4_dir(test: &str) -> PathBuf {
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
    eprintln!("h4 provenance: {body}");
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
/// coexistence proof: boot rows, dialog rows, titles, fact rows).
fn assert_line_has(text: &str, needle: &str, also: &str, what: &str) {
    assert!(
        text.lines().any(|l| l.contains(needle) && l.contains(also)),
        "{what}: no line contains both `{needle}` and `{also}`"
    );
}

/// DIALOG-UPGRADE-001 (`holla/flows/upgrade/confirm`): the upgrade
/// confirm dialog over the 7-included plan.
///
/// Flow: boot, move to step 06, Space (excluded), c (dialog), Esc
/// (cancelled), c (dialog again), Esc (cancelled again). Never
/// confirmed, never executed: Cancel stays focused and every dialog
/// only ever cancels.
#[test]
#[ignore = "holla h4 check; run with --ignored"]
fn h4_upgrade_confirm() {
    const DIALOG: &str = "Start upgrade everything?";
    const CANCELLED: &str = "Cancelled · nothing was executed";
    const FOOTER: &str = "← → Choose  Enter Confirm  Esc Cancel";
    let dir = h4_dir("h4_upgrade_confirm");
    let case = h4_case("upgrade_confirm", UPGRADE, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(
        boot.contains("8 steps · 8 included · 0 excluded"),
        "pre: boot has no full plan"
    );

    support::drive(
        &mut s,
        &["down", "down", "down", "down", "down", "space"],
        case_timeout(&case),
    );
    waits::wait_state(&mut s, "7 included · 1 excluded", "step excluded");

    // A1: c opens the confirm dialog over the 7-included plan.
    support::drive(
        &mut s,
        &["c", &format!("wait:{DIALOG}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-dialog");
    let dialog = live_text(&mut s);

    // V1: the dialog titles the upgrade question.
    assert!(dialog.contains(DIALOG), "V1: no dialog title");
    // V2: the steps fact carries the exclusion into the dialog.
    assert_line_has(&dialog, "Steps", "7 included · 1 excluded", "V2 steps");
    // S1: Cancel stays the focused choice.
    assert_line_has(&dialog, "▎Cancel", "Start plan", "S1 choice");
    // S2: the dialog footer offers the confirm gesture.
    assert!(dialog.contains(FOOTER), "S2: no dialog footer");
    // N1: the cancelled status is not set while the dialog is open.
    assert!(!dialog.contains(CANCELLED), "N1: cancelled before Esc");

    // A2: Esc cancels the dialog, c re-opens it, Esc cancels again.
    support::press_step(&s, "escape");
    wait_absent(&mut s, DIALOG, "dialog cancelled");
    waits::wait_state(&mut s, CANCELLED, "cancelled status");
    checkpoint(&mut s, &dir, "02-cancelled");
    let cancelled = live_text(&mut s);
    // N2: the dialog title is gone after the cancel.
    assert!(!cancelled.contains(DIALOG), "N2: dialog title stuck");
    support::drive(
        &mut s,
        &["c", &format!("wait:{DIALOG}")],
        case_timeout(&case),
    );
    support::press_step(&s, "escape");
    wait_absent(&mut s, DIALOG, "dialog cancelled again");
    waits::wait_state(&mut s, CANCELLED, "cancelled again");
    checkpoint(&mut s, &dir, "03-cancelled-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "8 steps", "7 included · 1 excluded", "plan intact");
    eprintln!("h4 upgrade_confirm: dialog, cancel roundtrip x2");
}

/// PANEL-UPGRADE-001 (`holla/flows/upgrade/excluded`): the upgrade plan
/// with step 06 excluded.
///
/// Flow: boot, move to step 06, Space (excluded), Space (included),
/// Space (excluded again).
#[test]
#[ignore = "holla h4 check; run with --ignored"]
fn h4_upgrade_excluded() {
    let dir = h4_dir("h4_upgrade_excluded");
    let case = h4_case("upgrade_excluded", UPGRADE, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(
        boot.contains("8 steps · 8 included · 0 excluded"),
        "pre: boot has no full plan"
    );

    // A1: Space excludes step 06.
    support::drive(
        &mut s,
        &[
            "down",
            "down",
            "down",
            "down",
            "down",
            "space",
            "wait:7 included · 1 excluded",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-excluded");
    let excluded = live_text(&mut s);

    // V1: the cursor row marks step 06 excluded.
    assert_line_has(
        &excluded,
        "▎[ ] 06 Upgrade selected mise tools",
        "excluded",
        "V1 row",
    );
    // V2: the header counts the exclusion.
    assert!(
        excluded.contains("7 included · 1 excluded"),
        "V2: no exclusion count"
    );
    // S1: the step facts drawer reports the excluded state.
    assert_line_has(&excluded, "State", "excluded", "S1 state");
    // S2: the header keeps the 8-step plan.
    assert_line_has(&excluded, "8 steps", "7 included · 1 excluded", "S2 header");
    // N1: the full-inclusion count is gone with the exclusion.
    assert!(!excluded.contains("8 included"), "N1: full count stuck");

    // A2: Space re-includes, Space excludes again.
    support::press_step(&s, "space");
    waits::wait_state(&mut s, "8 included · 0 excluded", "step included");
    checkpoint(&mut s, &dir, "02-included");
    support::press_step(&s, "space");
    waits::wait_state(&mut s, "7 included · 1 excluded", "step excluded again");
    checkpoint(&mut s, &dir, "03-excluded-again");
    let back = live_text(&mut s);
    // N2: the zero-exclusion count is gone after re-excluding.
    assert!(!back.contains("0 excluded"), "N2: zero count stuck");
    assert_line_has(
        &back,
        "▎[ ] 06 Upgrade selected mise tools",
        "excluded",
        "row intact",
    );
    eprintln!("h4 upgrade_excluded: exclusion toggle roundtrip");
}

/// PANEL-UPGRADE-002 (`holla/flows/upgrade/facts`): the plan step-facts
/// drawer with keyboard focus inside it.
///
/// Flow: boot (steps focus), p (drawer focus), p (steps back), p
/// (drawer again).
#[test]
#[ignore = "holla h4 check; run with --ignored"]
fn h4_upgrade_facts() {
    let dir = h4_dir("h4_upgrade_facts");
    let case = h4_case("upgrade_facts", UPGRADE_PAUSED, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(
        boot.contains("▎[✓] 01 Preflight"),
        "pre: boot has no steps cursor"
    );

    // A1: p moves focus into the step-facts drawer.
    support::drive(&mut s, &["p", "wait:▎01 Preflight"], case_timeout(&case));
    checkpoint(&mut s, &dir, "01-drawer");
    let drawer = live_text(&mut s);

    // V1: the drawer header carries focus with its required badge.
    assert_line_has(&drawer, "▎01 Preflight", "required", "V1 header");
    // V2: the runs fact names the first preflight check.
    assert_line_has(&drawer, "Runs", "cat /etc/os-release", "V2 runs");
    // S1: the privilege fact.
    assert_line_has(&drawer, "Privilege", "sudo", "S1 privilege");
    // S2: the ready state.
    assert_line_has(&drawer, "State", "ready", "S2 state");
    // N1: the steps cursor is gone with drawer focus.
    assert!(!drawer.contains("▎[✓]"), "N1: steps cursor stuck");

    // A2: p returns focus to the steps, p focuses the drawer again.
    support::press_step(&s, "p");
    waits::wait_state(&mut s, "▎[✓] 01 Preflight", "steps back");
    checkpoint(&mut s, &dir, "02-steps");
    let steps = live_text(&mut s);
    // N2: the drawer cursor is gone with steps focus.
    assert!(!steps.contains("▎01 Preflight"), "N2: drawer cursor stuck");
    support::drive(&mut s, &["p", "wait:▎01 Preflight"], case_timeout(&case));
    checkpoint(&mut s, &dir, "03-drawer-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "▎01 Preflight", "required", "drawer intact");
    eprintln!("h4 upgrade_facts: drawer focus roundtrip");
}

/// PARITY-BREW-SERVICES-001 (`holla/parity/brew-services`): the
/// brew-services parity boot plus the services listing behavior.
///
/// Flow: boot, type `Homebrew services`, Enter (listing), Esc (back),
/// Esc (query cleared), type + Enter (listing again).
#[test]
#[ignore = "holla h4 check; run with --ignored"]
fn h4_parity_brew_services() {
    const DETAIL: &str = "12 lines · y copies";
    let dir = h4_dir("h4_parity_brew_services");
    let case = h4_case("parity_brew_services", BREW, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the explore row lists the entry points with Tasks first.
    assert_line_has(&boot, "›Tasks", "Services", "V1 explore");
    // V2: the recent shelf is empty at boot.
    assert!(
        boot.contains("nothing used here yet"),
        "V2: no empty recent"
    );
    // S1: the facts anchor the listing world.
    assert_line_has(&boot, "In", "~/work", "S1 world");

    // A1: Enter opens the services listing.
    support::drive(
        &mut s,
        &["type:Homebrew services", "enter", "wait:svc02"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-services");
    let list = live_text(&mut s);
    // S2: the detail panel (the A2 close signal) is open.
    assert!(list.contains(DETAIL), "S2: no detail panel");
    // V3-flow: the errored service row pairs the name with its status.
    assert_line_has(&list, "svc02", "error", "flow row");
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
    eprintln!("h4 parity_brew_services: boot, listing roundtrip");
}

/// PARITY-BROWSER-001 (`holla/parity/browser`): the browser parity boot
/// plus the hidden-entries toggle behavior.
///
/// Flow: boot, type `Browse ~/work/site`, Enter (listing), Ctrl+H
/// (shown), Ctrl+H (hidden), Ctrl+H (shown again).
#[test]
#[ignore = "holla h4 check; run with --ignored"]
fn h4_parity_browser() {
    let dir = h4_dir("h4_parity_browser");
    let case = h4_case("parity_browser", BROWSER, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the top suggestion pairs the script with its manifest.
    assert_line_has(&boot, "› pnpm a", "package.json here", "V1 row");
    // V2: the script shelf lists the package scripts.
    assert!(boot.contains("pnpm e"), "V2: no script shelf");
    // S1: the facts anchor the site folder.
    assert_line_has(&boot, "In", "~/work/site", "S1 world");

    support::drive(
        &mut s,
        &["type:Browse ~/work/site", "enter", "wait:16 entries"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-listing");
    let listing = live_text(&mut s);
    assert!(
        listing.contains("2 hidden hidden"),
        "pre: no hidden-hidden status"
    );

    // A1: Ctrl+H toggles hidden entries on.
    support::drive(
        &mut s,
        &["ctrl-h", "wait:2 hidden shown"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-hidden");
    let hidden = live_text(&mut s);

    // S2: the header counts all entries with hidden shown.
    assert_line_has(&hidden, "18 entries", "2 hidden shown", "S2 header");
    // Flow: the dotfile row resolves with its kind.
    assert_line_has(&hidden, ".git", "directory", "flow dotfile");
    // Flow: the footer announces the shown hidden entries.
    assert!(
        hidden.contains("Hidden entries shown"),
        "flow: no shown footer"
    );
    // N1: the hidden-hidden status is gone with the toggle.
    assert!(
        !hidden.contains("2 hidden hidden"),
        "N1: hidden status stuck"
    );

    // A2: Ctrl+H toggles hidden back off, then on again.
    support::drive(
        &mut s,
        &["ctrl-h", "wait:2 hidden hidden"],
        case_timeout(&case),
    );
    let off = live_text(&mut s);
    // N2: the shown count is gone after the second toggle.
    assert!(!off.contains("2 hidden shown"), "N2: shown count stuck");
    support::drive(
        &mut s,
        &["ctrl-h", "wait:2 hidden shown"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-hidden-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "18 entries", "2 hidden shown", "header intact");
    eprintln!("h4 parity_browser: boot, hidden toggle roundtrip x2");
}

/// PARITY-CARGO-001 (`holla/parity/cargo`): the cargo parity boot plus
/// the clean-action dialog behavior.
///
/// Flow: boot, type `cargo clean`, Enter (dialog), Esc (cancel),
/// Enter (dialog again), Esc (cancel again). Never Run: the dialog
/// only ever cancels.
#[test]
#[ignore = "holla h4 check; run with --ignored"]
fn h4_parity_cargo() {
    const DIALOG_FOOTER: &str = "← → Choose  Enter Confirm  Esc Cancel";
    let dir = h4_dir("h4_parity_cargo");
    let case = h4_case("parity_cargo", CARGO, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the top suggestion pairs the build with its manifest.
    assert_line_has(&boot, "› cargo build", "Cargo.toml here", "V1 row");
    // V2: the clean action carries its byte badge.
    assert!(
        boot.contains("cargo clean · 900.0 MiB"),
        "V2: no clean badge"
    );
    // S1: the facts anchor the engine folder.
    assert_line_has(&boot, "In", "~/work/engine", "S1 world");

    // A1: typing filters to the clean action.
    support::drive(
        &mut s,
        &["type:cargo clean", "wait:900.0 MiB"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-clean");
    let clean = live_text(&mut s);
    assert_line_has(
        &clean,
        "› cargo clean · 900.0 MiB",
        "destructive",
        "flow row",
    );
    // S2: the gate fact names the one-confirmation flow.
    assert_line_has(&clean, "Gate", "one confirmation", "S2 gate");
    // N1: the confirm dialog is closed at the clean state.
    assert!(
        !clean.contains(DIALOG_FOOTER),
        "N1: dialog open before Enter"
    );

    // A2: Enter opens the dialog, Esc cancels it, twice.
    support::drive(
        &mut s,
        &["enter", &format!("wait:{DIALOG_FOOTER}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-dialog-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DIALOG_FOOTER, "dialog cancelled");
    waits::wait_state(&mut s, "900.0 MiB", "clean state back");
    support::drive(
        &mut s,
        &["enter", &format!("wait:{DIALOG_FOOTER}")],
        case_timeout(&case),
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
    eprintln!("h4 parity_cargo: boot, dialog roundtrip x2");
}

/// PARITY-CLEANUP-RESULTS-001 (`holla/parity/cleanup-results`): the
/// cleanup-results parity boot plus the history behavior.
///
/// Flow: boot, type `Show cleanup history`, Enter (history), Esc
/// (finder back), Enter (history again).
#[test]
#[ignore = "holla h4 check; run with --ignored"]
fn h4_parity_cleanup_results() {
    const HIST: &str = "operation log records";
    let dir = h4_dir("h4_parity_cleanup_results");
    let case = h4_case("parity_cleanup_results", CLEANUP_RESULTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the clean row pairs the action with the destructive badge.
    assert_line_has(&boot, "cargo clean · 1000.0 MiB", "destructive", "V1 row");
    // V2: the cargo shelf lists the project actions.
    assert!(boot.contains("cargo fmt --check"), "V2: no cargo shelf");
    // S1: the facts anchor the safe folder.
    assert_line_has(&boot, "In", "~/Projects/safe", "S1 world");
    // N1: the history page is not the boot state.
    assert!(!boot.contains(HIST), "N1: history visible at boot");

    // A1: Enter opens the cleanup history.
    support::drive(
        &mut s,
        &[
            "type:Show cleanup history",
            "enter",
            &format!("wait:{HIST}"),
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-history");
    let hist = live_text(&mut s);

    // S2: the history header counts the operation log.
    assert_line_has(&hist, "Entries", "1 operation log records", "S2 header");
    // Flow: the prior record row.
    assert_line_has(&hist, "2026-08-29 08:41", "trashed", "flow record");

    // A2: Esc leaves the history, Enter re-opens it over the intact query.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, HIST, "history left");
    checkpoint(&mut s, &dir, "02-finder");
    let left = live_text(&mut s);
    // N2: the history chrome is gone with the leave.
    assert!(!left.contains(HIST), "N2: history stuck");
    assert!(!left.contains("Cleanup history"), "N2: title stuck");
    support::drive(
        &mut s,
        &["enter", &format!("wait:{HIST}")],
        case_timeout(&case),
    );
    waits::wait_state(&mut s, "2026-08-29 08:41", "record back");
    checkpoint(&mut s, &dir, "03-history-again");
    eprintln!("h4 parity_cleanup_results: boot, history roundtrip");
}

/// PARITY-CUSTOM-ACTIONS-001 (`holla/parity/custom-actions`): the
/// custom-actions parity boot plus the trust-prompt behavior.
///
/// Flow: boot, type `Deploy preview`, Enter (prompt), Esc (back), Esc
/// (query cleared), type + Enter (prompt again). The prompt only ever
/// cancels; nothing is trusted, nothing runs.
#[test]
#[ignore = "holla h4 check; run with --ignored"]
fn h4_parity_custom_actions() {
    const PROMPT: &str = "Trust ~/work/team/.holla.toml?";
    let dir = h4_dir("h4_parity_custom_actions");
    let case = h4_case("parity_custom_actions", CUSTOM, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the top suggestion pairs the deploy with its unreviewed mark.
    assert_line_has(&boot, "› Deploy preview", "unreviewed", "V1 row");
    // V2: the trusted shelf lists the shared actions.
    assert!(boot.contains("Global shared id"), "V2: no trusted shelf");
    // S1: the gate fact names the trust requirement.
    assert_line_has(&boot, "Gate", "trust required", "S1 gate");

    // A1: Enter opens the trust review.
    support::drive(
        &mut s,
        &["type:Deploy preview", "enter", &format!("wait:{PROMPT}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-prompt");
    let prompt = live_text(&mut s);

    // S2: the review names the exact untrusted definition.
    assert!(
        prompt.contains("label = \"Deploy preview\""),
        "S2: no definition label"
    );
    // N1: the empty-query placeholder is gone on the review.
    assert!(!prompt.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: Esc backs to the finder, Esc clears the retained query.
    support::press_step(&s, "escape");
    wait_absent(&mut s, PROMPT, "review closed");
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-finder");
    let finder = live_text(&mut s);
    // N2: the trust prompt is gone after backing out.
    assert!(!finder.contains(PROMPT), "N2: prompt stuck");
    support::drive(
        &mut s,
        &["type:Deploy preview", "enter", &format!("wait:{PROMPT}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-prompt-again");
    let back = live_text(&mut s);
    assert!(back.contains("label = \"Deploy preview\""), "review intact");
    eprintln!("h4 parity_custom_actions: boot, prompt roundtrip");
}

/// PARITY-DELETE-SAFETY-001 (`holla/parity/delete-safety`): the
/// delete-safety parity boot plus the disk-scan behavior.
///
/// Flow: boot, type `Analyze disk usage`, Enter (scan completes), s
/// (apparent sort), s (allocated back).
#[test]
#[ignore = "holla h4 check; run with --ignored"]
fn h4_parity_delete_safety() {
    const APPARENT: &str = "largest first · apparent";
    const ALLOCATED: &str = "largest first · allocated";
    let dir = h4_dir("h4_parity_delete_safety");
    let case = h4_case_t("parity_delete_safety", DELSAFE, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    // The suggestion rows animate in after the header; settle them
    // before the boot asserts so V1/V2 cannot race the stagger.
    waits::wait_state(&mut s, "cargo fmt --check", "suggestions settled");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the clean row pairs the action with the destructive badge.
    assert_line_has(&boot, "cargo clean · 1000.0 MiB", "destructive", "V1 row");
    // V2: the cargo shelf lists the project actions.
    assert!(boot.contains("cargo fmt --check"), "V2: no cargo shelf");
    // S1: the freshness fact ticks with the live clock.
    assert_line_has(&boot, "Data", "live ·", "S1 freshness");

    // A1: Enter runs the scan to the unreadable warning.
    support::drive(
        &mut s,
        &["type:Analyze disk usage", "enter", "wait:1 unreadable"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-scan");
    let scan = live_text(&mut s);

    // S2: the allocated sort is active at the scan state.
    assert!(scan.contains(ALLOCATED), "S2: no allocated sort");
    // Flow: the completion summary.
    assert!(scan.contains("scan complete"), "flow: no completion");
    // N1: the apparent sort is not active at the scan state.
    assert!(!scan.contains(APPARENT), "N1: apparent sort active");

    // A2: s switches to apparent sort, s switches back.
    support::press_step(&s, "s");
    waits::wait_state(&mut s, APPARENT, "apparent on");
    checkpoint(&mut s, &dir, "02-apparent-presence");
    support::press_step(&s, "s");
    waits::wait_state(&mut s, ALLOCATED, "allocated back");
    let back = live_text(&mut s);
    // N2: the apparent sort is gone after the toggle-back.
    assert!(!back.contains(APPARENT), "N2: apparent stuck");
    assert!(back.contains("scan complete"), "N2: scan state lost");
    checkpoint(&mut s, &dir, "03-allocated-again");
    eprintln!("h4 parity_delete_safety: boot, scan + sort roundtrip");
}

/// PARITY-DISK-NAVIGATION-001 (`holla/parity/disk-navigation`): the
/// disk-navigation parity boot plus the disk-scan behavior.
///
/// Flow: boot, type `Analyze disk usage`, Enter (scan completes), s
/// (apparent sort), s (allocated back).
#[test]
#[ignore = "holla h4 check; run with --ignored"]
fn h4_parity_disk_navigation() {
    const APPARENT: &str = "largest first · apparent";
    const ALLOCATED: &str = "largest first · allocated";
    let dir = h4_dir("h4_parity_disk_navigation");
    let case = h4_case_t("parity_disk_navigation", NAV, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    // The suggestion rows animate in after the header; settle them
    // before the boot asserts so V1/V2 cannot race the stagger.
    waits::wait_state(&mut s, "cargo fmt --check", "suggestions settled");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the clean row pairs the action with the destructive badge.
    assert_line_has(&boot, "cargo clean · 2.0 MiB", "destructive", "V1 row");
    // V2: the cargo shelf lists the project actions.
    assert!(boot.contains("cargo fmt --check"), "V2: no cargo shelf");
    // S1: the facts anchor the big folder.
    assert_line_has(&boot, "In", "~/work/big", "S1 world");

    // A1: Enter runs the scan to completion.
    support::drive(
        &mut s,
        &["type:Analyze disk usage", "enter", "wait:scan complete"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-scan");
    let scan = live_text(&mut s);

    // S2: the allocated sort is active at the scan state.
    assert!(scan.contains(ALLOCATED), "S2: no allocated sort");
    // N1: the apparent sort is not active at the scan state.
    assert!(!scan.contains(APPARENT), "N1: apparent sort active");

    // A2: s switches to apparent sort, s switches back.
    support::press_step(&s, "s");
    waits::wait_state(&mut s, APPARENT, "apparent on");
    checkpoint(&mut s, &dir, "02-apparent-presence");
    support::press_step(&s, "s");
    waits::wait_state(&mut s, ALLOCATED, "allocated back");
    let back = live_text(&mut s);
    // N2: the apparent sort is gone after the toggle-back.
    assert!(!back.contains(APPARENT), "N2: apparent stuck");
    assert!(back.contains("scan complete"), "N2: scan state lost");
    checkpoint(&mut s, &dir, "03-allocated-again");
    eprintln!("h4 parity_disk_navigation: boot, scan + sort roundtrip");
}

/// PARITY-DOCKER-001 (`holla/parity/docker`): the docker parity boot
/// plus the stop-and-remove gate behavior.
///
/// Flow: boot, query the remove-all action, Enter (gate 1),
/// Right+Enter (gate 2), Esc (cancelled), Right+Enter (again), Esc
/// (cancelled again). Never armed, never executed.
#[test]
#[ignore = "holla h4 check; run with --ignored"]
fn h4_parity_docker() {
    let dir = h4_dir("h4_parity_docker");
    let case = h4_case("parity_docker", PDOCKER, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the top row pairs the unhealthy service with its state.
    assert_line_has(&boot, "› Follow acme-api-1 logs", "unhealthy", "V1 row");
    // V2: the healthy shelf lists the running services.
    assert!(
        boot.contains("Follow acme-db-1 logs"),
        "V2: no healthy shelf"
    );
    // S1: the facts anchor the stack folder.
    assert_line_has(&boot, "In", "~/work/stack", "S1 world");

    // A1: Enter opens the gate-1 review.
    support::drive(
        &mut s,
        &[
            "type:Stop and remove all",
            "enter",
            &format!("wait:{GATE1}"),
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-gate1");
    let gate = live_text(&mut s);

    assert_line_has(
        &gate,
        "Review · Stop and remove all containers",
        GATE1,
        "flow title",
    );
    // S2: the destructive changes fact.
    assert_line_has(&gate, "Changes", "6 containers removed", "S2 changes");
    // N1: the gate-2 dialog is closed at the gate-1 state.
    assert!(!gate.contains(GATE2), "N1: gate 2 open before Right+Enter");

    // A2: Right+Enter opens gate 2, Esc cancels it, twice.
    support::drive(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-gate2-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled");
    waits::wait_state(&mut s, GATE1, "gate 1 back");
    support::drive(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case_timeout(&case),
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
    eprintln!("h4 parity_docker: boot, gate roundtrip x2");
}

/// PARITY-EXECUTOR-001 (`holla/parity/executor`): the executor parity
/// boot plus the burst-run behavior.
///
/// Flow: boot, run `Emit a burst` (exit 0), r (restart, succeeds
/// again), Esc (back to Here). The script is the fixture world's
/// simulation; nothing real executes.
#[test]
#[ignore = "holla h4 check; run with --ignored"]
fn h4_parity_executor() {
    const DONE: &str = "emit a burst finished · exit 0";
    let dir = h4_dir("h4_parity_executor");
    let case = h4_case_t("parity_executor", EXECUTOR, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    // The suggestion rows animate in after the header; settle them
    // before the boot asserts so V1/V2 cannot race the stagger.
    waits::wait_state(&mut s, "Emit a mixed stream", "suggestions settled");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the top row pairs the batch status with its child state.
    assert_line_has(
        &boot,
        "› Status across 3 child",
        "0 dirty · 1 behind",
        "V1 row",
    );
    // V2: the burst shelf lists the fixture scripts.
    assert!(boot.contains("Emit a mixed stream"), "V2: no burst shelf");
    // S1: the facts anchor the batch folder.
    assert_line_has(&boot, "In", "~/work/batch", "S1 world");

    // A1: Enter runs the burst to the dropped-lines report.
    support::drive(
        &mut s,
        &[
            "type:Emit a burst",
            "enter",
            "wait:exit 0",
            "wait:500 earlier lines dropped",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-output");
    let output = live_text(&mut s);

    // S2: the title pairs the burst with its clean exit.
    assert_line_has(&output, "emit a burst · succeeded", "exit 0", "S2 title");
    // Flow: the output names the started-by action.
    assert!(
        output.contains("started by Emit a burst"),
        "flow: no started-by line"
    );
    // N1: the empty-query placeholder is gone on the output.
    assert!(!output.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: r restarts the script, the report returns.
    // press_step (no pacing sleep): the restart clears the output before
    // wait_gone's first sample would otherwise miss the presence proof.
    support::press_step(&s, "r");
    wait_absent(&mut s, DONE, "output restarted");
    waits::wait_state(&mut s, DONE, "burst finished again");
    checkpoint(&mut s, &dir, "02-output-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "emit a burst · succeeded", "exit 0", "title intact");
    support::press_step(&s, "escape");
    wait_absent(&mut s, "burst line", "back to Here");
    waits::wait_state(&mut s, "scope ‹ here ›", "finder back");
    checkpoint(&mut s, &dir, "03-here");
    let here = live_text(&mut s);
    // N2: the burst body is gone after leaving the executor.
    assert!(!here.contains("burst line"), "N2: burst body stuck");
    eprintln!("h4 parity_executor: boot, burst + restart roundtrip");
}

/// PARITY-FILES-001 (`holla/parity/files`): the files parity boot plus
/// the find-results behavior.
///
/// Flow: boot, find `readme` (3 results), Down (preview follows),
/// Up (preview back).
#[test]
#[ignore = "holla h4 check; run with --ignored"]
fn h4_parity_files() {
    const FIRST: &str = "~/Documents/README.md";
    const SECOND: &str = "~/work/app/README.md";
    let dir = h4_dir("h4_parity_files");
    let case = h4_case("parity_files", FILES, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the explore row lists the entry points with Tasks first.
    assert_line_has(&boot, "›Tasks", "Files", "V1 explore");
    // V2: the recent shelf is empty at boot.
    assert!(
        boot.contains("nothing used here yet"),
        "V2: no empty recent"
    );
    // S1: the facts anchor the notes folder.
    assert_line_has(&boot, "In", "~/work/notes", "S1 world");

    // A1: typing filters to the three readme results.
    support::drive(
        &mut s,
        &[
            "type:Find files under home",
            "enter",
            "wait:of 29 indexed",
            "type:readme",
            "wait:3 results",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    // S2: the preview titles the top result with its byte count.
    assert_line_has(&res, FIRST, "7 B of 7 B", "S2 preview");
    // Flow: the result count in home scope.
    assert!(res.contains("3 results · home scope"), "flow: no 3 results");

    // A2: Down moves the preview to the second row, Up moves it back.
    support::drive(
        &mut s,
        &["down", &format!("wait:{SECOND}")],
        case_timeout(&case),
    );
    let moved = live_text(&mut s);
    // N1: the first preview title is gone after moving down.
    assert!(!moved.contains(FIRST), "N1: first preview stuck");
    support::drive(
        &mut s,
        &["up", &format!("wait:{FIRST}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-results-again");
    let back = live_text(&mut s);
    // N2: the second preview title is gone after moving back.
    assert!(!back.contains(SECOND), "N2: second preview stuck");
    assert!(back.contains("3 results · home scope"), "count intact");
    eprintln!("h4 parity_files: boot, results cursor roundtrip");
}

/// PARITY-GIT-BATCH-001 (`holla/parity/git-batch`): the git-batch
/// parity boot plus the sibling-batch behavior.
///
/// Flow: boot, run `Pull 4 sibling repositories` (2 ok, 2 failed), r
/// (restart, same report), Alt+0 (back to Here). The scripts are the
/// fixture world's simulation; nothing real executes.
#[test]
#[ignore = "holla h4 check; run with --ignored"]
fn h4_parity_git_batch() {
    const FF: &str = "Fast-forward";
    let dir = h4_dir("h4_parity_git_batch");
    let case = h4_case_t("parity_git_batch", GITBATCH, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    // The explore shelf animates in after the suggestion rows; settle
    // it before the boot asserts so V2 cannot race the stagger.
    waits::wait_state(&mut s, "Children · 4 projects", "explore settled");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the top row pairs the batch status with its child state.
    assert_line_has(
        &boot,
        "› Status across 5 child",
        "0 dirty · 1 behind",
        "V1 row",
    );
    // V2: the explore shelf counts the child projects.
    assert!(boot.contains("Children · 4 projects"), "V2: no child count");
    // S1: the facts scope the batch to the children.
    assert_line_has(&boot, "Scope", "child", "S1 scope");

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

    // S2: the title pairs the pull with its clean exit.
    assert_line_has(&batch, "pull alpha · succeeded", "exit 0", "S2 title");
    // Flow: the output names the batch pull class.
    assert!(
        batch.contains("started by Pull 4 sibling repositories · batch git.pull-all"),
        "flow: no batch class"
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
    eprintln!("h4 parity_git_batch: boot, batch + restart roundtrip");
}

/// PARITY-GIT-CURRENT-001 (`holla/parity/git-current`): the git-current
/// parity boot plus the blocked-pull finder behavior.
///
/// Flow: boot, type `Pull` (blocked results), Esc (cleared), type
/// `Pull` again.
#[test]
#[ignore = "holla h4 check; run with --ignored"]
fn h4_parity_git_current() {
    let dir = h4_dir("h4_parity_git_current");
    let case = h4_case("parity_git_current", GITCURRENT, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the top row pairs the review with its file counts.
    assert_line_has(
        &boot,
        "› Review 1 modified files",
        "1 modified · 0 untracked",
        "V1 row",
    );
    // V2: the explore shelf names the parent project.
    assert!(boot.contains("Parent · svc"), "V2: no parent shelf");
    // S1: the facts scope the review to the parent.
    assert_line_has(&boot, "Scope", "parent", "S1 scope");

    // A1: typing filters to the pull actions.
    support::drive(
        &mut s,
        &["type:Pull", "wait:blocked · 1 modified"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    // S2: the result count.
    assert!(res.contains("Results · 3"), "S2: no Results · 3");
    // Flow: the top row pairs the pull with its blocked reason.
    assert_line_has(&res, "› Pull", "blocked · 1 modified", "flow row");
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
    eprintln!("h4 parity_git_current: boot, results clear roundtrip");
}

/// PARITY-GRADLE-001 (`holla/parity/gradle`): the gradle parity boot
/// plus the cleanup-review behavior.
///
/// Flow: boot, query the Gradle cleanup, Right (fold opens), Left
/// (closes), Right (opens), d (gate 1), Esc (review back). The gate
/// only ever opens and closes; nothing is armed, nothing runs.
#[test]
#[ignore = "holla h4 check; run with --ignored"]
fn h4_parity_gradle() {
    let dir = h4_dir("h4_parity_gradle");
    let case = h4_case("parity_gradle", GRADLE, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the top row pairs the cleanup with the destructive badge.
    assert_line_has(&boot, "› Clean Gradle outputs", "destructive", "V1 row");
    // V2: the gradle shelf lists the build actions.
    assert!(boot.contains("gradle test"), "V2: no gradle shelf");
    // S1: the gate fact names the review flow.
    assert_line_has(&boot, "Gate", "opens a review", "S1 gate");

    // A1: Enter opens the review, Right opens the fold.
    support::drive(
        &mut s,
        &[
            "type:Clean Gradle outputs",
            "enter",
            "wait:Cleanup · Gradle outputs",
            "right",
            "wait:▾ Gradle outputs",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-cleanup");
    let gradle = live_text(&mut s);

    // S2: the full eligibility.
    assert_line_has(
        &gradle,
        "Selection",
        "4 of 4 candidates eligible",
        "S2 selection",
    );
    // Flow: the opened fold with its bytes.
    assert_line_has(&gradle, "▾ Gradle outputs", "49.0 MiB", "flow fold");
    // N1: the gate review is not open at the cleanup state.
    assert!(!gradle.contains(GATE1), "N1: gate 1 open before d");

    // A2: Left closes the fold, Right re-opens it.
    support::press_step(&s, "left");
    waits::wait_gone(&mut s, "▾ Gradle outputs", "fold closed");
    support::press_step(&s, "right");
    waits::wait_state(&mut s, "▾ Gradle outputs", "fold reopened");
    // N1 presence: d opens gate 1, Esc returns to the review.
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
    eprintln!("h4 parity_gradle: boot, fold + gate roundtrips");
}

/// PARITY-HISTORY-001 (`holla/parity/history`): the history parity boot
/// plus the finder-query behavior.
///
/// Flow: boot, type `pull` (results), Esc (cleared), type `pull` again.
#[test]
#[ignore = "holla h4 check; run with --ignored"]
fn h4_parity_history() {
    let dir = h4_dir("h4_parity_history");
    let case = h4_case("parity_history", HISTORY, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the top row pairs the task with its manifest.
    assert_line_has(&boot, "› Run check", "mise.toml", "V1 row");
    // V2: the task shelf lists the mise tasks.
    assert!(boot.contains("Run tests"), "V2: no task shelf");
    // S1: the facts anchor the holla folder.
    assert_line_has(&boot, "In", "~/work/holla", "S1 world");

    // A1: typing filters to the pull results.
    support::drive(
        &mut s,
        &["type:pull", "wait:Results · 3"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    // S2: the footer offers the run gesture.
    assert!(res.contains("Enter Run"), "S2: no run footer");
    // Flow: the top row pairs the action with its state.
    assert_line_has(&res, "› Pull", "up to date", "flow row");
    // N1: the empty-query placeholder is gone with the results.
    assert!(!res.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: Esc clears the query back to the placeholder.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Results · 3", "results cleared");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);
    // N2: the result count is gone after the clear.
    assert!(!cleared.contains("Results · 3"), "N2: results stuck");
    support::drive(
        &mut s,
        &["type:pull", "wait:Results · 3"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-results-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "› Pull", "up to date", "row intact");
    eprintln!("h4 parity_history: boot, results clear roundtrip");
}

/// PARITY-IDEA-001 (`holla/parity/idea`): the idea parity boot plus the
/// cleanup-page behavior.
///
/// Flow: boot, type `Clean IntelliJ`, Enter (page), Esc (back), Esc
/// (query cleared), type + Enter (page again).
#[test]
#[ignore = "holla h4 check; run with --ignored"]
fn h4_parity_idea() {
    const TITLE: &str = "Cleanup · IntelliJ metadata";
    let dir = h4_dir("h4_parity_idea");
    let case = h4_case("parity_idea", IDEA, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the top row pairs the cleanup with the destructive badge.
    assert_line_has(&boot, "› Clean IntelliJ metadata", "destructive", "V1 row");
    // V2: the candidate count.
    assert!(boot.contains("4 candidates"), "V2: no candidate count");
    // S1: the facts anchor the ide folder.
    assert_line_has(&boot, "In", "~/work/ide", "S1 world");

    // A1: Enter opens the cleanup page.
    support::drive(
        &mut s,
        &["type:Clean IntelliJ", "enter", &format!("wait:{TITLE}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-page");
    let page = live_text(&mut s);

    // S2: the full eligibility.
    assert_line_has(
        &page,
        "Selection",
        "4 of 4 candidates eligible",
        "S2 selection",
    );
    // N1: the empty-query placeholder is gone on the page.
    assert!(!page.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: Esc backs to the finder, Esc clears the retained query.
    support::press_step(&s, "escape");
    wait_absent(&mut s, TITLE, "page closed");
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-finder");
    let finder = live_text(&mut s);
    // N2: the page title is gone after backing out.
    assert!(!finder.contains(TITLE), "N2: page title stuck");
    support::drive(
        &mut s,
        &["type:Clean IntelliJ", "enter", &format!("wait:{TITLE}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-page-again");
    let back = live_text(&mut s);
    assert_line_has(
        &back,
        "Selection",
        "4 of 4 candidates eligible",
        "page intact",
    );
    eprintln!("h4 parity_idea: boot, page roundtrip");
}

/// PARITY-INSIGHTS-001 (`holla/parity/insights`): the insights parity
/// boot plus the cleanup-review behavior.
///
/// Flow: boot, query the review, Down (cursor moves), Up (back),
/// Right (fold opens), Left (closes).
#[test]
#[ignore = "holla h4 check; run with --ignored"]
fn h4_parity_insights() {
    let dir = h4_dir("h4_parity_insights");
    let case = h4_case("parity_insights", INSIGHTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the top row pairs the review with its byte count.
    assert_line_has(&boot, "› Review cleanup candidates", "18.8 GiB", "V1 row");
    // V2: the category count.
    assert!(boot.contains("18 categories"), "V2: no category count");
    // S1: the facts scope the review host-wide.
    assert_line_has(&boot, "Scope", "host", "S1 scope");

    // A1: Enter opens the category review.
    support::drive(
        &mut s,
        &[
            "type:Review cleanup candidates",
            "enter",
            "wait:18 categories",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-categories");
    let cats = live_text(&mut s);

    // S2: the detail panel scopes to the cursor row.
    assert_line_has(&cats, "Safety", "rebuilt on demand", "S2 detail");
    // Flow: the review header with counts.
    assert_line_has(&cats, "18 categories", "18.8 GiB", "flow header");
    // N1: the first fold is closed at the landing state.
    assert!(
        !cats.contains("▾ Xcode DerivedData"),
        "N1: fold open before Right"
    );

    // A2: Down moves the cursor, Up returns it.
    support::press_step(&s, "down");
    waits::wait_state(&mut s, "▎[✓] ▸ Xcode device support", "cursor down");
    support::press_step(&s, "up");
    waits::wait_state(&mut s, "▎[–] ▸ Xcode DerivedData", "cursor back");
    // N1 presence: Right opens the fold, Left closes it again.
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
    eprintln!("h4 parity_insights: boot, cursor + fold roundtrips");
}

/// PARITY-PLATFORMS-001 (`holla/parity/platforms`): the platforms
/// parity boot plus the top-files page behavior.
///
/// Flow: boot, type `Top files on this Mac`, Enter (page), Esc (back),
/// Esc (query cleared), type + Enter (page again).
#[test]
#[ignore = "holla h4 check; run with --ignored"]
fn h4_parity_platforms() {
    const TIMEOUT: &str = "did not finish within 5 s";
    let dir = h4_dir("h4_parity_platforms");
    let case = h4_case("parity_platforms", PLATFORMS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the explore row lists the entry points with Tasks first.
    assert_line_has(&boot, "›Tasks", "System", "V1 explore");
    // V2: the recent shelf is empty at boot.
    assert!(
        boot.contains("nothing used here yet"),
        "V2: no empty recent"
    );
    // S1: the facts anchor the mac folder.
    assert_line_has(&boot, "In", "~/work/mac", "S1 world");

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

    // S2: the footer offers the back gesture.
    assert!(files.contains("Esc Back"), "S2: no back footer");
    // Flow: the page titles the Spotlight view.
    assert!(
        files.contains("Top files · Spotlight"),
        "flow: no page title"
    );
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
    eprintln!("h4 parity_platforms: boot, page roundtrip");
}

/// PARITY-PLATFORMS-LINUX-001 (`holla/parity/platforms-linux`): the
/// platforms-linux parity boot plus the Linux trash-gate behavior.
///
/// Flow: boot, open the review, select the row, d (gate 1),
/// Right+Enter (gate 2 opens), Esc (cancelled), Right+Enter (again),
/// Esc (cancelled again). Never armed, never executed.
#[test]
#[ignore = "holla h4 check; run with --ignored"]
fn h4_parity_platforms_linux() {
    let dir = h4_dir("h4_parity_platforms_linux");
    let case = h4_case("parity_platforms_linux", PLATFORMS_LINUX, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the explore row lists the entry points with Tasks first.
    assert_line_has(&boot, "›Tasks", "System", "V1 explore");
    // V2: the explore shelf names the Linux host.
    assert!(boot.contains("System · devbox"), "V2: no devbox shelf");
    // S1: the facts anchor the Linux host.
    assert_line_has(&boot, "Host", "devbox · local", "S1 host");

    // A1: d opens the gate-1 review over the selected row.
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

    assert_line_has(&gate, "Review · trash 1 item", GATE1, "flow title");
    // S2: the selection size survives into the review.
    assert!(
        gate.contains("1 item selected · 124.0 KiB"),
        "S2: no selection size"
    );
    // N1: the gate-2 dialog is closed at the gate-1 state.
    assert!(!gate.contains(GATE2), "N1: gate 2 open before Right+Enter");

    // A2: Right+Enter opens gate 2, Esc cancels it, twice.
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
    eprintln!("h4 parity_platforms_linux: boot, gate roundtrip x2");
}

/// PARITY-TASK-INPUT-001 (`holla/parity/task-input`): the task-input
/// parity boot plus the deploy-prompt behavior.
///
/// Flow: boot, type `Deploy the release`, Enter (password prompt),
/// Esc (back), Esc (query cleared), type + Enter (prompt again).
/// The prompt is never answered; nothing runs past the wait.
#[test]
#[ignore = "holla h4 check; run with --ignored"]
fn h4_parity_task_input() {
    const PASSWORD: &str = "Password:";
    let dir = h4_dir("h4_parity_task_input");
    let case = h4_case_t("parity_task_input", TASKINPUT, BOOT, 20_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    // The suggestion rows animate in after the header; settle them
    // before the boot asserts so V1/V2 cannot race the stagger.
    waits::wait_state(&mut s, "Run the stubborn worker", "suggestions settled");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the top row pairs the journal tail with its restart state.
    assert_line_has(&boot, "› Follow payments-worker", "restarting", "V1 row");
    // V2: the custom shelf lists the fixture scripts.
    assert!(
        boot.contains("Run the stubborn worker"),
        "V2: no custom shelf"
    );
    // S1: the facts scope the journal to the device.
    assert_line_has(&boot, "Scope", "on devbox", "S1 scope");

    // A1: Enter runs the deploy to the password prompt.
    support::drive(
        &mut s,
        &["type:Deploy the release", "enter", "wait:Password:"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-password");
    let password = live_text(&mut s);

    // S2: the title marks the run waiting for input.
    assert!(
        password.contains("waiting for input"),
        "S2: no waiting title"
    );
    // N1: the empty-query placeholder is gone on the output.
    assert!(!password.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: Esc backs to the finder, Esc clears the retained query.
    support::press_step(&s, "escape");
    wait_absent(&mut s, PASSWORD, "output left");
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-finder");
    let finder = live_text(&mut s);
    // N2: the password prompt is gone after backing out.
    assert!(!finder.contains(PASSWORD), "N2: password prompt stuck");
    support::drive(
        &mut s,
        &["type:Deploy the release", "enter", "wait:Password:"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-password-again");
    let back = live_text(&mut s);
    assert!(back.contains("waiting for input"), "prompt intact");
    eprintln!("h4 parity_task_input: boot, prompt roundtrip");
}
