use std::any::Any;

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};

use crate::core::event::{Key, Outcome};
use crate::core::focus::{Focus, FocusChange, FocusRing};
use crate::core::hit::Axis;
use crate::core::id::WidgetId;
pub use crate::core::intent::DismissReason;
use crate::core::intent::Phase;
use crate::core::pointer::PointerEvent;
use crate::core::response::Response;
use crate::ui::ctx::{Backdrop, RenderCtx};
use crate::ui::modal::{Modal, ModalStack};
use crate::ui::popup::{Dismiss, Placement};
use crate::widgets::icon_button::IconButton;
use crate::widgets::keyhint::Hint;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerKind {
    Modal,

    Popover,

    Tooltip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerSpec {
    pub kind: LayerKind,

    pub placement: Placement,

    pub anchor: Option<Rect>,

    pub backdrop: Option<Backdrop>,

    pub dismiss: Dismiss,

    pub initial_focus: Option<WidgetId>,

    pub restore_focus: bool,

    pub inert_below: bool,

    pub hides_page_focus: bool,
}

impl LayerSpec {
    pub const fn modal(dismiss: Dismiss) -> Self {
        Self {
            kind: LayerKind::Modal,
            placement: Placement::Center,
            anchor: None,
            backdrop: Some(Backdrop::DEFAULT),
            dismiss,
            initial_focus: None,
            restore_focus: true,
            inert_below: true,
            hides_page_focus: true,
        }
    }

    pub const fn popover(dismiss: Dismiss) -> Self {
        Self {
            kind: LayerKind::Popover,
            placement: Placement::Below,
            anchor: None,
            backdrop: None,
            dismiss,
            initial_focus: None,
            restore_focus: true,
            inert_below: true,
            hides_page_focus: true,
        }
    }

    pub const fn tooltip() -> Self {
        Self {
            kind: LayerKind::Tooltip,
            placement: Placement::Below,
            anchor: None,
            backdrop: None,
            dismiss: Dismiss::LIGHT,
            initial_focus: None,
            restore_focus: false,
            inert_below: false,
            hides_page_focus: false,
        }
    }

    pub const fn anchor(mut self, anchor: Rect, placement: Placement) -> Self {
        self.anchor = Some(anchor);
        self.placement = placement;
        self
    }

    pub const fn backdrop(mut self, backdrop: Option<Backdrop>) -> Self {
        self.backdrop = backdrop;
        self
    }

    pub const fn initial_focus(mut self, id: WidgetId) -> Self {
        self.initial_focus = Some(id);
        self
    }

    pub const fn restore_focus(mut self, on: bool) -> Self {
        self.restore_focus = on;
        self
    }

    pub const fn inert_below(mut self, on: bool) -> Self {
        self.inert_below = on;
        self
    }

    pub const fn hides_page_focus(mut self, on: bool) -> Self {
        self.hides_page_focus = on;
        self
    }
}

pub trait Overlay {
    type Action;

    fn dismiss(&self) -> Dismiss;

    fn spec(&self) -> LayerSpec {
        LayerSpec::modal(self.dismiss())
    }

    fn on_dismiss(&mut self, reason: DismissReason) -> Response<Self::Action>;

    fn close_button(&self) -> Option<&IconButton> {
        None
    }
}

pub fn dismiss<O: Overlay + ?Sized>(
    overlay: &mut O,
    reason: DismissReason,
) -> Option<Response<O::Action>> {
    overlay
        .dismiss()
        .allows(reason)
        .then(|| overlay.on_dismiss(reason))
}

pub struct LayerCx<'a, W: ?Sized = ()> {
    pub focus: &'a mut Focus,
    pub ring: &'a FocusRing,
    pub app: &'a W,
}

pub trait Layer<R, W: ?Sized = ()>: Any {
    fn spec(&self) -> LayerSpec;

    fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, app: &W);

    fn on_key(&mut self, key: &Key, cx: &mut LayerCx<'_, W>) -> Outcome;

    fn pointer(&mut self, ev: &PointerEvent, cx: &mut LayerCx<'_, W>) -> Outcome {
        if let Some((axis, delta)) = ev.wheel() {
            return match axis {
                Axis::Vertical => self.on_wheel(delta, ev.pos()),
                Axis::Horizontal => self.on_wheel_h(delta, ev.pos()),
            };
        }
        match ev.phase() {
            Some(Phase::Press) => self.on_press(ev.id, ev.pos(), ev.count()),
            Some(Phase::Click) => self.on_click_count(ev.id, ev.pos(), ev.count(), cx),
            Some(Phase::Drag) => self.on_drag(ev.id, ev.pos()),
            Some(Phase::Release) => self.on_release(ev.id, ev.pos()),
            _ => Outcome::Ignored,
        }
    }

    fn on_click(&mut self, _id: WidgetId, _pos: Position, _cx: &mut LayerCx<'_, W>) -> Outcome {
        Outcome::Consumed
    }

    fn on_click_count(
        &mut self,
        id: WidgetId,
        pos: Position,
        _count: u8,
        cx: &mut LayerCx<'_, W>,
    ) -> Outcome {
        self.on_click(id, pos, cx)
    }

    fn on_press(&mut self, _id: WidgetId, _pos: Position, _count: u8) -> Outcome {
        Outcome::Ignored
    }

    fn on_drag(&mut self, _pressed: WidgetId, _pos: Position) -> Outcome {
        Outcome::Ignored
    }

    fn on_release(&mut self, _pressed: WidgetId, _pos: Position) -> Outcome {
        Outcome::Ignored
    }

    fn on_wheel(&mut self, _delta: i32, _pos: Position) -> Outcome {
        Outcome::Consumed
    }

    fn on_wheel_h(&mut self, _delta: i32, _pos: Position) -> Outcome {
        Outcome::Consumed
    }

    fn on_paste(&mut self, _text: &str) -> Outcome {
        Outcome::Ignored
    }

    fn on_blur(&mut self, _change: FocusChange) -> Outcome {
        Outcome::Ignored
    }

    fn on_tick(&mut self, _app: &W) -> Outcome {
        Outcome::Ignored
    }

    fn hints(&self, _focused: Option<WidgetId>) -> Vec<Hint> {
        Vec::new()
    }

    fn is_editing(&self) -> bool {
        false
    }

    fn animating(&self) -> bool {
        false
    }

    fn on_dismiss(&mut self, reason: DismissReason);

    fn take_result(&mut self) -> Option<LayerOut<R>>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerOut<R> {
    Close(R),

    Event(R),
}

impl<R> LayerOut<R> {
    pub fn into_inner(self) -> R {
        match self {
            Self::Close(r) | Self::Event(r) => r,
        }
    }

    pub fn closes(&self) -> bool {
        matches!(self, Self::Close(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LayerTag {
    pub kind: &'static str,
    pub key: String,
    pub n: usize,
}

impl LayerTag {
    pub fn new(kind: &'static str) -> Self {
        Self {
            kind,
            key: String::new(),
            n: 0,
        }
    }
    pub fn key(mut self, k: impl Into<String>) -> Self {
        self.key = k.into();
        self
    }
    pub fn n(mut self, n: usize) -> Self {
        self.n = n;
        self
    }
}

pub struct LayerEntry<R, W: ?Sized = (), O = ()> {
    pub layer: Box<dyn Layer<R, W>>,
    pub tag: LayerTag,
    pub owner: O,
}

impl<R: 'static, W: ?Sized + 'static, O> LayerEntry<R, W, O> {
    pub fn get<L: Layer<R, W>>(&self) -> Option<&L> {
        let any: &dyn Any = &*self.layer;
        any.downcast_ref()
    }

    pub fn get_mut<L: Layer<R, W>>(&mut self) -> Option<&mut L> {
        let any: &mut dyn Any = &mut *self.layer;
        any.downcast_mut()
    }
}

impl<R: 'static, W: ?Sized + 'static, O> Modal for LayerEntry<R, W, O> {
    fn dismiss(&self) -> Dismiss {
        self.layer.spec().dismiss
    }
}

pub type ShellLayers<R, W = (), O = ()> = ModalStack<LayerEntry<R, W, O>>;

pub enum Taken<R, W: ?Sized = (), O = ()> {
    Event(R),

    Closed(LayerEntry<R, W, O>, R),
}

pub enum Dismissed<R, W: ?Sized = (), O = ()> {
    None,

    Refused,

    Closed(LayerEntry<R, W, O>, Option<R>),
}

pub enum LayerPointer<R, W: ?Sized = (), O = ()> {
    Handled(Outcome),

    Dismissed(Dismissed<R, W, O>),
}

impl<R: 'static, W: ?Sized + 'static, O> ModalStack<LayerEntry<R, W, O>> {
    pub fn pointer(
        &mut self,
        ev: &PointerEvent,
        focus: &mut Focus,
        ring: &FocusRing,
        app: &W,
    ) -> LayerPointer<R, W, O> {
        let Some(top) = self.top_mut() else {
            return LayerPointer::Handled(Outcome::Ignored);
        };

        if ev.is(Phase::Click) && ev.id == crate::ui::modal::BACKDROP {
            return LayerPointer::Dismissed(self.dismiss(DismissReason::OutsideClick, focus));
        }
        let out = {
            let mut cx = LayerCx { focus, ring, app };
            top.layer.pointer(ev, &mut cx)
        };
        if out == Outcome::Ignored
            && ev.is(Phase::Click)
            && ev.part().part == crate::core::id::Part::BACKDROP
        {
            return LayerPointer::Dismissed(self.dismiss(DismissReason::OutsideClick, focus));
        }
        LayerPointer::Handled(out)
    }

    pub fn open(
        &mut self,
        layer: Box<dyn Layer<R, W>>,
        tag: LayerTag,
        owner: O,
        focus: &mut Focus,
    ) {
        let initial = layer.spec().initial_focus;
        self.push(LayerEntry { layer, tag, owner }, focus, initial);
    }

    pub fn take(&mut self, focus: &mut Focus) -> Option<Taken<R, W, O>> {
        match self.top_mut()?.layer.take_result()? {
            LayerOut::Event(r) => Some(Taken::Event(r)),
            LayerOut::Close(r) => {
                let entry = self.pop(focus)?;
                Some(Taken::Closed(entry, r))
            }
        }
    }

    pub fn dismiss(&mut self, reason: DismissReason, focus: &mut Focus) -> Dismissed<R, W, O> {
        let Some(top) = self.top_mut() else {
            return Dismissed::None;
        };
        if !top.layer.spec().dismiss.allows(reason) {
            return Dismissed::Refused;
        }
        top.layer.on_dismiss(reason);
        let result = top.layer.take_result().map(LayerOut::into_inner);
        match self.pop(focus) {
            Some(entry) => Dismissed::Closed(entry, result),
            None => Dismissed::None,
        }
    }

    pub fn on_blur(&mut self, change: FocusChange) -> Outcome {
        self.iter_mut()
            .fold(Outcome::Ignored, |out, e| out.or(e.layer.on_blur(change)))
    }

    pub fn hides_page_focus(&self) -> bool {
        self.iter().any(|e| e.layer.spec().hides_page_focus)
    }

    pub fn top_as<L: Layer<R, W>>(&self) -> Option<&L> {
        self.top()?.get()
    }

    pub fn top_as_mut<L: Layer<R, W>>(&mut self) -> Option<&mut L> {
        self.top_mut()?.get_mut()
    }
}

use crate::widgets::choice_dialog::ChoiceDialog;
use crate::widgets::completion::{Completion, CompletionAction};
use crate::widgets::dialog::{Dialog, DialogResult};
use crate::widgets::form_dialog::{FormAction, FormDialog};
use crate::widgets::help::HelpOverlay;
use crate::widgets::info_dialog::{InfoDialog, InfoResult};
use crate::widgets::menu::{ContextMenu, MenuAction, MenuBar, MenuBarAction};
use crate::widgets::picker::{Picker, PickerAction};
use crate::widgets::select::{Select, SelectAction};

impl Overlay for Dialog {
    type Action = DialogResult;

    fn dismiss(&self) -> Dismiss {
        self.dismiss_in_force()
    }
    fn spec(&self) -> LayerSpec {
        LayerSpec::modal(Overlay::dismiss(self)).initial_focus(self.initial_focus)
    }

    fn on_dismiss(&mut self, _: DismissReason) -> Response<DialogResult> {
        let _ = self.dismissed();
        Response::emit(self.result.unwrap_or(DialogResult::Cancelled))
    }
    fn close_button(&self) -> Option<&IconButton> {
        Some(&self.close_button)
    }
}

impl Overlay for FormDialog {
    type Action = FormAction;
    fn dismiss(&self) -> Dismiss {
        self.dismiss
    }
    fn spec(&self) -> LayerSpec {
        LayerSpec::modal(self.dismiss).initial_focus(self.initial_focus())
    }
    fn on_dismiss(&mut self, _: DismissReason) -> Response<FormAction> {
        self.events.push(FormAction::Cancel);
        Response::emit(FormAction::Cancel)
    }
    fn close_button(&self) -> Option<&IconButton> {
        Some(&self.close_button)
    }
}

impl Overlay for ChoiceDialog {
    type Action = Option<usize>;
    fn dismiss(&self) -> Dismiss {
        self.dismiss
    }
    fn spec(&self) -> LayerSpec {
        LayerSpec::modal(self.dismiss).initial_focus(self.initial_focus())
    }
    fn on_dismiss(&mut self, _: DismissReason) -> Response<Option<usize>> {
        self.result = Some(None);
        Response::emit(None)
    }
    fn close_button(&self) -> Option<&IconButton> {
        Some(&self.close_button)
    }
}

impl Overlay for InfoDialog {
    type Action = InfoResult;
    fn dismiss(&self) -> Dismiss {
        self.dismiss
    }
    fn spec(&self) -> LayerSpec {
        LayerSpec::modal(self.dismiss).initial_focus(self.initial_focus())
    }
    fn on_dismiss(&mut self, _: DismissReason) -> Response<InfoResult> {
        self.result = Some(InfoResult::Closed);
        Response::emit(InfoResult::Closed)
    }
    fn close_button(&self) -> Option<&IconButton> {
        Some(&self.close_button)
    }
}

impl Overlay for HelpOverlay {
    type Action = ();
    fn dismiss(&self) -> Dismiss {
        self.dismiss
    }

    fn spec(&self) -> LayerSpec {
        LayerSpec::modal(self.dismiss).initial_focus(self.done.id)
    }
    fn on_dismiss(&mut self, _: DismissReason) -> Response<()> {
        self.closed = true;
        Response::changed()
    }
    fn close_button(&self) -> Option<&IconButton> {
        Some(&self.close_button)
    }
}

impl Overlay for Picker {
    type Action = PickerAction;
    fn dismiss(&self) -> Dismiss {
        self.list.dismiss
    }
    fn spec(&self) -> LayerSpec {
        LayerSpec::modal(self.list.dismiss).initial_focus(self.id)
    }
    fn on_dismiss(&mut self, _: DismissReason) -> Response<PickerAction> {
        Response::emit(PickerAction::Cancelled)
    }
}

impl Overlay for ContextMenu {
    type Action = MenuAction;
    fn dismiss(&self) -> Dismiss {
        self.list.dismiss
    }
    fn spec(&self) -> LayerSpec {
        LayerSpec::popover(self.list.dismiss)
            .initial_focus(self.id)
            .hides_page_focus(self.hides_page_focus())
    }
    fn on_dismiss(&mut self, _: DismissReason) -> Response<MenuAction> {
        Response::emit(MenuAction::Dismissed)
    }
}

impl Overlay for MenuBar {
    type Action = MenuBarAction;
    fn dismiss(&self) -> Dismiss {
        self.dismiss
    }
    fn spec(&self) -> LayerSpec {
        LayerSpec::popover(self.dismiss).hides_page_focus(self.hides_page_focus())
    }
    fn on_dismiss(&mut self, _: DismissReason) -> Response<MenuBarAction> {
        self.close();
        Response::emit(MenuBarAction::Closed)
    }
}

impl Overlay for Select {
    type Action = SelectAction;
    fn dismiss(&self) -> Dismiss {
        self.list.dismiss
    }
    fn spec(&self) -> LayerSpec {
        LayerSpec::popover(self.list.dismiss)
    }
    fn on_dismiss(&mut self, _: DismissReason) -> Response<SelectAction> {
        self.close().into()
    }
}

impl Overlay for Completion {
    type Action = CompletionAction;
    fn dismiss(&self) -> Dismiss {
        self.list.dismiss
    }
    fn spec(&self) -> LayerSpec {
        LayerSpec::popover(self.list.dismiss).hides_page_focus(false)
    }
    fn on_dismiss(&mut self, _: DismissReason) -> Response<CompletionAction> {
        self.close();
        Response::emit(CompletionAction::Dismiss)
    }
}

impl Overlay for crate::widgets::option_editor::ChoiceEditor {
    type Action = crate::widgets::option_editor::ChoiceResult;
    fn dismiss(&self) -> Dismiss {
        self.list.dismiss
    }
    fn spec(&self) -> LayerSpec {
        LayerSpec::popover(self.list.dismiss)
    }
    fn on_dismiss(&mut self, _: DismissReason) -> Response<Self::Action> {
        Response::emit(crate::widgets::option_editor::ChoiceResult::Cancel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::intent::DismissReason as R;

    struct Ask {
        id: WidgetId,
        dismiss: Dismiss,
        result: Option<bool>,
    }
    impl Layer<bool> for Ask {
        fn spec(&self) -> LayerSpec {
            LayerSpec::modal(self.dismiss).initial_focus(self.id)
        }
        fn render(&mut self, _: Rect, _: &mut Buffer, _: &mut RenderCtx, _: &()) {}
        fn on_key(&mut self, key: &Key, _: &mut LayerCx<'_>) -> Outcome {
            if key.code == ratatui::crossterm::event::KeyCode::Enter {
                self.result = Some(true);
            }
            Outcome::Consumed
        }
        fn on_dismiss(&mut self, _: DismissReason) {
            self.result = Some(false);
        }
        fn take_result(&mut self) -> Option<LayerOut<bool>> {
            self.result.take().map(LayerOut::Close)
        }
    }

    #[test]
    fn every_library_overlay_resolves_a_dismissal_to_cancel() {
        let id = WidgetId::of("o");
        let mut d = Dialog::confirm(id, "Delete?", "Gone.", "Delete").dismiss(Dismiss::ESCAPE);
        assert!(dismiss(&mut d, R::OutsideClick).is_none(), "refused");
        assert_eq!(d.result, None);

        let cancel = d
            .cancel_index
            .map_or(DialogResult::Cancelled, DialogResult::Action);
        let r = dismiss(&mut d, R::CloseButton).unwrap();
        assert_eq!(r.into_action(), Some(cancel));
        assert_eq!(d.result, Some(cancel));
        let mut esc = Dialog::confirm(id, "Delete?", "Gone.", "Delete");
        let ring = FocusRing::default();
        let mut focus = Focus::default();
        let _ = esc.on_key(
            &Key {
                code: ratatui::crossterm::event::KeyCode::Esc,
                mods: ratatui::crossterm::event::KeyModifiers::NONE,
            },
            &mut focus,
            &ring,
        );
        assert_eq!(esc.result, Some(cancel), "Esc resolves the same way");

        let mut f = FormDialog::new(id, "Form", vec![]);
        assert!(dismiss(&mut f, R::OutsideClick).is_none());
        assert_eq!(
            dismiss(&mut f, R::Esc).unwrap().into_action(),
            Some(FormAction::Cancel)
        );
        assert_eq!(f.events, vec![FormAction::Cancel]);

        let mut c = ChoiceDialog::new(id, "Pick", "One", &["a", "b"], 1);
        assert_eq!(dismiss(&mut c, R::Esc).unwrap().into_action(), Some(None));
        assert_eq!(c.result, Some(None));

        let mut i = InfoDialog::new(id, "Info", vec![]);
        assert_eq!(
            dismiss(&mut i, R::OutsideClick).unwrap().into_action(),
            Some(InfoResult::Closed)
        );

        let mut h = HelpOverlay::new(id, "", vec![]);
        assert!(dismiss(&mut h, R::Esc).is_some());
        assert!(h.closed);
        let mut kept = HelpOverlay::new(id, "", vec![]).dismiss(Dismiss::NEVER);
        assert!(dismiss(&mut kept, R::Esc).is_none());
        assert!(!kept.closed);

        let mut p = Picker::new(id, "Go to");
        assert_eq!(
            dismiss(&mut p, R::Esc).unwrap().into_action(),
            Some(PickerAction::Cancelled)
        );
        let mut m = ContextMenu::new(id, vec![]);
        assert_eq!(
            dismiss(&mut m, R::FocusOut).unwrap().into_action(),
            Some(MenuAction::Dismissed)
        );
        assert_eq!(Overlay::spec(&m).kind, LayerKind::Popover);

        let mut s = Select::new(id, "Size", &["S", "M"], 0);
        s.open = true;
        assert!(dismiss(&mut s, R::OutsideClick).is_some());
        assert!(!s.open, "the list closed without a choice");
        assert_eq!(s.selected, 0);

        let mut bar = MenuBar::new(id, vec![("File", vec![])]);
        assert_eq!(
            dismiss(&mut bar, R::Esc).unwrap().into_action(),
            Some(MenuBarAction::Closed)
        );
        let mut comp = Completion::new(id);
        assert_eq!(
            dismiss(&mut comp, R::Esc).unwrap().into_action(),
            Some(CompletionAction::Dismiss)
        );
        assert!(!comp.spec().hides_page_focus, "the editor keeps its caret");
    }

    #[test]
    fn every_framed_modal_takes_a_custom_close_button() {
        use crate::ui::ctx::Interaction;
        let t = crate::theme::Theme::junie();
        let custom = IconButton::close(crate::ui::modal::CLOSE).with_glyph("✕");
        let screen = Rect::new(0, 0, 80, 24);
        let id = WidgetId::of("o");
        let draw = |f: &mut dyn FnMut(&mut Buffer, &mut RenderCtx)| {
            let mut hits = crate::core::hit::HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut buf = Buffer::empty(screen);
            {
                let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
                f(&mut buf, &mut ctx);
                ctx.flush_overlays(&mut buf);
            }
            let plate = hits
                .area_of(crate::ui::modal::CLOSE)
                .expect("the frame draws its close button");
            (plate.left()..plate.right())
                .map(|x| buf[(x, plate.y)].symbol().to_owned())
                .collect::<String>()
        };
        let mut help = HelpOverlay::new(id, "", vec![]).close_button(custom.clone());
        assert_eq!(draw(&mut |b, c| help.render(screen, b, c)), " ✕ ");
        let mut form = FormDialog::new(id, "Form", vec![]).close_button(custom.clone());
        assert_eq!(draw(&mut |b, c| form.render(screen, b, c)), " ✕ ");
        let mut choice =
            ChoiceDialog::new(id, "Pick", "One", &["a", "b"], 0).close_button(custom.clone());
        assert_eq!(draw(&mut |b, c| choice.render(screen, b, c)), " ✕ ");
        let mut info = InfoDialog::new(id, "Info", vec![]).close_button(custom.clone());
        assert_eq!(draw(&mut |b, c| info.render(screen, b, c)), " ✕ ");

        let mut plain = InfoDialog::new(id, "Info", vec![]);
        assert_eq!(draw(&mut |b, c| plain.render(screen, b, c)), " × ");
    }

    fn ask(name: &'static str, dismiss: Dismiss) -> Box<dyn Layer<bool>> {
        Box::new(Ask {
            id: WidgetId::of(name),
            dismiss,
            result: None,
        })
    }

    #[test]
    fn the_policy_decides_every_dismissal_reason() {
        for (d, esc, out, blur) in [
            (Dismiss::LIGHT, true, true, true),
            (Dismiss::ESCAPE, true, false, false),
            (Dismiss::NEVER, false, false, false),
        ] {
            assert_eq!(d.allows(R::Esc), esc);
            assert_eq!(d.allows(R::CloseButton), esc, "× means Esc");
            assert_eq!(d.allows(R::OutsideClick), out);
            assert_eq!(d.allows(R::FocusOut), blur);
            assert!(d.allows(R::Programmatic), "code may always close");
        }
        assert!(Dismiss::ESCAPE.focus_out(true).allows(R::FocusOut));
    }

    #[test]
    fn layers_restore_the_owner_focus_and_resolve_to_cancel() {
        let mut focus = Focus::default();
        let owner = WidgetId::of("page.button");
        focus.focus(owner);
        let mut layers: ShellLayers<bool> = ShellLayers::new();
        layers.open(
            ask("a", Dismiss::ESCAPE),
            LayerTag::new("a"),
            (),
            &mut focus,
        );
        assert_eq!(focus.current(), Some(WidgetId::of("a")));
        assert!(layers.hides_page_focus());
        assert!(matches!(
            layers.dismiss(R::OutsideClick, &mut focus),
            Dismissed::Refused
        ));
        assert_eq!(layers.len(), 1);
        match layers.dismiss(R::Esc, &mut focus) {
            Dismissed::Closed(entry, result) => {
                assert_eq!(entry.tag.kind, "a");
                assert_eq!(result, Some(false), "a dismissal is the cancel result");
            }
            _ => panic!("Esc closes an ESCAPE layer"),
        }
        assert_eq!(focus.current(), Some(owner));

        layers.open(ask("b", Dismiss::NEVER), LayerTag::new("b"), (), &mut focus);
        assert!(layers.take(&mut focus).is_none());
        let ring = FocusRing::default();
        let mut cx = LayerCx {
            focus: &mut focus,
            ring: &ring,
            app: &(),
        };
        let enter = Key {
            code: ratatui::crossterm::event::KeyCode::Enter,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        let _ = layers.top_mut().unwrap().layer.on_key(&enter, &mut cx);
        assert!(layers.top_as::<Ask>().is_some(), "downcast to the app type");
        let Some(Taken::Closed(entry, result)) = layers.take(&mut focus) else {
            panic!("Enter resolved it");
        };
        assert_eq!((entry.tag.kind, result), ("b", true));
        assert_eq!(focus.current(), Some(owner));
        assert!(matches!(
            layers.dismiss(R::Esc, &mut focus),
            Dismissed::None
        ));
    }
}
