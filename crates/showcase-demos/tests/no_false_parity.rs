//! The showcase pages paint their content from live widgets.
//!
//! These tests guard the properties the showcase lost when a page pasted
//! frozen cells or mounted invisible widgets: no page holds a table of
//! frozen cells, no page picks a different screen for an exact page size,
//! no page replays a historical frame, no page mounts widgets in a
//! zero-area rect, and every page renders its content at more than one size.

use std::path::{Path, PathBuf};

use showcase_demos::pages::Page;
use showcase_demos::pages::{
    buttons, chips, chrome, dialogs, editable, editor, forms, grid, inputs, lists, overview,
    panels, pickers, progress, scrolling, settings, sidebars, tables, taskrunner, terminal,
    textareas, trees,
};
use termrock::{App, Cx, Response, Theme, Ui};
use termrock_test_support::Harness;

#[path = "../../visual-harness/tests/visual_baseline/rust_source.rs"]
mod rust_source;

// This exact body is the reviewed visible, inert Buttons state-gallery site.
const BUTTON_STATES_REVIEWED_BODY: &str = include_str!("fixtures/buttons_draw_states.source.txt");
// This exact body is the reviewed, row-clipped Chips state gallery.
const CHIPS_STATES_REVIEWED_BODY: &str = include_str!("fixtures/chips_draw_states.source.txt");
const CHIPS_REFERENCE_PREFIX_SPANS: [(usize, usize); 3] =
    [(4707, 4724), (5128, 5145), (6064, 6081)];

/// Every page source, so the source guards below cover new pages too. Adding
/// a page means adding its line here; the count assertion in
/// [`every_page_source_is_guarded`] refuses to let the list drift.
const SOURCES: [(&str, &str); 25] = [
    ("author", include_str!("../src/pages/author.rs")),
    ("buttons", include_str!("../src/pages/buttons.rs")),
    ("celleditors", include_str!("../src/pages/celleditors.rs")),
    ("chips", include_str!("../src/pages/chips.rs")),
    ("chrome", include_str!("../src/pages/chrome.rs")),
    ("dialogs", include_str!("../src/pages/dialogs.rs")),
    ("editable", include_str!("../src/pages/editable.rs")),
    ("editor", include_str!("../src/pages/editor.rs")),
    ("forms", include_str!("../src/pages/forms.rs")),
    ("grid", include_str!("../src/pages/grid.rs")),
    ("inputs", include_str!("../src/pages/inputs.rs")),
    ("lists", include_str!("../src/pages/lists.rs")),
    ("overview", include_str!("../src/pages/overview.rs")),
    ("panels", include_str!("../src/pages/panels.rs")),
    ("pickers", include_str!("../src/pages/pickers.rs")),
    ("progress", include_str!("../src/pages/progress.rs")),
    ("scrolling", include_str!("../src/pages/scrolling.rs")),
    ("selectable", include_str!("../src/pages/selectable.rs")),
    ("settings", include_str!("../src/pages/settings.rs")),
    ("sidebars", include_str!("../src/pages/sidebars.rs")),
    ("tables", include_str!("../src/pages/tables.rs")),
    ("taskrunner", include_str!("../src/pages/taskrunner.rs")),
    ("terminal", include_str!("../src/pages/terminal.rs")),
    ("textareas", include_str!("../src/pages/textareas.rs")),
    ("trees", include_str!("../src/pages/trees.rs")),
];

/// The page directories the captured-cell guard reads. Every page module of
/// the showcase and of the jackin preview lives in one of them.
const PAGE_DIRECTORIES: [&str; 2] = [
    "crates/showcase-demos/src/pages",
    "crates/jackin-preview-app/src/app",
];

/// The workspace root, two levels above the crate manifest.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("the workspace root")
        .to_path_buf()
}

/// Every page source the showcase and the jackin preview own.
///
/// The walk reads the page modules that exist today and any page module
/// added later: a captured cell table cannot hide in a file the guard does
/// not read.
fn page_sources() -> Vec<(String, String)> {
    let root = workspace_root();
    let mut out = Vec::new();
    for directory in PAGE_DIRECTORIES {
        collect_sources(&root.join(directory), &mut out);
    }
    out
}

fn collect_sources(directory: &Path, out: &mut Vec<(String, String)>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).collect();
    paths.sort();
    for path in paths {
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        out.push((path.display().to_string(), source));
    }
}

/// The index of the parenthesis that closes the one at `start`.
fn closing_paren(source: &str, start: usize) -> Option<usize> {
    let mut depth = 0u32;
    for (offset, ch) in source[start..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(start + offset);
                }
            }
            _ => {}
        }
    }
    None
}

/// The comma-separated fields of a tuple body.
fn tuple_fields(body: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut depth = 0u32;
    let mut last = 0;
    for (offset, ch) in body.char_indices() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                fields.push(body[last..offset].trim().to_string());
                last = offset + 1;
            }
            _ => {}
        }
    }
    fields.push(body[last..].trim().to_string());
    fields
}

fn is_number_field(field: &str) -> bool {
    matches!(
        field.trim(),
        "u8" | "u16" | "u32" | "u64" | "usize" | "i8" | "i16" | "i32" | "i64" | "isize"
    )
}

fn is_text_field(field: &str) -> bool {
    let field = field
        .trim()
        .trim_start_matches("&")
        .trim_start_matches("mut ");
    field == "str" || field == "String" || field.ends_with("str")
}

/// A captured cell table is an array of tuples that carry coordinates and
/// text: `(u16, u16, u32, u32, bool, &str)`. The guard matches the shape,
/// not one spelling, so a table written any other way still fails.
fn captured_table(source: &str) -> Option<String> {
    let mut rest = source;
    while let Some(at) = rest.find("&[(") {
        let tuple = &rest[at + 2..];
        let Some(end) = closing_paren(tuple, 0) else {
            break;
        };
        let fields = tuple_fields(&tuple[1..end]);
        if fields.len() >= 4
            && is_number_field(&fields[0])
            && is_number_field(&fields[1])
            && fields.iter().any(|field| is_text_field(field))
        {
            return Some(tuple[..end].to_string());
        }
        rest = &tuple[end..];
    }
    None
}

/// Pages whose exact-size switches are owned by later waves, not this one.
/// Every other page must already be free of them. All four converted pages
/// now pick their layout by width breakpoints, so the allowlist is empty.
/// The breakpoints reproduce the old exact-size layouts at the covered sizes
/// (51x16, 59x20 and 134x46 page rects), never a new screen there.
const SIZE_GATE_PAGES: [&str; 0] = [];

