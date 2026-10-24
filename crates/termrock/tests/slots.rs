use termrock::author::LayerSlotAuthor;
use termrock::{
    App, AsItem, Buffer, ColorLevel, Cx, DialogSlotExt, Id, Input, Item, ItemKey, KeyModifiers,
    Mouse, MouseKind, Part, PartRef, Picker, PickerAction, PickerSlotExt, PickerState, Position,
    PropsSlotExt, Rect, Response, Runtime, Theme, Ui,
};

const PICKER: Id = Id::root("external-consumer.file-picker");

struct FilePicker {
    items: Vec<FileItem>,
    state: PickerState,
    chosen: Option<ItemKey>,
}

struct FileItem {
    path: String,
}

impl AsItem for FileItem {
    fn as_item(&self) -> Item<'_> {
        Item::new(ItemKey::text(&self.path), &self.path)
    }
}

fn picker() -> Picker<'static, FileItem> {
    Picker::new(PICKER).title("Open a file")
}

fn publish<A: App>(runtime: &mut Runtime<A>, screen: Rect, buffer: &mut Buffer) {
    for _ in 0..16 {
        if runtime.needs_settle() {
            let _ = runtime.settle();
        }
        if runtime.needs_present() {
            runtime.draw_buffer(screen, buffer).commit_presented();
        }
        if !runtime.needs_settle() && !runtime.needs_present() {
            return;
        }
    }
    panic!("picker focus did not settle");
}

impl App for FilePicker {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let picker = picker();
        picker.reconcile(&mut self.state, &self.items);
        if !cx.is_open(PICKER) {
            cx.open_layer(PICKER, picker.layer(cx, &self.items));
        }
        let response = picker.update(cx, &mut self.state, &self.items);
        if let Some(PickerAction::Chosen(key)) = response.action_ref().copied() {
            self.chosen = Some(key);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.layer_slot(PICKER, |slot| {
            slot.picker(picker(), &self.state, &self.items);
        });
    }
}

