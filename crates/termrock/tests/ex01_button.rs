use termrock::{
    Activated, App, Buffer, Button, Cx, Flow, Id, Rect, Response, Runtime, Theme, Ui, Variant,
};

const SAVE_ID: Id = Id::root("ex01.save");
const SAVE_AREA: Rect = Rect::new(0, 0, 12, 1);
const CAN_SAVE: bool = false;

fn update_save(id: Id, cx: &mut Cx<'_>, can_save: bool) -> Response<Activated> {
    Button::new(id, "Save")
        .variant(Variant::PRIMARY)
        .disabled(!can_save)
        .update(cx)
}

fn draw_save(id: Id, ui: &mut Ui<'_>, area: Rect, can_save: bool) {
    Button::new(id, "Save")
        .variant(Variant::PRIMARY)
        .disabled(!can_save)
        .draw(ui, area);
}

struct SaveProbe {
    id: Id,
    area: Rect,
    can_save: bool,
    captured: Option<Response<Activated>>,
}

impl App for SaveProbe {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let response = update_save(self.id, cx, self.can_save);

        let forwarded = response.clone().erase();
        self.captured = Some(response);
        forwarded
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        draw_save(self.id, ui, self.area, self.can_save);
    }
}

fn row_symbols(buffer: &Buffer, area: Rect) -> Vec<&str> {
    let mut symbols = Vec::with_capacity(usize::from(area.width));
    let mut x = area.x;
    let end = area.x.saturating_add(area.width);
    while x < end {
        if let Some(cell) = buffer.cell((x, area.y)) {
            symbols.push(cell.symbol());
        }
        x = x.saturating_add(1);
    }
    symbols
}

fn row_contains_save(symbols: &[&str]) -> bool {
    const SAVE: [&str; 4] = ["S", "a", "v", "e"];
    symbols
        .windows(SAVE.len())
        .any(|window| window == SAVE.as_slice())
}

#[test]
fn ex01_disabled_save_without_input_paints_label() {
    const {
        assert!(!CAN_SAVE, "this case is the disabled button");
    }
    assert!(!SAVE_AREA.is_empty(), "draw_save requires a non-empty rect");

    let mut runtime = Runtime::new(
        SaveProbe {
            id: SAVE_ID,
            area: SAVE_AREA,
            can_save: CAN_SAVE,
            captured: None,
        },
        Theme::junie(),
    );

    let _bootstrap = runtime.initialize();

    let response = runtime
        .app()
        .captured
        .clone()
        .expect("initialize must call update_save through App::update");
    assert_eq!(response.id(), Some(SAVE_ID));
    assert!(!response.activated());
    assert!(response.action_ref().is_none());
    assert_eq!(response.flow(), Flow::Ignored);

    let mut buffer = Buffer::empty(SAVE_AREA);
    assert!(
        !row_contains_save(&row_symbols(&buffer, SAVE_AREA)),
        "an empty buffer must not already contain Save"
    );
    runtime
        .draw_buffer(SAVE_AREA, &mut buffer)
        .commit_presented();

    let symbols = row_symbols(&buffer, SAVE_AREA);
    assert!(
        row_contains_save(&symbols),
        "Button::draw must paint Save on the button row, got {symbols:?}"
    );
}
