use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
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

#[derive(Default)]
struct EmptyApp;

impl App for EmptyApp {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, _ui: &mut Ui<'_>) {}
}

fn main() {
    let form = Form::new(FORM, FIELDS).field_sizing(FieldSizingPolicy::Intrinsic);
    let area = Rect::new(0, 0, 64, 10);
    let mut buffer = Buffer::empty(area);
    Runtime::new(EmptyApp, Theme::junie())
        .draw_scene(area, &mut buffer, |ui, area| {
            form.draw(ui, area, &FormState::default(), &Data);
        })
        .commit_presented();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            if let Some(cell) = buffer.cell((x, y)) {
                print!("{}", cell.symbol());
            }
        }
        println!();
    }
}
