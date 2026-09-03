//! `Dialog` — layer content (`COMPONENT_ARCHITECTURE.md` §9.2, §14.1, §17.0
//! A7, Appendix A 4F).

use core::fmt;

use ratatui_core::layout::Rect;

use super::button::Button;
use super::field::Field;
use super::input::{TextAction, TextInput, TextInputState};
use super::{Acc, Overrides};
use crate::action::{Action, ActionKey};
use crate::event::{Chord, KeyCode};
use crate::id::{Id, Part, PartRef};
use crate::intent::Intent;
use crate::keymap::{Binding, BindingState, Bindings};
use crate::layer::{DismissReason, LayerEvent};
use crate::layout::{RowAlign, action_row};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::text::wrap;
use crate::theme::{Family, StylePatch, Surface, Variant};
use crate::ui::{Cx, FrameRead, Ui};

/// What a dialog reports.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DialogAction {
    /// An action-row button fired.
    Action(ActionKey),
    /// The layer was dismissed (Esc, outside click, …).
    Dismissed(DismissReason),
}

/// The const-constructible commands of the dialog keymap.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DialogCmd {
    /// Focus the previous enabled action.
    PrevAction,
    /// Focus the next enabled action.
    NextAction,
}

const BINDINGS: &[Binding<DialogCmd>] = &[
    Binding {
        chord: Chord::key(KeyCode::Left),
        cmd: DialogCmd::PrevAction,
        label: "Prev",
        priority: 20,
        visible: false,
    },
    Binding {
        chord: Chord::key(KeyCode::Right),
        cmd: DialogCmd::NextAction,
        label: "Next",
        priority: 20,
        visible: false,
    },
];

const CONFIRM_ACTIONS: [Action<'static>; 2] = [
    Action::quiet(ActionKey::CANCEL, "Cancel"),
    Action::new(ActionKey::CONFIRM, "OK"),
];
const DESTRUCTIVE_ACTIONS: [Action<'static>; 2] = [
    Action::new(ActionKey::CANCEL, "Cancel"),
    Action::danger(ActionKey::CONFIRM, "Delete"),
];
const ACK_ACTIONS: [Action<'static>; 2] = [
    Action::new(ActionKey::CANCEL, "Cancel"),
    Action::danger(ActionKey::CONFIRM, "Confirm"),
];

/// Durable state of a [`Dialog`]: the prompt / acknowledgement draft.
///
/// The draft is uncontrolled (S4's documented exception: a throwaway
/// field); read it with [`DialogState::draft`]. `Debug` redacts it.
#[derive(Clone, PartialEq, Eq, Default)]
pub struct DialogState {
    input: TextInputState,
    draft: String,
}

impl fmt::Debug for DialogState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DialogState")
            .field("input", &self.input)
            .field("draft", &"[redacted]")
            .finish()
    }
}

impl DialogState {
    /// The committed prompt text.
    pub fn draft(&self) -> &str {
        &self.draft
    }

    /// Overwrite the drafts.
    pub fn zeroize(&mut self) {
        self.input.zeroize();
        let mut bytes = core::mem::take(&mut self.draft).into_bytes();
        bytes.fill(0);
        self.draft = String::new();
    }
}

