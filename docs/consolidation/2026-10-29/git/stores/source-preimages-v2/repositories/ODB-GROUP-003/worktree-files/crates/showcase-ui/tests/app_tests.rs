use showcase_ui::{App, NAV_ENTRIES, PageId};
use termrock::{
    Axis, Color, ColorLevel, Id, ItemKey, KeyCode, LayerId, Modifier, MouseKind, Part, PartRef,
    Rect, StateFlags, Theme,
};
use termrock_test_support::Harness;

const FORM_SUMMARY: Id = Id::root("showcase_ui::pages::forms::forms.summary");
const SCROLL_LIST: Id = Id::root("showcase_ui::pages::scrolling::scrolling.list");
const APP_NAV: Id = Id::root("showcase_ui::app::navigation");
const CHROME_BRAND: Id = Id::root("showcase_ui::pages::chrome::chrome.brand");
const SIDEBAR_CONTENT: Id = Id::root("showcase_ui::pages::sidebars::sidebars.content");
const EDITOR: Id = Id::root("showcase_ui::pages::editor::editor.code");
const EDITOR_FIND: Id = Id::root("showcase_ui::pages::editor::editor.find");
const EDITOR_RUN: Id = Id::root("showcase_ui::pages::editor::editor.run");
const EDITOR_STATE_LIST: Id = Id::root("showcase_ui::pages::editor::editor.state.list");
const HEADER_INSPECT: Id = Id::root("showcase_ui::app::header.inspect");
const HELP: Id = Id::root("showcase_ui::app::help");
const INSPECTOR: Id = Id::root("showcase_ui::app::inspector");
const INSPECTOR_CLOSE: Id = INSPECTOR.part(Part::ACTIONS).index(0);
const DIALOG_OPEN: Id = Id::root("showcase_ui::pages::dialog_kinds::dialog_kinds.open_form");
const DIALOG_FORM: Id = Id::root("showcase_ui::pages::dialog_kinds::dialog_kinds.connection");
const DIALOG_NAME: Id = DIALOG_FORM.part(Part::BODY).part(Part::FIELD).index(0);
const DIALOG_KEY: Id = DIALOG_FORM.part(Part::BODY).part(Part::FIELD).index(6);
const DIALOG_SAVE: Id = DIALOG_FORM.part(Part::BODY).part(Part::ACTIONS).index(2);
const DIALOG_PICKER: Id = Id::root("showcase_ui::pages::dialog_kinds::dialog_kinds.ssh_keys");
const PICKERS_LAYER: Id = Id::root("showcase_ui::pages::pickers::pickers.layer");
const PICKERS_OPEN_QUICK: Id = Id::root("showcase_ui::pages::pickers::pickers.open.quick");
const PICKERS_OPEN_TABS: Id = Id::root("showcase_ui::pages::pickers::pickers.open.tabs");
const PICKERS_OPEN_LEVEL: Id = Id::root("showcase_ui::pages::pickers::pickers.open.level");
const PICKERS_OPEN_COMMANDS: Id = Id::root("showcase_ui::pages::pickers::pickers.open.commands");
const PICKERS_OPEN_REMOTE: Id = Id::root("showcase_ui::pages::pickers::pickers.open.remote");
fn harness(page: PageId) -> Harness<App> {
    Harness::new(App::with_page(page), Theme::junie(), 120, 40)
}

fn press(h: &mut Harness<App>, code: KeyCode) {
    let _ = h.key(code);
}

fn control(h: &mut Harness<App>, c: char) {
    let _ = h.ctrl(c);
}

fn type_text(h: &mut Harness<App>, text: &str) {
    let _ = h.type_str(text);
}

fn move_mouse(h: &mut Harness<App>, x: u16, y: u16) {
    let _ = h.mouse(MouseKind::Move, x, y);
}

fn click(h: &mut Harness<App>, x: u16, y: u16) {
    let _ = h.click(x, y);
}

// A depth-one catalog label starts at column 5; a same-named group starts at
// column 3. This selects the actual “Tables” leaf, not its disclosure row.
fn visible_nav_leaf_row(h: &Harness<App>, label: &str) -> Option<(u16, u16)> {
    let area = h.area_of(APP_NAV)?;
    (area.y..area.bottom()).find_map(|y| {
        let row = h.row(y);
        let sidebar = row
            .chars()
            .take(usize::from(area.width))
            .collect::<String>();
        sidebar.find(label).and_then(|byte| {
            let x = sidebar[..byte].chars().count();
            (x >= 5).then(|| (u16::try_from(x).expect("leaf label position fits u16"), y))
        })
    })
}

fn wheel(h: &mut Harness<App>, axis: Axis, delta: i16, x: u16, y: u16) {
    let _ = h.wheel(axis, delta, x, y);
}

#[expect(
    clippy::panic,
    reason = "a missing fixture is a real test failure, never a skipped scenario"
)]
fn require<T>(value: Option<T>, message: &str) -> T {
    match value {
        Some(value) => value,
        None => panic!("{message}"),
    }
}

fn drag(h: &mut Harness<App>, from: (u16, u16), to: (u16, u16)) {
    let _ = h.drag(from, to);
}

fn resize(h: &mut Harness<App>, width: u16, height: u16) {
    let _ = h.resize(width, height);
}

