use junie_tui::core::{
    event::{Input, Key, Mouse, MouseKind, Outcome},
    id::WidgetId,
};
use junie_tui::theme::Theme;
use ratatui::{
    Terminal,
    backend::TestBackend,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::{Position, Rect},
};

use crate::app::{App, NAV_ENTRIES, PageId};

const NAV: WidgetId = WidgetId::of("app.nav");
const HEADER_HELP: WidgetId = WidgetId::of("app.header.help");
const HEADER_INSPECT: WidgetId = WidgetId::of("app.header.inspect");

struct Harness {
    app: App,
    term: Terminal<TestBackend>,
}

impl Harness {
    fn new(width: u16, height: u16, page: PageId) -> Self {
        let mut app = App::new(Theme::junie());
        app.goto(page);
        let mut term = Terminal::new(TestBackend::new(width, height)).unwrap();
        term.draw(|frame| app.render(frame)).unwrap();
        Self { app, term }
    }

    fn draw(&mut self) {
        self.term.draw(|frame| self.app.render(frame)).unwrap();
    }

    fn send(&mut self, input: Input) -> Outcome {
        let outcome = self.app.handle(input);
        self.draw();
        outcome
    }

    fn key(&mut self, code: KeyCode) -> Outcome {
        self.send(Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        }))
    }

    fn key_mod(&mut self, code: KeyCode, mods: KeyModifiers) -> Outcome {
        self.send(Input::Key(Key { code, mods }))
    }

    fn type_str(&mut self, value: &str) {
        for ch in value.chars() {
            self.key(KeyCode::Char(ch));
        }
    }

    fn paste(&mut self, value: &str) -> Outcome {
        self.send(Input::Paste(value.to_owned()))
    }

    fn tick(&mut self) -> Outcome {
        self.send(Input::Tick)
    }

    fn mouse(&mut self, kind: MouseKind, x: u16, y: u16) -> Outcome {
        self.send(Input::Mouse(Mouse {
            kind,
            pos: Position::new(x, y),
        }))
    }

    fn click(&mut self, x: u16, y: u16) {
        self.mouse(MouseKind::Down, x, y);
        self.mouse(MouseKind::Up, x, y);
    }

    fn area(&self, id: WidgetId) -> Rect {
        self.app
            .hits
            .area_of(id)
            .unwrap_or_else(|| panic!("missing hit area for {id:?}\n{}", self.text()))
    }

    fn click_id(&mut self, id: WidgetId) {
        let area = self.area(id);
        let x = area.x + area.width.saturating_sub(1) / 2;
        let y = area.y + area.height.saturating_sub(1) / 2;
        self.click(x, y);
    }

    fn focus(&mut self, id: WidgetId) {
        self.app.focus.focus(id);
        self.draw();
    }

    fn row(&self, y: u16) -> String {
        let buffer = self.term.backend().buffer();
        (0..buffer.area.width)
            .map(|x| buffer[(x, y)].symbol().to_owned())
            .collect()
    }

    fn text(&self) -> String {
        let buffer = self.term.backend().buffer();
        (0..buffer.area.height)
            .map(|y| self.row(y))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn find(&self, needle: &str) -> Option<(u16, u16)> {
        let buffer = self.term.backend().buffer();
        for y in 0..buffer.area.height {
            let row = self.row(y);
            if let Some(x) = row.find(needle) {
                return Some((x as u16, y));
            }
        }
        None
    }

    fn click_label(&mut self, needle: &str) {
        let (x, y) = self
            .find(needle)
            .unwrap_or_else(|| panic!("missing label {needle:?}"));
        self.click(x, y);
    }

    fn status(&self) -> Option<String> {
        self.app.status.as_ref().map(|(value, _)| value.clone())
    }

    fn assert_status(&self, expected: &str) {
        assert_eq!(self.status().as_deref(), Some(expected));
    }

    fn assert_status_prefix(&self, expected: &str) {
        assert!(
            self.status()
                .as_deref()
                .is_some_and(|value| value.starts_with(expected)),
            "status {:?} does not start with {expected:?}",
            self.status()
        );
    }
}

#[test]
fn too_small_q_quits() {
    let mut h = Harness::new(60, 15, PageId::Buttons);

    assert_eq!(h.key(KeyCode::Char('q')), Outcome::Consumed);
    assert!(h.app.quit);
}

