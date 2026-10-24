use core::cell::Cell;

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use termrock_controls::choice::{LabelRadio, RadioNavigation};
use termrock_core::action::{Action, ActionKey};
use termrock_core::event::{Input, Key, KeyCode, KeyModifiers, MouseKind};
use termrock_core::id::{Id, ItemKey, Part, PartRef};
use termrock_core::validate::FieldError;
use termrock_fields::input::TextInput;
use termrock_fields::textarea::TextArea;
use termrock_forms::{
    FieldKind, FieldMut, FieldRef, FieldSpec, Form, FormAction, FormData, FormLane, FormLayout,
    FormLayoutError, FormRow, FormState, GroupKey,
};
use termrock_layout::{Constraints, Size, SplitAxis, SplitModel};
use termrock_overlays::select::LabelSelect;
use termrock_runtime::diagnostics::Diagnostic;
use termrock_runtime::{App, Cx, FrameRead, Response, Runtime, Ui};
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
const LANE_SECOND: Id = Id::root("consumer.notifications.secondary-lane");
const STORE: ActionKey = ActionKey::custom("store-preferences");
const STORE_ACTIONS: &[Action<'static>] = &[Action::new(STORE, "Store preferences")];

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

    fn value_and_options(&mut self, id: Id) -> (FieldMut<'_>, &[&str]) {
        let options = match id {
            INTERVAL => INTERVALS,
            TRANSPORT => self.transports,
            _ => &[],
        };
        (self.value_mut(id), options)
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

struct InvalidLayoutUpdateApp<'a> {
    form: Form<'a>,
    state: FormState,
    data: Notifications,
}

impl App for InvalidLayoutUpdateApp<'_> {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let _ = self.form.update(cx, &mut self.state, &mut self.data);
        Response::ignored()
    }

    fn draw(&self, _ui: &mut Ui<'_>) {}
}

struct LogicalAreaApp<'a> {
    form: Form<'a>,
    state: FormState,
    data: Notifications,
    update_area: Option<Rect>,
}

impl App for LogicalAreaApp<'_> {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.update_area = cx
            .layout(FORM)
            .and_then(|facts| facts.logical_area)
            .or_else(|| cx.area(FORM));
        self.form
            .update(cx, &mut self.state, &mut self.data)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.form.draw(ui, ui.full(), &self.state, &self.data);
    }
}

struct ClippedRadioConsumer {
    state: FormState,
    data: Notifications,
}

impl ClippedRadioConsumer {
    fn fields() -> [FieldSpec<'static>; 2] {
        [
            FieldSpec::new(
                TRANSPORT,
                "Transport",
                FieldKind::Radio(LabelRadio::new(TRANSPORT).navigation(RadioNavigation::Deferred)),
            ),
            FieldSpec::new(
                DESTINATION,
                "Recipient",
                FieldKind::Text(TextInput::new(DESTINATION)),
            ),
        ]
    }
}

