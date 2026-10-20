use core::fmt;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui_core::layout::{Position, Rect};
use termrock::{
    App, Button, Cx, Id, Input, ItemKey, Key, KeyCode, KeyModifiers, Mouse, MouseKind, Response,
    Runtime, Tabs, TabsAction, TabsState, Theme, composition,
    layout::{Composition, Insets},
};

const RUN: Id = Id::root("external.run");
const WORKSPACES: Id = Id::root("external.workspaces");

#[derive(Clone, Copy)]
struct Workspace {
    key: ItemKey,
    label: &'static str,
}

impl fmt::Display for Workspace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label)
    }
}

fn workspace_key(workspace: &Workspace) -> ItemKey {
    workspace.key
}

type WorkspaceTabs = Tabs<'static, Workspace, fn(&Workspace) -> ItemKey>;

struct ExternalConsumer {
    run: Button<'static>,
    tabs: WorkspaceTabs,
    tabs_state: TabsState,
    workspaces: [Workspace; 2],
    run_count: usize,
    last_tab_action: Option<TabsAction>,
}

impl ExternalConsumer {
    fn new() -> Self {
        Self {
            run: Button::new(RUN, "Run"),
            tabs: Tabs::new(WORKSPACES).key(workspace_key),
            tabs_state: TabsState::default(),
            workspaces: [
                Workspace {
                    key: ItemKey::text("source"),
                    label: "Source",
                },
                Workspace {
                    key: ItemKey::text("review"),
                    label: "Review",
                },
            ],
            run_count: 0,
            last_tab_action: None,
        }
    }
}

impl App for ExternalConsumer {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let run = self.run.update(cx);
        if run.activated() {
            self.run_count += 1;
        }
        let tabs = self.tabs.update(cx, &mut self.tabs_state, &self.workspaces);
        if let Some(action) = tabs.action_ref() {
            self.last_tab_action = Some(*action);
        }
        run.erase() | tabs.erase()
    }

    fn draw(&self, ui: &mut termrock::Ui<'_>) {
        composition::Group::row([
            composition::button(&self.run).max(Some(8), None),
            composition::tabs(&self.tabs, &self.tabs_state, &self.workspaces)
                .flex(1)
                .max(Some(8), None),
        ])
        .spec(Composition::new().gap(1).padding(Insets::all(1)))
        .show(ui);
    }
}

fn render(
    runtime: &mut Runtime<ExternalConsumer>,
    terminal: &mut Terminal<TestBackend>,
    area: Rect,
) {
    let prepared = runtime.prepare(area).expect("runtime prepares the frame");
    terminal
        .draw(|frame| {
            runtime
                .draw_prepared(prepared, frame)
                .expect("prepared frame matches terminal bounds");
        })
        .expect("test backend draws successfully");
    runtime
        .acknowledge_presented(prepared)
        .expect("prepared frame is physically acknowledged");
}

#[test]
fn public_facade_composes_borrowed_button_and_tabs_with_live_input() {
    let root = Rect::new(0, 0, 40, 8);
    let mut runtime = Runtime::new(ExternalConsumer::new(), Theme::junie());
    let mut terminal = Terminal::new(TestBackend::new(root.width, root.height)).unwrap();
    render(&mut runtime, &mut terminal, root);

    let button_area = runtime.area_of(RUN).expect("button hit area");
    let tabs_area = runtime.area_of(WORKSPACES).expect("tabs hit area");
    assert!(within(root, button_area));
    assert!(within(root, tabs_area));
    assert!(button_area.x < tabs_area.x);
    assert_eq!(
        runtime.app().tabs_state.active(),
        Some(ItemKey::text("source"))
    );

    let focus_point = Position::new(tabs_area.x, tabs_area.y);
    let _ = runtime
        .handle(Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos: focus_point,
            mods: KeyModifiers::NONE,
        }))
        .expect("tabs pointer input is dispatched");
    render(&mut runtime, &mut terminal, root);
    let _ = runtime
        .handle(Input::Mouse(Mouse {
            kind: MouseKind::Up,
            pos: focus_point,
            mods: KeyModifiers::NONE,
        }))
        .expect("tabs pointer release is dispatched");
    render(&mut runtime, &mut terminal, root);
    assert_eq!(runtime.focus(), Some(WORKSPACES));

    let _ = runtime
        .handle(Input::Key(Key {
            code: KeyCode::Right,
            mods: KeyModifiers::NONE,
        }))
        .expect("tabs keyboard input is dispatched");
    assert_eq!(
        runtime.app().tabs_state.active(),
        Some(ItemKey::text("review"))
    );
    assert_eq!(
        runtime.app().last_tab_action,
        Some(TabsAction::Activated(ItemKey::text("review")))
    );

    render(&mut runtime, &mut terminal, root);
    let button_area = runtime.area_of(RUN).expect("button hit area");
    let point = Position::new(button_area.x, button_area.y);
    let _ = runtime
        .handle(Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos: point,
            mods: KeyModifiers::NONE,
        }))
        .expect("button press input is dispatched");
    render(&mut runtime, &mut terminal, root);
    let _ = runtime
        .handle(Input::Mouse(Mouse {
            kind: MouseKind::Up,
            pos: point,
            mods: KeyModifiers::NONE,
        }))
        .expect("button release input is dispatched");
    assert_eq!(runtime.app().run_count, 1);
    assert_eq!(
        runtime.app().tabs_state.active(),
        Some(ItemKey::text("review"))
    );
}

fn within(root: Rect, child: Rect) -> bool {
    child.x >= root.x
        && child.y >= root.y
        && child.x.saturating_add(child.width) <= root.x.saturating_add(root.width)
        && child.y.saturating_add(child.height) <= root.y.saturating_add(root.height)
}

#[test]
fn composition_honors_intrinsic_gap_padding_and_child_maximum() {
    let root = Rect::new(0, 0, 50, 8);
    let mut runtime = Runtime::new(ExternalConsumer::new(), Theme::junie());
    let mut terminal = Terminal::new(TestBackend::new(root.width, root.height)).unwrap();
    render(&mut runtime, &mut terminal, root);
    let button = runtime.area_of(RUN).expect("button hit area");
    let tabs = runtime.area_of(WORKSPACES).expect("tabs hit area");
    assert_eq!(button.x, 1);
    assert_eq!(button.y, 1);
    assert!(button.width <= 8);
    assert!(tabs.x > button.x + button.width);
    assert!(tabs.width <= 8, "explicit child max is applied");
}
