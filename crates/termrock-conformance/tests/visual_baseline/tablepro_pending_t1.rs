use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tuiscotti::render::frame_from_screen;
use tuiscotti::tui::{CancelToken, Wheel};
use tuiscotti::{Color as TColor, Frame, Rgb};

use crate::support::Session;
use crate::support::state_waits as waits;
use crate::support::typed_input::Input;
use crate::support::{self, Case, Color, TABLEPRO};

const BOOT_CONN: &str = "Production";
const BOOT_WB: &str = "Query 1";

fn t1_case_conn(slug: &str) -> Case {
    Case::dynamic(
        format!("journeys/tablepro/t1/{slug}"),
        TABLEPRO,
        &[],
        120,
        40,
        Color::Truecolor,
        BOOT_CONN,
    )
}

fn t1_case_wb(slug: &str) -> Case {
    Case::dynamic(
        format!("journeys/tablepro/t1/{slug}"),
        TABLEPRO,
        &["--connect", "Production"],
        120,
        40,
        Color::Truecolor,
        BOOT_WB,
    )
}

fn t1_case_size(
    slug: &str,
    args: &'static [&'static str],
    cols: u16,
    rows: u16,
    boot: &'static str,
) -> Case {
    Case::dynamic(
        format!("journeys/tablepro/t1/{slug}"),
        TABLEPRO,
        args,
        cols,
        rows,
        Color::Truecolor,
        boot,
    )
}

fn t1_dir(test: &str) -> PathBuf {
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
        "profile: tuiscotti-default\ntransport: pty\nport_of: 07a56acf38ec7335d796c36c9d8ed03d4b1001d2\nbinary: {}\nsha256: {}\nsize_bytes: {}\nmodified_unix: {modified}\nargv: {}\n",
        subject.path.display(),
        subject.sha256,
        subject.len,
        argv.join(" "),
    );
    std::fs::write(dir.join(file), &body)
        .unwrap_or_else(|e| panic!("write {}: {e}", dir.join(file).display()));
    eprintln!("t1 provenance ({file}): {body}");
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
    let screen = s
        .inner
        .snapshot()
        .unwrap_or_else(|e| panic!("live frame sample failed: {e:#}"));
    frame_from_screen(&screen, "default")
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

fn wheel_until_stable(
    s: &mut Session,
    col: u16,
    row: u16,
    dir: Wheel,
    timeout: Duration,
    what: &str,
) -> String {
    let deadline = Instant::now() + timeout;
    let mut prev = live_text(s);
    for _ in 0..10 {
        wheel_at(s, col, row, dir, 5);
        std::thread::sleep(Duration::from_millis(400));
        let cur = live_text(s);
        if cur == prev {
            return cur;
        }
        assert!(
            Instant::now() < deadline,
            "{what}: never reached the boundary"
        );
        prev = cur;
    }
    panic!("{what}: still moving after 10 batches");
}

fn resize_to(s: &mut Session, cols: u16, rows: u16, timeout: Duration) {
    s.resize(cols, rows).expect("resize");
    support::wait_screen(
        s,
        timeout,
        &format!("never reached {cols}x{rows}"),
        |screen| screen.cols() == cols && screen.rows() == rows,
    );
    std::thread::sleep(Duration::from_millis(700));
}

fn expect_exit_text(s: &Session, timeout: Duration, what: &str) -> String {
    let cancel = CancelToken::new();
    let exit = s
        .inner
        .expect_exit(Instant::now() + timeout, &cancel)
        .unwrap_or_else(|e| panic!("{what}: session never exited: {e:#}"));
    let obs = exit
        .success()
        .unwrap_or_else(|e| panic!("{what}: session did not exit cleanly: {e:#}"));
    support::screen_text(&obs.screen)
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

fn row_fg_median(frame: &Frame, row: u16, what: &str) -> u32 {
    let mut lums: Vec<u32> = Vec::new();
    for col in 0..frame.cols {
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
        "{what}: row {row} has no resolvable-fg text cells"
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

fn assert_line_has(text: &str, needle: &str, also: &str, what: &str) {
    assert!(
        text.lines().any(|l| l.contains(needle) && l.contains(also)),
        "{what}: no line contains both `{needle}` and `{also}`"
    );
}

fn assert_no_grid_sort_marks(text: &str, what: &str) {
    assert!(
        !text.contains('▴'),
        "{what}: ascending sort marker rendered"
    );
    for line in text.lines().filter(|l| l.contains('▾')) {
        assert!(
            line.contains("D acme_prod") || line.contains("S public") || line.contains("Tables"),
            "{what}: non-tree ▾ line renders a sort marker: {line:?}"
        );
    }
}

fn assert_line_lacks(text: &str, needle: &str, also: &str, what: &str) {
    for line in text.lines().filter(|l| l.contains(needle)) {
        assert!(
            !line.contains(also),
            "{what}: `{needle}` line must not contain `{also}`: {line:?}"
        );
    }
}

#[test]
#[ignore = "tablepro t1 check; run with --ignored"]
fn t1_audit_production_boot() {
    let dir = t1_dir("t1_audit_production_boot");
    let case = t1_case_wb("audit_production_boot");
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);

    support::drive_with_timeout(&mut s, &["wait:S audit"], case.timeout_ms);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);

    assert_line_has(&boot, "Query 1", "+", "V1 strip");
    assert!(boot.contains("≡ Query 1 ×"), "V1: strip lacks the tab");
    for needle in ["Production", "◆ production", "acme_prod › public", "safe"] {
        assert!(boot.contains(needle), "V1: header lacks `{needle}`");
    }
    assert_eq!(
        boot.matches('×').count(),
        2,
        "V1: exactly one tab close (plus the 120×40 header size)"
    );

    assert_line_has(&boot, "Explorer", "public", "V2 explorer");
    assert!(
        boot.contains("Type SQL. Ctrl+R runs the statement under the cursor."),
        "V2: editor placeholder missing"
    );
    assert!(
        boot.contains("No results yet"),
        "V2: results empty state missing"
    );

    assert_line_has(&boot, "▎", "S public", "S1 cursor");
    assert_line_has(&boot, "↑ ↓ Move", "Enter Open", "S1 hints");

    for needle in [
        "D acme_prod",
        "S public",
        "Tables",
        "Views",
        "Functions",
        "Sequences",
        "S analytics",
        "S audit",
    ] {
        assert!(boot.contains(needle), "S2: tree lacks `{needle}`");
    }

    for needle in ["Open tabs", "Quit TablePro?", "Save changes?"] {
        assert!(!boot.contains(needle), "N1: boot shows `{needle}`");
    }

    support::press_step(&s, "tab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1 focus left the tree",
        |screen| {
            !support::screen_text(screen)
                .lines()
                .any(|l| l.contains("▎") && l.contains("S public"))
        },
    );
    let focused = live_text(&mut s);
    assert_line_has(&focused, "▎", "Type SQL", "A1 editor focus");
    assert!(
        focused.contains("Enter Edit"),
        "A1: tab hints never appeared"
    );
    checkpoint(&mut s, &dir, "01-tabbed");

    support::drive_with_timeout(&mut s, &["ctrl-g", "wait:Open tabs"], case.timeout_ms);
    checkpoint(&mut s, &dir, "02-tablist");
    support::press_step(&s, "escape");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "N1 tab list never closed",
        |screen| !support::screen_text(screen).contains("Open tabs"),
    );

    support::press_step(&s, "q");
    let exit_text = expect_exit_text(&s, case_timeout(&case), "A2 clean quit");
    assert!(
        !exit_text.contains("Quit TablePro?"),
        "N2: exit frame carries dialog text"
    );
    std::fs::write(
        dir.join("03-exit.txt"),
        format!("# exit frame\n{exit_text}\n"),
    )
    .expect("write 03-exit.txt");
    eprintln!("t1 audit_production_boot: regions proven, clean q exit proven");
}

