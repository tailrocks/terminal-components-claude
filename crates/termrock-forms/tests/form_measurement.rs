use core::cell::Cell;

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use termrock_controls::choice::LabelRadio;
use termrock_core::id::{Id, Part, PartRef};
use termrock_fields::input::TextInput;
use termrock_fields::textarea::TextArea;
use termrock_forms::form::{
    FieldKind, FieldMut, FieldRef, FieldSpan, FieldSpec, Form, FormData, FormState,
};
use termrock_layout::measure::{Constraints, Size};
use termrock_overlays::select::LabelSelect;
use termrock_runtime::{App, Cx, Response, Runtime, Ui};
use termrock_theme::Theme;

const FORM: Id = Id::root("consumer.notification-preferences.form");
const DISPLAY: Id = Id::root("consumer.notification-preferences.display");
const INTERVAL: Id = Id::root("consumer.notification-preferences.interval");
const TRANSPORT: Id = Id::root("consumer.notification-preferences.transport");
const MESSAGE: Id = Id::root("consumer.notification-preferences.message");
const ENDPOINT: Id = Id::root("consumer.notification-preferences.endpoint");
const CHANNEL_NAME: Id = Id::root("consumer.notification-preferences.channel-name");
const LOCALE: Id = Id::root("consumer.notification-preferences.locale");

const INTERVALS: &[&str] = &["Immediately", "Hourly", "Daily"];
const TRANSPORTS_SHORT: &[&str] = &["Mail", "Matrix", "Webhook"];
const TRANSPORTS_LONG: &[&str] = &[
    "Mail", "Matrix", "Webhook", "Pager", "Desktop", "Digest", "SMS", "Push", "Voice", "Archive",
];
const NO_TRANSPORTS: &[&str] = &[];

struct Preferences {
    display: String,
    interval: usize,
    transport: usize,
    message: String,
    endpoint: String,
    channel_name: String,
    locale: String,
    options: &'static [&'static str],
    validations: Cell<usize>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            display: "Ops 🧭".to_owned(),
            interval: 1,
            transport: 0,
            message: "A long body that remains application data.".to_owned(),
            endpoint: "https://example.invalid/hooks/quiet".to_owned(),
            channel_name: "Release notes".to_owned(),
            locale: "日本語とEnglish".to_owned(),
            options: TRANSPORTS_LONG,
            validations: Cell::new(0),
        }
    }
}

impl FormData for Preferences {
    fn value(&self, id: Id) -> FieldRef<'_> {
        match id {
            DISPLAY => FieldRef::Text(&self.display),
            INTERVAL => FieldRef::Choice(self.interval),
            TRANSPORT => FieldRef::Choice(self.transport),
            MESSAGE => FieldRef::Text(&self.message),
            ENDPOINT => FieldRef::Text(&self.endpoint),
            CHANNEL_NAME => FieldRef::Text(&self.channel_name),
            LOCALE => FieldRef::Text(&self.locale),
            _ => FieldRef::Text(""),
        }
    }

    fn value_mut(&mut self, id: Id) -> FieldMut<'_> {
        match id {
            DISPLAY => FieldMut::Text(&mut self.display),
            INTERVAL => FieldMut::Choice(&mut self.interval),
            TRANSPORT => FieldMut::Choice(&mut self.transport),
            MESSAGE => FieldMut::Text(&mut self.message),
            ENDPOINT => FieldMut::Text(&mut self.endpoint),
            CHANNEL_NAME => FieldMut::Text(&mut self.channel_name),
            LOCALE => FieldMut::Text(&mut self.locale),
            _ => FieldMut::ReadOnly,
        }
    }

    fn options(&self, id: Id) -> &[&str] {
        match id {
            INTERVAL => INTERVALS,
            TRANSPORT => self.options,
            _ => &[],
        }
    }

    fn visible(&self, id: Id) -> bool {
        id != ENDPOINT
    }

    fn disabled(&self, id: Id) -> bool {
        id == LOCALE
    }

    fn validate(
        &self,
        _id: Id,
        _value: FieldRef<'_>,
    ) -> Result<(), termrock_core::validate::FieldError> {
        self.validations
            .set(self.validations.get().saturating_add(1));
        Ok(())
    }
}

fn fields() -> [FieldSpec<'static>; 7] {
    [
        FieldSpec::new(
            DISPLAY,
            "表示名 — display name",
            FieldKind::Text(TextInput::new(DISPLAY)),
        )
        .help("A deliberately long help description whose tail clips at narrow widths."),
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
            MESSAGE,
            "Message",
            FieldKind::Area(TextArea::new(MESSAGE, 4)),
        ),
        FieldSpec::new(
            ENDPOINT,
            "Endpoint",
            FieldKind::Text(TextInput::new(ENDPOINT)),
        ),
        FieldSpec::new(
            CHANNEL_NAME,
            "Channel name",
            FieldKind::Text(TextInput::new(CHANNEL_NAME)),
        )
        .span(FieldSpan::Half),
        FieldSpec::new(LOCALE, "Locale", FieldKind::Text(TextInput::new(LOCALE)))
            .span(FieldSpan::Half),
    ]
}

