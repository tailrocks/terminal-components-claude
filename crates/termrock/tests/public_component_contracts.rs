use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use termrock::composition::{self, Group};
use termrock::{
    App, AsItem, Button, Cx, Focusability, Id, Input, Item, ItemKey, Key, KeyCode, KeyModifiers,
    Mouse, MouseKind, Part, PartRef, Picker, PickerAction, PickerSlotExt, PickerState, PropsList,
    PropsRow, PropsState, Response, Runtime, SelectionPolicy, Theme, Ui, UpdateCause,
};

const CLICK_ONLY: Id = Id::root("public.contracts.click-only");
const FOCUSABLE: Id = Id::root("public.contracts.focusable");
const SHEET: Id = Id::root("public.contracts.property-sheet");
const PICKER_ID: Id = Id::root("public.contracts.scope-picker");
const SCOPES: &[termrock::ScopeKey] = &[termrock::ScopeKey::new(0), termrock::ScopeKey::new(1)];
const FIRST_SCOPE: &[Resource] = &[
    Resource::new(11, "Résumé — 日本語"),
    Resource::new(23, "Workspace notes"),
    Resource::new(37, "Terminal layout"),
];
const SECOND_SCOPE: &[Resource] = &[
    Resource::disabled(41, "Unavailable mirror"),
    Resource::new(43, "東京 workspace"),
    Resource::new(47, "Release notes"),
];
const PROPERTIES: &[PropsRow<'static>] = &[
    PropsRow::new(ItemKey::num(101), "Engine", "PostgreSQL"),
    PropsRow::new(
        ItemKey::num(102),
        "Region",
        "München · e\u{301} · 東京 · 東京 — edge replica with a generated identity",
    )
    .wrap(),
];
const SCREEN: Rect = Rect::new(0, 0, 64, 16);

#[derive(Clone, Copy)]
struct Resource {
    key: u64,
    label: &'static str,
    disabled: bool,
}

impl Resource {
    const fn new(key: u64, label: &'static str) -> Self {
        Self {
            key,
            label,
            disabled: false,
        }
    }

    const fn disabled(key: u64, label: &'static str) -> Self {
        Self {
            key,
            label,
            disabled: true,
        }
    }
}

impl AsItem for Resource {
    fn as_item(&self) -> Item<'_> {
        Item::new(ItemKey::num(self.key), self.label).disabled(self.disabled)
    }
}

fn picker() -> Picker<'static, Resource> {
    Picker::new(PICKER_ID)
        .title("Choose a workspace resource")
        .searchable(false)
        .scopes(SCOPES)
        .selection_policy(SelectionPolicy::FirstEnabledOnFilterChange)
}

struct ComponentConsumer {
    pointer_activations: usize,
    keyboard_activations: usize,
    props_state: PropsState,
}

impl Default for ComponentConsumer {
    fn default() -> Self {
        let mut props_state = PropsState::default();
        props_state.set_cursor(1, ItemKey::num(102));
        Self {
            pointer_activations: 0,
            keyboard_activations: 0,
            props_state,
        }
    }
}

impl ComponentConsumer {
    fn click_only_button(&self) -> Button<'static> {
        Button::new(CLICK_ONLY, "Open resource").focusability(Focusability::ClickOnly)
    }

    fn focusable_button(&self) -> Button<'static> {
        Button::new(FOCUSABLE, "Keyboard action")
    }

    fn sheet(&self) -> PropsList<'static> {
        PropsList::new(SHEET).sheet()
    }
}

impl App for ComponentConsumer {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let click = self.click_only_button().update(cx);
        if click.activated() {
            self.pointer_activations = self.pointer_activations.saturating_add(1);
            return click.erase();
        }

        let keyboard = self.focusable_button().update(cx);
        if keyboard.activated() {
            self.keyboard_activations = self.keyboard_activations.saturating_add(1);
        }
        keyboard.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let click_only = self.click_only_button();
        let sheet = self.sheet();
        Group::column([
            composition::button(&click_only).min(16, 1),
            composition::button(&self.focusable_button()).min(16, 1),
            composition::props_list(&sheet, &self.props_state, PROPERTIES).flex(1),
        ])
        .show(ui)
        .expect("public composition should resolve the consumer components");
    }
}

struct PickerConsumer {
    state: PickerState,
    items: Vec<Resource>,
    scope_action: Option<termrock::ScopeKey>,
}

impl PickerConsumer {
    fn new() -> Self {
        let mut state = PickerState::default();
        let items = FIRST_SCOPE.to_vec();
        let component = picker();
        component.reconcile(&mut state, &items);
        state.set_cursor(2, ItemKey::num(37));
        Self {
            state,
            items,
            scope_action: None,
        }
    }
}

impl App for PickerConsumer {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let component = picker();
        component.reconcile(&mut self.state, &self.items);
        if cx.update_cause() == UpdateCause::Bootstrap {
            cx.open_layer(PICKER_ID, component.layer(cx, &self.items));
        }