#[test]
#[ignore = "tablepro t1 check; run with --ignored"]
fn t1_connections_default_boot() {
    let dir = t1_dir("t1_connections_default_boot");
    let case = t1_case_conn("connections_default_boot");
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);

    assert_line_has(&boot, "▎", "Personal", "V1/S1 cursor");
    assert!(boot.contains("▾ Acme"), "V1: Acme group missing");
    for needle in [
        "Local PostgreSQL",
        "Scratch",
        "Development",
        "Staging",
        "Analytics",
        "Production",
        "pg",
        "sqlite",
        "mysql",
    ] {
        assert!(boot.contains(needle), "V1: tree lacks `{needle}`");
    }

    for needle in [
        "Local PostgreSQL",
        "localhost:5432",
        "acme_dev",
        "postgres",
        "Connect",
        "Edit",
        "Duplicate",
        "Delete…",
    ] {
        assert!(boot.contains(needle), "V2: detail lacks `{needle}`");
    }

    assert_line_has(&boot, "▎", "Personal", "S1 group cursor");
    assert!(boot.contains("localhost:5432"), "S1: detail is not Local");

    assert!(boot.contains("6 saved"), "S2: header lacks `6 saved`");

    for needle in ["New connection", "Edit connection", "Delete connection?"] {
        assert!(!boot.contains(needle), "N1/N2: boot shows `{needle}`");
    }
    assert_line_lacks(&boot, "Local PostgreSQL", "Name", "N1 no form field");

    support::press_step(&s, "down");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1 cursor on Local",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains("▎") && l.contains("Local PostgreSQL"))
        },
    );
    let moved = live_text(&mut s);
    assert!(
        moved.contains("localhost:5432"),
        "A1: detail changed under the cursor move"
    );

    support::press_step(&s, "/");
    support::drive_with_timeout(&mut s, &["type:stag"], case.timeout_ms);
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A2 filter never applied",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains("▎") && l.contains("stag"))
        },
    );
    let filtered = live_text(&mut s);
    assert!(filtered.contains("Staging"), "A2: match hidden");
    for gone in ["Scratch", "Personal", "Production", "Development"] {
        assert!(!filtered.contains(gone), "A2: `{gone}` never filtered out");
    }
    assert!(filtered.contains("▾ Acme"), "A2: match ancestor hidden");
    checkpoint(&mut s, &dir, "01-filtered");
    support::press_step(&s, "escape");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A2 filter never cleared",
        |screen| support::screen_text(screen).contains("Filter connections"),
    );

    support::press_step(&s, "down");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "tree never refocused",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains("▎") && l.contains("Personal"))
        },
    );

    support::drive_with_timeout(
        &mut s,
        &[
            "down",
            "down",
            "down",
            "down",
            "down",
            "down",
            "down",
            "d",
            "wait:Delete connection?",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "02-delete-proof");
    support::press_step(&s, "escape");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "N2 dialog never closed",
        |screen| !support::screen_text(screen).contains("Delete connection?"),
    );
    let back = live_text(&mut s);
    assert!(back.contains("6 saved"), "N2: cancel dropped a row");

    support::drive_with_timeout(&mut s, &["ctrl-n", "wait:New connection"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-form-proof");
    let formed = live_text(&mut s);
    assert!(formed.contains("Name"), "N1: form lacks the Name field");
    eprintln!("t1 connections_default_boot: boot proven, dialog+form proven");
}

#[test]
#[ignore = "tablepro t1 check; run with --ignored"]
fn t1_connections_delete_dialog() {
    let dir = t1_dir("t1_connections_delete_dialog");
    let case = t1_case_conn("connections_delete_dialog");
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::drive_with_timeout(
        &mut s,
        &["down", "down", "down", "down", "down", "down", "down"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "00-production");
    support::press_step(&s, "d");
    waits::wait_state(&mut s, "Delete connection?", "delete dialog");
    checkpoint(&mut s, &dir, "01-open");
    let dlg = live_text(&mut s);

    for needle in [
        "Delete connection?",
        "Production (acme_ops@prod-db-1.acme.io)",
        "connections.json",
        "keychain",
    ] {
        assert!(dlg.contains(needle), "V1: dialog lacks `{needle}`");
    }

    assert_line_has(&dlg, "▎Cancel", "Delete", "V2 actions");
    assert_line_has(&dlg, "← → Choose", "y / n Quick answer", "V2 footer");

    assert!(dlg.contains("▎Cancel"), "S1: Cancel lacks focus");
    assert!(!dlg.contains("▎Delete"), "S1: Delete stole focus");

    for name in [
        "Personal",
        "Local PostgreSQL",
        "Scratch",
        "Acme",
        "Development",
        "Staging",
        "Analytics",
        "Production",
    ] {
        assert_line_lacks(&dlg, name, "▎", "S2 focus left the tree");
    }

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Delete connection?", "Esc closed the dialog");
    let cancelled = live_text(&mut s);
    assert!(cancelled.contains("6 saved"), "A1/N1: Esc dropped a row");
    assert!(
        cancelled.contains("◆ Production"),
        "A1/N1: Production row lost"
    );
    checkpoint(&mut s, &dir, "02-cancelled");

    support::press_step(&s, "d");
    waits::wait_state(&mut s, "Delete connection?", "reopened dialog");
    support::press_step(&s, "right");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A2 Delete never armed",
        |screen| support::screen_text(screen).contains("▎Delete"),
    );
    let armed = live_text(&mut s);
    assert!(!armed.contains("▎Cancel"), "A2: Cancel kept focus");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Delete connection?", "second Esc closed");
    let cancelled2 = live_text(&mut s);
    assert!(
        cancelled2.contains("6 saved"),
        "A2/N1: armed Esc dropped a row"
    );
    checkpoint(&mut s, &dir, "03-rearmed-cancelled");

    support::press_step(&s, "d");
    waits::wait_state(&mut s, "Delete connection?", "third dialog");
    let reopened = live_text(&mut s);
    assert!(reopened.contains("▎Cancel"), "N2: Cancel lacks focus");
    support::press_step(&s, "enter");
    waits::wait_gone(&mut s, "Delete connection?", "Enter-on-Cancel closed");
    let kept = live_text(&mut s);
    assert!(kept.contains("6 saved"), "N2: Enter-on-Cancel deleted");
    assert!(kept.contains("◆ Production"), "N2: Production row lost");
    checkpoint(&mut s, &dir, "04-enter-cancel");
    eprintln!("t1 connections_delete_dialog: three cancels, zero deletions");
}

#[test]
#[ignore = "tablepro t1 check; run with --ignored"]
fn t1_connections_duplicated() {
    let dir = t1_dir("t1_connections_duplicated");
    let case = t1_case_conn("connections_duplicated");
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::drive_with_timeout(
        &mut s,
        &["down", "down", "down", "down", "down", "down", "down"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "00-production");
    let pre = live_text(&mut s);
    assert!(pre.contains("6 saved"), "pre: boot is not 6 saved");

    support::press_step(&s, "ctrl-d");
    waits::wait_state(&mut s, "(Copy)", "duplicated row");
    checkpoint(&mut s, &dir, "01-duplicated");
    let dup = live_text(&mut s);

    assert!(dup.contains("Production (Copy)"), "V1: copy row missing");
    assert!(dup.contains("7 saved"), "V1: count never flipped to 7");
    assert!(!dup.contains("6 saved"), "V1: stale 6 saved persists");

    assert_line_has(&dup, "Duplicate", "Duplicated", "V2 status");
    assert_line_has(&dup, "▎", "◆ Production", "V2 cursor stays");
    assert_line_lacks(&dup, "(Copy)", "▎", "V2 cursor skips the copy");

    for needle in ["prod-db-1.acme.io:5432", "acme_prod"] {
        assert!(dup.contains(needle), "S1: detail changed `{needle}`");
    }

    assert!(dup.contains("▾ Personal"), "S2: Personal collapsed");
    assert!(dup.contains("▾ Acme"), "S2: Acme collapsed");

    for needle in ["Query 1", "Explorer", "New connection", "Edit connection"] {
        assert!(!dup.contains(needle), "N1/N2: dup shows `{needle}`");
    }

    support::press_step(&s, "down");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1 cursor on the copy",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains("▎") && l.contains("(Copy)"))
        },
    );
    let on_copy = live_text(&mut s);
    assert_line_has(&on_copy, "(Copy)", "pg", "A1 tree copy row");
    assert!(
        on_copy
            .lines()
            .any(|l| l.contains("(Copy)") && !l.contains("pg")),
        "A1: detail card never flipped to the copy"
    );

    support::press_step(&s, "d");
    waits::wait_state(&mut s, "Delete connection?", "copy delete dialog");
    support::press_step(&s, "right");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A2 Delete never armed",
        |screen| support::screen_text(screen).contains("▎Delete"),
    );
    support::press_step(&s, "enter");
    waits::wait_gone(&mut s, "Delete connection?", "delete executed");
    let removed = live_text(&mut s);
    assert!(removed.contains("6 saved"), "A2: count never fell back");
    assert!(
        !removed.contains("(Copy)"),
        "A2: copy row survived deletion"
    );
    checkpoint(&mut s, &dir, "02-copy-removed");

    support::drive_with_timeout(&mut s, &["ctrl-n", "wait:New connection"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-form-proof");

    let case2 = t1_case_wb("connections_duplicated_clean");
    write_provenance_named(&dir, &case2, "provenance-clean.txt");
    let mut s2 = support::spawn_boot(&case2);
    waits::wait_state(&mut s2, "Explorer", "N1 explorer needle");
    checkpoint(&mut s2, &dir, "04-clean-boot");
    eprintln!("t1 connections_duplicated: copy added, visited, removed");
}

#[test]
#[ignore = "tablepro t1 check; run with --ignored"]
fn t1_connections_production_detail() {
    let dir = t1_dir("t1_connections_production_detail");
    let case = t1_case_conn("connections_production_detail");
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(
        boot.contains("localhost:5432"),
        "pre: boot detail is not Local"
    );
    assert_line_has(&boot, "▎", "Personal", "pre: boot cursor moved");

    support::drive_with_timeout(
        &mut s,
        &[
            "down",
            "down",
            "down",
            "down",
            "down",
            "down",
            "down",
            "wait:prod-db-1.acme.io",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-production");
    let det = live_text(&mut s);

    for needle in [
        "PostgreSQL",
        "prod-db-1.acme.io:5432",
        "acme_prod",
        "acme_ops",
        "production",
        "Safe Mode",
        "deliberate acknowledgement",
        "on / bastion.acme.io",
        "1 hour ago",
    ] {
        assert!(det.contains(needle), "V1: detail lacks `{needle}`");
    }

    assert_line_has(&det, "▎", "◆ Production", "V2 cursor");
    for needle in ["pg", "sqlite", "mysql", "·", "◇", "◆"] {
        assert!(det.contains(needle), "V2: chrome lost `{needle}`");
    }

    assert!(
        !det.contains("localhost:5432"),
        "S1: Local detail never left"
    );

    assert!(det.contains("6 saved"), "S2: count moved");

    for needle in ["Edit connection", "Name", "Query 1", "Explorer"] {
        assert!(!det.contains(needle), "N1/N2: detail shows `{needle}`");
    }

    support::press_step(&s, "up");
    waits::wait_state(&mut s, "analytics.acme.io", "A1 Analytics detail");
    let ana = live_text(&mut s);
    assert!(ana.contains("warehouse"), "A1: Analytics db missing");
    assert!(
        !ana.contains("prod-db-1.acme.io"),
        "A1: prod host never left"
    );
    checkpoint(&mut s, &dir, "02-analytics");

    support::press_step(&s, "down");
    waits::wait_state(&mut s, "prod-db-1.acme.io", "A2 back on Production");
    let before = live_text(&mut s);
    support::press_step(&s, "down");
    std::thread::sleep(Duration::from_millis(400));
    let after = live_text(&mut s);
    assert_line_has(&after, "▎", "◆ Production", "A2 cursor held");
    assert!(
        after.contains("prod-db-1.acme.io:5432"),
        "A2: detail moved at the boundary"
    );
    assert_eq!(before, after, "A2: refused Down repainted the screen");

    support::drive_with_timeout(&mut s, &["e", "wait:Edit connection"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-edit-proof");

    let case2 = t1_case_wb("connections_production_detail_clean");
    write_provenance_named(&dir, &case2, "provenance-clean.txt");
    let mut s2 = support::spawn_boot(&case2);
    waits::wait_state(&mut s2, "Explorer", "N2 explorer needle");
    checkpoint(&mut s2, &dir, "04-clean-boot");
    eprintln!("t1 connections_production_detail: card follows cursor");
}

#[test]
#[ignore = "tablepro t1 check; run with --ignored"]
fn t1_fade_table_wheel() {
    let dir = t1_dir("t1_fade_table_wheel");
    let case = t1_case_wb("fade_table_wheel");
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::drive_with_timeout(
        &mut s,
        &[
            "down",
            "down",
            "down",
            "down",
            "down",
            "enter",
            "wait:public › orders",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "00-data");
    let boot = live_text(&mut s);
    let boot_frame = live_frame(&mut s);
    assert!(
        boot.contains("rows 1–27 of 500 loaded"),
        "pre: boot meta is not rows 1–27"
    );
    assert!(boot.contains("10000"), "pre: first order cell missing");

    let (_row, _col) = wheel_over(&mut s, "10000", Wheel::Down, 3, case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1 meta never advanced",
        |screen| support::screen_text(screen).contains("rows 10–36 of 500 loaded"),
    );
    checkpoint(&mut s, &dir, "01-faded");
    let faded = live_text(&mut s);
    let frame = live_frame(&mut s);

    assert!(
        faded.contains("rows 10–36 of 500 loaded · 1,203,338 total · cols 1–2 of 14"),
        "V1: faded meta wrong"
    );
    assert!(faded.contains("10009"), "V1: first faded row is not #10");
    assert!(!faded.contains("10000"), "V1: boot row never left");

    assert_line_has(&faded, "10009", "┃", "V2 thumb");
    assert_line_has(&faded, "10010", "│", "V2 track");

    assert_no_grid_sort_marks(&faded, "S1/S2");
    assert!(
        !faded.contains("• 1 pending"),
        "S1: wheeling dirtied a cell"
    );
    assert!(faded.contains("cols 1–2 of 14"), "S2: columns drifted");

    assert!(
        faded.contains("of 500 loaded"),
        "N2: page changed under the wheel"
    );

    let (header_row, _) = support::screen_find(
        &s.inner.snapshot().expect("header snapshot"),
        "order_number",
    )
    .expect("V2: header row missing");
    let top_body = header_row + 1;
    let interior = header_row + 10;
    let edge = row_fg_median(&frame, top_body, "V2 edge");
    let inner = row_fg_median(&frame, interior, "V2 interior");
    assert_faded(edge, inner, "V2 top edge");
    assert_row_same(&boot_frame, &frame, header_row, "N1 header never fades");

    let (erow, ecol) = support::screen_find(&s.inner.snapshot().expect("stance snapshot"), "10009")
        .expect("A2: stance cell missing");
    wheel_until_stable(
        &mut s,
        ecol,
        erow,
        Wheel::Up,
        case_timeout(&case),
        "A2 return to top",
    );
    let top = live_text(&mut s);
    assert!(
        top.contains("rows 1–27 of 500 loaded"),
        "A2: offset never returned to 0"
    );
    assert!(top.contains("10000"), "A2: first order never returned");
    checkpoint(&mut s, &dir, "02-back-top");
    eprintln!("t1 fade_table_wheel: offset 0 → 9 → 0, edge faded");
}

#[test]
#[ignore = "tablepro t1 check; run with --ignored"]
fn t1_overlays_history_tab() {
    let dir = t1_dir("t1_overlays_history_tab");
    let case = t1_case_wb("overlays_history_tab");
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(!boot.contains("H History"), "pre: history tab at boot");

    support::press_step(&s, "ctrl-y");
    waits::wait_state(&mut s, "Query history", "history panel");
    checkpoint(&mut s, &dir, "01-history");
    let h = live_text(&mut s);

    for needle in ["H History ×", "≡ Query 1 ×", "Query history", "15 entries"] {
        assert!(h.contains(needle), "V1: history lacks `{needle}`");
    }
    assert!(h.contains("━━"), "V1: active underline missing");

    for needle in [
        "Search history",
        "ANDed",
        "scope: Production",
        "status: any",
        "SELECT plan",
        "Connection",
        "Duration",
        "Rows",
        "Open in new tab",
        "Run in new tab",
    ] {
        assert!(h.contains(needle), "V2: history lacks `{needle}`");
    }

    assert_line_has(&h, "›", "SELECT plan", "S2 first entry");
    assert_line_has(&h, "!", "ordres", "S2 failed entry");
    assert!(h.contains("GROUP BY plan"), "S2: entry detail missing");

    assert!(h.contains("≡ Query 1 ×"), "N1: strip hidden");
    assert!(h.contains("Explorer"), "N1: explorer hidden");
    for needle in ["Open tabs", "Quit TablePro?", "Save changes?"] {
        assert!(!h.contains(needle), "N1: modal `{needle}` over history");
    }

    assert!(!h.contains("Query 2"), "N2: phantom Query 2");
    assert_eq!(h.matches('×').count(), 3, "N2: tab × count moved");

    support::press_step(&s, "ctrl-y");
    std::thread::sleep(Duration::from_millis(400));
    let reslot = live_text(&mut s);
    assert_eq!(
        reslot.matches("H History").count(),
        1,
        "S1: second ctrl-y duplicated the tab"
    );

    support::press_step(&s, "down");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1 detail never followed",
        |screen| !support::screen_text(screen).contains("GROUP BY plan"),
    );
    let moved = live_text(&mut s);
    assert_line_has(&moved, "›", "payme", "A1 cursor on entry 2");
    assert!(
        moved.matches("payme").count() >= 2,
        "A1: detail pane never followed the cursor"
    );
    assert!(moved.contains("15 entries"), "A1: entries count moved");

    support::press_step(&s, "up");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A2 cursor never returned",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains("›") && l.contains("SELECT plan"))
        },
    );
    support::press_step(&s, "up");
    std::thread::sleep(Duration::from_millis(400));
    let held = live_text(&mut s);
    assert_line_has(&held, "›", "SELECT plan", "A2 cursor held");
    assert!(held.contains("15 entries"), "A2: entries count moved");
    checkpoint(&mut s, &dir, "02-cursor");

    support::drive_with_timeout(&mut s, &["ctrl-g", "wait:Open tabs"], case.timeout_ms);
    checkpoint(&mut s, &dir, "03-tablist");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Open tabs", "N1 tab list never closed");

    support::drive_with_timeout(&mut s, &["ctrl-t", "wait:Query 2"], case.timeout_ms);
    checkpoint(&mut s, &dir, "04-query2");
    let tabbed = live_text(&mut s);
    assert_eq!(tabbed.matches('×').count(), 4, "N2: Query 2 never tabbed");
    eprintln!("t1 overlays_history_tab: work tab proven, cursor proven");
}

#[test]
#[ignore = "tablepro t1 check; run with --ignored"]
fn t1_resize_workbench_grown() {
    static ARGS: &[&str] = &["--connect", "Production"];
    let dir = t1_dir("t1_resize_workbench_grown");
    let case = t1_case_size("resize_workbench_grown", ARGS, 80, 24, BOOT_WB).timeout(15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-narrow");
    let narrow = live_text(&mut s);

    assert!(
        !narrow.contains("No results yet"),
        "pre: query panel leaks into the drawer"
    );

    resize_to(&mut s, 120, 40, case_timeout(&case));
    waits::wait_state(&mut s, "No results yet", "grown docked layout");
    waits::wait_state(&mut s, "S audit", "grown loaded tree");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "grown status never settled",
        |screen| !support::screen_text(screen).contains("Connected to"),
    );
    checkpoint(&mut s, &dir, "01-grown");
    let g = live_text(&mut s);

    assert_line_has(&g, "Explorer", "public", "V1 explorer");
    assert!(
        g.contains("Type SQL. Ctrl+R runs the statement under the cursor."),
        "V1: editor placeholder missing"
    );
    assert!(g.contains("No results yet"), "V1: results dock missing");

    for needle in [
        "D acme_prod",
        "S public",
        "S audit",
        "S analytics",
        "Tables",
    ] {
        assert!(g.contains(needle), "V2: tree lost `{needle}`");
    }

    assert!(
        !g.contains("Terminal too small"),
        "S1/N1: notice at healthy sizes"
    );
    assert_line_has(&g, "▎", "S public", "S2 focus survived");

    assert!(!g.contains("Query 2"), "N2: phantom Query 2");
    assert_eq!(g.matches('×').count(), 2, "N2: tab × count moved");

    resize_to(&mut s, 80, 24, case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A2 drawer never returned",
        |screen| {
            let text = support::screen_text(screen);
            text.contains("1.2 M") && !text.contains("No results yet")
        },
    );
    checkpoint(&mut s, &dir, "02-redrawer");

    resize_to(&mut s, 120, 40, case_timeout(&case));
    waits::wait_state(&mut s, "No results yet", "regrown docked layout");
    waits::wait_state(&mut s, "S audit", "regrown loaded tree");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "regrown status never settled",
        |screen| !support::screen_text(screen).contains("Connected to"),
    );
    checkpoint(&mut s, &dir, "03-regrown");

    support::press_step(&s, "tab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1 focus never reached the tab",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains("▎") && l.contains("Type SQL"))
        },
    );
    checkpoint(&mut s, &dir, "04-tabbed");

    support::drive_with_timeout(&mut s, &["ctrl-t", "wait:Query 2"], case.timeout_ms);
    checkpoint(&mut s, &dir, "05-query2");

    resize_to(&mut s, 60, 10, case_timeout(&case));
    waits::wait_state(&mut s, "Terminal too small", "N1 notice");
    let tiny = live_text(&mut s);
    assert!(tiny.contains("Need 72×20"), "N1: notice lacks the minimum");
    checkpoint(&mut s, &dir, "06-notice");
    eprintln!("t1 resize_workbench_grown: docked reflow proven");
}

