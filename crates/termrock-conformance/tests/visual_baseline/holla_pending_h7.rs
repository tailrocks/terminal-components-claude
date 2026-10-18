#![doc = r" Holla pending-roots slice 8A-H7 executable checks — impl port."]
#![doc = r""]
#![doc = r" Ported verbatim from VB commit `b682cb26d68b353aeeccf9e51653eddf097b39f5`"]
#![doc = r" (`tests/harness/tests/visual_baseline/holla_pending_h7.rs`), adapting"]
#![doc = r" only paths, binary resolution, and harness API:"]
#![doc = r""]
#![doc = r" - `Session` is the impl [`support::Session`] wrapper (live samples via"]
#![doc = r"   `s.inner.observe_now()`), not `tuiscotti::tui::Session` directly."]
#![doc = r" - The subject is the `holla` binary built from this impl worktree's"]
#![doc = r"   sources, resolved via [`support::try_resolve_bin`] (name -> executed"]
#![doc = r"   path + sha256); [`write_provenance`] records path, digest, size,"]
#![doc = r"   mtime, and argv behind each check."]
#![doc = r" - `support::drive` (3-arg) becomes [`support::drive_with_timeout`]"]
#![doc = r"   (timeout in ms, via [`drive_ms`])."]
#![doc = r" - `Session::click` (4-arg with button+mods) becomes the wrapper's"]
#![doc = r"   2-arg `click` (primary button, no mods); the held drag-select keeps"]
#![doc = r"   `s.inner.mouse_down`/`mouse_drag` (never released, S5 precedent)."]
#![doc = r""]
#![doc = r" One ignored PTY test per H7 registry row (2 rows: the last 2 `holla`"]
#![doc = r#" rows with `results.status == "unrun"` after H1-H6 — closing the whole"#]
#![doc = r" 293-row registry to zero unrun), over the real `holla` binary built"]
#![doc = r" from this worktree's impl sources (resolved via"]
#![doc = r" [`support::try_resolve_bin`])."]
#![doc = r" Each test drives the row's specified inputs at the row's specified"]
#![doc = r" viewport (SB at 80x24, VP at 120x40, both truecolor) and asserts its"]
#![doc = r" visual (V), state (S), action (A), and negative (N) checks in live-PTY"]
#![doc = r" executable form:"]
#![doc = r""]
#![doc = r" - V: live needles plus same-line coexistence for the status bar"]
#![doc = r"   (shell items, screen bits, the busy chip, the disk meter) and the"]
#![doc = r"   file preview (numbered gutter, truncation meta, scrollbar, find"]
#![doc = r"   counts). Tones (strong/faint/primary/accent, underline/reverse)"]
#![doc = r"   have no PTY-text form: priority is proven by narrow-drop/wide-return"]
#![doc = r"   roundtrips, the busy spinner by glyph adjacency to the chip, and"]
#![doc = r"   find marks by the `N of M` count plus current navigation."]
#![doc = r" - S: live labels (status bar, footers, preview meta) plus pre/post"]
#![doc = r"   comparisons from the same session; follow-off is proven by the top"]
#![doc = r"   hold (follow-on forces the tail), and the screen-bit swap by"]
#![doc = r"   navigating while the shell items persist."]
#![doc = r" - A: behavior probes — status clicks land (copy, snapshot, picker),"]
#![doc = r"   the find count/clear/navigate cycle runs, `y` copies the selection;"]
#![doc = r"   refused gestures are proven by byte-identical screens, since PTY"]
#![doc = r"   cannot see the outcome enum."]
#![doc = r" - N: absence assertions, each paired with a presence proof in the same"]
#![doc = r"   test (second session, second geometry, or pre/post transition) so no"]
#![doc = r"   absence passes vacuously."]
#![doc = r""]
#![doc = r" No new snapshots and no new static captures: both rows already have"]
#![doc = r" approved frames gated cell-exact by the ported matrices in `holla.rs`"]
#![doc = r#" (plan §Reconciliation: "rows plus checks, not recaptures"), so every"#]
#![doc = r" case here uses owned (dynamic) names and stays out of the `snapshots/`"]
#![doc = r" inventory. No isolated-component captures: every row's `requires` set"]
#![doc = r" (key-injection, glyph-capture, tick-control) is PTY-level, and every"]
#![doc = r" assertion has a live-PTY executable form."]
#![doc = r""]
#![doc = r" Per-test scratch lands under this conformance crate's"]
#![doc = r" `target/tuiscotti/journeys/<test>/` (gitignored): the row's named"]
#![doc = r" checkpoint(s) as text frames, `00-*-boot.txt` per session where the"]
#![doc = r" row needs a boot comparison, and `provenance*.txt` (binary path,"]
#![doc = r" digest, size, mtime, argv; one per session in the multi-session status"]
#![doc = r" test). Typed input is synthetic and in-memory only (simulation data);"]
#![doc = r" no test runs real commands, touches Git state, or performs destructive"]
#![doc = r" operations: status clicks only copy, open read-only snapshots, or open"]
#![doc = r" pickers, and every world is the fixture simulation."]
#![doc = r""]
#![doc = r" Row → test map (registry id → `h7_*` test):"]
#![doc = r""]
#![doc = r" - SB-SCREENS-003 → [`h7_statusbar_screens`]"]
#![doc = r" - VP-PREVIEW-002 → [`h7_preview_find`]"]
#![doc = r""]
#![doc = r" Fixture map (abstract row state → concrete fixture world):"]
#![doc = r""]
#![doc = r" - SB `live: 2` → the activities-multi world (`live: 4`): no 2-live"]
#![doc = r"   fixture exists, and the busy `N running` chip plus the primary"]
#![doc = r"   spinner is the same `StatusItem::busy().chip()` path at any N."]
#![doc = r" - SB `disk_pct: 91` → the disk-full worlds (842/926 GiB): disk-cleanup"]
#![doc = r"   shows the meter while discovery (611/926, 66%) hides it."]
#![doc = r" - SB `cwd: ~/src` → each world's own cwd (`~/work/probe`, `~/work/acme`,"]
#![doc = r"   `~/work`, `~/work/holla`): the path item is the shell's cwd short"]
#![doc = r"   form in every world."]
#![doc = r" - VP `big.log` → the parity-browser world's `~/work/site/big.log`"]
#![doc = r"   (2100 `line N` rows, the preview capped at the first 2000)."]