#[test]
fn header_help_and_inspector_are_mouse_actions() {
    let mut h = Harness::new(120, 40, PageId::Overview);

    h.click_id(HEADER_HELP);
    assert!(h.app.dialog.is_some());
    assert!(h.text().contains("Keyboard & mouse"));

    h.click_label("Close");
    assert!(h.app.dialog.is_none());

    h.click_id(HEADER_INSPECT);
    assert!(h.app.inspector);
}

#[test]
fn chrome_menus_context_and_status_actions_are_reachable() {
    let mut h = Harness::new(140, 45, PageId::Chrome);

    h.click_label("Weekly 59%");
    h.assert_status("Usage chip");
    h.click_label("PR #482");
    h.assert_status("PR context");

    let (x, y) = h.find("1 Claude Code").expect("session row");
    h.click(x, y);
    h.key(KeyCode::F(10));
    h.click_label("New tab");
    h.assert_status("New tab");

    h.click(x, y);
    h.key(KeyCode::Char('m'));
    assert!(h.text().contains("Change title"));
    h.key(KeyCode::Esc);
    assert!(!h.text().contains("Change title"));

    h.click_label("View");
    h.click_label("Zoom pane");
    h.assert_status("Zoom pane");
    assert!(h.text().contains("ZOOM"));
}

#[test]
fn navigation_edges_and_page_entry_are_deterministic() {
    let mut h = Harness::new(72, 20, PageId::Overview);

    h.key(KeyCode::Char('0'));
    assert_eq!(h.app.focus.current(), Some(NAV));

    h.key(KeyCode::PageDown);
    assert!(h.app.nav_cursor > 0);
    h.key(KeyCode::PageUp);
    assert_eq!(h.app.nav_cursor, 0);

    h.key(KeyCode::End);
    assert_eq!(h.app.nav_cursor, NAV_ENTRIES.len() - 1);
    h.key(KeyCode::Char('k'));
    assert_eq!(h.app.nav_cursor, NAV_ENTRIES.len() - 2);
    h.key(KeyCode::Char('j'));
    assert_eq!(h.app.nav_cursor, NAV_ENTRIES.len() - 1);
    h.key(KeyCode::Home);
    assert_eq!(h.app.nav_cursor, 0);

    h.key(KeyCode::Right);
    assert_eq!(h.app.page, PageId::Overview);
}

#[test]
fn buttons_activate_actions_and_toggles() {
    let mut h = Harness::new(120, 40, PageId::Buttons);

    for (label, status) in [
        ("Run task", "Run task ✓"),
        ("Preview", "Preview ✓"),
        ("Cancel", "Cancel ✓"),
        ("Delete branch", "Delete branch ✓"),
    ] {
        h.click_label(label);
        h.assert_status(status);
    }

    h.click_label("Auto-approve");
    h.assert_status("Auto-approve on");
    h.click_label("Verbose");
    h.assert_status("Verbose off");

    h.click_label("Start long job");
    h.assert_status("Working…");
    assert!(h.app.pages[PageId::Buttons.index()].animating());
    for _ in 0..28 {
        h.tick();
    }
    assert!(!h.app.pages[PageId::Buttons.index()].animating());
    h.assert_status("Long job finished ✓");
    assert!(h.text().contains("Long job finished ✓"));
}

#[test]
fn chips_selection_and_disabled_behavior() {
    let mut h = Harness::new(120, 40, PageId::Chips);
    let chips = WidgetId::of("chips");
    let filters = chips.sub("filters");

    h.focus(filters);
    h.key(KeyCode::Right);
    h.key(KeyCode::Right);
    h.key(KeyCode::Char(' '));
    assert!(
        h.text()
            .contains("last action: enabled country in (DE, FR)")
    );
    h.key(KeyCode::Left);
    h.key(KeyCode::Left);
    h.key(KeyCode::Enter);
    h.assert_status("Would open the editor for status = 'pending'");

    h.key(KeyCode::Char(' '));
    assert!(
        h.text()
            .contains("last action: disabled status = 'pending'")
    );
    h.key(KeyCode::Char('x'));
    assert!(h.text().contains("last action: removed status = 'pending'"));

    h.key(KeyCode::Right);
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert!(
        h.text()
            .contains("last action: added created_at > '2026-01-01'")
    );

    h.click_id(filters.sub("lead"));
    assert!(h.text().contains("last action: match any"));

    h.key(KeyCode::Char('X'));
    assert!(h.text().contains("last action: cleared all filters"));

    h.focus(chips.sub("sort"));
    h.key(KeyCode::Enter);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    h.assert_status("Sort by → total");

    h.focus(chips.sub("size"));
    h.key(KeyCode::Enter);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    h.assert_status("Page size → 100");

    h.focus(chips.sub("engine"));
    let before = h.status();
    h.key(KeyCode::Enter);
    assert_eq!(h.status(), before);
}