#[test]
#[ignore = "tablepro t1 check; run with --ignored"]
fn t1_resize_workbench_shrunk() {
    static ARGS: &[&str] = &["--connect", "Production"];
    let dir = t1_dir("t1_resize_workbench_shrunk");
    let case = t1_case_size("resize_workbench_shrunk", ARGS, 120, 40, BOOT_WB).timeout(15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);

    support::drive_with_timeout(&mut s, &["wait:S audit"], case.timeout_ms);
    checkpoint(&mut s, &dir, "00-wide");
    let wide = live_text(&mut s);
    assert!(wide.contains("Explorer"), "pre: explorer missing");
    assert!(wide.contains("No results yet"), "pre: results dock missing");
    assert!(
        wide.contains("Type SQL. Ctrl+R runs the statement under the cursor."),
        "pre: editor placeholder missing"
    );

    resize_to(&mut s, 80, 24, case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "shrunk status never settled",
        |screen| !support::screen_text(screen).contains("Connected to"),
    );
    waits::wait_state(&mut s, "TablePro", "shrunk header");
    checkpoint(&mut s, &dir, "01-shrunk");
    let shrunk = live_text(&mut s);

    assert_line_has(&shrunk, "Explorer", "public", "V1 drawer");
    for needle in ["Tables", "T orders", "1.2 M", "S analytics", "▎"] {
        assert!(shrunk.contains(needle), "V1: drawer lacks `{needle}`");
    }

    for needle in [
        "Type SQL. Ctrl+R runs the statement under the cursor.",
        "No results yet",
    ] {
        assert!(!shrunk.contains(needle), "V2: query leaked `{needle}`");
    }

    assert!(
        !shrunk.contains("Terminal too small"),
        "S1/N1: notice at healthy sizes"
    );
    assert_line_has(&shrunk, "▎", "S public", "S2 focus held");
    assert_line_has(&shrunk, "↑ ↓ Move", "Enter Open", "S2 hints");

    assert!(!shrunk.contains("Query 2"), "N2: phantom Query 2");

    support::press_step(&s, "tab");
    waits::wait_state(&mut s, "No results yet", "A1 query tab back");
    let aside = live_text(&mut s);
    assert!(
        !aside.contains("Explorer"),
        "A1: drawer never stepped aside"
    );
    checkpoint(&mut s, &dir, "02-aside");

    resize_to(&mut s, 120, 40, case_timeout(&case));
    waits::wait_state(&mut s, "No results yet", "A2 docked layout");
    waits::wait_state(&mut s, "S audit", "A2 loaded tree");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A2 status never settled",
        |screen| !support::screen_text(screen).contains("Connected to"),
    );
    let regrown = live_text(&mut s);
    assert!(regrown.contains("Explorer"), "A2: explorer never returned");
    assert_line_has(&regrown, "Explorer", "public", "A2 docked");
    checkpoint(&mut s, &dir, "03-regrown");

    support::drive_with_timeout(&mut s, &["ctrl-t", "wait:Query 2"], case.timeout_ms);
    checkpoint(&mut s, &dir, "04-query2");

    resize_to(&mut s, 60, 10, case_timeout(&case));
    waits::wait_state(&mut s, "Terminal too small", "N1 notice");
    let tiny = live_text(&mut s);
    assert!(tiny.contains("Need 72×20"), "N1: notice lacks the minimum");
    checkpoint(&mut s, &dir, "05-notice");
    eprintln!("t1 resize_workbench_shrunk: drawer reflow proven");
}

