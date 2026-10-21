//! Form composition through the shared modal dialog and form components.

use core::fmt;

use ratatui_core::layout::Rect;
use termrock_core::action::{Action, ActionKey};
use termrock_core::id::{Id, Part};
use termrock_core::response::{Flow, Invalidate, Response};
use termrock_layout::Constraints;
use termrock_overlays::dialog::{Dialog, DialogAction, DialogState};
use termrock_runtime::layer::{Dismiss, DismissReason, LayerSpec};
use termrock_runtime::{Cx, Ui};

use crate::form::{FieldKind, FieldSizingPolicy, FieldSpec, Form, FormAction, FormData, FormState};
use crate::theme::DesignTokens;
use termrock_runtime::FrameRead;
use termrock_runtime::author::LayerSlotAuthor;

/// Controls whether a successfully validated submit closes the dialog.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FormDialogSubmitPolicy {
    /// Close after the form emits its configured submit action.
    #[default]
    CloseOnSubmit,
    /// Leave the dialog open so the caller can accept or reject the result.
    KeepOpen,
}

/// Controls how many form rows the dialog body requests.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FormDialogLayoutPolicy {
    /// Measure the visible form from its live view data.
    #[default]
    Intrinsic,
    /// Request a fixed number of body rows; Form still clips and scrolls data.
    FixedBodyRows(u16),
}

/// Typed result of a form dialog update.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FormDialogAction {
    Changed(Id),
    Committed(Id),
    Chose(Id),
    Action(ActionKey),
    Invalid(Id),
    Cancelled,
}

/// Persistent state for one form dialog.
///
/// Field values remain owned by the caller's [`FormData`]. This state only
/// contains the existing form editing/scroll state and dialog lifecycle state.
#[derive(Default)]
pub struct FormDialogState {
    form: FormState,
    dialog: DialogState,
}

impl Clone for FormDialogState {
    fn clone(&self) -> Self {
        FormDialogState {
            form: self.form.clone(),
            dialog: self.dialog.clone(),
        }
    }
}

impl PartialEq for FormDialogState {
    fn eq(&self, other: &Self) -> bool {
        self.form == other.form && self.dialog == other.dialog
    }
}

impl Eq for FormDialogState {}

impl fmt::Debug for FormDialogState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FormDialogState")
            .field("form", &self.form)
            .field("dialog", &self.dialog)
            .finish()
    }
}

impl FormDialogState {
    pub const fn form(&self) -> &FormState {
        &self.form
    }

    pub fn form_mut(&mut self) -> &mut FormState {
        &mut self.form
    }

    pub const fn dialog(&self) -> &DialogState {
        &self.dialog
    }
}

/// Reusable modal form. Field behavior and the action row are delegated to
/// [`Form`]; title, chrome, layer scope, and dismissal are delegated to
/// [`Dialog`].
pub struct FormDialog<'a> {
    id: Id,
    title: &'a str,
    description: Option<&'a str>,
    meta: Option<&'a str>,
    width: Option<u16>,
    fields: &'a [FieldSpec<'a>],
    actions: &'a [Action<'a>],
    submit: ActionKey,
    cancel: ActionKey,
    dismiss: Dismiss,
    field_sizing: FieldSizingPolicy,
    layout: FormDialogLayoutPolicy,
    submit_policy: FormDialogSubmitPolicy,
}

impl fmt::Debug for FormDialog<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FormDialog")
            .field("id", &self.id)
            .field("title", &self.title)
            .field("fields", &self.fields.len())
            .field("actions", &self.actions.len())
            .field("submit", &self.submit)
            .field("cancel", &self.cancel)
            .field("dismiss", &self.dismiss)
            .field("layout", &self.layout)
            .field("submit_policy", &self.submit_policy)
            .finish_non_exhaustive()
    }
}

