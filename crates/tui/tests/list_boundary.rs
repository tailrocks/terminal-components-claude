//! Opt-in physical List boundaries preserve normal navigation and selection.
use junie_tui::{
    App, Cx, Id, ItemKey, KeyCode, KeyModifiers, List, ListAction, ListState, Response, SelectMode,
    StateFlags, Theme, Ui,
};
use junie_tui_testing::Harness;
const ID: Id = Id::root("boundary.list");
struct Page {
    state: ListState,
    items: Vec<&'static str>,
    leave: bool,
    disabled: bool,
    actions: Vec<ListAction>,
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = List::new(ID)
            .leave_at_boundary(self.leave)
            .select_mode(SelectMode::Range)
            .disabled_item(&|s: &&str| s.starts_with("disabled"))
            .key(|s: &&str| ItemKey::text(s))
            .update(cx, &mut self.state, &self.items);
        if let Some(action) = response.take_action() {
            self.actions.push(action);
        }
        response.erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        List::new(ID)
            .leave_at_boundary(self.leave)
            .select_mode(SelectMode::Range)
            .disabled_item(&|s: &&str| s.starts_with("disabled"))
            .key(|s: &&str| ItemKey::text(s))
            .draw(ui, ui.full(), &self.state, &self.items);
        if self.disabled {
            ui.declare_state(ID, StateFlags::DISABLED);
        }
    }
}
fn page(leave: bool, items: Vec<&'static str>) -> Harness<Page> {
    Harness::new(
        Page {
            state: ListState::default(),
            items,
            leave,
            disabled: false,
            actions: Vec::new(),
        },
        Theme::junie(),
        24,
        2,
    )
}
#[test]
fn physical_edges_exit_only_when_opted_in_and_preserve_selection() {
    for leave in [false, true] {
        let mut h = page(leave, vec!["disabled-first", "middle", "disabled-last"]);
        let _ = h.key(KeyCode::Home);
        h.app_mut().actions.clear();
        h.app_mut().state.choose(Some(ItemKey::text("middle")));
        h.app_mut()
            .state
            .checked_mut()
            .insert(ItemKey::text("middle"));
        let cursor = h.app().state.cursor();
        let chosen = h.app().state.chosen();
        let checked = h.app().state.checked().clone();
        let _ = h.key(KeyCode::Up);
        assert_eq!(h.app().state.cursor(), cursor);
        assert_eq!(h.app().state.chosen(), chosen);
        assert_eq!(h.app().state.checked(), &checked);
        assert_eq!(
            h.app().actions.as_slice(),
            if leave {
                &[ListAction::LeaveBackward][..]
            } else {
                &[ListAction::Moved][..]
            }
        );
        h.app_mut().actions.clear();
        let _ = h.key(KeyCode::Down);
        assert_eq!(h.app().state.cursor(), Some(ItemKey::text("middle")));
        assert_eq!(h.app().actions, vec![ListAction::Moved]);
        let _ = h.key(KeyCode::End);
        let _ = h.key(KeyCode::F(12));
        h.app_mut().actions.clear();
        let offset = h.app().state.scroll().offset();
        let _ = h.key(KeyCode::Down);
        assert_eq!(h.app().state.cursor(), Some(ItemKey::text("disabled-last")));
        assert_eq!(h.app().state.scroll().offset(), offset);
        assert_eq!(
            h.app().actions.as_slice(),
            if leave {
                &[ListAction::LeaveForward][..]
            } else {
                &[ListAction::Moved][..]
            }
        );
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}
#[test]
fn other_navigation_and_empty_or_disabled_control_never_exit() {
    let mut h = page(true, vec!["first", "middle", "last"]);
    for key in [
        KeyCode::Home,
        KeyCode::Char('k'),
        KeyCode::PageUp,
        KeyCode::End,
        KeyCode::Char('j'),
        KeyCode::PageDown,
    ] {
        let _ = h.key(key);
    }
    let _ = h.key_mod(KeyCode::Down, KeyModifiers::SHIFT);
    let _ = h.key(KeyCode::Home);
    let _ = h.key_mod(KeyCode::Up, KeyModifiers::SHIFT);
    assert!(
        !h.app()
            .actions
            .iter()
            .any(|a| matches!(a, ListAction::LeaveBackward | ListAction::LeaveForward))
    );
    for disabled in [false, true] {
        let mut h = page(true, if disabled { vec!["only"] } else { vec![] });
        h.app_mut().disabled = disabled;
        h.draw();
        let _ = h.key(KeyCode::Up);
        let _ = h.key(KeyCode::Down);
        assert!(
            !h.app().actions.iter().any(|action| matches!(
                action,
                ListAction::LeaveBackward | ListAction::LeaveForward
            ))
        );
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}
#[test]
fn disabled_items_remain_physical_navigation_targets() {
    let mut h = page(true, vec!["disabled-first", "disabled-last"]);
    let _ = h.key(KeyCode::Home);
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Up);
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Down);
    assert_eq!(
        h.app().actions,
        vec![
            ListAction::LeaveBackward,
            ListAction::Moved,
            ListAction::LeaveForward
        ]
    );
    assert_eq!(h.app().state.cursor(), Some(ItemKey::text("disabled-last")));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
