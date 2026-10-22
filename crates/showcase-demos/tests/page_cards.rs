//! The showcase cards keep the approved layout.
//!
//! These tests drive the real page modules: a card paints a secondary title
//! with the frozen seam under it, the step rail keeps its title and its
//! meta apart, and the page footer never lands on a panel border.

use showcase_demos::pages::Page;
use showcase_demos::pages::{taskrunner, terminal};
use termrock::{App, Color, Cx, Modifier, Response, Theme, Ui};
use termrock_test_support::Harness;

struct PageHost(Box<dyn Page>);

impl App for PageHost {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.0.draw(ui, ui.full());
    }
}

fn harness(page: Box<dyn Page>, width: u16, height: u16) -> Harness<PageHost> {
    Harness::new(PageHost(page), Theme::junie(), width, height)
}

const SECONDARY: Color = Color::Rgb(179, 179, 179);
const SEAM: Color = Color::Rgb(58, 58, 58);

/// The targets card paints a secondary title and the frozen seam under it.
#[test]
fn the_task_runner_card_paints_a_seam_under_a_secondary_title() {
    let harness = harness(Box::new(taskrunner::TaskRunnerPage::new()), 120, 40);
    let (title_x, title_y) = harness.find("Targets").expect("the targets card title");
    let title = harness.cell(title_x, title_y);
    assert_eq!(
        title.fg, SECONDARY,
        "the card title is secondary ink, never bold white"
    );
    assert!(
        !title.modifier.contains(Modifier::BOLD),
        "the card title is never bold"
    );
    let seam = harness.cell(title_x, title_y + 1);
    assert_eq!(seam.symbol(), "─", "the card paints a seam under its title");
    assert_eq!(seam.fg, SEAM, "the seam keeps the approved ink");
}

/// The pipeline card's meta rides the title row, clear of the title.
#[test]
fn the_task_runner_card_keeps_its_meta_clear_of_its_title() {
    let harness = harness(Box::new(taskrunner::TaskRunnerPage::new()), 120, 40);
    let (title_x, title_y) = harness.find("Targets").expect("the targets card title");
    let (meta_x, meta_y) = harness.find("of 6 done").expect("the pipeline card meta");
    assert_eq!(meta_y, title_y, "the meta rides the card's title row");
    assert!(
        meta_x > title_x + "Targets".len() as u16,
        "the meta never lands on the title"
    );
}

/// The page footer keeps its own rows: it never lands on a panel border.
#[test]
fn the_terminal_footer_never_lands_on_a_panel_border() {
    let harness = harness(Box::new(terminal::TerminalPage::new()), 120, 40);
    let (footer_x, footer_y) = harness.find("status: ready").expect("the page footer");
    let row = harness.row(footer_y);
    assert!(
        !row.contains('╰') && !row.contains('│'),
        "the footer row carries no panel border: {row:?}"
    );
    assert_eq!(footer_x, 0, "the footer starts at the page's left edge");
}

/// The step durations ride the step rows: the page paints no duration over
/// the rail.
#[test]
fn the_terminal_step_durations_ride_the_step_rows() {
    let harness = harness(Box::new(terminal::TerminalPage::new()), 120, 40);
    let (duration_x, duration_y) = harness.find("0.8 s").expect("the first duration");
    let row = harness.row(duration_y);
    assert!(
        row[..duration_x as usize].contains("01 Resolve workspace"),
        "the duration rides its own step row: {row:?}"
    );
}
