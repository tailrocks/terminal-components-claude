#![allow(
    clippy::unwrap_used,
    reason = "focused public Form window regression assertions"
)]

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use termrock_core::event::{Axis, MouseKind};
use termrock_core::id::{Id, ItemKey, Part, PartRef};
use termrock_fields::{input::TextInput, textarea::TextArea};
use termrock_forms::{
    FieldKind, FieldMut, FieldRef, FieldSpec, Form, FormData, FormLane, FormLayout, FormRow,
    FormState,
};
use termrock_layout::{SplitAxis, SplitModel};
use termrock_runtime::{App, Cx, Response, Runtime, Theme, Ui};

const FORM: Id = Id::root("consumer.preferences.form");
const BODY: Id = Id::root("consumer.preferences.long-note");
const SCREEN: Rect = Rect::new(0, 0, 80, 5);
const VALUE: &str = "row-zero\nrow-one\nrow-two\nrow-three\nrow-four\nrow-five\nrow-six";
const LANE_LEFT: Id = Id::root("consumer.preferences.notes");
const LANE_RIGHT: Id = Id::root("consumer.preferences.transport");
const OTHER: Id = Id::root("consumer.preferences.endpoint");

#[derive(Default)]
struct Preferences {
    note: String,
    endpoint: String,
}

impl FormData for Preferences {
    fn value(&self, id: Id) -> FieldRef<'_> {
        match id {
            BODY => FieldRef::Text(&self.note),
            OTHER => FieldRef::Text(&self.endpoint),
            _ => FieldRef::Text(""),
        }
    }

    fn value_mut(&mut self, id: Id) -> FieldMut<'_> {
        match id {
            BODY => FieldMut::Text(&mut self.note),
            OTHER => FieldMut::Text(&mut self.endpoint),
            _ => FieldMut::ReadOnly,
        }
    }
}

struct Consumer {
    state: FormState,
    data: Preferences,
}

const LEFT_ROWS: &[FormRow] = &[FormRow::Field(BODY)];
const RIGHT_ROWS: &[FormRow] = &[FormRow::Field(OTHER)];
const LANES: &[FormLane<'static>] = &[
    FormLane::new(LANE_LEFT, LEFT_ROWS),
    FormLane::new(LANE_RIGHT, RIGHT_ROWS),
];
const LAYOUT: FormLayout<'static> =
    FormLayout::new(LANES).split(SplitModel::new(SplitAxis::Horizontal, 50, 8, 4), 2);

impl Consumer {
    fn fields() -> [FieldSpec<'static>; 2] {
        [
            FieldSpec::new(
                BODY,
                "Notification notes",
                FieldKind::Area(TextArea::new(BODY, 5)),
            ),
            FieldSpec::new(
                OTHER,
                "Transport endpoint",
                FieldKind::Text(TextInput::new(OTHER)),
            ),
        ]
    }
}

impl App for Consumer {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == termrock_runtime::UpdateCause::Bootstrap {
            cx.focus(OTHER);
        }
        let fields = Self::fields();
        Form::new(FORM, &fields)
            .try_layout(&LAYOUT)
            .expect("two-lane preferences form is valid")
            .update(cx, &mut self.state, &mut self.data)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let fields = Self::fields();
        Form::new(FORM, &fields)
            .try_layout(&LAYOUT)
            .expect("two-lane preferences form is valid")
            .draw(ui, ui.full(), &self.state, &self.data);
    }
}

fn present(runtime: &mut Runtime<Consumer>) -> Buffer {
    let mut buffer = Buffer::empty(SCREEN);
    for _ in 0..8 {
        if runtime.needs_settle() {
            let _ = runtime.settle();
        }
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        if !runtime.needs_settle() {
            break;
        }
    }
    buffer
}

fn consumer() -> Runtime<Consumer> {
    let mut runtime = Runtime::new(
        Consumer {
            state: FormState::default(),
            data: Preferences {
                note: VALUE.to_owned(),
                endpoint: "https://notify.example.invalid".to_owned(),
            },
        },
        Theme::junie(),
    );
    let _ = runtime.initialize();
    let _ = present(&mut runtime);
    let scroll = FORM.part(Part::BODY).item(ItemKey::Num(LANE_LEFT.hash()));
    let track = runtime
        .area_of_part(scroll, PartRef::of(Part::THUMB))
        .expect("long lane has a thumb");
    for _ in 0..2 {
        let _ = termrock_runtime::runtime::stub::deliver(
            &mut runtime,
            termrock_runtime::runtime::stub::mouse(
                MouseKind::Wheel(Axis::V, 1),
                track.x,
                SCREEN.height.saturating_sub(1),
            ),
        );
        let _ = present(&mut runtime);
    }
    assert_ne!(
        Some(track),
        runtime.area_of_part(scroll, PartRef::of(Part::THUMB)),
        "wheel scroll moves the lane Form"
    );
    runtime
}

fn row(buffer: &Buffer, y: u16, x: u16, width: u16) -> String {
    (x..x.saturating_add(width))
        .map(|column| buffer[(column, y)].symbol())
        .collect::<String>()
        .trim_end()
        .to_owned()
}

#[test]
fn clipped_form_text_area_keeps_original_document_rows() {
    let mut runtime = consumer();
    let buffer = present(&mut runtime);
    let text = runtime
        .area_of_part(BODY, PartRef::of(Part::TEXT))
        .expect("partially visible TextArea publishes only its visible text body");
    assert!(text.height > 0);
    let visible: Vec<_> = (text.y..text.bottom())
        .map(|y| row(&buffer, y, text.x, text.width))
        .collect();
    assert_eq!(visible, ["row-one", "row-two", "row-three", "row-four"]);
}

#[test]
fn pointer_focus_and_cursor_stay_inside_the_visible_text_window() {
    let mut runtime = consumer();
    let text = runtime
        .area_of_part(BODY, PartRef::of(Part::TEXT))
        .expect("partially visible TextArea publishes its visible text body");
    let point = (text.x.saturating_add(1), text.y.saturating_add(1));
    for kind in [MouseKind::Down, MouseKind::Up] {
        let _ = termrock_runtime::runtime::stub::deliver(
            &mut runtime,
            termrock_runtime::runtime::stub::mouse(kind, point.0, point.1),
        );
        let _ = present(&mut runtime);
    }
    assert_eq!(runtime.focus(), Some(BODY));
    let cursor = runtime.cursor().expect("focused TextArea owns its cursor");
    assert!(
        cursor.x >= text.x
            && cursor.x < text.right()
            && cursor.y >= text.y
            && cursor.y < text.bottom(),
        "cursor {cursor:?} escapes visible TextArea text region {text:?}"
    );
}
