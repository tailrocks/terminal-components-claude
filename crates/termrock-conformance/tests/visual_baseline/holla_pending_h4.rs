use std::path::{Path, PathBuf};

use crate::support::Session;

use crate::support::state_waits as waits;
use crate::support::{self, Case, Color, HOLLA};

const BOOT: &str = "holla❯";
const PLACEHOLDER: &str = "Search actions and resources…";
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

fn h4_case_t(
    slug: &str,
    args: &'static [&'static str],
    boot: &'static str,
    timeout_ms: u64,
) -> Case {
    h4_case(slug, args, boot).timeout(timeout_ms)
}

fn h4_dir(test: &str) -> PathBuf {
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
        "profile: tuiscotti-default\ntransport: pty\nport_of: 2493cf1ccab6562c942b1d15d36a8df85d8652fa\nbinary: {}\nsha256: {}\nsize_bytes: {}\nmodified_unix: {modified}\nargv: {}\n",
        subject.path.display(),
        subject.sha256,
        subject.len,
        argv.join(" "),
    );
    std::fs::write(dir.join("provenance.txt"), &body)
        .unwrap_or_else(|e| panic!("write {}: {e}", dir.join("provenance.txt").display()));
    eprintln!("h4 provenance: {body}");
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

fn live_text(s: &mut Session) -> String {
    let obs = s
        .inner
        .observe_now()
        .unwrap_or_else(|e| panic!("live sample failed: {e:#}"));
    support::screen_text(&obs.screen)
}

fn wait_absent(s: &mut Session, needle: &str, what: &str) {
    support::wait_screen(
        s,
        support::DEFAULT_WAIT,
        &format!("{what}: `{needle}` never expired"),
        |screen| !support::screen_text(screen).contains(needle),
    );
}

