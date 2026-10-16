//! Layers: every popup and modal above a page (plan §1.7, step S5).
//!
//! Two traits, one stack:
//!
//! - [`Overlay`] is the widget capability: a popup or modal says how it may
//!   be dismissed ([`Dismiss`]), what kind of layer it is ([`LayerSpec`])
//!   and what a dismissal resolves to ([`Overlay::on_dismiss`], always its
//!   cancel result). Implemented by `Select`, `Picker`, `Completion`,
//!   `ContextMenu`, `MenuBar`, `Dialog`, `FormDialog`, `ChoiceDialog`,
//!   `InfoDialog` and `HelpOverlay`.
//! - [`Layer`] is the object-safe entry an app puts on its stack: the
//!   overlay plus input, rendering, hints and a typed result `R` the owner
//!   receives when it closes. An app's own modal (tablepro's filter
//!   editor, jackin's operation flow) implements it directly.
//! - [`ShellLayers`] is the stack ([`ModalStack`] of [`LayerEntry`]): it
//!   moves focus into a layer when it opens, gives the owner its focus
//!   back when it closes (K9), and runs every dismissal through the top
//!   layer's policy, so no app checks "may Esc close this?" itself.
//!
//! Names follow `main` (termrock): `LayerSpec`, `LayerKind`,
//! `DismissReason`; [`Dismiss`] keeps this branch's richer policy
//! (pass-through, refusal feedback, focus-out).

use std::any::Any;

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};

use crate::core::event::{Key, Outcome};
use crate::core::focus::{Focus, FocusChange, FocusRing};
use crate::core::id::WidgetId;
pub use crate::core::intent::DismissReason;
use crate::core::response::Response;
use crate::ui::ctx::{Backdrop, RenderCtx};
use crate::ui::modal::{Modal, ModalStack};
use crate::ui::popup::{Dismiss, Placement};
use crate::widgets::icon_button::IconButton;
use crate::widgets::keyhint::Hint;

/// What kind of layer a popup or modal is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerKind {
    /// Dims the page and traps focus until it closes (dialogs, forms,
    /// help, pickers).
    Modal,
    /// Floats next to an anchor without dimming (select lists, menus,
    /// completion); the owner keeps keyboard focus.
    Popover,
    /// Shows on hover or focus and never takes input.
    Tooltip,
}

/// How a layer opens, where it goes and how it closes (`main`:
/// termrock `LayerSpec`). Built from the existing [`Placement`],
/// [`Backdrop`] and [`Dismiss`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerSpec {
    pub kind: LayerKind,
    /// Where it goes relative to [`Self::anchor`] (centred without one).
    pub placement: Placement,
    /// The rectangle it opens next to. `None`: centred in the body.
    pub anchor: Option<Rect>,
    /// The dimmed backdrop under a modal; `None` dims nothing.
    pub backdrop: Option<Backdrop>,
    /// Which dismissals close it.
    pub dismiss: Dismiss,
    /// Where focus goes when it opens. `None`: focus stays (a popover
    /// whose owner keeps the keyboard).
    pub initial_focus: Option<WidgetId>,
    /// Closing it gives focus back to the control it covered (K9).
    /// Default on.
    pub restore_focus: bool,
    /// Nothing below it reacts to the pointer while it is open.
    pub inert_below: bool,
    /// The page under it draws no focus indicator, so the only one on
    /// screen is in this layer (AGENTS.md). Off for a context menu whose
    /// owner row stays marked.
    pub hides_page_focus: bool,
}

impl LayerSpec {
    /// A centred modal: dims the page, traps focus, hides the page focus.
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

    /// An anchored popover: no dimming; the page below is inert while it
    /// is open (a click there is the outside click).
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

    /// A tooltip: takes no input and changes no focus.
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

    /// Open next to `anchor` per `placement`.
    pub const fn anchor(mut self, anchor: Rect, placement: Placement) -> Self {
        self.anchor = Some(anchor);
        self.placement = placement;
        self
    }
    /// See [`Self::backdrop`].
    pub const fn backdrop(mut self, backdrop: Option<Backdrop>) -> Self {
        self.backdrop = backdrop;
        self
    }
    /// See [`Self::initial_focus`].
    pub const fn initial_focus(mut self, id: WidgetId) -> Self {
        self.initial_focus = Some(id);
        self
    }
    /// See [`Self::restore_focus`].
    pub const fn restore_focus(mut self, on: bool) -> Self {
        self.restore_focus = on;
        self
    }
    /// See [`Self::inert_below`].
    pub const fn inert_below(mut self, on: bool) -> Self {
        self.inert_below = on;
        self
    }
    /// See [`Self::hides_page_focus`].
    pub const fn hides_page_focus(mut self, on: bool) -> Self {
        self.hides_page_focus = on;
        self
    }
}

