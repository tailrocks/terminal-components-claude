use core::cell::Cell;

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use termrock_controls::choice::LabelRadio;
use termrock_core::id::{Id, Part, PartRef};
use termrock_fields::input::TextInput;
use termrock_fields::textarea::TextArea;
use termrock_forms::{
    FieldKind, FieldMut, FieldRef, FieldSpec, Form, FormData, FormLane, FormLayout,
    FormLayoutError, FormRow, FormState, GroupKey,
};
use termrock_layout::{Constraints, Size, SplitAxis, SplitModel};
use termrock_overlays::select::LabelSelect;
use termrock_runtime::{App, Cx, Response, Runtime, Ui};
use termrock_theme::Theme;

const FORM: Id = Id::root("consumer.notifications.form");
const DESTINATION: Id = Id::root("consumer.notifications.destination");
const CHANNEL: Id = Id::root("consumer.notifications.channel");
const PORT: Id = Id::root("consumer.notifications.port");
const INTERVAL: Id = Id::root("consumer.notifications.interval");
const TRANSPORT: Id = Id::root("consumer.notifications.transport");
const ENDPOINT: Id = Id::root("consumer.notifications.endpoint");
const MESSAGE: Id = Id::root("consumer.notifications.message");
const LANE_LEFT: Id = Id::root("consumer.notifications.left-lane");
const LANE_RIGHT: Id = Id::root("consumer.notifications.right-lane");

const INTERVALS: &[&str] = &["Immediately", "Hourly", "Daily"];
const SHORT_TRANSPORTS: &[&str] = &["Mail", "Matrix", "Webhook"];
const LONG_TRANSPORTS: &[&str] = &[
    "Mail", "Matrix", "Webhook", "Pager", "Desktop", "Digest", "SMS", "Push", "Voice", "Archive",
];

struct Notifications {
    destination: String,
    channel: String,
    port: String,
    interval: usize,
    transport: usize,
    endpoint: String,
    message: String,
    transports: &'static [&'static str],
    show_endpoint: bool,
    reads: Cell<usize>,
}

impl Default for Notifications {
    fn default() -> Self {
        Self {
            destination: "Deploy alerts 🧭".to_owned(),
            channel: "release notes".to_owned(),
            port: "8080".to_owned(),
            interval: 1,
            transport: 0,
            endpoint: "https://hooks.example.invalid".to_owned(),
            message: "Unicode: 表示名 and combining: e\u{301}.".to_owned(),
            transports: LONG_TRANSPORTS,
            show_endpoint: false,
            reads: Cell::new(0),
        }
    }
}

impl FormData for Notifications {
    fn value(&self, id: Id) -> FieldRef<'_> {
        match id {
            DESTINATION => FieldRef::Text(&self.destination),
            CHANNEL => FieldRef::Text(&self.channel),
            PORT => FieldRef::Text(&self.port),
            INTERVAL => FieldRef::Choice(self.interval),
            TRANSPORT => FieldRef::Choice(self.transport),
            ENDPOINT => FieldRef::Text(&self.endpoint),
            MESSAGE => FieldRef::Text(&self.message),
            _ => FieldRef::Text(""),
        }
    }

    fn value_mut(&mut self, id: Id) -> FieldMut<'_> {
        match id {
            DESTINATION => FieldMut::Text(&mut self.destination),
            CHANNEL => FieldMut::Text(&mut self.channel),
            PORT => FieldMut::Text(&mut self.port),
            INTERVAL => FieldMut::Choice(&mut self.interval),
            TRANSPORT => FieldMut::Choice(&mut self.transport),
            ENDPOINT => FieldMut::Text(&mut self.endpoint),
            MESSAGE => FieldMut::Text(&mut self.message),
            _ => FieldMut::ReadOnly,
        }
    }

    fn options(&self, id: Id) -> &[&str] {
        match id {
            INTERVAL => INTERVALS,
            TRANSPORT => self.transports,
            _ => &[],
        }
    }

    fn visible(&self, id: Id) -> bool {
        id != ENDPOINT || self.show_endpoint
    }

    fn validate(
        &self,
        _id: Id,
        _value: FieldRef<'_>,
    ) -> Result<(), termrock_core::validate::FieldError> {
        self.reads.set(self.reads.get().saturating_add(1));
        Ok(())
    }
}

