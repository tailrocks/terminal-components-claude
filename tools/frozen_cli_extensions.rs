//! Additional real-CLI workflows for the frozen paired-capture harness.
//!
//! This module is copied into the isolated test-harness source by
//! `frozen_baseline.py`. It does not alter frozen application code or the
//! approved snapshot tree.

use std::time::Duration;

use libtest_mimic::{Failed, Trial};

use crate::scenarios::Bin;
use crate::support::{self, Case, Color, MouseButton, Session};

const PREFIX: &str = "termrock_extensions::showcase::table_sort::";
const COLOR_MODES: [Color; 5] = [
    Color::Truecolor,
    Color::Ansi256,
    Color::Ansi16,
    Color::None,
    Color::NoColorEnv,
];
const SIZES: [(u16, u16); 5] = [(72, 20), (80, 24), (100, 30), (120, 40), (160, 50)];
const NARROW_SIZES: [(u16, u16); 2] = [(40, 12), (60, 18)];
const JOURNEYS: [&str; 6] = [
    "owner_asc_desc_clear",
    "changes_asc_desc_clear",
    "keyboard_cycle",
    "hscroll_then_sort",
    "sort_then_hover_stable_id",
    "sort_then_activate_result",
];

/// Build ignored CLI workflows across the frozen size and color matrix.
pub fn trials() -> Vec<Trial> {
    let mut trials = Vec::new();
    for journey in JOURNEYS {
        for (cols, rows) in SIZES {
            for color in COLOR_MODES {
                trials.push(trial(journey, cols, rows, color));
            }
        }
        for (cols, rows) in NARROW_SIZES {
            for color in [Color::Truecolor, Color::NoColorEnv] {
                trials.push(trial(journey, cols, rows, color));
            }
        }
    }
    trials
}

fn trial(journey: &'static str, cols: u16, rows: u16, color: Color) -> Trial {
    let name = format!("{PREFIX}{journey}::{cols}x{rows}::{}", color.suffix());
    let test_name = name.clone();
    Trial::test(name, move || {
        run(&test_name, journey, cols, rows, color).map_err(Failed::from)
    })
    .with_ignored_flag(true)
}

fn run(test_name: &str, journey: &str, cols: u16, rows: u16, color: Color) -> Result<(), String> {
    crate::frozen_cli_pair::begin_case(test_name)
        .map_err(|error| format!("start paired case `{test_name}`: {error}"))?;

    let is_narrow = cols <= 60 && rows <= 18;
    let (boot_cols, boot_rows) = if is_narrow { (72, 20) } else { (cols, rows) };
    let case_name: &'static str = Box::leak(test_name.to_owned().into_boxed_str());
    let mut case = Case::new(
        case_name,
        Bin::Showcase.path(),
        &["--page", "tables", "--motion", "paused"],
        boot_cols,
        boot_rows,
        color,
        "Tasks",
    );
    case.env = &[("TUISCOTTI_FIXED_TIME", "2026-01-01T00:00:00Z")];
    let mut session = support::spawn_boot(&case);

    if is_narrow {
        session
            .resize(cols, rows)
            .map_err(|error| format!("resize to narrow table case: {error}"))?;
    }
    capture(&mut session, test_name, "initial")?;
    if cols < 120 {
        session
            .resize(120, 40)
            .map_err(|error| format!("resize table case to usable width: {error}"))?;
        capture(
            &mut session,
            test_name,
            if is_narrow {
                "narrow_revealed_120x40"
            } else {
                "table_revealed_120x40"
            },
        )?;
    } else if !visible_table_header(&mut session, "Owner")? {
        return Err(format!(
            "Owner header unexpectedly unreachable at {cols}x{rows}"
        ));
    }

    match journey {
        "owner_asc_desc_clear" => owner_cycle(&mut session, test_name, "Owner"),
        "changes_asc_desc_clear" => owner_cycle(&mut session, test_name, "Changes"),
        "keyboard_cycle" => keyboard_cycle(&mut session, test_name),
        "hscroll_then_sort" => hscroll_then_sort(&mut session, test_name),
        "sort_then_hover_stable_id" => sort_then_hover(&mut session, test_name),
        "sort_then_activate_result" => sort_then_activate(&mut session, test_name),
        _ => Err(format!("unknown frozen table workflow: {journey}")),
    }?;

    Ok(())
}

fn capture(session: &mut Session, case_id: &str, stage: &str) -> Result<String, String> {
    let frame = session
        .wait_stable(Duration::from_millis(50))
        .map_err(|error| format!("settle `{stage}` in `{case_id}`: {error}"))?;
    let text = frame.text();
    let logical_name = format!("{case_id}/{stage}");
    support::assert_gated(&logical_name, &frame);
    Ok(text)
}