fn area_text(h: &Harness<App>, area: Rect) -> String {
    (area.y..area.bottom())
        .map(|y| {
            (area.x..area.right())
                .map(|x| h.cell(x, y).symbol().to_owned())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn focus_bar_x(h: &Harness<App>, y: u16) -> Option<u16> {
    (20..h.buffer().area().width).find(|x| h.cell(*x, y).symbol() == "▎")
}

fn cell_style(h: &Harness<App>, needle: &str) -> (Color, Color, Modifier) {
    let (x, y) = require(h.find(needle), needle);
    let cell = h.cell(x, y);
    (cell.fg, cell.bg, cell.modifier)
}

fn exercise_focus_ring(h: &mut Harness<App>, page: PageId) {
    let reachable: Vec<Id> = h.ring().reachable().map(|entry| entry.id).collect();
    assert!(!reachable.is_empty(), "{page:?} has no reachable controls");
    let initial = require(h.focus(), "initial focus");
    assert!(
        reachable.contains(&initial),
        "{page:?} initial focus is outside the reachable ring"
    );

    let mut seen = Vec::with_capacity(reachable.len());
    loop {
        let focused = require(h.focus(), "focus disappeared during traversal");
        assert!(
            reachable.contains(&focused),
            "{page:?} focus escaped its initial ring"
        );
        assert!(
            h.state_of(focused).contains(StateFlags::FOCUSED),
            "{page:?} focused control lacks FOCUSED state"
        );
        if seen.contains(&focused) {
            break;
        }
        seen.push(focused);
        press(h, KeyCode::Tab);
    }
    assert_eq!(h.focus(), Some(initial), "{page:?} focus ring did not wrap");
    assert_eq!(
        seen.len(),
        reachable.len(),
        "{page:?} ring has an unreachable stop"
    );
}

#[expect(
    clippy::too_many_lines,
    reason = "the exhaustive page-state matrix keeps each page's flow next to its identity"
)]
fn exercise_page_state(h: &mut Harness<App>, page: PageId) {
    match page {
        PageId::Overview => {
            let (x, y) = require(h.find("Playground"), "overview playground tab");
            click(h, x, y);
            assert!(h.text().contains("Tokens · planes and text"));
        }
        PageId::Buttons => {
            let (x, y) = require(h.find("Run task"), "run button");
            click(h, x, y);
            assert!(h.text().contains("Run task ✓"));
            press(h, KeyCode::Tab);
            press(h, KeyCode::Enter);
            assert!(h.text().contains("Preview ✓"));
        }
        PageId::Inputs => {
            press(h, KeyCode::Tab);
            press(h, KeyCode::End);
            type_text(h, "-visited");
            press(h, KeyCode::Enter);
            assert!(h.text().contains("payments-gateway-visited"));
            press(h, KeyCode::Enter);
            type_text(h, "discarded");
            press(h, KeyCode::Esc);
            assert!(!h.text().contains("discarded"));
        }
        PageId::TextAreas => {
            let (x, y) = require(h.find("1. Read"), "checklist");
            let before = h.snapshot().digest();
            wheel(h, Axis::V, 3, x, y);
            assert_ne!(
                before,
                h.snapshot().digest(),
                "textarea wheel did not scroll"
            );
            press(h, KeyCode::Tab);
            press(h, KeyCode::Enter);
            for _ in 0..30 {
                press(h, KeyCode::Down);
            }
            assert!(h.text().contains("28. Run"));
            press(h, KeyCode::Esc);
        }
        PageId::Forms => {
            control(h, 's');
            assert!(h.text().contains("Required: summary"));
            assert_eq!(h.focus(), Some(FORM_SUMMARY));
            let summary = require(h.area_of(FORM_SUMMARY), "summary field");
            click(h, summary.x.saturating_add(1), summary.y);
            type_text(h, "Fix navigation");
            press(h, KeyCode::Enter);
            control(h, 's');
            assert!(h.text().contains("Creating task"));
        }
        PageId::Lists => {
            press(h, KeyCode::Tab);
            for _ in 0..19 {
                press(h, KeyCode::Down);
            }
            assert!(h.text().contains("Erlang"));
            press(h, KeyCode::Enter);
            assert!(h.text().contains("Chosen: Erlang"));
            press(h, KeyCode::Tab);
            press(h, KeyCode::Char(' '));
            press(h, KeyCode::Char('a'));
            assert!(h.text().contains("src/api/auth"));
        }
        PageId::Trees => {
            press(h, KeyCode::Tab);
            let src_y = require(h.find_row("src"), "tree root");
            let bar_x = require(focus_bar_x(h, src_y), "tree focus bar");
            assert!(h.text().contains("config.rs"));
            press(h, KeyCode::Left);
            assert!(!h.text().contains("config.rs"));
            press(h, KeyCode::Right);
            press(h, KeyCode::Down);
            press(h, KeyCode::Right);
            assert!(h.text().contains("auth.rs"));
            press(h, KeyCode::Down);
            let auth_y = require(h.find_row("auth.rs"), "expanded api file");
            assert_eq!(focus_bar_x(h, auth_y), Some(bar_x));
        }
        PageId::Tables => {
            let (x, y) = require(h.find("Owner"), "owner header");
            click(h, x, y);
            assert!(first_data_row(h).contains("ana"));
        }
        PageId::Editable => {
            press(h, KeyCode::Tab);
            press(h, KeyCode::Enter);
            assert!(h.text().contains("EDIT"));
            press(h, KeyCode::End);
            type_text(h, " now");
            press(h, KeyCode::Enter);
            assert!(h.text().contains("Add rate limiting to auth endpoi"));
        }
        PageId::Panels => {
            assert!(h.text().contains("Titled card"));
            assert!(h.text().contains("Nested"));
        }
        PageId::Sidebars => {
            press(h, KeyCode::Tab);
            press(h, KeyCode::Down);
            press(h, KeyCode::Enter);
            let content = require(h.area_of(SIDEBAR_CONTENT), "sidebar content");
            assert!(area_text(h, content).contains("Runs"));
        }
        PageId::Dialogs => {
            press(h, KeyCode::Tab);
            press(h, KeyCode::Enter);
            assert!(h.text().contains("Run task now?"));
            press(h, KeyCode::Esc);
            assert!(h.text().contains("Cancelled"));
            assert_eq!(h.top_layer(), LayerId::PAGE);
        }
        PageId::DialogKinds => {
            let (x, y) = require(h.find("Add connection…"), "Add connection opener");
            click(h, x.saturating_add(1), y);
            assert!(h.text().contains("Add connection"));
            assert_ne!(h.top_layer(), LayerId::PAGE, "opener opens the form");
            let (x, y) = require(h.find("Cancel"), "dialog cancel action");
            click(h, x.saturating_add(1), y);
            assert_eq!(h.top_layer(), LayerId::PAGE);
            assert!(!h.is_open(DIALOG_FORM), "Cancel closes the form");
            assert!(h.text().contains("Form · Cancel"), "{}", h.text());
            assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
        }
        PageId::Progress => {
            let before = h.snapshot().digest();
            let _ = h.advance(std::time::Duration::from_millis(80));
            assert_ne!(
                before,
                h.snapshot().digest(),
                "elapsed progress deadline did not repaint"
            );
            let text = h.text();
            let building = require(
                text.lines().find(|line| line.contains("Building")),
                "live progress bar",
            );
            assert!(building.contains(" 1%"));
        }
        PageId::Scrolling => {
            let (x, y) = require(h.find("Row 001"), "scroll list row");
            let before = h.snapshot().digest();
            wheel(h, Axis::V, 8, x, y);
            assert_ne!(before, h.snapshot().digest(), "scroll list did not move");
            assert!(h.text().contains("list="));
        }
        PageId::Terminal => {
            let (x, y) = require(h.find("Resolving workspace members"), "terminal output");
            let before = h.snapshot().digest();
            wheel(h, Axis::V, 4, x, y);
            assert_ne!(
                before,
                h.snapshot().digest(),
                "terminal output did not scroll"
            );
            assert!(h.text().contains("status: ready"));
        }
        PageId::Editor => {
            let before = h.snapshot().digest();
            press(h, KeyCode::Tab);
            press(h, KeyCode::Char('i'));
            type_text(h, "x");
            assert_ne!(before, h.snapshot().digest(), "editor edit did not repaint");
            assert!(h.text().contains("retry.rs"));
            press(h, KeyCode::Esc);
        }
        PageId::Grid => {
            press(h, KeyCode::Tab);
            press(h, KeyCode::Down);
            press(h, KeyCode::Enter);
            assert!(!h.text().contains("selected metric: none"));
        }
        PageId::Chips => {
            press(h, KeyCode::Tab);
            press(h, KeyCode::Char(' '));
            assert!(h.text().contains("filter toggled"));
        }
        PageId::Pickers => {
            let (x, y) = require(h.find("Open a picker"), "picker launcher");
            click(h, x, y);
            assert!(h.text().contains("Open quickly"));
            press(h, KeyCode::Down);
            press(h, KeyCode::Enter);
            assert!(h.text().contains("Chosen          README.md"));
        }
        PageId::Chrome => {
            let brand = require(h.area_of(CHROME_BRAND), "chrome brand");
            let x = brand.x.saturating_add(1);
            let y = brand.y;
            click(h, x, y);
            assert!(h.text().contains("brand activations: 1"));
        }
        PageId::Settings => {
            press(h, KeyCode::Tab);
            press(h, KeyCode::Right);
            assert!(h.text().contains("members"));
            press(h, KeyCode::Tab);
            press(h, KeyCode::Down);
            press(h, KeyCode::Tab);
            press(h, KeyCode::Tab);
            press(h, KeyCode::Enter);
            assert!(h.text().contains("Remove member?"));
            press(h, KeyCode::Esc);
            assert_eq!(h.top_layer(), LayerId::PAGE);
        }
        PageId::CellEditors => {
            press(h, KeyCode::Tab);
            press(h, KeyCode::Tab);
            press(h, KeyCode::Enter);
            assert!(
                h.text().contains("Disaster recovery"),
                "env select did not open: {}",
                h.text()
            );
        }
        PageId::Toolbars => {
            assert!(h.text().contains("Query toolbar"));
            assert!(h.text().contains("Narrow toolbar"));
            assert!(h.text().contains("Formatting toolbar"));
            assert!(h.text().contains("Last"));
        }
        PageId::TaskRunner => {
            press(h, KeyCode::Char('r'));
            assert!(h.text().contains("Pipeline · running"));
            let before = h.snapshot().digest();
            h.ticks(4);
            assert_ne!(
                before,
                h.snapshot().digest(),
                "task runner tick did not advance"
            );
            assert!(h.text().contains("compile started"));
        }
    }
}

#[test]
fn editor_playground_keeps_live_find_completion_and_run_contracts() {
    let mut h = harness(PageId::Editor);
    assert!(
        h.area_of(EDITOR_STATE_LIST).is_some(),
        "state list lacks layout: {}",
        h.text()
    );
    let editor_area = require(h.area_of(EDITOR), "editor viewport");
    wheel(
        &mut h,
        Axis::V,
        16,
        editor_area.x.saturating_add(1),
        editor_area.y.saturating_add(1),
    );
    assert!(h.text().contains("gzip(true)"), "{}", h.text());
    assert!(
        h.text().contains("Find"),
        "Find action absent: {}",
        h.text()
    );
    assert!(
        h.text().contains("Run block"),
        "Run action absent: {}",
        h.text()
    );

    assert!(h.tab_to(EDITOR));
    press(&mut h, KeyCode::F(6));
    assert_eq!(h.focus(), Some(EDITOR_FIND));
    press(&mut h, KeyCode::Enter);
    assert_eq!(h.focus(), Some(EDITOR));
    type_text(&mut h, "delay");
    assert!(h.text().contains("“delay” · 1 of 3"), "{}", h.text());
    press(&mut h, KeyCode::Esc);
    assert!(h.text().contains("Find         closed"), "{}", h.text());

    assert!(h.tab_to(EDITOR));
    press(&mut h, KeyCode::Char('i'));
    press(&mut h, KeyCode::Backspace);
    type_text(&mut h, "fe");
    assert!(h.text().contains("Completion   open"), "{}", h.text());
    assert!(
        h.text().contains("async fn (url: &str)"),
        "completion popup missing: {}",
        h.text()
    );
    press(&mut h, KeyCode::Esc);
    assert!(h.text().contains("Completion   closed"), "{}", h.text());

    let mut h = harness(PageId::Editor);
    assert!(h.tab_to(EDITOR));
    for _ in 0..17 {
        press(&mut h, KeyCode::Down);
    }
    assert!(h.text().contains("Block        2 of 3"), "{}", h.text());
    control(&mut h, 'r');
    assert!(
        h.text().contains("running"),
        "shortcut run state missing: {}",
        h.text()
    );
    for _ in 0..12 {
        let _ = h.tick();
    }
    assert!(
        h.text().contains("Runs         1"),
        "run count missing: {}",
        h.text()
    );
    assert!(
        h.text().contains("unwrap() panics on Err"),
        "live run diagnostics missing: {}",
        h.text()
    );
    assert!(
        h.text().contains("Block ran in"),
        "run status missing: {}",
        h.text()
    );

    let mut h = harness(PageId::Editor);
    assert!(h.tab_to(EDITOR));
    press(&mut h, KeyCode::F(6));
    assert_eq!(h.focus(), Some(EDITOR_FIND));
    press(&mut h, KeyCode::Tab);
    assert_eq!(h.focus(), Some(EDITOR_RUN));
    press(&mut h, KeyCode::Enter);
    assert!(
        h.text().contains("running"),
        "run state missing: {}",
        h.text()
    );
    for _ in 0..12 {
        let _ = h.tick();
    }
    assert!(
        h.text().contains("Runs         1"),
        "run count missing: {}",
        h.text()
    );
    assert!(h.text().contains("Block ran in"), "{}", h.text());
}

#[test]
fn editor_shell_tab_indents_without_leaving_editor_focus() {
    let mut h = harness(PageId::Editor);
    assert!(h.tab_to(EDITOR));
    press(&mut h, KeyCode::Char('i'));
    press(&mut h, KeyCode::Tab);

    assert_eq!(
        h.focus(),
        Some(EDITOR),
        "Tab indentation retains editor focus"
    );
    assert!(
        h.text().contains("Cursor       ln 1 · col 6"),
        "Tab must add the configured four spaces: {}",
        h.text()
    );
}

#[test]
fn editor_shell_completion_accepts_enter_then_tab_indents() {
    let mut h = harness(PageId::Editor);
    assert!(h.tab_to(EDITOR));
    press(&mut h, KeyCode::Char('i'));
    press(&mut h, KeyCode::Backspace);
    type_text(&mut h, "fe");
    assert!(
        h.text().contains("async fn (url: &str)"),
        "the live completion row must render before acceptance: {}",
        h.text()
    );

    press(&mut h, KeyCode::Enter);
    assert!(
        h.text().contains("fetch()"),
        "Enter must accept fetch(): {}",
        h.text()
    );
    assert!(h.text().contains("Completion   closed"), "{}", h.text());
    assert_eq!(h.focus(), Some(EDITOR));

    press(&mut h, KeyCode::Tab);
    assert_eq!(
        h.focus(),
        Some(EDITOR),
        "Tab after acceptance indents in place"
    );
    assert!(
        h.text().contains("Cursor       ln 1 · col 12"),
        "the following Tab must add four spaces after fetch(): {}",
        h.text()
    );
}

#[test]
fn editor_shell_completion_accepts_tab_then_next_tab_indents() {
    let mut h = harness(PageId::Editor);
    assert!(h.tab_to(EDITOR));
    press(&mut h, KeyCode::Char('i'));
    press(&mut h, KeyCode::Backspace);
    type_text(&mut h, "fe");
    assert!(h.text().contains("async fn (url: &str)"), "{}", h.text());

    press(&mut h, KeyCode::Tab);
    assert!(
        h.text().contains("fetch()"),
        "Tab must accept fetch(): {}",
        h.text()
    );
    assert!(h.text().contains("Completion   closed"), "{}", h.text());
    assert_eq!(h.focus(), Some(EDITOR));

    press(&mut h, KeyCode::Tab);
    assert_eq!(
        h.focus(),
        Some(EDITOR),
        "Tab after acceptance indents in place"
    );
    assert!(
        h.text().contains("Cursor       ln 1 · col 12"),
        "the following Tab must add four spaces after fetch(): {}",
        h.text()
    );
}

#[test]
fn editor_shell_navigation_tab_moves_to_find_after_commit() {
    let mut h = harness(PageId::Editor);
    assert!(h.tab_to(EDITOR));
    press(&mut h, KeyCode::Esc);
    assert!(h.text().contains("Mode         navigating"), "{}", h.text());

    press(&mut h, KeyCode::Tab);
    assert_eq!(h.focus(), Some(EDITOR_FIND));
}

#[test]
fn editor_shell_help_layer_blocks_the_page_run_binding() {
    let mut h = harness(PageId::Editor);
    assert!(h.tab_to(EDITOR));
    press(&mut h, KeyCode::Esc);
    press(&mut h, KeyCode::Char('?'));
    assert!(h.top_layer().index() > LayerId::PAGE.index());

    control(&mut h, 'r');
    assert!(
        !h.text().contains("Running block…"),
        "the page Ctrl+R binding must not pass through help: {}",
        h.text()
    );
    press(&mut h, KeyCode::Esc);
    assert_eq!(h.top_layer(), LayerId::PAGE);
}

#[test]
fn editor_run_clock_does_not_advance_for_pointer_events() {
    let mut h = harness(PageId::Editor);
    assert!(h.tab_to(EDITOR));
    control(&mut h, 'r');
    assert!(h.text().contains("running"), "{}", h.text());

    for _ in 0..20 {
        move_mouse(&mut h, 0, 0);
    }

    assert!(
        h.text().contains("running"),
        "pointer input must not advance the run clock: {}",
        h.text()
    );
    assert!(h.text().contains("Runs         0"), "{}", h.text());
}

#[test]
fn editor_run_diagnoses_the_block_submitted_before_cursor_movement() {
    let mut h = harness(PageId::Editor);
    assert!(h.tab_to(EDITOR));
    for _ in 0..17 {
        press(&mut h, KeyCode::Down);
    }
    assert!(h.text().contains("Block        2 of 3"), "{}", h.text());
    control(&mut h, 'r');
    assert!(h.text().contains("running"), "{}", h.text());
    for _ in 0..17 {
        press(&mut h, KeyCode::Up);
    }
    assert!(h.text().contains("Block        1 of 3"), "{}", h.text());

    for _ in 0..12 {
        let _ = h.tick();
    }
    assert!(
        h.text().contains("unwrap() panics on Err"),
        "diagnostics must belong to the submitted second block: {}",
        h.text()
    );
}

#[test]
fn editor_manual_empty_completion_keeps_frozen_alphabetic_choice() {
    let mut h = harness(PageId::Editor);
    assert!(h.tab_to(EDITOR));
    control(&mut h, ' ');
    assert!(h.text().contains("Completion   open"), "{}", h.text());
    press(&mut h, KeyCode::Enter);

    assert!(
        h.text().contains("Body// Retry"),
        "empty manual completion must choose the alphabetically first label: {}",
        h.text()
    );
}

#[test]
fn editor_repeated_runs_accumulate_diagnostics_until_text_changes() {
    let mut h = harness(PageId::Editor);
    assert!(h.tab_to(EDITOR));
    for _ in 0..17 {
        press(&mut h, KeyCode::Down);
    }
    assert!(h.text().contains("Block        2 of 3"), "{}", h.text());

    control(&mut h, 'r');
    h.ticks(10);
    assert!(h.text().contains("Diagnostics  1"), "{}", h.text());

    control(&mut h, 'r');
    h.ticks(10);
    assert!(h.text().contains("Runs         2"), "{}", h.text());
    assert!(
        h.text().contains("Diagnostics  2"),
        "repeated unchanged runs retain the frozen accumulated diagnostics: {}",
        h.text()
    );

    press(&mut h, KeyCode::Char('i'));
    press(&mut h, KeyCode::Char('x'));
    assert!(h.text().contains("Diagnostics  0"), "{}", h.text());
}

#[test]
fn editor_blank_separator_readout_has_no_fallback_execution_block() {
    let mut h = harness(PageId::Editor);
    assert!(h.tab_to(EDITOR));
    for _ in 0..16 {
        press(&mut h, KeyCode::Down);
    }

    assert!(
        h.text().contains("Cursor       ln 17 · col 1"),
        "{}",
        h.text()
    );
    assert!(
        h.text().contains("between blocks"),
        "readout uses exact cursor containment, while execution may fall back to the previous block: {}",
        h.text()
    );
}

#[test]
fn launches_and_renders_shell() {
    let h = harness(PageId::Overview);
    let text = h.text();
    assert!(text.contains("Junie Design system"));
    assert!(text.contains("Overview"));
    assert!(text.contains("Junie"));
    assert!(text.contains("Playground"));
    assert!(text.contains("Tokens and principles behind every component"));
    assert_eq!(NAV_ENTRIES.len(), 54);
    assert_eq!(PageId::ALL.len(), 26);
    assert_eq!(PageId::from_name("Dialog kinds"), Some(PageId::DialogKinds));
}

#[test]
fn dialog_kinds_add_connection_uses_live_form_and_stacked_picker() {
    let mut h = harness(PageId::DialogKinds);
    let open = require(h.find("Add connection…"), "Add connection opener");
    click(&mut h, open.0.saturating_add(1), open.1);
    assert!(h.text().contains("Add connection"));
    assert!(h.text().contains("orders-prod"));
    assert!(h.text().contains("db.eu-west-1.internal"));
    assert!(!h.text().contains("demo-only"));
    assert!(
        h.text()
            .contains("postgres://db.eu-west-1.internal/orders-prod")
    );

    let tunnel = require(h.find("Connect through an SSH tunnel"), "tunnel checkbox");
    click(&mut h, tunnel.0, tunnel.1);
    assert!(
        h.text().contains("~/.ssh/id_ed25519"),
        "the form begins with its first stable key choice\n{}",
        h.text()
    );
    assert!(
        h.tab_to(DIALOG_NAME),
        "name field remains in the form focus order"
    );
    press(&mut h, KeyCode::Home);
    for _ in 0..11 {
        press(&mut h, KeyCode::Delete);
    }
    type_text(&mut h, "billing");
    assert!(
        h.text()
            .contains("postgres://db.eu-west-1.internal/billing"),
        "preview follows the draft before opening the picker\n{}",
        h.text()
    );

    let _chooser = require(h.find("Choose…"), "visible SSH key chooser");
    assert!(h.tab_to(DIALOG_KEY), "chooser is in the dialog focus order");
    press(&mut h, KeyCode::Enter);
    assert!(
        h.is_open(DIALOG_FORM),
        "parent form remains open under the picker"
    );
    assert!(h.is_open(DIALOG_PICKER), "chooser opens the nested picker");
    assert!(
        h.text().contains("~/.ssh/prod_rsa"),
        "picker did not open\n{}",
        h.text()
    );
    press(&mut h, KeyCode::Down);
    press(&mut h, KeyCode::Enter);
    assert!(
        !h.is_open(DIALOG_PICKER),
        "key choice closes only the picker"
    );
    assert!(h.is_open(DIALOG_FORM), "key choice leaves the form open");
    assert!(
        h.text().contains("~/.ssh/prod_rsa"),
        "chosen key not reflected\n{}",
        h.text()
    );
    assert!(
        !h.text().contains("~/.ssh/id_ed25519"),
        "the chosen key replaces the initial stable key identity\n{}",
        h.text()
    );
    assert!(
        h.text()
            .contains("postgres://db.eu-west-1.internal/billing"),
        "the parent draft survives the picker\n{}",
        h.text()
    );
    assert!(
        h.diagnostics().is_empty(),
        "after choosing SSH key: {:?}",
        h.diagnostics()
    );
    assert_eq!(
        h.focus(),
        Some(DIALOG_KEY),
        "picker choice restores form focus"
    );

    let test = require(h.find("Test connection"), "test action");
    click(&mut h, test.0.saturating_add(1), test.1);
    assert!(h.text().contains("Connected in 42 ms"));
    assert!(h.text().contains("Add connection"), "test keeps form open");
    assert!(
        h.diagnostics().is_empty(),
        "after test action: {:?}",
        h.diagnostics()
    );

    assert!(
        h.tab_to(DIALOG_SAVE),
        "Save action is in the dialog focus order"
    );
    press(&mut h, KeyCode::Enter);
    assert!(!h.is_open(DIALOG_FORM), "Save closes the form");
    assert!(h.text().contains("Form · Save · billing"));
    assert_eq!(h.top_layer(), LayerId::PAGE);
    assert!(
        h.diagnostics().is_empty(),
        "after save: {:?}",
        h.diagnostics()
    );
    assert_eq!(
        h.focus(),
        Some(DIALOG_OPEN),
        "closing the form restores focus to its opener"
    );
    press(&mut h, KeyCode::Tab);
    assert!(
        h.text().contains("Form · Save · billing"),
        "a later input must not reinterpret the Save receipt as Cancel\n{}",
        h.text()
    );
    let open = require(
        h.find("Add connection…"),
        "Add connection opener after Save",
    );
    click(&mut h, open.0.saturating_add(1), open.1);
    assert!(
        h.text().contains("orders-prod"),
        "Save leaves the next form fresh"
    );
    assert!(
        h.is_open(DIALOG_FORM),
        "reopening remains possible after Save"
    );
}

#[test]
fn dialog_kinds_outside_picker_dismissal_restores_form_lifecycle() {
    let mut h = harness(PageId::DialogKinds);
    let open = require(h.find("Add connection…"), "Add connection opener");
    click(&mut h, open.0.saturating_add(1), open.1);
    let tunnel = require(h.find("Connect through an SSH tunnel"), "tunnel checkbox");
    click(&mut h, tunnel.0, tunnel.1);
    let _chooser = require(h.find("Choose…"), "visible SSH key chooser");
    assert!(h.tab_to(DIALOG_KEY), "chooser is in the dialog focus order");
    press(&mut h, KeyCode::Enter);
    assert!(h.is_open(DIALOG_PICKER), "chooser opens the nested picker");

    let _ = h.mouse(MouseKind::Down, 119, 39);
    assert!(
        h.diagnostics().is_empty(),
        "after outside-click picker dismissal: {:?}",
        h.diagnostics()
    );
    assert!(!h.is_open(DIALOG_PICKER), "outside click closes the picker");
    assert!(h.is_open(DIALOG_FORM), "outside click leaves the form open");
    assert_eq!(
        h.focus(),
        Some(DIALOG_KEY),
        "outside click restores chooser focus"
    );

    assert!(
        h.tab_to(DIALOG_SAVE),
        "form remains responsive after outside-click dismissal"
    );
    press(&mut h, KeyCode::Enter);
    assert!(!h.is_open(DIALOG_FORM), "Save closes the form");
    assert!(h.text().contains("Form · Save · orders-prod"));
    assert!(
        h.diagnostics().is_empty(),
        "after Save following outside-click dismissal: {:?}",
        h.diagnostics()
    );
}

#[test]
fn dialog_kinds_picker_escape_restores_form_and_cancel_discards_changes() {
    let mut h = harness(PageId::DialogKinds);
    let open = require(h.find("Add connection…"), "Add connection opener");
    click(&mut h, open.0.saturating_add(1), open.1);
    let tunnel = require(h.find("Connect through an SSH tunnel"), "tunnel checkbox");
    click(&mut h, tunnel.0, tunnel.1);
    let _chooser = require(h.find("Choose…"), "visible SSH key chooser");
    assert!(h.tab_to(DIALOG_KEY), "chooser is in the dialog focus order");
    press(&mut h, KeyCode::Enter);
    assert!(h.is_open(DIALOG_PICKER), "chooser opens the nested picker");
    assert!(
        h.text().contains("~/.ssh/prod_rsa"),
        "picker did not open\n{}",
        h.text()
    );
    press(&mut h, KeyCode::Esc);
    assert!(h.text().contains("Add connection"));
    assert!(
        h.text().contains("Choose…"),
        "form returned without its chooser\n{}",
        h.text()
    );
    assert_eq!(h.focus(), Some(DIALOG_KEY), "Escape restores form focus");
    assert!(
        h.diagnostics().is_empty(),
        "after picker Escape: {:?}",
        h.diagnostics()
    );
    press(&mut h, KeyCode::Esc);
    assert!(
        !h.is_open(DIALOG_FORM),
        "Escape closes the form\n{}",
        h.text()
    );
    assert_eq!(h.top_layer(), LayerId::PAGE);
    assert!(h.text().contains("Form · Cancel"), "{}", h.text());
    assert!(
        h.diagnostics().is_empty(),
        "after form Escape: {:?}",
        h.diagnostics()
    );

    let open = require(
        h.find("Add connection…"),
        "Add connection opener after cancel",
    );
    click(&mut h, open.0.saturating_add(1), open.1);
    assert!(
        h.text().contains("orders-prod"),
        "cancel discards the draft"
    );
    assert!(
        !h.text().contains("Choose…"),
        "cancel resets the tunnel flag"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn every_page_renders_at_representative_sizes_without_panic() {
    for (width, height) in [
        (72, 20),
        (80, 24),
        (100, 30),
        (120, 40),
        (160, 50),
        (200, 60),
    ] {
        for page in PageId::ALL {
            let mut h = Harness::new(App::with_page(page), Theme::junie(), width, height);
            assert!(
                h.text().contains(page.title()),
                "page {page:?} title is not visible at {width}x{height}"
            );
            for _ in 0..4 {
                press(&mut h, KeyCode::Tab);
                press(&mut h, KeyCode::Down);
                press(&mut h, KeyCode::Right);
            }
            assert!(!h.text().trim().is_empty());
        }
    }
}

#[test]
fn below_minimum_size_shows_reduced_state() {
    let h = Harness::new(App::with_page(PageId::Buttons), Theme::junie(), 60, 15);
    let text = h.text();
    assert!(text.contains("Terminal too small"));
    assert!(text.contains("Need 72×20, have 60×15"));
    assert!(!text.contains("Playground"));
}

#[test]
fn resize_recovers_from_too_small() {
    let mut h = Harness::new(App::with_page(PageId::Buttons), Theme::junie(), 60, 15);
    resize(&mut h, 120, 40);
    assert!(h.text().contains("Playground"));
    assert!(h.text().contains("State matrix"));
}

#[test]
fn tab_traversal_is_deterministic_and_wraps() {
    let mut h = harness(PageId::Buttons);
    let start = h.focus();
    let reachable: Vec<_> = h.ring().reachable().map(|entry| entry.id).collect();
    assert!(start.is_some());
    let buttons = Id::root("showcase_ui::pages::buttons::buttons");
    let expected = [
        APP_NAV,
        buttons.index(0),
        buttons.index(1),
        buttons.index(2),
        buttons.index(3),
        buttons.index(4),
        buttons.index(5),
        buttons.index(8),
        Id::root("showcase_ui::pages::buttons.page_tabs"),
    ];
    assert_eq!(reachable.as_slice(), expected.as_slice());
    assert_eq!(
        reachable.len(),
        9,
        "nav, seven enabled playground buttons, then native page Tabs"
    );
    let start = require(start, "initial navigation focus");
    let mut seen = vec![start];
    for _ in 0..reachable.len().saturating_add(1) {
        press(&mut h, KeyCode::Tab);
        let current = require(h.focus(), "focus remains in the ring");
        if current == start {
            break;
        }
        seen.push(current);
    }
    assert_eq!(
        h.focus(),
        Some(start),
        "focus wraps to the initial navigation stop"
    );
    assert_eq!(seen.len(), reachable.len());
    let mut backwards = Vec::new();
    for _ in 0..seen.len() {
        press(&mut h, KeyCode::BackTab);
        backwards.push(require(h.focus(), "backward traversal stays reachable"));
    }
    let mut expected = seen;
    expected.reverse();
    assert_eq!(backwards, expected);
}

#[test]
fn disabled_buttons_are_skipped_and_cannot_activate() {
    let mut h = harness(PageId::Buttons);
    let (x, y) = require(h.find("Disabled primary"), "disabled fixture is visible");
    let before = h.snapshot().digest();
    click(&mut h, x, y);
    assert_eq!(
        h.snapshot().digest(),
        before,
        "disabled pointer input is inert"
    );
    assert!(h.text().contains("Disabled primary"));
    assert!(!h.text().contains("last: Disabled primary"));
    assert_eq!(h.text().matches("activations").count(), 0);
}

#[test]
fn mouse_click_activates_and_keyboard_enter_activates() {
    let mut h = harness(PageId::Buttons);
    let (x, y) = require(h.find("Run task"), "run button is visible");
    click(&mut h, x, y);
    assert!(h.text().contains("Run task ✓"));
    assert!(h.text().contains("1 activations"));
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("Preview ✓"));
    assert!(h.text().contains("2 activations"));
    press(&mut h, KeyCode::Char(' '));
    assert!(h.text().contains("3 activations"));
}

#[test]
fn picker_launcher_is_reachable_after_navigation_focus() {
    let mut h = harness(PageId::Pickers);
    assert_eq!(h.focus(), Some(APP_NAV));
    press(&mut h, KeyCode::Tab);
    let focused = require(h.focus(), "picker launcher focus");
    assert_ne!(focused, APP_NAV, "Tab stayed on the shell navigation");
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("Open quickly"), "{focused:?}");
}

#[test]
fn remote_picker_loads_and_selects_a_branch() {
    let mut h = harness(PageId::Pickers);
    let (x, y) = require(h.find("Remote branch"), "remote launcher");
    click(&mut h, x, y);
    assert!(h.text().contains("Fetching branches from origin"));

    h.ticks(3);
    assert!(h.text().contains("feat/rate-limit"), "{}", h.text());
    press(&mut h, KeyCode::Down);
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("feat/rate-limit"), "{}", h.text());
    assert!(!h.text().contains("Remote branches · origin"));
}

