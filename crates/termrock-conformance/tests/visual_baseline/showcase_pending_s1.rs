use std::path::{Path, PathBuf};
use std::time::Duration;

use tuiscotti::render::frame_from_screen;
use tuiscotti::tui::{MouseButton, Wheel};
use tuiscotti::{Color as TColor, Frame, Rgb};

use crate::support::Session;
use crate::support::state_waits as waits;
use crate::support::typed_input::Input;
use crate::support::{self, Case, Color, SHOWCASE};

const BOOT: &str = "Junie Design system";

fn s1_case(slug: &str, args: &'static [&'static str], cols: u16, rows: u16) -> Case {
    Case::dynamic(
        format!("journeys/showcase/s1/{slug}"),
        SHOWCASE,
        args,
        cols,
        rows,
        Color::Truecolor,
        BOOT,
    )
}

fn s1_dir(test: &str) -> PathBuf {
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
        "profile: tuiscotti-default\ntransport: pty\nport_of: 76537c27e6e00ef93a2d09d280c8c9cb6097e446\nbinary: {}\nsha256: {}\nsize_bytes: {}\nmodified_unix: {modified}\nargv: {}\n",
        subject.path.display(),
        subject.sha256,
        subject.len,
        argv.join(" "),
    );
    std::fs::write(dir.join(file), &body)
        .unwrap_or_else(|e| panic!("write {}: {e}", dir.join(file).display()));
    eprintln!("s1 provenance ({file}): {body}");
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