fn fields() -> [FieldSpec<'static>; 7] {
    [
        FieldSpec::new(
            DESTINATION,
            "通知先 — notification destination",
            FieldKind::Text(TextInput::new(DESTINATION)),
        )
        .help("A description that remains readable as its lane narrows."),
        FieldSpec::new(CHANNEL, "Channel", FieldKind::Text(TextInput::new(CHANNEL))),
        FieldSpec::new(PORT, "Port", FieldKind::Text(TextInput::new(PORT))),
        FieldSpec::new(
            INTERVAL,
            "Delivery interval",
            FieldKind::Select(LabelSelect::new(INTERVAL)),
        ),
        FieldSpec::new(
            TRANSPORT,
            "Transport",
            FieldKind::Radio(LabelRadio::new(TRANSPORT)),
        ),
        FieldSpec::new(
            ENDPOINT,
            "Webhook endpoint",
            FieldKind::Text(TextInput::new(ENDPOINT)),
        ),
        FieldSpec::new(
            MESSAGE,
            "Message template",
            FieldKind::Area(TextArea::new(MESSAGE, 3)),
        ),
    ]
}

fn layout() -> FormLayout<'static> {
    const LEFT_ROWS: &[FormRow] = &[
        FormRow::Field(DESTINATION),
        FormRow::Pair {
            first: CHANNEL,
            second: PORT,
            split: SplitModel::new(SplitAxis::Horizontal, 70, 12, 8),
            gap: 2,
        },
    ];
    const RIGHT_ROWS: &[FormRow] = &[
        FormRow::Field(INTERVAL),
        FormRow::Field(TRANSPORT),
        FormRow::Field(ENDPOINT),
        FormRow::Fill(MESSAGE),
    ];
    const LANES: &[FormLane<'static>] = &[
        FormLane::new(LANE_LEFT, LEFT_ROWS),
        FormLane::new(LANE_RIGHT, RIGHT_ROWS),
    ];
    FormLayout::new(LANES).split(SplitModel::new(SplitAxis::Horizontal, 58, 30, 24), 4)
}

#[derive(Default)]
struct EmptyApp;

impl App for EmptyApp {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, _ui: &mut Ui<'_>) {}
}

#[test]
fn borrowed_lanes_resolve_live_data_into_the_drawn_field_bounds() {
    let fields = fields();
    let form_layout = layout();
    let form = Form::new(FORM, &fields)
        .try_layout(&form_layout)
        .expect("valid borrowed layout");
    let data = Notifications::default();
    let area = Rect::new(0, 0, 100, 32);
    let mut runtime = Runtime::new(EmptyApp, Theme::junie());
    let mut buffer = Buffer::empty(area);
    let mut measured = Size::default();
    let state = FormState::default();

    runtime
        .draw_scene(area, &mut buffer, |ui, area| {
            measured =
                form.measure_with_data(ui, Constraints::loose(area.width, area.height), &data);
            form.draw(ui, area, &state, &data);
        })
        .commit_presented();

    let destination = runtime
        .area_of_part(DESTINATION, PartRef::of(Part::LABEL))
        .expect("left lane field was drawn");
    let channel = runtime
        .area_of_part(CHANNEL, PartRef::of(Part::LABEL))
        .expect("first paired field was drawn");
    let port = runtime
        .area_of_part(PORT, PartRef::of(Part::LABEL))
        .expect("second paired field was drawn");
    let interval = runtime
        .area_of_part(INTERVAL, PartRef::of(Part::LABEL))
        .expect("right lane field was drawn");
    let transport = runtime
        .area_of_part(TRANSPORT, PartRef::of(Part::LABEL))
        .expect("radio field was drawn");
    let message_label = runtime
        .area_of_part(MESSAGE, PartRef::of(Part::LABEL))
        .expect("fill field label was drawn");
    let message = runtime
        .area_of_part(MESSAGE, PartRef::of(Part::TEXT))
        .expect("fill field was drawn");

    assert!(destination.x < interval.x, "lanes use the shared 58% split");
    assert_eq!(channel.y, port.y, "paired fields share one resolved row");
    assert!(channel.x < port.x, "pair fields follow the 70% split");
    assert_eq!(
        message_label.y,
        transport.y + LONG_TRANSPORTS.len() as u16 + 2,
        "radio option count affects lane placement"
    );
    assert!(
        message.height >= 3,
        "fill retains the TextArea's measured minimum"
    );
    assert_eq!(
        measured.preferred.1, area.height,
        "fill uses the measured available height"
    );
    assert!(
        runtime
            .area_of_part(ENDPOINT, PartRef::of(Part::LABEL))
            .is_none(),
        "data-hidden fields do not register or reserve a row"
    );
    assert_eq!(
        data.reads.get(),
        0,
        "measurement and drawing do not validate data"
    );
}

