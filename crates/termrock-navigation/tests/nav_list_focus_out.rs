use termrock_controls::Button;
use termrock_core::{Id, Intent, ItemKey, KeyCode, MouseKind, Part, PartRef, Phase, Response};
use termrock_navigation::{NavList, NavListAction, NavListState};
use termrock_runtime::{App, Cx, Rect, Ui, UpdateCause};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const LIST: Id = Id::root("nav-list.focus-out");
const OTHER: Id = Id::root("nav-list.focus-out.other");
const ROWS: [&str; 12] = [
    "row-0", "row-1", "row-2", "row-3", "row-4", "row-5", "row-6", "row-7", "row-8", "row-9",
    "row-10", "row-11",
];

fn key(item: &&'static str) -> ItemKey {
    ItemKey::text(item)
}

fn list() -> NavList<'static, &'static str, fn(&&'static str) -> ItemKey> {
    NavList::new(LIST)
        .key(key as fn(&&'static str) -> ItemKey)
        .scrollable(true)
}

struct Page {
    state: NavListState,
    items: Vec<&'static str>,
    mounted: bool,
    probe_before_normal_update: bool,
    focus_away_on_press: bool,
    focus_out_handled: usize,
    unsupported: usize,
    actions: Vec<NavListAction>,
}

impl Page {
    fn new() -> Self {
        Self {
            state: NavListState::new(),
            items: ROWS.to_vec(),
            mounted: true,
            probe_before_normal_update: false,
            focus_away_on_press: false,
            focus_out_handled: 0,
            unsupported: 0,
            actions: Vec::new(),
        }
    }

    fn update_list(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let response = list().update(cx, &mut self.state, &self.items);
        if let Some(action) = response.action_ref() {
            self.actions.push(*action);
        }
        response.erase()
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut result = Button::new(OTHER, "Other").update(cx).erase();
        if !self.mounted {
            if cx.update_cause() == UpdateCause::Settle {
                match list().update_focus_out(cx) {
                    Some(response) => {
                        self.focus_out_handled += 1;
                        result |= response;
                    }
                    None => self.unsupported += 1,
                }
            }
            return result;
        }

        if self.probe_before_normal_update && cx.update_cause() == UpdateCause::Event {
            assert!(list().update_focus_out(cx).is_none());
            result |= self.update_list(cx);
        } else {
            result |= self.update_list(cx);
        }
        if self.focus_away_on_press
            && cx.intents(LIST).any(|intent| {
                matches!(
                    intent,
                    Intent::Pointer {
                        phase: Phase::Press,
                        ..
                    }
                )
            })
        {
            self.focus_away_on_press = false;
            self.items.pop();
            self.mounted = false;
            cx.focus(OTHER);
        }
        result
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Button::new(OTHER, "Other").draw(ui, Rect::new(0, 0, 24, 1));
        list().draw(ui, Rect::new(0, 1, 24, 5), &self.state, &self.items);
    }
}

fn harness() -> Harness<Page> {
    Harness::new(Page::new(), Theme::junie(), 24, 6)
}

#[test]
fn focused_route_settle_preserves_nav_state_without_source_reconciliation() {
    let mut h = harness();
    assert!(h.tab_to(LIST));
    let _ = h.key(KeyCode::End);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().state.current(), Some(ItemKey::text("row-11")));
    assert!(h.app().state.scroll().offset() > 0);
    h.app_mut().actions.clear();
    let before = h.app().state.clone();
    let area = h
        .area_of_part(LIST, PartRef::item(Part::ROW, ItemKey::text("row-11")))
        .expect("last current row is visible after End");

    h.app_mut().focus_away_on_press = true;
    let _ = h.mouse(
        MouseKind::Down,
        area.x.saturating_add(area.width / 2),
        area.y.saturating_add(area.height / 2),
    );

    assert_eq!(h.focus(), Some(OTHER));
    assert_eq!(
        h.app().state,
        before,
        "NavList focus handling must not reconcile source"
    );
    assert!(h.app().focus_out_handled > 0);
    assert_eq!(h.app().unsupported, 0);
    assert!(h.app().actions.is_empty());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

    h.app_mut().mounted = true;
    let _ = h.mouse(
        MouseKind::Up,
        area.x.saturating_add(area.width / 2),
        area.y.saturating_add(area.height / 2),
    );
    assert_eq!(h.app().state.current(), None);
    assert_ne!(h.app().state.cursor(), Some(ItemKey::text("row-11")));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn wrong_cause_falls_back_to_same_real_nav_list_action_once() {
    let mut h = harness();
    assert!(h.tab_to(LIST));
    h.app_mut().probe_before_normal_update = true;

    let _ = h.key(KeyCode::Down);

    assert_eq!(
        h.app().actions,
        vec![NavListAction::Moved(ItemKey::text("row-1"))]
    );
    assert_eq!(h.app().state.cursor(), Some(ItemKey::text("row-1")));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
