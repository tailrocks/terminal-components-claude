use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Margin, Position, Rect};

use crate::core::event::{Key, Outcome};
use crate::core::hints::{HasHints, HintCx};
use crate::core::hit::Axis;
use crate::core::id::WidgetId;
use crate::core::keymap::{Actions, KeyBinding, KeyMap, Nav, NavKeys, lookup};
use crate::core::response::Response;
use crate::core::scroll::{HScroll, ScrollPane, ScrollState, Scrollable};
use crate::theme::{ButtonKind, StatusKind, Strings};
use crate::ui::ctx::{Backdrop, RenderCtx, fill};
use crate::ui::popup::{Dismiss, Place, Placement, place_with};
use crate::widgets::button::{Button, ButtonRow, RowAlign, RowOverflow};
use crate::widgets::input::{InputAction, TextInput};
use crate::widgets::keyhint::{Hint, hint};
use crate::widgets::panel::Panel;
use crate::widgets::scrollbar;

#[derive(Debug, Clone)]
pub enum DialogBody {
    Text(String),
    Input(TextInput),

    Facts {
        facts: Vec<crate::widgets::props::Prop>,
        code: Vec<String>,
        ack: Option<AckInput>,
    },
}

#[derive(Debug, Clone)]
pub struct AckInput {
    pub input: TextInput,
    pub token: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogResult {
    Action(usize),
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct Dialog {
    pub id: WidgetId,
    pub title: String,

    pub title_status: Option<StatusKind>,
    pub body: DialogBody,
    pub actions: Vec<Button>,

    pub cancel_index: Option<usize>,
    pub width: u16,

    pub keymap: KeyMap<DialogCmd>,

    pub nav: NavKeys,
    pub area: Rect,
    pub result: Option<DialogResult>,
    pub initial_focus: WidgetId,

    pub dismiss: Dismiss,

    pub close_button: crate::widgets::icon_button::IconButton,

    pub confirm_index: Option<usize>,

    pub body_scroll: bool,

    pub max_height: Option<u16>,

    pub outer_margin: Margin,

    pub placement: Placement,

    pub anchor: Rect,

    pub backdrop: Backdrop,

    pub chrome: (u16, u16),

    pub button_row: ButtonRow,

    gap: u16,

    pad: crate::theme::Insets,

    pub scroll: ScrollPane,

    pub horizontal: Option<HScroll>,

    strings: Strings,

    pub ack_enter: AckEnter,

    pub mnemonics: bool,

    pristine: String,

    pub guard_dirty_prompt: bool,

    pub guard_destructive_dismiss: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AckEnter {
    #[default]
    Submit,

    NextField,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogCmd {
    Confirm,

    Cancel,
}

impl Actions for DialogCmd {
    fn label(self) -> &'static str {
        match self {
            DialogCmd::Confirm => "Confirm",
            DialogCmd::Cancel => "Cancel",
        }
    }

    fn default_keymap() -> KeyMap<Self> {
        vec![]
    }
}

pub const OUTER_MARGIN: Margin = Margin {
    horizontal: 3,
    vertical: 1,
};

impl HasHints for Dialog {
    fn hints(&self, cx: &HintCx) -> Vec<Hint> {
        Dialog::hints(self, cx.focus)
    }
}

impl Dialog {
    pub fn width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    pub fn accelerators(mut self, keys: Option<(char, char)>) -> Self {
        self.keymap = match keys {
            Some((yes, no)) => vec![
                (KeyBinding::char(yes), DialogCmd::Confirm),
                (KeyBinding::char(no), DialogCmd::Cancel),
            ],
            None => vec![],
        };
        self
    }

    pub fn keymap(mut self, keymap: KeyMap<DialogCmd>) -> Self {
        self.keymap = keymap;
        self
    }

    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.nav = nav;
        self
    }

    pub fn ack_enter(mut self, ack_enter: AckEnter) -> Self {
        self.ack_enter = ack_enter;
        self
    }

    pub fn mnemonics(mut self, on: bool) -> Self {
        self.mnemonics = on;
        self
    }

    pub fn guard_dirty_prompt(mut self, on: bool) -> Self {
        self.guard_dirty_prompt = on;
        self
    }

    pub fn guard_destructive_dismiss(mut self, on: bool) -> Self {
        self.guard_destructive_dismiss = on;
        self
    }

    fn dismissal_is_destructive(&self) -> bool {
        self.cancel_index
            .and_then(|i| self.actions.get(i))
            .is_some_and(|b| b.kind == ButtonKind::Danger)
    }

    pub(crate) fn dismiss_in_force(&self) -> Dismiss {
        let mut d = self.dismiss;
        if self.guard_dirty_prompt && self.prompt_dirty() {
            d.outside_click = false;
        }
        if self.guard_destructive_dismiss && self.dismissal_is_destructive() {
            d.outside_click = false;
            d.focus_out = false;
        }
        d
    }

    fn has_text_field(&self) -> bool {
        matches!(
            self.body,
            DialogBody::Input(_) | DialogBody::Facts { ack: Some(_), .. }
        )
    }

    pub fn mnemonic_keys(&self) -> Vec<Option<(char, usize)>> {
        let mut taken: Vec<char> = vec![];
        self.actions
            .iter()
            .map(|b| {
                let m = b
                    .label
                    .chars()
                    .enumerate()
                    .filter(|(_, c)| c.is_alphanumeric())
                    .map(|(i, c)| (c.to_lowercase().next().unwrap_or(c), i))
                    .find(|(c, _)| !taken.contains(c));
                if let Some((c, _)) = m {
                    taken.push(c);
                }
                m
            })
            .collect()
    }

    fn mnemonics_live(&self) -> bool {
        self.mnemonics && !self.has_text_field()
    }

    fn mnemonic_action(&self, key: &Key) -> Option<usize> {
        let KeyCode::Char(c) = key.code else {
            return None;
        };
        if !self.mnemonics_live() || key.chord() {
            return None;
        }
        let c = c.to_lowercase().next().unwrap_or(c);
        self.mnemonic_keys()
            .iter()
            .position(|m| m.is_some_and(|(k, _)| k == c))
    }

    pub fn prompt_dirty(&self) -> bool {
        matches!(&self.body, DialogBody::Input(i) if i.text() != self.pristine)
    }

    fn gated_action(&self) -> Option<usize> {
        self.confirm_index.or_else(|| {
            self.actions
                .iter()
                .position(|b| matches!(b.kind, ButtonKind::Primary | ButtonKind::Danger))
        })
    }

    pub fn confirm(id: WidgetId, title: &str, text: &str, confirm: &str) -> Self {
        let cancel = Button::subtle(id.sub("cancel"), Strings::DEFAULT.cancel);
        let ok = Button::primary(id.sub("ok"), confirm);
        let ok_id = ok.id;
        Self {
            id,
            title: title.to_owned(),
            body: DialogBody::Text(text.to_owned()),
            actions: vec![cancel, ok],
            cancel_index: Some(0),
            width: 54,
            keymap: DialogCmd::default_keymap(),
            nav: NavKeys::default(),
            area: Rect::ZERO,
            result: None,
            initial_focus: ok_id,
            dismiss: Dismiss::LIGHT,
            close_button: crate::widgets::icon_button::IconButton::close(crate::ui::modal::CLOSE),
            title_status: None,
            confirm_index: None,
            body_scroll: true,
            max_height: None,
            outer_margin: OUTER_MARGIN,
            placement: Placement::Center,
            anchor: Rect::ZERO,
            backdrop: Backdrop::DEFAULT,
            chrome: (0, 0),
            button_row: ButtonRow::new()
                .align(RowAlign::End)
                .overflow(RowOverflow::Wrap),
            gap: crate::theme::Spacing::DEFAULT.button_gap,
            pad: crate::theme::Spacing::DEFAULT.dialog,
            scroll: ScrollPane::default(),
            horizontal: None,
            strings: Strings::DEFAULT,
            ack_enter: AckEnter::Submit,
            mnemonics: true,
            pristine: String::new(),
            guard_dirty_prompt: true,
            guard_destructive_dismiss: true,
        }
    }

    pub fn destructive(id: WidgetId, title: &str, text: &str, confirm: &str) -> Self {
        let cancel = Button::secondary(id.sub("cancel"), Strings::DEFAULT.cancel);
        let cancel_id = cancel.id;
        let del = Button::danger(id.sub("ok"), confirm);
        Self {
            id,
            title: title.to_owned(),
            body: DialogBody::Text(text.to_owned()),
            actions: vec![cancel, del],
            cancel_index: Some(0),
            width: 54,
            keymap: DialogCmd::default_keymap(),
            nav: NavKeys::default(),
            area: Rect::ZERO,
            result: None,
            initial_focus: cancel_id,
            dismiss: Dismiss::LIGHT,
            close_button: crate::widgets::icon_button::IconButton::close(crate::ui::modal::CLOSE),
            title_status: None,
            confirm_index: None,
            body_scroll: true,
            max_height: None,
            outer_margin: OUTER_MARGIN,
            placement: Placement::Center,
            anchor: Rect::ZERO,
            backdrop: Backdrop::DEFAULT,
            chrome: (0, 0),
            button_row: ButtonRow::new()
                .align(RowAlign::End)
                .overflow(RowOverflow::Wrap),
            gap: crate::theme::Spacing::DEFAULT.button_gap,
            pad: crate::theme::Spacing::DEFAULT.dialog,
            scroll: ScrollPane::default(),
            horizontal: None,
            strings: Strings::DEFAULT,
            ack_enter: AckEnter::Submit,
            mnemonics: true,
            pristine: String::new(),
            guard_dirty_prompt: true,
            guard_destructive_dismiss: true,
        }
    }

    pub fn prompt(id: WidgetId, title: &str, input: TextInput, confirm: &str) -> Self {
        let cancel = Button::subtle(id.sub("cancel"), Strings::DEFAULT.cancel);
        let ok = Button::primary(id.sub("ok"), confirm);
        let input_id = input.id;
        let pristine = input.text().to_owned();
        Self {
            id,
            title: title.to_owned(),
            body: DialogBody::Input(input),
            actions: vec![cancel, ok],
            cancel_index: Some(0),
            width: 54,
            keymap: DialogCmd::default_keymap(),
            nav: NavKeys::default(),
            area: Rect::ZERO,
            result: None,
            initial_focus: input_id,
            dismiss: Dismiss::LIGHT,
            close_button: crate::widgets::icon_button::IconButton::close(crate::ui::modal::CLOSE),
            title_status: None,
            confirm_index: None,
            body_scroll: true,
            max_height: None,
            outer_margin: OUTER_MARGIN,
            placement: Placement::Center,
            anchor: Rect::ZERO,
            backdrop: Backdrop::DEFAULT,
            chrome: (0, 0),
            button_row: ButtonRow::new()
                .align(RowAlign::End)
                .overflow(RowOverflow::Wrap),
            gap: crate::theme::Spacing::DEFAULT.button_gap,
            pad: crate::theme::Spacing::DEFAULT.dialog,
            scroll: ScrollPane::default(),
            horizontal: None,
            strings: Strings::DEFAULT,
            ack_enter: AckEnter::Submit,
            mnemonics: true,
            pristine,
            guard_dirty_prompt: true,
            guard_destructive_dismiss: true,
        }
    }

    fn follow_strings(&mut self, s: Strings) {
        let old = std::mem::replace(&mut self.strings, s);
        if old == s {
            return;
        }
        if let Some(b) = self.cancel_index.and_then(|i| self.actions.get_mut(i)) {
            Strings::follow(&mut b.label, old.cancel, s.cancel);
        }
        if let DialogBody::Facts { ack: Some(a), .. } = &mut self.body {
            let tok: &dyn std::fmt::Display = &a.token;
            Strings::follow(
                &mut a.input.label,
                &Strings::fill(old.type_to_confirm, &[tok]),
                &Strings::fill(s.type_to_confirm, &[tok]),
            );
        }
    }

    pub fn facts(
        id: WidgetId,
        title: &str,
        facts: Vec<crate::widgets::props::Prop>,
        code: Vec<String>,
        token: Option<&str>,
        confirm: Button,
    ) -> Self {
        let cancel = Button::secondary(id.sub("cancel"), Strings::DEFAULT.cancel);
        let cancel_id = cancel.id;
        let ack = token.map(|tok| AckInput {
            input: TextInput::new(
                id.sub("ack"),
                &Strings::fill(Strings::DEFAULT.type_to_confirm, &[&tok]),
            )
            .plain_label()
            .wrap_label(true),
            token: tok.to_owned(),
        });
        let initial = ack.as_ref().map(|a| a.input.id).unwrap_or(cancel_id);
        Self {
            id,
            title: title.to_owned(),
            body: DialogBody::Facts { facts, code, ack },
            actions: vec![cancel, confirm],
            cancel_index: Some(0),
            width: 66,
            keymap: DialogCmd::default_keymap(),
            nav: NavKeys::default(),
            area: Rect::ZERO,
            result: None,
            initial_focus: initial,
            dismiss: if token.is_some() {
                Dismiss::ESCAPE
            } else {
                Dismiss::LIGHT
            },
            close_button: crate::widgets::icon_button::IconButton::close(crate::ui::modal::CLOSE),
            title_status: None,
            confirm_index: token.map(|_| 1),
            body_scroll: true,
            max_height: None,
            outer_margin: OUTER_MARGIN,
            placement: Placement::Center,
            anchor: Rect::ZERO,
            backdrop: Backdrop::DEFAULT,
            chrome: (0, 0),
            button_row: ButtonRow::new()
                .align(RowAlign::End)
                .overflow(RowOverflow::Wrap),
            gap: crate::theme::Spacing::DEFAULT.button_gap,
            pad: crate::theme::Spacing::DEFAULT.dialog,
            scroll: ScrollPane::default(),
            horizontal: None,
            strings: Strings::DEFAULT,
            ack_enter: AckEnter::Submit,
            mnemonics: true,
            pristine: String::new(),
            guard_dirty_prompt: true,
            guard_destructive_dismiss: true,
        }
    }

    pub fn armed(&self) -> bool {
        match &self.body {
            DialogBody::Facts { ack: Some(a), .. } => a.input.text().trim() == a.token,
            _ => true,
        }
    }

    pub fn hints(&self, focused: Option<WidgetId>) -> Vec<Hint> {
        let s = &self.strings;
        let cancel_label = self
            .cancel_index
            .and_then(|i| self.actions.get(i))
            .map(|b| b.label.clone());
        let dismiss_hint = || {
            self.dismiss.escape.then(|| {
                Hint::new(
                    "Esc",
                    cancel_label.clone().unwrap_or_else(|| s.hint_close.into()),
                )
            })
        };
        let mut v = Vec::new();
        let input = match &self.body {
            DialogBody::Input(i) => Some(i),
            DialogBody::Facts { ack: Some(a), .. } => Some(&a.input),
            _ => None,
        };
        if let Some(inp) = input
            && focused == Some(inp.id)
        {
            if let DialogBody::Facts { ack: Some(a), .. } = &self.body {
                let gated = self
                    .gated_action()
                    .and_then(|i| self.actions.get(i))
                    .map(|b| b.label.clone());
                match (self.ack_enter, self.armed(), gated) {
                    (AckEnter::Submit, true, Some(label)) => v.push(Hint::new("Enter", label)),
                    (AckEnter::Submit, _, _) => v.push(Hint::new(
                        "Type",
                        Strings::fill(s.type_to_confirm_hint, &[&a.token]),
                    )),
                    (AckEnter::NextField, ..) => {
                        v.push(Hint::new(
                            "Type",
                            Strings::fill(s.type_to_confirm_hint, &[&a.token]),
                        ));
                        v.push(hint("Enter", s.hint_next));
                    }
                }
            } else if inp.editing {
                let primary = self
                    .actions
                    .iter()
                    .find(|b| b.kind == ButtonKind::Primary)
                    .map(|b| b.label.clone());
                v.push(Hint::new(
                    "Enter",
                    primary.unwrap_or_else(|| s.hint_apply.into()),
                ));
            } else {
                v.push(hint("Enter", s.hint_edit));
                v.push(hint("Tab", s.hint_next));
            }
            if inp.editing {
                v.push(hint("Esc", s.hint_cancel));
            } else {
                v.extend(dismiss_hint());
            }
            return v;
        }

        let usable =
            |i: usize, b: &Button| !b.disabled && (Some(i) != self.confirm_index || self.armed());
        let enabled = self
            .actions
            .iter()
            .enumerate()
            .filter(|(i, b)| usable(*i, b))
            .count();
        if enabled > 1 {
            v.push(hint("←→", s.hint_choose));
        }
        if let Some((_, b)) = self
            .actions
            .iter()
            .enumerate()
            .find(|(i, b)| Some(b.id) == focused && usable(*i, b))
        {
            v.push(Hint::new("Enter", b.label.clone()));
        }
        if self.scroll.overflows() {
            v.push(hint("↑↓", s.hint_scroll));
        }
        if self.scroll.h_cfg.enabled && self.scroll.h.overflows() {
            v.push(hint("⇧←→", s.hint_scroll_code));
        }
        v.extend(dismiss_hint());

        if matches!(self.body, DialogBody::Text(_)) {
            if let Some(k) = crate::core::keymap::hint_key(&self.keymap, DialogCmd::Confirm)
                && let Some(p) = self.actions.iter().find(|b| {
                    matches!(b.kind, ButtonKind::Primary | ButtonKind::Danger) && !b.disabled
                })
            {
                v.push(Hint::new(k, p.label.clone()));
            }
            if let Some(k) = crate::core::keymap::hint_key(&self.keymap, DialogCmd::Cancel)
                && let Some(c) = cancel_label
            {
                v.push(Hint::new(k, c));
            }
        }
        v
    }

    pub fn cancel_label(mut self, label: &str) -> Self {
        if let Some(b) = self.cancel_index.and_then(|i| self.actions.get_mut(i)) {
            b.label = label.to_owned();
        }
        self
    }

    pub fn title_status(mut self, kind: StatusKind) -> Self {
        self.title_status = Some(kind);
        self
    }

    pub fn dismiss(mut self, dismiss: Dismiss) -> Self {
        self.dismiss = dismiss;
        self
    }

    pub fn close_button(mut self, button: crate::widgets::icon_button::IconButton) -> Self {
        self.close_button = button;
        self
    }

    pub fn edit_entry(mut self, entry: crate::core::edit::EditEntry) -> Self {
        if let Some(inp) = self.input_mut() {
            inp.set_edit_entry(entry);
        }
        self
    }

    pub(crate) fn dismissed(&mut self) -> Outcome {
        match self.cancel_index {
            Some(ci) => self.finish(DialogResult::Action(ci)),
            None => self.finish(DialogResult::Cancelled),
        }
    }

    pub fn actions(mut self, actions: Vec<Button>, cancel_index: Option<usize>) -> Self {
        self.actions = actions;
        self.cancel_index = cancel_index;
        self
    }

    #[deprecated(note = "renamed to `Dialog::actions`")]
    pub fn with_actions(mut self, actions: Vec<Button>, cancel_index: Option<usize>) -> Self {
        self.actions = actions;
        self.cancel_index = cancel_index;
        self
    }

    pub fn is_editing(&self) -> bool {
        match &self.body {
            DialogBody::Input(i) => i.editing,
            DialogBody::Facts { ack: Some(a), .. } => a.input.editing,
            _ => false,
        }
    }

    fn input_mut(&mut self) -> Option<&mut TextInput> {
        match &mut self.body {
            DialogBody::Input(i) => Some(i),
            DialogBody::Facts { ack: Some(a), .. } => Some(&mut a.input),
            _ => None,
        }
    }

    fn code_rows(code: &[String]) -> u16 {
        if code.is_empty() {
            0
        } else {
            code.len() as u16 + 1
        }
    }

    fn code_bar_rows(code: &[String], inner_w: u16, h_cfg: &HScroll) -> u16 {
        u16::from(!code.is_empty() && h_cfg.shows_bar(&Self::code_extent(code, inner_w)))
    }

    fn code_extent(code: &[String], inner_w: u16) -> ScrollState {
        let widest = code
            .iter()
            .map(|l| crate::ui::text::width(l))
            .max()
            .unwrap_or(0);
        let mut s = ScrollState::new(widest);
        s.set_viewport(inner_w as usize);
        s
    }

    fn scroll_code(&mut self, delta: isize) -> Outcome {
        self.scroll.wheel(Axis::Horizontal, delta as i32).outcome()
    }

    pub fn on_drag(&mut self, pressed: WidgetId, pos: Position) -> Outcome {
        if let Some(inp) = self.input_mut()
            && inp.id == pressed
        {
            return inp.on_drag(pos);
        }
        if scrollbar::is_bar_of(pressed, self.id) {
            return match self.pane().bar_of(pressed) {
                Some(bar) => self.bar_input(&bar, pos, false).outcome(),
                None => Outcome::Consumed,
            };
        }
        Outcome::Consumed
    }

    fn ack_rows(&self, ack: Option<&AckInput>, width: u16) -> u16 {
        ack.map_or(0, |a| a.input.height_for(self.field_width(width)) + 1)
    }

    fn field_width(&self, width: u16) -> u16 {
        self.inner_width(width) + TextInput::GUTTER
    }

    fn inner_width(&self, width: u16) -> u16 {
        width.saturating_sub(2 * (1 + self.pad.x))
    }

    fn scroll_rows(&self, width: u16) -> u16 {
        let inner_w = self.inner_width(width);
        match &self.body {
            DialogBody::Text(t) => crate::ui::text::wrap(t, inner_w as usize).len() as u16,
            DialogBody::Input(_) => 0,
            DialogBody::Facts { facts, code, .. } => {
                crate::widgets::props::measure(facts, inner_w) + Self::code_rows(code)
            }
        }
    }

    fn fixed_rows(&self, width: u16) -> u16 {
        match &self.body {
            DialogBody::Text(_) => 0,
            DialogBody::Input(i) => i.height_for(self.field_width(width)),
            DialogBody::Facts { code, ack, .. } => {
                Self::code_bar_rows(code, self.inner_width(width), &self.scroll.h_cfg)
                    + self.ack_rows(ack.as_ref(), width)
            }
        }
    }

    fn chrome_rows(&self) -> u16 {
        5 + 2 * self.pad.y
    }

    fn action_rows(&self, width: u16) -> u16 {
        self.button_row
            .rows_at_gap(self.inner_width(width), &self.actions, self.gap)
            .max(1)
    }

    pub fn height(&self, width: u16) -> u16 {
        self.chrome_rows()
            + self.action_rows(width)
            + self.scroll_rows(width)
            + self.fixed_rows(width)
    }

    pub fn button_row(mut self, row: ButtonRow) -> Self {
        self.button_row = row;
        self
    }

    pub fn body_scroll(mut self, on: bool) -> Self {
        self.body_scroll = on;
        self
    }

    pub fn max_height(mut self, rows: Option<u16>) -> Self {
        self.max_height = rows;
        self
    }

    pub fn outer_margin(mut self, margin: Margin) -> Self {
        self.outer_margin = margin;
        self
    }

    pub fn backdrop(mut self, backdrop: Backdrop) -> Self {
        self.backdrop = backdrop;
        self
    }

    pub fn chrome(mut self, top: u16, bottom: u16) -> Self {
        self.chrome = (top, bottom);
        self
    }

    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    pub fn anchor(mut self, anchor: Rect, placement: Placement) -> Self {
        self.anchor = anchor;
        self.placement = placement;
        self
    }

    pub fn confirm_index(mut self, index: Option<usize>) -> Self {
        self.confirm_index = index;
        self
    }

    fn finish(&mut self, r: DialogResult) -> Outcome {
        if let DialogResult::Action(i) = r
            && Some(i) == self.confirm_index
            && !self.armed()
        {
            return Outcome::Consumed;
        }
        if let DialogResult::Action(i) = r
            && let DialogBody::Input(inp) = &mut self.body
            && Some(i) != self.cancel_index
        {
            if inp.editing {
                inp.commit();
            }
            if !inp.validate() {
                inp.resume();
                return Outcome::Changed;
            }
        }
        self.result = Some(r);
        Outcome::Changed
    }

    pub fn on_key(
        &mut self,
        key: &Key,
        focus: &mut crate::core::focus::Focus,
        ring: &crate::core::focus::FocusRing,
    ) -> Outcome {
        let cur = focus.current();

        if key.is(KeyCode::Esc)
            && self.dismiss.escape
            && let Some(inp) = self.input_mut()
            && cur == Some(inp.id)
            && inp.entry() == crate::core::edit::EditEntry::Immediate
        {
            return self.dismissed();
        }

        let is_facts = matches!(self.body, DialogBody::Facts { .. });
        if let Some(inp) = self.input_mut()
            && cur == Some(inp.id)
        {
            let (o, ev) = inp.on_key(key);
            match ev {
                Some(InputAction::CommittedTab { backward }) => {
                    if backward {
                        focus.prev(ring);
                    } else {
                        focus.next(ring);
                    }
                    return Outcome::Changed;
                }
                Some(InputAction::Committed) => {
                    if is_facts {
                        if self.ack_enter == AckEnter::NextField {
                            focus.next(ring);
                            return Outcome::Changed;
                        }
                        if self.armed()
                            && let Some(i) = self.gated_action()
                        {
                            return self.finish(DialogResult::Action(i));
                        }
                        let mismatch = self.strings.ack_mismatch;
                        if let Some(inp) = self.input_mut() {
                            inp.begin_edit();
                            inp.field.error = Some(mismatch.into());
                        }
                        return Outcome::Changed;
                    }
                    let primary = self
                        .actions
                        .iter()
                        .position(|b| b.kind == ButtonKind::Primary);
                    if let Some(p) = primary {
                        return self.finish(DialogResult::Action(p));
                    }
                    return o;
                }
                _ => {}
            }
            if o.consumed() {
                return o;
            }
        }
        for (i, b) in self.actions.iter_mut().enumerate() {
            if cur == Some(b.id) {
                let (o, activated) = b.on_key(key);
                if activated {
                    return self.finish(DialogResult::Action(i));
                }
                if o.consumed() {
                    return o;
                }
            }
        }
        if matches!(self.body, DialogBody::Text(_)) {
            match lookup(&self.keymap, key) {
                Some(DialogCmd::Confirm) => {
                    let primary = self.actions.iter().position(|b| {
                        matches!(b.kind, ButtonKind::Primary | ButtonKind::Danger) && !b.disabled
                    });
                    return match primary {
                        Some(p) => self.finish(DialogResult::Action(p)),
                        None => Outcome::Consumed,
                    };
                }
                Some(DialogCmd::Cancel) => {
                    return match self.cancel_index {
                        Some(ci) => self.finish(DialogResult::Action(ci)),
                        None => Outcome::Consumed,
                    };
                }
                None => {}
            }
        }

        if let Some(i) = self.mnemonic_action(key) {
            let usable =
                !self.actions[i].disabled && (Some(i) != self.confirm_index || self.armed());
            return if usable {
                self.finish(DialogResult::Action(i))
            } else {
                Outcome::Consumed
            };
        }
        let overflows = self.scroll.overflows();
        let arrow = !matches!(key.code, KeyCode::Char(_));
        match (self.nav.action(key), key.code) {
            (_, KeyCode::Esc) if self.dismiss.escape => self.dismissed(),
            (_, KeyCode::Tab) => {
                focus.next(ring);
                Outcome::Changed
            }
            (_, KeyCode::BackTab) => {
                focus.prev(ring);
                Outcome::Changed
            }

            (Some(Nav::Up), _) if overflows => self.wheel(Axis::Vertical, -1).outcome(),
            (Some(Nav::Down), _) if overflows => self.wheel(Axis::Vertical, 1).outcome(),
            (Some(Nav::PageUp), _) if overflows => {
                self.scroll.page_up();
                Outcome::Changed
            }
            (Some(Nav::PageDown), _) if overflows => {
                self.scroll.page_down();
                Outcome::Changed
            }
            (Some(Nav::Home), _) if overflows => {
                self.scroll.jump_start();
                Outcome::Changed
            }
            (Some(Nav::End), _) if overflows => {
                self.scroll.jump_end();
                Outcome::Changed
            }

            (Some(Nav::Left), _) if arrow && key.shift() && self.scroll.h.overflows() => {
                self.scroll_code(-1)
            }
            (Some(Nav::Right), _) if arrow && key.shift() && self.scroll.h.overflows() => {
                self.scroll_code(1)
            }
            (Some(nav @ (Nav::Left | Nav::Right)), _) => {
                if let Some(id) = self.button_row.step(&self.actions, cur, nav == Nav::Right) {
                    focus.focus(id);
                }
                Outcome::Changed
            }
            _ => Outcome::Consumed,
        }
    }

    pub fn on_blur(&mut self, change: crate::core::focus::FocusChange) -> Outcome {
        match self.input_mut() {
            Some(inp) if inp.id == change.from => match inp.on_blur() {
                Some(_) => Outcome::Changed,
                None => Outcome::Ignored,
            },
            _ => Outcome::Ignored,
        }
    }

    pub fn on_paste(&mut self, text: &str) -> Outcome {
        match self.input_mut() {
            Some(inp) => inp.on_paste(text).or(Outcome::Consumed),
            None => Outcome::Consumed,
        }
    }

    pub fn on_press(&mut self, id: WidgetId, pos: Position, count: u8) -> Outcome {
        match self.input_mut() {
            Some(inp) if inp.id == id => inp.on_press_count(pos, count),
            _ => Outcome::Ignored,
        }
    }

    pub fn on_click(
        &mut self,
        id: WidgetId,
        pos: Position,
        focus: &mut crate::core::focus::Focus,
    ) -> Outcome {
        self.on_click_count(id, pos, 1, focus)
    }

    pub fn on_click_count(
        &mut self,
        id: WidgetId,
        pos: Position,
        count: u8,
        focus: &mut crate::core::focus::Focus,
    ) -> Outcome {
        if id == crate::ui::modal::CLOSE && self.dismiss.shows_close() {
            return self.dismissed();
        }
        if let Some(inp) = self.input_mut()
            && inp.id == id
        {
            focus.focus(id);
            return inp.on_click_count(pos, count.max(1));
        }
        if scrollbar::is_bar_of(id, self.id) {
            let o = self.bar_press(pos).outcome().or(Outcome::Consumed);
            if id == scrollbar::id_for(self.id) {
                self.scroll.v.release_track();
            }
            return o;
        }
        for i in 0..self.actions.len() {
            if self.actions[i].id == id {
                focus.focus(id);
                if self.actions[i].on_click() {
                    return self.finish(DialogResult::Action(i));
                }
                return Outcome::Changed;
            }
        }
        Outcome::Consumed
    }

    pub fn on_click_outside(&mut self) -> Outcome {
        if self.dismiss_in_force().outside_click {
            self.dismissed()
        } else {
            Outcome::Consumed
        }
    }

    pub fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let bounds = body_bounds(screen, self.chrome.0, self.chrome.1);
        self.render_in(screen, bounds, buf, ctx);
    }

    pub fn render_in(&mut self, screen: Rect, bounds: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        self.follow_strings(ctx.theme.strings);
        let t = ctx.theme;
        self.scroll.h_cfg = self.horizontal.unwrap_or(ctx.hscroll);
        self.scroll.begin(ctx.track_click);
        self.gap = t.spacing.button_gap;

        self.pad = crate::theme::Insets::new(t.spacing.dialog.x.max(1), t.spacing.dialog.y);

        ctx.modal_backdrop_with(buf, screen, self.backdrop);
        let bounds = bounds.intersection(screen);
        let bounds = if bounds.is_empty() { screen } else { bounds };
        let m = self.outer_margin;

        let width = self
            .width
            .min(bounds.width.saturating_sub(m.horizontal * 2))
            .max(20)
            .min(bounds.width);
        let room = bounds
            .height
            .saturating_sub(m.vertical * 2)
            .min(self.max_height.unwrap_or(u16::MAX));
        let height = self.height(width).min(room);
        let area = match self.placement {
            Placement::Center => {
                bounds.centered(Constraint::Length(width), Constraint::Length(height))
            }
            side => place_with(
                bounds,
                self.anchor,
                width,
                height,
                side,
                Place::DEFAULT
                    .margin(m.horizontal)
                    .vertical_margin(Some(m.vertical)),
            ),
        };
        self.area = area;
        let bg = t.surface_elevated;
        fill(buf, area, ratatui::style::Style::new().bg(bg));
        let panel = Panel::framed(None).focused(true);
        let block = ratatui::widgets::Block::new()
            .borders(ratatui::widgets::Borders::ALL)
            .border_set(t.glyphs.border)
            .border_style(crate::ui::modal::border_style(ctx, bg));
        ratatui::widgets::Widget::render(block, area, buf);
        let _ = panel;
        let close = if self.dismiss.shows_close() {
            crate::ui::modal::close_button_with(&self.close_button, area, buf, ctx, bg)
        } else {
            Rect::ZERO
        };
        let inner = area.inner(Margin::new(1 + self.pad.x, 1 + self.pad.y));
        if inner.is_empty() {
            return;
        }
        let mut title_x = inner.x;
        if let Some(kind) = self.title_status {
            let (glyph, st) = t.status(kind);
            let fg = t.legible(st.fg.unwrap_or(t.text_primary), bg);
            buf.set_string(title_x, inner.y, glyph, st.fg(fg).bg(bg));
            title_x += crate::ui::text::width(glyph) as u16 + 1;
        }
        buf.set_string(
            title_x,
            inner.y,
            crate::ui::text::truncate(&self.title, inner.right().saturating_sub(title_x) as usize),
            t.title().bg(bg),
        );
        let body_y = inner.y + 2;
        let action_rows = self.action_rows(width);
        let actions_y = area.bottom().saturating_sub(1 + self.pad.y + action_rows);

        let fixed = self.fixed_rows(width);
        let scroll_bottom = actions_y.saturating_sub(1 + fixed).max(body_y);
        let viewport = scroll_bottom - body_y;
        let content = self.scroll_rows(width);
        self.scroll.set_content(content as usize);
        self.scroll.set_viewport(viewport as usize);
        if !self.body_scroll {
            self.scroll.offset = 0;
        }
        self.scroll.clamp();
        let offset = self.scroll.offset as u16;

        let field_x = inner.x.saturating_sub(TextInput::GUTTER).max(area.x + 1);
        let field_w = inner.right() - field_x;
        match &mut self.body {
            DialogBody::Text(text) => {
                let lines = crate::ui::text::wrap(text, inner.width as usize);
                for (i, l) in lines
                    .iter()
                    .skip(offset as usize)
                    .take(viewport as usize)
                    .enumerate()
                {
                    buf.set_string(inner.x, body_y + i as u16, l, t.secondary().bg(bg));
                }
            }
            DialogBody::Input(inp) => {
                let r = Rect::new(field_x, body_y, field_w, inp.height_for(field_w));
                inp.render(r, buf, ctx, bg);
            }
            DialogBody::Facts { facts, code, ack } => {
                let sheet_area = Rect::new(0, 0, inner.width, content);
                let mut sheet = Buffer::empty(sheet_area);
                fill(&mut sheet, sheet_area, ratatui::style::Style::new().bg(bg));
                crate::widgets::props::render(sheet_area, &mut sheet, t, facts, bg);
                let facts_h = crate::widgets::props::measure(facts, inner.width);
                let mut y = scroll_bottom;
                if !code.is_empty() {
                    let extent = Self::code_extent(code, inner.width);
                    let off = self.scroll.h.offset;
                    self.scroll.h.set_content(extent.content_len);
                    self.scroll.h.set_viewport(extent.viewport_len);
                    let off = if self.scroll.h_cfg.enabled {
                        off.min(self.scroll.h.max_offset())
                    } else {
                        0
                    };
                    self.scroll.h.offset = off;
                    let cut = |line: &str, w: u16| {
                        if off > 0 {
                            crate::ui::text::slice_cells(line, off, w as usize)
                        } else {
                            crate::ui::text::truncate(line, w as usize)
                        }
                    };
                    let code_y = facts_h + 1;
                    for (i, line) in code.iter().enumerate() {
                        sheet.set_string(
                            0,
                            code_y + i as u16,
                            cut(line, inner.width),
                            t.secondary().bg(bg),
                        );
                    }
                    if self.scroll.h_cfg.shows_bar(&self.scroll.h) {
                        let hb = Rect::new(inner.x, y, inner.width, 1);
                        if hb.y < actions_y {
                            self.scroll.draw_h(hb, buf, ctx, self.id, true);
                        }
                        y += 1;
                    }
                }
                blit_rows(
                    &sheet,
                    offset,
                    buf,
                    Rect::new(inner.x, body_y, inner.width, viewport),
                );
                if let Some(a) = ack {
                    y += 1;
                    let r = Rect::new(field_x, y, field_w, a.input.height_for(field_w));
                    a.input.render(r, buf, ctx, bg);
                }
            }
        }
        if self.scroll.overflows() && viewport > 0 {
            let track = Rect::new(area.right() - 2, body_y, 1, viewport);
            self.scroll.set_track(self.id, Axis::Vertical, track);
            let below = self.scroll.max_offset() - self.scroll.offset;

            let hb = self.scroll.h_track();
            let h_bar = !hb.is_empty() && hb.y == scroll_bottom;
            if below > 0 && scroll_bottom < actions_y && !h_bar {
                buf.set_string(
                    inner.x,
                    scroll_bottom,
                    crate::ui::text::truncate(
                        &format!(
                            "{} {}",
                            t.glyphs.arrows[1],
                            Strings::fill(t.strings.more_count, &[&below])
                        ),
                        inner.width as usize,
                    ),
                    t.muted().bg(bg),
                );
            }
        }

        if let DialogBody::Facts { ack: Some(a), .. } = &self.body {
            let armed = a.input.text().trim() == a.token;
            if let Some(b) = self.confirm_index.and_then(|i| self.actions.get_mut(i)) {
                b.disabled = !armed;
            }
        }

        let live = self.mnemonics_live();
        let keys = self.mnemonic_keys();
        for (b, m) in self.actions.iter_mut().zip(keys) {
            b.mnemonic = m.filter(|_| live).map(|(_, i)| i);
        }

        self.button_row.render(
            self.id,
            Rect::new(inner.x, actions_y, inner.width, action_rows),
            &mut self.actions,
            buf,
            ctx,
            bg,
        );

        ctx.hits.register_surface(self.id, area);

        match &self.body {
            DialogBody::Input(inp) => ctx.hits.register(inp.id, inp.area),
            DialogBody::Facts { ack: Some(a), .. } => ctx.hits.register(a.input.id, a.input.area),
            _ => {}
        }
        for b in &self.actions {
            ctx.hits.register(b.id, b.area);
        }
        if !close.is_empty() {
            ctx.clickable(crate::ui::modal::CLOSE, close);
        }

        let track = self.scroll.v_track();
        if !track.is_empty() {
            self.scroll.draw_v(track, buf, ctx, self.id, true);
        }
        let htrack = self.scroll.h_track();
        if !htrack.is_empty() && self.scroll.h.overflows() {
            ctx.hits.register(scrollbar::h_id_for(self.id), htrack);
        }
    }
}

pub fn body_bounds(screen: Rect, top: u16, bottom: u16) -> Rect {
    let h = screen.height.saturating_sub(top + bottom);
    if h == 0 {
        return screen;
    }
    Rect::new(screen.x, screen.y + top, screen.width, h)
}

fn blit_rows(src: &Buffer, from: u16, dst: &mut Buffer, at: Rect) {
    let at = at.intersection(*dst.area());
    for dy in 0..at.height {
        let sy = from + dy;
        if sy >= src.area.bottom() {
            break;
        }
        for dx in 0..at.width.min(src.area.width) {
            if let (Some(c), Some(d)) = (
                src.cell((src.area.x + dx, sy)),
                dst.cell_mut((at.x + dx, at.y + dy)),
            ) {
                *d = c.clone();
            }
        }
    }
}

impl Scrollable for Dialog {
    fn pane(&mut self) -> &mut ScrollPane {
        &mut self.scroll
    }

