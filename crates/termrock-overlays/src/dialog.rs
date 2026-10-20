use core::fmt;

use ratatui_core::layout::Rect;

use super::{Acc, PartStyle, overlay_chrome};
use crate::action::{Action, ActionKey};
use crate::event::{Chord, KeyCode};
use crate::id::{Id, Part, PartRef};
use crate::intent::Intent;
use crate::keymap::{Binding, BindingState, Bindings};
use crate::layer::{DismissReason, LayerEvent, LayerSize, LayerSpec};
use crate::layout::{RowAlign, action_row};
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
    }
}

impl Eq for DialogState {}

impl fmt::Debug for DialogState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DialogState")
            .field("input", &self.input)
            .field("draft", &"[redacted]")
            .field("ack_draft", &"[redacted]")
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
            description: None,
            actions: &[],
            cancel: None,
            primary: None,
            width: None,
            body_rows: None,
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

    pub fn update(&self, cx: &mut Cx<'_>, st: &mut DialogState) -> Response<DialogAction> {
        let size = LayerSize::Fixed(
            self.measured_width(cx.design()),
            self.measured_height(cx.design()),
        );
        cx.resize_layer(self.id, size);
        let mut acc = Acc::<DialogAction>::new();
        let mut action_fired = false;
        for it in cx.intents(self.id) {
            match it {
                Intent::Layer(LayerEvent::Dismissed(r)) => {
                    st.zeroize();
                    acc.action(DialogAction::Dismissed(r));
                }

                Intent::Layer(_) => acc.repaint(),
                Intent::Cancel => {
                    st.zeroize();
                    acc.changed();
                }
                _ => {}
            }
        }
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
            acc.fold(&r.erase());
            if committed && self.prompt.is_some() {
                if let Some(i) = self.primary_index(st)
                    && let Some(a) = self.actions.get(i)
                {
                    acc.action(DialogAction::Action(a.key()));
                    action_fired = true;
                }
            } else if committed
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
            if r.activated() {
                acc.action(DialogAction::Action(a.key()));
                action_fired = true;
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

    fn inner_width(&self, d: &DesignTokens) -> u16 {
        // Baseline pad is 2 columns inside the border: width - 2 * (1 + 2).
        self.measured_width(d).saturating_sub(2 * (1 + 2))
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
        let rows = self.body_rows.unwrap_or(d.size.code_preview_lines);
        if rows == 0 { 0 } else { rows.saturating_add(1) }
    }

    pub fn measured_width(&self, d: &DesignTokens) -> u16 {
        self.width.unwrap_or(d.size.dialog_width)
    }

    pub fn measured_height(&self, d: &DesignTokens) -> u16 {
        let inner = self.inner_width(d);
        let desc = self.description.map_or(0, |s| wrapped_rows(s, inner));
        // Borders, title, the gap under the title, the gap above the buttons,
        // and one row of padding above and below (pad.y = 1).
        let actions: u16 = if self.actions.is_empty() { 0 } else { 1 };
        5u16.saturating_add(2)
            .saturating_add(desc)
            .saturating_add(self.input_rows(d))
            .saturating_add(self.body_block(d))
            .saturating_add(actions)
    }

    pub fn layer(&self, cx: &Cx<'_>) -> LayerSpec {
        let d = cx.design();
        LayerSpec::modal(self.id).size(LayerSize::Fixed(
            self.measured_width(d),
            self.measured_height(d),
        ))
    }

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
        let (rect, desc_h) = self.frame_rect(ui, area);
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
                let style = |ui: &mut Ui<'_>, part: Part, flags: StateFlags| {
                    ov.style(ui, id, Family::DIALOG, Variant::DEFAULT, part, flags | live)
                };

                let pad_x = 2u16;
                let pad_y = 1u16;
                let inner = crate::layout::inset(
                    framed,
                    crate::layout::Insets {
                        l: pad_x,
                        t: pad_y,
                        r: pad_x,
                        b: pad_y,
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
                y = y.saturating_add(if self.title.is_some() { 2 } else { 1 });
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
                    Field::new(label, input)
                        .plain(true)
                        .error(self.error)
                        .draw(ui, r, &st.input);
                    y = y.saturating_add(field_h);
                    if self.ack.is_some() {
                        y = y.saturating_add(1);
                    }
                }

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
                    let rects = action_row(row, widths.get(..n).unwrap_or(&[]), 2, RowAlign::End);
                    let mut used_mnemonic = 0u32;
                    for ((i, a), r) in actions.iter().enumerate().zip(rects) {
                        let action_id = self.action_id(i);
                        let enabled = self.enabled(i, a, st);
                        Button::new(action_id, a.label())
                            .variant(self.variant_of(a))
                            .disabled(!enabled)
                            .draw(ui, r);
                        if let Some((offset, ch)) = mnemonic_char(&mut used_mnemonic, a.label()) {
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
        self.paint_close(ui, chrome);
        painted
    }

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
    use crate::theme::Theme;

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
            let bare_h = bare.measured_height(d).saturating_add(2);
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
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut rt, Input::Tick);
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        let before = rt.layer_area(DLG).expect("open layer");
        rt.app_mut().rows = 5;
        let _ = crate::runtime::stub::deliver(&mut rt, Input::Tick);
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
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
        let _ = rt.initialize();
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut rt, Input::Tick);
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        assert!(rt.is_open(DLG), "the dialog sized and opened its own layer");
        let _ = crate::runtime::stub::deliver(&mut rt, esc());
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
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