#[test]
fn public_picker_slot_draws_facade_component_with_domain_data() {
    let app = FilePicker {
        items: vec![
            FileItem {
                path: "src/main.rs".to_owned(),
            },
            FileItem {
                path: "Cargo.toml".to_owned(),
            },
        ],
        state: PickerState::default(),
        chosen: None,
    };
    let mut runtime = Runtime::new(app, Theme::junie());
    let _ = runtime.initialize();

    let initial_screen = Rect::new(0, 0, 80, 24);
    let mut initial_buffer = Buffer::empty(initial_screen);
    runtime
        .draw_buffer(initial_screen, &mut initial_buffer)
        .commit_presented();

    let screen = Rect::new(0, 0, 100, 30);
    let _ = runtime
        .handle(Input::Resize(screen.width, screen.height))
        .unwrap();
    let click = Input::Mouse(Mouse {
        kind: MouseKind::Down,
        pos: Position::new(0, 0),
        mods: KeyModifiers::NONE,
    });
    assert!(runtime.handle(click).is_err());

    let mut buffer = Buffer::empty(screen);
    runtime.draw_buffer(screen, &mut buffer).commit_presented();

    let mut rendered = String::new();
    for y in 0..screen.height {
        for x in 0..screen.width {
            if let Some(cell) = buffer.cell((x, y)) {
                rendered.push_str(cell.symbol());
            }
        }
    }
    assert!(rendered.contains("Open a file"));
    assert!(rendered.contains("src/main.rs"));
    assert!(rendered.contains("Cargo.toml"));

    let chosen_key = ItemKey::text("Cargo.toml");
    let row = runtime
        .registry()
        .regions()
        .iter()
        .find(|region| region.part == PartRef::item(Part::ROW, chosen_key))
        .expect("the component renderer must publish the row hit area");
    let position = Position::new(row.area.x, row.area.y);
    let _ = runtime
        .handle(Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos: position,
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    publish(&mut runtime, screen, &mut buffer);
    let _ = runtime
        .handle(Input::Mouse(Mouse {
            kind: MouseKind::Up,
            pos: position,
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    assert_eq!(runtime.app().chosen, Some(chosen_key));
}

const INFO: Id = Id::root("external-consumer.info-dialog");
const INFO_PROPS: Id = Id::root("external-consumer.info-properties");

#[derive(Clone)]
struct DeviceProperty {
    key: ItemKey,
    label: &'static str,
    value: &'static str,
    copyable: bool,
}

struct DeviceInfo {
    dialog: termrock::Dialog<'static>,
    dialog_state: termrock::DialogState,
    props: termrock::PropsList<'static>,
    props_state: termrock::PropsState,
    rows: std::sync::Arc<std::sync::Mutex<Vec<DeviceProperty>>>,
    copied: Option<ItemKey>,
    use_typed_slots: bool,
    clip_parent: bool,
}

impl DeviceInfo {
    fn rows(&self) -> Vec<termrock::PropsRow<'static>> {
        self.rows
            .lock()
            .expect("device properties remain available")
            .iter()
            .map(|row| {
                let property = termrock::PropsRow::new(row.key, row.label, row.value);
                if row.copyable {
                    property.copyable()
                } else {
                    property
                }
            })
            .collect()
    }
}

impl App for DeviceInfo {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if !cx.is_open(INFO) {
            cx.open_layer(INFO, self.dialog.layer(cx).initial_focus(INFO_PROPS));
        }
        let dialog = self.dialog.update(cx, &mut self.dialog_state);
        let rows = self.rows();
        let props = self.props.update(cx, &mut self.props_state, &rows);
        if let Some(termrock::PropsAction::Copy(key)) = props.action_ref().copied() {
            self.copied = Some(key);
        }
        dialog.erase() | props.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let rows = self.rows();
        ui.layer_slot(INFO, |slot| {
            if self.clip_parent {
                // Clip inside the layer's component-author boundary. Layer
                // dispatch establishes the layer viewport itself, so an
                // outer Ui::with_area would not model an inner parent clip.
                slot.render_component(|ui, area| {
                    ui.with_area(Rect::new(20, 7, 12, 4), |ui| {
                        if self.use_typed_slots {
                            self.dialog
                                .draw_body_slot(ui, area, &self.dialog_state, |body| {
                                    body.props_list(&self.props, &self.props_state, &rows);
                                });
                        } else {
                            self.dialog
                                .draw(ui, area, &self.dialog_state, |ui, body_area| {
                                    self.props.draw(ui, body_area, &self.props_state, &rows);
                                });
                        }
                    });
                });
            } else if self.use_typed_slots {
                slot.dialog(&self.dialog, &self.dialog_state, |body| {
                    body.props_list(&self.props, &self.props_state, &rows);
                });
            } else {
                slot.render_component(|ui, area| {
                    self.dialog
                        .draw(ui, area, &self.dialog_state, |ui, body_area| {
                            self.props.draw(ui, body_area, &self.props_state, &rows);
                        });
                });
            }
        });
    }
}

fn device_info(
    use_typed_slots: bool,
) -> (
    DeviceInfo,
    std::sync::Arc<std::sync::Mutex<Vec<DeviceProperty>>>,
) {
    let rows = std::sync::Arc::new(std::sync::Mutex::new(vec![
        DeviceProperty {
            key: ItemKey::text("serial"),
            label: "序列号",
            value: "ZX-91-🛰️",
            copyable: true,
        },
        DeviceProperty {
            key: ItemKey::text("region"),
            label: "Region",
            value: "東京・北",
            copyable: true,
        },
        DeviceProperty {
            key: ItemKey::text("owner"),
            label: "Owner",
            value: "read only",
            copyable: false,
        },
    ]));
    let app = DeviceInfo {
        dialog: termrock::Dialog::new(INFO)
            .title("Device details")
            .body_rows(4),
        dialog_state: termrock::DialogState::default(),
        props: termrock::PropsList::new(INFO_PROPS),
        props_state: termrock::PropsState::default(),
        rows: rows.clone(),
        copied: None,
        use_typed_slots,
        clip_parent: false,
    };
    (app, rows)
}

#[test]
fn public_dialog_and_props_slots_preserve_live_keyed_interaction() {
    let screen = Rect::new(0, 0, 60, 20);
    let (app, rows) = device_info(true);
    let mut runtime = Runtime::new(app, Theme::junie());
    let _ = runtime.initialize();
    let mut buffer = Buffer::empty(screen);
    runtime.draw_buffer(screen, &mut buffer).commit_presented();

    assert_eq!(runtime.focus(), Some(INFO_PROPS));
    assert!(
        runtime
            .registry()
            .regions()
            .iter()
            .any(|region| { region.owner == INFO_PROPS && region.part.part == Part::ROW })
    );
    assert_eq!(runtime.app().copied, None, "drawing has no domain effects");

    let _ = runtime
        .handle(Input::Key(termrock::Key {
            code: termrock::KeyCode::Down,
            mods: KeyModifiers::NONE,
        }))
        .expect("the list handles Down within the modal");
    assert_eq!(
        runtime.app().props_state.cursor(),
        Some(ItemKey::text("region"))
    );
    publish(&mut runtime, screen, &mut buffer);

    // Reorder domain rows while preserving stable keys. Reconciliation keeps
    // the current property's identity, then activation reports that key.
    rows.lock()
        .expect("device properties remain available")
        .swap(0, 1);
    let _ = runtime
        .handle(Input::Key(termrock::Key {
            code: termrock::KeyCode::Up,
            mods: KeyModifiers::NONE,
        }))
        .expect("the reordered list receives keyboard input");
    assert_eq!(
        runtime.app().props_state.cursor(),
        Some(ItemKey::text("region"))
    );
    publish(&mut runtime, screen, &mut buffer);
    let _ = runtime
        .handle(Input::Key(termrock::Key {
            code: termrock::KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }))
        .expect("the focused property handles its copy command");
    assert_eq!(runtime.app().copied, Some(ItemKey::text("region")));
    assert_eq!(
        runtime.app().props_state.cursor(),
        Some(ItemKey::text("region"))
    );
    assert!(runtime.is_open(INFO), "copy keeps the owning dialog open");
}

fn frame_cells(
    buffer: &Buffer,
    area: Rect,
) -> Vec<(
    String,
    ratatui::style::Color,
    ratatui::style::Color,
    ratatui::style::Modifier,
)> {
    let mut cells = Vec::with_capacity(usize::from(area.width) * usize::from(area.height));
    for y in area.y..area.bottom() {
        for x in area.x..area.right() {
            if let Some(cell) = buffer.cell((x, y)) {
                cells.push((cell.symbol().to_owned(), cell.fg, cell.bg, cell.modifier));
            }
        }
    }
    cells
}

#[test]
fn typed_dialog_and_props_slots_match_existing_component_renderers() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        for screen in [Rect::new(0, 0, 60, 20), Rect::new(0, 0, 24, 8)] {
            let (typed_app, _) = device_info(true);
            let (author_app, _) = device_info(false);
            let theme = Theme::junie().for_level(level);
            let mut typed = Runtime::new(typed_app, theme.clone());
            let mut author = Runtime::new(author_app, theme);
            let _ = typed.initialize();
            let _ = author.initialize();
            let mut typed_buffer = Buffer::empty(screen);
            let mut author_buffer = Buffer::empty(screen);
            typed
                .draw_buffer(screen, &mut typed_buffer)
                .commit_presented();
            author
                .draw_buffer(screen, &mut author_buffer)
                .commit_presented();

            assert_eq!(
                frame_cells(&typed_buffer, screen),
                frame_cells(&author_buffer, screen)
            );
            assert_eq!(typed.registry().regions(), author.registry().regions());
            assert_eq!(typed.focus(), author.focus());
            assert_eq!(typed.cursor(), author.cursor());
        }
    }
}