fn settle_text(s: &mut Session, timeout: Duration, what: &str) -> String {
    let deadline = std::time::Instant::now() + timeout;
    let mut prev = live_text(s);
    loop {
        std::thread::sleep(Duration::from_millis(400));
        let cur = live_text(s);
        if cur == prev {
            return cur;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "{what}: text never settled"
        );
        prev = cur;
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
    let deadline = std::time::Instant::now() + timeout;
    let mut prev = settle_text(s, Duration::from_millis(3000), what);
    for _ in 0..10 {
        wheel_at(s, col, row, dir, 5);
        let cur = settle_text(s, Duration::from_millis(3000), what);
        if cur == prev {
            return cur;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "{what}: never reached the boundary"
        );
        prev = cur;
    }
    panic!("{what}: still moving after 10 batches");
}

fn hover_over(s: &mut Session, needle: &str, timeout: Duration) -> (u16, u16) {
    let (row, col) = find_pos(s, needle, timeout);
    Input::move_to(col, row).send(s);
    std::thread::sleep(Duration::from_millis(120));
    (row, col)
}

fn click_at(s: &mut Session, needle: &str, timeout: Duration) -> (u16, u16) {
    let (row, col) = find_pos(s, needle, timeout);
    Input::down(MouseButton::Left, col, row).send(s);
    Input::up(MouseButton::Left, col, row).send(s);
    std::thread::sleep(Duration::from_millis(120));
    (row, col)
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

fn span_fg_median(frame: &Frame, row: u16, col: u16, len: usize, what: &str) -> u32 {
    let mut lums: Vec<u32> = Vec::new();
    for c in col..col.saturating_add(len as u16) {
        let Some(cell) = frame.get(c, row) else {
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
        "{what}: span ({col}..+{len}, row {row}) has no resolvable-fg cells"
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

fn assert_same_tone(a: u32, b: u32, what: &str) {
    assert!(
        a == b,
        "{what}: rows must read at the same tone (lum {a} == {b})"
    );
}

fn assert_cell_same(a: &Frame, b: &Frame, col: u16, row: u16, what: &str) {
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

fn assert_row_same(a: &Frame, b: &Frame, row: u16, what: &str) {
    assert_eq!(
        a.cols, b.cols,
        "{what}: frame widths differ ({} vs {})",
        a.cols, b.cols
    );
    for col in 0..a.cols {
        assert_cell_same(a, b, col, row, what);
    }
}

fn assert_line_has(text: &str, needle: &str, also: &str, what: &str) {
    assert!(
        text.lines().any(|l| l.contains(needle) && l.contains(also)),
        "{what}: no line contains both `{needle}` and `{also}`"
    );
}

fn assert_line_lacks(text: &str, needle: &str, also: &str, what: &str) {
    for line in text.lines().filter(|l| l.contains(needle)) {
        assert!(
            !line.contains(also),
            "{what}: `{needle}` line must not contain `{also}`: {line:?}"
        );
    }
}

fn assert_right_lacks(text: &str, needle: &str, also: &str, what: &str) {
    for line in text.lines().filter(|l| l.contains(needle)) {
        let right = line
            .find(['┃', '│'])
            .map(|off| &line[off..])
            .unwrap_or(line);
        assert!(
            !right.contains(also),
            "{what}: card side of `{needle}` line must not contain `{also}`: {line:?}"
        );
    }
}

fn assert_body_lacks(text: &str, label: &str, what: &str) {
    for line in text.lines().skip(1) {
        assert!(
            !line.contains(label),
            "{what}: `{label}` must not render in the body: {line:?}"
        );
    }
}

fn assert_body_has(text: &str, needle: &str, what: &str) {
    assert!(
        text.lines().skip(1).any(|l| l.contains(needle)),
        "{what}: no body line contains `{needle}`"
    );
}

fn parse_window(label: &str, what: &str) -> (u32, u32) {
    let sep = if label.contains('–') { '–' } else { '-' };
    let (a, b) = label
        .split_once(sep)
        .unwrap_or_else(|| panic!("{what}: no window separator in `{label}`"));
    let first: u32 = a
        .trim()
        .parse()
        .unwrap_or_else(|_| panic!("{what}: bad window start in `{label}`"));
    let last: u32 = b
        .trim()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .unwrap_or_else(|_| panic!("{what}: bad window end in `{label}`"));
    (first, last)
}

#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_table_page_boot() {
    let dir = s1_dir("s1_table_page_boot");
    let case = s1_case("table_page_boot", &["--page", "tables"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);

    for needle in ["#1040", "#1041", "#1042", "#1043", "#1044"] {
        assert!(boot.contains(needle), "V1: boot lacks row {needle}");
    }
    assert_line_has(&boot, "Tasks", "unsorted · 1–5 of 24", "V1 meta");
    assert_line_has(&boot, "ID", "Task", "V1 header");
    assert!(
        boot.contains("No checks have run yet"),
        "V1: Checks card lacks the empty state"
    );

    assert_eq!(
        boot.matches('▎').count(),
        1,
        "V2: exactly the nav focus bar may show at boot"
    );
    assert_line_has(&boot, "▎", "Tables", "V2 nav focus");
    assert!(
        !boot.contains('▴') && !boot.contains('▾'),
        "V2: no sort marker at boot"
    );

    assert!(boot.contains("unsorted"), "S1/S2: sort is None at boot");

    assert!(!boot.contains("Selected"), "N1: no Selected status at boot");

    support::press_step(&s, "tab");
    waits::wait_state(&mut s, "Columns", "A1 table footer");
    let focused = live_text(&mut s);
    assert_line_has(&focused, "▎", "Tasks", "A1 focus bar on the card");

    assert_line_has(&focused, "#1040", "▎", "S1 cursor at row 0");

    support::press_step(&s, "down");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A2 cursor on row 1",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains("#1041") && l.contains('▎'))
        },
    );
    support::press_step(&s, "s");
    support::wait_screen(&mut s, case_timeout(&case), "A2 sort applied", |screen| {
        let text = support::screen_text(screen);
        !text.contains("unsorted") && (text.contains('▴') || text.contains('▾'))
    });

    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "Selected", "N2 selection status");
    let selected = live_text(&mut s);
    assert!(!selected.contains("EDIT"), "N2: Enter selects, never edits");
    checkpoint(&mut s, &dir, "01-selected");
    eprintln!("s1 table_page_boot: boot proven, tab/down/s/enter all behaved");
}

#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_table_select_cursor() {
    let dir = s1_dir("s1_table_select_cursor");
    let case = s1_case("table_select_cursor", &["--page", "tables"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::drive_with_timeout(&mut s, &["tab", "down", "down"], case.timeout_ms);
    waits::wait_state(&mut s, "Columns", "cursor-moved table footer");
    checkpoint(&mut s, &dir, "cursor-moved");
    let text = live_text(&mut s);

    assert_line_has(&text, "#1042", "▎", "V1 cursor row");
    for needle in ["Move", "Columns", "Sort", "Select", "Next"] {
        assert!(text.contains(needle), "V1: footer lacks hint {needle}");
    }

    assert_right_lacks(&text, "#104", "›", "V2 no selection marker");
    assert!(
        text.contains("unsorted · 1–5 of 24"),
        "V2: meta keeps the window"
    );

    assert_line_has(&text, "#1042", "▎", "S1 cursor at (2,0)");

    assert!(text.contains("1–5 of 24"), "S2: viewport offset stays 0");

    assert!(!text.contains("Selected"), "N1: no Selected status");

    assert!(text.contains("unsorted"), "N2: sort untouched");

    support::press_step(&s, "j");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1 cursor on row 3",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains("#1043") && l.contains('▎'))
        },
    );

    support::press_step(&s, "space");
    waits::wait_state(&mut s, "Selected", "A2 space selects");
    assert!(
        !live_text(&mut s).contains("EDIT"),
        "A2: selection, not editing"
    );
    checkpoint(&mut s, &dir, "01-space-selected");
    eprintln!("s1 table_select_cursor: cursor (2,0) proven, j moved, space selected");
}

#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_table_edit_open() {
    let dir = s1_dir("s1_table_edit_open");
    let case = s1_case("table_edit_open", &["--page", "editabletables"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::drive_with_timeout(&mut s, &["tab", "enter"], case.timeout_ms);
    waits::wait_state(&mut s, "EDIT", "editing footer");
    checkpoint(&mut s, &dir, "editing");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    for needle in ["EDIT", "Commit", "Cancel", "Next cell"] {
        assert!(text.contains(needle), "V1: footer lacks {needle}");
    }
    assert!(
        text.contains("…te limiting to auth endpoints"),
        "V1: draft prefilled with the cell tail"
    );
    let (draft_row, draft_col) = find_pos(&mut s, "auth endpoints", case_timeout(&case));
    let underlined = (draft_col.saturating_sub(10)..draft_col + 14)
        .filter_map(|col| frame.get(col, draft_row))
        .any(|cell| cell.mods.underline);
    assert!(underlined, "V1: draft carries the accent underline");
    assert!(
        frame.cursor.visible && frame.cursor.y == draft_row,
        "V1: hardware cursor sits on the draft row (cursor {:?} vs row {draft_row})",
        (frame.cursor.x, frame.cursor.y, frame.cursor.visible),
    );

    assert!(text.contains("0 edits · 1–10 of 14"), "V2: zero-edits meta");
    for needle in [
        "cell cursor (navigation)",
        "editing cursor + accent underline",
    ] {
        assert!(text.contains(needle), "V2: legend lacks {needle:?}");
    }

    assert_line_has(&text, "#1040", "▎", "S1 editor on row 0");

    assert!(text.contains("0 edits"), "S2: edits counter stays 0");

    assert_line_lacks(&text, "#104", "!", "N1 no pre-commit error");

    assert_eq!(
        text.matches("…te limiting").count(),
        1,
        "N2: exactly one draft cell"
    );
    assert_eq!(text.matches("EDIT").count(), 1, "N2: exactly one EDIT");

    support::drive_with_timeout(&mut s, &["type:z"], case.timeout_ms);
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A2 typed key inserted",
        |screen| support::screen_text(screen).contains("endpointsz"),
    );

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "EDIT", "A1 edit cancelled");
    assert!(
        live_text(&mut s).contains("0 edits"),
        "A1: cancel keeps the counter at 0"
    );
    checkpoint(&mut s, &dir, "01-cancelled");
    eprintln!("s1 table_edit_open: draft + underline + cursor proven, z inserted, esc cancelled");
}

#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_table_edit_commit() {
    let dir = s1_dir("s1_table_edit_commit");
    let case = s1_case("table_edit_commit", &["--page", "editabletables"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::drive_with_timeout(
        &mut s,
        &["tab", "enter", "type:x", "enter", "wait:1 edits"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "committed");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    assert!(
        text.contains("1 edits · 1–10 of 14"),
        "V1/S1: one-edits meta"
    );
    assert!(text.contains("Cell saved"), "V1: Cell saved status");
    assert!(!text.contains("EDIT"), "V1/S2/N1: EDIT mode cleared");

    let (cell_row, _) = find_pos(&mut s, "#1040", case_timeout(&case));
    let underlined = (0..frame.cols)
        .filter_map(|col| frame.get(col, cell_row))
        .any(|cell| cell.mods.underline);
    assert!(!underlined, "V2: no underline remains on the cell");

    assert_line_lacks(&text, "#104", "!", "N2 no validator error");

    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "EDIT", "V2 re-edit shows the draft");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "V2 draft carries the appended x",
        |screen| support::screen_text(screen).contains("endpointsx"),
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "EDIT", "V2 draft closed");

    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "EDIT", "A2 editing (0,1) again");
    support::press_step(&s, "tab");
    waits::wait_state(&mut s, "2 edits", "A2 tab committed (0,1)");
    assert!(
        live_text(&mut s).contains("EDIT"),
        "A2: still editing after tab"
    );

    let owner_frame = live_frame(&mut s);
    let (mira_row, mira_col) = find_pos(&mut s, "mira", case_timeout(&case));
    let owner_underlined = (mira_col..mira_col + 4)
        .filter_map(|col| owner_frame.get(col, mira_row))
        .any(|cell| cell.mods.underline);
    assert!(owner_underlined, "A2: draft on Owner (0,2)");
    support::press_step(&s, "tab");
    waits::wait_state(&mut s, "3 edits", "A2 tab committed (0,2)");
    support::press_step(&s, "tab");
    waits::wait_state(&mut s, "4 edits", "A2 tab committed (0,4)");
    checkpoint(&mut s, &dir, "01-tab3");

    support::drive_with_timeout(&mut s, &["type:7"], case.timeout_ms);
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A2 distinguishing suffix typed",
        |screen| support::screen_text(screen).contains("147"),
    );
    support::press_step(&s, "tab");
    waits::wait_gone(&mut s, "EDIT", "A2 LeaveForward left the card");
    assert!(
        live_text(&mut s).contains("4 edits"),
        "A2: leaving commit drops the Committed event (counter stays 4)"
    );
    checkpoint(&mut s, &dir, "02-left-card");

    support::press_step(&s, "shift+tab");
    waits::wait_state(&mut s, "▎Tasks", "A2 focus back on the card");
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "EDIT", "A2 re-editing (0,5)");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A2 leaving commit wrote the data",
        |screen| support::screen_text(screen).contains("147"),
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "EDIT", "A2 draft closed");
    checkpoint(&mut s, &dir, "03-data-proven");

    let case2 = s1_case(
        "table_edit_commit_empty",
        &["--page", "editabletables"],
        80,
        24,
    );
    write_provenance_named(&dir, &case2, "provenance-empty.txt");
    let mut s2 = support::spawn_boot(&case2);
    let mut clear = vec!["tab", "enter", "wait:EDIT"];
    clear.extend(std::iter::repeat_n("backspace", 35));
    support::drive_with_timeout(&mut s2, &clear, case2.timeout_ms);
    support::press_step(&s2, "enter");

    support::wait_screen(
        &mut s2,
        case_timeout(&case2),
        "A1 validator error on the row",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains("#1040") && l.contains('!'))
        },
    );
    checkpoint(&mut s2, &dir, "02-empty-state");
    assert!(live_text(&mut s2).contains("EDIT"), "A1: stays editing");

    assert!(
        live_text(&mut s2).contains("Task name cannot be empty"),
        "A1: validator message in the meta"
    );

    support::press_step(&s2, "escape");
    waits::wait_gone(&mut s2, "EDIT", "A1 error cancelled");
    assert!(
        live_text(&mut s2).contains("0 edits"),
        "A1: empty commit counts nothing"
    );
    checkpoint(&mut s2, &dir, "03-empty-rejected");
    eprintln!("s1 table_edit_commit: commit + x proven, tab chain + LeaveForward, empty rejected");
}