#[test]
fn remote_fetch_failure_retries_only_after_the_focused_error_state() {
    let mut h = harness(PageId::Pickers);
    let (x, y) = require(h.find("Remote branch"), "remote launcher");
    click(&mut h, x, y);
    h.ticks(3);
    press(&mut h, KeyCode::Esc);

    let (x, y) = require(h.find("Remote branch"), "remote launcher after first fetch");
    click(&mut h, x, y);
    assert!(h.text().contains("Fetching branches from origin"));
    press(&mut h, KeyCode::Char('r'));
    assert!(h.text().contains("Fetching branches from origin"));
    h.ticks(3);
    assert!(h.text().contains("Could not reach origin"), "{}", h.text());
    assert!(h.text().contains("Press r to retry"));

    press(&mut h, KeyCode::Char('r'));
    assert!(h.text().contains("Fetching branches from origin"));
    h.ticks(3);
    assert!(h.text().contains("feat/rate-limit"), "{}", h.text());
}

#[test]
fn command_picker_filters_stable_keys_and_refuses_disabled_rows() {
    let mut h = harness(PageId::Pickers);
    let (x, y) = require(h.find("Run command"), "command launcher");
    click(&mut h, x, y);
    assert!(h.text().contains("Run a query first"));
    type_text(&mut h, "export");
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("Run a query first"));
    assert!(h.text().contains("Export results"));

    press(&mut h, KeyCode::Esc);
    type_text(&mut h, "format");
    let layer = require(h.layer_area(PICKERS_LAYER), "filtered command layer");
    let visible = area_text(&h, layer);
    assert!(visible.contains("Format SQL"), "{visible}");
    assert!(
        !visible.contains("Run query"),
        "unmatched rows must be hidden: {visible}"
    );
    assert!(
        !visible.contains("Explain query"),
        "unmatched rows must be hidden: {visible}"
    );
    let (x, y) = require(h.find("Format SQL"), "the matching command is visible");
    click(&mut h, x, y);
    assert!(h.text().contains("Format SQL"), "{}", h.text());
    assert!(
        h.text().contains("edit.format"),
        "the displayed row keeps its stable key"
    );

    let (x, y) = require(h.find("Run command"), "command launcher after selection");
    click(&mut h, x, y);
    type_text(&mut h, "zzzz");
    let layer = require(h.layer_area(PICKERS_LAYER), "zero-match command layer");
    let visible = area_text(&h, layer);
    assert!(visible.contains("No matches"), "{visible}");
    assert!(
        !visible.contains("Run query"),
        "zero matches paint no command rows: {visible}"
    );
    assert!(
        !visible.contains("Format SQL"),
        "zero matches paint no command rows: {visible}"
    );
}