/// Pages whose `ui.reference(None, ..)` blocks mount visible widgets. The
/// invisible mounts are gone from pickers and overview; a new reference
/// block must draw visible widgets to land here.
const VISIBLE_REFERENCE_PAGES: [&str; 3] = ["forms", "inputs", "textareas"];

/// `area.width == <nonzero>` picks a screen by its exact size. `== 0` is an
/// empty-area guard, which stays legal.
fn exact_size_gate(source: &str) -> Option<String> {
    for (index, line) in source.lines().enumerate() {
        for probe in ["area.width ==", "area.height =="] {
            let mut rest = line;
            while let Some(at) = rest.find(probe) {
                let rhs = rest[at + probe.len()..].trim_start();
                if !rhs.starts_with('0') {
                    return Some(format!("line {}: {}", index + 1, line.trim()));
                }
                rest = &rest[at + probe.len()..];
            }
        }
    }
    None
}

/// A `Rect::ZERO` that initialises a layout array (`[Rect::ZERO; N]`) is
/// overwritten before use. Any other one mounts a widget nowhere.
fn stray_zero_rect(source: &str) -> bool {
    source.replace("[Rect::ZERO;", "").contains("Rect::ZERO")
}

/// A `Rect::new(.., 0, 0)` mounts a widget in a zero-area rect. The
/// `[Rect::new(0, 0, 0, 0); N]` array initialisers are overwritten before
/// use, and a bare `, 0, 0)` also matches non-rect argument lists, so the
/// check parses each `Rect::new` call instead of substring matching.
fn zero_area_mount(source: &str) -> Option<String> {
    let mut index = 0;
    while let Some(found) = source[index..].find("Rect::new(") {
        let start = index + found;
        if source[..start].trim_end().ends_with('[') {
            index = start + "Rect::new(".len();
            continue;
        }
        let mut depth = 0u32;
        let mut end = None;
        for (offset, ch) in source[start..].char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(start + offset);
                        break;
                    }
                }
                _ => {}
            }
        }
        let end = end.expect("a Rect::new call is parenthesised");
        let call = &source[start..=end];
        let args = &call["Rect::new(".len()..call.len() - 1];
        let mut parts = Vec::new();
        let mut depth = 0u32;
        let mut last = 0;
        for (offset, ch) in args.char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => depth -= 1,
                ',' if depth == 0 => {
                    parts.push(args[last..offset].trim());
                    last = offset + 1;
                }
                _ => {}
            }
        }
        parts.push(args[last..].trim());
        if parts.len() == 4 && parts[2] == "0" && parts[3] == "0" {
            return Some(call.to_string());
        }
        index = end + 1;
    }
    None
}

#[test]
fn every_page_source_is_guarded() {
    assert_eq!(
        SOURCES.len(),
        25,
        "a page was added without extending the guard sources"
    );
    assert_eq!(
        page_factories().len(),
        22,
        "a page was added without extending the render guard"
    );
}

/// The captured-cell guard reads the shape of a table, not its spelling.
#[test]
fn the_captured_table_guard_reads_the_shape_not_the_spelling() {
    for table in [
        "const SETTLED: &[(u16, u16, u32, u32, bool, &str)] = &[",
        "const CELLS: &[(u32, u32, u32, u32, String)] = &[",
        "static FRAME: &[(usize, usize, u16, u16, &str)] = &[",
    ] {
        assert!(
            captured_table(table).is_some(),
            "the guard misses a captured table: {table}"
        );
    }
    for legal in [
        "let points = &[(u16, u16)];",
        "let rows: &[(u16, &str)] = &[];",
        "let cells = &[(u8, u8, u8)];",
    ] {
        assert!(
            captured_table(legal).is_none(),
            "the guard flags a legal table: {legal}"
        );
    }
}

/// The captured-cell guard reads every page module of the showcase and of
/// the jackin preview, so a table cannot hide in an uncovered file.
#[test]
fn every_page_module_is_covered_by_the_guard() {
    let sources = page_sources();
    assert!(
        sources.len() >= 35,
        "the guard read only {} page sources",
        sources.len()
    );
    for page in [
        "taskrunner.rs",
        "terminal.rs",
        "chrome.rs",
        "accounts_page.rs",
    ] {
        assert!(
            sources.iter().any(|(path, _)| path.ends_with(page)),
            "the guard does not read {page}"
        );
    }
}

#[test]
fn no_page_holds_a_captured_cell_table() {
    for (name, source) in page_sources() {
        assert!(
            !source.contains("&[(u16, u16, u32, u32, bool, &str)]"),
            "{name}: a captured cell table is back"
        );
        assert!(
            !source.contains("Cells from the approved snapshot"),
            "{name}: a captured cell table is back"
        );
        assert!(
            !source.contains("Truecolor cells from the approved snapshot"),
            "{name}: a captured cell table is back"
        );
        assert!(
            !source.contains(".ansi"),
            "{name}: an ansi frame bypass is back"
        );
        let table = captured_table(&source);
        assert!(
            table.is_none(),
            "{name}: a captured cell table is back: {table:?}"
        );
    }
}

#[test]
fn no_page_switches_on_an_exact_page_size() {
    for (name, source) in SOURCES {
        if SIZE_GATE_PAGES.contains(&name) {
            continue;
        }
        let code = production_source(name, source).unwrap_or_else(|error| panic!("{error}"));
        let gate = exact_size_gate(&code);
        assert!(
            gate.is_none(),
            "{name}: a page still picks a screen by its exact size: {gate:?}"
        );
        for marker in [
            "WIDE_PAGE_W",
            "WIDE_PAGE_H",
            "NARROW_PAGE",
            "SHELL_80_PAGE",
            "COMPACT_PAGE",
        ] {
            assert!(
                !code.contains(marker),
                "{name}: exact-size gate {marker} spread outside its owning wave"
            );
        }
    }
}

#[test]
fn no_page_replays_a_historical_frame() {
    for (name, source) in SOURCES {
        let matches =
            historical_identifiers(name, source).unwrap_or_else(|error| panic!("{error}"));
        assert!(
            matches.is_empty(),
            "{name}: a historical replay is back at {:?}",
            matches.iter().map(|range| range.start).collect::<Vec<_>>()
        );
    }
}

#[test]
fn no_page_mounts_widgets_in_a_zero_area_rect() {
    for (name, source) in SOURCES {
        let code = production_source(name, source).unwrap_or_else(|error| panic!("{error}"));
        assert!(
            !stray_zero_rect(&code),
            "{name}: a widget draws into Rect::ZERO"
        );
        let mount = zero_area_mount(&code);
        assert!(
            mount.is_none(),
            "{name}: a widget mounts in a zero-area rect: {mount:?}"
        );
    }
}