#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_table_hover_lift() {
    let dir = s1_dir("s1_table_hover_lift");
    let case = s1_case("table_hover_lift", &["--page", "tables"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let idle_text = live_text(&mut s);
    let idle = live_frame(&mut s);
    let (hover_row, _) = find_pos(&mut s, "#1042", case_timeout(&case));

    hover_over(&mut s, "#1042", case_timeout(&case));

    let hovered = waits::wait_frame_where(
        &mut s,
        case_timeout(&case),
        "V1 hover lift paints",
        |frame| frame.digest() != idle.digest(),
    );
    checkpoint(&mut s, &dir, "hovered");
    let text = live_text(&mut s);

    assert_eq!(hovered.text(), idle.text(), "V1: identical text");
    assert_eq!(text, idle_text, "V1: live text identical too");

    assert_eq!(
        text.matches('▎').count(),
        1,
        "V2: only the nav focus bar shows"
    );

    let mut diff_rows = std::collections::BTreeSet::new();
    for col in 0..idle.cols {
        let a = idle.get(col, hover_row);
        let b = hovered.get(col, hover_row);
        if a.map(cell_sig) != b.map(cell_sig) {
            diff_rows.insert(hover_row);
            break;
        }
    }
    for row in 0..idle.rows {
        if row == hover_row {
            continue;
        }
        for col in 0..idle.cols {
            if idle.get(col, row).map(cell_sig) != hovered.get(col, row).map(cell_sig) {
                diff_rows.insert(row);
                break;
            }
        }
    }
    assert_eq!(
        diff_rows,
        std::collections::BTreeSet::from([hover_row]),
        "S1/N1: exactly the hovered row changes plane"
    );

    assert!(
        text.contains("Tab Into page"),
        "S1: footer still the nav footer"
    );

    let idle_footer = idle_text.lines().last().unwrap_or_default().to_string();
    let footer = text.lines().last().unwrap_or_default().to_string();
    assert_eq!(footer, idle_footer, "A1: footer untouched by hover");

    assert_right_lacks(&text, "#104", "›", "N2 no selection marker");
    assert!(!text.contains("Selected"), "N2: no selection status");

    hover_over(&mut s, "#1043", case_timeout(&case));
    let moved = waits::wait_frame_where(
        &mut s,
        case_timeout(&case),
        "A2 hover follows the pointer",
        |frame| frame.digest() != hovered.digest(),
    );
    let (row3, _) = find_pos(&mut s, "#1043", case_timeout(&case));
    for col in 0..idle.cols {
        assert_cell_same(&moved, &idle, col, hover_row, "A2 #1042 rested");
    }
    let lifted = (0..idle.cols)
        .filter(|col| {
            let a = idle.get(*col, row3).map(cell_sig);
            let b = moved.get(*col, row3).map(cell_sig);
            a != b
        })
        .count();
    assert!(lifted > 0, "A2: #1043 row lifts under the pointer");
    checkpoint(&mut s, &dir, "01-hover-moved");
    eprintln!("s1 table_hover_lift: plane lift proven, follows pointer, selects nothing");
}

fn cell_sig(cell: &tuiscotti::Cell) -> (String, TColor, TColor) {
    (cell.symbol.clone(), cell.fg, cell.bg)
}

#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_table_editable_boot() {
    let dir = s1_dir("s1_table_editable_boot");
    let case = s1_case("table_editable_boot", &["--page", "editabletables"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);

    assert_eq!(
        boot.matches('▎').count(),
        1,
        "V1/N2: only the nav focus bar shows"
    );
    assert!(boot.contains("0 edits · 1–10 of 14"), "V1: zero-edits meta");
    assert!(
        boot.contains("cell cursor (navigation)"),
        "V1: legend renders"
    );

    assert_line_has(&boot, "Owner", "…", "V2 overflow marker");
    assert_line_lacks(&boot, "#104", "!", "V2/S2 preset error off-screen");

    assert!(!boot.contains("EDIT"), "N1: no EDIT at boot");

    support::press_step(&s, "tab");
    waits::wait_state(&mut s, "Cell", "S1 table footer");
    let frame = live_frame(&mut s);
    let (row0, id_col) = find_pos(&mut s, "#1040", case_timeout(&case));
    let (_, task_col) = find_pos(&mut s, "Add rate limiting", case_timeout(&case));
    let id_bg = frame
        .get(id_col, row0)
        .unwrap_or_else(|| panic!("S1: ID cell ({id_col}, {row0}) missing"))
        .bg;
    let task_bg = frame
        .get(task_col, row0)
        .unwrap_or_else(|| panic!("S1: Task cell ({task_col}, {row0}) missing"))
        .bg;
    assert_ne!(task_bg, id_bg, "S1/A1: cursor cell (0,1) paints reversed");

    support::press_step(&s, "right");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A2 reversed cell moved right",
        |screen| {
            let probe = frame_from_screen(screen, "default");
            probe.get(task_col, row0).map(|c| c.bg) != Some(task_bg)
        },
    );
    support::press_step(&s, "left");
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "EDIT", "A2/S1 enter edits (validator path)");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "EDIT", "A2 editor closed");
    checkpoint(&mut s, &dir, "01-entered");

    support::drive_with_timeout(
        &mut s,
        &["down", "down", "right", "right", "right", "right"],
        case.timeout_ms,
    );
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "S2 preset error scrolled into view",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('!') && l.contains("fix/checkout") && !l.contains("valid"))
        },
    );
    checkpoint(&mut s, &dir, "02-preset-error");
    eprintln!("s1 table_editable_boot: unfocused boot + reversed (0,1) + preset error proven");
}