#[test]
fn clipped_dialog_slot_preserves_logical_body_geometry_and_regions() {
    let screen = Rect::new(0, 0, 60, 20);
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let (mut typed_app, _) = device_info(true);
        typed_app.clip_parent = true;
        let (mut direct_app, _) = device_info(false);
        direct_app.clip_parent = true;
        let theme = Theme::junie().for_level(level);
        let mut typed = Runtime::new(typed_app, theme.clone());
        let mut direct = Runtime::new(direct_app, theme);
        let _ = typed.initialize();
        let _ = direct.initialize();
        let mut typed_buffer = Buffer::empty(screen);
        let mut direct_buffer = Buffer::empty(screen);
        typed
            .draw_buffer(screen, &mut typed_buffer)
            .commit_presented();
        direct
            .draw_buffer(screen, &mut direct_buffer)
            .commit_presented();

        assert_eq!(
            frame_cells(&typed_buffer, screen),
            frame_cells(&direct_buffer, screen),
            "cell output must match with parent clipping at {level:?}"
        );
        assert_eq!(
            typed.registry().regions(),
            direct.registry().regions(),
            "registered regions must match with parent clipping at {level:?}"
        );
        assert_eq!(typed.focus(), direct.focus());
        assert_eq!(typed.cursor(), direct.cursor());
    }
}

