//! Holla pending-roots slice 8A-H5 executable checks (VB Phase-8r).
//!
//! One ignored PTY test per H5 registry row (20 rows: the two parity
//! leftovers deferred by the H4 slice — `parity/task-sources`,
//! `parity/upgrade-managers` — plus the first 18 of the plan's 25-root
//! 8A-H5 chrome slice in plan order, through `flows/activities/overlay`;
//! `flows/alternatives`, `flows/help/overlay`, `flows/menu/file`,
//! `flows/menu/go`, `flows/quit/confirm` and both `resize/rust-dirty`
//! roots stay pending for a later tranche), over the real `holla`
//! binary built from this worktree's verified VB sources
//! (`env!("CARGO_BIN_EXE_holla")`, the harness `[[bin]]` compiled from
//! `../../src/bin/holla/main.rs`). Each test drives the row's specified
//! inputs and asserts its visual (V), state (S), action (A), and negative
//! (N) checks in live-PTY executable form:
//!
//! - V: live needles from the row's boot frame (suggestion rows,
//!   counts, badges, scrolled offsets, fade-edge dimming) plus
//!   same-line coexistence proofs.
//! - S: live labels (finder scope, facts, footers, status rows) from
//!   the boot frame (S1) and the flow landing state (S2).
//! - A: behavior probes — the specified gesture runs and the specified
//!   advance (filter, scope widen, sort toggle, plan cursor, dialog,
//!   cancel, restart, wheel scroll, or boundary stance) is observed;
//!   `Changed`/`Consumed` outcomes are proven by their screen
//!   correlates (moved, byte-identical), since PTY cannot see the enum.
//! - N: absence assertions, each paired with a presence proof in the
//!   same test (boot checkpoint, mid-flow checkpoint, or pre/post
//!   transition) so no absence passes vacuously.
//!
//! No new snapshots and no new static captures: all 20 roots already
//! have approved frames gated cell-exact by the ported matrices in
//! `holla.rs`, `pointer.rs`, and `audit.rs` (plan §Reconciliation:
//! "rows plus checks, not recaptures"), so every case here uses owned
//! (dynamic) names and stays out of the `snapshots/` inventory. No
//! isolated-component captures: every row's `requires` set
//! (key-injection, glyph-capture) is PTY-level, and every assertion has
//! a live-PTY executable form; the three wheel-fade rows prove the fade
//! itself live through cell-luminance geometry (faded edge row reads
//! strictly below the interior row) plus a header row that never fades.
//!
//! Per-test scratch lands under this harness crate's
//! `target/tuiscotti/journeys/<test>/` (gitignored): the row's named
//! checkpoint(s) as text frames, `00-boot.txt` where the row needs a
//! boot comparison, and `provenance.txt` (binary path, size, mtime,
//! argv). Typed input is synthetic and in-memory only (simulation
//! data); no test runs real commands, touches Git state, or performs
//! destructive operations: the executor rows run the fixture world's
//! simulated scripts only, every trust dialog is cancelled, never
//! trusted, and every world is the fixture simulation.
//!
//! Row → test map (registry id → `h5_*` test):
//!
//! - PARITY-TASK-SOURCES-001 → [`h5_parity_task_sources`]
//! - PARITY-UPGRADE-MANAGERS-001 → [`h5_parity_upgrade_managers`]
//! - PANEL-FINDER-003 → [`h5_audit_rust`]
//! - PANEL-UPGRADE-003 → [`h5_audit_upgrade`]
//! - PANEL-FINDER-004 → [`h5_concept_activities_multi`]
//! - PANEL-DISK-003 → [`h5_concept_disk_cleanup`]
//! - PANEL-DOCKER-003 → [`h5_concept_docker_cleanup`]
//! - PANEL-FINDER-005 → [`h5_concept_first_use`]
//! - PANEL-FINDER-006 → [`h5_concept_hard_cases`]
//! - PANEL-TASK-005 → [`h5_concept_launch_failure`]
//! - PANEL-CHILD-002 → [`h5_concept_monorepo_child`]
//! - PANEL-FINDER-007 → [`h5_concept_monorepo_root`]
//! - PANEL-REMOTE-002 → [`h5_concept_remote_host`]
//! - PANEL-BROWSER-002 → [`h5_fade_browser_wheel`]
//! - PANEL-CLEANUP-008 → [`h5_fade_cleanup_list_wheel`]
//! - PANEL-TASK-006 → [`h5_fade_executor_burst_end`]
//! - PANEL-TRUST-003 → [`h5_fade_trust_body_wheel`]
//! - PICKER-ACTIVITIES-001 → [`h5_activities_empty`]
//! - PANEL-TASK-007 → [`h5_activities_jump`]
//! - PICKER-ACTIVITIES-002 → [`h5_activities_overlay`]

use std::path::{Path, PathBuf};
use std::time::Duration;

use tuiscotti::tui::{Session, Wheel};
use tuiscotti::{Color as TColor, Frame, Provenance, Rgb};

use crate::pointer::wheel_below;
use crate::support::state_waits as waits;
use crate::support::typed_input::Input;
use crate::support::{self, Case, Color, HOLLA};

/// Boot needle on every holla route: the brand.
const BOOT: &str = "holla❯";
/// Empty finder query placeholder (boot/cleared states).
const PLACEHOLDER: &str = "Search actions and resources…";

