//! Modal dialogs. A dialog owns its own focus scope; the app draws it over a
//! dimmed backdrop and routes every event to it while it is open.
//!
//! Closing without an action is a [`Dismiss`] policy on the dialog, never a
//! decision of the app. A dismissal resolves to the cancel action when there
//! is one, so by default Esc and a click on the dimmed backdrop both close
//! (cancelling is the safe answer, destructive dialogs included). Dialogs
//! with a typed acknowledgement close on Esc only: a stray click must not
//! throw away what was typed.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Margin, Position, Rect};

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::scroll::{HScroll, ScrollState, ScrollbarVisibility};
use crate::theme::ButtonKind;
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::popup::Dismiss;
use crate::widgets::button::{Button, ButtonRow, RowAlign, RowOverflow};
use crate::widgets::input::{InputEvent, TextInput};
use crate::widgets::keyhint::{Hint, hint};
use crate::widgets::panel::Panel;
use crate::widgets::scrollbar;

#[derive(Debug, Clone)]
pub enum DialogBody {
    Text(String),
    Input(TextInput),
    /// Label/value facts, an optional preformatted block (SQL), and an
    /// optional typed acknowledgement that arms the confirming action.
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
    /// Index into `actions`.
    Action(usize),
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct Dialog {
    pub id: WidgetId,
    pub title: String,
    pub body: DialogBody,
    pub actions: Vec<Button>,
    /// Index of the action Esc maps to (usually a Cancel button).
    pub cancel_index: Option<usize>,
    pub width: u16,
    /// Letter accelerators on a text dialog: `(yes, no)` press the primary
    /// action and the cancel action. Default `Some(('y', 'n'))`; `None`
    /// keeps letters inert, as macOS alerts do.
    pub accelerators: Option<(char, char)>,
    pub area: Rect,
    pub result: Option<DialogResult>,
    pub initial_focus: WidgetId,
    /// How the dialog closes without an action (Esc, backdrop click).
    pub dismiss: Dismiss,
    /// The action a typed acknowledgement arms: disabled, and refused even
    /// if triggered, until the token matches. `None`: nothing is gated.
    pub confirm_index: Option<usize>,
    /// A body taller than the room scrolls (wheel, ↑↓, PgUp/PgDn,
    /// Home/End, the scrollbar) and says how much is below. Default on.
    /// Off: the body is cut at the bottom, as before.
    pub body_scroll: bool,
    /// Cap on the dialog's height in rows; the body scrolls past it.
    /// `None` (default): as tall as the content, up to the bounds.
    pub max_height: Option<u16>,
    /// Space kept free between the dialog and the edges of the area it is
    /// placed in, so its frame never touches a panel border underneath.
    /// Default 3 columns, 1 row.
    pub outer_margin: Margin,
    /// How the actions are laid out. Default: packed right, the theme's
    /// button gap, and a row that wraps onto a second line rather than cut
    /// or hide an action ([`RowOverflow::Wrap`]).
    pub button_row: ButtonRow,
    /// The theme's button gap, captured at render so the natural height
    /// can be measured without a theme.
    gap: u16,
    /// The theme's dialog padding (`spacing.dialog`), captured at render
    /// like `gap`.
    pad: crate::theme::Insets,
    /// Body scroll position (Text and Facts bodies).
    pub scroll: ScrollState,
    /// The scrollbar track drawn last frame (empty when the body fits).
    track: Rect,
    /// Horizontal scrolling of the code preview; `None` follows
    /// [`RenderCtx::hscroll`]. Preformatted lines are never wrapped, so a
    /// line wider than the dialog scrolls sideways (Shift+←/→, the
    /// sideways wheel, the bar under the preview).
    pub horizontal: Option<HScroll>,
    /// Horizontal position of the code preview, in cells.
    pub code_scroll: ScrollState,
    /// The policy resolved at the last render (sizing and events use it).
    h_cfg: HScroll,
    /// The code preview's horizontal track drawn last frame.
    htrack: Rect,
}

/// Default [`Dialog::outer_margin`].
pub const OUTER_MARGIN: Margin = Margin {
    horizontal: 3,
    vertical: 1,
};

impl Dialog {
    /// Width in cells before clamping to the screen.
    pub fn width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    /// Letter accelerators `(yes, no)` on a text dialog; `None` turns them
    /// off.
    pub fn accelerators(mut self, keys: Option<(char, char)>) -> Self {
        self.accelerators = keys;
        self
    }