#[test]
#[ignore = "tablepro t1 check; run with --ignored"]
fn t1_table_structure() {
    let dir = t1_dir("t1_table_structure");
    let case = t1_case_wb("table_structure");
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::drive_with_timeout(
        &mut s,
        &[
            "down",
            "down",
            "down",
            "down",
            "down",
            "enter",
            "wait:public › orders",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "00-data");
    let data = live_text(&mut s);
    assert!(data.contains("10000"), "pre: data cell missing");

    support::press_step(&s, "ctrl-d");
    waits::wait_state(&mut s, "Foreign keys", "structure sub-tabs");
    checkpoint(&mut s, &dir, "01-structure");
    let st = live_text(&mut s);

    assert_line_has(&st, "Data", "Structure", "V1 mode tabs");
    assert_line_has(&st, "Columns", "Foreign keys", "V1 sub-tabs");
    for needle in ["Indexes", "Constraints", "Triggers", "DDL", "━"] {
        assert!(st.contains(needle), "V1: tabs lack `{needle}`");
    }

    for needle in [
        "gen_random_uuid()",
        "PK",
        "FK",
        "now()",
        "updated_at",
        "14 columns · read from the catalog · changes are queued until Save",
    ] {
        assert!(st.contains(needle), "V2: structure lacks `{needle}`");
    }

    assert_line_has(&st, "▎", "gen_random_uuid", "S1 cursor on id");

    for needle in ["T orders ×", "14 cols"] {
        assert!(st.contains(needle), "S2: tab lost `{needle}`");
    }
    assert!(!st.contains("• 1 pending"), "S2: Structure dirtied a cell");
    assert_no_grid_sort_marks(&st, "S2");

    assert!(!st.contains("EDIT"), "N1: editor open in Structure");

    assert_eq!(
        st.matches("T orders").count(),
        2,
        "N2: orders tab count moved"
    );

    support::press_step(&s, "down");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A2 cursor never moved",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains("▎") && l.contains("order_number"))
        },
    );

    support::press_step(&s, "enter");
    std::thread::sleep(Duration::from_millis(400));
    let noop = live_text(&mut s);
    assert!(noop.contains("Foreign keys"), "N1: Enter left Structure");
    assert!(!noop.contains("EDIT"), "N1: Enter opened an editor");
    assert!(!noop.contains("• 1 pending"), "N1: Enter dirtied a cell");
    checkpoint(&mut s, &dir, "02-enter-noop");

    support::press_step(&s, "ctrl-d");
    waits::wait_state(&mut s, "10000", "A1 data grid back");
    let back = live_text(&mut s);
    assert_no_grid_sort_marks(&back, "A1");
    assert!(!back.contains("Foreign keys"), "A1: sub-tabs never left");
    assert!(!back.contains("Columns"), "A1: Columns sub-tab never left");
    checkpoint(&mut s, &dir, "03-data");

    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "EDIT", "N1 editor proof");
    checkpoint(&mut s, &dir, "04-edit-proof");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "EDIT", "N1 edit never cancelled");

    support::drive_with_timeout(&mut s, &["ctrl-t", "wait:Query 2"], case.timeout_ms);
    checkpoint(&mut s, &dir, "05-query2");
    eprintln!("t1 table_structure: modes flip, Structure is read-only");
}