#[test]
fn narrow_and_zero_width_lane_layouts_are_safe_and_data_aware() {
    let fields = fields();
    let layout = layout();
    let form = Form::new(FORM, &fields)
        .try_layout(&layout)
        .expect("valid layout");
    let data = Notifications {
        transports: SHORT_TRANSPORTS,
        show_endpoint: true,
        ..Notifications::default()
    };
    let mut runtime = Runtime::new(EmptyApp, Theme::junie());
    for width in [0, 1, 53, 54, 55, 80, 100] {
        let area = Rect::new(0, 0, width, 18);
        let mut buffer = Buffer::empty(area);
        let mut measured = Size::default();
        runtime
            .draw_scene(area, &mut buffer, |ui, area| {
                measured =
                    form.measure_with_data(ui, Constraints::loose(area.width, area.height), &data);
                form.draw(ui, area, &FormState::default(), &data);
            })
            .commit_presented();
        assert_eq!(measured.preferred.0, width);
        assert_eq!(buffer.area.width, width);
    }
    assert_eq!(data.reads.get(), 0);
}

#[test]
fn malformed_layouts_fail_closed_before_a_form_can_be_drawn() {
    let fields = fields();
    let unknown_rows = [FormRow::Field(Id::root("consumer.notifications.unknown"))];
    let unknown_lane = [FormLane::new(LANE_LEFT, &unknown_rows)];
    assert!(matches!(
        Form::new(FORM, &fields).try_layout(&FormLayout::new(&unknown_lane)),
        Err(FormLayoutError::UnknownField(_))
    ));

    let duplicate_rows = [FormRow::Field(DESTINATION), FormRow::Field(DESTINATION)];
    let duplicate_lane = [FormLane::new(LANE_LEFT, &duplicate_rows)];
    assert!(matches!(
        Form::new(FORM, &fields).try_layout(&FormLayout::new(&duplicate_lane)),
        Err(FormLayoutError::DuplicateField(DESTINATION))
    ));

    let duplicate_lanes = [
        FormLane::new(LANE_LEFT, &unknown_rows),
        FormLane::new(LANE_LEFT, &unknown_rows),
    ];
    assert!(matches!(
        Form::new(FORM, &fields).try_layout(
            &FormLayout::new(&duplicate_lanes)
                .split(SplitModel::new(SplitAxis::Horizontal, 50, 1, 1), 1,)
        ),
        Err(FormLayoutError::DuplicateLane(_))
    ));
}