fn visible_table_header(session: &mut Session, header: &str) -> Result<bool, String> {
    let (row, _) = match session.find(header) {
        Ok(position) => position,
        Err(_) => return Ok(false),
    };
    let frame = session
        .wait_stable(Duration::from_millis(50))
        .map_err(|error| format!("settle header `{header}`: {error}"))?;
    let text = frame.text();
    let line = text.lines().nth(usize::from(row)).unwrap_or_default();
    Ok(is_table_header(line, header))
}

fn click_header(session: &mut Session, header: &str) -> Result<(), String> {
    let (row, col) = session
        .find(header)
        .map_err(|error| format!("resolve table header `{header}`: {error}"))?;
    let frame = session
        .wait_stable(Duration::from_millis(50))
        .map_err(|error| format!("settle table before `{header}` click: {error}"))?;
    let text = frame.text();
    let line = text.lines().nth(usize::from(row)).unwrap_or_default();
    if !is_table_header(line, header) {
        return Err(format!(
            "resolved `{header}` outside task-table header: {line:?}"
        ));
    }
    session
        .click_with(MouseButton::Left, col.saturating_add(1), row)
        .map_err(|error| format!("click table header `{header}`: {error}"))
}

fn owner_cycle(session: &mut Session, case_id: &str, header: &str) -> Result<(), String> {
    if header == "Changes" {
        let (row, col) = header_position(session, "Owner")?;
        for stage in ["changes_scroll_1", "changes_scroll_2"] {
            session
                .scroll_right(col, row.saturating_add(2))
                .map_err(|error| format!("scroll to Changes header: {error}"))?;
            capture(session, case_id, stage)?;
        }
        if !visible_table_header(session, "Changes")? {
            return Err("horizontal scroll did not reveal Changes header".into());
        }
    }
    click_header(session, header)?;
    let asc = capture(session, case_id, "ascending")?;
    if !asc.contains(&format!("{header} ▴")) {
        return Err(format!("ascending sort marker missing for {header}: {asc}"));
    }
    if header == "Owner" && !first_table_row(&asc, header).is_some_and(|line| line.contains("ana"))
    {
        return Err(format!(
            "owner ascending order did not put ana first: {asc}"
        ));
    }
    let (row, col) = header_position(session, header)?;
    session
        .click_with(MouseButton::Left, col, row)
        .map_err(|error| format!("click `{header}` for descending sort: {error}"))?;
    let desc = capture(session, case_id, "descending")?;
    if !desc.contains(&format!("{header} ▾")) {
        return Err(format!(
            "descending sort marker missing for {header}: {desc}"
        ));
    }
    if header == "Owner"
        && !first_table_row(&desc, header).is_some_and(|line| line.contains("sofia"))
    {
        return Err(format!(
            "owner descending order did not put sofia first: {desc}"
        ));
    }
    let (row, col) = header_position(session, header)?;
    session
        .click_with(MouseButton::Left, col, row)
        .map_err(|error| format!("click `{header}` to clear sort: {error}"))?;
    let clear = capture(session, case_id, "cleared")?;
    if !clear.contains("unsorted") {
        return Err(format!("sort did not clear for {header}: {clear}"));
    }
    Ok(())
}

fn header_position(session: &mut Session, header: &str) -> Result<(u16, u16), String> {
    let (row, col) = session
        .find(header)
        .map_err(|error| format!("re-resolve table header `{header}`: {error}"))?;
    let frame = session
        .wait_stable(Duration::from_millis(50))
        .map_err(|error| format!("settle header `{header}`: {error}"))?;
    let text = frame.text();
    let line = text.lines().nth(usize::from(row)).unwrap_or_default();
    if !is_table_header(line, header) {
        return Err(format!(
            "resolved `{header}` outside task-table header: {line:?}"
        ));
    }
    Ok((row, col.saturating_add(1)))
}

fn first_table_row<'a>(frame: &'a str, header: &str) -> Option<&'a str> {
    let lines: Vec<_> = frame.lines().collect();
    let row = lines
        .iter()
        .position(|line| is_table_header(line, header))?;
    lines.get(row + 1).copied()
}

fn is_table_header(line: &str, required: &str) -> bool {
    const COLUMNS: [&str; 7] = [
        "ID", "Task", "Owner", "Status", "Branch", "Changes", "Duration",
    ];
    line.contains(required) && COLUMNS.iter().filter(|name| line.contains(**name)).count() >= 2
}