#[derive(Default)]
struct EmptyApp;

impl App for EmptyApp {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, _ui: &mut Ui<'_>) {}
}

fn row_text(buffer: &Buffer, y: u16) -> String {
    (0..buffer.area.width)
        .filter_map(|x| buffer.cell((x, y)).map(|cell| cell.symbol()))
        .collect()
}

#[test]
fn data_measurement_matches_visible_radio_area_and_paired_half_rows() {
    let fields = fields();
    let form = Form::new(FORM, &fields).columns(2);
    let data = Preferences::default();
    let mut runtime = Runtime::new(EmptyApp, Theme::junie());
    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 40));
    let mut legacy_height = 0;
    let short_height = Cell::new(Size::default());
    let empty_height = Cell::new(Size::default());
    let long_height = Cell::new(Size::default());
    let narrow_height = Cell::new(Size::default());
    let zero_width_height = Cell::new(Size::default());

    runtime
        .draw_scene(Rect::new(0, 0, 80, 40), &mut buffer, |ui, area| {
            legacy_height = form
                .measure(ui, Constraints::loose(area.width, area.height))
                .preferred
                .1;
            short_height.set(form.measure_with_data(
                ui,
                Constraints::loose(area.width, area.height),
                &Preferences {
                    options: TRANSPORTS_SHORT,
                    ..Preferences::default()
                },
            ));
            empty_height.set(form.measure_with_data(
                ui,
                Constraints::loose(area.width, area.height),
                &Preferences {
                    options: NO_TRANSPORTS,
                    ..Preferences::default()
                },
            ));
            long_height.set(form.measure_with_data(
                ui,
                Constraints::loose(area.width, area.height),
                &data,
            ));
            narrow_height.set(form.measure_with_data(
                ui,
                Constraints::loose(1, area.height),
                &data,
            ));
            zero_width_height.set(form.measure_with_data(
                ui,
                Constraints::loose(0, area.height),
                &data,
            ));
            form.draw(ui, area, &FormState::default(), &data);
        })
        .commit_presented();

    let radio_label = runtime
        .area_of_part(TRANSPORT, PartRef::of(Part::LABEL))
        .expect("radio label registered");
    let channel_label = runtime
        .area_of_part(CHANNEL_NAME, PartRef::of(Part::LABEL))
        .expect("first half field label registered");
    let locale_label = runtime
        .area_of_part(LOCALE, PartRef::of(Part::LABEL))
        .expect("disabled second half field remains laid out");
    assert_eq!(channel_label.y - radio_label.y, 18);
    assert_eq!(locale_label.y, channel_label.y, "half fields share one row");
    assert!(
        runtime
            .area_of_part(ENDPOINT, PartRef::of(Part::LABEL))
            .is_none(),
        "hidden data has no rendered field bounds"
    );
    assert!(row_text(&buffer, radio_label.y + 10).contains("Archive"));
    let unicode_label_row = row_text(&buffer, 0);
    assert!(
        unicode_label_row.contains("表 示 名"),
        "wide Unicode label stays visible: {unicode_label_row:?}"
    );
    assert!(row_text(&buffer, channel_label.y + 1).contains("日 本 語"));
    assert!(row_text(&buffer, 2).contains("A deliberately long help description"));

    let actual_height = channel_label.y + 3;
    assert_eq!(
        legacy_height, 24,
        "legacy estimate remains available for compatibility"
    );
    assert_ne!(
        legacy_height, actual_height,
        "legacy estimate omits live option counts and row pairing"
    );
    assert_eq!(long_height.get().preferred.1, actual_height);
    assert_eq!(short_height.get().preferred.1, 20);
    assert_eq!(empty_height.get().preferred.1, 18);
    assert_eq!(narrow_height.get().preferred.0, 1);
    assert_eq!(narrow_height.get().preferred.1, actual_height);
    assert_eq!(zero_width_height.get().preferred.0, 0);
    assert_eq!(zero_width_height.get().preferred.1, actual_height);
    assert_eq!(
        data.validations.get(),
        0,
        "measurement must not validate domain data"
    );
}

#[test]
fn narrow_public_consumer_measures_and_draws_without_panicking() {
    let fields = fields();
    let form = Form::new(FORM, &fields).columns(2);
    let data = Preferences {
        options: TRANSPORTS_SHORT,
        ..Preferences::default()
    };
    let area = Rect::new(0, 0, 1, 30);
    let mut runtime = Runtime::new(EmptyApp, Theme::junie());
    let mut buffer = Buffer::empty(area);
    let mut measured = Size::default();

    runtime
        .draw_scene(area, &mut buffer, |ui, area| {
            measured =
                form.measure_with_data(ui, Constraints::loose(area.width, area.height), &data);
            form.draw(ui, area, &FormState::default(), &data);
        })
        .commit_presented();

    assert_eq!(measured.preferred, (1, 20));
    assert_eq!(buffer.area.width, 1);
}