use std::path::{Path, PathBuf};
use std::time::Duration;

use tuiscotti::tui::{MouseButton, MouseMods};

use crate::support::Session;
use crate::support::state_waits as waits;
use crate::support::{self, Case, Color, HOLLA};

#[doc = r" Boot needle on every holla route: the brand."]
const BOOT: &str = "holla❯";
#[doc = r" Empty finder query placeholder (boot/cleared states)."]
const PLACEHOLDER: &str = "Search actions and resources…";
#[doc = r" Finder footer gesture hint (boot states; pages and dialogs replace it)."]
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

#[doc = r" Owned-name H7 case at the canonical 120x40 truecolor geometry."]
fn h7_case(slug: &str, args: &'static [&'static str], boot: &'static str) -> Case {
    h7_case_geom(slug, args, boot, 120, 40)
}

#[doc = r" Owned-name H7 case at an explicit geometry (the status row boots at"]
#[doc = r" its specified 80x24 and resizes into 120x40 and back)."]
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

#[doc = r" [`h7_case_geom`] with an explicit per-step timeout for tick-driven flows."]
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

#[doc = r" Unique scratch dir for one H7 test (created, never shared)."]
fn h7_dir(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/tuiscotti/journeys")
        .join(test);
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("mkdir {}: {e}", dir.display()));
    dir
}

#[doc = r" Record the binary path + identity: absolute path, byte size, mtime, argv."]
#[doc = r" Written to `provenance.txt`, echoed too (the build-identity record)."]
fn write_provenance(dir: &Path, case: &Case) {
    write_provenance_named(dir, case, "provenance.txt");
}

#[doc = r" [`write_provenance`] with an explicit filename, for multi-session tests"]
#[doc = r" where each session's argv deserves its own record."]
fn write_provenance_named(dir: &Path, case: &Case, file: &str) {
    let subject =
        support::try_resolve_bin(case.bin).unwrap_or_else(|e| panic!("resolve {}: {e}", case.bin));
    let modified = subject
        .mtime_unix
        .map_or_else(|| "unknown".to_string(), |m| m.to_string());
    let argv = support::argv_for(case);
    let body = format!(
        "profile: tuiscotti-default\ntransport: pty\nport_of: b682cb26d68b353aeeccf9e51653eddf097b39f5\nbinary: {}\nsha256: {}\nsize_bytes: {}\nmodified_unix: {modified}\nargv: {}\n",
        subject.path.display(),
        subject.sha256,
        subject.len,
        argv.join(" "),
    );
    std::fs::write(dir.join(file), &body)
        .unwrap_or_else(|e| panic!("write {}: {e}", dir.join(file).display()));
    eprintln!("h7 provenance ({file}): {body}");
}

