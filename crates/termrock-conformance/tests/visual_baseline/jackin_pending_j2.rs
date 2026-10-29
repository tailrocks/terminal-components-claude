use std::path::{Path, PathBuf};
use std::time::Duration;

use tuiscotti::tui::{MouseButton, Wheel};

use crate::support::Session;
use crate::support::state_waits as waits;
use crate::support::typed_input::Input;
use crate::support::{self, Case, Color, JACKIN};

const BOOT: &str = "jackin❯";
const BOOT_FAILURE: &str = "Launch failed";
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

const ACCOUNTS_PLAY: &[&str] = &["--scenario", "accounts-mixed", "--motion", "full"];
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
const CAPSULE: &[&str] = &[
    "--scenario",
    "capsule-multi",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const CAPSULE_PLAY: &[&str] = &["--scenario", "capsule-multi", "--motion", "full"];
const FIRST40: &[&str] = &[
    "--scenario",
    "first-use",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const FIRST400: &[&str] = &[
    "--scenario",
    "first-use",
    "--motion",
    "paused",
    "--frame",
    "400",
];

const PRELUDE_S1: &str = "step 1 of 5 · Source";
const COPIED: &str = "Copied to the preview clipboard";
const SPIN: &str = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏";

fn j2_case(
    slug: &str,
    args: &'static [&'static str],
    boot: &'static str,
    cols: u16,
    rows: u16,
) -> Case {
    Case::dynamic(
        format!("journeys/jackin/j2/{slug}"),
        JACKIN,
        args,
        cols,
        rows,
        Color::Truecolor,
        boot,
    )
}

fn j2_dir(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/tuiscotti/journeys")
        .join(test);
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("mkdir {}: {e}", dir.display()));
    dir
}

fn write_provenance(dir: &Path, case: &Case) {
    write_provenance_named(dir, case, "provenance.txt");
}

fn write_provenance_named(dir: &Path, case: &Case, file: &str) {
    let subject =
        support::try_resolve_bin(case.bin).unwrap_or_else(|e| panic!("resolve {}: {e}", case.bin));
    let modified = subject
        .mtime_unix
        .map_or_else(|| "unknown".to_string(), |m| m.to_string());
    let argv = support::argv_for(case);
    let body = format!(
        "profile: tuiscotti-default\ntransport: pty\nport_of: 9ae8b871266185fcd85ab9658e0b0092c38c7370\nbinary: {}\nsha256: {}\nsize_bytes: {}\nmodified_unix: {modified}\nargv: {}\n",
        subject.path.display(),
        subject.sha256,
        subject.len,
        argv.join(" "),
    );
    std::fs::write(dir.join(file), &body)
        .unwrap_or_else(|e| panic!("write {}: {e}", dir.join(file).display()));
    eprintln!("j2 provenance ({file}): {body}");
}

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

fn live_text(s: &mut Session) -> String {
    let obs = s
        .inner
        .observe_now()
        .unwrap_or_else(|e| panic!("live sample failed: {e:#}"));
    support::screen_text(&obs.screen)
}

fn live_screen(s: &mut Session) -> tuiscotti::Screen {
    s.inner
        .observe_now()
        .unwrap_or_else(|e| panic!("live screen sample failed: {e:#}"))
        .screen
}

fn find_pos(s: &mut Session, needle: &str, timeout: Duration) -> (u16, u16) {
    let obs = support::wait_screen(
        s,
        timeout,
        &format!("`{needle}` never appeared"),
        |screen| support::screen_find(screen, needle).is_some(),
    );
    support::screen_find(&obs.screen, needle).expect("wait passed with the needle on screen")
}

fn assert_line_has(text: &str, needle: &str, also: &str, what: &str) {
    assert!(
        text.lines().any(|l| l.contains(needle) && l.contains(also)),
        "{what}: no line contains both `{needle}` and `{also}`"
    );
}

fn count_lines(text: &str, needle: &str) -> usize {
    text.lines().filter(|l| l.contains(needle)).count()
}

fn display_col(line: &str, needle: &str) -> usize {
    let off = line.find(needle).expect("needle present");
    unicode_width::UnicodeWidthStr::width(&line[..off])
}

fn footer_line(text: &str) -> &str {
    text.lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
}

fn click_text(s: &mut Session, needle: &str, dcol: i16, timeout: Duration, what: &str) {
    let (row, col) = find_pos(s, needle, timeout);
    let target = col.saturating_add_signed(dcol);
    s.click(target, row)
        .unwrap_or_else(|e| panic!("{what}: click failed: {e:?}"));
    std::thread::sleep(Duration::from_millis(400));
}

fn right_click_text(s: &mut Session, needle: &str, dcol: i16, timeout: Duration, what: &str) {
    let (row, col) = find_pos(s, needle, timeout);
    let target = col.saturating_add_signed(dcol);
    s.click_with(MouseButton::Right, target, row)
        .unwrap_or_else(|e| panic!("{what}: right click failed: {e:?}"));
    std::thread::sleep(Duration::from_millis(400));
}

fn wheel_at(s: &mut Session, col: u16, row: u16, dir: Wheel, notches: u32) {
    for _ in 0..notches {
        Input::wheel(dir, col, row).send(s);
        std::thread::sleep(Duration::from_millis(120));
    }
}

fn wait_absent(s: &mut Session, needle: &str, timeout: Duration, what: &str) {
    support::wait_screen(
        s,
        timeout,
        &format!("{what}: `{needle}` never cleared"),
        |screen| !support::screen_text(screen).contains(needle),
    );
}

fn props_goto(s: &mut Session, label: &str, what: &str) {
    for _ in 0..12 {
        let text = live_text(s);
        if text.lines().any(|l| l.contains("▎") && l.contains(label)) {
            return;
        }
        support::press_step(s, "down");
        std::thread::sleep(Duration::from_millis(150));
    }
    panic!("{what}: gutter never reached `{label}`");
}

#[test]
#[ignore = "jackin j2 check; run with --ignored"]
fn j2_input_required_error() {
    let dir = j2_dir("j2_input_required_error");
    let case = j2_case("input_required_error", ACCOUNTS, BOOT, 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["a", "wait:New account"],
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "01-form-shown");

    assert!(
        !live_text(&mut s).contains("Required"),
        "fresh form must not show Required"
    );

    support::drive_with_timeout(
        &mut s,
        &["enter", "enter", "wait:Required"],
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "02-required-error");
    let err = live_text(&mut s);

    let rows: Vec<&str> = err.lines().collect();
    let ei = rows
        .iter()
        .position(|l| l.contains("Required"))
        .expect("V1: Required row");
    assert!(
        ei >= 2 && rows[ei - 2].contains("Display name"),
        "V1: Required sits under the Display name field"
    );
    assert!(
        rows[ei - 1].contains("!"),
        "V1: `!` marks the field row: {:?}",
        rows[ei - 1]
    );

    assert_eq!(count_lines(&err, "Required"), 1, "S2: one error row");

    support::drive_with_timeout(&mut s, &["enter", "type:x"], drive_ms(case_timeout(&case)));
    wait_absent(
        &mut s,
        "Required",
        case_timeout(&case),
        "error clears on typing",
    );

    support::press_step(&s, "enter");
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        !live_text(&mut s).contains("Required"),
        "A1: non-empty commit stays clean"
    );

    support::drive_with_timeout(
        &mut s,
        &["enter", "ctrl-e", "ctrl-u"],
        drive_ms(case_timeout(&case)),
    );
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        !live_text(&mut s).contains("Required"),
        "N2: live_validate paints no fresh error"
    );

    support::drive_with_timeout(
        &mut s,
        &["enter", "wait:Required"],
        drive_ms(case_timeout(&case)),
    );

    support::drive_with_timeout(
        &mut s,
        &["enter", "type:y", "wait:Display name"],
        drive_ms(case_timeout(&case)),
    );
    wait_absent(&mut s, "Required", case_timeout(&case), "second clear");
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, "Required", "cancel revalidates");
    waits::wait_state(&mut s, "New account", "form still open after Esc");
    checkpoint(&mut s, &dir, "03-cancel-reverted");

    support::drive_with_timeout(
        &mut s,
        &["tab", "enter", "enter"],
        drive_ms(case_timeout(&case)),
    );
    std::thread::sleep(Duration::from_millis(300));
    let back = live_text(&mut s);
    assert_eq!(
        count_lines(&back, "Required"),
        1,
        "N1: optional empty commit adds no error"
    );
    let brows: Vec<&str> = back.lines().collect();
    let pi = brows
        .iter()
        .position(|l| l.contains("Purpose label"))
        .expect("N1: purpose field still rendered");
    assert!(
        !brows[pi..(pi + 3).min(brows.len())]
            .iter()
            .any(|l| l.contains("Required")),
        "N1: no error under the optional field"
    );
    checkpoint(&mut s, &dir, "04-optional-clean");
    eprintln!("j2 input_required_error: error, live clear, commit+cancel validate, optional clean");
}

#[test]
#[ignore = "jackin j2 check; run with --ignored"]
fn j2_cockpit_rail() {
    let dir = j2_dir("j2_cockpit_rail");
    let case = j2_case("cockpit_rail", LAUNCH, BOOT, 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    waits::wait_state(&mut s, "stage 3 of 11", "rail header");
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);

    assert_line_has(&boot, "✓ 01 Identity", "1.4 s", "V1 done row");
    assert_line_has(&boot, "✓ 02 Role", "1.8 s", "V1 done row");
    let run: Vec<&str> = boot
        .lines()
        .filter(|l| l.contains("03 Credentials"))
        .collect();
    assert_eq!(run.len(), 1, "V1: one running row");
    assert!(
        run[0].chars().any(|c| SPIN.contains(c)),
        "V1: running row shows a spinner: {:?}",
        run[0]
    );
    assert!(boot.contains("queued"), "V1: later stages queued");

    assert!(
        boot.contains("stage 3 of 11 · Credentials · 2 done · 0 skipped"),
        "V2: frontier header"
    );

    for line in boot.lines().filter(|l| {
        l.contains("Identity")
            || l.contains("Credentials")
            || l.contains("Construct")
            || l.contains("Role")
    }) {
        assert!(!line.contains("▎"), "N2: no cursor on rail: {line:?}");
    }

    for key in ["down", "up", "left", "right"] {
        support::press_step(&s, key);
        std::thread::sleep(Duration::from_millis(250));
        assert_eq!(live_text(&mut s), boot, "A1: `{key}` at rail is refused");
    }

    for _ in 0..4 {
        support::press_step(&s, "tab");
        std::thread::sleep(Duration::from_millis(250));
        assert_eq!(live_text(&mut s), boot, "N1: Tab never reaches the rail");
    }
    checkpoint(&mut s, &dir, "01-cockpit-rail");

    let (row, col) = find_pos(&mut s, "03 Credentials", case_timeout(&case));
    s.click(col, row).expect("rail click");
    std::thread::sleep(Duration::from_millis(400));
    assert_eq!(live_text(&mut s), boot, "A1: rail click changes nothing");

    let rail_before: Vec<String> = boot
        .lines()
        .filter(|l| {
            l.contains("Identity")
                || l.contains("Role")
                || l.contains("Credentials")
                || l.contains("Construct")
                || l.contains("queued")
        })
        .map(str::to_string)
        .collect();
    support::drive_with_timeout(
        &mut s,
        &["d", "wait:run-2026"],
        drive_ms(case_timeout(&case)),
    );
    let debugged = live_text(&mut s);
    let rail_after: Vec<String> = debugged
        .lines()
        .filter(|l| {
            l.contains("Identity")
                || l.contains("Role")
                || l.contains("Credentials")
                || l.contains("Construct")
                || l.contains("queued")
        })
        .map(str::to_string)
        .collect();
    assert_eq!(rail_before, rail_after, "A2: rail untouched by debug");
    checkpoint(&mut s, &dir, "02-debug");

    let fcase = j2_case("cockpit_rail_failed", FAILURE, BOOT_FAILURE, 80, 24);
    write_provenance_named(&dir, &fcase, "provenance-failed.txt");
    let mut f = support::spawn_boot(&fcase);
    waits::wait_state(
        &mut f,
        "The Construct network could not be attached",
        "failure",
    );
    waits::wait_state(
        &mut f,
        "Check that Docker's jackin-net bridge exists",
        "next step",
    );
    checkpoint(&mut f, &dir, "03-cockpit-failed");
    eprintln!("j2 cockpit_rail: rail, refused keys/clicks/tabs, debug, failed");
}