        let mut response = component.update(cx, &mut self.state, &self.items);
        if let Some(PickerAction::Scope(scope)) = response.take_action() {
            self.scope_action = Some(scope);
            self.items = SECOND_SCOPE.to_vec();
            component.reconcile(&mut self.state, &self.items);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.layer_slot(PICKER_ID, |slot| {
            slot.picker(picker(), &self.state, &self.items);
        });
    }
}

fn present<A: App>(runtime: &mut Runtime<A>) -> Buffer {
    let mut buffer = Buffer::empty(SCREEN);
    for _ in 0..16 {
        if runtime.needs_settle() {
            let _ = runtime.settle();
        }
        if runtime.needs_present() {
            runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        }
        if !runtime.needs_settle() && !runtime.needs_present() {
            return buffer;
        }
    }
    panic!("public consumer did not settle");
}

fn rendered_text(buffer: &Buffer) -> String {
    let area = *buffer.area();
    let mut text = String::new();
    for y in area.y..area.y.saturating_add(area.height) {
        for x in area.x..area.x.saturating_add(area.width) {
            if let Some(cell) = buffer.cell(Position::new(x, y)) {
                text.push_str(cell.symbol());
            }
        }
    }
    text
}

#[test]
fn facade_consumer_composes_button_and_unicode_property_sheet() {
    let mut runtime = Runtime::new(ComponentConsumer::default(), Theme::junie());
    let _ = runtime.initialize();
    let _ = present(&mut runtime);
    runtime.set_focus(Some(SHEET));
    let buffer = present(&mut runtime);

    assert!(!runtime.ring().is_registered(CLICK_ONLY));
    assert!(!runtime.ring().contains(CLICK_ONLY));
    assert!(runtime.ring().contains(FOCUSABLE));
    let row = runtime
        .area_of_part(SHEET, PartRef::item(Part::ROW, ItemKey::num(102)))
        .expect("sheet preserves keyed row identity");
    assert!(
        row.height > 1,
        "wrapped row should own multiline bounds: {row:?}"
    );

    let text = rendered_text(&buffer);
    assert!(text.contains("Open resource"), "{text:?}");
    assert!(text.contains("Engine"), "{text:?}");
    assert!(text.contains("PostgreSQL"), "{text:?}");
    assert!(text.contains('東'), "{text:?}");
    assert!(text.contains('京'), "{text:?}");
    assert!(text.contains("replica"), "{text:?}");
}

#[test]
fn facade_click_only_button_keeps_pointer_action_without_keyboard_focus() {
    let mut runtime = Runtime::new(ComponentConsumer::default(), Theme::junie());
    let _ = runtime.initialize();
    let _ = present(&mut runtime);
    let area = runtime
        .area_of(CLICK_ONLY)
        .expect("button has a live hit area");
    let position = Position::new(area.x.saturating_add(1), area.y);

    for kind in [MouseKind::Down, MouseKind::Up] {
        let input = Input::Mouse(Mouse {
            kind,
            pos: position,
            mods: KeyModifiers::NONE,
        });
        let _ = runtime.handle(input).expect("button accepts pointer input");
        let _ = present(&mut runtime);
    }

    assert_eq!(runtime.app().pointer_activations, 1);
    assert_ne!(runtime.focus(), Some(CLICK_ONLY));
    assert!(!runtime.ring().contains(CLICK_ONLY));
}

#[test]
fn facade_picker_scope_change_resets_to_first_enabled_and_reorder_keeps_key() {
    let mut runtime = Runtime::new(PickerConsumer::new(), Theme::junie());
    let _ = runtime.initialize();
    let _ = present(&mut runtime);
    assert_eq!(runtime.app().state.cursor(), Some(ItemKey::num(37)));

    let input = Input::Key(Key {
        code: KeyCode::BackTab,
        mods: KeyModifiers::NONE,
    });
    let _ = runtime
        .handle(input)
        .expect("picker accepts previous-scope key");
    let _ = present(&mut runtime);

    assert_eq!(runtime.app().scope_action, Some(termrock::ScopeKey::new(1)));
    assert_eq!(
        runtime.app().state.scope(SCOPES),
        Some(termrock::ScopeKey::new(1))
    );
    assert_eq!(runtime.app().state.cursor(), Some(ItemKey::num(43)));

    let component = picker();
    let mut state = PickerState::default();
    component.reconcile(&mut state, FIRST_SCOPE);
    state.set_cursor(2, ItemKey::num(37));
    component.reconcile(
        &mut state,
        &[FIRST_SCOPE[2], FIRST_SCOPE[0], FIRST_SCOPE[1]],
    );
    assert_eq!(state.cursor(), Some(ItemKey::num(37)));
    component.reconcile(&mut state, &[]);
    assert_eq!(state.cursor(), None);
}