const DEVICE_PROPERTIES: Id = Id::root("external-consumer.device-properties");
fn device_property_rows() -> [termrock::PropsRow<'static>; 3] {
    [
        termrock::PropsRow::new(ItemKey::text("serial"), "序列号", "ZX-91-🛰️").copyable(),
        termrock::PropsRow::new(ItemKey::text("region"), "Region", "東京・北").copyable(),
        termrock::PropsRow::new(ItemKey::text("owner"), "Owner", "read only"),
    ]
}

struct DeviceProperties {
    list: termrock::PropsList<'static>,
    state: termrock::PropsState,
    copied: Option<ItemKey>,
}

const COMMAND_DIALOG: Id = Id::root("external-consumer.command-dialog");
const COMMAND_PROPS: Id = Id::root("external-consumer.command-properties");
const COMMAND_BUTTON: Id = Id::root("external-consumer.command-background-button");
const COMMAND_SCOPE: Id = Id::root("external-consumer.command-scope");
const COMMAND_I: termrock::ActionKey = termrock::ActionKey::custom("external.command.inspect");
const COMMAND_X: termrock::ActionKey = termrock::ActionKey::custom("external.command.export");

#[derive(Clone, Copy)]
enum Command {
    Inspect,
    Export,
}

const COMMAND_BINDINGS: &[termrock::Binding<Command>] = &[
    termrock::Binding {
        action: COMMAND_I,
        chord: Some(termrock::Chord::key(termrock::KeyCode::Char('i'))),
        cmd: Command::Inspect,
        label: "Inspect",
        priority: 1,
        visible: false,
    },
    termrock::Binding {
        action: COMMAND_X,
        chord: Some(termrock::Chord::key(termrock::KeyCode::Char('x'))),
        cmd: Command::Export,
        label: "Export",
        priority: 1,
        visible: false,
    },
];

fn remapped_command_keymap() -> &'static termrock::KeyMap {
    static KEYMAP: std::sync::OnceLock<termrock::KeyMap> = std::sync::OnceLock::new();
    KEYMAP.get_or_init(|| {
        let mut keymap = termrock::KeyMap::new();
        keymap.remap_component(
            COMMAND_SCOPE,
            COMMAND_I,
            termrock::Chord::key(termrock::KeyCode::Char('r')),
        );
        keymap.remove_component(COMMAND_SCOPE, COMMAND_X);
        keymap
    })
}

struct CommandScopeDialog {
    dialog: termrock::Dialog<'static>,
    dialog_state: termrock::DialogState,
    props: termrock::PropsList<'static>,
    props_state: termrock::PropsState,
    remapped: bool,
    commands_enabled: bool,
    author_scope: bool,
    inspected: usize,
    exported: usize,
}

impl App for CommandScopeDialog {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == termrock::UpdateCause::Bootstrap {
            cx.focus(COMMAND_BUTTON);
            cx.open_layer(
                COMMAND_DIALOG,
                self.dialog.layer(cx).initial_focus(COMMAND_PROPS),
            );
        }