/// A titled surface with a description, an optional prompt or typed
/// acknowledgement, an arbitrary body slot and a right-aligned action row.
///
/// ## Construction
/// `Dialog::new(id)`; conveniences over the same path: `confirm(id, title,
/// question)`, `destructive(id, title, question)`, `prompt(id, title,
/// label)`, `acknowledge(id, title, token)`. The dialog is layer
/// **content**: open the layer with `cx.open_layer(id, LayerSpec::modal(id))`
/// and draw inside `ui.layer(id, …)`.
///
/// ## Ownership
/// The caller owns a [`DialogState`] (the prompt draft); the layer stack,
/// trap, backdrop, Esc, outside-click and focus restore belong to the
/// runtime.
///
/// ## Configuration
/// `.title(&str)`, `.description(&str)`, `.actions(&[Action])`,
/// `.cancel(ActionKey)` (the action Esc stands for), `.width(u16)`
/// (`design.size.dialog_width`), `.body_rows(u16)` (rows for the body slot;
/// `code_preview_lines` for `new`, `0` for the conveniences), `.patch`,
/// `.patch_part`.
///
/// ## Variants
/// `Family::DIALOG`, `DEFAULT` only; the action buttons carry their
/// `Action`'s variant.
///
/// ## States
/// None of its own; the border is painted strong (the legacy focused
/// frame); action buttons derive `DISABLED` from arming.
///
/// ## Actions
/// `Action(key)` when a button fires, `Dismissed(reason)` when the layer
/// closed without one.
///
/// ## Focus
/// The dialog registers no stop; its buttons and prompt do. Focus enters
/// the first registered control when the modal opens (rule (c)).
///
/// ## Keyboard
/// `←` / `→` move between enabled actions. Esc reaches the focused control
/// first, then the layer's `Dismiss.esc`.
///
/// ## Mouse
/// The surface is `Decorative` (a click inside is never "outside"); buttons
/// and the prompt take their own pointer intents.
///
/// ## Layout
/// Centred in `area`, `min(width, area.width - 4)` wide, rows for title,
/// description, prompt, body and actions; `draw` runs the body slot and
/// returns `Some(R)`, or `None` when nothing fits. `measure` is the
/// dialog's preferred size.
///
/// ## Parts
/// `CONTAINER`, `BORDER`, `TITLE`, `DETAIL` (the description), `BODY`,
/// `ACTIONS`, `BACKDROP` (painted by the runtime).
///
/// ## Overrides
/// `.patch`, `.patch_part`; no slots (the body is the slot).
///
/// ## Identity
/// Child ids: the prompt is `id.part(Part::FIELD)`, action `i` is
/// `id.part(Part::ACTIONS).index(i)`.
///
/// ## Testing
/// `DialogCase` with `OVERLAY | FOCUSABLE | ACTIVATES` (over a launcher
/// button); `render::components::dialog::*`.
///
/// ## Invariants
/// Arming is an `update` predicate (`Action::enabled`, or the typed token);
/// `draw` never changes it. The conveniences render through the body slot
/// path.
pub struct Dialog<'a> {
    id: Id,
    title: Option<&'a str>,
    description: Option<&'a str>,
    actions: &'a [Action<'a>],
    cancel: Option<ActionKey>,
    primary: Option<ActionKey>,
    width: Option<u16>,
    body_rows: Option<u16>,
    prompt: Option<&'a str>,
    ack: Option<&'a str>,
    ov: Overrides<'a>,
}

impl fmt::Debug for Dialog<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dialog")
            .field("id", &self.id)
            .field("title", &self.title)
            .field("actions", &self.actions.len())
            .field("cancel", &self.cancel)
            .field("width", &self.width)
            .field("prompt", &self.prompt)
            .field("ack", &self.ack.map(|_| "[token]"))
            .finish_non_exhaustive()
    }
}

