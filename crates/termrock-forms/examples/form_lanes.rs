use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use termrock_controls::choice::LabelRadio;
use termrock_core::id::Id;
use termrock_fields::input::TextInput;
use termrock_forms::{
    FieldKind, FieldMut, FieldRef, FieldSpec, Form, FormData, FormLane, FormLayout, FormRow,
    FormState,
};
use termrock_layout::{SplitAxis, SplitModel};
use termrock_runtime::{App, Cx, Response, Runtime, Ui};
use termrock_theme::Theme;

const FORM: Id = Id::root("example.notifications");
const DESTINATION: Id = Id::root("example.notifications.destination");
const TRANSPORT: Id = Id::root("example.notifications.transport");
const LEFT: Id = Id::root("example.notifications.left");
const RIGHT: Id = Id::root("example.notifications.right");
const TRANSPORTS: &[&str] = &["Mail", "Chat", "Webhook"];

struct Preferences {
    destination: String,
    transport: usize,
}

impl FormData for Preferences {
    fn value(&self, id: Id) -> FieldRef<'_> {
        match id {
            DESTINATION => FieldRef::Text(&self.destination),
            TRANSPORT => FieldRef::Choice(self.transport),
            _ => FieldRef::Text(""),
        }
    }

    fn value_mut(&mut self, id: Id) -> FieldMut<'_> {
        match id {
            DESTINATION => FieldMut::Text(&mut self.destination),
            TRANSPORT => FieldMut::Choice(&mut self.transport),
            _ => FieldMut::ReadOnly,
        }
    }

    fn options(&self, id: Id) -> &[&str] {
        if id == TRANSPORT { TRANSPORTS } else { &[] }
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
    const FIELDS: &[FieldSpec<'static>] = &[
        FieldSpec::new(
            DESTINATION,
            "Destination",
            FieldKind::Text(TextInput::new(DESTINATION)),
        ),
        FieldSpec::new(
            TRANSPORT,
            "Transport",
            FieldKind::Radio(LabelRadio::new(TRANSPORT)),
        ),
    ];
    const LEFT_ROWS: &[FormRow] = &[FormRow::Field(DESTINATION)];
    const RIGHT_ROWS: &[FormRow] = &[FormRow::Field(TRANSPORT)];
    const LANES: &[FormLane<'static>] = &[
        FormLane::new(LEFT, LEFT_ROWS),
        FormLane::new(RIGHT, RIGHT_ROWS),
    ];
    const LAYOUT: FormLayout<'static> =
        FormLayout::new(LANES).split(SplitModel::new(SplitAxis::Horizontal, 55, 24, 20), 2);

    let form = Form::new(FORM, FIELDS)
        .try_layout(&LAYOUT)
        .expect("example layout references each field exactly once");
    let data = Preferences {
        destination: "Build and release notices 🧭".to_owned(),
        transport: 1,
    };
    let area = Rect::new(0, 0, 72, 12);
    let mut buffer = Buffer::empty(area);
    Runtime::new(EmptyApp, Theme::junie())
        .draw_scene(area, &mut buffer, |ui, area| {
            form.draw(ui, area, &FormState::default(), &data);
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
