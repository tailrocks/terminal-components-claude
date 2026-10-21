//! The showcase pages paint their content from live widgets.
//!
//! These tests guard the two properties the showcase lost when a page pasted
//! frozen cells: no page holds a table of frozen cells, and no page
//! picks a different screen for an exact page size.

use showcase_demos::pages::Page;
use termrock::{App, Cx, Response, Theme, Ui};
use termrock_test_support::Harness;

const CHROME: &str = include_str!("../src/pages/chrome.rs");
const SELECTABLE: &str = include_str!("../src/pages/selectable.rs");
const TASKRUNNER: &str = include_str!("../src/pages/taskrunner.rs");
const TERMINAL: &str = include_str!("../src/pages/terminal.rs");

#[test]
fn no_page_holds_a_captured_cell_table() {
    for (name, source) in [
        ("chrome", CHROME),
        ("selectable", SELECTABLE),
        ("taskrunner", TASKRUNNER),
        ("terminal", TERMINAL),
    ] {
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
    }
}

#[test]
fn no_page_switches_on_an_exact_page_size() {
    for (name, source) in [
        ("chrome", CHROME),
        ("selectable", SELECTABLE),
        ("taskrunner", TASKRUNNER),
        ("terminal", TERMINAL),
    ] {
        assert!(
            !source.contains("area.width =="),
            "{name}: a page still picks a screen by its exact width"
        );
        assert!(
            !source.contains("area.height =="),
            "{name}: a page still picks a screen by its exact height"
        );
    }
}

#[test]
fn no_page_replays_a_historical_frame() {
    for (name, source) in [
        ("chrome", CHROME),
        ("selectable", SELECTABLE),
        ("taskrunner", TASKRUNNER),
        ("terminal", TERMINAL),
    ] {
        assert!(
            !source.contains("historical"),
            "{name}: a historical replay is back"
        );
        assert!(
            !source.contains("paint_historical"),
            "{name}: a historical replay is back"
        );
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
        for needle in ["Selection", "Dana Whitfield", "Owner"] {
            assert!(text.contains(needle), "{width}x{height}: {needle} missing");
        }
    }
}