#[test]
fn no_page_hides_reference_widgets_off_screen() {
    for (name, source) in SOURCES {
        if name == "buttons" {
            let matches =
                hidden_reference_calls(name, source).unwrap_or_else(|error| panic!("{error}"));
            admit_reviewed_button_reference(name, source, &matches)
                .unwrap_or_else(|error| panic!("{error}"));
            continue;
        }
        if name == "chips" {
            let matches =
                hidden_reference_calls(name, source).unwrap_or_else(|error| panic!("{error}"));
            admit_reviewed_chips_references(name, source, &matches)
                .unwrap_or_else(|error| panic!("{error}"));
            continue;
        }
        if VISIBLE_REFERENCE_PAGES.contains(&name) {
            continue;
        }
        let matches =
            hidden_reference_calls(name, source).unwrap_or_else(|error| panic!("{error}"));
        assert!(
            matches.is_empty(),
            "{name}: a reference block mounts widgets; draw them visibly or not at all (matches at {:?})",
            matches.iter().map(|range| range.start).collect::<Vec<_>>()
        );
    }
}

fn production_source(name: &str, source: &str) -> Result<String, String> {
    rust_source::production_code(source).map_err(|error| format!("scan {name}: {error}"))
}

fn unique_pattern_range(
    name: &str,
    code: &str,
    pattern: &str,
) -> Result<std::ops::Range<usize>, String> {
    let mut scan = rust_source::matching_patterns_checked(code, &[pattern])
        .map_err(|error| format!("tokenize {name}: {error}"))?;
    let mut matches = scan.matches.remove(0);
    if matches.len() != 1 {
        return Err(format!(
            "{name}: expected one {pattern:?} binding, found {}",
            matches.len()
        ));
    }
    Ok(matches.remove(0))
}

fn token_index_at(
    name: &str,
    tokens: &[rust_source::Token],
    start: usize,
) -> Result<usize, String> {
    tokens
        .iter()
        .position(|token| token.start == start)
        .ok_or_else(|| format!("{name}: scanner range does not start at a token"))
}

fn checked_token_text<'a>(
    name: &str,
    code: &'a str,
    tokens: &[rust_source::Token],
    index: usize,
) -> Result<&'a str, String> {
    let token = tokens
        .get(index)
        .ok_or_else(|| format!("{name}: expected a token at index {index}"))?;
    Ok(&code[token.start..token.end])
}

fn checked_matching(
    name: &str,
    matching: &[Option<usize>],
    index: usize,
    context: &str,
) -> Result<usize, String> {
    matching
        .get(index)
        .copied()
        .flatten()
        .ok_or_else(|| format!("{name}: {context} is unmatched"))
}

fn has_enclosing_delimiter(
    code: &str,
    tokens: &[rust_source::Token],
    matching: &[Option<usize>],
    openers: std::ops::Range<usize>,
    item_at: usize,
) -> bool {
    openers.into_iter().any(|open| {
        matches!(rust_source::token_text(code, tokens, open), "(" | "[" | "{")
            && matching
                .get(open)
                .copied()
                .flatten()
                .is_some_and(|close| close > item_at)
    })
}

fn reviewed_inherent_method_item(
    name: &str,
    source: &str,
    page_impl_pattern: &str,
    owner_pattern: &str,
    method_name: &str,
    expected_body: &str,
) -> Result<std::ops::Range<usize>, String> {
    let code = production_source(name, source)?;
    let (tokens, matching) =
        rust_source::tokenize(&code).map_err(|error| format!("tokenize {name}: {error}"))?;

    // Keep the Page implementation as an independent sanity binding. The reviewed method lives
    // in the page's later inherent impl, not inside this trait impl.
    let page_impl = unique_pattern_range(name, &code, page_impl_pattern)?;
    let page_impl_at = token_index_at(name, &tokens, page_impl.start)?;
    if has_enclosing_delimiter(&code, &tokens, &matching, 0..page_impl_at, page_impl_at) {
        return Err(format!(
            "{name}: reviewed Page impl is nested inside another item"
        ));
    }
    let page_body = tokens
        .iter()
        .position(|token| token.start >= page_impl.end)
        .ok_or_else(|| format!("{name}: reviewed Page impl has no body"))?;
    if checked_token_text(name, &code, &tokens, page_body)? != "{" {
        return Err(format!("{name}: reviewed Page impl boundary changed"));
    }
    checked_matching(name, &matching, page_body, "reviewed Page impl")?;

    let method_pattern = format!("fn {method_name}");
    let method = unique_pattern_range(name, &code, &method_pattern)?;
    let method_at = token_index_at(name, &tokens, method.start)?;
    if checked_token_text(name, &code, &tokens, method_at)? != "fn"
        || checked_token_text(name, &code, &tokens, method_at + 1)? != method_name
    {
        return Err(format!("{name}: {method_name} signature changed"));
    }
    let parameters = method_at + 2;
    if checked_token_text(name, &code, &tokens, parameters)? != "(" {
        return Err(format!("{name}: {method_name} signature changed"));
    }
    let parameters_close = checked_matching(name, &matching, parameters, "method parameters")?;
    let body_open = parameters_close + 1;
    if checked_token_text(name, &code, &tokens, body_open)? != "{" {
        return Err(format!("{name}: {method_name} body boundary changed"));
    }
    let body_close = checked_matching(name, &matching, body_open, "method body")?;

    // There can be multiple legitimate inherent impls. Bind the unique directly containing
    // top-level impl instead of requiring the type to have only one impl block.
    let owner_scan = rust_source::matching_patterns_checked(&code, &[owner_pattern])
        .map_err(|error| format!("tokenize {name}: {error}"))?;
    let owner_matches = owner_scan
        .matches
        .into_iter()
        .next()
        .ok_or_else(|| format!("{name}: inherent impl scan returned no pattern result"))?;
    let mut containing_owners = 0usize;
    for owner in owner_matches {
        let owner_at = token_index_at(name, &tokens, owner.start)?;
        let owner_body = tokens
            .iter()
            .position(|token| token.start >= owner.end)
            .ok_or_else(|| format!("{name}: inherent impl has no body"))?;
        if checked_token_text(name, &code, &tokens, owner_body)? != "{" {
            continue;
        }
        let owner_close = checked_matching(name, &matching, owner_body, "inherent impl")?;
        if !(owner_body < method_at && method_at < owner_close) {
            continue;
        }
        if has_enclosing_delimiter(&code, &tokens, &matching, 0..owner_at, owner_at) {
            return Err(format!(
                "{name}: {method_name} owner is nested inside another item"
            ));
        }
        if has_enclosing_delimiter(
            &code,
            &tokens,
            &matching,
            owner_body + 1..method_at,
            method_at,
        ) {
            return Err(format!(
                "{name}: {method_name} is nested inside another item"
            ));
        }
        if body_close >= owner_close {
            return Err(format!(
                "{name}: {method_name} extends outside its inherent impl"
            ));
        }
        containing_owners += 1;
    }
    if containing_owners != 1 {
        return Err(format!(
            "{name}: expected one direct top-level inherent impl for {method_name}, found {containing_owners}"
        ));
    }

    let item = tokens[method_at].start..tokens[body_close].end;
    let expected = expected_body
        .strip_suffix('\n')
        .ok_or_else(|| format!("{name}: reviewed method fixture must end with one newline"))?;
    if &source[item.clone()] != expected {
        return Err(format!("{name}: reviewed method source binding changed"));
    }
    Ok(item)
}

