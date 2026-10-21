//! The external-consumer proof.
//!
//! These tests drive the consumer crate exactly like an outside application
//! would: through the public `termrock` facade only. They assert on rendered
//! cells, on the typed actions the consumer produces for keyboard and pointer
//! input, and on the stability of the composed layout.
//!
//! The only geometry in this file is the root terminal area handed to the
//! framework; the consumer application itself never computes a rectangle.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use termrock::{
    Buffer, Input, ItemKey, Key, KeyCode, KeyModifiers, Mouse, MouseKind, Part, PartRef, Position,
    Rect, Runtime, Theme,
};
use termrock_consumer_example::{
    Action, MENU_BAR, NOTE_ITEM, Note, RECIPE_FACTS, RECIPE_LIST, RecipeBox, SCALE_BUTTON,
    STATUS_BAR,
};

/// The root terminal area used by every case.
const SCREEN: Rect = Rect::new(0, 0, 96, 24);

type Consumer = Runtime<RecipeBox>;

fn consumer() -> (Consumer, Terminal<TestBackend>) {
    consumer_with(SCREEN)
}

fn consumer_with(area: Rect) -> (Consumer, Terminal<TestBackend>) {
    let runtime = Runtime::new(RecipeBox::default(), Theme::junie());
    let terminal =
        Terminal::new(TestBackend::new(area.width, area.height)).expect("test terminal starts");
    (runtime, terminal)
}

/// Render the runtime through the public production path.
fn present(runtime: &mut Consumer, terminal: &mut Terminal<TestBackend>) {
    present_in(runtime, terminal, SCREEN);
}

fn present_in(runtime: &mut Consumer, terminal: &mut Terminal<TestBackend>, area: Rect) {
    let prepared = runtime.prepare(area).expect("runtime prepares the frame");
    terminal
        .draw(|frame| {
            runtime
                .draw_prepared(prepared, frame)
                .expect("prepared frame matches the terminal bounds");
        })
        .expect("test backend draws the frame");
    runtime
        .acknowledge_presented(prepared)
        .expect("prepared frame is physically acknowledged");
}

/// Every rendered symbol, row by row.
fn rendered_text(buffer: &Buffer) -> String {
    let area = *buffer.area();
    let mut text = String::new();
    for y in area.y..area.y.saturating_add(area.height) {
        for x in area.x..area.x.saturating_add(area.width) {
            if let Some(cell) = buffer.cell(Position::new(x, y)) {
                text.push_str(cell.symbol());
            }
        }
        text.push('\n');
    }
    text
}

fn press(runtime: &mut Consumer, terminal: &mut Terminal<TestBackend>, code: KeyCode) {
    let _ = runtime
        .handle(Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        }))
        .expect("consumer accepts key input");
    present(runtime, terminal);
}

fn click(runtime: &mut Consumer, terminal: &mut Terminal<TestBackend>, position: Position) {
    for kind in [MouseKind::Down, MouseKind::Up] {
        let _ = runtime
            .handle(Input::Mouse(Mouse {
                kind,
                pos: position,
                mods: KeyModifiers::NONE,
            }))
            .expect("consumer accepts pointer input");
        present(runtime, terminal);
    }
}

#[test]
fn external_consumer_renders_its_own_domain_data_in_a_stable_layout() {
    let (mut runtime, mut terminal) = consumer();
    present(&mut runtime, &mut terminal);
    let text = rendered_text(terminal.backend().buffer());

    // The shell carries a menu bar and a status bar.
    assert!(text.contains("Recipe"), "{text:?}");
    assert!(text.contains("Help"), "{text:?}");
    assert!(text.contains("Recipe Box"), "{text:?}");

    // The consumer's own domain data is on screen.
    assert!(text.contains("Miso Butter Ramen"), "{text:?}");
    assert!(text.contains("Roasted Squash Soup"), "{text:?}");
    assert!(text.contains("Chickpea Stew"), "{text:?}");
    assert!(text.contains("Olive Oil Cake"), "{text:?}");

    // The layout block and the widgets own their geometry.
    let list = runtime
        .area_of(RECIPE_LIST)
        .expect("recipe list owns bounds");
    let facts = runtime
        .area_of(RECIPE_FACTS)
        .expect("recipe facts own bounds");
    let menu = runtime.area_of(MENU_BAR).expect("the menu bar owns bounds");
    let note = runtime
        .area_of_part(STATUS_BAR, PartRef::item(Part::LABEL, NOTE_ITEM))
        .expect("the status bar registers its keyed note");
    assert!(list.right() <= SCREEN.right(), "{list:?}");
    assert!(facts.right() <= SCREEN.right(), "{facts:?}");
    assert!(list.x < facts.x, "{list:?} {facts:?}");
    // The shell header and footer own the outer rows; the master-detail block
    // owns everything between them.
    assert_eq!(menu.y, SCREEN.y, "{menu:?}");
    assert_eq!(note.y, SCREEN.bottom().saturating_sub(1), "{note:?}");
    assert!(list.y > menu.bottom(), "{list:?} {menu:?}");
    assert!(facts.bottom() <= note.y, "{facts:?} {note:?}");
}

