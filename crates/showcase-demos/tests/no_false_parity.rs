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
        let gate = exact_size_gate(source);
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
                !source.contains(marker),
                "{name}: exact-size gate {marker} spread outside its owning wave"
            );
        }
    }
}

#[test]
fn no_page_replays_a_historical_frame() {
    for (name, source) in SOURCES {
        assert!(
            !source.contains("paint_historical"),
            "{name}: a historical replay is back"
        );
        // editable.rs keeps one comment naming the historical ID-first paint
        // order. It describes a convention, not a replay; its cleanup belongs
        // to a later wave.
        if name != "editable" {
            assert!(
                !source.contains("historical"),
                "{name}: a historical replay is back"
            );
        }
    }
}

#[test]
fn no_page_mounts_widgets_in_a_zero_area_rect() {
    for (name, source) in SOURCES {
        assert!(
            !stray_zero_rect(source),
            "{name}: a widget draws into Rect::ZERO"
        );
        let mount = zero_area_mount(source);
        assert!(
            mount.is_none(),
            "{name}: a widget mounts in a zero-area rect: {mount:?}"
        );
    }
}

#[test]
fn no_page_hides_reference_widgets_off_screen() {
    for (name, source) in SOURCES {
        if VISIBLE_REFERENCE_PAGES.contains(&name) {
            continue;
        }
        assert!(
            !source.contains("ui.reference(None"),
            "{name}: a reference block mounts widgets; draw them visibly or not at all"
        );
    }
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
