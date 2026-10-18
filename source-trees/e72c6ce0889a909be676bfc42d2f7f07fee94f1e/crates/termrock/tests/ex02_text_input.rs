//! External facade integration test for recipe EX-02.
//!
//! Compile this file as a `termrock` integration test (`termrock` is the
//! only crate dependency). It does not import `termrock_controls` or
//! `termrock_fields`.
//!
//! The EX-02 snippet matches the facade. `TextInput::new` takes only an
//! `Id`. Draw borrows the fixture through `.value(&str)`. There is no
//! `TextInput::new(id, value, revision)` constructor.

use termrock::{
    App, Buffer, Cx, EditPhase, Flow, Id, Rect, Response, Runtime, TextAction, TextInput,
    TextInputState, Theme, Ui,
};

const NAME_ID: Id = Id::root("ex02.name");
/// `TextInput::measure` minimum is eight columns: gutter, indent, text, trailing cell.
const NAME_AREA: Rect = Rect::new(0, 0, 8, 1);
/// Two graphemes. Neither is the empty-cell symbol `" "`.
const NAME: &str = "ab";
/// Editable field. This test supplies no key and no mouse.
const LOCKED: bool = false;

fn update_name(
    id: Id,
    cx: &mut Cx<'_>,
    state: &mut TextInputState,
    value: &mut String,
    locked: bool,
) -> Response<TextAction> {
    TextInput::new(id)
        .read_only(locked)
        .update(cx, state, value)
}

fn draw_name(
    id: Id,
    ui: &mut Ui<'_>,
    area: Rect,
    state: &TextInputState,
    value: &str,
    locked: bool,
) {
    TextInput::new(id)
        .read_only(locked)
        .value(value)
        .draw(ui, area, state);
}

struct NameProbe {
    id: Id,
    area: Rect,
    locked: bool,
    state: TextInputState,
    value: String,
    /// The value `update_name` returned. Never a constructed `Response`.
    captured: Option<Response<TextAction>>,
}

impl App for NameProbe {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let response = update_name(self.id, cx, &mut self.state, &mut self.value, self.locked);
        // `App::update` must return `Response<()>`. Keep the original
        // `Response<TextAction>` and forward only its erased form.
        let forwarded = response.clone().erase();
        self.captured = Some(response);
        forwarded
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        draw_name(
            self.id,
            ui,
            self.area,
            &self.state,
            &self.value,
            self.locked,
        );
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

fn row_contains_name(symbols: &[&str]) -> bool {
    const NAME_CELLS: [&str; 2] = ["a", "b"];
    symbols
        .windows(NAME_CELLS.len())
        .any(|window| window == NAME_CELLS.as_slice())
}

#[test]
fn ex02_idle_name_without_input_paints_value() {
    assert!(!LOCKED, "this case is an editable field");
    assert!(!NAME_AREA.is_empty(), "draw_name requires a non-empty rect");
    assert_eq!(NAME, "ab");

    let mut runtime = Runtime::new(
        NameProbe {
            id: NAME_ID,
            area: NAME_AREA,
            locked: LOCKED,
            state: TextInputState::default(),
            value: NAME.to_owned(),
            captured: None,
        },
        Theme::junie(),
    );

    // Bootstrap only. No `Runtime::handle`, key, or mouse.
    let _bootstrap = runtime.initialize();

    let response = runtime
        .app()
        .captured
        .clone()
        .expect("initialize must call update_name through App::update");
    // No intents: `Acc::finish` returns `Response::ignored()` and does not
    // tag an id. `Response<TextAction>` has no `activated()` method.
    let action: Option<TextAction> = response.action_ref().copied();
    assert_eq!(
        action, None,
        "TextInput::update must not return TextAction::Committed with no key events"
    );
    assert_ne!(action, Some(TextAction::Committed));
    assert_eq!(response, Response::<TextAction>::ignored());
    assert_eq!(response.flow(), Flow::Ignored);
    assert_eq!(runtime.app().value, NAME);
    assert!(!runtime.app().state.is_editing());
    assert_eq!(runtime.app().state.phase(), EditPhase::Idle);

    let mut buffer = Buffer::empty(NAME_AREA);
    assert!(
        !row_contains_name(&row_symbols(&buffer, NAME_AREA)),
        "an empty buffer must not already contain {NAME}"
    );
    runtime
        .draw_buffer(NAME_AREA, &mut buffer)
        .commit_presented();

    let symbols = row_symbols(&buffer, NAME_AREA);
    assert!(
        row_contains_name(&symbols),
        "TextInput::draw must paint {NAME} on the field row, got {symbols:?}"
    );
}