#[test]
fn command_picker_scroll_projects_effective_rows_for_cursor_and_pointer() {
    let mut h = harness(PageId::Pickers);
    let (x, y) = require(h.find("Run command"), "command launcher");
    click(&mut h, x, y);

    // Run query -> Explain query -> Format SQL (skip disabled Export) ->
    // Reconnect (skip disabled Commit).
    press(&mut h, KeyCode::Down);
    press(&mut h, KeyCode::Down);
    press(&mut h, KeyCode::Down);
    let layer = require(h.layer_area(PICKERS_LAYER), "scrolled command layer");
    let at_end = area_text(&h, layer);
    assert!(
        at_end.contains("Reconnect"),
        "Down to the last enabled row reveals Reconnect: {at_end}"
    );
    assert!(
        !at_end.contains("Run query"),
        "the first row leaves the effective view: {at_end}"
    );

    // Reverse the same three enabled-row moves across the two disabled rows.
    press(&mut h, KeyCode::Up);
    press(&mut h, KeyCode::Up);
    press(&mut h, KeyCode::Up);
    let layer = require(h.layer_area(PICKERS_LAYER), "command layer at first row");
    let at_start = area_text(&h, layer);
    assert!(
        at_start.contains("Run query"),
        "Up to the first enabled row restores Run query: {at_start}"
    );
    assert!(
        !at_start.contains("Reconnect"),
        "the last row leaves the effective view: {at_start}"
    );
    press(&mut h, KeyCode::Enter);
    assert!(
        !h.is_open(PICKERS_LAYER),
        "Enter chooses the first enabled row"
    );
    let chosen_y = require(h.find("Chosen"), "chosen result row").1;
    assert!(
        h.row(chosen_y).contains("Run query"),
        "the cursor returned to the first row: {}",
        h.row(chosen_y)
    );
    assert!(
        h.row(chosen_y.saturating_add(1)).contains("run.query"),
        "the first row keeps its exact stable key: {}",
        h.row(chosen_y.saturating_add(1))
    );

    let (x, y) = require(h.find("Run command"), "command launcher after selection");
    click(&mut h, x, y);
    press(&mut h, KeyCode::Down);
    press(&mut h, KeyCode::Down);
    press(&mut h, KeyCode::Down);
    let layer = require(h.layer_area(PICKERS_LAYER), "command layer at Reconnect");
    let at_end = area_text(&h, layer);
    assert!(
        at_end.contains("Reconnect"),
        "Down reaches the displayed Reconnect row: {at_end}"
    );
    assert!(
        !at_end.contains("Run query"),
        "the stale first row is not painted at the end: {at_end}"
    );
    let (x, y) = require(h.find("Reconnect"), "displayed scrolled command");
    click(&mut h, x, y);

    assert!(
        !h.is_open(PICKERS_LAYER),
        "pointer selects the displayed row"
    );
    let chosen_y = require(h.find("Chosen"), "chosen Reconnect result row").1;
    assert!(
        h.row(chosen_y).contains("Reconnect"),
        "the pointer selected the displayed label: {}",
        h.row(chosen_y)
    );
    let detail_row = h.row(chosen_y.saturating_add(1));
    assert!(
        detail_row.contains("Detail") && detail_row.contains("conn.reconnect"),
        "the pointer selected the displayed stable key: {detail_row}"
    );
}