#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_grid_wheel_fade() {
    let dir = s1_dir("s1_grid_wheel_fade");
    let case = s1_case("grid_wheel_fade", &["--page", "datagrid"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot_text = live_text(&mut s);
    let boot = live_frame(&mut s);
    assert!(boot_text.contains("rows 1–14 of 40"), "S1: boot offset 0");

    wheel_over(
        &mut s,
        "Northwind Traders",
        Wheel::Down,
        1,
        case_timeout(&case),
    );
    waits::wait_state(&mut s, "rows 4–17 of 40", "A2 first notch advances");
    wheel_over(
        &mut s,
        "Northwind Traders",
        Wheel::Down,
        1,
        case_timeout(&case),
    );
    waits::wait_state(&mut s, "rows 7–20 of 40", "wheel-faded meta");
    checkpoint(&mut s, &dir, "wheel-faded");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    assert!(
        text.contains("rows 7–20 of 40 loaded · ~4,812 total"),
        "V1: scrolled meta"
    );
    assert_line_has(&text, "1007", "Wide World Importers", "V1 first row");

    let (r7, _) = find_pos(&mut s, "1007", case_timeout(&case));
    let (r8, _) = find_pos(&mut s, "1008", case_timeout(&case));
    let (r_mid, _) = find_pos(&mut s, "1013", case_timeout(&case));
    let outer = row_fg_median(&frame, r7, "V2 outer edge");
    let inner = row_fg_median(&frame, r8, "V2 inner edge");
    let mid = row_fg_median(&frame, r_mid, "V2 interior");
    assert_faded(inner, mid, "V2 inner edge fades");
    assert_faded(outer, inner, "V2 outer edge fades deepest");
    let (header_row, _) = find_pos(&mut s, "customer", case_timeout(&case));
    assert_row_same(&boot, &frame, header_row, "V2/N1 header never fades");
    let track_col = (0..frame.cols)
        .rfind(|col| {
            frame
                .get(*col, r7)
                .is_some_and(|c| c.symbol == "│" || c.symbol == "┃")
        })
        .expect("V2: scrollbar column on the first body row");
    assert_eq!(
        frame.get(track_col, r7).map(|c| c.symbol.as_str()),
        Some("│"),
        "V2: track (not thumb) at the top"
    );
    let thumb_below =
        (r7 + 1..frame.rows).any(|row| frame.get(track_col, row).is_some_and(|c| c.symbol == "┃"));
    assert!(thumb_below, "V2: thumb sits below the track top");

    assert!(
        !text.contains('▴') && !text.contains('▾'),
        "S1: sort stays null"
    );
    assert!(!text.contains("EDIT"), "S1: editing untouched");

    assert!(
        text.contains("Tab Into page"),
        "S2: footer still the nav footer"
    );
    assert_eq!(text.matches('▎').count(), 1, "S2: no focus bar in the card");

    assert!(text.contains("of 40 loaded"), "N2: page_rows stays 40");

    let (end_row, end_col) = find_pos(&mut s, "Northwind Traders", case_timeout(&case));
    let end = wheel_until_stable(
        &mut s,
        end_col,
        end_row,
        Wheel::Down,
        case_timeout(&case),
        "A1 bottom offset",
    );
    checkpoint(&mut s, &dir, "01-end-reached");

    assert!(
        end.contains("rows 28–40 of 40"),
        "A1: bottom offset is rows 28–40"
    );
    assert!(
        end.contains("Enter fetches more"),
        "A1: fetch row anchors the bottom"
    );
    wheel_at(&mut s, end_col, end_row, Wheel::Down, 3);
    let stance = settle_text(&mut s, Duration::from_millis(3000), "A1 stance");
    assert_eq!(stance, end, "A1: Consumed at the bottom");
    checkpoint(&mut s, &dir, "01-bottom");
    eprintln!("s1 grid_wheel_fade: offset 0→3→6, two-deep fade, bottom Consumed");
}

#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_code_wheel_fade() {
    let dir = s1_dir("s1_code_wheel_fade");
    let case = s1_case("code_wheel_fade", &["--page", "codeeditor"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_frame(&mut s);

    wheel_over(
        &mut s,
        "pub async fn fetch",
        Wheel::Down,
        2,
        case_timeout(&case),
    );
    waits::wait_state(&mut s, "7–11 of 26", "wheel-faded window");
    checkpoint(&mut s, &dir, "wheel-faded");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    assert!(text.contains("7–11 of 26"), "V1: window label");
    assert!(
        text.contains("Err(e) if e.is_transient()"),
        "V1: line 7 content"
    );
    assert_line_has(&text, "retry.rs", "3 blocks", "V1 header");

    let (r7, _) = find_pos(&mut s, "Err(e)", case_timeout(&case));
    let (r9, _) = find_pos(&mut s, "sleep(delay)", case_timeout(&case));
    let (r10, _) = find_pos(&mut s, "delay *= 2", case_timeout(&case));
    assert_faded(
        row_fg_median(&frame, r7, "V2 top edge"),
        row_fg_median(&frame, r9, "V2 interior"),
        "V2 top edge fades",
    );

    let bottom = r10 + 1;
    let bottom_line = live_text(&mut s)
        .lines()
        .nth(usize::from(bottom))
        .unwrap_or_default()
        .to_string();
    assert!(
        bottom_line.contains('}'),
        "V2: line 11 closes the viewport: {bottom_line:?}"
    );
    assert_faded(
        row_fg_median(&frame, bottom, "V2 bottom edge"),
        row_fg_median(&frame, r9, "V2 interior"),
        "V2 bottom edge fades",
    );

    assert_line_has(&text, "Block", "1 of 3", "S1: blocks untouched");

    assert_line_has(&text, "Cursor", "ln 1 · col 1", "S2/A2: cursor stays");
    assert_line_has(&text, "Mode", "navigating", "S2/A2: still navigating");

    let (header_row, _) = find_pos(&mut s, "retry.rs", case_timeout(&case));
    assert_row_same(&boot, &frame, header_row, "N1 header never fades");

    assert_line_has(&text, "Diagnostics", "0", "N2: diagnostics stay");

    let (end_row, end_col) = find_pos(&mut s, "sleep(delay)", case_timeout(&case));
    let end = wheel_until_stable(
        &mut s,
        end_col,
        end_row,
        Wheel::Down,
        case_timeout(&case),
        "A1 bottom offset",
    );
    assert!(end.contains("22–26 of 26"), "A1: bottom offset is 22–26");
    wheel_at(&mut s, end_col, end_row, Wheel::Down, 3);
    let stance = settle_text(&mut s, Duration::from_millis(3000), "A1 stance");
    assert_eq!(stance, end, "A1: Consumed at the bottom");
    checkpoint(&mut s, &dir, "01-bottom");
    eprintln!("s1 code_wheel_fade: 7–11 proven, both edges fade, bottom Consumed");
}

#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_diff_wheel_review() {
    let dir = s1_dir("s1_diff_wheel_review");
    let case = s1_case("diff_wheel_review", &["--page", "diff"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::drive_with_timeout(&mut s, &["tab", "enter", "wait:● Review"], case.timeout_ms);

    let review_text = settle_text(&mut s, Duration::from_millis(5000), "review steady");
    checkpoint(&mut s, &dir, "00-review");
    let review = live_frame(&mut s);

    wheel_over(&mut s, "attempts = 3", Wheel::Down, 2, case_timeout(&case));
    waits::wait_state(&mut s, "@@ -9,6 +10,7 @@", "wheel-faded hunk header");
    settle_text(&mut s, Duration::from_millis(5000), "wheel-faded steady");
    checkpoint(&mut s, &dir, "wheel-faded");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    assert!(text.contains("@@ -9,6 +10,7 @@"), "V1: hunk header");
    assert!(text.contains("attempts = 3"), "V1: old column");
    assert!(text.contains("attempts = 5"), "V1: new column");
    let (att_row, att_col) = find_pos(&mut s, "attempts = 3", case_timeout(&case));
    let bold_run = (att_col..att_col + 12)
        .filter_map(|col| frame.get(col, att_row))
        .any(|cell| cell.mods.bold);
    assert!(bold_run, "V1: changed run reads bold");

    let (top_row, _) = find_pos(&mut s, "╭─ Diff", case_timeout(&case));
    let (bot_row, _) = find_pos(&mut s, "╰─", case_timeout(&case));
    assert_faded(
        row_fg_median(&frame, top_row + 1, "V2 top edge"),
        row_fg_median(&frame, att_row, "V2 interior"),
        "V2 top edge fades",
    );
    assert_faded(
        row_fg_median(&frame, bot_row - 1, "V2 bottom edge"),
        row_fg_median(&frame, att_row, "V2 interior"),
        "V2 bottom edge fades",
    );

    assert_ne!(text, review_text, "S2: the wheel moved the offset");
    assert!(text.contains("● Review"), "S1/A2: mode stays Review");

    let (ctl_row, _) = find_pos(&mut s, "● Review", case_timeout(&case));
    assert_row_same(&review, &frame, ctl_row, "N1 controls row never fades");

    assert!(
        !text.contains("No file selected"),
        "N2/A2: file stays loaded"
    );

    let (end_row, end_col) = find_pos(&mut s, "attempts = 3", case_timeout(&case));
    let end = wheel_until_stable(
        &mut s,
        end_col,
        end_row,
        Wheel::Down,
        case_timeout(&case),
        "A1 end offset",
    );

    assert!(
        end.contains("@@") || end.contains("attempts"),
        "A1: wheeled to the hunk tail, fixture intact"
    );
    wheel_at(&mut s, end_col, end_row, Wheel::Down, 3);
    let stance = settle_text(&mut s, Duration::from_millis(3000), "A1 stance");
    assert_eq!(stance, end, "A1: Consumed at the end");
    checkpoint(&mut s, &dir, "01-end");
    eprintln!("s1 diff_wheel_review: review + bold runs + both fades + end Consumed");
}

#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_scroll_page_prose() {
    let dir = s1_dir("s1_scroll_page_prose");
    let case = s1_case(
        "scroll_page_prose",
        &[
            "--page",
            "scrolling",
            "--motion",
            "paused",
            "--frame",
            "1600",
        ],
        80,
        24,
    );
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    waits::wait_state(&mut s, "739.63s", "paused log endpoint");
    checkpoint(&mut s, &dir, "00-boot");
    let boot_text = live_text(&mut s);
    assert!(boot_text.contains("1–15 of"), "S1: prose boot window 1–15");

    support::drive_with_timeout(&mut s, &["tab", "pagedown"], case.timeout_ms);
    waits::wait_state(&mut s, "16–30", "paged prose window");
    checkpoint(&mut s, &dir, "paged");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    assert_line_has(&text, "16–30", "▎", "V1 prose window + focus");
    assert!(text.contains("Log 1986–2000"), "V1/S1 log tail intact");

    let (prose_first, prose_first_col) = find_pos(&mut s, "reports back", case_timeout(&case));
    let (prose_mid, prose_mid_col) = find_pos(&mut s, "Each step is", case_timeout(&case));
    let (prose_last, prose_last_col) = find_pos(&mut s, "and every", case_timeout(&case));
    assert_faded(
        span_fg_median(&frame, prose_first, prose_first_col, 12, "V2 prose top"),
        span_fg_median(&frame, prose_mid, prose_mid_col, 12, "V2 prose interior"),
        "V2 prose top edge fades",
    );
    assert_faded(
        span_fg_median(&frame, prose_last, prose_last_col, 9, "V2 prose bottom"),
        span_fg_median(&frame, prose_mid, prose_mid_col, 12, "V2 prose interior"),
        "V2 prose bottom edge fades",
    );
    let (log_first, log_first_col) = find_pos(&mut s, "734.45s", case_timeout(&case));
    let (log_mid, log_mid_col) = find_pos(&mut s, "737.04s", case_timeout(&case));
    let (log_last, log_last_col) = find_pos(&mut s, "739.63s", case_timeout(&case));
    assert_faded(
        span_fg_median(&frame, log_first, log_first_col, 7, "V2 log top"),
        span_fg_median(&frame, log_mid, log_mid_col, 7, "V2 log interior"),
        "V2 log top edge fades",
    );
    assert_same_tone(
        span_fg_median(&frame, log_last, log_last_col, 7, "V2 log bottom"),
        span_fg_median(&frame, log_mid, log_mid_col, 7, "V2 log interior"),
        "V2/N1 log bottom whole at the tail",
    );

    let (first, last) = parse_window("16–30", "S1 paged window");
    assert_eq!((first, last), (16, 30), "S1: prose offset 0 → 15");

    assert!(text.contains("PgUp PgDn"), "S2: focused-column footer");

    assert!(text.contains("1986–2000"), "N1: log keeps the tail");

    assert!(text.contains("1–15 of"), "N2: list keeps 1–15");
    assert!(
        text.contains("Row 001") && text.contains("Row 015"),
        "N2: list rows intact"
    );

    let (b_first, b_last) = parse_window("1–15", "A1 boot window");
    assert_eq!(
        last - first,
        b_last - b_first,
        "A1: advance equals viewport_len"
    );

    support::press_step(&s, "down");
    waits::wait_state(&mut s, "17–31", "A2 prose advanced one line");
    let moved = live_text(&mut s);
    assert!(
        moved.contains("Lon… 1–15") || moved.contains("1–15 of"),
        "A2: list still 1–15"
    );
    assert!(moved.contains("1986–2000"), "A2: log still at the tail");
    checkpoint(&mut s, &dir, "01-down");
    eprintln!("s1 scroll_page_prose: 16–30 + both fades, log tail whole, down is column-local");
}

#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_scroll_wheel_offtail() {
    let dir = s1_dir("s1_scroll_wheel_offtail");
    let case = s1_case(
        "scroll_wheel_offtail",
        &[
            "--page",
            "scrolling",
            "--motion",
            "paused",
            "--frame",
            "1600",
        ],
        80,
        24,
    );
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    waits::wait_state(&mut s, "739.63s", "paused log endpoint");
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_frame(&mut s);
    let (boot_last, boot_last_col) = find_pos(&mut s, "739.63s", case_timeout(&case));
    let (boot_mid, boot_mid_col) = find_pos(&mut s, "737.04s", case_timeout(&case));

    assert_same_tone(
        span_fg_median(&boot, boot_last, boot_last_col, 7, "boot log bottom"),
        span_fg_median(&boot, boot_mid, boot_mid_col, 7, "boot log interior"),
        "S1: boot log bottom whole at the tail",
    );

    wheel_over(&mut s, "739.63s", Wheel::Up, 2, case_timeout(&case));
    waits::wait_state(&mut s, "1980–1994", "off-tail window");
    checkpoint(&mut s, &dir, "off-tail");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    assert!(text.contains("Log 1980–1994"), "V1: off-tail window");

    let (log_first, log_first_col) = find_pos(&mut s, "732.23s", case_timeout(&case));
    let (log_mid, log_mid_col) = find_pos(&mut s, "734.82s", case_timeout(&case));
    let (log_last, log_last_col) = find_pos(&mut s, "737.41s", case_timeout(&case));
    assert_faded(
        span_fg_median(&frame, log_first, log_first_col, 7, "V2 log top"),
        span_fg_median(&frame, log_mid, log_mid_col, 7, "V2 log interior"),
        "V2 log top edge fades",
    );
    assert_faded(
        span_fg_median(&frame, log_last, log_last_col, 7, "V2 log bottom"),
        span_fg_median(&frame, log_mid, log_mid_col, 7, "V2 log interior"),
        "V2/N2 log bottom fade returns off the tail",
    );
    let (prose_first, prose_first_col) = find_pos(&mut s, "Junie works", case_timeout(&case));
    let (prose_mid, prose_mid_col) = find_pos(&mut s, "relevant", case_timeout(&case));
    let (prose_last, prose_last_col) = find_pos(&mut s, "tests, and", case_timeout(&case));
    assert_same_tone(
        span_fg_median(&frame, prose_first, prose_first_col, 11, "V2 prose top"),
        span_fg_median(&frame, prose_mid, prose_mid_col, 8, "V2 prose interior"),
        "V2 prose top whole at the top",
    );
    assert_faded(
        span_fg_median(&frame, prose_last, prose_last_col, 10, "V2 prose bottom"),
        span_fg_median(&frame, prose_mid, prose_mid_col, 8, "V2 prose interior"),
        "V2 prose bottom edge fades",
    );

    let (first, _) = parse_window("1980–1994", "S1 off-tail window");
    assert_eq!(first, 1980, "S1: log offset up six");

    assert!(
        text.contains("Wra… 1–15") || text.contains("1–15 of"),
        "S2/N1 prose stays"
    );
    assert!(
        text.contains("Row 001") && text.contains("Row 015"),
        "S2/N1 list stays"
    );

    assert!(
        text.contains("Tab Into page"),
        "A2: footer still the nav footer"
    );

    wheel_over(&mut s, "737.41s", Wheel::Up, 2, case_timeout(&case));
    waits::wait_state(&mut s, "1974–1988", "A1 further ups scroll");
    checkpoint(&mut s, &dir, "01-further-up");
    eprintln!("s1 scroll_wheel_offtail: 1980–1994 + both fades, prose/list pinned, ups continue");
}

#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_nav_wheel_demo() {
    let dir = s1_dir("s1_nav_wheel_demo");
    let case = s1_case("nav_wheel_demo", &["--page", "sidebars"], 72, 20);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot_text = live_text(&mut s);
    let boot = live_frame(&mut s);

    assert!(
        boot_text.contains("Workspace"),
        "S1: boot shows the demo top"
    );
    assert!(
        !boot_text.contains("Preferences"),
        "S1: Preferences folded at boot"
    );
    assert!(
        !boot_text.contains("Appearance"),
        "S1: Appearance folded at boot"
    );

    wheel_over(&mut s, "Members", Wheel::Down, 2, case_timeout(&case));
    waits::wait_state(&mut s, "Preferences", "wheel-faded demo window");
    checkpoint(&mut s, &dir, "wheel-faded");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    assert!(text.contains("B Branches"), "V1: first demo row");
    assert!(text.contains("A Appearance"), "V1: last demo row");
    assert!(text.contains("Collapse"), "V1: Collapse footer persists");
    let (demo_row, _) = find_pos(&mut s, "M Members", case_timeout(&case));
    let scrollbar = (0..frame.cols).any(|col| {
        frame
            .get(col, demo_row)
            .is_some_and(|c| c.symbol == "│" || c.symbol == "┃")
    });
    assert!(scrollbar, "V1: scrollbar beside the demo list");

    assert!(
        !text.contains("Workspace"),
        "S1: Workspace left above the fold"
    );
    for needle in ["Preferences", "K Keyboard", "A Appearance"] {
        assert!(text.contains(needle), "S1: {needle} entered the viewport");
    }
    assert!(
        boot_text.contains("Collapse") && text.contains("Collapse"),
        "S1: Collapse footer persists"
    );

    let (branches_row, branches_col) = find_pos(&mut s, "B Branches", case_timeout(&case));
    let (members_row, members_col) = find_pos(&mut s, "M Members", case_timeout(&case));
    assert_faded(
        span_fg_median(&frame, branches_row, branches_col, 10, "V2 demo top"),
        span_fg_median(&frame, members_row, members_col, 9, "V2 demo interior"),
        "V2 demo top edge fades",
    );
    let (nav_first, nav_first_col) = find_pos(&mut s, "Buttons", case_timeout(&case));
    let (nav_last, nav_last_col) = find_pos(&mut s, "Code editor", case_timeout(&case));
    assert_faded(
        span_fg_median(&frame, nav_last, nav_last_col, 11, "V2 shell nav bottom"),
        span_fg_median(&frame, nav_first, nav_first_col, 7, "V2 shell nav interior"),
        "V2 shell nav bottom edge fades",
    );

    assert!(text.contains("Tasks"), "S2/N1: demo current persists");
    assert!(
        text.contains("Tab Into page"),
        "S2: footer still the nav footer"
    );

    let bars: Vec<&str> = text.lines().filter(|l| l.contains('▎')).collect();
    assert_eq!(bars.len(), 2, "S2: nav bar + legend sample only: {bars:?}");
    assert!(
        bars.iter().any(|l| l.contains("Sidebars")),
        "S2: nav bar present"
    );
    assert!(
        bars.iter().any(|l| l.contains("keyboard")),
        "S2: legend sample present"
    );

    let demo_marks = |t: &str| {
        t.lines()
            .filter(|l| {
                l.contains("Branches")
                    || l.contains("Members")
                    || l.contains("Collapse")
                    || l.contains("Preferences")
            })
            .filter(|l| l.contains('›'))
            .count()
    };
    assert_eq!(
        demo_marks(&text),
        demo_marks(&boot_text),
        "N1: demo › marks stable"
    );

    for row in 0..boot.rows {
        let a: String = (0..17)
            .filter_map(|col| boot.get(col, row).map(|c| c.symbol.clone()))
            .collect();
        let b: String = (0..17)
            .filter_map(|col| frame.get(col, row).map(|c| c.symbol.clone()))
            .collect();
        assert_eq!(a, b, "N2: shell nav row {row} unmoved");
    }

    let (end_row, end_col) = find_pos(&mut s, "M Members", case_timeout(&case));
    let end = wheel_until_stable(
        &mut s,
        end_col,
        end_row,
        Wheel::Down,
        case_timeout(&case),
        "A2 demo bottom",
    );
    wheel_at(&mut s, end_col, end_row, Wheel::Down, 3);
    let stance = settle_text(&mut s, Duration::from_millis(3000), "A2 stance");
    assert_eq!(stance, end, "A2: Consumed at the demo bottom");
    checkpoint(&mut s, &dir, "01-bottom");
    eprintln!("s1 nav_wheel_demo: Collapse persists, both fades, gutter pinned, bottom Consumed");
}