#[test]
#[ignore = "tablepro t1 check; run with --ignored"]
fn t1_workbench_commit_dialog() {
    let dir = t1_dir("t1_workbench_commit_dialog");
    let case = t1_case_wb("workbench_commit_dialog").timeout(15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);

    support::drive_with_timeout(
        &mut s,
        &[
            "down",
            "down",
            "down",
            "down",
            "down",
            "enter",
            "wait:public › orders",
            "home",
            "right",
            "right",
            "right",
            "right",
            "enter",
            "ctrl-l",
            "type:paid",
            "enter",
            "wait:1 pending",
        ],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "00-dirty");
    let dirty = live_text(&mut s);
    assert!(
        dirty.contains("• 1 pending"),
        "pre: cell edit never dirtied"
    );

    support::press_step(&s, "ctrl-s");
    waits::wait_state(&mut s, "Save changes?", "commit review");
    checkpoint(&mut s, &dir, "01-open");
    let co = live_text(&mut s);

    for needle in [
        "Save changes?",
        "Action",
        "Target",
        "Production · production · acme_prod · public.orders",
        "Scope",
        "1 update · 0 inserts · 0 deletes",
        "Transaction",
        "Safe Mode",
        "deliberate confirmation required",
        "UPDATE public.orders",
    ] {
        assert!(co.contains(needle), "V1: review lacks `{needle}`");
    }

    assert!(
        co.contains("Type orders to confirm"),
        "V2: ack label missing"
    );
    assert_line_lacks(&co, "Cancel", "▎", "V2 focus in the ack field");
    assert!(
        co.contains("• 1 pending"),
        "V2: header lost the pending marker"
    );

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Save changes?", "N1 review never closed");
    waits::wait_state(&mut s, "Changes kept pending", "S2 cancel status");
    let kept = live_text(&mut s);
    assert!(
        kept.contains("Changes kept pending"),
        "S2: cancel status missing"
    );
    assert!(kept.contains("• 1 pending"), "N1: Esc saved the edit");
    checkpoint(&mut s, &dir, "02-cancelled");

    support::press_step(&s, "ctrl-s");
    waits::wait_state(&mut s, "Save changes?", "reopened review");

    support::press_step(&s, "enter");
    std::thread::sleep(Duration::from_millis(300));
    let begun = live_text(&mut s);
    assert!(
        begun.contains("Save changes?"),
        "A2: first Enter executed the dialog"
    );
    assert!(
        !begun.contains("▎Cancel"),
        "A2: first Enter skipped editing"
    );
    support::press_step(&s, "enter");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A2 focus never advanced",
        |screen| support::screen_text(screen).contains("▎Cancel"),
    );
    let advanced = live_text(&mut s);
    assert!(
        advanced.contains("Save changes?"),
        "A2: Enter executed the dialog"
    );

    support::press_step(&s, "y");
    std::thread::sleep(Duration::from_millis(400));
    let inert = live_text(&mut s);
    assert!(inert.contains("Save changes?"), "N2: y closed the review");
    assert!(inert.contains("• 1 pending"), "N2: y saved the edit");

    support::press_step(&s, "right");
    std::thread::sleep(Duration::from_millis(400));
    let refused = live_text(&mut s);
    assert!(
        refused.contains("▎Cancel"),
        "S1: focus left Cancel for disabled Save"
    );
    assert!(!refused.contains("▎Save"), "S1: disabled Save took focus");
    checkpoint(&mut s, &dir, "03-unarmed");

    support::press_step(&s, "backtab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1 ack never refocused",
        |screen| !support::screen_text(screen).contains("▎Cancel"),
    );
    support::press_step(&s, "enter");
    std::thread::sleep(Duration::from_millis(300));
    support::drive_with_timeout(&mut s, &["type:orders"], case.timeout_ms);
    support::press_step(&s, "enter");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1 focus never left the ack",
        |screen| support::screen_text(screen).contains("▎Cancel"),
    );
    support::press_step(&s, "right");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1 Save never armed",
        |screen| support::screen_text(screen).contains("▎Save"),
    );
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "Saving…", "A1 commit started");
    waits::wait_state(&mut s, "Saved 1 change to public.orders", "A1 committed");
    let saved = live_text(&mut s);
    assert!(
        !saved.contains("• 1 pending"),
        "A1: pending marker survived the commit"
    );
    assert!(
        !saved.contains("Save changes?"),
        "A1: review survived the commit"
    );
    checkpoint(&mut s, &dir, "04-saved");
    eprintln!("t1 workbench_commit_dialog: token gate proven end to end");
}

