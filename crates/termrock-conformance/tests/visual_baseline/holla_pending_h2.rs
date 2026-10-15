//! Holla pending-roots slice 8A-H2 executable checks — impl port.
//!
//! Ported verbatim from VB commit `d23472bcacf200bf5b56ea6d821374325a9d102d`
//! (`tests/harness/tests/visual_baseline/holla_pending_h2.rs`), adapting
//! only paths, binary resolution, and harness API:
//!
//! - `Session` is the impl [`support::Session`] wrapper (live samples via
//!   `s.inner.observe_now()`), not `tuiscotti::tui::Session` directly.
//! - The subject is the `holla` binary built from this impl
//!   worktree’s sources, resolved via [`support::try_resolve_bin`]
//!   (name -> executed path + sha256); [`write_provenance`] records path,
//!   digest, size, mtime, and argv behind each check.
//! - `support::drive` (3-arg) becomes [`support::drive_with_timeout`]
//!   (timeout in ms).
//! - The VB `case_timeout` helper is dropped: every call site passes
//!   `case.timeout_ms` directly, so no dead helper remains.
//!
//! One ignored PTY test per H2 registry row (22 rows), over the real
//! `holla` binary built from this worktree's impl sources
//! (resolved via [`support::try_resolve_bin`]). Each test drives the row's
//! inputs and asserts its visual (V), state (S), action (A), and
//! negative (N) checks in live-PTY executable form:
//!
//! - V: live needles plus same-line coexistence for finder rows,
//!   browser rows, find rows, scope rows, config facts, executor output,
//!   picker titles, and menu choice rows.
//! - S: live labels (cursor/focus/footer/status rows) plus selection,
//!   undo/redo, scope, and follow state.
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
//! Per-test scratch lands under this conformance crate's
//! `target/tuiscotti/journeys/<test>/` (gitignored): the row's named
//! checkpoint(s) as text frames, `00-boot.txt` where the row needs a boot
//! comparison, and `provenance.txt` (binary path, digest, size, mtime, argv).
//! Typed input is synthetic and in-memory only (simulation data); no test
//! runs real commands, touches Git state, or performs destructive
//! operations: the executor rows run the fixture world's simulated scripts
//! only, the typed password is synthetic fixture text, every cancel/kill
//! targets the simulation, and every world is the fixture simulation.
//!
//! Row → test map (registry id → `h2_*` test):
//!
//! - PANEL-BROWSER-001 → [`h2_browser_hidden`]
//! - PANEL-CHILD-001 → [`h2_child_query`]
//! - PANEL-CONFIG-001 → [`h2_config_page`]
//! - CTXMENU-FILES-001 → [`h2_files_actions`]
//! - PICKER-FILES-001 → [`h2_files_jump`]
//! - PICKER-FILES-002 → [`h2_files_jump_error`]
//! - PANEL-FILES-001 → [`h2_files_jumped`]
//! - PANEL-FILES-002 → [`h2_files_preview_control`]
//! - PANEL-FILES-003 → [`h2_files_results`]
//! - PANEL-FILES-004 → [`h2_files_unicode`]
//! - PANEL-FINDER-001 → [`h2_finder_query`]
//! - PANEL-FINDER-002 → [`h2_finder_query_selected`]
//! - EMPTY-QUERY-001 → [`h2_query_empty_preview`]
//! - PANEL-QUERY-001 → [`h2_query_redone`]
//! - PANEL-QUERY-002 → [`h2_query_undone`]
//! - PANEL-SCOPE-001 → [`h2_scope_children`]
//! - PANEL-SCOPE-002 → [`h2_scope_parent`]
//! - PANEL-SCOPE-003 → [`h2_scope_system`]
//! - PANEL-TASK-001 → [`h2_task_input_cancelled`]
//! - PANEL-TASK-002 → [`h2_task_input_killed`]
//! - PANEL-TASK-003 → [`h2_task_sources`]
//! - PANEL-TASK-004 → [`h2_task_sources_diagnostic`]

use std::path::{Path, PathBuf};

use crate::support::Session;

use crate::support::state_waits as waits;
use crate::support::{self, Case, Color, HOLLA};

/// Boot needle on every holla route: the brand.
const BOOT: &str = "holla❯";
/// Empty finder query placeholder (boot/cleared states).
const PLACEHOLDER: &str = "Search actions and resources…";

