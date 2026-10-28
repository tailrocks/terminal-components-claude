use ratatui::backend::TestBackend;
use ratatui::{Terminal, buffer::Buffer, layout::Rect};
use termrock_controls::button::Button;
use termrock_core::id::Id;
use termrock_forms::form::{
    FieldKind, FieldMut, FieldRef, FieldSizingPolicy, FieldSpec, Form, FormData, FormState,
};
use termrock_runtime::{App, Cx, Response, Runtime, Ui};
use termrock_theme::{Role, Theme};

const FORM: Id = Id::root("example.intrinsic-content");
const NOTE: Id = Id::root("example.intrinsic-content.note");
const SOURCE: Id = Id::root("example.intrinsic-content.source");
const NOTES: &[(&str, Role)] = &[
    ("Live rows determine note height.", Role::Info),
    ("Each row keeps its semantic tone.", Role::Warning),
];
const FIELDS: &[FieldSpec<'static>] = &[
    FieldSpec::new(NOTE, "", FieldKind::Note),
    FieldSpec::new(
        SOURCE,
        "Source",
        FieldKind::Chooser(Button::new(SOURCE, "Choose")),
    ),
];

struct Data;

impl FormData for Data {
    fn value(&self, id: Id) -> FieldRef<'_> {
        match id {
            NOTE => FieldRef::Note(NOTES),
            SOURCE => FieldRef::Display {
                value: "live provider data",
                detail: Some("Fetched from the current connection"),
            },
            _ => FieldRef::Text(""),
        }
    }

    fn value_mut(&mut self, _id: Id) -> FieldMut<'_> {
        FieldMut::ReadOnly
    }
}

struct IntrinsicContentApp {
    form: Form<'static>,
    state: FormState,
    data: Data,
    area: Rect,
}

impl App for IntrinsicContentApp {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.form.draw(ui, self.area, &self.state, &self.data);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let area = Rect::new(0, 0, 64, 10);
    let app = IntrinsicContentApp {
        form: Form::new(FORM, FIELDS).field_sizing(FieldSizingPolicy::Intrinsic),
        state: FormState::default(),
        data: Data,
        area,
    };
    render(app, area)
}

fn render(app: impl App, area: Rect) -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = Runtime::new(app, Theme::junie());
    let prepared = runtime.prepare(area)?;
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height))?;
    let mut draw_error = None;
    terminal.try_draw(|frame| {
        draw_error = runtime.draw_prepared(prepared, frame).err();
        Ok::<(), std::convert::Infallible>(())
    })?;
    if let Some(error) = draw_error {
        return Err(std::io::Error::other(error).into());
    }
    runtime.acknowledge_presented(prepared)?;
    let buffer: &Buffer = terminal.backend().buffer();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            if let Some(cell) = buffer.cell((x, y)) {
                print!("{}", cell.symbol());
            }
        }
        println!();
    }
    Ok(())
}
