//! Holla pending-roots slice 8A-H7 executable checks (VB Phase-8aj).
//!
//! One ignored PTY test per H7 registry row (2 rows: the last 2 `holla`
//! rows with `results.status == "unrun"` after H1-H6 — closing the whole
//! 293-row registry to zero unrun), over the real `holla` binary built
//! from this worktree's verified VB sources (`env!("CARGO_BIN_EXE_holla")`,
//! the harness `[[bin]]` compiled from `../../src/bin/holla/main.rs`).
//! Each test drives the row's specified inputs at the row's specified
//! viewport (SB at 80x24, VP at 120x40, both truecolor) and asserts its
//! visual (V), state (S), action (A), and negative (N) checks in live-PTY
//! executable form:
//!
//! - V: live needles plus same-line coexistence for the status bar
//!   (shell items, screen bits, the busy chip, the disk meter) and the
//!   file preview (numbered gutter, truncation meta, scrollbar, find
//!   counts). Tones (strong/faint/primary/accent, underline/reverse)
//!   have no PTY-text form: priority is proven by narrow-drop/wide-return
//!   roundtrips, the busy spinner by glyph adjacency to the chip, and
//!   find marks by the `N of M` count plus current navigation.
//! - S: live labels (status bar, footers, preview meta) plus pre/post
//!   comparisons from the same session; follow-off is proven by the top
//!   hold (follow-on forces the tail), and the screen-bit swap by
//!   navigating while the shell items persist.
//! - A: behavior probes — status clicks land (copy, snapshot, picker),
//!   the find count/clear/navigate cycle runs, `y` copies the selection;
//!   refused gestures are proven by byte-identical screens, since PTY
//!   cannot see the outcome enum.
//! - N: absence assertions, each paired with a presence proof in the same
//!   test (second session, second geometry, or pre/post transition) so no
//!   absence passes vacuously.
//!
//! No new snapshots and no new static captures: both rows already have
//! approved frames gated cell-exact by the ported matrices in `holla.rs`
//! (plan §Reconciliation: "rows plus checks, not recaptures"), so every
//! case here uses owned (dynamic) names and stays out of the `snapshots/`
//! inventory. No isolated-component captures: every row's `requires` set
//! (key-injection, glyph-capture, tick-control) is PTY-level, and every
//! assertion has a live-PTY executable form.
//!
//! Per-test scratch lands under this harness crate's
//! `target/tuiscotti/journeys/<test>/` (gitignored): the row's named
//! checkpoint(s) as text frames, `00-*-boot.txt` per session where the
//! row needs a boot comparison, and `provenance*.txt` (binary path, size,
//! mtime, argv; one per session in the multi-session status test). Typed
//! input is synthetic and in-memory only (simulation data); no test runs
//! real commands, touches Git state, or performs destructive operations:
//! status clicks only copy, open read-only snapshots, or open pickers,
//! and every world is the fixture simulation.
//!
//! Row → test map (registry id → `h7_*` test):
//!
//! - SB-SCREENS-003 → [`h7_statusbar_screens`]
//! - VP-PREVIEW-002 → [`h7_preview_find`]
//!
//! Fixture map (abstract row state → concrete fixture world):
//!
//! - SB `live: 2` → the activities-multi world (`live: 4`): no 2-live
//!   fixture exists, and the busy `N running` chip plus the primary
//!   spinner is the same `StatusItem::busy().chip()` path at any N.
//! - SB `disk_pct: 91` → the disk-full worlds (842/926 GiB): disk-cleanup
//!   shows the meter while discovery (611/926, 66%) hides it.
//! - SB `cwd: ~/src` → each world's own cwd (`~/work/probe`, `~/work/acme`,
//!   `~/work`, `~/work/holla`): the path item is the shell's cwd short
//!   form in every world.
//! - VP `big.log` → the parity-browser world's `~/work/site/big.log`
//!   (2100 `line N` rows, the preview capped at the first 2000).

use std::path::{Path, PathBuf};
use std::time::Duration;

use tuiscotti::tui::{MouseButton, MouseMods, Session};

use crate::support::state_waits as waits;
use crate::support::{self, Case, Color, HOLLA};

/// Boot needle on every holla route: the brand.
const BOOT: &str = "holla❯";
/// Empty finder query placeholder (boot/cleared states).
const PLACEHOLDER: &str = "Search actions and resources…";
/// Finder footer gesture hint (boot states; pages and dialogs replace it).
const TYPE_SEARCH: &str = "Type Search";