#[test]
fn empty_query_backspace_keeps_picker_layer_drawn_and_usable() {
    let mut h = harness(PageId::Pickers);
    for (launcher, title) in [
        (PICKERS_OPEN_QUICK, "Open quickly"),
        (PICKERS_OPEN_TABS, "Open tabs"),
        (PICKERS_OPEN_LEVEL, "Safe Mode"),
        (PICKERS_OPEN_COMMANDS, "Run command"),
        (PICKERS_OPEN_REMOTE, "Remote branches · origin"),
    ] {
        assert!(
            h.tab_to(launcher),
            "picker launcher {launcher:?} is reachable"
        );
        let prior_focus = h.focus();
        assert_eq!(prior_focus, Some(launcher));
        press(&mut h, KeyCode::Enter);
        let modal_focus = h.focus();
        press(&mut h, KeyCode::Backspace);
        assert!(
            h.is_open(PICKERS_LAYER),
            "Backspace keeps {launcher:?} open"
        );
        let layer = require(h.layer_area(PICKERS_LAYER), "picker remains drawn");
        let visible = area_text(&h, layer);
        assert!(
            visible.contains(title),
            "{launcher:?} stays visible: {visible}"
        );
        assert_eq!(
            h.focus(),
            modal_focus,
            "Backspace retains modal focus for {launcher:?}"
        );

        press(&mut h, KeyCode::Esc);
        assert!(!h.is_open(PICKERS_LAYER), "Escape closes {launcher:?}");
        assert!(h.layer_area(PICKERS_LAYER).is_none());
        assert_eq!(
            h.focus(),
            prior_focus,
            "Escape restores focus after {launcher:?}"
        );
        if launcher == PICKERS_OPEN_REMOTE {
            h.ticks(3);
            assert!(!h.is_open(PICKERS_LAYER));
            assert!(h.layer_area(PICKERS_LAYER).is_none());
            assert!(!h.text().contains("feat/rate-limit"));
        }
    }
}

#[test]
fn escape_returns_focus_to_navigation_without_changing_page() {
    let mut h = harness(PageId::Pickers);
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Esc);

    assert_eq!(h.app().page(), PageId::Pickers);
    assert_eq!(h.focus(), Some(APP_NAV));
}

#[test]
fn hover_and_focus_render_differently() {
    let mut h = harness(PageId::Buttons);
    let (x, y) = require(h.find("Run task"), "run button is visible");
    let initial = h.snapshot().digest();
    move_mouse(&mut h, x, y);
    let hovered = h.snapshot().digest();
    assert!(h.hover().is_some());
    assert_ne!(
        initial, hovered,
        "hover state changes the resolved button style"
    );
    press(&mut h, KeyCode::Tab);
    let focused = h.snapshot().digest();
    assert!(h.focus().is_some());
    assert_ne!(hovered, focused, "keyboard focus has a distinct rendering");
}

#[test]
fn hit_testing_prefers_rows_over_their_container() {
    let mut h = harness(PageId::Tables);
    let (x, y) = require(h.find("#1042"), "task row is visible");
    move_mouse(&mut h, x.saturating_add(12), y);
    let hovered = require(h.hover(), "table receives pointer hover");
    let area = require(
        h.area_of_part(hovered, PartRef::item(Part::CELL, ItemKey::num(1042))),
        "hovered task cell has an area",
    );
    assert_eq!(area.height, 1, "the keyed cell region is one row tall");
}

#[test]
fn list_scrolling_and_selection() {
    let mut h = harness(PageId::Lists);
    press(&mut h, KeyCode::Tab);
    for _ in 0..19 {
        press(&mut h, KeyCode::Down);
    }
    assert!(h.text().contains("Erlang"));
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("Chosen: Erlang"));
    press(&mut h, KeyCode::Tab);
    let before = h.text().matches("✓").count();
    press(&mut h, KeyCode::Char(' '));
    assert_ne!(h.text().matches("✓").count(), before);
    press(&mut h, KeyCode::Char('a'));
    assert!(h.text().contains("src/api/auth"));
}

#[test]
fn tree_expand_collapse_and_focus_bar_column_is_stable() {
    let mut h = harness(PageId::Trees);
    press(&mut h, KeyCode::Tab);
    let src_y = require(h.find_row("src"), "tree root is visible");
    let bar_x = require(focus_bar_x(&h, src_y), "focused root has a focus bar");
    assert!(h.text().contains("config.rs"));
    press(&mut h, KeyCode::Left);
    assert!(!h.text().contains("config.rs"));
    press(&mut h, KeyCode::Right);
    press(&mut h, KeyCode::Down);
    press(&mut h, KeyCode::Right);
    assert!(h.text().contains("auth.rs"));
    press(&mut h, KeyCode::Down);
    let auth_y = require(h.find_row("auth.rs"), "expanded api file is visible");
    assert_eq!(
        focus_bar_x(&h, auth_y),
        Some(bar_x),
        "focus bar column is stable across depth"
    );
}

fn first_data_row(h: &Harness<App>) -> String {
    let header = require(h.find_row("ID"), "table header");
    h.row(header.saturating_add(1))
}

#[test]
fn table_sorts_both_directions_and_clears() {
    let mut h = harness(PageId::Tables);
    resize(&mut h, 160, 40);
    let (x, y) = require(h.find("Changes"), "sortable Changes header");
    click(&mut h, x, y);
    assert!(
        first_data_row(&h).contains("#1043"),
        "ascending puts zero changes first"
    );
    click(&mut h, x, y);
    assert!(
        first_data_row(&h).contains("#1049"),
        "descending puts 118 changes first"
    );
    assert!(h.text().contains("sorted by changes ▾"));
}

#[test]
fn header_click_sorts() {
    let mut h = harness(PageId::Tables);
    let (x, y) = require(h.find("Owner"), "sortable Owner header");
    click(&mut h, x, y);
    assert!(first_data_row(&h).contains("ana"));
    click(&mut h, x, y);
    assert!(first_data_row(&h).contains("sofia"));
}

#[test]
fn editable_table_commit_cancel_and_validation() {
    let mut h = harness(PageId::Editable);
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("EDIT"));
    press(&mut h, KeyCode::End);
    type_text(&mut h, " now");
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("Add rate limiting to auth endpoi"));
    press(&mut h, KeyCode::Right);
    press(&mut h, KeyCode::Right);
    press(&mut h, KeyCode::Enter);
    type_text(&mut h, "zzz");
    press(&mut h, KeyCode::Esc);
    assert!(!h.text().contains("zzz"));
    press(&mut h, KeyCode::Right);
    press(&mut h, KeyCode::Right);
    press(&mut h, KeyCode::Right);
    press(&mut h, KeyCode::Enter);
    control(&mut h, 'l');
    type_text(&mut h, "abc");
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("Changes must be a whole number"));
    assert!(h.text().contains("EDIT"));
    press(&mut h, KeyCode::Esc);
}

#[test]
fn button_toggle_variants_remain_live_without_hidden_choice_controls() {
    let mut h = harness(PageId::Buttons);
    let root = Id::root("showcase_ui::pages::buttons::buttons");
    let auto = root.index(4);
    let verbose = root.index(5);
    let auto_area = require(h.area_of(auto), "Auto-approve button");
    let verbose_area = require(h.area_of(verbose), "Verbose button");
    assert!(area_text(&h, auto_area).contains("○ Auto-approve"));
    assert!(area_text(&h, verbose_area).contains("● Verbose"));
    click(&mut h, auto_area.x + 2, auto_area.y);
    assert!(area_text(&h, auto_area).contains("● Auto-approve"));
    assert!(h.text().contains("Auto-approve on"));
    click(&mut h, verbose_area.x + 2, verbose_area.y);
    assert!(area_text(&h, verbose_area).contains("○ Verbose"));
    assert!(h.text().contains("Verbose off"));
    assert!(h.diagnostics().is_empty());
}

#[test]
fn input_defaults_match_the_reference_project_and_empty_branch() {
    let h = harness(PageId::Inputs);
    let project = Id::root("showcase_ui::pages::inputs::inputs.name");
    let branch = Id::root("showcase_ui::pages::inputs::inputs.branch");
    assert_eq!(
        area_text(&h, require(h.area_of(project), "project input")).trim(),
        "▎ payments-gateway"
    );
    assert_eq!(
        area_text(&h, require(h.area_of(branch), "branch input")).trim(),
        "▎ feat/…"
    );
}

#[test]
fn input_editing_commit_and_revert() {
    let mut h = harness(PageId::Inputs);
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::End);
    type_text(&mut h, "-v2");
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("payments-gateway-v2"));
    press(&mut h, KeyCode::Enter);
    type_text(&mut h, "XX");
    press(&mut h, KeyCode::Esc);
    assert!(!h.text().contains("XX"));
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::End);
    type_text(&mut h, "-v2");
    press(&mut h, KeyCode::Tab);
    let branch = Id::root("showcase_ui::pages::inputs::inputs.branch");
    assert_eq!(
        area_text(&h, require(h.area_of(branch), "branch input")).trim(),
        "▎ -v2"
    );
}

#[test]
fn textarea_scrolls_with_wheel_and_keys() {
    let mut h = harness(PageId::TextAreas);
    assert!(h.text().contains("1. Read"));
    let (x, y) = require(h.find("1. Read"), "checklist starts at the top");
    let before = h.snapshot().digest();
    wheel(&mut h, Axis::V, 3, x, y);
    assert_ne!(before, h.snapshot().digest());
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Enter);
    for _ in 0..30 {
        press(&mut h, KeyCode::Down);
    }
    assert!(h.text().contains("28. Run"));
}

#[test]
fn form_validation_blocks_submit_and_focuses_first_error() {
    let mut h = harness(PageId::Forms);
    control(&mut h, 's');
    assert!(h.text().contains("Required: summary"));
    assert_eq!(h.focus(), Some(FORM_SUMMARY), "summary field is focused");
    let summary = require(h.area_of(FORM_SUMMARY), "summary field has an area");
    click(&mut h, summary.x.saturating_add(1), summary.y);
    type_text(&mut h, "Fix login bug");
    press(&mut h, KeyCode::Enter);
    control(&mut h, 's');
    assert!(h.text().contains("Creating task"));
}

#[test]
fn modal_traps_focus_and_restores_it() {
    let mut h = harness(PageId::Dialogs);
    press(&mut h, KeyCode::Tab);
    let launcher = require(h.focus(), "confirm launcher is focusable");
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("Run task now?"));
    assert!(h.top_layer().index() > LayerId::PAGE.index());
    assert_ne!(h.focus(), Some(launcher));
    press(&mut h, KeyCode::Tab);
    assert_ne!(h.focus(), Some(launcher));
    press(&mut h, KeyCode::Esc);
    assert!(h.text().contains("Cancelled"));
    assert_eq!(h.top_layer(), LayerId::PAGE);
    assert_eq!(h.focus(), Some(launcher));

    press(&mut h, KeyCode::Enter);
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("Task started"));
}

#[test]
fn prompt_dialog_validates_and_returns_value() {
    let mut h = harness(PageId::Dialogs);
    let field = Id::root("showcase_ui::pages::dialogs::dialogs.prompt.layer").part(Part::FIELD);
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("Rename task"));
    assert_eq!(h.focus(), Some(field));
    assert!(h.state_of(field).contains(StateFlags::EDITING));

    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("Name cannot be empty"));
    assert!(!h.state_of(field).contains(StateFlags::EDITING));
    press(&mut h, KeyCode::Enter);
    assert!(h.state_of(field).contains(StateFlags::EDITING));
    type_text(&mut h, "Ship it");
    assert!(h.text().contains("Ship it"));
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("Task: Ship it"));
}