#[test]
#[ignore = "tablepro t1 check; run with --ignored"]
fn t1_workbench_explorer_hidden() {
    let dir = t1_dir("t1_workbench_explorer_hidden");
    let case = t1_case_wb("workbench_explorer_hidden");
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::drive_with_timeout(&mut s, &["wait:S audit"], case.timeout_ms);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert_line_has(&boot, "▎", "S public", "pre: boot focus moved");

    support::press_step(&s, "ctrl-b");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "explorer never hid",
        |screen| !support::screen_text(screen).contains("Explorer"),
    );
    checkpoint(&mut s, &dir, "01-hidden");
    let h = live_text(&mut s);

    for needle in ["Explorer", "Filter objects", "S public", "D acme_prod"] {
        assert!(!h.contains(needle), "V1/N1: explorer leaks `{needle}`");
    }
    assert!(
        h.contains("Type SQL. Ctrl+R runs the statement under the cursor."),
        "V1: editor placeholder missing"
    );
    assert!(h.contains("No results yet"), "V1: results dock missing");

    for needle in [
        "Production",
        "◆ production",
        "acme_prod › public",
        "safe",
        "≡ Query 1 ×",
    ] {
        assert!(h.contains(needle), "V2: chrome lost `{needle}`");
    }
    assert!(!h.contains("Query 2"), "N2: phantom Query 2");

    assert_line_has(&h, "Enter Edit", "Ctrl+R Run", "S1 tab hints");
    assert_line_has(&h, "▎", "Type SQL", "S1 editor focus");

    support::drive_with_timeout(&mut s, &["i", "type:x"], case.timeout_ms);
    waits::wait_state(&mut s, "1 x", "A2 typed draft");
    support::press_step(&s, "escape");
    std::thread::sleep(Duration::from_millis(400));
    let edited = live_text(&mut s);
    assert!(edited.contains("1 x"), "A2: draft never landed");
    assert!(
        !edited.contains("Explorer"),
        "A2: editing restored the explorer"
    );
    checkpoint(&mut s, &dir, "02-edited");

    support::press_step(&s, "ctrl-b");
    waits::wait_state(&mut s, "Explorer", "A1 explorer back");
    let restored = live_text(&mut s);
    assert_line_has(&restored, "▎", "S public", "A1 focus back");
    assert_line_has(&restored, "↑ ↓ Move", "Enter Open", "A1 hints back");
    assert!(restored.contains("S audit"), "S2/N2: schema dropped");
    assert!(
        restored.contains("≡ Query 1 ×"),
        "N2: tab lost across the toggle"
    );
    checkpoint(&mut s, &dir, "03-restored");
    eprintln!("t1 workbench_explorer_hidden: toggle proven both ways");
}