#[test]
fn dialogs_cover_choice_destructive_and_prompt_validation() {
    let mut h = Harness::new(120, 40, PageId::Dialogs);

    h.click_label("Three choices");
    assert!(h.app.dialog.is_some());
    h.key(KeyCode::Left);
    h.key(KeyCode::Enter);
    h.assert_status("Changes discarded");

    h.click_label("Three choices");
    h.key(KeyCode::Esc);
    h.assert_status("Cancelled");

    h.click_label("Three choices");
    h.key(KeyCode::Enter);
    h.assert_status("Description saved");

    h.key(KeyCode::Char('d'));
    assert!(h.app.dialog.is_some());
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    h.assert_status("Branch feat/rate-limit deleted");

    h.click_label("Rename task");
    h.key(KeyCode::Enter);
    h.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
    h.type_str(&"x".repeat(41));
    h.key(KeyCode::Enter);
    assert!(h.app.dialog.is_some());
    assert!(h.text().contains("Keep it under 40"));

    h.key(KeyCode::Enter);
    h.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
    h.type_str("Ship it");
    h.key(KeyCode::Enter);
    h.assert_status("Renamed to “Ship it”");
}

#[test]
fn forms_reset_controls_and_reviewer_paste_validation() {
    let mut h = Harness::new(120, 40, PageId::Forms);
    let forms = WidgetId::of("forms");

    let thorough = forms.sub("mode").child(2);
    h.click_id(thorough);
    assert!(h.row(h.area(thorough).y).contains("Thorough"));
    assert!(h.row(h.area(thorough).y).contains("●"));

    h.click_label("Run tests");
    h.click_label("Open a pull request");
    h.click_label("Auto-approve changes");
    let auto_y = h.find("Auto-approve changes").expect("auto-approve row").1;
    assert!(h.row(auto_y).contains("on"));
    assert!(h.row(auto_y).contains("●"));
    let notify_y = h.find("Notify on completion").expect("notify row").1;
    let notify_row = h.row(notify_y);
    h.click_label("Notify on completion");
    assert_eq!(h.row(notify_y), notify_row);

    h.click_label("Reset");
    h.assert_status("Form reset");
    let tests_y = h.find("Run tests").expect("run tests row").1;
    let pr_y = h.find("Open a pull request").expect("open pr row").1;
    assert!(h.row(tests_y).contains("[✓]"));
    assert!(h.row(pr_y).contains("[ ]"));

    h.click_label("Short imperative summary");
    h.paste("Build task");
    h.key(KeyCode::Enter);

    let reviewer = forms.sub("reviewer");
    h.click_id(reviewer);
    h.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
    h.paste("bad");
    h.key(KeyCode::Enter);
    h.key_mod(KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert!(h.text().contains("Enter a valid email address"));
    assert_eq!(h.app.focus.current(), Some(reviewer));

    h.click_id(reviewer);
    h.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
    h.paste("reviewer@example.com");
    h.key(KeyCode::Enter);
    h.key_mod(KeyCode::Char('s'), KeyModifiers::CONTROL);
    h.assert_status("Creating task…");
    assert!(h.app.pages[PageId::Forms.index()].animating());
    for _ in 0..23 {
        h.tick();
    }
    assert!(!h.app.pages[PageId::Forms.index()].animating());
    h.assert_status("Task created ✓");
    assert!(h.text().contains("Task created ✓"));
}

#[test]
fn inputs_paste_validation_and_disabled_field() {
    let mut h = Harness::new(120, 40, PageId::Inputs);
    let inputs = WidgetId::of("inputs");
    let owner = inputs.child(2);
    let branch = inputs.child(1);
    let api_token = inputs.child(3);

    h.click_id(owner);
    h.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
    h.paste("bad");
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Enter a valid email address"));

    h.click_id(branch);
    h.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
    h.paste("feat/payments");
    h.key(KeyCode::Enter);
    h.assert_status("Branch saved");

    h.click_id(api_token);
    assert!(!h.app.pages[PageId::Inputs.index()].editing());
}

#[test]
fn pickers_cancel_scope_and_secondary_action() {
    let mut h = Harness::new(120, 40, PageId::Pickers);
    let pickers = WidgetId::of("pickers");
    let quick = pickers.sub("quick");
    let quick_picker = pickers.sub("picker.quick");

    h.focus(quick);
    h.key(KeyCode::Enter);
    h.type_str("auth");
    assert!(h.text().contains("auth.rs"));
    h.key(KeyCode::Tab);
    assert!(h.text().contains("Files"));
    h.key_mod(KeyCode::Enter, KeyModifiers::ALT);
    h.assert_status_prefix("Opened in a new tab:");

    h.focus(quick);
    h.key(KeyCode::Enter);
    h.type_str("auth");
    h.key(KeyCode::Esc);
    assert!(h.app.hits.area_of(quick_picker).is_some());
    h.key(KeyCode::Esc);
    assert!(h.app.hits.area_of(quick_picker).is_none());

    h.focus(pickers.sub("tabs"));
    h.key(KeyCode::Enter);
    h.key(KeyCode::Delete);
    h.assert_status_prefix("Closed ");
}

#[test]
fn grid_pending_save_and_discard_lifecycle() {
    let mut h = Harness::new(160, 50, PageId::Grid);
    let grid = WidgetId::of("grid").sub("grid");

    h.focus(grid);
    h.key(KeyCode::Right);
    h.key(KeyCode::Right);
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    h.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
    h.type_str("12");
    h.key(KeyCode::Enter);
    h.assert_status("1 pending");

    h.key_mod(KeyCode::Char('s'), KeyModifiers::CONTROL);
    h.assert_status("Saving…");
    for _ in 0..4 {
        h.tick();
    }
    h.assert_status("Saved 1 changes");

    h.key(KeyCode::Char('+'));
    h.assert_status_prefix("Row inserted");
    h.key(KeyCode::Char('-'));
    h.assert_status_prefix("Row queued for deletion");
    h.key(KeyCode::Char('U'));
    h.assert_status("Changes discarded");
}

#[test]
fn grid_fetch_refresh_and_viewer_paths_are_reachable() {
    let mut h = Harness::new(160, 50, PageId::Grid);
    let grid = WidgetId::of("grid").sub("grid");

    h.focus(grid);
    h.key_mod(KeyCode::End, KeyModifiers::CONTROL);
    h.key(KeyCode::Enter);
    h.assert_status("Fetched rows 41–80");

    h.key(KeyCode::Char('r'));
    h.assert_status("Reloaded from the source");

    h.key_mod(KeyCode::Home, KeyModifiers::CONTROL);
    for _ in 0..7 {
        h.key(KeyCode::Right);
    }
    h.key(KeyCode::Enter);
    h.assert_status("Would open the viewer for notes on row 1");
}

#[test]
fn settings_save_cancel_and_environment_remove() {
    let mut h = Harness::new(140, 45, PageId::Settings);
    let settings = WidgetId::of("settings");
    let save = settings.sub("save");

    h.click_id(settings.sub("vis").child(2));
    h.click_id(settings.sub("automerge"));
    assert!(h.text().contains("General · unsaved"));

    h.click_id(save);
    assert!(h.app.dialog.is_some());
    h.key(KeyCode::Esc);
    assert!(h.app.dialog.is_none());
    assert!(h.text().contains("General · unsaved"));

    h.click_id(save);
    h.key(KeyCode::Enter);
    h.assert_status("Settings saved ✓");
    assert!(h.text().contains("General"));
    assert!(!h.text().contains("General · unsaved"));

    let tabs = settings.sub("tabs");
    h.focus(tabs);
    h.key(KeyCode::Right);
    h.key(KeyCode::Right);
    h.focus(settings.sub("env"));
    h.key(KeyCode::Char('a'));
    h.click_id(settings.sub("rmvars"));
    h.assert_status_prefix("Removed ");
}

#[test]
fn progress_pause_freezes_tick_until_resumed() {
    let mut h = Harness::new(120, 40, PageId::Progress);
    let progress = WidgetId::of("progress");
    let pause = progress.sub("pause");
    let building_y = h.find("Building").expect("building row").1;

    h.click_id(pause);
    assert!(h.text().contains("Resume"));
    let paused_row = h.row(building_y);
    h.tick();
    assert_eq!(h.row(building_y), paused_row);

    h.click_id(pause);
    h.tick();
    assert_ne!(h.row(building_y), paused_row);

    h.click_id(progress.sub("restart"));
    assert!(h.row(building_y).contains("0%"));
}

#[test]
fn taskrunner_reports_immediate_running_and_cancel() {
    let mut h = Harness::new(140, 45, PageId::TaskRunner);
    let runner = WidgetId::of("taskrunner");

    h.click_id(runner.sub("run"));
    h.assert_status("Pipeline running");
    assert!(h.app.animating());

    h.tick();
    h.click_id(runner.sub("cancel"));
    assert!(h.app.dialog.is_some());
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    h.assert_status("Cancelled");
    assert!(!h.app.pages[PageId::TaskRunner.index()].animating());
}

#[test]
fn taskrunner_reaches_failure_completion_and_reports_it() {
    let mut h = Harness::new(140, 45, PageId::TaskRunner);
    h.click_label("Run pipeline");
    for _ in 0..300 {
        h.tick();
    }
    assert!(!h.app.animating());
    assert!(h.text().contains("Pipeline finished with 1 failure"));
    assert!(h.text().contains("integration failed"));
    assert!(h.status().is_some_and(|s| s.contains("1 task failed")));
}

#[test]
fn terminal_selection_and_status_actions() {
    let mut h = Harness::new(120, 40, PageId::Terminal);
    let term = h.area(WidgetId::of("terminal").sub("term"));
    let start_x = term.x + 2;
    let end_x = (term.x + term.width.saturating_sub(2)).max(start_x);
    let y = term.y + 1;

    h.mouse(MouseKind::Down, start_x, y);
    h.mouse(MouseKind::Drag, end_x, y);
    h.mouse(MouseKind::Up, end_x, y);
    assert!(h.text().contains("selection · y copies"));

    h.key(KeyCode::Char('y'));
    h.assert_status_prefix("Copied ");
    h.key(KeyCode::Char('f'));
    h.assert_status("Paused at the scrollback position");
    h.key(KeyCode::End);
    h.assert_status("Following the tail");
}

#[test]
fn editor_completion_paste_and_diagnostic_paths() {
    let mut h = Harness::new(140, 45, PageId::Editor);

    h.key(KeyCode::Tab);
    h.key(KeyCode::Char('i'));
    h.type_str("cl");
    assert!(h.text().contains("Client"));
    h.key(KeyCode::Enter);
    assert!(
        h.app
            .hits
            .area_of(WidgetId::of("editor").sub("complete").child(0))
            .is_none()
    );

    h.paste(" pasted");
    assert!(h.text().contains("pasted"));

    let mut diagnostic = Harness::new(140, 45, PageId::Editor);
    diagnostic.key(KeyCode::Tab);
    diagnostic.key(KeyCode::Char('}'));
    diagnostic.key_mod(KeyCode::Char('r'), KeyModifiers::CONTROL);
    for _ in 0..10 {
        diagnostic.tick();
    }
    diagnostic.assert_status("Block ran in 77 ms");
    assert!(diagnostic.text().contains("unwrap() panics on Err"));
}

#[test]
fn chrome_secondary_click_opens_and_selects_context_action() {
    let mut h = Harness::new(140, 45, PageId::Chrome);
    let sessions = WidgetId::of("chrome.sessions");
    let (x, y) = h.find("2 Codex (Primary)").expect("session row");

    assert_eq!(
        h.mouse(MouseKind::Secondary, x, y),
        Outcome::Changed,
        "secondary click is routed to the session row"
    );
    assert_eq!(h.app.focus.current(), Some(sessions));
    assert!(h.text().contains("Change title"));

    h.key(KeyCode::Enter);
    h.assert_status("Change title…");
    assert!(h.app.hits.area_of(WidgetId::of("chrome.context")).is_none());
}

#[test]
fn panels_route_wheel_and_nested_list_clicks() {
    let mut h = Harness::new(140, 45, PageId::Panels);
    let prose = WidgetId::of("panels").sub("prose");
    let before = h.text();
    let area = h.area(prose);
    assert_eq!(
        h.mouse(MouseKind::WheelDown, area.x + 2, area.y + 1),
        Outcome::Changed
    );
    assert_ne!(h.text(), before, "prose content moved under the wheel");

    let nested = WidgetId::of("panels").sub("nested");
    h.click_label("CLI");
    assert_eq!(h.app.focus.current(), Some(nested));
    assert!(h.text().contains("CLI"));

    let log = WidgetId::of("panels").sub("log");
    let log_area = h.area(log);
    let before = h.text();
    h.mouse(MouseKind::WheelDown, log_area.x + 2, log_area.y + 1);
    assert_ne!(h.text(), before, "log content moved under the wheel");
}

#[test]
fn sidebars_collapse_round_trips_and_disabled_rows_do_nothing() {
    let mut h = Harness::new(120, 40, PageId::Sidebars);
    let collapse = WidgetId::of("sidebars").sub("collapse");
    let button_y = h.area(collapse).y;
    let expanded = h.row(button_y);

    h.click_id(collapse);
    let collapsed = h.row(button_y);
    assert_ne!(collapsed, expanded);
    assert!(collapsed.contains("›"), "collapsed control is rendered");

    h.key(KeyCode::Enter);
    assert!(h.row(button_y).contains("Collapse"));
    assert!(!h.row(button_y).contains("›"));

    let mut disabled = Harness::new(120, 40, PageId::Sidebars);
    disabled.key(KeyCode::Tab);
    let nav = WidgetId::of("sidebars").sub("nav");
    let (x, y) = disabled.find("Billing").expect("disabled row");
    disabled.click(x, y);
    assert_eq!(disabled.app.focus.current(), Some(nav));
    assert!(
        disabled.text().contains("Tasks"),
        "current item stayed stable"
    );
}

#[test]
fn textareas_paste_and_disabled_state_are_routed() {
    let mut h = Harness::new(120, 40, PageId::TextAreas);
    let first = WidgetId::of("textareas").child(0);
    h.focus(first);
    h.key(KeyCode::Enter);
    assert!(h.app.pages[PageId::TextAreas.index()].editing());
    h.paste("pasted");
    assert!(h.text().contains("pasted"));
    h.key(KeyCode::Esc);
    assert!(!h.app.pages[PageId::TextAreas.index()].editing());

    let mut disabled = Harness::new(120, 40, PageId::TextAreas);
    let transcript = WidgetId::of("textareas").child(2);
    disabled.focus(transcript);
    disabled.key(KeyCode::Enter);
    assert!(!disabled.app.pages[PageId::TextAreas.index()].editing());
    assert!(disabled.text().contains("Read-only transcript"));
}

#[test]
fn lists_and_trees_accept_mouse_selection() {
    let mut lists = Harness::new(120, 40, PageId::Lists);
    let (x, y) = lists.find("TypeScript").expect("language row");
    lists.click(x, y);
    assert_eq!(
        lists.app.focus.current(),
        Some(WidgetId::of("lists").sub("single"))
    );
    assert!(lists.text().contains("Chosen: TypeScript"));

    let mut trees = Harness::new(120, 40, PageId::Trees);
    trees.key(KeyCode::Tab);
    trees.key(KeyCode::Left);
    trees.key(KeyCode::Right);
    trees.key(KeyCode::Down);
    trees.key(KeyCode::Right);
    let (x, y) = trees.find("auth.rs").expect("tree leaf");
    trees.click(x, y);
    assert_eq!(
        trees.app.focus.current(),
        Some(WidgetId::of("trees").sub("tree"))
    );
    assert!(trees.text().contains("src/api/auth.rs"));
}

#[test]
fn diff_router_covers_review_empty_and_copy_states() {
    let mut h = Harness::new(140, 45, PageId::Diff);
    let diff = WidgetId::of("diff");
    h.focus(diff.sub("review"));
    h.key(KeyCode::Enter);
    assert!(h.text().contains("● Review"));

    h.focus(diff.sub("empty"));
    h.key(KeyCode::Enter);
    assert!(h.text().contains("No file selected"));

    let mut selected = Harness::new(140, 45, PageId::Diff);
    selected.focus(diff.sub("view"));
    let (x, y) = selected.find("attempts = 3").expect("diff line");
    selected.mouse(MouseKind::Down, x, y);
    selected.mouse(MouseKind::Drag, x + 10, y);
    selected.mouse(MouseKind::Up, x + 10, y);
    selected.key(KeyCode::Char('y'));
    selected.assert_status("Selection copied in demo");
}

#[test]
fn picker_level_selection_updates_result_state() {
    let mut h = Harness::new(140, 45, PageId::Pickers);
    let pickers = WidgetId::of("pickers");
    let level = pickers.sub("level");
    h.focus(level);
    h.key(KeyCode::Enter);
    assert!(h.app.hits.area_of(pickers.sub("picker.level")).is_some());
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    h.assert_status("Chose Safe Mode (Full)");
    assert!(h.text().contains("Safe Mode (Full)"));
    assert!(h.app.hits.area_of(pickers.sub("picker.level")).is_none());
}

#[test]
fn grid_validates_copies_filters_previews_and_reports_server_errors() {
    let mut h = Harness::new(160, 50, PageId::Grid);
    let grid = WidgetId::of("grid").sub("grid");
    h.focus(grid);
    h.key(KeyCode::Right);
    h.key(KeyCode::Right);
    h.key(KeyCode::Right);

    h.key(KeyCode::Enter);
    h.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
    h.type_str("not-a-number");
    h.key(KeyCode::Enter);
    assert!(h.text().contains("EDIT"));
    assert!(
        h.text().contains("!"),
        "the invalid cell renders an error marker"
    );
    assert!(h.app.pages[PageId::Grid.index()].editing());
    h.key(KeyCode::Esc);

    h.key(KeyCode::Char('y'));
    h.assert_status_prefix("Copied ");
    h.key(KeyCode::Char('f'));
    h.assert_status_prefix("Would filter seats = ");
    h.key(KeyCode::Char('/'));
    h.assert_status("The filter editor belongs to the app");
    h.key(KeyCode::Char('F'));
    h.assert_status("No filters to clear");

    h.key(KeyCode::Enter);
    h.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
    h.type_str("600");
    h.key(KeyCode::Enter);
    h.key_mod(KeyCode::Char('s'), KeyModifiers::CONTROL);
    for _ in 0..4 {
        h.tick();
    }
    assert!(h.text().contains("Save failed"));
    assert!(
        h.text().contains("!"),
        "the rejected row renders an error marker"
    );

    h.key(KeyCode::Char('p'));
    assert!(h.text().contains("Pending changes"));
    h.click_label("Copy SQL");
    h.assert_status_prefix("Copied ");
}

#[test]
fn terminal_reaches_success_failure_and_splitter_states() {
    let mut success = Harness::new(140, 45, PageId::Terminal);
    assert!(success.app.animating());
    for _ in 0..130 {
        success.tick();
    }
    assert!(!success.app.animating());
    assert!(success.text().contains("7 of 7"));
    assert!(success.text().contains("Ready"));

    let mut failure = Harness::new(140, 45, PageId::Terminal);
    let term = WidgetId::of("terminal").sub("term");
    let before = failure.area(term).width;
    let seam = failure.area(WidgetId::of("terminal.seam"));
    failure.mouse(MouseKind::Down, seam.x, seam.y + 2);
    failure.mouse(MouseKind::Drag, seam.x + 8, seam.y + 2);
    failure.mouse(MouseKind::Up, seam.x + 8, seam.y + 2);
    assert_ne!(
        failure.area(term).width,
        before,
        "splitter drag resizes viewport"
    );

    let fail = WidgetId::of("terminal").sub("fail");
    failure.focus(fail);
    failure.key(KeyCode::Enter);
    for _ in 0..50 {
        failure.tick();
    }
    assert!(
        failure.text().contains("failed: network unreachable"),
        "animating={} focus={:?}\n{}",
        failure.app.animating(),
        failure.app.focus.current(),
        failure.text()
    );
    assert!(!failure.app.pages[PageId::Terminal.index()].animating());
    assert!(failure.text().contains("failed"));
}
