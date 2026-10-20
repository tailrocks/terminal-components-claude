use core::fmt;
use std::cell::Cell;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui_core::layout::{Position, Rect};
use termrock::editors::{CompletionBatch, CompletionTrigger};
use termrock::{
    App, Button, CodeEditor, CodeEditorState, CompletingCodeEditor, CompletingCodeEditorState, Cx,
    Id, Input, ItemKey, Key, KeyCode, KeyModifiers, Mouse, MouseKind, Response, Runtime, StepState,
    Steps, StepsAction, StepsState, Tabs, TabsAction, TabsState, Theme, composition,
    layout::{Composition, Constraints, Insets, Size},
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

const IMPORT_STEPS: Id = Id::root("external.import_steps");

#[derive(Clone, Copy)]
struct ImportStep {
    key: usize,
    label: &'static str,
    state: StepState,
    elapsed: Option<&'static str>,
}

fn import_step_key(step: &ImportStep) -> ItemKey {
    ItemKey::index(step.key)
}

fn import_step_state(step: &ImportStep) -> StepState {
    step.state
}

fn import_step_metadata(step: &ImportStep) -> Option<&str> {
    step.elapsed
}

fn import_step_label_width(step: &ImportStep) -> u16 {
    termrock::text::width(step.label)
}

fn draw_import_step(step: &ImportStep, row: &mut termrock::RowUi<'_>) {
    row.label(step.label);
}

type ImportSteps = Steps<
    'static,
    ImportStep,
    fn(&ImportStep) -> ItemKey,
    fn(&ImportStep, &mut termrock::RowUi<'_>),
>;

struct ImportConsumer {
    steps: ImportSteps,
    state: StepsState,
    items: Vec<ImportStep>,
    last_action: Option<StepsAction>,
    measured_steps: Cell<Size>,
}

impl ImportConsumer {
    fn new() -> Self {
        let key: fn(&ImportStep) -> ItemKey = import_step_key;
        let row: fn(&ImportStep, &mut termrock::RowUi<'_>) = draw_import_step;
        let steps = Steps::new(IMPORT_STEPS)
            .key(key)
            .step(&import_step_state)
            .numbered(true)
            .metadata(&import_step_metadata)
            .measure_label_width(&import_step_label_width)
            .row(row);
        Self {
            steps,
            state: StepsState::new(),
            items: vec![
                ImportStep {
                    key: 0,
                    label: "Fetch source",
                    state: StepState::Done,
                    elapsed: Some("2s"),
                },
                ImportStep {
                    key: 1,
                    label: "Parse manifest",
                    state: StepState::Running,
                    elapsed: Some("11s"),
                },
                ImportStep {
                    key: 2,
                    label: "Index package files",
                    state: StepState::Queued,
                    elapsed: None,
                },
                ImportStep {
                    key: 3,
                    label: "Publish imported package",
                    state: StepState::Queued,
                    elapsed: None,
                },
                ImportStep {
                    key: 4,
                    label: "Verify signatures",
                    state: StepState::Blocked,
                    elapsed: Some("waiting for review"),
                },
                ImportStep {
                    key: 5,
                    label: "Clean temporary files",
                    state: StepState::Skipped,
                    elapsed: None,
                },
                ImportStep {
                    key: 6,
                    label: "Final report — 確認",
                    state: StepState::Failed,
                    elapsed: Some("retry needed"),
                },
            ],
            last_action: None,
            measured_steps: Cell::new(Size::default()),
        }
    }
}

impl App for ImportConsumer {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let response = self.steps.update(cx, &mut self.state, &self.items);
        self.last_action = response.action_ref().copied();
        response.erase()
    }

    fn draw(&self, ui: &mut termrock::Ui<'_>) {
        let composition =
            composition::Group::row([composition::steps(&self.steps, &self.state, &self.items)]);
        self.measured_steps.set(
            composition
                .measure(ui, Constraints::loose(200, 120))
                .expect("public deployment steps fit the composition depth"),
        );
        composition
            .show(ui)
            .expect("public deployment steps fit the composition depth");
    }
}

fn terminal_text(terminal: &Terminal<TestBackend>) -> String {
    let area = *terminal.backend().buffer().area();
    (0..area.height)
        .map(|y| {
            (0..area.width)
                .filter_map(|x| terminal.backend().buffer().cell((x, y)))
                .map(|cell| cell.symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn public_steps_composition_uses_live_import_data_and_stable_keys() {
    let root = Rect::new(0, 0, 48, 9);
    let mut runtime = Runtime::new(ImportConsumer::new(), Theme::junie());
    let mut terminal = Terminal::new(TestBackend::new(root.width, root.height)).unwrap();
    render(&mut runtime, &mut terminal, root);
    let text = terminal_text(&terminal);
    assert!(text.contains("01"), "rendered import steps:\n{text}");
    assert!(text.contains("Fetch source"));
    assert!(text.contains("2s"));
    assert!(text.contains("02"));
    assert!(text.contains("Parse manifest"));
    assert!(text.contains("11s"));
    assert!(text.contains("07"));
    assert!(text.contains("Final report"));
    assert!(
        !text.lines().nth(2).unwrap_or_default().contains("queued"),
        "an explicit absent metadata value hides the default state label"
    );
    assert_eq!(
        runtime.app().last_action,
        None,
        "drawing emits no app action"
    );

    let stable = ItemKey::index(2);
    let first = runtime
        .area_of_part(
            IMPORT_STEPS,
            termrock::PartRef::item(termrock::Part::ROW, stable),
        )
        .expect("the original stable key has a rendered row");
    runtime.app_mut().items.swap(2, 4);
    render(&mut runtime, &mut terminal, root);
    let moved = runtime
        .area_of_part(
            IMPORT_STEPS,
            termrock::PartRef::item(termrock::Part::ROW, stable),
        )
        .expect("the same stable key remains rendered after reorder");
    assert_ne!(first.y, moved.y, "stable key follows reordered item");
    assert_eq!(runtime.app().last_action, None);

    let narrow = Rect::new(0, 0, 18, 4);
    let mut narrow_terminal = Terminal::new(TestBackend::new(narrow.width, narrow.height)).unwrap();
    render(&mut runtime, &mut narrow_terminal, narrow);
    assert!(terminal_text(&narrow_terminal).contains("01"));
    assert_eq!(runtime.app().last_action, None);

    runtime.app_mut().items.clear();
    render(&mut runtime, &mut narrow_terminal, narrow);
    assert!(runtime.area_of(IMPORT_STEPS).is_some());
    assert_eq!(runtime.app().last_action, None);

    runtime
        .prepare(Rect::new(0, 0, 0, 0))
        .expect("zero-sized composition is valid");
    let resized = Rect::new(0, 0, 36, 5);
    let mut resized_terminal =
        Terminal::new(TestBackend::new(resized.width, resized.height)).unwrap();
    render(&mut runtime, &mut resized_terminal, resized);
    assert!(within(
        resized,
        runtime
            .area_of(IMPORT_STEPS)
            .expect("steps layout refreshes after zero-area resize")
    ));
    assert_eq!(runtime.app().last_action, None);
}

#[test]
fn public_steps_intrinsic_measurement_uses_live_labels_metadata_and_ordinals() {
    let mut app = ImportConsumer::new();
    app.items = vec![ImportStep {
        key: 0,
        label: "Upload files",
        state: StepState::Queued,
        elapsed: Some("transferred 2048 files"),
    }];
    let root = Rect::new(0, 0, 100, 4);
    let mut runtime = Runtime::new(app, Theme::junie());
    let mut terminal = Terminal::new(TestBackend::new(root.width, root.height)).unwrap();
    render(&mut runtime, &mut terminal, root);
    assert_eq!(runtime.app().measured_steps.get().preferred.0, 43);
    assert!(terminal_text(&terminal).contains("transferred 2048 files"));

    runtime.app_mut().items = (0..105)
        .map(|key| ImportStep {
            key,
            label: "Upload files",
            state: StepState::Queued,
            elapsed: Some("transferred 2048 files"),
        })
        .collect();
    render(&mut runtime, &mut terminal, root);
    assert_eq!(runtime.app().measured_steps.get().preferred.0, 44);
}

#[test]
fn public_steps_custom_non_display_row_has_explicit_intrinsic_label_width() {
    let mut app = ImportConsumer::new();
    app.items = vec![ImportStep {
        key: 8,
        label: "Archive historical package objects",
        state: StepState::Queued,
        elapsed: Some("copied from storage archive"),
    }];
    let root = Rect::new(0, 0, 100, 4);
    let mut runtime = Runtime::new(app, Theme::junie());
    let mut terminal = Terminal::new(TestBackend::new(root.width, root.height)).unwrap();
    render(&mut runtime, &mut terminal, root);

    let expected = 6usize
        + usize::from(termrock::text::width("Archive historical package objects"))
        + usize::from(termrock::text::width("copied from storage archive"))
        + 3;
    assert_eq!(
        runtime.app().measured_steps.get().preferred.0,
        expected as u16
    );
    let text = terminal_text(&terminal);
    assert!(text.contains("Archive historical package objects"));
    assert!(text.contains("copied from storage archive"));
}

#[test]
fn numbered_steps_format_ordinals_above_two_digits() {
    let root = Rect::new(0, 0, 28, 104);
    let mut app = ImportConsumer::new();
    app.items = (0..104)
        .map(|index| ImportStep {
            key: index,
            label: "Import file",
            state: StepState::Queued,
            elapsed: None,
        })
        .collect();
    let mut runtime = Runtime::new(app, Theme::junie());
    let mut terminal = Terminal::new(TestBackend::new(root.width, root.height)).unwrap();
    render(&mut runtime, &mut terminal, root);
    assert!(terminal_text(&terminal).contains("100"));
    assert!(terminal_text(&terminal).contains("104"));
}

const DEFAULT_DISPLAY_STEPS: Id = Id::root("external.default_display_steps");

struct DefaultDisplayItem {
    label: String,
    metadata: &'static str,
}

impl core::fmt::Display for DefaultDisplayItem {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.label)
    }
}

fn default_display_step_state(_: &DefaultDisplayItem) -> StepState {
    StepState::Queued
}

fn default_display_metadata(item: &DefaultDisplayItem) -> Option<&str> {
    Some(item.metadata)
}

struct DefaultDisplaySteps {
    state: StepsState,
    items: Vec<DefaultDisplayItem>,
    measured: Cell<Size>,
}

impl App for DefaultDisplaySteps {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Steps::new(DEFAULT_DISPLAY_STEPS)
            .step(&default_display_step_state)
            .numbered(true)
            .update(cx, &mut self.state, &self.items)
            .erase()
    }

    fn draw(&self, ui: &mut termrock::Ui<'_>) {
        let steps = Steps::new(DEFAULT_DISPLAY_STEPS)
            .step(&default_display_step_state)
            .numbered(true)
            .metadata(&default_display_metadata);
        let composition =
            composition::Group::row([composition::steps(&steps, &self.state, &self.items)]);
        self.measured.set(
            composition
                .measure(ui, Constraints::loose(200, 120))
                .expect("default Display steps fit the composition depth"),
        );
        composition
            .show(ui)
            .expect("default Display steps fit the composition depth");
    }
}

#[test]
fn public_default_display_steps_measure_the_actual_label_without_custom_projection() {
    let label = "Archive historical package objects";
    let root = Rect::new(0, 0, 100, 4);
    let mut runtime = Runtime::new(
        DefaultDisplaySteps {
            state: StepsState::new(),
            items: vec![DefaultDisplayItem {
                label: label.to_owned(),
                metadata: "copied from storage archive",
            }],
            measured: Cell::new(Size::default()),
        },
        Theme::junie(),
    );
    let mut terminal = Terminal::new(TestBackend::new(root.width, root.height)).unwrap();
    render(&mut runtime, &mut terminal, root);

    let expected = 6usize
        + usize::from(termrock::text::width(label))
        + usize::from(termrock::text::width("copied from storage archive"))
        + 3;
    assert_eq!(runtime.app().measured.get().preferred.0, expected as u16);
    let text = terminal_text(&terminal);
    assert!(text.contains(label), "default Display row clipped:\n{text}");
    assert!(text.contains("copied from storage archive"));
}
