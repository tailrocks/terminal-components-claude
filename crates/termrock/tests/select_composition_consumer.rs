use termrock::{
    App, Buffer, Constraints, Cx, Id, Input, ItemKey, KeyModifiers, Mouse, MouseKind, Part,
    PartRef, Position, Rect, Response, Runtime, Select, SelectAction, SelectState, Theme, Ui,
    composition::{self, Group},
};

const SELECT: Id = Id::root("external-consumer.select-composition");
const ITEMS: [&str; 3] = ["Résumé", "東京の資料", "München · logs"];

fn select() -> Select<'static, &'static str> {
    Select::new(SELECT)
        .placeholder("Choose a document…")
        .popup_rows(4)
}

struct SelectConsumer {
    state: SelectState,
    chosen: Option<ItemKey>,
}

impl App for SelectConsumer {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let response = select().update(cx, &mut self.state, &ITEMS);
        if let Some(SelectAction::Chose(key)) = response.action_ref().copied() {
            self.chosen = Some(key);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let component = select();
        let group = Group::column([composition::select(&component, &self.state, &ITEMS)]);
        let state_before_measure = self.state.clone();
        let measured = group
            .measure(ui, Constraints::loose(80, 24))
            .expect("one select leaf has valid composition depth");
        assert!(measured.preferred.0 > 0);
        assert_eq!(self.state, state_before_measure, "measurement is read-only");

        // Empty bounds exercise the same component measurement without
        // registering a field, opening an overlay, or changing select state.
        let empty = group
            .measure(ui, Constraints::loose(0, 0))
            .expect("a zero-sized composition remains measurable");
        assert_eq!(empty.min, (0, 0));
        assert_eq!(
            self.state, state_before_measure,
            "empty measurement is read-only"
        );
        group
            .show(ui)
            .expect("one select leaf fits composition depth");
    }
}

fn key(code: termrock::KeyCode) -> Input {
    Input::Key(termrock::Key {
        code,
        mods: KeyModifiers::NONE,
    })
}

fn publish(runtime: &mut Runtime<SelectConsumer>, screen: Rect, buffer: &mut Buffer) {
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
    panic!("select composition did not settle");
}

fn text(buffer: &Buffer, screen: Rect) -> String {
    let mut rendered = String::new();
    for y in 0..screen.height {
        for x in 0..screen.width {
            if let Some(cell) = buffer.cell((x, y)) {
                rendered.push_str(cell.symbol());
            }
        }
    }
    rendered
}

#[test]
fn public_select_composition_keeps_live_component_layout_and_interactions() {
    let screen = Rect::new(0, 0, 80, 24);
    let mut state = SelectState::default();
    state.set_value(Some(ItemKey::index(0)));
    let app = SelectConsumer {
        state,
        chosen: None,
    };
    let mut runtime = Runtime::new(app, Theme::junie());
    let _ = runtime.initialize();
    let mut buffer = Buffer::empty(screen);
    publish(&mut runtime, screen, &mut buffer);

    assert_eq!(runtime.focus(), Some(SELECT));
    assert_eq!(runtime.app().state.value(), Some(ItemKey::index(0)));
    assert!(!runtime.app().state.is_open());
    assert!(text(&buffer, screen).contains("Résumé"));

    let _ = runtime.handle(key(termrock::KeyCode::Enter)).unwrap();
    publish(&mut runtime, screen, &mut buffer);
    assert!(runtime.app().state.is_open());
    let popup = text(&buffer, screen);
    assert!(popup.contains("東 京"));
    assert!(popup.contains("München · logs"));

    let _ = runtime.handle(key(termrock::KeyCode::Down)).unwrap();
    publish(&mut runtime, screen, &mut buffer);
    let _ = runtime.handle(key(termrock::KeyCode::Enter)).unwrap();
    publish(&mut runtime, screen, &mut buffer);
    assert_eq!(runtime.app().chosen, Some(ItemKey::index(1)));
    assert_eq!(runtime.app().state.value(), Some(ItemKey::index(1)));
    assert!(!runtime.app().state.is_open());

    let field = runtime
        .registry()
        .regions()
        .iter()
        .find(|region| region.part == PartRef::of(Part::FIELD))
        .expect("the select renderer publishes its field hit region");
    let position = Position::new(field.area.x, field.area.y);
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
    publish(&mut runtime, screen, &mut buffer);
    assert!(runtime.app().state.is_open());
}

#[test]
fn public_select_composition_survives_narrow_and_zero_sized_regions() {
    let mut state = SelectState::default();
    state.set_value(Some(ItemKey::index(0)));
    let app = SelectConsumer {
        state,
        chosen: None,
    };
    let mut runtime = Runtime::new(app, Theme::junie());
    let _ = runtime.initialize();

    for screen in [Rect::new(0, 0, 8, 2), Rect::new(0, 0, 0, 0)] {
        let _ = runtime.handle(Input::Resize(screen.width, screen.height));
        let mut buffer = Buffer::empty(screen);
        publish(&mut runtime, screen, &mut buffer);
    }
    assert_eq!(runtime.app().state.value(), Some(ItemKey::index(0)));
    assert!(!runtime.app().state.is_open());
    assert_eq!(runtime.app().chosen, None);
}