const BROWSER: &[&str] = &["--scenario", "parity-browser", "--motion", "reduced"];
const CHILD: &[&str] = &[
    "--scenario",
    "monorepo-child",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const DISCOVERY: &[&str] = &[
    "--scenario",
    "parity-discovery",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const FILES: &[&str] = &["--scenario", "parity-files", "--motion", "reduced"];
const HISTORY: &[&str] = &[
    "--scenario",
    "parity-history",
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
const ROOT: &[&str] = &[
    "--scenario",
    "monorepo-root",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const TASKINPUT: &[&str] = &["--scenario", "parity-task-input", "--motion", "reduced"];
const TASKSRC: &[&str] = &[
    "--scenario",
    "parity-task-sources",
    "--motion",
    "paused",
    "--frame",
    "40",
];

/// Owned-name H2 case at the canonical 120x40 truecolor geometry.
fn h2_case(slug: &str, args: &'static [&'static str], boot: &'static str) -> Case {
    Case::dynamic(
        format!("journeys/holla/h2/{slug}"),
        HOLLA,
        args,
        120,
        40,
        Color::Truecolor,
        boot,
    )
}

/// [`h2_case`] with an explicit per-step timeout for tick-driven flows.
fn h2_case_t(
    slug: &str,
    args: &'static [&'static str],
    boot: &'static str,
    timeout_ms: u64,
) -> Case {
    h2_case(slug, args, boot).timeout(timeout_ms)
}

/// Unique scratch dir for one H2 test (created, never shared).
fn h2_dir(test: &str) -> PathBuf {
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
        "profile: tuiscotti-default\ntransport: pty\nport_of: d23472bcacf200bf5b56ea6d821374325a9d102d\nbinary: {}\nsha256: {}\nsize_bytes: {}\nmodified_unix: {modified}\nargv: {}\n",
        subject.path.display(),
        subject.sha256,
        subject.len,
        argv.join(" "),
    );
    std::fs::write(dir.join("provenance.txt"), &body)
        .unwrap_or_else(|e| panic!("write {}: {e}", dir.join("provenance.txt").display()));
    eprintln!("h2 provenance: {body}");
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

/// One fresh live text sample (no wait).
fn live_text(s: &mut Session) -> String {
    let obs = s
        .inner
        .observe_now()
        .unwrap_or_else(|e| panic!("live sample failed: {e:#}"));
    support::screen_text(&obs.screen)
}

/// Some line must contain both `needle` and `also` (the same-row
/// coexistence proof: finder rows, browser rows, dialog rows, titles).
fn assert_line_has(text: &str, needle: &str, also: &str, what: &str) {
    assert!(
        text.lines().any(|l| l.contains(needle) && l.contains(also)),
        "{what}: no line contains both `{needle}` and `{also}`"
    );
}

/// PANEL-BROWSER-001 (`holla/flows/browser/hidden`): the browser listing
/// with hidden entries shown.
///
/// Flow: boot, browse `~/work/site`, Ctrl+H (hidden shown), Ctrl+H
/// (hidden off), Ctrl+H (shown again).
#[test]
#[ignore = "holla h2 check; run with --ignored"]
fn h2_browser_hidden() {
    let dir = h2_dir("h2_browser_hidden");
    let case = h2_case("browser_hidden", BROWSER, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["type:Browse ~/work/site", "enter", "wait:16 entries"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-listing");
    let listing = live_text(&mut s);
    // Pre-toggle presence: the listing starts with hidden off.
    assert!(listing.contains("hidden off"), "pre: no hidden-off status");

    // A1: Ctrl+H toggles hidden entries on.
    support::drive_with_timeout(&mut s, &["ctrl-h", "wait:2 hidden shown"], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-hidden");
    let hidden = live_text(&mut s);

    // V1: the header counts all entries with hidden shown.
    assert_line_has(&hidden, "18 entries", "2 hidden shown", "V1 header");
    // V2: the dotfile row resolves with its kind.
    assert_line_has(&hidden, ".git", "directory", "V2 dotfile");
    // S1: the footer announces the shown hidden entries.
    assert!(
        hidden.contains("Hidden entries shown"),
        "S1: no shown footer"
    );
    // S2: the status pairs the folder with the hidden-shown state.
    assert!(
        hidden.contains("~/work/site · hidden shown"),
        "S2: no hidden-shown status"
    );
    // N1: the hidden-off status is gone with the toggle.
    assert!(
        !hidden.contains("hidden off"),
        "N1: hidden-off status stuck"
    );

    // A2: Ctrl+H toggles hidden back off, then on again.
    support::drive_with_timeout(&mut s, &["ctrl-h", "wait:2 hidden hidden"], case.timeout_ms);
    let off = live_text(&mut s);
    // N2: the shown count is gone after the second toggle.
    assert!(!off.contains("2 hidden shown"), "N2: shown count stuck");
    support::drive_with_timeout(&mut s, &["ctrl-h", "wait:2 hidden shown"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-hidden-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "18 entries", "2 hidden shown", "header intact");
    eprintln!("h2 browser_hidden: hidden toggle roundtrip x2");
}

/// PANEL-CHILD-001 (`holla/flows/child/query`): the finder query on the
/// monorepo-child fixture.
///
/// Flow: boot, type `test`, Esc (cleared), type `test` again.
#[test]
#[ignore = "holla h2 check; run with --ignored"]
fn h2_child_query() {
    let dir = h2_dir("h2_child_query");
    let case = h2_case("child_query", CHILD, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // A1: typing filters to the child results.
    support::drive_with_timeout(&mut s, &["type:test", "wait:Results · 17"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    // V1: the top row pairs the task with its kind.
    assert_line_has(&res, "› Run tests", "task", "V1 row");
    // V2: the result count.
    assert!(res.contains("Results · 17"), "V2: no Results · 17");
    // S1: the footer offers the trust gesture.
    assert!(res.contains("Enter Trust…"), "S1: no trust footer");
    // S2: the scope readout stays here.
    assert!(res.contains("scope ‹ here ›"), "S2: no here scope");
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
    support::drive_with_timeout(&mut s, &["type:test", "wait:Results · 17"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-results-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "› Run tests", "task", "row intact");
    eprintln!("h2 child_query: results, clear roundtrip");
}

/// PANEL-CONFIG-001 (`holla/flows/config/page`): the custom-actions
/// configuration page.
///
/// Flow: boot, type `Custom action configuration`, Enter (page), Esc
/// (back), Esc (query cleared), type + Enter (page again).
#[test]
#[ignore = "holla h2 check; run with --ignored"]
fn h2_config_page() {
    const DIGEST: &str = "sha256:371a5fe65de3fa31c4d718afa4beb4a90bfcfba1953a1df79fe3a3ffae7e8324";
    let dir = h2_dir("h2_config_page");
    let case = h2_case("config_page", DISCOVERY, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // A1: Enter opens the configuration page.
    support::drive_with_timeout(
        &mut s,
        &["type:Custom action configuration", "enter", "wait:sha256:"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-page");
    let page = live_text(&mut s);

    // V1: the page shows the config digest.
    assert!(page.contains(DIGEST), "V1: no config digest");
    // V2: the trust fact reads untrusted.
    assert_line_has(&page, "Trust", "Untrusted", "V2 trust");
    // S1: the footer offers the back gesture.
    assert!(page.contains("Esc Back"), "S1: no back footer");
    // S2: the action count with its diagnostics.
    assert!(page.contains("1 · 2 diagnostics"), "S2: no action count");
    // N1: the empty-query placeholder is gone on the page.
    assert!(!page.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: Esc backs to the finder, Esc clears the retained query.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "sha256:", "page closed");
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-finder");
    let finder = live_text(&mut s);
    // N2: the page title is gone after backing out.
    assert!(!finder.contains("Custom actions"), "N2: page title stuck");
    support::drive_with_timeout(
        &mut s,
        &["type:Custom action configuration", "enter", "wait:sha256:"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "03-page-again");
    let back = live_text(&mut s);
    assert!(back.contains(DIGEST), "digest intact");
    eprintln!("h2 config_page: page, back roundtrip");
}

/// CTXMENU-FILES-001 (`holla/flows/files/actions`): the file actions
/// context menu over the single find result.
///
/// Flow: boot, find `todo` (1 result), Enter (menu), Esc (closed),
/// Enter (menu again).
#[test]
#[ignore = "holla h2 check; run with --ignored"]
fn h2_files_actions() {
    let dir = h2_dir("h2_files_actions");
    let case = h2_case("files_actions", FILES, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Find files under home",
            "enter",
            "wait:of 29 indexed",
            "ctrl-u",
            "type:todo",
            "wait:1 result",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-found");
    let found = live_text(&mut s);
    assert!(found.contains("Enter Actions"), "pre: no actions footer");

    // A1: Enter opens the actions menu for the result.
    support::drive_with_timeout(&mut s, &["enter", "wait:Open in the OS"], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-menu");
    let menu = live_text(&mut s);

    // V1: the first choice pairs the action with its shortcut.
    assert_line_has(&menu, "Open in the OS", "Enter", "V1 choice");
    // V2: the menu titles the resolved file.
    assert!(
        menu.contains("/Users/alex/work/notes/todo.md"),
        "V2: no menu title"
    );
    // S1: the menu footer offers the close gesture.
    assert!(menu.contains("Esc Close"), "S1: no close footer");
    // S2: the single result stays behind the menu.
    assert!(
        menu.contains("1 result · home scope"),
        "S2: no result count"
    );
    // N1: the find footer is gone under the menu footer.
    assert!(!menu.contains("Enter Actions"), "N1: find footer stuck");

    // A2: Esc closes the menu back to the find state.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Open in the OS", "menu closed");
    waits::wait_state(&mut s, "Enter Actions", "find footer back");
    checkpoint(&mut s, &dir, "03-closed");
    let closed = live_text(&mut s);
    // N2: the menu choices are gone with the close.
    assert!(
        !closed.contains("Reveal in the file manager"),
        "N2: menu choice stuck"
    );
    support::drive_with_timeout(&mut s, &["enter", "wait:Open in the OS"], case.timeout_ms);
    checkpoint(&mut s, &dir, "04-menu-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "Open in the OS", "Enter", "menu intact");
    eprintln!("h2 files_actions: menu, close roundtrip");
}

/// PICKER-FILES-001 (`holla/flows/files/jump`): the Go-to-path picker
/// over the browser listing.
///
/// Flow: boot, browse `~/work/site`, g (picker), Esc (cancelled),
/// g (picker again). The picker title also lives in the browser footer,
/// so the footer swap (`Enter Switch` vs `Open / preview`) is the
/// open/close signal, not the title.
#[test]
#[ignore = "holla h2 check; run with --ignored"]
fn h2_files_jump() {
    let dir = h2_dir("h2_files_jump");
    let case = h2_case("files_jump", BROWSER, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["type:Browse ~/work/site", "enter", "wait:16 entries"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-listing");
    let listing = live_text(&mut s);
    assert!(listing.contains("Open / preview"), "pre: no browser footer");

    // A1: g opens the Go-to-path picker.
    support::drive_with_timeout(&mut s, &["g", "wait:Enter Switch"], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-picker");
    let picker = live_text(&mut s);

    // V1: the picker dialog titles the current folder.
    assert_line_has(&picker, "Go to path", "~/work/site", "V1 title");
    // V2: the picker invites a path or a name.
    assert!(
        picker.contains("type a path or a name"),
        "V2: no picker invitation"
    );
    // S1: the picker footer offers the switch gesture.
    assert!(picker.contains("Enter Switch"), "S1: no switch footer");
    // S2: the input names the relative-path mode.
    assert!(
        picker.contains("relative to this folder"),
        "S2: no relative-path hint"
    );
    // N1: the browser footer is gone under the picker footer.
    assert!(
        !picker.contains("Open / preview"),
        "N1: browser footer stuck"
    );

    // A2: Esc cancels the empty-query picker back to the listing.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Enter Switch", "picker cancelled");
    waits::wait_state(&mut s, "Open / preview", "browser footer back");
    checkpoint(&mut s, &dir, "03-cancelled");
    let cancelled = live_text(&mut s);
    // N2: the picker footer is gone after the cancel.
    assert!(!cancelled.contains("Enter Switch"), "N2: picker stuck");
    support::drive_with_timeout(&mut s, &["g", "wait:Enter Switch"], case.timeout_ms);
    checkpoint(&mut s, &dir, "04-picker-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "Go to path", "~/work/site", "picker intact");
    eprintln!("h2 files_jump: picker, cancel roundtrip");
}

/// PICKER-FILES-002 (`holla/flows/files/jump_error`): the Go-to-path
/// picker error with recovery through a good jump.
///
/// Flow: boot, browse `~/work/site`, g, type `nope`, Enter (error),
/// Esc (query cleared), Esc (picker cancelled), g, type `~/work`,
/// Enter (jumped). Recovery takes two Escapes by construction
/// (`Picker::on_key`): the first clears the non-empty query, the second
/// cancels the picker.
#[test]
#[ignore = "holla h2 check; run with --ignored"]
fn h2_files_jump_error() {
    let dir = h2_dir("h2_files_jump_error");
    let case = h2_case("files_jump_error", BROWSER, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Browse ~/work/site",
            "enter",
            "wait:16 entries",
            "g",
            "wait:Enter Switch",
            "type:nope",
            "enter",
            "wait:no such file or directory",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-error");
    let error = live_text(&mut s);

    // V1: the picker reports the bad path.
    assert!(
        error.contains("nope: no such file or directory"),
        "V1: no picker error"
    );
    // V2: the picker offers the recovery hint.
    assert!(
        error.contains("edit the path or choose a suggestion"),
        "V2: no recovery hint"
    );
    // S1: the picker stays open on the error.
    assert!(error.contains("Enter Switch"), "S1: picker closed on error");
    // S2: the bad input stays for editing.
    assert!(error.contains("▎ nope"), "S2: bad input lost");
    // N1: nothing jumped on the bad path.
    assert!(!error.contains("Jumped to"), "N1: jumped on bad path");

    // A1: the bad path errors inside the picker (observed above).
    // A2: two Escapes recover, then a good path jumps.
    support::press_step(&s, "escape");
    support::wait_screen(&mut s, support::DEFAULT_WAIT, "query cleared", |screen| {
        !support::screen_text(screen).contains("▎ nope")
    });
    checkpoint(&mut s, &dir, "02-query-cleared");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Enter Switch", "picker dismissed");
    waits::wait_state(&mut s, "Open / preview", "browser footer back");
    waits::wait_state(&mut s, "16 entries", "listing intact");
    support::drive_with_timeout(
        &mut s,
        &[
            "g",
            "wait:Enter Switch",
            "type:~/work",
            "enter",
            "wait:Jumped to ~/work",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "03-jumped");
    let jumped = live_text(&mut s);
    assert!(jumped.contains("Files · ~/work"), "jumped: no new folder");
    // N2: the picker error is gone after the good jump.
    assert!(
        !jumped.contains("no such file or directory"),
        "N2: picker error stuck"
    );
    eprintln!("h2 files_jump_error: error, recovery, good jump");
}

/// PANEL-FILES-001 (`holla/flows/files/jumped`): the browser listing
/// after jumping to `~/work`.
///
/// Flow: boot, browse `~/work/site`, g, type `~/work`, Enter (jumped),
/// g (picker), Esc (cancelled).
#[test]
#[ignore = "holla h2 check; run with --ignored"]
fn h2_files_jumped() {
    let dir = h2_dir("h2_files_jumped");
    let case = h2_case("files_jumped", BROWSER, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Browse ~/work/site",
            "enter",
            "wait:16 entries",
            "g",
            "wait:Enter Switch",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-picker");
    let picker = live_text(&mut s);
    assert!(picker.contains("Enter Switch"), "pre: picker never opened");

    // A1: Enter on a good path jumps the browser there.
    support::drive_with_timeout(
        &mut s,
        &["type:~/work", "enter", "wait:Jumped to ~/work"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-jumped");
    let jumped = live_text(&mut s);

    // V1: the header names the jumped-to folder with its single entry.
    assert!(jumped.contains("Files · ~/work"), "V1: no jumped header");
    assert!(jumped.contains("1 entry"), "V1: no single entry");
    // V2: the status confirms the jump target.
    assert!(jumped.contains("Jumped to ~/work"), "V2: no jump status");
    // S1: the breadcrumb follows the jump.
    assert!(jumped.contains("Files › work"), "S1: no jumped breadcrumb");
    // S2: the hidden state survives the jump.
    assert!(jumped.contains("hidden off"), "S2: no hidden status");
    // N1: the picker footer is gone after the jump lands.
    assert!(!jumped.contains("Enter Switch"), "N1: picker stuck");

    // A2: g reopens the picker over the new folder, Esc cancels it.
    support::drive_with_timeout(&mut s, &["g", "wait:Enter Switch"], case.timeout_ms);
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Enter Switch", "picker cancelled");
    waits::wait_state(&mut s, "Files · ~/work", "jumped folder intact");
    checkpoint(&mut s, &dir, "03-cancelled");
    let back = live_text(&mut s);
    // N2: the old listing count is gone after the jump.
    assert!(!back.contains("16 entries"), "N2: old listing stuck");
    assert!(back.contains("Files · ~/work"), "folder intact");
    assert!(back.contains("1 entry"), "count intact");
    eprintln!("h2 files_jumped: jump, picker roundtrip");
}

/// PANEL-FILES-002 (`holla/flows/files/preview_control`): the browser
/// listing with the cursor on the control-character file.
///
/// Flow: boot, browse `~/work/site`, Down x7 (control.txt), Down
/// (preview follows), Up (preview back).
#[test]
#[ignore = "holla h2 check; run with --ignored"]
fn h2_files_preview_control() {
    const TITLE: &str = "~/work/site/control.txt";
    let dir = h2_dir("h2_files_preview_control");
    let case = h2_case("files_preview_control", BROWSER, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    // A1: seven downs land the cursor on control.txt.
    support::drive_with_timeout(
        &mut s,
        &[
            "type:Browse ~/work/site",
            "enter",
            "wait:16 entries",
            "down",
            "down",
            "down",
            "down",
            "down",
            "down",
            "down",
            "sleep:300",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-control");
    let control = live_text(&mut s);

    // V1: the preview titles the control file with its byte count.
    assert_line_has(&control, TITLE, "13 B of 13 B", "V1 preview");
    // V2: the preview body shows the control content.
    assert!(control.contains("red"), "V2: no preview body");
    // S1: the cursor row names the control file with its size.
    assert_line_has(&control, "control.txt", "13 B", "S1 cursor");
    // S2: the footer offers the back gesture.
    assert!(control.contains("Esc Back"), "S2: no back footer");

    // A2: Down moves the preview off, Up moves it back.
    support::drive_with_timeout(&mut s, &["down", "sleep:300"], case.timeout_ms);
    let moved = live_text(&mut s);
    // N1: the control preview title is gone after moving off.
    assert!(!moved.contains(TITLE), "N1: control preview stuck");
    support::drive_with_timeout(&mut s, &["up", &format!("wait:{TITLE}")], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-control-again");
    let back = live_text(&mut s);
    // N2: the moved-to preview is gone after moving back.
    assert!(
        !back.contains("~/work/site/current"),
        "N2: moved-to preview stuck"
    );
    assert_line_has(&back, TITLE, "13 B of 13 B", "preview intact");
    eprintln!("h2 files_preview_control: preview, cursor roundtrip");
}

/// PANEL-FILES-003 (`holla/flows/files/results`): the find results for
/// `readme` across home scope.
///
/// Flow: boot, find `readme` (3 results), Down (preview follows),
/// Up (preview back).
#[test]
#[ignore = "holla h2 check; run with --ignored"]
fn h2_files_results() {
    const FIRST: &str = "~/Documents/README.md";
    const SECOND: &str = "~/work/app/README.md";
    let dir = h2_dir("h2_files_results");
    let case = h2_case("files_results", FILES, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    // A1: typing filters to the three readme results.
    support::drive_with_timeout(
        &mut s,
        &[
            "type:Find files under home",
            "enter",
            "wait:of 29 indexed",
            "type:readme",
            "wait:3 results",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    // V1: the result count in home scope.
    assert!(res.contains("3 results · home scope"), "V1: no 3 results");
    // V2: the top row pairs the file with its folder.
    assert_line_has(&res, "README.md", "~/Documents", "V2 row");
    // S1: the preview titles the top result with its byte count.
    assert_line_has(&res, FIRST, "7 B of 7 B", "S1 preview");
    // S2: the footer offers the actions gesture.
    assert!(res.contains("Enter Actions"), "S2: no actions footer");

    // A2: Down moves the preview to the second row, Up moves it back.
    support::drive_with_timeout(
        &mut s,
        &["down", &format!("wait:{SECOND}")],
        case.timeout_ms,
    );
    let moved = live_text(&mut s);
    // N1: the first preview title is gone after moving down.
    assert!(!moved.contains(FIRST), "N1: first preview stuck");
    support::drive_with_timeout(&mut s, &["up", &format!("wait:{FIRST}")], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-results-again");
    let back = live_text(&mut s);
    // N2: the second preview title is gone after moving back.
    assert!(!back.contains(SECOND), "N2: second preview stuck");
    assert!(back.contains("3 results · home scope"), "count intact");
    eprintln!("h2 files_results: results, cursor roundtrip");
}

/// PANEL-FILES-004 (`holla/flows/files/unicode`): the find result for
/// the unicode query `café`.
///
/// Flow: boot, find `café` (1 result), Ctrl+U (cleared), type `café`.
#[test]
#[ignore = "holla h2 check; run with --ignored"]
fn h2_files_unicode() {
    let dir = h2_dir("h2_files_unicode");
    let case = h2_case("files_unicode", FILES, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // A1: the unicode query filters to its single result.
    support::drive_with_timeout(
        &mut s,
        &[
            "type:Find files under home",
            "enter",
            "wait:of 29 indexed",
            "ctrl-u",
            "type:café",
            "wait:1 result",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-result");
    let res = live_text(&mut s);

    // V1: the single result in home scope.
    assert!(res.contains("1 result · home scope"), "V1: no 1 result");
    // V2: the row pairs the unicode name with its folder.
    assert_line_has(&res, "café menu.txt", "~/Documents", "V2 row");
    // S1: the preview titles the result with its byte count.
    assert_line_has(&res, "café menu.txt", "4 B of 4 B", "S1 preview");
    // S2: the preview body shows the unicode content.
    assert!(res.contains("☕"), "S2: no preview body");
    // N1: the empty-finder placeholder is gone in find mode.
    assert!(!res.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: Ctrl+U clears the query, retyping restores the result.
    // press_step (no pacing sleep): the clear lands before wait_gone's
    // first sample would otherwise miss the presence proof.
    support::press_step(&s, "ctrl-u");
    waits::wait_gone(&mut s, "1 result", "query cleared");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);
    // N2: the single result is gone after the clear.
    assert!(!cleared.contains("1 result"), "N2: result stuck");
    support::drive_with_timeout(&mut s, &["type:café", "wait:1 result"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-result-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "café menu.txt", "~/Documents", "row intact");
    eprintln!("h2 files_unicode: unicode result, clear roundtrip");
}

/// PANEL-FINDER-001 (`holla/flows/finder/query`): the finder query
/// `pull` with its ranked git results.
///
/// Flow: boot, type `pull`, Esc (cleared), type `pull` again.
#[test]
#[ignore = "holla h2 check; run with --ignored"]
fn h2_finder_query() {
    let dir = h2_dir("h2_finder_query");
    let case = h2_case("finder_query", HISTORY, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // A1: typing filters to the pull results.
    support::drive_with_timeout(&mut s, &["type:pull", "wait:Results · 3"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    // V1: the top row pairs the action with its state.
    assert_line_has(&res, "› Pull", "up to date", "V1 row");
    // V2: the result count.
    assert!(res.contains("Results · 3"), "V2: no Results · 3");
    // S1: the footer offers the run gesture.
    assert!(res.contains("Enter Run"), "S1: no run footer");
    // S2: the scope readout stays here.
    assert!(res.contains("scope ‹ here ›"), "S2: no here scope");
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
    support::drive_with_timeout(&mut s, &["type:pull", "wait:Results · 3"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-results-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "› Pull", "up to date", "row intact");
    eprintln!("h2 finder_query: results, clear roundtrip");
}

/// PANEL-FINDER-002 (`holla/flows/finder/query-selected`): the finder
/// with the whole query selected.
///
/// Flow: boot, type `pull`, Ctrl+A (selected), Esc (deselected),
/// type `!` (appended, proving the deselect), Ctrl+Z (edit undone),
/// Ctrl+A (selected again). Esc on a selection only drops the flag by
/// construction (`FinderPage::on_key`): the status line keeps its text
/// until the next status, so the appended `!` is the deselect proof.
#[test]
#[ignore = "holla h2 check; run with --ignored"]
fn h2_finder_query_selected() {
    let dir = h2_dir("h2_finder_query_selected");
    let case = h2_case("finder_query_selected", HISTORY, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(&mut s, &["type:pull", "wait:Results · 3"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);
    assert!(res.contains("Esc Clear"), "pre: no clear footer");

    // A1: Ctrl+A selects the whole query.
    support::drive_with_timeout(&mut s, &["ctrl-a", "wait:Query selected"], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-selected");
    let sel = live_text(&mut s);

    // V1: the status names the selection with its consequence.
    assert!(
        sel.contains("Query selected · typing replaces it"),
        "V1: no selected status"
    );
    // V2: the results stay behind the selection.
    assert!(sel.contains("Results · 3"), "V2: no Results · 3");
    // S1: the query row still shows the query.
    assert!(sel.contains("▎ pull"), "S1: no selected query");
    // S2: the scope readout stays here.
    assert!(sel.contains("scope ‹ here ›"), "S2: no here scope");
    // N1: the clear footer is gone under the selection status.
    assert!(!sel.contains("Esc Clear"), "N1: clear footer stuck");

    // A2: Esc deselects (the status text stays by construction), so a
    // typed `!` appends instead of replacing; Ctrl+Z undoes the edit.
    support::press_step(&s, "escape");
    support::drive_with_timeout(&mut s, &["type:!", "wait:▎ pull!"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-appended");
    let appended = live_text(&mut s);
    assert!(appended.contains("▎ pull!"), "appended: no pull!");
    support::drive_with_timeout(&mut s, &["ctrl-z", "wait:Undone"], case.timeout_ms);
    let reunited = live_text(&mut s);
    // N2: the appended edit is gone after the undo.
    assert!(!reunited.contains("▎ pull!"), "N2: appended edit stuck");
    assert!(reunited.contains("▎ pull"), "reunited: no pull");
    support::drive_with_timeout(&mut s, &["ctrl-a", "wait:Query selected"], case.timeout_ms);
    checkpoint(&mut s, &dir, "04-selected-again");
    let back = live_text(&mut s);
    assert!(
        back.contains("Query selected · typing replaces it"),
        "selection intact"
    );
    eprintln!("h2 finder_query_selected: selection, clear roundtrip");
}

/// EMPTY-QUERY-001 (`holla/flows/query/empty_preview`): the holla Empty
/// preview for a query with no matches.
///
/// Flow: boot, open and cancel the gate-1 dialog, open and cancel the
/// blocking dialog, type the matchless query (empty preview), Esc
/// (cleared), type `resources` (empty preview again).
#[test]
#[ignore = "holla h2 check; run with --ignored"]
fn h2_query_empty_preview() {
    let dir = h2_dir("h2_query_empty_preview");
    let case = h2_case("query_empty_preview", REMOTE, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    // A1: the dialog roundtrips plus the matchless query reach the Empty.
    support::drive_with_timeout(
        &mut s,
        &[
            "type:restart payments",
            "enter",
            "wait:gate 1 of 2",
            "escape",
            "escape",
            "type:blocking",
            "enter",
            "wait:Who is blocking",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-blocking");
    let blocking = live_text(&mut s);
    assert!(
        blocking.contains("Who is blocking"),
        "mid: no blocking dialog"
    );
    support::drive_with_timeout(
        &mut s,
        &["escape", "type:resources", "wait:No matches for"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-empty");
    let empty = live_text(&mut s);

    // V1: the empty state names the matchless query.
    assert!(
        empty.contains("No matches for “blockingresources”"),
        "V1: no no-matches row"
    );
    // V2: the preview is the holla Empty with its hint.
    assert!(empty.contains("Nothing to preview"), "V2: no Empty preview");
    // S1: the footer records the cancelled gate.
    assert!(
        empty.contains("Cancelled · nothing was executed"),
        "S1: no cancelled status"
    );
    // S2: the query row holds the full matchless query.
    assert!(
        empty.contains("▎ blockingresources"),
        "S2: no matchless query"
    );
    // N1: the blocking dialog is gone under the empty state.
    assert!(!empty.contains("Who is blocking"), "N1: dialog stuck");

    // A2: Esc clears the query, retyping restores the Empty.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "No matches for", "query cleared");
    waits::wait_state(&mut s, PLACEHOLDER, "placeholder back");
    checkpoint(&mut s, &dir, "03-cleared");
    let cleared = live_text(&mut s);
    // N2: the matchless query is gone after the clear.
    assert!(!cleared.contains("blockingresources"), "N2: query stuck");
    // Retype the full matchless query: bare `resources` matches rows on
    // this fixture, only `blockingresources` is matchless.
    support::drive_with_timeout(
        &mut s,
        &["type:blockingresources", "wait:No matches for"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "04-empty-again");
    let back = live_text(&mut s);
    assert!(
        back.contains("No matches for “blockingresources”"),
        "no-matches intact"
    );
    assert!(back.contains("Nothing to preview"), "Empty intact");
    eprintln!("h2 query_empty_preview: Empty, clear roundtrip");
}

/// PANEL-QUERY-001 (`holla/flows/query/redone`): the finder query
/// restored by redo.
///
/// Flow: boot, type `pull`, Ctrl+Z x4 (undone), Ctrl+Y x4 (redone),
/// Ctrl+Z (undone), Ctrl+Y (redone again).
#[test]
#[ignore = "holla h2 check; run with --ignored"]
fn h2_query_redone() {
    let dir = h2_dir("h2_query_redone");
    let case = h2_case("query_redone", HISTORY, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:pull",
            "ctrl-z",
            "ctrl-z",
            "ctrl-z",
            "ctrl-z",
            "wait:Undone",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-undone");
    let undone = live_text(&mut s);
    assert!(undone.contains("Undone"), "mid: no Undone status");

    // A1: redo restores the undone query.
    support::drive_with_timeout(
        &mut s,
        &["ctrl-y", "ctrl-y", "ctrl-y", "ctrl-y", "wait:Redone"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-redone");
    let redone = live_text(&mut s);

    // V1: the restored query ranks its results again.
    assert!(redone.contains("Results · 3"), "V1: no Results · 3");
    // V2: the footer records the redo.
    assert!(redone.contains("Redone"), "V2: no Redone status");
    // S1: the top row pairs the action with its state.
    assert_line_has(&redone, "› Pull", "up to date", "S1 row");
    // S2: the scope readout stays here.
    assert!(redone.contains("scope ‹ here ›"), "S2: no here scope");
    // N1: the Undone status is gone after the redo.
    assert!(!redone.contains("Undone"), "N1: Undone stuck");

    // A2: Ctrl+Z undoes again, Ctrl+Y redoes again.
    support::drive_with_timeout(&mut s, &["ctrl-z", "wait:Undone"], case.timeout_ms);
    let undone2 = live_text(&mut s);
    // N2: the Redone status is gone after the undo.
    assert!(!undone2.contains("Redone"), "N2: Redone stuck");
    support::drive_with_timeout(&mut s, &["ctrl-y", "wait:Redone"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-redone-again");
    let back = live_text(&mut s);
    assert!(back.contains("Results · 3"), "results intact");
    eprintln!("h2 query_redone: redo, undo roundtrip");
}

/// PANEL-QUERY-002 (`holla/flows/query/undone`): the finder with the
/// query cleared by undo.
///
/// Flow: boot, type `pull`, Ctrl+Z x4 (undone), Ctrl+Y x4 (redone),
/// Ctrl+Z x4 (undone again).
#[test]
#[ignore = "holla h2 check; run with --ignored"]
fn h2_query_undone() {
    let dir = h2_dir("h2_query_undone");
    let case = h2_case("query_undone", HISTORY, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(&mut s, &["type:pull", "wait:Results · 3"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-typed");
    let typed = live_text(&mut s);
    assert!(typed.contains("Results · 3"), "mid: no typed results");

    // A1: undo clears the typed query.
    support::drive_with_timeout(
        &mut s,
        &["ctrl-z", "ctrl-z", "ctrl-z", "ctrl-z", "wait:Undone"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-undone");
    let undone = live_text(&mut s);

    // V1: the empty-query placeholder is back.
    assert!(undone.contains(PLACEHOLDER), "V1: no placeholder");
    // V2: the footer records the undo.
    assert!(undone.contains("Undone"), "V2: no Undone status");
    // S1: the suggestions replace the results.
    assert!(undone.contains("Suggested here"), "S1: no suggestions");
    // S2: the top suggestion pairs the task with its kind.
    assert_line_has(&undone, "Run check", "task", "S2 suggestion");
    // N1: the typed result count is gone after the undo.
    assert!(!undone.contains("Results · 3"), "N1: results stuck");

    // A2: four Ctrl+Ys redo the whole query, four Ctrl+Zs undo it
    // again (edits are per-keystroke, so one chord restores one char).
    support::drive_with_timeout(
        &mut s,
        &[
            "ctrl-y",
            "ctrl-y",
            "ctrl-y",
            "ctrl-y",
            "wait:Redone",
            "wait:Results · 3",
        ],
        case.timeout_ms,
    );
    let redone = live_text(&mut s);
    // N2: the Undone status is gone after the redo.
    assert!(!redone.contains("Undone"), "N2: Undone stuck");
    support::drive_with_timeout(
        &mut s,
        &["ctrl-z", "ctrl-z", "ctrl-z", "ctrl-z", "wait:Undone"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "03-undone-again");
    let back = live_text(&mut s);
    assert!(back.contains(PLACEHOLDER), "placeholder intact");
    eprintln!("h2 query_undone: undo, redo roundtrip");
}

/// PANEL-SCOPE-001 (`holla/flows/scope/children`): the finder widened
/// to the children scope.
///
/// Flow: boot, Ctrl+Down (children), Ctrl+Up (here), Ctrl+Down
/// (children again).
#[test]
#[ignore = "holla h2 check; run with --ignored"]
fn h2_scope_children() {
    let dir = h2_dir("h2_scope_children");
    let case = h2_case("scope_children", ROOT, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains("scope ‹ here ›"), "pre: boot not at here");

    // A1: Ctrl+Down widens to the children scope.
    support::drive_with_timeout(
        &mut s,
        &["ctrl-down", "wait:Children · 3 projects"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-children");
    let children = live_text(&mut s);

    // V1: the header counts the child projects with their entries.
    assert!(
        children.contains("Children · 3 projects · 11 entries"),
        "V1: no children header"
    );
    // V2: the scope readout names children.
    assert!(
        children.contains("scope ‹ children ›"),
        "V2: no children scope"
    );
    // S1: the footer records the widened scope.
    assert!(
        children.contains("Scope children · 3 projects"),
        "S1: no children footer"
    );
    // S2: the top row pairs the plan with its risk.
    assert_line_has(&children, "Clean developer", "destructive", "S2 row");
    // N1: the here readout is gone under children.
    assert!(!children.contains("scope ‹ here ›"), "N1: here scope stuck");

    // A2: Ctrl+Up narrows back to here, Ctrl+Down widens again.
    support::drive_with_timeout(&mut s, &["ctrl-up", "wait:scope ‹ here ›"], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-here");
    let here = live_text(&mut s);
    // N2: the children readout is gone after narrowing (the footer keeps
    // its stale status text until the next status, so only the readout
    // proves the narrowing).
    assert!(
        !here.contains("scope ‹ children ›"),
        "N2: children readout stuck"
    );
    support::drive_with_timeout(
        &mut s,
        &["ctrl-down", "wait:Children · 3 projects"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "03-children-again");
    let back = live_text(&mut s);
    assert!(back.contains("scope ‹ children ›"), "children intact");
    eprintln!("h2 scope_children: widen, narrow roundtrip");
}

/// PANEL-SCOPE-002 (`holla/flows/scope/parent`): the finder widened to
/// the parent scope with no project root above.
///
/// Flow: boot, Ctrl+Up (parent), Esc (here), Ctrl+Up (parent again).
#[test]
#[ignore = "holla h2 check; run with --ignored"]
fn h2_scope_parent() {
    let dir = h2_dir("h2_scope_parent");
    let case = h2_case("scope_parent", ROOT, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains("scope ‹ here ›"), "pre: boot not at here");

    // A1: Ctrl+Up widens to the parent scope.
    support::drive_with_timeout(
        &mut s,
        &["ctrl-up", "wait:No project root above"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-parent");
    let parent = live_text(&mut s);

    // V1: the empty state names the rootless folder.
    assert!(
        parent.contains("No project root above ~/work/acme"),
        "V1: no rootless row"
    );
    // V2: the scope readout names parent.
    assert!(parent.contains("scope ‹ parent ›"), "V2: no parent scope");
    // S1: the footer records the parent scope with no root above.
    assert!(
        parent.contains("Scope parent · no root above"),
        "S1: no parent footer"
    );
    // S2: the preview is empty with its widen hint.
    assert!(
        parent.contains("Nothing to preview"),
        "S2: no Empty preview"
    );
    // N1: the here readout is gone under parent.
    assert!(!parent.contains("scope ‹ here ›"), "N1: here scope stuck");

    // A2: Esc narrows back to here, Ctrl+Up widens again.
    support::drive_with_timeout(&mut s, &["escape", "wait:scope ‹ here ›"], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-here");
    let here = live_text(&mut s);
    // N2: the rootless row is gone after narrowing.
    assert!(
        !here.contains("No project root above"),
        "N2: rootless row stuck"
    );
    support::drive_with_timeout(
        &mut s,
        &["ctrl-up", "wait:No project root above"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "03-parent-again");
    let back = live_text(&mut s);
    assert!(back.contains("scope ‹ parent ›"), "parent intact");
    eprintln!("h2 scope_parent: widen, narrow roundtrip");
}

/// PANEL-SCOPE-003 (`holla/flows/scope/system`): the finder widened to
/// the host-wide system scope.
///
/// Flow: boot, Ctrl+Up x2 (system), Esc (here), Ctrl+Up x2 (system).
#[test]
#[ignore = "holla h2 check; run with --ignored"]
fn h2_scope_system() {
    let dir = h2_dir("h2_scope_system");
    let case = h2_case("scope_system", ROOT, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains("scope ‹ here ›"), "pre: boot not at here");

    // A1: two Ctrl+Ups widen past parent to the system scope.
    support::drive_with_timeout(
        &mut s,
        &["ctrl-up", "ctrl-up", "wait:scope ‹ system ›"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-system");
    let system = live_text(&mut s);

    // V1: the header counts the host-wide entries.
    assert!(
        system.contains("System · mbp · 45 entries"),
        "V1: no system header"
    );
    // V2: the scope readout names system.
    assert!(system.contains("scope ‹ system ›"), "V2: no system scope");
    // S1: the footer records the system scope.
    assert!(
        system.contains("Scope system · mbp"),
        "S1: no system footer"
    );
    // S2: the top row pairs the review with its size.
    assert_line_has(&system, "Review cleanup candidates", "10.6 GiB", "S2 row");
    // N1: the here readout is gone under system.
    assert!(!system.contains("scope ‹ here ›"), "N1: here scope stuck");

    // A2: Esc narrows back to here, two Ctrl+Ups widen again.
    support::drive_with_timeout(&mut s, &["escape", "wait:scope ‹ here ›"], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-here");
    let here = live_text(&mut s);
    // N2: the system readout is gone after narrowing.
    assert!(!here.contains("scope ‹ system ›"), "N2: system scope stuck");
    support::drive_with_timeout(
        &mut s,
        &["ctrl-up", "ctrl-up", "wait:scope ‹ system ›"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "03-system-again");
    let back = live_text(&mut s);
    assert!(back.contains("System · mbp · 45 entries"), "system intact");
    eprintln!("h2 scope_system: widen, narrow roundtrip");
}

/// PANEL-TASK-001 (`holla/flows/task/input_cancelled`): the executor
/// output for the operator-cancelled rollout.
///
/// Flow: boot, run `Deploy the release`, answer the password prompt
/// with synthetic fixture text, answer `n` (cancelled), r (restart),
/// answer again (cancelled again), Alt+0 (back to Here). The scripts
/// are the fixture world's simulation; nothing real executes.
#[test]
#[ignore = "holla h2 check; run with --ignored"]
fn h2_task_input_cancelled() {
    const CANCELLED: &str = "rollout cancelled by operator";
    let dir = h2_dir("h2_task_input_cancelled");
    let case = h2_case_t("task_input_cancelled", TASKINPUT, BOOT, 20_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    // A1: the password + decline answers run the rollout to cancellation.
    support::drive_with_timeout(
        &mut s,
        &[
            "type:Deploy the release",
            "enter",
            "right",
            "enter",
            "wait:Password:",
            "i",
            "type:hunter2",
            "enter",
            "wait:Proceed with rollout?",
            "type:n",
            "enter",
            &format!("wait:{CANCELLED}"),
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-cancelled");
    let cancelled = live_text(&mut s);

    // V1: the output records the operator cancellation.
    assert!(cancelled.contains(CANCELLED), "V1: no cancelled line");
    // V2: the title pairs the release with its failed exit.
    assert_line_has(&cancelled, "deploy the release", "exit 2", "V2 title");
    // S1: the output shows the prompt was answered.
    assert!(cancelled.contains("authenticated"), "S1: no auth line");
    // S2: the footer records the failed release.
    assert!(
        cancelled.contains("failed · exit 2"),
        "S2: no failed footer"
    );

    // A2: r restarts the script, the answers cancel it again.
    // press_step (no pacing sleep): the restart clears the output before
    // wait_gone's first sample would otherwise miss the presence proof.
    support::press_step(&s, "r");
    waits::wait_gone(&mut s, CANCELLED, "output restarted");
    waits::wait_state(&mut s, "Password:", "password prompt back");
    let restarted = live_text(&mut s);
    // N1: the cancelled line is gone after the restart.
    assert!(!restarted.contains(CANCELLED), "N1: cancelled line stuck");
    support::drive_with_timeout(
        &mut s,
        &[
            "i",
            "type:hunter2",
            "enter",
            "wait:Proceed with rollout?",
            "type:n",
            "enter",
            &format!("wait:{CANCELLED}"),
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-cancelled-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "deploy the release", "exit 2", "title intact");
    support::press_step(&s, "alt-0");
    waits::wait_gone(&mut s, CANCELLED, "back to Here");
    waits::wait_state(&mut s, "scope ‹ here ›", "finder back");
    checkpoint(&mut s, &dir, "03-here");
    let here = live_text(&mut s);
    // N2: the cancelled line is gone after leaving the executor.
    assert!(!here.contains(CANCELLED), "N2: cancelled line stuck");
    eprintln!("h2 task_input_cancelled: cancel, restart roundtrip");
}

/// PANEL-TASK-002 (`holla/flows/task/input_killed`): the executor
/// output for the SIGKILL-stopped stubborn worker.
///
/// Flow: boot, cancel `Deploy the release`, Alt+0, run the stubborn
/// worker, s (SIGTERM then SIGKILL), r (restart), s (killed again),
/// Alt+0 (back to Here). The scripts are the fixture world's
/// simulation; nothing real executes.
#[test]
#[ignore = "holla h2 check; run with --ignored"]
fn h2_task_input_killed() {
    const KILLED: &str = "killed (SIGKILL)";
    const CANCELLED: &str = "rollout cancelled by operator";
    let dir = h2_dir("h2_task_input_killed");
    let case = h2_case_t("task_input_killed", TASKINPUT, BOOT, 20_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Deploy the release",
            "enter",
            "right",
            "enter",
            "wait:Password:",
            "i",
            "type:hunter2",
            "enter",
            "wait:Proceed with rollout?",
            "type:n",
            "enter",
            &format!("wait:{CANCELLED}"),
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-cancelled");
    let cancelled = live_text(&mut s);
    assert!(cancelled.contains(CANCELLED), "mid: no cancelled line");

    // A1: s escalates the resistant worker from SIGTERM to SIGKILL.
    support::drive_with_timeout(
        &mut s,
        &[
            "alt-0",
            "ctrl-u",
            "type:Run the stubborn worker",
            "enter",
            "wait:ignoring SIGTERM",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-spawned");
    support::drive_with_timeout(&mut s, &["s", &format!("wait:{KILLED}")], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-killed");
    let killed = live_text(&mut s);

    // V1: the output records the SIGKILL with its reaping.
    assert!(
        killed.contains("killed (SIGKILL) · descendants reaped"),
        "V1: no killed line"
    );
    // V2: the title pairs the worker with its stopped exit.
    assert_line_has(&killed, "the stubborn worker", "exit 137", "V2 title");
    // S1: the output shows the SIGTERM resistance first.
    assert!(
        killed.contains("SIGKILL to the process group"),
        "S1: no escalation line"
    );
    // S2: the footer records the stopped worker.
    assert!(
        killed.contains("the stubborn worker stopped · exit 137"),
        "S2: no stopped footer"
    );
    // N1: the deploy cancellation stays out of the worker output.
    assert!(!killed.contains(CANCELLED), "N1: deploy output leaked");

    // A2: r restarts the worker, s kills it again.
    // press_step (no pacing sleep): the restart clears the output before
    // wait_gone's first sample would otherwise miss the presence proof.
    support::press_step(&s, "r");
    waits::wait_gone(&mut s, KILLED, "worker restarted");
    waits::wait_state(&mut s, "ignoring SIGTERM", "worker running again");
    support::drive_with_timeout(&mut s, &["s", &format!("wait:{KILLED}")], case.timeout_ms);
    checkpoint(&mut s, &dir, "04-killed-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "the stubborn worker", "exit 137", "title intact");
    support::press_step(&s, "alt-0");
    waits::wait_gone(&mut s, KILLED, "back to Here");
    waits::wait_state(&mut s, "scope ‹ here ›", "finder back");
    checkpoint(&mut s, &dir, "05-here");
    let here = live_text(&mut s);
    // N2: the killed line is gone after leaving the executor.
    assert!(!here.contains(KILLED), "N2: killed line stuck");
    eprintln!("h2 task_input_killed: kill, restart roundtrip");
}

/// PANEL-TASK-003 (`holla/flows/task/sources`): the finder query over
/// the yarn-script task sources.
///
/// Flow: boot, type `yarn`, Esc (cleared), type `yarn` again.
#[test]
#[ignore = "holla h2 check; run with --ignored"]
fn h2_task_sources() {
    let dir = h2_dir("h2_task_sources");
    let case = h2_case("task_sources", TASKSRC, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // A1: typing filters to the yarn task sources.
    support::drive_with_timeout(&mut s, &["type:yarn", "wait:yarn s00"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    // V1: the result count.
    assert!(res.contains("Results · 31"), "V1: no Results · 31");
    // V2: the top row pairs the script with its manifest.
    assert_line_has(&res, "› yarn dev", "package.json here", "V2 row");
    // S1: the preview names the run command.
    assert!(res.contains("yarn run dev"), "S1: no run command");
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
    support::drive_with_timeout(&mut s, &["type:yarn", "wait:yarn s00"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-results-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "› yarn dev", "package.json here", "row intact");
    eprintln!("h2 task_sources: results, clear roundtrip");
}

/// PANEL-TASK-004 (`holla/flows/task/sources_diagnostic`): the finder
/// row for the broken-Taskfile discovery diagnostic.
///
/// Flow: boot, type `Taskfile`, Esc (cleared), type `Taskfile` again.
#[test]
#[ignore = "holla h2 check; run with --ignored"]
fn h2_task_sources_diagnostic() {
    let dir = h2_dir("h2_task_sources_diagnostic");
    let case = h2_case("task_sources_diagnostic", TASKSRC, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    // A1: typing surfaces the discovery diagnostic row.
    support::drive_with_timeout(
        &mut s,
        &["type:Taskfile", "wait:discovery diagnostic"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    // V1: the single result.
    assert!(res.contains("Results · 1"), "V1: no Results · 1");
    // V2: the row pairs the Taskfile with its diagnostic.
    assert_line_has(
        &res,
        "Taskfile · Taskfile.yaml",
        "discovery diagnostic",
        "V2 row",
    );
    // S1: the preview names the yaml failure (the sentence wraps, so
    // only its first visual line is a contiguous needle).
    assert!(
        res.contains("mapping values are not"),
        "S1: no yaml failure"
    );
    // S2: the risk reads read-only.
    assert_line_has(&res, "Risk", "read-only", "S2 risk");
    // N1: the empty-query placeholder is gone with the result.
    assert!(!res.contains(PLACEHOLDER), "N1: placeholder stuck");

    // A2: Esc clears the query back to the placeholder.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Taskfile · Taskfile.yaml", "result cleared");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);
    // N2: the diagnostic is gone after the clear.
    assert!(
        !cleared.contains("discovery diagnostic"),
        "N2: diagnostic stuck"
    );
    support::drive_with_timeout(
        &mut s,
        &["type:Taskfile", "wait:discovery diagnostic"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "03-results-again");
    let back = live_text(&mut s);
    assert_line_has(
        &back,
        "Taskfile · Taskfile.yaml",
        "discovery diagnostic",
        "row intact",
    );
    eprintln!("h2 task_sources_diagnostic: diagnostic, clear roundtrip");
}