impl<'a> Dialog<'a> {
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::BORDER,
        Part::TITLE,
        Part::DETAIL,
        Part::BODY,
        Part::ACTIONS,
        Part::BACKDROP,
    ];

    /// An empty dialog.
    pub const fn new(id: Id) -> Self {
        Dialog {
            id,
            title: None,
            description: None,
            actions: &[],
            cancel: None,
            primary: None,
            width: None,
            body_rows: None,
            prompt: None,
            ack: None,
            ov: Overrides::new(),
        }
    }

    /// Cancel / OK over `question`.
    pub const fn confirm(id: Id, title: &'a str, question: &'a str) -> Self {
        let mut d = Self::new(id).title(title).description(question);
        d.actions = &CONFIRM_ACTIONS;
        d.cancel = Some(ActionKey::CANCEL);
        d.primary = Some(ActionKey::CONFIRM);
        d.body_rows = Some(0);
        d
    }

    /// Cancel / Delete over `question`.
    pub const fn destructive(id: Id, title: &'a str, question: &'a str) -> Self {
        let mut d = Self::new(id).title(title).description(question);
        d.actions = &DESTRUCTIVE_ACTIONS;
        d.cancel = Some(ActionKey::CANCEL);
        d.body_rows = Some(0);
        d
    }

    /// A labelled prompt; Enter in the prompt fires the primary action.
    pub const fn prompt(id: Id, title: &'a str, label: &'a str) -> Self {
        let mut d = Self::new(id).title(title);
        d.actions = &CONFIRM_ACTIONS;
        d.cancel = Some(ActionKey::CANCEL);
        d.primary = Some(ActionKey::CONFIRM);
        d.prompt = Some(label);
        d.body_rows = Some(0);
        d
    }

    /// A typed acknowledgement: the confirming action is armed only while
    /// the draft equals `token`.
    pub const fn acknowledge(id: Id, title: &'a str, token: &'a str) -> Self {
        let mut d = Self::new(id).title(title);
        d.actions = &ACK_ACTIONS;
        d.cancel = Some(ActionKey::CANCEL);
        d.ack = Some(token);
        d.body_rows = Some(0);
        d
    }

    /// The id.
    pub const fn id(&self) -> Id {
        self.id
    }

    /// The prompt / acknowledgement control's id.
    pub const fn input_id(&self) -> Id {
        self.id.part(Part::FIELD)
    }

    /// The id of action button `i`.
    pub const fn action_id(&self, i: usize) -> Id {
        self.id.part(Part::ACTIONS).index(i)
    }

    /// The title.
    #[must_use]
    pub const fn title(mut self, s: &'a str) -> Self {
        self.title = Some(s);
        self
    }

    /// The description, wrapped under the title.
    #[must_use]
    pub const fn description(mut self, s: &'a str) -> Self {
        self.description = Some(s);
        self
    }

    /// The action row.
    #[must_use]
    pub const fn actions(mut self, a: &'a [Action<'a>]) -> Self {
        self.actions = a;
        self
    }

    /// The action Esc / dismissal stands for.
    #[must_use]
    pub const fn cancel(mut self, k: ActionKey) -> Self {
        self.cancel = Some(k);
        self
    }

    /// The dialog width (clamped to the area).
    #[must_use]
    pub const fn width(mut self, w: u16) -> Self {
        self.width = Some(w);
        self
    }

    /// Rows reserved for the body slot.
    #[must_use]
    pub const fn body_rows(mut self, n: u16) -> Self {
        self.body_rows = Some(n);
        self
    }

    /// An instance patch over every part.
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.patch(p);
        self
    }

    /// Per-part instance patches.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.patch_part(ps);
        self
    }

    const fn has_input(&self) -> bool {
        self.prompt.is_some() || self.ack.is_some()
    }

    fn armed(&self, st: &DialogState) -> bool {
        self.ack.is_none_or(|tok| st.draft.trim() == tok)
    }

    fn enabled(&self, i: usize, a: &Action<'_>, st: &DialogState) -> bool {
        let _ = i;
        a.is_enabled() && (self.armed(st) || Some(a.key()) == self.cancel)
    }

    fn variant_of(&self, a: &Action<'_>) -> Variant {
        if Some(a.key()) == self.primary {
            Variant::PRIMARY
        } else {
            a.variant()
        }
    }

    /// The first enabled action that is not the cancel action.
    fn primary_index(&self, st: &DialogState) -> Option<usize> {
        self.actions.iter().enumerate().find_map(|(i, a)| {
            (Some(a.key()) != self.cancel && self.enabled(i, a, st)).then_some(i)
        })
    }

    /// The update phase: the prompt, the action buttons, `←`/`→` and the
    /// layer's lifecycle events.
    pub fn update(&self, cx: &mut Cx<'_>, st: &mut DialogState) -> Response<DialogAction> {
        let mut acc = Acc::<DialogAction>::new();
        for it in cx.intents(self.id) {
            match it {
                Intent::Layer(LayerEvent::Dismissed(r)) => acc.action(DialogAction::Dismissed(r)),
                Intent::Layer(_) | Intent::Cancel => acc.changed(),
                _ => {}
            }
        }
        if self.has_input() {
            let r = TextInput::new(self.input_id()).update(cx, &mut st.input, &mut st.draft);
            let committed = matches!(r.action_ref(), Some(TextAction::Committed));
            acc.fold(&r.erase());
            if committed && self.prompt.is_some() {
                // Enter in a prompt submits; in an acknowledgement it only arms
                if let Some(i) = self.primary_index(st)
                    && let Some(a) = self.actions.get(i)
                {
                    acc.action(DialogAction::Action(a.key()));
                }
            }
        }
        for (i, a) in self.actions.iter().enumerate() {
            let bid = self.action_id(i);
            let enabled = self.enabled(i, a, st);
            let r = Button::new(bid, a.label())
                .variant(self.variant_of(a))
                .disabled(!enabled)
                .update(cx);
            if r.activated() {
                acc.action(DialogAction::Action(a.key()));
            } else {
                acc.fold(&r.erase());
            }
            for it in cx.intents(bid) {
                if let Intent::Key(k) = it {
                    match Binding::lookup(BINDINGS, &k) {
                        Some(DialogCmd::PrevAction) => {
                            if let Some(p) = self.neighbour(st, i, false) {
                                cx.focus(self.action_id(p));
                            }
                            acc.changed();
                        }
                        Some(DialogCmd::NextAction) => {
                            if let Some(n) = self.neighbour(st, i, true) {
                                cx.focus(self.action_id(n));
                            }
                            acc.changed();
                        }
                        None => {}
                    }
                }
            }
        }
        acc.finish(self.id)
    }

    fn neighbour(&self, st: &DialogState, from: usize, forward: bool) -> Option<usize> {
        let n = self.actions.len();
        let enabled = |i: usize| self.actions.get(i).is_some_and(|a| self.enabled(i, a, st));
        if forward {
            (from.saturating_add(1)..n).find(|&i| enabled(i))
        } else {
            (0..from).rev().find(|&i| enabled(i))
        }
    }

    /// The dialog rect inside `area` for its content height.
    fn frame_rect(&self, ui: &Ui<'_>, area: Rect) -> (Rect, u16) {
        let design = ui.design();
        let width = self
            .width
            .unwrap_or(design.size.dialog_width)
            .min(area.width.saturating_sub(4))
            .max(20)
            .min(area.width);
        let inner_w = width.saturating_sub(6);
        let desc_h = self.description.map_or(0, |d| {
            wrap(d, inner_w).len().min(usize::from(u16::MAX)) as u16
        });
        let input_h = if self.prompt.is_some() {
            design.size.field_height
        } else if self.ack.is_some() {
            design.size.field_height.saturating_add(1)
        } else {
            0
        };
        let body_h = self.body_rows.unwrap_or(design.size.code_preview_lines);
        let actions_h: u16 = if self.actions.is_empty() { 0 } else { 2 };
        // border(2) + pad(1) + title(1) + gap(1) + body + pad(1)
        let height = 2u16
            .saturating_add(1)
            .saturating_add(1)
            .saturating_add(1)
            .saturating_add(desc_h)
            .saturating_add(input_h)
            .saturating_add(body_h)
            .saturating_add(actions_h)
            .saturating_add(1)
            .min(area.height);
        let x = area.x.saturating_add(area.width.saturating_sub(width) / 2);
        let y = area
            .y
            .saturating_add(area.height.saturating_sub(height) / 2);
        (
            Rect {
                x,
                y,
                width,
                height,
            },
            desc_h,
        )
    }

    /// The draw phase: chrome, description, prompt, the body slot and the
    /// action row. Returns `None` when the dialog cannot draw.
    #[expect(
        clippy::too_many_lines,
        reason = "one pass over title, description, prompt, body and actions"
    )]
    pub fn draw<R>(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &DialogState,
        body: impl FnOnce(&mut Ui<'_>, Rect) -> R,
    ) -> Option<R> {
        if area.is_empty() {
            return None;
        }
        let (rect, desc_h) = self.frame_rect(ui, area);
        if rect.width < 4 || rect.height < 4 {
            return None;
        }
        let ov = self.ov;
        let id = self.id;
        ui.with_surface(Surface::Elevated, |ui| {
            let style = |ui: &mut Ui<'_>, part: Part, flags: StateFlags| {
                ov.style(ui, id, Family::DIALOG, Variant::DEFAULT, part, flags)
            };
            let container = style(ui, Part::CONTAINER, StateFlags::empty());
            ui.fill(rect, container.style);
            let border = style(ui, Part::BORDER, StateFlags::FOCUSED);
            let framed = ui.frame(rect, border.style);
            ui.register_decor(id, PartRef::of(Part::CONTAINER), rect);
            ui.register_decor(id, PartRef::of(Part::BORDER), rect);
            let inner = crate::layout::inset(
                framed,
                crate::layout::Insets {
                    l: 2,
                    t: 1,
                    r: 2,
                    b: 1,
                },
            );
            if inner.is_empty() {
                return None;
            }
            let mut y = inner.y;
            if let Some(t) = self.title {
                let ts = style(ui, Part::TITLE, StateFlags::empty());
                let row = Rect {
                    y,
                    height: 1,
                    ..inner
                };
                ui.paint_str(row, t, ts.style);
                ui.register_decor(id, PartRef::of(Part::TITLE), row);
                y = y.saturating_add(2);
            }
            let actions_y = if self.actions.is_empty() {
                inner.bottom()
            } else {
                inner.bottom().saturating_sub(1)
            };
            if let Some(d) = self.description {
                let ds = style(ui, Part::DETAIL, StateFlags::empty());
                for line in wrap(d, inner.width).iter().take(usize::from(desc_h)) {
                    if y >= actions_y {
                        break;
                    }
                    let row = Rect {
                        y,
                        height: 1,
                        ..inner
                    };
                    ui.paint_str(row, line, ds.style);
                    y = y.saturating_add(1);
                }
            }
            if self.has_input() {
                let field_h = ui.design().size.field_height;
                let r = Rect {
                    x: inner.x.saturating_sub(1),
                    y,
                    width: inner.width.saturating_add(1),
                    height: field_h.min(actions_y.saturating_sub(y)),
                };
                let input = TextInput::new(self.input_id()).value(&st.draft);
                let label = self.prompt.unwrap_or("Type the token to confirm");
                Field::new(label, input).plain(true).draw(ui, r, &st.input);
                y = y.saturating_add(field_h);
                if self.ack.is_some() {
                    y = y.saturating_add(1);
                }
            }
            let body_rect = Rect {
                x: inner.x,
                y: y.min(actions_y),
                width: inner.width,
                height: actions_y
                    .saturating_sub(y)
                    .saturating_sub(u16::from(!self.actions.is_empty())),
            };
            ui.register_decor(id, PartRef::of(Part::BODY), body_rect);
            let out = ui.with_area(body_rect, |ui| body(ui, body_rect));
            if !self.actions.is_empty() {
                let row = Rect {
                    x: inner.x,
                    y: actions_y,
                    width: inner.width,
                    height: 1,
                };
                ui.register_decor(id, PartRef::of(Part::ACTIONS), row);
                let mut widths = [0u16; 8];
                let n = self.actions.len().min(widths.len());
                for (i, a) in self.actions.iter().take(n).enumerate() {
                    let w = Button::new(self.action_id(i), a.label())
                        .measure(ui, Constraints::loose(row.width, 1))
                        .preferred
                        .0;
                    if let Some(slot) = widths.get_mut(i) {
                        *slot = w;
                    }
                }
                let rects = action_row(row, widths.get(..n).unwrap_or(&[]), 1, RowAlign::End);
                for ((i, a), r) in self.actions.iter().take(n).enumerate().zip(rects) {
                    Button::new(self.action_id(i), a.label())
                        .variant(self.variant_of(a))
                        .disabled(!self.enabled(i, a, st))
                        .draw(ui, r);
                }
            }
            Some(out)
        })
    }

    /// The preferred size.
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        let (rect, _) = self.frame_rect(
            ui,
            Rect {
                x: 0,
                y: 0,
                width: c.max.0,
                height: c.max.1,
            },
        );
        Size {
            min: (20, 6),
            preferred: (rect.width, rect.height),
        }
        .fit(c)
    }
}

impl Bindings for Dialog<'_> {
    type Cmd = DialogCmd;

    fn bindings(&self, _s: BindingState) -> &'static [Binding<DialogCmd>] {
        BINDINGS
    }
}