fn keyboard_cycle(session: &mut Session, case_id: &str) -> Result<(), String> {
    session
        .send_key("tab")
        .map_err(|error| format!("focus showcase table page: {error}"))?;
    session
        .send_key("tab")
        .map_err(|error| format!("focus task table: {error}"))?;
    for (stage, suffix) in [("keyboard_ascending", "▴"), ("keyboard_descending", "▾")] {
        session
            .send_key("s")
            .map_err(|error| format!("send keyboard sort: {error}"))?;
        let frame = capture(session, case_id, stage)?;
        if !frame.contains(&format!("ID {suffix}")) {
            return Err(format!("keyboard sort state missing `{suffix}`: {frame}"));
        }
    }
    session
        .send_key("s")
        .map_err(|error| format!("send keyboard sort clear: {error}"))?;
    let frame = capture(session, case_id, "keyboard_cleared")?;
    if !frame.contains("unsorted") {
        return Err(format!("keyboard sort did not clear: {frame}"));
    }
    Ok(())
}

fn hscroll_then_sort(session: &mut Session, case_id: &str) -> Result<(), String> {
    let (row, col) = header_position(session, "Owner")?;
    for stage in ["horizontally_scrolled_1", "horizontally_scrolled_2"] {
        session
            .scroll_right(col, row.saturating_add(2))
            .map_err(|error| format!("horizontal table scroll: {error}"))?;
        capture(session, case_id, stage)?;
    }
    if !visible_table_header(session, "Changes")? {
        return Err("horizontal scroll did not reveal Changes header".into());
    }
    click_header(session, "Changes")?;
    let sorted = capture(session, case_id, "scrolled_changes_ascending")?;
    if !sorted.contains("Changes ▴") {
        return Err(format!("sort after horizontal scroll missing: {sorted}"));
    }
    Ok(())
}

fn sort_then_hover(session: &mut Session, case_id: &str) -> Result<(), String> {
    click_header(session, "Owner")?;
    let sorted = capture(session, case_id, "owner_sorted")?;
    if !sorted.contains("Owner ▴") {
        return Err(format!("owner sort missing before hover: {sorted}"));
    }
    session
        .find("#1042")
        .map_err(|error| format!("resolve stable row ID after sort: {error}"))?;
    let before = session
        .wait_stable(Duration::from_millis(50))
        .map_err(|error| format!("settle stable row before hover: {error}"))?;
    let (col, row) = stable_row_position(&before, "#1042")
        .ok_or_else(|| "stable row ID is not present in the captured frame".to_owned())?;
    let before_row = row_cells(&before, row);
    session
        .mouse_move(col.saturating_add(2), row)
        .map_err(|error| format!("hover stable row ID: {error}"))?;
    let hovered_frame = session
        .wait_stable(Duration::from_millis(50))
        .map_err(|error| format!("settle stable row hover: {error}"))?;
    let hovered = hovered_frame.text();
    support::assert_gated(&format!("{case_id}/stable_id_hovered"), &hovered_frame);
    if !hovered.contains("Owner ▴") || !hovered.lines().any(|line| line.contains("#1042")) {
        return Err(format!("stable row not retained under hover: {hovered}"));
    }
    if before_row == row_cells(&hovered_frame, row) {
        return Err("hover did not change stable row cells".to_owned());
    }
    Ok(())
}

fn row_cells(frame: &tuiscotti::Frame, row: u16) -> Vec<tuiscotti::Cell> {
    frame
        .cells
        .iter()
        .filter(|cell| cell.y == row)
        .cloned()
        .collect()
}

fn stable_row_position(frame: &tuiscotti::Frame, needle: &str) -> Option<(u16, u16)> {
    let symbols: Vec<_> = needle.chars().map(|ch| ch.to_string()).collect();
    for row in 0..frame.rows {
        let cells: Vec<_> = frame.cells.iter().filter(|cell| cell.y == row).collect();
        if let Some(start) = cells.windows(symbols.len()).position(|window| {
            window
                .iter()
                .zip(&symbols)
                .all(|(cell, expected)| cell.symbol == *expected)
        }) {
            return Some((cells[start].x, row));
        }
    }
    None
}

fn sort_then_activate(session: &mut Session, case_id: &str) -> Result<(), String> {
    click_header(session, "Owner")?;
    let sorted = capture(session, case_id, "owner_sorted")?;
    if !sorted.contains("Owner ▴") {
        return Err(format!("owner sort missing before activation: {sorted}"));
    }
    session
        .find("#1042")
        .map_err(|error| format!("resolve stable row ID before activation: {error}"))?;
    let frame = session
        .wait_stable(Duration::from_millis(50))
        .map_err(|error| format!("settle stable row before activation: {error}"))?;
    let (col, row) = stable_row_position(&frame, "#1042")
        .ok_or_else(|| "stable row ID is not present in the captured frame".to_owned())?;
    session
        .click_with(MouseButton::Left, col.saturating_add(1), row)
        .map_err(|error| format!("focus stable row before activation: {error}"))?;
    session
        .send_key("enter")
        .map_err(|error| format!("activate sorted result: {error}"))?;
    let activated = capture(session, case_id, "result_activated")?;
    if !activated.contains("Selected Fix flaky checkout integration test") {
        return Err(format!("activation result missing: {activated}"));
    }
    Ok(())
}