/// The widget capability of a popup or modal (plan §1.4).
pub trait Overlay {
    /// What the widget reports (`SelectAction`, `PickerAction`, ...).
    type Action;

    /// Which dismissals close it without choosing.
    fn dismiss(&self) -> Dismiss;

    /// How it opens and closes. Default: a centred modal under
    /// [`Self::dismiss`].
    fn spec(&self) -> LayerSpec {
        LayerSpec::modal(self.dismiss())
    }

    /// Close it as cancelled for `reason`. Call it through [`dismiss`],
    /// which checks the policy first; a dismissal always resolves to the
    /// cancel result (AGENTS.md).
    fn on_dismiss(&mut self, reason: DismissReason) -> Response<Self::Action>;

    /// The `×` its frame draws when [`Dismiss::shows_close`]. `None`: the
    /// overlay has no frame button (a popover or a menu, which
    /// light-dismiss instead).
    fn close_button(&self) -> Option<&IconButton> {
        None
    }
}

/// Run `reason` through `overlay`'s [`Dismiss`] policy: `None` when the
/// policy refuses it (the layer stays; an outside click may flash it),
/// otherwise the overlay's cancel result.
pub fn dismiss<O: Overlay + ?Sized>(
    overlay: &mut O,
    reason: DismissReason,
) -> Option<Response<O::Action>> {
    overlay
        .dismiss()
        .allows(reason)
        .then(|| overlay.on_dismiss(reason))
}

/// What a layer's handlers may touch: focus and the ring of the frame
/// they were drawn in, and the app's own state `W` (read-only; changes go
/// through the result the layer reports).
pub struct LayerCx<'a, W: ?Sized = ()> {
    pub focus: &'a mut Focus,
    pub ring: &'a FocusRing,
    pub app: &'a W,
}

/// An entry on an app's layer stack (plan §1.7): an [`Overlay`] with
/// input, rendering and hints, reporting a typed result `R` to the owner
/// when it closes. Object-safe; `W` is app state a layer reads while it
/// draws or handles input (default none).
///
/// Every handler has a "do nothing" default except the three a layer
/// cannot do without: [`Self::spec`], [`Self::render`] and
/// [`Self::on_key`]. A layer that resolves (a button chose, a dismissal
/// cancelled) keeps the result until [`Self::take_result`].
pub trait Layer<R, W: ?Sized = ()>: Any {
    /// How it opens and closes.
    fn spec(&self) -> LayerSpec;
    /// Draw it (the frame, its backdrop and its content).
    fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, app: &W);
    /// A key while it is the top layer.
    fn on_key(&mut self, key: &Key, cx: &mut LayerCx<'_, W>) -> Outcome;
    /// A completed click on `id` inside it.
    fn on_click(&mut self, _id: WidgetId, _pos: Position, _cx: &mut LayerCx<'_, W>) -> Outcome {
        Outcome::Consumed
    }
    /// A click on `id` with a click count (double, triple).
    fn on_click_count(
        &mut self,
        id: WidgetId,
        pos: Position,
        _count: u8,
        cx: &mut LayerCx<'_, W>,
    ) -> Outcome {
        self.on_click(id, pos, cx)
    }
    /// The primary button went down on `id` inside it.
    fn on_press(&mut self, _id: WidgetId, _pos: Position, _count: u8) -> Outcome {
        Outcome::Ignored
    }
    /// A drag that started on `pressed`.
    fn on_drag(&mut self, _pressed: WidgetId, _pos: Position) -> Outcome {
        Outcome::Ignored
    }
    /// The button came up after a press on `pressed`.
    fn on_release(&mut self, _pressed: WidgetId, _pos: Position) -> Outcome {
        Outcome::Ignored
    }
    /// The vertical wheel over it.
    fn on_wheel(&mut self, _delta: i32, _pos: Position) -> Outcome {
        Outcome::Consumed
    }
    /// The horizontal wheel (or Shift+wheel) over it.
    fn on_wheel_h(&mut self, _delta: i32, _pos: Position) -> Outcome {
        Outcome::Consumed
    }
    /// Pasted text.
    fn on_paste(&mut self, _text: &str) -> Outcome {
        Outcome::Ignored
    }
    /// Focus moved between controls inside it (an edit ends on blur).
    fn on_blur(&mut self, _change: FocusChange) -> Outcome {
        Outcome::Ignored
    }
    /// The app's tick while it is open (a spinner, a live list).
    fn on_tick(&mut self, _app: &W) -> Outcome {
        Outcome::Ignored
    }
    /// The footer's key hints while it is on top.
    fn hints(&self, _focused: Option<WidgetId>) -> Vec<Hint> {
        Vec::new()
    }
    /// Resolve to the cancel result for `reason`. [`ShellLayers::dismiss`]
    /// calls it only when [`Self::spec`]'s policy allows `reason`.
    fn on_dismiss(&mut self, reason: DismissReason);
    /// The result, once it resolved (taken once).
    fn take_result(&mut self) -> Option<R>;
    /// Whether `id` is one of its controls (routes a click to it).
    fn owns(&self, _id: WidgetId) -> bool {
        false
    }
}