impl<'a> FormDialog<'a> {
    pub const fn new(id: Id, title: &'a str, fields: &'a [FieldSpec<'a>]) -> Self {
        FormDialog {
            id,
            title,
            description: None,
            meta: None,
            width: None,
            fields,
            actions: &[],
            submit: ActionKey::SAVE,
            cancel: ActionKey::CANCEL,
            dismiss: Dismiss::ESC,
            field_sizing: FieldSizingPolicy::Fixed,
            layout: FormDialogLayoutPolicy::Intrinsic,
            submit_policy: FormDialogSubmitPolicy::CloseOnSubmit,
        }
    }

    #[must_use]
    pub const fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    #[must_use]
    pub const fn meta(mut self, meta: &'a str) -> Self {
        self.meta = Some(meta);
        self
    }

    #[must_use]
    pub const fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    #[must_use]
    pub const fn actions(mut self, actions: &'a [Action<'a>]) -> Self {
        self.actions = actions;
        self
    }

    #[must_use]
    pub const fn submit(mut self, submit: ActionKey) -> Self {
        self.submit = submit;
        self
    }

    #[must_use]
    pub const fn cancel_action(mut self, cancel: ActionKey) -> Self {
        self.cancel = cancel;
        self
    }

    #[must_use]
    pub const fn dismiss(mut self, dismiss: Dismiss) -> Self {
        self.dismiss = dismiss;
        self
    }

    #[must_use]
    pub const fn field_sizing(mut self, sizing: FieldSizingPolicy) -> Self {
        self.field_sizing = sizing;
        self
    }

    #[must_use]
    pub const fn layout_policy(mut self, policy: FormDialogLayoutPolicy) -> Self {
        self.layout = policy;
        self
    }

    #[must_use]
    pub const fn submit_policy(mut self, policy: FormDialogSubmitPolicy) -> Self {
        self.submit_policy = policy;
        self
    }

    const fn form_id(&self) -> Id {
        self.id.part(Part::BODY)
    }

