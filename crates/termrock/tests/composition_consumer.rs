use core::fmt;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui_core::layout::{Position, Rect};
use termrock::editors::{CompletionBatch, CompletionTrigger};
use termrock::{
    App, Button, CodeEditor, CodeEditorState, CompletingCodeEditor, CompletingCodeEditorState, Cx,
    Id, Input, ItemKey, Key, KeyCode, KeyModifiers, Mouse, MouseKind, Response, Runtime, Tabs,
    TabsAction, TabsState, Theme, composition,
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
                    label: "源泉 🚀",
                },
                Workspace {
                    key: ItemKey::text("review"),
                    label: "Long workspace · 👩🏽‍💻 — Unicode text beyond the preferred tab width",
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
            composition::Group::column([
                composition::Group::wrap([composition::tabs(
                    &self.tabs,
                    &self.tabs_state,
                    &self.workspaces,
                )
                .flex(1)
                .max(Some(8), None)])
                .into_child()
                .flex(1),
                composition::Group::wrap(std::iter::empty()).into_child(),
            ])
            .into_child()
            .flex(1),
        ])
        .spec(Composition::new().gap(1).padding(Insets::all(1)))
        .show(ui)
        .expect("composition is within the supported depth");
    }
}

fn render<A: App>(runtime: &mut Runtime<A>, terminal: &mut Terminal<TestBackend>, area: Rect) {
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
    assert_eq!(runtime.app().run_count, 0, "drawing has no domain effects");
    assert_eq!(runtime.app().last_tab_action, None);

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

const CODE: Id = Id::root("external.code");
const COMPLETING_CODE: Id = Id::root("external.completing_code");
const COMPLETION: Id = Id::root("external.completion");

fn no_completions(_: &str, _: usize, _: CompletionTrigger) -> Option<CompletionBatch> {
    None
}

struct EditorConsumer {
    editor: CodeEditor<'static>,
    editor_state: CodeEditorState,
    completing: CompletingCodeEditor<'static>,
    completing_state: CompletingCodeEditorState,
}

impl EditorConsumer {
    fn new() -> Self {
        let editor = CodeEditor::new(CODE, 4).fill_height(true);
        let completing = CodeEditor::new(COMPLETING_CODE, 4)
            .fill_height(true)
            .with_completion(COMPLETION, &no_completions);
        let mut editor_state = CodeEditorState::new("SELECT 👩🏽‍💻\nλ");
        editor_state.begin_edit();
        let mut completing_state = CompletingCodeEditorState::new("");
        completing_state.editor_mut().begin_edit();
        Self {
            editor,
            editor_state,
            completing,
            completing_state,
        }
    }
}

impl App for EditorConsumer {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.editor.update(cx, &mut self.editor_state).erase()
            | self
                .completing
                .update(cx, &mut self.completing_state)
                .erase()
    }

    fn draw(&self, ui: &mut termrock::Ui<'_>) {
        composition::Group::column([composition::Group::row([
            composition::code_editor(&self.editor, &self.editor_state).flex(1),
            composition::completing_code_editor(&self.completing, &self.completing_state).flex(1),
        ])
        .into_child()
        .flex(1)])
        .show(ui)
        .expect("editor composition is within the supported depth");
    }
}

