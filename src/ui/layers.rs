//! The library's modals and popups as [`Layer`]s: an app pushes a
//! `Dialog`, `Picker`, `ContextMenu`, `FormDialog`, `ChoiceDialog`,
//! `InfoDialog` or `HelpOverlay` onto its [`ShellLayers`] as is, and
//! receives a [`LayerResult`] (through `From<LayerResult>` for its own
//! result type). Input routing, focus restore and dismissal are the
//! stack's; no app matches on which modal is open (plan S5).
//!
//! [`ShellLayers`]: crate::ui::layer::ShellLayers

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};

use crate::core::event::{Key, Outcome};
use crate::core::focus::FocusChange;
use crate::core::hit::Axis;
use crate::core::id::WidgetId;
use crate::core::pointer::{MouseTarget, PointerEvent};
use crate::core::scroll::Scrollable;
use crate::ui::ctx::RenderCtx;
use crate::ui::layer::{DismissReason, Layer, LayerCx, LayerOut, LayerSpec, Overlay};
use crate::widgets::choice_dialog::ChoiceDialog;
use crate::widgets::dialog::{Dialog, DialogBody, DialogResult};
use crate::widgets::form_dialog::{FormAction, FormDialog};
use crate::widgets::help::HelpOverlay;
use crate::widgets::info_dialog::{InfoDialog, InfoResult};
use crate::widgets::keyhint::Hint;
use crate::widgets::menu::{ContextMenu, MenuAction};
use crate::widgets::picker::{Picker, PickerAction};

/// What a library modal reports to its owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerResult {
    /// A [`Dialog`] resolved: the pressed action's index (`None`:
    /// cancelled) and a prompt's text.
    Dialog {
        action: Option<usize>,
        text: Option<String>,
    },
    /// A [`Picker`] reported: `Chosen`, `ChosenAlt` and `Cancelled` close
    /// it; every other action is an event (the owner re-supplies rows on
    /// `QueryChanged`, cycles scopes on `NextScope`/`PrevScope`).
    Picker(PickerAction),
    /// A [`ContextMenu`] closed: `Chosen(i)` or `Dismissed`.
    Menu(MenuAction),
    /// A [`FormDialog`] reported: `Save` and `Cancel` close it (read the
    /// values from the closed entry); `Changed`, `Choose` and `Action`
    /// are events.
    Form(FormAction),
    /// A [`ChoiceDialog`] closed with the chosen option (`None`:
    /// cancelled).
    Choice(Option<usize>),
    /// An [`InfoDialog`] reported: `Copy` is an event (the dialog stays
    /// open), the rest close it.
    Info(InfoResult),
    /// A [`HelpOverlay`] closed.
    Help,
}

fn out<R: From<LayerResult>>(r: LayerResult, closes: bool) -> LayerOut<R> {
    if closes {
        LayerOut::Close(r.into())
    } else {
        LayerOut::Event(r.into())
    }
}

// ------------------------------------------------------------------ Dialog

