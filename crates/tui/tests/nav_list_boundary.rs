//! Opt-in list exits preserve collection state and default navigation.
use junie_tui::{
    App, Cx, Id, ItemKey, KeyCode, NavList, NavListAction, NavListState, Response, Theme, Ui,
};
use junie_tui_testing::Harness;
const ID: Id = Id::root("boundary.nav");
struct Page {
    state: NavListState,
    items: Vec<&'static str>,
    leave: bool,
    actions: Vec<NavListAction>,
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = NavList::new(ID)
            .leave_at_boundary(self.leave)
            .scrollable(true)
            .disabled_item(&|s: &&str| s.starts_with("disabled"))
            .key(|s: &&str| ItemKey::text(s))
            .update(cx, &mut self.state, &self.items);
        if let Some(action) = response.take_action() {
            self.actions.push(action);
        }
        response.erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        NavList::new(ID)
            .leave_at_boundary(self.leave)
            .scrollable(true)
            .disabled_item(&|s: &&str| s.starts_with("disabled"))
            .key(|s: &&str| ItemKey::text(s))
            .draw(ui, ui.full(), &self.state, &self.items);
    }
}
fn page(leave: bool, items: Vec<&'static str>) -> Harness<Page> {
    Harness::new(
        Page {
            state: NavListState::new(),
            items,
            leave,
            actions: Vec::new(),
        },
        Theme::junie(),
        24,
        2,
    )
}
#[test]
fn default_edges_remain_consumed_and_opt_in_edges_preserve_state() {
    for leave in [false, true] {
        let mut h = page(
            leave,
            vec![
                "disabled-first",
                "first",
                "disabled-mid",
                "last",
                "disabled-last",
            ],
        );
        let _ = h.key(KeyCode::Enter);
        h.app_mut().actions.clear();
        let cursor = h.app().state.cursor();
        let current = h.app().state.current();
        let _ = h.key(KeyCode::Up);
        assert_eq!(h.app().state.cursor(), cursor);
        assert_eq!(h.app().state.current(), current);
        assert_eq!(
            h.app().actions.as_slice(),
            if leave {
                &[NavListAction::LeaveBackward][..]
            } else {
                &[]
            }
        );
        h.app_mut().actions.clear();
        let _ = h.key(KeyCode::Down);
        assert_eq!(
            h.app().actions,
            vec![NavListAction::Moved(ItemKey::text("last"))]
        );
        // Commit the preceding move's reveal before probing an exit attempt.
        let _ = h.key(KeyCode::F(12));
        h.app_mut().actions.clear();
        let offset = h.app().state.scroll().offset();
        let _ = h.key(KeyCode::Down);
        assert_eq!(h.app().state.cursor(), Some(ItemKey::text("last")));
        assert_eq!(h.app().state.current(), current);
        assert_eq!(h.app().state.scroll().offset(), offset);
        assert_eq!(
            h.app().actions.as_slice(),
            if leave {
                &[NavListAction::LeaveForward][..]
            } else {
                &[]
            }
        );
        h.app_mut().actions.clear();
        let _ = h.key(KeyCode::End);
        let _ = h.key(KeyCode::Home);
        let _ = h.key(KeyCode::Home);
        assert_eq!(
            h.app().actions,
            vec![NavListAction::Moved(ItemKey::text("first"))]
        );
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}
#[test]
fn no_enabled_rows_never_requests_exit() {
    for items in [vec![], vec!["disabled-only"]] {
        let mut h = page(true, items);
        for key in [KeyCode::Up, KeyCode::Down, KeyCode::Home, KeyCode::End] {
            let _ = h.key(key);
        }
        assert!(h.app().actions.is_empty());
        assert_eq!(h.app().state.cursor(), None);
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}
