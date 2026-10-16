//! Pointer text selection in jackin's fields through the real app: click
//! counts, press and drag reach the editor's fields and the fields of its
//! dialogs (interaction #16).

use junie_tui::core::event::MouseKind;
use junie_tui::widgets::dialog::DialogBody;
use ratatui::crossterm::event::KeyCode;

use crate::app::Route;
use crate::app_tests::H;
use crate::scenario::{Motion, Scenario};
use crate::screens::Modal;

fn editor() -> H {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::Down);
    h.key(KeyCode::Char('e'));
    assert_eq!(h.app.route, Route::Editor, "{}", h.text());
    h
}

fn name_selection(h: &H) -> Option<String> {
    let e = h.app.screens.editor.as_ref().expect("the editor");
    e.name.buffer.selected_text().map(str::to_owned)
}

#[test]
fn dragging_across_the_editor_name_selects_it() {
    let mut h = editor();
    // the value inside the name field as drawn
    let field = h
        .app
        .hits
        .area_of(crate::screens::editor::NAME)
        .expect("the name field");
    let text = h.text();
    let (x, y) = (field.y..field.bottom())
        .find_map(|y| {
            let line: Vec<char> = text.lines().nth(y as usize)?.chars().collect();
            let want: Vec<char> = "payments-platform".chars().collect();
            (field.x as usize..line.len().saturating_sub(want.len()))
                .find(|&x| line[x..x + want.len()] == want[..])
                .map(|x| (x as u16, y))
        })
        .expect("the name value");
    h.mouse(MouseKind::Down, x, y);
    h.mouse(MouseKind::Drag, x + 3, y);
    h.mouse(MouseKind::Up, x + 3, y);
    assert_eq!(name_selection(&h).as_deref(), Some("pay"), "{}", h.text());
    // a triple click selects the whole value
    h.click(x + 1, y);
    h.click(x + 1, y);
    h.click(x + 1, y);
    assert_eq!(
        name_selection(&h).as_deref(),
        Some("payments-platform"),
        "{}",
        h.text()
    );
}

fn rename_input(h: &H) -> Option<String> {
    match h.app.top_modal() {
        Some(Modal::Dialog(d)) => match &d.body {
            DialogBody::Input(i) => i.buffer.selected_text().map(str::to_owned),
            _ => panic!("a prompt"),
        },
        _ => panic!("the rename prompt is open"),
    }
}

#[test]
fn a_double_click_in_a_prompt_selects_the_word_and_a_drag_extends_by_words() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    h.ctrl('b');
    h.key(KeyCode::Char(','));
    // the label field edits on Enter (16 characters at most)
    h.key(KeyCode::Enter);
    h.type_str("ab cd ef");
    let (x, y) = h.find("ab cd ef").unwrap_or_else(|| panic!("{}", h.text()));
    h.click(x + 3, y);
    h.click(x + 3, y);
    assert_eq!(rename_input(&h).as_deref(), Some("cd"), "{}", h.text());
    // the second press of another double click, then a drag: by words
    h.click(x, y);
    h.mouse(MouseKind::Down, x, y);
    h.mouse(MouseKind::Drag, x + 4, y);
    h.mouse(MouseKind::Up, x + 4, y);
    assert_eq!(rename_input(&h).as_deref(), Some("ab cd"));
}