const DISCOVERY: &[&str] = &[
    "--scenario",
    "parity-discovery",
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
const DISKCLEAN: &[&str] = &[
    "--scenario",
    "disk-cleanup",
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
const BROWSER: &[&str] = &["--scenario", "parity-browser", "--motion", "reduced"];

/// Owned-name H7 case at the canonical 120x40 truecolor geometry.
fn h7_case(slug: &str, args: &'static [&'static str], boot: &'static str) -> Case {
    h7_case_geom(slug, args, boot, 120, 40)
}

/// Owned-name H7 case at an explicit geometry (the status row boots at
/// its specified 80x24 and resizes into 120x40 and back).
fn h7_case_geom(
    slug: &str,
    args: &'static [&'static str],
    boot: &'static str,
    cols: u16,
    rows: u16,
) -> Case {
    Case::dynamic(
        format!("journeys/holla/h7/{slug}"),
        HOLLA,
        args,
        cols,
        rows,
        Color::Truecolor,
        boot,
    )
}

/// [`h7_case_geom`] with an explicit per-step timeout for tick-driven flows.
fn h7_case_geom_t(
    slug: &str,
    args: &'static [&'static str],
    boot: &'static str,
    cols: u16,
    rows: u16,
    timeout_ms: u64,
) -> Case {
    h7_case_geom(slug, args, boot, cols, rows).timeout(timeout_ms)
}

