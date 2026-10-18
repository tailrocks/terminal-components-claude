//! Pointer-ineligible rows retain the source List keyboard/control contract.
use junie_tui::{
    App, Button, Cx, Family, Id, Intent, ItemKey, KeyCode, List, ListAction, ListState, MouseKind,
    Part, PartRef, Phase, Rect, Response, StateFlags, Theme, Ui, Variant,
};
use junie_tui_testing::Harness;
use std::cell::RefCell;
const ID: Id = Id::root("list.pointer.rows");
const OTHER: Id = Id::root("list.pointer.other");
#[derive(Clone, Copy)]
struct Entry {
    name: &'static str,
    pointer: bool,
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
    policy: bool,
    custom: bool,
    action: Option<ListAction>,
    deny_on_release: bool,
    seen: RefCell<Vec<(ItemKey, StateFlags)>>,
}
impl Page {
    fn new(policy: bool, custom: bool) -> Self {
        Self {
            state: ListState::default(),
            rows: vec![
                Entry {
                    name: "heading",
                    pointer: false,
                },
                Entry {
                    name: "pane",
                    pointer: true,
                },
                Entry {
                    name: "session",
                    pointer: false,
                },
            ],
            policy,
            custom,
            action: None,
            deny_on_release: false,
            seen: RefCell::new(Vec::new()),
        }
    }
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut r = Button::new(OTHER, "Other").update(cx).erase();
        if self.deny_on_release
            && cx.intents(ID).any(|intent| {
                matches!(
                    intent,
                    Intent::Pointer {
                        phase: Phase::Click | Phase::DoubleClick,
                        ..
                    }
                )
            })
        {
            for row in &mut self.rows {
                row.pointer = false;
            }
        }
        let allowed = |e: &Entry| !self.policy || e.pointer;
        let list =
            List::new(ID)
                .key(key)
                .pointer_item(&allowed)
                .update(cx, &mut self.state, &self.rows);
        self.action = list.action_ref().copied();
        r |= list.erase();
        r
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        Button::new(OTHER, "Other").draw(ui, Rect::new(0, 0, 20, 1));
        self.seen.borrow_mut().clear();
        let paint = |ui: &mut Ui<'_>, area, flags, key, item: &Entry| {
            self.seen.borrow_mut().push((key, flags));
            // Source Manager topology decor rows ignore cursor presentation.
            let visual = if item.pointer {
                flags
            } else {
                StateFlags::empty()
            };
            let style = ui
                .style(Family::LIST, Variant::DEFAULT, Part::CONTAINER, visual)
                .style;
            ui.fill(area, style);
            ui.paint_str(area, item.name, style);
        };
        let allowed = |e: &Entry| !self.policy || e.pointer;
        let list = List::new(ID).pointer_item(&allowed).key(key);
        let area = Rect::new(0, 1, 20, 4);
        if self.custom {
            list.render_row(&paint)
                .draw(ui, area, &self.state, &self.rows);
        } else {
            list.draw(ui, area, &self.state, &self.rows);
        }
    }
}
fn harness(policy: bool, custom: bool) -> Harness<Page> {
    Harness::new(Page::new(policy, custom), Theme::junie(), 24, 7)
}
#[test]
fn nonpointer_row_click_focuses_owner_without_changing_cursor_or_choice() {
    for custom in [false, true] {
        let mut h = harness(true, custom);
        assert_eq!(h.focus(), Some(OTHER));
        h.app_mut().state.set_cursor(1, ItemKey::text("pane"));
        let _ = h.tick();
        assert!(
            h.area_of_part(ID, PartRef::item(Part::ROW, ItemKey::text("heading")))
                .is_none()
        );
        let _ = h.click(10, 1);
        assert_eq!(h.focus(), Some(ID));
        assert_eq!(h.app().state.cursor(), Some(ItemKey::text("pane")));
        assert_eq!(h.app().state.chosen(), None);
        assert_eq!(h.app().action, None);
    }
}
#[test]
fn keyboard_traverses_and_activates_nonpointer_rows() {
    let mut h = harness(true, true);
    let _ = h.key(KeyCode::Tab);
    assert_eq!(h.focus(), Some(ID));
    assert_eq!(h.app().state.cursor(), Some(ItemKey::text("heading")));
    let _ = h.key(KeyCode::Enter);
    assert_eq!(
        h.app().action,
        Some(ListAction::Activated(ItemKey::text("heading")))
    );
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Down);
    assert_eq!(h.app().state.cursor(), Some(ItemKey::text("session")));
    let _ = h.key(KeyCode::Enter);
    assert_eq!(
        h.app().action,
        Some(ListAction::Activated(ItemKey::text("session")))
    );
}
#[test]
fn eligible_pane_click_chooses_and_default_accepts_all_rows() {
    for custom in [false, true] {
        let mut h = harness(true, custom);
        let _ = h.click_part(ID, PartRef::item(Part::ROW, ItemKey::text("pane")));
        assert_eq!(
            h.app().action,
            Some(ListAction::Chose(ItemKey::text("pane")))
        );
        let mut default = harness(false, custom);
        let _ = default.click_part(ID, PartRef::item(Part::ROW, ItemKey::text("heading")));
        assert_eq!(
            default.app().action,
            Some(ListAction::Chose(ItemKey::text("heading")))
        );
    }
}
#[test]
fn row_hover_and_press_do_not_leak_from_broad_owner_feedback() {
    let mut h = harness(true, true);
    let _ = h.mouse(MouseKind::Move, 10, 1);
    let _ = h.mouse(MouseKind::Down, 10, 1);
    for (_, flags) in h.app().seen.borrow().iter() {
        assert!(!flags.intersects(StateFlags::HOVERED | StateFlags::PRESSED));
    }
    let _ = h.mouse(MouseKind::Up, 10, 1);
    let _ = h.mouse(MouseKind::Move, 10, 2);
    let _ = h.mouse(MouseKind::Down, 10, 2);
    assert!(
        h.app()
            .seen
            .borrow()
            .iter()
            .any(|(key, flags)| *key == ItemKey::text("pane")
                && flags.contains(StateFlags::HOVERED | StateFlags::PRESSED))
    );
}
#[test]
fn stale_row_pointer_intent_cannot_choose_after_eligibility_changes() {
    let mut h = harness(true, true);
    let _ = h.mouse(MouseKind::Down, 10, 2);
    for row in &mut h.app_mut().rows {
        row.pointer = false;
    }
    // No publication between mutation and release: update must reject old row intent.
    let _ = h.mouse(MouseKind::Up, 10, 2);
    assert_eq!(h.app().state.chosen(), None);
    assert_eq!(h.app().action, None);
    assert!(
        h.app()
            .seen
            .borrow()
            .iter()
            .all(|(_, f)| !f.intersects(StateFlags::HOVERED | StateFlags::PRESSED))
    );
}
#[test]
fn reorder_recomputes_keyed_eligibility_and_keeps_keyboard_selection() {
    let mut h = harness(true, true);
    let _ = h.click(10, 2);
    h.app_mut().rows.swap(0, 1);
    let _ = h.tick();
    assert_eq!(h.app().state.cursor(), Some(ItemKey::text("pane")));
    assert!(
        h.area_of_part(ID, PartRef::item(Part::ROW, ItemKey::text("heading")))
            .is_none()
    );
    let _ = h.click(10, 2);
    assert_eq!(h.app().state.cursor(), Some(ItemKey::text("pane")));
    assert_eq!(h.app().action, None);
}

#[test]
fn eligibility_changed_after_delivery_rejects_the_queued_row_action() {
    let mut h = harness(true, true);
    let _ = h.mouse(MouseKind::Down, 10, 2);
    h.app_mut().deny_on_release = true;
    let _ = h.mouse(MouseKind::Up, 10, 2);
    assert_eq!(h.app().state.chosen(), None);
    assert_eq!(h.app().action, None);
}