fn reviewed_button_states_item(source: &str) -> Result<std::ops::Range<usize>, String> {
    reviewed_inherent_method_item(
        "buttons",
        source,
        "impl Page for ButtonsPage",
        "impl ButtonsPage",
        "draw_states",
        BUTTON_STATES_REVIEWED_BODY,
    )
}

fn reviewed_chips_states_item(source: &str) -> Result<std::ops::Range<usize>, String> {
    reviewed_inherent_method_item(
        "chips",
        source,
        "impl Page for ChipsPage",
        "impl ChipsPage",
        "draw_states",
        CHIPS_STATES_REVIEWED_BODY,
    )
}

fn admit_reviewed_button_reference(
    name: &str,
    source: &str,
    matches: &[std::ops::Range<usize>],
) -> Result<(), String> {
    let item = reviewed_button_states_item(source)?;
    if matches.len() != 1 {
        return Err(format!(
            "{name}: expected exactly one reviewed visible reference call, found {}",
            matches.len()
        ));
    }
    let body_matches = hidden_reference_calls("buttons.draw_states", &source[item.clone()])?;
    if body_matches.len() != 1 {
        return Err(format!(
            "{name}: reviewed draw_states must contain exactly one visible reference call, found {}",
            body_matches.len()
        ));
    }
    let expected = item.start + body_matches[0].start..item.start + body_matches[0].end;
    if matches[0] != expected {
        return Err(format!(
            "{name}: the reference call is outside the reviewed draw_states site"
        ));
    }
    Ok(())
}

fn admit_reviewed_chips_references(
    name: &str,
    source: &str,
    matches: &[std::ops::Range<usize>],
) -> Result<(), String> {
    let item = reviewed_chips_states_item(source)?;
    let body_matches = hidden_reference_calls("chips.draw_states", &source[item.clone()])?;
    if body_matches.len() != CHIPS_REFERENCE_PREFIX_SPANS.len() {
        return Err(format!(
            "{name}: expected three reviewed draw_states reference calls, found {}",
            body_matches.len()
        ));
    }
    let expected_body: Vec<_> = CHIPS_REFERENCE_PREFIX_SPANS
        .iter()
        .map(|(start, end)| *start..*end)
        .collect();
    if body_matches != expected_body {
        return Err(format!(
            "{name}: reviewed draw_states reference spans changed: {body_matches:?}"
        ));
    }
    let expected_page: Vec<_> = expected_body
        .iter()
        .map(|span| item.start + span.start..item.start + span.end)
        .collect();
    if matches != expected_page {
        return Err(format!(
            "{name}: expected exactly the three reviewed visible reference sites, found {matches:?}"
        ));
    }
    Ok(())
}

fn historical_identifiers(name: &str, source: &str) -> Result<Vec<std::ops::Range<usize>>, String> {
    let code = production_source(name, source)?;
    let identifiers = rust_source::matching_patterns_checked(&code, &[])
        .map_err(|error| format!("tokenize {name}: {error}"))?;
    Ok(identifiers
        .identifiers
        .into_iter()
        .filter(|range| code[range.clone()].contains("historical"))
        .collect())
}

fn hidden_reference_calls(name: &str, source: &str) -> Result<Vec<std::ops::Range<usize>>, String> {
    let code = production_source(name, source)?;
    let mut matches = rust_source::matching_patterns_checked(&code, &["ui.reference(None"])
        .map_err(|error| format!("tokenize {name}: {error}"))?;
    Ok(matches.matches.remove(0))
}