impl<R: From<LayerResult> + 'static, W: ?Sized + 'static> Layer<R, W> for Dialog {
    fn spec(&self) -> LayerSpec {
        Overlay::spec(self)
    }
    fn is_editing(&self) -> bool {
        Dialog::is_editing(self)
    }
    fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, _: &W) {
        Dialog::render(self, screen, buf, ctx);
    }
    fn on_key(&mut self, key: &Key, cx: &mut LayerCx<'_, W>) -> Outcome {
        Dialog::on_key(self, key, cx.focus, cx.ring).or(Outcome::Consumed)
    }
    fn pointer(&mut self, ev: &PointerEvent, cx: &mut LayerCx<'_, W>) -> Outcome {
        Dialog::pointer_in(self, ev, cx.focus).outcome()
    }
    fn on_click_count(
        &mut self,
        id: WidgetId,
        pos: Position,
        count: u8,
        cx: &mut LayerCx<'_, W>,
    ) -> Outcome {
        Dialog::on_click_count(self, id, pos, count, cx.focus)
    }
    fn on_click(&mut self, id: WidgetId, pos: Position, cx: &mut LayerCx<'_, W>) -> Outcome {
        Dialog::on_click_count(self, id, pos, 1, cx.focus)
    }
    fn on_press(&mut self, id: WidgetId, pos: Position, count: u8) -> Outcome {
        Dialog::on_press(self, id, pos, count)
    }
    fn on_drag(&mut self, pressed: WidgetId, pos: Position) -> Outcome {
        Dialog::on_drag(self, pressed, pos)
    }
    fn on_wheel(&mut self, delta: i32, _: Position) -> Outcome {
        Scrollable::wheel(self, Axis::Vertical, delta).outcome()
    }
    fn on_wheel_h(&mut self, delta: i32, _: Position) -> Outcome {
        Scrollable::wheel(self, Axis::Horizontal, delta).outcome()
    }
    fn on_paste(&mut self, text: &str) -> Outcome {
        Dialog::on_paste(self, text)
    }
    fn on_blur(&mut self, change: FocusChange) -> Outcome {
        Dialog::on_blur(self, change)
    }
    fn hints(&self, focused: Option<WidgetId>) -> Vec<Hint> {
        Dialog::hints(self, focused)
    }
    fn on_dismiss(&mut self, reason: DismissReason) {
        let _ = Overlay::on_dismiss(self, reason);
    }
    fn take_result(&mut self) -> Option<LayerOut<R>> {
        let action = match self.result.take()? {
            DialogResult::Action(i) => Some(i),
            DialogResult::Cancelled => None,
        };
        let text = match &self.body {
            DialogBody::Input(i) => Some(i.text().to_owned()),
            _ => None,
        };
        Some(out(LayerResult::Dialog { action, text }, true))
    }
}

// ------------------------------------------------------------------ Picker

/// A picker reports through its return values; the adapter keeps the
/// last one until the stack takes it.
fn picker_event(p: &mut Picker, ev: Option<PickerAction>) {
    if let Some(ev) = ev {
        p.pending = Some(ev);
    }
}

impl<R: From<LayerResult> + 'static, W: ?Sized + 'static> Layer<R, W> for Picker {
    fn spec(&self) -> LayerSpec {
        Overlay::spec(self)
    }
    fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, _: &W) {
        Picker::draw(self, screen, buf, ctx);
    }
    fn on_key(&mut self, key: &Key, _: &mut LayerCx<'_, W>) -> Outcome {
        let (o, ev) = Picker::on_key(self, key);
        picker_event(self, ev);
        o.or(Outcome::Consumed)
    }
    /// The picker's own gestures ([`MouseTarget`]), its backdrop's click
    /// included: an outside click cancels it under its dismiss policy.
    fn pointer(&mut self, ev: &PointerEvent, _: &mut LayerCx<'_, W>) -> Outcome {
        // its backdrop is the stack's outside click (ShellLayers::pointer)
        if ev.part().part == crate::core::id::Part::BACKDROP {
            return Outcome::Ignored;
        }
        let mut r = MouseTarget::pointer(self, ev);
        let ev = r.take_action();
        picker_event(self, ev);
        r.outcome()
    }
    fn on_click(&mut self, id: WidgetId, _: Position, _: &mut LayerCx<'_, W>) -> Outcome {
        let ev = Picker::on_click(self, id);
        picker_event(self, ev);
        Outcome::Changed
    }
    fn on_wheel(&mut self, delta: i32, _: Position) -> Outcome {
        Scrollable::wheel(&mut self.list, Axis::Vertical, delta).outcome()
    }
    fn on_paste(&mut self, text: &str) -> Outcome {
        let (o, ev) = Picker::on_paste(self, text);
        picker_event(self, ev);
        o
    }
    fn on_dismiss(&mut self, reason: DismissReason) {
        let ev = Overlay::on_dismiss(self, reason).into_action();
        picker_event(self, ev);
    }
    fn take_result(&mut self) -> Option<LayerOut<R>> {
        let ev = self.pending.take()?;
        let closes = matches!(
            ev,
            PickerAction::Chosen(_) | PickerAction::ChosenAlt(_) | PickerAction::Cancelled
        );
        Some(out(LayerResult::Picker(ev), closes))
    }
}

// ------------------------------------------------------------- ContextMenu