    fn wheel(&mut self, axis: Axis, delta: i32) -> Response<()> {
        let r = self.scroll.wheel(axis, delta);
        if r.is_consumed() {
            r
        } else {
            Response::consumed()
        }
    }
}

impl Dialog {
    pub fn pointer_in(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
        focus: &mut crate::core::focus::Focus,
    ) -> Response<()> {
        use crate::core::intent::Phase;
        use crate::core::pointer::MouseTarget;
        if let Some(inp) = self.input_mut()
            && let Some(r) = inp.route(ev)
        {
            if ev.is(Phase::Click) {
                focus.focus(inp.id);
            }
            return r;
        }
        if ev.wheel().is_some() && !ev.on(self.id) {
            return Response::ignored();
        }
        if ev.on(self.id)
            && let Some(r) = crate::core::pointer::scroll_input(self, ev)
        {
            return r;
        }
        let (id, pos) = (ev.id, ev.pos());
        match ev.phase() {
            Some(Phase::Press) => self.on_press(id, pos, ev.count()).into(),
            Some(Phase::Drag) => self.on_drag(id, pos).into(),
            Some(Phase::Click) => self.on_click_count(id, pos, ev.count(), focus).into(),
            _ => Response::ignored(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;
    use crate::widgets::props::Prop;

    fn gate(width: u16) -> Dialog {
        let facts = vec![
            Prop::new("Action", "move to Trash 4 items under ~/work/ide"),
            Prop::new("Mode", "Trash · native macOS Trash · space returns after the Trash is emptied").wrap(),
            Prop::new("Estimate", "44.0 KiB allocated · APFS clones may overcount · purgeable space excluded · not a free-space guarantee").wrap(),
            Prop::new("Threat model", "an unprivileged user · ancestors are re-resolved at commit · a concurrent rename of the final path cannot be excluded").wrap(),
            Prop::new("Paths", "~/work/ide/.idea · ~/work/ide/a/b/c/d/e/deep.iml · ~/work/ide/app.iml · ~/work/ide/mod/mod.iml").wrap(),
        ];
        let code = vec![
            "/Users/alex/work/ide/.idea".to_owned(),
            "/Users/alex/work/ide/a/b/c/d/e/deep.iml".to_owned(),
        ];
        let mut d = Dialog::facts(
            WidgetId::of("gate2"),
            "Trash 4 items · gate 2 of 2",
            facts,
            code,
            Some("TRASH 4 UNDER /Users/alex/work/ide ON mbp"),
            Button::danger(WidgetId::of("gate2").sub("ok"), "Execute"),
        );
        d.width = width;
        d
    }

    #[test]
    fn a_title_status_leads_the_title_with_its_glyph_and_hue() {
        let theme = Theme::junie();
        let screen = Rect::new(0, 0, 80, 24);
        let draw = |d: &mut Dialog| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut buf = Buffer::empty(screen);
            let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
            d.render(screen, &mut buf, &mut ctx);
            buf
        };
        let row = |buf: &Buffer, y: u16| {
            (0..80)
                .map(|x| buf[(x, y)].symbol().to_owned())
                .collect::<String>()
        };
        let mut plain = gate(70);
        let buf = draw(&mut plain);
        let y = (0..24)
            .find(|&y| row(&buf, y).contains("Trash 4 items"))
            .unwrap();
        assert!(!row(&buf, y).contains("▲"));
        let mut d = gate(70).title_status(StatusKind::Warn);
        let buf = draw(&mut d);
        let line = row(&buf, y);
        assert!(line.contains("▲ Trash 4 items"), "{line:?}");
        let x = (0..80).find(|&x| buf[(x, y)].symbol() == "▲").unwrap();
        let (_, st) = theme.status(StatusKind::Warn);
        let c = &buf[(x, y)];
        assert_eq!(c.fg, theme.legible(st.fg.unwrap(), c.bg));
    }

    #[test]
    fn every_line_of_a_long_code_preview_is_reached_by_scrolling_the_body() {
        let code: Vec<String> = (1..=14).map(|i| format!("step {i:02} apt-get")).collect();
        let mut d = Dialog::facts(
            WidgetId::of("plan"),
            "Start plan",
            vec![Prop::new("Host", "mbp"), Prop::new("Steps", "14")],
            code,
            None,
            Button::primary(WidgetId::of("plan").sub("ok"), "Start"),
        );
        let screen = Rect::new(0, 0, 72, 20);
        let text = rows(&draw(&mut d, screen, screen)).join("\n");
        assert!(text.contains("Host"), "{text}");
        assert!(text.contains("step 01"), "{text}");
        assert!(!text.contains("… "), "no line is summarised: {text}");
        assert!(text.contains("more"), "the cut is announced: {text}");
        let mut focus = crate::core::focus::Focus::default();
        focus.focus(d.actions[0].id);
        let ring = FocusRing::default();
        let mut seen = text;
        for _ in 0..20 {
            d.on_key(&key(KeyCode::Down), &mut focus, &ring);
            seen.push_str(&rows(&draw(&mut d, screen, screen)).join("\n"));
        }
        for i in 1..=14 {
            assert!(seen.contains(&format!("step {i:02}")), "line {i}: {seen}");
        }
    }

    #[test]
    fn a_code_line_wider_than_the_dialog_scrolls_sideways_under_its_own_bar() {
        let sql = "DELETE FROM customers WHERE id IN ('9e3d2906-68f5-4661-9c1c-e263532d8d84', 'da8dba6a-5f40-4c1e-9ebe-c4cfcc81aa10');";
        let mut d = Dialog::facts(
            WidgetId::of("del"),
            "Delete 2 rows",
            vec![Prop::new("Table", "public.customers")],
            vec![sql.to_owned()],
            None,
            Button::danger(WidgetId::of("del").sub("ok"), "Delete"),
        );
        let screen = Rect::new(0, 0, 80, 24);
        let theme = Theme::junie();
        let draw = |d: &mut Dialog| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut buf = Buffer::empty(screen);
            {
                let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
                d.render(screen, &mut buf, &mut ctx);
            }
            (buf, hits)
        };
        let (buf, hits) = draw(&mut d);
        let bar = d.scroll.h_track();
        assert!(!bar.is_empty(), "the preview gets a horizontal bar");
        assert_eq!(buf[(bar.x, bar.y)].symbol(), scrollbar::H_THUMB);
        assert_eq!(
            hits.hit(Position::new(bar.x + 3, bar.y)),
            Some(scrollbar::h_id_for(d.id))
        );
        assert!(d.hints(None).iter().any(|h| h.key == "⇧←→"));
        let line = |buf: &Buffer| -> String {
            (0..80)
                .map(|x| buf[(x, bar.y - 1)].symbol().to_owned())
                .collect()
        };
        assert!(line(&buf).contains("DELETE FROM"));

        let mut focus = crate::core::focus::Focus::default();
        let ring = FocusRing::default();
        let shift = Key {
            code: KeyCode::Right,
            mods: ratatui::crossterm::event::KeyModifiers::SHIFT,
        };
        assert_eq!(d.on_key(&shift, &mut focus, &ring), Outcome::Changed);
        assert_eq!(d.scroll.h.offset, HScroll::default().step);
        let (buf, _) = draw(&mut d);
        assert!(line(&buf).contains('…'), "cut on the left: {}", line(&buf));
        assert!(
            buf[(bar.x, bar.y)].symbol() != scrollbar::H_THUMB,
            "the thumb moved"
        );

        assert_eq!(d.wheel(Axis::Horizontal, 1).outcome(), Outcome::Changed);
        let (buf, _) = draw(&mut d);
        let grab = (bar.x..bar.right())
            .find(|&x| buf[(x, bar.y)].symbol() == scrollbar::H_THUMB)
            .unwrap();
        d.on_click(
            scrollbar::h_id_for(d.id),
            Position::new(grab, bar.y),
            &mut focus,
        );
        assert_eq!(
            d.on_drag(
                scrollbar::h_id_for(d.id),
                Position::new(bar.right() + 4, bar.y)
            ),
            Outcome::Changed
        );
        assert_eq!(d.scroll.h.offset, d.scroll.h.max_offset());
        let (buf, _) = draw(&mut d);
        assert!(
            line(&buf).contains("aa10');"),
            "the line's end: {}",
            line(&buf)
        );
    }

    fn render(d: &mut Dialog, screen: Rect) -> (Rect, Rect, Vec<Rect>) {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(screen);
        d.render(screen, &mut buf, &mut ctx);
        let ack = match &d.body {
            DialogBody::Facts { ack: Some(a), .. } => a.input.area,
            _ => unreachable!(),
        };
        (d.area, ack, d.actions.iter().map(|b| b.area).collect())
    }

    #[test]
    fn wrapped_facts_are_measured_so_the_acknowledgement_and_actions_stay_inside_the_frame() {
        let mut d = gate(78);
        let (area, ack, actions) = render(&mut d, Rect::new(0, 0, 120, 40));
        let inner = area.inner(ratatui::layout::Margin::new(1, 1));
        assert!(
            inner.contains(Position::new(ack.x, ack.bottom() - 1)),
            "the typed phrase field is inside the frame: frame {area:?}, field {ack:?}"
        );
        for b in &actions {
            assert!(
                inner.contains(Position::new(b.x, b.y)),
                "button {b:?} inside {area:?}"
            );
            assert!(
                b.y > ack.bottom() - 1,
                "actions sit below the field: {b:?} vs {ack:?}"
            );
        }

        let measured = d.height(78);
        assert_eq!(area.height, measured);
        let facts_rows = crate::widgets::props::measure(
            match &d.body {
                DialogBody::Facts { facts, .. } => facts,
                _ => unreachable!(),
            },
            72,
        );
        assert!(
            facts_rows > 5,
            "the wrapped facts take more rows than they have entries"
        );
    }

    #[test]
    fn backdrop_clicks_follow_the_dismiss_policy() {
        let id = WidgetId::of("d");
        let mut plain = Dialog::confirm(id, "Run?", "Run the suite.", "Run");
        plain.on_click_outside();
        assert_eq!(plain.result, Some(DialogResult::Action(0)));

        let mut gated = gate(78);
        assert_eq!(gated.on_click_outside(), Outcome::Consumed);
        assert_eq!(gated.result, None);

        let mut del =
            Dialog::destructive(id, "Delete?", "Gone for good.", "Delete").dismiss(Dismiss::ESCAPE);
        assert_eq!(del.on_click_outside(), Outcome::Consumed);
        assert_eq!(del.result, None);
        let mut locked =
            Dialog::confirm(id, "Run?", "Run the suite.", "Run").dismiss(Dismiss::NEVER);
        let mut focus = crate::core::focus::Focus::default();
        let ring = FocusRing::default();
        let esc = Key {
            code: KeyCode::Esc,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        locked.on_key(&esc, &mut focus, &ring);
        locked.on_click_outside();
        assert_eq!(locked.result, None);
    }

    #[test]
    fn a_destructive_cancel_refuses_light_dismissal_but_not_esc() {
        use crate::core::intent::DismissReason as R;
        use crate::ui::layer::Overlay;
        let id = WidgetId::of("d");
        let destructive_cancel = || {
            Dialog::confirm(id, "Credentials?", "The launch is paused.", "Retry").actions(
                vec![
                    Button::secondary(id.sub("retry"), "Retry"),
                    Button::danger(id.sub("cancel"), "Cancel launch"),
                ],
                Some(1),
            )
        };
        let mut d = destructive_cancel();
        assert_eq!(d.on_click_outside(), Outcome::Consumed);
        assert_eq!(d.result, None, "a stray click does not cancel the launch");
        let policy = Overlay::dismiss(&d);
        assert!(
            !policy.allows(R::OutsideClick),
            "the layer stack refuses it"
        );
        assert!(!policy.allows(R::FocusOut));
        assert!(policy.allows(R::Esc) && policy.allows(R::CloseButton));
        let mut focus = crate::core::focus::Focus::default();
        let ring = FocusRing::default();
        let esc = Key {
            code: KeyCode::Esc,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        d.on_key(&esc, &mut focus, &ring);
        assert_eq!(d.result, Some(DialogResult::Action(1)), "Esc still cancels");

        let mut light = destructive_cancel().guard_destructive_dismiss(false);
        light.on_click_outside();
        assert_eq!(light.result, Some(DialogResult::Action(1)));

        let mut del = Dialog::destructive(id, "Delete?", "Gone for good.", "Delete");
        del.on_click_outside();
        assert_eq!(del.result, Some(DialogResult::Action(0)));
    }

    #[test]
    fn the_gated_action_is_refused_until_the_token_matches_even_before_a_redraw() {
        let mut d = gate(78);
        assert_eq!(d.confirm_index, Some(1));

        assert_eq!(d.finish(DialogResult::Action(1)), Outcome::Consumed);
        assert_eq!(d.result, None);
        if let DialogBody::Facts { ack: Some(a), .. } = &mut d.body {
            let token = a.token.clone();
            a.input = a.input.clone().value(&token);
        }
        d.finish(DialogResult::Action(1));
        assert_eq!(d.result, Some(DialogResult::Action(1)));
    }

    #[test]
    fn a_short_screen_clips_the_facts_never_the_field_or_the_actions() {
        let mut d = gate(78);
        let (area, ack, actions) = render(&mut d, Rect::new(0, 0, 120, 16));
        let inner = area.inner(ratatui::layout::Margin::new(1, 1));
        assert!(area.height <= 14);
        assert!(
            inner.contains(Position::new(ack.x, ack.bottom() - 1)),
            "{ack:?} in {area:?}"
        );
        for b in &actions {
            assert!(inner.contains(Position::new(b.x, b.y)), "{b:?} in {area:?}");
        }
    }

    fn hint_text(hints: &[Hint]) -> String {
        hints
            .iter()
            .map(|h| format!("{} {}", h.key, h.action))
            .collect::<Vec<_>>()
            .join(" · ")
    }

    #[test]
    fn a_destructive_dialog_names_the_focused_cancel_button_not_confirm() {
        let d = Dialog::destructive(WidgetId::of("del"), "Delete?", "Gone for good.", "Delete");

        let text = hint_text(&d.hints(Some(d.initial_focus)));
        assert_eq!(text, "←→ Choose · Enter Cancel · Esc Cancel");
        assert!(!text.contains("Confirm"), "{text}");

        let text = hint_text(&d.hints(Some(d.actions[1].id)));
        assert!(text.contains("Enter Delete"), "{text}");
    }

    #[test]
    fn a_typed_acknowledgement_says_what_to_type_and_hides_the_gated_action() {
        let d = gate(78);
        let text = hint_text(&d.hints(Some(d.initial_focus)));
        assert_eq!(
            text,
            "Type \"TRASH 4 UNDER /Users/alex/work/ide ON mbp\" to confirm · Esc Cancel"
        );
        let next = gate(78).ack_enter(AckEnter::NextField);
        assert_eq!(
            hint_text(&next.hints(Some(next.initial_focus))),
            "Type \"TRASH 4 UNDER /Users/alex/work/ide ON mbp\" to confirm · Enter Next · Esc Cancel"
        );

        let text = hint_text(&d.hints(Some(d.actions[1].id)));
        assert_eq!(text, "Esc Cancel");
    }

    #[test]
    fn a_prompt_names_its_primary_action_while_editing() {
        let input = TextInput::new(WidgetId::of("name"), "Name");
        let mut d = Dialog::prompt(WidgetId::of("p"), "Rename", input, "Rename");
        let focused = Some(d.initial_focus);
        assert_eq!(
            hint_text(&d.hints(focused)),
            "Enter Edit · Tab Next · Esc Cancel"
        );
        if let DialogBody::Input(i) = &mut d.body {
            i.begin_edit();
        }
        assert_eq!(hint_text(&d.hints(focused)), "Enter Rename · Esc Cancel");
    }

    #[test]
    fn the_cancel_label_renames_the_button_and_the_hints() {
        let d = Dialog::destructive(
            WidgetId::of("c"),
            "Cancel the launch?",
            "The pipeline stops.",
            "Cancel launch",
        )
        .cancel_label("Keep launching");
        assert_eq!(d.actions[0].label, "Keep launching");
        assert_eq!(d.cancel_index, Some(0));
        assert_eq!(
            hint_text(&d.hints(Some(d.initial_focus))),
            "←→ Choose · Enter Keep launching · Esc Keep launching"
        );

        assert_eq!(
            hint_text(&d.hints(Some(d.actions[1].id))),
            "←→ Choose · Enter Cancel launch · Esc Keep launching"
        );
    }

    fn draw(d: &mut Dialog, screen: Rect, bounds: Rect) -> Buffer {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(screen);
        d.render_in(screen, bounds, &mut buf, &mut ctx);
        buf
    }

    #[test]
    fn chrome_rows_place_render_like_render_in_body_bounds() {
        let screen = Rect::new(0, 0, 60, 20);
        let mut a = Dialog::confirm(WidgetId::of("c"), "Quit?", "Leave now.", "Quit");
        let want = draw(&mut a, screen, body_bounds(screen, 1, 2));
        let mut b = Dialog::confirm(WidgetId::of("c"), "Quit?", "Leave now.", "Quit").chrome(1, 2);
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut got = Buffer::empty(screen);
        b.render(screen, &mut got, &mut ctx);
        assert_eq!(rows(&got), rows(&want));
        assert_eq!(a.area, b.area);

        let mut c = Dialog::confirm(WidgetId::of("c"), "Quit?", "Leave now.", "Quit");
        let whole = draw(&mut c, screen, screen);
        assert_ne!(rows(&whole), rows(&want));
    }

    fn rows(buf: &Buffer) -> Vec<String> {
        let a = buf.area;
        (a.y..a.bottom())
            .map(|y| {
                (a.x..a.right())
                    .map(|x| buf.cell((x, y)).map_or(" ", |c| c.symbol()))
                    .collect::<String>()
            })
            .collect()
    }

    fn key(code: KeyCode) -> Key {
        Key {
            code,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        }
    }

    fn help_text() -> String {
        (1..=30)
            .map(|i| format!("line {i:02}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn ack_dialog(enter: AckEnter) -> (Dialog, crate::core::focus::Focus, FocusRing) {
        let mut d = Dialog::facts(
            WidgetId::of("drop"),
            "Drop table",
            vec![Prop::new("Table", "public.orders")],
            vec![],
            Some("orders"),
            Button::danger(WidgetId::of("drop").sub("ok"), "Drop"),
        )
        .ack_enter(enter);
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let screen = Rect::new(0, 0, 100, 30);
        let mut buf = Buffer::empty(screen);
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        d.render(screen, &mut buf, &mut ctx);
        let mut focus = crate::core::focus::Focus::default();
        focus.focus(d.initial_focus);
        (d, focus, ring)
    }

    fn type_into(d: &mut Dialog, focus: &mut crate::core::focus::Focus, ring: &FocusRing, s: &str) {
        for c in s.chars() {
            d.on_key(&key(KeyCode::Char(c)), focus, ring);
        }
    }

    #[test]
    fn enter_in_a_matching_acknowledgement_presses_the_gated_action() {
        let (mut d, mut focus, ring) = ack_dialog(AckEnter::Submit);
        let ack = d.initial_focus;

        type_into(&mut d, &mut focus, &ring, "orders");
        assert_eq!(hint_text(&d.hints(Some(ack))), "Enter Drop · Esc Cancel");
        d.on_key(&key(KeyCode::Enter), &mut focus, &ring);
        assert_eq!(
            d.result,
            Some(DialogResult::Action(1)),
            "Enter on the matching text presses Drop, never Cancel"
        );
    }

    #[test]
    fn enter_in_a_mismatched_acknowledgement_keeps_editing_and_says_so() {
        let (mut d, mut focus, ring) = ack_dialog(AckEnter::Submit);
        let ack = d.initial_focus;

        type_into(&mut d, &mut focus, &ring, "order");
        d.on_key(&key(KeyCode::Enter), &mut focus, &ring);

        d.on_key(&key(KeyCode::Enter), &mut focus, &ring);
        assert_eq!(d.result, None);
        assert_eq!(focus.current(), Some(ack), "focus stays in the field");
        assert!(d.is_editing(), "the field keeps the caret");
        let DialogBody::Facts { ack: Some(a), .. } = &d.body else {
            unreachable!()
        };
        assert_eq!(a.input.field.error.as_deref(), Some("Does not match"));

        type_into(&mut d, &mut focus, &ring, "s");
        d.on_key(&key(KeyCode::Enter), &mut focus, &ring);
        assert_eq!(d.result, Some(DialogResult::Action(1)));
    }

    #[test]
    fn ack_enter_next_field_keeps_the_two_step_confirmation() {
        let (mut d, mut focus, ring) = ack_dialog(AckEnter::NextField);
        let ack = d.initial_focus;

        type_into(&mut d, &mut focus, &ring, "orders");
        d.on_key(&key(KeyCode::Enter), &mut focus, &ring);
        assert_eq!(d.result, None);
        assert_ne!(focus.current(), Some(ack), "Enter moved on to the buttons");
    }

    #[test]
    fn label_mnemonics_press_their_action_and_never_a_y_on_cancel() {
        let ring = FocusRing::default();
        let d = Dialog::destructive(WidgetId::of("del"), "Delete?", "Gone for good.", "Delete")
            .cancel_label("Keep");
        assert_eq!(d.mnemonic_keys(), vec![Some(('k', 0)), Some(('d', 0))]);

        let mut y = d.clone();
        let mut focus = crate::core::focus::Focus::default();
        focus.focus(y.initial_focus);
        y.on_key(&key(KeyCode::Char('y')), &mut focus, &ring);
        assert_eq!(y.result, None, "y with Keep focused does nothing");

        let mut del = d.clone();
        del.on_key(&key(KeyCode::Char('d')), &mut focus, &ring);
        assert_eq!(del.result, Some(DialogResult::Action(1)));
        let mut keep = d.clone();
        keep.on_key(&key(KeyCode::Char('K')), &mut focus, &ring);
        assert_eq!(keep.result, Some(DialogResult::Action(0)));

        let three = Dialog::confirm(WidgetId::of("x"), "Unsaved", "Save?", "Save").actions(
            vec![
                Button::subtle(WidgetId::of("c"), "Cancel"),
                Button::danger(WidgetId::of("d"), "Discard"),
                Button::primary(WidgetId::of("s"), "Save"),
            ],
            Some(0),
        );
        assert_eq!(
            three.mnemonic_keys(),
            vec![Some(('c', 0)), Some(('d', 0)), Some(('s', 0))]
        );
        let clash =
            Dialog::confirm(WidgetId::of("x"), "Stop?", "Stop it?", "Stop").cancel_label("Stay");
        assert_eq!(clash.mnemonic_keys(), vec![Some(('s', 0)), Some(('t', 1))]);

        let mut off = d.clone().mnemonics(false);
        off.on_key(&key(KeyCode::Char('d')), &mut focus, &ring);
        assert_eq!(off.result, None);
    }

    #[test]
    fn mnemonics_never_fire_while_a_text_field_takes_letters() {
        let (mut d, mut focus, ring) = ack_dialog(AckEnter::Submit);
        d.on_key(&key(KeyCode::Enter), &mut focus, &ring);

        type_into(&mut d, &mut focus, &ring, "c");
        assert_eq!(d.result, None);
        let DialogBody::Facts { ack: Some(a), .. } = &d.body else {
            unreachable!()
        };
        assert_eq!(a.input.text(), "c");
    }

    #[test]
    fn the_mnemonic_is_underlined_in_its_button() {
        let mut d = Dialog::destructive(WidgetId::of("del"), "Delete?", "Gone for good.", "Delete")
            .cancel_label("Keep");
        let screen = Rect::new(0, 0, 80, 20);
        let buf = draw(&mut d, screen, screen);
        let b = d.actions[1].area;
        let cell = buf.cell((b.x + 1, b.y)).unwrap();
        assert_eq!(cell.symbol(), "D");
        assert!(cell.modifier.contains(ratatui::style::Modifier::UNDERLINED));
        let next = buf.cell((b.x + 2, b.y)).unwrap();
        assert!(!next.modifier.contains(ratatui::style::Modifier::UNDERLINED));

        let mut p = Dialog::prompt(
            WidgetId::of("p"),
            "Rename",
            TextInput::new(WidgetId::of("n"), "Name"),
            "Rename",
        );
        let buf = draw(&mut p, screen, screen);
        let b = p.actions[1].area;
        let cell = buf.cell((b.x + 1, b.y)).unwrap();
        assert!(!cell.modifier.contains(ratatui::style::Modifier::UNDERLINED));
    }

    #[test]
    fn an_edited_prompt_ignores_a_backdrop_click() {
        let input = TextInput::new(WidgetId::of("name"), "Name").value("draft");
        let mut d = Dialog::prompt(WidgetId::of("p"), "Rename", input, "Rename");

        let mut clean = d.clone();
        clean.on_click_outside();
        assert_eq!(clean.result, Some(DialogResult::Action(0)));

        d.on_paste(" v2");
        assert!(d.prompt_dirty());
        assert_eq!(d.on_click_outside(), Outcome::Consumed);
        assert_eq!(d.result, None);
        let mut off = d.clone().guard_dirty_prompt(false);
        off.on_click_outside();
        assert_eq!(off.result, Some(DialogResult::Action(0)));
    }

    #[test]
    fn letter_accelerators_follow_the_dialog_config() {
        let mut focus = crate::core::focus::Focus::default();
        let ring = FocusRing::default();

        let mut d = Dialog::confirm(WidgetId::of("q"), "Quit?", "Leave now?", "Quit");
        d.on_key(&key(KeyCode::Char('y')), &mut focus, &ring);
        assert_eq!(d.result, None);
        d.on_key(&key(KeyCode::Char('q')), &mut focus, &ring);
        assert_eq!(d.result, Some(DialogResult::Action(1)));
        let mut yes = Dialog::confirm(WidgetId::of("q"), "Quit?", "Leave now?", "Quit")
            .accelerators(Some(('y', 'n')));
        assert_eq!(
            hint_text(&yes.hints(Some(yes.initial_focus))),
            "←→ Choose · Enter Quit · Esc Cancel · y Quit · n Cancel"
        );
        yes.on_key(&key(KeyCode::Char('y')), &mut focus, &ring);
        assert_eq!(yes.result, Some(DialogResult::Action(1)));
        let mut off = Dialog::confirm(WidgetId::of("q"), "Quit?", "Leave now?", "Quit")
            .accelerators(None)
            .width(40);
        off.on_key(&key(KeyCode::Char('y')), &mut focus, &ring);
        assert_eq!(off.result, None, "letters are inert without accelerators");
        assert_eq!(off.width, 40);
        let mut custom = Dialog::confirm(WidgetId::of("q"), "Quit?", "Leave now?", "Quit")
            .accelerators(Some(('o', 'a')));
        custom.on_key(&key(KeyCode::Char('o')), &mut focus, &ring);
        assert!(matches!(custom.result, Some(DialogResult::Action(_))));
    }

    #[test]
    fn a_text_body_taller_than_the_screen_scrolls_to_its_last_line() {
        let screen = Rect::new(0, 0, 72, 20);
        let mut d = Dialog::confirm(WidgetId::of("help"), "Keys", &help_text(), "Close");
        let buf = draw(&mut d, screen, screen);
        let text = rows(&buf).join("\n");
        assert!(text.contains("line 01"), "{text}");
        assert!(!text.contains("line 30"), "{text}");

        assert!(text.contains("more"), "a \"↓ N more\" marker: {text}");
        assert!(text.contains(scrollbar::THUMB), "{text}");
        assert!(
            hint_text(&d.hints(Some(d.initial_focus))).contains("↑↓ Scroll"),
            "the hints say the body scrolls"
        );

        let mut focus = crate::core::focus::Focus::default();
        let ring = FocusRing::default();
        d.on_key(&key(KeyCode::End), &mut focus, &ring);
        let text = rows(&draw(&mut d, screen, screen)).join("\n");
        assert!(text.contains("line 30"), "{text}");
        assert!(!text.contains("more"), "{text}");

        for _ in 0..40 {
            d.wheel(Axis::Vertical, -100).outcome();
        }
        let text = rows(&draw(&mut d, screen, screen)).join("\n");
        assert!(text.contains("line 01"), "{text}");
    }

    #[test]
    fn body_scroll_off_cuts_the_body_as_before() {
        let screen = Rect::new(0, 0, 72, 20);
        let mut d = Dialog::confirm(WidgetId::of("help"), "Keys", &help_text(), "Close")
            .body_scroll(false)
            .max_height(Some(12));
        let mut focus = crate::core::focus::Focus::default();
        let ring = FocusRing::default();
        draw(&mut d, screen, screen);
        assert_eq!(d.area.height, 12);
        d.on_key(&key(KeyCode::End), &mut focus, &ring);
        let text = rows(&draw(&mut d, screen, screen)).join("\n");
        assert!(text.contains("line 01"), "{text}");
    }

    #[test]
    fn every_fact_of_a_short_gate_is_reachable_and_the_typed_phrase_is_whole() {
        let screen = Rect::new(0, 0, 72, 20);
        let mut d = gate(66);
        let text = rows(&draw(&mut d, screen, screen)).join("\n");

        assert!(text.contains("ON mbp to confirm"), "{text}");
        assert!(!text.contains("confi…"), "{text}");
        assert!(text.contains("more"), "{text}");
        let mut focus = crate::core::focus::Focus::default();
        focus.focus(d.actions[0].id);
        let ring = FocusRing::default();
        let mut seen = text;
        for _ in 0..20 {
            d.on_key(&key(KeyCode::Down), &mut focus, &ring);
            seen.push_str(&rows(&draw(&mut d, screen, screen)).join("\n"));
        }
        for label in ["Action", "Mode", "Estimate", "Threat model", "Paths"] {
            assert!(seen.contains(label), "{label} is reachable: {seen}");
        }

        let ack = match &d.body {
            DialogBody::Facts { ack: Some(a), .. } => a.input.area,
            _ => unreachable!(),
        };
        let actions_y = d.actions[0].area.y;
        assert_eq!(
            actions_y,
            ack.y + 2,
            "one blank row between the field and the actions"
        );
    }

    #[test]
    fn an_anchored_dialog_opens_under_the_control_that_asked() {
        let screen = Rect::new(0, 0, 100, 30);
        let button = Rect::new(10, 4, 12, 1);
        let mut d = Dialog::confirm(WidgetId::of("d"), "Discard?", "Unsaved edits.", "Discard")
            .anchor(button, Placement::Below);
        draw(&mut d, screen, screen);
        assert_eq!((d.area.x, d.area.y), (10, 5), "{:?}", d.area);
        let mut c = Dialog::confirm(WidgetId::of("d"), "Discard?", "Unsaved edits.", "Discard");
        draw(&mut c, screen, screen);
        assert_eq!(
            c.area.x,
            (100 - c.area.width).div_ceil(2),
            "centred by default"
        );
    }

    #[test]
    fn a_dialog_stays_inside_its_bounds_with_a_margin() {
        let screen = Rect::new(0, 0, 72, 20);
        let bounds = body_bounds(screen, 1, 2);
        let mut d = gate(78);
        draw(&mut d, screen, bounds);
        let a = d.area;
        assert!(a.y > bounds.y, "{a:?} below the header in {bounds:?}");
        assert!(
            a.bottom() < bounds.bottom(),
            "{a:?} above the status in {bounds:?}"
        );
        assert!(a.x >= bounds.x + OUTER_MARGIN.horizontal, "{a:?}");
        assert!(
            a.right() + OUTER_MARGIN.horizontal <= bounds.right(),
            "{a:?}"
        );

        let mut d = gate(78).outer_margin(Margin::new(0, 0));
        draw(&mut d, screen, bounds);
        assert_eq!(d.area.width, 72);
    }

    #[test]
    fn the_padding_comes_from_the_theme_dialog_insets() {
        let screen = Rect::new(0, 0, 80, 24);
        let mut theme = Theme::junie();
        theme.spacing.dialog = crate::theme::Insets::new(4, 2);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(screen);
        let mut d = Dialog::confirm(WidgetId::of("c"), "Run?", "Run the suite.", "Run");
        d.render_in(screen, screen, &mut buf, &mut ctx);
        let a = d.area;
        let title_at = (a.x..a.right())
            .find(|&x| buf.cell((x, a.y + 3)).map_or("", |c| c.symbol()) == "R")
            .unwrap();
        assert_eq!(title_at, a.x + 5, "border + 4 columns of padding");
        assert_eq!(a.height, d.height(d.width), "the height counts the padding");
    }

    #[test]
    fn a_narrow_dialog_wraps_its_actions_and_never_cuts_the_primary() {
        let screen = Rect::new(0, 0, 30, 20);
        let mut d = Dialog::destructive(
            WidgetId::of("del"),
            "Delete?",
            "Gone for good.",
            "Delete all",
        )
        .cancel_label("Keep files");
        let text = rows(&draw(&mut d, screen, screen)).join("\n");
        assert!(text.contains("Delete all"), "{text}");
        assert!(text.contains("Keep files"), "{text}");
        let (keep, del) = (d.actions[0].area, d.actions[1].area);
        assert_ne!(keep.y, del.y, "two lines: {keep:?} {del:?}");
        assert!(del.bottom() < d.area.bottom(), "inside the frame");
    }

    #[test]
    fn a_prompt_field_label_lines_up_with_the_title() {
        let screen = Rect::new(0, 0, 80, 24);
        let input = TextInput::new(WidgetId::of("name"), "Task name").required(true);
        let mut d = Dialog::prompt(WidgetId::of("p"), "Rename task", input, "Rename");
        let r = rows(&draw(&mut d, screen, screen));
        let col = |needle: &str| {
            r.iter()
                .find_map(|l| l.find(needle).map(|b| l[..b].chars().count()))
                .unwrap()
        };
        assert_eq!(col("Rename task"), col("Task name"));
    }
}

#[cfg(test)]
mod track_click_tests {
    use super::*;
    use crate::core::focus::{Focus, FocusRing};
    use crate::core::hit::HitRegistry;
    use crate::core::scroll::TrackClick;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;

    fn pressed_at_the_bottom(click: Option<TrackClick>, app: TrackClick) -> usize {
        let text: Vec<String> = (1..=40).map(|i| format!("line {i:02}")).collect();
        let mut d = Dialog::confirm(WidgetId::of("help"), "Keys", &text.join("\n"), "Close");
        d.scroll.track_click = click;
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let screen = Rect::new(0, 0, 72, 20);
        let mut buf = Buffer::empty(screen);
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        ctx.track_click = app;
        d.render_in(screen, screen, &mut buf, &mut ctx);
        assert!(!d.scroll.v_track().is_empty(), "the body scrolls");
        let bottom = Position::new(d.scroll.v_track().x, d.scroll.v_track().bottom() - 1);
        let mut focus = Focus::default();
        assert_eq!(
            d.on_click(scrollbar::id_for(d.id), bottom, &mut focus),
            Outcome::Changed
        );
        d.scroll.offset
    }

    #[test]
    fn a_dialog_body_track_click_can_page() {
        let page = pressed_at_the_bottom(Some(TrackClick::Page), TrackClick::Jump);
        assert!(page > 0, "one page down");
        assert_eq!(
            pressed_at_the_bottom(None, TrackClick::Page),
            page,
            "app-wide"
        );
        assert!(
            pressed_at_the_bottom(None, TrackClick::Jump) > page,
            "default jumps"
        );
    }
}