#[test]
#[ignore = "jackin j2 check; run with --ignored"]
fn j2_cockpit_info_copy() {
    let dir = j2_dir("j2_cockpit_info_copy");
    let case = j2_case("cockpit_info_copy", LAUNCH, BOOT, 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["i", "wait:Debug info"],
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "01-info-dialog");
    let info = live_text(&mut s);

    for (label, value) in [
        ("Target", "the-architect"),
        ("Role", "the-architect"),
        ("Agent", "Claude Code"),
        ("Run id", "run-2026"),
    ] {
        assert_line_has(&info, label, value, "V1 row");
    }
    let mut cols = vec![];
    for (label, value) in [
        ("Target", "the-architect"),
        ("Role", "the-architect"),
        ("Agent", "Claude Code"),
        ("Run id", "run-2026"),
    ] {
        let line = info
            .lines()
            .find(|l| l.contains(label) && l.contains(value))
            .unwrap_or_else(|| panic!("V1: no `{label}` row"));
        cols.push(display_col(line, label));
    }
    assert!(
        cols.windows(2).all(|w| w[0] == w[1]),
        "V1: labels share one column: {cols:?}"
    );
    assert!(info.contains("run-2026"), "run id value present");

    props_goto(&mut s, "Target", "S1 target row");

    support::press_step(&s, "y");
    std::thread::sleep(Duration::from_millis(300));
    let plain = live_text(&mut s);
    assert!(!plain.contains(COPIED), "N1: plain `y` copies nothing");
    assert!(plain.contains("Debug info"), "N1: dialog still open");

    support::press_step(&s, "enter");
    std::thread::sleep(Duration::from_millis(300));
    let plain_enter = live_text(&mut s);
    assert!(
        !plain_enter.contains(COPIED),
        "A2: plain Enter copies nothing"
    );
    assert!(plain_enter.contains("Debug info"), "A2: dialog still open");

    props_goto(&mut s, "Run id", "V2 run id row");
    let on_run = live_text(&mut s);
    assert_line_has(&on_run, "Run id", "y copy", "V2 copy hint");

    support::drive_with_timeout(
        &mut s,
        &["y", &format!("wait:{COPIED}")],
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "02-run-copied");
    let copied = live_text(&mut s);
    assert!(
        copied.contains("Debug info"),
        "A1: copying keeps dialog open"
    );
    assert_line_has(&copied, "▎", "Run id", "S1 cursor unmoved");

    click_text(&mut s, "Target", 0, case_timeout(&case), "A2 click row");
    let clicked = live_text(&mut s);
    assert_line_has(&clicked, "▎", "Target", "A1 click moves cursor");
    assert!(
        clicked.contains("Debug info"),
        "A2: plain click keeps dialog"
    );

    support::press_step(&s, "tab");
    std::thread::sleep(Duration::from_millis(250));
    support::press_step(&s, "enter");
    wait_absent(
        &mut s,
        "Debug info",
        case_timeout(&case),
        "Close path shuts dialog",
    );
    waits::wait_state(&mut s, "stage 3 of 11", "cockpit back");
    checkpoint(&mut s, &dir, "03-closed");
    eprintln!("j2 cockpit_info_copy: plain refused, run id copied, click, close");
}

#[test]
#[ignore = "jackin j2 check; run with --ignored"]
fn j2_capsule_about_scroll() {
    let dir = j2_dir("j2_capsule_about_scroll");
    let case = j2_case("capsule_about_scroll", CAPSULE, BOOT, 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    click_text(&mut s, BOOT, 2, case_timeout(&case), "brand menu");
    waits::wait_state(&mut s, "About jackin-preview", "brand menu open");
    support::drive_with_timeout(
        &mut s,
        &["enter", "wait:About"],
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "01-about-dialog");
    let about = live_text(&mut s);

    let mut label_cols = vec![];
    let mut value_cols = vec![];
    for (label, value) in [
        ("Product", "jackin-preview"),
        ("Scenario", "capsule-multi"),
        ("Construct", "simulated"),
        ("Design system", "Junie-inspired"),
    ] {
        let line = about
            .lines()
            .find(|l| l.contains(label) && l.contains(value))
            .unwrap_or_else(|| panic!("V1: no `{label}` row"));
        label_cols.push(display_col(line, label));
        value_cols.push(display_col(line, value));
    }
    assert!(
        label_cols.windows(2).all(|w| w[0] == w[1]),
        "V1: labels share one column: {label_cols:?}"
    );
    assert!(
        value_cols.windows(2).all(|w| w[0] == w[1]),
        "V1: values share one column: {value_cols:?}"
    );
    assert_eq!(
        value_cols[0] - label_cols[0],
        15,
        "V1: longest label (13) plus two cells"
    );
    assert_line_has(&about, "▎", "Product", "list focus starts row 0");

    click_text(&mut s, "Scenario", 0, case_timeout(&case), "A1 click row");
    let clicked = live_text(&mut s);
    assert_line_has(&clicked, "▎", "Scenario", "A1 click moves cursor");
    assert!(clicked.contains("About"), "A1: plain Activate keeps dialog");

    let (wrow, wcol) = find_pos(&mut s, "Scenario", case_timeout(&case));
    wheel_at(&mut s, wcol, wrow, Wheel::Down, 3);
    assert_eq!(live_text(&mut s), clicked, "N1: wheel down at fit is inert");
    wheel_at(&mut s, wcol, wrow, Wheel::Up, 3);
    assert_eq!(live_text(&mut s), clicked, "N1: wheel up at fit is inert");

    support::drive_with_timeout(
        &mut s,
        &["down", "down", "down", "down", "down"],
        drive_ms(case_timeout(&case)),
    );
    assert_line_has(&live_text(&mut s), "▎", "Design system", "S1 bottom clamp");
    support::drive_with_timeout(
        &mut s,
        &["up", "up", "up", "up", "up", "up"],
        drive_ms(case_timeout(&case)),
    );
    assert_line_has(&live_text(&mut s), "▎", "Product", "S1 top clamp");

    for label in ["Product", "Scenario", "Construct", "Design system"] {
        click_text(&mut s, label, 0, case_timeout(&case), "N2 row click");
        assert_line_has(
            &live_text(&mut s),
            "▎",
            label,
            &format!("N2: click resolves `{label}`"),
        );
    }

    s.click(10, 13).expect("padding click");
    std::thread::sleep(Duration::from_millis(400));
    let padded = live_text(&mut s);
    assert_line_has(
        &padded,
        "▎",
        "Design system",
        "N2: padding click moves nothing",
    );
    assert!(padded.contains("About"), "N2: dialog still open");
    click_text(&mut s, "Close", 0, case_timeout(&case), "About close click");
    wait_absent(&mut s, "Design system", case_timeout(&case), "About closed");

    let bcase = j2_case("capsule_about_scroll_detail", FAILURE, BOOT_FAILURE, 80, 24);
    write_provenance_named(&dir, &bcase, "provenance-detail.txt");
    let mut b = support::spawn_boot(&bcase);
    waits::wait_state(&mut b, "network: attach", "detail head");
    checkpoint(&mut b, &dir, "02-detail-overflow");

    let (sbrow0, sbcol0) = find_pos(&mut b, "┃", case_timeout(&bcase));
    assert_eq!(sbcol0, 73, "V2: thumb on the detail track");
    assert!(
        (10..20).contains(&sbrow0),
        "V2: thumb inside the detail rows"
    );

    b.click(73, 18).expect("detail press");
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        !live_text(&mut b).contains("network: attach"),
        "A2: scrollbar press jumps the detail"
    );
    wheel_at(&mut b, 40, 14, Wheel::Up, 6);
    std::thread::sleep(Duration::from_millis(300));
    waits::wait_state(&mut b, "network: attach", "detail head back");
    let (drow, dcol) = find_pos(&mut b, "Stage", case_timeout(&bcase));
    wheel_at(&mut b, dcol, drow, Wheel::Down, 3);
    std::thread::sleep(Duration::from_millis(300));
    let dwheeled = live_text(&mut b);
    assert!(
        !dwheeled.contains("network: attach"),
        "S2: wheel scrolled the detail"
    );
    assert_line_has(&dwheeled, "▎", "Stage", "S2: props cursor unmoved");
    wheel_at(&mut b, dcol, drow, Wheel::Up, 6);
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        live_text(&mut b).contains("network: attach"),
        "wheel back restores the detail head"
    );

    support::drag_path(&mut b, (73, 10), (73, 14));
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        !live_text(&mut b).contains("network: attach"),
        "A2: scrollbar drag tracks"
    );
    checkpoint(&mut b, &dir, "04-detail-scroll");
    eprintln!("j2 capsule_about_scroll: label math, click, clamps, overflow scroll");
}

#[test]
#[ignore = "jackin j2 check; run with --ignored"]
fn j2_capsule_palette() {
    let dir = j2_dir("j2_capsule_palette");
    let case = j2_case("capsule_palette", CAPSULE, BOOT, 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["ctrl-b", "space", "wait:Command palette"],
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "01-palette-open");
    let open = live_text(&mut s);

    assert_line_has(&open, "Command palette", "20 of 20", "V1 title+scope");
    assert_line_has(&open, "New tab", "Ctrl+B c", "V1 detail");
    assert!(
        open.contains("Type to filter commands…"),
        "V1: filter placeholder"
    );

    assert!(
        open.contains("no file under cursor"),
        "V2: disabled reason visible"
    );
    support::drive_with_timeout(
        &mut s,
        &["down", "down", "down", "down", "down", "down"],
        drive_ms(case_timeout(&case)),
    );
    let scrolled = live_text(&mut s);
    assert!(
        scrolled.contains("no selection"),
        "V2: disabled selection reason after scroll"
    );

    support::drive_with_timeout(
        &mut s,
        &["type:zoom", "wait:1 of 20"],
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "02-palette-filtered");
    let filtered = live_text(&mut s);
    assert!(
        filtered.contains("Zoom / unzoom pane"),
        "S1: match survives"
    );
    assert!(!filtered.contains("Clear pane"), "S1: non-match filtered");
    support::drive_with_timeout(
        &mut s,
        &["escape", "wait:20 of 20"],
        drive_ms(case_timeout(&case)),
    );

    support::drive_with_timeout(
        &mut s,
        &["type:link", "wait:1 of 20"],
        drive_ms(case_timeout(&case)),
    );
    assert!(
        live_text(&mut s).contains("Open link under cursor"),
        "N1: disabled row isolated"
    );
    support::press_step(&s, "enter");
    std::thread::sleep(Duration::from_millis(300));
    let refused = live_text(&mut s);
    assert!(
        refused.contains("Command palette") && refused.contains("1 of 20"),
        "N1: Enter on disabled dispatches nothing"
    );
    support::drive_with_timeout(
        &mut s,
        &["escape", "wait:20 of 20"],
        drive_ms(case_timeout(&case)),
    );

    support::drive_with_timeout(
        &mut s,
        &["type:usage", "wait:1 of 20"],
        drive_ms(case_timeout(&case)),
    );
    support::drive_with_timeout(
        &mut s,
        &["enter", "wait:read-only projection"],
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "03-palette-ran");
    assert!(
        live_text(&mut s).contains("Usage"),
        "A1: Usage dispatched from the palette"
    );
    support::press_step(&s, "escape");
    wait_absent(
        &mut s,
        "read-only projection",
        case_timeout(&case),
        "usage closed",
    );

    support::drive_with_timeout(
        &mut s,
        &["ctrl-b", "space", "wait:Command palette"],
        drive_ms(case_timeout(&case)),
    );
    support::drive_with_timeout(
        &mut s,
        &["type:zzzqqq", "wait:No matching commands"],
        drive_ms(case_timeout(&case)),
    );
    support::press_step(&s, "enter");
    std::thread::sleep(Duration::from_millis(300));
    let empty = live_text(&mut s);
    assert!(
        empty.contains("Command palette") && empty.contains("No matching commands"),
        "N2: Enter on empty picks nothing"
    );
    support::drive_with_timeout(&mut s, &["escape", "escape"], drive_ms(case_timeout(&case)));
    wait_absent(
        &mut s,
        "Command palette",
        case_timeout(&case),
        "palette closed",
    );

    support::drive_with_timeout(
        &mut s,
        &["ctrl-b", ":", "wait:Command palette"],
        drive_ms(case_timeout(&case)),
    );
    support::drive_with_timeout(&mut s, &["escape"], drive_ms(case_timeout(&case)));
    wait_absent(
        &mut s,
        "Command palette",
        case_timeout(&case),
        "colon palette closed",
    );

    support::drive_with_timeout(
        &mut s,
        &["f10", "wait:New tab"],
        drive_ms(case_timeout(&case)),
    );
    support::drive_with_timeout(
        &mut s,
        &["right", "right", "right", "right", "wait:Key reference"],
        drive_ms(case_timeout(&case)),
    );
    support::drive_with_timeout(
        &mut s,
        &["down", "enter", "wait:Command palette"],
        drive_ms(case_timeout(&case)),
    );
    support::drive_with_timeout(&mut s, &["escape"], drive_ms(case_timeout(&case)));
    wait_absent(
        &mut s,
        "Command palette",
        case_timeout(&case),
        "menu palette closed",
    );

    support::drive_with_timeout(
        &mut s,
        &["ctrl-b", "space", "wait:Command palette"],
        drive_ms(case_timeout(&case)),
    );
    for _ in 0..16 {
        support::press_step(&s, "down");
        std::thread::sleep(Duration::from_millis(80));
    }
    let mix = live_text(&mut s);
    assert_line_has(&mix, "Close", "Ctrl+B x", "S2 multi-pane Close");
    assert!(
        !mix.lines().any(|l| l.contains("Close tab")),
        "S2: no Close tab on the multi-pane tab"
    );
    support::drive_with_timeout(&mut s, &["escape"], drive_ms(case_timeout(&case)));
    wait_absent(
        &mut s,
        "Command palette",
        case_timeout(&case),
        "palette closed",
    );
    support::drive_with_timeout(
        &mut s,
        &["ctrl-b", "2", "sleep:400"],
        drive_ms(case_timeout(&case)),
    );
    support::drive_with_timeout(
        &mut s,
        &["ctrl-b", "space", "wait:Command palette"],
        drive_ms(case_timeout(&case)),
    );
    for _ in 0..16 {
        support::press_step(&s, "down");
        std::thread::sleep(Duration::from_millis(80));
    }
    assert!(
        live_text(&mut s).contains("Close tab"),
        "S2: Close tab on the single-leaf tab"
    );
    support::drive_with_timeout(&mut s, &["escape"], drive_ms(case_timeout(&case)));
    wait_absent(
        &mut s,
        "Command palette",
        case_timeout(&case),
        "palette closed",
    );
    eprintln!("j2 capsule_palette: filter, disabled refused, dispatch, empty, openers, scope");
}