    fn form(&self) -> Form<'a> {
        Form::new(self.form_id(), self.fields)
            .actions(self.actions)
            .submit(self.submit)
            .field_sizing(self.field_sizing)
    }

    fn dialog<'d>(&'d self, body_rows: u16) -> Dialog<'d> {
        let mut dialog = Dialog::chrome(self.id)
            .title(self.title)
            .body_rows(body_rows);
        if let Some(description) = self.description {
            dialog = dialog.description(description);
        }
        if let Some(meta) = self.meta {
            dialog = dialog.meta(meta);
        }
        if let Some(width) = self.width {
            dialog = dialog.width(width);
        }
        dialog
    }

    fn requested_width(&self, design: &DesignTokens, viewport: Rect) -> u16 {
        self.width
            .unwrap_or(design.size.dialog_width)
            .min(viewport.width)
    }

    fn body_rows<D: FormData + ?Sized>(
        &self,
        design: &DesignTokens,
        viewport: Rect,
        data: &D,
    ) -> u16 {
        match self.layout {
            FormDialogLayoutPolicy::FixedBodyRows(rows) => rows,
            FormDialogLayoutPolicy::Intrinsic => {
                // Dialog content is inset by three cells on both sides.
                let body_width = self.requested_width(design, viewport).saturating_sub(6);
                self.form()
                    .measure_with_data_design(
                        design,
                        Constraints::loose(body_width, u16::MAX),
                        data,
                    )
                    .preferred
                    .1
            }
        }
    }

    fn first_focus<D: FormData + ?Sized>(&self, data: &D) -> Option<Id> {
        self.fields
            .iter()
            .find(|field| {
                data.visible(field.id)
                    && !data.disabled(field.id)
                    && !matches!(field.kind, FieldKind::Note)
            })
            .map(|field| field.id)
            .or_else(|| {
                self.actions
                    .iter()
                    .position(|action| action.is_enabled())
                    .map(|index| self.form_id().part(Part::ACTIONS).index(index))
            })
    }

    /// Returns the modal layer specification with live intrinsic body sizing.
    pub fn layer<D: FormData + ?Sized>(&self, cx: &Cx<'_>, data: &D) -> LayerSpec {
        let rows = self.body_rows(cx.design(), cx.viewport(), data);
        let mut spec = self.dialog(rows).layer(cx).dismiss(self.dismiss);
        if let Some(id) = self.first_focus(data) {
            spec = spec.initial_focus(id);
        }
        spec
    }

    /// Updates dialog dismissal and form controls from one resolved runtime
    /// input frame. Measurement reads only design and view data.
    pub fn update<D: FormData + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        state: &mut FormDialogState,
        data: &mut D,
    ) -> Response<FormDialogAction> {
        let rows = self.body_rows(cx.design(), cx.viewport(), data);
        let dialog = self.dialog(rows);
        // The form gets first refusal on field-scoped Escape (for example an
        // editor or an open select). The layer then handles remaining dismissals.
        let form_response = self.form().update(cx, &mut state.form, data);
        let dialog_response = dialog.update(cx, &mut state.dialog);

        let mut action = form_response
            .action_ref()
            .copied()
            .map(|action| match action {
                FormAction::Changed(id) => FormDialogAction::Changed(id),
                FormAction::Committed(id) => FormDialogAction::Committed(id),
                FormAction::Chose(id) => FormDialogAction::Chose(id),
                FormAction::Invalid(id) => FormDialogAction::Invalid(id),
                FormAction::Action(key) if key == self.cancel => FormDialogAction::Cancelled,
                FormAction::Action(key) => FormDialogAction::Action(key),
            });

        if let Some(FormAction::Action(key)) = form_response.action_ref().copied()
            && (key == self.cancel
                || (key == self.submit
                    && self.submit_policy == FormDialogSubmitPolicy::CloseOnSubmit))
        {
            cx.close_layer(self.id, Some(key));
        }

        if action == Some(FormDialogAction::Cancelled)
            || (form_response.action_ref().is_some_and(
                |action| matches!(action, FormAction::Action(key) if *key == self.cancel),
            ))
        {
            action = Some(FormDialogAction::Cancelled);
        }

        if action.is_none()
            && let Some(dialog_action) = dialog_response.action_ref().copied()
        {
            match dialog_action {
                DialogAction::Dismissed(DismissReason::Programmatic) => {}
                DialogAction::Dismissed(_) => action = Some(FormDialogAction::Cancelled),
                DialogAction::Action(key) if key == self.cancel => {
                    cx.close_layer(self.id, Some(self.cancel));
                    action = Some(FormDialogAction::Cancelled);
                }
                DialogAction::Action(key) => action = Some(FormDialogAction::Action(key)),
            }
        }

        combine_responses(self.id, action, form_response, dialog_response)
    }

    /// Draws the dialog chrome and delegates its resolved body slot to Form.
    pub fn draw<D: FormData + ?Sized>(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        state: &FormDialogState,
        data: &D,
    ) {
        let rows = self.body_rows(ui.design(), area, data);
        let form = self.form();
        self.dialog(rows)
            .draw_body_slot(ui, area, &state.dialog, |slot| {
                slot.render_component(|ui, body| {
                    form.draw(ui, body, &state.form, data);
                });
            });
    }
}

