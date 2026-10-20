use termrock::{
    App, AsItem, Buffer, Cx, Id, Input, Item, ItemKey, KeyModifiers, Mouse, MouseKind, Part,
    PartRef, Picker, PickerAction, PickerSlotExt, PickerState, Position, Rect, Response, Runtime,
    Theme, Ui,
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

fn publish(runtime: &mut Runtime<FilePicker>, screen: Rect, buffer: &mut Buffer) {
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