#[test]
#[ignore = "jackin j2 check; run with --ignored"]
fn j2_settings_op_chain() {
    let dir = j2_dir("j2_settings_op_chain");
    let case = j2_case("settings_op_chain", ACCOUNTS_PLAY, BOOT, 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["s", "wait:Settings › global"],
        drive_ms(case_timeout(&case)),
    );
    support::drive_with_timeout(
        &mut s,
        &["3", "enter", "wait:GH_TOKEN"],
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "01-env");

    support::press_step(&s, "p");
    waits::wait_state(&mut s, "Choose 1Password account", "chain opens");

    let mut saw_loading = false;
    for _ in 0..30 {
        let t = live_text(&mut s);
        if t.contains("loading accounts…") {
            saw_loading = true;
            support::press_step(&s, "down");
            std::thread::sleep(Duration::from_millis(60));
            let t2 = live_text(&mut s);
            assert!(
                t2.contains("loading accounts…") || t2.contains("chainargos"),
                "N1: mid-load key changes nothing observable"
            );
            break;
        }
        if t.contains("chainargos.1password.com") {
            break;
        }
        std::thread::sleep(Duration::from_millis(60));
    }
    waits::wait_state(&mut s, "chainargos.1password.com", "accounts loaded");
    waits::wait_state(&mut s, "my.1password.com", "three accounts listed");
    checkpoint(&mut s, &dir, "02-chain-open");
    eprintln!("op chain loading observed: {saw_loading}");

    support::drive_with_timeout(
        &mut s,
        &["enter", "wait:Choose vault"],
        drive_ms(case_timeout(&case)),
    );
    waits::wait_state(&mut s, "1Password › chainargos", "vault crumb");
    waits::wait_state(&mut s, "Engineering", "vault rows");
    assert!(
        live_text(&mut s).contains("Search vaults…"),
        "S1: query cleared on advance"
    );
    checkpoint(&mut s, &dir, "03-chain-mid");
    support::drive_with_timeout(
        &mut s,
        &["enter", "wait:Choose item"],
        drive_ms(case_timeout(&case)),
    );
    waits::wait_state(&mut s, "OpenAI · Codex Primary", "item rows");
    support::drive_with_timeout(
        &mut s,
        &["enter", "wait:Choose field"],
        drive_ms(case_timeout(&case)),
    );
    waits::wait_state(&mut s, "credential", "field rows");

    support::drive_with_timeout(
        &mut s,
        &["enter", "wait:now resolves from"],
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "04-chain-stored");
    let stored = live_text(&mut s);
    assert!(
        stored.contains("GH_TOKEN now resolves from Engineering ›"),
        "A1: reference stored as metadata"
    );
    assert!(
        !stored.contains("Choose field"),
        "chain closed after storing"
    );

    assert_line_has(&stored, "GH_TOKEN", "[op]", "N2 metadata tag");
    assert!(
        !stored.contains("openai:valid") && !stored.contains("k7Qz"),
        "N2: no secret value on screen"
    );

    support::press_step(&s, "p");
    waits::wait_state(&mut s, "Choose 1Password account", "chain reopened");
    waits::wait_state(&mut s, "chainargos.1password.com", "accounts reloaded");
    support::drive_with_timeout(
        &mut s,
        &["enter", "wait:Choose vault"],
        drive_ms(case_timeout(&case)),
    );

    waits::wait_state(&mut s, "▪ Engineering", "vault rows again");
    checkpoint(&mut s, &dir, "04b-vault-again");
    support::press_step(&s, "backspace");
    std::thread::sleep(Duration::from_millis(500));
    checkpoint(&mut s, &dir, "04c-after-back");
    waits::wait_state(&mut s, "Choose 1Password account", "Back rewinds");
    support::press_step(&s, "escape");
    wait_absent(
        &mut s,
        "Choose 1Password account",
        case_timeout(&case),
        "Esc cancels chain",
    );
    assert!(
        live_text(&mut s).contains("GH_TOKEN"),
        "S2: env screen back, draft intact"
    );

    support::press_step(&s, "p");
    waits::wait_state(&mut s, "Choose 1Password account", "chain reopened");
    waits::wait_state(&mut s, "my.1password.com", "locked row listed");
    support::drive_with_timeout(&mut s, &["down", "enter"], drive_ms(case_timeout(&case)));
    waits::wait_state(&mut s, "1Password is locked", "V2 error message");
    let locked = live_text(&mut s);
    assert!(locked.contains("!"), "V2: error marker");
    assert!(locked.contains("press r to retry"), "V2: retry detail");
    support::press_step(&s, "r");
    std::thread::sleep(Duration::from_millis(500));
    waits::wait_state(&mut s, "1Password is locked", "r retries into error");

    support::press_step(&s, "backspace");
    waits::wait_state(&mut s, "Choose 1Password account", "back at accounts");
    waits::wait_state(&mut s, "chainargos.1password.com", "accounts reloaded");
    support::drive_with_timeout(
        &mut s,
        &["enter", "wait:Choose vault"],
        drive_ms(case_timeout(&case)),
    );
    waits::wait_state(&mut s, "▪ Engineering", "vault rows");

    support::press_step(&s, "down");
    std::thread::sleep(Duration::from_millis(200));
    click_text(
        &mut s,
        "Engineering",
        1,
        case_timeout(&case),
        "N2 vault click",
    );
    waits::wait_state(&mut s, "Choose item", "click chooses the row");
    checkpoint(&mut s, &dir, "05-click-chooses");
    eprintln!("j2 settings_op_chain: full chain, rewind, cancel, locked+retry, click");
}

#[test]
#[ignore = "jackin j2 check; run with --ignored"]
fn j2_dialog_cancel_quit() {
    let dir = j2_dir("j2_dialog_cancel_quit");
    let case = j2_case("dialog_cancel_quit", LAUNCH, BOOT, 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);

    support::drive_with_timeout(
        &mut s,
        &["c", "wait:Cancel the launch?"],
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "01-cancel-open");
    let cancel = live_text(&mut s);

    assert_line_has(&cancel, "▎Cancel", "Cancel launch", "V1 action row");
    assert!(
        cancel.contains("partially prepared instance is marked failed"),
        "V1: destructive body"
    );

    support::drive_with_timeout(&mut s, &["y"], drive_ms(case_timeout(&case)));
    wait_absent(
        &mut s,
        "Cancel the launch?",
        case_timeout(&case),
        "cancel confirmed",
    );
    waits::wait_state(&mut s, "Current directory", "manager back");
    checkpoint(&mut s, &dir, "02-cancel-confirmed");
    let ack = live_text(&mut s);
    assert!(
        ack.contains("still running in the Construct") || ack.contains("Launch cancelled"),
        "A1: ack status after confirm"
    );

    support::press_step(&s, "c");
    std::thread::sleep(Duration::from_millis(400));
    let after = live_text(&mut s);
    assert!(
        !after.contains("Cancel the launch?"),
        "N1: no dialog once terminal"
    );
    assert!(
        after.contains("Accounts"),
        "S1: the key reaches the new route instead"
    );

    let bcase = j2_case("dialog_cancel_quit_dismiss", LAUNCH, BOOT, 80, 24);
    write_provenance_named(&dir, &bcase, "provenance-dismiss.txt");
    let mut b = support::spawn_boot(&bcase);
    support::drive_with_timeout(
        &mut b,
        &["c", "wait:Cancel the launch?"],
        drive_ms(case_timeout(&bcase)),
    );
    support::press_step(&b, "n");
    wait_absent(
        &mut b,
        "Cancel the launch?",
        case_timeout(&bcase),
        "`n` dismisses",
    );
    let n_dismiss = live_text(&mut b);
    assert!(
        n_dismiss.contains("stage 3 of 11"),
        "A1: `n` keeps the launch running"
    );
    assert!(
        !n_dismiss.contains("Launch cancelled") && !n_dismiss.contains("Current directory"),
        "N2: `n` never confirms"
    );
    support::drive_with_timeout(
        &mut b,
        &["c", "wait:Cancel the launch?"],
        drive_ms(case_timeout(&bcase)),
    );
    support::press_step(&b, "escape");
    wait_absent(
        &mut b,
        "Cancel the launch?",
        case_timeout(&bcase),
        "Esc dismisses",
    );
    let esc_dismiss = live_text(&mut b);
    assert!(
        esc_dismiss.contains("stage 3 of 11"),
        "N2: Esc keeps the launch running"
    );
    assert!(
        !esc_dismiss.contains("Launch cancelled"),
        "N2: Esc never confirms"
    );

    let ccase = j2_case("dialog_cancel_quit_running", RETURNING, BOOT, 80, 24);
    write_provenance_named(&dir, &ccase, "provenance-quit.txt");
    let mut c = support::spawn_boot(&ccase);
    support::drive_with_timeout(
        &mut c,
        &["ctrl-q", "wait:Exit jackin❯?"],
        drive_ms(case_timeout(&ccase)),
    );
    checkpoint(&mut c, &dir, "03-quit-open");
    let quit = live_text(&mut c);
    assert!(
        quit.contains("2 instances keep running"),
        "V2: quit names the running count"
    );
    assert!(
        quit.contains("reconnect from a new") && quit.contains("terminal."),
        "V2: quit names the reconnect path"
    );
    assert_line_has(&quit, "Cancel", "▎Quit", "A2 pushed quit dialog");
    support::press_step(&c, "escape");
    wait_absent(
        &mut c,
        "Exit jackin❯?",
        case_timeout(&ccase),
        "quit cancelled",
    );
    waits::wait_state(&mut c, "Current directory", "manager back");

    let dcase = j2_case("dialog_cancel_quit_zero", FIRST400, BOOT, 80, 24);
    write_provenance_named(&dir, &dcase, "provenance-quit-zero.txt");
    let mut d = support::spawn_boot(&dcase);
    support::drive_with_timeout(
        &mut d,
        &["ctrl-q", "wait:Exit jackin❯?"],
        drive_ms(case_timeout(&dcase)),
    );
    checkpoint(&mut d, &dir, "04-quit-zero-open");
    let zero = live_text(&mut d);
    assert!(
        zero.contains("No instances are running"),
        "S2: zero-instance body"
    );
    assert!(
        zero.contains("pending Construct") && zero.contains("entry is released"),
        "S2: zero-instance release line"
    );
    support::press_step(&d, "escape");
    wait_absent(
        &mut d,
        "Exit jackin❯?",
        case_timeout(&dcase),
        "zero quit cancelled",
    );
    eprintln!("j2 dialog_cancel_quit: confirm, dismissals, terminal refusal, quits");
}