#[doc = r" Save the live screen text as `<name>.txt` in the test dir."]
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

fn drive_ms(timeout: Duration) -> u64 {
    u64::try_from(timeout.as_millis()).expect("case timeout fits u64 ms")
}

#[doc = r" One fresh live text sample (no wait)."]
fn live_text(s: &mut Session) -> String {
    let obs = s
        .inner
        .observe_now()
        .unwrap_or_else(|e| panic!("live sample failed: {e:#}"));
    support::screen_text(&obs.screen)
}

#[doc = r" Absence wait for a just-landed transition, without the vacuous-absence"]
#[doc = r" guard. Every call site proves presence in its pre-transition sample,"]
#[doc = r" so the guard would only add a race: a descheduled test thread can take"]
#[doc = r" its guard snapshot after the app already repainted, failing a correct"]
#[doc = r" transition (observed live on the shrink-back repaint). The predicate"]
#[doc = r" form passes at once when the transition already landed."]
fn wait_absent(s: &mut Session, needle: &str, what: &str) {
    support::wait_screen(
        s,
        support::DEFAULT_WAIT,
        &format!("{what}: `{needle}` never expired"),
        |screen| !support::screen_text(screen).contains(needle),
    );
}

#[doc = r" First occurrence of `needle` as `(col, row)` for mouse targeting,"]
#[doc = r" from already-sampled text (the target is stable on screen, so no"]
#[doc = r" wait is needed). Display columns resolve wide cells exactly."]
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

#[doc = r" Some line must contain both `needle` and `also` (the same-row"]
#[doc = r" coexistence proof: status rows, preview rows, picker rows)."]
fn assert_line_has(text: &str, needle: &str, also: &str, what: &str) {
    assert!(
        text.lines().any(|l| l.contains(needle) && l.contains(also)),
        "{what}: no line contains both `{needle}` and `{also}`"
    );
}

#[doc = r" The status bar: the second-to-last non-empty screen line (the footer"]
#[doc = r" hint bar is the last). Scopes meter/center/shell asserts to the bar so"]
#[doc = r" body separators and fact rows cannot satisfy them."]
fn status_line(text: &str) -> &str {
    text.lines()
        .rev()
        .filter(|l| !l.trim().is_empty())
        .nth(1)
        .unwrap_or("")
}