    pub fn confirm(id: WidgetId, title: &str, text: &str, confirm: &str) -> Self {
        let cancel = Button::subtle(id.sub("cancel"), "Cancel");
        let ok = Button::primary(id.sub("ok"), confirm);
        let ok_id = ok.id;
        Self {
            id,
            title: title.to_owned(),
            body: DialogBody::Text(text.to_owned()),
            actions: vec![cancel, ok],
            cancel_index: Some(0),
            width: 54,
            accelerators: Some(('y', 'n')),
            area: Rect::ZERO,
            result: None,
            initial_focus: ok_id,
            dismiss: Dismiss::LIGHT,
            confirm_index: None,
            body_scroll: true,
            max_height: None,
            outer_margin: OUTER_MARGIN,
            button_row: ButtonRow::new()
                .align(RowAlign::End)
                .overflow(RowOverflow::Wrap),
            gap: crate::theme::Spacing::DEFAULT.button_gap,
            pad: crate::theme::Spacing::DEFAULT.dialog,
            scroll: ScrollState::default(),
            horizontal: None,
            code_scroll: ScrollState::default(),
            h_cfg: HScroll::default(),
            htrack: Rect::ZERO,
            track: Rect::ZERO,
        }
    }

    pub fn destructive(id: WidgetId, title: &str, text: &str, confirm: &str) -> Self {
        let cancel = Button::secondary(id.sub("cancel"), "Cancel");
        let cancel_id = cancel.id;
        let del = Button::danger(id.sub("ok"), confirm);
        Self {
            id,
            title: title.to_owned(),
            body: DialogBody::Text(text.to_owned()),
            actions: vec![cancel, del],
            cancel_index: Some(0),
            width: 54,
            accelerators: Some(('y', 'n')),
            area: Rect::ZERO,
            result: None,
            initial_focus: cancel_id,
            dismiss: Dismiss::LIGHT,
            confirm_index: None,
            body_scroll: true,
            max_height: None,
            outer_margin: OUTER_MARGIN,
            button_row: ButtonRow::new()
                .align(RowAlign::End)
                .overflow(RowOverflow::Wrap),
            gap: crate::theme::Spacing::DEFAULT.button_gap,
            pad: crate::theme::Spacing::DEFAULT.dialog,
            scroll: ScrollState::default(),
            horizontal: None,
            code_scroll: ScrollState::default(),
            h_cfg: HScroll::default(),
            htrack: Rect::ZERO,
            track: Rect::ZERO,
        }
    }

    pub fn prompt(id: WidgetId, title: &str, input: TextInput, confirm: &str) -> Self {
        let cancel = Button::subtle(id.sub("cancel"), "Cancel");
        let ok = Button::primary(id.sub("ok"), confirm);
        let input_id = input.id;
        Self {
            id,
            title: title.to_owned(),
            body: DialogBody::Input(input),
            actions: vec![cancel, ok],
            cancel_index: Some(0),
            width: 54,
            accelerators: Some(('y', 'n')),
            area: Rect::ZERO,
            result: None,
            initial_focus: input_id,
            dismiss: Dismiss::LIGHT,
            confirm_index: None,
            body_scroll: true,
            max_height: None,
            outer_margin: OUTER_MARGIN,
            button_row: ButtonRow::new()
                .align(RowAlign::End)
                .overflow(RowOverflow::Wrap),
            gap: crate::theme::Spacing::DEFAULT.button_gap,
            pad: crate::theme::Spacing::DEFAULT.dialog,
            scroll: ScrollState::default(),
            horizontal: None,
            code_scroll: ScrollState::default(),
            h_cfg: HScroll::default(),
            htrack: Rect::ZERO,
            track: Rect::ZERO,
        }
    }