#[test]
#[ignore = "jackin j2 check; run with --ignored"]
fn j2_capsule_tab_menu() {
    let dir = j2_dir("j2_capsule_tab_menu");
    let case = j2_case("capsule_tab_menu", CAPSULE, BOOT, 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    let (strip_row, _) = find_pos(&mut s, "Mix (3)", case_timeout(&case));
    right_click_text(&mut s, "Mix (3)", 1, case_timeout(&case), "S1 tab menu");
    waits::wait_state(&mut s, "Change title…", "tab menu open");
    checkpoint(&mut s, &dir, "01-tabmenu-open");
    let menu = live_text(&mut s);

    assert_eq!(
        count_lines(&menu, "Mix (3)"),
        2,
        "V1: strip cell plus menu title"
    );
    let menu_row = menu
        .lines()
        .position(|l| l.contains("Change title…"))
        .expect("V1: menu row");
    assert!(
        menu_row as u16 > strip_row,
        "V1: menu anchors under the strip"
    );

    assert_line_has(&menu, "Change title…", "2×click", "V2 shortcuts");
    assert_line_has(&menu, "Split right", "Ctrl+B %", "V2 shortcuts");
    assert_line_has(&menu, "Close tab", "Ctrl+B &", "V2 shortcuts");
    let mrows: Vec<&str> = menu.lines().collect();
    let split_i = mrows
        .iter()
        .position(|l| l.contains("Split right"))
        .expect("V2: split row");
    let close_i = mrows
        .iter()
        .position(|l| l.contains("Close tab"))
        .expect("V2: close row");
    assert!(
        mrows[split_i..close_i].iter().any(|l| l.contains("─")),
        "V2: separator rule above Close tab"
    );

    support::drive_with_timeout(
        &mut s,
        &["enter", "wait:Change tab title"],
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "02-tabmenu-chosen");
    support::press_step(&s, "escape");
    wait_absent(
        &mut s,
        "Change tab title",
        case_timeout(&case),
        "rename cancelled",
    );
    assert!(
        !live_text(&mut s).contains("Change title…"),
        "menu consumed by the choice"
    );

    support::drive_with_timeout(
        &mut s,
        &["ctrl-b", "m", "wait:Change title…"],
        drive_ms(case_timeout(&case)),
    );
    support::press_step(&s, "escape");
    wait_absent(
        &mut s,
        "Change title…",
        case_timeout(&case),
        "Esc clears menu",
    );
    assert!(
        !live_text(&mut s).contains("Change tab title"),
        "A2: Esc acts on nothing"
    );

    support::drive_with_timeout(
        &mut s,
        &["ctrl-b", "m", "wait:Change title…"],
        drive_ms(case_timeout(&case)),
    );
    click_text(
        &mut s,
        "payments-platform ❯",
        0,
        case_timeout(&case),
        "A2 outside click",
    );
    wait_absent(
        &mut s,
        "Change title…",
        case_timeout(&case),
        "outside click clears",
    );
    assert!(
        !live_text(&mut s).contains("Change tab title"),
        "A2: outside click acts on nothing"
    );

    let (prow, pcol) = find_pos(&mut s, "payments-platform ❯", case_timeout(&case));
    s.click_with(MouseButton::Right, pcol, prow)
        .expect("pane right click");
    std::thread::sleep(Duration::from_millis(400));
    waits::wait_state(&mut s, "Copy selection", "pane menu open");
    let pane_menu = live_text(&mut s);
    assert_eq!(
        count_lines(&pane_menu, "Mix (3)"),
        1,
        "S1: pane menu carries no tab title"
    );
    assert!(
        pane_menu.contains("Paste clipboard") && pane_menu.contains("Close pane"),
        "S1: Edit menu rows"
    );
    support::press_step(&s, "escape");
    wait_absent(
        &mut s,
        "Copy selection",
        case_timeout(&case),
        "pane menu cleared",
    );

    click_text(&mut s, BOOT, 2, case_timeout(&case), "S2 brand menu");
    waits::wait_state(&mut s, "About jackin-preview", "brand menu open");
    checkpoint(&mut s, &dir, "03-brandmenu-open");
    let brand = live_text(&mut s);
    assert!(brand.contains("Usage"), "S2: brand menu usage row");
    assert!(brand.contains("Workspace manager"), "S2: brand menu row");
    support::press_step(&s, "escape");
    wait_absent(
        &mut s,
        "About jackin-preview",
        case_timeout(&case),
        "brand menu cleared",
    );

    let (srow2, scol2) = find_pos(&mut s, "docs", case_timeout(&case));
    s.click_with(MouseButton::Right, scol2 + 18, srow2)
        .expect("empty strip click");
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        !live_text(&mut s).contains("Change title…"),
        "N1: empty strip opens nothing"
    );

    right_click_text(&mut s, "Shell", 1, case_timeout(&case), "N2 shell menu");
    waits::wait_state(&mut s, "Change title…", "shell menu open");
    assert_eq!(
        count_lines(&live_text(&mut s), "Shell"),
        2,
        "N2: shell strip cell plus menu title"
    );
    support::press_step(&s, "escape");
    wait_absent(
        &mut s,
        "Change title…",
        case_timeout(&case),
        "shell menu cleared",
    );
    eprintln!("j2 capsule_tab_menu: titled menus, rename, dismissals, brand, empty");
}

#[test]
#[ignore = "jackin j2 check; run with --ignored"]
fn j2_menubar_capsule_host() {
    let dir = j2_dir("j2_menubar_capsule_host");
    let case = j2_case("menubar_capsule_host", CAPSULE, BOOT, 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);

    let strip = boot.lines().next().unwrap_or("").to_string();
    for label in ["File", "Edit", "View", "Session", "Help"] {
        assert!(strip.contains(label), "V1: strip shows `{label}`");
    }
    assert!(strip.contains("jackin❯"), "V1: brand lockup on the strip");
    checkpoint(&mut s, &dir, "01-capsule-bar-open");

    support::drive_with_timeout(
        &mut s,
        &["f10", "wait:New tab"],
        drive_ms(case_timeout(&case)),
    );
    support::drive_with_timeout(
        &mut s,
        &["right", "wait:Copy selection"],
        drive_ms(case_timeout(&case)),
    );
    assert!(
        !live_text(&mut s).contains("New tab"),
        "S1: Right swaps File → Edit"
    );
    support::drive_with_timeout(
        &mut s,
        &["left", "wait:New tab"],
        drive_ms(case_timeout(&case)),
    );
    assert!(
        !live_text(&mut s).contains("Copy selection"),
        "S1: Left swaps Edit → File"
    );
    support::press_step(&s, "escape");
    wait_absent(&mut s, "New tab", case_timeout(&case), "menu closed");

    support::drive_with_timeout(
        &mut s,
        &["f10", "wait:New tab"],
        drive_ms(case_timeout(&case)),
    );
    support::drive_with_timeout(
        &mut s,
        &["right", "right", "wait:Zoom pane"],
        drive_ms(case_timeout(&case)),
    );
    support::drive_with_timeout(
        &mut s,
        &["down", "enter", "wait:Redrawn"],
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "02-capsule-menu-chosen");

    support::drive_with_timeout(
        &mut s,
        &["f10", "wait:New tab"],
        drive_ms(case_timeout(&case)),
    );
    click_text(
        &mut s,
        "payments-platform ❯",
        0,
        case_timeout(&case),
        "N1 outside click",
    );
    wait_absent(
        &mut s,
        "New tab",
        case_timeout(&case),
        "N1: outside click closes",
    );
    let n1 = live_text(&mut s);
    assert!(
        !n1.contains("Command palette") && count_lines(&n1, "Mix (3)") == 1,
        "N1: nothing chosen, no tab spawned"
    );

    click_text(&mut s, BOOT, 2, case_timeout(&case), "A2 brand menu");
    waits::wait_state(&mut s, "About jackin-preview", "brand menu open");
    support::press_step(&s, "escape");
    wait_absent(
        &mut s,
        "About jackin-preview",
        case_timeout(&case),
        "brand menu cleared",
    );

    let bcase = j2_case("menubar_host_manager", RETURNING, BOOT, 80, 24);
    write_provenance_named(&dir, &bcase, "provenance-host.txt");
    let mut b = support::spawn_boot(&bcase);

    support::drive_with_timeout(
        &mut b,
        &["f10", "wait:New workspace…"],
        drive_ms(case_timeout(&bcase)),
    );
    checkpoint(&mut b, &dir, "03-host-menu-open");
    let host = live_text(&mut b);

    assert!(host.contains("Edit workspace…"), "V2: edit row");
    assert!(host.contains("Launch…"), "V2: launch row");
    let del_line = host
        .lines()
        .find(|l| l.contains("Delete workspace…"))
        .expect("V2: delete row");
    let after = del_line.split("Delete workspace…").nth(1).unwrap_or("");
    let cell = after.split('│').next().unwrap_or("").trim();
    assert_eq!(cell, "d", "V2: delete shortcut right-aligned: {del_line:?}");
    let hrows: Vec<&str> = host.lines().collect();
    let del_i = hrows
        .iter()
        .position(|l| l.contains("Delete workspace…"))
        .expect("V2: delete row");
    let ref_i = hrows
        .iter()
        .position(|l| l.contains("Refresh"))
        .expect("V2: refresh row");
    assert!(
        hrows[del_i..ref_i].iter().any(|l| l.contains("─")),
        "V2: separator under Delete"
    );
    assert!(host.contains("Quit"), "V2: quit row");
    support::press_step(&b, "escape");
    wait_absent(
        &mut b,
        "New workspace…",
        case_timeout(&bcase),
        "host menu closed",
    );

    support::drive_with_timeout(
        &mut b,
        &["f10", "wait:New workspace…"],
        drive_ms(case_timeout(&bcase)),
    );
    support::drive_with_timeout(
        &mut b,
        &["right", "wait:Workspace manager"],
        drive_ms(case_timeout(&bcase)),
    );
    assert!(
        live_text(&mut b).contains("Global settings"),
        "S2: fixed Go menu"
    );
    support::drive_with_timeout(
        &mut b,
        &["right", "wait:Key reference"],
        drive_ms(case_timeout(&bcase)),
    );
    support::press_step(&b, "escape");
    wait_absent(
        &mut b,
        "Key reference",
        case_timeout(&bcase),
        "help menu closed",
    );

    click_text(&mut b, BOOT, 2, case_timeout(&bcase), "A2 host brand");
    waits::wait_state(&mut b, "About", "host About open");
    support::press_step(&b, "escape");
    wait_absent(
        &mut b,
        "Design system",
        case_timeout(&bcase),
        "host About closed",
    );

    support::drive_with_timeout(
        &mut b,
        &["f10", "wait:New workspace…"],
        drive_ms(case_timeout(&bcase)),
    );
    support::drive_with_timeout(
        &mut b,
        &["right", "wait:Workspace manager"],
        drive_ms(case_timeout(&bcase)),
    );
    support::drive_with_timeout(
        &mut b,
        &["down", "down", "enter", "wait:Usage › Overview"],
        drive_ms(case_timeout(&bcase)),
    );
    checkpoint(&mut b, &dir, "04-host-menu-chosen");

    let ccase = j2_case("menubar_host_accounts", ACCOUNTS, BOOT, 80, 24);
    write_provenance_named(&dir, &ccase, "provenance-accounts.txt");
    let mut c = support::spawn_boot(&ccase);
    support::drive_with_timeout(
        &mut c,
        &["f10", "wait:Add account…"],
        drive_ms(case_timeout(&ccase)),
    );
    assert!(
        live_text(&mut c).contains("Filter…"),
        "S2: accounts File menu"
    );
    support::press_step(&c, "escape");
    wait_absent(
        &mut c,
        "Add account…",
        case_timeout(&ccase),
        "accounts menu closed",
    );

    support::drive_with_timeout(
        &mut c,
        &["a", "wait:New account"],
        drive_ms(case_timeout(&ccase)),
    );
    support::drive_with_timeout(&mut c, &["enter"], drive_ms(case_timeout(&ccase)));
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        live_text(&mut c).contains("EDIT"),
        "N2: form is editing (presence)"
    );
    support::press_step(&c, "f10");
    std::thread::sleep(Duration::from_millis(400));
    let editing = live_text(&mut c);
    assert!(
        !editing.contains("Add account…") && editing.contains("New account"),
        "N2: F10 refused while editing"
    );

    support::press_step(&c, "escape");
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        live_text(&mut c).contains("New account"),
        "N2: first Esc only reverts"
    );
    support::press_step(&c, "escape");
    wait_absent(
        &mut c,
        "New account",
        case_timeout(&ccase),
        "form cancelled",
    );
    support::drive_with_timeout(
        &mut c,
        &["f10", "wait:Add account…"],
        drive_ms(case_timeout(&ccase)),
    );
    eprintln!("j2 menubar_capsule_host: strips, fixed/per-route menus, dispatch, brand, f10-guard");
}