#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_term_wheel_scrollback() {
    let dir = s1_dir("s1_term_wheel_scrollback");
    let case = s1_case(
        "term_wheel_scrollback",
        &["--page", "terminal", "--motion", "paused", "--frame", "60"],
        80,
        24,
    );
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot_text = live_text(&mut s);
    let boot = live_frame(&mut s);
    assert!(boot_text.contains("2 of 7"), "S2: boot rail at 2 of 7");

    wheel_over(
        &mut s,
        "#4 RUN cargo build",
        Wheel::Up,
        2,
        case_timeout(&case),
    );
    waits::wait_state(&mut s, "scrollback ↑5", "scrollback header");
    checkpoint(&mut s, &dir, "scrollback");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    assert!(text.contains("scrollback ↑5"), "V1: scrollback header");
    for needle in ["Resolve workspace", "Pull base image", "layer 01/12"] {
        assert!(text.contains(needle), "V1: transcript lacks {needle}");
    }

    let (prompt_row, prompt_col) = find_pos(&mut s, "payments-platform", case_timeout(&case));
    let prompt = span_fg_median(&frame, prompt_row, prompt_col, 17, "V2 prompt row");
    assert_eq!(
        prompt, 2_550_000,
        "V2: prompt row whole (white; any fade would dim below 255)"
    );
    let (r_run, r_run_col) = find_pos(&mut s, "▶ Resolve", case_timeout(&case));
    let (r_pull, r_pull_col) = find_pos(&mut s, "▶ Pull", case_timeout(&case));
    assert_same_tone(
        span_fg_median(&frame, r_run, r_run_col, 11, "V2 top ▶ row"),
        span_fg_median(&frame, r_pull, r_pull_col, 8, "V2 interior ▶ row"),
        "V2: top ▶ row whole",
    );
    let (r_done, r_done_col) = find_pos(&mut s, "✓ Resolve", case_timeout(&case));
    let (r_done2, r_done2_col) = find_pos(&mut s, "✓ Pull", case_timeout(&case));
    assert_same_tone(
        span_fg_median(&frame, r_done, r_done_col, 11, "V2 top ✓ row"),
        span_fg_median(&frame, r_done2, r_done2_col, 8, "V2 interior ✓ row"),
        "V2: top ✓ row whole",
    );
    let (b_whole, b_whole_col) = find_pos(&mut s, "#1 RUN", case_timeout(&case));
    let (b_inner, b_inner_col) = find_pos(&mut s, "#2 COPY", case_timeout(&case));
    let (b_outer, b_outer_col) = find_pos(&mut s, "#3 RUN", case_timeout(&case));
    let whole = span_fg_median(&frame, b_whole, b_whole_col, 12, "V2 bottom interior");
    let inner = span_fg_median(&frame, b_inner, b_inner_col, 12, "V2 bottom inner");
    let outer = span_fg_median(&frame, b_outer, b_outer_col, 12, "V2 bottom outer");
    assert_faded(inner, whole, "V2 viewport bottom inner fades");
    assert_faded(outer, inner, "V2 viewport bottom outer fades deepest");
    for needle in ["01 Resolve", "03 Build", "07 Ready"] {
        let (rail_row, rail_col) = find_pos(&mut s, needle, case_timeout(&case));
        assert_cell_same(
            &boot,
            &frame,
            rail_col,
            rail_row,
            "V2 step rail never fades",
        );
    }

    assert!(text.contains("2 of 7"), "S2/N2/A2: rail stays at 2 of 7");

    assert!(!text.contains("following"), "N1: no following indicator");

    wheel_over(&mut s, "layer 04/12", Wheel::Down, 2, case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "S1 returned to the boot tail",
        |screen| support::screen_text(screen) == boot_text,
    );

    let (top_row, top_col) = find_pos(&mut s, "#4 RUN cargo build", case_timeout(&case));
    let top = wheel_until_stable(
        &mut s,
        top_col,
        top_row,
        Wheel::Up,
        case_timeout(&case),
        "A1 top line",
    );
    wheel_at(&mut s, top_col, top_row, Wheel::Up, 3);
    let stance = settle_text(&mut s, Duration::from_millis(3000), "A1 stance");
    assert_eq!(stance, top, "A1: Consumed at the top line");
    checkpoint(&mut s, &dir, "01-top");
    eprintln!("s1 term_wheel_scrollback: ↑5 top-anchored, bottom fade, rail pinned, top Consumed");
}