/// Identifies what a layer is for, so its owner can route the result:
/// a kind, a key (the target's identity) and a number.
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

/// A layer on the stack with its tag and the page that owns it.
pub struct LayerEntry<R, W: ?Sized = (), O = ()> {
    pub layer: Box<dyn Layer<R, W>>,
    pub tag: LayerTag,
    pub owner: O,
}

impl<R: 'static, W: ?Sized + 'static, O> LayerEntry<R, W, O> {
    /// The layer as its concrete type, if it is an `L`.
    pub fn get<L: Layer<R, W>>(&self) -> Option<&L> {
        let any: &dyn Any = &*self.layer;
        any.downcast_ref()
    }
    /// The layer as its concrete type, if it is an `L`.
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

/// An app's layer stack (plan §1.7): opening a layer moves focus to its
/// [`LayerSpec::initial_focus`] and remembers the focus it covers; closing
/// it gives that focus back (K9).
pub type ShellLayers<R, W = (), O = ()> = ModalStack<LayerEntry<R, W, O>>;

/// How a dismissal of the top layer went.
pub enum Dismissed<R, W: ?Sized = (), O = ()> {
    /// No layer is open.
    None,
    /// The policy refuses it: the layer stays.
    Refused,
    /// The layer closed as cancelled; its result goes to the owner.
    Closed(LayerEntry<R, W, O>, Option<R>),
}

impl<R: 'static, W: ?Sized + 'static, O> ModalStack<LayerEntry<R, W, O>> {
    /// Open `layer` on top for `owner`.
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

    /// The top layer resolved: close it (restoring focus) and hand back
    /// its entry and result. `None` while it is still open.
    pub fn take_finished(&mut self, focus: &mut Focus) -> Option<(LayerEntry<R, W, O>, R)> {
        let result = self.top_mut()?.layer.take_result()?;
        let entry = self.pop(focus)?;
        Some((entry, result))
    }

    /// Dismiss the top layer for `reason` under its policy. A closed layer
    /// gives focus back and reports its cancel result.
    pub fn dismiss(&mut self, reason: DismissReason, focus: &mut Focus) -> Dismissed<R, W, O> {
        let Some(top) = self.top_mut() else {
            return Dismissed::None;
        };
        if !top.layer.spec().dismiss.allows(reason) {
            return Dismissed::Refused;
        }
        top.layer.on_dismiss(reason);
        let result = top.layer.take_result();
        match self.pop(focus) {
            Some(entry) => Dismissed::Closed(entry, result),
            None => Dismissed::None,
        }
    }

    /// Whether a layer that hides the page focus is open: pass it to
    /// [`RenderCtx::page_under_layer`] at frame start.
    pub fn hides_page_focus(&self) -> bool {
        self.iter().any(|e| e.layer.spec().hides_page_focus)
    }