        let button = termrock::Button::new(COMMAND_BUTTON, "Open device details").update(cx);
        if button.action_ref().is_some() && !cx.is_open(COMMAND_DIALOG) {
            cx.open_layer(
                COMMAND_DIALOG,
                self.dialog.layer(cx).initial_focus(COMMAND_PROPS),
            );
        }

        let mut handled_scope_command = false;
        for intent in cx.intents(COMMAND_SCOPE) {
            if let termrock::Intent::Binding(action) = intent {
                match termrock::Binding::command(COMMAND_BINDINGS, action) {
                    Some(Command::Inspect) => self.inspected += 1,
                    Some(Command::Export) => self.exported += 1,
                    None => continue,
                }
                handled_scope_command = true;
            }
        }

        let rows = device_property_rows();
        let props = self.props.update(cx, &mut self.props_state, &rows);
        let dialog = self.dialog.update(cx, &mut self.dialog_state);
        button.erase()
            | props.erase()
            | dialog.erase()
            | if handled_scope_command {
                Response::consumed()
            } else {
                Response::ignored()
            }
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let button = termrock::Button::new(COMMAND_BUTTON, "Open device details");
        static NO_COMMANDS: &[termrock::Binding<Command>] = &[];
        let bindings = if self.commands_enabled {
            COMMAND_BINDINGS
        } else {
            NO_COMMANDS
        };
        if self.author_scope {
            ui.focus_scope(COMMAND_SCOPE, termrock::ScopeMode::Normal, |ui| {
                termrock::composition::Group::row([termrock::composition::button(&button)])
                    .show(ui)
                    .expect("the single button fits the root composition");
                termrock::author::publish_scope_bindings(
                    ui,
                    termrock::ScopeId::new(COMMAND_SCOPE),
                    COMMAND_SCOPE,
                    termrock::StateFlags::empty(),
                    bindings,
                );
            });
        } else {
            ui.command_scope(COMMAND_SCOPE, bindings, |ui| {
                termrock::composition::Group::row([termrock::composition::button(&button)])
                    .show(ui)
                    .expect("the single button fits the root composition");
            });
        }
        let _ = ui.layer(COMMAND_DIALOG, |ui, area| {
            self.dialog
                .draw_body_slot(ui, area, &self.dialog_state, |body| {
                    body.props_list(&self.props, &self.props_state, &device_property_rows());
                });
        });
    }

    fn keymap(&self) -> &termrock::KeyMap {
        if self.remapped {
            remapped_command_keymap()
        } else {
            termrock::KeyMap::EMPTY_REF
        }
    }
}

fn command_scope_dialog(remapped: bool) -> CommandScopeDialog {
    CommandScopeDialog {
        dialog: termrock::Dialog::new(COMMAND_DIALOG)
            .title("Device details")
            .body_rows(4),
        dialog_state: termrock::DialogState::default(),
        props: termrock::PropsList::new(COMMAND_PROPS),
        props_state: termrock::PropsState::default(),
        remapped,
        commands_enabled: true,
        author_scope: false,
        inspected: 0,
        exported: 0,
    }
}

#[test]
fn public_command_scope_matches_existing_author_primitives() {
    let screen = Rect::new(0, 0, 60, 20);
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        for screen in [screen, Rect::new(0, 0, 24, 8)] {
            let scoped_app = command_scope_dialog(false);
            let mut author_app = command_scope_dialog(false);
            author_app.author_scope = true;
            let theme = Theme::junie().for_level(level);
            let mut scoped = Runtime::new(scoped_app, theme.clone());
            let mut author = Runtime::new(author_app, theme);
            let _ = scoped.initialize();
            let _ = author.initialize();
            let mut scoped_buffer = Buffer::empty(screen);
            let mut author_buffer = Buffer::empty(screen);
            publish(&mut scoped, screen, &mut scoped_buffer);
            publish(&mut author, screen, &mut author_buffer);

            assert_eq!(
                frame_cells(&scoped_buffer, screen),
                frame_cells(&author_buffer, screen),
                "the API wrapper must not change cells in {level:?}"
            );
            assert_eq!(scoped.registry().regions(), author.registry().regions());
            assert_eq!(scoped.focus(), author.focus());
            assert_eq!(scoped.cursor(), author.cursor());
        }
    }
}