#[test]
fn external_consumer_keyboard_input_produces_typed_actions() {
    let (mut runtime, mut terminal) = consumer();
    present(&mut runtime, &mut terminal);
    assert_eq!(runtime.app().last_action(), None);
    assert_eq!(
        runtime.focus(),
        Some(RECIPE_LIST),
        "the app focuses the recipe list when the shelf opens"
    );

    press(&mut runtime, &mut terminal, KeyCode::Down);
    assert_eq!(
        runtime.app().last_action(),
        Some(Action::Focused(ItemKey::num(102))),
        "a cursor move is a typed action"
    );

    press(&mut runtime, &mut terminal, KeyCode::Enter);
    assert_eq!(
        runtime.app().last_action(),
        Some(Action::Chose(ItemKey::num(102))),
        "choosing a row is a typed action"
    );
    assert_eq!(runtime.app().selected(), Some(ItemKey::num(102)));
    assert!(
        runtime
            .app()
            .actions()
            .iter()
            .any(|action| *action == Action::Chose(ItemKey::num(102))),
        "the consumer keeps the typed action history"
    );
}

#[test]
fn external_consumer_pointer_input_produces_typed_actions() {
    let (mut runtime, mut terminal) = consumer();
    present(&mut runtime, &mut terminal);

    let row = runtime
        .area_of_part(RECIPE_LIST, PartRef::item(Part::ROW, ItemKey::num(104)))
        .expect("a live recipe row keeps its hit region");
    click(&mut runtime, &mut terminal, Position::new(row.x, row.y));

    assert_eq!(
        runtime.app().last_action(),
        Some(Action::Chose(ItemKey::num(104))),
        "a pointer activation is a typed action"
    );
    assert_eq!(runtime.app().selected(), Some(ItemKey::num(104)));
}

#[test]
fn external_consumer_summary_button_produces_a_typed_domain_action() {
    let (mut runtime, mut terminal) = consumer();
    present(&mut runtime, &mut terminal);

    press(&mut runtime, &mut terminal, KeyCode::Down);
    press(&mut runtime, &mut terminal, KeyCode::Enter);

    let button = runtime
        .area_of(SCALE_BUTTON)
        .expect("the summary action owns bounds");
    click(
        &mut runtime,
        &mut terminal,
        Position::new(button.x, button.y),
    );

    assert_eq!(
        runtime.app().last_action(),
        Some(Action::Scaled(ItemKey::num(102))),
        "the summary action is a typed domain action"
    );
    assert_eq!(runtime.app().servings(), 3);
    assert_eq!(runtime.app().note(), Note::Scaled);
}

#[test]
fn external_consumer_status_bar_reports_a_typed_action() {
    let (mut runtime, mut terminal) = consumer();
    present(&mut runtime, &mut terminal);

    let note = runtime
        .area_of_part(STATUS_BAR, PartRef::item(Part::LABEL, NOTE_ITEM))
        .expect("the status bar registers its keyed note");
    click(&mut runtime, &mut terminal, Position::new(note.x, note.y));

    assert_eq!(
        runtime.app().last_action(),
        Some(Action::StatusChosen(NOTE_ITEM)),
        "the status bar answers with its own typed action"
    );
}

#[test]
fn external_consumer_collapses_to_one_pane_on_a_narrow_terminal() {
    let narrow = Rect::new(0, 0, 48, 12);
    let (mut runtime, mut terminal) = consumer_with(narrow);
    present_in(&mut runtime, &mut terminal, narrow);

    let list = runtime
        .area_of(RECIPE_LIST)
        .expect("the recipe list keeps its bounds when the layout collapses");
    assert!(list.right() <= narrow.right(), "{list:?}");
    assert!(
        runtime.area_of(RECIPE_FACTS).is_none(),
        "the layout block collapses the detail pane on a narrow terminal"
    );
    let text = rendered_text(terminal.backend().buffer());
    assert!(text.contains("Miso Butter Ramen"), "{text:?}");
}

#[test]
fn external_consumer_renders_identically_on_two_runs() {
    let (mut first, mut first_terminal) = consumer();
    present(&mut first, &mut first_terminal);
    let (mut second, mut second_terminal) = consumer();
    present(&mut second, &mut second_terminal);

    let left = rendered_text(first_terminal.backend().buffer());
    let right = rendered_text(second_terminal.backend().buffer());
    assert_eq!(
        left, right,
        "two runs of the consumer render the same frame"
    );

    let area = *first_terminal.backend().buffer().area();
    for y in area.y..area.y.saturating_add(area.height) {
        for x in area.x..area.x.saturating_add(area.width) {
            let position = Position::new(x, y);
            let left_cell = first_terminal
                .backend()
                .buffer()
                .cell(position)
                .expect("cell in frame");
            let right_cell = second_terminal
                .backend()
                .buffer()
                .cell(position)
                .expect("cell in frame");
            assert_eq!(
                left_cell.symbol(),
                right_cell.symbol(),
                "cell at {x},{y} differs between runs"
            );
        }
    }
}
