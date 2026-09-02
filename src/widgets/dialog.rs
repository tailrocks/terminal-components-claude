//! Modal dialogs. A dialog owns its own focus scope; the app draws it over a
//! dimmed backdrop and routes every event to it while it is open.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Position, Rect};
use ratatui::style::Modifier;

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::theme::ButtonKind;
use crate::ui::ctx::{RenderCtx, fill};
use crate::widgets::button::{Button, row_layout_right};
use crate::widgets::input::{InputEvent, TextInput};
use crate::widgets::panel::Panel;

#[derive(Debug, Clone)]
pub enum DialogBody {
    Text(String),
    Input(TextInput),
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
        }
    }

    pub fn with_actions(mut self, actions: Vec<Button>, cancel_index: Option<usize>) -> Self {
        self.actions = actions;
        self.cancel_index = cancel_index;
        self
    }

    pub fn is_editing(&self) -> bool {
        matches!(&self.body, DialogBody::Input(i) if i.editing)
    }

    pub fn height(&self, width: u16) -> u16 {
        let inner_w = width.saturating_sub(6) as usize;
        let body_h = match &self.body {
            DialogBody::Text(t) => crate::ui::text::wrap(t, inner_w).len() as u16,
            DialogBody::Input(_) => TextInput::HEIGHT,
        };
        // border(2) + pad(1) + title(1) + gap(1) + body + gap(1) + actions(1) + pad(1)
        2 + 1 + 1 + 1 + body_h + 1 + 1 + 1
    }

    fn finish(&mut self, r: DialogResult) -> Outcome {
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
        if let DialogBody::Input(inp) = &mut self.body
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
                    // Enter in the field submits the dialog
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
            KeyCode::Esc => {
                if let Some(ci) = self.cancel_index {
                    self.finish(DialogResult::Action(ci))
                } else {
                    self.finish(DialogResult::Cancelled)
                }
            }
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
        match &mut self.body {
            DialogBody::Input(inp) => inp.on_paste(text).or(Outcome::Consumed),
            DialogBody::Text(_) => Outcome::Consumed,
        }
    }

    /// A completed click on `id`.
    pub fn on_click(
        &mut self,
        id: WidgetId,
        pos: Position,
        focus: &mut crate::core::focus::Focus,
    ) -> Outcome {
        if let DialogBody::Input(inp) = &mut self.body
            && inp.id == id
        {
            let was = focus.is(id);
            focus.focus(id);
            return inp.on_click(pos, was);
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

    /// Click outside the dialog: cancel if cancelable.
    pub fn on_click_outside(&mut self) -> Outcome {
        match self.cancel_index {
            Some(ci) => self.finish(DialogResult::Action(ci)),
            None => Outcome::Consumed,
        }
    }

    pub fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        // dim backdrop; the footer row stays live because its hints belong to the dialog
        let dim = Rect::new(
            screen.x,
            screen.y,
            screen.width,
            screen.height.saturating_sub(1),
        );
        for pos in dim.positions() {
            if let Some(c) = buf.cell_mut(pos) {
                let st = t.backdrop(c.style());
                c.set_style(st);
                c.modifier = Modifier::empty();
            }
        }
        ctx.begin_modal();
        let width = self.width.min(screen.width.saturating_sub(4)).max(20);
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
        if let DialogBody::Input(inp) = &self.body {
            ctx.hits.register(inp.id, inp.area);
        }
        for b in &self.actions {
            ctx.hits.register(b.id, b.area);
        }
    }
}