#[test]
fn public_facade_composes_editors_with_unicode_empty_state_and_resolved_bounds() {
    let root = Rect::new(0, 0, 56, 5);
    let mut runtime = Runtime::new(EditorConsumer::new(), Theme::junie());
    let mut terminal = Terminal::new(TestBackend::new(root.width, root.height)).unwrap();
    render(&mut runtime, &mut terminal, root);

    let editor_area = runtime.area_of(CODE).expect("code editor hit area");
    let completing_area = runtime
        .area_of(COMPLETING_CODE)
        .expect("completing editor hit area");
    assert!(within(root, editor_area));
    assert!(within(root, completing_area));
    assert!(editor_area.width > 0 && editor_area.height > 0);
    assert!(completing_area.width > 0 && completing_area.height > 0);
    assert_eq!(editor_area.height.saturating_add(1), root.height);
    assert_eq!(completing_area.height.saturating_add(1), root.height);
    assert!(editor_area.x < completing_area.x);

    let point = Position::new(editor_area.x, editor_area.y);
    let _ = runtime
        .handle(Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos: point,
            mods: KeyModifiers::NONE,
        }))
        .expect("pointer focus is dispatched through the editor hit region");
    render(&mut runtime, &mut terminal, root);
    let _ = runtime
        .handle(Input::Key(Key {
            code: KeyCode::Char('界'),
            mods: KeyModifiers::NONE,
        }))
        .expect("Unicode text input is dispatched through the focused editor");

    assert!(runtime.app().editor_state.text().contains('界'));
    assert_eq!(runtime.app().completing_state.editor().text(), "");
    assert!(
        runtime
            .cursor_position()
            .is_some_and(|cursor| within(root, Rect::new(cursor.x, cursor.y, 1, 1)))
    );

    let resized = Rect::new(0, 0, 36, 3);
    let mut resized_terminal =
        Terminal::new(TestBackend::new(resized.width, resized.height)).unwrap();
    render(&mut runtime, &mut resized_terminal, resized);
    let resized_editor_area = runtime
        .area_of(CODE)
        .expect("editor layout refreshes after resize");
    assert!(within(resized, resized_editor_area));
    assert!(
        runtime
            .cursor_position()
            .is_some_and(|cursor| within(resized, Rect::new(cursor.x, cursor.y, 1, 1)))
    );
}

struct DepthGuardConsumer;

impl App for DepthGuardConsumer {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut termrock::Ui<'_>) {
        let mut nested = composition::Group::wrap(std::iter::empty());
        for _ in 0..8_000 {
            nested = composition::Group::row([nested.into_child()]);
        }
        assert_eq!(
            nested.show(ui),
            Err(composition::CompositionError::DepthExceeded {
                depth: 8_001,
                maximum: 128,
            })
        );
        // Destruction at the end of this method also verifies that a rejected
        // over-depth consumer tree does not recursively drop each group.
    }
}

#[test]
fn over_depth_nested_public_groups_fail_before_recursive_runtime_conversion() {
    let area = Rect::new(0, 0, 10, 4);
    let mut runtime = Runtime::new(DepthGuardConsumer, Theme::junie());
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
    render(&mut runtime, &mut terminal, area);
}

#[test]
fn zero_area_prepare_then_resize_keeps_composed_input_live() {
    let initial = Rect::new(0, 0, 40, 8);
    let resized = Rect::new(0, 0, 24, 6);
    let mut runtime = Runtime::new(ExternalConsumer::new(), Theme::junie());
    let mut terminal = Terminal::new(TestBackend::new(initial.width, initial.height)).unwrap();
    render(&mut runtime, &mut terminal, initial);

    runtime
        .prepare(Rect::new(0, 0, 0, 0))
        .expect("zero-area preparation is valid");
    let mut terminal = Terminal::new(TestBackend::new(resized.width, resized.height)).unwrap();
    render(&mut runtime, &mut terminal, resized);

    let tabs_area = runtime.area_of(WORKSPACES).expect("resized tabs hit area");
    assert!(within(resized, tabs_area));
    let point = Position::new(tabs_area.x, tabs_area.y);
    let _ = runtime
        .handle(Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos: point,
            mods: KeyModifiers::NONE,
        }))
        .expect("resized tabs pointer input is dispatched");
    render(&mut runtime, &mut terminal, resized);
    let _ = runtime
        .handle(Input::Mouse(Mouse {
            kind: MouseKind::Up,
            pos: point,
            mods: KeyModifiers::NONE,
        }))
        .expect("resized tabs pointer release is dispatched");
    render(&mut runtime, &mut terminal, resized);
    assert_eq!(runtime.focus(), Some(WORKSPACES));
}