#[test]
#[ignore = "tablepro t1 check; run with --ignored"]
fn t1_workbench_maximized() {
    let dir = t1_dir("t1_workbench_maximized");
    let case = t1_case_wb("workbench_maximized");
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::drive_with_timeout(&mut s, &["wait:S audit"], case.timeout_ms);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_text(&mut s);
    assert!(boot.contains("No results yet"), "pre: results dock missing");
    assert!(boot.contains("Type SQL"), "pre: editor missing");

    support::press_step(&s, "z");
    support::wait_screen(&mut s, case_timeout(&case), "zoom never landed", |screen| {
        let text = support::screen_text(screen);
        text.contains("No results yet") && !text.contains("Type SQL")
    });
    checkpoint(&mut s, &dir, "01-zoomed");
    let m = live_text(&mut s);

    for needle in ["Explorer", "S public", "Type SQL"] {
        assert!(!m.contains(needle), "V1: zoom leaks `{needle}`");
    }
    assert!(m.contains("No results yet"), "V1: results pane missing");
    assert!(
        m.contains("Ctrl+R runs the statement under the cursor"),
        "V1: results hints missing"
    );

    for needle in ["Production", "≡ Query 1 ×"] {
        assert!(m.contains(needle), "V2: chrome lost `{needle}`");
    }
    assert_line_has(&m, "↑ ↓ Move", "Enter Open", "V2 stale hints");
    assert!(!m.contains('▎'), "S1: focus bar leaks into zoom");

    assert!(!m.contains("Enter Edit"), "N1: zoom focused the editor");

    assert!(!m.contains("Query 2"), "N2: phantom Query 2");
    assert_eq!(m.matches('×').count(), 2, "N2: tab × count moved");

    support::press_step(&s, "z");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1 layout never restored",
        |screen| {
            let text = support::screen_text(screen);
            text.contains("Explorer") && text.contains("Type SQL")
        },
    );
    let restored = live_text(&mut s);
    assert_line_has(&restored, "← → Switch", "Ctrl+G Tab list", "A1 strip focus");
    assert_line_lacks(&restored, "S public", "▎", "A1 tree unfocused");
    assert!(
        restored.contains("No results yet"),
        "A1: results dock missing"
    );
    checkpoint(&mut s, &dir, "02-restored");

    let mut editor = false;
    for _ in 0..4 {
        support::press_step(&s, "tab");
        std::thread::sleep(Duration::from_millis(300));
        if live_text(&mut s).contains("Enter Edit") {
            editor = true;
            break;
        }
    }
    assert!(editor, "A2: Tab never reached the editor");
    support::press_step(&s, "z");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A2 editor zoom never landed",
        |screen| {
            let text = support::screen_text(screen);
            text.contains("Type SQL") && !text.contains("No results yet")
        },
    );
    let ezoom = live_text(&mut s);
    assert!(!ezoom.contains("Explorer"), "A2: explorer leaks into zoom");
    checkpoint(&mut s, &dir, "03-editor-zoomed");

    support::press_step(&s, "escape");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Esc never restored",
        |screen| {
            let text = support::screen_text(screen);
            text.contains("Explorer")
                && text.contains("Type SQL")
                && text.contains("No results yet")
        },
    );
    checkpoint(&mut s, &dir, "04-esc-restored");

    support::drive_with_timeout(&mut s, &["ctrl-t", "wait:Query 2"], case.timeout_ms);
    checkpoint(&mut s, &dir, "05-query2");
    eprintln!("t1 workbench_maximized: both zoom targets proven");
}

