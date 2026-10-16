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
use ratatui::layout::{Constraint, Position, Rect};

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::theme::ButtonKind;
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::popup::Dismiss;
use crate::widgets::button::{Button, row_layout_right};
use crate::widgets::input::{InputEvent, TextInput};
use crate::widgets::keyhint::{Hint, hint};
use crate::widgets::panel::Panel;

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
    pub area: Rect,
    pub result: Option<DialogResult>,
    pub initial_focus: WidgetId,
    /// How the dialog closes without an action (Esc, backdrop click).
    pub dismiss: Dismiss,
    /// The action a typed acknowledgement arms: disabled, and refused even
    /// if triggered, until the token matches. `None`: nothing is gated.
    pub confirm_index: Option<usize>,
}

impl Dialog {
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
            area: Rect::ZERO,
            result: None,
            initial_focus: ok_id,
            dismiss: Dismiss::LIGHT,
            confirm_index: None,
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
            area: Rect::ZERO,
            result: None,
            initial_focus: cancel_id,
            dismiss: Dismiss::LIGHT,
            confirm_index: None,
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
            area: Rect::ZERO,
            result: None,
            initial_focus: input_id,
            dismiss: Dismiss::LIGHT,
            confirm_index: None,
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
            input: TextInput::new(id.sub("ack"), &format!("Type {tok} to confirm")).plain_label(),
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
            area: Rect::ZERO,
            result: None,
            initial_focus: initial,
            dismiss: if token.is_some() {
                Dismiss::ESCAPE
            } else {
                Dismiss::LIGHT
            },
            confirm_index: token.map(|_| 1),
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

    /// Rows the code preview takes below the facts: at most six lines and
    /// the blank row above them.
    fn code_rows(code: &[String]) -> u16 {
        if code.is_empty() {
            0
        } else {
            code.len().min(6) as u16 + 1
        }
    }

    /// Rows the typed acknowledgement takes: the field and the blank row
    /// above it.
    fn ack_rows(present: bool) -> u16 {
        if present { TextInput::HEIGHT + 1 } else { 0 }
    }

    pub fn height(&self, width: u16) -> u16 {
        let inner_w = width.saturating_sub(6) as usize;
        let body_h = match &self.body {
            DialogBody::Text(t) => crate::ui::text::wrap(t, inner_w).len() as u16,
            DialogBody::Input(_) => TextInput::HEIGHT,
            DialogBody::Facts { facts, code, ack } => {
                crate::widgets::props::measure(facts, inner_w as u16)
                    + Self::code_rows(code)
                    + Self::ack_rows(ack.is_some())
            }
        };
        // border(2) + pad(1) + title(1) + gap(1) + body + gap(1) + actions(1) + pad(1)
        2 + 1 + 1 + 1 + body_h + 1 + 1 + 1
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
            KeyCode::Char('y') if matches!(self.body, DialogBody::Text(_)) => {
                let primary = self.actions.iter().position(|b| {
                    matches!(b.kind, ButtonKind::Primary | ButtonKind::Danger) && !b.disabled
                });
                match primary {
                    Some(p) => self.finish(DialogResult::Action(p)),
                    None => Outcome::Consumed,
                }
            }
            KeyCode::Char('n') if matches!(self.body, DialogBody::Text(_)) => {
                match self.cancel_index {
                    Some(ci) => self.finish(DialogResult::Action(ci)),
                    None => Outcome::Consumed,
                }
            }
            _ => Outcome::Consumed,
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

    pub fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        // dim backdrop; the footer row stays live because its hints belong to the dialog
        ctx.modal_backdrop(buf, screen);
        // never wider than the screen, however narrow it gets
        let width = self
            .width
            .min(screen.width.saturating_sub(4))
            .max(20)
            .min(screen.width);
        let height = self.height(width).min(screen.height.saturating_sub(2));
        let area = screen.centered(Constraint::Length(width), Constraint::Length(height));
        self.area = area;
        let bg = t.surface_elevated;
        fill(buf, area, ratatui::style::Style::new().bg(bg));
        let panel = Panel::framed(None).focused(true);
        let block = ratatui::widgets::Block::new()
            .borders(ratatui::widgets::Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(t.border(true).bg(bg));
        ratatui::widgets::Widget::render(block, area, buf);
        let _ = panel;
        let inner = area.inner(ratatui::layout::Margin::new(3, 2));
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
        let actions_y = area.bottom().saturating_sub(3);
        match &mut self.body {
            DialogBody::Text(text) => {
                let lines = crate::ui::text::wrap(text, inner.width as usize);
                for (i, l) in lines.iter().enumerate() {
                    let y = body_y + i as u16;
                    if y >= actions_y.saturating_sub(1) {
                        break;
                    }
                    buf.set_string(inner.x, y, l, t.secondary().bg(bg));
                }
            }
            DialogBody::Input(inp) => {
                let r = Rect::new(
                    inner.x.saturating_sub(1),
                    body_y,
                    inner.width + 1,
                    TextInput::HEIGHT,
                );
                inp.render(r, buf, ctx, bg);
            }
            DialogBody::Facts { facts, code, ack } => {
                // the acknowledgement and the code preview are anchored above
                // the actions; the facts get the remaining rows and are the
                // only part that clips when the screen is too short
                let fixed = Self::code_rows(code) + Self::ack_rows(ack.is_some());
                let facts_bottom = actions_y.saturating_sub(1 + fixed);
                let mut y = body_y;
                let used = crate::widgets::props::render(
                    Rect::new(inner.x, y, inner.width, facts_bottom.saturating_sub(y)),
                    buf,
                    t,
                    facts,
                    bg,
                );
                y += used;
                if !code.is_empty() {
                    y += 1;
                    let max = code.len().min(6);
                    for (i, line) in code.iter().take(max).enumerate() {
                        let shown = if i == max - 1 && code.len() > max {
                            format!(
                                "{} … {} more",
                                crate::ui::text::truncate(
                                    line,
                                    inner.width.saturating_sub(12) as usize
                                ),
                                code.len() - max
                            )
                        } else {
                            crate::ui::text::truncate(line, inner.width as usize)
                        };
                        buf.set_string(inner.x, y + i as u16, &shown, t.secondary().bg(bg));
                    }
                    y += max as u16;
                }
                if let Some(a) = ack {
                    y += 1;
                    let r = Rect::new(
                        inner.x.saturating_sub(1),
                        y,
                        inner.width + 1,
                        TextInput::HEIGHT,
                    );
                    a.input.render(r, buf, ctx, bg);
                }
            }
        }
        // the confirming action stays disabled until the acknowledgement matches
        if let DialogBody::Facts { ack: Some(a), .. } = &self.body {
            let armed = a.input.text().trim() == a.token;
            if let Some(b) = self.confirm_index.and_then(|i| self.actions.get_mut(i)) {
                b.disabled = !armed;
            }
        }
        // actions, right aligned
        let widths: Vec<u16> = self.actions.iter().map(|b| b.width()).collect();
        let rects = row_layout_right(Rect::new(inner.x, actions_y, inner.width, 1), &widths, 1);
        for (b, r) in self.actions.iter_mut().zip(rects) {
            b.render(r, buf, ctx, bg);
        }
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
}
