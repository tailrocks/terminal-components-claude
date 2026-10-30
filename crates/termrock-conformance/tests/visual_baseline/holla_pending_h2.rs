use std::path::{Path, PathBuf};

use crate::support::Session;

use crate::support::state_waits as waits;
use crate::support::{self, Case, Color, HOLLA};

const BOOT: &str = "holla❯";
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

fn h2_case_t(
    slug: &str,
    args: &'static [&'static str],
    boot: &'static str,
    timeout_ms: u64,
) -> Case {
    h2_case(slug, args, boot).timeout(timeout_ms)
}

fn h2_dir(test: &str) -> PathBuf {
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

fn assert_line_has(text: &str, needle: &str, also: &str, what: &str) {
    assert!(
        text.lines().any(|l| l.contains(needle) && l.contains(also)),
        "{what}: no line contains both `{needle}` and `{also}`"
    );
}

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

    assert!(listing.contains("hidden off"), "pre: no hidden-off status");

    support::drive_with_timeout(&mut s, &["ctrl-h", "wait:2 hidden shown"], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-hidden");
    let hidden = live_text(&mut s);

    assert_line_has(&hidden, "18 entries", "2 hidden shown", "V1 header");

    assert_line_has(&hidden, ".git", "directory", "V2 dotfile");

    assert!(
        hidden.contains("Hidden entries shown"),
        "S1: no shown footer"
    );

    assert!(
        hidden.contains("~/work/site · hidden shown"),
        "S2: no hidden-shown status"
    );

    assert!(
        !hidden.contains("hidden off"),
        "N1: hidden-off status stuck"
    );

    support::drive_with_timeout(&mut s, &["ctrl-h", "wait:2 hidden hidden"], case.timeout_ms);
    let off = live_text(&mut s);

    assert!(!off.contains("2 hidden shown"), "N2: shown count stuck");
    support::drive_with_timeout(&mut s, &["ctrl-h", "wait:2 hidden shown"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-hidden-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "18 entries", "2 hidden shown", "header intact");
    eprintln!("h2 browser_hidden: hidden toggle roundtrip x2");
}

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

    support::drive_with_timeout(&mut s, &["type:test", "wait:Results · 17"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    assert_line_has(&res, "› Run tests", "task", "V1 row");

    assert!(res.contains("Results · 17"), "V2: no Results · 17");

    assert!(res.contains("Enter Trust…"), "S1: no trust footer");

    assert!(res.contains("scope ‹ here ›"), "S2: no here scope");

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
    eprintln!("h2 child_query: results, clear roundtrip");
}

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

    support::drive_with_timeout(
        &mut s,
        &["type:Custom action configuration", "enter", "wait:sha256:"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-page");
    let page = live_text(&mut s);

    assert!(page.contains(DIGEST), "V1: no config digest");

    assert_line_has(&page, "Trust", "Untrusted", "V2 trust");

    assert!(page.contains("Esc Back"), "S1: no back footer");

    assert!(page.contains("1 · 2 diagnostics"), "S2: no action count");

    assert!(!page.contains(PLACEHOLDER), "N1: placeholder stuck");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "sha256:", "page closed");
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-finder");
    let finder = live_text(&mut s);

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

    support::drive_with_timeout(&mut s, &["enter", "wait:Open in the OS"], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-menu");
    let menu = live_text(&mut s);

    assert_line_has(&menu, "Open in the OS", "Enter", "V1 choice");

    assert!(
        menu.contains("/Users/alex/work/notes/todo.md"),
        "V2: no menu title"
    );

    assert!(menu.contains("Esc Close"), "S1: no close footer");

    assert!(
        menu.contains("1 result · home scope"),
        "S2: no result count"
    );

    assert!(!menu.contains("Enter Actions"), "N1: find footer stuck");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Open in the OS", "menu closed");
    waits::wait_state(&mut s, "Enter Actions", "find footer back");
    checkpoint(&mut s, &dir, "03-closed");
    let closed = live_text(&mut s);

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

    support::drive_with_timeout(&mut s, &["g", "wait:Enter Switch"], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-picker");
    let picker = live_text(&mut s);

    assert_line_has(&picker, "Go to path", "~/work/site", "V1 title");

    assert!(
        picker.contains("type a path or a name"),
        "V2: no picker invitation"
    );

    assert!(picker.contains("Enter Switch"), "S1: no switch footer");

    assert!(
        picker.contains("relative to this folder"),
        "S2: no relative-path hint"
    );

    assert!(
        !picker.contains("Open / preview"),
        "N1: browser footer stuck"
    );

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Enter Switch", "picker cancelled");
    waits::wait_state(&mut s, "Open / preview", "browser footer back");
    checkpoint(&mut s, &dir, "03-cancelled");
    let cancelled = live_text(&mut s);

    assert!(!cancelled.contains("Enter Switch"), "N2: picker stuck");
    support::drive_with_timeout(&mut s, &["g", "wait:Enter Switch"], case.timeout_ms);
    checkpoint(&mut s, &dir, "04-picker-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "Go to path", "~/work/site", "picker intact");
    eprintln!("h2 files_jump: picker, cancel roundtrip");
}

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

    assert!(
        error.contains("nope: no such file or directory"),
        "V1: no picker error"
    );

    assert!(
        error.contains("edit the path or choose a suggestion"),
        "V2: no recovery hint"
    );

    assert!(error.contains("Enter Switch"), "S1: picker closed on error");

    assert!(error.contains("▎ nope"), "S2: bad input lost");

    assert!(!error.contains("Jumped to"), "N1: jumped on bad path");

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

    assert!(
        !jumped.contains("no such file or directory"),
        "N2: picker error stuck"
    );
    eprintln!("h2 files_jump_error: error, recovery, good jump");
}

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

    support::drive_with_timeout(
        &mut s,
        &["type:~/work", "enter", "wait:Jumped to ~/work"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-jumped");
    let jumped = live_text(&mut s);

    assert!(jumped.contains("Files · ~/work"), "V1: no jumped header");
    assert!(jumped.contains("1 entry"), "V1: no single entry");

    assert!(jumped.contains("Jumped to ~/work"), "V2: no jump status");

    assert!(jumped.contains("Files › work"), "S1: no jumped breadcrumb");

    assert!(jumped.contains("hidden off"), "S2: no hidden status");

    assert!(!jumped.contains("Enter Switch"), "N1: picker stuck");

    support::drive_with_timeout(&mut s, &["g", "wait:Enter Switch"], case.timeout_ms);
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Enter Switch", "picker cancelled");
    waits::wait_state(&mut s, "Files · ~/work", "jumped folder intact");
    checkpoint(&mut s, &dir, "03-cancelled");
    let back = live_text(&mut s);

    assert!(!back.contains("16 entries"), "N2: old listing stuck");
    assert!(back.contains("Files · ~/work"), "folder intact");
    assert!(back.contains("1 entry"), "count intact");
    eprintln!("h2 files_jumped: jump, picker roundtrip");
}

#[test]
#[ignore = "holla h2 check; run with --ignored"]
fn h2_files_preview_control() {
    const TITLE: &str = "~/work/site/control.txt";
    let dir = h2_dir("h2_files_preview_control");
    let case = h2_case("files_preview_control", BROWSER, BOOT);
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

    assert_line_has(&control, TITLE, "13 B of 13 B", "V1 preview");

    assert!(control.contains("red"), "V2: no preview body");

    assert_line_has(&control, "control.txt", "13 B", "S1 cursor");

    assert!(control.contains("Esc Back"), "S2: no back footer");

    support::drive_with_timeout(&mut s, &["down", "sleep:300"], case.timeout_ms);
    let moved = live_text(&mut s);

    assert!(!moved.contains(TITLE), "N1: control preview stuck");
    support::drive_with_timeout(&mut s, &["up", &format!("wait:{TITLE}")], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-control-again");
    let back = live_text(&mut s);

    assert!(
        !back.contains("~/work/site/current"),
        "N2: moved-to preview stuck"
    );
    assert_line_has(&back, TITLE, "13 B of 13 B", "preview intact");
    eprintln!("h2 files_preview_control: preview, cursor roundtrip");
}

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

    assert!(res.contains("3 results · home scope"), "V1: no 3 results");

    assert_line_has(&res, "README.md", "~/Documents", "V2 row");

    assert_line_has(&res, FIRST, "7 B of 7 B", "S1 preview");

    assert!(res.contains("Enter Actions"), "S2: no actions footer");

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
    eprintln!("h2 files_results: results, cursor roundtrip");
}

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

    assert!(res.contains("1 result · home scope"), "V1: no 1 result");

    assert_line_has(&res, "café menu.txt", "~/Documents", "V2 row");

    assert_line_has(&res, "café menu.txt", "4 B of 4 B", "S1 preview");

    assert!(res.contains("☕"), "S2: no preview body");

    assert!(!res.contains(PLACEHOLDER), "N1: placeholder stuck");

    support::press_step(&s, "ctrl-u");
    waits::wait_gone(&mut s, "1 result", "query cleared");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);

    assert!(!cleared.contains("1 result"), "N2: result stuck");
    support::drive_with_timeout(&mut s, &["type:café", "wait:1 result"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-result-again");
    let back = live_text(&mut s);
    assert_line_has(&back, "café menu.txt", "~/Documents", "row intact");
    eprintln!("h2 files_unicode: unicode result, clear roundtrip");
}

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

    support::drive_with_timeout(&mut s, &["type:pull", "wait:Results · 3"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    assert_line_has(&res, "› Pull", "up to date", "V1 row");

    assert!(res.contains("Results · 3"), "V2: no Results · 3");

    assert!(res.contains("Enter Run"), "S1: no run footer");

    assert!(res.contains("scope ‹ here ›"), "S2: no here scope");

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
    eprintln!("h2 finder_query: results, clear roundtrip");
}

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

    support::drive_with_timeout(&mut s, &["ctrl-a", "wait:Query selected"], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-selected");
    let sel = live_text(&mut s);

    assert!(
        sel.contains("Query selected · typing replaces it"),
        "V1: no selected status"
    );

    assert!(sel.contains("Results · 3"), "V2: no Results · 3");

    assert!(sel.contains("▎ pull"), "S1: no selected query");

    assert!(sel.contains("scope ‹ here ›"), "S2: no here scope");

    assert!(!sel.contains("Esc Clear"), "N1: clear footer stuck");

    support::press_step(&s, "escape");
    support::drive_with_timeout(&mut s, &["type:!", "wait:▎ pull!"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-appended");
    let appended = live_text(&mut s);
    assert!(appended.contains("▎ pull!"), "appended: no pull!");
    support::drive_with_timeout(&mut s, &["ctrl-z", "wait:Undone"], case.timeout_ms);
    let reunited = live_text(&mut s);

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

#[test]
#[ignore = "holla h2 check; run with --ignored"]
fn h2_query_empty_preview() {
    let dir = h2_dir("h2_query_empty_preview");
    let case = h2_case("query_empty_preview", REMOTE, BOOT);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

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

    assert!(
        empty.contains("No matches for “blockingresources”"),
        "V1: no no-matches row"
    );

    assert!(empty.contains("Nothing to preview"), "V2: no Empty preview");

    assert!(
        empty.contains("Cancelled · nothing was executed"),
        "S1: no cancelled status"
    );

    assert!(
        empty.contains("▎ blockingresources"),
        "S2: no matchless query"
    );

    assert!(!empty.contains("Who is blocking"), "N1: dialog stuck");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "No matches for", "query cleared");
    waits::wait_state(&mut s, PLACEHOLDER, "placeholder back");
    checkpoint(&mut s, &dir, "03-cleared");
    let cleared = live_text(&mut s);

    assert!(!cleared.contains("blockingresources"), "N2: query stuck");

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

    support::drive_with_timeout(
        &mut s,
        &["ctrl-y", "ctrl-y", "ctrl-y", "ctrl-y", "wait:Redone"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-redone");
    let redone = live_text(&mut s);

    assert!(redone.contains("Results · 3"), "V1: no Results · 3");

    assert!(redone.contains("Redone"), "V2: no Redone status");

    assert_line_has(&redone, "› Pull", "up to date", "S1 row");

    assert!(redone.contains("scope ‹ here ›"), "S2: no here scope");

    assert!(!redone.contains("Undone"), "N1: Undone stuck");

    support::drive_with_timeout(&mut s, &["ctrl-z", "wait:Undone"], case.timeout_ms);
    let undone2 = live_text(&mut s);

    assert!(!undone2.contains("Redone"), "N2: Redone stuck");
    support::drive_with_timeout(&mut s, &["ctrl-y", "wait:Redone"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-redone-again");
    let back = live_text(&mut s);
    assert!(back.contains("Results · 3"), "results intact");
    eprintln!("h2 query_redone: redo, undo roundtrip");
}

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

    support::drive_with_timeout(
        &mut s,
        &["ctrl-z", "ctrl-z", "ctrl-z", "ctrl-z", "wait:Undone"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-undone");
    let undone = live_text(&mut s);

    assert!(undone.contains(PLACEHOLDER), "V1: no placeholder");

    assert!(undone.contains("Undone"), "V2: no Undone status");

    assert!(undone.contains("Suggested here"), "S1: no suggestions");

    assert_line_has(&undone, "Run check", "task", "S2 suggestion");

    assert!(!undone.contains("Results · 3"), "N1: results stuck");

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

    support::drive_with_timeout(
        &mut s,
        &["ctrl-down", "wait:Children · 3 projects"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-children");
    let children = live_text(&mut s);

    assert!(
        children.contains("Children · 3 projects · 11 entries"),
        "V1: no children header"
    );

    assert!(
        children.contains("scope ‹ children ›"),
        "V2: no children scope"
    );

    assert!(
        children.contains("Scope children · 3 projects"),
        "S1: no children footer"
    );

    assert_line_has(&children, "Clean developer", "destructive", "S2 row");

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
    assert!(back.contains("scope ‹ children ›"), "children intact");
    eprintln!("h2 scope_children: widen, narrow roundtrip");
}

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

    support::drive_with_timeout(
        &mut s,
        &["ctrl-up", "wait:No project root above"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-parent");
    let parent = live_text(&mut s);

    assert!(
        parent.contains("No project root above ~/work/acme"),
        "V1: no rootless row"
    );

    assert!(parent.contains("scope ‹ parent ›"), "V2: no parent scope");

    assert!(
        parent.contains("Scope parent · no root above"),
        "S1: no parent footer"
    );

    assert!(
        parent.contains("Nothing to preview"),
        "S2: no Empty preview"
    );

    assert!(!parent.contains("scope ‹ here ›"), "N1: here scope stuck");

    support::drive_with_timeout(&mut s, &["escape", "wait:scope ‹ here ›"], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-here");
    let here = live_text(&mut s);

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

    support::drive_with_timeout(
        &mut s,
        &["ctrl-up", "ctrl-up", "wait:scope ‹ system ›"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-system");
    let system = live_text(&mut s);

    assert!(
        system.contains("System · mbp · 45 entries"),
        "V1: no system header"
    );

    assert!(system.contains("scope ‹ system ›"), "V2: no system scope");

    assert!(
        system.contains("Scope system · mbp"),
        "S1: no system footer"
    );

    assert_line_has(&system, "Review cleanup candidates", "10.6 GiB", "S2 row");

    assert!(!system.contains("scope ‹ here ›"), "N1: here scope stuck");

    support::drive_with_timeout(&mut s, &["escape", "wait:scope ‹ here ›"], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-here");
    let here = live_text(&mut s);

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

#[test]
#[ignore = "holla h2 check; run with --ignored"]
fn h2_task_input_cancelled() {
    const CANCELLED: &str = "rollout cancelled by operator";
    let dir = h2_dir("h2_task_input_cancelled");
    let case = h2_case_t("task_input_cancelled", TASKINPUT, BOOT, 20_000);
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

    assert!(cancelled.contains(CANCELLED), "V1: no cancelled line");

    assert_line_has(&cancelled, "deploy the release", "exit 2", "V2 title");

    assert!(cancelled.contains("authenticated"), "S1: no auth line");

    assert!(
        cancelled.contains("failed · exit 2"),
        "S2: no failed footer"
    );

    support::press_step(&s, "r");
    waits::wait_gone(&mut s, CANCELLED, "output restarted");
    waits::wait_state(&mut s, "Password:", "password prompt back");
    let restarted = live_text(&mut s);

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

    assert!(!here.contains(CANCELLED), "N2: cancelled line stuck");
    eprintln!("h2 task_input_cancelled: cancel, restart roundtrip");
}

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

    assert!(
        killed.contains("killed (SIGKILL) · descendants reaped"),
        "V1: no killed line"
    );

    assert_line_has(&killed, "the stubborn worker", "exit 137", "V2 title");

    assert!(
        killed.contains("SIGKILL to the process group"),
        "S1: no escalation line"
    );

    assert!(
        killed.contains("the stubborn worker stopped · exit 137"),
        "S2: no stopped footer"
    );

    assert!(!killed.contains(CANCELLED), "N1: deploy output leaked");

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

    assert!(!here.contains(KILLED), "N2: killed line stuck");
    eprintln!("h2 task_input_killed: kill, restart roundtrip");
}

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

    support::drive_with_timeout(&mut s, &["type:yarn", "wait:yarn s00"], case.timeout_ms);
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    assert!(res.contains("Results · 31"), "V1: no Results · 31");

    assert_line_has(&res, "› yarn dev", "package.json here", "V2 row");

    assert!(res.contains("yarn run dev"), "S1: no run command");

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
    eprintln!("h2 task_sources: results, clear roundtrip");
}

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

    support::drive_with_timeout(
        &mut s,
        &["type:Taskfile", "wait:discovery diagnostic"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-results");
    let res = live_text(&mut s);

    assert!(res.contains("Results · 1"), "V1: no Results · 1");

    assert_line_has(
        &res,
        "Taskfile · Taskfile.yaml",
        "discovery diagnostic",
        "V2 row",
    );

    assert!(
        res.contains("mapping values are not"),
        "S1: no yaml failure"
    );

    assert_line_has(&res, "Risk", "read-only", "S2 risk");

    assert!(!res.contains(PLACEHOLDER), "N1: placeholder stuck");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Taskfile · Taskfile.yaml", "result cleared");
    waits::wait_state(&mut s, PLACEHOLDER, "query cleared");
    checkpoint(&mut s, &dir, "02-cleared");
    let cleared = live_text(&mut s);

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