fn key(code: termrock::KeyCode) -> Input {
    Input::Key(termrock::Key {
        code,
        mods: KeyModifiers::NONE,
    })
}

#[test]
fn public_command_scope_routes_only_within_focused_layer_and_restores_focus() {
    let screen = Rect::new(0, 0, 60, 20);
    let mut runtime = Runtime::new(command_scope_dialog(false), Theme::junie());
    let _ = runtime.initialize();
    let mut buffer = Buffer::empty(screen);
    publish(&mut runtime, screen, &mut buffer);
    assert_eq!(runtime.focus(), Some(COMMAND_PROPS));

    let narrow = Rect::new(0, 0, 24, 8);
    let _ = runtime
        .handle(Input::Resize(narrow.width, narrow.height))
        .unwrap();
    let mut narrow_buffer = Buffer::empty(narrow);
    publish(&mut runtime, narrow, &mut narrow_buffer);
    assert_eq!(
        runtime.focus(),
        None,
        "a narrow modal may have no focus target"
    );
    let _ = runtime
        .handle(Input::Resize(screen.width, screen.height))
        .unwrap();
    publish(&mut runtime, screen, &mut buffer);
    assert_eq!(runtime.focus(), Some(COMMAND_PROPS));

    for code in [termrock::KeyCode::Char('i'), termrock::KeyCode::Char('x')] {
        let _ = runtime.handle(key(code)).unwrap();
        publish(&mut runtime, screen, &mut buffer);
    }
    assert_eq!(
        runtime.app().inspected,
        0,
        "background scope is inert under modal"
    );
    assert_eq!(
        runtime.app().exported,
        0,
        "background scope is inert under modal"
    );

    for cycle in 0..3 {
        let _ = runtime.handle(key(termrock::KeyCode::Esc)).unwrap();
        publish(&mut runtime, screen, &mut buffer);
        assert_eq!(runtime.focus(), Some(COMMAND_BUTTON));
        let _ = runtime.handle(key(termrock::KeyCode::Char('i'))).unwrap();
        publish(&mut runtime, screen, &mut buffer);
        let _ = runtime.handle(key(termrock::KeyCode::Char('x'))).unwrap();
        publish(&mut runtime, screen, &mut buffer);
        assert_eq!(runtime.app().inspected, runtime.app().exported);
        if cycle == 0 {
            let button = runtime
                .registry()
                .regions()
                .iter()
                .find(|region| region.owner == COMMAND_BUTTON)
                .expect("the background button is the actual pointer target");
            let point = Position::new(
                button.area.x + button.area.width.saturating_sub(1) / 2,
                button.area.y,
            );
            let _ = runtime
                .handle(Input::Mouse(Mouse {
                    kind: MouseKind::Down,
                    pos: point,
                    mods: KeyModifiers::NONE,
                }))
                .unwrap();
            publish(&mut runtime, screen, &mut buffer);
            let _ = runtime
                .handle(Input::Mouse(Mouse {
                    kind: MouseKind::Up,
                    pos: point,
                    mods: KeyModifiers::NONE,
                }))
                .unwrap();
        } else {
            let _ = runtime.handle(key(termrock::KeyCode::Enter)).unwrap();
        }
        publish(&mut runtime, screen, &mut buffer);
        assert_eq!(runtime.focus(), Some(COMMAND_PROPS));
        for code in [termrock::KeyCode::Char('i'), termrock::KeyCode::Char('x')] {
            let _ = runtime.handle(key(code)).unwrap();
            publish(&mut runtime, screen, &mut buffer);
        }
        assert_eq!(runtime.app().inspected, cycle + 1);
        assert_eq!(runtime.app().exported, cycle + 1);
    }
}