#[test]
fn settings_screen_remove_member_flow() {
    let mut h = harness(PageId::Settings);
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Right);
    assert!(h.text().contains("Mira Okafor"));
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Down);
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Tab);
    let remove = require(h.focus(), "remove launcher is focused");
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("Remove member?"));
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Enter);
    assert!(!h.text().contains("jonas@acme.dev"));
    assert!(h.text().contains("5 members"));
    assert_eq!(h.focus(), Some(remove), "focus returns to remove launcher");
}

#[test]
fn settings_member_rows_render_account_kind_and_lifecycle() {
    let theme = Theme::junie();
    let person_ink = theme.color.info;
    let service_ink = theme.color.accent;
    let mut h = Harness::new(App::with_page(PageId::Settings), theme, 120, 40);
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Right);

    let (person_x, person_y) = require(h.find("Person · Mira Okafor"), "person row label");
    let person_marker = h.cell(person_x - 2, person_y);
    assert_eq!(person_marker.symbol(), "▪");
    assert_eq!(person_marker.fg, person_ink);

    let (service_x, service_y) = require(
        h.find("Service account · deploy-bot"),
        "service account row label",
    );
    let service_marker = h.cell(service_x - 2, service_y);
    assert_eq!(service_marker.symbol(), "→");
    assert_eq!(service_marker.fg, service_ink);
    assert_ne!(person_marker.fg, service_marker.fg);

    let (active_x, active_y) = require(h.find("active"), "active lifecycle label");
    let active_suffix = h.cell(active_x + "active".len() as u16, active_y);
    assert_eq!(active_suffix.symbol(), "●");
    assert_eq!(active_suffix.fg, Color::Green);

    let pending_label = "invitation pending";
    let (pending_x, pending_y) = require(h.find(pending_label), "pending invite label");
    let pending_suffix = h.cell(pending_x + pending_label.len() as u16, pending_y);
    assert_eq!(pending_suffix.symbol(), "○");
    assert_eq!(pending_suffix.fg, Color::Yellow);

    let mut compact = Harness::new(App::with_page(PageId::Settings), Theme::junie(), 72, 24);
    press(&mut compact, KeyCode::Tab);
    press(&mut compact, KeyCode::Right);
    let compact_text = compact.text();
    assert!(
        compact_text.contains("Service"),
        "compact Members tab does not show service account: {compact_text:?}"
    );
    let (service_x, service_y) = require(
        compact.find("Service account"),
        "compact service account label",
    );
    let service_row = compact.row(service_y);
    assert!(
        service_row.contains('…'),
        "compact member label or lifecycle status uses its ellipsis budget: {service_row:?}"
    );
    assert!(
        service_x.saturating_add("Service account".len() as u16) <= 72,
        "service account label stays inside the viewport"
    );
    let pending_y = (0..24u16)
        .find(|y| compact.row(*y).contains("invitation"))
        .expect("compact frame retains pending lifecycle text");
    let pending_row = compact.row(pending_y);
    assert!(
        pending_row.contains('…')
            && pending_row.contains("○")
            && !pending_row.contains(pending_label),
        "compact status text is shortened and retains its lifecycle suffix: {pending_row:?}"
    );
}

#[test]
fn task_runner_animates_and_can_be_cancelled() {
    let mut h = harness(PageId::TaskRunner);
    press(&mut h, KeyCode::Char('r'));
    assert!(h.text().contains("Pipeline · running"));
    assert!(h.text().contains("compile started"));
    h.ticks(8);
    assert!(h.text().contains('%'));
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Tab);
    let cancel = require(h.focus(), "cancel launcher is focused");
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("Cancel pipeline?"));
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("pipeline cancelled"));
    assert!(!h.text().contains("Pipeline · running"));
    assert_ne!(
        h.focus(),
        Some(cancel),
        "disabled cancel launcher cannot retain focus"
    );
    assert_eq!(h.focus(), Some(APP_NAV), "focus reconciles to navigation");
}

#[test]
fn scrollbar_click_and_drag_move_the_view() {
    let mut h = harness(PageId::Scrolling);
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Tab);
    let (row_x, row_y) = require(h.find("Row 001"), "long-list row is visible");
    let before = h.snapshot().digest();
    wheel(&mut h, Axis::V, 8, row_x, row_y);
    assert_ne!(before, h.snapshot().digest());
    let thumb = require(
        h.area_of_part(SCROLL_LIST, PartRef::of(Part::THUMB)),
        "long-list scrollbar thumb is registered",
    );
    let track = require(
        h.area_of_part(SCROLL_LIST, PartRef::of(Part::TRACK)),
        "long-list scrollbar track is registered",
    );
    let drag_before = h.snapshot().digest();
    drag(
        &mut h,
        (thumb.x, thumb.y.saturating_add(thumb.height / 2)),
        (track.x, track.bottom().saturating_sub(2)),
    );
    assert_ne!(
        drag_before,
        h.snapshot().digest(),
        "scrollbar drag changes the viewport"
    );
    assert!(h.text().contains("list="));
}

#[test]
fn keyboard_navigation_between_pages() {
    let mut h = harness(PageId::Overview);
    press(&mut h, KeyCode::Char(']'));
    assert_eq!(h.app().page(), PageId::Chrome);
    press(&mut h, KeyCode::Char('['));
    assert_eq!(h.app().page(), PageId::Overview);

    press(&mut h, KeyCode::Down);
    assert_eq!(
        h.app().page(),
        PageId::Overview,
        "a group row does not route"
    );
    assert!(h.text().contains("App shell"));
    press(&mut h, KeyCode::Down);
    assert_eq!(h.app().page(), PageId::Chrome, "the next leaf is Chrome");
    let chrome_has_focusable_content = h.ring().reachable().any(|entry| entry.id != APP_NAV);
    press(&mut h, KeyCode::Enter);
    if chrome_has_focusable_content {
        assert_ne!(
            h.focus(),
            Some(APP_NAV),
            "Enter hands focus to page content"
        );
    } else {
        assert_eq!(
            h.focus(),
            Some(APP_NAV),
            "Chrome has no keyboard focus target; its brand is pointer-only"
        );
    }

    let mut tables = Harness::new(App::with_page(PageId::Grid), Theme::junie(), 120, 40);
    let (x, y) = require(
        visible_nav_leaf_row(&tables, "Tables"),
        "visible Basic table alias beside Data grid",
    );
    click(&mut tables, x, y);
    assert_eq!(tables.app().page(), PageId::Tables);
}

fn visible_text_part(h: &Harness<App>, id: Id, message: &str) -> (Rect, String) {
    let area = require(h.area_of_part(id, PartRef::of(Part::TEXT)), message);
    let text = area_text(h, area).trim_end_matches(' ').to_owned();
    (area, text)
}

fn quit_keys_with_default_select_all() {
    const INPUTS_NAME: Id = Id::root("showcase_ui::pages::inputs::inputs.name");
    const PROJECT_NAME: &str = "payments-gateway";

    let mut h = harness(PageId::Inputs);
    let (name_text_area, initial_name) =
        visible_text_part(&h, INPUTS_NAME, "visible NAME TEXT part");
    assert_eq!(initial_name, PROJECT_NAME, "default NAME value");
    assert!(h.tab_to(INPUTS_NAME), "Tab reaches NAME");
    assert_eq!(h.focus(), Some(INPUTS_NAME));
    assert!(
        h.state_of(INPUTS_NAME)
            .contains(StateFlags::FOCUSED | StateFlags::EDITING)
    );
    assert_eq!(
        visible_text_part(&h, INPUTS_NAME, "focused NAME TEXT part").1,
        PROJECT_NAME,
        "focus does not change the committed NAME value"
    );

    type_text(&mut h, "q");
    assert!(!h.app().quit(), "printable q belongs to the active editor");
    assert_eq!(
        visible_text_part(&h, INPUTS_NAME, "edited NAME TEXT part").1,
        "q",
        "default SelectAll replaces the selected project name"
    );
    let cursor = require(h.cursor(), "focused NAME publishes a cursor");
    assert!(name_text_area.contains(cursor), "cursor stays in NAME TEXT");
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

    press(&mut h, KeyCode::Esc);
    assert_eq!(
        visible_text_part(&h, INPUTS_NAME, "restored NAME TEXT part").1,
        PROJECT_NAME,
        "Esc restores the original project name"
    );
    control(&mut h, 'c');
    assert!(h.app().quit());
}