#[test]
#[ignore = "jackin j2 check; run with --ignored"]
fn j2_manager_help() {
    let dir = j2_dir("j2_manager_help");
    let case = j2_case("manager_help", RETURNING, BOOT, 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["?", "wait:Keyboard shortcuts"],
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "01-help-open");
    let help = live_text(&mut s);

    let wrow = help.lines().position(|l| l.contains("Workspaces"));
    let irow = help.lines().position(|l| l.contains("Instances"));
    assert!(
        wrow.is_some() && irow.is_some() && wrow.unwrap() < irow.unwrap(),
        "V1: Workspaces stacks above Instances"
    );

    support::press_step(&s, "pagedown");
    std::thread::sleep(Duration::from_millis(250));
    assert!(
        live_text(&mut s).contains("Everywhere"),
        "V1: everywhere section"
    );
    support::press_step(&s, "pagedown");
    std::thread::sleep(Duration::from_millis(250));
    assert!(live_text(&mut s).contains("Mouse"), "V1: mouse section");
    for _ in 0..50 {
        support::press_step(&s, "up");
        std::thread::sleep(Duration::from_millis(20));
    }
    waits::wait_state(&mut s, "1–", "back at head");

    assert_line_has(&help, "Keyboard shortcuts", "of 45", "V2 position label");
    assert!(help.contains("↑↓ Scroll · Esc Close"), "V2: scroll footer");

    assert!(help.contains("new workspace"), "S2: manager rows");
    assert!(help.contains("expand / collapse all"), "S2: manager rows");

    support::press_step(&s, "down");
    std::thread::sleep(Duration::from_millis(250));
    let down1 = live_text(&mut s);
    assert!(
        down1.contains("2–") && down1.contains("of 45"),
        "S1: down moves one row"
    );
    support::press_step(&s, "pagedown");
    std::thread::sleep(Duration::from_millis(250));
    let paged = live_text(&mut s);
    assert_ne!(paged, down1, "S1: pagedown jumps further");
    assert!(
        paged.contains("Keyboard shortcuts"),
        "still open after page"
    );
    let (hrow, hcol) = find_pos(&mut s, "Keyboard shortcuts", case_timeout(&case));
    wheel_at(&mut s, hcol, hrow + 4, Wheel::Down, 2);
    std::thread::sleep(Duration::from_millis(250));
    assert_ne!(live_text(&mut s), paged, "S1: wheel scrolls");
    checkpoint(&mut s, &dir, "02-help-scrolled");

    let before_x = live_text(&mut s);
    support::press_step(&s, "x");
    std::thread::sleep(Duration::from_millis(250));
    assert_eq!(live_text(&mut s), before_x, "A2: `x` swallowed");
    support::press_step(&s, "e");
    std::thread::sleep(Duration::from_millis(250));
    assert_eq!(live_text(&mut s), before_x, "A2: `e` swallowed");

    for _ in 0..50 {
        support::press_step(&s, "up");
        std::thread::sleep(Duration::from_millis(20));
    }
    let top = live_text(&mut s);
    assert!(top.contains("Keyboard shortcuts"), "N1: still open at top");
    assert!(top.contains("1–"), "N1: back at the head");
    support::press_step(&s, "up");
    std::thread::sleep(Duration::from_millis(250));
    assert_eq!(live_text(&mut s), top, "N1: past-top moves nothing");
    for _ in 0..6 {
        support::press_step(&s, "pagedown");
        std::thread::sleep(Duration::from_millis(80));
    }
    let bottom = live_text(&mut s);
    assert!(
        bottom.contains("Keyboard shortcuts"),
        "N1: still open at bottom"
    );
    support::press_step(&s, "pagedown");
    std::thread::sleep(Duration::from_millis(250));
    assert_eq!(live_text(&mut s), bottom, "N1: past-bottom moves nothing");

    support::press_step(&s, "escape");
    wait_absent(
        &mut s,
        "Keyboard shortcuts",
        case_timeout(&case),
        "Esc closes",
    );
    waits::wait_state(&mut s, "Current directory", "manager back");
    for key in ["?", "q", "enter"] {
        support::drive_with_timeout(
            &mut s,
            &["?", "wait:Keyboard shortcuts"],
            drive_ms(case_timeout(&case)),
        );
        support::press_step(&s, key);
        wait_absent(
            &mut s,
            "Keyboard shortcuts",
            case_timeout(&case),
            &format!("`{key}` closes"),
        );
    }
    checkpoint(&mut s, &dir, "03-help-closed");

    let bcase = j2_case("manager_help_fits", RETURNING, BOOT, 120, 40);
    write_provenance_named(&dir, &bcase, "provenance-fits.txt");
    let mut b = support::spawn_boot(&bcase);
    support::drive_with_timeout(
        &mut b,
        &["?", "wait:Keyboard shortcuts"],
        drive_ms(case_timeout(&bcase)),
    );
    let fits = live_text(&mut b);
    assert_line_has(&fits, "Workspaces", "Instances", "V1: column heads row 0");
    assert_line_has(&fits, "Workspaces", "Everywhere", "V1: three columns");
    assert!(
        !fits.contains("of 45") && !fits.contains("┃"),
        "N2: no scrollbar, no position label when the list fits"
    );
    support::press_step(&b, "escape");
    wait_absent(
        &mut b,
        "Keyboard shortcuts",
        case_timeout(&bcase),
        "fits overlay closed",
    );

    let ccase = j2_case("manager_help_capsule", CAPSULE, BOOT, 80, 24);
    write_provenance_named(&dir, &ccase, "provenance-capsule.txt");
    let mut c = support::spawn_boot(&ccase);
    support::drive_with_timeout(
        &mut c,
        &["f10", "wait:New tab"],
        drive_ms(case_timeout(&ccase)),
    );
    support::drive_with_timeout(
        &mut c,
        &["right", "right", "right", "right", "wait:Key reference"],
        drive_ms(case_timeout(&ccase)),
    );
    support::drive_with_timeout(
        &mut c,
        &["enter", "wait:Keyboard shortcuts"],
        drive_ms(case_timeout(&ccase)),
    );
    let chelp = live_text(&mut c);
    assert!(chelp.contains("Prefix commands"), "S2: capsule table");
    assert!(
        !chelp.contains("Workspaces"),
        "S2: capsule table differs from manager"
    );
    support::press_step(&c, "escape");
    wait_absent(
        &mut c,
        "Keyboard shortcuts",
        case_timeout(&ccase),
        "capsule help closed",
    );
    eprintln!("j2 manager_help: columns, scroll, swallow, bounds, closers, per-route");
}

fn form_focus_on(text: &str, label: &str) -> bool {
    let rows: Vec<&str> = text.lines().collect();
    rows.iter()
        .position(|l| l.contains(label))
        .is_some_and(|i| {
            [0, 1, 2]
                .iter()
                .any(|off| rows.get(i + off).is_some_and(|l| l.contains("▎")))
        })
}

#[test]
#[ignore = "jackin j2 check; run with --ignored"]
fn j2_accounts_add_form() {
    let dir = j2_dir("j2_accounts_add_form");
    let case = j2_case("accounts_add_form", ACCOUNTS, BOOT, 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["a", "wait:New account"],
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "01-open");
    let open = live_text(&mut s);

    for row in [
        "Display name",
        "Purpose label",
        "Provider",
        "Credential source",
    ] {
        assert!(open.contains(row), "V1: visible field `{row}`");
    }
    assert_line_has(&open, "Cancel", "Save", "V1 action row");

    for hidden in [
        "Claude profile / home folder",
        "Anthropic API key",
        "Browse…",
    ] {
        assert!(!open.contains(hidden), "V2: `{hidden}` hidden");
    }
    assert!(form_focus_on(&open, "Display name"), "focus starts on name");

    for (i, label) in ["Purpose label", "Provider", "Credential source"]
        .iter()
        .enumerate()
    {
        support::press_step(&s, "tab");
        std::thread::sleep(Duration::from_millis(200));
        assert!(
            form_focus_on(&live_text(&mut s), label),
            "A1: tab {} reaches `{label}`",
            i + 1
        );
    }
    assert!(
        !live_text(&mut s).contains("Display name"),
        "A1: keep-visible scrolled the name field out"
    );

    for (i, label) in ["Enter plain text instead", "Validate", "Cancel", "Save"]
        .iter()
        .enumerate()
    {
        support::press_step(&s, "tab");
        std::thread::sleep(Duration::from_millis(200));
        assert!(
            live_text(&mut s).contains(&format!("▎{label}")),
            "A1: tab {} reaches `{label}`",
            i + 4
        );
    }

    support::press_step(&s, "tab");
    std::thread::sleep(Duration::from_millis(200));
    assert!(
        form_focus_on(&live_text(&mut s), "Purpose label"),
        "A1: tab 8 wraps to the first registered stop"
    );

    wheel_at(&mut s, 40, 12, Wheel::Up, 10);
    std::thread::sleep(Duration::from_millis(300));
    support::press_step(&s, "backtab");
    std::thread::sleep(Duration::from_millis(200));
    assert!(
        form_focus_on(&live_text(&mut s), "Display name"),
        "A1: backtab reaches the re-registered name"
    );
    support::press_step(&s, "backtab");
    std::thread::sleep(Duration::from_millis(200));
    assert!(
        live_text(&mut s).contains("▎Save"),
        "A1: backtab wraps over the top to Save"
    );
    support::press_step(&s, "tab");
    std::thread::sleep(Duration::from_millis(200));
    assert!(
        form_focus_on(&live_text(&mut s), "Display name"),
        "A1: tab wraps over the bottom to name"
    );

    for _ in 0..3 {
        support::press_step(&s, "tab");
        std::thread::sleep(Duration::from_millis(120));
    }
    assert!(
        form_focus_on(&live_text(&mut s), "Credential source"),
        "staged on source for the wheel-down"
    );
    wheel_at(&mut s, 40, 12, Wheel::Down, 3);
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        live_text(&mut s).contains("1Password reference"),
        "wheel reveals the chooser"
    );
    click_text(
        &mut s,
        "Choose…",
        0,
        case_timeout(&case),
        "open account picker",
    );
    waits::wait_state(&mut s, "Choose 1Password account", "account picker open");
    support::press_step(&s, "escape");
    wait_absent(
        &mut s,
        "Choose 1Password account",
        case_timeout(&case),
        "picker closed",
    );
    wheel_at(&mut s, 40, 12, Wheel::Up, 10);
    std::thread::sleep(Duration::from_millis(300));

    for _ in 0..10 {
        support::press_step(&s, "tab");
        std::thread::sleep(Duration::from_millis(120));
        let t = live_text(&mut s);
        assert!(!t.contains("Claude profile"), "N2: folder row never shown");
        assert!(!t.contains("Browse…"), "N2: browse row never shown");
        assert!(!t.contains("Anthropic API key"), "N2: key row never shown");
        assert_eq!(
            count_lines(&t, "API key"),
            1,
            "N2: only the radio option mentions API key"
        );
    }

    for _ in 0..12 {
        if form_focus_on(&live_text(&mut s), "Display name") {
            break;
        }
        wheel_at(&mut s, 40, 12, Wheel::Up, 10);
        std::thread::sleep(Duration::from_millis(150));
        support::press_step(&s, "backtab");
        std::thread::sleep(Duration::from_millis(120));
    }
    assert!(
        form_focus_on(&live_text(&mut s), "Display name"),
        "ring returns to name"
    );

    support::press_step(&s, "enter");
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        live_text(&mut s).contains("EDIT"),
        "S1: EDIT badge while editing"
    );

    support::drive_with_timeout(&mut s, &["type:zzz"], drive_ms(case_timeout(&case)));
    assert!(live_text(&mut s).contains("zzz"), "draft typed");
    support::press_step(&s, "escape");
    std::thread::sleep(Duration::from_millis(300));
    let reverted = live_text(&mut s);
    assert!(!reverted.contains("zzz"), "N1: Esc reverts the draft");
    assert!(
        reverted.contains("New account"),
        "N1: no Cancel fired while editing"
    );

    support::drive_with_timeout(
        &mut s,
        &["enter", "type:zz9", "enter"],
        drive_ms(case_timeout(&case)),
    );
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        live_text(&mut s).contains("zz9"),
        "S2: committed value retained"
    );

    for _ in 0..8 {
        if form_focus_on(&live_text(&mut s), "Credential source") {
            break;
        }
        support::press_step(&s, "tab");
        std::thread::sleep(Duration::from_millis(120));
    }
    assert!(
        form_focus_on(&live_text(&mut s), "Credential source"),
        "ring reaches source"
    );
    support::press_step(&s, "down");
    std::thread::sleep(Duration::from_millis(300));

    wheel_at(&mut s, 40, 12, Wheel::Down, 3);
    std::thread::sleep(Duration::from_millis(250));
    wheel_at(&mut s, 40, 12, Wheel::Down, 3);
    waits::wait_state(
        &mut s,
        "Claude profile / home folder",
        "Changed reveals folder",
    );
    let revealed = live_text(&mut s);
    assert!(
        !revealed.contains("1Password reference"),
        "op row hides on the folder branch"
    );

    support::press_step(&s, "tab");
    std::thread::sleep(Duration::from_millis(200));
    if !form_focus_on(&live_text(&mut s), "Claude profile / home folder") {
        click_text(
            &mut s,
            "Claude profile / home folder",
            0,
            case_timeout(&case),
            "focus folder",
        );
    }
    assert!(
        form_focus_on(&live_text(&mut s), "Claude profile / home folder"),
        "revealed row focusable"
    );

    support::press_step(&s, "escape");
    wait_absent(
        &mut s,
        "New account",
        case_timeout(&case),
        "Cancel closes form",
    );
    waits::wait_state(&mut s, "Cancelled · nothing saved", "cancel status");
    checkpoint(&mut s, &dir, "02-cancelled");
    support::drive_with_timeout(
        &mut s,
        &["a", "wait:New account"],
        drive_ms(case_timeout(&case)),
    );
    assert!(
        !live_text(&mut s).contains("zz9"),
        "A2: cancelled drafts dropped"
    );

    support::press_step(&s, "escape");
    wait_absent(&mut s, "New account", case_timeout(&case), "clean cancel");
    eprintln!("j2 accounts_add_form: ring, hidden, edit/cancel, changed-reveal");
}