fn assert_line_has(text: &str, needle: &str, also: &str, what: &str) {
    assert!(
        text.lines().any(|l| l.contains(needle) && l.contains(also)),
        "{what}: no line contains both `{needle}` and `{also}`"
    );
}

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

    support::drive_with_timeout(
        &mut s,
        &["down", "down", "down", "down", "down", "space"],
        case.timeout_ms,
    );
    waits::wait_state(&mut s, "7 included · 1 excluded", "step excluded");

    support::drive_with_timeout(&mut s, &["c", &format!("wait:{DIALOG}")], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-dialog");
    let dialog = live_text(&mut s);

    assert!(dialog.contains(DIALOG), "V1: no dialog title");

    assert_line_has(&dialog, "Steps", "7 included · 1 excluded", "V2 steps");

    assert_line_has(&dialog, "▎Cancel", "Start plan", "S1 choice");

    assert!(dialog.contains(FOOTER), "S2: no dialog footer");

    assert!(!dialog.contains(CANCELLED), "N1: cancelled before Esc");

    support::press_step(&s, "escape");
    wait_absent(&mut s, DIALOG, "dialog cancelled");
    waits::wait_state(&mut s, CANCELLED, "cancelled status");
    checkpoint(&mut s, &dir, "02-cancelled");
    let cancelled = live_text(&mut s);

    assert!(!cancelled.contains(DIALOG), "N2: dialog title stuck");
    support::drive_with_timeout(&mut s, &["c", &format!("wait:{DIALOG}")], case.timeout_ms);
    support::press_step(&s, "escape");
    wait_absent(&mut s, DIALOG, "dialog cancelled again");
    waits::wait_state(&mut s, CANCELLED, "cancelled again");
    checkpoint(&mut s, &dir, "03-cancelled-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "8 steps", "7 included · 1 excluded", "plan intact");
    eprintln!("h4 upgrade_confirm: dialog, cancel roundtrip x2");
}

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

    support::drive_with_timeout(
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
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-excluded");
    let excluded = live_text(&mut s);

    assert_line_has(
        &excluded,
        "▎[ ] 06 Upgrade selected mise tools",
        "excluded",
        "V1 row",
    );

    assert!(
        excluded.contains("7 included · 1 excluded"),
        "V2: no exclusion count"
    );

    assert_line_has(&excluded, "State", "excluded", "S1 state");

    assert_line_has(&excluded, "8 steps", "7 included · 1 excluded", "S2 header");

    assert!(!excluded.contains("8 included"), "N1: full count stuck");

    support::press_step(&s, "space");
    waits::wait_state(&mut s, "8 included · 0 excluded", "step included");
    checkpoint(&mut s, &dir, "02-included");
    support::press_step(&s, "space");
    waits::wait_state(&mut s, "7 included · 1 excluded", "step excluded again");
    checkpoint(&mut s, &dir, "03-excluded-again");
    let back = live_text(&mut s);

    assert!(!back.contains("0 excluded"), "N2: zero count stuck");
    assert_line_has(
        &back,
        "▎[ ] 06 Upgrade selected mise tools",
        "excluded",
        "row intact",
    );
    eprintln!("h4 upgrade_excluded: exclusion toggle roundtrip");
}

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

    support::drive_with_timeout(&mut s, &["p", "wait:▎01 Preflight"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-drawer");
    let drawer = live_text(&mut s);

    assert_line_has(&drawer, "▎01 Preflight", "required", "V1 header");

    assert_line_has(&drawer, "Runs", "cat /etc/os-release", "V2 runs");

    assert_line_has(&drawer, "Privilege", "sudo", "S1 privilege");

    assert_line_has(&drawer, "State", "ready", "S2 state");

    assert!(!drawer.contains("▎[✓]"), "N1: steps cursor stuck");

    support::press_step(&s, "p");
    waits::wait_state(&mut s, "▎[✓] 01 Preflight", "steps back");
    checkpoint(&mut s, &dir, "02-steps");
    let steps = live_text(&mut s);

    assert!(!steps.contains("▎01 Preflight"), "N2: drawer cursor stuck");
    support::drive_with_timeout(&mut s, &["p", "wait:▎01 Preflight"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-drawer-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "▎01 Preflight", "required", "drawer intact");
    eprintln!("h4 upgrade_facts: drawer focus roundtrip");
}

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

    assert_line_has(&boot, "›Tasks", "Services", "V1 explore");

    assert!(
        boot.contains("nothing used here yet"),
        "V2: no empty recent"
    );

    assert_line_has(&boot, "In", "~/work", "S1 world");

    support::drive_with_timeout(
        &mut s,
        &["type:Homebrew services", "enter", "wait:svc02"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-services");
    let list = live_text(&mut s);

    assert!(list.contains(DETAIL), "S2: no detail panel");

    assert_line_has(&list, "svc02", "error", "flow row");

    assert!(!list.contains(PLACEHOLDER), "N1: placeholder stuck");

    support::press_step(&s, "escape");
    wait_absent(&mut s, DETAIL, "listing closed");
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-finder");
    let finder = live_text(&mut s);

    assert!(!finder.contains(DETAIL), "N2: detail panel stuck");
    support::drive_with_timeout(
        &mut s,
        &["type:Homebrew services", "enter", "wait:svc02"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "03-services-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "svc02", "error", "listing intact");
    eprintln!("h4 parity_brew_services: boot, listing roundtrip");
}

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

    assert_line_has(&boot, "› pnpm a", "package.json here", "V1 row");

    assert!(boot.contains("pnpm e"), "V2: no script shelf");

    assert_line_has(&boot, "In", "~/work/site", "S1 world");

    support::drive_with_timeout(
        &mut s,
        &["type:Browse ~/work/site", "enter", "wait:16 entries"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-listing");
    let listing = live_text(&mut s);
    assert!(
        listing.contains("2 hidden hidden"),
        "pre: no hidden-hidden status"
    );

    support::drive_with_timeout(&mut s, &["ctrl-h", "wait:2 hidden shown"], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-hidden");
    let hidden = live_text(&mut s);

    assert_line_has(&hidden, "18 entries", "2 hidden shown", "S2 header");

    assert_line_has(&hidden, ".git", "directory", "flow dotfile");

    assert!(
        hidden.contains("Hidden entries shown"),
        "flow: no shown footer"
    );

    assert!(
        !hidden.contains("2 hidden hidden"),
        "N1: hidden status stuck"
    );

    support::drive_with_timeout(&mut s, &["ctrl-h", "wait:2 hidden hidden"], case.timeout_ms);
    let off = live_text(&mut s);

    assert!(!off.contains("2 hidden shown"), "N2: shown count stuck");
    support::drive_with_timeout(&mut s, &["ctrl-h", "wait:2 hidden shown"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-hidden-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "18 entries", "2 hidden shown", "header intact");
    eprintln!("h4 parity_browser: boot, hidden toggle roundtrip x2");
}

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

    assert_line_has(&boot, "› cargo build", "Cargo.toml here", "V1 row");

    assert!(
        boot.contains("cargo clean · 900.0 MiB"),
        "V2: no clean badge"
    );

    assert_line_has(&boot, "In", "~/work/engine", "S1 world");

    support::drive_with_timeout(
        &mut s,
        &["type:cargo clean", "wait:900.0 MiB"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-clean");
    let clean = live_text(&mut s);
    assert_line_has(
        &clean,
        "› cargo clean · 900.0 MiB",
        "destructive",
        "flow row",
    );

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
    eprintln!("h4 parity_cargo: boot, dialog roundtrip x2");
}

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

    assert_line_has(&boot, "cargo clean · 1000.0 MiB", "destructive", "V1 row");

    assert!(boot.contains("cargo fmt --check"), "V2: no cargo shelf");

    assert_line_has(&boot, "In", "~/Projects/safe", "S1 world");

    assert!(!boot.contains(HIST), "N1: history visible at boot");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Show cleanup history",
            "enter",
            &format!("wait:{HIST}"),
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-history");
    let hist = live_text(&mut s);

    assert_line_has(&hist, "Entries", "1 operation log records", "S2 header");

    assert_line_has(&hist, "2026-08-29 08:41", "trashed", "flow record");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, HIST, "history left");
    checkpoint(&mut s, &dir, "02-finder");
    let left = live_text(&mut s);

    assert!(!left.contains(HIST), "N2: history stuck");
    assert!(!left.contains("Cleanup history"), "N2: title stuck");
    support::drive_with_timeout(&mut s, &["enter", &format!("wait:{HIST}")], case.timeout_ms);
    waits::wait_state(&mut s, "2026-08-29 08:41", "record back");
    checkpoint(&mut s, &dir, "03-history-again");
    eprintln!("h4 parity_cleanup_results: boot, history roundtrip");
}

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

    assert_line_has(&boot, "› Deploy preview", "unreviewed", "V1 row");

    assert!(boot.contains("Global shared id"), "V2: no trusted shelf");

    assert_line_has(&boot, "Gate", "trust required", "S1 gate");

    support::drive_with_timeout(
        &mut s,
        &["type:Deploy preview", "enter", &format!("wait:{PROMPT}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-prompt");
    let prompt = live_text(&mut s);

    assert!(
        prompt.contains("label = \"Deploy preview\""),
        "S2: no definition label"
    );

    assert!(!prompt.contains(PLACEHOLDER), "N1: placeholder stuck");

    support::press_step(&s, "escape");
    wait_absent(&mut s, PROMPT, "review closed");
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-finder");
    let finder = live_text(&mut s);

    assert!(!finder.contains(PROMPT), "N2: prompt stuck");
    support::drive_with_timeout(
        &mut s,
        &["type:Deploy preview", "enter", &format!("wait:{PROMPT}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "03-prompt-again");
    let back = live_text(&mut s);
    assert!(back.contains("label = \"Deploy preview\""), "review intact");
    eprintln!("h4 parity_custom_actions: boot, prompt roundtrip");
}

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

    waits::wait_state(&mut s, "cargo fmt --check", "suggestions settled");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    assert_line_has(&boot, "cargo clean · 1000.0 MiB", "destructive", "V1 row");

    assert!(boot.contains("cargo fmt --check"), "V2: no cargo shelf");

    assert_line_has(&boot, "Data", "live ·", "S1 freshness");

    support::drive_with_timeout(
        &mut s,
        &["type:Analyze disk usage", "enter", "wait:1 unreadable"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-scan");
    let scan = live_text(&mut s);

    assert!(scan.contains(ALLOCATED), "S2: no allocated sort");

    assert!(scan.contains("scan complete"), "flow: no completion");

    assert!(!scan.contains(APPARENT), "N1: apparent sort active");

    support::press_step(&s, "s");
    waits::wait_state(&mut s, APPARENT, "apparent on");
    checkpoint(&mut s, &dir, "02-apparent-presence");
    support::press_step(&s, "s");
    waits::wait_state(&mut s, ALLOCATED, "allocated back");
    let back = live_text(&mut s);

    assert!(!back.contains(APPARENT), "N2: apparent stuck");
    assert!(back.contains("scan complete"), "N2: scan state lost");
    checkpoint(&mut s, &dir, "03-allocated-again");
    eprintln!("h4 parity_delete_safety: boot, scan + sort roundtrip");
}

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

    waits::wait_state(&mut s, "cargo fmt --check", "suggestions settled");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    assert_line_has(&boot, "cargo clean · 2.0 MiB", "destructive", "V1 row");

    assert!(boot.contains("cargo fmt --check"), "V2: no cargo shelf");

    assert_line_has(&boot, "In", "~/work/big", "S1 world");

    support::drive_with_timeout(
        &mut s,
        &["type:Analyze disk usage", "enter", "wait:scan complete"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-scan");
    let scan = live_text(&mut s);

    assert!(scan.contains(ALLOCATED), "S2: no allocated sort");

    assert!(!scan.contains(APPARENT), "N1: apparent sort active");

    support::press_step(&s, "s");
    waits::wait_state(&mut s, APPARENT, "apparent on");
    checkpoint(&mut s, &dir, "02-apparent-presence");
    support::press_step(&s, "s");
    waits::wait_state(&mut s, ALLOCATED, "allocated back");
    let back = live_text(&mut s);

    assert!(!back.contains(APPARENT), "N2: apparent stuck");
    assert!(back.contains("scan complete"), "N2: scan state lost");
    checkpoint(&mut s, &dir, "03-allocated-again");
    eprintln!("h4 parity_disk_navigation: boot, scan + sort roundtrip");
}

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

    assert_line_has(&boot, "› Follow acme-api-1 logs", "unhealthy", "V1 row");

    assert!(
        boot.contains("Follow acme-db-1 logs"),
        "V2: no healthy shelf"
    );

    assert_line_has(&boot, "In", "~/work/stack", "S1 world");

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
        "flow title",
    );

    assert_line_has(&gate, "Changes", "6 containers removed", "S2 changes");

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
    eprintln!("h4 parity_docker: boot, gate roundtrip x2");
}

#[test]
#[ignore = "holla h4 check; run with --ignored"]
fn h4_parity_executor() {
    const DONE: &str = "emit a burst finished · exit 0";
    let dir = h4_dir("h4_parity_executor");
    let case = h4_case_t("parity_executor", EXECUTOR, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

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
    checkpoint(&mut s, &dir, "01-output");
    let output = live_text(&mut s);

    assert_line_has(&output, "emit a burst · succeeded", "exit 0", "S2 title");

    assert!(
        output.contains("started by Emit a burst"),
        "flow: no started-by line"
    );

    assert!(!output.contains(PLACEHOLDER), "N1: placeholder stuck");

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

    assert!(!here.contains("burst line"), "N2: burst body stuck");
    eprintln!("h4 parity_executor: boot, burst + restart roundtrip");
}

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

    assert_line_has(&boot, "›Tasks", "Files", "V1 explore");

    assert!(
        boot.contains("nothing used here yet"),
        "V2: no empty recent"
    );

    assert_line_has(&boot, "In", "~/work/notes", "S1 world");

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

    assert_line_has(&res, FIRST, "7 B of 7 B", "S2 preview");

    assert!(res.contains("3 results · home scope"), "flow: no 3 results");

    support::drive_with_timeout(
        &mut s,
        &["down", &format!("wait:{SECOND}")],
        case.timeout_ms,
    );
    let moved = live_text(&mut s);

    assert!(!moved.contains(FIRST), "N1: first preview stuck");
    support::drive_with_timeout(&mut s, &["up", &format!("wait:{FIRST}")], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-results-again");
    let back = live_text(&mut s);

    assert!(!back.contains(SECOND), "N2: second preview stuck");
    assert!(back.contains("3 results · home scope"), "count intact");
    eprintln!("h4 parity_files: boot, results cursor roundtrip");
}

#[test]
#[ignore = "holla h4 check; run with --ignored"]
fn h4_parity_git_batch() {
    const FF: &str = "Fast-forward";
    let dir = h4_dir("h4_parity_git_batch");
    let case = h4_case_t("parity_git_batch", GITBATCH, BOOT, 15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    waits::wait_state(&mut s, "Children · 4 projects", "explore settled");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    assert_line_has(
        &boot,
        "› Status across 5 child",
        "0 dirty · 1 behind",
        "V1 row",
    );

    assert!(boot.contains("Children · 4 projects"), "V2: no child count");

    assert_line_has(&boot, "Scope", "child", "S1 scope");

    support::drive_with_timeout(
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
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-batch");
    let batch = live_text(&mut s);

    assert_line_has(&batch, "pull alpha · succeeded", "exit 0", "S2 title");

    assert!(
        batch.contains("started by Pull 4 sibling repositories · batch git.pull-all"),
        "flow: no batch class"
    );

    assert!(!batch.contains(PLACEHOLDER), "N1: placeholder stuck");

    support::press_step(&s, "r");
    wait_absent(&mut s, FF, "output restarted");
    support::drive_with_timeout(
        &mut s,
        &[&format!("wait:{FF}"), "wait:pull alpha · succeeded"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-batch-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "pull alpha · succeeded", "exit 0", "title intact");
    support::press_step(&s, "alt-0");
    wait_absent(&mut s, FF, "back to Here");
    waits::wait_state(&mut s, "scope ‹ here ›", "finder back");
    checkpoint(&mut s, &dir, "03-here");
    let here = live_text(&mut s);

    assert!(!here.contains(FF), "N2: report line stuck");
    eprintln!("h4 parity_git_batch: boot, batch + restart roundtrip");
}

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

    assert_line_has(
        &boot,
        "› Review 1 modified files",
        "1 modified · 0 untracked",
        "V1 row",
    );

    assert!(boot.contains("Parent · svc"), "V2: no parent shelf");

    assert_line_has(&boot, "Scope", "parent", "S1 scope");

    support::drive_with_timeout(
        &mut s,
        &["type:Pull", "wait:blocked · 1 modified"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    assert!(res.contains("Results · 3"), "S2: no Results · 3");

    assert_line_has(&res, "› Pull", "blocked · 1 modified", "flow row");

    assert!(!res.contains(PLACEHOLDER), "N1: placeholder stuck");

    support::press_step(&s, "escape");
    wait_absent(&mut s, "Results · 3", "results cleared");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);

    assert!(!cleared.contains("Results · 3"), "N2: results stuck");
    support::drive_with_timeout(
        &mut s,
        &["type:Pull", "wait:blocked · 1 modified"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "03-results-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "› Pull", "blocked · 1 modified", "row intact");
    eprintln!("h4 parity_git_current: boot, results clear roundtrip");
}

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

    assert_line_has(&boot, "› Clean Gradle outputs", "destructive", "V1 row");

    assert!(boot.contains("gradle test"), "V2: no gradle shelf");

    assert_line_has(&boot, "Gate", "opens a review", "S1 gate");

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

    assert_line_has(
        &gradle,
        "Selection",
        "4 of 4 candidates eligible",
        "S2 selection",
    );

    assert_line_has(&gradle, "▾ Gradle outputs", "49.0 MiB", "flow fold");

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
    eprintln!("h4 parity_gradle: boot, fold + gate roundtrips");
}

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

    assert_line_has(&boot, "› Run check", "mise.toml", "V1 row");

    assert!(boot.contains("Run tests"), "V2: no task shelf");

    assert_line_has(&boot, "In", "~/work/holla", "S1 world");

    support::drive_with_timeout(&mut s, &["type:pull", "wait:Results · 3"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    assert!(res.contains("Enter Run"), "S2: no run footer");

    assert_line_has(&res, "› Pull", "up to date", "flow row");

    assert!(!res.contains(PLACEHOLDER), "N1: placeholder stuck");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Results · 3", "results cleared");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);

    assert!(!cleared.contains("Results · 3"), "N2: results stuck");
    support::drive_with_timeout(&mut s, &["type:pull", "wait:Results · 3"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-results-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "› Pull", "up to date", "row intact");
    eprintln!("h4 parity_history: boot, results clear roundtrip");
}

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

    assert_line_has(&boot, "› Clean IntelliJ metadata", "destructive", "V1 row");

    assert!(boot.contains("4 candidates"), "V2: no candidate count");

    assert_line_has(&boot, "In", "~/work/ide", "S1 world");

    support::drive_with_timeout(
        &mut s,
        &["type:Clean IntelliJ", "enter", &format!("wait:{TITLE}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-page");
    let page = live_text(&mut s);

    assert_line_has(
        &page,
        "Selection",
        "4 of 4 candidates eligible",
        "S2 selection",
    );

    assert!(!page.contains(PLACEHOLDER), "N1: placeholder stuck");

    support::press_step(&s, "escape");
    wait_absent(&mut s, TITLE, "page closed");
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-finder");
    let finder = live_text(&mut s);

    assert!(!finder.contains(TITLE), "N2: page title stuck");
    support::drive_with_timeout(
        &mut s,
        &["type:Clean IntelliJ", "enter", &format!("wait:{TITLE}")],
        case.timeout_ms,
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

    assert_line_has(&boot, "› Review cleanup candidates", "18.8 GiB", "V1 row");

    assert!(boot.contains("18 categories"), "V2: no category count");

    assert_line_has(&boot, "Scope", "host", "S1 scope");

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

    assert_line_has(&cats, "Safety", "rebuilt on demand", "S2 detail");

    assert_line_has(&cats, "18 categories", "18.8 GiB", "flow header");

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
    eprintln!("h4 parity_insights: boot, cursor + fold roundtrips");
}

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

    assert_line_has(&boot, "›Tasks", "System", "V1 explore");

    assert!(
        boot.contains("nothing used here yet"),
        "V2: no empty recent"
    );

    assert_line_has(&boot, "In", "~/work/mac", "S1 world");

    support::drive_with_timeout(
        &mut s,
        &[
            "type:Top files on this Mac",
            "enter",
            &format!("wait:{TIMEOUT}"),
            "wait:0 files · 0 selected",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-files");
    let files = live_text(&mut s);

    assert!(files.contains("Esc Back"), "S2: no back footer");

    assert!(
        files.contains("Top files · Spotlight"),
        "flow: no page title"
    );

    assert!(!files.contains(PLACEHOLDER), "N1: placeholder stuck");

    support::press_step(&s, "escape");
    wait_absent(&mut s, "Esc Back", "page closed");
    waits::wait_state(&mut s, "Esc Clear", "finder back");
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-finder");
    let finder = live_text(&mut s);

    assert!(!finder.contains(TIMEOUT), "N2: timeout line stuck");
    support::drive_with_timeout(
        &mut s,
        &[
            "type:Top files on this Mac",
            "enter",
            &format!("wait:{TIMEOUT}"),
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "03-files-again");
    let back = live_text(&mut s);
    assert!(back.contains("Top files · Spotlight"), "page intact");
    eprintln!("h4 parity_platforms: boot, page roundtrip");
}

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

    assert_line_has(&boot, "›Tasks", "System", "V1 explore");

    assert!(boot.contains("System · devbox"), "V2: no devbox shelf");

    assert_line_has(&boot, "Host", "devbox · local", "S1 host");

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
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-gate1");
    let gate = live_text(&mut s);

    assert_line_has(&gate, "Review · trash 1 item", GATE1, "flow title");

    assert!(
        gate.contains("1 item selected · 124.0 KiB"),
        "S2: no selection size"
    );

    assert!(!gate.contains(GATE2), "N1: gate 2 open before Right+Enter");

    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-gate2-presence");
    support::press_step(&s, "escape");
    wait_absent(&mut s, GATE2, "gate 2 cancelled");
    wait_absent(&mut s, "to confirm", "prompt fully closed");
    waits::wait_state(&mut s, GATE1, "gate 1 back");
    support::drive_with_timeout(
        &mut s,
        &["right", "enter", &format!("wait:{GATE2}")],
        case.timeout_ms,
    );
    support::press_step(&s, "escape");
    wait_absent(&mut s, GATE2, "gate 2 cancelled again");
    wait_absent(&mut s, "to confirm", "prompt fully closed");
    let back = live_text(&mut s);

    assert!(!back.contains(GATE2), "N2: gate 2 stuck");
    assert!(!back.contains("to confirm"), "N2: phrase prompt stuck");
    assert_line_has(&back, "Review · trash 1 item", GATE1, "gate 1 intact");
    checkpoint(&mut s, &dir, "03-gate1-again");
    eprintln!("h4 parity_platforms_linux: boot, gate roundtrip x2");
}

#[test]
#[ignore = "holla h4 check; run with --ignored"]
fn h4_parity_task_input() {
    const PASSWORD: &str = "Password:";
    let dir = h4_dir("h4_parity_task_input");
    let case = h4_case_t("parity_task_input", TASKINPUT, BOOT, 20_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    waits::wait_state(&mut s, "Run the stubborn worker", "suggestions settled");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    assert_line_has(&boot, "› Follow payments-worker", "restarting", "V1 row");

    assert!(
        boot.contains("Run the stubborn worker"),
        "V2: no custom shelf"
    );

    assert_line_has(&boot, "Scope", "on devbox", "S1 scope");

    support::drive_with_timeout(
        &mut s,
        &["type:Deploy the release", "enter", "wait:Password:"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-password");
    let password = live_text(&mut s);

    assert!(
        password.contains("waiting for input"),
        "S2: no waiting title"
    );

    assert!(!password.contains(PLACEHOLDER), "N1: placeholder stuck");

    support::press_step(&s, "escape");
    wait_absent(&mut s, PASSWORD, "output left");
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-finder");
    let finder = live_text(&mut s);

    assert!(!finder.contains(PASSWORD), "N2: password prompt stuck");
    support::drive_with_timeout(
        &mut s,
        &["type:Deploy the release", "enter", "wait:Password:"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "03-password-again");
    let back = live_text(&mut s);
    assert!(back.contains("waiting for input"), "prompt intact");
    eprintln!("h4 parity_task_input: boot, prompt roundtrip");
}