#[test]
fn exact_size_and_zero_area_guards_use_the_fail_closed_production_projection() {
    let textareas = SOURCES
        .iter()
        .find_map(|(name, source)| (*name == "textareas").then_some(*source))
        .expect("TextAreas source is included in SOURCES");
    let code = production_source("textareas", textareas).unwrap();
    assert!(exact_size_gate(&code).is_none());
    assert!(!stray_zero_rect(&code));
    assert!(zero_area_mount(&code).is_none());

    let test_only = concat!(
        "#[cfg(test)] mod outer {\n",
        "    #[cfg(test)] mod tests {\n",
        "        fn observations(area: Rect) { let _ = area.height == 1; let _ = Rect::ZERO; ",
        "let _ = Rect::new(PAGE_AREA.x, PAGE_AREA.y, 0, 0); const WIDE_PAGE_W: u16 = 80; }\n",
        "    }\n",
        "}\n",
    );
    let code = production_source("test-only", test_only).unwrap();
    assert!(exact_size_gate(&code).is_none());
    assert!(!stray_zero_rect(&code));
    assert!(zero_area_mount(&code).is_none());
    assert!(!code.contains("WIDE_PAGE_W"));

    let production_patterns = [
        (
            "fn before(area: Rect) { let _ = area.width == 80; let _ = Rect::ZERO; let _ = Rect::new(0, 0, 0, 0); }\n",
            true,
        ),
        (
            "#[cfg(test)] mod tests { fn ignored(area: Rect) { let _ = area.height == 1; let _ = Rect::new(0, 0, 0, 0); } }\nfn after(area: Rect) { let _ = area.height == 24; let _ = Rect::new(1, 2, 0, 0); }\n",
            false,
        ),
        (
            "mod tests { fn production(area: Rect) { let _ = area.width == 51; let _ = Rect::new(0, 0, 0, 0); } }\n",
            false,
        ),
        (
            "#[cfg(any(test, feature = \"live\"))] mod conditional { fn production(area: Rect) { let _ = area.height == 9; let _ = Rect::new(0, 0, 0, 0); } }\n",
            false,
        ),
        (
            "macro_rules! production_tokens { () => { fn production(area: Rect) { let _ = area.width == 99; let _ = Rect::ZERO; let _ = Rect::new(0, 0, 0, 0); } }; }\n",
            true,
        ),
        (
            "macro_rules! test_shaped_tokens { () => { #[cfg(test)] mod tests { fn observations(area: Rect) { let _ = area.height == 9; let _ = Rect::ZERO; let _ = Rect::new(0, 0, 0, 0); } } }; }\n",
            true,
        ),
    ];
    for (source, has_rect_zero) in production_patterns {
        let code = production_source("production-control", source).unwrap();
        assert!(
            exact_size_gate(&code).is_some(),
            "missed production size guard: {source}"
        );
        assert_eq!(stray_zero_rect(&code), has_rect_zero);
        assert!(
            zero_area_mount(&code).is_some(),
            "missed production zero-area call: {source}"
        );
    }

    let noncode = concat!(
        "// #[cfg(test)] mod fake { fn hidden(area: Rect) { let _ = area.width == 88; let _ = Rect::new(0, 0, 0, 0); } }\n",
        "fn strings() { let fake = r###\"#[cfg(test)] mod fake { let _ = area.width == 7; Rect::new(0, 0, 0, 0); }\"###; }\n",
        "fn after_lookalikes(area: Rect) { let _ = area.width == 88; let _ = Rect::ZERO; let _ = Rect::new(0, 0, 0, 0); }\n",
    );
    let code = production_source("noncode", noncode).unwrap();
    assert!(exact_size_gate(&code).is_some());
    assert!(stray_zero_rect(&code));
    assert!(zero_area_mount(&code).is_some());

    let malformed = production_source("malformed.rs", "#[cfg(test)] mod tests {").unwrap_err();
    assert!(malformed.contains("malformed.rs"));
}

#[test]
fn buttons_admits_only_the_exact_visible_draw_states_reference() {
    let buttons = SOURCES
        .iter()
        .find_map(|(name, source)| (*name == "buttons").then_some(*source))
        .expect("Buttons source is included in SOURCES");
    let matches = hidden_reference_calls("buttons", buttons).unwrap();
    assert_eq!(
        matches.len(),
        1,
        "the reviewed visible call must stay present"
    );
    admit_reviewed_button_reference("buttons", buttons, &matches).unwrap();

    let extra_page =
        format!("{buttons}\nfn extra_reference() {{ ui.reference(None, |_| {{}}); }}\n");
    let matches = hidden_reference_calls("buttons+extra", &extra_page).unwrap();
    assert_eq!(matches.len(), 2);
    assert!(admit_reviewed_button_reference("buttons+extra", &extra_page, &matches).is_err());

    let split_receiver = format!(
        "{buttons}\nfn split_reference() {{ ui /* split */ . reference (None, |_| {{}}); }}\n"
    );
    let matches = hidden_reference_calls("buttons+split", &split_receiver).unwrap();
    assert_eq!(
        matches.len(),
        2,
        "the scanner must see split receiver syntax"
    );
    assert!(admit_reviewed_button_reference("buttons+split", &split_receiver, &matches).is_err());

    let extra_inside = buttons.replacen(
        "for source_y in window.visible_source_range() {",
        "for source_y in window.visible_source_range() { ui.reference(None, |_| {});",
        1,
    );
    let matches = hidden_reference_calls("buttons+inside", &extra_inside).unwrap();
    assert_eq!(matches.len(), 2);
    assert!(admit_reviewed_button_reference("buttons+inside", &extra_inside, &matches).is_err());

    for (name, changed_body) in [
        (
            "zero geometry",
            BUTTON_STATES_REVIEWED_BODY.replace(
                "Rect::new(button_x, y, button_width, 1)",
                "Rect::new(view.x, view.y, 0, 0)",
            ),
        ),
        (
            "offscreen geometry",
            BUTTON_STATES_REVIEWED_BODY.replace(
                "Rect::new(button_x, y, button_width, 1)",
                "Rect::new(u16::MAX, y, button_width, 1)",
            ),
        ),
        (
            "missing empty-area guard",
            BUTTON_STATES_REVIEWED_BODY.replace("if area.is_empty() {", "if false {"),
        ),
        (
            "missing visible-source range",
            BUTTON_STATES_REVIEWED_BODY.replace("window.visible_source_range()", "0..STATE_ROWS"),
        ),
        (
            "missing nonempty button guard",
            BUTTON_STATES_REVIEWED_BODY.replace("if !button_area.is_empty() {", "if true {"),
        ),
    ] {
        assert_ne!(
            changed_body, BUTTON_STATES_REVIEWED_BODY,
            "fixture mutation {name}"
        );
        let body = BUTTON_STATES_REVIEWED_BODY.trim_end_matches('\n');
        let changed = buttons.replace(body, changed_body.trim_end_matches('\n'));
        let matches = hidden_reference_calls("buttons+changed", &changed).unwrap();
        assert_eq!(matches.len(), 1);
        assert!(
            admit_reviewed_button_reference("buttons+changed", &changed, &matches).is_err(),
            "source binding ignored {name}"
        );
    }

    let body = BUTTON_STATES_REVIEWED_BODY.trim_end_matches('\n');
    let call = "ui.reference(None, |ui| button.draw(ui, button_area));";
    let moved = buttons.replace(call, "button.draw(ui, button_area);");
    let moved = format!("{moved}\nfn moved_reference() {{ ui.reference(None, |_| {{}}); }}\n");
    let matches = hidden_reference_calls("buttons+moved", &moved).unwrap();
    assert_eq!(matches.len(), 1);
    assert!(admit_reviewed_button_reference("buttons+moved", &moved, &matches).is_err());

    let duplicate = format!("{buttons}\n{body}\n");
    let matches = hidden_reference_calls("buttons+duplicate", &duplicate).unwrap();
    assert_eq!(matches.len(), 2);
    assert!(admit_reviewed_button_reference("buttons+duplicate", &duplicate, &matches).is_err());

    let wrapped = buttons.replace(
        body,
        &format!("macro_rules! wrapped_states {{ () => {{ {body} }}; }}"),
    );
    let matches = hidden_reference_calls("buttons+macro", &wrapped).unwrap();
    assert_eq!(matches.len(), 1);
    assert!(admit_reviewed_button_reference("buttons+macro", &wrapped, &matches).is_err());

    let page_stub = "impl Page for ButtonsPage {}\n";
    let wrapped_owner = format!(
        "{page_stub}macro_rules! wrapped_inherent {{ () => {{ impl ButtonsPage {{ {body} }} }}; }}\n"
    );
    let matches = hidden_reference_calls("buttons+wrapped-owner", &wrapped_owner).unwrap();
    assert_eq!(matches.len(), 1);
    assert!(
        admit_reviewed_button_reference("buttons+wrapped-owner", &wrapped_owner, &matches).is_err()
    );

    let owner_header = buttons
        .rfind("impl ButtonsPage {")
        .expect("the reviewed draw_states owner is the later inherent impl");
    let mut changed_owner = buttons.to_owned();
    changed_owner.replace_range(
        owner_header..owner_header + "impl ButtonsPage".len(),
        "impl RenamedButtonsPage",
    );
    let matches = hidden_reference_calls("buttons+changed-owner", &changed_owner).unwrap();
    assert_eq!(matches.len(), 1);
    assert!(
        admit_reviewed_button_reference("buttons+changed-owner", &changed_owner, &matches).is_err(),
        "a changed containing type must lose the reviewed inherent-owner binding"
    );

    let mut ambiguous_owner = buttons.to_owned();
    ambiguous_owner.insert_str(
        owner_header + "impl ButtonsPage {".len(),
        "\n    impl ButtonsPage {",
    );
    let tests_module = ambiguous_owner
        .find("\n#[cfg(test)]\nmod tests")
        .expect("the source ends with its test-only module");
    ambiguous_owner.insert_str(tests_module, "\n}\n");
    let matches = hidden_reference_calls("buttons+ambiguous-owner", &ambiguous_owner).unwrap();
    assert_eq!(matches.len(), 1);
    assert!(
        admit_reviewed_button_reference("buttons+ambiguous-owner", &ambiguous_owner, &matches)
            .is_err(),
        "overlapping nested owner blocks must not make the method owner ambiguous"
    );

    let absent = buttons.replace(call, "button.draw(ui, button_area);");
    let matches = hidden_reference_calls("buttons+absent", &absent).unwrap();
    assert!(matches.is_empty());
    assert!(admit_reviewed_button_reference("buttons+absent", &absent, &matches).is_err());
}