    /// Safety-style dialog: facts, code preview, optional typed token.
    pub fn facts(
        id: WidgetId,
        title: &str,
        facts: Vec<crate::widgets::props::Prop>,
        code: Vec<String>,
        token: Option<&str>,
        confirm: Button,
    ) -> Self {
        let cancel = Button::secondary(id.sub("cancel"), "Cancel");
        let cancel_id = cancel.id;
        let ack = token.map(|tok| AckInput {
            // the label is the instruction: it wraps, it is never cut
            input: TextInput::new(id.sub("ack"), &format!("Type {tok} to confirm"))
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
            accelerators: Some(('y', 'n')),
            area: Rect::ZERO,
            result: None,
            initial_focus: initial,
            dismiss: if token.is_some() {
                Dismiss::ESCAPE
            } else {
                Dismiss::LIGHT
            },
            confirm_index: token.map(|_| 1),
            body_scroll: true,
            max_height: None,
            outer_margin: OUTER_MARGIN,
            button_row: ButtonRow::new()
                .align(RowAlign::End)
                .overflow(RowOverflow::Wrap),
            gap: crate::theme::Spacing::DEFAULT.button_gap,
            pad: crate::theme::Spacing::DEFAULT.dialog,
            scroll: ScrollState::default(),
            horizontal: None,
            code_scroll: ScrollState::default(),
            h_cfg: HScroll::default(),
            htrack: Rect::ZERO,
            track: Rect::ZERO,
        }
    }

    /// True when no acknowledgement is required or its token matches.
    pub fn armed(&self) -> bool {
        match &self.body {
            DialogBody::Facts { ack: Some(a), .. } => a.input.text().trim() == a.token,
            _ => true,
        }
    }

    /// The footer hints for this dialog, built from its state so they can
    /// never disagree with what the keys do: Enter names the focused
    /// button, y and n name the real primary and cancel actions, Esc names
    /// what a dismissal resolves to, and a typed acknowledgement says what
    /// to type. `focused` is the widget that has keyboard focus.
    pub fn hints(&self, focused: Option<WidgetId>) -> Vec<Hint> {
        let cancel_label = self
            .cancel_index
            .and_then(|i| self.actions.get(i))
            .map(|b| b.label.clone());
        let dismiss_hint = || {
            self.dismiss.escape.then(|| {
                Hint::new(
                    "Esc",
                    cancel_label.clone().unwrap_or_else(|| "Close".into()),
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
                // Enter only moves on to the buttons; the gated one stays
                // disabled until the token is typed
                v.push(Hint::new("Type", format!("\"{}\" to confirm", a.token)));
                v.push(hint("Enter", "Next"));
            } else if inp.editing {
                let primary = self
                    .actions
                    .iter()
                    .find(|b| b.kind == ButtonKind::Primary)
                    .map(|b| b.label.clone());
                v.push(Hint::new(
                    "Enter",
                    primary.unwrap_or_else(|| "Apply".into()),
                ));
            } else {
                v.push(hint("Enter", "Edit"));
                v.push(hint("Tab", "Next"));
            }
            if inp.editing {
                // Esc reverts the field first; the dialog stays open
                v.push(hint("Esc", "Cancel"));
            } else {
                v.extend(dismiss_hint());
            }
            return v;
        }
        // the gated action is disabled until armed, even before a redraw
        let usable =
            |i: usize, b: &Button| !b.disabled && (Some(i) != self.confirm_index || self.armed());
        let enabled = self
            .actions
            .iter()
            .enumerate()
            .filter(|(i, b)| usable(*i, b))
            .count();
        if enabled > 1 {
            v.push(hint("←→", "Choose"));
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
            v.push(hint("↑↓", "Scroll"));
        }
        if self.h_cfg.enabled && self.code_scroll.overflows() {
            v.push(hint("⇧←→", "Scroll code"));
        }
        v.extend(dismiss_hint());
        // the quick answers exist for plain text dialogs only (see on_key)
        if matches!(self.body, DialogBody::Text(_))
            && let Some(p) = self
                .actions
                .iter()
                .find(|b| matches!(b.kind, ButtonKind::Primary | ButtonKind::Danger) && !b.disabled)
        {
            v.push(Hint::new("y", p.label.clone()));
            if let Some(c) = cancel_label {
                v.push(Hint::new("n", c));
            }
        }
        v
    }

    /// Rename the cancel (dismiss) action. `Cancel` is the default, but it
    /// is ambiguous next to an action that itself cancels or stops
    /// something: `Keep launching   Cancel launch` reads unambiguously where
    /// `Cancel   Cancel launch` does not. The footer hints follow the label.
    pub fn cancel_label(mut self, label: &str) -> Self {
        if let Some(b) = self.cancel_index.and_then(|i| self.actions.get_mut(i)) {
            b.label = label.to_owned();
        }
        self
    }

    /// Set how the dialog closes without an action.
    pub fn dismiss(mut self, dismiss: Dismiss) -> Self {
        self.dismiss = dismiss;
        self
    }

    /// The result a dismissal resolves to: the cancel action if there is
    /// one, so the owner handles "closed" and "Cancel" the same way.
    fn dismissed(&mut self) -> Outcome {
        match self.cancel_index {
            Some(ci) => self.finish(DialogResult::Action(ci)),
            None => self.finish(DialogResult::Cancelled),
        }
    }

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

    /// Rows the code preview takes below the facts: at most six lines,
    /// the blank row above them and, when a line is wider than the text
    /// column `inner_w`, the horizontal bar under them.
    fn code_rows(code: &[String], inner_w: u16, h_cfg: &HScroll) -> u16 {
        if code.is_empty() {
            0
        } else {
            let bar = h_cfg.shows_bar(&Self::code_extent(code, inner_w));
            code.len().min(6) as u16 + 1 + u16::from(bar)
        }
    }

    /// The code preview's horizontal extent: its widest shown line
    /// against the text column.
    fn code_extent(code: &[String], inner_w: u16) -> ScrollState {
        let widest = code
            .iter()
            .take(6)
            .map(|l| crate::ui::text::width(l))
            .max()
            .unwrap_or(0);
        let mut s = ScrollState::new(widest);
        s.set_viewport(inner_w as usize);
        s
    }

    /// Scroll the code preview sideways by `delta` steps.
    fn scroll_code(&mut self, delta: isize) -> Outcome {
        if !self.h_cfg.enabled || !self.code_scroll.overflows() {
            return Outcome::Ignored;
        }
        let step = self.h_cfg.step.max(1) as isize;
        if self.code_scroll.scroll_by(delta * step) {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    /// The sideways wheel: scrolls a code preview wider than the dialog.
    /// Consumed either way (nothing under a modal scrolls).
    pub fn on_wheel_h(&mut self, delta: i32) -> Outcome {
        match self.scroll_code(delta.signum() as isize) {
            Outcome::Ignored => Outcome::Consumed,
            o => o,
        }
    }

    /// The pointer moved while held on one of the dialog's scrollbars.
    pub fn on_drag(&mut self, pressed: WidgetId, pos: Position) -> Outcome {
        let moved = if pressed == scrollbar::h_id_for(self.id) && !self.htrack.is_empty() {
            scrollbar::drag_h(self.htrack, pos, &mut self.code_scroll)
        } else if pressed == scrollbar::id_for(self.id) && !self.track.is_empty() {
            scrollbar::drag(self.track, pos, &mut self.scroll)
        } else {
            return Outcome::Consumed;
        };
        if moved {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    /// Rows the typed acknowledgement takes in a dialog `width` wide: the
    /// field with its whole (wrapped) label, and the blank row above it.
    fn ack_rows(&self, ack: Option<&AckInput>, width: u16) -> u16 {
        ack.map_or(0, |a| a.input.height_for(self.field_width(width)) + 1)
    }

    /// Width of an input field: the text column plus the focus gutter to
    /// its left, so the field's label lines up with the title.
    fn field_width(&self, width: u16) -> u16 {
        self.inner_width(width) + TextInput::GUTTER
    }

    /// Text column of a dialog `width` wide: inside the border and the
    /// theme's dialog padding (`spacing.dialog.x`) on each side.
    fn inner_width(&self, width: u16) -> u16 {
        width.saturating_sub(2 * (1 + self.pad.x))
    }

    /// Rows the body's scrolling part needs at `width`: every wrapped line
    /// of a Text body, every fact row of a Facts body. Zero for a prompt.
    fn scroll_rows(&self, width: u16) -> u16 {
        let inner_w = self.inner_width(width);
        match &self.body {
            DialogBody::Text(t) => crate::ui::text::wrap(t, inner_w as usize).len() as u16,
            DialogBody::Input(_) => 0,
            DialogBody::Facts { facts, .. } => crate::widgets::props::measure(facts, inner_w),
        }
    }

    /// Rows the body takes that never scroll: the prompt field, the code
    /// preview and the typed acknowledgement.
    fn fixed_rows(&self, width: u16) -> u16 {
        match &self.body {
            DialogBody::Text(_) => 0,
            DialogBody::Input(i) => i.height_for(self.field_width(width)),
            DialogBody::Facts { code, ack, .. } => {
                Self::code_rows(code, self.inner_width(width), &self.h_cfg)
                    + self.ack_rows(ack.as_ref(), width)
            }
        }
    }

    /// Rows around the body and the actions: border(2) + the padding
    /// above and below (`spacing.dialog.y` each) + title(1) + gap(1) +
    /// gap(1).
    fn chrome_rows(&self) -> u16 {
        5 + 2 * self.pad.y
    }

    /// Lines the actions take at `width` (more than one only when they
    /// wrap).
    fn action_rows(&self, width: u16) -> u16 {
        self.button_row
            .rows_at_gap(self.inner_width(width), &self.actions, self.gap)
            .max(1)
    }

    /// Natural height at `width`: every row of the body shown.
    pub fn height(&self, width: u16) -> u16 {
        self.chrome_rows()
            + self.action_rows(width)
            + self.scroll_rows(width)
            + self.fixed_rows(width)
    }

    /// See [`Self::button_row`].
    pub fn button_row(mut self, row: ButtonRow) -> Self {
        self.button_row = row;
        self
    }

    /// See [`Self::body_scroll`].
    pub fn body_scroll(mut self, on: bool) -> Self {
        self.body_scroll = on;
        self
    }

    /// See [`Self::max_height`].
    pub fn max_height(mut self, rows: Option<u16>) -> Self {
        self.max_height = rows;
        self
    }

    /// See [`Self::outer_margin`].
    pub fn outer_margin(mut self, margin: Margin) -> Self {
        self.outer_margin = margin;
        self
    }

    /// Scroll the body by `delta` rows (the wheel). Consumed either way:
    /// nothing under a modal scrolls.
    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        if self.scroll.scroll_by(delta as isize) {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    pub fn confirm_index(mut self, index: Option<usize>) -> Self {
        self.confirm_index = index;
        self
    }

    fn finish(&mut self, r: DialogResult) -> Outcome {
        // the gated action is checked here, not only drawn disabled: a key
        // or click between an edit and the next frame cannot slip through
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
        // input editing captures first
        let is_facts = matches!(self.body, DialogBody::Facts { .. });
        if let Some(inp) = self.input_mut()
            && cur == Some(inp.id)
        {
            let (o, ev) = inp.on_key(key);
            match ev {
                Some(InputEvent::CommittedTab { backward }) => {
                    if backward {
                        focus.prev(ring);
                    } else {
                        focus.next(ring);
                    }
                    return Outcome::Changed;
                }
                Some(InputEvent::Committed) => {
                    // Enter submits a prompt; for a typed acknowledgement it only
                    // moves on, so the confirming button is reached deliberately
                    if is_facts {
                        focus.next(ring);
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
        match key.code {
            KeyCode::Esc if self.dismiss.escape => self.dismissed(),
            KeyCode::Tab => {
                focus.next(ring);
                Outcome::Changed
            }
            KeyCode::BackTab => {
                focus.prev(ring);
                Outcome::Changed
            }
            // a body taller than the dialog scrolls under the keyboard too
            KeyCode::Up if self.scroll.overflows() => self.on_wheel(-1),
            KeyCode::Down if self.scroll.overflows() => self.on_wheel(1),
            KeyCode::PageUp if self.scroll.overflows() => {
                self.scroll.page_up();
                Outcome::Changed
            }
            KeyCode::PageDown if self.scroll.overflows() => {
                self.scroll.page_down();
                Outcome::Changed
            }
            KeyCode::Home if self.scroll.overflows() => {
                self.scroll.jump_start();
                Outcome::Changed
            }
            KeyCode::End if self.scroll.overflows() => {
                self.scroll.jump_end();
                Outcome::Changed
            }
            // Shift+←/→ scroll a code preview wider than the dialog; the
            // plain arrows keep moving between the actions
            KeyCode::Left if key.shift() && self.code_scroll.overflows() => self.scroll_code(-1),
            KeyCode::Right if key.shift() && self.code_scroll.overflows() => self.scroll_code(1),
            KeyCode::Left | KeyCode::Char('h') => {
                // move between actions
                if let Some(i) = self.actions.iter().position(|b| Some(b.id) == cur) {
                    let prev = self.actions[..i].iter().rev().find(|b| !b.disabled);
                    if let Some(p) = prev {
                        focus.focus(p.id);
                    }
                }
                Outcome::Changed
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if let Some(i) = self.actions.iter().position(|b| Some(b.id) == cur) {
                    let next = self.actions[i + 1..].iter().find(|b| !b.disabled);
                    if let Some(n) = next {
                        focus.focus(n.id);
                    }
                }
                Outcome::Changed
            }
            KeyCode::Char(c)
                if self.accelerators.is_some_and(|(yes, _)| yes == c)
                    && matches!(self.body, DialogBody::Text(_)) =>
            {
                let primary = self.actions.iter().position(|b| {
                    matches!(b.kind, ButtonKind::Primary | ButtonKind::Danger) && !b.disabled
                });
                match primary {
                    Some(p) => self.finish(DialogResult::Action(p)),
                    None => Outcome::Consumed,
                }
            }
            KeyCode::Char(c)
                if self.accelerators.is_some_and(|(_, no)| no == c)
                    && matches!(self.body, DialogBody::Text(_)) =>
            {
                match self.cancel_index {
                    Some(ci) => self.finish(DialogResult::Action(ci)),
                    None => Outcome::Consumed,
                }
            }
            _ => Outcome::Consumed,
        }
    }

    /// Focus moved off one of the dialog's controls (see
    /// [`crate::core::focus::Focus::take_changes`]): its field ends the
    /// edit per its `blur_action`. Leaving the field never submits the
    /// dialog; only Enter or a button does.
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

    /// A completed click on `id`.
    pub fn on_click(
        &mut self,
        id: WidgetId,
        pos: Position,
        focus: &mut crate::core::focus::Focus,
    ) -> Outcome {
        if let Some(inp) = self.input_mut()
            && inp.id == id
        {
            focus.focus(id);
            return inp.on_click(pos);
        }
        if id == scrollbar::h_id_for(self.id) && !self.htrack.is_empty() {
            let click = self.h_cfg.track_click;
            let moved = scrollbar::press_h(self.htrack, pos, &mut self.code_scroll, click);
            return if moved {
                Outcome::Changed
            } else {
                Outcome::Consumed
            };
        }
        if id == scrollbar::id_for(self.id) {
            let moved = scrollbar::press(self.track, pos, &mut self.scroll);
            self.scroll.release_track();
            return if moved {
                Outcome::Changed
            } else {
                Outcome::Consumed
            };
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

    /// Click on the dimmed backdrop: dismisses when the policy allows,
    /// consumed either way (nothing under a modal is reachable).
    pub fn on_click_outside(&mut self) -> Outcome {
        if self.dismiss.outside_click {
            self.dismissed()
        } else {
            Outcome::Consumed
        }
    }

    /// Draw the dialog centred on `screen`. Prefer [`Self::render_in`] with
    /// the app's body area, so the dialog never covers the app's chrome.
    pub fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        self.render_in(screen, screen, buf, ctx);
    }

    /// Dim `screen` and draw the dialog centred in `bounds` (the app's body:
    /// below its header, above its status and footer rows), kept
    /// [`Self::outer_margin`] away from the edges of `bounds`. A body taller
    /// than the room scrolls ([`Self::body_scroll`]).
    pub fn render_in(&mut self, screen: Rect, bounds: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        self.h_cfg = self.horizontal.unwrap_or(ctx.hscroll);
        self.gap = t.spacing.button_gap;
        // at least one column of padding: the scrollbar lives there
        self.pad = crate::theme::Insets::new(t.spacing.dialog.x.max(1), t.spacing.dialog.y);
        // dim backdrop; the footer row stays live because its hints belong to the dialog
        ctx.modal_backdrop(buf, screen);
        let bounds = bounds.intersection(screen);
        let bounds = if bounds.is_empty() { screen } else { bounds };
        let m = self.outer_margin;
        // never wider than the bounds, however narrow they get
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
        let area = bounds.centered(Constraint::Length(width), Constraint::Length(height));
        self.area = area;
        self.track = Rect::ZERO;
        self.htrack = Rect::ZERO;
        let bg = t.surface_elevated;
        fill(buf, area, ratatui::style::Style::new().bg(bg));
        let panel = Panel::framed(None).focused(true);
        let block = ratatui::widgets::Block::new()
            .borders(ratatui::widgets::Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(t.border(true).bg(bg));
        ratatui::widgets::Widget::render(block, area, buf);
        let _ = panel;
        let inner = area.inner(Margin::new(1 + self.pad.x, 1 + self.pad.y));
        if inner.is_empty() {
            return;
        }
        buf.set_string(
            inner.x,
            inner.y,
            crate::ui::text::truncate(&self.title, inner.width as usize),
            t.title().bg(bg),
        );
        let body_y = inner.y + 2;
        let action_rows = self.action_rows(width);
        let actions_y = area.bottom().saturating_sub(1 + self.pad.y + action_rows);
        // the scrolling part of the body ends above the parts anchored over
        // the actions; the row right below it is a blank separator, where
        // "↓ N more" goes while there is more below
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
        // the gutter goes in the padding, never over the frame
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
                // the acknowledgement and the code preview are anchored above
                // the actions; the facts get the remaining rows and scroll
                // when the screen is too short
                let sheet_area = Rect::new(0, 0, inner.width, content);
                let mut sheet = Buffer::empty(sheet_area);
                fill(&mut sheet, sheet_area, ratatui::style::Style::new().bg(bg));
                crate::widgets::props::render(sheet_area, &mut sheet, t, facts, bg);
                blit_rows(
                    &sheet,
                    offset,
                    buf,
                    Rect::new(inner.x, body_y, inner.width, viewport),
                );
                let mut y = scroll_bottom;
                if !code.is_empty() {
                    y += 1;
                    let max = code.len().min(6);
                    // preformatted lines never wrap: a wider one scrolls
                    // sideways, cut with … on the side it continues past
                    let extent = Self::code_extent(code, inner.width);
                    let off = self.code_scroll.offset;
                    self.code_scroll.set_content(extent.content_len);
                    self.code_scroll.set_viewport(extent.viewport_len);
                    let off = if self.h_cfg.enabled {
                        off.min(self.code_scroll.max_offset())
                    } else {
                        0
                    };
                    self.code_scroll.offset = off;
                    let cut = |line: &str, w: u16| {
                        if off > 0 {
                            crate::ui::text::slice_cells(line, off, w as usize)
                        } else {
                            crate::ui::text::truncate(line, w as usize)
                        }
                    };
                    for (i, line) in code.iter().take(max).enumerate() {
                        let shown = if i == max - 1 && code.len() > max {
                            format!(
                                "{} … {} more",
                                cut(line, inner.width.saturating_sub(12)),
                                code.len() - max
                            )
                        } else {
                            cut(line, inner.width)
                        };
                        buf.set_string(inner.x, y + i as u16, &shown, t.secondary().bg(bg));
                    }
                    y += max as u16;
                    if self.h_cfg.shows_bar(&self.code_scroll) {
                        let hb = Rect::new(inner.x, y, inner.width, 1);
                        if hb.y < actions_y {
                            self.htrack = hb;
                            scrollbar::render_horizontal_with(
                                hb,
                                buf,
                                ctx,
                                self.id,
                                &self.code_scroll,
                                true,
                                self.h_cfg.bar == ScrollbarVisibility::Always,
                            );
                        }
                        y += 1;
                    }
                }
                if let Some(a) = ack {
                    y += 1;
                    let r = Rect::new(field_x, y, field_w, a.input.height_for(field_w));
                    a.input.render(r, buf, ctx, bg);
                }
            }
        }
        if self.scroll.overflows() && viewport > 0 {
            // the scrollbar sits in the padding just inside the frame, one
            // column off the text, so the text keeps its width
            self.track = Rect::new(area.right() - 2, body_y, 1, viewport);
            let below = self.scroll.max_offset() - self.scroll.offset;
            if below > 0 && scroll_bottom < actions_y {
                buf.set_string(
                    inner.x,
                    scroll_bottom,
                    crate::ui::text::truncate(&format!("↓ {below} more"), inner.width as usize),
                    t.muted().bg(bg),
                );
            }
        }
        // the confirming action stays disabled until the acknowledgement matches
        if let DialogBody::Facts { ack: Some(a), .. } = &self.body {
            let armed = a.input.text().trim() == a.token;
            if let Some(b) = self.confirm_index.and_then(|i| self.actions.get_mut(i)) {
                b.disabled = !armed;
            }
        }
        // actions: the shared button row, packed right, never cut
        self.button_row.render(
            self.id,
            Rect::new(inner.x, actions_y, inner.width, action_rows),
            &mut self.actions,
            buf,
            ctx,
            bg,
        );
        // the dialog surface itself blocks clicks from falling through
        ctx.hits.register(self.id, area);
        // re-register the controls on top of the surface
        match &self.body {
            DialogBody::Input(inp) => ctx.hits.register(inp.id, inp.area),
            DialogBody::Facts { ack: Some(a), .. } => ctx.hits.register(a.input.id, a.input.area),
            _ => {}
        }
        for b in &self.actions {
            ctx.hits.register(b.id, b.area);
        }
        if !self.track.is_empty() {
            scrollbar::render_vertical(self.track, buf, ctx, self.id, &self.scroll, true);
        }
        if !self.htrack.is_empty() && self.code_scroll.overflows() {
            ctx.hits.register(scrollbar::h_id_for(self.id), self.htrack);
        }
    }
}

/// The area a modal is placed in: `screen` without the app's chrome rows,
/// `top` rows of header and `bottom` rows of status and footer. Pass it to
/// [`Dialog::render_in`] so a dialog never covers the header or the hints
/// that belong to it. Falls back to `screen` when nothing would be left.
pub fn body_bounds(screen: Rect, top: u16, bottom: u16) -> Rect {
    let h = screen.height.saturating_sub(top + bottom);
    if h == 0 {
        return screen;
    }
    Rect::new(screen.x, screen.y + top, screen.width, h)
}

/// Copy rows `from..` of `src` into `dst` at `at` (as many as fit), so a
/// part drawn whole off screen can be shown through a scrolled window.
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
        let bar = d.htrack;
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

        // Shift+→ scrolls the preview; the plain arrows still move between buttons
        let mut focus = crate::core::focus::Focus::default();
        let ring = FocusRing::default();
        let shift = Key {
            code: KeyCode::Right,
            mods: ratatui::crossterm::event::KeyModifiers::SHIFT,
        };
        assert_eq!(d.on_key(&shift, &mut focus, &ring), Outcome::Changed);
        assert_eq!(d.code_scroll.offset, HScroll::default().step);
        let (buf, _) = draw(&mut d);
        assert!(line(&buf).contains('…'), "cut on the left: {}", line(&buf));
        assert!(
            buf[(bar.x, bar.y)].symbol() != scrollbar::H_THUMB,
            "the thumb moved"
        );

        // the wheel and the bar reach the end
        assert_eq!(d.on_wheel_h(1), Outcome::Changed);
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
        assert_eq!(d.code_scroll.offset, d.code_scroll.max_offset());
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
        // the height accounts for every wrapped row: nothing collides
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

        // a typed acknowledgement ignores a stray click
        let mut gated = gate(78);
        assert_eq!(gated.on_click_outside(), Outcome::Consumed);
        assert_eq!(gated.result, None);

        // the policy is the caller's to choose either way
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
    fn the_gated_action_is_refused_until_the_token_matches_even_before_a_redraw() {
        let mut d = gate(78);
        assert_eq!(d.confirm_index, Some(1));
        // never rendered, so the button was never drawn disabled
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
        // destructive dialogs open on Cancel, so Enter cancels
        let text = hint_text(&d.hints(Some(d.initial_focus)));
        assert_eq!(
            text,
            "←→ Choose · Enter Cancel · Esc Cancel · y Delete · n Cancel"
        );
        assert!(!text.contains("Confirm"), "{text}");
        // moving to the destructive button renames Enter
        let text = hint_text(&d.hints(Some(d.actions[1].id)));
        assert!(text.contains("Enter Delete"), "{text}");
    }

    #[test]
    fn a_typed_acknowledgement_says_what_to_type_and_hides_the_gated_action() {
        let d = gate(78);
        let text = hint_text(&d.hints(Some(d.initial_focus)));
        assert_eq!(
            text,
            "Type \"TRASH 4 UNDER /Users/alex/work/ide ON mbp\" to confirm · Enter Next · Esc Cancel"
        );
        // on the buttons, the unarmed action offers no choice and no Enter
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
            "←→ Choose · Enter Keep launching · Esc Keep launching · y Cancel launch · n Keep launching"
        );
    }

    /// Draw `d` in `bounds` on a `screen`, returning the buffer.
    fn draw(d: &mut Dialog, screen: Rect, bounds: Rect) -> Buffer {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(screen);
        d.render_in(screen, bounds, &mut buf, &mut ctx);
        buf
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

    #[test]
    fn letter_accelerators_follow_the_dialog_config() {
        let mut focus = crate::core::focus::Focus::default();
        let ring = FocusRing::default();
        let mut d = Dialog::confirm(WidgetId::of("q"), "Quit?", "Leave now?", "Quit");
        d.on_key(&key(KeyCode::Char('y')), &mut focus, &ring);
        assert!(matches!(d.result, Some(DialogResult::Action(_))));
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
        // the cut is announced, with a scrollbar beside the text
        assert!(text.contains("more"), "a \"↓ N more\" marker: {text}");
        assert!(text.contains(scrollbar::THUMB), "{text}");
        assert!(
            hint_text(&d.hints(Some(d.initial_focus))).contains("↑↓ Scroll"),
            "the hints say the body scrolls"
        );
        // End reaches the last line, and the marker goes away
        let mut focus = crate::core::focus::Focus::default();
        let ring = FocusRing::default();
        d.on_key(&key(KeyCode::End), &mut focus, &ring);
        let text = rows(&draw(&mut d, screen, screen)).join("\n");
        assert!(text.contains("line 30"), "{text}");
        assert!(!text.contains("more"), "{text}");
        // the wheel scrolls back up
        d.on_wheel(-100);
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
        // the instruction wraps instead of being cut
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
        // the field has no help, so it takes no empty help row
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
        // the margin is the caller's to choose
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
