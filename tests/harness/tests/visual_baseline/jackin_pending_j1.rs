//! Jackin pending-roots slice 8A-J1 executable checks (VB Phase-8g).
//!
//! One ignored PTY test per J1 registry row (30 rows), over the real
//! `jackin-preview` binary built from this worktree's verified VB sources
//! (`env!("CARGO_BIN_EXE_jackin-preview")`, the harness `[[bin]]` compiled
//! from `../../src/bin/jackin_preview/main.rs`). Each test drives the row's
//! specified inputs and asserts its visual (V), state (S), action (A), and
//! negative (N) checks in live-PTY executable form:
//!
//! - V: live needles plus same-line coexistence for tab strips, inspector
//!   rows, dialog action rows, and picker rows.
//! - S: live labels (cursor/focus/footer/crumb rows) plus boot-vs-final
//!   comparisons captured from the same session.
//! - A: behavior probes — the specified gesture runs and the specified
//!   advance (or boundary refusal) is observed; `Changed`/`Consumed`/
//!   `Ignored` outcomes are proven by their screen correlates (moved,
//!   byte-identical), since PTY cannot see the enum.
//! - N: absence assertions, each paired with a presence proof in the same
//!   test (boot checkpoint, mid-flow checkpoint, or pre/post transition)
//!   so no absence passes vacuously.
//!
//! No new snapshots and no new static captures: all 30 roots already have
//! approved frames gated cell-exact by the ported matrices in `jackin.rs`,
//! `jackin_journeys.rs`, and `audit.rs` (plan §Reconciliation: "rows plus
//! checks, not recaptures"), so every case here uses owned (dynamic) names
//! and stays out of the `snapshots/` inventory. No isolated-component
//! captures: every row's `requires` set (key-injection, glyph-capture) is
//! PTY-level, and every assertion has a live-PTY executable form.
//!
//! Per-test scratch lands under this harness crate's
//! `target/tuiscotti/journeys/<test>/` (gitignored): the row's named
//! checkpoint(s) as text frames, `00-boot.txt` where the row needs a boot
//! comparison, and `provenance.txt` (binary path, size, mtime, argv).
//! Typed input is synthetic and in-memory only (simulation data); no test
//! launches a real container, logs into an account, or saves anything:
//! both save flows cancel their preview dialogs, the failure flow only
//! acknowledges the frozen-failure dialog, and every world is the fixture
//! simulation.
//!
//! Row → test map (registry id → `j1_*` test):
//!
//! - INSPECT-ACCOUNT-003 → [`j1_accounts_detail`]
//! - INSPECT-DRAWER-004 → [`j1_accounts_drawer`]
//! - FLIST-ACCOUNT-003 → [`j1_accounts_filter`]
//! - PANEL-AUDIT-006 → [`j1_audit_accounts_boot`]
//! - FORM-EDITOR-005 → [`j1_editor_general`]
//! - FORM-EDITOR-006 → [`j1_editor_mounts`]
//! - FORM-EDITOR-007 → [`j1_editor_mounts_dirty`]
//! - FORM-EDITOR-008 → [`j1_editor_env`]
//! - FORM-EDITOR-009 → [`j1_editor_auth`]
//! - FORM-EDITOR-010 → [`j1_editor_roles`]
//! - FORM-EDITOR-011 → [`j1_editor_save_preview`]
//! - PANEL-INTRO-007 → [`j1_intro_f300`]
//! - PANEL-INTRO-008 → [`j1_intro_f400`]
//! - PANEL-INTRO-009 → [`j1_intro_skipped`]
//! - INSPECT-MANAGER-005 → [`j1_manager_detail_drawer`]
//! - VP-ATTACH-003 → [`j1_manager_hard_launch_locked`]
//! - PICKER-HARDLAUNCH-005 → [`j1_manager_hard_launch_picker`]
//! - PICKER-LAUNCH-004 → [`j1_manager_launch_picker`]
//! - TREE-MANAGER-007 → [`j1_manager_tree_expanded`]
//! - PANEL-SCENARIO-010 → [`j1_scenarios_first_use`]
//! - PANEL-SCENARIO-011 → [`j1_scenarios_hard_cases`]
//! - DIALOG-FAILURE-008 → [`j1_scenarios_launch_failure`]
//! - PANEL-SCENARIO-012 → [`j1_scenarios_launch_running`]
//! - PANEL-SCENARIO-014 → [`j1_scenarios_outro_last`]
//! - PANEL-SCENARIO-013 → [`j1_scenarios_returning`]
//! - FORM-SETTINGS-014 → [`j1_settings_agents`]
//! - FORM-SETTINGS-013 → [`j1_settings_mounts`]
//! - FORM-SETTINGS-012 → [`j1_settings_route`]
//! - FORM-SETTINGS-016 → [`j1_settings_save_preview`]
//! - FORM-SETTINGS-015 → [`j1_settings_trust`]

use std::path::{Path, PathBuf};
use std::time::Duration;

use tuiscotti::tui::{CancelToken, Session};

use crate::support::state_waits as waits;
use crate::support::{self, Case, Color, JACKIN};

/// Boot needle on chrome routes: the brand.
const BOOT: &str = "jackin❯";
/// Boot needle on the frozen failure: the dialog title.
const BOOT_FAILURE: &str = "Launch failed";
/// Boot needle on wordless rain screens (intro/outro warp): the footer.
const BOOT_SKIP: &str = "Enter Skip";