#[test]
fn visibility_and_final_group_are_revalidated_before_any_registration() {
    const BASIC: GroupKey = GroupKey::custom("notifications-basic");
    const ADVANCED: GroupKey = GroupKey::custom("notifications-advanced");
    let fields = [
        FieldSpec::new(
            DESTINATION,
            "Destination",
            FieldKind::Text(TextInput::new(DESTINATION)),
        )
        .group(BASIC),
        FieldSpec::new(
            ENDPOINT,
            "Endpoint",
            FieldKind::Text(TextInput::new(ENDPOINT)),
        )
        .group(ADVANCED),
    ];
    let rows = [FormRow::Field(DESTINATION)];
    let lanes = [FormLane::new(LANE_LEFT, &rows)];
    let layout = FormLayout::new(&lanes);

    let all_visible_fields = [
        FieldSpec::new(
            DESTINATION,
            "Destination",
            FieldKind::Text(TextInput::new(DESTINATION)),
        ),
        FieldSpec::new(
            ENDPOINT,
            "Endpoint",
            FieldKind::Text(TextInput::new(ENDPOINT)),
        ),
    ];
    let visibility_form = Form::new(FORM, &all_visible_fields)
        .try_layout(&layout)
        .expect("hidden endpoint may be omitted structurally");
    let mut visibility_data = Notifications::default();
    assert_eq!(visibility_form.validate_layout(&visibility_data), Ok(()));
    visibility_data.show_endpoint = true;
    assert_eq!(
        visibility_form.validate_layout(&visibility_data),
        Err(FormLayoutError::MissingVisibleField(ENDPOINT))
    );

    let form = Form::new(FORM, &fields)
        .group(BASIC)
        .try_layout(&layout)
        .expect("inactive Advanced field may be omitted");
    let data = Notifications {
        show_endpoint: true,
        ..Notifications::default()
    };
    assert_eq!(form.validate_layout(&data), Ok(()));
    let invalid_form = form.group(ADVANCED);
    assert_eq!(
        invalid_form.validate_layout(&data),
        Err(FormLayoutError::MissingVisibleField(ENDPOINT))
    );

    let area = Rect::new(0, 0, 40, 10);
    let mut buffer = Buffer::empty(area);
    let mut runtime = Runtime::new(EmptyApp, Theme::junie());
    runtime
        .draw_scene(area, &mut buffer, |ui, area| {
            let measured = invalid_form.measure_with_data(
                ui,
                Constraints::loose(area.width, area.height),
                &data,
            );
            assert_eq!(measured.preferred, (0, 0));
            invalid_form.draw(ui, area, &FormState::default(), &data);
        })
        .commit_presented();
    assert!(
        runtime
            .area_of_part(DESTINATION, PartRef::of(Part::LABEL))
            .is_none(),
        "invalid layout aborts before registering even included fields"
    );

    let mut data = Notifications::default();
    let final_group_first = Form::new(FORM, &fields)
        .try_layout(&layout)
        .expect("layout is structurally valid")
        .group(BASIC);
    assert_eq!(final_group_first.validate_layout(&data), Ok(()));
    data.show_endpoint = true;
    assert_eq!(
        final_group_first.group(ADVANCED).validate_layout(&data),
        Err(FormLayoutError::MissingVisibleField(ENDPOINT))
    );
}

#[test]
fn live_radio_options_match_measurement_to_rendered_lane_extent() {
    let fields = [
        FieldSpec::new(
            DESTINATION,
            "Notification destination",
            FieldKind::Text(TextInput::new(DESTINATION)),
        ),
        FieldSpec::new(
            TRANSPORT,
            "Transport",
            FieldKind::Radio(LabelRadio::new(TRANSPORT)),
        ),
    ];
    let rows = [FormRow::Field(DESTINATION), FormRow::Field(TRANSPORT)];
    let lanes = [FormLane::new(LANE_LEFT, &rows)];
    let layout = FormLayout::new(&lanes);
    let form = Form::new(FORM, &fields)
        .try_layout(&layout)
        .expect("valid single lane");
    let mut runtime = Runtime::new(EmptyApp, Theme::junie());

    for transports in [SHORT_TRANSPORTS, LONG_TRANSPORTS] {
        let data = Notifications {
            transports,
            ..Notifications::default()
        };
        let area = Rect::new(0, 0, 50, 40);
        let mut buffer = Buffer::empty(area);
        let mut measured = Size::default();
        runtime
            .draw_scene(area, &mut buffer, |ui, area| {
                measured =
                    form.measure_with_data(ui, Constraints::loose(area.width, area.height), &data);
                form.draw(ui, area, &FormState::default(), &data);
            })
            .commit_presented();
        let radio = runtime
            .area_of_part(TRANSPORT, PartRef::of(Part::LABEL))
            .expect("radio label is registered");
        assert_eq!(
            measured.preferred.1,
            radio.y + transports.len() as u16 + 2,
            "measured extent matches the last live radio row"
        );
    }
}