#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_list_wheel_fade() {
    let dir = s1_dir("s1_list_wheel_fade");
    let case = s1_case("list_wheel_fade", &["--page", "lists"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot_text = live_text(&mut s);
    assert!(boot_text.contains("Python"), "S1: Python visible at boot");

    wheel_over(&mut s, "Python", Wheel::Down, 2, case_timeout(&case));
    waits::wait_state(&mut s, "7–19", "wheel-faded window");
    checkpoint(&mut s, &dir, "wheel-faded");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    assert!(text.contains("7–19 of"), "V1: window label");
    let (swift_row, swift_col) = find_pos(&mut s, "Swift", case_timeout(&case));
    let (csharp_row, _) = find_pos(&mut s, "C#", case_timeout(&case));
    assert!(swift_row < csharp_row, "V1: Swift first in the window");
    assert!(text.contains("Chosen: Rust"), "V1/S2/N1: choice persists");

    let (last_row, last_col) = find_pos(&mut s, "Clojure", case_timeout(&case));
    let (mid_row, mid_col) = find_pos(&mut s, "Haskell", case_timeout(&case));
    assert_faded(
        span_fg_median(&frame, swift_row, swift_col, 5, "V2 top edge"),
        span_fg_median(&frame, mid_row, mid_col, 7, "V2 interior"),
        "V2 top edge fades",
    );
    assert_faded(
        span_fg_median(&frame, last_row, last_col, 7, "V2 bottom edge"),
        span_fg_median(&frame, mid_row, mid_col, 7, "V2 interior"),
        "V2 bottom edge fades",
    );
    assert_eq!(text.matches('▎').count(), 1, "V2: no focus bar on the list");

    let (first, _) = parse_window("7–19", "S1 window");
    assert_eq!(first, 7, "S1: offset 0 → 6");
    assert!(
        text.contains("Tab Into page"),
        "S1: footer still the nav footer"
    );

    assert!(text.contains("2 select"), "N2: filter column intact");
    assert!(text.contains("Space toggle"), "N2: filter hints intact");
    assert!(text.contains("No results for"), "N2: search column intact");

    let (end_row, end_col) = find_pos(&mut s, "Swift", case_timeout(&case));
    let end = wheel_until_stable(
        &mut s,
        end_col,
        end_row,
        Wheel::Down,
        case_timeout(&case),
        "A1 end offset",
    );

    assert!(end.contains("8–20 of"), "A1: end offset is 8–20");
    wheel_at(&mut s, end_col, end_row, Wheel::Down, 3);
    let stance = settle_text(&mut s, Duration::from_millis(3000), "A1 stance");
    assert_eq!(stance, end, "A1: Consumed at the end");

    support::press_step(&s, "tab");
    waits::wait_state(&mut s, "▎", "A2 list focused");
    support::press_step(&s, "up");
    waits::wait_state(&mut s, "Python", "A2 row 0 re-revealed");
    assert!(
        live_text(&mut s).contains("Chosen: Rust"),
        "A2: choice still Rust"
    );
    checkpoint(&mut s, &dir, "01-rerevealed");
    eprintln!("s1 list_wheel_fade: 7–19 + both fades, end Consumed, up re-reveals row 0");
}

#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_tree_wheel_fade() {
    let dir = s1_dir("s1_tree_wheel_fade");
    let case = s1_case("tree_wheel_fade", &["--page", "trees"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    wheel_over(&mut s, "config.rs", Wheel::Down, 1, case_timeout(&case));
    waits::wait_state(&mut s, "2–16 of 16", "wheel-faded window");
    checkpoint(&mut s, &dir, "wheel-faded");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    assert!(text.contains("2–16 of 16"), "V1/S1: window label");
    for needle in ["▸ api", "▸ db", "▸ workers", "▾ tests"] {
        assert!(text.contains(needle), "V1: expansion lacks {needle}");
    }

    let (first_row, _) = find_pos(&mut s, "▸ api", case_timeout(&case));
    let (mid_row, mid_col) = find_pos(&mut s, "main.rs", case_timeout(&case));
    let (last_row, last_col) = find_pos(&mut s, "README.md", case_timeout(&case));
    assert_faded(
        row_fg_median(&frame, first_row, "V2 top edge"),
        row_fg_median(&frame, mid_row, "V2 interior"),
        "V2 top edge fades",
    );
    assert_same_tone(
        span_fg_median(&frame, last_row, last_col, 9, "V2 bottom row"),
        span_fg_median(&frame, mid_row, mid_col, 7, "V2 interior"),
        "V2 bottom stays whole at the last row",
    );

    assert!(text.contains("cursor  src"), "S2: cursor stays on src");

    assert!(
        text.contains("open    5 folders"),
        "N1: still 5 folders open"
    );
    assert!(
        text.contains("Nothing selected"),
        "N2: still nothing selected"
    );

    wheel_over(&mut s, "config.rs", Wheel::Down, 1, case_timeout(&case));
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        live_text(&mut s).contains("2–16 of 16"),
        "A1: second notch Consumed at the end"
    );

    support::press_step(&s, "tab");
    waits::wait_state(&mut s, "▎", "A2 tree focused");
    support::press_step(&s, "down");
    waits::wait_state(&mut s, "cursor  api", "A2 cursor on the api row");
    support::press_step(&s, "right");
    waits::wait_state(&mut s, "open    6 folders", "A2 right unfolds one more");
    support::press_step(&s, "left");
    waits::wait_state(&mut s, "open    5 folders", "A2 left folds it back");
    checkpoint(&mut s, &dir, "01-toggled");
    eprintln!("s1 tree_wheel_fade: 2–16 + top fade, second notch Consumed, toggles work");
}

#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_textarea_wheel_fade() {
    let dir = s1_dir("s1_textarea_wheel_fade");
    let case = s1_case("textarea_wheel_fade", &["--page", "textareas"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot_text = live_text(&mut s);
    assert!(boot_text.contains("1–8 of 28"), "S1: boot window 1–8");

    wheel_over(&mut s, "1. Read", Wheel::Down, 2, case_timeout(&case));
    waits::wait_state(&mut s, "7–14 of 28", "wheel-faded window");
    checkpoint(&mut s, &dir, "wheel-faded");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    assert!(text.contains("7–14 of 28"), "V1: window label");
    assert!(text.contains("7. Open a PR"), "V1: first line");
    assert!(text.contains("14. Keep the public"), "V1: last line");
    assert_eq!(
        text.matches('▎').count(),
        1,
        "V1: no cursor bar in the area"
    );
    assert!(!text.contains('▁'), "V1: no editing cursor mark");

    let (first_row, _) = find_pos(&mut s, "7. Open a PR", case_timeout(&case));
    let (mid_row, _) = find_pos(&mut s, "10. Keep the public", case_timeout(&case));
    let (last_row, _) = find_pos(&mut s, "14. Keep the public", case_timeout(&case));
    assert_faded(
        row_fg_median(&frame, first_row, "V2 top edge"),
        row_fg_median(&frame, mid_row, "V2 interior"),
        "V2 top edge fades",
    );
    assert_faded(
        row_fg_median(&frame, last_row, "V2 bottom edge"),
        row_fg_median(&frame, mid_row, "V2 interior"),
        "V2 bottom edge fades",
    );

    let (first, _) = parse_window("7–14", "S1 window");
    assert_eq!(first, 7, "S1: view offset 0 → 6");
    assert!(
        text.contains("Tab Into page"),
        "S2: footer still the nav footer"
    );

    for needle in [
        "Anything the agent sh",
        "Read-only transcript",
        "Commit message",
    ] {
        assert!(text.contains(needle), "N2: area keeps {needle:?}");
    }

    let (end_row, end_col) =
        wheel_over(&mut s, "7. Open a PR", Wheel::Down, 20, case_timeout(&case));
    waits::wait_state(&mut s, "21–28 of 28", "A1 end offset");
    let end = live_text(&mut s);
    wheel_at(&mut s, end_col, end_row, Wheel::Down, 3);
    std::thread::sleep(Duration::from_millis(400));
    assert_eq!(live_text(&mut s), end, "A1: Consumed at the boundary");

    support::press_step(&s, "tab");
    waits::wait_state(&mut s, "▎", "A2 area focused");
    support::press_step(&s, "up");
    waits::wait_state(&mut s, "20–27 of 28", "A2 Up moves the offset");
    let viewed = live_text(&mut s);
    assert!(
        viewed.contains("Enter Edit"),
        "A2: edit available, not active"
    );
    assert!(!viewed.contains("EDIT"), "N1: buffer cursor never engaged");
    checkpoint(&mut s, &dir, "01-view-key");
    eprintln!(
        "s1 textarea_wheel_fade: 7–14 + both fades, end Consumed, view keys stay out of edit"
    );
}

#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_inspect_open() {
    let dir = s1_dir("s1_inspect_open");
    let case = s1_case("inspect_open", &["--page", "overview"], 120, 40);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    assert!(
        !live_text(&mut s).contains("hit regions"),
        "S1: inspector closed at boot"
    );

    support::press_step(&s, "i");
    waits::wait_state(&mut s, "hit regions 26", "open state card");
    checkpoint(&mut s, &dir, "open");
    let text = live_text(&mut s);

    for needle in [
        "mode        NAV",
        "0·2 24×36",
        "hover       suppressed",
        "pressed     —",
        "last key    i",
        "focus ring  1 stops",
        "hit regions 26",
        "tick        0",
        "colors      truecolor",
        "i Inspector · on",
    ] {
        assert!(text.contains(needle), "V1: card lacks {needle:?}");
    }

    assert!(text.contains("suppressed"), "N2: hover suppressed");

    support::press_step(&s, "i");
    waits::wait_gone(&mut s, "hit regions", "A1 card closed");
    assert!(
        !live_text(&mut s).contains("· on"),
        "A1: header toggle flipped back"
    );
    checkpoint(&mut s, &dir, "01-closed");

    click_at(&mut s, "Inspector", case_timeout(&case));
    waits::wait_state(&mut s, "hit regions 26", "A2 header click reopens");

    click_at(&mut s, "hit regions", case_timeout(&case));
    let clicked = settle_text(&mut s, Duration::from_millis(3000), "A2 card click");
    assert!(
        clicked.contains("hit regions 26"),
        "A2: card still open after card click"
    );
    assert!(
        clicked.contains("Tab Into page"),
        "A2: card click takes no focus"
    );
    checkpoint(&mut s, &dir, "02-clicked");

    let case2 = s1_case("inspect_open_narrow", &["--page", "overview"], 80, 24);
    write_provenance_named(&dir, &case2, "provenance-narrow.txt");
    let mut s2 = support::spawn_boot(&case2);
    support::press_step(&s2, "i");
    waits::wait_state(&mut s2, "· on", "N1 narrow toggle flips");
    let narrow = live_text(&mut s2);
    assert!(
        !narrow.contains("hit regions"),
        "N1: no card below width 100"
    );
    assert!(!narrow.contains("focus ring"), "N1: no readout rows either");
    checkpoint(&mut s2, &dir, "03-narrow");
    eprintln!("s1 inspect_open: 10 readout rows, i toggles, header click toggles, narrow gated");
}

#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_inspect_scrolled() {
    let dir = s1_dir("s1_inspect_scrolled");
    let case = s1_case("inspect_scrolled", &["--page", "lists"], 120, 40);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::press_step(&s, "i");
    waits::wait_state(&mut s, "hit regions 58", "inspector open on lists");
    checkpoint(&mut s, &dir, "01-open");

    assert!(
        !live_text(&mut s).contains("31·10"),
        "V2: no mouse cell before the wheel"
    );

    wheel_over(&mut s, "Python", Wheel::Down, 2, case_timeout(&case));
    waits::wait_state(&mut s, "7–19 of 20", "scrolled lists window");
    checkpoint(&mut s, &dir, "scrolled");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    assert!(text.contains("31·10"), "V1: mouse row reads the wheel cell");
    assert!(text.contains("7–19 of 20"), "V1/S1: lists window 7–19");

    let (first_row, first_col) = find_pos(&mut s, "Swift", case_timeout(&case));
    let (mid_row, mid_col) = find_pos(&mut s, "Haskell", case_timeout(&case));
    assert_faded(
        span_fg_median(&frame, first_row, first_col, 5, "V2 lists top"),
        span_fg_median(&frame, mid_row, mid_col, 7, "V2 lists interior"),
        "V2 lists top edge fades",
    );

    let (first, _) = parse_window("7–19", "S1 window");
    assert_eq!(first, 7, "S1: lists offset 0 → 6");
    assert!(text.contains("suppressed"), "S1/N1: hover stays suppressed");

    assert!(text.contains("4 stops"), "S2: 4 focus stops");
    assert!(text.contains("hit regions 58"), "S2: 58 hit regions");

    let before = live_text(&mut s);
    let (card_row, card_col) = find_pos(&mut s, "hit regions", case_timeout(&case));
    wheel_at(&mut s, card_col, card_row, Wheel::Down, 2);
    std::thread::sleep(Duration::from_millis(400));
    assert_eq!(live_text(&mut s), before, "A2/N2: wheel over card Ignored");
    checkpoint(&mut s, &dir, "01-card-wheel-ignored");
    eprintln!("s1 inspect_scrolled: mouse 31·10 + 7–19, top fades, card wheel Ignored");
}