    /// The top layer as its concrete type, if it is an `L`.
    pub fn top_as<L: Layer<R, W>>(&self) -> Option<&L> {
        self.top()?.get()
    }
    /// The top layer as its concrete type, if it is an `L`.
    pub fn top_as_mut<L: Layer<R, W>>(&mut self) -> Option<&mut L> {
        self.top_mut()?.get_mut()
    }
}

// ---------------------------------------------------------------- library
// overlays: every popup and modal of the library resolves a dismissal to
// its cancel result here, in one place.

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
        self.dismiss
    }
    fn spec(&self) -> LayerSpec {
        LayerSpec::modal(self.dismiss).initial_focus(self.initial_focus)
    }
    fn on_dismiss(&mut self, _: DismissReason) -> Response<DialogResult> {
        self.result = Some(DialogResult::Cancelled);
        Response::emit(DialogResult::Cancelled)
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
        LayerSpec::modal(self.dismiss).initial_focus(self.id)
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
        self.dismiss
    }
    fn spec(&self) -> LayerSpec {
        LayerSpec::modal(self.dismiss).initial_focus(self.id)
    }
    fn on_dismiss(&mut self, _: DismissReason) -> Response<PickerAction> {
        Response::emit(PickerAction::Cancelled)
    }
}

impl Overlay for ContextMenu {
    type Action = MenuAction;
    fn dismiss(&self) -> Dismiss {
        self.dismiss
    }
    fn spec(&self) -> LayerSpec {
        LayerSpec::popover(self.dismiss)
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
        self.dismiss
    }
    fn spec(&self) -> LayerSpec {
        // the field keeps the keyboard; its list marks the cursor with a
        // fill, never a second focus bar
        LayerSpec::popover(self.dismiss)
    }
    fn on_dismiss(&mut self, _: DismissReason) -> Response<SelectAction> {
        self.close().into()
    }
}

impl Overlay for Completion {
    type Action = CompletionAction;
    fn dismiss(&self) -> Dismiss {
        self.dismiss
    }
    fn spec(&self) -> LayerSpec {
        // the editor keeps the keyboard and its caret
        LayerSpec::popover(self.dismiss).hides_page_focus(false)
    }
    fn on_dismiss(&mut self, _: DismissReason) -> Response<CompletionAction> {
        self.close();
        Response::emit(CompletionAction::Dismiss)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::intent::DismissReason as R;

    /// A minimal layer: Enter confirms, a dismissal cancels.
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
        fn take_result(&mut self) -> Option<bool> {
            self.result.take()
        }
    }

    /// Dismissal per library overlay: the policy decides, and a dismissal
    /// always resolves to the overlay's cancel result.
    #[test]
    fn every_library_overlay_resolves_a_dismissal_to_cancel() {
        let id = WidgetId::of("o");
        let mut d = Dialog::confirm(id, "Delete?", "Gone.", "Delete").dismiss(Dismiss::ESCAPE);
        assert!(dismiss(&mut d, R::OutsideClick).is_none(), "refused");
        assert_eq!(d.result, None);
        let r = dismiss(&mut d, R::CloseButton).unwrap();
        assert_eq!(r.into_action(), Some(DialogResult::Cancelled));
        assert_eq!(d.result, Some(DialogResult::Cancelled));

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
        assert_eq!(m.spec().kind, LayerKind::Popover);

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

    /// Open finding (c): HelpOverlay, FormDialog, ChoiceDialog and
    /// InfoDialog take a custom close button the way Frame and Dialog do,
    /// through [`Overlay::close_button`] and [`Frame::overlay`].
    ///
    /// [`Frame::overlay`]: crate::ui::modal::Frame::overlay
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
        // the default stays the theme's `×`
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

    /// K9: closing a layer, by result or by dismissal, gives the owner its
    /// focus back; a refused dismissal leaves it open and focus inside.
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
        // a result closes it too, through the same focus restore
        layers.open(ask("b", Dismiss::NEVER), LayerTag::new("b"), (), &mut focus);
        assert!(layers.take_finished(&mut focus).is_none());
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
        let (entry, result) = layers.take_finished(&mut focus).unwrap();
        assert_eq!((entry.tag.kind, result), ("b", true));
        assert_eq!(focus.current(), Some(owner));
        assert!(matches!(
            layers.dismiss(R::Esc, &mut focus),
            Dismissed::None
        ));
    }
}