#[test]
fn public_command_scope_keeps_component_remap_and_remove_overrides() {
    let screen = Rect::new(0, 0, 60, 20);
    let mut runtime = Runtime::new(command_scope_dialog(true), Theme::junie());
    let _ = runtime.initialize();
    let mut buffer = Buffer::empty(screen);
    publish(&mut runtime, screen, &mut buffer);

    let _ = runtime.handle(key(termrock::KeyCode::Esc)).unwrap();
    publish(&mut runtime, screen, &mut buffer);
    let _ = runtime.handle(key(termrock::KeyCode::Char('i'))).unwrap();
    publish(&mut runtime, screen, &mut buffer);
    let _ = runtime.handle(key(termrock::KeyCode::Char('r'))).unwrap();
    publish(&mut runtime, screen, &mut buffer);
    let _ = runtime.handle(key(termrock::KeyCode::Char('x'))).unwrap();
    publish(&mut runtime, screen, &mut buffer);

    assert_eq!(
        runtime.app().inspected,
        1,
        "remapped chord activates the binding"
    );
    assert_eq!(
        runtime.app().exported,
        0,
        "removed component binding stays absent"
    );
}

#[test]
fn public_command_scope_does_not_forge_focus_without_rendered_controls() {
    let screen = Rect::ZERO;
    let mut runtime = Runtime::new(command_scope_dialog(false), Theme::junie());
    let _ = runtime.initialize();
    let mut buffer = Buffer::empty(screen);
    publish(&mut runtime, screen, &mut buffer);
    assert_eq!(runtime.focus(), None);
    let _ = runtime.handle(key(termrock::KeyCode::Char('i'))).unwrap();
    publish(&mut runtime, screen, &mut buffer);
    assert_eq!(runtime.focus(), None);
    assert_eq!(runtime.app().inspected, 0);
    assert_eq!(runtime.app().exported, 0);
}

#[test]
fn public_command_scope_empty_table_disables_domain_actions() {
    let screen = Rect::new(0, 0, 60, 20);
    let mut app = command_scope_dialog(false);
    app.commands_enabled = false;
    let mut runtime = Runtime::new(app, Theme::junie());
    let _ = runtime.initialize();
    let mut buffer = Buffer::empty(screen);
    publish(&mut runtime, screen, &mut buffer);
    let _ = runtime.handle(key(termrock::KeyCode::Esc)).unwrap();
    publish(&mut runtime, screen, &mut buffer);

    for code in [termrock::KeyCode::Char('i'), termrock::KeyCode::Char('x')] {
        let _ = runtime.handle(key(code)).unwrap();
        publish(&mut runtime, screen, &mut buffer);
    }

    assert_eq!(runtime.focus(), Some(COMMAND_BUTTON));
    assert_eq!(runtime.app().inspected, 0);
    assert_eq!(runtime.app().exported, 0);
}

impl App for DeviceProperties {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let rows = device_property_rows();
        let response = self.list.update(cx, &mut self.state, &rows);
        if let Some(termrock::PropsAction::Copy(key)) = response.action_ref().copied() {
            self.copied = Some(key);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let rows = device_property_rows();
        termrock::composition::Group::column([termrock::composition::props_list(
            &self.list,
            &self.state,
            &rows,
        )])
        .show(ui)
        .expect("single property component fits supported composition depth");
    }
}

#[test]
fn public_composition_adapts_props_list_with_different_unicode_data() {
    let screen = Rect::new(0, 0, 36, 8);
    let app = DeviceProperties {
        list: termrock::PropsList::new(DEVICE_PROPERTIES),
        state: termrock::PropsState::default(),
        copied: None,
    };
    let mut runtime = Runtime::new(app, Theme::junie());
    let _ = runtime.initialize();
    let mut buffer = Buffer::empty(screen);
    runtime.draw_buffer(screen, &mut buffer).commit_presented();

    assert!(runtime.area_of(DEVICE_PROPERTIES).is_some());
    assert_eq!(
        runtime.app().copied,
        None,
        "composition has no domain effects"
    );
    let mut rendered = String::new();
    for y in 0..screen.height {
        for x in 0..screen.width {
            if let Some(cell) = buffer.cell((x, y)) {
                rendered.push_str(cell.symbol());
            }
        }
    }
    assert!(rendered.contains("序 列 号"));
    assert!(rendered.contains("ZX-91"));
    assert!(rendered.contains("東 京"));
}