const RETURNING: &[&str] = &[
    "--scenario",
    "returning",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const ACCOUNTS: &[&str] = &[
    "--scenario",
    "accounts-mixed",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const LAUNCH: &[&str] = &[
    "--scenario",
    "launch-running",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const FAILURE: &[&str] = &[
    "--scenario",
    "launch-failure",
    "--motion",
    "paused",
    "--frame",
    "240",
];
const HARD: &[&str] = &[
    "--scenario",
    "hard-cases",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const FIRSTUSE40: &[&str] = &[
    "--scenario",
    "first-use",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const FIRSTUSE300: &[&str] = &[
    "--scenario",
    "first-use",
    "--motion",
    "paused",
    "--frame",
    "300",
];
const FIRST400: &[&str] = &[
    "--scenario",
    "first-use",
    "--motion",
    "paused",
    "--frame",
    "400",
];
const OUTRO40: &[&str] = &[
    "--scenario",
    "outro-last",
    "--motion",
    "paused",
    "--frame",
    "40",
];

/// Editor crumb on the payments-platform workspace.
const CRUMB_EDIT: &str = "Workspaces › payments-platform › edit";
/// Manager tree footer marker (tree focus, not drawer/modal).
const TREE_FOOTER: &str = "Tab Details";
/// Drawer/modal footer marker (detail focus).
const DRAWER_FOOTER: &str = "Tab Actions";
/// Launch picker dialog title.
const PICKER_TITLE: &str = "Launch · choose Agent";
/// New-workspace prelude step-1 needle.
const PRELUDE_S1: &str = "step 1 of 5 · Source";

/// Owned-name J1 case at the canonical 120x40 truecolor geometry.
fn j1_case(slug: &str, args: &'static [&'static str], boot: &'static str) -> Case {
    Case::dynamic(
        format!("journeys/jackin/j1/{slug}"),
        JACKIN,
        args,
        120,
        40,
        Color::Truecolor,
        boot,
    )
}

/// Unique scratch dir for one J1 test (created, never shared).
fn j1_dir(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/tuiscotti/journeys")
        .join(test);
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("mkdir {}: {e}", dir.display()));
    dir
}

/// Record the binary path + identity: absolute path, byte size, mtime, argv.
/// Written to `provenance.txt`, echoed too (the build-identity record).
fn write_provenance(dir: &Path, case: &Case) {
    write_provenance_named(dir, case, "provenance.txt");
}

/// [`write_provenance`] with an explicit filename, for multi-session tests
/// where each session's argv deserves its own record.
fn write_provenance_named(dir: &Path, case: &Case, file: &str) {
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
    std::fs::write(dir.join(file), &body)
        .unwrap_or_else(|e| panic!("write {}: {e}", dir.join(file).display()));
    eprintln!("j1 provenance ({file}): {body}");
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

/// Wait for the session to exit cleanly and return the exit-frame text.
/// Used for the outro dismiss path (the outro quits the session).
fn expect_exit_text(s: &Session, timeout: Duration, what: &str) -> String {
    let cancel = CancelToken::new();
    let exit = s
        .expect_exit(std::time::Instant::now() + timeout, &cancel)
        .unwrap_or_else(|e| panic!("{what}: session never exited: {e:#}"));
    let obs = exit
        .success()
        .unwrap_or_else(|e| panic!("{what}: session did not exit cleanly: {e:#}"));
    support::screen_text(&obs.screen)
}

/// Some line must contain both `needle` and `also` (the same-row
/// coexistence proof: tab rows, inspector rows, dialog action rows).
fn assert_line_has(text: &str, needle: &str, also: &str, what: &str) {
    assert!(
        text.lines().any(|l| l.contains(needle) && l.contains(also)),
        "{what}: no line contains both `{needle}` and `{also}`"
    );
}

/// No line containing `needle` may contain `also` (region-scoped absence).
fn assert_line_lacks(text: &str, needle: &str, also: &str, what: &str) {
    for line in text.lines().filter(|l| l.contains(needle)) {
        assert!(
            !line.contains(also),
            "{what}: `{needle}` line must not contain `{also}`: {line:?}"
        );
    }
}

/// INSPECT-ACCOUNT-003 (`jackin/accounts/detail`): account inspector body
/// behind a tree selection.
///
/// Flow: boot, down×4 (Work selected), Enter (drawer opens), Esc (tree
/// back), a (add-form presence proof), Esc (form cancelled).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_accounts_detail() {
    const CRUMB: &str = "Accounts › Claude › Work";
    let dir = j1_dir("j1_accounts_detail");
    let case = j1_case("accounts_detail", ACCOUNTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
        &mut s,
        &["down", "down", "down", "down", &format!("wait:{CRUMB}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-detail");
    let detail = live_text(&mut s);

    // V1: inspector header names the selected account and its exhaustion.
    assert!(detail.contains(CRUMB), "V1: crumb lacks `{CRUMB}`");
    assert_line_has(&detail, "Claude · Work", "exhausted", "V1 header");
    // V2: provider rows plus the stale quota block.
    assert_line_has(&detail, "Provider", "Anthropic / Claude", "V2 provider");
    assert_line_has(&detail, "Quota", "stale · last good 47 min ago", "V2 quota");
    // S1: the tree cursor sits on the Work row.
    assert_line_has(&detail, "▎", "Work", "S1 cursor");
    // S2: tree footer (also the N1 presence proof).
    assert!(detail.contains("Enter Details"), "S2: no Enter Details");
    assert!(detail.contains("m Usage"), "S2: no Usage hint");

    // A1: Enter focuses the inspector drawer.
    support::drive(
        &mut s,
        &["enter", &format!("wait:{DRAWER_FOOTER}")],
        case_timeout(&case),
    );
    let drawer = live_text(&mut s);
    assert!(
        drawer.contains("▎Claude · Work"),
        "A1: drawer title unfocused"
    );
    // N1: the tree footer is gone while the drawer holds focus.
    assert!(
        !drawer.contains("Enter Details"),
        "N1: tree footer visible in drawer"
    );
    checkpoint(&mut s, &dir, "02-drawer");

    // A1 back: Esc returns focus to the tree.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DRAWER_FOOTER, "drawer closed");
    waits::wait_state(&mut s, "Enter Details", "tree footer back");

    // A2/N2: the add form opens on `a` and cancels on Esc; it was never
    // open at the detail state.
    support::drive(&mut s, &["a", "wait:New account"], case_timeout(&case));
    checkpoint(&mut s, &dir, "03-add-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "New account", "add form cancelled");
    let back = live_text(&mut s);
    assert!(
        !back.contains("New account"),
        "N2: add form stuck open after Esc"
    );
    assert_line_has(&back, "▎", "Work", "selection intact");
    checkpoint(&mut s, &dir, "04-tree");
    eprintln!("j1 accounts_detail: detail, drawer roundtrip, add-form roundtrip");
}

/// INSPECT-DRAWER-004 (`jackin/accounts/drawer`): focused inspector drawer
/// with its action row.
///
/// Flow: boot, down×4, Enter (drawer), Esc (tree), Enter (drawer again).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_accounts_drawer() {
    const CRUMB: &str = "Accounts › Claude › Work";
    let dir = j1_dir("j1_accounts_drawer");
    let case = j1_case("accounts_drawer", ACCOUNTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
        &mut s,
        &[
            "down",
            "down",
            "down",
            "down",
            "enter",
            &format!("wait:{DRAWER_FOOTER}"),
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-drawer");
    let drawer = live_text(&mut s);

    // V1: the focused inspector title plus the exhaustion badge.
    assert_line_has(&drawer, "▎Claude · Work", "exhausted", "V1 title");
    // V2: the redacted 1Password credential locator.
    assert!(
        drawer.contains("op://v_eng01/it_ant01/credential"),
        "V2: no op locator"
    );
    assert!(drawer.contains("••••••••…3c9e"), "V2: no redacted tail");
    // S1: drawer footer (also the N1 presence proof).
    assert!(drawer.contains(DRAWER_FOOTER), "S1: no Tab Actions");
    assert!(drawer.contains("Esc Back"), "S1: no Esc Back");
    // S2: the inspector action row.
    assert_line_has(&drawer, "Refresh", "Remove…", "S2 actions");

    // A1: Esc returns focus to the tree.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DRAWER_FOOTER, "drawer closed");
    waits::wait_state(&mut s, "Enter Details", "tree footer back");
    checkpoint(&mut s, &dir, "02-tree");
    let tree = live_text(&mut s);
    // N1/N2: drawer chrome is gone with the focus.
    assert!(!tree.contains(DRAWER_FOOTER), "N1: Tab Actions stuck");
    assert!(!tree.contains("▎Claude · Work"), "N2: drawer title stuck");

    // A2: Enter re-opens the drawer from the same selection.
    support::drive(
        &mut s,
        &["enter", &format!("wait:{DRAWER_FOOTER}")],
        case_timeout(&case),
    );
    waits::wait_state(&mut s, "▎Claude · Work", "drawer title refocused");
    assert!(live_text(&mut s).contains(CRUMB), "A2: crumb lost");
    checkpoint(&mut s, &dir, "03-drawer-again");
    eprintln!("j1 accounts_drawer: drawer, tree roundtrip, drawer again");
}

/// FLIST-ACCOUNT-003 (`jackin/accounts/filter`): applied tree filter with
/// its truncated title.
///
/// Flow: boot, / + Enter + type work + Enter (filter applied), Esc
/// (filter cleared), a (add-form presence proof), Esc (cancelled).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_accounts_filter() {
    const TITLE: &str = "Accounts · filt";
    let dir = j1_dir("j1_accounts_filter");
    let case = j1_case("accounts_filter", ACCOUNTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    // S2 first: the unfiltered tree (the N1/V2 presence proof).
    assert!(boot.contains("★ Personal"), "S2: boot lacks ★ Personal");

    support::drive(
        &mut s,
        &["/", "enter", "type:work", "enter", &format!("wait:{TITLE}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-filtered");
    let filtered = live_text(&mut s);

    // V1: the applied filter renders truncated in the tree title.
    assert!(filtered.contains(TITLE), "V1: no filtered title");
    assert!(filtered.contains("12 · 4 ▲ · 4 !"), "V1: no tree counts");
    // V2: only the Work match (plus parents) survives.
    assert!(filtered.contains("▾ Claude"), "V2: no Claude parent");
    assert_line_has(&filtered, "▎", "Overview", "V2 cursor");
    assert!(
        !filtered.contains("★ Personal"),
        "V2: Personal survives the work filter"
    );
    // S1: the filtered tree keeps its footer.
    assert!(filtered.contains("Enter Details"), "S1: no Enter Details");
    assert!(filtered.contains("/ Filter"), "S1: no Filter hint");

    // A1: Esc clears the filter and restores the full tree.
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, "★ Personal", "full tree restored");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);
    // N1: the filtered title is gone with the filter.
    assert!(!cleared.contains(TITLE), "N1: filtered title stuck");

    // A2/N2: the add form opens on `a` and cancels on Esc.
    support::drive(&mut s, &["a", "wait:New account"], case_timeout(&case));
    checkpoint(&mut s, &dir, "03-add-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "New account", "add form cancelled");
    assert!(
        !live_text(&mut s).contains("New account"),
        "N2: add form stuck open"
    );
    checkpoint(&mut s, &dir, "04-tree");
    eprintln!("j1 accounts_filter: filtered, cleared, add-form roundtrip");
}

/// PANEL-AUDIT-006 (`jackin/audit/accounts`): accounts route regions at
/// audit boot.
///
/// Flow: boot, Enter (drawer presence), Esc (tree), a (form presence),
/// Esc (cancelled).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_audit_accounts_boot() {
    let dir = j1_dir("j1_audit_accounts_boot");
    let case = j1_case("audit_accounts_boot", ACCOUNTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);

    // V1: the overview crumb plus the registry totals.
    assert!(
        boot.contains("Accounts › Overview"),
        "V1: no overview crumb"
    );
    assert!(
        boot.contains("12 accounts · 8 providers"),
        "V1: no registry totals"
    );
    // V2: the degraded health summary.
    assert_line_has(&boot, "Health", "degraded · 4 warnings", "V2 health");
    assert!(boot.contains("1 exhausted"), "V2: no exhausted count");
    // S1: the tree cursor sits on Overview.
    assert_line_has(&boot, "▎", "Overview", "S1 cursor");
    // S2: the accounts tree footer.
    assert!(boot.contains("Enter Details"), "S2: no Enter Details");
    assert!(boot.contains("r Refresh all"), "S2: no Refresh all");

    // A1/N1: Enter opens the drawer (presence), Esc closes it again.
    support::drive(
        &mut s,
        &["enter", &format!("wait:{DRAWER_FOOTER}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-drawer-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DRAWER_FOOTER, "drawer closed");
    waits::wait_state(&mut s, "Enter Details", "tree footer back");
    assert!(
        !live_text(&mut s).contains(DRAWER_FOOTER),
        "N1: Tab Actions stuck"
    );

    // A2/N2: the add form opens on `a` and cancels on Esc.
    support::drive(&mut s, &["a", "wait:New account"], case_timeout(&case));
    checkpoint(&mut s, &dir, "02-add-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "New account", "add form cancelled");
    let back = live_text(&mut s);
    assert!(!back.contains("New account"), "N2: add form stuck open");
    assert_line_has(&back, "▎", "Overview", "cursor intact");
    checkpoint(&mut s, &dir, "03-boot");
    eprintln!("j1 audit_accounts_boot: boot regions, drawer+form roundtrips");
}

/// FORM-EDITOR-005 (`jackin/editor/general`): workspace editor on General
/// with the tab strip focused.
///
/// Flow: boot, e (editor), 2 + Enter (Mounts body), Esc (TABS refocus),
/// Esc (clean leave to the manager).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_editor_general() {
    let dir = j1_dir("j1_editor_general");
    let case = j1_case("editor_general", RETURNING, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
        &mut s,
        &["e", &format!("wait:{CRUMB_EDIT}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-general");
    let general = live_text(&mut s);

    // V1: the editor crumb plus the General body.
    assert!(general.contains(CRUMB_EDIT), "V1: no editor crumb");
    assert!(general.contains("Name *"), "V1: no Name row");
    assert!(
        general.contains("payments-platform"),
        "V1: no workspace name"
    );
    // V2: the shipped General flags.
    assert!(general.contains("[✓] Keep awake"), "V2: no Keep awake");
    assert!(
        general.contains("[✓] Git pull before launch"),
        "V2: no Git pull"
    );
    // S1: the TABS footer (also the A2/N2 presence proof).
    assert!(general.contains("1–5 Jump"), "S1: no 1–5 Jump");
    assert!(general.contains("Enter Body"), "S1: no Enter Body");
    // S2: the footer action buttons.
    assert_line_has(&general, "Cancel", "Save…", "S2 buttons");

    // A1: 2 + Enter jumps to the Mounts body.
    support::drive(
        &mut s,
        &["2", "enter", "wait:+ Add mount"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-mounts");
    let mounts = live_text(&mut s);
    assert!(mounts.contains("r Read-only"), "A1: no mounts footer");
    // N1: the General body is gone.
    assert!(!mounts.contains("Keep awake"), "N1: General body stuck");

    // A2: Esc refocuses TABS without leaving, Esc leaves cleanly.
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, "1–5 Jump", "TABS refocused");
    assert!(
        live_text(&mut s).contains("+ Add mount"),
        "A2: mounts body lost on refocus"
    );
    checkpoint(&mut s, &dir, "03-tabs");
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, "Current directory", "manager back");
    checkpoint(&mut s, &dir, "04-manager");
    let manager = live_text(&mut s);
    // N2: the editor is gone with the leave.
    assert!(!manager.contains("› edit"), "N2: editor crumb stuck");
    assert!(manager.contains("Enter Launch"), "manager footer back");
    eprintln!("j1 editor_general: general, mounts jump, clean leave");
}

/// FORM-EDITOR-006 (`jackin/editor/mounts`): workspace editor on the Mounts
/// tab with its mode/isolation grid.
///
/// Flow: boot, e, 2 + Enter (Mounts body), [ (General), ] (Mounts).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_editor_mounts() {
    let dir = j1_dir("j1_editor_mounts");
    let case = j1_case("editor_mounts", RETURNING, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
        &mut s,
        &[
            "e",
            &format!("wait:{CRUMB_EDIT}"),
            "2",
            "enter",
            "wait:+ Add mount",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-mounts");
    let mounts = live_text(&mut s);

    // V1: the grid header plus the first mount row.
    assert_line_has(&mounts, "Destination", "Isolation", "V1 header");
    assert_line_has(
        &mounts,
        "/workspace/payments-platform",
        "worktree",
        "V1 first row",
    );
    // V2: the shipped modes (rw worktree plus ro shared).
    assert!(mounts.contains("/workspace/libs"), "V2: no libs row");
    assert_line_has(&mounts, "/workspace/libs", "ro", "V2 libs mode");
    // S1: the body cursor sits on the first row.
    assert!(mounts.contains("▎›"), "S1: no body cursor");
    // S2: the mounts body footer (also the N2 presence proof).
    assert!(mounts.contains("r Read-only"), "S2: no Read-only hint");
    assert!(mounts.contains("i Isolation"), "S2: no Isolation hint");

    // A1: [ jumps to General with TABS focused.
    support::drive(&mut s, &["[", "wait:Keep awake"], case_timeout(&case));
    checkpoint(&mut s, &dir, "02-general");
    let general = live_text(&mut s);
    assert!(general.contains("1–5 Jump"), "A1: no TABS footer");
    // N1/N2: the mounts body and its footer are gone.
    assert!(!general.contains("+ Add mount"), "N1: mounts body stuck");
    assert!(!general.contains("r Read-only"), "N2: mounts footer stuck");

    // A2: ] jumps back to Mounts.
    support::drive(&mut s, &["]", "wait:+ Add mount"], case_timeout(&case));
    waits::wait_state(&mut s, "1–5 Jump", "TABS footer persists");
    assert!(
        live_text(&mut s).contains("/workspace/payments-platform"),
        "A2: mounts body lost"
    );
    checkpoint(&mut s, &dir, "03-mounts");
    eprintln!("j1 editor_mounts: mounts, general jump, mounts back");
}

/// FORM-EDITOR-007 (`jackin/editor/mounts_dirty`): dirty mounts tab with
/// its change badge and row marker.
///
/// Flow: boot, e, 2 + Enter, r + i (dirty), Esc + Esc (exit dialog),
/// Esc (dialog cancelled).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_editor_mounts_dirty() {
    let dir = j1_dir("j1_editor_mounts_dirty");
    let case = j1_case("editor_mounts_dirty", RETURNING, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
        &mut s,
        &[
            "e",
            &format!("wait:{CRUMB_EDIT}"),
            "2",
            "enter",
            "wait:+ Add mount",
            "r",
            "i",
            "wait:• 1 change",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-dirty");
    let dirty = live_text(&mut s);

    // V1: the header badge plus the dotted tab.
    assert!(dirty.contains("• 1 change"), "V1: no change badge");
    assert!(dirty.contains("Mounts •"), "V1: no dotted tab");
    // V2: the toggled first row (rw→ro, worktree→clone).
    assert_line_has(&dirty, "/workspace/payments-platform", "clone", "V2 clone");
    assert_line_has(&dirty, "› •", "/workspace/payments-platform", "V2 marker");
    // S1: the row-hint footer names the new isolation.
    assert!(dirty.contains("isolation clone"), "S1: no isolation hint");
    // S2: the second row is untouched.
    assert_line_has(&dirty, "/workspace/libs", "shared", "S2 libs intact");

    // A1: Esc refocuses TABS, Esc attempts the dirty leave.
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, "1–5 Jump", "TABS refocused");
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, "Unsaved changes", "exit dialog opened");
    checkpoint(&mut s, &dir, "02-exit-dialog");
    let dialog = live_text(&mut s);
    assert!(
        dialog.contains("Save changes before leaving?"),
        "A1: no exit question"
    );
    for button in ["Cancel", "Discard", "Save"] {
        assert!(dialog.contains(button), "A1: no {button} button");
    }

    // A2/N1/N2: Esc cancels the exit; the dialog (and its Discard
    // action) is gone, the edits are intact, nothing was saved.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Unsaved changes", "exit dialog cancelled");
    checkpoint(&mut s, &dir, "03-dirty");
    let back = live_text(&mut s);
    assert!(!back.contains("Unsaved changes"), "N1: exit dialog stuck");
    assert!(!back.contains("Discard"), "N2: Discard action stuck");
    assert!(back.contains("• 1 change"), "edits lost on cancel");
    assert_line_has(&back, "/workspace/payments-platform", "clone", "row lost");
    eprintln!("j1 editor_mounts_dirty: dirty, exit dialog, cancelled intact");
}

/// FORM-EDITOR-008 (`jackin/editor/env`): workspace editor on the
/// Environments tab with masked values and role overrides.
///
/// Flow: boot, e, 4 + Enter (Environments body), [ (Roles), ] (back).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_editor_env() {
    let dir = j1_dir("j1_editor_env");
    let case = j1_case("editor_env", RETURNING, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
        &mut s,
        &[
            "e",
            &format!("wait:{CRUMB_EDIT}"),
            "4",
            "enter",
            "wait:DATABASE_URL",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-env");
    let env = live_text(&mut s);

    // V1: the workspace section plus the masked first var.
    assert!(env.contains("4 vars"), "V1: no var count");
    assert_line_has(&env, "DATABASE_URL", "plain", "V1 first var");
    // V2: the host-env passthrough plus the 1Password row.
    assert_line_has(&env, "GH_TOKEN", "host env", "V2 host env");
    assert!(
        env.contains("[op] Engineering › Stripe"),
        "V2: no 1Password row"
    );
    // S1: the role-override section.
    assert!(env.contains("Role overrides"), "S1: no overrides section");
    assert!(env.contains("▾ Role: backend"), "S1: no backend role");
    // S2: the env body footer (also the N2 presence proof).
    assert!(env.contains("p 1Password"), "S2: no 1Password hint");
    assert!(env.contains("s Scope"), "S2: no Scope hint");

    // A1: [ jumps to Roles with TABS focused.
    support::drive(&mut s, &["[", "wait:Allowed roles"], case_timeout(&case));
    checkpoint(&mut s, &dir, "02-roles");
    let roles = live_text(&mut s);
    assert!(roles.contains("1–5 Jump"), "A1: no TABS footer");
    // N1/N2: the env body and its footer are gone.
    assert!(!roles.contains("DATABASE_URL"), "N1: env body stuck");
    assert!(!roles.contains("p 1Password"), "N2: env footer stuck");

    // A2: ] jumps back to Environments.
    support::drive(&mut s, &["]", "wait:DATABASE_URL"], case_timeout(&case));
    assert!(
        live_text(&mut s).contains("Role overrides"),
        "A2: env body lost"
    );
    checkpoint(&mut s, &dir, "03-env");
    eprintln!("j1 editor_env: env, roles jump, env back");
}

/// FORM-EDITOR-009 (`jackin/editor/auth`): workspace editor on the Accounts
/// tab with per-provider enable rows.
///
/// Flow: boot, e, 5 + Enter (Accounts body), [ (Environments), ] (back).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_editor_auth() {
    let dir = j1_dir("j1_editor_auth");
    let case = j1_case("editor_auth", RETURNING, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
        &mut s,
        &[
            "e",
            &format!("wait:{CRUMB_EDIT}"),
            "5",
            "enter",
            "wait:Active accounts",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-auth");
    let auth = live_text(&mut s);

    // V1: the effective-accounts header.
    assert!(auth.contains("Active accounts"), "V1: no Active accounts");
    assert!(
        auth.contains("5 effective · 4 inherited · 1 enabled here"),
        "V1: no effective counts"
    );
    // V2: the here-enabled Work row plus a globally-disabled row.
    assert_line_has(&auth, "[✓] Work", "enabled here", "V2 Work row");
    assert_line_has(
        &auth,
        "Archived contractor laptop",
        "disabled globally",
        "V2 disabled row",
    );
    // S1: the registry pointer.
    assert!(
        auth.contains("registry in Accounts (c)"),
        "S1: no registry hint"
    );
    // S2: the auth body footer (also the N2 presence proof).
    assert!(auth.contains("p Prefer"), "S2: no Prefer hint");
    assert!(auth.contains("/ Filter"), "S2: no Filter hint");

    // A1: [ jumps to Environments with TABS focused.
    support::drive(&mut s, &["[", "wait:DATABASE_URL"], case_timeout(&case));
    checkpoint(&mut s, &dir, "02-env");
    let env = live_text(&mut s);
    assert!(env.contains("1–5 Jump"), "A1: no TABS footer");
    // N1/N2: the auth body and its footer are gone.
    assert!(!env.contains("Active accounts"), "N1: auth body stuck");
    assert!(!env.contains("p Prefer"), "N2: auth footer stuck");

    // A2: ] jumps back to Accounts.
    support::drive(&mut s, &["]", "wait:Active accounts"], case_timeout(&case));
    assert!(
        live_text(&mut s).contains("5 effective"),
        "A2: auth body lost"
    );
    checkpoint(&mut s, &dir, "03-auth");
    eprintln!("j1 editor_auth: auth, env jump, auth back");
}

/// FORM-EDITOR-010 (`jackin/editor/roles`): workspace editor on the Roles
/// tab with the allow-list and default marker.
///
/// Flow: boot, e, 3 + Enter (Roles body), [ (Mounts), ] (back).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_editor_roles() {
    let dir = j1_dir("j1_editor_roles");
    let case = j1_case("editor_roles", RETURNING, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
        &mut s,
        &[
            "e",
            &format!("wait:{CRUMB_EDIT}"),
            "3",
            "enter",
            "wait:Allowed roles",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-roles");
    let roles = live_text(&mut s);

    // V1: the allow-list header plus the default.
    assert!(roles.contains("Allowed roles"), "V1: no Allowed roles");
    assert!(roles.contains("3 of 46"), "V1: no allow counts");
    assert!(roles.contains("default ★ the-architect"), "V1: no default");
    // V2: an allowed row plus the trust-blocked row.
    assert_line_has(
        &roles,
        "[✓] the-architect ★",
        "Full-stack design",
        "V2 architect row",
    );
    assert_line_has(
        &roles,
        "[ ] data-eng",
        "! load error · trust required",
        "V2 blocked row",
    );
    // S1: the body cursor sits on the first row.
    assert!(roles.contains("▎›"), "S1: no body cursor");
    // S2: the roles body footer (also the N2 presence proof).
    assert!(roles.contains("Space Allow"), "S2: no Allow hint");
    assert!(
        roles.contains("Enter Set default"),
        "S2: no Set default hint"
    );

    // A1: [ jumps to Mounts with TABS focused.
    support::drive(&mut s, &["[", "wait:+ Add mount"], case_timeout(&case));
    checkpoint(&mut s, &dir, "02-mounts");
    let mounts = live_text(&mut s);
    assert!(mounts.contains("1–5 Jump"), "A1: no TABS footer");
    // N1/N2: the roles body and its footer are gone.
    assert!(!mounts.contains("Allowed roles"), "N1: roles body stuck");
    assert!(
        !mounts.contains("Enter Set default"),
        "N2: roles footer stuck"
    );

    // A2: ] jumps back to Roles.
    support::drive(&mut s, &["]", "wait:Allowed roles"], case_timeout(&case));
    assert!(live_text(&mut s).contains("3 of 46"), "A2: roles body lost");
    checkpoint(&mut s, &dir, "03-roles");
    eprintln!("j1 editor_roles: roles, mounts jump, roles back");
}

/// FORM-EDITOR-011 (`jackin/editor/save_preview`): the save preview dialog
/// with its diff line — the deterministic save boundary.
///
/// Flow: boot, e, down×3 + space (dirty), ctrl-s (preview), Esc
/// (cancelled), ctrl-s (reopened), Esc (cancelled again). Nothing is
/// saved: both previews are cancelled, not confirmed.
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_editor_save_preview() {
    const DIALOG: &str = "Save workspace";
    const DIFF: &str = "~ keep_awake true → false";
    let dir = j1_dir("j1_editor_save_preview");
    let case = j1_case("editor_save_preview", RETURNING, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
        &mut s,
        &[
            "e",
            &format!("wait:{CRUMB_EDIT}"),
            "down",
            "down",
            "down",
            "space",
            "wait:• 1 change",
        ],
        case_timeout(&case),
    );
    support::press_step(&s, "ctrl-s");
    waits::wait_state(&mut s, DIALOG, "save preview open");
    waits::wait_state(&mut s, DIFF, "preview diff line");
    checkpoint(&mut s, &dir, "01-preview");
    let preview = live_text(&mut s);

    // V1: the dialog names the workspace, scope, and change count.
    assert!(preview.contains(DIALOG), "V1: no dialog title");
    assert!(
        preview.contains("payments-platform"),
        "V1: no workspace name"
    );
    assert!(preview.contains("1 change"), "V1: no change count");
    // V2: the keep_awake diff line.
    assert!(preview.contains(DIFF), "V2: no diff line");
    // S1: the dirty badge plus the dotted tab behind the dialog.
    assert!(preview.contains("• 1 change"), "S1: no change badge");
    assert!(preview.contains("General •"), "S1: no dotted tab");
    // S2: Cancel holds the dialog focus.
    assert_line_has(&preview, "▎Cancel", "Save", "S2 actions");

    // A1: Esc cancels the preview; the edits stay intact.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DIALOG, "preview cancelled");
    waits::wait_state(&mut s, "Not saved · keep editing", "cancel status");
    waits::wait_state(&mut s, "• 1 change", "edits intact");
    checkpoint(&mut s, &dir, "02-cancelled");

    // A2: ctrl-s reopens the preview (the dirt is intact), Esc cancels.
    support::press_step(&s, "ctrl-s");
    waits::wait_state(&mut s, DIALOG, "preview reopened");
    waits::wait_state(&mut s, DIFF, "diff line back");
    checkpoint(&mut s, &dir, "03-reopened");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DIALOG, "preview cancelled again");
    checkpoint(&mut s, &dir, "04-cancelled");
    let back = live_text(&mut s);
    // N1/N2: the dialog and its diff line are gone; the badge persists.
    assert!(!back.contains(DIALOG), "N1: preview stuck open");
    assert!(!back.contains(DIFF), "N2: diff line stuck");
    assert!(back.contains("• 1 change"), "edits lost on cancel");
    assert!(back.contains(CRUMB_EDIT), "editor left on cancel");
    eprintln!("j1 editor_save_preview: preview, cancel, reopen, cancel");
}

/// PANEL-INTRO-007 (`jackin/intro/f300`): the intro rain screen at frame
/// 300 with its skip-only footer.
///
/// Frame 300 sits in the warp phase (past WARP_START), so one Enter
/// finishes the intro — the two-step skip only applies to the phrases
/// phase. Flow: boot frame 300, Enter (skip to the empty manager),
/// n (new-workspace prelude).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_intro_f300() {
    let dir = j1_dir("j1_intro_f300");
    let case = j1_case("intro_f300", FIRSTUSE300, BOOT_SKIP);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-rain");
    let rain = live_text(&mut s);

    // V1: the skip-only footer.
    assert!(rain.contains("Enter Skip"), "V1: no skip footer");
    // V2: the rain field covers the screen (deterministic at frame 300).
    for glyph in ["─", "+", "·"] {
        assert!(rain.contains(glyph), "V2: no rain glyph `{glyph}`");
    }
    let first = rain.lines().next().unwrap_or_default();
    assert!(!first.trim().is_empty(), "V2: top line blank");
    // S1: no host menubar on the intro route (01-manager proves File).
    assert!(!rain.contains("File"), "S1: menubar on intro");
    // S2: no workspace tree on the intro route (01-manager proves it).
    assert!(!rain.contains("Current directory"), "S2: tree on intro");

    // A1: one Enter finishes the warp-phase intro (a second Enter would
    // leak into the manager and open the launch picker).
    support::drive(
        &mut s,
        &["enter", "wait:Current directory"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-manager");
    let manager = live_text(&mut s);
    assert!(
        manager.contains("+ New workspace"),
        "A1: no New workspace row"
    );
    assert!(
        manager.contains("no instances"),
        "A1: instances in fresh world"
    );
    assert!(manager.contains("File"), "A1: no menubar (S1 presence)");
    assert!(manager.contains("Workspaces"), "A1: no workspace chrome");
    assert!(!manager.contains(PICKER_TITLE), "A1: picker leaked open");
    // N2: the skip footer is gone with the skip.
    assert!(!manager.contains("Enter Skip"), "N2: skip footer stuck");

    // A2: n opens the new-workspace prelude from the empty manager.
    support::drive(
        &mut s,
        &["n", &format!("wait:{PRELUDE_S1}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-prelude");
    let prelude = live_text(&mut s);
    assert!(
        !prelude.contains("Current directory"),
        "A2: manager tree visible in prelude"
    );
    // N1: the tree rows are gone under the prelude (the header status
    // stays visible above it).
    assert!(!prelude.contains("+ New workspace"), "N1: tree rows stuck");
    eprintln!("j1 intro_f300: rain, skip to manager, prelude");
}

/// PANEL-INTRO-008 (`jackin/intro/f400`): the settled empty manager at
/// frame 400 (the intro finished on its own clock).
///
/// Flow: boot frame 400, n (prelude), Esc (prelude cancelled).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_intro_f400() {
    let dir = j1_dir("j1_intro_f400");
    let case = j1_case("intro_f400", FIRST400, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-manager");
    let manager = live_text(&mut s);

    // V1: the empty workspace tree.
    assert!(manager.contains("Workspaces"), "V1: no Workspaces chrome");
    assert!(manager.contains("no instances"), "V1: no empty status");
    assert!(
        manager.contains("+ New workspace"),
        "V1: no New workspace row"
    );
    // V2: the Current directory detail with its mount note.
    assert!(manager.contains("Current directory"), "V2: no tree root");
    assert!(manager.contains("not saved"), "V2: no not-saved badge");
    assert!(
        manager.contains("Create a workspace from this directory."),
        "V2: no empty hint"
    );
    // S1: the tree cursor sits on Current directory.
    assert_line_has(&manager, "▎", "Current directory", "S1 cursor");
    // S2: the empty-manager footer.
    assert!(manager.contains("Enter Launch"), "S2: no Launch hint");
    assert!(manager.contains("n New"), "S2: no New hint");

    // A1: n opens the new-workspace prelude.
    support::drive(
        &mut s,
        &["n", &format!("wait:{PRELUDE_S1}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-prelude");
    let prelude = live_text(&mut s);
    // N1: the manager tree is gone under the prelude.
    assert!(!prelude.contains("Current directory"), "N1: tree stuck");

    // A2/N2: Esc cancels the prelude; nothing was created.
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, "Cancelled · nothing created", "prelude cancelled");
    waits::wait_state(&mut s, "Current directory", "manager back");
    checkpoint(&mut s, &dir, "02-manager");
    assert!(
        !live_text(&mut s).contains(PRELUDE_S1),
        "N2: prelude stuck open"
    );
    eprintln!("j1 intro_f400: settled manager, prelude roundtrip");
}

/// PANEL-INTRO-009 (`jackin/intro/skipped`): the empty manager after the
/// two-step intro skip.
///
/// Flow: boot frame 40 (intro), Enter + Enter (skip), n (prelude),
/// Esc (prelude cancelled).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_intro_skipped() {
    let dir = j1_dir("j1_intro_skipped");
    let case = j1_case("intro_skipped", FIRSTUSE40, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-intro");
    let intro = live_text(&mut s);
    // The N1 presence proof: the boot frame is the intro itself.
    assert!(intro.contains("Enter Skip"), "boot is not the intro");

    support::drive(
        &mut s,
        &["enter", "enter", "wait:Current directory"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-manager");
    let manager = live_text(&mut s);

    // V1: the skipped-to empty manager.
    assert!(manager.contains("Workspaces"), "V1: no Workspaces chrome");
    assert!(manager.contains("no instances"), "V1: no empty status");
    assert!(
        manager.contains("+ New workspace"),
        "V1: no New workspace row"
    );
    // V2: the Current directory detail.
    assert!(manager.contains("not saved"), "V2: no not-saved badge");
    assert!(
        manager.contains("Enter launches with defaults"),
        "V2: no launch hint"
    );
    // S1: the tree cursor sits on Current directory.
    assert_line_has(&manager, "▎", "Current directory", "S1 cursor");
    // S2: the empty-manager footer.
    assert!(manager.contains("Enter Launch"), "S2: no Launch hint");
    assert!(manager.contains("q Quit"), "S2: no Quit hint");
    // N1: the intro is gone with the skip.
    assert!(!manager.contains("Enter Skip"), "N1: intro footer stuck");

    // A1: n opens the new-workspace prelude.
    support::drive(
        &mut s,
        &["n", &format!("wait:{PRELUDE_S1}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-prelude");
    // A2/N2: Esc cancels the prelude; nothing was created.
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, "Cancelled · nothing created", "prelude cancelled");
    waits::wait_state(&mut s, "Current directory", "manager back");
    checkpoint(&mut s, &dir, "02-manager");
    assert!(
        !live_text(&mut s).contains(PRELUDE_S1),
        "N2: prelude stuck open"
    );
    eprintln!("j1 intro_skipped: skip to manager, prelude roundtrip");
}

/// INSPECT-MANAGER-005 (`jackin/manager/detail_drawer`): the focused
/// workspace detail drawer.
///
/// Flow: boot, Tab (drawer), Esc (tree), e (editor presence), Esc (back).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_manager_detail_drawer() {
    let dir = j1_dir("j1_manager_detail_drawer");
    let case = j1_case("manager_detail_drawer", RETURNING, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
        &mut s,
        &["tab", &format!("wait:{DRAWER_FOOTER}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-drawer");
    let drawer = live_text(&mut s);

    // V1: the focused detail title plus the saved badge.
    assert_line_has(
        &drawer,
        "▎Current directory · payments-platform",
        "saved workspace",
        "V1 title",
    );
    // V2: the working-dir row plus the daemon instance rows.
    assert_line_has(
        &drawer,
        "Working dir",
        "/workspace/payments-platform",
        "V2 workdir",
    );
    assert_line_has(&drawer, "◉ 7f3a", "running", "V2 running row");
    // S1: the drawer footer (also the N1 presence proof).
    assert!(drawer.contains("↑↓ Move"), "S1: no Move hint");
    assert!(drawer.contains("Esc Back"), "S1: no Esc Back");
    // S2: the detail action row.
    assert_line_has(&drawer, "Launch", "Edit", "S2 actions");

    // A1: Esc returns focus to the tree.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DRAWER_FOOTER, "drawer closed");
    waits::wait_state(&mut s, TREE_FOOTER, "tree footer back");
    checkpoint(&mut s, &dir, "02-tree");
    let tree = live_text(&mut s);
    // N1: the drawer footer is gone with the focus.
    assert!(!tree.contains(DRAWER_FOOTER), "N1: Tab Actions stuck");
    assert!(tree.contains("Enter Launch"), "tree footer back");

    // A2/N2: e opens the editor (presence), Esc leaves it cleanly again.
    support::drive(
        &mut s,
        &["e", &format!("wait:{CRUMB_EDIT}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-editor-presence");
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, TREE_FOOTER, "manager back");
    assert!(
        !live_text(&mut s).contains("› edit"),
        "N2: editor crumb stuck"
    );
    checkpoint(&mut s, &dir, "04-tree");
    eprintln!("j1 manager_detail_drawer: drawer, tree, editor roundtrip");
}

/// VP-ATTACH-003 (`jackin/manager/hard_launch_locked`): capsule attach to
/// the daemon-deaf running instance in the degraded world.
///
/// Flow: boot hard-cases, Enter (picker presence), Esc (cancelled),
/// down×5 + space + down + Enter (attach), ctrl-b d (detach).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_manager_hard_launch_locked() {
    let dir = j1_dir("j1_manager_hard_launch_locked");
    let case = j1_case("manager_hard_launch_locked", HARD, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    // Picker presence proof (for N2): Enter opens it from the tree.
    support::drive(
        &mut s,
        &["enter", &format!("wait:{PICKER_TITLE}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "00-picker");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, PICKER_TITLE, "picker cancelled");
    waits::wait_state(&mut s, "Enter Launch", "tree footer back");

    // Attach to jk-e0e0, the running data-pipeline instance whose daemon
    // never answers; wait for the focus-corrected menubar.
    support::drive(
        &mut s,
        &[
            "down",
            "down",
            "down",
            "down",
            "down",
            "space",
            "down",
            "enter",
            "wait:▎ File",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-attached");
    let attached = live_text(&mut s);

    // V1: the capsule menubar plus the attached instance id.
    assert!(attached.contains("▎ File"), "V1: no focused menubar");
    assert!(
        attached.contains("jackin-data-pipeline-e0e0"),
        "V1: no instance id"
    );
    // V2: the attach status.
    assert_line_has(
        &attached,
        "Attached to",
        "tabs and panes restored",
        "V2 attach",
    );
    // S1: the shell status row.
    assert!(attached.contains("main"), "S1: no branch");
    assert!(
        attached.contains("no account · shell"),
        "S1: no shell status"
    );
    // S2: the capsule footer.
    assert!(attached.contains("Ctrl+B Prefix"), "S2: no Prefix hint");
    assert!(attached.contains("F10 Menu"), "S2: no Menu hint");
    // N2: the launch picker never opened on the attach path.
    assert!(
        !attached.contains(PICKER_TITLE),
        "N2: picker opened on attach"
    );

    // A1/N1: ctrl-b d detaches back to the manager.
    support::drive(
        &mut s,
        &["ctrl-b", "d", "wait:Detached ·"],
        case_timeout(&case),
    );
    waits::wait_state(&mut s, "Current directory", "manager back");
    checkpoint(&mut s, &dir, "02-detached");
    let manager = live_text(&mut s);
    assert!(manager.contains("keeps running"), "A1: no detach status");
    assert!(!manager.contains("Attached to"), "N1: capsule chrome stuck");
    eprintln!("j1 manager_hard_launch_locked: attach, detach to manager");
}

/// PICKER-HARDLAUNCH-005 (`jackin/manager/hard_launch_picker`): the launch
/// agent picker over the degraded manager.
///
/// Flow: boot hard-cases, Enter (picker), Down (move choice), Esc
/// (cancelled), down + space + space (expand/collapse roundtrip).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_manager_hard_launch_picker() {
    let dir = j1_dir("j1_manager_hard_launch_picker");
    let case = j1_case("manager_hard_launch_picker", HARD, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
        &mut s,
        &["enter", &format!("wait:{PICKER_TITLE}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-picker");
    let picker = live_text(&mut s);

    // V1: the picker title plus the workspace › role scope.
    assert_line_has(
        &picker,
        PICKER_TITLE,
        "payments-platform › the-architect",
        "V1 title",
    );
    // V2: the focused first choice plus its session-start account.
    assert_line_has(&picker, "Claude Code", "choose at start", "V2 choice");
    assert!(picker.contains("▎▪ Claude Code"), "V2: choice unfocused");
    // S1: the degraded badges stay up behind the modal.
    assert!(
        picker.contains("! instance index unreadable"),
        "S1: index badge lost"
    );
    assert!(picker.contains("▲ daemon stale"), "S1: daemon badge lost");
    // S2: the degraded footer keeps its warning under the modal (there
    // is no Enter Choose / Esc Cancel row in the degraded world).
    assert!(picker.contains("↑↓ Move"), "S2: no Move hint");
    assert!(
        picker.contains("Could not confirm running instances"),
        "S2: no footer warning"
    );

    // A1: Down moves the choice to Codex.
    support::drive(&mut s, &["down"], case_timeout(&case));
    std::thread::sleep(std::time::Duration::from_millis(300));
    checkpoint(&mut s, &dir, "02-moved");
    let moved = live_text(&mut s);
    assert_line_has(&moved, "▎", "Codex", "A1: choice did not move");
    assert_line_lacks(&moved, "Claude Code", "▎", "A1: old choice focused");

    // A2: Esc cancels the picker; the degraded tree is back.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, PICKER_TITLE, "picker cancelled");
    waits::wait_state(&mut s, "Enter Launch", "tree footer back");
    checkpoint(&mut s, &dir, "03-tree");
    let tree = live_text(&mut s);
    // N1: the picker is gone with the cancel.
    assert!(!tree.contains(PICKER_TITLE), "N1: picker stuck open");
    assert!(
        tree.contains("! instance index unreadable"),
        "badges lost on cancel"
    );

    // A2/N2 continued: the tree still expands and collapses (presence
    // for the no-expansion claim, then the claim itself).
    support::drive(
        &mut s,
        &["down", "space", "wait:▾ payments-platform"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "04-expanded");
    support::drive(
        &mut s,
        &["space", "wait:▸ payments-platform"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "05-tree");
    assert!(
        !live_text(&mut s).contains("▾ payments-platform"),
        "N2: tree stuck expanded"
    );
    eprintln!("j1 manager_hard_launch_picker: picker, move, cancel, toggle");
}

/// PICKER-LAUNCH-004 (`jackin/manager/launch_picker`): the launch agent
/// picker over the healthy manager.
///
/// Flow: boot, Enter (picker), Down (move choice), Esc (cancelled),
/// down + space + space (expand/collapse roundtrip).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_manager_launch_picker() {
    let dir = j1_dir("j1_manager_launch_picker");
    let case = j1_case("manager_launch_picker", RETURNING, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
        &mut s,
        &["enter", &format!("wait:{PICKER_TITLE}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-picker");
    let picker = live_text(&mut s);

    // V1: the picker title plus the workspace › role scope.
    assert_line_has(
        &picker,
        PICKER_TITLE,
        "payments-platform › the-architect",
        "V1 title",
    );
    // V2: the focused first choice plus the ready alternates.
    assert_line_has(&picker, "Claude Code", "choose at start", "V2 choice");
    assert_line_has(&picker, "▪ Codex", "ready", "V2 alternate");
    assert!(picker.contains("▎▪ Claude Code"), "V2: choice unfocused");
    // S1: the collapsed tree stays up behind the modal.
    assert!(picker.contains("▸ payments-platform"), "S1: tree lost");
    // S2: the picker footer (also the A2 presence proof).
    assert!(picker.contains("Enter Choose"), "S2: no Choose hint");
    assert!(picker.contains("Esc Cancel"), "S2: no Cancel hint");

    // A1: Down moves the choice to Codex.
    support::drive(&mut s, &["down"], case_timeout(&case));
    std::thread::sleep(std::time::Duration::from_millis(300));
    checkpoint(&mut s, &dir, "02-moved");
    let moved = live_text(&mut s);
    assert_line_has(&moved, "▎", "Codex", "A1: choice did not move");
    assert_line_lacks(&moved, "Claude Code", "▎", "A1: old choice focused");

    // A2: Esc cancels the picker; the tree is back.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, PICKER_TITLE, "picker cancelled");
    waits::wait_state(&mut s, "Enter Launch", "tree footer back");
    checkpoint(&mut s, &dir, "03-tree");
    let tree = live_text(&mut s);
    // N1: the picker is gone with the cancel.
    assert!(!tree.contains(PICKER_TITLE), "N1: picker stuck open");
    assert!(tree.contains("▸ payments-platform"), "tree lost on cancel");

    // A2/N2 continued: expand/collapse roundtrip proves the tree is
    // interactive and ends collapsed.
    support::drive(
        &mut s,
        &["down", "space", "wait:▾ payments-platform"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "04-expanded");
    support::drive(
        &mut s,
        &["space", "wait:▸ payments-platform"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "05-tree");
    assert!(
        !live_text(&mut s).contains("▾ payments-platform"),
        "N2: tree stuck expanded"
    );
    eprintln!("j1 manager_launch_picker: picker, move, cancel, toggle");
}

/// TREE-MANAGER-007 (`jackin/manager/tree_expanded`): the manager with the
/// payments-platform workspace expanded to its instance rows.
///
/// Flow: boot, down + space (expand), space (collapse), space (expand),
/// ctrl-q (quit presence), Esc (quit cancelled).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_manager_tree_expanded() {
    const TREE_ROW: &str = "the-archite…ude Code";
    let dir = j1_dir("j1_manager_tree_expanded");
    let case = j1_case("manager_tree_expanded", RETURNING, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
        &mut s,
        &["down", "space", "wait:▾ payments-platform"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-expanded");
    let expanded = live_text(&mut s);

    // V1: the expanded workspace plus its crumb.
    assert!(expanded.contains("▾ payments-platform"), "V1: not expanded");
    assert!(
        expanded.contains("Workspaces › payments-platform"),
        "V1: no crumb"
    );
    // V2: the tree instance rows (truncated agent labels).
    assert_line_has(&expanded, "◉ 7f3a", TREE_ROW, "V2 running row");
    assert_line_has(&expanded, "◌ c41e", "•", "V2 superseded row");
    // S1: the cursor sits on the expanded workspace.
    assert_line_has(&expanded, "▎", "payments-platform", "S1 cursor");
    // S2: the tree footer.
    assert!(expanded.contains("→ Expand"), "S2: no Expand hint");
    assert!(expanded.contains(TREE_FOOTER), "S2: no Details hint");

    // A1: space collapses the workspace again.
    support::drive(
        &mut s,
        &["space", "wait:▸ payments-platform"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-collapsed");
    let collapsed = live_text(&mut s);
    assert!(
        !collapsed.contains(TREE_ROW),
        "A1: tree instance rows survive collapse"
    );
    // The detail panel keeps its own (untruncated) instance rows.
    assert!(
        collapsed.contains("the-architect · Claude Code · running"),
        "A1: detail rows lost on collapse"
    );

    // A2: space expands once more (the toggle roundtrip).
    support::drive(
        &mut s,
        &["space", "wait:▾ payments-platform"],
        case_timeout(&case),
    );
    assert!(
        live_text(&mut s).contains(TREE_ROW),
        "A2: instance rows lost on re-expand"
    );
    checkpoint(&mut s, &dir, "03-expanded");

    // N1/N2: ctrl-q opens the quit confirm (presence), Esc cancels it;
    // the tree is untouched.
    support::drive(
        &mut s,
        &["ctrl-q", "wait:Exit jackin❯?"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "04-quit-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Exit jackin❯?", "quit cancelled");
    checkpoint(&mut s, &dir, "05-tree");
    let back = live_text(&mut s);
    assert!(!back.contains("Exit jackin❯?"), "N1: quit dialog stuck");
    assert!(!back.contains("▸ payments-platform"), "N2: tree collapsed");
    assert!(back.contains("▾ payments-platform"), "tree lost on cancel");
    eprintln!("j1 manager_tree_expanded: expand, collapse, expand, quit roundtrip");
}

/// PANEL-SCENARIO-010 (`jackin/scenarios/first-use`): the first-use intro
/// phrases at frame 40.
///
/// Flow: boot frame 40, Enter + Enter (skip), n (prelude).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_scenarios_first_use() {
    const PHRASE: &str = "Stand up, operator…";
    let dir = j1_dir("j1_scenarios_first_use");
    let case = j1_case("scenarios_first_use", FIRSTUSE40, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-phrases");
    let phrases = live_text(&mut s);

    // V1: the centered intro phrase plus the brand.
    assert!(phrases.contains(PHRASE), "V1: no intro phrase");
    assert!(phrases.contains(BOOT), "V1: no brand");
    // V2: the skip-only footer.
    assert!(phrases.contains("Enter Skip"), "V2: no skip footer");
    // S1: no host menubar on the intro route (01-manager proves File).
    assert!(!phrases.contains("File"), "S1: menubar on intro");
    // S2: no workspace tree on the intro route (01-manager proves it).
    assert!(!phrases.contains("Current directory"), "S2: tree on intro");

    // A1: Enter + Enter skips to the empty manager.
    support::drive(
        &mut s,
        &["enter", "enter", "wait:Current directory"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-manager");
    let manager = live_text(&mut s);
    assert!(manager.contains("File"), "A1: no menubar (S1 presence)");
    assert!(
        manager.contains("no instances"),
        "A1: instances in fresh world"
    );
    // N1/N2: the intro is gone with the skip.
    assert!(!manager.contains(PHRASE), "N1: phrase stuck");
    assert!(!manager.contains("Enter Skip"), "N2: skip footer stuck");

    // A2: n opens the new-workspace prelude from the empty manager.
    support::drive(
        &mut s,
        &["n", &format!("wait:{PRELUDE_S1}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-prelude");
    assert!(
        !live_text(&mut s).contains("Current directory"),
        "A2: manager tree visible in prelude"
    );
    eprintln!("j1 scenarios_first_use: phrases, skip to manager, prelude");
}

/// PANEL-SCENARIO-011 (`jackin/scenarios/hard-cases`): the degraded manager
/// at hard-cases boot.
///
/// Flow: boot, Enter (picker presence), Esc (cancelled), down + space
/// (expand presence), space (collapsed).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_scenarios_hard_cases() {
    let dir = j1_dir("j1_scenarios_hard_cases");
    let case = j1_case("scenarios_hard_cases", HARD, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);

    // V1: the degraded header badges.
    assert!(
        boot.contains("! instance index unreadable"),
        "V1: no index badge"
    );
    assert!(boot.contains("▲ daemon stale"), "V1: no daemon badge");
    // V2: the crowded degraded tree.
    assert!(boot.contains("▸ data-pipeline"), "V2: no data-pipeline row");
    assert!(boot.contains("3 running"), "V2: no running count");
    // S1: the footer warning names the unreadable index.
    assert!(
        boot.contains("Could not confirm running instances"),
        "S1: no footer warning"
    );
    // S2: the detail instances carry the stale daemon stamp.
    assert_line_has(&boot, "Instances", "▲ daemon stale · 3 s ago", "S2 stamp");

    // A1/N1: Enter opens the picker (presence), Esc cancels it again.
    support::drive(
        &mut s,
        &["enter", &format!("wait:{PICKER_TITLE}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-picker-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, PICKER_TITLE, "picker cancelled");
    waits::wait_state(&mut s, "Enter Launch", "tree footer back");
    assert!(
        !live_text(&mut s).contains(PICKER_TITLE),
        "N1: picker stuck open"
    );

    // A2/N2: the tree expands and collapses in the degraded world.
    support::drive(
        &mut s,
        &["down", "space", "wait:▾ payments-platform"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-expanded");
    support::drive(
        &mut s,
        &["space", "wait:▸ payments-platform"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-tree");
    let back = live_text(&mut s);
    assert!(
        !back.contains("▾ payments-platform"),
        "N2: tree stuck expanded"
    );
    assert!(
        back.contains("! instance index unreadable"),
        "badges lost on toggle"
    );
    eprintln!("j1 scenarios_hard_cases: degraded boot, picker+toggle roundtrips");
}

/// DIALOG-FAILURE-008 (`jackin/scenarios/launch-failure`): the frozen
/// FailNetwork failure with its dialog at frame 240.
///
/// Flow: boot frame 240, Esc (acknowledge), Tab (drawer), Esc (tree).
/// No container is touched: the failure and its retries are the fixture
/// simulation's recorded log lines.
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_scenarios_launch_failure() {
    const SUMMARY: &str = "The Construct network could not be attached";
    let dir = j1_dir("j1_scenarios_launch_failure");
    let case = j1_case("scenarios_launch_failure", FAILURE, BOOT_FAILURE);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    waits::wait_state(&mut s, SUMMARY, "failure summary");
    checkpoint(&mut s, &dir, "00-failure");
    let failure = live_text(&mut s);

    // V1: the dialog title plus the failure summary.
    assert!(failure.contains("! Launch failed"), "V1: no dialog title");
    assert!(failure.contains(SUMMARY), "V1: no summary");
    // V2: the run id plus the Docker next step.
    assert!(failure.contains("run-202609030914-b5df"), "V2: no run id");
    assert!(
        failure.contains("Check that Docker's jackin-net bridge exists"),
        "V2: no next step"
    );
    // S1: the retry log tail plus the build-log counter.
    assert!(
        failure.contains("retry 3/3 after 1600 ms"),
        "S1: no retry tail"
    );
    assert!(
        failure.contains("44 lines · b build log"),
        "S1: no log counter"
    );
    // S2: the dialog footer.
    assert!(failure.contains("y Copy"), "S2: no Copy hint");
    assert!(failure.contains("Esc Close"), "S2: no Close hint");

    // A1: Esc acknowledges the failure; the manager returns with the ack
    // status (the title text itself survives into the status line, so the
    // summary's disappearance is the close proof).
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, SUMMARY, "failure dialog closed");
    waits::wait_state(&mut s, "still running in the Construct", "ack status");
    waits::wait_state(&mut s, "Current directory", "manager tree back");
    checkpoint(&mut s, &dir, "01-acknowledged");
    let acked = live_text(&mut s);
    // N1/N2: the dialog rows are gone with the ack.
    assert!(!acked.contains(SUMMARY), "N1: summary stuck");
    assert!(!acked.contains("Run id"), "N2: dialog rows stuck");

    // A2: the acked manager is interactive (drawer roundtrip).
    support::drive(
        &mut s,
        &["tab", &format!("wait:{DRAWER_FOOTER}")],
        case_timeout(&case),
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DRAWER_FOOTER, "drawer closed");
    waits::wait_state(&mut s, TREE_FOOTER, "tree footer back");
    checkpoint(&mut s, &dir, "02-tree");
    eprintln!("j1 scenarios_launch_failure: failure, ack, drawer roundtrip");
}

/// PANEL-SCENARIO-012 (`jackin/scenarios/launch-running`): the launch
/// cockpit at the Credentials stage.
///
/// Flow: boot, d (debug chip on), d (chip off), i (info presence),
/// Esc (info closed).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_scenarios_launch_running() {
    let dir = j1_dir("j1_scenarios_launch_running");
    let case = j1_case("scenarios_launch_running", LAUNCH, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    waits::wait_state(&mut s, "stage 3 of 11 · Credentials", "cockpit stage");
    checkpoint(&mut s, &dir, "00-cockpit");
    let cockpit = live_text(&mut s);

    // V1: the launch header plus the Credentials stage.
    assert!(
        cockpit.contains("Launch › payments-platform › the-architect"),
        "V1: no launch header"
    );
    assert!(
        cockpit.contains("stage 3 of 11 · Credentials"),
        "V1: no stage line"
    );
    // V2: the finished stages plus the spinning frontier.
    assert!(cockpit.contains("✓ 01 Identity"), "V2: no stage 01");
    assert!(cockpit.contains("✓ 02 Role"), "V2: no stage 02");
    assert!(cockpit.contains("⠋ 03 Credentials"), "V2: no stage 03");
    // S1: the session-choice account line.
    assert!(
        cockpit.contains("account Claude · Work (session choice)"),
        "S1: no session choice"
    );
    // S2: the cockpit footer.
    assert!(cockpit.contains("i Container info"), "S2: no info hint");
    assert!(cockpit.contains("c Cancel"), "S2: no Cancel hint");

    // A1: d toggles the debug chip on; d toggles it off again.
    support::drive(&mut s, &["d", "wait:run-2026"], case_timeout(&case));
    checkpoint(&mut s, &dir, "01-chip");
    support::press_step(&s, "d");
    waits::wait_gone(&mut s, "run-2026", "debug chip off");
    checkpoint(&mut s, &dir, "02-nochip");
    // N1: the chip is gone with the second toggle.
    assert!(
        !live_text(&mut s).contains("run-2026"),
        "N1: debug chip stuck"
    );

    // A2/N2: i opens the container info dialog (presence), Esc closes it.
    support::drive(&mut s, &["i", "wait:Debug info"], case_timeout(&case));
    checkpoint(&mut s, &dir, "03-info-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Debug info", "info dialog closed");
    let back = live_text(&mut s);
    assert!(!back.contains("Debug info"), "N2: info dialog stuck");
    assert!(
        back.contains("stage 3 of 11 · Credentials"),
        "cockpit lost on close"
    );
    checkpoint(&mut s, &dir, "04-cockpit");
    eprintln!("j1 scenarios_launch_running: cockpit, chip toggle, info roundtrip");
}

/// PANEL-SCENARIO-014 (`jackin/scenarios/outro-last`): the outro warp with
/// its two-step dismiss-to-quit.
///
/// Flow (session 1): boot frame 40 (warp), x (ignored), Enter (caption),
/// Enter (clean exit). Session 2 (first-use frame 40) proves the N
/// needles render.
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_scenarios_outro_last() {
    const PHRASE: &str = "Stand up, operator…";
    let dir = j1_dir("j1_scenarios_outro_last");
    let case = j1_case("scenarios_outro_last", OUTRO40, BOOT_SKIP);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-warp");
    let warp = live_text(&mut s);

    // V1: the skip footer on the warp.
    assert!(warp.contains("Enter Skip"), "V1: no skip footer");
    // V2: the outro particle field (star + signal glyphs).
    for glyph in ["*", "+", "·"] {
        assert!(warp.contains(glyph), "V2: no outro glyph `{glyph}`");
    }

    // A1/S1: x is Consumed and the warp is tick-frozen — the frame is
    // byte-identical across the ignored key.
    let before = live_text(&mut s);
    support::press_step(&s, "x");
    std::thread::sleep(std::time::Duration::from_millis(300));
    let after = live_text(&mut s);
    assert_eq!(before, after, "A1/S1: x changed the warp frame");

    // A2: Enter advances the warp to the caption; Enter quits cleanly.
    support::drive(&mut s, &["enter", "wait:Enter Close"], case_timeout(&case));
    checkpoint(&mut s, &dir, "01-caption");
    // S2: the caption swaps the hint (the warp hint is gone).
    assert!(
        !live_text(&mut s).contains("Enter Skip"),
        "S2: caption keeps the warp hint"
    );
    support::press_step(&s, "enter");
    let exit = expect_exit_text(&s, case_timeout(&case), "outro quit");
    assert!(
        !exit.contains("panic"),
        "outro exit frame mentions panic: {exit:?}"
    );
    eprintln!("outro exit frame ({} bytes)", exit.len());

    // Session 2: the N presence proofs — the intro phrase renders at
    // first-use frame 40 and the manager follows its skip.
    let case2 = j1_case("outro_presence", FIRSTUSE40, BOOT);
    write_provenance_named(&dir, &case2, "provenance-s2.txt");
    let mut s2 = support::spawn_boot(&case2);
    waits::wait_state(&mut s2, PHRASE, "N1 presence: intro phrase renders");
    // N1: no intro phrases on the outro warp.
    assert!(!warp.contains(PHRASE), "N1: intro phrase on outro");
    support::drive(
        &mut s2,
        &["enter", "enter", "wait:Current directory"],
        case_timeout(&case2),
    );
    // N2: no manager chrome on the outro warp.
    assert!(!warp.contains("Current directory"), "N2: tree on outro");
    checkpoint(&mut s2, &dir, "02-presence");
    eprintln!("j1 scenarios_outro_last: warp, ignored x, caption, clean exit");
}

/// PANEL-SCENARIO-013 (`jackin/scenarios/returning`): the populated manager
/// at returning boot.
///
/// Flow: boot, Enter (picker presence), Esc (cancelled), Tab (drawer),
/// Esc (tree).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_scenarios_returning() {
    let dir = j1_dir("j1_scenarios_returning");
    let case = j1_case("scenarios_returning", RETURNING, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);

    // V1: the populated tree plus the saved detail.
    assert!(boot.contains("Workspaces"), "V1: no Workspaces chrome");
    assert!(boot.contains("2 running"), "V1: no running count");
    assert!(
        boot.contains("Current directory · payments-platform"),
        "V1: no detail title"
    );
    // V2: the daemon instance rows.
    assert_line_has(&boot, "◉ 7f3a", "running", "V2 running row");
    assert_line_has(&boot, "◌ c41e", "preserved · dirty", "V2 dirty row");
    // S1: the cursor sits on Current directory.
    assert_line_has(&boot, "▎", "Current directory", "S1 cursor");
    // S2: the populated-manager footer.
    assert!(boot.contains("Enter Launch"), "S2: no Launch hint");
    assert!(boot.contains(TREE_FOOTER), "S2: no Details hint");
    assert!(boot.contains("q Quit"), "S2: no Quit hint");

    // A1/N1: Enter opens the picker (presence), Esc cancels it again.
    support::drive(
        &mut s,
        &["enter", &format!("wait:{PICKER_TITLE}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-picker-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, PICKER_TITLE, "picker cancelled");
    waits::wait_state(&mut s, "Enter Launch", "tree footer back");
    assert!(
        !live_text(&mut s).contains(PICKER_TITLE),
        "N1: picker stuck open"
    );

    // A2/N2: Tab opens the drawer (presence), Esc closes it again.
    support::drive(
        &mut s,
        &["tab", &format!("wait:{DRAWER_FOOTER}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-drawer-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DRAWER_FOOTER, "drawer closed");
    waits::wait_state(&mut s, TREE_FOOTER, "tree footer back");
    let back = live_text(&mut s);
    assert!(!back.contains(DRAWER_FOOTER), "N2: drawer stuck open");
    assert_line_has(&back, "▎", "Current directory", "cursor moved");
    checkpoint(&mut s, &dir, "03-boot");
    eprintln!("j1 scenarios_returning: boot, picker+drawer roundtrips");
}

/// FORM-SETTINGS-014 (`jackin/settings/agents`): global settings on the
/// Agents tab with per-agent runtime modes.
///
/// Flow: boot accounts-mixed, s, 4 + Enter (Agents body), [ (Environments),
/// ] (back).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_settings_agents() {
    let dir = j1_dir("j1_settings_agents");
    let case = j1_case("settings_agents", ACCOUNTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
        &mut s,
        &[
            "s",
            "wait:Settings › global",
            "4",
            "enter",
            "wait:Agent runtime mode",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-agents");
    let agents = live_text(&mut s);

    // V1: the mode table plus the synced default.
    assert!(agents.contains("Agent runtime mode"), "V1: no mode header");
    assert_line_has(&agents, "Claude Code", "sync", "V1 Claude row");
    // V2: the ignored agent plus its no-credentials note.
    assert_line_has(&agents, "Kimi Code", "ignore", "V2 Kimi row");
    assert!(
        agents.contains("no credentials handed to the container"),
        "V2: no ignore note"
    );
    // S1: the registry pointer.
    assert!(
        agents.contains("accounts are registered in Accounts (c)"),
        "S1: no registry hint"
    );
    // S2: the agents body footer (also the N2 presence proof).
    assert!(agents.contains("Space Cycle mode"), "S2: no Cycle hint");
    assert!(agents.contains("d Reset to sync"), "S2: no Reset hint");

    // A1: [ jumps to Environments with TABS focused.
    support::drive(&mut s, &["[", "wait:GH_TOKEN"], case_timeout(&case));
    checkpoint(&mut s, &dir, "02-env");
    let env = live_text(&mut s);
    assert!(env.contains("1–5 Jump"), "A1: no TABS footer");
    // N1/N2: the agents body and its footer are gone.
    assert!(!env.contains("Agent runtime mode"), "N1: agents body stuck");
    assert!(!env.contains("Space Cycle mode"), "N2: agents footer stuck");

    // A2: ] jumps back to Agents.
    support::drive(
        &mut s,
        &["]", "wait:Agent runtime mode"],
        case_timeout(&case),
    );
    assert!(
        live_text(&mut s).contains("Kimi Code"),
        "A2: agents body lost"
    );
    checkpoint(&mut s, &dir, "03-agents");
    eprintln!("j1 settings_agents: agents, env jump, agents back");
}

/// FORM-SETTINGS-013 (`jackin/settings/mounts`): global settings on the
/// Mounts tab with its scope column.
///
/// Flow: boot accounts-mixed, s, 2 + Enter (Mounts body), [ (General),
/// ] (back).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_settings_mounts() {
    const ROW: &str = "/home/agent/.gitconfig";
    let dir = j1_dir("j1_settings_mounts");
    let case = j1_case("settings_mounts", ACCOUNTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
        &mut s,
        &[
            "s",
            "wait:Settings › global",
            "2",
            "enter",
            &format!("wait:{ROW}"),
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-mounts");
    let mounts = live_text(&mut s);

    // V1: the grid header plus the global gitconfig row.
    assert_line_has(&mounts, "Destination", "Scope", "V1 header");
    assert_line_has(&mounts, ROW, "global", "V1 gitconfig row");
    // V2: the role-scoped kube row.
    assert_line_has(&mounts, "/home/agent/.kube", "role sre", "V2 kube row");
    // S1: the body cursor sits on the first row.
    assert!(mounts.contains("▎›"), "S1: no body cursor");
    // S2: the mounts body footer (also the N2 presence proof).
    assert!(mounts.contains("s Scope"), "S2: no Scope hint");
    assert!(mounts.contains("a Add mount"), "S2: no Add hint");

    // A1: [ jumps to General with TABS focused.
    support::drive(
        &mut s,
        &["[", "wait:Sign off commits (DCO)"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-general");
    let general = live_text(&mut s);
    assert!(general.contains("1–5 Jump"), "A1: no TABS footer");
    // N1/N2: the mounts body and its footer are gone.
    assert!(!general.contains(ROW), "N1: mounts body stuck");
    assert!(!general.contains("s Scope"), "N2: mounts footer stuck");

    // A2: ] jumps back to Mounts.
    support::drive(&mut s, &["]", &format!("wait:{ROW}")], case_timeout(&case));
    assert!(
        live_text(&mut s).contains("/home/agent/.kube"),
        "A2: mounts body lost"
    );
    checkpoint(&mut s, &dir, "03-mounts");
    eprintln!("j1 settings_mounts: mounts, general jump, mounts back");
}

/// FORM-SETTINGS-012 (`jackin/settings/route`): global settings on General
/// with the commit flags.
///
/// Flow: boot accounts-mixed, s (General), ] (Mounts), [ (General).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_settings_route() {
    const DCO: &str = "Sign off commits (DCO)";
    let dir = j1_dir("j1_settings_route");
    let case = j1_case("settings_route", ACCOUNTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
        &mut s,
        &["s", "wait:Settings › global"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-general");
    let general = live_text(&mut s);

    // V1: the settings crumb plus the Commits section.
    assert!(
        general.contains("Settings › global › General"),
        "V1: no crumb"
    );
    assert!(general.contains("Commits"), "V1: no Commits section");
    // V2: the shipped commit flags.
    assert!(
        general.contains("[✓] Add Co-authored-by trailer"),
        "V2: no trailer flag"
    );
    assert!(
        general.contains("[✓] Sign off commits (DCO)"),
        "V2: no DCO flag"
    );
    // S1: the TABS footer.
    assert!(general.contains("1–5 Jump"), "S1: no 1–5 Jump");
    assert!(general.contains("Enter Body"), "S1: no Enter Body");
    // S2: the rendered trailers (also the N2 presence proof).
    assert!(general.contains("Co-authored-by:"), "S2: no trailer line");
    assert!(
        general.contains("Signed-off-by: Alexey Zhokhov"),
        "S2: no signoff line"
    );

    // A1: ] jumps to Mounts with TABS focused.
    support::drive(
        &mut s,
        &["]", "wait:/home/agent/.gitconfig"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-mounts");
    let mounts = live_text(&mut s);
    assert!(mounts.contains("1–5 Jump"), "A1: TABS focus lost");
    // N1/N2: the General body is gone.
    assert!(!mounts.contains(DCO), "N1: General body stuck");
    assert!(!mounts.contains("Co-authored-by:"), "N2: trailers stuck");

    // A2: [ jumps back to General.
    support::drive(&mut s, &["[", &format!("wait:{DCO}")], case_timeout(&case));
    assert!(
        live_text(&mut s).contains("Co-authored-by:"),
        "A2: General body lost"
    );
    checkpoint(&mut s, &dir, "03-general");
    eprintln!("j1 settings_route: general, mounts jump, general back");
}

/// FORM-SETTINGS-016 (`jackin/settings/save_preview`): the settings save
/// preview dialog with its two trust diff lines.
///
/// Flow: boot, s, 5 + Enter, space + down + space (2 changes), ctrl-s
/// (preview), Esc (cancelled), ctrl-s (reopened), Esc (cancelled again).
/// Nothing is saved: both previews are cancelled, not confirmed.
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_settings_save_preview() {
    const DIALOG: &str = "Save settings";
    const DIFF1: &str = "~ trust github.com/chainargos/roles true → false";
    const DIFF2: &str = "~ trust github.com/acme-labs/roles-experimental false → true";
    let dir = j1_dir("j1_settings_save_preview");
    let case = j1_case("settings_save_preview", ACCOUNTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
        &mut s,
        &[
            "s",
            "wait:Settings › global",
            "5",
            "enter",
            "wait:Role sources",
            "space",
            "down",
            "space",
            "wait:• 2 changes",
        ],
        case_timeout(&case),
    );
    support::press_step(&s, "ctrl-s");
    waits::wait_state(&mut s, DIALOG, "save preview open");
    waits::wait_state(&mut s, DIFF1, "first diff line");
    waits::wait_state(&mut s, DIFF2, "second diff line");
    checkpoint(&mut s, &dir, "01-preview");
    let preview = live_text(&mut s);

    // V1: the dialog names the global scope and the change count.
    assert!(preview.contains(DIALOG), "V1: no dialog title");
    assert!(preview.contains("global config"), "V1: no scope");
    assert!(preview.contains("config.toml"), "V1: no config path");
    assert!(preview.contains("2 changes"), "V1: no change count");
    // V2: both trust diff lines.
    assert!(preview.contains(DIFF1), "V2: no chainargos diff");
    assert!(preview.contains(DIFF2), "V2: no acme-labs diff");
    // S1: the dirty badge plus the dotted tab behind the dialog.
    assert!(preview.contains("• 2 changes"), "S1: no change badge");
    assert!(preview.contains("Trust •"), "S1: no dotted tab");
    // S2: Cancel holds the dialog focus.
    assert_line_has(&preview, "▎Cancel", "Save", "S2 actions");

    // A1: Esc cancels the preview; the edits stay intact.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DIALOG, "preview cancelled");
    waits::wait_state(&mut s, "Save aborted · settings unchanged", "cancel status");
    waits::wait_state(&mut s, "• 2 changes", "edits intact");
    checkpoint(&mut s, &dir, "02-cancelled");

    // A2: ctrl-s reopens the preview (the dirt is intact), Esc cancels.
    support::press_step(&s, "ctrl-s");
    waits::wait_state(&mut s, DIALOG, "preview reopened");
    waits::wait_state(&mut s, DIFF1, "diff line back");
    checkpoint(&mut s, &dir, "03-reopened");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DIALOG, "preview cancelled again");
    checkpoint(&mut s, &dir, "04-cancelled");
    let back = live_text(&mut s);
    // N1/N2: the dialog and its diff lines are gone; the badge persists.
    assert!(!back.contains(DIALOG), "N1: preview stuck open");
    assert!(!back.contains(DIFF1), "N2: diff line stuck");
    assert!(back.contains("• 2 changes"), "edits lost on cancel");
    assert!(back.contains("Role sources"), "trust body lost on cancel");
    eprintln!("j1 settings_save_preview: preview, cancel, reopen, cancel");
}

/// FORM-SETTINGS-015 (`jackin/settings/trust`): global settings on the
/// Trust tab with the role-source trust flags.
///
/// Flow: boot accounts-mixed, s, 5 + Enter (Trust body), [ (Agents),
/// ] (back).
#[test]
#[ignore = "jackin j1 check; run with --ignored"]
fn j1_settings_trust() {
    let dir = j1_dir("j1_settings_trust");
    let case = j1_case("settings_trust", ACCOUNTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive(
        &mut s,
        &[
            "s",
            "wait:Settings › global",
            "5",
            "enter",
            "wait:Role sources",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-trust");
    let trust = live_text(&mut s);

    // V1: the role-source header plus the trust counts.
    assert!(trust.contains("Role sources"), "V1: no Role sources");
    assert!(
        trust.contains("3 trusted · 1 untrusted"),
        "V1: no trust counts"
    );
    // V2: the trusted chainargos row plus the untrusted acme-labs row.
    assert_line_has(
        &trust,
        "github.com/chainargos/roles",
        "[✓] trusted",
        "V2 chainargos row",
    );
    assert_line_has(
        &trust,
        "github.com/acme-labs/roles-experimental",
        "[ ] untrusted",
        "V2 acme-labs row",
    );
    // S1: the untrusted-source warning.
    assert!(
        trust.contains("An untrusted source blocks + Load role"),
        "S1: no trust warning"
    );
    // S2: the trust body footer (also the N2 presence proof).
    assert!(trust.contains("Space Toggle trust"), "S2: no Toggle hint");
    assert!(trust.contains("o Open source"), "S2: no Open hint");

    // A1: [ jumps to Agents with TABS focused.
    support::drive(
        &mut s,
        &["[", "wait:Agent runtime mode"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-agents");
    let agents = live_text(&mut s);
    assert!(agents.contains("1–5 Jump"), "A1: no TABS footer");
    // N1/N2: the trust body and its footer are gone.
    assert!(!agents.contains("Role sources"), "N1: trust body stuck");
    assert!(
        !agents.contains("Space Toggle trust"),
        "N2: trust footer stuck"
    );

    // A2: ] jumps back to Trust.
    support::drive(&mut s, &["]", "wait:Role sources"], case_timeout(&case));
    assert!(
        live_text(&mut s).contains("3 trusted · 1 untrusted"),
        "A2: trust body lost"
    );
    checkpoint(&mut s, &dir, "03-trust");
    eprintln!("j1 settings_trust: trust, agents jump, trust back");
}