impl<R: From<LayerResult> + 'static, W: ?Sized + 'static> Layer<R, W> for ContextMenu {
    fn spec(&self) -> LayerSpec {
        Overlay::spec(self)
    }
    fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, _: &W) {
        ContextMenu::render(self, screen, buf, ctx);
    }
    fn on_key(&mut self, key: &Key, _: &mut LayerCx<'_, W>) -> Outcome {
        let (o, ev) = ContextMenu::on_key(self, key);
        if ev.is_some() {
            self.pending = ev;
        }
        o.or(Outcome::Consumed)
    }
    fn pointer(&mut self, ev: &PointerEvent, _: &mut LayerCx<'_, W>) -> Outcome {
        let mut r = MouseTarget::pointer(self, ev);
        if let Some(ev) = r.take_action() {
            self.pending = Some(ev);
        }
        r.outcome()
    }
    fn on_click(&mut self, id: WidgetId, _: Position, _: &mut LayerCx<'_, W>) -> Outcome {
        if let Some(ev) = ContextMenu::on_click(self, id) {
            self.pending = Some(ev);
        }
        Outcome::Changed
    }
    fn on_wheel(&mut self, delta: i32, _: Position) -> Outcome {
        ContextMenu::on_wheel(self, delta)
    }
    fn on_dismiss(&mut self, reason: DismissReason) {
        self.pending = Overlay::on_dismiss(self, reason).into_action();
    }
    fn take_result(&mut self) -> Option<LayerOut<R>> {
        self.pending
            .take()
            .map(|ev| out(LayerResult::Menu(ev), true))
    }
}

// -------------------------------------------------------------- FormDialog

impl<R: From<LayerResult> + 'static, W: ?Sized + 'static> Layer<R, W> for FormDialog {
    fn spec(&self) -> LayerSpec {
        Overlay::spec(self)
    }
    fn is_editing(&self) -> bool {
        FormDialog::is_editing(self)
    }
    fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, _: &W) {
        FormDialog::render(self, screen, buf, ctx);
    }
    fn on_key(&mut self, key: &Key, cx: &mut LayerCx<'_, W>) -> Outcome {
        FormDialog::on_key(self, key, cx.focus, cx.ring).or(Outcome::Consumed)
    }
    /// Routed by the event's owner: a wheel over an open select scrolls
    /// its list, not the form behind it (M18).
    fn pointer(&mut self, ev: &PointerEvent, cx: &mut LayerCx<'_, W>) -> Outcome {
        FormDialog::pointer_in(self, ev, cx.focus).outcome()
    }
    fn on_click_count(
        &mut self,
        id: WidgetId,
        pos: Position,
        count: u8,
        cx: &mut LayerCx<'_, W>,
    ) -> Outcome {
        FormDialog::on_click_count(self, id, pos, count, cx.focus)
    }
    fn on_click(&mut self, id: WidgetId, pos: Position, cx: &mut LayerCx<'_, W>) -> Outcome {
        FormDialog::on_click_count(self, id, pos, 1, cx.focus)
    }
    fn on_press(&mut self, id: WidgetId, pos: Position, count: u8) -> Outcome {
        FormDialog::on_press(self, id, pos, count)
    }
    fn on_drag(&mut self, pressed: WidgetId, pos: Position) -> Outcome {
        FormDialog::on_drag(self, pressed, pos)
    }
    fn on_release(&mut self, pressed: WidgetId, pos: Position) -> Outcome {
        FormDialog::on_release(self, pressed, pos)
    }
    fn on_wheel(&mut self, delta: i32, _: Position) -> Outcome {
        Scrollable::wheel(self, Axis::Vertical, delta).outcome()
    }
    fn on_paste(&mut self, text: &str) -> Outcome {
        FormDialog::on_paste(self, text)
    }
    fn on_blur(&mut self, change: FocusChange) -> Outcome {
        FormDialog::on_blur(self, change)
    }
    fn on_dismiss(&mut self, reason: DismissReason) {
        let _ = Overlay::on_dismiss(self, reason);
    }
    fn take_result(&mut self) -> Option<LayerOut<R>> {
        if self.events.is_empty() {
            return None;
        }
        let ev = self.events.remove(0);
        let closes = matches!(ev, FormAction::Save | FormAction::Cancel);
        Some(out(LayerResult::Form(ev), closes))
    }
}