#[test]
fn chips_admits_only_the_three_exact_visible_draw_states_references() {
    let chips = SOURCES
        .iter()
        .find_map(|(name, source)| (*name == "chips").then_some(*source))
        .expect("Chips source is included in SOURCES");
    let matches = hidden_reference_calls("chips", chips).unwrap();
    assert_eq!(matches.len(), 3);
    admit_reviewed_chips_references("chips", chips, &matches).unwrap();

    let extra_page = format!("{chips}\nfn extra_reference() {{ ui.reference(None, |_| {{}}); }}\n");
    let matches = hidden_reference_calls("chips+extra", &extra_page).unwrap();
    assert_eq!(matches.len(), 4);
    assert!(admit_reviewed_chips_references("chips+extra", &extra_page, &matches).is_err());

    let split_receiver = format!(
        "{chips}\nfn split_reference() {{ ui /* split */ . reference (None, |_| {{}}); }}\n"
    );
    let matches = hidden_reference_calls("chips+split", &split_receiver).unwrap();
    assert_eq!(matches.len(), 4);
    assert!(admit_reviewed_chips_references("chips+split", &split_receiver, &matches).is_err());

    let body = CHIPS_STATES_REVIEWED_BODY.trim_end_matches('\n');
    let changed_body = CHIPS_STATES_REVIEWED_BODY.replacen(
        "for (source_y, label) in labels.into_iter().enumerate() {",
        "for (source_y, label) in labels.into_iter().enumerate() { ui.reference(None, |_| {});",
        1,
    );
    assert_ne!(changed_body, CHIPS_STATES_REVIEWED_BODY);
    let extra_inside = chips.replacen(body, changed_body.trim_end_matches('\n'), 1);
    assert_ne!(extra_inside, chips);
    let matches = hidden_reference_calls("chips+inside", &extra_inside).unwrap();
    assert_eq!(matches.len(), 4);
    assert!(admit_reviewed_chips_references("chips+inside", &extra_inside, &matches).is_err());

    for (label, changed_body) in [
        (
            "zero sample geometry",
            CHIPS_STATES_REVIEWED_BODY.replace(
                "Rect::new(control_x, y, control_width, 1)",
                "Rect::new(view.x, view.y, 0, 0)",
            ),
        ),
        (
            "offscreen sample row",
            CHIPS_STATES_REVIEWED_BODY.replace(
                "let Some(y) = window.physical_row_for_source(source_y) else {\n                continue;\n            };",
                "let y = u16::MAX;",
            ),
        ),
        (
            "missing empty-area guard",
            CHIPS_STATES_REVIEWED_BODY.replace("if area.is_empty() {", "if false {"),
        ),
        (
            "missing visible-view guard",
            CHIPS_STATES_REVIEWED_BODY.replace("if view.is_empty() {", "if false {"),
        ),
        (
            "missing sample-area guard",
            CHIPS_STATES_REVIEWED_BODY.replace("if sample_area.is_empty() {", "if false {"),
        ),
        (
            "missing visible-source range",
            CHIPS_STATES_REVIEWED_BODY.replace("window.visible_source_range()", "0..STATE_ROWS"),
        ),
    ] {
        assert_ne!(changed_body, CHIPS_STATES_REVIEWED_BODY, "fixture mutation {label}");
        let changed = chips.replacen(body, changed_body.trim_end_matches('\n'), 1);
        assert_ne!(changed, chips, "source fixture mutation {label} was not applied");
        let matches = hidden_reference_calls("chips+changed", &changed).unwrap();
        assert_eq!(matches.len(), 3, "{label} should leave the residual call count unchanged");
        assert!(
            admit_reviewed_chips_references("chips+changed", &changed, &matches).is_err(),
            "source binding ignored {label}"
        );
    }

    let call = "ui.reference(None";
    let moved = chips.replacen(
        call,
        "ui.reference(Some(ReferenceTarget::new(CHIPS, ReferenceState::HOVERED))",
        1,
    );
    let moved = format!("{moved}\nfn moved_reference() {{ ui.reference(None, |_| {{}}); }}\n");
    let matches = hidden_reference_calls("chips+moved", &moved).unwrap();
    assert_eq!(matches.len(), 3);
    assert!(admit_reviewed_chips_references("chips+moved", &moved, &matches).is_err());

    let duplicate = format!("{chips}\n{body}\n");
    let matches = hidden_reference_calls("chips+duplicate", &duplicate).unwrap();
    assert_eq!(matches.len(), 6);
    assert!(admit_reviewed_chips_references("chips+duplicate", &duplicate, &matches).is_err());

    let wrapped = chips.replace(
        body,
        &format!("macro_rules! wrapped_states {{ () => {{ {body} }}; }}"),
    );
    let matches = hidden_reference_calls("chips+macro", &wrapped).unwrap();
    assert_eq!(matches.len(), 3);
    assert!(admit_reviewed_chips_references("chips+macro", &wrapped, &matches).is_err());

    let page_stub = "impl Page for ChipsPage {}\n";
    let wrapped_owner = format!(
        "{page_stub}macro_rules! wrapped_inherent {{ () => {{ impl ChipsPage {{ {body} }} }}; }}\n"
    );
    let matches = hidden_reference_calls("chips+wrapped-owner", &wrapped_owner).unwrap();
    assert_eq!(matches.len(), 3);
    assert!(
        admit_reviewed_chips_references("chips+wrapped-owner", &wrapped_owner, &matches).is_err()
    );

    let method = chips
        .find(body)
        .expect("reviewed draw_states method is present");
    let owner_header = chips[..method]
        .rfind("impl ChipsPage {")
        .expect("the reviewed draw_states owner is an inherent impl");
    let mut changed_owner = chips.to_owned();
    changed_owner.replace_range(
        owner_header..owner_header + "impl ChipsPage".len(),
        "impl RenamedChipsPage",
    );
    let matches = hidden_reference_calls("chips+changed-owner", &changed_owner).unwrap();
    assert_eq!(matches.len(), 3);
    assert!(
        admit_reviewed_chips_references("chips+changed-owner", &changed_owner, &matches).is_err()
    );

    let absent = chips.replacen(
        "ui.reference(None",
        "ui.reference(Some(ReferenceTarget::new(CHIPS, ReferenceState::HOVERED))",
        1,
    );
    let matches = hidden_reference_calls("chips+absent", &absent).unwrap();
    assert_eq!(matches.len(), 2);
    assert!(admit_reviewed_chips_references("chips+absent", &absent, &matches).is_err());
}

