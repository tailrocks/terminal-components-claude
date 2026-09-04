//! `Dialog` — layer content (`COMPONENT_ARCHITECTURE.md` §9.2, §14.1, §17.0
//! A7, Appendix A 4F).

use core::fmt;

use ratatui_core::layout::Rect;
use ratatui_core::style::Modifier;

use super::button::Button;
use super::field::Field;
use super::input::{TextAction, TextInput, TextInputState};
use super::keyhint::ChordText;
use super::{Acc, Overrides};
use crate::action::{Action, ActionKey};
use crate::event::{Chord, KeyCode};
use crate::id::{Id, Part, PartRef};
use crate::intent::Intent;
use crate::keymap::{Binding, BindingState, Bindings};
use crate::layer::{DismissReason, LayerEvent, LayerSize, LayerSpec};
use crate::layout::{RowAlign, action_row};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::text::{width, wrap, wrapped_rows};
use crate::theme::{DesignTokens, Family, FgStep, Role, StylePatch, Surface, Variant};
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
    /// Activate the focused action.
    Activate,
}

const BINDINGS: &[Binding<DialogCmd>] = &[
    Binding {
        action: ActionKey::custom("dialog.previous-action"),
        chord: Some(Chord::key(KeyCode::Left)),
        cmd: DialogCmd::PrevAction,
        label: "Prev",
        priority: 20,
        visible: false,
    },
    Binding {
        action: ActionKey::custom("dialog.activate.enter"),
        chord: Some(Chord::key(KeyCode::Enter)),
        cmd: DialogCmd::Activate,
        label: "Choose",
        priority: 80,
        visible: true,
    },
    Binding {
        action: ActionKey::custom("dialog.activate.space"),
        chord: Some(Chord::key(KeyCode::Char(' '))),
        cmd: DialogCmd::Activate,
        label: "Choose",
        priority: 10,
        visible: false,
    },
    Binding {
        action: ActionKey::custom("dialog.next-action"),
        chord: Some(Chord::key(KeyCode::Right)),
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

// Disabled actions retain the disabled background and gain a non-colour DIM
// signal, while using primary text so the action label remains readable on
// every built-in theme and colour capability.
const DISABLED_ACTION_PATCH: StylePatch = StylePatch::new()
    .set_fg(Role::Fg(FgStep::Primary))
    .remove(Modifier::all())
    .add(Modifier::DIM);
const INFO_ACTIONS: [Action<'static>; 1] = [Action::new(ActionKey::CLOSE, "Close")];

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
/// `code_preview_lines` for `new`, `0` for the conveniences — the dialog
/// never sees the body closure before `draw`, so the caller states it),
/// `.patch`, `.patch_part`.
///
/// ## Variants
/// `Family::DIALOG`, `DEFAULT` only; the action buttons carry their
/// `Action`'s variant.
///
/// ## States
/// None of its own; the border is painted strong (the legacy focused
/// frame); action buttons derive `DISABLED` from arming.
/// Reference fixtures target an individual prompt or action through
/// [`Ui::reference`](crate::Ui::reference) (A11).
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
/// The dialog computes a **size**, never a rect: `measured_width` /
/// `measured_height` are pure functions of the props and the design tokens,
/// [`Dialog::layer`] hands them to the one layer resolver, and
/// [`Dialog::update`] re-asserts them every frame (§26 N1). `draw` lays out
/// from `area`'s origin against that measurement — title, description,
/// prompt, a blank row, the body slot, a blank row, the action row — runs
/// the body slot exactly once and returns its value. When no chrome fits, the
/// body receives an origin-anchored empty rect under an empty clip.
/// `measure` returns the same size.
///
/// ## Parts
/// `CONTAINER`, `BORDER`, `TITLE`, `DETAIL` (the description), `BODY`,
/// `ACTIONS`, `BACKDROP` (painted by the runtime).
///
/// ## Overrides
/// `.patch`, `.patch_part`; no slots (the body is the
/// slot).
///
/// ## Identity
/// Child ids: the prompt is `id.part(Part::FIELD)`, action `i` is
/// `id.part(Part::ACTIONS).index(i)`.
///
/// ## Testing
/// `DialogCase` with `OVERLAY | TRAPS_FOCUS | FOCUSABLE | ACTIVATES` (over a
/// launcher button); `render::components::dialog::*`.
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
    /// Maximum actions represented by both update and draw.
    pub const MAX_ACTIONS: usize = 8;

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

    /// A facts body, sized for one row per property and closed by one action.
    /// Paint `props` through [`Props`](crate::components::Props) in `draw`'s
    /// body slot; this constructor stores no closed body representation.
    pub fn facts(id: Id, title: &'a str, props: &'a [(&'a str, &'a str)]) -> Self {
        let mut d = Self::info(id, title);
        d.body_rows = Some(props.len().min(usize::from(u16::MAX)) as u16);
        d
    }

    /// A choice body, sized for one row per option with Cancel / OK actions.
    /// Paint `options` through a choice control in `draw`'s body slot.
    pub fn choice(id: Id, title: &'a str, options: &'a [&'a str]) -> Self {
        let mut d = Self::confirm(id, title, "");
        d.description = None;
        d.body_rows = Some(options.len().min(usize::from(u16::MAX)) as u16);
        d
    }

    /// Read-only body content with one Close action.
    pub const fn info(id: Id, title: &'a str) -> Self {
        let mut d = Self::new(id).title(title);
        d.actions = &INFO_ACTIONS;
        d.cancel = Some(ActionKey::CLOSE);
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

    /// The action row. At most [`Self::MAX_ACTIONS`] entries participate in
    /// both update and draw; later entries are inert.
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

    fn action_count(&self) -> usize {
        self.actions.len().min(Self::MAX_ACTIONS)
    }

    fn effective_actions(&self) -> &[Action<'a>] {
        self.actions.get(..self.action_count()).unwrap_or(&[])
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
        self.effective_actions()
            .iter()
            .enumerate()
            .find_map(|(i, a)| {
                (Some(a.key()) != self.cancel && self.enabled(i, a, st)).then_some(i)
            })
    }

    /// The update phase: the prompt, the action buttons, `←`/`→` and the
    /// layer's lifecycle events.
    pub fn update(&self, cx: &mut Cx<'_>, st: &mut DialogState) -> Response<DialogAction> {
        // invariant D1: the dialog re-asserts its size every frame, so a
        // description that grows or a theme swap corrects the layer on the
        // next draw without the opener predicting anything (§26 N1).
        let size = LayerSize::Fixed(
            self.measured_width(cx.design()),
            self.measured_height(cx.design()),
        );
        cx.resize_layer(self.id, size);
        let mut acc = Acc::<DialogAction>::new();
        for it in cx.intents(self.id) {
            match it {
                Intent::Layer(LayerEvent::Dismissed(r)) => acc.action(DialogAction::Dismissed(r)),
                // A lifecycle notification is drained and repainted but NOT
                // consumed: `Opened` arrives in the same `update` as the key
                // that is still travelling the Esc ladder (§3.3 step 8), and
                // consuming it would make the first Esc after opening a modal
                // do nothing.
                Intent::Layer(_) => acc.repaint(),
                Intent::Cancel => acc.changed(),
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
        for (i, a) in self.effective_actions().iter().enumerate() {
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
                if let Intent::Binding(action) = it {
                    match Binding::command(BINDINGS, action) {
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
                        Some(DialogCmd::Activate) if enabled => {
                            acc.action(DialogAction::Action(a.key()));
                        }
                        Some(DialogCmd::Activate) => acc.consumed(),
                        None if action == a.key() && enabled => {
                            acc.action(DialogAction::Action(a.key()));
                        }
                        None => {}
                    }
                }
            }
        }
        acc.finish(self.id)
    }

    fn neighbour(&self, st: &DialogState, from: usize, forward: bool) -> Option<usize> {
        let n = self.action_count();
        let enabled = |i: usize| self.actions.get(i).is_some_and(|a| self.enabled(i, a, st));
        if forward {
            (from.saturating_add(1)..n).find(|&i| enabled(i))
        } else {
            (0..from).rev().find(|&i| enabled(i))
        }
    }

    /// Columns available to the content: the frame minus one border column
    /// and `design.space.dialog_inset` on each side (§26 N1).
    fn inner_width(&self, d: &DesignTokens) -> u16 {
        self.measured_width(d)
            .saturating_sub(2)
            .saturating_sub(d.space.dialog_inset.saturating_mul(2))
    }

    /// Rows the prompt / acknowledgement control needs, `0` when there is none.
    fn input_rows(&self, d: &DesignTokens) -> u16 {
        if self.prompt.is_some() {
            d.size.field_height
        } else if self.ack.is_some() {
            d.size.field_height.saturating_add(1)
        } else {
            0
        }
    }

    /// Rows the body slot needs, plus the blank row that separates it.
    fn body_block(&self, d: &DesignTokens) -> u16 {
        let rows = self.body_rows.unwrap_or(d.size.code_preview_lines);
        if rows == 0 { 0 } else { rows.saturating_add(1) }
    }

    /// `.width(w)` when set, else `design.size.dialog_width` (§26 N1).
    pub fn measured_width(&self, d: &DesignTokens) -> u16 {
        self.width.unwrap_or(d.size.dialog_width)
    }

    /// `border(2)` + `title(1)` + the wrapped description + the prompt +
    /// `[blank + body]` + `[blank + actions]` (§26 N1).
    ///
    /// A pure function of the props and the design tokens, and the number
    /// [`Dialog::draw`] lays out against — the two share
    /// [`wrapped_rows`](crate::text::wrapped_rows), so a description that
    /// rewraps moves both together.
    pub fn measured_height(&self, d: &DesignTokens) -> u16 {
        let inner = self.inner_width(d);
        let desc = self.description.map_or(0, |s| wrapped_rows(s, inner));
        let actions: u16 = if self.actions.is_empty() { 0 } else { 2 };
        3u16.saturating_add(desc)
            .saturating_add(self.input_rows(d))
            .saturating_add(self.body_block(d))
            .saturating_add(actions)
    }

    /// The layer this dialog wants: a modal sized from the props and the
    /// design tokens. Call it at the moment of opening —
    /// `cx.open_layer(id, dialog().layer(cx))` — and let [`Dialog::update`]
    /// re-assert it every frame (§26 N1, invariant D1).
    pub fn layer(&self, cx: &Cx<'_>) -> LayerSpec {
        let d = cx.design();
        LayerSpec::modal(self.id).size(LayerSize::Fixed(
            self.measured_width(d),
            self.measured_height(d),
        ))
    }

    /// The rect the dialog paints into: `area`'s origin, sized to what it
    /// asked the resolver for and clamped to `area`. Anchoring, flipping and
    /// centring belong to the one layer resolver (§9.1, §26 N1); no component
    /// computes a screen rect.
    fn frame_rect(&self, ui: &Ui<'_>, area: Rect) -> (Rect, u16) {
        let d = ui.design();
        let desc_h = self
            .description
            .map_or(0, |s| wrapped_rows(s, self.inner_width(d)));
        (
            Rect {
                x: area.x,
                y: area.y,
                width: self.measured_width(d).min(area.width),
                height: self.measured_height(d).min(area.height),
            },
            desc_h,
        )
    }

    /// The draw phase: chrome, description, prompt, the body slot and the
    /// action row.
    ///
    /// The body runs exactly once for every area. When the dialog or its
    /// inner rect cannot draw, it receives an empty rect anchored inside
    /// `area`; its paint and registrations remain clipped away (R1/R5).
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
    ) -> R {
        ui.with_surface(Surface::Elevated, |ui| {
            if area.is_empty() {
                let empty = Rect {
                    x: area.x,
                    y: area.y,
                    width: 0,
                    height: 0,
                };
                return ui.with_area(empty, |ui| body(ui, empty));
            }
            let (rect, desc_h) = self.frame_rect(ui, area);
            if rect.width < 4 || rect.height < 4 {
                let empty = Rect {
                    x: area.x,
                    y: area.y,
                    width: 0,
                    height: 0,
                };
                return ui.with_area(empty, |ui| body(ui, empty));
            }
            let ov = self.ov;
            let id = self.id;
            // Dialog chrome has no runtime state of its own. Its border is the
            // authored strong rule; fixture interaction flags belong to one
            // prompt/action child below, never to the composite surface.
            let live = StateFlags::empty();
            let style = |ui: &mut Ui<'_>, part: Part, flags: StateFlags| {
                ov.style(ui, id, Family::DIALOG, Variant::DEFAULT, part, flags | live)
            };
            let container = style(ui, Part::CONTAINER, StateFlags::empty());
            ui.fill(rect, container.style);
            let border = style(ui, Part::BORDER, StateFlags::FOCUSED);
            let framed = ui.frame(rect, border.style);
            ui.register_decor(id, PartRef::of(Part::CONTAINER), rect);
            ui.register_decor(id, PartRef::of(Part::BORDER), rect);
            // the horizontal inset `measured_height` wraps the description
            // against; vertically the frame is the padding (§26 N1)
            let pad = ui.design().space.dialog_inset;
            let inner = crate::layout::inset(
                framed,
                crate::layout::Insets {
                    l: pad,
                    t: 0,
                    r: pad,
                    b: 0,
                },
            );
            if inner.is_empty() {
                return ui.with_area(inner, |ui| body(ui, inner));
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
            }
            y = y.saturating_add(1);
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
            // one blank row separates the body from what precedes it, exactly
            // as `measured_height`'s `[blank + body]` term says
            let body_top = if self.body_block(ui.design()) == 0 {
                y
            } else {
                y.saturating_add(1)
            };
            let body_bottom = actions_y.saturating_sub(u16::from(!self.actions.is_empty()));
            let body_rect = Rect {
                x: inner.x,
                y: body_top.min(inner.bottom()),
                width: inner.width,
                height: body_bottom.saturating_sub(body_top),
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
                let mut widths = [0u16; Self::MAX_ACTIONS];
                let actions = self.effective_actions();
                let n = actions.len();
                for (i, a) in actions.iter().enumerate() {
                    let action_id = self.action_id(i);
                    let chord_width = ui
                        .effective_chord(action_id, a.key(), a.chord_ref())
                        .map_or(0, |chord| {
                            width(ChordText::of(chord).as_str()).saturating_add(1)
                        });
                    let w = Button::new(action_id, a.label())
                        .measure(ui, Constraints::loose(row.width, 1))
                        .preferred
                        .0
                        .saturating_add(chord_width);
                    if let Some(slot) = widths.get_mut(i) {
                        *slot = w;
                    }
                }
                let rects = action_row(row, widths.get(..n).unwrap_or(&[]), 1, RowAlign::End);
                for ((i, a), r) in actions.iter().enumerate().zip(rects) {
                    let action_id = self.action_id(i);
                    let enabled = self.enabled(i, a, st);
                    let button = Button::new(action_id, a.label())
                        .variant(self.variant_of(a))
                        .disabled(!enabled);
                    if enabled {
                        button.draw(ui, r);
                    } else {
                        button.patch(&DISABLED_ACTION_PATCH).draw(ui, r);
                    }
                    if enabled {
                        ui.publish_dynamic_bindings(
                            action_id,
                            ui.state(action_id),
                            core::iter::once((a.key(), a.chord_ref())),
                        );
                    }
                    if let Some(chord) = ui.effective_chord(action_id, a.key(), a.chord_ref()) {
                        let text = ChordText::of(chord);
                        let key_width = width(text.as_str()).min(r.width);
                        let key = Rect {
                            x: r.right().saturating_sub(key_width),
                            width: key_width,
                            ..r
                        };
                        let style = if enabled {
                            ui.resolve(
                                Family::BUTTON,
                                self.variant_of(a),
                                Part::LABEL,
                                ui.state(action_id),
                            )
                        } else {
                            ui.style_patched(
                                Family::BUTTON,
                                self.variant_of(a),
                                Part::LABEL,
                                ui.state(action_id),
                                &DISABLED_ACTION_PATCH,
                            )
                        };
                        ui.paint_str(key, text.as_str(), style.style);
                    }
                }
            }
            out
        })
    }

    /// The preferred size: exactly what [`Dialog::layer`] asks the resolver
    /// for.
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        let d = ui.design();
        Size {
            min: (20, 6),
            preferred: (self.measured_width(d), self.measured_height(d)),
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

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Position;

    use super::*;
    use crate::event::{Input, Key, KeyModifiers};
    use crate::keymap::KeyMap;
    use crate::runtime::stub::{SCREEN, Stub};
    use crate::runtime::{App, Runtime};
    use crate::theme::builder::contrast;
    use crate::theme::{ColorLevel, Theme};

    const DLG: Id = Id::root("dialog.tests");
    const BODY: Id = Id::root("dialog.tests.body");
    const TOKEN: &str = "delete";

    fn confirm() -> Dialog<'static> {
        Dialog::confirm(DLG, "Remove person", "Remove this person from the roster?")
    }

    fn prompt() -> Dialog<'static> {
        Dialog::prompt(DLG, "Rename", "New name")
    }

    fn acknowledge() -> Dialog<'static> {
        Dialog::acknowledge(DLG, "Delete table", TOKEN)
    }

    const DISABLED_ACTIONS: [Action<'static>; 2] = [
        Action::quiet(ActionKey::CANCEL, "Cancel").enabled(false),
        Action::new(ActionKey::CONFIRM, "OK").enabled(false),
    ];

    fn esc() -> Input {
        Input::Key(Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::NONE,
        })
    }

    /// A headless runtime whose `App` draws nothing, for `draw_scene`.
    fn scene() -> (Runtime<Stub>, Buffer) {
        (
            Runtime::new(Stub::default(), Theme::junie()),
            Buffer::empty(SCREEN),
        )
    }

    #[test]
    fn disabled_action_labels_keep_contrast_and_dim_at_every_level() {
        let levels = [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ];
        for base in [Theme::junie(), Theme::paper()] {
            for level in levels {
                let mut runtime = Runtime::new(Stub::default(), base.downgrade(level));
                let mut buffer = Buffer::empty(SCREEN);
                runtime.draw_scene(SCREEN, &mut buffer, |ui, _| {
                    Dialog::new(DLG)
                        .width(32)
                        .body_rows(0)
                        .actions(&DISABLED_ACTIONS)
                        .draw(
                            ui,
                            Rect::new(0, 0, 32, 5),
                            &DialogState::default(),
                            |_, _| (),
                        );
                });
                for label in ["Cancel", "OK"] {
                    let cells: Vec<_> = SCREEN
                        .positions()
                        .filter_map(|position| {
                            buffer
                                .cell(position)
                                .filter(|cell| cell.symbol() != " ")
                                .filter(|cell| label.contains(cell.symbol()))
                                .map(|cell| (position, cell))
                        })
                        .collect();
                    assert_eq!(
                        cells.len(),
                        label.chars().count(),
                        "{base:?}/{level:?}: dialog action {label:?} was not painted once"
                    );
                    for (_, cell) in cells {
                        assert!(
                            contrast(cell.fg, cell.bg) >= 3.0,
                            "{base:?}/{level:?}: dialog action {label:?} has {:?} on {:?}",
                            cell.fg,
                            cell.bg
                        );
                        assert!(
                            cell.modifier.contains(Modifier::DIM),
                            "{base:?}/{level:?}: dialog action {label:?} lost DIM"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn body_is_total_and_clipped_when_the_dialog_cannot_draw() {
        let areas = [
            Rect::new(7, 5, 0, 0),
            Rect::new(7, 5, 1, 1),
            Rect::new(7, 5, 3, 8),
            Rect::new(7, 5, 8, 3),
            // The frame fits, but the dialog inset collapses its inner rect.
            Rect::new(7, 5, 4, 4),
        ];
        for area in areas {
            let (mut rt, mut buf) = scene();
            let calls = Cell::new(0);
            let seen = Cell::new(Rect::ZERO);
            let surface = Cell::new(Surface::Canvas);
            let mut answer = 0;
            rt.draw_scene(SCREEN, &mut buf, |ui, _| {
                answer = Dialog::new(DLG).draw(ui, area, &DialogState::default(), |ui, inner| {
                    calls.set(calls.get() + 1);
                    seen.set(inner);
                    surface.set(ui.surface());
                    let style = ui.surface_style();
                    for row in SCREEN.rows() {
                        ui.paint_str(row, "ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ", style);
                    }
                    ui.register_decor(BODY, PartRef::of(Part::BODY), SCREEN);
                    42
                });
            });
            let inner = seen.get();
            assert_eq!(answer, 42, "body result became optional for {area:?}");
            assert_eq!(calls.get(), 1, "body traversal count for {area:?}");
            assert!(inner.is_empty(), "nondrawable {area:?} yielded {inner:?}");
            assert!(
                inner.x >= area.x
                    && inner.y >= area.y
                    && inner.x <= area.right()
                    && inner.y <= area.bottom(),
                "empty body {inner:?} is not anchored inside {area:?}"
            );
            assert_eq!(surface.get(), Surface::Elevated);
            assert!(
                !buf.content().iter().any(|cell| cell.symbol() == "Z"),
                "body paint escaped the empty clip for {area:?}"
            );
            assert!(
                rt.registry().regions().iter().all(|r| r.owner != BODY),
                "body registration escaped the empty clip for {area:?}"
            );
            if area.width < 4 || area.height < 4 {
                assert!(
                    rt.registry().is_empty(),
                    "nondrawable dialog registered chrome for {area:?}"
                );
            }
        }
    }

    #[test]
    fn valid_dialog_preserves_chrome_and_clips_its_body() {
        let (mut rt, mut buf) = scene();
        let area = Rect::new(4, 3, 20, 6);
        let mut inner = Rect::ZERO;
        let mut answer = 0;
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            answer = Dialog::new(DLG)
                .title("Title")
                .width(area.width)
                .body_rows(2)
                .draw(ui, area, &DialogState::default(), |ui, body| {
                    inner = body;
                    let style = ui.surface_style();
                    for row in SCREEN.rows() {
                        ui.paint_str(row, "ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ", style);
                    }
                    ui.register_decor(BODY, PartRef::of(Part::BODY), SCREEN);
                    73
                });
        });
        assert_eq!(answer, 73);
        assert!(!inner.is_empty());
        for pos in SCREEN.positions() {
            let is_z = buf.cell(pos).is_some_and(|cell| cell.symbol() == "Z");
            assert_eq!(is_z, inner.contains(pos), "body clip mismatch at {pos:?}");
        }
        let body_region = rt
            .registry()
            .regions()
            .iter()
            .find(|r| r.owner == BODY)
            .map(|r| r.area)
            .expect("body registration was lost");
        assert_eq!(body_region, inner);
        let borders = Theme::junie().design.borders;
        assert_eq!(
            buf.cell(Position::new(area.x, area.y))
                .map(ratatui_core::buffer::Cell::symbol),
            Some(borders.top_left)
        );
        assert_eq!(
            buf.cell(Position::new(
                area.right().saturating_sub(1),
                area.bottom().saturating_sub(1)
            ))
            .map(ratatui_core::buffer::Cell::symbol),
            Some(borders.bottom_right)
        );
    }

    #[test]
    fn convenience_constructors_render_through_the_body_slot() {
        let render = |dialog: &Dialog<'_>| {
            let (mut runtime, mut buffer) = scene();
            let calls = Cell::new(0usize);
            runtime.draw_scene(SCREEN, &mut buffer, |ui, area| {
                dialog.draw(ui, area, &DialogState::default(), |ui, body| {
                    calls.set(calls.get().saturating_add(1));
                    ui.paint_str(body, "body-slot", ui.surface_style());
                });
            });
            assert_eq!(calls.get(), 1);
            buffer
        };

        let mut composed = Dialog::new(DLG)
            .title("Remove person")
            .description("Remove this person from the roster?")
            .actions(&CONFIRM_ACTIONS)
            .cancel(ActionKey::CANCEL)
            .body_rows(0);
        composed.primary = Some(ActionKey::CONFIRM);
        assert_eq!(render(&confirm()), render(&composed));

        let props = [("Owner", "Junie"), ("State", "ready")];
        let options = ["Alpha", "Beta", "Gamma"];
        for dialog in [
            Dialog::facts(DLG, "Facts", &props),
            Dialog::choice(DLG, "Choose", &options),
            Dialog::info(DLG, "Info"),
        ] {
            let buffer = render(&dialog);
            assert!(buffer.content().iter().any(|cell| cell.symbol() == "b"));
        }
        assert_eq!(Dialog::facts(DLG, "Facts", &props).body_rows, Some(2));
        assert_eq!(Dialog::choice(DLG, "Choose", &options).body_rows, Some(3));
        assert_eq!(Dialog::info(DLG, "Info").effective_actions(), &INFO_ACTIONS);
    }

    #[test]
    fn action_cap_is_shared_by_update_and_draw() {
        const ACTIONS: [Action<'static>; 9] = [
            Action::new(ActionKey::custom("0"), "0"),
            Action::new(ActionKey::custom("1"), "1"),
            Action::new(ActionKey::custom("2"), "2"),
            Action::new(ActionKey::custom("3"), "3"),
            Action::new(ActionKey::custom("4"), "4"),
            Action::new(ActionKey::custom("5"), "5"),
            Action::new(ActionKey::custom("6"), "6"),
            Action::new(ActionKey::custom("7"), "7"),
            Action::new(ActionKey::custom("8"), "8"),
        ];
        let dialog = Dialog::new(DLG).actions(&ACTIONS);
        assert_eq!(dialog.effective_actions(), &ACTIONS[..Dialog::MAX_ACTIONS]);

        let (mut runtime, mut buffer) = scene();
        runtime.draw_scene(SCREEN, &mut buffer, |ui, area| {
            dialog.draw(ui, area, &DialogState::default(), |_, _| {});
        });
        for i in 0..Dialog::MAX_ACTIONS {
            assert!(runtime.area_of(dialog.action_id(i)).is_some());
        }
        assert!(
            runtime
                .area_of(dialog.action_id(Dialog::MAX_ACTIONS))
                .is_none()
        );
    }

    /// §26 N1: the layer size is a function of `(props, DesignTokens)` and
    /// of nothing else — no focus, no frame counter, no measured content.
    /// The `prompt` and `acknowledge` cases are what the `input_rows` term
    /// added (§28 P4): without it the resolver is asked for a layer shorter
    /// than the dialog's own content and `draw` clamps the field away.
    #[test]
    fn layer_size_is_a_pure_function_of_props_and_design_tokens() {
        for theme in [Theme::junie(), Theme::paper()] {
            let d = &theme.design;
            for dlg in [confirm(), prompt(), acknowledge()] {
                let once = (dlg.measured_width(d), dlg.measured_height(d));
                let twice = (dlg.measured_width(d), dlg.measured_height(d));
                assert_eq!(once, twice, "the same props must measure the same");
                assert!(once.0 > 0 && once.1 > 0);
            }
            // the input term, isolated: `prompt` costs one field, an
            // acknowledgement costs one field plus its echo row
            let bare = Dialog::new(DLG).title("Rename").body_rows(0);
            let bare_h = bare
                .measured_height(d)
                .saturating_add(2 /* the action row prompt() carries */);
            assert_eq!(
                prompt().measured_height(d),
                bare_h.saturating_add(d.size.field_height),
                "prompt owes exactly `field_height` rows"
            );
            assert_eq!(
                acknowledge().measured_height(d),
                bare_h.saturating_add(d.size.field_height).saturating_add(1),
                "an acknowledgement owes the echo row too"
            );
            assert_eq!(prompt().input_rows(d), d.size.field_height);
            assert_eq!(
                acknowledge().input_rows(d),
                d.size.field_height.saturating_add(1)
            );
            assert_eq!(confirm().input_rows(d), 0);
        }
        // two themes, two answers, each read off that theme's own tokens
        for theme in [Theme::junie(), Theme::paper()] {
            let d = &theme.design;
            assert_eq!(prompt().measured_width(d), d.size.dialog_width);
            assert_eq!(
                prompt().measured_height(d),
                3u16.saturating_add(d.size.field_height).saturating_add(2),
                "title + border, the prompt's field, the action block"
            );
        }
    }

    #[test]
    fn draw_lays_out_against_the_height_it_asked_for() {
        let theme = Theme::junie();
        let d = Dialog::new(DLG)
            .title("Title")
            .description("A description that wraps predictably.")
            .width(30)
            .body_rows(3)
            .actions(&CONFIRM_ACTIONS);
        let asked = Rect::new(
            7,
            0,
            d.measured_width(&theme.design),
            d.measured_height(&theme.design),
        );
        let (mut rt, _) = scene();
        let mut buf = Buffer::empty(SCREEN);
        let mut body = Rect::ZERO;
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            d.draw(ui, asked, &DialogState::default(), |_, area| body = area);
        });
        assert_eq!(body.height, 3);
        assert!(asked.contains(Position::new(body.x, body.y)));
        let borders = theme.design.borders;
        assert_eq!(
            buf.cell(Position::new(asked.x, asked.y))
                .map(ratatui_core::buffer::Cell::symbol),
            Some(borders.top_left)
        );
    }

    #[test]
    fn confirm_is_centred_by_the_resolver_not_by_the_dialog() {
        #[derive(Default)]
        struct Centered {
            st: DialogState,
            opened: bool,
        }
        impl App for Centered {
            fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
                if !self.opened {
                    self.opened = true;
                    cx.open_layer(DLG, confirm().layer(cx));
                }
                confirm().update(cx, &mut self.st).erase()
            }
            fn draw(&self, ui: &mut Ui<'_>) {
                ui.layer(DLG, |ui, area| {
                    confirm().draw(ui, area, &self.st, |_, _| {});
                });
            }
        }
        let mut rt = Runtime::new(Centered::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_buffer(SCREEN, &mut buf);
        let _ = rt.handle(Input::Tick);
        rt.draw_buffer(SCREEN, &mut buf);
        let area = rt
            .layer_area(DLG)
            .expect("resolver assigned the dialog area");
        let spec = rt.open_spec(DLG).expect("dialog owns an open spec");
        assert_eq!(
            area,
            crate::layer::resolve_anchor(SCREEN, spec.anchor, spec.size)
        );
    }

    #[test]
    fn a_growing_body_resizes_the_layer_on_the_next_frame() {
        #[derive(Default)]
        struct Growing {
            st: DialogState,
            opened: bool,
            rows: u16,
        }
        impl App for Growing {
            fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
                let d = Dialog::new(DLG).title("Growing").body_rows(self.rows);
                if !self.opened {
                    self.opened = true;
                    cx.open_layer(DLG, d.layer(cx));
                }
                d.update(cx, &mut self.st).erase()
            }
            fn draw(&self, ui: &mut Ui<'_>) {
                ui.layer(DLG, |ui, area| {
                    Dialog::new(DLG).title("Growing").body_rows(self.rows).draw(
                        ui,
                        area,
                        &self.st,
                        |_, _| {},
                    );
                });
            }
        }
        let mut rt = Runtime::new(Growing::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_buffer(SCREEN, &mut buf);
        let _ = rt.handle(Input::Tick);
        rt.draw_buffer(SCREEN, &mut buf);
        let before = rt.layer_area(DLG).expect("open layer");
        rt.app_mut().rows = 5;
        let _ = rt.handle(Input::Tick);
        rt.draw_buffer(SCREEN, &mut buf);
        let after = rt.layer_area(DLG).expect("resized layer");
        assert_eq!(after.height, before.height.saturating_add(6));
    }

    #[test]
    fn action_arming_is_evaluated_in_update() {
        let action = Action::danger(ActionKey::CONFIRM, "Delete");
        let actions = [action];
        let d = Dialog::acknowledge(DLG, "Delete", TOKEN).actions(&actions);
        let mut st = DialogState::default();
        assert!(!d.enabled(0, &action, &st));
        st.draft.push_str(TOKEN);
        assert!(d.enabled(0, &action, &st));
    }

    /// §28 P4: the prompt's `Field` gets exactly the rows `input_rows`
    /// charged for it, so a prompt is never silently squeezed out of the
    /// layer its own `measured_height` asked for.
    #[test]
    fn a_prompt_dialog_sizes_its_own_field_row() {
        let (mut rt, _) = scene();
        let st = DialogState::default();
        let d = prompt();
        let dt = &Theme::junie().design;
        // exactly the layer the dialog asks the resolver for
        let area = Rect::new(0, 0, d.measured_width(dt), d.measured_height(dt));
        let mut buf = Buffer::empty(area);
        rt.draw_scene(area, &mut buf, |ui, a| {
            prompt().draw(ui, a, &st, |_, _| {});
        });
        // the chrome's own region, not the editor's: `Field` registers its
        // block as `Decorative` under the control's id, and the control then
        // registers its one-row editor under the same `(id, CONTAINER)` key
        let field = rt
            .registry()
            .regions()
            .iter()
            .find(|r| {
                r.owner == d.input_id()
                    && r.part == PartRef::of(Part::CONTAINER)
                    && matches!(r.kind, crate::hit::RegionKind::Decorative)
            })
            .map(|r| r.area)
            .expect("the prompt's Field registers its chrome");
        assert_eq!(
            field.height, dt.size.field_height,
            "the drawn field row is not the row `input_rows` charged for"
        );
        // the arithmetic half: `measured_height` minus every other term is
        // exactly `input_rows`
        let desc = d
            .description
            .map_or(0, |s| wrapped_rows(s, d.inner_width(dt)));
        let actions: u16 = if d.actions.is_empty() { 0 } else { 2 };
        let rest = 3u16
            .saturating_add(desc)
            .saturating_add(d.body_block(dt))
            .saturating_add(actions);
        assert_eq!(d.measured_height(dt).saturating_sub(rest), d.input_rows(dt));
    }

    /// §13 / §28 P3: a component that owns a layer runs its `update`
    /// **unconditionally**. Esc dismisses the layer at §3.3 step 8 and then
    /// re-runs `update`; by then `cx.is_open` is false, so the gated shape
    /// would drain neither the `Cancel` nor the `Layer(Dismissed)` the
    /// dismissal addressed to the dialog.
    #[test]
    fn an_unconditional_update_receives_the_dismissal() {
        #[derive(Default)]
        struct DialogApp {
            st: DialogState,
            opened: bool,
            dismissed: Vec<DismissReason>,
        }

        impl App for DialogApp {
            fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
                if !self.opened {
                    self.opened = true;
                    cx.open_layer(DLG, confirm().layer(cx));
                }
                let r = confirm().update(cx, &mut self.st);
                if let Some(DialogAction::Dismissed(reason)) = r.action_ref() {
                    self.dismissed.push(*reason);
                }
                r.erase()
            }

            fn draw(&self, ui: &mut Ui<'_>) {
                ui.layer(DLG, |ui, a| {
                    confirm().draw(ui, a, &self.st, |_, _| {});
                });
            }
        }

        let mut rt = Runtime::new(DialogApp::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_buffer(SCREEN, &mut buf);
        let _ = rt.handle(Input::Tick);
        rt.draw_buffer(SCREEN, &mut buf);
        assert!(rt.is_open(DLG), "the dialog sized and opened its own layer");
        let _ = rt.handle(esc());
        rt.draw_buffer(SCREEN, &mut buf);
        assert!(!rt.is_open(DLG));
        assert_eq!(rt.app().dismissed, vec![DismissReason::Esc]);
        assert!(
            rt.diagnostics().is_empty(),
            "the unconditional shape leaves nothing undelivered: {:?}",
            rt.diagnostics()
        );
    }

    #[test]
    fn action_chord_routes_remaps_removes_and_paints_effective_binding() {
        const QUICK: ActionKey = ActionKey::custom("dialog.quick");
        const ACTIONS: [Action<'static>; 1] =
            [Action::new(QUICK, "Quick").chord(Chord::key(KeyCode::F(4)))];

        fn dialog() -> Dialog<'static> {
            Dialog::new(DLG)
                .title("Dynamic")
                .actions(&ACTIONS)
                .body_rows(0)
        }

        #[derive(Default)]
        struct DynamicDialogApp {
            state: DialogState,
            opened: bool,
            chosen: Option<ActionKey>,
            keymap: KeyMap,
        }

        impl App for DynamicDialogApp {
            fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
                if !self.opened {
                    self.opened = true;
                    cx.open_layer(DLG, dialog().layer(cx));
                }
                let response = dialog().update(cx, &mut self.state);
                if let Some(DialogAction::Action(action)) = response.action_ref() {
                    self.chosen = Some(*action);
                }
                response.erase()
            }

            fn draw(&self, ui: &mut Ui<'_>) {
                let _ = ui.layer(DLG, |ui, area| {
                    dialog().draw(ui, area, &self.state, |_, _| {});
                });
            }

            fn keymap(&self) -> &KeyMap {
                &self.keymap
            }
        }

        let mut runtime = Runtime::new(DynamicDialogApp::default(), Theme::junie());
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer);
        let _ = runtime.handle(Input::Tick);
        runtime.draw_buffer(SCREEN, &mut buffer);
        runtime.draw_buffer(SCREEN, &mut buffer);
        let key = |code| {
            Input::Key(Key {
                code,
                mods: KeyModifiers::NONE,
            })
        };
        let _ = runtime.handle(key(KeyCode::F(4)));
        assert_eq!(runtime.app().chosen, Some(QUICK));

        runtime.app_mut().chosen = None;
        runtime.app_mut().keymap.remap_component(
            dialog().action_id(0),
            QUICK,
            Chord::key(KeyCode::F(5)),
        );
        let _ = runtime.handle(key(KeyCode::F(4)));
        assert_eq!(runtime.app().chosen, None);
        let _ = runtime.handle(key(KeyCode::F(5)));
        assert_eq!(runtime.app().chosen, Some(QUICK));
        runtime.draw_buffer(SCREEN, &mut buffer);
        let painted = buffer
            .content()
            .iter()
            .map(ratatui_core::buffer::Cell::symbol)
            .collect::<String>();
        assert!(painted.contains("F5"));

        runtime.app_mut().chosen = None;
        runtime
            .app_mut()
            .keymap
            .remove_component(dialog().action_id(0), QUICK);
        let _ = runtime.handle(key(KeyCode::F(5)));
        assert_eq!(runtime.app().chosen, None);
    }

    /// A reference rendering is a picture, not a control at any depth. The
    /// dialog registers no region of its own, its action
    /// buttons register nothing, its prompt registers no editor, and the
    /// focus ring stays empty.
    #[test]
    fn a_reference_dialog_registers_no_control() {
        for dlg in [confirm(), prompt()] {
            let (mut rt, mut buf) = scene();
            let st = DialogState::default();
            let id = dlg.id();
            let action0 = dlg.action_id(0);
            let input = dlg.input_id();
            rt.draw_scene(SCREEN, &mut buf, |ui, a| {
                ui.reference(None, |ui| dlg.draw(ui, a, &st, |_, _| {}));
            });
            assert!(rt.registry().area_of(id).is_none(), "the chrome is live");
            assert!(
                rt.registry().area_of(action0).is_none(),
                "an action button is live"
            );
            assert!(
                !rt.registry().delivers_to(input),
                "the prompt control is live"
            );
            assert_eq!(rt.ring().reachable().count(), 0, "a focus stop survived");
        }
    }

    #[test]
    fn reference_dialog_targets_one_owned_control_without_broadcasting() {
        let render = |dialog: Dialog<'_>, target| {
            let (mut runtime, mut buffer) = scene();
            runtime.set_theme(Theme::junie().downgrade(crate::ColorLevel::Mono));
            runtime.draw_scene(SCREEN, &mut buffer, |ui, area| {
                ui.reference(target, |ui| {
                    dialog.draw(ui, area, &DialogState::default(), |_, _| {});
                });
            });
            buffer
                .content()
                .iter()
                .map(ratatui_core::buffer::Cell::symbol)
                .collect::<String>()
        };

        let pressed = render(
            confirm(),
            Some(crate::ReferenceTarget::new(
                confirm().action_id(0),
                crate::ReferenceState::PRESSED | crate::ReferenceState::FOCUSED,
            )),
        );
        assert_eq!(pressed.matches("[Cancel]").count(), 1);
        assert!(!pressed.contains("[OK]"));
        let default = render(confirm(), None);
        assert_eq!(render(confirm(), None), default);

        let prompt_focused = render(
            prompt(),
            Some(crate::ReferenceTarget::new(
                prompt().input_id(),
                crate::ReferenceState::FOCUSED | crate::ReferenceState::FOCUS_VISIBLE,
            )),
        );
        assert_eq!(prompt_focused.matches('▎').count(), 1);
        assert!(!prompt_focused.contains("▎Cancel"));
        assert!(!prompt_focused.contains("▎OK"));
    }
}