impl App for ClippedRadioConsumer {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        const ROWS: &[FormRow] = &[FormRow::Field(TRANSPORT), FormRow::Field(DESTINATION)];
        const LANES: &[FormLane<'static>] = &[FormLane::new(LANE_LEFT, ROWS)];
        let fields = Self::fields();
        let layout = FormLayout::new(LANES);
        let form = Form::new(FORM, &fields)
            .try_layout(&layout)
            .expect("valid single lane");
        form.update(cx, &mut self.state, &mut self.data).erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        const ROWS: &[FormRow] = &[FormRow::Field(TRANSPORT), FormRow::Field(DESTINATION)];
        const LANES: &[FormLane<'static>] = &[FormLane::new(LANE_LEFT, ROWS)];
        let fields = Self::fields();
        let layout = FormLayout::new(LANES);
        let form = Form::new(FORM, &fields)
            .try_layout(&layout)
            .expect("valid single lane");
        form.draw(ui, ui.full(), &self.state, &self.data);
    }
}

struct ActionConsumer {
    state: FormState,
    data: Notifications,
    reverse: bool,
    saves: usize,
}

impl ActionConsumer {
    fn fields() -> [FieldSpec<'static>; 2] {
        [
            FieldSpec::new(
                TRANSPORT,
                "Primary mode",
                FieldKind::Radio(LabelRadio::new(TRANSPORT)),
            ),
            FieldSpec::new(
                INTERVAL,
                "Secondary mode",
                FieldKind::Radio(LabelRadio::new(INTERVAL)),
            ),
        ]
    }
}

impl App for ActionConsumer {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        const LEFT_ROWS: &[FormRow] = &[FormRow::Field(TRANSPORT), FormRow::Spacer(20)];
        const RIGHT_ROWS: &[FormRow] = &[FormRow::Field(INTERVAL), FormRow::Spacer(20)];
        let mut lanes = [
            FormLane::new(LANE_LEFT, LEFT_ROWS),
            FormLane::new(LANE_SECOND, RIGHT_ROWS),
        ];
        if self.reverse {
            lanes.swap(0, 1);
        }
        let layout =
            FormLayout::new(&lanes).split(SplitModel::new(SplitAxis::Horizontal, 58, 10, 10), 4);
        let fields = Self::fields();
        let form = Form::new(FORM, &fields)
            .actions(STORE_ACTIONS)
            .try_layout(&layout)
            .expect("valid two lane layout");
        let response = form.update(cx, &mut self.state, &mut self.data);
        if matches!(response.action_ref(), Some(FormAction::Action(STORE))) {
            self.saves += 1;
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        const LEFT_ROWS: &[FormRow] = &[FormRow::Field(TRANSPORT), FormRow::Spacer(20)];
        const RIGHT_ROWS: &[FormRow] = &[FormRow::Field(INTERVAL), FormRow::Spacer(20)];
        let mut lanes = [
            FormLane::new(LANE_LEFT, LEFT_ROWS),
            FormLane::new(LANE_SECOND, RIGHT_ROWS),
        ];
        if self.reverse {
            lanes.swap(0, 1);
        }
        let layout =
            FormLayout::new(&lanes).split(SplitModel::new(SplitAxis::Horizontal, 58, 10, 10), 4);
        let fields = Self::fields();
        let form = Form::new(FORM, &fields)
            .actions(STORE_ACTIONS)
            .try_layout(&layout)
            .expect("valid two lane layout");
        form.draw(ui, ui.full(), &self.state, &self.data);
    }
}

fn present<A: App>(runtime: &mut Runtime<A>, area: Rect) {
    for _ in 0..3 {
        if runtime.needs_settle() {
            let _ = runtime.settle();
        }
        let mut buffer = Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
    }
}

fn mouse_input<A: App>(runtime: &mut Runtime<A>, area: Rect, kind: MouseKind, x: u16, y: u16) {
    let _ = termrock_runtime::runtime::stub::deliver(
        runtime,
        termrock_runtime::runtime::stub::mouse(kind, x, y),
    );
    present(runtime, area);
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
    assert_eq!(runtime.diagnostics().len(), 1, "measurement is pure");
    assert!(matches!(
        runtime.diagnostics(),
        [Diagnostic::InvalidComponentLayout {
            owner: FORM,
            field: Some(ENDPOINT),
        }]
    ));

    let mut state = FormState::default();
    state.set_error(DESTINATION, Some(FieldError::new("sentinel")));
    let expected_state = state.clone();
    let invalid_update_form = invalid_form;
    let data = Notifications {
        show_endpoint: true,
        ..Notifications::default()
    };
    let app = InvalidLayoutUpdateApp {
        form: invalid_update_form,
        state,
        data,
    };
    let mut runtime = Runtime::new(app, Theme::junie());
    let _ = runtime.initialize();
    assert!(matches!(
        runtime.diagnostics(),
        [Diagnostic::InvalidComponentLayout {
            owner: FORM,
            field: Some(ENDPOINT),
        }]
    ));
    assert_eq!(
        runtime.app().state,
        expected_state,
        "invalid update leaves existing form state unchanged"
    );
    assert_eq!(
        runtime.app().data.reads.get(),
        0,
        "invalid update has no domain effects"
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

#[test]
fn fill_reserves_trailing_fixed_spacers_before_allocating_height() {
    let fields = [FieldSpec::new(
        MESSAGE,
        "Message",
        FieldKind::Area(TextArea::new(MESSAGE, 4)),
    )];
    let rows = [FormRow::fill(MESSAGE), FormRow::spacer(2)];
    let lanes = [FormLane::new(LANE_LEFT, &rows)];
    let layout = FormLayout::new(&lanes);
    let form = Form::new(FORM, &fields)
        .try_layout(&layout)
        .expect("Fill may be followed by fixed spacer rows");
    let scroll_id = FORM.part(Part::BODY).item(ItemKey::Num(LANE_LEFT.hash()));

    for (height, should_overflow) in [(12, false), (5, true)] {
        let area = Rect::new(0, 0, 40, height);
        let mut buffer = Buffer::empty(area);
        let mut runtime = Runtime::new(EmptyApp, Theme::junie());
        runtime
            .draw_scene(area, &mut buffer, |ui, area| {
                form.draw(ui, area, &FormState::default(), &Notifications::default());
            })
            .commit_presented();
        assert_eq!(
            runtime
                .area_of_part(scroll_id, PartRef::of(Part::THUMB))
                .is_some(),
            should_overflow,
            "Fill minimum plus trailing spacer is considered at height {height}"
        );
    }
}

#[test]
fn lane_update_reads_the_last_drawn_form_area_after_resize() {
    let fields = [FieldSpec::new(
        DESTINATION,
        "Destination",
        FieldKind::Text(TextInput::new(DESTINATION)),
    )];
    let rows = [FormRow::field(DESTINATION)];
    let lanes = [FormLane::new(LANE_LEFT, &rows)];
    let layout = FormLayout::new(&lanes);
    let form = Form::new(FORM, &fields)
        .try_layout(&layout)
        .expect("valid one-lane layout");
    let app = LogicalAreaApp {
        form,
        state: FormState::default(),
        data: Notifications::default(),
        update_area: None,
    };
    let mut runtime = Runtime::new(app, Theme::junie());
    let _ = runtime.initialize();

    for area in [Rect::new(0, 0, 40, 12), Rect::new(0, 0, 60, 20)] {
        let mut buffer = Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        assert!(
            !runtime.registry().regions().iter().any(|region| {
                region.owner == FORM
                    && (region.part == PartRef::of(Part::CONTAINER)
                        || region.part == PartRef::of(Part::BODY))
                    && region.area == area
            }),
            "logical bounds must not create a hit region covering child controls"
        );
        let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, Input::Tick);
        assert_eq!(runtime.app().update_area, Some(area));
    }
}

#[test]
fn clipped_radio_row_pointer_activates_through_the_real_form() {
    let area = Rect::new(0, 0, 40, 6);
    let mut runtime = Runtime::new(
        ClippedRadioConsumer {
            state: FormState::default(),
            data: Notifications::default(),
        },
        Theme::junie(),
    );
    let _ = runtime.initialize();
    present(&mut runtime, area);
    runtime.set_focus(Some(DESTINATION));
    present(&mut runtime, area);
    let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, Input::Tick);
    present(&mut runtime, area);

    let option = runtime
        .area_of_part(TRANSPORT, PartRef::item(Part::ROW, ItemKey::index(8)))
        .expect("the original eighth option was revealed in the short viewport");
    for kind in [MouseKind::Down, MouseKind::Up] {
        mouse_input(
            &mut runtime,
            area,
            kind,
            option.x.saturating_add(2),
            option.y,
        );
    }

    assert_eq!(runtime.focus(), Some(TRANSPORT));
    assert_eq!(runtime.app().data.transport, 8);

    let _ = termrock_runtime::runtime::stub::deliver(
        &mut runtime,
        Input::Key(Key {
            code: KeyCode::End,
            mods: KeyModifiers::NONE,
        }),
    );
    present(&mut runtime, area);
    assert_eq!(
        runtime.app().data.transport,
        8,
        "moving the cursor is not selection"
    );
    assert!(
        runtime
            .area_of_part(TRANSPORT, PartRef::item(Part::ROW, ItemKey::index(9)))
            .is_some()
    );
    let _ = termrock_runtime::runtime::stub::deliver(
        &mut runtime,
        Input::Key(Key {
            code: KeyCode::Char(' '),
            mods: KeyModifiers::NONE,
        }),
    );
    present(&mut runtime, area);
    assert_eq!(runtime.app().data.transport, 9);
}

struct ImmediateRadioForm {
    state: FormState,
    data: Notifications,
    lanes: bool,
}

impl ImmediateRadioForm {
    fn update_form(&mut self, cx: &mut Cx<'_>) -> Response<FormAction> {
        let fields = [FieldSpec::new(
            TRANSPORT,
            "Transport",
            FieldKind::Radio(LabelRadio::new(TRANSPORT)),
        )];
        if self.lanes {
            const ROWS: &[FormRow] = &[FormRow::Field(TRANSPORT)];
            const LANES: &[FormLane<'static>] = &[FormLane::new(LANE_LEFT, ROWS)];
            Form::new(FORM, &fields)
                .try_layout(&FormLayout::new(LANES))
                .expect("valid one-radio lane")
                .update(cx, &mut self.state, &mut self.data)
        } else {
            Form::new(FORM, &fields).update(cx, &mut self.state, &mut self.data)
        }
    }

    fn draw_form(&self, ui: &mut Ui<'_>) {
        let fields = [FieldSpec::new(
            TRANSPORT,
            "Transport",
            FieldKind::Radio(LabelRadio::new(TRANSPORT)),
        )];
        if self.lanes {
            const ROWS: &[FormRow] = &[FormRow::Field(TRANSPORT)];
            const LANES: &[FormLane<'static>] = &[FormLane::new(LANE_LEFT, ROWS)];
            let layout = FormLayout::new(LANES);
            let form = Form::new(FORM, &fields)
                .try_layout(&layout)
                .expect("valid one-radio lane");
            form.draw(ui, ui.full(), &self.state, &self.data);
        } else {
            Form::new(FORM, &fields).draw(ui, ui.full(), &self.state, &self.data);
        }
    }
}

impl App for ImmediateRadioForm {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.update_form(cx).erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.draw_form(ui);
    }
}

#[test]
fn immediate_radio_navigation_updates_values_through_flat_and_lane_forms() {
    let area = Rect::new(0, 0, 60, 16);
    for lanes in [false, true] {
        let mut runtime = Runtime::new(
            ImmediateRadioForm {
                state: FormState::default(),
                data: Notifications::default(),
                lanes,
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        runtime.set_focus(Some(TRANSPORT));
        let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, Input::Tick);
        let _ = termrock_runtime::runtime::stub::deliver(
            &mut runtime,
            Input::Key(Key {
                code: KeyCode::Down,
                mods: KeyModifiers::NONE,
            }),
        );
        assert_eq!(
            runtime.app().data.transport,
            1,
            "keyboard navigation selects through {} Form update",
            if lanes { "lane" } else { "flat" }
        );
    }
}

#[test]
fn fixed_form_action_pointer_activates_after_scrolling_and_lane_reorder() {
    let mut area = Rect::new(0, 0, 100, 8);
    let mut runtime = Runtime::new(
        ActionConsumer {
            state: FormState::default(),
            data: Notifications::default(),
            reverse: false,
            saves: 0,
        },
        Theme::junie(),
    );
    let _ = runtime.initialize();
    present(&mut runtime, area);

    let first_row = PartRef::item(Part::ROW, ItemKey::index(0));
    let right_before = runtime
        .area_of_part(INTERVAL, first_row)
        .expect("right lane row visible");
    let _ = termrock_runtime::runtime::stub::deliver(
        &mut runtime,
        termrock_runtime::runtime::stub::mouse(
            MouseKind::Wheel(termrock_core::event::Axis::V, 3),
            right_before.x.saturating_add(2),
            right_before.y,
        ),
    );
    present(&mut runtime, area);
    assert!(runtime.area_of_part(INTERVAL, first_row).is_none());
    assert!(runtime.area_of_part(TRANSPORT, first_row).is_some());

    runtime.app_mut().reverse = true;
    present(&mut runtime, area);
    assert!(runtime.area_of_part(INTERVAL, first_row).is_none());
    assert!(runtime.area_of_part(TRANSPORT, first_row).unwrap().x > 50);

    area = Rect::new(0, 0, 81, 9);
    let _ = termrock_runtime::runtime::stub::deliver(
        &mut runtime,
        Input::Resize(area.width, area.height),
    );
    present(&mut runtime, area);
    assert!(runtime.area_of_part(INTERVAL, first_row).is_none());

    let action = FORM.part(Part::ACTIONS).index(0);
    let button = runtime
        .area_of(action)
        .expect("fixed action button is rendered in the footer");
    assert_eq!(button.y, area.height - 1);
    for kind in [MouseKind::Down, MouseKind::Up] {
        mouse_input(
            &mut runtime,
            area,
            kind,
            button.x.saturating_add(1),
            button.y,
        );
    }
    assert_eq!(runtime.app().saves, 1);
}