#[test]
fn production_source_guards_mask_only_nonproduction_text_and_exact_test_modules() {
    let source = concat!(
        "fn before() { paint_historical(); }\n",
        "#[cfg(test)] mod early { fn ignored() { paint_historical(); ui.reference(None, Rect::ZERO, |_| {}); } }\n",
        "fn after() { ui /* receiver */ . reference ( None, Rect::ZERO, |_| {}); }\n",
        "mod nested { #[cfg(test)] mod tests { fn ignored() { paint_historical(); } } fn live() { paint_historical(); } }\n",
        "mod tests { fn production() { paint_historical(); } }\n",
        "#[cfg(any(test, feature = \"live\"))] mod conditional { fn production() { ui.reference(None, Rect::ZERO, |_| {}); } }\n",
        "macro_rules! generated { () => { fn macro_generated() { paint_historical(); } }; }\n",
        "// #[cfg(test)] mod fake { fn erased() { paint_historical(); } }\n",
        "fn raw_lookalike() { let fake = r#\"#[cfg(test)] mod fake { paint_historical(); }\"#; ui.reference(None, Rect::ZERO, |_| {}); }\n",
        "fn after_lookalike() { ui.reference(None, Rect::ZERO, |_| {}); }\n",
    );
    assert_eq!(
        historical_identifiers("fixture.rs", source).unwrap().len(),
        4
    );
    assert_eq!(
        hidden_reference_calls("fixture.rs", source).unwrap().len(),
        4
    );

    let noncode = concat!(
        "// paint_historical(); ui.reference(None, Rect::ZERO, |_| {});\n",
        "/* outer /* nested paint_historical ui.reference(None */ done */\n",
        "fn strings() { let a = \"paint_historical ui.reference(None\"; ",
        "let b = r###\"#[cfg(test)] mod fake { paint_historical(); ui.reference(None); }\"###; ",
        "let c = b\"paint_historical ui.reference(None\"; ",
        "let _lifetime: &'static str = \"café\"; let _character = '}'; }\n",
    );
    assert!(
        historical_identifiers("noncode.rs", noncode)
            .unwrap()
            .is_empty()
    );
    assert!(
        hidden_reference_calls("noncode.rs", noncode)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn production_source_guards_fail_closed_and_reject_controlled_lists_injections() {
    for malformed in [
        "#[cfg(test)] mod tests {",
        "fn broken() { let text = \"unterminated; }",
        "fn broken() { let text = r#\"unterminated; }",
        "/* unterminated comment",
        "fn broken() { ) }",
    ] {
        assert!(
            historical_identifiers("malformed.rs", malformed)
                .unwrap_err()
                .contains("malformed.rs"),
            "malformed source was not rejected with its path: {malformed:?}"
        );
    }

    let lists = SOURCES
        .iter()
        .find_map(|(name, source)| (*name == "lists").then_some(*source))
        .expect("Lists source is included in SOURCES");
    assert!(historical_identifiers("lists", lists).unwrap().is_empty());
    assert!(hidden_reference_calls("lists", lists).unwrap().is_empty());

    let injected = format!(
        "{lists}\nfn injected_historical() {{ paint_historical(); }}\nfn injected_reference(ui: &mut Ui<'_>) {{ ui /* split */ . reference (None, Rect::ZERO, |_| {{}}); }}\n"
    );
    assert!(
        !historical_identifiers("lists+injected", &injected)
            .unwrap()
            .is_empty(),
        "a production replay identifier added to the same Lists source must be caught"
    );
    assert!(
        !hidden_reference_calls("lists+injected", &injected)
            .unwrap()
            .is_empty(),
        "a production reference call added to the same Lists source must be caught"
    );
}

struct PageHost(Box<dyn Page>);

impl App for PageHost {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.0.draw(ui, ui.full());
    }
}

type PageFactory = Box<dyn Fn() -> Box<dyn Page>>;

/// Every page with a [`Page`] implementation. selectable renders through the
/// showcase-ui app shell instead, so the shell and visual baselines cover it.
fn page_factories() -> Vec<(&'static str, PageFactory)> {
    vec![
        (
            "buttons",
            Box::new(|| Box::new(buttons::ButtonsPage::new())),
        ),
        ("chips", Box::new(|| Box::new(chips::ChipsPage::new()))),
        ("chrome", Box::new(|| Box::new(chrome::ChromePage::new()))),
        (
            "dialogs",
            Box::new(|| Box::new(dialogs::DialogsPage::new())),
        ),
        (
            "editable",
            Box::new(|| Box::new(editable::EditablePage::new())),
        ),
        ("editor", Box::new(|| Box::new(editor::EditorPage::new()))),
        ("forms", Box::new(|| Box::new(forms::FormsPage::new()))),
        ("grid", Box::new(|| Box::new(grid::GridPage::new()))),
        ("inputs", Box::new(|| Box::new(inputs::InputsPage::new()))),
        ("lists", Box::new(|| Box::new(lists::ListsPage::new()))),
        (
            "overview",
            Box::new(|| Box::new(overview::OverviewPage::new())),
        ),
        ("panels", Box::new(|| Box::new(panels::PanelsPage::new()))),
        (
            "pickers",
            Box::new(|| Box::new(pickers::PickersPage::new())),
        ),
        (
            "progress",
            Box::new(|| Box::new(progress::ProgressPage::new())),
        ),
        (
            "scrolling",
            Box::new(|| Box::new(scrolling::ScrollingPage::new())),
        ),
        (
            "settings",
            Box::new(|| Box::new(settings::SettingsPage::new())),
        ),
        (
            "sidebars",
            Box::new(|| Box::new(sidebars::SidebarsPage::new())),
        ),
        ("tables", Box::new(|| Box::new(tables::TablesPage::new()))),
        (
            "taskrunner",
            Box::new(|| Box::new(taskrunner::TaskRunnerPage::new())),
        ),
        (
            "terminal",
            Box::new(|| Box::new(terminal::TerminalPage::new())),
        ),
        (
            "textareas",
            Box::new(|| Box::new(textareas::TextAreasPage::new())),
        ),
        ("trees", Box::new(|| Box::new(trees::TreesPage::new()))),
    ]
}

/// Every page paints its own title at every size: a page that renders
/// nothing, or that picks a different screen per size, drops its title.
#[test]
fn every_page_renders_its_content_at_three_sizes() {
    for (name, make) in page_factories() {
        for (width, height) in [(120u16, 40u16), (80, 24), (51, 16)] {
            let page = make();
            let title = page.title();
            assert!(!title.is_empty(), "{name}: page has no title");
            let harness = Harness::new(PageHost(page), Theme::junie(), width, height);
            let text = harness.text();
            // Case-insensitive: some pages frame a longer title, such as the
            // settings page painting "Project settings".
            assert!(
                text.to_lowercase().contains(&title.to_lowercase()),
                "{name} {width}x{height}: title {title:?} missing"
            );
        }
    }
}

struct ChromeHost;

impl App for ChromeHost {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        showcase_demos::pages::chrome::ChromePage::new().draw(ui, ui.full());
    }
}