/// Unique scratch dir for one H7 test (created, never shared).
fn h7_dir(test: &str) -> PathBuf {
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
    eprintln!("h7 provenance ({file}): {body}");
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

/// Absence wait for a just-landed transition, without the vacuous-absence
/// guard. Every call site proves presence in its pre-transition sample,
/// so the guard would only add a race: a descheduled test thread can take
/// its guard snapshot after the app already repainted, failing a correct
/// transition (observed live on the shrink-back repaint). The predicate
/// form passes at once when the transition already landed.
fn wait_absent(s: &mut Session, needle: &str, what: &str) {
    support::wait_screen(
        s,
        support::DEFAULT_WAIT,
        &format!("{what}: `{needle}` never expired"),
        |screen| !support::screen_text(screen).contains(needle),
    );
}

/// First occurrence of `needle` as `(col, row)` for mouse targeting,
/// from already-sampled text (the target is stable on screen, so no
/// wait is needed). Display columns resolve wide cells exactly.
fn first_pos(text: &str, needle: &str) -> (u16, u16) {
    text.lines()
        .position(|l| l.contains(needle))
        .map(|row| {
            let line = text.lines().nth(row).expect("row from this text");
            let off = line.find(needle).expect("line holds the needle");
            let col = unicode_width::UnicodeWidthStr::width(&line[..off]) as u16;
            (col, row as u16)
        })
        .expect("needle on screen")
}

/// Some line must contain both `needle` and `also` (the same-row
/// coexistence proof: status rows, preview rows, picker rows).
fn assert_line_has(text: &str, needle: &str, also: &str, what: &str) {
    assert!(
        text.lines().any(|l| l.contains(needle) && l.contains(also)),
        "{what}: no line contains both `{needle}` and `{also}`"
    );
}

/// The status bar: the second-to-last non-empty screen line (the footer
/// hint bar is the last). Scopes meter/center/shell asserts to the bar so
/// body separators and fact rows cannot satisfy them.
fn status_line(text: &str) -> &str {
    text.lines()
        .rev()
        .filter(|l| !l.trim().is_empty())
        .nth(1)
        .unwrap_or("")
}

/// Last occurrence of `needle` as `(row, col)`, waiting until it appears.
/// Status items repeat body text (`main`, the cwd), and the bar paints
/// below the body, so the last occurrence is the bar hit region.
fn find_last_pos(s: &mut Session, needle: &str, timeout: Duration) -> (u16, u16) {
    support::wait_screen(
        s,
        timeout,
        &format!("`{needle}` never appeared"),
        |screen| support::screen_text(screen).contains(needle),
    );
    let obs = s
        .observe_now()
        .unwrap_or_else(|e| panic!("last-pos sample failed: {e:#}"));
    let text = support::screen_text(&obs.screen);
    let lines: Vec<&str> = text.lines().collect();
    lines
        .iter()
        .rposition(|l| l.contains(needle))
        .map(|row| {
            let off = lines[row].rfind(needle).expect("line holds the needle");
            let col = unicode_width::UnicodeWidthStr::width(&lines[row][..off]) as u16;
            (row as u16, col)
        })
        .expect("wait passed with the needle on screen")
}

/// Left-click the last occurrence of `needle` shifted by `dcol` display
/// columns (into the status item, off its leading edge).
fn click_last(s: &mut Session, needle: &str, dcol: i16, timeout: Duration, what: &str) {
    let (row, col) = find_last_pos(s, needle, timeout);
    let target = col.saturating_add_signed(dcol);
    s.click(MouseButton::Left, target, row, MouseMods::NONE)
        .unwrap_or_else(|e| panic!("{what}: click failed: {e:?}"));
    std::thread::sleep(Duration::from_millis(400));
}

/// SB-SCREENS-003: the status bar shell items (path, git, size) plus one
/// screen's bits, the busy chip, and the disk meter.
///
/// Flow: four sessions, all truecolor. (A) discovery at 80x24: shell
/// items plus screen bits, the config-page bit swap with the shell
/// untouched, path and git clicks, then the 120x40 grow and the shrink
/// back proving the prio-1 size cell drops first and returns with room
/// while the prio-10 path survives. (B) activities-multi: the busy
/// `4 running` chip plus its spinner, and the chip click opening the
/// Activities picker. (C) disk-cleanup: the inline disk meter plus the
/// disk screen's center. (D) history: shell items only, no center.
#[test]
#[ignore = "holla h7 check; run with --ignored"]
fn h7_statusbar_screens() {
    const CENTER: &str = "! docker, github unavailable";
    const RIGHT: &str = "▲ 4 config warnings";
    const SWAPPED: &str = "snapshot · read-only";
    const NARROW: &str = "truecolor · 80×24";
    const WIDE: &str = "truecolor · 120×40";
    const PATH: &str = "~/work/probe";
    const COPIED: &str = "Copied /Users/alex/work/probe";
    let dir = h7_dir("h7_statusbar_screens");

    // Session A: discovery at the row's 80x24.
    let case_a = h7_case_geom("statusbar_discovery", DISCOVERY, BOOT, 80, 24);
    write_provenance_named(&dir, &case_a, "provenance-a.txt");
    let mut a = support::spawn_boot(&case_a);
    checkpoint(&mut a, &dir, "00-a-boot");
    // The footer paints after the brand needle; settle it before the
    // boot asserts so the pre-checks cannot race the stagger.
    waits::wait_state(&mut a, TYPE_SEARCH, "a boot settled");
    let boot = live_text(&mut a);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");
    assert!(boot.contains(TYPE_SEARCH), "pre: boot has no finder footer");

    // S1: the center and the extra right item come from the discovery
    // screen's StatusBits.
    let bar = status_line(&boot).to_string();
    assert!(bar.contains(CENTER), "S1: no screen center");
    assert!(bar.contains(RIGHT), "S1: no screen right");
    // V1 narrow: the prio-10 path item survives the narrow bar while the
    // prio-1 size cell drops first.
    assert!(bar.contains(PATH), "V1: no path item");
    assert_line_has(&boot, PATH, "main", "V1 shell");
    assert!(
        !boot.contains(NARROW),
        "V1: size cell stuck at narrow width"
    );
    // N1: no disk meter at 66% (below the 85% floor). The body carries
    // rule separators, so the absence is scoped to the bar; session C
    // proves the meter paints when the floor is met.
    assert!(!bar.contains('━'), "N1: meter run stuck");
    assert!(!bar.contains('%'), "N1: meter pct stuck");

    // A2: the config page swaps the screen bits without touching the
    // shell items.
    support::drive(
        &mut a,
        &["type:Custom action configuration", "enter", "wait:sha256:"],
        case_timeout(&case_a),
    );
    checkpoint(&mut a, &dir, "01-a-config");
    let config = live_text(&mut a);
    let cbar = status_line(&config).to_string();
    assert!(cbar.contains(SWAPPED), "A2: no swapped center");
    assert!(cbar.contains(PATH), "A2: path moved");
    assert!(cbar.contains("main"), "A2: git moved");
    assert!(!cbar.contains(CENTER), "A2: old center stuck");
    support::press_step(&a, "escape");
    waits::wait_gone(&mut a, "sha256:", "config closed");
    support::press_step(&a, "escape");
    waits::wait_state(&mut a, PLACEHOLDER, "query cleared");
    checkpoint(&mut a, &dir, "02-a-finder");
    let back = live_text(&mut a);
    assert!(
        status_line(&back).contains(CENTER),
        "A2: center not restored"
    );

    // A1 path: clicking the path item copies the cwd.
    click_last(&mut a, PATH, 1, case_timeout(&case_a), "A1 path click");
    waits::wait_state(&mut a, COPIED, "path copied");
    checkpoint(&mut a, &dir, "03-a-copied");

    // A1 git: clicking the branch opens the git status snapshot.
    click_last(&mut a, "main", 1, case_timeout(&case_a), "A1 git click");
    waits::wait_state(&mut a, "Git status", "git snapshot");
    checkpoint(&mut a, &dir, "04-a-git");
    let git = live_text(&mut a);
    assert!(git.contains("Git status"), "A1: no git snapshot");
    assert!(git.contains("main"), "A1: snapshot lost the branch");
    support::press_step(&a, "escape");
    waits::wait_state(&mut a, PLACEHOLDER, "finder back");

    // V1 wide: room returns the prio-1 size cell; the path stays.
    a.resize(120, 40).expect("resize to 120x40");
    waits::wait_size(&mut a, 120, 40, "wide geometry");
    waits::wait_state(&mut a, WIDE, "wide repaint");
    checkpoint(&mut a, &dir, "05-a-wide");
    let wide = live_text(&mut a);
    let wbar = status_line(&wide).to_string();
    assert!(wbar.contains(WIDE), "V1: no size cell at wide");
    assert!(wbar.contains(PATH), "V1: path gone at wide");
    assert!(wbar.contains(CENTER), "V1: center gone at wide");
    // Shrink back: the size cell drops again, the path survives.
    // Guard-free: the repaint routinely lands before a guard snapshot.
    a.resize(80, 24).expect("resize to 80x24");
    waits::wait_size(&mut a, 80, 24, "narrow geometry");
    wait_absent(&mut a, "120×40", "narrow repaint");
    checkpoint(&mut a, &dir, "06-a-narrow");
    let narrow = live_text(&mut a);
    assert!(!narrow.contains(NARROW), "V1: size cell stuck after shrink");
    assert!(
        status_line(&narrow).contains(PATH),
        "V1: path gone after shrink"
    );

    // Session B: activities-multi carries the live chip.
    let case_b = h7_case_geom_t("statusbar_running", ACTMULTI, BOOT, 80, 24, 20_000);
    write_provenance_named(&dir, &case_b, "provenance-b.txt");
    let mut b = support::spawn_boot(&case_b);
    checkpoint(&mut b, &dir, "10-b-boot");
    waits::wait_state(&mut b, "4 running", "b live chip");
    let bb = live_text(&mut b);
    // V2 chip half plus S2: the busy spinner glyph sits on the chip row.
    assert_line_has(&bb, "⠋", "4 running", "V2 busy chip");
    // A1 running: clicking the chip opens the Activities picker.
    click_last(
        &mut b,
        "4 running",
        1,
        case_timeout(&case_b),
        "A1 running click",
    );
    waits::wait_state(&mut b, "Activities", "activities picker");
    checkpoint(&mut b, &dir, "11-b-activities");
    let acts = live_text(&mut b);
    assert!(acts.contains("Activities"), "A1: no picker title");
    assert!(acts.contains("frontend dev"), "A1: no activity row");
    support::press_step(&b, "escape");
    // Guard-free: the tab strip keeps a `frontend dev` row, so the picker
    // title is the absence needle, and the close routinely lands before
    // a guard snapshot.
    wait_absent(&mut b, "Activities", "picker closed");
    waits::wait_state(&mut b, PLACEHOLDER, "finder back");

    // Session C: disk-cleanup at 91% shows the inline meter.
    let case_c = h7_case_geom("statusbar_meter", DISKCLEAN, BOOT, 80, 24);
    write_provenance_named(&dir, &case_c, "provenance-c.txt");
    let mut c = support::spawn_boot(&case_c);
    checkpoint(&mut c, &dir, "20-c-boot");
    waits::wait_state(&mut c, "3 selected · 24.2 GiB", "c disk settled");
    let cb = live_text(&mut c);
    let cbar = status_line(&cb).to_string();
    // V2 meter half: the mount plus the run cells plus the percent.
    assert!(cbar.contains("/ ━"), "V2: no meter run");
    assert!(cbar.contains("91%"), "V2: no meter pct");
    // S1 second screen: the disk center plus the shell path.
    assert!(cbar.contains("3 selected · 24.2 GiB"), "S1: no disk center");
    assert!(cbar.contains("~/work"), "S1: no disk path");

    // Session D: history offers no center, so none appears.
    let case_d = h7_case_geom("statusbar_history", HISTORY, BOOT, 80, 24);
    write_provenance_named(&dir, &case_d, "provenance-d.txt");
    let mut d = support::spawn_boot(&case_d);
    checkpoint(&mut d, &dir, "30-d-boot");
    waits::wait_state(&mut d, TYPE_SEARCH, "d boot settled");
    let db = live_text(&mut d);
    let dbar = status_line(&db).to_string();
    assert!(dbar.contains("~/work/holla"), "N2: no path");
    assert!(dbar.contains("main"), "N2: no git");
    assert!(dbar.contains(NARROW), "N2: no size");
    // N2: neither the failed-source center nor the warning right survive
    // on a screen offering neither; session A proves both paint.
    assert!(!dbar.contains('!'), "N2: center stuck");
    assert!(!dbar.contains('▲'), "N2: right stuck");
    eprintln!("h7 statusbar_screens: shell, bits, chip, meter, clicks, prio");
}

/// VP-PREVIEW-002: the file preview pane (numbered gutter, truncation
/// meta, scrollbar) plus the in-preview find with its marks.
///
/// Flow: boot (parity-browser), browse `~/work/site`, Down x3 (big.log),
/// Right (preview focus), `/` (find), type `line 1` (1111 matches),
/// Enter/Up (current navigation), Backspace x6 (empty query clears),
/// Esc (find closed), End/Home (tail roundtrip), `y` alone (refused),
/// drag-select `line 0` plus `y` (copies the selected source bytes).
/// Keyboard shift+Left cannot select here: the app's `plain()` counts
/// shift as plain, so the screen-level Left arm steals focus back to the
/// list before the viewport ever sees the chord (observed live: the list
/// walked to `/`). The drag is the executable selection path.
#[test]
#[ignore = "holla h7 check; run with --ignored"]
fn h7_preview_find() {
    const META: &str = "first 2000 lines · 19.4 KiB of 19.4 KiB";
    const TOP: &str = "1 line 0";
    const TAIL: &str = "2000 line 1999";
    const QUERY: &str = "line 1";
    const COUNT: &str = "of 1111";
    let dir = h7_dir("h7_preview_find");
    let case = h7_case("preview_find", BROWSER, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    // A1-setup: three downs land the cursor on big.log and the preview
    // follows with the truncation meta.
    support::drive(
        &mut s,
        &[
            "type:Browse ~/work/site",
            "enter",
            "wait:16 entries",
            "down",
            "down",
            "down",
            "sleep:300",
        ],
        case_timeout(&case),
    );
    waits::wait_state(&mut s, META, "preview loaded");
    checkpoint(&mut s, &dir, "01-preview");
    let preview = live_text(&mut s);

    // V1: the numbered gutter, the truncation meta, the scrollbar thumb.
    assert!(preview.contains(META), "V1: no truncation meta");
    assert!(preview.contains(TOP), "V1: no gutter top");
    assert!(preview.contains('┃'), "V1: no scrollbar thumb");
    // S-cursor: the preview follows the big.log row with its size.
    assert_line_has(&preview, "big.log", "19.4 KiB", "S1 cursor");
    // S1: the load holds the top with follow off (follow-on would force
    // the tail, so the top hold proves both halves).
    assert!(preview.contains(TOP), "S1: preview missed the top");
    // N1: the tail stays out after the load; the End roundtrip below
    // proves it is reachable, not missing.
    assert!(!preview.contains(TAIL), "N1: preview followed the tail");

    // Right moves focus into the preview pane.
    support::drive(
        &mut s,
        &["right", "wait:Shift+↑↓ Select"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "02-focused");
    let focused = live_text(&mut s);
    assert!(focused.contains("Shift+↑↓ Select"), "pre: no select footer");
    assert_line_has(&focused, "y", "Copy", "pre copy hint");
    assert_line_has(&focused, "/", "Find", "pre find hint");

    // `/` opens the find field; typing maps matches into marks.
    support::press_step(&s, "/");
    waits::wait_state(&mut s, "find “”", "find opened");
    checkpoint(&mut s, &dir, "03-find-open");
    support::drive(
        &mut s,
        &[&format!("type:{QUERY}"), &format!("wait:{COUNT}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "04-found");
    let found = live_text(&mut s);
    // V2 count half: every `line 1*` row matches through the gutter.
    assert!(found.contains("find “line 1”"), "V2: no find meta");
    assert!(found.contains("1 of 1111"), "V2: no match count");
    // S2: the current mark walks the gutter-mapped matches.
    support::drive(&mut s, &["enter", "wait:2 of 1111"], case_timeout(&case));
    support::drive(&mut s, &["up", "wait:1 of 1111"], case_timeout(&case));
    checkpoint(&mut s, &dir, "05-navigated");

    // A1: backspacing to the empty query clears the marks instead of
    // searching; the 1111 above prove the clear is real, not vacuous.
    support::drive(
        &mut s,
        &[
            "backspace",
            "backspace",
            "backspace",
            "backspace",
            "backspace",
            "backspace",
            "wait:0 of 0",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "06-cleared");
    let cleared = live_text(&mut s);
    assert!(cleared.contains("find “” · 0 of 0"), "A1: marks stuck");

    // Esc closes the find field and the truncation meta returns.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "find “", "find closed");
    waits::wait_state(&mut s, META, "meta restored");
    checkpoint(&mut s, &dir, "07-closed");

    // N1 pair: End reaches the tail, Home holds the top again.
    support::drive(
        &mut s,
        &["end", &format!("wait:{TAIL}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "08-tail");
    let tail = live_text(&mut s);
    assert!(tail.contains(TAIL), "N1: tail unreachable");
    support::drive(
        &mut s,
        &["home", &format!("wait:{TOP}")],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "09-top");
    let top = live_text(&mut s);
    assert!(top.contains(TOP), "N1: top not restored");
    assert!(!top.contains(TAIL), "N1: tail stuck after home");

    // A2 setup: `y` with no selection is consumed silently.
    support::press_step(&s, "y");
    std::thread::sleep(Duration::from_millis(300));
    let refused = live_text(&mut s);
    assert!(!refused.contains("Copied"), "A2: copy without selection");

    // A2: drag-select the first row's content and `y` emits Copy with
    // the source bytes. The drag covers exactly `line 0` (6 cells) and
    // deliberately never releases: an Up on the same widget reads as a
    // click, and the click clears the selection it just made.
    let (col, row) = first_pos(&refused, "line 0");
    s.mouse_down(MouseButton::Left, col, row, MouseMods::NONE)
        .expect("drag press");
    for c in col + 1..=col + 6 {
        s.mouse_drag(MouseButton::Left, c, row, MouseMods::NONE)
            .unwrap_or_else(|e| panic!("drag motion to ({c}, {row}) failed: {e:?}"));
    }
    support::drive(&mut s, &["y", "wait:Copied"], case_timeout(&case));
    checkpoint(&mut s, &dir, "10-copied");
    let copied = live_text(&mut s);
    assert!(copied.contains("Copied line 0"), "A2: no exact copy");
    let line = copied
        .lines()
        .find(|l| l.contains("Copied"))
        .expect("A2: no Copied status");
    let rest = line.split("Copied ").nth(1).unwrap_or("");
    assert!(
        rest.starts_with("line 0"),
        "A2: copied text is not the row content: `{rest}`"
    );
    assert!(!rest.contains('·'), "A2: copied text spans lines: `{rest}`");
    // N2: the copied bytes carry no display gutter and no wrapped
    // shaping (wrap is off); the preview above still shows the gutter,
    // so the exclusion is proven against its presence.
    assert!(
        !rest.starts_with(' ') && !rest.starts_with("1 line"),
        "N2: copied text carries the gutter: `{rest}`"
    );
    assert!(copied.contains(TOP), "N2: gutter proof gone");
    eprintln!("h7 preview_find: gutter, find, marks, clear, copy");
}