fn quit_keys_with_explicit_caret_end() {
    const INPUTS_NAME: Id = Id::root("showcase_ui::pages::inputs::inputs.name");
    const FOCUS_SELECT: Id = Id::root("showcase_ui::pages::inputs.focus_select");
    const PROJECT_NAME: &str = "payments-gateway";
    const CARET_END_LABEL: &str = "Focus selects: Caret end";

    let mut h = harness(PageId::Inputs);
    let (name_text_area, initial_name) =
        visible_text_part(&h, INPUTS_NAME, "visible NAME TEXT part");
    assert_eq!(
        initial_name, PROJECT_NAME,
        "fresh CaretEnd journey starts unchanged"
    );
    assert!(h.tab_to(FOCUS_SELECT), "Tab reaches focus policy selector");
    press(&mut h, KeyCode::Enter);
    assert!(h.is_open(FOCUS_SELECT), "Enter opens the real selector");
    press(&mut h, KeyCode::Down);
    press(&mut h, KeyCode::Enter);
    assert!(!h.is_open(FOCUS_SELECT), "Enter closes the selector");
    let selector_area = require(h.area_of(FOCUS_SELECT), "focus selector owner area");
    let selector_text = area_text(&h, selector_area);
    assert!(
        selector_text.contains(CARET_END_LABEL),
        "selector shows the chosen CaretEnd policy: {selector_text:?}"
    );

    assert!(
        h.tab_to(INPUTS_NAME),
        "Tab reaches NAME after policy selection"
    );
    assert_eq!(h.focus(), Some(INPUTS_NAME));
    assert!(
        h.state_of(INPUTS_NAME)
            .contains(StateFlags::FOCUSED | StateFlags::EDITING)
    );
    assert_eq!(
        visible_text_part(&h, INPUTS_NAME, "fresh NAME TEXT part").1,
        PROJECT_NAME
    );
    let appended_value = "payments-gatewayq";
    let appended_width = u16::try_from(appended_value.len()).expect("ASCII value fits u16");
    assert!(
        name_text_area.width > appended_width,
        "NAME TEXT fits full value and end cursor: {name_text_area:?}"
    );
    let initial_cursor_x = name_text_area
        .x
        .saturating_add(u16::try_from(PROJECT_NAME.len()).expect("ASCII project name fits u16"));
    assert_eq!(
        h.cursor().map(|cursor| (cursor.x, cursor.y)),
        Some((initial_cursor_x, name_text_area.y)),
        "CaretEnd starts after the existing visible text"
    );

    type_text(&mut h, "q");
    assert!(
        !h.app().quit(),
        "printable q still belongs to the active editor"
    );
    assert_eq!(
        visible_text_part(&h, INPUTS_NAME, "appended NAME TEXT part").1,
        appended_value,
        "CaretEnd preserves append behavior"
    );
    let appended_cursor_x = name_text_area.x.saturating_add(appended_width);
    assert_eq!(
        h.cursor().map(|cursor| (cursor.x, cursor.y)),
        Some((appended_cursor_x, name_text_area.y)),
        "CaretEnd cursor remains after the appended text"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

    press(&mut h, KeyCode::Esc);
    assert_eq!(
        visible_text_part(&h, INPUTS_NAME, "restored NAME TEXT part").1,
        PROJECT_NAME,
        "Esc restores the original project name"
    );
    control(&mut h, 'c');
    assert!(h.app().quit());
}

#[test]
fn quit_keys() {
    let mut overview = harness(PageId::Overview);
    press(&mut overview, KeyCode::Char('q'));
    assert!(overview.app().quit());

    quit_keys_with_default_select_all();
    quit_keys_with_explicit_caret_end();
}

#[test]
fn color_downgrade_still_renders() {
    let truecolor = harness(PageId::Overview).snapshot().digest();
    let mono = harness(PageId::Overview)
        .with_color(ColorLevel::Ansi16)
        .snapshot()
        .digest();
    assert_ne!(truecolor, mono);
}

#[test]
fn overview_playground_accepts_pointer_and_tab() {
    let mut h = harness(PageId::Overview);
    let (x, y) = require(h.find("Playground"), "playground tab is visible");
    move_mouse(&mut h, x, y);
    click(&mut h, x, y);
    assert!(h.text().contains("the live widget"));
    press(&mut h, KeyCode::Tab);
    assert!(h.focus().is_some());
}

#[test]
fn complete_navigation_visits_every_page_and_every_state() {
    let mut navigation = harness(PageId::Overview);
    let mut visited = Vec::with_capacity(NAV_ENTRIES.len());
    visited.push(navigation.app().page());
    assert_eq!(navigation.app().page(), PageId::Overview);

    for _ in 1..NAV_ENTRIES.len() {
        press(&mut navigation, KeyCode::Char(']'));
        assert!(
            navigation.diagnostics().is_empty(),
            "{:?}",
            navigation.diagnostics()
        );
        visited.push(navigation.app().page());
    }

    assert_eq!(visited[1], PageId::Chrome);
    assert!(visited.contains(&PageId::Buttons));
    assert!(visited.contains(&PageId::TaskRunner));
    press(&mut navigation, KeyCode::Char(']'));
    assert_eq!(navigation.app().page(), PageId::Overview);
    press(&mut navigation, KeyCode::Char('['));
    assert!(navigation.text().contains("Read-only report"));

    let mut ring = harness(PageId::Overview);
    exercise_focus_ring(&mut ring, PageId::Overview);

    for page in PageId::ALL {
        let mut h = harness(page);
        exercise_page_state(&mut h, page);
        assert_eq!(h.app().page(), page);
        assert!(h.text().contains(page.title()));
        assert!(
            h.diagnostics().is_empty(),
            "{page:?} diagnostics: {:?}",
            h.diagnostics()
        );
    }
}

#[test]
fn custom_theme_injection_repaints_every_page() {
    for page in PageId::ALL {
        let junie = Harness::new(App::with_page(page), Theme::junie(), 120, 40);
        let paper = Harness::new(App::with_page(page), Theme::paper(), 120, 40);

        assert_eq!(paper.app().page(), page);
        assert!(paper.text().contains(page.title()));
        assert!(junie.diagnostics().is_empty(), "{page:?} Junie diagnostics");
        assert!(paper.diagnostics().is_empty(), "{page:?} Paper diagnostics");
        assert_ne!(
            junie.snapshot().digest(),
            paper.snapshot().digest(),
            "Paper theme failed to repaint {page:?}"
        );
    }
}

#[test]
fn local_override_page_shows_three_distinct_buttons() {
    let buttons = harness(PageId::Buttons);
    let primary = cell_style(&buttons, "Run task");
    let secondary = cell_style(&buttons, "Preview");
    let danger = cell_style(&buttons, "Delete branch");
    assert_ne!(primary, secondary, "primary and secondary buttons merged");
    assert_ne!(primary, danger, "primary and danger buttons merged");
    assert_ne!(secondary, danger, "secondary and danger buttons merged");

    let panels = harness(PageId::Panels);
    assert!(panels.text().contains("Titled card"));
    assert!(panels.text().contains("Card · scrollable"));
    let patched_title = cell_style(&panels, "Titled card");
    assert_eq!(
        patched_title.0,
        require(
            Theme::junie()
                .color
                .fg
                .get(termrock::FgStep::Secondary.index())
                .copied(),
            "secondary foreground token"
        )
    );
    assert!(!patched_title.2.contains(Modifier::BOLD));
    let page_title = panels.cell(26, 2);
    assert_eq!(page_title.symbol(), "P");
    assert_eq!(
        page_title.fg,
        require(
            Theme::junie()
                .color
                .fg
                .get(termrock::FgStep::Primary.index())
                .copied(),
            "primary foreground token"
        )
    );
    assert!(page_title.modifier.contains(Modifier::BOLD));
    assert!(panels.diagnostics().is_empty(), "Panels diagnostics");
}

#[test]
fn application_navigation_help_and_inspector_commands_keep_their_routes() {
    let mut h = harness(PageId::Overview);
    press(&mut h, KeyCode::Char(']'));
    assert_eq!(h.app().page(), PageId::Chrome);
    press(&mut h, KeyCode::Char('['));
    assert_eq!(h.app().page(), PageId::Overview);
    press(&mut h, KeyCode::Char('?'));
    assert!(h.text().contains("Keyboard & mouse"));
    assert!(h.top_layer().index() > LayerId::PAGE.index());
    press(&mut h, KeyCode::Esc);
    assert_eq!(h.top_layer(), LayerId::PAGE);
    press(&mut h, KeyCode::Char('i'));
    assert!(h.text().contains("Inspector · on"));
    press(&mut h, KeyCode::Char('i'));
    assert!(!h.text().contains("Inspector · on"));
    assert!(!h.app().quit());
}

#[test]
fn overview_header_crumb_omits_foundations_and_idle_nav_rows_omit_the_gutter() {
    let h = Harness::new(App::new(), Theme::junie(), 120, 40);
    let screen = h.text();
    let header = screen.lines().next().unwrap_or("").to_owned();
    assert!(header.contains("/ Foundations / Overview"), "{header}");
    assert!(!header.contains('›'), "{header}");
    assert!(screen.contains("Buttons"));
    assert_eq!(NAV_ENTRIES.len(), 54);
    let idle = screen
        .lines()
        .find(|line| line.contains("Text areas"))
        .unwrap_or("")
        .to_owned();
    let sidebar: String = idle.chars().take(25).collect();
    assert!(
        !sidebar.starts_with('▎'),
        "non-focused nav row starts with the gutter: {sidebar}"
    );
    assert!(
        !sidebar.contains('▎'),
        "non-focused nav row paints the gutter: {sidebar}"
    );

    let diff = Harness::new(App::for_page_name("diff"), Theme::junie(), 120, 40);
    let diff_screen = diff.text();
    let diff_header = diff_screen.lines().next().unwrap_or("").to_owned();
    assert!(diff_header.contains("/ Components / Diff"), "{diff_header}");
    let editors = Harness::new(App::for_page_name("celleditors"), Theme::junie(), 100, 30);
    let editors_header = editors.text().lines().next().unwrap_or("").to_owned();
    assert!(
        editors_header.contains("/ Components / Cell editors"),
        "{editors_header}"
    );
    assert_eq!(diff.app().page(), PageId::Overview);
    let diff_row = diff_screen
        .lines()
        .find(|line| {
            let side: String = line.chars().take(25).collect();
            side.contains("Diff")
        })
        .unwrap_or("")
        .to_owned();
    let diff_side: String = diff_row.chars().take(25).collect();
    assert!(
        diff_side.contains('▎'),
        "focused diff row lost the gutter: {diff_side}"
    );
}

#[test]
fn sidebar_reveals_long_rows_at_frozen_width_breakpoints() {
    for (width, height, expected_sidebar, visible_label) in [
        (72, 20, 20, "Editable tab…"),
        (100, 30, 20, "Editable tab…"),
        (120, 40, 25, "Editable tables"),
    ] {
        let h = Harness::new(
            App::for_page_name("Editable"),
            Theme::junie(),
            width,
            height,
        );
        assert_eq!(h.app().page(), PageId::Editable);
        let area = h.area_of(APP_NAV).expect("catalog Tree region");
        assert_eq!(area.width, expected_sidebar, "terminal width {width}");
        let row = (area.y..area.bottom()).find(|y| {
            h.row(*y)
                .chars()
                .take(usize::from(area.width))
                .collect::<String>()
                .contains(visible_label)
        });
        let row = require(row, "route-revealed Editable tables row");
        let sidebar = h
            .row(row)
            .chars()
            .take(usize::from(area.width))
            .collect::<String>();
        assert!(sidebar.contains(visible_label), "{sidebar}");
        if width < 110 {
            assert!(!sidebar.contains("Editable tables"), "{sidebar}");
        }

        let track = h
            .area_of_part(APP_NAV, PartRef::of(Part::TRACK))
            .expect("63-node catalog overflows this viewport");
        let thumb = h
            .area_of_part(APP_NAV, PartRef::of(Part::THUMB))
            .expect("overflow has a visible thumb");
        assert_eq!(track.width, 1, "track stays in the final sidebar column");
        assert_eq!(track.right(), area.right());
        assert_eq!(thumb.width, 1);
        assert!(thumb.y >= track.y && thumb.bottom() <= track.bottom());
    }
}

#[test]
fn inspector_props_page_follows_real_navigation_and_closes() {
    let mut h = harness(PageId::Buttons);
    let inspector_id = Id::root("showcase_ui::app::inspector");

    // Use the existing shell command to open the real Inspector dock.
    press(&mut h, KeyCode::Char('i'));
    let inspector = require(
        h.area_of(inspector_id),
        "Inspector body has a registered rectangle after opening",
    );
    let opened = area_text(&h, inspector);
    let opened_page = require(
        opened.lines().find(|row| row.contains("page")),
        "Props page row is painted inside the Inspector rectangle",
    );
    assert!(
        opened_page.contains("Buttons"),
        "page Props row shows the current route: {opened_page:?}"
    );

    // Click a visible leaf in the real navigation Tree, not a direct App route.
    let (x, y) = require(
        visible_nav_leaf_row(&h, "Inputs"),
        "Inputs is a visible leaf beside the current Buttons route",
    );
    click(&mut h, x, y);
    assert_eq!(
        h.focus(),
        Some(APP_NAV),
        "choosing a different tree row keeps navigation focused"
    );
    assert!(
        h.runtime().typing_owner().is_none(),
        "the chosen route has no active text-input owner"
    );
    assert_eq!(h.app().page(), PageId::Inputs);

    let after_area = require(
        h.area_of(inspector_id),
        "Inspector remains registered after navigation",
    );
    assert_eq!(
        after_area, inspector,
        "navigation preserves the dock geometry"
    );
    let after = area_text(&h, after_area);
    let after_page = require(
        after.lines().find(|row| row.contains("page")),
        "updated Props page row remains inside the Inspector rectangle",
    );
    assert!(
        after_page.contains("Inputs") && !after_page.contains("Buttons"),
        "page Props row follows the newly selected route: {after_page:?}"
    );

    // Close through the same documented shell key and verify the owner disappears.
    press(&mut h, KeyCode::Char('i'));
    assert_eq!(h.app().page(), PageId::Inputs);
    assert!(
        h.area_of(inspector_id).is_none(),
        "the Inspector owner is absent after its close command"
    );
    assert!(
        h.diagnostics().is_empty(),
        "real open, route and close journey has no diagnostics: {:?}",
        h.diagnostics()
    );
}

#[test]
fn narrow_inspector_is_a_focused_dismissible_modal() {
    let mut h = Harness::new(App::with_page(PageId::Buttons), Theme::junie(), 80, 24);
    let prior_focus = h.focus();
    let prior_page = h.app().page();
    assert_eq!(prior_focus, Some(APP_NAV));

    press(&mut h, KeyCode::Char('i'));
    let modal = require(
        h.layer_area(INSPECTOR),
        "narrow Inspector opens a registered native layer",
    );
    assert_eq!(
        modal,
        Rect::new(44, 1, 34, 22),
        "80x24 Inspector keeps the PR26 right anchor, width, and vertical margin",
    );
    assert!(h.is_open(INSPECTOR));
    assert!(h.top_layer().index() > LayerId::PAGE.index());
    let body = area_text(&h, modal);
    assert!(body.contains("Inspector"), "modal title is visible: {body}");
    assert!(body.contains("State"), "state heading is visible: {body}");
    assert!(
        body.contains("page") && body.contains("Buttons") && body.contains("inspector"),
        "live Props body is visible: {body}"
    );

    let close = require(h.focus(), "modal initial focus is its Close action");
    assert_eq!(close, INSPECTOR_CLOSE);
    let close_area = require(h.area_of(close), "the focused Close action is registered");
    assert!(close_area.width > 0 && close_area.height > 0);
    assert!(
        close_area.x >= modal.x
            && close_area.y >= modal.y
            && close_area.right() <= modal.right()
            && close_area.bottom() <= modal.bottom(),
        "focused control stays inside the modal: {close_area:?} vs {modal:?}"
    );
    assert!(h.state_of(close).contains(StateFlags::FOCUSED));
    assert!(h.runtime().typing_owner().is_none());
    let focused_owners = h
        .ring()
        .entries()
        .iter()
        .filter(|entry| h.state_of(entry.id).contains(StateFlags::FOCUSED))
        .count();
    assert_eq!(focused_owners, 1, "the modal has exactly one focus owner");

    for _ in 0..6 {
        press(&mut h, KeyCode::Tab);
        assert_eq!(
            h.focus(),
            Some(close),
            "Tab stays inside the modal focus trap"
        );
    }
    for _ in 0..6 {
        press(&mut h, KeyCode::BackTab);
        assert_eq!(
            h.focus(),
            Some(close),
            "BackTab stays inside the modal focus trap"
        );
    }
    press(&mut h, KeyCode::Char(']'));
    assert_eq!(
        h.app().page(),
        prior_page,
        "route commands do not escape the modal"
    );

    press(&mut h, KeyCode::Esc);
    assert!(!h.is_open(INSPECTOR));
    assert!(h.layer_area(INSPECTOR).is_none());
    assert!(
        h.area_of(INSPECTOR_CLOSE).is_none(),
        "Esc unregisters the Close action"
    );
    assert_eq!(h.app().page(), prior_page);
    assert_eq!(h.focus(), prior_focus, "Esc restores the exact prior owner");
    assert_eq!(
        h.runtime().capture_owner(),
        None,
        "Esc clears pointer capture"
    );

    let header = require(
        h.area_of_part(HEADER_INSPECT, PartRef::of(Part::LABEL)),
        "real Inspector header hit is registered",
    );
    click(&mut h, header.x.saturating_add(header.width / 2), header.y);
    require(
        h.layer_area(INSPECTOR),
        "real header activation reopens the narrow modal",
    );
    let close_area = require(
        h.area_of(INSPECTOR_CLOSE),
        "the real Close action has a registered pointer target",
    );
    assert!(close_area.width > 0 && close_area.height > 0);
    click(
        &mut h,
        close_area.x.saturating_add(close_area.width / 2),
        close_area.y.saturating_add(close_area.height / 2),
    );
    assert!(
        !h.is_open(INSPECTOR),
        "real Close action activation closes the modal"
    );
    assert!(h.layer_area(INSPECTOR).is_none());
    assert!(
        h.area_of(INSPECTOR_CLOSE).is_none(),
        "closed action target is unregistered"
    );
    assert_eq!(
        h.app().page(),
        prior_page,
        "Close preserves the current page"
    );
    assert_eq!(
        h.focus(),
        prior_focus,
        "Close restores the exact prior owner"
    );
    assert_eq!(
        h.runtime().capture_owner(),
        None,
        "Close clears pointer capture"
    );
    assert!(
        h.diagnostics().is_empty(),
        "no Close diagnostics: {:?}",
        h.diagnostics()
    );

    press(&mut h, KeyCode::Char('i'));
    let modal = require(
        h.layer_area(INSPECTOR),
        "the shared i command reopens Inspector before outside dismissal",
    );
    let outside = (2, 10);
    assert!(
        outside.0 < modal.x
            || outside.0 >= modal.right()
            || outside.1 < modal.y
            || outside.1 >= modal.bottom(),
        "the backdrop test point is outside the modal"
    );
    click(&mut h, outside.0, outside.1);
    assert!(
        !h.is_open(INSPECTOR),
        "a real outside click dismisses the modal"
    );
    assert!(h.layer_area(INSPECTOR).is_none());
    assert!(
        h.area_of(INSPECTOR_CLOSE).is_none(),
        "outside dismissal unregisters Close"
    );
    assert_eq!(
        h.app().page(),
        prior_page,
        "outside click does not activate the page"
    );
    assert_eq!(
        h.focus(),
        prior_focus,
        "outside dismissal restores prior focus"
    );
    assert_eq!(
        h.runtime().capture_owner(),
        None,
        "outside dismissal clears pointer capture"
    );
    assert!(
        h.diagnostics().is_empty(),
        "no diagnostics: {:?}",
        h.diagnostics()
    );

    press(&mut h, KeyCode::Char('i'));
    assert!(
        h.is_open(INSPECTOR),
        "the shared i command reopens the modal"
    );
    press(&mut h, KeyCode::Char('i'));
    assert!(
        !h.is_open(INSPECTOR),
        "the shared i command closes the modal"
    );
    assert!(h.layer_area(INSPECTOR).is_none());
    assert!(
        h.area_of(INSPECTOR_CLOSE).is_none(),
        "the i toggle unregisters Close"
    );
    assert_eq!(h.focus(), prior_focus, "the i toggle restores prior focus");
    assert_eq!(
        h.runtime().capture_owner(),
        None,
        "the i toggle clears pointer capture"
    );
    assert!(
        h.diagnostics().is_empty(),
        "no diagnostics after the i toggle: {:?}",
        h.diagnostics()
    );
}

#[test]
fn narrow_inspector_geometry_matches_supported_reference_and_minimum_sizes() {
    let mut h = Harness::new(App::with_page(PageId::Buttons), Theme::junie(), 80, 24);
    let prior_focus = h.focus();

    press(&mut h, KeyCode::Char('i'));
    let reference = require(
        h.layer_area(INSPECTOR),
        "80x24 Inspector opens at the reference size",
    );
    assert_eq!(
        reference,
        Rect::new(44, 1, 34, 22),
        "80x24 Inspector uses exact right-anchored geometry",
    );

    resize(&mut h, 72, 20);
    let minimum = require(
        h.layer_area(INSPECTOR),
        "Inspector stays open at the existing 72x20 shell minimum",
    );
    assert_eq!(
        minimum,
        Rect::new(36, 1, 34, 18),
        "72x20 Inspector keeps its exact width, anchor, and clamped height",
    );
    let body = area_text(&h, minimum);
    assert!(
        body.contains("Inspector")
            && body.contains("State")
            && body.contains("page")
            && body.contains("Buttons")
            && body.contains("inspector"),
        "native headings and live page Props remain visible at the shell minimum: {body}",
    );
    assert_eq!(h.focus(), Some(INSPECTOR_CLOSE));

    press(&mut h, KeyCode::Esc);
    assert!(!h.is_open(INSPECTOR));
    assert!(h.layer_area(INSPECTOR).is_none());
    assert!(h.area_of(INSPECTOR_CLOSE).is_none());
    assert_eq!(
        h.focus(),
        prior_focus,
        "Esc restores focus after the minimum-size resize"
    );
    assert_eq!(
        h.runtime().capture_owner(),
        None,
        "Esc clears pointer capture"
    );
    assert!(
        h.diagnostics().is_empty(),
        "no diagnostics: {:?}",
        h.diagnostics()
    );
}

#[test]
fn narrow_inspector_resize_preserves_help_and_reconciles_the_100_column_mode() {
    let mut h = Harness::new(App::with_page(PageId::Buttons), Theme::junie(), 80, 24);
    let prior_focus = h.focus();
    press(&mut h, KeyCode::Char('i'));
    assert!(h.is_open(INSPECTOR));

    // Help becomes a child of Inspector. A resize must keep that unrelated top
    // layer open until its user dismissal lets Inspector reconcile to the dock.
    press(&mut h, KeyCode::Char('?'));
    assert!(h.is_open(HELP));
    let help_top = h.top_layer();
    resize(&mut h, 100, 40);
    assert!(h.is_open(HELP), "resize preserves the unrelated Help layer");
    assert!(
        h.is_open(INSPECTOR),
        "Inspector waits while Help owns the top layer"
    );
    assert_eq!(
        h.top_layer(),
        help_top,
        "resize does not replace the Help opening"
    );

    press(&mut h, KeyCode::Esc);
    assert!(!h.is_open(HELP));
    assert!(
        !h.is_open(INSPECTOR),
        "100 columns reconcile Inspector into the dock"
    );
    assert!(
        h.area_of(INSPECTOR_CLOSE).is_none(),
        "dock mode unregisters modal Close"
    );
    let dock = require(
        h.area_of(INSPECTOR),
        "wide mode paints the existing Inspector dock",
    );
    assert!(area_text(&h, dock).contains("Buttons"));
    assert_eq!(
        h.focus(),
        prior_focus,
        "mode transition restores prior focus"
    );
    assert_eq!(
        h.runtime().capture_owner(),
        None,
        "dock transition clears pointer capture"
    );

    resize(&mut h, 80, 24);
    let modal = require(
        h.layer_area(INSPECTOR),
        "shrinking the same Runtime reopens the native Inspector modal",
    );
    assert!(h.is_open(INSPECTOR));
    assert!(modal.width > 0 && modal.right() <= 80);
    assert_eq!(h.focus(), Some(INSPECTOR_CLOSE));
    assert!(h.area_of(INSPECTOR_CLOSE).is_some());
    assert!(h.state_of(INSPECTOR_CLOSE).contains(StateFlags::FOCUSED));

    press(&mut h, KeyCode::Esc);
    assert!(
        !h.is_open(INSPECTOR),
        "a closed modal does not revive on the next update"
    );
    assert!(h.layer_area(INSPECTOR).is_none());
    assert!(
        h.area_of(INSPECTOR_CLOSE).is_none(),
        "closed modal unregisters Close"
    );
    assert_eq!(h.focus(), prior_focus);
    assert_eq!(
        h.runtime().capture_owner(),
        None,
        "final dismissal clears pointer capture"
    );
    assert!(
        h.diagnostics().is_empty(),
        "no diagnostics: {:?}",
        h.diagnostics()
    );
}