fn combine_responses(
    id: Id,
    action: Option<FormDialogAction>,
    form: Response<FormAction>,
    dialog: Response<DialogAction>,
) -> Response<FormDialogAction> {
    let flow = form.flow().max(dialog.flow());
    let invalidate = form.invalidate().max(dialog.invalidate());
    let response = if let Some(action) = action {
        Response::action(action)
    } else if flow == Flow::Consumed {
        Response::consumed()
    } else {
        Response::ignored()
    };
    let response = match invalidate {
        Invalidate::None => response,
        Invalidate::Paint => response.repaint(),
        Invalidate::Layout => response.relayout(),
    };
    response.for_id(id)
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Rect;
    use termrock_core::action::{Action, ActionKey};
    use termrock_core::event::{Input, Key, KeyCode, KeyModifiers, MouseKind};
    use termrock_core::id::Id;
    use termrock_core::response::Response;
    use termrock_fields::input::TextInput;
    use termrock_runtime::layer::{Dismiss, LayerSpec};
    use termrock_runtime::{App, Cx, Runtime, Ui};
    use termrock_theme::Theme;

    use super::*;
    use crate::form::{FieldMut, FieldRef};

    const DIALOG: Id = Id::root("form-dialog.tests");
    const NAME: Id = Id::root("form-dialog.tests.name");
    const NOTE: Id = Id::root("form-dialog.tests.note");
    const SAVE: ActionKey = ActionKey::custom("form-dialog.tests.save");
    const CANCEL: ActionKey = ActionKey::custom("form-dialog.tests.cancel");
    const ACTIONS: &[Action<'static>] =
        &[Action::quiet(CANCEL, "Cancel"), Action::new(SAVE, "Save")];
    const FIELDS: &[FieldSpec<'static>] = &[
        FieldSpec::new(NAME, "Name", FieldKind::Text(TextInput::new(NAME))),
        FieldSpec::new(NOTE, "Details", FieldKind::Note),
    ];
    const SCREEN: Rect = Rect::new(0, 0, 64, 24);

    #[derive(Default)]
    struct Data {
        name: String,
        required: bool,
        notes: Vec<(&'static str, termrock_theme::Role)>,
    }

    impl FormData for Data {
        fn value(&self, id: Id) -> FieldRef<'_> {
            if id == NAME {
                FieldRef::Text(&self.name)
            } else if id == NOTE {
                FieldRef::Note(&self.notes)
            } else {
                FieldRef::Text("")
            }
        }

        fn value_mut(&mut self, id: Id) -> FieldMut<'_> {
            if id == NAME {
                FieldMut::Text(&mut self.name)
            } else {
                FieldMut::ReadOnly
            }
        }

        fn validate(&self, id: Id, value: FieldRef<'_>) -> Result<(), termrock_core::FieldError> {
            if self.required && id == NAME && matches!(value, FieldRef::Text("")) {
                Err(termrock_core::FieldError::new("Name is required"))
            } else {
                Ok(())
            }
        }
    }

    #[derive(Default)]
    struct TestApp {
        state: FormDialogState,
        data: Data,
        opened: bool,
        layer: Option<LayerSpec>,
        actions: Vec<FormDialogAction>,
    }

    impl TestApp {
        fn dialog(&self) -> FormDialog<'static> {
            FormDialog::new(DIALOG, "Edit record", FIELDS)
                .actions(ACTIONS)
                .submit(SAVE)
                .cancel_action(CANCEL)
                .field_sizing(FieldSizingPolicy::Intrinsic)
                .width(42)
        }
    }

    impl App for TestApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if !self.opened {
                self.opened = true;
                let dialog = self.dialog();
                let layer = dialog.layer(cx, &self.data);
                self.layer = Some(layer);
                cx.open_layer(DIALOG, layer);
            }
            let dialog = self.dialog();
            let response = dialog.update(cx, &mut self.state, &mut self.data);
            if let Some(action) = response.action_ref() {
                self.actions.push(*action);
            }
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let dialog = self.dialog();
            ui.layer(DIALOG, |ui, area| {
                dialog.draw(ui, area, &self.state, &self.data)
            });
        }
    }

    fn press(code: KeyCode) -> Input {
        Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        })
    }

    #[test]
    fn dialog_open_focuses_first_field_and_submit_validates_before_acceptance() {
        let mut app = TestApp::default();
        app.data.required = true;
        let mut runtime = Runtime::new(app, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert!(runtime.is_open(DIALOG));
        assert!(runtime.render_snapshot().is_ok());

        let save_id = DIALOG.part(Part::BODY).part(Part::ACTIONS).index(1);
        let save = runtime
            .area_of(save_id)
            .expect("visible Save button hit area");
        let point = save.x.saturating_add(save.width / 2);
        let row = save.y.saturating_add(save.height / 2);
        let _ = termrock_runtime::stub::deliver(
            &mut runtime,
            termrock_runtime::stub::mouse(MouseKind::Down, point, row),
        );
        let _ = termrock_runtime::stub::deliver(
            &mut runtime,
            termrock_runtime::stub::mouse(MouseKind::Up, point, row),
        );
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(
            runtime.app().actions.last(),
            Some(&FormDialogAction::Invalid(NAME))
        );
        assert!(runtime.render_snapshot().is_ok());
        assert!(runtime.is_open(DIALOG));
    }

    #[test]
    fn focused_editor_handles_escape_before_dialog_dismissal() {
        let mut runtime = Runtime::new(TestApp::default(), Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert!(runtime.render_snapshot().is_ok());
        let _ = termrock_runtime::stub::deliver(
            &mut runtime,
            termrock_runtime::stub::mouse(MouseKind::Down, 0, 0),
        );
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert!(runtime.is_open(DIALOG));
        let _ = termrock_runtime::stub::deliver(&mut runtime, press(KeyCode::Esc));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert!(runtime.is_open(DIALOG));
        let _ = termrock_runtime::stub::deliver(&mut runtime, press(KeyCode::Esc));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert!(!runtime.is_open(DIALOG));
        assert_eq!(
            runtime.app().actions.last(),
            Some(&FormDialogAction::Cancelled)
        );
    }

    #[test]
    fn dialog_and_form_measurement_use_live_intrinsic_content() {
        let app = TestApp::default();
        let mut runtime = Runtime::new(app, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let first = runtime.layer_area(DIALOG).expect("dialog layer");
        runtime.app_mut().data.name = "Δelta 🧪".to_owned();
        runtime.app_mut().data.notes = vec![
            (
                "First live row",
                termrock_theme::Role::Fg(termrock_theme::FgStep::Primary),
            ),
            (
                "Second live row",
                termrock_theme::Role::Fg(termrock_theme::FgStep::Primary),
            ),
            (
                "Third live row",
                termrock_theme::Role::Fg(termrock_theme::FgStep::Primary),
            ),
        ];
        let _ = termrock_runtime::stub::deliver(&mut runtime, Input::Tick);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let second = runtime
            .layer_area(DIALOG)
            .expect("dialog layer after update");
        assert!(second.height > first.height);
        assert!(second.x >= buffer.area.x);
        assert!(second.y >= buffer.area.y);
        assert!(second.right() <= buffer.area.right());
        assert!(second.bottom() <= buffer.area.bottom());

        let narrow = Rect::new(0, 0, 18, 8);
        let _ = termrock_runtime::stub::deliver(&mut runtime, Input::Resize(18, 8));
        let mut narrow_buffer = Buffer::empty(narrow);
        runtime
            .draw_buffer(narrow, &mut narrow_buffer)
            .commit_presented();
        let clipped = runtime.layer_area(DIALOG).expect("dialog survives resize");
        assert!(clipped.width <= narrow.width);
        assert!(clipped.height <= narrow.height);
        assert!(clipped.right() <= narrow.right());
        assert!(clipped.bottom() <= narrow.bottom());
    }

    #[test]
    fn layer_spec_keeps_escape_only_dismissal_and_focuses_first_field() {
        let mut runtime = Runtime::new(TestApp::default(), Theme::junie());
        let _ = runtime.initialize();
        let spec = runtime.app().layer.expect("layer spec recorded by app");
        assert_eq!(spec.dismiss, Dismiss::ESC);
        assert_eq!(spec.initial_focus, Some(NAME));
    }
}