// ------------------------------------------------------------ ChoiceDialog

impl<R: From<LayerResult> + 'static, W: ?Sized + 'static> Layer<R, W> for ChoiceDialog {
    fn spec(&self) -> LayerSpec {
        Overlay::spec(self)
    }
    fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, _: &W) {
        ChoiceDialog::render(self, screen, buf, ctx);
    }
    fn on_key(&mut self, key: &Key, cx: &mut LayerCx<'_, W>) -> Outcome {
        ChoiceDialog::on_key(self, key, cx.focus, cx.ring).or(Outcome::Consumed)
    }
    fn pointer(&mut self, ev: &PointerEvent, cx: &mut LayerCx<'_, W>) -> Outcome {
        ChoiceDialog::pointer_in(self, ev, cx.focus).outcome()
    }
    fn on_click(&mut self, id: WidgetId, _: Position, cx: &mut LayerCx<'_, W>) -> Outcome {
        ChoiceDialog::on_click(self, id, cx.focus)
    }
    fn on_dismiss(&mut self, reason: DismissReason) {
        let _ = Overlay::on_dismiss(self, reason);
    }
    fn take_result(&mut self) -> Option<LayerOut<R>> {
        self.result
            .take()
            .map(|r| out(LayerResult::Choice(r), true))
    }
}

// -------------------------------------------------------------- InfoDialog

impl<R: From<LayerResult> + 'static, W: ?Sized + 'static> Layer<R, W> for InfoDialog {
    fn spec(&self) -> LayerSpec {
        Overlay::spec(self)
    }
    fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, _: &W) {
        InfoDialog::render(self, screen, buf, ctx);
    }
    fn on_key(&mut self, key: &Key, cx: &mut LayerCx<'_, W>) -> Outcome {
        InfoDialog::on_key(self, key, cx.focus, cx.ring).or(Outcome::Consumed)
    }
    fn pointer(&mut self, ev: &PointerEvent, cx: &mut LayerCx<'_, W>) -> Outcome {
        InfoDialog::pointer_in(self, ev, cx.focus).outcome()
    }
    fn on_click(&mut self, id: WidgetId, pos: Position, cx: &mut LayerCx<'_, W>) -> Outcome {
        InfoDialog::on_click(self, id, pos, cx.focus)
    }
    /// A drag from a press on the detail's scrollbar keeps placing the
    /// thumb under the pointer, as the press on the track does.
    fn on_drag(&mut self, pressed: WidgetId, pos: Position) -> Outcome {
        if pressed != crate::widgets::scrollbar::id_for(self.id) {
            return Outcome::Ignored;
        }
        // the scrollbar takes no focus
        InfoDialog::on_click(
            self,
            pressed,
            pos,
            &mut crate::core::focus::Focus::default(),
        )
    }
    fn on_wheel(&mut self, delta: i32, _: Position) -> Outcome {
        Scrollable::wheel(self, Axis::Vertical, delta).outcome()
    }
    fn on_dismiss(&mut self, reason: DismissReason) {
        let _ = Overlay::on_dismiss(self, reason);
    }
    fn take_result(&mut self) -> Option<LayerOut<R>> {
        let r = self.result.take()?;
        let closes = !matches!(r, InfoResult::Copy(_));
        Some(out(LayerResult::Info(r), closes))
    }
}

// ------------------------------------------------------------- HelpOverlay

