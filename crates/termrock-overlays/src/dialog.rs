use core::fmt;

use ratatui_core::layout::Rect;

use super::{Acc, PartStyle, overlay_chrome};
use crate::action::{Action, ActionKey};
use crate::event::{Chord, KeyCode};
use crate::id::{Id, Part, PartRef};
use crate::intent::Intent;
use crate::keymap::{Binding, BindingState, Bindings};
use crate::layer::{
    Anchor, DismissReason, LayerEvent, LayerOpening, LayerSize, LayerSpec, ScreenAlign, UpdateToken,
};
use crate::layout::RowAlign;
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::secret::{Secret, SecretPolicy};
use crate::text::{width, wrap, wrapped_rows};
use crate::theme::{DesignTokens, Family, FgStep, Modifier, Role, StylePatch, Surface, Variant};
use crate::ui::{Cx, FrameRead, Ui};
use termrock_controls::button::Button;
use termrock_feedback::keyhint::ChordText;
use termrock_fields::field::Field;
use termrock_fields::input::{TextAction, TextInput, TextInputState, redacted_text};
use termrock_runtime::author::NestedLayerSlotAuthor;
use termrock_runtime::focus::Focusability;
use termrock_runtime::ui::LayerSlot;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DialogAction {
    Action(ActionKey),

    Dismissed(DismissReason),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DialogCmd {
    PrevAction,

    NextAction,

    Activate,
}

const PREVIOUS_ACTION_BINDING: Binding<DialogCmd> = Binding {
    action: ActionKey::custom("dialog.previous-action"),
    chord: Some(Chord::key(KeyCode::Left)),
    cmd: DialogCmd::PrevAction,
    label: "Prev",
    priority: 20,
    visible: false,
};
const NEXT_ACTION_BINDING: Binding<DialogCmd> = Binding {
    action: ActionKey::custom("dialog.next-action"),
    chord: Some(Chord::key(KeyCode::Right)),
    cmd: DialogCmd::NextAction,
    label: "Next",
    priority: 20,
    visible: false,
};
const NAVIGATION_BINDINGS: &[Binding<DialogCmd>] = &[PREVIOUS_ACTION_BINDING, NEXT_ACTION_BINDING];
const BINDINGS: &[Binding<DialogCmd>] = &[
    PREVIOUS_ACTION_BINDING,
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
    NEXT_ACTION_BINDING,
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
const INFO_ACTIONS: [Action<'static>; 1] = [Action::new(ActionKey::CLOSE, "Close")];

#[derive(Default)]
pub struct DialogState {
    input: TextInputState,
    draft: String,
    ack_draft: Secret,
    action_close: Option<(LayerOpening, UpdateToken)>,
}

impl Clone for DialogState {
    fn clone(&self) -> Self {
        DialogState {
            input: self.input.clone(),
            draft: if self.input.is_sensitive() {
                redacted_text(&self.draft)
            } else {
                self.draft.clone()
            },
            ack_draft: Secret::new(redacted_text(self.ack_draft.expose())),
            action_close: self.action_close,
        }
    }
}

impl PartialEq for DialogState {
    fn eq(&self, other: &Self) -> bool {
        self.input == other.input
            && if self.input.is_sensitive() || other.input.is_sensitive() {
                true
            } else {
                self.draft == other.draft
            }
            && self.action_close == other.action_close
    }
}

impl Eq for DialogState {}

impl fmt::Debug for DialogState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DialogState")
            .field("input", &self.input)
            .field("draft", &"[redacted]")
            .field("ack_draft", &"[redacted]")
            .field("action_close", &self.action_close)
            .finish()
    }
}

impl DialogState {
    pub const fn is_editing(&self) -> bool {
        self.input.is_editing()
    }

    pub fn draft(&self) -> &str {
        if self.input.is_sensitive() {
            ""
        } else {
            &self.draft
        }
    }

    pub fn zeroize(&mut self) {
        self.input.zeroize();
        zeroize_string(&mut self.draft);
        self.ack_draft.zeroize();
        self.action_close = None;
    }

    fn set_secret_mode(&mut self, secret: bool) {
        if self.input.is_sensitive() == secret {
            return;
        }
        if secret {
            zeroize_string(&mut self.draft);
        } else {
            self.ack_draft.zeroize();
        }
        self.input.set_sensitive(secret);
    }
}

fn zeroize_string(value: &mut String) {
    crate::secret::wipe_string(core::mem::take(value));
}

fn mnemonic_char(used: &mut u32, label: &str) -> Option<(u16, char)> {
    let mut fallback = None;
    for (index, ch) in label.chars().enumerate() {
        if !ch.is_ascii_alphanumeric() {
            continue;
        }
        let bit = u32::from(ch.to_ascii_lowercase() as u8).saturating_sub(u32::from(b'a'));
        if bit >= 26 {
            continue;
        }
        let mask = 1u32 << bit;
        if fallback.is_none() {
            fallback = Some((index, ch));
        }
        if *used & mask == 0 {
            *used |= mask;
            return Some((u16::try_from(index).unwrap_or(0), ch));
        }
    }
    fallback.map(|(index, ch)| (u16::try_from(index).unwrap_or(0), ch))
}

#[derive(Clone, Copy, Debug)]
struct ActionLine {
    start: usize,
    end: usize,
}

#[derive(Debug)]
struct DialogLayout {
    height: u16,
    content: Rect,
    title: Option<Rect>,
    description: Rect,
    input: Rect,
    body: Rect,
    actions: Vec<Rect>,
}

fn action_lines(widths: &[u16], available: u16, spacing: u16) -> Vec<ActionLine> {
    let mut lines = Vec::new();
    if widths.is_empty() || available == 0 {
        return lines;
    }
    let mut start = 0;
    let mut used = 0u16;
    for (index, &natural) in widths.iter().enumerate() {
        let width = natural.min(available).max(1);
        let gap = if index == start { 0 } else { spacing };
        if index > start && used.saturating_add(gap).saturating_add(width) > available {
            lines.push(ActionLine { start, end: index });
            start = index;
            used = width;
        } else {
            used = used.saturating_add(gap).saturating_add(width);
        }
    }
    lines.push(ActionLine {
        start,
        end: widths.len(),
    });
    lines
}

fn action_rects(area: Rect, widths: &[u16], lines: &[ActionLine], spacing: u16) -> Vec<Rect> {
    let mut rects = vec![Rect::ZERO; widths.len()];
    for (line_index, line) in lines.iter().enumerate() {
        let Some(row_y) = area
            .y
            .checked_add(u16::try_from(line_index).unwrap_or(u16::MAX))
        else {
            continue;
        };
        if row_y >= area.bottom() {
            continue;
        }
        let Some(line_widths) = widths.get(line.start..line.end) else {
            continue;
        };
        let clamped: Vec<u16> = line_widths
            .iter()
            .map(|width| (*width).min(area.width))
            .collect();
        let row = Rect {
            y: row_y,
            height: 1,
            ..area
        };
        for (offset, rect) in crate::layout::action_row(row, &clamped, spacing, RowAlign::End)
            .into_iter()
            .enumerate()
        {
            if let Some(slot) = rects.get_mut(line.start.saturating_add(offset)) {
                *slot = rect;
            }
        }
    }
    rects
}

pub struct Dialog<'a> {
    id: Id,
    title: Option<&'a str>,
    meta: Option<&'a str>,
    head_in_border: bool,
    description: Option<&'a str>,
    actions: &'a [Action<'a>],
    cancel: Option<ActionKey>,
    primary: Option<ActionKey>,
    width: Option<u16>,
    body_rows: Option<u16>,
    align: Option<ScreenAlign>,
    prompt: Option<&'a str>,
    input_label: Option<&'a str>,
    ack: Option<&'a str>,
    error: Option<&'a str>,
    ov: PartStyle<'a>,
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
    pub const MAX_ACTIONS: usize = 8;

    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::BORDER,
        Part::TITLE,
        Part::DETAIL,
        Part::BODY,
        Part::ACTIONS,
        Part::BACKDROP,
    ];

    pub const fn new(id: Id) -> Self {
        Dialog {
            id,
            title: None,
            meta: None,
            head_in_border: false,
            description: None,
            actions: &[],
            cancel: None,
            primary: None,
            width: None,
            body_rows: None,
            align: None,
            prompt: None,
            input_label: None,
            ack: None,
            error: None,
            ov: PartStyle::new(),
        }
    }

    pub const fn confirm(id: Id, title: &'a str, question: &'a str) -> Self {
        let mut d = Self::new(id).title(title).description(question);
        d.actions = &CONFIRM_ACTIONS;
        d.cancel = Some(ActionKey::CANCEL);
        d.primary = Some(ActionKey::CONFIRM);
        d.body_rows = Some(0);
        d
    }

    pub const fn destructive(id: Id, title: &'a str, question: &'a str) -> Self {
        let mut d = Self::new(id).title(title).description(question);
        d.actions = &DESTRUCTIVE_ACTIONS;
        d.cancel = Some(ActionKey::CANCEL);
        d.body_rows = Some(0);
        d
    }

    pub const fn prompt(id: Id, title: &'a str, label: &'a str) -> Self {
        let mut d = Self::new(id).title(title);
        d.actions = &CONFIRM_ACTIONS;
        d.cancel = Some(ActionKey::CANCEL);
        d.primary = Some(ActionKey::CONFIRM);
        d.prompt = Some(label);
        d.body_rows = Some(0);
        d
    }

    #[must_use]
    pub const fn input_label(mut self, label: &'a str) -> Self {
        self.input_label = Some(label);
        self
    }

    pub const fn acknowledge(id: Id, title: &'a str, token: &'a str) -> Self {
        let mut d = Self::new(id).title(title);
        d.actions = &ACK_ACTIONS;
        d.cancel = Some(ActionKey::CANCEL);
        d.ack = Some(token);
        d.body_rows = Some(0);
        d
    }

    pub fn facts(id: Id, title: &'a str, props: &'a [(&'a str, &'a str)]) -> Self {
        let mut d = Self::info(id, title);
        d.body_rows = Some(props.len().min(usize::from(u16::MAX)) as u16);
        d
    }

    pub fn choice(id: Id, title: &'a str, options: &'a [&'a str]) -> Self {
        let mut d = Self::confirm(id, title, "");
        d.description = None;
        d.body_rows = Some(options.len().min(usize::from(u16::MAX)) as u16);
        d
    }

    pub const fn info(id: Id, title: &'a str) -> Self {
        let mut d = Self::new(id).title(title);
        d.actions = &INFO_ACTIONS;
        d.cancel = Some(ActionKey::CLOSE);
        d
    }

    pub const fn id(&self) -> Id {
        self.id
    }

    pub const fn input_id(&self) -> Id {
        self.id.part(Part::FIELD)
    }

    pub const fn action_id(&self, i: usize) -> Id {
        self.id.part(Part::ACTIONS).index(i)
    }

    #[must_use]
    pub const fn title(mut self, s: &'a str) -> Self {
        self.title = Some(s);
        self
    }

    /// Right-aligned border caption shown in the frame head next to the close
    /// mark (for example `read-only`).
    #[must_use]
    pub const fn meta(mut self, s: &'a str) -> Self {
        self.meta = Some(s);
        self
    }

    /// Draws the title inside the top border row instead of the body, with the
    /// body starting directly under the border (read-only card heads).
    #[must_use]
    pub const fn head_in_border(mut self) -> Self {
        self.head_in_border = true;
        self
    }

    #[must_use]
    pub const fn description(mut self, s: &'a str) -> Self {
        self.description = Some(s);
        self
    }

    #[must_use]
    pub const fn error(mut self, error: Option<&'a str>) -> Self {
        self.error = error;
        self
    }

    #[must_use]
    pub const fn actions(mut self, a: &'a [Action<'a>]) -> Self {
        self.actions = a;
        self
    }

    #[must_use]
    pub const fn cancel(mut self, k: ActionKey) -> Self {
        self.cancel = Some(k);
        self
    }

    #[must_use]
    pub const fn primary(mut self, k: ActionKey) -> Self {
        self.primary = Some(k);
        self
    }

    #[must_use]
    pub const fn width(mut self, w: u16) -> Self {
        self.width = Some(w);
        self
    }

    #[must_use]
    pub const fn body_rows(mut self, n: u16) -> Self {
        self.body_rows = Some(n);
        self
    }

    /// Overrides the default modal screen alignment for this dialog.
    #[must_use]
    pub const fn align(mut self, a: ScreenAlign) -> Self {
        self.align = Some(a);
        self
    }

    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    const fn has_input(&self) -> bool {
        self.prompt.is_some() || self.ack.is_some()
    }

    fn input_control(&self) -> TextInput<'static> {
        let input = TextInput::new(self.input_id());
        if self.ack.is_some() {
            input.secret(SecretPolicy::default())
        } else {
            input
        }
    }

    fn armed(&self, st: &DialogState) -> bool {
        self.ack
            .is_none_or(|tok| st.input.visible_text_equals(st.ack_draft.expose(), tok))
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

    fn primary_index(&self, st: &DialogState) -> Option<usize> {
        self.effective_actions()
            .iter()
            .enumerate()
            .find_map(|(i, a)| {
                (Some(a.key()) != self.cancel && self.enabled(i, a, st)).then_some(i)
            })
    }

    fn resolve_dismissal(&self, reason: DismissReason, st: &mut DialogState) -> DialogAction {
        st.zeroize();
        self.cancel
            .filter(|key| {
                self.effective_actions()
                    .iter()
                    .any(|action| action.key() == *key)
            })
            .map_or(DialogAction::Dismissed(reason), DialogAction::Action)
    }

    pub fn update(&self, cx: &mut Cx<'_>, st: &mut DialogState) -> Response<DialogAction> {
        let layer_open = cx.is_open(self.id);
        let action_close = st.action_close.take();
        let d = cx.design();
        let action_widths = self.action_widths(cx);
        let viewport_width = cx.viewport().width;
        let width = if viewport_width == 0 {
            self.measured_width(d)
        } else {
            self.measured_width(d).min(viewport_width)
        };
        let size = LayerSize::Fixed(
            width,
            self.measured_height_for_width(d, width, &action_widths),
        );
        cx.resize_layer(self.id, size);
        let mut acc = Acc::<DialogAction>::new();
        let mut action_fired = false;
        let mut dismissal = None;
        let mut cancelled = false;
        let mut dismissed_after_action = false;
        for it in cx.intents(self.id) {
            match it {
                Intent::Layer(LayerEvent::Dismissed(r)) => {
                    let receipt = cx.layer_event_receipt(self.id);
                    if r == DismissReason::Programmatic
                        && action_close.is_some_and(|action_close| {
                            receipt.is_some_and(|receipt| {
                                receipt.update().map(|update| (receipt.opening(), update))
                                    == Some(action_close)
                            })
                        })
                    {
                        dismissed_after_action = true;
                    } else {
                        dismissal.get_or_insert(r);
                    }
                }

                Intent::Layer(_) => {
                    let _ = cx.layer_event_receipt(self.id);
                    acc.repaint();
                }
                Intent::Cancel => cancelled = true,
                _ => {}
            }
        }
        let dismissal = dismissal.or_else(|| cancelled.then_some(DismissReason::Esc));
        if let Some(reason) = dismissal {
            acc.action(self.resolve_dismissal(reason, st));
        }
        let suppress_activation = dismissal.is_some() || dismissed_after_action || !layer_open;
        let is_ack = self.ack.is_some();
        st.set_secret_mode(is_ack);
        if self.has_input() {
            let r = if is_ack {
                self.input_control()
                    .update_in_form(cx, &mut st.input, &mut st.ack_draft, false)
            } else {
                self.input_control()
                    .update(cx, &mut st.input, &mut st.draft)
            };
            let committed = matches!(r.action_ref(), Some(TextAction::Committed));
            if !suppress_activation {
                acc.fold(&r.erase());
            }
            if !suppress_activation && committed && self.prompt.is_some() {
                if let Some(i) = self.primary_index(st)
                    && let Some(a) = self.actions.get(i)
                {
                    acc.action(DialogAction::Action(a.key()));
                    action_fired = true;
                }
            } else if !suppress_activation
                && committed
                && is_ack
                && let Some(id) =
                    self.effective_actions()
                        .iter()
                        .enumerate()
                        .find_map(|(i, action)| {
                            self.enabled(i, action, st).then_some(self.action_id(i))
                        })
            {
                cx.focus(id);
            }
        }
        for (i, a) in self.effective_actions().iter().enumerate() {
            let bid = self.action_id(i);
            let enabled = self.enabled(i, a, st);
            let r = Button::new(bid, a.label())
                .variant(self.variant_of(a))
                .disabled(!enabled)
                .update(cx);
            if !suppress_activation {
                if r.activated() {
                    acc.action(DialogAction::Action(a.key()));
                    action_fired = true;
                } else {
                    acc.fold(&r.erase());
                }
            }
            for it in cx.intents(bid) {
                if suppress_activation {
                    continue;
                }
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
                            action_fired = true;
                        }
                        Some(DialogCmd::Activate) => acc.consumed(),
                        None if action == a.key() && enabled => {
                            acc.action(DialogAction::Action(a.key()));
                            action_fired = true;
                        }
                        None => {}
                    }
                }
            }
        }
        if action_fired && self.ack.is_some() {
            st.zeroize();
        }
        if action_fired {
            st.action_close = cx
                .layer_opening(self.id)
                .map(|opening| (opening, cx.update_token()));
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

    fn paint_close(&self, ui: &mut Ui<'_>, outer: Rect) {
        if outer.width < 8 || outer.height < 3 {
            return;
        }
        let x = outer.right().saturating_sub(5);
        let y = outer.y;
        let border = ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::BorderStrong)
                .set_bg(Role::Surface(Surface::Elevated)),
        );
        let mark = ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Secondary))
                .set_bg(Role::Surface(Surface::Elevated)),
        );
        ui.paint_str(
            Rect {
                x,
                y,
                width: 1,
                height: 1,
            },
            " ",
            border,
        );
        ui.paint_str(
            Rect {
                x: x.saturating_add(1),
                y,
                width: 1,
                height: 1,
            },
            "×",
            mark,
        );
        ui.paint_str(
            Rect {
                x: x.saturating_add(2),
                y,
                width: 1,
                height: 1,
            },
            " ",
            border,
        );
    }

    fn input_rows(&self, d: &DesignTokens) -> u16 {
        if self.prompt.is_some() {
            d.size.field_height
        } else if self.ack.is_some() {
            d.size.field_height.saturating_add(1)
        } else {
            0
        }
    }

    fn body_block(&self, d: &DesignTokens) -> u16 {
        self.body_rows.unwrap_or(d.size.code_preview_lines)
    }

    fn action_widths_with(
        &self,
        mut effective_chord: impl FnMut(Id, ActionKey, Option<Chord>) -> Option<Chord>,
    ) -> Vec<u16> {
        self.effective_actions()
            .iter()
            .enumerate()
            .map(|(index, action)| {
                let chord =
                    effective_chord(self.action_id(index), action.key(), action.chord_ref())
                        .map_or(0, |chord| {
                            width(ChordText::of(chord).as_str()).saturating_add(1)
                        });
                width(action.label())
                    .saturating_add(2)
                    .saturating_add(chord)
            })
            .collect()
    }

    fn action_widths(&self, cx: &Cx<'_>) -> Vec<u16> {
        self.action_widths_with(|owner, action, default| cx.effective_chord(owner, action, default))
    }

    fn action_widths_ui(&self, ui: &Ui<'_>) -> Vec<u16> {
        self.action_widths_with(|owner, action, default| ui.effective_chord(owner, action, default))
    }

    fn layout(&self, outer: Rect, d: &DesignTokens, action_widths: &[u16]) -> DialogLayout {
        // A border head carries the title in the top border row, so the body
        // starts directly under the border and the card keeps no inner title
        // row or gap rows.
        let border_head = self.head_in_border;
        let vertical_pad = if border_head {
            0
        } else {
            u16::from(outer.height >= 8)
        };
        let content = crate::layout::inset(
            outer,
            crate::layout::Insets {
                // A border head keeps the focus gutter column inside the body,
                // so the left inset is one cell narrower.
                l: if border_head { 2 } else { 3 },
                t: 1 + vertical_pad,
                r: 3,
                b: 1 + vertical_pad,
            },
        );
        let lines = action_lines(action_widths, content.width, 2);
        let requested_action_height = u16::try_from(lines.len().max(1)).unwrap_or(u16::MAX);
        let title_rows = u16::from(self.title.is_some() && !border_head);
        let title_gap = title_rows.min(vertical_pad);
        let action_height = requested_action_height.min(
            content
                .height
                .saturating_sub(title_rows)
                .saturating_sub(title_gap),
        );
        let action_gap =
            u16::from(!border_head && action_height > 0 && content.height > action_height);
        let body_block = self.body_block(d);
        let natural_description_rows = self
            .description
            .map_or(0, |text| wrapped_rows(text, content.width));
        let height = 2u16
            .saturating_add(vertical_pad.saturating_mul(2))
            .saturating_add(title_rows)
            .saturating_add(title_gap)
            .saturating_add(natural_description_rows)
            .saturating_add(self.input_rows(d))
            .saturating_add(body_block)
            .saturating_add(requested_action_height)
            .saturating_add(u16::from(!border_head));
        let mut y = content.y;
        let mut remaining = content.height;
        let title = if border_head {
            None
        } else {
            self.title.map(|_| {
                let rect = Rect {
                    y,
                    height: u16::from(remaining > 0),
                    ..content
                };
                y = y.saturating_add(rect.height);
                remaining = remaining.saturating_sub(rect.height);
                rect
            })
        };
        y = y.saturating_add(title_gap);
        remaining = remaining.saturating_sub(title_gap);
        let fixed_after_description = self
            .input_rows(d)
            .saturating_add(action_height)
            .saturating_add(action_gap)
            .saturating_add(body_block);
        let desc_capacity = remaining.saturating_sub(fixed_after_description);
        let description_rows = self.description.map_or(0, |text| {
            wrapped_rows(text, content.width).min(desc_capacity)
        });
        let description = Rect {
            x: content.x,
            y,
            width: content.width,
            height: description_rows,
        };
        y = y.saturating_add(description_rows);
        remaining = remaining.saturating_sub(description_rows);

        let input_rows = self.input_rows(d).min(
            remaining
                .saturating_sub(action_height)
                .saturating_sub(action_gap),
        );
        let input = Rect {
            x: content.x.saturating_sub(1),
            y,
            width: content.width.saturating_add(1),
            height: input_rows,
        };
        y = y.saturating_add(input_rows);

        let action_y = content.bottom().saturating_sub(action_height);
        let action_area = Rect {
            x: content.x,
            y: action_y,
            width: content.width,
            height: action_height,
        };
        let actions = action_rects(action_area, action_widths, &lines, 2);
        let body_top = y;
        let body_bottom = action_y.saturating_sub(action_gap);
        let body_height = body_bottom.saturating_sub(body_top).min(body_block);
        let body = Rect {
            x: content.x,
            y: body_top,
            width: content.width,
            height: body_height,
        };

        DialogLayout {
            height,
            content,
            title,
            description,
            input,
            body,
            actions,
        }
    }

    pub fn measured_width(&self, d: &DesignTokens) -> u16 {
        self.width.unwrap_or(d.size.dialog_width)
    }

    pub fn measured_height(&self, d: &DesignTokens) -> u16 {
        let action_widths = self.action_widths_with(|_, _, default| default);
        self.measured_height_for_width(d, self.measured_width(d), &action_widths)
    }

    fn measured_height_for_width(
        &self,
        d: &DesignTokens,
        width: u16,
        action_widths: &[u16],
    ) -> u16 {
        self.layout(
            Rect {
                x: 0,
                y: 0,
                width,
                height: u16::MAX,
            },
            d,
            action_widths,
        )
        .height
    }

    pub fn layer(&self, cx: &Cx<'_>) -> LayerSpec {
        let d = cx.design();
        let action_widths = self.action_widths(cx);
        let viewport_width = cx.viewport().width;
        let width = if viewport_width == 0 {
            self.measured_width(d)
        } else {
            self.measured_width(d).min(viewport_width)
        };
        let spec = LayerSpec::modal(self.id).size(LayerSize::Fixed(
            width,
            self.measured_height_for_width(d, width, &action_widths),
        ));
        match self.align {
            Some(a) => spec.anchor(Anchor::Screen(a)),
            None => spec,
        }
    }

    fn frame_rect(&self, ui: &Ui<'_>, area: Rect, action_widths: &[u16]) -> Rect {
        let d = ui.design();
        Rect {
            x: area.x,
            y: area.y,
            width: self.measured_width(d).min(area.width),
            height: self
                .measured_height_for_width(d, self.measured_width(d).min(area.width), action_widths)
                .min(area.height),
        }
    }

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
        let action_widths = self.action_widths_ui(ui);
        let rect = self.frame_rect(ui, area, &action_widths);
        let chrome = if rect.width < 4 || rect.height < 4 {
            Rect {
                x: area.x,
                y: area.y,
                width: 0,
                height: 0,
            }
        } else {
            rect
        };
        let ov = self.ov;
        let id = self.id;

        let live = StateFlags::empty();
        let painted = overlay_chrome(
            ui,
            id,
            chrome,
            Family::DIALOG,
            Surface::Elevated,
            ov,
            live,
            StateFlags::FOCUSED,
            |ui, framed| {
                if framed.is_empty() {
                    return body(ui, framed);
                }
                let layout = self.layout(chrome, ui.design(), &action_widths);
                let style = |ui: &mut Ui<'_>, part: Part, flags: StateFlags| {
                    ov.style(ui, id, Family::DIALOG, Variant::DEFAULT, part, flags | live)
                };
                if let Some(t) = self.title
                    && let Some(title) = layout.title
                    && !title.is_empty()
                {
                    let ts = style(ui, Part::TITLE, StateFlags::empty());
                    ui.paint_str(title, t, ts.style);
                    ui.register_decor(id, PartRef::of(Part::TITLE), title);
                }
                if let Some(d) = self.description
                    && !layout.description.is_empty()
                {
                    let ds = style(ui, Part::DETAIL, StateFlags::empty());
                    for (offset, line) in wrap(d, layout.content.width)
                        .iter()
                        .take(usize::from(layout.description.height))
                        .enumerate()
                    {
                        let row = Rect {
                            y: layout
                                .description
                                .y
                                .saturating_add(u16::try_from(offset).unwrap_or(u16::MAX)),
                            height: 1,
                            ..layout.description
                        };
                        ui.paint_str(row, line, ds.style);
                    }
                }
                if self.has_input() && !layout.input.is_empty() {
                    let value = if self.ack.is_some() {
                        st.ack_draft.expose()
                    } else {
                        &st.draft
                    };
                    let input = self.input_control().value(value);
                    let label = self
                        .input_label
                        .or(self.prompt)
                        .unwrap_or("Type the token to confirm");
                    Field::new(label, input).plain(true).error(self.error).draw(
                        ui,
                        layout.input,
                        &st.input,
                    );
                }
                ui.register_decor(id, PartRef::of(Part::BODY), layout.body);
                if self.head_in_border && !layout.body.is_empty() {
                    ui.register_control(id.part(Part::BODY), layout.body, Focusability::Focusable);
                }
                let out = ui.with_area(layout.body, |ui| body(ui, layout.body));
                if !self.actions.is_empty() {
                    let actions_y = layout
                        .actions
                        .iter()
                        .filter(|rect| !rect.is_empty())
                        .map(|rect| rect.y)
                        .min()
                        .unwrap_or(layout.content.bottom());
                    let actions_bottom = layout
                        .actions
                        .iter()
                        .filter(|rect| !rect.is_empty())
                        .map(|rect| rect.bottom())
                        .max()
                        .unwrap_or(actions_y);
                    let row = Rect {
                        x: layout.content.x,
                        y: actions_y,
                        width: layout.content.width,
                        height: actions_bottom.saturating_sub(actions_y),
                    };
                    ui.register_decor(id, PartRef::of(Part::ACTIONS), row);
                    let actions = self.effective_actions();
                    let mut used_mnemonic = 0u32;
                    for (i, (a, r)) in actions
                        .iter()
                        .zip(layout.actions.iter().copied())
                        .enumerate()
                    {
                        if r.is_empty() {
                            continue;
                        }
                        let action_id = self.action_id(i);
                        let enabled = self.enabled(i, a, st);
                        Button::new(action_id, a.label())
                            .variant(self.variant_of(a))
                            .disabled(!enabled)
                            .draw(ui, r);
                        if self.head_in_border {
                            // The frozen plain button keeps its leading pad in
                            // the container ink, not the label ink.
                            let pad = ui.paint_patch(
                                &StylePatch::new()
                                    .set_fg(Role::Surface(Surface::Overlay))
                                    .set_bg(Role::Surface(Surface::Overlay)),
                            );
                            ui.paint_str(
                                Rect {
                                    x: r.x,
                                    y: r.y,
                                    width: 1,
                                    height: 1,
                                },
                                " ",
                                pad,
                            );
                        }
                        // A border-head facts card draws its plain close
                        // action without a mnemonic mark.
                        let mnemonics = !self.head_in_border;
                        if mnemonics
                            && let Some((offset, ch)) = mnemonic_char(&mut used_mnemonic, a.label())
                        {
                            let mut patch = StylePatch::new()
                                .set_bg(Role::Surface(Surface::Overlay))
                                .add(Modifier::UNDERLINED);
                            if a.key() == ActionKey::CANCEL {
                                patch = patch.set_fg(Role::Fg(FgStep::Primary)).add(Modifier::BOLD);
                            } else {
                                patch = patch.set_fg(Role::Custom(
                                    ratatui_core::style::Color::Rgb(0xff, 0x6b, 0x6b),
                                ));
                            }
                            let mark = ui.paint_patch(&patch);
                            ui.paint_str(
                                Rect {
                                    x: r.x.saturating_add(1).saturating_add(offset),
                                    y: r.y,
                                    width: 1,
                                    height: 1,
                                },
                                &ch.to_string(),
                                mark,
                            );
                        }
                        if enabled {
                            ui.publish_dynamic_bindings(
                                action_id,
                                ui.state(action_id),
                                NAVIGATION_BINDINGS
                                    .iter()
                                    .map(|binding| (binding.action, binding.chord))
                                    .chain(core::iter::once((a.key(), a.chord_ref()))),
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
                            let style = ui.resolve(
                                Family::BUTTON,
                                self.variant_of(a),
                                Part::LABEL,
                                ui.state(action_id),
                            );
                            ui.paint_str(key, text.as_str(), style.style);
                        }
                    }
                }
                out
            },
        );
        // Border head and meta paint after the chrome: they live in the
        // border row, outside the body clip.
        if self.head_in_border
            && let Some(t) = self.title
        {
            let ts = ui.paint_patch(
                &StylePatch::new()
                    .set_fg(Role::Fg(FgStep::Primary))
                    .set_bg(Role::Surface(Surface::Elevated))
                    .add(Modifier::BOLD),
            );
            let text = format!(" {t} ");
            let cols = u16::try_from(text.chars().count()).unwrap_or(0);
            let head = Rect {
                x: chrome.x.saturating_add(2),
                y: chrome.y,
                width: cols,
                height: 1,
            };
            ui.paint_str(head, &text, ts);
            ui.register_decor(id, PartRef::of(Part::TITLE), head);
        }
        if let Some(m) = self.meta {
            // The frozen border caption is a faint ink, dimmer than the
            // DETAIL body style.
            let ms = ui.paint_patch(
                &StylePatch::new()
                    .set_fg(Role::Fg(FgStep::Faint))
                    .set_bg(Role::Surface(Surface::Elevated)),
            );
            let text = format!(" {m} ");
            let cols = u16::try_from(text.chars().count()).unwrap_or(0);
            let meta_rect = Rect {
                x: chrome.right().saturating_sub(6).saturating_sub(cols),
                y: chrome.y,
                width: cols,
                height: 1,
            };
            ui.paint_str(meta_rect, &text, ms);
            ui.register_decor(id, PartRef::of(Part::DETAIL), meta_rect);
        }
        self.paint_close(ui, chrome);
        painted
    }

    /// Draw this dialog through its existing body layout and expose the body
    /// as a typed component slot.
    ///
    /// The body closure runs once, including when the resolved body has zero
    /// area. Dialog chrome, focus registration, clipping, and hit regions
    /// remain owned by [`Dialog::draw`].
    pub fn draw_body_slot(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &DialogState,
        body: impl FnOnce(&mut LayerSlot<'_, '_>),
    ) {
        self.draw(ui, area, st, |ui, body_area| {
            ui.with_nested_layer_slot(body_area, body);
        });
    }

    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        let d = ui.design();
        let width = self.measured_width(d).min(c.max.0);
        Size {
            min: (20, 6),
            preferred: (
                width,
                self.measured_height_for_width(d, width, &self.action_widths_ui(ui)),
            ),
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
    use crate::layer::Dismiss;
    use crate::runtime::stub::{SCREEN, Stub};
    use crate::runtime::{App, Runtime};
    use crate::theme::Theme;
    use termrock_test_support::Harness;

    const DLG: Id = Id::root("dialog.tests");
    const PARENT: Id = Id::root("dialog.tests.parent");
    const BODY: Id = Id::root("dialog.tests.body");
    const TOKEN: &str = "delete";
    const REMAPPED_ACTIONS: [Action<'static>; 2] = [
        Action::new(ActionKey::custom("go"), "Go"),
        Action::new(ActionKey::custom("do"), "Do"),
    ];
    const RACE_CANCEL: ActionKey = ActionKey::custom("dialog.tests.race-cancel");
    const RACE_ACTIONS: [Action<'static>; 2] = [
        Action::new(RACE_CANCEL, "Stop"),
        Action::new(ActionKey::CONFIRM, "Go"),
    ];

    fn remapped_dialog() -> Dialog<'static> {
        Dialog::new(DLG)
            .title("Title")
            .description("Keep")
            .width(20)
            .body_rows(0)
            .actions(&REMAPPED_ACTIONS)
    }

    fn confirm() -> Dialog<'static> {
        Dialog::confirm(DLG, "Remove person", "Remove this person from the roster?")
    }

    fn prompt() -> Dialog<'static> {
        Dialog::prompt(DLG, "Rename", "New name")
    }

    fn acknowledge() -> Dialog<'static> {
        Dialog::acknowledge(DLG, "Delete table", TOKEN)
    }

    fn esc() -> Input {
        Input::Key(Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::NONE,
        })
    }

    fn scene() -> (Runtime<Stub>, Buffer) {
        (
            Runtime::new(Stub::default(), Theme::junie()),
            Buffer::empty(SCREEN),
        )
    }

    #[test]
    fn border_head_paints_the_padded_title_in_the_top_border_row() {
        let (mut rt, mut buf) = scene();
        drop(rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            Dialog::info(DLG, "Debug info")
                .width(38)
                .body_rows(8)
                .head_in_border()
                .draw(
                    ui,
                    Rect::new(0, 0, 38, 11),
                    &DialogState::default(),
                    |_, _| {},
                );
        }));
        let row: String = (0..16)
            .map(|x| buf[(x, 0)].symbol().chars().next().unwrap())
            .collect();
        assert!(row.starts_with("\u{256d}\u{2500} Debug info"), "{row}");
        // The body starts directly under the border: row 1 holds the first
        // body row, not an inner title row.
        assert_eq!(buf[(4, 1)].symbol(), " ");
    }

    #[test]
    fn border_head_height_is_border_plus_body_plus_actions() {
        let measured = Cell::new(0u16);
        let (mut rt, mut buf) = scene();
        drop(rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            let dialog = Dialog::info(DLG, "Debug info")
                .width(66)
                .body_rows(8)
                .head_in_border();
            let action_widths = dialog.action_widths_with(|_, _, default| default);
            measured.set(dialog.measured_height_for_width(ui.design(), 66, &action_widths));
        }));
        assert_eq!(measured.get(), 11, "border 2 + body 8 + close action 1");
    }

    #[test]
    fn meta_paints_the_faint_caption_before_the_close_mark() {
        let (mut rt, mut buf) = scene();
        drop(rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            Dialog::info(DLG, "Debug info")
                .width(38)
                .body_rows(8)
                .meta("read-only")
                .head_in_border()
                .draw(
                    ui,
                    Rect::new(0, 0, 38, 11),
                    &DialogState::default(),
                    |_, _| {},
                );
        }));
        let row: String = (18..34)
            .map(|x| buf[(x, 0)].symbol().chars().next().unwrap())
            .collect();
        assert!(row.contains(" read-only "), "{row}");
        // The close mark keeps its cell right of the caption.
        assert_eq!(buf[(34, 0)].symbol(), "\u{d7}");
    }

    #[test]
    fn border_head_close_action_drops_the_mnemonic_mark() {
        let (mut rt, mut buf) = scene();
        drop(rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            Dialog::info(DLG, "Debug info")
                .width(38)
                .body_rows(8)
                .head_in_border()
                .draw(
                    ui,
                    Rect::new(0, 0, 38, 11),
                    &DialogState::default(),
                    |_, _| {},
                );
        }));
        // The action row is the last inner row; its label is plain, without a
        // red underlined first letter.
        for x in 4..36 {
            let cell = buf[(x, 10)].clone();
            assert_ne!(
                cell.fg,
                ratatui_core::style::Color::from_u32(0x00ff_6b6b),
                "x={x}"
            );
        }
    }

    #[test]
    fn body_is_total_and_clipped_when_the_dialog_cannot_draw() {
        let areas = [
            Rect::new(7, 5, 0, 0),
            Rect::new(7, 5, 1, 1),
            Rect::new(7, 5, 3, 8),
            Rect::new(7, 5, 8, 3),
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
            })
            .commit_presented();
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
        })
        .commit_presented();
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
            runtime
                .draw_scene(SCREEN, &mut buffer, |ui, area| {
                    dialog.draw(ui, area, &DialogState::default(), |ui, body| {
                        calls.set(calls.get().saturating_add(1));
                        ui.paint_str(body, "body-slot", ui.surface_style());
                    });
                })
                .commit_presented();
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
        runtime
            .draw_scene(SCREEN, &mut buffer, |ui, area| {
                dialog.draw(ui, area, &DialogState::default(), |_, _| {});
            })
            .commit_presented();
        for i in 0..Dialog::MAX_ACTIONS {
            assert!(runtime.area_of(dialog.action_id(i)).is_some());
        }
        assert!(
            runtime
                .area_of(dialog.action_id(Dialog::MAX_ACTIONS))
                .is_none()
        );
    }

    #[test]
    fn action_lines_keep_long_unicode_labels_inside_narrow_frames() {
        const ACTIONS: [Action<'static>; 3] = [
            Action::new(ActionKey::custom("first"), "確認する"),
            Action::new(ActionKey::custom("second"), "保存"),
            Action::new(ActionKey::custom("third"), "キャンセル"),
        ];
        let dialog = Dialog::new(DLG).width(18).body_rows(0).actions(&ACTIONS);
        let d = &Theme::junie().design;
        let height = dialog.measured_height(d);
        assert!(height >= 5, "wrapped actions need one row per action");

        let area = Rect::new(2, 1, dialog.measured_width(d), height);
        let (mut runtime, mut buffer) = scene();
        runtime
            .draw_scene(area, &mut buffer, |ui, _| {
                dialog.draw(ui, area, &DialogState::default(), |_, _| {});
            })
            .commit_presented();
        for index in 0..ACTIONS.len() {
            let button = runtime
                .area_of(dialog.action_id(index))
                .expect("every wrapped action must register");
            assert!(
                button.x >= area.x
                    && button.y >= area.y
                    && button.right() <= area.right()
                    && button.bottom() <= area.bottom(),
                "action escaped dialog: {button:?}"
            );
        }
    }

    #[test]
    fn actions_reflow_to_a_narrow_live_viewport_before_the_layer_is_measured() {
        const ACTIONS: [Action<'static>; 8] = [
            Action::new(ActionKey::custom("a"), "First"),
            Action::new(ActionKey::custom("b"), "Second"),
            Action::new(ActionKey::custom("c"), "Third"),
            Action::new(ActionKey::custom("d"), "Fourth"),
            Action::new(ActionKey::custom("e"), "Fifth"),
            Action::new(ActionKey::custom("f"), "Sixth"),
            Action::new(ActionKey::custom("g"), "Seventh"),
            Action::new(ActionKey::custom("h"), "Eighth"),
        ];
        let dialog = Dialog::new(DLG)
            .title("Title")
            .width(60)
            .body_rows(0)
            .actions(&ACTIONS);
        let viewport = Rect::new(0, 0, 20, 40);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(viewport);
        runtime
            .draw_scene(viewport, &mut buffer, |ui, _| {
                dialog.draw(ui, viewport, &DialogState::default(), |_, _| {});
            })
            .commit_presented();
        for index in 0..ACTIONS.len() {
            let button = runtime
                .area_of(dialog.action_id(index))
                .expect("each action must receive a visible row after width reflow");
            assert!(viewport.contains(Position::new(button.x, button.y)));
            assert!(button.right() <= viewport.right());
            assert!(button.bottom() <= viewport.bottom());
        }
    }

    #[test]
    fn narrow_viewport_keeps_the_action_cap_and_omits_only_excess_actions() {
        const ACTIONS: [Action<'static>; 9] = [
            Action::new(ActionKey::custom("a"), "First"),
            Action::new(ActionKey::custom("b"), "Second"),
            Action::new(ActionKey::custom("c"), "Third"),
            Action::new(ActionKey::custom("d"), "Fourth"),
            Action::new(ActionKey::custom("e"), "Fifth"),
            Action::new(ActionKey::custom("f"), "Sixth"),
            Action::new(ActionKey::custom("g"), "Seventh"),
            Action::new(ActionKey::custom("h"), "Eighth"),
            Action::new(ActionKey::custom("i"), "Ninth"),
        ];
        let dialog = Dialog::new(DLG)
            .title("Title")
            .width(60)
            .body_rows(0)
            .actions(&ACTIONS);
        let viewport = Rect::new(0, 0, 20, 40);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(viewport);
        runtime
            .draw_scene(viewport, &mut buffer, |ui, _| {
                dialog.draw(ui, viewport, &DialogState::default(), |_, _| {});
            })
            .commit_presented();
        for index in 0..Dialog::MAX_ACTIONS {
            assert!(runtime.area_of(dialog.action_id(index)).is_some());
        }
        assert!(
            runtime
                .area_of(dialog.action_id(Dialog::MAX_ACTIONS))
                .is_none()
        );
    }

    #[test]
    fn title_survives_when_a_short_view_cannot_fit_all_action_rows() {
        const ACTIONS: [Action<'static>; 2] = [
            Action::new(ActionKey::custom("a"), "First"),
            Action::new(ActionKey::custom("b"), "Second"),
        ];
        let dialog = Dialog::new(DLG)
            .title("TITLE")
            .width(16)
            .body_rows(0)
            .actions(&ACTIONS);
        let viewport = Rect::new(0, 0, 16, 4);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(viewport);
        runtime
            .draw_scene(viewport, &mut buffer, |ui, _| {
                dialog.draw(ui, viewport, &DialogState::default(), |_, _| {});
            })
            .commit_presented();
        let rendered: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(rendered.contains("TITLE"), "title overwritten: {rendered}");
        let title = runtime
            .area_of_part(DLG, PartRef::of(Part::TITLE))
            .expect("the visible title owns its measured bounds");
        let mut visible_actions = 0;
        for index in 0..ACTIONS.len() {
            if let Some(button) = runtime.area_of(dialog.action_id(index)) {
                visible_actions += 1;
                assert!(
                    button.y >= title.bottom(),
                    "overlap: {title:?} / {button:?}"
                );
            }
        }
        assert_eq!(visible_actions, 1, "one action row fits below the title");
    }

    #[test]
    fn title_only_measured_height_contains_the_drawn_title() {
        let dialog = Dialog::new(DLG).title("TITLE").width(30).body_rows(0);
        let viewport = Rect::new(0, 0, 40, dialog.measured_height(&Theme::junie().design));
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(viewport);
        runtime
            .draw_scene(viewport, &mut buffer, |ui, _| {
                dialog.draw(ui, viewport, &DialogState::default(), |_, _| {});
            })
            .commit_presented();
        let rendered: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            rendered.contains("TITLE"),
            "measured height {} hid title: {rendered}",
            dialog.measured_height(&Theme::junie().design)
        );
    }

    #[test]
    fn remapped_shortcuts_are_shared_by_measure_layer_and_draw() {
        struct ShortcutApp {
            keymap: KeyMap,
            state: DialogState,
            opened: bool,
            measured_height: Cell<u16>,
        }

        impl App for ShortcutApp {
            fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
                if !self.opened {
                    self.opened = true;
                    cx.open_layer(DLG, remapped_dialog().layer(cx));
                }
                remapped_dialog().update(cx, &mut self.state).erase()
            }

            fn draw(&self, ui: &mut Ui<'_>) {
                ui.layer(DLG, |ui, area| {
                    let size = remapped_dialog()
                        .measure(ui, Constraints::loose(ui.full().width, ui.full().height))
                        .preferred;
                    self.measured_height.set(size.1);
                    remapped_dialog().draw(
                        ui,
                        Rect {
                            height: size.1,
                            ..area
                        },
                        &self.state,
                        |_, _| {},
                    );
                });
            }

            fn keymap(&self) -> &KeyMap {
                &self.keymap
            }
        }

        let dialog = remapped_dialog();
        let mut keymap = KeyMap::default();
        keymap.remap_component(
            dialog.action_id(0),
            REMAPPED_ACTIONS[0].key(),
            Chord::key(KeyCode::F(1)),
        );
        keymap.remap_component(
            dialog.action_id(1),
            REMAPPED_ACTIONS[1].key(),
            Chord::key(KeyCode::F(12)),
        );
        let app = ShortcutApp {
            keymap,
            state: DialogState::default(),
            opened: false,
            measured_height: Cell::new(0),
        };
        let mut runtime = Runtime::new(app, Theme::junie());
        let viewport = Rect::new(0, 0, 20, 30);
        let mut buffer = Buffer::empty(viewport);
        let _ = runtime.initialize();
        runtime
            .draw_buffer(viewport, &mut buffer)
            .commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
        runtime
            .draw_buffer(viewport, &mut buffer)
            .commit_presented();
        let measured = runtime.app().measured_height.get();
        let rendered: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            rendered.contains("Keep"),
            "description clipped at measured height {measured}: {rendered}"
        );
        let container = runtime
            .area_of_part(DLG, PartRef::of(Part::CONTAINER))
            .expect("measured dialog container was drawn");
        assert_eq!(container.height, measured);
        assert_eq!(
            runtime.open_spec(DLG).expect("dialog layer").size,
            LayerSize::Fixed(20, measured)
        );
    }

    #[test]
    fn disabled_action_shortcuts_still_participate_in_shared_measurement() {
        struct ShortcutApp {
            keymap: KeyMap,
        }

        impl App for ShortcutApp {
            fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
                Response::ignored()
            }

            fn draw(&self, _ui: &mut Ui<'_>) {}

            fn keymap(&self) -> &KeyMap {
                &self.keymap
            }
        }

        const ACTIONS: [Action<'static>; 2] = [
            Action::new(ActionKey::custom("go"), "Go"),
            Action::new(ActionKey::custom("disabled"), "Do").enabled(false),
        ];
        let dialog = Dialog::new(DLG)
            .title("Title")
            .width(20)
            .body_rows(0)
            .actions(&ACTIONS);
        let mut keymap = KeyMap::default();
        keymap.remap_component(
            dialog.action_id(0),
            ACTIONS[0].key(),
            Chord::key(KeyCode::F(1)),
        );
        keymap.remap_component(
            dialog.action_id(1),
            ACTIONS[1].key(),
            Chord::key(KeyCode::F(12)),
        );
        let mut runtime = Runtime::new(ShortcutApp { keymap }, Theme::junie());
        let viewport = Rect::new(0, 0, 20, 30);
        let mut buffer = Buffer::empty(viewport);
        let mut measured = 0;
        runtime
            .draw_scene(viewport, &mut buffer, |ui, area| {
                measured = dialog
                    .measure(ui, Constraints::loose(viewport.width, viewport.height))
                    .preferred
                    .1;
                dialog.draw(
                    ui,
                    Rect {
                        height: measured,
                        ..area
                    },
                    &DialogState::default(),
                    |_, _| {},
                );
            })
            .commit_presented();
        assert!(runtime.area_of(dialog.action_id(0)).is_some());
        assert!(runtime.area_of(dialog.action_id(1)).is_some());
        let container = runtime
            .area_of_part(DLG, PartRef::of(Part::CONTAINER))
            .expect("measured dialog container was drawn");
        assert_eq!(container.height, measured);
    }

    #[test]
    fn zero_width_dialog_keeps_body_callback_total() {
        let (mut runtime, mut buffer) = scene();
        let calls = Cell::new(0);
        runtime
            .draw_scene(SCREEN, &mut buffer, |ui, _| {
                Dialog::new(DLG).actions(&CONFIRM_ACTIONS).draw(
                    ui,
                    Rect::new(7, 5, 0, 0),
                    &DialogState::default(),
                    |_, body| {
                        calls.set(calls.get() + 1);
                        assert!(body.is_empty());
                    },
                );
            })
            .commit_presented();
        assert_eq!(calls.get(), 1);
        assert!(runtime.registry().is_empty());
    }

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

            let bare = Dialog::new(DLG).title("Rename").body_rows(0);
            let bare_h = bare.measured_height(d);
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

        for theme in [Theme::junie(), Theme::paper()] {
            let d = &theme.design;
            assert_eq!(prompt().measured_width(d), d.size.dialog_width);
            assert_eq!(
                prompt().measured_height(d),
                5u16.saturating_add(2)
                    .saturating_add(d.size.field_height)
                    .saturating_add(1),
                "frozen Dialog::height charges chrome_rows (5 + 2*pad.y), field rows, and action rows"
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
        })
        .commit_presented();
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
        let _ = rt.initialize();
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut rt, Input::Tick);
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
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
        let _ = rt.initialize();
        let screen = Rect::new(0, 0, 40, 24);
        let mut buf = Buffer::empty(screen);
        rt.draw_buffer(screen, &mut buf).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut rt, Input::Tick);
        rt.draw_buffer(screen, &mut buf).commit_presented();
        let before = rt.layer_area(DLG).expect("open layer");
        rt.app_mut().rows = 5;
        let _ = crate::runtime::stub::deliver(&mut rt, Input::Tick);
        rt.draw_buffer(screen, &mut buf).commit_presented();
        let after = rt.layer_area(DLG).expect("resized layer");
        // Frozen Dialog::height adds visible body rows directly; the old
        // candidate-only extra row was not part of the reference measure.
        assert_eq!(after.height, before.height.saturating_add(5));
    }

    #[test]
    fn action_arming_is_evaluated_in_update() {
        let action = Action::danger(ActionKey::CONFIRM, "Delete");
        let actions = [action];
        let d = Dialog::acknowledge(DLG, "Delete", TOKEN).actions(&actions);
        let mut st = DialogState::default();
        assert!(!d.enabled(0, &action, &st));
        st.ack_draft.set(TOKEN);
        st.set_secret_mode(true);
        assert!(
            st.draft().is_empty(),
            "acknowledgement state exposed its token"
        );
        let copy = st.clone();
        assert_eq!(copy.ack_draft.expose(), "••••••");
        assert!(!copy.ack_draft.expose().contains(TOKEN));
        assert!(!d.enabled(0, &action, &copy));
        assert!(d.enabled(0, &action, &st));
    }

    #[test]
    fn acknowledgement_requires_exact_current_draft_not_stale_commit() {
        let d = acknowledge();
        for draft in ["delet", "DELETE", " delete", "delete ", "deletex"] {
            let mut st = DialogState::default();
            st.set_secret_mode(true);
            st.ack_draft.set(TOKEN);
            st.input.begin(draft);
            assert!(!d.armed(&st), "a differing live draft armed confirmation");
        }
        let mut st = DialogState::default();
        st.set_secret_mode(true);
        st.input.begin(TOKEN);
        assert!(d.armed(&st), "exact live draft needs no preliminary commit");
        assert!(!d.armed(&st.clone()), "redacted clone is not authorization");
        for draft in [" delete", "delete "] {
            let mut st = DialogState::default();
            st.set_secret_mode(true);
            st.ack_draft.set(draft);
            assert!(!d.armed(&st), "committed whitespace must not be trimmed");
        }
    }

    #[test]
    fn acknowledgement_frame_masks_confirmation_token() {
        let (mut rt, mut buf) = scene();
        let mut st = DialogState::default();
        st.ack_draft.set(TOKEN);
        st.set_secret_mode(true);
        rt.draw_scene(SCREEN, &mut buf, |ui, area| {
            acknowledge().draw(ui, area, &st, |_, _| {});
        })
        .commit_presented();
        let frame: String = buf
            .content()
            .iter()
            .map(ratatui_core::buffer::Cell::symbol)
            .collect();
        assert!(
            !frame.contains(TOKEN),
            "acknowledgement token reached the frame"
        );
        let policy = SecretPolicy::default();
        let mask = Theme::junie().design.glyphs.get(policy.mask);
        assert!(
            frame.matches(mask).count()
                >= TOKEN.chars().count().saturating_sub(policy.synthetic_tail),
            "acknowledgement field did not paint its mask: {frame}"
        );
    }

    #[test]
    fn a_prompt_dialog_sizes_its_own_field_row() {
        let (mut rt, _) = scene();
        let st = DialogState::default();
        let d = prompt();
        let dt = &Theme::junie().design;

        let area = Rect::new(0, 0, d.measured_width(dt), d.measured_height(dt));
        let mut buf = Buffer::empty(area);
        rt.draw_scene(area, &mut buf, |ui, a| {
            prompt().draw(ui, a, &st, |_, _| {});
        })
        .commit_presented();

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

        let no_input = Dialog::new(DLG)
            .title("Rename")
            .actions(&CONFIRM_ACTIONS)
            .body_rows(0);
        assert_eq!(
            d.measured_height(dt)
                .saturating_sub(no_input.measured_height(dt)),
            d.input_rows(dt),
            "the prompt adds exactly the field rows to the frozen dialog chrome"
        );
    }

    #[test]
    fn caller_owned_prompt_error_is_rendered_by_the_field() {
        const ERROR: &str = "Name cannot be empty";
        let (mut rt, mut buf) = scene();
        let st = DialogState::default();
        let d = prompt().error(Some(ERROR));
        let before = d.measured_height(&Theme::junie().design);

        rt.draw_scene(SCREEN, &mut buf, |ui, area| {
            d.draw(ui, area, &st, |_, _| {});
        })
        .commit_presented();

        let frame: String = buf
            .content()
            .iter()
            .map(ratatui_core::buffer::Cell::symbol)
            .collect();
        assert!(
            frame.contains(ERROR),
            "prompt field dropped caller error: {frame}"
        );
        assert_eq!(st, DialogState::default(), "draw mutated prompt state");
        assert_eq!(before, prompt().measured_height(&Theme::junie().design));
    }

    #[test]
    fn an_unconditional_update_receives_the_dismissal() {
        #[derive(Default)]
        struct DialogApp {
            st: DialogState,
            opened: bool,
            dismissed: Vec<DialogAction>,
        }

        impl App for DialogApp {
            fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
                if !self.opened {
                    self.opened = true;
                    cx.open_layer(DLG, confirm().layer(cx));
                }
                let r = confirm().update(cx, &mut self.st);
                if let Some(action) = r.action_ref() {
                    self.dismissed.push(*action);
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
        let _ = rt.initialize();
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut rt, Input::Tick);
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        assert!(rt.is_open(DLG), "the dialog sized and opened its own layer");
        let _ = crate::runtime::stub::deliver(&mut rt, esc());
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        assert!(!rt.is_open(DLG));
        assert_eq!(
            rt.app().dismissed,
            vec![DialogAction::Action(ActionKey::CANCEL)]
        );
        assert!(
            rt.diagnostics().is_empty(),
            "the unconditional shape leaves nothing undelivered: {:?}",
            rt.diagnostics()
        );
    }

    #[test]
    fn harness_dismissal_resolves_cancel_once_and_zeroizes_acknowledgement() {
        #[derive(Default)]
        struct DialogApp {
            state: DialogState,
            opened: bool,
            acknowledgement: bool,
            race: bool,
            close_programmatically: bool,
            close_after_action: bool,
            reopen_after_action: bool,
            close_reopened_after_action: bool,
            close_parent: bool,
            parented: bool,
            dismiss: Dismiss,
            actions: Vec<DialogAction>,
        }

        impl DialogApp {
            fn dialog(&self) -> Dialog<'static> {
                if self.race {
                    Dialog::new(DLG)
                        .title("Review")
                        .body_rows(0)
                        .actions(&RACE_ACTIONS)
                        .cancel(RACE_CANCEL)
                } else if self.acknowledgement {
                    acknowledge()
                } else {
                    confirm()
                }
            }
        }

        impl App for DialogApp {
            fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
                if !self.opened {
                    self.opened = true;
                    if self.parented {
                        cx.open_layer(PARENT, LayerSpec::modal(PARENT));
                    }
                    let dialog = self.dialog();
                    cx.open_layer(DLG, dialog.layer(cx).dismiss(self.dismiss));
                    if self.acknowledgement {
                        self.state.set_secret_mode(true);
                        self.state.ack_draft.set(TOKEN);
                    }
                }
                if self.close_programmatically {
                    self.close_programmatically = false;
                    cx.close_layer(DLG, None);
                }
                if self.close_parent {
                    self.close_parent = false;
                    cx.close_layer(PARENT, None);
                }
                for _ in cx.intents(PARENT) {}
                let response = self.dialog().update(cx, &mut self.state);
                if let Some(action) = response.action_ref() {
                    self.actions.push(*action);
                    if self.close_after_action {
                        self.close_after_action = false;
                        cx.close_layer(DLG, None);
                        if self.reopen_after_action {
                            self.reopen_after_action = false;
                            let dialog = self.dialog();
                            cx.open_layer(DLG, dialog.layer(cx).dismiss(self.dismiss));
                            if self.close_reopened_after_action {
                                self.close_reopened_after_action = false;
                                cx.close_layer(DLG, None);
                            }
                        }
                    }
                }
                response.erase()
            }

            fn draw(&self, ui: &mut Ui<'_>) {
                ui.layer(DLG, |ui, area| {
                    self.dialog().draw(ui, area, &self.state, |_, _| {});
                });
            }
        }

        for (outside_click, programmatic) in [(false, false), (true, false), (false, true)] {
            let app = DialogApp {
                acknowledgement: outside_click || programmatic,
                dismiss: Dismiss::ESC_AND_OUTSIDE,
                ..DialogApp::default()
            };
            let mut harness = Harness::new(app, Theme::junie(), 80, 24);
            let _ = harness.handle(Input::Tick);
            assert!(harness.runtime().is_open(DLG), "the live dialog was opened");
            if programmatic {
                harness.app_mut().close_programmatically = true;
                let _ = harness.handle(Input::Tick);
                let _ = harness.handle(Input::Tick);
            } else if outside_click {
                let _ = harness.click(0, 0);
            } else {
                let _ = harness.key(KeyCode::Esc);
            }
            assert!(
                !harness.runtime().is_open(DLG),
                "the dismissal event should close its modal layer: {:?}",
                harness.runtime().diagnostics()
            );
            assert_eq!(
                harness.app().actions,
                vec![DialogAction::Action(ActionKey::CANCEL)],
                "dismissal must resolve to the configured cancel action exactly once"
            );
            assert!(
                harness.app().state.ack_draft.expose().is_empty(),
                "dismissal must clear the acknowledgement secret"
            );
        }

        let app = DialogApp {
            dismiss: Dismiss::NONE,
            ..DialogApp::default()
        };
        let mut harness = Harness::new(app, Theme::junie(), 80, 24);
        let _ = harness.handle(Input::Tick);
        let _ = harness.key(KeyCode::Esc);
        assert!(harness.runtime().is_open(DLG));
        assert!(harness.app().actions.is_empty());

        let app = DialogApp {
            parented: true,
            acknowledgement: true,
            ..DialogApp::default()
        };
        let mut harness = Harness::new(app, Theme::junie(), 80, 24);
        let _ = harness.handle(Input::Tick);
        assert!(harness.runtime().is_open(PARENT));
        assert!(harness.runtime().is_open(DLG));
        harness.app_mut().close_parent = true;
        let _ = harness.handle(Input::Tick);
        let _ = harness.handle(Input::Tick);
        assert!(!harness.runtime().is_open(DLG));
        assert_eq!(
            harness.app().actions,
            vec![DialogAction::Action(ActionKey::CANCEL)],
            "a cascaded close without an action receipt still resolves to Cancel"
        );
        assert!(harness.app().state.ack_draft.expose().is_empty());

        let app = DialogApp {
            race: true,
            dismiss: Dismiss::ESC_AND_OUTSIDE,
            ..DialogApp::default()
        };
        let mut harness = Harness::new(app, Theme::junie(), 80, 24);
        let _ = harness.handle(Input::Tick);
        let confirm_id = harness.app().dialog().action_id(1);
        harness.runtime_mut().set_focus(Some(confirm_id));
        harness.draw();
        harness.app_mut().close_programmatically = true;
        let _ = harness.key(KeyCode::Enter);
        let _ = harness.handle(Input::Tick);

        assert_eq!(
            harness.app().actions,
            vec![DialogAction::Action(RACE_CANCEL)],
            "the stale confirm intent must be drained while the queued close resolves once"
        );
        assert!(harness.runtime().diagnostics().is_empty());

        let app = DialogApp {
            close_after_action: true,
            ..DialogApp::default()
        };
        let mut harness = Harness::new(app, Theme::junie(), 80, 24);
        let _ = harness.handle(Input::Tick);
        let confirm_id = harness.app().dialog().action_id(1);
        harness.runtime_mut().set_focus(Some(confirm_id));
        harness.draw();
        let _ = harness.key(KeyCode::Enter);
        let _ = harness.handle(Input::Tick);

        assert_eq!(
            harness.app().actions,
            vec![DialogAction::Action(ActionKey::CONFIRM)],
            "closing after a chosen action is not a second dismissal action"
        );

        let app = DialogApp {
            close_after_action: true,
            reopen_after_action: true,
            ..DialogApp::default()
        };
        let mut harness = Harness::new(app, Theme::junie(), 80, 24);
        let _ = harness.handle(Input::Tick);
        let confirm_id = harness.app().dialog().action_id(1);
        harness.runtime_mut().set_focus(Some(confirm_id));
        harness.draw();
        let _ = harness.key(KeyCode::Enter);
        assert!(harness.runtime().is_open(DLG), "the same id was reopened");
        let _ = harness.handle(Input::Tick);
        assert_eq!(
            harness.app().actions,
            vec![DialogAction::Action(ActionKey::CONFIRM)],
            "the previous generation's queued programmatic close is not a new cancel"
        );
        assert!(harness.runtime().is_open(DLG));

        let app = DialogApp {
            close_after_action: true,
            reopen_after_action: true,
            close_reopened_after_action: true,
            ..DialogApp::default()
        };
        let mut harness = Harness::new(app, Theme::junie(), 80, 24);
        let _ = harness.handle(Input::Tick);
        let confirm_id = harness.app().dialog().action_id(1);
        harness.runtime_mut().set_focus(Some(confirm_id));
        harness.draw();
        let _ = harness.key(KeyCode::Enter);
        let _ = harness.handle(Input::Tick);
        let reopened_actions = harness.app().actions.clone();
        let reopened_is_open = harness.runtime().is_open(DLG);

        let app = DialogApp::default();
        let mut harness = Harness::new(app, Theme::junie(), 80, 24);
        let _ = harness.handle(Input::Tick);
        let confirm_id = harness.app().dialog().action_id(1);
        harness.runtime_mut().set_focus(Some(confirm_id));
        harness.draw();
        let _ = harness.key(KeyCode::Enter);
        harness.app_mut().close_programmatically = true;
        let _ = harness.handle(Input::Tick);
        let later_close_actions = harness.app().actions.clone();

        assert_eq!(
            (reopened_actions, reopened_is_open, later_close_actions,),
            (
                vec![
                    DialogAction::Action(ActionKey::CONFIRM),
                    DialogAction::Action(ActionKey::CANCEL),
                ],
                false,
                vec![
                    DialogAction::Action(ActionKey::CONFIRM),
                    DialogAction::Action(ActionKey::CANCEL),
                ],
            ),
            "only a close of the action's exact opening during that update is suppressed"
        );

        // An action that leaves its layer open only suppresses the immediate
        // close receipt window. A later programmatic dismissal remains real.
        let app = DialogApp::default();
        let mut harness = Harness::new(app, Theme::junie(), 80, 24);
        let _ = harness.handle(Input::Tick);
        let confirm_id = harness.app().dialog().action_id(1);
        harness.runtime_mut().set_focus(Some(confirm_id));
        harness.draw();
        let _ = harness.key(KeyCode::Enter);
        let _ = harness.handle(Input::Tick);
        assert_eq!(
            harness.app().actions,
            vec![DialogAction::Action(ActionKey::CONFIRM)]
        );
        harness.app_mut().close_programmatically = true;
        let _ = harness.handle(Input::Tick);
        let _ = harness.handle(Input::Tick);
        assert_eq!(
            harness.app().actions,
            vec![
                DialogAction::Action(ActionKey::CONFIRM),
                DialogAction::Action(ActionKey::CANCEL),
            ],
            "a later independent dismissal still resolves to cancel"
        );
    }

    #[test]
    fn dismissal_without_cancel_action_preserves_reason_and_clears_secret() {
        let dialog = Dialog::new(DLG).title("Notice");
        let mut state = DialogState::default();
        state.set_secret_mode(true);
        state.ack_draft.set(TOKEN);

        assert_eq!(
            dialog.resolve_dismissal(DismissReason::OutsideClick, &mut state),
            DialogAction::Dismissed(DismissReason::OutsideClick)
        );
        assert!(state.ack_draft.expose().is_empty());
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
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let key = |code| {
            Input::Key(Key {
                code,
                mods: KeyModifiers::NONE,
            })
        };
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::F(4)));
        assert_eq!(runtime.app().chosen, Some(QUICK));

        runtime.app_mut().chosen = None;
        runtime.app_mut().keymap.remap_component(
            dialog().action_id(0),
            QUICK,
            Chord::key(KeyCode::F(5)),
        );
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::F(4)));
        assert_eq!(runtime.app().chosen, None);
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::F(5)));
        assert_eq!(runtime.app().chosen, Some(QUICK));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
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
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::F(5)));
        assert_eq!(runtime.app().chosen, None);
    }

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
            })
            .commit_presented();
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
            runtime
                .draw_scene(SCREEN, &mut buffer, |ui, area| {
                    ui.reference(target, |ui| {
                        dialog.draw(ui, area, &DialogState::default(), |_, _| {});
                    });
                })
                .commit_presented();
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
