use ratatui::Terminal;
use ratatui::backend::TestBackend;
use termrock::{
    Action, ActionKey, App, Cx, FieldKind, FieldMut, FieldRef, FieldSizingPolicy, FieldSpec,
    FormData, FormDialog, FormDialogSlotExt, FormDialogState, FormDialogSubmitPolicy, Id, Rect,
    Response, Runtime, TextInput, Theme, Ui,
};

const DIALOG: Id = Id::root("example.form-dialog");
const DISPLAY: Id = Id::root("example.form-dialog.display");
const OPEN: ActionKey = ActionKey::custom("example.form-dialog.open");
const CLOSE: ActionKey = ActionKey::custom("example.form-dialog.close");
const ACTIONS: &[Action<'static>] = &[
    Action::quiet(CLOSE, "Cancel"),
    Action::new(OPEN, "Accept changes"),
];
const FIELDS: &[FieldSpec<'static>] = &[FieldSpec::new(
    DISPLAY,
    "Unicode label",
    FieldKind::Text(TextInput::new(DISPLAY).placeholder("Enter a value…")),
)];

#[derive(Default)]
struct Data {
    display: String,
}

impl FormData for Data {
    fn value(&self, id: Id) -> FieldRef<'_> {
        if id == DISPLAY {
            FieldRef::Text(&self.display)
        } else {
            FieldRef::Text("")
        }
    }

    fn value_mut(&mut self, id: Id) -> FieldMut<'_> {
        if id == DISPLAY {
            FieldMut::Text(&mut self.display)
        } else {
            FieldMut::ReadOnly
        }
    }
}

#[derive(Default)]
struct ExampleApp {
    state: FormDialogState,
    data: Data,
    opened: bool,
}

impl ExampleApp {
    fn dialog(&self) -> FormDialog<'static> {
        FormDialog::new(DIALOG, "Edit label", FIELDS)
            .actions(ACTIONS)
            .submit(OPEN)
            .cancel_action(CLOSE)
            .field_sizing(FieldSizingPolicy::Intrinsic)
            .submit_policy(FormDialogSubmitPolicy::KeepOpen)
    }
}

impl App for ExampleApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let component = self.dialog();
        if !self.opened {
            self.opened = true;
            cx.open_layer(DIALOG, component.layer(cx, &self.data));
        }
        component
            .update(cx, &mut self.state, &mut self.data)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let component = self.dialog();
        ui.layer_slot(DIALOG, |slot| {
            slot.form_dialog(&component, &self.state, &self.data);
        });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let area = Rect::new(0, 0, 72, 24);
    let mut runtime = Runtime::new(ExampleApp::default(), Theme::junie());
    let _ = runtime.initialize();
    let prepared = runtime.prepare(area)?;
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height))?;
    terminal.draw(|frame| {
        runtime
            .draw_prepared(prepared, frame)
            .expect("prepared area matches terminal");
    })?;
    runtime.acknowledge_presented(prepared)?;
    Ok(())
}
