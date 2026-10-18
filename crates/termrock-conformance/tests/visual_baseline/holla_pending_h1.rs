use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::support::Session;

use crate::support::state_waits as waits;
use crate::support::{self, Case, Color, HOLLA};

const BOOT: &str = "holla❯";

const CARGO: &[&str] = &[
    "--scenario",
    "parity-cargo",
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
const PLAN80: &[&str] = &[
    "--scenario",
    "disk-cleanup",
    "--motion",
    "paused",
    "--frame",
    "80",
];
const DELSAFE: &[&str] = &["--scenario", "parity-delete-safety", "--motion", "reduced"];
const HIST_PAUSED: &[&str] = &[
    "--scenario",
    "parity-cleanup-results",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const HIST_REDUCED: &[&str] = &[
    "--scenario",
    "parity-cleanup-results",
    "--motion",
    "reduced",
];
const LINUX: &[&str] = &[
    "--scenario",
    "parity-platforms-linux",
    "--motion",
    "reduced",
];
const SCAN: &[&str] = &["--scenario", "parity-disk-scan", "--motion", "reduced"];
const NAV: &[&str] = &[
    "--scenario",
    "parity-disk-navigation",
    "--motion",
    "reduced",
];
const DOCKER_PAUSED: &[&str] = &[
    "--scenario",
    "docker-cleanup",
    "--motion",
    "paused",
    "--frame",
    "40",
];
const DOCKER_REDUCED: &[&str] = &["--scenario", "docker-cleanup", "--motion", "reduced"];
const PDOCKER_RED: &[&str] = &["--scenario", "parity-docker", "--motion", "reduced"];
const PDOCKER_PAUSED: &[&str] = &[
    "--scenario",
    "parity-docker",
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
const IDEA: &[&str] = &[
    "--scenario",
    "parity-idea",
    "--motion",
    "paused",
    "--frame",
    "40",
];

const GATE2: &str = "gate 2 of 2";
const GATE1: &str = "gate 1 of 2";

fn h1_case(slug: &str, args: &'static [&'static str], boot: &'static str) -> Case {
    Case::dynamic(
        format!("journeys/holla/h1/{slug}"),
        HOLLA,
        args,
        120,
        40,
        Color::Truecolor,
        boot,
    )
}

fn h1_case_t(
    slug: &str,
    args: &'static [&'static str],
    boot: &'static str,
    timeout_ms: u64,
) -> Case {
    h1_case(slug, args, boot).timeout(timeout_ms)
}

fn h1_dir(test: &str) -> PathBuf {
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
        "profile: tuiscotti-default\ntransport: pty\nport_of: e740009d378f2ace27b8df20870c3177005a9230\nbinary: {}\nsha256: {}\nsize_bytes: {}\nmodified_unix: {modified}\nargv: {}\n",
        subject.path.display(),
        subject.sha256,
        subject.len,
        argv.join(" "),
    );
    std::fs::write(dir.join("provenance.txt"), &body)
        .unwrap_or_else(|e| panic!("write {}: {e}", dir.join("provenance.txt").display()));
    eprintln!("h1 provenance: {body}");
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

fn assert_line_has(text: &str, needle: &str, also: &str, what: &str) {
    assert!(
        text.lines().any(|l| l.contains(needle) && l.contains(also)),
        "{what}: no line contains both `{needle}` and `{also}`"
    );
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cargo_clean() {
    const DIALOG_FOOTER: &str = "← → Choose  Enter Confirm  Esc Cancel";
    let dir = h1_dir("h1_cargo_clean");
    let case = h1_case("cargo_clean", CARGO, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["type:cargo clean", "wait:900.0 MiB"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-clean");
    let clean = live_text(&mut s);

    assert_line_has(&clean, "› cargo clean · 900.0 MiB", "destructive", "V1 row");
    assert!(clean.contains("Results · 3"), "V1: no Results · 3");

    assert!(
        clean.contains("remove the resolved"),
        "V2: no resolved-target fact"
    );
    assert!(clean.contains("~/work/engine/target"), "V2: no target path");

    assert_line_has(&clean, "Enter Confirm…", "Alt+Enter More", "S1 footer");

    assert_line_has(&clean, "Gate", "one confirmation", "S2 gate");

    assert!(
        !clean.contains(DIALOG_FOOTER),
        "N1: dialog open before Enter"
    );

    support::drive_with_timeout(
        &mut s,
        &["enter", &format!("wait:{DIALOG_FOOTER}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-dialog-presence");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DIALOG_FOOTER, "dialog cancelled");
    waits::wait_state(&mut s, "900.0 MiB", "clean state back");

    support::drive_with_timeout(
        &mut s,
        &["enter", &format!("wait:{DIALOG_FOOTER}")],
        case.timeout_ms,
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DIALOG_FOOTER, "dialog cancelled again");
    let back = live_text(&mut s);

    assert!(!back.contains(DIALOG_FOOTER), "N2: dialog stuck after Esc");
    assert_line_has(
        &back,
        "› cargo clean · 900.0 MiB",
        "destructive",
        "row intact",
    );
    checkpoint(&mut s, &dir, "03-clean-again");
    eprintln!("h1 cargo_clean: row, dialog roundtrip x2");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cargo_clean_confirm() {
    const DIALOG_FOOTER: &str = "← → Choose  Enter Confirm  Esc Cancel";
    let dir = h1_dir("h1_cargo_clean_confirm");
    let case = h1_case("cargo_clean_confirm", CARGO, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:cargo clean",
            "enter",
            &format!("wait:{DIALOG_FOOTER}"),
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-dialog");
    let dialog = live_text(&mut s);

    assert_line_has(
        &dialog,
        "Action",
        "remove the resolved target directory",
        "V1 action",
    );

    assert_line_has(&dialog, "▎Cancel", "Run", "V2 choice");

    assert!(dialog.contains(DIALOG_FOOTER), "S1: no dialog footer");

    assert_line_has(
        &dialog,
        "Confirmation",
        "one explicit confirmation",
        "S2 confirmation",
    );

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DIALOG_FOOTER, "dialog cancelled");
    waits::wait_state(&mut s, "900.0 MiB", "clean row back");
    checkpoint(&mut s, &dir, "02-cancelled");
    let cancelled = live_text(&mut s);

    assert!(
        !cancelled.contains(DIALOG_FOOTER),
        "N1: dialog footer stuck"
    );
    assert!(
        !cancelled.contains("one explicit confirmation"),
        "N1: confirmation fact stuck"
    );

    support::drive_with_timeout(
        &mut s,
        &["enter", &format!("wait:{DIALOG_FOOTER}")],
        case.timeout_ms,
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, DIALOG_FOOTER, "dialog cancelled again");
    let back = live_text(&mut s);

    assert!(!back.contains(DIALOG_FOOTER), "N2: dialog stuck again");
    assert_line_has(
        &back,
        "› cargo clean · 900.0 MiB",
        "destructive",
        "row intact",
    );
    checkpoint(&mut s, &dir, "03-cancelled-again");
    eprintln!("h1 cargo_clean_confirm: dialog, cancel roundtrip x2");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cleanup_artifacts() {
    let dir = h1_dir("h1_cleanup_artifacts");
    let case = h1_case("cleanup_artifacts", INSIGHTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Review cleanup candidates",
            "enter",
            "wait:18 categories",
            "end",
            "wait:Project artifacts",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-artifacts");
    let arts = live_text(&mut s);

    assert_line_has(&arts, "▎[ ] ▸ Project artifacts", "2.5 GiB", "V1 cursor");

    assert!(
        arts.contains("14 of 14 candidates eligible"),
        "V2: no eligibility fact"
    );

    assert_line_has(
        &arts,
        "18 categories",
        "19 selected (12.4 GiB)",
        "S1 header",
    );

    assert!(arts.contains("d Review deletion"), "S2: no d hint");

    assert!(
        !arts.contains("▾ Project artifacts"),
        "N1: fold open before Right"
    );

    support::press_step(&s, "up");
    waits::wait_state(&mut s, "▎[✓] ▸ JetBrains logs", "cursor up");
    support::press_step(&s, "down");
    waits::wait_state(&mut s, "▎[ ] ▸ Project artifacts", "cursor back");

    support::press_step(&s, "right");
    waits::wait_state(&mut s, "▾ Project artifacts", "fold opened");
    checkpoint(&mut s, &dir, "02-fold-presence");
    support::press_step(&s, "left");
    waits::wait_gone(&mut s, "▾ Project artifacts", "fold closed");
    waits::wait_state(&mut s, "▎[ ] ▸ Project artifacts", "cursor intact");
    let back = live_text(&mut s);

    assert!(!back.contains("▾ Project artifacts"), "N2: fold stuck open");
    checkpoint(&mut s, &dir, "03-fold-closed");
    eprintln!("h1 cleanup_artifacts: last row, cursor + fold roundtrips");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cleanup_categories() {
    let dir = h1_dir("h1_cleanup_categories");
    let case = h1_case("cleanup_categories", INSIGHTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Review cleanup candidates",
            "enter",
            "wait:18 categories",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-categories");
    let cats = live_text(&mut s);

    assert_line_has(&cats, "18 categories", "18.8 GiB", "V1 header");

    assert_line_has(
        &cats,
        "▎[–] ▸ Xcode DerivedData",
        "810.0 MiB",
        "V2 first row",
    );

    assert_line_has(&cats, "Safety", "rebuilt on demand", "S1 detail");

    assert_line_has(&cats, "m Permanent", "n Dry run", "S2 footer");

    assert!(
        !cats.contains("▾ Xcode DerivedData"),
        "N1: fold open before Right"
    );

    support::press_step(&s, "down");
    waits::wait_state(&mut s, "▎[✓] ▸ Xcode device support", "cursor down");
    support::press_step(&s, "up");
    waits::wait_state(&mut s, "▎[–] ▸ Xcode DerivedData", "cursor back");

    support::press_step(&s, "right");
    waits::wait_state(&mut s, "▾ Xcode DerivedData", "fold opened");
    checkpoint(&mut s, &dir, "02-fold-presence");
    support::press_step(&s, "left");
    waits::wait_gone(&mut s, "▾ Xcode DerivedData", "fold closed");
    waits::wait_state(&mut s, "▎[–] ▸ Xcode DerivedData", "cursor intact");
    let back = live_text(&mut s);

    assert!(!back.contains("▾ Xcode DerivedData"), "N2: fold stuck open");
    checkpoint(&mut s, &dir, "03-fold-closed");
    eprintln!("h1 cleanup_categories: landing, cursor + fold roundtrips");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cleanup_derived_data() {
    let dir = h1_dir("h1_cleanup_derived_data");
    let case = h1_case("cleanup_derived_data", INSIGHTS, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Review cleanup candidates",
            "enter",
            "wait:18 categories",
            "right",
            "wait:▾ Xcode DerivedData",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-derived");
    let open = live_text(&mut s);

    assert_line_has(&open, "▾ Xcode DerivedData", "810.0 MiB", "V1 fold");

    assert_line_has(
        &open,
        "~/Library/Developer/Xcode/D…",
        "30 d",
        "V2 candidate",
    );

    assert_line_has(&open, "Guard", "Xcode is running", "S1 guard");

    assert!(open.contains("d Review deletion"), "S2: no d hint");

    support::press_step(&s, "left");
    waits::wait_gone(&mut s, "▾ Xcode DerivedData", "fold closed");
    waits::wait_state(&mut s, "▸ Xcode DerivedData", "closed marker back");
    checkpoint(&mut s, &dir, "02-closed");
    let closed = live_text(&mut s);

    assert!(
        !closed.contains("▾ Xcode DerivedData"),
        "N1: fold stuck open"
    );
    assert!(
        !closed.contains("~/Library/Developer/Xcode/D…"),
        "N1: candidate row stuck"
    );

    support::press_step(&s, "right");
    waits::wait_state(&mut s, "▾ Xcode DerivedData", "fold reopened");
    let again = live_text(&mut s);

    assert!(
        !again.contains("▸ Xcode DerivedData"),
        "N2: closed marker stuck"
    );
    assert_line_has(&again, "▾ Xcode DerivedData", "810.0 MiB", "fold intact");
    checkpoint(&mut s, &dir, "03-reopened");
    eprintln!("h1 cleanup_derived_data: fold, close roundtrip, reopen");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cleanup_gate_seq() {
    let dir = h1_dir("h1_cleanup_gate_seq");
    let case = h1_case("cleanup_gate_seq", PLAN80, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(&mut s, &["c", "wait:4 steps · 4 included"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-plan");
    support::drive_with_timeout(&mut s, &["c", &format!("wait:{GATE1}")], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-gate1");
    let gate = live_text(&mut s);

    assert_line_has(
        &gate,
        "Review · Clean developer artifacts under ~/work",
        GATE1,
        "V1 title",
    );

    assert_line_has(&gate, "Sequence · 4 commands", "y copies", "V2 sequence");
    assert_line_has(
        &gate,
        "cargo clean",
        "/Users/alex/work/backend",
        "V2 sequence row",
    );

    assert_line_has(&gate, "y Copy sequence", "Esc Cancel", "S1 footer");

    assert!(
        gate.contains("review · gate 1 of 2"),
        "S2: no review status"
    );

    assert!(!gate.contains(GATE2), "N1: gate 2 open before Right+Enter");

    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "03-gate2-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled");
    waits::wait_state(&mut s, GATE1, "gate 1 back");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE1, "gate 1 left");
    waits::wait_state(&mut s, "4 steps · 4 included", "plan back");
    checkpoint(&mut s, &dir, "04-plan");
    let back = live_text(&mut s);

    assert!(!back.contains(GATE1), "N2: gate 1 stuck");
    assert!(!back.contains(GATE2), "N2: gate 2 stuck");
    assert_line_has(
        &back,
        "▎[✓] 01 Trash frontend › node_modules",
        "optional",
        "plan intact",
    );
    eprintln!("h1 cleanup_gate_seq: gate 1, gate-2 probe, plan back");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cleanup_gate1() {
    let dir = h1_dir("h1_cleanup_gate1");
    let case = h1_case_t("cleanup_gate1", DELSAFE, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Analyze disk usage",
            "enter",
            "wait:1 unreadable",
            "down",
            "space",
            "d",
            &format!("wait:{GATE1}"),
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-gate1");
    let gate = live_text(&mut s);

    assert_line_has(&gate, "Review · trash 1 item", GATE1, "V1 title");

    assert_line_has(&gate, "Revision", "68c21e8eb8ed0a71", "V2 revision");

    assert_line_has(
        &gate,
        "Recoverable",
        "yes · until the Trash is emptied",
        "S1 recovery",
    );

    assert!(
        gate.contains("1 item selected · 1000.0 MiB estimated"),
        "S2: no selection estimate"
    );

    assert!(!gate.contains(GATE2), "N1: gate 2 open before Right+Enter");

    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-gate2-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled");
    waits::wait_state(&mut s, GATE1, "gate 1 back");

    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled again");
    let back = live_text(&mut s);

    assert!(!back.contains(GATE2), "N2: gate 2 stuck");
    assert!(!back.contains("to confirm"), "N2: phrase prompt stuck");
    assert_line_has(&back, "Review · trash 1 item", GATE1, "gate 1 intact");
    checkpoint(&mut s, &dir, "03-gate1-again");
    eprintln!("h1 cleanup_gate1: gate 1, gate-2 roundtrip x2");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cleanup_gate2() {
    const PHRASE: &str = "Type TRASH 1 UNDER /Users/alex/Projects/safe ON mbp to confirm";
    let dir = h1_dir("h1_cleanup_gate2");
    let case = h1_case_t("cleanup_gate2", DELSAFE, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Analyze disk usage",
            "enter",
            "wait:1 unreadable",
            "down",
            "space",
            "d",
            &format!("wait:{GATE1}"),
            "right",
            "enter",
            &format!("wait:{GATE2}"),
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-gate2");
    let gate = live_text(&mut s);

    assert!(
        gate.contains("Trash 1 items · gate 2 of 2"),
        "V1: no gate-2 title"
    );

    assert!(gate.contains(PHRASE), "V2: no phrase prompt");

    assert_line_has(&gate, "Enter Confirm", "Esc Cancel", "S1 footer");

    assert_line_has(&gate, "Cancel", "Execute", "S2 choice");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled");
    waits::wait_state(&mut s, GATE1, "gate 1 back");
    checkpoint(&mut s, &dir, "02-cancelled");
    let cancelled = live_text(&mut s);

    assert!(!cancelled.contains(GATE2), "N1: gate 2 stuck");
    assert!(!cancelled.contains("to confirm"), "N1: phrase prompt stuck");

    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled again");
    let back = live_text(&mut s);

    assert!(!back.contains(GATE2), "N2: gate 2 stuck again");
    assert_line_has(&back, "Review · trash 1 item", GATE1, "gate 1 intact");
    checkpoint(&mut s, &dir, "03-cancelled-again");
    eprintln!("h1 cleanup_gate2: gate 2, cancel roundtrip x2");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cleanup_gate2_typed() {
    const PHRASE: &str = "TRASH 1 UNDER /Users/alex/Projects/safe ON mbp";
    let dir = h1_dir("h1_cleanup_gate2_typed");
    let case = h1_case_t("cleanup_gate2_typed", DELSAFE, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Analyze disk usage",
            "enter",
            "wait:1 unreadable",
            "down",
            "space",
            "d",
            &format!("wait:{GATE1}"),
            "right",
            "enter",
            &format!("wait:{GATE2}"),
            "enter",
            &format!("type:{PHRASE}"),
        ],
        case.timeout_ms,
    );
    waits::wait_state(&mut s, &format!("▎ {PHRASE}"), "phrase typed");
    checkpoint(&mut s, &dir, "01-typed");
    let typed = live_text(&mut s);

    assert!(
        typed.contains(&format!("▎ {PHRASE}")),
        "V1: no typed phrase"
    );

    assert!(
        typed.contains("Trash 1 items · gate 2 of 2"),
        "V2: no gate-2 title"
    );

    assert_line_has(&typed, "EDIT", "Type Phrase", "S1 edit footer");

    assert!(typed.contains(GATE1), "S2: no gate-1 status");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, &format!("▎ {PHRASE}"), "phrase cleared");
    waits::wait_state(&mut s, GATE2, "dialog still open");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);

    assert!(
        !cleared.contains(&format!("▎ {PHRASE}")),
        "N1: typed phrase stuck"
    );
    assert!(cleared.contains("to confirm"), "N1: prompt lost");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "dialog cancelled");
    waits::wait_state(&mut s, GATE1, "gate 1 back");
    checkpoint(&mut s, &dir, "03-cancelled");
    let back = live_text(&mut s);

    assert!(!back.contains(GATE2), "N2: gate 2 stuck");
    assert!(!back.contains("to confirm"), "N2: phrase prompt stuck");
    eprintln!("h1 cleanup_gate2_typed: typed, cleared, cancelled");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cleanup_history() {
    let dir = h1_dir("h1_cleanup_history");
    let case = h1_case("cleanup_history", HIST_PAUSED, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Show cleanup history",
            "enter",
            "wait:operation log records",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-history");
    let hist = live_text(&mut s);

    assert_line_has(&hist, "Entries", "1 operation log records", "V1 header");

    assert_line_has(&hist, "2026-08-29 08:41", "trashed", "V2 record");

    assert_line_has(&hist, "Log", "~/.cache/holla/ops.log", "S1 log");

    assert_line_has(&hist, "↑↓ Scroll", "Esc Back", "S2 footer");

    assert!(
        !boot.contains("operation log records"),
        "N1: history visible at boot"
    );

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "operation log records", "history left");
    checkpoint(&mut s, &dir, "02-finder");
    let left = live_text(&mut s);

    assert!(!left.contains("operation log records"), "N2: history stuck");
    assert!(!left.contains("Cleanup history"), "N2: title stuck");

    support::drive_with_timeout(
        &mut s,
        &["enter", "wait:operation log records"],
        case.timeout_ms,
    );
    waits::wait_state(&mut s, "2026-08-29 08:41", "record back");
    checkpoint(&mut s, &dir, "03-history-again");
    eprintln!("h1 cleanup_history: history, leave, reopen");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cleanup_linux_report() {
    const PHRASE: &str = "TRASH 1 UNDER /home/alex ON devbox";
    let dir = h1_dir("h1_cleanup_linux_report");
    let case = h1_case_t("cleanup_linux_report", LINUX, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);

    support::drive_with_timeout(
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
            &format!("wait:Type {PHRASE}"),
            "enter",
            &format!("type:{PHRASE}"),
            "enter",
            "tab",
            "enter",
            "wait:Cleanup report",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-report");
    let rep = live_text(&mut s);

    assert_line_has(&rep, "Outcome", "1 failed", "V1 outcome");

    assert!(
        rep.contains("Trash unavailable: no Trash backend on this host"),
        "V2: no backend reason"
    );

    assert!(rep.contains("! 1 failed"), "S1: no failure status");

    assert_line_has(&rep, "Revision", "c4d729e20ca0805b", "S2 revision");

    assert!(
        !boot.contains("Cleanup report"),
        "N1: report visible at boot"
    );

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Cleanup report", "report left");
    checkpoint(&mut s, &dir, "02-left");
    let left = live_text(&mut s);

    assert!(!left.contains("Cleanup report"), "N2: report stuck");
    assert!(
        left.contains("› Review cleanup candidates"),
        "N2: finder not back"
    );
    checkpoint(&mut s, &dir, "03-left");
    eprintln!("h1 cleanup_linux_report: failed report, left");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cleanup_plan() {
    let dir = h1_dir("h1_cleanup_plan");
    let case = h1_case("cleanup_plan", PLAN80, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(&mut s, &["c", "wait:4 steps · 4 included"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-plan");
    let plan = live_text(&mut s);

    assert!(
        plan.contains("4 steps · 4 included · 0 excluded · ~9 s"),
        "V1: no plan header"
    );

    assert_line_has(
        &plan,
        "▎[✓] 01 Trash frontend › node_modules",
        "optional",
        "V2 cursor",
    );

    assert_line_has(&plan, "c Confirm", "p Facts", "S1 footer");

    assert_line_has(&plan, "State", "ready", "S2 state");

    assert!(!plan.contains(GATE1), "N1: gate 1 open before c");

    support::press_step(&s, "down");
    waits::wait_state(&mut s, "▎[✓] 02 cargo clean · backend", "cursor down");
    support::press_step(&s, "up");
    waits::wait_state(
        &mut s,
        "▎[✓] 01 Trash frontend › node_modules",
        "cursor back",
    );

    support::press_step(&s, "c");
    waits::wait_state(&mut s, GATE1, "gate 1 opened");
    checkpoint(&mut s, &dir, "02-gate1-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE1, "gate 1 left");
    waits::wait_state(&mut s, "4 steps · 4 included", "plan back");
    let back = live_text(&mut s);

    assert!(!back.contains(GATE1), "N2: gate 1 stuck");
    assert_line_has(
        &back,
        "▎[✓] 01 Trash frontend › node_modules",
        "optional",
        "cursor intact",
    );
    checkpoint(&mut s, &dir, "03-plan-again");
    eprintln!("h1 cleanup_plan: plan, cursor + gate roundtrips");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_cleanup_report() {
    const PHRASE: &str = "TRASH 1 UNDER /Users/alex/Projects/safe ON mbp";
    let dir = h1_dir("h1_cleanup_report");
    let case = h1_case_t("cleanup_report", HIST_REDUCED, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Analyze disk usage",
            "enter",
            "wait:1 unreadable",
            "down",
            "space",
            "d",
            &format!("wait:{GATE1}"),
            "right",
            "enter",
            &format!("wait:{GATE2}"),
            "enter",
            &format!("type:{PHRASE}"),
            "enter",
            "tab",
            "enter",
            "wait:Cleanup report",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-report");
    let rep = live_text(&mut s);

    assert_line_has(&rep, "Outcome", "1 trashed", "V1 outcome");

    assert_line_has(&rep, "trashed", "~/Projects/safe/big", "V2 detail");
    assert!(rep.contains("5.9 GiB left the tree"), "V2: no bytes fact");

    assert!(rep.contains("1 trashed"), "S1: no trashed status");

    assert_line_has(&rep, "Next", "Cleanup history", "S2 next");

    assert!(
        !boot.contains("Cleanup report"),
        "N1: report visible at boot"
    );

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Cleanup report", "report left");
    checkpoint(&mut s, &dir, "02-left");
    let left = live_text(&mut s);

    assert!(!left.contains("Cleanup report"), "N2: report stuck");
    assert!(
        !left.contains("~/Projects/safe/big"),
        "N2: detail row stuck"
    );
    checkpoint(&mut s, &dir, "03-left");
    eprintln!("h1 cleanup_report: trashed report, left");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_disk_scan_complete() {
    let dir = h1_dir("h1_disk_scan_complete");
    let case = h1_case_t("disk_scan_complete", SCAN, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["type:Analyze disk usage", "enter", "wait:2 unreadable"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-scan");
    let scan = live_text(&mut s);

    assert!(
        scan.contains("scan complete · 852.0 KiB allocated"),
        "V1: no completion summary"
    );

    assert_line_has(&scan, "2 unreadable", "Full Disk Access", "V2 warning");

    assert_line_has(&scan, "Freshness", "live · complete", "S1 freshness");

    assert!(scan.contains("0 selected · 0 B"), "S2: no empty selection");

    assert!(
        !scan.contains("largest first · apparent"),
        "N1: apparent sort active"
    );

    support::press_step(&s, "down");
    support::wait_screen(&mut s, case_timeout(&case), "cursor on many", |screen| {
        support::screen_text(screen)
            .lines()
            .any(|l| l.contains("▎") && l.contains("many"))
    });
    let moved = live_text(&mut s);
    assert_line_has(&moved, "▎", "many", "A1 cursor on many");
    support::press_step(&s, "up");
    waits::wait_state(&mut s, "▎▾   data", "cursor back");

    support::press_step(&s, "s");
    waits::wait_state(&mut s, "largest first · apparent", "apparent on");
    checkpoint(&mut s, &dir, "02-apparent-presence");
    support::press_step(&s, "s");
    waits::wait_state(&mut s, "largest first · allocated", "allocated back");
    let back = live_text(&mut s);

    assert!(
        !back.contains("largest first · apparent"),
        "N2: apparent stuck"
    );
    assert!(back.contains("scan complete"), "N2: scan state lost");
    checkpoint(&mut s, &dir, "03-allocated-again");
    eprintln!("h1 disk_scan_complete: scan, cursor + sort roundtrips");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_disk_top_files() {
    let dir = h1_dir("h1_disk_top_files");
    let case = h1_case_t("disk_top_files", NAV, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Analyze disk usage",
            "enter",
            "wait:scan complete",
            "t",
            "wait:Top files · Spotlight",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-top");
    let top = live_text(&mut s);

    assert!(top.contains("Top files · Spotlight"), "V1: no title");

    assert_line_has(&top, "iso.iso", "700.0 MiB", "V2 first row");
    assert!(top.contains("stat failed"), "V2: no stat-failed row");

    assert!(top.contains("3 files · 0 selected"), "S1: no counts");

    assert_line_has(&top, "y Copy path", "Esc Back", "S2 footer");

    assert!(!top.contains("largest first"), "N2: tree sort label stuck");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Top files · Spotlight", "top page left");
    waits::wait_state(&mut s, "largest first", "tree back");
    checkpoint(&mut s, &dir, "02-tree");
    let tree = live_text(&mut s);

    assert!(
        !tree.contains("Top files · Spotlight"),
        "N1: top page stuck"
    );

    support::press_step(&s, "t");
    waits::wait_state(&mut s, "Top files · Spotlight", "top reopened");
    waits::wait_state(&mut s, "3 files · 0 selected", "counts back");
    checkpoint(&mut s, &dir, "03-top-again");
    eprintln!("h1 disk_top_files: top page, tree roundtrip, reopen");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_disk_tree() {
    let dir = h1_dir("h1_disk_tree");
    let case = h1_case_t("disk_tree", NAV, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["type:Analyze disk usage", "enter", "wait:scan complete"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-tree");
    let tree = live_text(&mut s);

    assert!(
        tree.contains("scan complete · 2.3 MiB allocated · 21 entries"),
        "V1: no completion summary"
    );

    assert_line_has(&tree, "▾   big", "2.3 MiB · 100%", "V2 root");

    assert!(
        tree.contains("largest first · allocated · noise folded"),
        "S1: no sort state"
    );

    assert_line_has(
        &tree,
        "Selection",
        "not selected · Space selects",
        "S2 selection",
    );

    assert!(
        !tree.contains("largest first · apparent"),
        "N1: apparent sort active"
    );

    support::press_step(&s, "down");
    support::wait_screen(&mut s, case_timeout(&case), "cursor on target", |screen| {
        support::screen_text(screen)
            .lines()
            .any(|l| l.contains("▎") && l.contains("target"))
    });
    let moved = live_text(&mut s);
    assert_line_has(&moved, "▎", "target", "A1 cursor on target");
    support::press_step(&s, "up");
    waits::wait_state(&mut s, "▎▾   big", "cursor back");

    support::press_step(&s, "s");
    waits::wait_state(&mut s, "largest first · apparent", "apparent on");
    checkpoint(&mut s, &dir, "02-apparent-presence");
    support::press_step(&s, "s");
    waits::wait_state(&mut s, "largest first · allocated", "allocated back");
    let back = live_text(&mut s);

    assert!(
        !back.contains("largest first · apparent"),
        "N2: apparent stuck"
    );
    assert!(back.contains("noise folded"), "N2: fold state lost");
    checkpoint(&mut s, &dir, "03-allocated-again");
    eprintln!("h1 disk_tree: tree, cursor + sort roundtrips");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_disk_tree_apparent() {
    let dir = h1_dir("h1_disk_tree_apparent");
    let case = h1_case_t("disk_tree_apparent", NAV, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Analyze disk usage",
            "enter",
            "wait:scan complete",
            "s",
            "wait:largest first · apparent",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-apparent");
    let app = live_text(&mut s);

    assert!(
        app.contains("largest first · apparent · noise folded"),
        "V1: no apparent label"
    );

    assert_line_has(&app, "vm.img", "3.5 MiB", "V2 apparent row");

    assert!(
        app.contains("Sorted by apparent size · rows still show allocated bytes"),
        "S1: no sort footer"
    );

    assert_line_has(&app, "Apparent", "5.6 MiB · sparse", "S2 apparent total");

    support::press_step(&s, "s");
    waits::wait_state(&mut s, "largest first · allocated", "allocated on");
    checkpoint(&mut s, &dir, "02-allocated");
    let alloc = live_text(&mut s);

    assert!(
        !alloc.contains("largest first · apparent"),
        "N1: apparent stuck"
    );

    support::press_step(&s, "s");
    waits::wait_state(&mut s, "largest first · apparent", "apparent back");
    let again = live_text(&mut s);

    assert!(
        !again.contains("largest first · allocated"),
        "N2: allocated stuck"
    );
    assert_line_has(&again, "vm.img", "3.5 MiB", "order intact");
    checkpoint(&mut s, &dir, "03-apparent-again");
    eprintln!("h1 disk_tree_apparent: apparent, toggle roundtrip");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_disk_tree_selected() {
    let dir = h1_dir("h1_disk_tree_selected");
    let case = h1_case_t("disk_tree_selected", NAV, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Analyze disk usage",
            "enter",
            "wait:scan complete",
            "s",
            "s",
            "down",
            "down",
            "f",
            "f",
            "space",
            "wait:1 selected (176.0",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-selected");
    let sel = live_text(&mut s);

    assert!(
        sel.contains("1 selected (176.0 KiB)"),
        "V1: no selection summary"
    );

    assert_line_has(&sel, "✓ noise", "176.0 KiB", "V2 checked row");

    assert_line_has(&sel, "◐ big", "2.3 MiB · 100%", "S1 partial parent");

    assert_line_has(&sel, "Selection", "selected · Space unselects", "S2 detail");

    support::press_step(&s, "space");
    waits::wait_gone(&mut s, "✓ noise", "row unchecked");
    waits::wait_state(&mut s, "0 selected", "selection empty");
    checkpoint(&mut s, &dir, "02-unselected");
    let unsel = live_text(&mut s);

    assert!(!unsel.contains("✓ noise"), "N1: check stuck");
    assert!(!unsel.contains("◐ big"), "N1: partial marker stuck");

    support::press_step(&s, "space");
    waits::wait_state(&mut s, "1 selected (176.0 KiB)", "selected again");
    let again = live_text(&mut s);

    assert!(!again.contains("0 selected"), "N2: empty state stuck");
    assert_line_has(&again, "✓ noise", "176.0 KiB", "check intact");
    checkpoint(&mut s, &dir, "03-selected-again");
    eprintln!("h1 disk_tree_selected: selected, unselect roundtrip, reselect");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_disk_tree_unfolded() {
    let dir = h1_dir("h1_disk_tree_unfolded");
    let case = h1_case_t("disk_tree_unfolded", NAV, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Analyze disk usage",
            "enter",
            "wait:scan complete",
            "s",
            "s",
            "down",
            "down",
            "f",
            "wait:allocated · unfolded",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-unfolded");
    let unf = live_text(&mut s);

    assert!(
        unf.contains("largest first · allocated · unfolded"),
        "V1: no unfolded label"
    );

    assert_line_has(&unf, "▎", "noise", "V2 cursor");
    assert_line_has(&unf, "Path", "~/work/big/noise", "V2 detail path");

    assert!(
        unf.contains("Noise folders unfolded"),
        "S1: no unfold footer"
    );

    assert_line_has(&unf, "Entries", "15 · hardlinks counted once", "S2 entries");

    support::press_step(&s, "f");
    waits::wait_state(&mut s, "noise folded", "folded again");
    checkpoint(&mut s, &dir, "02-folded");
    let folded = live_text(&mut s);

    assert!(!folded.contains("unfolded"), "N1: unfolded stuck");
    assert!(
        !folded.contains("Noise folders unfolded"),
        "N1: unfold footer stuck"
    );

    support::press_step(&s, "f");
    waits::wait_state(&mut s, "allocated · unfolded", "unfolded again");
    let again = live_text(&mut s);

    assert!(!again.contains("noise folded"), "N2: folded stuck");
    assert_line_has(&again, "▎", "noise", "cursor intact");
    checkpoint(&mut s, &dir, "03-unfolded-again");
    eprintln!("h1 disk_tree_unfolded: unfolded, fold roundtrip, unfold");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_docker_done() {
    const PHRASE: &str = "I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox";
    let dir = h1_dir("h1_docker_done");
    let case = h1_case_t("docker_done", DOCKER_REDUCED, BOOT, 60_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "enter",
            "c",
            &format!("wait:{GATE1}"),
            "right",
            "enter",
            &format!("wait:{GATE2}"),
            "enter",
            &format!("type:{PHRASE}"),
            "enter",
            "tab",
            "enter",
            "wait:Plan changed",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-drift");

    support::drive_with_timeout(
        &mut s,
        &[
            "right",
            "enter",
            "enter",
            &format!("type:{PHRASE}"),
            "enter",
            "tab",
            "enter",
            "wait:7 succeeded",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-done");
    let done = live_text(&mut s);

    assert!(
        done.contains("Clean Docker completely · 7 of 7 done"),
        "V1: no done header"
    );

    assert!(
        done.contains("reclaimed 18.4 GB on devbox"),
        "V2: no reclaimed fact"
    );

    assert!(done.contains("finished · 7 ok"), "S1: no finished status");

    assert!(done.contains("7 succeeded"), "S2: no success count");
    assert_line_has(&done, "Next", "Docker disk usage", "S2 next");

    assert!(!done.contains("Plan changed"), "N1: drift bounce stuck");

    support::press_step(&s, "up");
    waits::wait_state(&mut s, "▎ ✓ 06 Prune builder", "cursor up");
    support::press_step(&s, "down");
    waits::wait_state(&mut s, "▎ ✓ 07 Rescan and report", "cursor back");
    let back = live_text(&mut s);

    assert!(!back.contains(GATE1), "N2: gate 1 stuck");
    assert!(!back.contains(GATE2), "N2: gate 2 stuck");
    checkpoint(&mut s, &dir, "03-cursor-back");
    eprintln!("h1 docker_done: drift bounce, 7/7 done, cursor roundtrip");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_docker_drift() {
    const PHRASE: &str = "I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox";
    let dir = h1_dir("h1_docker_drift");
    let case = h1_case("docker_drift", DOCKER_PAUSED, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "enter",
            "c",
            &format!("wait:{GATE1}"),
            "right",
            "enter",
            &format!("wait:{GATE2}"),
            "enter",
            &format!("type:{PHRASE}"),
            "enter",
            "tab",
            "enter",
            "wait:a new container",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-drift");
    let drift = live_text(&mut s);

    assert_line_has(&drift, "Changed", "tmp-shell-3", "V1 changed row");

    assert!(
        drift.contains("! Plan changed · a new container (tmp-shell-3) appeared"),
        "V2: no drift footer"
    );

    assert_line_has(
        &drift,
        "Review · Clean Docker completely",
        GATE1,
        "S1 title",
    );

    assert_line_has(&drift, "Reclaimable", "~18.4 GB", "S2 reclaimable");

    assert!(!drift.contains(GATE2), "N1: gate 2 open after bounce");

    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-gate2-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled");
    waits::wait_state(&mut s, "a new container", "drift intact");

    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled again");
    let back = live_text(&mut s);

    assert!(!back.contains(GATE2), "N2: gate 2 stuck");
    assert_line_has(&back, "Changed", "tmp-shell-3", "drift intact");
    checkpoint(&mut s, &dir, "03-drift-again");
    eprintln!("h1 docker_drift: drift bounce, gate-2 roundtrip x2");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_docker_gate2_typed() {
    const PHRASE: &str = "I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox";
    let dir = h1_dir("h1_docker_gate2_typed");
    let case = h1_case("docker_gate2_typed", DOCKER_PAUSED, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "enter",
            "c",
            &format!("wait:{GATE1}"),
            "right",
            "enter",
            &format!("wait:{GATE2}"),
            "enter",
            &format!("type:{PHRASE}"),
        ],
        case.timeout_ms,
    );
    waits::wait_state(&mut s, &format!("▎ {PHRASE}"), "phrase typed");
    checkpoint(&mut s, &dir, "01-typed");
    let typed = live_text(&mut s);

    assert!(
        typed.contains(&format!("▎ {PHRASE}")),
        "V1: no typed phrase"
    );

    assert!(
        typed.contains("Clean Docker completely · gate 2 of 2"),
        "V2: no gate-2 title"
    );

    assert_line_has(&typed, "EDIT", "Type Phrase", "S1 edit footer");

    assert_line_has(&typed, "Reclaimable", "~18.4 GB", "S2 reclaimable");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, &format!("▎ {PHRASE}"), "phrase cleared");
    waits::wait_state(&mut s, GATE2, "dialog still open");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);

    assert!(
        !cleared.contains(&format!("▎ {PHRASE}")),
        "N1: typed phrase stuck"
    );
    assert!(cleared.contains("to confirm"), "N1: prompt lost");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "dialog cancelled");
    waits::wait_state(&mut s, GATE1, "gate 1 back");
    checkpoint(&mut s, &dir, "03-cancelled");
    let back = live_text(&mut s);

    assert!(!back.contains(GATE2), "N2: gate 2 stuck");
    assert!(!back.contains("to confirm"), "N2: phrase prompt stuck");
    eprintln!("h1 docker_gate2_typed: typed, cleared, cancelled");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_docker_remove_failed() {
    const HEADER: &str = "stop all containers · failed · 0 s · exit 1";
    let dir = h1_dir("h1_docker_remove_failed");
    let case = h1_case_t("docker_remove_failed", PDOCKER_RED, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Stop all containers",
            "enter",
            "right",
            "enter",
            &format!("wait:{HEADER}"),
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-failed");
    let failed = live_text(&mut s);

    assert!(failed.contains(HEADER), "V1: no failed header");

    assert!(
        failed.contains("cannot stop container"),
        "V2: no daemon refusal"
    );
    assert!(failed.contains("acme-scheduler-1"), "V2: no container id");

    assert!(
        failed.contains("! stop all containers failed · exit 1"),
        "S1: no failure status"
    );

    assert_line_has(&failed, "r Restart", "x Close", "S2 footer");

    assert!(!boot.contains("exit 1"), "N1: failure visible at boot");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, HEADER, "failed tab left");
    checkpoint(&mut s, &dir, "02-here");
    let here = live_text(&mut s);

    assert!(!here.contains(HEADER), "N2: failed header stuck");
    assert!(
        !here.contains("cannot stop container"),
        "N2: daemon output stuck"
    );

    support::drive_with_timeout(
        &mut s,
        &["enter", "right", "enter", &format!("wait:{HEADER}")],
        case.timeout_ms,
    );
    waits::wait_state(&mut s, "cannot stop container", "failed again");
    checkpoint(&mut s, &dir, "03-failed-again");
    eprintln!("h1 docker_remove_failed: failed, left, failed again");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_docker_remove_gate1() {
    let dir = h1_dir("h1_docker_remove_gate1");
    let case = h1_case("docker_remove_gate1", PDOCKER_PAUSED, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Stop and remove all",
            "enter",
            &format!("wait:{GATE1}"),
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-gate1");
    let gate = live_text(&mut s);

    assert_line_has(
        &gate,
        "Review · Stop and remove all containers",
        GATE1,
        "V1 title",
    );

    assert_line_has(&gate, "Changes", "6 containers removed", "V2 changes");

    assert_line_has(&gate, "Risk", "destructive", "S1 risk");

    assert_line_has(&gate, "Sequence · 2 commands", "y copies", "S2 sequence");

    assert!(!gate.contains(GATE2), "N1: gate 2 open before Right+Enter");

    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-gate2-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled");
    waits::wait_state(&mut s, GATE1, "gate 1 back");

    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled again");
    let back = live_text(&mut s);

    assert!(!back.contains(GATE2), "N2: gate 2 stuck");
    assert!(!back.contains("to confirm"), "N2: phrase prompt stuck");
    assert_line_has(
        &back,
        "Review · Stop and remove all containers",
        GATE1,
        "gate 1 intact",
    );
    checkpoint(&mut s, &dir, "03-gate1-again");
    eprintln!("h1 docker_remove_gate1: gate 1, gate-2 roundtrip x2");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_docker_remove_gate2() {
    const PHRASE: &str = "REMOVE ALL CONTAINERS ON mbp";
    let dir = h1_dir("h1_docker_remove_gate2");
    let case = h1_case("docker_remove_gate2", PDOCKER_PAUSED, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Stop and remove all",
            "enter",
            &format!("wait:{GATE1}"),
            "right",
            "enter",
            &format!("wait:{GATE2}"),
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-gate2");
    let gate = live_text(&mut s);

    assert!(
        gate.contains("Stop and remove all containers · gate 2 of 2"),
        "V1: no gate-2 title"
    );

    assert!(
        gate.contains(&format!("Type {PHRASE} to confirm")),
        "V2: no phrase prompt"
    );

    assert_line_has(&gate, "Enter Confirm", "Esc Cancel", "S1 footer");

    assert_line_has(&gate, "Risk", "destructive", "S2 risk");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled");
    waits::wait_state(&mut s, GATE1, "gate 1 back");
    checkpoint(&mut s, &dir, "02-cancelled");
    let cancelled = live_text(&mut s);

    assert!(!cancelled.contains(GATE2), "N1: gate 2 stuck");
    assert!(!cancelled.contains("to confirm"), "N1: phrase prompt stuck");

    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled again");
    let back = live_text(&mut s);

    assert!(!back.contains(GATE2), "N2: gate 2 stuck again");
    assert_line_has(
        &back,
        "Review · Stop and remove all containers",
        GATE1,
        "gate 1 intact",
    );
    checkpoint(&mut s, &dir, "03-cancelled-again");
    eprintln!("h1 docker_remove_gate2: gate 2, cancel roundtrip x2");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_gradle_cleanup() {
    let dir = h1_dir("h1_gradle_cleanup");
    let case = h1_case("gradle_cleanup", GRADLE, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Clean Gradle outputs",
            "enter",
            "wait:Cleanup · Gradle outputs",
            "right",
            "wait:▾ Gradle outputs",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-cleanup");
    let gradle = live_text(&mut s);

    assert!(
        gradle.contains("Cleanup · Gradle outputs"),
        "V1: no cleanup title"
    );

    assert_line_has(&gradle, "▾ Gradle outputs", "49.0 MiB", "V2 fold");

    assert_line_has(&gradle, "Prerequisite", "gradle --stop", "S1 prerequisite");

    assert_line_has(
        &gradle,
        "Selection",
        "4 of 4 candidates eligible",
        "S2 selection",
    );

    assert!(!gradle.contains(GATE1), "N1: gate 1 open before d");

    support::press_step(&s, "left");
    waits::wait_gone(&mut s, "▾ Gradle outputs", "fold closed");
    support::press_step(&s, "right");
    waits::wait_state(&mut s, "▾ Gradle outputs", "fold reopened");

    support::press_step(&s, "d");
    waits::wait_state(&mut s, GATE1, "gate 1 opened");
    checkpoint(&mut s, &dir, "02-gate1-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE1, "gate 1 left");
    waits::wait_state(&mut s, "Cleanup · Gradle outputs", "review back");
    let back = live_text(&mut s);

    assert!(!back.contains(GATE1), "N2: gate 1 stuck");
    assert!(!back.contains(GATE2), "N2: gate 2 stuck");
    checkpoint(&mut s, &dir, "03-review-again");
    eprintln!("h1 gradle_cleanup: review, fold + gate roundtrips");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_gradle_gate1() {
    let dir = h1_dir("h1_gradle_gate1");
    let case = h1_case("gradle_gate1", GRADLE, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Clean Gradle outputs",
            "enter",
            "wait:Cleanup · Gradle outputs",
            "right",
            "wait:▾ Gradle outputs",
            "d",
            &format!("wait:{GATE1}"),
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-gate1");
    let gate = live_text(&mut s);

    assert_line_has(&gate, "Review · trash 4 items", GATE1, "V1 title");

    assert_line_has(
        &gate,
        "Prerequisite",
        "gradle --stop first",
        "V2 prerequisite",
    );

    assert_line_has(&gate, "Sequence · 4 commands", "y copies", "S1 sequence");

    assert_line_has(&gate, "Paths", ".gradle", "S2 paths");

    assert!(!gate.contains(GATE2), "N1: gate 2 open before Right+Enter");

    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-gate2-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled");
    waits::wait_state(&mut s, GATE1, "gate 1 back");

    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled again");
    let back = live_text(&mut s);

    assert!(!back.contains(GATE2), "N2: gate 2 stuck");
    assert!(!back.contains("to confirm"), "N2: phrase prompt stuck");
    assert_line_has(&back, "Review · trash 4 items", GATE1, "gate 1 intact");
    checkpoint(&mut s, &dir, "03-gate1-again");
    eprintln!("h1 gradle_gate1: gate 1, gate-2 roundtrip x2");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_idea_cleanup() {
    let dir = h1_dir("h1_idea_cleanup");
    let case = h1_case("idea_cleanup", IDEA, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["type:Clean IntelliJ", "enter", "right", "wait:idea.clean"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-cleanup");
    let idea = live_text(&mut s);

    assert!(
        idea.contains("Cleanup · IntelliJ metadata"),
        "V1: no cleanup title"
    );

    assert_line_has(&idea, "▾ IntelliJ metadata", "44.0 KiB", "V2 fold");

    assert!(idea.contains("Unreadable"), "S1: no unreadable label");
    assert!(idea.contains("~/work/ide/locked"), "S1: no locked path");

    assert_line_has(
        &idea,
        "Selection",
        "4 of 4 candidates eligible",
        "S2 selection",
    );

    assert!(!idea.contains(GATE1), "N1: gate 1 open before d");

    support::press_step(&s, "left");
    waits::wait_gone(&mut s, "▾ IntelliJ metadata", "fold closed");
    support::press_step(&s, "right");
    waits::wait_state(&mut s, "▾ IntelliJ metadata", "fold reopened");

    support::press_step(&s, "d");
    waits::wait_state(&mut s, GATE1, "gate 1 opened");
    checkpoint(&mut s, &dir, "02-gate1-presence");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE1, "gate 1 left");
    waits::wait_state(&mut s, "Cleanup · IntelliJ metadata", "review back");
    let back = live_text(&mut s);

    assert!(!back.contains(GATE1), "N2: gate 1 stuck");
    assert!(!back.contains(GATE2), "N2: gate 2 stuck");
    checkpoint(&mut s, &dir, "03-review-again");
    eprintln!("h1 idea_cleanup: review, fold + gate roundtrips");
}

#[test]
#[ignore = "holla h1 check; run with --ignored"]
fn h1_idea_gate2() {
    const PHRASE: &str = "TRASH 4 UNDER /Users/alex/work/ide ON mbp";
    let dir = h1_dir("h1_idea_gate2");
    let case = h1_case("idea_gate2", IDEA, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Clean IntelliJ",
            "enter",
            "right",
            "wait:idea.clean",
            "d",
            &format!("wait:{GATE1}"),
            "right",
            "enter",
            &format!("wait:{GATE2}"),
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-gate2");
    let gate = live_text(&mut s);

    assert!(
        gate.contains("Trash 4 items · gate 2 of 2"),
        "V1: no gate-2 title"
    );

    assert!(
        gate.contains(&format!("Type {PHRASE} to confirm")),
        "V2: no phrase prompt"
    );

    assert_line_has(&gate, "Paths", "~/work/ide/.idea", "S1 paths");
    assert!(gate.contains("mod/mod.iml"), "S1: no fourth path");

    assert_line_has(&gate, "Enter Confirm", "Esc Cancel", "S2 footer");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled");
    waits::wait_state(&mut s, GATE1, "gate 1 back");
    checkpoint(&mut s, &dir, "02-cancelled");
    let cancelled = live_text(&mut s);

    assert!(!cancelled.contains(GATE2), "N1: gate 2 stuck");
    assert!(!cancelled.contains("to confirm"), "N1: phrase prompt stuck");

    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, GATE2, "gate 2 cancelled again");
    let back = live_text(&mut s);

    assert!(!back.contains(GATE2), "N2: gate 2 stuck again");
    assert_line_has(&back, "Review · trash 4 items", GATE1, "gate 1 intact");
    checkpoint(&mut s, &dir, "03-cancelled-again");
    eprintln!("h1 idea_gate2: gate 2, cancel roundtrip x2");
}