#[doc = r" Last occurrence of `needle` as `(row, col)`, waiting until it appears."]
#[doc = r" Status items repeat body text (`main`, the cwd), and the bar paints"]
#[doc = r" below the body, so the last occurrence is the bar hit region."]
fn find_last_pos(s: &mut Session, needle: &str, timeout: Duration) -> (u16, u16) {
    support::wait_screen(
        s,
        timeout,
        &format!("`{needle}` never appeared"),
        |screen| support::screen_text(screen).contains(needle),
    );
    let obs = s
        .inner
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

#[doc = r" Left-click the last occurrence of `needle` shifted by `dcol` display"]
#[doc = r" columns (into the status item, off its leading edge)."]
fn click_last(s: &mut Session, needle: &str, dcol: i16, timeout: Duration, what: &str) {
    let (row, col) = find_last_pos(s, needle, timeout);
    let target = col.saturating_add_signed(dcol);
    s.click(target, row)
        .unwrap_or_else(|e| panic!("{what}: click failed: {e:?}"));
    std::thread::sleep(Duration::from_millis(400));
}

#[doc = r" SB-SCREENS-003: the status bar shell items (path, git, size) plus one"]
#[doc = r" screen's bits, the busy chip, and the disk meter."]
#[doc = r""]
#[doc = r" Flow: four sessions, all truecolor. (A) discovery at 80x24: shell"]
#[doc = r" items plus screen bits, the config-page bit swap with the shell"]
#[doc = r" untouched, path and git clicks, then the 120x40 grow and the shrink"]
#[doc = r" back proving the prio-1 size cell drops first and returns with room"]
#[doc = r" while the prio-10 path survives. (B) activities-multi: the busy"]
#[doc = r" `4 running` chip plus its spinner, and the chip click opening the"]
#[doc = r" Activities picker. (C) disk-cleanup: the inline disk meter plus the"]
#[doc = r" disk screen's center. (D) history: shell items only, no center."]
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

    let case_a = h7_case_geom("statusbar_discovery", DISCOVERY, BOOT, 80, 24);
    write_provenance_named(&dir, &case_a, "provenance-a.txt");
    let mut a = support::spawn_boot(&case_a);
    checkpoint(&mut a, &dir, "00-a-boot");

    waits::wait_state(&mut a, TYPE_SEARCH, "a boot settled");
    let boot = live_text(&mut a);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");
    assert!(boot.contains(TYPE_SEARCH), "pre: boot has no finder footer");

    let bar = status_line(&boot).to_string();
    assert!(bar.contains(CENTER), "S1: no screen center");
    assert!(bar.contains(RIGHT), "S1: no screen right");

    assert!(bar.contains(PATH), "V1: no path item");
    assert_line_has(&boot, PATH, "main", "V1 shell");
    assert!(
        !boot.contains(NARROW),
        "V1: size cell stuck at narrow width"
    );

    assert!(!bar.contains('━'), "N1: meter run stuck");
    assert!(!bar.contains('%'), "N1: meter pct stuck");

    support::drive_with_timeout(
        &mut a,
        &["type:Custom action configuration", "enter", "wait:sha256:"],
        drive_ms(case_timeout(&case_a)),
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

    click_last(&mut a, PATH, 1, case_timeout(&case_a), "A1 path click");
    waits::wait_state(&mut a, COPIED, "path copied");
    checkpoint(&mut a, &dir, "03-a-copied");

    click_last(&mut a, "main", 1, case_timeout(&case_a), "A1 git click");
    waits::wait_state(&mut a, "Git status", "git snapshot");
    checkpoint(&mut a, &dir, "04-a-git");
    let git = live_text(&mut a);
    assert!(git.contains("Git status"), "A1: no git snapshot");
    assert!(git.contains("main"), "A1: snapshot lost the branch");
    support::press_step(&a, "escape");
    waits::wait_state(&mut a, PLACEHOLDER, "finder back");

    a.resize(120, 40).expect("resize to 120x40");
    waits::wait_size(&mut a, 120, 40, "wide geometry");
    waits::wait_state(&mut a, WIDE, "wide repaint");
    checkpoint(&mut a, &dir, "05-a-wide");
    let wide = live_text(&mut a);
    let wbar = status_line(&wide).to_string();
    assert!(wbar.contains(WIDE), "V1: no size cell at wide");
    assert!(wbar.contains(PATH), "V1: path gone at wide");
    assert!(wbar.contains(CENTER), "V1: center gone at wide");

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

    let case_b = h7_case_geom_t("statusbar_running", ACTMULTI, BOOT, 80, 24, 20_000);
    write_provenance_named(&dir, &case_b, "provenance-b.txt");
    let mut b = support::spawn_boot(&case_b);
    checkpoint(&mut b, &dir, "10-b-boot");
    waits::wait_state(&mut b, "4 running", "b live chip");
    let bb = live_text(&mut b);

    assert_line_has(&bb, "⠋", "4 running", "V2 busy chip");

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

    wait_absent(&mut b, "Activities", "picker closed");
    waits::wait_state(&mut b, PLACEHOLDER, "finder back");

    let case_c = h7_case_geom("statusbar_meter", DISKCLEAN, BOOT, 80, 24);
    write_provenance_named(&dir, &case_c, "provenance-c.txt");
    let mut c = support::spawn_boot(&case_c);
    checkpoint(&mut c, &dir, "20-c-boot");
    waits::wait_state(&mut c, "3 selected · 24.2 GiB", "c disk settled");
    let cb = live_text(&mut c);
    let cbar = status_line(&cb).to_string();

    assert!(cbar.contains("/ ━"), "V2: no meter run");
    assert!(cbar.contains("91%"), "V2: no meter pct");

    assert!(cbar.contains("3 selected · 24.2 GiB"), "S1: no disk center");
    assert!(cbar.contains("~/work"), "S1: no disk path");

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

    assert!(!dbar.contains('!'), "N2: center stuck");
    assert!(!dbar.contains('▲'), "N2: right stuck");
    eprintln!("h7 statusbar_screens: shell, bits, chip, meter, clicks, prio");
}

#[doc = r" VP-PREVIEW-002: the file preview pane (numbered gutter, truncation"]
#[doc = r" meta, scrollbar) plus the in-preview find with its marks."]
#[doc = r""]
#[doc = r" Flow: boot (parity-browser), browse `~/work/site`, Down x3 (big.log),"]
#[doc = r" Right (preview focus), `/` (find), type `line 1` (1111 matches),"]
#[doc = r" Enter/Up (current navigation), Backspace x6 (empty query clears),"]
#[doc = r" Esc (find closed), End/Home (tail roundtrip), `y` alone (refused),"]
#[doc = r" drag-select `line 0` plus `y` (copies the selected source bytes)."]
#[doc = r" Keyboard shift+Left cannot select here: the app's `plain()` counts"]
#[doc = r" shift as plain, so the screen-level Left arm steals focus back to the"]
#[doc = r" list before the viewport ever sees the chord (observed live: the list"]
#[doc = r" walked to `/`). The drag is the executable selection path."]
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

    support::drive_with_timeout(
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
        drive_ms(case_timeout(&case)),
    );
    waits::wait_state(&mut s, META, "preview loaded");
    checkpoint(&mut s, &dir, "01-preview");
    let preview = live_text(&mut s);

    assert!(preview.contains(META), "V1: no truncation meta");
    assert!(preview.contains(TOP), "V1: no gutter top");
    assert!(preview.contains('┃'), "V1: no scrollbar thumb");

    assert_line_has(&preview, "big.log", "19.4 KiB", "S1 cursor");

    assert!(preview.contains(TOP), "S1: preview missed the top");

    assert!(!preview.contains(TAIL), "N1: preview followed the tail");

    support::drive_with_timeout(
        &mut s,
        &["right", "wait:Shift+↑↓ Select"],
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "02-focused");
    let focused = live_text(&mut s);
    assert!(focused.contains("Shift+↑↓ Select"), "pre: no select footer");
    assert_line_has(&focused, "y", "Copy", "pre copy hint");
    assert_line_has(&focused, "/", "Find", "pre find hint");

    support::press_step(&s, "/");
    waits::wait_state(&mut s, "find “”", "find opened");
    checkpoint(&mut s, &dir, "03-find-open");
    support::drive_with_timeout(
        &mut s,
        &[&format!("type:{QUERY}"), &format!("wait:{COUNT}")],
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "04-found");
    let found = live_text(&mut s);

    assert!(found.contains("find “line 1”"), "V2: no find meta");
    assert!(found.contains("1 of 1111"), "V2: no match count");

    support::drive_with_timeout(
        &mut s,
        &["enter", "wait:2 of 1111"],
        drive_ms(case_timeout(&case)),
    );
    support::drive_with_timeout(
        &mut s,
        &["up", "wait:1 of 1111"],
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "05-navigated");

    support::drive_with_timeout(
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
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "06-cleared");
    let cleared = live_text(&mut s);
    assert!(cleared.contains("find “” · 0 of 0"), "A1: marks stuck");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "find “", "find closed");
    waits::wait_state(&mut s, META, "meta restored");
    checkpoint(&mut s, &dir, "07-closed");

    support::drive_with_timeout(
        &mut s,
        &["end", &format!("wait:{TAIL}")],
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "08-tail");
    let tail = live_text(&mut s);
    assert!(tail.contains(TAIL), "N1: tail unreachable");
    support::drive_with_timeout(
        &mut s,
        &["home", &format!("wait:{TOP}")],
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "09-top");
    let top = live_text(&mut s);
    assert!(top.contains(TOP), "N1: top not restored");
    assert!(!top.contains(TAIL), "N1: tail stuck after home");

    support::press_step(&s, "y");
    std::thread::sleep(Duration::from_millis(300));
    let refused = live_text(&mut s);
    assert!(!refused.contains("Copied"), "A2: copy without selection");

    let (col, row) = first_pos(&refused, "line 0");
    s.inner
        .mouse_down(MouseButton::Left, col, row, MouseMods::NONE)
        .expect("drag press");
    for c in col + 1..=col + 6 {
        s.inner
            .mouse_drag(MouseButton::Left, c, row, MouseMods::NONE)
            .unwrap_or_else(|e| panic!("drag motion to ({c}, {row}) failed: {e:?}"));
    }
    support::drive_with_timeout(&mut s, &["y", "wait:Copied"], drive_ms(case_timeout(&case)));
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

    assert!(
        !rest.starts_with(' ') && !rest.starts_with("1 line"),
        "N2: copied text carries the gutter: `{rest}`"
    );
    assert!(copied.contains(TOP), "N2: gutter proof gone");
    eprintln!("h7 preview_find: gutter, find, marks, clear, copy");
}