#[test]
#[ignore = "jackin j2 check; run with --ignored"]
fn j2_prelude_chain() {
    let dir = j2_dir("j2_prelude_chain");
    let case = j2_case("prelude_chain", FIRST400, BOOT, 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["n", &format!("wait:{PRELUDE_S1}")],
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "01-step1-source");

    assert!(
        live_text(&mut s).contains("New workspace · step 1 of 5 · Source"),
        "V1: source title"
    );
    let source_before = live_text(&mut s);

    support::drive_with_timeout(
        &mut s,
        &["space", "wait:step 2 of 5 · Destination"],
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "02-step2-destination");
    let dest = live_text(&mut s);
    assert!(
        dest.contains("New workspace · step 2 of 5 · Destination"),
        "V1: destination title"
    );

    assert!(
        dest.contains("✓ Source · Destination · Edit · Working dir · Name"),
        "V2: stepper line"
    );

    assert!(dest.contains("Same path"), "S1: default destination row");
    assert!(dest.contains("Source  "), "S1: source summary line");
    let default_line = dest
        .lines()
        .find(|l| l.contains("Same path"))
        .expect("S1: default line")
        .to_string();

    support::press_step(&s, "escape");
    waits::wait_state(&mut s, PRELUDE_S1, "rewind to source");
    checkpoint(&mut s, &dir, "03-rewound-source");

    let rewound = live_text(&mut s);
    assert!(
        rewound.contains("~/src") && rewound.contains("payments-platform"),
        "S2: rewound browser sits at the choice's parent"
    );
    assert_line_has(&rewound, "▎", "..", "S2: fresh browser cursor home");
    assert_ne!(
        rewound, source_before,
        "S2: the listing is the parent, not the retained view"
    );

    for _ in 0..12 {
        if live_text(&mut s)
            .lines()
            .any(|l| l.contains("▎") && l.contains("payments-platform"))
        {
            break;
        }
        support::press_step(&s, "down");
        std::thread::sleep(Duration::from_millis(120));
    }
    support::drive_with_timeout(
        &mut s,
        &[
            "enter",
            "wait:Cargo.toml",
            "space",
            "wait:step 2 of 5 · Destination",
        ],
        drive_ms(case_timeout(&case)),
    );
    let dest2 = live_text(&mut s);
    let default_line2 = dest2
        .lines()
        .find(|l| l.contains("Same path"))
        .expect("N2: default line again");
    assert_eq!(default_line2, default_line, "N2: accepted value retained");

    support::press_step(&s, "escape");
    waits::wait_state(&mut s, PRELUDE_S1, "rewind to source");
    support::press_step(&s, "escape");
    wait_absent(&mut s, PRELUDE_S1, case_timeout(&case), "chain cancelled");
    waits::wait_state(&mut s, "Cancelled · nothing created", "cancel status");

    let manager = live_text(&mut s);
    assert!(
        manager.contains("+ New workspace"),
        "N1: manager tree intact"
    );
    assert!(!manager.contains("step 1 of 5"), "N1: no prelude residue");
    checkpoint(&mut s, &dir, "04-cancelled");
    eprintln!("j2 prelude_chain: advance, stepper, rewind, cancel");
}

#[test]
#[ignore = "jackin j2 check; run with --ignored"]
fn j2_prelude_validate() {
    let dir = j2_dir("j2_prelude_validate");
    let case = j2_case("prelude_validate", RETURNING, BOOT, 120, 40);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["n", &format!("wait:{PRELUDE_S1}")],
        drive_ms(case_timeout(&case)),
    );
    support::drive_with_timeout(
        &mut s,
        &["space", "wait:step 2 of 5 · Destination"],
        drive_ms(case_timeout(&case)),
    );
    support::drive_with_timeout(
        &mut s,
        &["down", "enter", "wait:step 3 of 5 · Edit destination"],
        drive_ms(case_timeout(&case)),
    );

    support::drive_with_timeout(
        &mut s,
        &["ctrl-e", "ctrl-u", "type:relative/path", "enter"],
        drive_ms(case_timeout(&case)),
    );
    waits::wait_state(&mut s, "Destination must be an absolute path", "S1 error");
    checkpoint(&mut s, &dir, "01-edit-error");
    let edit_err = live_text(&mut s);

    assert!(
        edit_err.contains("New workspace · step 3 of 5 · Edit destination"),
        "V2: same step after the error"
    );
    assert!(
        edit_err.contains("relative/path"),
        "draft retained with the error"
    );
    assert!(
        !edit_err.contains("step 4 of 5"),
        "N1: invalid Next never advances"
    );

    support::drive_with_timeout(
        &mut s,
        &["ctrl-e", "ctrl-u", "type:/a//b", "enter"],
        drive_ms(case_timeout(&case)),
    );
    waits::wait_state(
        &mut s,
        "Destination has an empty path segment",
        "S1 segments",
    );
    assert!(
        live_text(&mut s).contains("step 3 of 5 · Edit destination"),
        "N1: still on Edit"
    );

    support::drive_with_timeout(
        &mut s,
        &["ctrl-e", "ctrl-u", "enter"],
        drive_ms(case_timeout(&case)),
    );
    waits::wait_state(&mut s, "Required", "blank stops at widget");
    assert!(
        live_text(&mut s).contains("step 3 of 5 · Edit destination"),
        "N1: blank never advances"
    );

    support::drive_with_timeout(
        &mut s,
        &[
            "enter",
            "ctrl-e",
            "ctrl-u",
            "type:/work/ok",
            "enter",
            "wait:step 4 of 5 · Working dir",
        ],
        drive_ms(case_timeout(&case)),
    );
    support::drive_with_timeout(
        &mut s,
        &["enter", "wait:step 5 of 5 · Name"],
        drive_ms(case_timeout(&case)),
    );

    support::drive_with_timeout(
        &mut s,
        &["type:/bad", "enter"],
        drive_ms(case_timeout(&case)),
    );
    waits::wait_state(&mut s, "Name cannot contain /", "S2 slash");
    checkpoint(&mut s, &dir, "02-name-error");
    assert!(
        live_text(&mut s).contains("New workspace · step 5 of 5 · Name"),
        "N1: slash never advances"
    );

    support::drive_with_timeout(
        &mut s,
        &["ctrl-e", "ctrl-u", "type:payments-platform", "enter"],
        drive_ms(case_timeout(&case)),
    );
    waits::wait_state(&mut s, "already exists", "S2 duplicate");
    assert!(
        live_text(&mut s).contains("New workspace · step 5 of 5 · Name"),
        "N1: duplicate never advances"
    );

    support::drive_with_timeout(
        &mut s,
        &[
            "ctrl-e",
            "ctrl-u",
            "type:zzok9",
            "enter",
            "wait:review and save",
        ],
        drive_ms(case_timeout(&case)),
    );
    let created = live_text(&mut s);
    assert!(created.contains("› edit"), "A1: editor opened on pending");
    assert!(
        created.contains("Workspace zzok9 · review and save"),
        "A1: pending status"
    );
    checkpoint(&mut s, &dir, "03-created");

    let bcase = j2_case("prelude_validate_cancel", RETURNING, BOOT, 120, 40);
    write_provenance_named(&dir, &bcase, "provenance-cancel.txt");
    let mut b = support::spawn_boot(&bcase);
    support::drive_with_timeout(
        &mut b,
        &["n", &format!("wait:{PRELUDE_S1}")],
        drive_ms(case_timeout(&bcase)),
    );
    support::press_step(&b, "escape");
    wait_absent(&mut b, PRELUDE_S1, case_timeout(&bcase), "source cancelled");
    waits::wait_state(&mut b, "Cancelled · nothing created", "A2 status");
    let manager = live_text(&mut b);
    assert!(!manager.contains("› edit"), "N2: no editor opened");
    assert!(!manager.contains("zzok9"), "N2: nothing persisted");
    eprintln!("j2 prelude_validate: destination+name errors, fixes, create, cancel");
}

#[test]
#[ignore = "jackin j2 check; run with --ignored"]
fn j2_prelude_branch_create() {
    let dir = j2_dir("j2_prelude_branch_create");
    let case = j2_case("prelude_branch_create", FIRST400, BOOT, 100, 30);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["n", &format!("wait:{PRELUDE_S1}")],
        drive_ms(case_timeout(&case)),
    );
    support::drive_with_timeout(
        &mut s,
        &["space", "wait:step 2 of 5 · Destination"],
        drive_ms(case_timeout(&case)),
    );
    assert!(
        live_text(&mut s).contains("✓ Source · Destination · Edit · Working dir · Name"),
        "destination stepper (presence)"
    );

    support::drive_with_timeout(
        &mut s,
        &["enter", "wait:step 4 of 5 · Working dir"],
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "01-skipped-edit");
    let skipped = live_text(&mut s);
    assert!(
        !skipped.contains("step 3 of 5"),
        "V1: Edit skipped on the same path"
    );

    assert_line_has(&skipped, "▪", "~/src/payments-platform", "V2 dest row");
    assert_line_has(&skipped, "·", "crates", "V2 subdir row");
    let wrows: Vec<&str> = skipped.lines().collect();
    let dest_i = wrows
        .iter()
        .position(|l| l.contains("▪"))
        .expect("V2: dest row");
    let sub_i = wrows
        .iter()
        .position(|l| l.contains("crates"))
        .expect("V2: subdir row");
    assert!(dest_i < sub_i, "V2: destination lists first");

    support::press_step(&s, "escape");
    waits::wait_state(&mut s, "step 2 of 5 · Destination", "skipped rewind");
    assert!(
        !live_text(&mut s).contains("step 3 of 5"),
        "rewind skips Edit too"
    );
    support::drive_with_timeout(
        &mut s,
        &["enter", "wait:step 4 of 5 · Working dir"],
        drive_ms(case_timeout(&case)),
    );

    support::drive_with_timeout(
        &mut s,
        &["enter", "wait:step 5 of 5 · Name"],
        drive_ms(case_timeout(&case)),
    );

    assert!(
        live_text(&mut s).contains("payments-platform"),
        "name defaults to the basename"
    );
    support::drive_with_timeout(
        &mut s,
        &[
            "ctrl-e",
            "ctrl-u",
            "type:zzbr3",
            "enter",
            "wait:review and save",
        ],
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "02-created");

    let created = live_text(&mut s);
    assert!(
        created.contains("Workspace zzbr3 · review and save in the editor"),
        "A2: pending status"
    );
    assert!(created.contains("› edit"), "S2: editor opened");

    let bcase = j2_case("prelude_branch_edit", FIRST400, BOOT, 100, 30);
    write_provenance_named(&dir, &bcase, "provenance-edit.txt");
    let mut b = support::spawn_boot(&bcase);
    support::drive_with_timeout(
        &mut b,
        &["n", &format!("wait:{PRELUDE_S1}")],
        drive_ms(case_timeout(&bcase)),
    );
    support::drive_with_timeout(
        &mut b,
        &["space", "wait:step 2 of 5 · Destination"],
        drive_ms(case_timeout(&bcase)),
    );
    support::drive_with_timeout(
        &mut b,
        &["down", "enter", "wait:step 3 of 5 · Edit destination"],
        drive_ms(case_timeout(&bcase)),
    );
    assert!(
        live_text(&mut b).contains("New workspace · step 3 of 5 · Edit destination"),
        "N2: Edit opens on the edit branch"
    );
    support::drive_with_timeout(
        &mut b,
        &[
            "ctrl-e",
            "ctrl-u",
            "type:/work/eb",
            "enter",
            "wait:step 4 of 5 · Working dir",
        ],
        drive_ms(case_timeout(&bcase)),
    );
    support::press_step(&b, "escape");
    waits::wait_state(
        &mut b,
        "step 3 of 5 · Edit destination",
        "N2: rewind lands on Edit",
    );
    eprintln!("j2 prelude_branch_create: skip-edit, workdir rows, create, edit-branch");
}