const TASKSRC: &[&str] = &[
    "--scenario",
    "parity-task-sources",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const UPGRADERS: &[&str] = &[
    "--scenario",
    "parity-upgrade-managers",
    "--motion",
    "reduced",
];
const RUST_P: &[&str] = &[
    "--scenario",
    "rust-dirty",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const UPGRADE_P: &[&str] = &[
    "--scenario",
    "upgrade-plan",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const ACTMULTI: &[&str] = &[
    "--scenario",
    "activities-multi",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const DISK_P: &[&str] = &[
    "--scenario",
    "disk-cleanup",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const DOCKER_P: &[&str] = &[
    "--scenario",
    "docker-cleanup",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const FIRSTUSE: &[&str] = &[
    "--scenario",
    "first-use",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const HARD: &[&str] = &[
    "--scenario",
    "hard-cases",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const LAUNCHFAIL: &[&str] = &[
    "--scenario",
    "launch-failure",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const CHILD_P: &[&str] = &[
    "--scenario",
    "monorepo-child",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const ROOT_P: &[&str] = &[
    "--scenario",
    "monorepo-root",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const REMOTE_P: &[&str] = &[
    "--scenario",
    "remote-host",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const BROWSER: &[&str] = &["--scenario", "parity-browser", "--motion", "reduced"];
const INSIGHTS: &[&str] = &["--scenario", "parity-insights", "--motion", "reduced"];
const EXECUTOR: &[&str] = &["--scenario", "parity-executor", "--motion", "reduced"];
const CUSTOM: &[&str] = &["--scenario", "parity-custom-actions", "--motion", "reduced"];

/// Owned-name H5 case at the canonical 120x40 truecolor geometry.
fn h5_case(slug: &str, args: &'static [&'static str], boot: &'static str) -> Case {
    Case::dynamic(
        format!("journeys/holla/h5/{slug}"),
        HOLLA,
        args,
        120,
        40,
        Color::Truecolor,
        boot,
    )
}

/// [`h5_case`] with an explicit per-step timeout for tick-driven flows.
fn h5_case_t(
    slug: &str,
    args: &'static [&'static str],
    boot: &'static str,
    timeout_ms: u64,
) -> Case {
    h5_case(slug, args, boot).timeout(timeout_ms)
}

/// Unique scratch dir for one H5 test (created, never shared).
fn h5_dir(test: &str) -> PathBuf {
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
    eprintln!("h5 provenance: {body}");
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

/// One fresh live frame with cell colors (no wait). Provenance is
/// informational only (excluded from digests).
fn live_frame(s: &mut Session) -> Frame {
    let screen = s
        .snapshot()
        .unwrap_or_else(|e| panic!("live frame sample failed: {e:#}"));
    support::frame_from_screen(
        &screen,
        Provenance::now("tuiscotti-default", "h5-checks", vec![]),
    )
}

/// Absence wait for a just-pressed key, without the vacuous-absence
/// guard. Every call site proves presence in its pre-keypress sample
/// (a V/S/pre assert or a drive `wait:`), so the guard would only add
/// a race: a descheduled test thread can take its guard snapshot after
/// the app already redrew, failing a correct transition. The predicate
/// form passes at once when the transition already landed, and also
/// drains one-frame lingers.
fn wait_absent(s: &mut Session, needle: &str, what: &str) {
    support::wait_screen(
        s,
        support::DEFAULT_WAIT,
        &format!("{what}: `{needle}` never expired"),
        |screen| !support::screen_text(screen).contains(needle),
    );
}

/// Bounded wait for the screen text to differ from `baseline` (the
/// wheel-scroll settle: the wheel has no completion signal, so the
/// scrolled content itself is the arrival proof).
fn wait_changed(s: &mut Session, baseline: &str, timeout: Duration, what: &str) -> String {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        let cur = live_text(s);
        if cur != baseline {
            return cur;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "{what}: screen never changed"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

/// Some line must contain both `needle` and `also` (the same-row
/// coexistence proof: boot rows, dialog rows, titles, fact rows).
fn assert_line_has(text: &str, needle: &str, also: &str, what: &str) {
    assert!(
        text.lines().any(|l| l.contains(needle) && l.contains(also)),
        "{what}: no line contains both `{needle}` and `{also}`"
    );
}

/// First occurrence of `needle` as `(row, col)`, waiting until it appears.
fn find_pos(s: &mut Session, needle: &str, timeout: Duration) -> (u16, u16) {
    let obs = support::wait_screen(
        s,
        timeout,
        &format!("`{needle}` never appeared"),
        |screen| support::screen_find(screen, needle).is_some(),
    );
    support::screen_find(&obs.screen, needle).expect("wait passed with the needle on screen")
}

/// `notches` wheel steps over `needle`'s cell, paced like a send step.
/// Returns the `(row, col)` wheeled, for stance probes after the content
/// scrolled (the cell stays inside the scroll region).
fn wheel_over(
    s: &mut Session,
    needle: &str,
    dir: Wheel,
    notches: u32,
    timeout: Duration,
) -> (u16, u16) {
    let (row, col) = find_pos(s, needle, timeout);
    wheel_at(s, col, row, dir, notches);
    (row, col)
}

/// `notches` wheel steps at fixed `(col, row)` — the stance-probe form,
/// used when the needle may have scrolled off-screen.
fn wheel_at(s: &mut Session, col: u16, row: u16, dir: Wheel, notches: u32) {
    for _ in 0..notches {
        Input::wheel(dir, col, row).send(s);
        std::thread::sleep(Duration::from_millis(120));
    }
}

/// Resolve a live cell color to RGB. `Default` has no live-resolvable
/// value (it is the terminal's own color), so resolving one fails the
/// check loudly instead of guessing.
fn resolve_rgb(c: TColor, what: &str) -> Rgb {
    match c {
        TColor::Rgb(rgb) => rgb,
        TColor::Indexed(n) => Rgb::from_indexed(n),
        TColor::Default => panic!("{what}: cell color is Default, cannot resolve"),
    }
}

/// Relative luminance (Rec. 709 scaled by 1000) of a live cell color.
fn lum(c: TColor, what: &str) -> u32 {
    let rgb = resolve_rgb(c, what);
    u32::from(rgb.r) * 2126 + u32::from(rgb.g) * 7152 + u32::from(rgb.b) * 722
}

/// Median foreground luminance of the non-blank text cells on `row`
/// between columns `c0` and `c1` (inclusive). Blank/continuation cells
/// carry no text tone and are skipped; at least one text cell must
/// resolve. The column window keeps the listing-side cells out of a
/// preview-panel fade probe (and vice versa).
fn row_fg_median_cols(frame: &Frame, row: u16, c0: u16, c1: u16, what: &str) -> u32 {
    let mut lums: Vec<u32> = Vec::new();
    for col in c0..=c1.min(frame.cols.saturating_sub(1)) {
        let Some(cell) = frame.get(col, row) else {
            continue;
        };
        if cell.continuation || cell.symbol.trim().is_empty() {
            continue;
        }
        if matches!(cell.fg, TColor::Default) {
            continue;
        }
        lums.push(lum(cell.fg, what));
    }
    assert!(
        !lums.is_empty(),
        "{what}: row {row} cols {c0}..={c1} have no resolvable-fg text cells"
    );
    lums.sort_unstable();
    lums[lums.len() / 2]
}

/// The fade dims toward the container background: the faded row's median
/// foreground luminance must read strictly below the interior row's.
fn assert_faded(edge: u32, interior: u32, what: &str) {
    assert!(
        edge < interior,
        "{what}: edge row must fade (edge lum {edge} < interior lum {interior})"
    );
}

/// Whole text row identical across two frames (all cells incl. colors).
fn assert_row_same(a: &Frame, b: &Frame, row: u16, what: &str) {
    assert_eq!(
        a.cols, b.cols,
        "{what}: frame widths differ ({} vs {})",
        a.cols, b.cols
    );
    for col in 0..a.cols {
        let ca = a
            .get(col, row)
            .unwrap_or_else(|| panic!("{what}: cell ({col}, {row}) missing in first frame"));
        let cb = b
            .get(col, row)
            .unwrap_or_else(|| panic!("{what}: cell ({col}, {row}) missing in second frame"));
        assert!(
            ca.symbol == cb.symbol && ca.fg == cb.fg && ca.bg == cb.bg,
            "{what}: cell ({col}, {row}) differs: {ca:?} vs {cb:?}"
        );
    }
}

/// PARITY-TASK-SOURCES-001 (`holla/parity/task-sources`): the
/// task-sources parity boot plus the yarn filter behavior.
///
/// Flow: boot, type `yarn` (results), Esc (cleared), type `yarn`
/// (results again).
#[test]
#[ignore = "holla h5 check; run with --ignored"]
fn h5_parity_task_sources() {
    let dir = h5_dir("h5_parity_task_sources");
    let case = h5_case("parity_task_sources", TASKSRC, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the top suggestion pairs the recipe with its manifest.
    assert_line_has(&boot, "› just build", ".justfile here", "V1 row");
    // V2: the make targets list below the recipes.
    assert!(boot.contains("make build"), "V2: no make target");
    // S1: the facts anchor the polyglot folder.
    assert_line_has(&boot, "In", "~/work/poly", "S1 world");

    // A1: typing filters to the yarn task sources.
    support::drive(&mut s, &["type:yarn", "wait:yarn s00"], case_timeout(&case));
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    // V3-flow: the result count.
    assert!(res.contains("Results · 31"), "flow: no Results · 31");
    // S2: the scope readout stays here.
    assert!(res.contains("scope ‹ here ›"), "S2: no here scope");
    // N1: the empty-query placeholder is gone with the results.
    assert!(!res.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: Esc clears the query back to the placeholder.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "yarn s00", "results cleared");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);
    // N2: the yarn rows are gone after the clear.
    assert!(!cleared.contains("yarn s00"), "N2: yarn rows stuck");
    support::drive(&mut s, &["type:yarn", "wait:yarn s00"], case_timeout(&case));
    checkpoint(&mut s, &dir, "03-results-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "› yarn dev", "package.json here", "row intact");
    eprintln!("h5 parity_task_sources: boot, filter roundtrip");
}

/// PARITY-UPGRADE-MANAGERS-001 (`holla/parity/upgrade-managers`): the
/// upgrade-managers parity boot plus the brew batch upgrade behavior.
///
/// Flow: boot, run the Homebrew batch to the outdated report, r
/// (restart, the report returns), alt-0 (Here).
#[test]
#[ignore = "holla h5 check; run with --ignored"]
fn h5_parity_upgrade_managers() {
    const OUTDATED: &str = "Outdated Formulae · 2";
    let dir = h5_dir("h5_parity_upgrade_managers");
    let case = h5_case_t("parity_upgrade_managers", UPGRADERS, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    // The explore shelf animates in after the suggestion rows; settle
    // it before the boot asserts so V1 cannot race the stagger.
    waits::wait_state(&mut s, "›Tasks", "explore settled");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the explore row lists the entry points with Tasks first.
    assert_line_has(&boot, "›Tasks", "Services", "V1 explore");
    // V2: the recent shelf defers to the suggestions at boot.
    assert!(
        boot.contains("everything used here is suggested above"),
        "V2: no suggested-above recent"
    );
    // S1: the facts scope the world to here.
    assert_line_has(&boot, "Scope", "here", "S1 scope");

    // A1: running the upgrade lands on the outdated-formulae report.
    // The report line arrives while the run is still `running`, so the
    // drive gates the member title and then the settled batch footer —
    // not the bare exit.
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

    // V3-flow: the output reports the outdated formulae.
    assert!(
        batch.contains("==> Outdated Formulae · 2"),
        "flow: no outdated line"
    );
    // S2: the footer summarizes the batch outcome.
    assert!(
        batch.contains("! Upgrade Homebrew packages · 4 ok · 1 failed · 0 cancelled"),
        "S2: no batch footer"
    );
    // N1: the empty-query placeholder is gone on the output.
    assert!(!batch.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: r restarts the script, the report returns.
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
    eprintln!("h5 parity_upgrade_managers: boot, batch + restart roundtrip");
}

/// PANEL-FINDER-003 (`holla/audit/rust`): the rust-dirty audit boot plus
/// the tests filter behavior.
///
/// Flow: boot, type `tests` (results), Esc (cleared), type `tests`
/// (results again).
#[test]
#[ignore = "holla h5 check; run with --ignored"]
fn h5_audit_rust() {
    let dir = h5_dir("h5_audit_rust");
    let case = h5_case("audit_rust", RUST_P, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the top suggestion pairs the disk probe with its badge.
    assert_line_has(
        &boot,
        "› Inspect 12 GB",
        "12 GB generated artifacts",
        "V1 row",
    );
    // V2: the git row pairs the pull with its behind count.
    assert_line_has(&boot, "Pull", "3 commits behind", "V2 row");
    // S1: the facts anchor the holla folder.
    assert_line_has(&boot, "In", "~/work/holla", "S1 world");
    // S2: the status line pins the branch drift.
    assert_line_has(&boot, "main", "↓3 • 5", "S2 status");

    // A1: typing filters to the tests task.
    support::drive(
        &mut s,
        &["type:tests", "wait:Results ·"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    // V3-flow: the filtered row pairs the task with its kind.
    assert_line_has(&res, "› Run tests", "task", "flow row");
    // N1: the empty-query placeholder is gone with the results.
    assert!(!res.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: Esc clears the query back to the placeholder.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Results ·", "results cleared");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);
    // N2: the result count is gone after the clear.
    assert!(!cleared.contains("Results ·"), "N2: results stuck");
    support::drive(
        &mut s,
        &["type:tests", "wait:Results ·"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-results-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "› Run tests", "task", "row intact");
    eprintln!("h5 audit_rust: boot, filter roundtrip");
}

/// PANEL-UPGRADE-003 (`holla/audit/upgrade`): the upgrade-plan audit boot
/// plus the step-cursor behavior.
///
/// Flow: boot, Down (cursor on step 02), Up (cursor back on step 01).
#[test]
#[ignore = "holla h5 check; run with --ignored"]
fn h5_audit_upgrade() {
    const STEP1: &str = "▎[✓] 01 Preflight";
    const STEP2: &str = "▎[✓] 02 Refresh Debian metadata";
    let dir = h5_dir("h5_audit_upgrade");
    let case = h5_case("audit_upgrade", UPGRADE_P, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);

    // V1: the plan header counts the full 8-step plan.
    assert_line_has(
        &boot,
        "Upgrade everything · devbox",
        "8 steps · 8 included",
        "V1 header",
    );
    // V2: the boot cursor sits on the first step.
    assert_line_has(&boot, STEP1, "sudo", "V2 cursor");
    // S1: the privilege fact.
    assert_line_has(&boot, "Privilege", "sudo", "S1 privilege");
    // S2: the ready state.
    assert_line_has(&boot, "State", "ready", "S2 state");

    // A1: Down moves the step cursor to step 02.
    support::drive(
        &mut s,
        &["down", &format!("wait:{STEP2}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-moved");
    let moved = live_text(&mut s);

    // V3-flow: the cursor row pairs the step with its note.
    assert_line_has(&moved, STEP2, "sudo", "flow cursor");
    // N1: the step-01 cursor is gone with the move.
    assert!(!moved.contains(STEP1), "N1: step-01 cursor stuck");

    // A2: Up returns the cursor to step 01.
    support::drive(
        &mut s,
        &["up", &format!("wait:{STEP1}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-back");
    let back = live_text(&mut s);
    // N2: the step-02 cursor is gone after the return.
    assert!(!back.contains(STEP2), "N2: step-02 cursor stuck");
    assert_line_has(&back, STEP1, "sudo", "cursor intact");
    eprintln!("h5 audit_upgrade: boot, cursor roundtrip");
}

/// PANEL-FINDER-004 (`holla/concept/activities-multi`): the multi-activity
/// finder boot plus the preview filter behavior.
///
/// Flow: boot, type `preview` (results), Esc (cleared), type `preview`
/// (results again).
#[test]
#[ignore = "holla h5 check; run with --ignored"]
fn h5_concept_activities_multi() {
    let dir = h5_dir("h5_concept_activities_multi");
    let case = h5_case_t("concept_activities_multi", ACTMULTI, BOOT, 20_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the tab strip carries all five activities.
    assert_line_has(&boot, "frontend dev ⠋ ×", "btm ○ ×", "V1 strip");
    // V2: the running shelf pairs the worker with its tick age.
    assert_line_has(&boot, "frontend dev", "running · 3 s", "V2 running");
    // S1: the facts anchor the acme folder.
    assert_line_has(&boot, "In", "~/work/acme", "S1 world");
    // S2: the status line counts the live activities.
    assert!(boot.contains("4 running"), "S2: no running count");

    // A1: typing filters to the preview action.
    support::drive(
        &mut s,
        &["type:preview", "wait:Results ·"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    // V3-flow: the filtered row pairs the action with its folder.
    assert_line_has(&res, "› Deploy preview", "Current folder", "flow row");
    // N1: the empty-query placeholder is gone with the results.
    assert!(!res.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: Esc clears the query back to the placeholder.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Results ·", "results cleared");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);
    // N2: the result count is gone after the clear.
    assert!(!cleared.contains("Results ·"), "N2: results stuck");
    support::drive(
        &mut s,
        &["type:preview", "wait:Results ·"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-results-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "› Deploy preview", "Current folder", "row intact");
    eprintln!("h5 concept_activities_multi: boot, filter roundtrip");
}

/// PANEL-DISK-003 (`holla/concept/disk-cleanup`): the Disk page boot plus
/// the sort-toggle behavior.
///
/// Flow: boot, s (apparent), s (allocated back).
#[test]
#[ignore = "holla h5 check; run with --ignored"]
fn h5_concept_disk_cleanup() {
    const ALLOC: &str = "largest first · allocated";
    const APPARENT: &str = "largest first · apparent";
    let dir = h5_dir("h5_concept_disk_cleanup");
    let case = h5_case("concept_disk_cleanup", DISK_P, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);

    // V1: the scan meta counts the finished scan.
    assert!(
        boot.contains("scan complete · 31.8 GiB allocated · 35 entries · 3 selected (24.2 GiB)"),
        "V1: no scan meta"
    );
    // V2: the sort row names the allocated order.
    assert_line_has(&boot, ALLOC, "noise folded", "V2 sort");
    // S1: the entries fact counts hardlinks once.
    assert_line_has(
        &boot,
        "Entries",
        "35 · hardlinks counted once",
        "S1 entries",
    );
    // S2: the footer carries the selection.
    assert!(
        boot.contains("3 selected · 24.2 GiB"),
        "S2: no selection footer"
    );

    // A1: s toggles the sort to apparent.
    support::drive(
        &mut s,
        &["s", &format!("wait:{APPARENT}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-apparent");
    let apparent = live_text(&mut s);

    // V3-flow: the sort row names the apparent order.
    assert_line_has(&apparent, APPARENT, "noise folded", "flow sort");
    // N1: the allocated order is gone with the toggle.
    assert!(!apparent.contains(ALLOC), "N1: allocated order stuck");

    // A2: s toggles the sort back to allocated.
    support::drive(
        &mut s,
        &["s", &format!("wait:{ALLOC}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-allocated");
    let back = live_text(&mut s);
    // N2: the apparent order is gone after the toggle back.
    assert!(!back.contains(APPARENT), "N2: apparent order stuck");
    assert_line_has(&back, ALLOC, "noise folded", "sort intact");
    eprintln!("h5 concept_disk_cleanup: boot, sort toggle roundtrip");
}

/// PANEL-DOCKER-003 (`holla/concept/docker-cleanup`): the pre-typed docker
/// query boot plus the clear/retype behavior.
///
/// Flow: boot (query typed), Esc (cleared), type `docker clean`
/// (results again).
#[test]
#[ignore = "holla h5 check; run with --ignored"]
fn h5_concept_docker_cleanup() {
    const RESULTS: &str = "Results · 7";
    let dir = h5_dir("h5_concept_docker_cleanup");
    let case = h5_case("concept_docker_cleanup", DOCKER_P, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);

    // V1: the top row pairs the plan with its risk.
    assert_line_has(&boot, "› Clean Docker completely", "destructive", "V1 row");
    // V2: the result count.
    assert!(boot.contains(RESULTS), "V2: no Results · 7");
    // S1: the scope readout names the devbox.
    assert_line_has(&boot, "Scope", "on devbox", "S1 scope");
    // S2: the preview names the two-gate contract.
    assert_line_has(&boot, "Gate", "two gates", "S2 gate");

    // A1: Esc clears the pre-typed query.
    support::drive(
        &mut s,
        &["escape", &format!("wait:{PLACEHOLDER}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-cleared");
    let cleared = live_text(&mut s);

    // V3-flow: the suggestions return with the clear.
    assert!(cleared.contains("Suggested"), "flow: no suggestions back");
    // N1: the result count is gone after the clear.
    assert!(!cleared.contains(RESULTS), "N1: results stuck");

    // A2: retyping restores the docker results.
    support::drive(
        &mut s,
        &["type:docker clean", &format!("wait:{RESULTS}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-results-again");
    let back = live_text(&mut s);
    // N2: the placeholder is gone on the results.
    assert!(!back.contains(PLACEHOLDER), "N2: placeholder stuck");
    assert_line_has(
        &back,
        "› Clean Docker completely",
        "destructive",
        "row intact",
    );
    eprintln!("h5 concept_docker_cleanup: boot, clear/retype roundtrip");
}

/// PANEL-FINDER-005 (`holla/concept/first-use`): the first-use finder boot
/// plus the btm filter behavior.
///
/// Flow: boot, type `btm` (results), Esc (cleared), type `btm`
/// (results again).
#[test]
#[ignore = "holla h5 check; run with --ignored"]
fn h5_concept_first_use() {
    let dir = h5_dir("h5_concept_first_use");
    let case = h5_case("concept_first_use", FIRSTUSE, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the top suggestion pairs the snapshot with its pressure badge.
    assert_line_has(
        &boot,
        "› Show system resources",
        "CPU 78% for 4 min",
        "V1 row",
    );
    // V2: the disk probe lists below.
    assert!(boot.contains("Analyze disk usage"), "V2: no disk probe");
    // S1: the scope readout stays host.
    assert_line_has(&boot, "Scope", "host", "S1 scope");
    // S2: the preview runs directly.
    assert_line_has(&boot, "Gate", "runs directly", "S2 gate");

    // A1: typing filters to the btm action.
    support::drive(&mut s, &["type:btm", "wait:Results ·"], case_timeout(&case));
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    // V3-flow: the filtered row pairs the monitor with its badge.
    assert_line_has(&res, "› Open btm", "sustained pressure", "flow row");
    // N1: the empty-query placeholder is gone with the results.
    assert!(!res.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: Esc clears the query back to the placeholder.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Results ·", "results cleared");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);
    // N2: the result count is gone after the clear.
    assert!(!cleared.contains("Results ·"), "N2: results stuck");
    support::drive(&mut s, &["type:btm", "wait:Results ·"], case_timeout(&case));
    checkpoint(&mut s, &dir, "03-results-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "› Open btm", "sustained pressure", "row intact");
    eprintln!("h5 concept_first_use: boot, filter roundtrip");
}

/// PANEL-FINDER-006 (`holla/concept/hard-cases`): the hard-cases finder
/// boot plus the cleanup filter behavior.
///
/// Flow: boot, type `cleanup` (results), Esc (cleared), type `cleanup`
/// (results again).
#[test]
#[ignore = "holla h5 check; run with --ignored"]
fn h5_concept_hard_cases() {
    let dir = h5_dir("h5_concept_hard_cases");
    let case = h5_case("concept_hard_cases", HARD, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the top suggestion pairs the disk probe with its badge.
    assert_line_has(&boot, "› Analyze disk usage", "/ is 91% full", "V1 row");
    // V2: the explore shelf counts the child projects.
    assert!(
        boot.contains("Children · 14 projects"),
        "V2: no child count"
    );
    // S1: the status line names the unreachable backends.
    assert!(
        boot.contains("docker, github unavailable"),
        "S1: no backend warning"
    );
    // S2: the scope readout stays here.
    assert_line_has(&boot, "Scope", "here", "S2 scope");

    // A1: typing filters to the cleanup review.
    support::drive(
        &mut s,
        &["type:cleanup", "wait:Results ·"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    // V3-flow: the filtered row pairs the review with its badge.
    assert_line_has(&res, "› Review cleanup candidates", "8.0 GiB", "flow row");
    // N1: the empty-query placeholder is gone with the results.
    assert!(!res.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: Esc clears the query back to the placeholder.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Results ·", "results cleared");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);
    // N2: the result count is gone after the clear.
    assert!(!cleared.contains("Results ·"), "N2: results stuck");
    support::drive(
        &mut s,
        &["type:cleanup", "wait:Results ·"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-results-again");
    let back = live_text(&mut s);
    assert_line_has(
        &back,
        "› Review cleanup candidates",
        "8.0 GiB",
        "row intact",
    );
    eprintln!("h5 concept_hard_cases: boot, filter roundtrip");
}

/// PANEL-TASK-005 (`holla/concept/launch-failure`): the failed child-task
/// output plus the Here/jump behavior.
///
/// Flow: boot (failed output), Esc (Here), alt-1 (output again).
#[test]
#[ignore = "holla h5 check; run with --ignored"]
fn h5_concept_launch_failure() {
    const EADDR: &str = "EADDRINUSE";
    const TITLE: &str = "frontend dev · failed";
    let dir = h5_dir("h5_concept_launch_failure");
    let case = h5_case("concept_launch_failure", LAUNCHFAIL, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);

    // V1: the output names the port conflict.
    assert!(
        boot.contains("Error: listen EADDRINUSE: address already in use :::5173"),
        "V1: no port error"
    );
    // V2: the title pairs the task with its failed exit.
    assert_line_has(&boot, TITLE, "exit 1", "V2 title");
    // S1: the follow-up row offers the port probe and the retry.
    assert_line_has(
        &boot,
        "Find the process on port 5173",
        "Retry frontend dev",
        "S1 next",
    );
    // S2: the footer records the failure.
    assert!(
        boot.contains("frontend dev failed · exit 1"),
        "S2: no failure footer"
    );

    // A1: Esc leaves the output for Here.
    support::drive(
        &mut s,
        &["escape", &format!("wait:{PLACEHOLDER}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-here");
    let here = live_text(&mut s);

    // V3-flow: the finder scope readout is back.
    assert!(here.contains("scope ‹ here ›"), "flow: no here scope");
    // N1: the port error is gone off the output.
    assert!(!here.contains(EADDR), "N1: port error stuck");

    // A2: alt-1 jumps back to the failed output.
    support::drive(
        &mut s,
        &["alt-1", &format!("wait:{EADDR}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-output-again");
    let back = live_text(&mut s);
    // N2: the placeholder is gone on the output.
    assert!(!back.contains(PLACEHOLDER), "N2: placeholder stuck");
    assert_line_has(&back, TITLE, "exit 1", "title intact");
    eprintln!("h5 concept_launch_failure: boot, Here/jump roundtrip");
}

/// PANEL-CHILD-002 (`holla/concept/monorepo-child`): the monorepo-child
/// finder boot plus the tests filter behavior.
///
/// Flow: boot, type `test` (results), Esc (cleared), type `test`
/// (results again).
#[test]
#[ignore = "holla h5 check; run with --ignored"]
fn h5_concept_monorepo_child() {
    let dir = h5_dir("h5_concept_monorepo_child");
    let case = h5_case("concept_monorepo_child", CHILD_P, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the top suggestion pairs the disk probe with its badge.
    assert_line_has(
        &boot,
        "› Inspect 3 GB",
        "3 GB generated artifacts",
        "V1 row",
    );
    // V2: the parent shelf names the root.
    assert!(
        boot.contains("From acme · the parent root"),
        "V2: no parent shelf"
    );
    // S1: the facts anchor the frontend folder.
    assert_line_has(&boot, "In", "~/work/acme/apps", "S1 world");
    // S2: the scope readout stays here.
    assert_line_has(&boot, "Scope", "here", "S2 scope");

    // A1: typing filters to the child results.
    support::drive(
        &mut s,
        &["type:test", "wait:Results · 17"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    // V3-flow: the top row pairs the task with its kind.
    assert_line_has(&res, "› Run tests", "task", "flow row");
    // N1: the empty-query placeholder is gone with the results.
    assert!(!res.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: Esc clears the query back to the placeholder.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Results · 17", "results cleared");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);
    // N2: the result count is gone after the clear.
    assert!(!cleared.contains("Results · 17"), "N2: results stuck");
    support::drive(
        &mut s,
        &["type:test", "wait:Results · 17"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-results-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "› Run tests", "task", "row intact");
    eprintln!("h5 concept_monorepo_child: boot, filter roundtrip");
}

/// PANEL-FINDER-007 (`holla/concept/monorepo-root`): the monorepo-root
/// finder boot plus the scope-widen behavior.
///
/// Flow: boot, ctrl-down (children), ctrl-up (here), ctrl-down
/// (children again).
#[test]
#[ignore = "holla h5 check; run with --ignored"]
fn h5_concept_monorepo_root() {
    let dir = h5_dir("h5_concept_monorepo_root");
    let case = h5_case("concept_monorepo_root", ROOT_P, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the top suggestion pairs the preview with its trust badge.
    assert_line_has(&boot, "› Deploy preview", "unreviewed", "V1 row");
    // V2: the explore shelf counts the child projects.
    assert!(boot.contains("Children · 3 projects"), "V2: no child count");
    // S1: the preview names the trust gate.
    assert_line_has(&boot, "Gate", "trust required", "S1 gate");
    // S2: the scope readout stays here.
    assert_line_has(&boot, "Scope", "here", "S2 scope");

    // A1: Ctrl+Down widens to the children scope.
    support::drive(
        &mut s,
        &["ctrl-down", "wait:Children · 3 projects"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-children");
    let children = live_text(&mut s);

    // V3-flow: the header counts the child projects with their entries.
    assert!(
        children.contains("Children · 3 projects · 11 entries"),
        "flow: no children header"
    );
    // N1: the here readout is gone under children.
    assert!(!children.contains("scope ‹ here ›"), "N1: here scope stuck");

    // A2: Ctrl+Up narrows back to here, Ctrl+Down widens again.
    support::drive(
        &mut s,
        &["ctrl-up", "wait:scope ‹ here ›"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-here");
    let here = live_text(&mut s);
    // N2: the children readout is gone after narrowing.
    assert!(
        !here.contains("scope ‹ children ›"),
        "N2: children readout stuck"
    );
    support::drive(
        &mut s,
        &["ctrl-down", "wait:Children · 3 projects"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-children-again");
    let back = live_text(&mut s);
    assert!(
        back.contains("Children · 3 projects · 11 entries"),
        "header intact"
    );
    eprintln!("h5 concept_monorepo_root: boot, scope roundtrip");
}

/// PANEL-REMOTE-002 (`holla/concept/remote-host`): the remote-host finder
/// boot plus the restart filter behavior.
///
/// Flow: boot, type `restart payments` (results), Esc (cleared),
/// type `restart payments` (results again).
#[test]
#[ignore = "holla h5 check; run with --ignored"]
fn h5_concept_remote_host() {
    let dir = h5_dir("h5_concept_remote_host");
    let case = h5_case("concept_remote_host", REMOTE_P, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the top suggestion pairs the journal tail with its restart state.
    assert_line_has(
        &boot,
        "› Follow payments-worker",
        "unit is restarting",
        "V1 row",
    );
    // V2: the explore shelf names the production host.
    assert!(boot.contains("System · prod-eu-1"), "V2: no host shelf");
    // S1: the facts name the production host.
    assert_line_has(&boot, "Host", "prod-eu-1", "S1 host");
    // S2: the scope readout names the host scope.
    assert_line_has(&boot, "Scope", "on prod-eu-1", "S2 scope");

    // A1: typing filters to the remote restart actions.
    support::drive(
        &mut s,
        &["type:restart payments", "wait:Results · 2"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    // V3-flow: the top row pairs the restart with its privilege.
    assert_line_has(&res, "› Restart payments", "privileged", "flow row");
    // N1: the empty-query placeholder is gone with the results.
    assert!(!res.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: Esc clears the query back to the placeholder.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Results · 2", "results cleared");
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
    eprintln!("h5 concept_remote_host: boot, filter roundtrip");
}

/// Bounded wait for the screen text to equal `want` (the wheel-back
/// stance: scrolling back up restores the pre-wheel text byte-exactly).
/// Returns the matched sample so the stance asserts read the proven
/// frame instead of a fresh (possibly next-tick) sample.
fn wait_text_eq(s: &mut Session, want: &str, timeout: Duration, what: &str) -> String {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        let cur = live_text(s);
        if cur == want {
            return cur;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "{what}: screen never matched"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

/// PANEL-BROWSER-002 (`holla/fade/browser_wheel`): the browser preview
/// drawer wheel-scrolled plus its top-edge fade.
///
/// Flow: boot, open the site listing, select big.log (preview), wheel
/// down 2 (scrolled + faded), wheel up 2 (top restored).
#[test]
#[ignore = "holla h5 check; run with --ignored"]
fn h5_fade_browser_wheel() {
    const PREVIEW_COLS: (u16, u16) = (70, 115);
    let dir = h5_dir("h5_fade_browser_wheel");
    let case = h5_case("fade_browser_wheel", BROWSER, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::drive(
        &mut s,
        &[
            "type:Browse ~/work/site",
            "enter",
            "wait:16 entries",
            "down",
            "down",
            "down",
            "wait:first 2000 lines",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "00-listing");
    let pre = live_text(&mut s);
    let pre_frame = live_frame(&mut s);
    assert!(pre.contains("16 entries"), "pre: no 16 entries");
    assert!(pre.contains("line 5"), "pre: no line 5 target");
    assert!(!pre.contains("line 33"), "pre: already scrolled");
    let (header_row, _) = find_pos(&mut s, "first 2000 lines", case_timeout(&case));

    // A1: two down-notches scroll the preview drawer.
    let (wrow, wcol) = wheel_over(&mut s, "line 5", Wheel::Down, 2, case_timeout(&case));
    let post = wait_changed(&mut s, &pre, case_timeout(&case), "preview scrolled");
    checkpoint(&mut s, &dir, "01-scrolled");
    let frame = live_frame(&mut s);

    // V1: the scrolled top line, the surfaced bottom line, and the title.
    assert!(post.contains("7 line 6"), "V1: no scrolled top line");
    assert!(post.contains("line 33"), "V1: no surfaced bottom line");
    assert!(post.contains("first 2000 lines"), "V1: no preview title");
    // V2: the top preview edge fades; the interior does not.
    let (edge_row, _) = find_pos(&mut s, "7 line 6", case_timeout(&case));
    let edge = row_fg_median_cols(&frame, edge_row, PREVIEW_COLS.0, PREVIEW_COLS.1, "V2 edge");
    let inner = row_fg_median_cols(
        &frame,
        edge_row + 8,
        PREVIEW_COLS.0,
        PREVIEW_COLS.1,
        "V2 interior",
    );
    eprintln!("h5 browser_wheel fade: edge lum {edge}, interior lum {inner}");
    assert_faded(edge, inner, "V2 top edge");
    // S1: the listing keeps its cursor on big.log.
    assert_line_has(&post, "▎", "big.log", "S1 cursor");
    // S2: the listing meta is untouched.
    assert!(
        post.contains("16 entries · 2 hidden hidden"),
        "S2: no listing meta"
    );
    // N1: the preview header never fades.
    assert_row_same(&pre_frame, &frame, header_row, "N1 header never fades");

    // A2: two up-notches restore the pre-wheel text byte-exactly.
    wheel_at(&mut s, wcol, wrow, Wheel::Up, 2);
    let top = wait_text_eq(&mut s, &pre, case_timeout(&case), "back to top");
    checkpoint(&mut s, &dir, "02-back-top");
    // N2: the surfaced bottom line is gone after wheeling back.
    assert!(!top.contains("line 33"), "N2: surfaced line stuck");
    assert!(top.contains("line 5"), "top target back");
    eprintln!("h5 fade_browser_wheel: scroll + fade, back to top");
}

/// PANEL-CLEANUP-008 (`holla/fade/cleanup_list_wheel`): the cleanup
/// categories list wheel-scrolled plus its top-edge fade.
///
/// Flow: boot, open the cleanup review, wheel down 2 over the list
/// (scrolled + faded), wheel up 2 (top restored).
#[test]
#[ignore = "holla h5 check; run with --ignored"]
fn h5_fade_cleanup_list_wheel() {
    const LIST_COLS: (u16, u16) = (2, 71);
    let dir = h5_dir("h5_fade_cleanup_list_wheel");
    let case = h5_case("fade_cleanup_list_wheel", INSIGHTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::drive(
        &mut s,
        &[
            "type:Review cleanup candidates",
            "enter",
            "wait:18 categories",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "00-review");
    let pre = live_text(&mut s);
    let pre_frame = live_frame(&mut s);
    assert!(pre.contains("18 categories"), "pre: no 18 categories");
    assert!(pre.contains("Yarn cache"), "pre: no Yarn cache target");
    assert!(!pre.contains("JetBrains logs"), "pre: already scrolled");
    let (header_row, _) = find_pos(&mut s, "Cleanup review", case_timeout(&case));

    // A1: two down-notches scroll the categories list. Rows are
    // clickable children and shadow the list's scroll region, so the
    // wheel lands on the blank line below the needle (the matrix
    // technique).
    wheel_below(&mut s, "Yarn cache", 2, case_timeout(&case));
    let post = wait_changed(&mut s, &pre, case_timeout(&case), "list scrolled");
    checkpoint(&mut s, &dir, "01-scrolled");
    let frame = live_frame(&mut s);

    // V1: the review meta and the surfaced bottom category.
    assert!(post.contains("18 categories"), "V1: no categories meta");
    assert!(post.contains("JetBrains logs"), "V1: no surfaced category");
    // V2: the top list edge fades; the interior does not; the thumb
    // sits on the scrolled body. The interior row carries the same
    // checkbox and badge as the edge row, so only the fade can separate
    // their tones.
    let (edge_row, _) = find_pos(&mut s, "Xcode archives", case_timeout(&case));
    let (inner_row, _) = find_pos(&mut s, "Maven repository", case_timeout(&case));
    let edge = row_fg_median_cols(&frame, edge_row, LIST_COLS.0, LIST_COLS.1, "V2 edge");
    let inner = row_fg_median_cols(&frame, inner_row, LIST_COLS.0, LIST_COLS.1, "V2 interior");
    eprintln!("h5 cleanup_list_wheel fade: edge lum {edge}, interior lum {inner}");
    assert_faded(edge, inner, "V2 top edge");
    assert_line_has(&post, "Yarn cache", "┃", "V2 thumb");
    // S1: the selection count is untouched.
    assert!(
        post.contains("19 selected · 12.4 GiB"),
        "S1: no selection count"
    );
    // S2: the detail panel keeps its category.
    assert!(post.contains("Xcode DerivedData"), "S2: no detail category");
    // N1: the review title never fades.
    assert_row_same(&pre_frame, &frame, header_row, "N1 title never fades");

    // A2: two up-notches restore the pre-wheel text byte-exactly.
    let (wrow, wcol) = find_pos(&mut s, "Yarn cache", case_timeout(&case));
    wheel_at(&mut s, wcol, wrow + 1, Wheel::Up, 2);
    let top = wait_text_eq(&mut s, &pre, case_timeout(&case), "back to top");
    checkpoint(&mut s, &dir, "02-back-top");
    // N2: the surfaced category is gone after wheeling back.
    assert!(!top.contains("JetBrains logs"), "N2: surfaced row stuck");
    assert!(top.contains("Xcode device support"), "top rows back");
    eprintln!("h5 fade_cleanup_list_wheel: scroll + fade, back to top");
}

/// PANEL-TASK-006 (`holla/fade/executor_burst-end`): the burst output tail
/// with its retention report plus the restart behavior.
///
/// Flow: boot, run the burst to the dropped-lines report, r (restart,
/// the report returns), Esc (Here).
#[test]
#[ignore = "holla h5 check; run with --ignored"]
fn h5_fade_executor_burst_end() {
    const DONE: &str = "emit a burst finished · exit 0";
    const TAIL: &str = "burst line 4499";
    let dir = h5_dir("h5_fade_executor_burst_end");
    let case = h5_case_t("fade_executor_burst_end", EXECUTOR, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    // The suggestion rows animate in after the header; settle the top
    // row and the shelf before the boot asserts so V1/V2 cannot race
    // the stagger under full-suite load.
    waits::wait_state(&mut s, "› Status across 3 child", "top row settled");
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
    checkpoint(&mut s, &dir, "01-tail");
    let tail = live_text(&mut s);

    // V1-end: the retention header states the drop and the kept tail.
    assert!(
        tail.contains("500 earlier lines dropped"),
        "V1-end: no drop report"
    );
    assert!(tail.contains("last 4000 kept"), "V1-end: no kept count");
    // V2-end: the tail sits at the bottom of the output.
    assert!(tail.contains(TAIL), "V2-end: no last burst line");
    assert!(
        tail.contains("burst line 4472"),
        "V2-end: no first tail line"
    );
    // S2: the title pairs the burst with its clean exit.
    assert_line_has(&tail, "emit a burst · succeeded", "exit 0", "S2 title");
    // N1: the empty-query placeholder is gone on the output.
    assert!(!tail.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: r restarts the script, the report returns.
    support::press_step(&s, "r");
    wait_absent(&mut s, DONE, "output restarted");
    waits::wait_state(&mut s, DONE, "burst finished again");
    checkpoint(&mut s, &dir, "02-tail-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "emit a burst · succeeded", "exit 0", "title intact");
    assert!(back.contains(TAIL), "tail intact");
    support::press_step(&s, "escape");
    wait_absent(&mut s, "burst line", "back to Here");
    waits::wait_state(&mut s, "scope ‹ here ›", "finder back");
    checkpoint(&mut s, &dir, "03-here");
    let here = live_text(&mut s);
    // N2: the burst body is gone after leaving the executor.
    assert!(!here.contains("burst line"), "N2: burst body stuck");
    eprintln!("h5 fade_executor_burst_end: boot, burst + restart roundtrip");
}

/// PANEL-TRUST-003 (`holla/fade/trust_body_wheel`): the trust definition
/// body wheel-scrolled plus its top-edge fade.
///
/// Flow: boot, open the trust review, wheel down 2 over the definition
/// (scrolled + faded), wheel up 2 (top restored). Never trusted: the
/// review only ever cancels.
#[test]
#[ignore = "holla h5 check; run with --ignored"]
fn h5_fade_trust_body_wheel() {
    const BODY_COLS: (u16, u16) = (2, 115);
    const TRUST: &str = "Trust ~/work/team/.holla.toml?";
    let dir = h5_dir("h5_fade_trust_body_wheel");
    let case = h5_case("fade_trust_body_wheel", CUSTOM, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::drive(
        &mut s,
        &["type:Deploy preview", "enter", &format!("wait:{TRUST}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "00-review");
    let pre = live_text(&mut s);
    let pre_frame = live_frame(&mut s);
    assert!(pre.contains(TRUST), "pre: no trust title");
    assert!(
        pre.contains("id = \"deploy.preview\""),
        "pre: no wheel target"
    );
    assert!(!pre.contains("id = \"shared\""), "pre: already scrolled");
    let (header_row, _) = find_pos(&mut s, TRUST, case_timeout(&case));

    // A1: two down-notches scroll the definition body.
    let (wrow, wcol) = wheel_over(
        &mut s,
        "id = \"deploy.preview\"",
        Wheel::Down,
        2,
        case_timeout(&case),
    );
    let post = wait_changed(&mut s, &pre, case_timeout(&case), "body scrolled");
    checkpoint(&mut s, &dir, "01-scrolled");
    let frame = live_frame(&mut s);

    // V1: the scrolled body surfaces the later actions.
    assert!(post.contains("id = \"shared\""), "V1: no shared action row");
    assert!(post.contains("Definition"), "V1: no definition title");
    // V2: the top body edge fades; the interior does not.
    let (edge_row, _) = find_pos(&mut s, "description = ", case_timeout(&case));
    let edge = row_fg_median_cols(&frame, edge_row, BODY_COLS.0, BODY_COLS.1, "V2 edge");
    let inner = row_fg_median_cols(
        &frame,
        edge_row + 8,
        BODY_COLS.0,
        BODY_COLS.1,
        "V2 interior",
    );
    eprintln!("h5 trust_body_wheel fade: edge lum {edge}, interior lum {inner}");
    assert_faded(edge, inner, "V2 top edge");
    // S1: the facts keep the action census.
    assert_line_has(&post, "Defines", "3 custom actions", "S1 defines");
    // S2: Cancel stays the focused choice.
    assert_line_has(&post, "▎Cancel", "Trust this file", "S2 choice");
    // N1: the trust title never fades.
    assert_row_same(&pre_frame, &frame, header_row, "N1 title never fades");

    // A2: two up-notches restore the pre-wheel text byte-exactly.
    wheel_at(&mut s, wcol, wrow, Wheel::Up, 2);
    let top = wait_text_eq(&mut s, &pre, case_timeout(&case), "back to top");
    checkpoint(&mut s, &dir, "02-back-top");
    // N2: the scrolled shared row is gone after wheeling back.
    assert!(!top.contains("id = \"shared\""), "N2: shared row stuck");
    assert!(top.contains("id = \"deploy.preview\""), "top target back");
    eprintln!("h5 fade_trust_body_wheel: scroll + fade, back to top");
}

/// PICKER-ACTIVITIES-001 (`holla/flows/activities/empty`): the activities
/// picker with nothing started plus the open/cancel behavior.
///
/// Flow: boot (first-use), ctrl-g (empty picker), Esc (cancelled),
/// ctrl-g (picker again).
#[test]
#[ignore = "holla h5 check; run with --ignored"]
fn h5_activities_empty() {
    const EMPTY: &str = "Nothing has been started yet";
    const FOCUSED_FIND: &str = "▎ Search actions and resources…";
    let dir = h5_dir("h5_activities_empty");
    let case = h5_case("activities_empty", FIRSTUSE, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(FOCUSED_FIND), "pre: finder not focused");

    // A1: ctrl-g opens the empty activities picker.
    support::drive(
        &mut s,
        &["ctrl-g", &format!("wait:{EMPTY}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-picker");
    let picker = live_text(&mut s);

    // V1: the picker titles the empty live count.
    assert_line_has(&picker, "Activities", "0 live", "V1 title");
    // V2: the empty state names itself.
    assert!(picker.contains(EMPTY), "V2: no empty state");
    // S1: the footer offers the switch gesture.
    assert!(
        picker.contains("↑↓ Move  Enter Switch  Esc Cancel"),
        "S1: no picker footer"
    );
    // S2: the finder shelf behind keeps its suggestion.
    assert!(
        picker.contains("Show system resources"),
        "S2: no shelf behind"
    );
    // N1: the finder focus bar is gone with the picker open.
    assert!(!picker.contains(FOCUSED_FIND), "N1: finder focus stuck");

    // A2: Esc cancels the picker, ctrl-g re-opens it.
    support::press_step(&s, "escape");
    wait_absent(&mut s, EMPTY, "picker cancelled");
    waits::wait_state(&mut s, FOCUSED_FIND, "finder back");
    checkpoint(&mut s, &dir, "02-finder");
    let finder = live_text(&mut s);
    // N2: the live count is gone after the cancel.
    assert!(!finder.contains("0 live"), "N2: live count stuck");
    support::drive(
        &mut s,
        &["ctrl-g", &format!("wait:{EMPTY}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-picker-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "Activities", "0 live", "picker intact");
    eprintln!("h5 activities_empty: picker, cancel roundtrip");
}

/// PANEL-TASK-007 (`holla/flows/activities/jump`): the activity-tab jump
/// plus the Here/jump behavior.
///
/// Flow: boot (activities-multi), alt-1 (frontend dev output), Esc
/// (Here), alt-1 (output again).
#[test]
#[ignore = "holla h5 check; run with --ignored"]
fn h5_activities_jump() {
    const TITLE: &str = "frontend dev · running";
    const VITE: &str = "VITE v6.3.1  ready in 412 ms";
    let dir = h5_dir("h5_activities_jump");
    let case = h5_case_t("activities_jump", ACTMULTI, BOOT, 20_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // V1: the tab strip carries all five activities.
    assert_line_has(&boot, "frontend dev ⠋ ×", "btm ○ ×", "V1 strip");
    // V2: the cursor sits on the preview action.
    assert_line_has(&boot, "› Deploy preview", "Current folder", "V2 cursor");
    // S1: the status line counts the live activities.
    assert!(boot.contains("4 running"), "S1: no running count");
    // S2: the preview names the trust gate.
    assert_line_has(&boot, "Gate", "trust required", "S2 gate");

    // A1: alt-1 jumps to the frontend dev output.
    support::drive(
        &mut s,
        &["alt-1", &format!("wait:{TITLE}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-output");
    let output = live_text(&mut s);

    // V3-flow: the output shows the ready server.
    assert!(output.contains(VITE), "flow: no vite ready line");
    // S2-flow: the title pins the running age.
    assert_line_has(&output, TITLE, "3 s", "flow title");
    // N1: the empty-query placeholder is gone on the output.
    assert!(!output.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: Esc returns to Here, alt-1 jumps back to the output.
    support::press_step(&s, "escape");
    wait_absent(&mut s, TITLE, "back to Here");
    waits::wait_state(&mut s, PLACEHOLDER, "finder back");
    checkpoint(&mut s, &dir, "02-here");
    let here = live_text(&mut s);
    // N2: the vite line is gone after leaving the output.
    assert!(!here.contains(VITE), "N2: vite line stuck");
    support::drive(
        &mut s,
        &["alt-1", &format!("wait:{TITLE}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-output-again");
    let back = live_text(&mut s);
    assert_line_has(&back, TITLE, "3 s", "title intact");
    eprintln!("h5 activities_jump: boot, Here/jump roundtrip");
}

/// PICKER-ACTIVITIES-002 (`holla/flows/activities/overlay`): the live
/// activities picker plus the open/cancel behavior.
///
/// Flow: boot (activities-multi), ctrl-g (4-live picker), Esc
/// (cancelled), ctrl-g (picker again).
#[test]
#[ignore = "holla h5 check; run with --ignored"]
fn h5_activities_overlay() {
    const LIVE4: &str = "4 live";
    const FOCUSED_FIND: &str = "▎ Search actions and resources…";
    let dir = h5_dir("h5_activities_overlay");
    let case = h5_case_t("activities_overlay", ACTMULTI, BOOT, 20_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(FOCUSED_FIND), "pre: finder not focused");

    // V1: the tab strip carries all five activities.
    assert_line_has(&boot, "frontend dev ⠋ ×", "btm ○ ×", "V1 strip");
    // V2: the cursor sits on the preview action.
    assert_line_has(&boot, "› Deploy preview", "Current folder", "V2 cursor");
    // S1: the status line counts the live activities.
    assert!(boot.contains("4 running"), "S1: no running count");

    // A1: ctrl-g opens the live activities picker.
    support::drive(
        &mut s,
        &["ctrl-g", &format!("wait:{LIVE4}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-picker");
    let picker = live_text(&mut s);

    // V3-flow: the picker titles the live count.
    assert_line_has(&picker, "Activities", LIVE4, "flow title");
    // V4-flow: the top row pairs the worker with its running state.
    assert_line_has(
        &picker,
        "⠋ frontend dev",
        "running · child · 3 s",
        "flow row",
    );
    // S2: the picker lists the detached monitor.
    assert_line_has(&picker, "○ btm", "detached · host · 3 s", "S2 detached");
    // N1: the finder focus bar is gone with the picker open.
    assert!(!picker.contains(FOCUSED_FIND), "N1: finder focus stuck");

    // A2: Esc cancels the picker, ctrl-g re-opens it.
    support::press_step(&s, "escape");
    wait_absent(&mut s, LIVE4, "picker cancelled");
    waits::wait_state(&mut s, FOCUSED_FIND, "finder back");
    checkpoint(&mut s, &dir, "02-finder");
    let finder = live_text(&mut s);
    // N2: the picker row state is gone after the cancel.
    assert!(
        !finder.contains("running · child · 3 s"),
        "N2: picker row stuck"
    );
    support::drive(
        &mut s,
        &["ctrl-g", &format!("wait:{LIVE4}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "03-picker-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "Activities", LIVE4, "picker intact");
    eprintln!("h5 activities_overlay: picker, cancel roundtrip");
}
