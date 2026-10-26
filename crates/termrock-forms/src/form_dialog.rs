use core::fmt;

use ratatui_core::layout::Rect;
use termrock_core::action::{Action, ActionKey};
use termrock_core::id::{Id, Part};
use termrock_core::response::{Flow, Invalidate, Response};
use termrock_layout::Constraints;
use termrock_overlays::dialog::{Dialog, DialogAction, DialogState};
use termrock_runtime::layer::{Dismiss, DismissReason, LayerSpec};
use termrock_runtime::{Cx, Ui};

use crate::form::{
    EnterPolicy, FieldKind, FieldSizingPolicy, FieldSpec, Form, FormAction, FormData, FormState,
};
use crate::theme::DesignTokens;
use termrock_runtime::FrameRead;
use termrock_runtime::author::LayerSlotAuthor;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FormDialogSubmitPolicy {
    #[default]
    CloseOnSubmit,

    KeepOpen,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FormDialogLayoutPolicy {
    #[default]
    Intrinsic,

    FixedBodyRows(u16),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FormDialogAction {
    Changed(Id),
    Committed(Id),
    Chose(Id),
    Action(ActionKey),
    Invalid(Id),
    Cancelled,
}

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
    enter_policy: EnterPolicy,
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
            enter_policy: EnterPolicy::SubmitsWhenIdle,
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
    pub const fn enter_policy(mut self, policy: EnterPolicy) -> Self {
        self.enter_policy = policy;
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
            .enter(self.enter_policy)
            .field_sizing(self.field_sizing)
    }

    fn dialog<'d>(&'d self, body_rows: u16) -> Dialog<'d> {
        let mut dialog = Dialog::chrome(self.id)
            .title(self.title)
            .body_rows(body_rows)
            .close_button(self.cancel);
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

    pub fn layer<D: FormData + ?Sized>(&self, cx: &Cx<'_>, data: &D) -> LayerSpec {
        let rows = self.body_rows(cx.design(), cx.viewport(), data);
        let mut spec = self.dialog(rows).layer(cx).dismiss(self.dismiss);
        if let Some(id) = self.first_focus(data) {
            spec = spec.initial_focus(id);
        }
        spec
    }

    pub fn update<D: FormData + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        state: &mut FormDialogState,
        data: &mut D,
    ) -> Response<FormDialogAction> {
        let rows = self.body_rows(cx.design(), cx.viewport(), data);
        let dialog = self.dialog(rows);

        let form_response = self.form().update(cx, &mut state.form, data);

        let dialog_response = if matches!(form_response.action_ref(), Some(FormAction::Chose(_))) {
            Response::ignored()
        } else {
            dialog.update(cx, &mut state.dialog)
        };

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

    pub fn submit_current<D: FormData + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        state: &mut FormDialogState,
        data: &mut D,
    ) -> Response<FormDialogAction> {
        let form_response = self.form().submit_current(cx, &mut state.form, data);
        let action = form_response
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

        combine_responses(self.id, action, form_response, Response::ignored())
    }

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
