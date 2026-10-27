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
use termrock_fields::input::{TextAction, TextCmd, TextInput, TextInputState, redacted_text};
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
    chrome_only: bool,
    close_action: Option<ActionKey>,
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
        Part::CLOSE,
        Part::BACKDROP,
    ];

    pub const fn new(id: Id) -> Self {
        Dialog {
            id,
            title: None,
            meta: None,
            head_in_border: false,
            chrome_only: false,
            close_action: None,
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

    pub const fn chrome(id: Id) -> Self {
        let mut dialog = Self::new(id);
        dialog.chrome_only = true;
        dialog
    }

    #[must_use]
    pub const fn close_button(mut self, action: ActionKey) -> Self {
        self.close_action = Some(action);
        self
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

    pub const fn close_id(&self) -> Id {
        self.id.part(Part::CLOSE)
    }

    #[must_use]
    pub const fn title(mut self, s: &'a str) -> Self {
        self.title = Some(s);
        self
    }

    #[must_use]
    pub const fn meta(mut self, s: &'a str) -> Self {
        self.meta = Some(s);
        self
    }

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

    fn prompt_commit_requested(&self, cx: &Cx<'_>) -> bool {
        let input = self.input_control();
        let bindings = input.bindings(BindingState::default());
        let commit = cx.intents(self.input_id()).any(|intent| match intent {
            Intent::Binding(action) => Binding::command(bindings, action) == Some(TextCmd::Commit),
            _ => false,
        });
        let focus_loss = cx
            .intents(self.input_id())
            .any(|intent| matches!(intent, Intent::FocusOut { .. }));
        commit && !focus_loss
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
        let primary = self.primary?;
        self.effective_actions()
            .iter()
            .enumerate()
            .find_map(|(i, a)| (a.key() == primary && self.enabled(i, a, st)).then_some(i))
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
        if let Some(action) = self.close_action
            && let Some(area) = cx.area(self.id)
            && area.width >= 16
            && area.height >= 4
        {
            let response = Button::new(self.close_id(), "×")
                .variant(Variant::QUIET)
                .focusability(Focusability::ClickOnly)
                .update(cx);
            if !suppress_activation {
                if response.activated() {
                    acc.action(DialogAction::Action(action));
                    action_fired = true;
                } else {
                    acc.fold(&response.erase());
                }
            }
        }
        if self.has_input() {
            let prompt_commit_requested = self.prompt.is_some() && self.prompt_commit_requested(cx);
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
            if !suppress_activation && committed && prompt_commit_requested {
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
        if self.close_action.is_some() {
            if outer.width >= 16 && outer.height >= 4 {
                let x = outer.right().saturating_sub(5);
                Button::new(self.close_id(), "×")
                    .variant(Variant::QUIET)
                    .focusability(Focusability::ClickOnly)
                    .draw(
                        ui,
                        Rect {
                            x,
                            y: outer.y,
                            width: 3,
                            height: 1,
                        },
                    );
            }
            return;
        }
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
        let border_head = self.head_in_border;
        let vertical_pad = if border_head {
            0
        } else {
            u16::from(outer.height >= 8)
        };
        let content = crate::layout::inset(
            outer,
            crate::layout::Insets {
                l: if border_head { 2 } else { 3 },
                t: 1 + vertical_pad,
                r: 3,
                b: 1 + vertical_pad,
            },
        );
        let lines = action_lines(action_widths, content.width, 2);
        let requested_action_height = if self.chrome_only && self.actions.is_empty() {
            0
        } else {
            u16::try_from(lines.len().max(1)).unwrap_or(u16::MAX)
        };
        let title_rows = u16::from(self.title.is_some() && !border_head);
        let title_gap = title_rows.min(vertical_pad);

        let natural_footer_gap = !border_head && (!self.chrome_only || !self.actions.is_empty());
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
            .saturating_add(u16::from(natural_footer_gap));
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
                if self.head_in_border && !self.chrome_only && !layout.body.is_empty() {
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