#[test]
#[ignore = "jackin j2 check; run with --ignored"]
fn j2_footer_layers() {
    let dir = j2_dir("j2_footer_layers");
    let case = j2_case("footer_layers", RETURNING, BOOT, 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    let screen_footer = footer_line(&live_text(&mut s)).to_string();
    assert!(
        screen_footer.contains("Launch"),
        "screen hints: {screen_footer:?}"
    );
    assert!(!screen_footer.trim().is_empty(), "N1: footer never blank");

    assert!(
        screen_footer.starts_with(' ') && screen_footer.ends_with(' '),
        "V2: footer row centered: {screen_footer:?}"
    );
    checkpoint(&mut s, &dir, "01-screen-hints");

    let bcase = j2_case("footer_layers_modal", FIRST400, BOOT, 80, 24);
    write_provenance_named(&dir, &bcase, "provenance-modal.txt");
    let mut b = support::spawn_boot(&bcase);
    support::drive_with_timeout(
        &mut b,
        &["n", &format!("wait:{PRELUDE_S1}")],
        drive_ms(case_timeout(&bcase)),
    );
    checkpoint(&mut b, &dir, "02-modal-hints");
    let modal_footer = footer_line(&live_text(&mut b)).to_string();
    assert!(
        modal_footer.contains("Space") && modal_footer.contains("Choose"),
        "V1: modal hints win: {modal_footer:?}"
    );
    assert!(
        !modal_footer.contains("Backspace"),
        "V1: screen layer shadowed: {modal_footer:?}"
    );
    assert!(
        !modal_footer.trim().is_empty(),
        "N1: modal footer never blank"
    );

    support::drive_with_timeout(
        &mut b,
        &["space", "wait:step 2 of 5 · Destination"],
        drive_ms(case_timeout(&bcase)),
    );
    support::drive_with_timeout(
        &mut b,
        &["down", "enter", "wait:step 3 of 5 · Edit destination"],
        drive_ms(case_timeout(&bcase)),
    );
    let edit_footer = footer_line(&live_text(&mut b)).to_string();
    assert!(
        edit_footer.contains("EDIT"),
        "V2/S1: EDIT badge on dialog edit: {edit_footer:?}"
    );

    for _ in 0..5 {
        support::press_step(&b, "escape");
        std::thread::sleep(Duration::from_millis(200));
        if !live_text(&mut b).contains("step ") {
            break;
        }
    }
    waits::wait_state(&mut b, "Cancelled · nothing created", "prelude cancelled");
    let restored = footer_line(&live_text(&mut b)).to_string();
    assert!(
        restored.contains("Launch"),
        "A1: screen layer restored: {restored:?}"
    );

    assert!(
        restored.contains("Cancelled · nothing created"),
        "S2: status owns the slot: {restored:?}"
    );

    let ccase = j2_case("footer_layers_form", ACCOUNTS, BOOT, 80, 24);
    write_provenance_named(&dir, &ccase, "provenance-form.txt");
    let mut c = support::spawn_boot(&ccase);
    support::drive_with_timeout(
        &mut c,
        &["a", "wait:New account"],
        drive_ms(case_timeout(&ccase)),
    );
    support::drive_with_timeout(&mut c, &["enter"], drive_ms(case_timeout(&ccase)));
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        footer_line(&live_text(&mut c)).contains("EDIT"),
        "S1: EDIT badge on form edit"
    );

    let dcase = j2_case("footer_layers_browser", FIRST400, BOOT, 80, 24);
    write_provenance_named(&dir, &dcase, "provenance-browser.txt");
    let mut d = support::spawn_boot(&dcase);
    support::drive_with_timeout(
        &mut d,
        &["n", &format!("wait:{PRELUDE_S1}")],
        drive_ms(case_timeout(&dcase)),
    );
    support::drive_with_timeout(&mut d, &["g"], drive_ms(case_timeout(&dcase)));
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        footer_line(&live_text(&mut d)).contains("EDIT"),
        "S1: EDIT badge on browser edit"
    );

    let ecase = j2_case("footer_layers_screen", RETURNING, BOOT, 80, 24);
    write_provenance_named(&dir, &ecase, "provenance-screen.txt");
    let mut e = support::spawn_boot(&ecase);
    support::drive_with_timeout(
        &mut e,
        &["e", "wait:› edit"],
        drive_ms(case_timeout(&ecase)),
    );
    support::drive_with_timeout(&mut e, &["down", "enter"], drive_ms(case_timeout(&ecase)));
    std::thread::sleep(Duration::from_millis(300));
    let escreen = live_text(&mut e);
    assert!(
        footer_line(&escreen).contains("EDIT"),
        "S1: EDIT badge on screen edit"
    );
    assert!(escreen.contains("› edit"), "editor still bare (no modal)");

    support::drive_with_timeout(
        &mut s,
        &["f10", "wait:New workspace…"],
        drive_ms(case_timeout(&case)),
    );
    let menu_footer = footer_line(&live_text(&mut s)).to_string();
    assert!(
        menu_footer.contains("Menu") && menu_footer.contains("Choose"),
        "A2: menu layer wins: {menu_footer:?}"
    );
    assert!(
        !menu_footer.contains("Launch"),
        "A2: screen layer shadowed: {menu_footer:?}"
    );
    support::press_step(&s, "escape");
    wait_absent(&mut s, "New workspace…", case_timeout(&case), "menu closed");
    assert!(
        footer_line(&live_text(&mut s)).contains("Launch"),
        "A2: screen layer restored"
    );

    let fcase = j2_case("footer_layers_intro", FIRST40, BOOT_SKIP, 80, 24);
    write_provenance_named(&dir, &fcase, "provenance-intro.txt");
    let mut f = support::spawn_boot(&fcase);
    let intro_footer = footer_line(&live_text(&mut f)).to_string();
    assert!(
        intro_footer.contains("Enter") && intro_footer.contains("Skip"),
        "N1: cinematic paints its own hint: {intro_footer:?}"
    );

    let gcase = j2_case("footer_layers_created", FIRST400, BOOT, 80, 24);
    write_provenance_named(&dir, &gcase, "provenance-created.txt");
    let mut g = support::spawn_boot(&gcase);
    support::drive_with_timeout(
        &mut g,
        &["n", &format!("wait:{PRELUDE_S1}")],
        drive_ms(case_timeout(&gcase)),
    );
    support::drive_with_timeout(
        &mut g,
        &["space", "wait:step 2 of 5 · Destination"],
        drive_ms(case_timeout(&gcase)),
    );
    support::drive_with_timeout(
        &mut g,
        &["enter", "wait:step 4 of 5 · Working dir"],
        drive_ms(case_timeout(&gcase)),
    );
    support::drive_with_timeout(
        &mut g,
        &["enter", "wait:step 5 of 5 · Name"],
        drive_ms(case_timeout(&gcase)),
    );
    support::drive_with_timeout(
        &mut g,
        &[
            "ctrl-e",
            "ctrl-u",
            "type:zzhb7",
            "enter",
            "wait:review and save",
        ],
        drive_ms(case_timeout(&gcase)),
    );
    let created_footer = footer_line(&live_text(&mut g)).to_string();
    assert!(
        created_footer.contains("Workspace zzhb7 · review and save in the editor"),
        "V2: full status on the right: {created_footer:?}"
    );
    assert!(
        created_footer.contains("Tab") && created_footer.contains("Next"),
        "N2: hints intact beside the status: {created_footer:?}"
    );
    eprintln!("j2 footer_layers: modal>screen>fallback, EDIT everywhere, status slot");
}

#[test]
#[ignore = "jackin j2 check; run with --ignored"]
fn j2_capsule_inspect_diff() {
    let dir = j2_dir("j2_capsule_inspect_diff");
    let case = j2_case("capsule_inspect_diff", CAPSULE, BOOT, 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["f10", "wait:New tab"],
        drive_ms(case_timeout(&case)),
    );
    support::drive_with_timeout(
        &mut s,
        &["right", "right", "wait:Zoom pane"],
        drive_ms(case_timeout(&case)),
    );
    support::drive_with_timeout(
        &mut s,
        &["end", "enter", "wait:Inspect changes ·"],
        drive_ms(case_timeout(&case)),
    );
    checkpoint(&mut s, &dir, "01-inspect");
    let list = live_text(&mut s);

    assert!(list.contains("M"), "V2: modified markers");
    assert_line_has(&list, "↑", "not pushed", "V2: unpushed row");
    assert!(list.contains("compact · unified"), "compact list mode meta");

    support::drive_with_timeout(&mut s, &["down", "enter"], drive_ms(case_timeout(&case)));
    waits::wait_state(&mut s, "d switches", "diff open");
    let open = live_text(&mut s);
    assert!(open.contains("unified · d switches"), "V1: diff mode meta");

    let head_path = open
        .lines()
        .find_map(|l| {
            l.split_whitespace()
                .find(|t| t.contains("src/") || t.contains("docs/"))
                .map(str::to_string)
        })
        .expect("V1: diff header names the file");

    assert_ne!(open, list, "S1: list replaced by the file diff");

    support::press_step(&s, "d");
    waits::wait_state(&mut s, "review · d switches", "review mode");
    assert!(
        live_text(&mut s).contains(&head_path),
        "N1: file unchanged across the toggle"
    );
    support::press_step(&s, "d");
    waits::wait_state(&mut s, "unified · d switches", "unified mode");
    assert!(
        live_text(&mut s).contains(&head_path),
        "N1: file unchanged toggling back"
    );

    support::press_step(&s, "m");
    waits::wait_state(&mut s, "Tab tree / diff", "advanced mode");
    let adv = live_text(&mut s);
    let base = head_path.rsplit('/').next().unwrap_or("").to_string();
    assert!(
        !base.is_empty() && adv.lines().any(|l| l.contains("▎") && l.contains(&base)),
        "S1: tree cursor synced to {base:?}"
    );

    support::press_step(&s, "tab");
    std::thread::sleep(Duration::from_millis(250));
    let (hhrow, hhcol) = find_pos(&mut s, "@@", case_timeout(&case));
    let before_wheel = live_text(&mut s);
    let gutter_before = before_wheel
        .lines()
        .find(|l| l.contains("▎"))
        .unwrap_or("")
        .to_string();
    wheel_at(&mut s, hhcol, hhrow, Wheel::Down, 3);
    std::thread::sleep(Duration::from_millis(300));
    let wheeled = live_text(&mut s);
    assert_ne!(wheeled, before_wheel, "A1: wheel scrolls the diff");

    let gutter_wheeled = wheeled
        .lines()
        .find(|l| l.contains("▎"))
        .unwrap_or("")
        .to_string();
    assert_eq!(
        gutter_wheeled, gutter_before,
        "A1: tree cursor unmoved by the diff wheel"
    );
    assert!(
        wheeled.contains("mod.rs") && wheeled.contains("retry.rs"),
        "A1: tree intact while scrolling"
    );

    let grid: Vec<Vec<char>> = wheeled.lines().map(|l| l.chars().collect()).collect();
    let in_run = |c: usize, r: usize| {
        grid.get(r)
            .and_then(|l| l.get(c))
            .is_some_and(|ch| *ch == '│' || *ch == '┃')
    };
    let (sbcol, sbtop) = (65..77)
        .filter_map(|c| {
            let rows: Vec<usize> = (5..21).filter(|r| in_run(c, *r)).collect();
            let bars = rows.iter().filter(|r| grid[**r][c] == '┃').count();
            (rows.len() >= 6 && bars >= 1).then(|| (c, rows[0]))
        })
        .max_by_key(|(c, _)| *c)
        .expect("A2: diff scrollbar column");
    s.click(sbcol as u16, (sbtop + 1) as u16)
        .expect("diff scrollbar press");
    std::thread::sleep(Duration::from_millis(400));
    let jumped = live_text(&mut s);
    assert_ne!(jumped, wheeled, "A2: scrollbar press scrolls the diff");
    let gutter_jumped = jumped
        .lines()
        .find(|l| l.contains("▎"))
        .unwrap_or("")
        .to_string();
    assert_eq!(
        gutter_jumped, gutter_before,
        "A2: tree cursor unmoved by the jump"
    );

    let gutter_row = |t: &str| {
        t.lines()
            .find(|l| l.contains("▎"))
            .unwrap_or("")
            .to_string()
    };
    let header_line = |t: &str| {
        t.lines()
            .find(|l| l.contains("src/") || l.contains("docs/"))
            .unwrap_or("")
            .to_string()
    };

    wheel_at(&mut s, 40, 15, Wheel::Up, 10);
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        live_text(&mut s).contains(&head_path),
        "header back on screen"
    );

    support::press_step(&s, "tab");
    std::thread::sleep(Duration::from_millis(250));
    let mut n2_proven = false;
    for _ in 0..6 {
        let t0 = live_text(&mut s);
        let g0 = gutter_row(&t0);
        let p0 = header_line(&t0);
        support::press_step(&s, "up");
        std::thread::sleep(Duration::from_millis(200));
        let t1 = live_text(&mut s);
        let g1 = gutter_row(&t1);
        let p1 = header_line(&t1);
        if g1 != g0 && p1 == p0 && !p0.is_empty() {
            n2_proven = true;
            break;
        }
    }
    assert!(n2_proven, "N2: tree keys move the tree, never the diff");

    support::press_step(&s, "f2");
    wait_absent(
        &mut s,
        "Tab tree / diff",
        case_timeout(&case),
        "F2 back to compact",
    );
    let compact2 = live_text(&mut s);
    assert!(
        compact2.contains("compact · unified"),
        "S2: compact list meta back"
    );
    assert!(
        compact2.contains("mod.rs") && compact2.contains("retry.rs"),
        "S2: compact list rows back"
    );
    support::press_step(&s, "m");
    waits::wait_state(&mut s, "Tab tree / diff", "`m` back to advanced");
    checkpoint(&mut s, &dir, "02-advanced");

    support::press_step(&s, "escape");
    wait_absent(
        &mut s,
        "Inspect changes",
        case_timeout(&case),
        "inspect closed",
    );
    eprintln!("j2 capsule_inspect_diff: open, toggles, wheel, scrollbar, regions");
}