#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_panel_overview_static() {
    let dir = s1_dir("s1_panel_overview_static");
    let case = s1_case("panel_overview_static", &["--page", "overview"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);

    assert_line_has(&boot, "canvas", "text.muted", "V1 tokens two-column");
    assert_line_has(&boot, "popover", "accent.bg", "V1 tokens two-column");
    for (mark, label) in [
        ('▎', "focus"),
        ('░', "hover lifts"),
        ('›', "current / chosen"),
        ('✓', "checked"),
        ('!', "error"),
    ] {
        let mark_str = mark.to_string();
        assert_line_has(&boot, &mark_str, label, "V1 legend row");
    }

    assert!(!boot.contains("Principles"), "V2/N1: no Principles title");

    assert_eq!(
        boot.matches('▎').count(),
        2,
        "N2: nav bar + legend sample only"
    );
    assert_line_has(&boot, "▎", "Overview", "N2 focus on the nav");
    for token in ["canvas", "surface", "field", "popover", "accent", "text."] {
        assert_line_lacks(&boot, token, "▎", "N2 no focus bar on Tokens rows");
    }

    support::press_step(&s, "s");
    std::thread::sleep(Duration::from_millis(400));
    assert_eq!(live_text(&mut s), boot, "S2: page key Ignored");

    support::press_step(&s, "i");
    waits::wait_state(&mut s, "· on", "A2 inspector arm fires");
    assert!(
        !live_text(&mut s).contains("hit regions"),
        "A2: card gated below width 100"
    );
    checkpoint(&mut s, &dir, "01-inspector-arm");

    let case2 = s1_case(
        "panel_overview_static_next",
        &["--page", "overview"],
        80,
        24,
    );
    write_provenance_named(&dir, &case2, "provenance-next.txt");
    let mut s2 = support::spawn_boot(&case2);
    support::press_step(&s2, "]");
    waits::wait_state(&mut s2, "/ Components / Buttons", "A1 next page crumb");
    checkpoint(&mut s2, &dir, "02-next-page");
    eprintln!("s1 panel_overview_static: tokens + legend, s no-op, i arm, ] cycles");
}