struct TaskRunnerHost;

impl App for TaskRunnerHost {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        showcase_demos::pages::taskrunner::TaskRunnerPage::new().draw(ui, ui.full());
    }
}

struct TerminalHost;

impl App for TerminalHost {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        showcase_demos::pages::terminal::TerminalPage::new().draw(ui, ui.full());
    }
}

/// The chrome page paints the same playground at every size.
#[test]
fn the_chrome_page_renders_its_playground_at_more_than_one_size() {
    for (width, height) in [(120u16, 40u16), (80, 24), (72, 20), (160, 50)] {
        let harness = Harness::new(ChromeHost, Theme::junie(), width, height);
        let text = harness.text();
        let mut needles = vec!["Chrome", "Sessions", "1 Claude Code (Work)"];
        if width >= 80 {
            // The card meta is cut when the card cannot show it whole.
            needles.push("right-click or Shift+F10");
        }
        if height >= 40 {
            // The playground only shows its tail when the page is tall enough.
            needles.extend([
                "payments-platform",
                "PR #482 · settlement backoff",
                "hint bar · topmost layer wins:",
                "last: nothing yet",
            ]);
        }
        for needle in needles {
            assert!(text.contains(needle), "{width}x{height}: {needle} missing");
        }
    }
}

/// The chrome page scrolls sideways when the page cannot show the playground.
#[test]
fn the_chrome_page_scrolls_when_the_page_is_narrow() {
    let harness = Harness::new(ChromeHost, Theme::junie(), 80, 24);
    let text = harness.text();
    assert!(
        text.contains('┆') || text.contains('┃'),
        "80x24: the narrow chrome page shows no scrollbar"
    );
    assert!(
        text.contains('┄') || text.contains('━'),
        "80x24: the narrow chrome page does not scroll sideways"
    );
}

/// The task runner page paints the targets tree and the pipeline.
#[test]
fn the_task_runner_page_renders_the_tree_and_the_pipeline_at_more_than_one_size() {
    for (width, height) in [(120u16, 40u16), (80, 24)] {
        let harness = Harness::new(TaskRunnerHost, Theme::junie(), width, height);
        let text = harness.text();
        for needle in [
            "Task runner",
            "Targets",
            "Pipeline",
            "payments-gateway",
            "shared-libs",
            "Run pipeline",
            "Log",
        ] {
            assert!(text.contains(needle), "{width}x{height}: {needle} missing");
        }
    }
}

/// The terminal page paints the viewport and the step rail at every size.
#[test]
fn the_terminal_page_renders_the_viewport_and_the_rail_at_more_than_one_size() {
    for (width, height) in [(120u16, 40u16), (80, 24)] {
        let harness = Harness::new(TerminalHost, Theme::junie(), width, height);
        let text = harness.text();
        for needle in ["Terminal", "Viewport", "Step rail", "of 7"] {
            assert!(text.contains(needle), "{width}x{height}: {needle} missing");
        }
    }
}

struct SelectableHost;

impl App for SelectableHost {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let model = showcase_demos::pages::selectable::Model::default();
        showcase_demos::pages::selectable::paint(ui, ui.full(), &model);
    }
}

/// The selectable page paints the selection table at every size without cell bypass.
#[test]
fn the_selectable_page_renders_the_table_at_more_than_one_size() {
    for (width, height) in [(120u16, 40u16), (80, 24)] {
        let harness = Harness::new(SelectableHost, Theme::junie(), width, height);
        let text = harness.text();
        for needle in ["Selectable table", "Dana Whitfield", "Owner"] {
            assert!(text.contains(needle), "{width}x{height}: {needle} missing");
        }
    }
}