#[test]
#[ignore = "jackin j2 check; run with --ignored"]
fn j2_capsule_panes() {
    let dir = j2_dir("j2_capsule_panes");
    let case = j2_case("capsule_panes", CAPSULE_PLAY, BOOT, 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    waits::wait_state(&mut s, "Mix (3)", "tabs up");

    waits::wait_state(&mut s, "0003-retry-policy.md", "shell script settled");
    std::thread::sleep(Duration::from_millis(300));
    checkpoint(&mut s, &dir, "00-boot");

    let boot = live_text(&mut s);
    for tab in ["Mix (3)", "Shell", "docs"] {
        assert!(boot.contains(tab), "V1: tab `{tab}`");
    }
    click_text(
        &mut s,
        "payments-platform ❯",
        0,
        case_timeout(&case),
        "focus shell",
    );
    std::thread::sleep(Duration::from_millis(300));
    support::drive_with_timeout(&mut s, &["ctrl-b", "ctrl-l"], drive_ms(case_timeout(&case)));
    std::thread::sleep(Duration::from_millis(400));
    let boot = live_text(&mut s);

    let prompts = |t: &str| {
        t.lines()
            .filter(|l| l.contains("❯") && !l.contains("jackin❯"))
            .count()
    };
    let seg_has = |t: &str, want: &str| {
        t.lines().any(|l| {
            l.split('│').any(|s| {
                let clean: String = s.chars().filter(|c| *c != '┃').collect();
                clean.trim() == want
            })
        })
    };
    assert_eq!(prompts(&boot), 3, "V1: one prompt row per pane");
    assert!(boot.contains("payments-platform ❯"), "V1: shell prompt row");

    let screen = live_screen(&mut s);
    let cursor = screen.cursor();
    assert!(cursor.visible, "V1: caret visible");
    assert!(
        support::row_text(&screen, cursor.y).contains("❯"),
        "V1: caret sits on a prompt row"
    );
    checkpoint(&mut s, &dir, "01-split");

    let is_spinner_line = |l: &str| l.chars().any(|c| SPIN.contains(c));
    let spins_in = |t: &str| t.lines().filter(|l| is_spinner_line(l)).count();

    click_text(
        &mut s,
        "MAX_ATTEMPTS",
        0,
        case_timeout(&case),
        "focus left pane",
    );
    std::thread::sleep(Duration::from_millis(300));
    support::drive_with_timeout(&mut s, &["type:y"], drive_ms(case_timeout(&case)));
    waits::wait_state(&mut s, "❯ y", "typed `y` echoed");
    support::press_step(&s, "enter");
    std::thread::sleep(Duration::from_millis(300));
    let mut samples = vec![live_text(&mut s)];
    for _ in 0..6 {
        std::thread::sleep(Duration::from_millis(400));
        let t = live_text(&mut s);
        assert!(
            (2..=3).contains(&prompts(&t)),
            "N1: prompt envelope holds: {}",
            prompts(&t)
        );
        assert!(spins_in(&t) <= 1, "N2: no spinner pileup");
        samples.push(t);
    }
    let distinct = {
        let mut d = samples.clone();
        d.dedup();
        d.len()
    };
    assert!(distinct >= 2, "A1: the released reply evolves the stream");
    waits::wait_state(&mut s, "Done · 3 files changed", "permission reply lands");
    assert_eq!(
        prompts(&live_text(&mut s)),
        3,
        "N1: all prompts back once idle"
    );
    eprintln!("pane stream: {distinct}/7 distinct frames on release");

    click_text(
        &mut s,
        "1,204.49",
        0,
        case_timeout(&case),
        "focus codex pane",
    );
    std::thread::sleep(Duration::from_millis(300));
    support::drive_with_timeout(&mut s, &["type:hi"], drive_ms(case_timeout(&case)));
    waits::wait_state(&mut s, "❯ hi", "typed `hi` echoed");
    support::press_step(&s, "enter");
    let mut saw_spinner = false;
    let mut codex = vec![live_text(&mut s)];
    for _ in 0..8 {
        std::thread::sleep(Duration::from_millis(300));
        let t = live_text(&mut s);
        let spins = spins_in(&t);
        assert!(spins <= 1, "N2: at most one spinner row: {spins}");
        saw_spinner |= spins > 0;
        let pc = prompts(&t);
        assert!(
            (3..=4).contains(&pc),
            "N1: prompt envelope holds: {pc}: {:?}",
            t.lines().filter(|l| l.contains("❯")).collect::<Vec<_>>()
        );
        codex.push(t);
    }
    assert!(saw_spinner, "V2: spinner shows while the reply lands");
    for t in &codex {
        for line in t.lines().filter(|l| is_spinner_line(l)) {
            let seg = line.split('│').find(|s| is_spinner_line(s)).unwrap_or("");
            let clean: String = seg.chars().filter(|c| *c != '┃').collect();
            let trimmed = clean.trim();
            assert!(
                trimmed.chars().next().is_some_and(|c| SPIN.contains(c))
                    && trimmed.chars().count() <= 16
                    && !trimmed.contains("❯"),
                "V2: spinner line stands alone: {trimmed:?}"
            );
        }
    }
    let landed = &codex[codex.len() - 1];
    assert_eq!(spins_in(landed), 0, "V2: the reply replaces the spinner");
    assert!(
        landed.contains("↳") && landed.contains("Done · 4 s · 1.1k tokens"),
        "A1: the codex reply lands whole"
    );

    assert_eq!(prompts(landed), 4, "N1: echo + prompt once idle");

    support::drive_with_timeout(&mut s, &["type:again"], drive_ms(case_timeout(&case)));
    waits::wait_state(&mut s, "❯ again", "typed `again` echoed");
    support::press_step(&s, "enter");

    waits::wait_state(&mut s, "Thinking…", "commit consumed");
    support::drive_with_timeout(&mut s, &["type:a"], drive_ms(case_timeout(&case)));
    std::thread::sleep(Duration::from_millis(300));
    let ahead = live_text(&mut s);

    assert!(
        (4..=5).contains(&prompts(&ahead)),
        "S2: prompt envelope holds: {}",
        prompts(&ahead)
    );
    assert!(
        seg_has(&ahead, "❯ a"),
        "S2: type-ahead echoed while working"
    );
    support::press_step(&s, "backspace");
    std::thread::sleep(Duration::from_millis(300));
    let unahead = live_text(&mut s);
    assert!(
        !seg_has(&unahead, "❯ a"),
        "S2: backspace removes the type-ahead"
    );
    waits::wait_state(&mut s, "Done · 6 s · 2.4k tokens", "second reply lands");
    assert_eq!(
        prompts(&live_text(&mut s)),
        4,
        "S2: lingering codex echo + prompts once idle"
    );

    click_text(
        &mut s,
        "MAX_ATTEMPTS",
        0,
        case_timeout(&case),
        "focus left pane",
    );
    std::thread::sleep(Duration::from_millis(300));
    let before_s1 = live_text(&mut s);
    let (lrow, lcol) = find_pos(&mut s, "MAX_ATTEMPTS", case_timeout(&case));
    wheel_at(&mut s, lcol, lrow, Wheel::Up, 10);
    std::thread::sleep(Duration::from_millis(300));
    assert_ne!(live_text(&mut s), before_s1, "S1: history retained");
    support::drive_with_timeout(&mut s, &["type:q"], drive_ms(case_timeout(&case)));
    std::thread::sleep(Duration::from_millis(300));
    let typed = live_text(&mut s);
    assert_eq!(
        prompts(&typed),
        4,
        "S2: type-ahead reuses the prompt: {:?}",
        typed
            .lines()
            .filter(|l| l.contains("❯"))
            .collect::<Vec<_>>()
    );
    assert!(
        seg_has(&typed, "❯ q"),
        "S2: type-ahead echoed on the live prompt"
    );
    support::press_step(&s, "backspace");
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        !seg_has(&live_text(&mut s), "❯ q"),
        "S2: backspace snaps and removes"
    );

    click_text(
        &mut s,
        "payments-platform ❯",
        0,
        case_timeout(&case),
        "focus shell",
    );
    std::thread::sleep(Duration::from_millis(300));
    support::drive_with_timeout(&mut s, &["type:ls"], drive_ms(case_timeout(&case)));
    waits::wait_state(&mut s, "payments-platform ❯ ls", "typed `ls` echoed");
    support::press_step(&s, "enter");
    std::thread::sleep(Duration::from_millis(400));
    let committed = live_text(&mut s);
    assert!(
        seg_has(&committed, "payments-platform ❯ ls"),
        "S2: shell commit echoes"
    );
    assert!(
        committed.contains("Cargo.toml   crates/"),
        "S2: commit snaps and runs"
    );
    assert_eq!(prompts(&committed), 5, "S2: shell echo joins the echoes");

    click_text(
        &mut s,
        "payments-platform ❯",
        0,
        case_timeout(&case),
        "focus pane",
    );
    let before_clear = live_text(&mut s);
    support::drive_with_timeout(&mut s, &["ctrl-b", "ctrl-l"], drive_ms(case_timeout(&case)));
    std::thread::sleep(Duration::from_millis(400));
    let cleared = live_text(&mut s);
    assert_ne!(cleared, before_clear, "A2: clear wipes the viewport");

    assert_eq!(prompts(&cleared), 4, "A2: shell bare, codex linger kept");
    assert!(
        !cleared
            .lines()
            .any(|l| l.trim() == "payments-platform ❯ ls"),
        "A2: shell echo wiped"
    );
    let cscreen = live_screen(&mut s);
    let ccur = cscreen.cursor();
    assert!(
        ccur.visible && support::row_text(&cscreen, ccur.y).contains("❯"),
        "A2: caret back on the live prompt"
    );
    support::drive_with_timeout(&mut s, &["type:z"], drive_ms(case_timeout(&case)));
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        live_text(&mut s)
            .lines()
            .any(|l| l.contains("❯") && l.contains('z')),
        "A2: cleared pane still accepts type-ahead"
    );
    checkpoint(&mut s, &dir, "02-cleared");
    eprintln!("j2 capsule_panes: prompts, caret, stream, snap-back, clear");
}