#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_resize_grown() {
    let dir = s1_dir("s1_resize_grown");
    let case = s1_case("resize_grown", &["--page", "overview"], 80, 24).timeout(15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot-80");

    assert!(
        !live_text(&mut s).contains("Terminal too small"),
        "S1/N1: no notice at 80x24"
    );

    resize_to(&mut s, 120, 40, case_timeout(&case));

    s.inner.press("ctrl+l").expect("redraw tick");
    waits::wait_state(&mut s, "Foundations / Overview", "A1 grown header");
    checkpoint(&mut s, &dir, "grown");
    let grown_text = live_text(&mut s);
    let grown = live_frame(&mut s);

    let case2 = s1_case("resize_grown_static", &["--page", "overview"], 120, 40);
    write_provenance_named(&dir, &case2, "provenance-static.txt");
    let mut s2 = support::spawn_boot(&case2);
    checkpoint(&mut s2, &dir, "01-static-120");
    let static_text = live_text(&mut s2);
    let static_frame = live_frame(&mut s2);

    assert_eq!(grown_text, static_text, "V1: grown text equals static boot");
    assert_eq!(
        grown.digest(),
        static_frame.digest(),
        "V1: grown frame digest equals static boot"
    );

    assert!(
        grown_text.contains("120×40"),
        "V2: grown lacks the cap badge"
    );
    for needle in ["Foundations", "Components", "Screens", "Principles"] {
        assert_body_has(&grown_text, needle, "V2: grown lacks {needle}");
    }

    assert!(
        !grown_text.contains("Terminal too small"),
        "S1/N1/A2: no notice at 120x40"
    );

    assert_line_has(&grown_text, "▎", "Overview", "S2: nav cursor re-revealed");

    assert!(
        grown_text.contains("Editable tables"),
        "N2: full nav labels"
    );
    assert!(!grown_text.contains("tabl…"), "N2: no stale truncation");

    let case3 = s1_case("resize_grown_notice", &["--page", "overview"], 60, 10).timeout(15_000);
    write_provenance_named(&dir, &case3, "provenance-notice.txt");
    let mut s3 = support::spawn(&case3);
    waits::wait_state(
        &mut s3,
        "Terminal too small",
        "N1 notice renders below 72x20",
    );
    assert!(
        live_text(&mut s3).contains("Need 72×20, have 60×10"),
        "N1: size line reports the grid"
    );
    checkpoint(&mut s3, &dir, "02-notice");
    eprintln!(
        "s1 resize_grown: grown == static boot, badge + labels + Principles, notice below gate"
    );
}

#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_resize_shrunk() {
    let dir = s1_dir("s1_resize_shrunk");
    let case = s1_case("resize_shrunk", &["--page", "overview"], 120, 40).timeout(15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot-120");

    let wide = live_text(&mut s);
    for needle in [
        "120×40",
        "Foundations",
        "Components",
        "Screens",
        "Principles",
    ] {
        assert!(
            wide.contains(needle),
            "N1/N2 presence: 120x40 shows {needle}"
        );
    }

    resize_to(&mut s, 80, 24, case_timeout(&case));

    s.inner.press("ctrl+l").expect("redraw tick");
    waits::wait_state(&mut s, "Foundations / Overview", "A1 shrunk header");
    checkpoint(&mut s, &dir, "shrunk");
    let shrunk_text = live_text(&mut s);
    let shrunk = live_frame(&mut s);

    let case2 = s1_case("resize_shrunk_static", &["--page", "overview"], 80, 24);
    write_provenance_named(&dir, &case2, "provenance-static.txt");
    let mut s2 = support::spawn_boot(&case2);
    checkpoint(&mut s2, &dir, "01-static-80");
    let static_text = live_text(&mut s2);
    let static_frame = live_frame(&mut s2);

    assert_eq!(
        shrunk_text, static_text,
        "V1: shrunk text equals static boot"
    );
    assert_eq!(
        shrunk.digest(),
        static_frame.digest(),
        "V1: shrunk frame digest equals static boot"
    );

    assert!(!shrunk_text.contains("×40"), "V2/N1: cap badge dropped");
    assert!(!shrunk_text.contains("truecolor ·"), "V2/N1: no badge row");
    for needle in ["Foundations", "Components", "Screens", "Principles"] {
        assert_body_lacks(&shrunk_text, needle, "V2/N2: {needle} dropped");
    }
    assert_line_has(&shrunk_text, "canvas", "text.muted", "V2 tokens persist");
    assert!(
        shrunk_text.contains("hover lifts the surface"),
        "V2 legend persists"
    );

    assert!(
        !shrunk_text.contains("Terminal too small"),
        "S1/A2: no notice while both sizes clear 72x20"
    );
    let (tokens_row, _) = find_pos(&mut s, "Tokens", case_timeout(&case));
    let (legend_row, _) = find_pos(&mut s, "State language", case_timeout(&case));
    assert!(tokens_row < legend_row, "S1: content stacks vertically");

    assert!(shrunk_text.contains("popover"), "S2: Tokens intact");
    assert!(shrunk_text.contains("checked"), "S2: legend intact");
    eprintln!("s1 resize_shrunk: shrunk == static boot, badge + labels + Principles dropped");
}