impl<R: From<LayerResult> + 'static, W: ?Sized + 'static> Layer<R, W> for HelpOverlay {
    fn spec(&self) -> LayerSpec {
        Overlay::spec(self)
    }
    fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, _: &W) {
        HelpOverlay::render(self, screen, buf, ctx);
    }
    fn on_key(&mut self, key: &Key, _: &mut LayerCx<'_, W>) -> Outcome {
        HelpOverlay::on_key(self, key).or(Outcome::Consumed)
    }
    fn pointer(&mut self, ev: &PointerEvent, _: &mut LayerCx<'_, W>) -> Outcome {
        MouseTarget::pointer(self, ev).outcome()
    }
    fn on_click(&mut self, id: WidgetId, _: Position, _: &mut LayerCx<'_, W>) -> Outcome {
        HelpOverlay::on_click(self, id)
    }
    fn on_wheel(&mut self, delta: i32, _: Position) -> Outcome {
        Scrollable::wheel(self, Axis::Vertical, delta).outcome()
    }
    fn on_dismiss(&mut self, reason: DismissReason) {
        let _ = Overlay::on_dismiss(self, reason);
    }
    fn take_result(&mut self) -> Option<LayerOut<R>> {
        if !self.closed {
            return None;
        }
        self.closed = false;
        Some(out(LayerResult::Help, true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::{Focus, FocusRing};
    use crate::ui::layer::{Dismissed, LayerTag, ShellLayers, Taken};
    use crate::ui::popup::Dismiss;
    use ratatui::crossterm::event::{KeyCode, KeyModifiers};

    fn key(code: KeyCode) -> Key {
        Key {
            code,
            mods: KeyModifiers::NONE,
        }
    }

    /// The library modals on one stack: results reach the owner, events
    /// leave the layer open, and focus comes back to the page (K9).
    #[test]
    fn library_modals_report_through_the_stack() {
        let page = WidgetId::of("page");
        let mut focus = Focus::default();
        focus.focus(page);
        let ring = FocusRing::default();
        let mut layers: ShellLayers<LayerResult> = ShellLayers::new();

        let d = Dialog::confirm(WidgetId::of("d"), "Delete?", "Gone.", "Delete");
        layers.open(Box::new(d), LayerTag::new("confirm"), (), &mut focus);
        assert_ne!(focus.current(), Some(page));
        let mut cx = LayerCx {
            focus: &mut focus,
            ring: &ring,
            app: &(),
        };
        let _ = layers
            .top_mut()
            .unwrap()
            .layer
            .on_key(&key(KeyCode::Esc), &mut cx);
        let Some(Taken::Closed(entry, r)) = layers.take(&mut focus) else {
            panic!("Esc resolved the dialog");
        };
        assert_eq!(entry.tag.kind, "confirm");
        // Esc runs the dialog's Cancel action
        let cancel = entry.get::<Dialog>().unwrap().cancel_index;
        assert_eq!(
            r,
            LayerResult::Dialog {
                action: cancel,
                text: None
            }
        );
        assert_eq!(focus.current(), Some(page), "K9: focus back on the page");

        // a picker's query edit is an event; Esc closes it as cancelled
        let p = Picker::new(WidgetId::of("p"), "Go to");
        layers.open(Box::new(p), LayerTag::new("go"), (), &mut focus);
        let mut cx = LayerCx {
            focus: &mut focus,
            ring: &ring,
            app: &(),
        };
        let _ = layers
            .top_mut()
            .unwrap()
            .layer
            .on_key(&key(KeyCode::Char('a')), &mut cx);
        assert!(matches!(
            layers.take(&mut focus),
            Some(Taken::Event(LayerResult::Picker(
                PickerAction::QueryChanged
            )))
        ));
        assert_eq!(layers.len(), 1, "an event leaves it open");
        match layers.dismiss(DismissReason::OutsideClick, &mut focus) {
            Dismissed::Closed(_, r) => {
                assert_eq!(r, Some(LayerResult::Picker(PickerAction::Cancelled)))
            }
            _ => panic!("a light picker closes on an outside click"),
        }
        assert_eq!(focus.current(), Some(page));

        // a refused dismissal leaves an ESCAPE form open
        let f = FormDialog::new(WidgetId::of("f"), "Form", vec![]);
        assert_eq!(Overlay::dismiss(&f), Dismiss::ESCAPE);
        layers.open(Box::new(f), LayerTag::new("form"), (), &mut focus);
        assert!(matches!(
            layers.dismiss(DismissReason::OutsideClick, &mut focus),
            Dismissed::Refused
        ));
        match layers.dismiss(DismissReason::CloseButton, &mut focus) {
            Dismissed::Closed(entry, r) => {
                assert_eq!(r, Some(LayerResult::Form(FormAction::Cancel)));
                assert!(entry.get::<FormDialog>().is_some(), "values stay readable");
            }
            _ => panic!("× closes an ESCAPE form"),
        }
        assert_eq!(focus.current(), Some(page));
    }
}
