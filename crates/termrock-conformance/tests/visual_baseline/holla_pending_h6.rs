use std::path::{Path, PathBuf};

use crate::support::Session;
use crate::support::state_waits as waits;
use crate::support::{self, Case, Color, HOLLA};

const BOOT: &str = "holla❯";
const PLACEHOLDER: &str = "Search actions and resources…";
const TYPE_SEARCH: &str = "Type Search";

const RUST_P: &[&str] = &[
    "--scenario",
    "rust-dirty",
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

fn h6_case(slug: &str, args: &'static [&'static str], boot: &'static str) -> Case {
    h6_case_geom(slug, args, boot, 120, 40)
}

fn h6_case_geom(
    slug: &str,
    args: &'static [&'static str],
    boot: &'static str,
    cols: u16,
    rows: u16,
) -> Case {
    Case::dynamic(
        format!("journeys/holla/h6/{slug}"),
        HOLLA,
        args,
        cols,
        rows,
        Color::Truecolor,
        boot,
    )
}

fn h6_case_t(
    slug: &str,
    args: &'static [&'static str],
    boot: &'static str,
    timeout_ms: u64,
) -> Case {
    h6_case(slug, args, boot).timeout(timeout_ms)
}

fn h6_dir(test: &str) -> PathBuf {
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
        "profile: tuiscotti-default\ntransport: pty\nport_of: 75afd0a681869ed59b705ae576314dd1a57306de\nbinary: {}\nsha256: {}\nsize_bytes: {}\nmodified_unix: {modified}\nargv: {}\n",
        subject.path.display(),
        subject.sha256,
        subject.len,
        argv.join(" "),
    );
    std::fs::write(dir.join("provenance.txt"), &body)
        .unwrap_or_else(|e| panic!("write {}: {e}", dir.join("provenance.txt").display()));
    eprintln!("h6 provenance: {body}");
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
#[ignore = "holla h6 check; run with --ignored"]
fn h6_alternatives() {
    const ALT_ENTER: &str = "type:\u{1b}\r";
    const PIN: &str = "Pin here";
    let dir = h6_dir("h6_alternatives");
    let case = h6_case("alternatives", RUST_P, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    waits::wait_state(&mut s, TYPE_SEARCH, "boot settled");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");
    assert!(boot.contains(TYPE_SEARCH), "pre: boot has no finder footer");

    assert_line_has(
        &boot,
        "› Inspect 12 GB of generat…",
        "12 GB generated artifacts",
        "V1 cursor",
    );

    assert_line_has(
        &boot,
        "Who is blocking the data…",
        "2 sessions blocked for 4 min",
        "V2 row",
    );

    assert_line_has(&boot, "In", "~/work/holla", "S1 world");

    support::drive_with_timeout(
        &mut s,
        &[ALT_ENTER, &format!("wait:{PIN}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-menu");
    let menu = live_text(&mut s);

    assert!(
        menu.contains("Inspect 12 GB of generated artifacts"),
        "flow: no menu title"
    );

    assert_line_has(&menu, "Open", "Enter", "flow open");
    assert_line_has(&menu, "Copy command", "y", "flow copy");

    assert!(
        menu.contains("↑↓ Move  Enter Choose  Esc Close"),
        "S2: no menu footer"
    );

    assert!(!menu.contains(TYPE_SEARCH), "N1: finder footer stuck");

    support::press_step(&s, "escape");
    wait_absent(&mut s, PIN, "menu closed");
    waits::wait_state(&mut s, PLACEHOLDER, "finder back");
    checkpoint(&mut s, &dir, "02-finder");
    let finder = live_text(&mut s);

    assert!(!finder.contains("Why is this here?"), "N2: menu row stuck");
    support::drive_with_timeout(
        &mut s,
        &[ALT_ENTER, &format!("wait:{PIN}")],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "03-menu-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "Open", "Enter", "menu intact");
    eprintln!("h6 alternatives: menu, close roundtrip");
}

#[test]
#[ignore = "holla h6 check; run with --ignored"]
fn h6_help_overlay() {
    const TITLE: &str = "Key reference";
    let dir = h6_dir("h6_help_overlay");
    let case = h6_case("help_overlay", RUST_P, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    waits::wait_state(&mut s, TYPE_SEARCH, "boot settled");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");
    assert!(boot.contains(TYPE_SEARCH), "pre: boot has no finder footer");

    assert_line_has(
        &boot,
        "› Inspect 12 GB of generat…",
        "12 GB generated artifacts",
        "V1 cursor",
    );

    assert_line_has(&boot, "In", "~/work/holla", "S1 world");

    support::drive_with_timeout(&mut s, &["f1", "wait:Key reference"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-help");
    let help = live_text(&mut s);

    assert_line_has(&help, TITLE, "Here", "flow title");

    assert_line_has(
        &help,
        "Type",
        "search actions and resources together",
        "flow finder",
    );

    assert_line_has(&help, "Ctrl+Q", "quit with confirmation", "flow quit");

    assert!(help.contains("↑↓ Scroll  Esc Close"), "S2: no help footer");

    assert!(!help.contains(TYPE_SEARCH), "N1: finder footer stuck");

    support::press_step(&s, "escape");
    wait_absent(&mut s, TITLE, "overlay closed");
    waits::wait_state(&mut s, PLACEHOLDER, "finder back");
    checkpoint(&mut s, &dir, "02-finder");
    let finder = live_text(&mut s);

    assert!(
        !finder.contains("quit with confirmation"),
        "N2: help row stuck"
    );
    support::drive_with_timeout(&mut s, &["f1", "wait:Key reference"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-help-again");
    let back = live_text(&mut s);
    assert_line_has(&back, TITLE, "Here", "overlay intact");
    eprintln!("h6 help_overlay: overlay, close roundtrip");
}

#[test]
#[ignore = "holla h6 check; run with --ignored"]
fn h6_menu_file() {
    const CLOSE_TAB: &str = "Close tab";
    let dir = h6_dir("h6_menu_file");
    let case = h6_case("menu_file", RUST_P, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    waits::wait_state(&mut s, TYPE_SEARCH, "boot settled");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");
    assert!(boot.contains(TYPE_SEARCH), "pre: boot has no finder footer");

    assert_line_has(
        &boot,
        "› Inspect 12 GB of generat…",
        "12 GB generated artifacts",
        "V1 cursor",
    );

    assert_line_has(&boot, "In", "~/work/holla", "S1 world");

    support::drive_with_timeout(&mut s, &["f10", "wait:Alternatives…"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-menu");
    let menu = live_text(&mut s);

    assert_line_has(&menu, "Run", "Enter", "flow run");
    assert_line_has(&menu, "Alternatives…", "Alt+Enter", "flow alt");
    assert_line_has(&menu, CLOSE_TAB, "Ctrl+W", "flow close");
    assert_line_has(&menu, "Quit", "Ctrl+Q", "flow quit");

    assert!(
        menu.contains("← → Menu  ↑↓ Move  Enter Choose  Esc Close"),
        "S2: no menu footer"
    );

    assert!(!menu.contains(TYPE_SEARCH), "N1: finder footer stuck");

    support::press_step(&s, "escape");
    wait_absent(&mut s, CLOSE_TAB, "menu closed");
    waits::wait_state(&mut s, PLACEHOLDER, "finder back");
    checkpoint(&mut s, &dir, "02-finder");
    let finder = live_text(&mut s);

    assert!(!finder.contains("Alternatives…"), "N2: menu row stuck");
    support::drive_with_timeout(&mut s, &["f10", "wait:Alternatives…"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-menu-again");
    let back = live_text(&mut s);
    assert_line_has(&back, CLOSE_TAB, "Ctrl+W", "menu intact");
    eprintln!("h6 menu_file: menu, close roundtrip");
}

#[test]
#[ignore = "holla h6 check; run with --ignored"]
fn h6_menu_go() {
    const PARENT: &str = "Parent scope";
    let dir = h6_dir("h6_menu_go");
    let case = h6_case("menu_go", RUST_P, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    waits::wait_state(&mut s, TYPE_SEARCH, "boot settled");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");
    assert!(boot.contains(TYPE_SEARCH), "pre: boot has no finder footer");

    assert_line_has(
        &boot,
        "› Inspect 12 GB of generat…",
        "12 GB generated artifacts",
        "V1 cursor",
    );

    assert_line_has(&boot, "In", "~/work/holla", "S1 world");

    support::drive_with_timeout(
        &mut s,
        &["f10", "right", "wait:Parent scope"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-menu");
    let menu = live_text(&mut s);

    assert_line_has(&menu, "Here", "Alt+0", "flow here");
    assert_line_has(&menu, "Activities…", "Ctrl+G", "flow activities");
    assert_line_has(&menu, PARENT, "Ctrl+", "flow parent");

    assert!(menu.contains("Children scope"), "flow: no children row");
    assert!(menu.contains("System scope"), "flow: no system row");

    assert!(
        menu.contains("← → Menu  ↑↓ Move  Enter Choose  Esc Close"),
        "S2: no menu footer"
    );

    assert!(!menu.contains(TYPE_SEARCH), "N1: finder footer stuck");

    support::press_step(&s, "escape");
    wait_absent(&mut s, PARENT, "menu closed");
    waits::wait_state(&mut s, PLACEHOLDER, "finder back");
    checkpoint(&mut s, &dir, "02-finder");
    let finder = live_text(&mut s);

    assert!(!finder.contains("Children scope"), "N2: menu row stuck");
    support::drive_with_timeout(
        &mut s,
        &["f10", "right", "wait:Parent scope"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "03-menu-again");
    let back = live_text(&mut s);
    assert_line_has(&back, PARENT, "Ctrl+", "menu intact");
    eprintln!("h6 menu_go: menu, close roundtrip");
}

#[test]
#[ignore = "holla h6 check; run with --ignored"]
fn h6_quit_confirm() {
    const TITLE: &str = "Quit holla❯?";
    const STOP: &str = "Stop and quit";
    let dir = h6_dir("h6_quit_confirm");
    let case = h6_case_t("quit_confirm", ACTMULTI, BOOT, 20_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    waits::wait_state(&mut s, TYPE_SEARCH, "boot settled");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");
    assert!(boot.contains(TYPE_SEARCH), "pre: boot has no finder footer");

    assert_line_has(&boot, "frontend dev ⠋ ×", "btm ○ ×", "V1 strip");

    assert_line_has(&boot, "› Deploy preview", "Current folder", "V2 cursor");

    assert!(boot.contains("4 running"), "S1: no running count");

    support::drive_with_timeout(&mut s, &["ctrl-q", "wait:still running"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-dialog");
    let dialog = live_text(&mut s);

    assert!(dialog.contains(TITLE), "flow: no dialog title");

    assert_line_has(
        &dialog,
        "4 activities are still running",
        "Quitting stops",
        "flow body",
    );

    assert_line_has(&dialog, "Cancel", STOP, "flow buttons");

    assert!(
        dialog.contains("y / n Quick answer"),
        "S2: no dialog footer"
    );

    assert!(!dialog.contains(TYPE_SEARCH), "N1: finder footer stuck");

    support::press_step(&s, "escape");
    wait_absent(&mut s, STOP, "dialog cancelled");
    waits::wait_state(&mut s, PLACEHOLDER, "finder back");
    checkpoint(&mut s, &dir, "02-finder");
    let finder = live_text(&mut s);

    assert!(!finder.contains(TITLE), "N2: dialog title stuck");
    support::drive_with_timeout(&mut s, &["ctrl-q", "wait:still running"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-dialog-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "Cancel", STOP, "dialog intact");
    eprintln!("h6 quit_confirm: dialog, cancel roundtrip");
}

#[test]
#[ignore = "holla h6 check; run with --ignored"]
fn h6_resize_grown() {
    const NARROW: &str = "truecolor · 80×24";
    const WIDE: &str = "truecolor · 120×40";
    const TRUNC: &str = "Who is blocking the datab…";
    const FULL: &str = "Who is blocking the data…";
    let dir = h6_dir("h6_resize_grown");
    let case = h6_case_geom("resize_grown", RUST_P, BOOT, 80, 24).timeout(30_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot-narrow");

    waits::wait_state(&mut s, NARROW, "boot settled");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    assert!(boot.contains(TRUNC), "V1: no truncated row");

    assert!(boot.contains(NARROW), "S1: no narrow readout");

    s.resize(120, 40).expect("resize to 120x40");
    waits::wait_size(&mut s, 120, 40, "grown geometry");
    waits::wait_state(&mut s, WIDE, "grown repaint");
    checkpoint(&mut s, &dir, "01-grown");
    let grown = live_text(&mut s);

    assert!(grown.contains(FULL), "flow: no full row");

    assert!(grown.contains("Ctrl+G Activities"), "S2: no wide footer");

    assert!(!grown.contains(TRUNC), "N1: truncated row stuck");

    assert!(!grown.contains(NARROW), "N2: narrow readout stuck");

    s.resize(80, 24).expect("resize to 80x24");
    waits::wait_size(&mut s, 80, 24, "narrow geometry");
    waits::wait_state(&mut s, NARROW, "narrow repaint");
    checkpoint(&mut s, &dir, "02-narrow-again");
    let back = live_text(&mut s);
    assert!(back.contains(TRUNC), "narrow shape intact");
    eprintln!("h6 resize_grown: grow reflow, shrink stance");
}

#[test]
#[ignore = "holla h6 check; run with --ignored"]
fn h6_resize_shrunk() {
    const NARROW: &str = "truecolor · 80×24";
    const WIDE: &str = "truecolor · 120×40";
    const TRUNC: &str = "Who is blocking the datab…";
    const FULL: &str = "Who is blocking the data…";
    let dir = h6_dir("h6_resize_shrunk");
    let case = h6_case("resize_shrunk", RUST_P, BOOT).timeout(30_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot-wide");

    waits::wait_state(&mut s, WIDE, "boot settled");
    let boot = live_text(&mut s);
    assert!(boot.contains(PLACEHOLDER), "pre: boot has no placeholder");

    assert!(boot.contains(FULL), "V1: no full row");

    assert!(boot.contains(WIDE), "S1: no wide readout");

    s.resize(80, 24).expect("resize to 80x24");
    waits::wait_size(&mut s, 80, 24, "shrunk geometry");
    waits::wait_state(&mut s, NARROW, "shrunk repaint");
    checkpoint(&mut s, &dir, "01-shrunk");
    let shrunk = live_text(&mut s);

    assert!(shrunk.contains(TRUNC), "flow: no truncated row");

    assert!(
        shrunk.contains("→ 'holla disk analyze --artifacts' · here · read-only"),
        "S2: no compact action line"
    );

    assert!(
        !shrunk.contains("Ctrl+G Activities"),
        "N1: wide footer stuck"
    );

    assert!(!shrunk.contains(WIDE), "N2: wide readout stuck");

    s.resize(120, 40).expect("resize to 120x40");
    waits::wait_size(&mut s, 120, 40, "wide geometry");
    waits::wait_state(&mut s, WIDE, "wide repaint");
    checkpoint(&mut s, &dir, "02-wide-again");
    let back = live_text(&mut s);
    assert!(back.contains(FULL), "wide shape intact");
    eprintln!("h6 resize_shrunk: shrink reflow, grow stance");
}
