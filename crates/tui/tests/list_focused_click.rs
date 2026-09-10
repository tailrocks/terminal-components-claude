//! Selected-pane clicks use prior runtime focus, not a double-click deadline.
use junie_tui::{
    App, Button, Cx, Id, Intent, ItemKey, KeyCode, List, ListAction, ListState, MouseKind, Part,
    PartRef, Phase, Rect, Response, Theme, Ui,
};
use junie_tui_testing::Harness;
use std::time::Duration;
const ID: Id = Id::root("list.focused.click");
const OTHER: Id = Id::root("list.focused.other");
#[derive(Clone, Copy)]
struct Entry {
    name: &'static str,
    disabled: bool,
}
impl std::fmt::Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}
fn key(e: &Entry) -> ItemKey {
    ItemKey::text(e.name)
}
struct Page {
    state: ListState,
    rows: Vec<Entry>,
    enabled: bool,
    focus_away: bool,
    action: Option<ListAction>,
}
impl Page {
    fn new(enabled: bool) -> Self {
        Self {
            state: ListState::default(),
            rows: vec![
                Entry {
                    name: "one",
                    disabled: false,
                },
                Entry {
                    name: "two",
                    disabled: false,
                },
                Entry {
                    name: "three",
                    disabled: false,
                },
            ],
            enabled,
            focus_away: false,
            action: None,
        }
    }
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut r = Button::new(OTHER, "Other").update(cx).erase();
        let list = List::new(ID)
            .key(key)
            .disabled_item(&|e| e.disabled)
            .activate_focused_on_click(self.enabled)
            .update(cx, &mut self.state, &self.rows);
        self.action = list.action_ref().copied();
        r |= list.erase();
        if self.focus_away
            && cx.intents(ID).any(|intent| {
                matches!(
                    intent,
                    Intent::Pointer {
                        phase: Phase::Press,
                        ..
                    }
                )
            })
        {
            cx.focus(OTHER);
        }
        r
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        Button::new(OTHER, "Other").draw(ui, Rect::new(0, 0, 20, 1));
        List::new(ID)
            .key(key)
            .disabled_item(&|e| e.disabled)
            .activate_focused_on_click(self.enabled)
            .draw(ui, Rect::new(0, 1, 20, 4), &self.state, &self.rows);
    }
}
fn harness(enabled: bool) -> Harness<Page> {
    Harness::new(Page::new(enabled), Theme::junie(), 24, 7)
}
fn click(h: &mut Harness<Page>, name: &str) {
    let _ = h.click_part(ID, PartRef::item(Part::ROW, ItemKey::text(name)));
}
fn action(h: &Harness<Page>, a: ListAction) {
    assert_eq!(h.app().action, Some(a));
}
#[test]
fn first_click_acquires_focus_then_delayed_selected_repeat_activates() {
    let mut h = harness(true);
    assert_eq!(h.focus(), Some(OTHER));
    assert_eq!(h.app().state.cursor(), Some(ItemKey::text("one")));
    click(&mut h, "one");
    action(&h, ListAction::Chose(ItemKey::text("one")));
    assert_eq!(h.focus(), Some(ID));
    let _ = h.advance(Duration::from_secs(10));
    click(&mut h, "one");
    action(&h, ListAction::Activated(ItemKey::text("one")));
}
#[test]
fn keyboard_focused_selected_row_activates_on_first_click() {
    let mut h = harness(true);
    let _ = h.key(KeyCode::Tab);
    let _ = h.key(KeyCode::Down);
    click(&mut h, "two");
    action(&h, ListAction::Activated(ItemKey::text("two")));
}
#[test]
fn fast_second_click_does_not_override_a_changed_cursor() {
    let mut h = harness(true);
    click(&mut h, "two");
    h.app_mut().state.set_cursor(2, ItemKey::text("three"));
    let _ = h.tick();
    click(&mut h, "two");
    action(&h, ListAction::Chose(ItemKey::text("two")));
}
#[test]
fn default_slow_repeat_and_double_click_remain_unchanged() {
    let mut h = harness(false);
    click(&mut h, "two");
    let _ = h.advance(Duration::from_secs(10));
    click(&mut h, "two");
    action(&h, ListAction::Chose(ItemKey::text("two")));
    click(&mut h, "two");
    action(&h, ListAction::Activated(ItemKey::text("two")));
}
#[test]
fn reacquiring_focus_requires_choice_even_for_the_retained_cursor() {
    let mut h = harness(true);
    click(&mut h, "two");
    let _ = h.click_id(OTHER);
    click(&mut h, "two");
    action(&h, ListAction::Chose(ItemKey::text("two")));
}
#[test]
fn disabled_removed_and_no_cursor_rows_never_activate() {
    let mut h = harness(true);
    let _ = h.key(KeyCode::Tab);
    let _ = h.mouse(MouseKind::Down, 10, 1);
    for row in &mut h.app_mut().rows {
        row.disabled = true;
    }
    let _ = h.mouse(MouseKind::Up, 10, 1);
    assert!(!matches!(h.app().action, Some(ListAction::Activated(_))));
    let mut h = harness(true);
    let _ = h.key(KeyCode::Tab);
    let _ = h.mouse(MouseKind::Down, 10, 1);
    h.app_mut().rows.remove(0);
    let _ = h.mouse(MouseKind::Up, 10, 1);
    assert!(!matches!(h.app().action, Some(ListAction::Activated(_))));
    let mut p = Page::new(true);
    for row in &mut p.rows {
        row.disabled = true;
    }
    let mut h = Harness::new(p, Theme::junie(), 24, 7);
    assert_eq!(h.app().state.cursor(), None);
    let _ = h.click(10, 1);
    assert!(!matches!(h.app().action, Some(ListAction::Activated(_))));
    assert_eq!(h.app().state.cursor(), None);
}
#[test]
fn reorder_uses_identity_instead_of_screen_row() {
    let mut h = harness(true);
    click(&mut h, "two");
    h.app_mut().rows.swap(0, 1);
    let _ = h.tick();
    click(&mut h, "two");
    action(&h, ListAction::Activated(ItemKey::text("two")));
    click(&mut h, "one");
    action(&h, ListAction::Chose(ItemKey::text("one")));
}
#[test]
fn dragging_and_canceling_press_do_not_activate() {
    let mut h = harness(true);
    let _ = h.key(KeyCode::Tab);
    let _ = h.mouse(MouseKind::Down, 10, 1);
    let _ = h.mouse(MouseKind::Drag, 10, 3);
    let _ = h.mouse(MouseKind::Up, 10, 3);
    assert!(!matches!(h.app().action, Some(ListAction::Activated(_))));
    let _ = h.mouse(MouseKind::Down, 10, 1);
    let _ = h.mouse(MouseKind::Up, 23, 6);
    assert!(!matches!(h.app().action, Some(ListAction::Activated(_))));
    click(&mut h, "two");
    action(&h, ListAction::Chose(ItemKey::text("two")));
}
#[test]
fn focus_loss_during_press_cancels_repeat_activation() {
    let mut h = harness(true);
    let _ = h.key(KeyCode::Tab);
    h.app_mut().focus_away = true;
    let _ = h.mouse(MouseKind::Down, 10, 1);
    assert_eq!(h.focus(), Some(OTHER));
    let _ = h.mouse(MouseKind::Up, 10, 1);
    assert!(!matches!(h.app().action, Some(ListAction::Activated(_))));
}
