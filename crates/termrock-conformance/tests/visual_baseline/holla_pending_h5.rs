use std::path::{Path, PathBuf};
use std::time::Duration;

use tuiscotti::tui::Wheel;
use tuiscotti::{Color as TColor, Frame, Rgb};

use crate::pointer::wheel_below;
use crate::support::Session;
use crate::support::state_waits as waits;
use crate::support::typed_input::Input;
use crate::support::{self, Case, Color, HOLLA};

const BOOT: &str = "holla❯";
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

fn h5_case_t(
    slug: &str,
    args: &'static [&'static str],
    boot: &'static str,
    timeout_ms: u64,
) -> Case {
    h5_case(slug, args, boot).timeout(timeout_ms)
}

fn h5_dir(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/tuiscotti/journeys")
        .join(test);
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("mkdir {}: {e}", dir.display()));
    dir
}

fn write_provenance(dir: &Path, case: &Case) {
    let subject =
        support::try_resolve_bin(case.bin).unwrap_or_else(|e| panic!("resolve {}: {e}", case.bin));
    let modified = subject
        .mtime_unix
        .map_or_else(|| "unknown".to_string(), |m| m.to_string());
    let argv = support::argv_for(case);
    let body = format!(
        "profile: tuiscotti-default\ntransport: pty\nport_of: eebe9efb62c9ab5f20676fcb2395c1586ec07c01\nbinary: {}\nsha256: {}\nsize_bytes: {}\nmodified_unix: {modified}\nargv: {}\n",
        subject.path.display(),
        subject.sha256,
        subject.len,
        argv.join(" "),
    );
    std::fs::write(dir.join("provenance.txt"), &body)
        .unwrap_or_else(|e| panic!("write {}: {e}", dir.join("provenance.txt").display()));
    eprintln!("h5 provenance: {body}");
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

fn live_text(s: &mut Session) -> String {
    let obs = s
        .inner
        .observe_now()
        .unwrap_or_else(|e| panic!("live sample failed: {e:#}"));
    support::screen_text(&obs.screen)
}

fn live_frame(s: &mut Session) -> Frame {
    s.snapshot()
        .unwrap_or_else(|e| panic!("live frame sample failed: {e:#}"))
}

fn wait_absent(s: &mut Session, needle: &str, what: &str) {
    support::wait_screen(
        s,
        support::DEFAULT_WAIT,
        &format!("{what}: `{needle}` never expired"),
        |screen| !support::screen_text(screen).contains(needle),
    );
}

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

fn assert_line_has(text: &str, needle: &str, also: &str, what: &str) {
    assert!(
        text.lines().any(|l| l.contains(needle) && l.contains(also)),
        "{what}: no line contains both `{needle}` and `{also}`"
    );
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

fn wheel_at(s: &mut Session, col: u16, row: u16, dir: Wheel, notches: u32) {
    for _ in 0..notches {
        Input::wheel(dir, col, row).send(s);
        std::thread::sleep(Duration::from_millis(120));
    }
}

fn resolve_rgb(c: TColor, what: &str) -> Rgb {
    match c {
        TColor::Rgb(rgb) => rgb,
        TColor::Indexed(n) => Rgb::from_indexed(n),
        TColor::Default => panic!("{what}: cell color is Default, cannot resolve"),
    }
}

fn lum(c: TColor, what: &str) -> u32 {
    let rgb = resolve_rgb(c, what);
    u32::from(rgb.r) * 2126 + u32::from(rgb.g) * 7152 + u32::from(rgb.b) * 722
}

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

fn assert_faded(edge: u32, interior: u32, what: &str) {
    assert!(
        edge < interior,
        "{what}: edge row must fade (edge lum {edge} < interior lum {interior})"
    );
}

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

    assert_line_has(&boot, "› just build", ".justfile here", "V1 row");

    assert!(boot.contains("make build"), "V2: no make target");

    assert_line_has(&boot, "In", "~/work/poly", "S1 world");

    support::drive_with_timeout(&mut s, &["type:yarn", "wait:yarn s00"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    assert!(res.contains("Results · 31"), "flow: no Results · 31");

    assert!(res.contains("scope ‹ here ›"), "S2: no here scope");

    assert!(!res.contains(PLACEHOLDER), "N1: placeholder stuck");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "yarn s00", "results cleared");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);

    assert!(!cleared.contains("yarn s00"), "N2: yarn rows stuck");
    support::drive_with_timeout(&mut s, &["type:yarn", "wait:yarn s00"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-results-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "› yarn dev", "package.json here", "row intact");
    eprintln!("h5 parity_task_sources: boot, filter roundtrip");
}

#[test]
#[ignore = "holla h5 check; run with --ignored"]
fn h5_parity_upgrade_managers() {
    const OUTDATED: &str = "Outdated Formulae · 2";
    let dir = h5_dir("h5_parity_upgrade_managers");
    let case = h5_case_t("parity_upgrade_managers", UPGRADERS, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    waits::wait_state(&mut s, "›Tasks", "explore settled");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    assert_line_has(&boot, "›Tasks", "Services", "V1 explore");

    assert!(
        boot.contains("everything used here is suggested above"),
        "V2: no suggested-above recent"
    );

    assert_line_has(&boot, "Scope", "here", "S1 scope");

    support::drive_with_timeout(
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
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-batch");
    let batch = live_text(&mut s);

    assert!(
        batch.contains("==> Outdated Formulae · 2"),
        "flow: no outdated line"
    );

    assert!(
        batch.contains("! Upgrade Homebrew packages · 4 ok · 1 failed · 0 cancelled"),
        "S2: no batch footer"
    );

    assert!(!batch.contains(PLACEHOLDER), "N1: placeholder stuck");

    support::press_step(&s, "r");
    wait_absent(&mut s, OUTDATED, "output restarted");
    support::drive_with_timeout(
        &mut s,
        &[&format!("wait:{OUTDATED}"), "wait:brew update · succeeded"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-batch-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "brew update · succeeded", "exit 0", "title intact");
    support::press_step(&s, "alt-0");
    wait_absent(&mut s, OUTDATED, "back to Here");
    waits::wait_state(&mut s, "scope ‹ here ›", "finder back");
    checkpoint(&mut s, &dir, "03-here");
    let here = live_text(&mut s);

    assert!(!here.contains(OUTDATED), "N2: report line stuck");
    eprintln!("h5 parity_upgrade_managers: boot, batch + restart roundtrip");
}

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

    assert_line_has(
        &boot,
        "› Inspect 12 GB",
        "12 GB generated artifacts",
        "V1 row",
    );

    assert_line_has(&boot, "Pull", "3 commits behind", "V2 row");

    assert_line_has(&boot, "In", "~/work/holla", "S1 world");

    assert_line_has(&boot, "main", "↓3 • 5", "S2 status");

    support::drive_with_timeout(&mut s, &["type:tests", "wait:Results ·"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    assert_line_has(&res, "› Run tests", "task", "flow row");

    assert!(!res.contains(PLACEHOLDER), "N1: placeholder stuck");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Results ·", "results cleared");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);

    assert!(!cleared.contains("Results ·"), "N2: results stuck");
    support::drive_with_timeout(&mut s, &["type:tests", "wait:Results ·"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-results-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "› Run tests", "task", "row intact");
    eprintln!("h5 audit_rust: boot, filter roundtrip");
}

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

    assert_line_has(
        &boot,
        "Upgrade everything · devbox",
        "8 steps · 8 included",
        "V1 header",
    );

    assert_line_has(&boot, STEP1, "sudo", "V2 cursor");

    assert_line_has(&boot, "Privilege", "sudo", "S1 privilege");

    assert_line_has(&boot, "State", "ready", "S2 state");

    support::drive_with_timeout(&mut s, &["down", &format!("wait:{STEP2}")], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-moved");
    let moved = live_text(&mut s);

    assert_line_has(&moved, STEP2, "sudo", "flow cursor");

    assert!(!moved.contains(STEP1), "N1: step-01 cursor stuck");

    support::drive_with_timeout(&mut s, &["up", &format!("wait:{STEP1}")], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-back");
    let back = live_text(&mut s);

    assert!(!back.contains(STEP2), "N2: step-02 cursor stuck");
    assert_line_has(&back, STEP1, "sudo", "cursor intact");
    eprintln!("h5 audit_upgrade: boot, cursor roundtrip");
}

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

    assert_line_has(&boot, "frontend dev ⠋ ×", "btm ○ ×", "V1 strip");

    assert_line_has(&boot, "frontend dev", "running · 3 s", "V2 running");

    assert_line_has(&boot, "In", "~/work/acme", "S1 world");

    assert!(boot.contains("4 running"), "S2: no running count");

    support::drive_with_timeout(&mut s, &["type:preview", "wait:Results ·"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    assert_line_has(&res, "› Deploy preview", "Current folder", "flow row");

    assert!(!res.contains(PLACEHOLDER), "N1: placeholder stuck");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Results ·", "results cleared");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);

    assert!(!cleared.contains("Results ·"), "N2: results stuck");
    support::drive_with_timeout(&mut s, &["type:preview", "wait:Results ·"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-results-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "› Deploy preview", "Current folder", "row intact");
    eprintln!("h5 concept_activities_multi: boot, filter roundtrip");
}

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

    assert!(
        boot.contains("scan complete · 31.8 GiB allocated · 35 entries · 3 selected (24.2 GiB)"),
        "V1: no scan meta"
    );

    assert_line_has(&boot, ALLOC, "noise folded", "V2 sort");

    assert_line_has(
        &boot,
        "Entries",
        "35 · hardlinks counted once",
        "S1 entries",
    );

    assert!(
        boot.contains("3 selected · 24.2 GiB"),
        "S2: no selection footer"
    );

    support::drive_with_timeout(&mut s, &["s", &format!("wait:{APPARENT}")], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-apparent");
    let apparent = live_text(&mut s);

    assert_line_has(&apparent, APPARENT, "noise folded", "flow sort");

    assert!(!apparent.contains(ALLOC), "N1: allocated order stuck");

    support::drive_with_timeout(&mut s, &["s", &format!("wait:{ALLOC}")], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-allocated");
    let back = live_text(&mut s);

    assert!(!back.contains(APPARENT), "N2: apparent order stuck");
    assert_line_has(&back, ALLOC, "noise folded", "sort intact");
    eprintln!("h5 concept_disk_cleanup: boot, sort toggle roundtrip");
}

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

    assert_line_has(&boot, "› Clean Docker completely", "destructive", "V1 row");

    assert!(boot.contains(RESULTS), "V2: no Results · 7");

    assert_line_has(&boot, "Scope", "on devbox", "S1 scope");

    assert_line_has(&boot, "Gate", "two gates", "S2 gate");

    support::drive_with_timeout(
        &mut s,
        &["escape", &format!("wait:{PLACEHOLDER}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-cleared");
    let cleared = live_text(&mut s);

    assert!(cleared.contains("Suggested"), "flow: no suggestions back");

    assert!(!cleared.contains(RESULTS), "N1: results stuck");

    support::drive_with_timeout(
        &mut s,
        &["type:docker clean", &format!("wait:{RESULTS}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-results-again");
    let back = live_text(&mut s);

    assert!(!back.contains(PLACEHOLDER), "N2: placeholder stuck");
    assert_line_has(
        &back,
        "› Clean Docker completely",
        "destructive",
        "row intact",
    );
    eprintln!("h5 concept_docker_cleanup: boot, clear/retype roundtrip");
}

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

    assert_line_has(
        &boot,
        "› Show system resources",
        "CPU 78% for 4 min",
        "V1 row",
    );

    assert!(boot.contains("Analyze disk usage"), "V2: no disk probe");

    assert_line_has(&boot, "Scope", "host", "S1 scope");

    assert_line_has(&boot, "Gate", "runs directly", "S2 gate");

    support::drive_with_timeout(&mut s, &["type:btm", "wait:Results ·"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    assert_line_has(&res, "› Open btm", "sustained pressure", "flow row");

    assert!(!res.contains(PLACEHOLDER), "N1: placeholder stuck");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Results ·", "results cleared");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);

    assert!(!cleared.contains("Results ·"), "N2: results stuck");
    support::drive_with_timeout(&mut s, &["type:btm", "wait:Results ·"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-results-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "› Open btm", "sustained pressure", "row intact");
    eprintln!("h5 concept_first_use: boot, filter roundtrip");
}

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

    assert_line_has(&boot, "› Analyze disk usage", "/ is 91% full", "V1 row");

    assert!(
        boot.contains("Children · 14 projects"),
        "V2: no child count"
    );

    assert!(
        boot.contains("docker, github unavailable"),
        "S1: no backend warning"
    );

    assert_line_has(&boot, "Scope", "here", "S2 scope");

    support::drive_with_timeout(&mut s, &["type:cleanup", "wait:Results ·"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    assert_line_has(&res, "› Review cleanup candidates", "8.0 GiB", "flow row");

    assert!(!res.contains(PLACEHOLDER), "N1: placeholder stuck");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Results ·", "results cleared");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);

    assert!(!cleared.contains("Results ·"), "N2: results stuck");
    support::drive_with_timeout(&mut s, &["type:cleanup", "wait:Results ·"], case.timeout_ms);
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

    assert!(
        boot.contains("Error: listen EADDRINUSE: address already in use :::5173"),
        "V1: no port error"
    );

    assert_line_has(&boot, TITLE, "exit 1", "V2 title");

    assert_line_has(
        &boot,
        "Find the process on port 5173",
        "Retry frontend dev",
        "S1 next",
    );

    assert!(
        boot.contains("frontend dev failed · exit 1"),
        "S2: no failure footer"
    );

    support::drive_with_timeout(
        &mut s,
        &["escape", &format!("wait:{PLACEHOLDER}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-here");
    let here = live_text(&mut s);

    assert!(here.contains("scope ‹ here ›"), "flow: no here scope");

    assert!(!here.contains(EADDR), "N1: port error stuck");

    support::drive_with_timeout(
        &mut s,
        &["alt-1", &format!("wait:{EADDR}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-output-again");
    let back = live_text(&mut s);

    assert!(!back.contains(PLACEHOLDER), "N2: placeholder stuck");
    assert_line_has(&back, TITLE, "exit 1", "title intact");
    eprintln!("h5 concept_launch_failure: boot, Here/jump roundtrip");
}

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

    assert_line_has(
        &boot,
        "› Inspect 3 GB",
        "3 GB generated artifacts",
        "V1 row",
    );

    assert!(
        boot.contains("From acme · the parent root"),
        "V2: no parent shelf"
    );

    assert_line_has(&boot, "In", "~/work/acme/apps", "S1 world");

    assert_line_has(&boot, "Scope", "here", "S2 scope");

    support::drive_with_timeout(&mut s, &["type:test", "wait:Results · 17"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    assert_line_has(&res, "› Run tests", "task", "flow row");

    assert!(!res.contains(PLACEHOLDER), "N1: placeholder stuck");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Results · 17", "results cleared");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);

    assert!(!cleared.contains("Results · 17"), "N2: results stuck");
    support::drive_with_timeout(&mut s, &["type:test", "wait:Results · 17"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-results-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "› Run tests", "task", "row intact");
    eprintln!("h5 concept_monorepo_child: boot, filter roundtrip");
}

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

    assert_line_has(&boot, "› Deploy preview", "unreviewed", "V1 row");

    assert!(boot.contains("Children · 3 projects"), "V2: no child count");

    assert_line_has(&boot, "Gate", "trust required", "S1 gate");

    assert_line_has(&boot, "Scope", "here", "S2 scope");

    support::drive_with_timeout(
        &mut s,
        &["ctrl-down", "wait:Children · 3 projects"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-children");
    let children = live_text(&mut s);

    assert!(
        children.contains("Children · 3 projects · 11 entries"),
        "flow: no children header"
    );

    assert!(!children.contains("scope ‹ here ›"), "N1: here scope stuck");

    support::drive_with_timeout(&mut s, &["ctrl-up", "wait:scope ‹ here ›"], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-here");
    let here = live_text(&mut s);

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
    assert!(
        back.contains("Children · 3 projects · 11 entries"),
        "header intact"
    );
    eprintln!("h5 concept_monorepo_root: boot, scope roundtrip");
}

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

    assert_line_has(
        &boot,
        "› Follow payments-worker",
        "unit is restarting",
        "V1 row",
    );

    assert!(boot.contains("System · prod-eu-1"), "V2: no host shelf");

    assert_line_has(&boot, "Host", "prod-eu-1", "S1 host");

    assert_line_has(&boot, "Scope", "on prod-eu-1", "S2 scope");

    support::drive_with_timeout(
        &mut s,
        &["type:restart payments", "wait:Results · 2"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    assert_line_has(&res, "› Restart payments", "privileged", "flow row");

    assert!(!res.contains(PLACEHOLDER), "N1: placeholder stuck");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Results · 2", "results cleared");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);

    assert!(!cleared.contains("Results · 2"), "N2: results stuck");
    support::drive_with_timeout(
        &mut s,
        &["type:restart payments", "wait:Results · 2"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "03-results-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "› Restart payments", "privileged", "row intact");
    eprintln!("h5 concept_remote_host: boot, filter roundtrip");
}

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

#[test]
#[ignore = "holla h5 check; run with --ignored"]
fn h5_fade_browser_wheel() {
    const PREVIEW_COLS: (u16, u16) = (70, 115);
    let dir = h5_dir("h5_fade_browser_wheel");
    let case = h5_case("fade_browser_wheel", BROWSER, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::drive_with_timeout(
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
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "00-listing");
    let pre = live_text(&mut s);
    let pre_frame = live_frame(&mut s);
    assert!(pre.contains("16 entries"), "pre: no 16 entries");
    assert!(pre.contains("line 5"), "pre: no line 5 target");
    assert!(!pre.contains("line 33"), "pre: already scrolled");
    let (header_row, _) = find_pos(&mut s, "first 2000 lines", case_timeout(&case));

    let (wrow, wcol) = wheel_over(&mut s, "line 5", Wheel::Down, 2, case_timeout(&case));
    let post = wait_changed(&mut s, &pre, case_timeout(&case), "preview scrolled");
    checkpoint(&mut s, &dir, "01-scrolled");
    let frame = live_frame(&mut s);

    assert!(post.contains("7 line 6"), "V1: no scrolled top line");
    assert!(post.contains("line 33"), "V1: no surfaced bottom line");
    assert!(post.contains("first 2000 lines"), "V1: no preview title");

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

    assert_line_has(&post, "▎", "big.log", "S1 cursor");

    assert!(
        post.contains("16 entries · 2 hidden hidden"),
        "S2: no listing meta"
    );

    assert_row_same(&pre_frame, &frame, header_row, "N1 header never fades");

    wheel_at(&mut s, wcol, wrow, Wheel::Up, 2);
    let top = wait_text_eq(&mut s, &pre, case_timeout(&case), "back to top");
    checkpoint(&mut s, &dir, "02-back-top");

    assert!(!top.contains("line 33"), "N2: surfaced line stuck");
    assert!(top.contains("line 5"), "top target back");
    eprintln!("h5 fade_browser_wheel: scroll + fade, back to top");
}

#[test]
#[ignore = "holla h5 check; run with --ignored"]
fn h5_fade_cleanup_list_wheel() {
    const LIST_COLS: (u16, u16) = (2, 71);
    let dir = h5_dir("h5_fade_cleanup_list_wheel");
    let case = h5_case("fade_cleanup_list_wheel", INSIGHTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::drive_with_timeout(
        &mut s,
        &[
            "type:Review cleanup candidates",
            "enter",
            "wait:18 categories",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "00-review");
    let pre = live_text(&mut s);
    let pre_frame = live_frame(&mut s);
    assert!(pre.contains("18 categories"), "pre: no 18 categories");
    assert!(pre.contains("Yarn cache"), "pre: no Yarn cache target");
    assert!(!pre.contains("JetBrains logs"), "pre: already scrolled");
    let (header_row, _) = find_pos(&mut s, "Cleanup review", case_timeout(&case));

    wheel_below(&mut s, "Yarn cache", 2);
    let post = wait_changed(&mut s, &pre, case_timeout(&case), "list scrolled");
    checkpoint(&mut s, &dir, "01-scrolled");
    let frame = live_frame(&mut s);

    assert!(post.contains("18 categories"), "V1: no categories meta");
    assert!(post.contains("JetBrains logs"), "V1: no surfaced category");

    let (edge_row, _) = find_pos(&mut s, "Xcode archives", case_timeout(&case));
    let (inner_row, _) = find_pos(&mut s, "Maven repository", case_timeout(&case));
    let edge = row_fg_median_cols(&frame, edge_row, LIST_COLS.0, LIST_COLS.1, "V2 edge");
    let inner = row_fg_median_cols(&frame, inner_row, LIST_COLS.0, LIST_COLS.1, "V2 interior");
    eprintln!("h5 cleanup_list_wheel fade: edge lum {edge}, interior lum {inner}");
    assert_faded(edge, inner, "V2 top edge");
    assert_line_has(&post, "Yarn cache", "┃", "V2 thumb");

    assert!(
        post.contains("19 selected · 12.4 GiB"),
        "S1: no selection count"
    );

    assert!(post.contains("Xcode DerivedData"), "S2: no detail category");

    assert_row_same(&pre_frame, &frame, header_row, "N1 title never fades");

    let (wrow, wcol) = find_pos(&mut s, "Yarn cache", case_timeout(&case));
    wheel_at(&mut s, wcol, wrow + 1, Wheel::Up, 2);
    let top = wait_text_eq(&mut s, &pre, case_timeout(&case), "back to top");
    checkpoint(&mut s, &dir, "02-back-top");

    assert!(!top.contains("JetBrains logs"), "N2: surfaced row stuck");
    assert!(top.contains("Xcode device support"), "top rows back");
    eprintln!("h5 fade_cleanup_list_wheel: scroll + fade, back to top");
}

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

    waits::wait_state(&mut s, "› Status across 3 child", "top row settled");
    waits::wait_state(&mut s, "Emit a mixed stream", "suggestions settled");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    assert_line_has(
        &boot,
        "› Status across 3 child",
        "0 dirty · 1 behind",
        "V1 row",
    );

    assert!(boot.contains("Emit a mixed stream"), "V2: no burst shelf");

    assert_line_has(&boot, "In", "~/work/batch", "S1 world");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Emit a burst",
            "enter",
            "wait:exit 0",
            "wait:500 earlier lines dropped",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-tail");
    let tail = live_text(&mut s);

    assert!(
        tail.contains("500 earlier lines dropped"),
        "V1-end: no drop report"
    );
    assert!(tail.contains("last 4000 kept"), "V1-end: no kept count");

    assert!(tail.contains(TAIL), "V2-end: no last burst line");
    assert!(
        tail.contains("burst line 4472"),
        "V2-end: no first tail line"
    );

    assert_line_has(&tail, "emit a burst · succeeded", "exit 0", "S2 title");

    assert!(!tail.contains(PLACEHOLDER), "N1: placeholder stuck");

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

    assert!(!here.contains("burst line"), "N2: burst body stuck");
    eprintln!("h5 fade_executor_burst_end: boot, burst + restart roundtrip");
}

#[test]
#[ignore = "holla h5 check; run with --ignored"]
fn h5_fade_trust_body_wheel() {
    const BODY_COLS: (u16, u16) = (2, 115);
    const TRUST: &str = "Trust ~/work/team/.holla.toml?";
    let dir = h5_dir("h5_fade_trust_body_wheel");
    let case = h5_case("fade_trust_body_wheel", CUSTOM, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::drive_with_timeout(
        &mut s,
        &["type:Deploy preview", "enter", &format!("wait:{TRUST}")],
        case.timeout_ms,
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

    assert!(post.contains("id = \"shared\""), "V1: no shared action row");
    assert!(post.contains("Definition"), "V1: no definition title");

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

    assert_line_has(&post, "Defines", "3 custom actions", "S1 defines");

    assert_line_has(&post, "▎Cancel", "Trust this file", "S2 choice");

    assert_row_same(&pre_frame, &frame, header_row, "N1 title never fades");

    wheel_at(&mut s, wcol, wrow, Wheel::Up, 2);
    let top = wait_text_eq(&mut s, &pre, case_timeout(&case), "back to top");
    checkpoint(&mut s, &dir, "02-back-top");

    assert!(!top.contains("id = \"shared\""), "N2: shared row stuck");
    assert!(top.contains("id = \"deploy.preview\""), "top target back");
    eprintln!("h5 fade_trust_body_wheel: scroll + fade, back to top");
}

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

    support::drive_with_timeout(
        &mut s,
        &["ctrl-g", &format!("wait:{EMPTY}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-picker");
    let picker = live_text(&mut s);

    assert_line_has(&picker, "Activities", "0 live", "V1 title");

    assert!(picker.contains(EMPTY), "V2: no empty state");

    assert!(
        picker.contains("↑↓ Move  Enter Switch  Esc Cancel"),
        "S1: no picker footer"
    );

    assert!(
        picker.contains("Show system resources"),
        "S2: no shelf behind"
    );

    assert!(!picker.contains(FOCUSED_FIND), "N1: finder focus stuck");

    support::press_step(&s, "escape");
    wait_absent(&mut s, EMPTY, "picker cancelled");
    waits::wait_state(&mut s, FOCUSED_FIND, "finder back");
    checkpoint(&mut s, &dir, "02-finder");
    let finder = live_text(&mut s);

    assert!(!finder.contains("0 live"), "N2: live count stuck");
    support::drive_with_timeout(
        &mut s,
        &["ctrl-g", &format!("wait:{EMPTY}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "03-picker-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "Activities", "0 live", "picker intact");
    eprintln!("h5 activities_empty: picker, cancel roundtrip");
}

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

    assert_line_has(&boot, "frontend dev ⠋ ×", "btm ○ ×", "V1 strip");

    assert_line_has(&boot, "› Deploy preview", "Current folder", "V2 cursor");

    assert!(boot.contains("4 running"), "S1: no running count");

    assert_line_has(&boot, "Gate", "trust required", "S2 gate");

    support::drive_with_timeout(
        &mut s,
        &["alt-1", &format!("wait:{TITLE}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-output");
    let output = live_text(&mut s);

    assert!(output.contains(VITE), "flow: no vite ready line");

    assert_line_has(&output, TITLE, "3 s", "flow title");

    assert!(!output.contains(PLACEHOLDER), "N1: placeholder stuck");

    support::press_step(&s, "escape");
    wait_absent(&mut s, TITLE, "back to Here");
    waits::wait_state(&mut s, PLACEHOLDER, "finder back");
    checkpoint(&mut s, &dir, "02-here");
    let here = live_text(&mut s);

    assert!(!here.contains(VITE), "N2: vite line stuck");
    support::drive_with_timeout(
        &mut s,
        &["alt-1", &format!("wait:{TITLE}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "03-output-again");
    let back = live_text(&mut s);
    assert_line_has(&back, TITLE, "3 s", "title intact");
    eprintln!("h5 activities_jump: boot, Here/jump roundtrip");
}

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

    assert_line_has(&boot, "frontend dev ⠋ ×", "btm ○ ×", "V1 strip");

    assert_line_has(&boot, "› Deploy preview", "Current folder", "V2 cursor");

    assert!(boot.contains("4 running"), "S1: no running count");

    support::drive_with_timeout(
        &mut s,
        &["ctrl-g", &format!("wait:{LIVE4}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-picker");
    let picker = live_text(&mut s);

    assert_line_has(&picker, "Activities", LIVE4, "flow title");

    assert_line_has(
        &picker,
        "⠋ frontend dev",
        "running · child · 3 s",
        "flow row",
    );

    assert_line_has(&picker, "○ btm", "detached · host · 3 s", "S2 detached");

    assert!(!picker.contains(FOCUSED_FIND), "N1: finder focus stuck");

    support::press_step(&s, "escape");
    wait_absent(&mut s, LIVE4, "picker cancelled");
    waits::wait_state(&mut s, FOCUSED_FIND, "finder back");
    checkpoint(&mut s, &dir, "02-finder");
    let finder = live_text(&mut s);

    assert!(
        !finder.contains("running · child · 3 s"),
        "N2: picker row stuck"
    );
    support::drive_with_timeout(
        &mut s,
        &["ctrl-g", &format!("wait:{LIVE4}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "03-picker-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "Activities", LIVE4, "picker intact");
    eprintln!("h5 activities_overlay: picker, cancel roundtrip");
}