#[test]
#[ignore = "tablepro t1 check; run with --ignored"]
fn t1_workbench_quit_confirm() {
    let dir = t1_dir("t1_workbench_quit_confirm");
    let case = t1_case_wb("workbench_quit_confirm");
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::drive_with_timeout(&mut s, &["tab", "i", "type:x", "escape"], case.timeout_ms);
    let dirty = live_text(&mut s);
    assert!(dirty.contains("1 x"), "pre: draft never landed");
    checkpoint(&mut s, &dir, "00-dirty");

    support::press_step(&s, "q");
    waits::wait_state(&mut s, "Quit TablePro?", "quit dialog");
    checkpoint(&mut s, &dir, "01-open");
    let qd = live_text(&mut s);

    for needle in ["Quit TablePro?", "1 unsaved query will be lost."] {
        assert!(qd.contains(needle), "V1: dialog lacks `{needle}`");
    }
    assert_line_has(&qd, "▎Cancel", "Quit", "V2 actions");
    assert_line_has(&qd, "← → Choose", "y / n Quick answer", "V2 footer");

    assert!(qd.contains("▎Cancel"), "S1: Cancel lacks focus");
    assert!(!qd.contains("▎Quit"), "S1: Quit stole focus");
    assert!(qd.contains("1 x"), "S2: draft hidden by the dialog");

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Quit TablePro?", "A1 dialog never closed");
    let kept = live_text(&mut s);
    assert!(kept.contains("1 x"), "A1/N1: Esc dropped the draft");
    assert!(kept.contains("Query 1"), "A1/N1: Esc quit the app");
    checkpoint(&mut s, &dir, "02-cancelled");

    support::press_step(&s, "q");
    waits::wait_state(&mut s, "Quit TablePro?", "second dialog");
    let reopened = live_text(&mut s);
    assert!(reopened.contains("▎Cancel"), "N2: Cancel lacks focus");
    support::press_step(&s, "enter");
    waits::wait_gone(&mut s, "Quit TablePro?", "N2 dialog never closed");
    let kept2 = live_text(&mut s);
    assert!(
        kept2.contains("1 x"),
        "N2: Enter-on-Cancel dropped the draft"
    );
    checkpoint(&mut s, &dir, "03-enter-cancel");

    support::press_step(&s, "q");
    waits::wait_state(&mut s, "Quit TablePro?", "third dialog");
    support::press_step(&s, "y");
    let exit_text = expect_exit_text(&s, case_timeout(&case), "A2 quit");
    std::fs::write(
        dir.join("04-exit.txt"),
        format!("# exit frame\n{exit_text}\n"),
    )
    .expect("write 04-exit.txt");
    eprintln!("t1 workbench_quit_confirm: cancels keep, y quits");
}
