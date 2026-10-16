//! A modal form: text inputs (masked secrets included), selects,
//! checkboxes, radio groups, "choose…" buttons and notes, with Save /
//! Cancel and per-field values read back by name ([`FormValues`]). Moved
//! from jackin-preview so every app can use it; values are strings, bools
//! and indices ([`FieldValue`]), never app types. An owner that needs a
//! picker or a file browser for a "choose…" field opens its own modal on
//! [`FormEvent::Choose`] and writes the result back with
//! [`FormDialog::set_chooser`].

use crate::core::event::{Key, Outcome};
use crate::core::focus::{Focus, FocusRing};
use crate::core::id::WidgetId;
use crate::core::scroll::ScrollState;
use crate::theme::Tone;
use crate::ui::ctx::RenderCtx;
use crate::ui::text::truncate;
use crate::widgets::button::{Button, row_layout_right};
use crate::widgets::choice::{Checkbox, RadioGroup};
use crate::widgets::input::{InputEvent, TextInput};
use crate::widgets::scrollbar;
use crate::widgets::select::Select;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldValue {
    Text(String),
    Choice(usize),
    Bool(bool),
}

pub type FormValues = Vec<(String, FieldValue)>;

pub enum FormFieldKind {
    Input(TextInput),
    Select(Select),
    Check(Checkbox),
    Radio(RadioGroup),
    /// Read-only value with a `Choose…` button that the owner handles.
    Chooser {
        label: String,
        value: String,
        detail: Option<String>,
        button: Button,
    },
    /// Static text rows (helper lines, validation results).
    Note(Vec<(String, Tone)>),
}

pub struct FormField {
    pub name: String,
    pub kind: FormFieldKind,
    pub visible: bool,
}

impl FormField {
    pub fn input(name: &str, input: TextInput) -> Self {
        Self {
            name: name.into(),
            kind: FormFieldKind::Input(input),
            visible: true,
        }
    }
    pub fn select(name: &str, select: Select) -> Self {
        Self {
            name: name.into(),
            kind: FormFieldKind::Select(select),
            visible: true,
        }
    }
    pub fn check(name: &str, c: Checkbox) -> Self {
        Self {
            name: name.into(),
            kind: FormFieldKind::Check(c),
            visible: true,
        }
    }
    pub fn radio(name: &str, r: RadioGroup) -> Self {
        Self {
            name: name.into(),
            kind: FormFieldKind::Radio(r),
            visible: true,
        }
    }
    pub fn chooser(name: &str, id: WidgetId, label: &str, value: &str, button: &str) -> Self {
        Self {
            name: name.into(),
            kind: FormFieldKind::Chooser {
                label: label.into(),
                value: value.into(),
                detail: None,
                button: Button::secondary(id, button),
            },
            visible: true,
        }
    }
    pub fn note(name: &str, lines: Vec<(String, Tone)>) -> Self {
        Self {
            name: name.into(),
            kind: FormFieldKind::Note(lines),
            visible: true,
        }
    }
    pub fn hidden(mut self) -> Self {
        self.visible = false;
        self
    }

    fn height(&self) -> u16 {
        match &self.kind {
            FormFieldKind::Input(_) => TextInput::HEIGHT,
            FormFieldKind::Select(_) => Select::HEIGHT,
            FormFieldKind::Check(_) => 1,
            FormFieldKind::Radio(r) => r.height(),
            FormFieldKind::Chooser { detail, .. } => 2 + u16::from(detail.is_some()),
            FormFieldKind::Note(l) => l.len() as u16,
        }
    }

    pub fn value(&self) -> FieldValue {
        match &self.kind {
            FormFieldKind::Input(i) => FieldValue::Text(i.text().to_owned()),
            FormFieldKind::Select(s) => FieldValue::Choice(s.selected),
            FormFieldKind::Check(c) => FieldValue::Bool(c.checked),
            FormFieldKind::Radio(r) => FieldValue::Choice(r.selected),
            FormFieldKind::Chooser { value, .. } => FieldValue::Text(value.clone()),
            FormFieldKind::Note(_) => FieldValue::Text(String::new()),
        }
    }

    fn focus_id(&self) -> Option<WidgetId> {
        match &self.kind {
            FormFieldKind::Input(i) => Some(i.id),
            FormFieldKind::Select(s) => Some(s.id),
            FormFieldKind::Check(c) => Some(c.id),
            FormFieldKind::Radio(r) => Some(r.id),
            FormFieldKind::Chooser { button, .. } => Some(button.id),
            FormFieldKind::Note(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormEvent {
    Changed(String),
    /// A chooser button fired.
    Choose(String),
    /// A named action button fired (`validate`, `plain`, …).
    Action(String),
    Save,
    Cancel,
}

pub struct FormDialog {
    /// Which dismissals close it. Default `Dismiss::ESCAPE`.
    pub dismiss: crate::ui::popup::Dismiss,
    pub id: WidgetId,
    pub title: String,
    pub meta: Option<String>,
    pub fields: Vec<FormField>,
    /// Extra buttons before Cancel/Save: (name, button).
    pub actions: Vec<(String, Button)>,
    pub cancel: Button,
    pub save: Button,
    pub width: u16,
    pub error: Option<String>,
    pub scroll: ScrollState,
    pub area: Rect,
    pub events: Vec<FormEvent>,
    pub dirty: bool,
    /// When false, Save emits `Action("save")` and the owner closes the form
    /// once validation passes.
    pub pop_on_save: bool,
}

impl FormDialog {
    pub fn new(id: WidgetId, title: &str, fields: Vec<FormField>) -> Self {
        Self {
            dismiss: crate::ui::popup::Dismiss::ESCAPE,
            id,
            title: title.to_owned(),
            meta: None,
            fields,
            actions: vec![],
            cancel: Button::subtle(id.sub("cancel"), "Cancel"),
            save: Button::primary(id.sub("save"), "Save"),
            width: 66,
            error: None,
            scroll: ScrollState::default(),
            area: Rect::ZERO,
            events: vec![],
            dirty: false,
            pop_on_save: true,
        }
    }

    pub fn keep_open_on_save(mut self) -> Self {
        self.pop_on_save = false;
        self
    }

    pub fn action(mut self, name: &str, button: Button) -> Self {
        self.actions.push((name.into(), button));
        self
    }

    pub fn width(mut self, w: u16) -> Self {
        self.width = w;
        self
    }

    pub fn meta(mut self, m: &str) -> Self {
        self.meta = Some(m.to_owned());
        self
    }

    pub fn field(&self, name: &str) -> Option<&FormField> {
        self.fields.iter().find(|f| f.name == name)
    }

    pub fn field_mut(&mut self, name: &str) -> Option<&mut FormField> {
        self.fields.iter_mut().find(|f| f.name == name)
    }

    pub fn set_visible(&mut self, name: &str, v: bool) {
        if let Some(f) = self.field_mut(name) {
            f.visible = v;
        }
    }

    pub fn text(&self, name: &str) -> String {
        match self.field(name).map(|f| f.value()) {
            Some(FieldValue::Text(s)) => s,
            _ => String::new(),
        }
    }

    pub fn choice(&self, name: &str) -> usize {
        match self.field(name).map(|f| f.value()) {
            Some(FieldValue::Choice(i)) => i,
            _ => 0,
        }
    }

    pub fn set_text(&mut self, name: &str, v: &str) {
        if let Some(f) = self.field_mut(name) {
            match &mut f.kind {
                FormFieldKind::Input(i) => {
                    *i = TextInput::new(i.id, &i.label.clone())
                        .value(v)
                        .placeholder(&i.placeholder.clone())
                }
                FormFieldKind::Chooser { value, .. } => *value = v.to_owned(),
                _ => {}
            }
        }
    }

    pub fn set_chooser(&mut self, name: &str, value: &str, detail: Option<&str>) {
        if let Some(f) = self.field_mut(name)
            && let FormFieldKind::Chooser {
                value: v,
                detail: d,
                ..
            } = &mut f.kind
        {
            *v = value.to_owned();
            *d = detail.map(str::to_owned);
        }
    }

    pub fn set_note(&mut self, name: &str, lines: Vec<(String, Tone)>) {
        if let Some(f) = self.field_mut(name)
            && let FormFieldKind::Note(l) = &mut f.kind
        {
            *l = lines;
        }
    }

    pub fn values(&self) -> FormValues {
        self.fields
            .iter()
            .map(|f| (f.name.clone(), f.value()))
            .collect()
    }

    pub fn initial_focus(&self) -> WidgetId {
        self.fields
            .iter()
            .filter(|f| f.visible)
            .find_map(|f| f.focus_id())
            .unwrap_or(self.save.id)
    }

    pub fn is_editing(&self) -> bool {
        self.fields
            .iter()
            .any(|f| matches!(&f.kind, FormFieldKind::Input(i) if i.editing))
    }

    fn save_event(&self) -> FormEvent {
        if self.pop_on_save {
            FormEvent::Save
        } else {
            FormEvent::Action("save".into())
        }
    }

    fn any_open_select(&self) -> bool {
        self.fields
            .iter()
            .any(|f| matches!(&f.kind, FormFieldKind::Select(s) if s.open))
    }

    pub fn on_key(&mut self, key: &Key, focus: &mut Focus, ring: &FocusRing) -> Outcome {
        let cur = focus.current();
        let editing = self.is_editing();
        let open = self.any_open_select();
        if !editing && !open {
            match key.code {
                KeyCode::Esc => {
                    self.events.push(FormEvent::Cancel);
                    return Outcome::Changed;
                }
                KeyCode::Tab => {
                    focus.next(ring);
                    return Outcome::Changed;
                }
                KeyCode::BackTab => {
                    focus.prev(ring);
                    return Outcome::Changed;
                }
                _ => {}
            }
        }
        for f in self.fields.iter_mut() {
            if !f.visible {
                continue;
            }
            let name = f.name.clone();
            match &mut f.kind {
                FormFieldKind::Input(i) if cur == Some(i.id) => {
                    let (o, ev) = i.on_key(key);
                    match ev {
                        Some(InputEvent::CommittedTab { backward }) => {
                            if backward {
                                focus.prev(ring)
                            } else {
                                focus.next(ring)
                            }
                            self.dirty = true;
                            self.events.push(FormEvent::Changed(name));
                            return Outcome::Changed;
                        }
                        Some(InputEvent::Committed) => {
                            self.dirty = true;
                            self.events.push(FormEvent::Changed(name));
                            return Outcome::Changed;
                        }
                        Some(InputEvent::Changed) => {
                            self.dirty = true;
                        }
                        _ => {}
                    }
                    if o.consumed() {
                        return o;
                    }
                }
                FormFieldKind::Select(s) if cur == Some(s.id) => {
                    let (o, ev) = s.on_key(key);
                    if ev.is_some() {
                        self.dirty = true;
                        self.events.push(FormEvent::Changed(name));
                    }
                    if o.consumed() {
                        return o;
                    }
                }
                FormFieldKind::Check(c) if cur == Some(c.id) => {
                    let o = c.on_key(key);
                    if o == Outcome::Changed {
                        self.dirty = true;
                        self.events.push(FormEvent::Changed(name));
                    }
                    if o.consumed() {
                        return o;
                    }
                }
                FormFieldKind::Radio(r) if cur == Some(r.id) => {
                    let before = r.selected;
                    let o = r.on_key(key);
                    if r.selected != before {
                        self.dirty = true;
                        self.events.push(FormEvent::Changed(name));
                    }
                    if o.consumed() {
                        return o;
                    }
                }
                FormFieldKind::Chooser { button, .. } if cur == Some(button.id) => {
                    let (o, fired) = button.on_key(key);
                    if fired {
                        self.events.push(FormEvent::Choose(name));
                        return Outcome::Changed;
                    }
                    if o.consumed() {
                        return o;
                    }
                }
                _ => {}
            }
        }
        for (name, b) in self.actions.iter_mut() {
            if cur == Some(b.id) {
                let (o, fired) = b.on_key(key);
                if fired {
                    self.events.push(FormEvent::Action(name.clone()));
                    return Outcome::Changed;
                }
                if o.consumed() {
                    return o;
                }
            }
        }
        if cur == Some(self.cancel.id) {
            let (o, fired) = self.cancel.on_key(key);
            if fired {
                self.events.push(FormEvent::Cancel);
                return Outcome::Changed;
            }
            if o.consumed() {
                return o;
            }
        }
        if cur == Some(self.save.id) {
            let (o, fired) = self.save.on_key(key);
            if fired {
                self.events.push(self.save_event());
                return Outcome::Changed;
            }
            if o.consumed() {
                return o;
            }
        }
        match key.code {
            KeyCode::Enter if !editing => {
                if self.save.can_activate() {
                    self.events.push(self.save_event());
                } else {
                    focus.focus(self.initial_focus());
                }
                Outcome::Changed
            }
            KeyCode::Left | KeyCode::Right if !editing => {
                let ids: Vec<WidgetId> = self
                    .actions
                    .iter()
                    .map(|(_, b)| b.id)
                    .chain([self.cancel.id, self.save.id])
                    .collect();
                if let Some(i) = ids.iter().position(|id| Some(*id) == cur) {
                    let n = ids.len();
                    let j = if key.code == KeyCode::Left {
                        (i + n - 1) % n
                    } else {
                        (i + 1) % n
                    };
                    focus.focus(ids[j]);
                }
                Outcome::Changed
            }
            _ => Outcome::Consumed,
        }
    }

    /// The form's text inputs, for pointer routing.
    fn inputs(&mut self) -> impl Iterator<Item = &mut TextInput> {
        self.fields.iter_mut().filter_map(|f| match &mut f.kind {
            FormFieldKind::Input(i) => Some(i),
            _ => None,
        })
    }

    /// A press on `id` with the click count it starts: an input
    /// remembers it for the drag (the second press of a double click
    /// selects the word).
    pub fn on_press(&mut self, id: WidgetId, pos: Position, count: u8) -> Outcome {
        self.inputs()
            .find(|i| i.id == id)
            .map_or(Outcome::Ignored, |i| i.on_press_count(pos, count))
    }

    /// A drag held since a press on `pressed`: an input selects its text.
    pub fn on_drag(&mut self, pressed: WidgetId, pos: Position) -> Outcome {
        self.inputs()
            .find(|i| i.id == pressed)
            .map_or(Outcome::Ignored, |i| i.on_drag(pos))
    }

    /// A completed click. Same as [`Self::on_click_count`] with a count
    /// of 1.
    pub fn on_click(&mut self, id: WidgetId, pos: Position, focus: &mut Focus) -> Outcome {
        self.on_click_count(id, pos, 1, focus)
    }

    /// A completed click with its multi-click `count`: in an input 2
    /// selects the word and 3 the line.
    pub fn on_click_count(
        &mut self,
        id: WidgetId,
        pos: Position,
        count: u8,
        focus: &mut Focus,
    ) -> Outcome {
        for f in self.fields.iter_mut() {
            let name = f.name.clone();
            match &mut f.kind {
                FormFieldKind::Input(i) if i.id == id => {
                    focus.focus(id);
                    return i.on_click_count(pos, count.max(1));
                }
                FormFieldKind::Select(s) if s.owns(id) => {
                    focus.focus(s.id);
                    let (o, ev) = s.on_click(id);
                    if ev.is_some() {
                        self.dirty = true;
                        self.events.push(FormEvent::Changed(name));
                    }
                    return o.or(Outcome::Changed);
                }
                FormFieldKind::Check(c) if c.id == id => {
                    focus.focus(id);
                    self.dirty = true;
                    self.events.push(FormEvent::Changed(name));
                    return c.on_click();
                }
                FormFieldKind::Radio(r) => {
                    for i in 0..r.options.len() {
                        if r.option_id(i) == id {
                            focus.focus(r.id);
                            let o = r.on_click(i);
                            self.dirty = true;
                            self.events.push(FormEvent::Changed(name));
                            return o;
                        }
                    }
                }
                FormFieldKind::Chooser { button, .. } if button.id == id => {
                    focus.focus(id);
                    if button.on_click() {
                        self.events.push(FormEvent::Choose(name));
                    }
                    return Outcome::Changed;
                }
                _ => {}
            }
        }
        for (name, b) in self.actions.iter_mut() {
            if b.id == id {
                focus.focus(id);
                if b.on_click() {
                    self.events.push(FormEvent::Action(name.clone()));
                }
                return Outcome::Changed;
            }
        }
        if id == self.cancel.id {
            focus.focus(id);
            if self.cancel.on_click() {
                self.events.push(FormEvent::Cancel);
            }
            return Outcome::Changed;
        }
        if id == self.save.id {
            focus.focus(id);
            if self.save.on_click() {
                self.events.push(self.save_event());
            }
            return Outcome::Changed;
        }
        // click elsewhere closes an open select
        for f in self.fields.iter_mut() {
            if let FormFieldKind::Select(s) = &mut f.kind {
                s.close();
            }
        }
        Outcome::Consumed
    }

    /// A control lost focus: the form field ends its edit; a committed
    /// value marks the form dirty and queues `Changed` like Enter does.
    pub fn on_blur(&mut self, ch: crate::core::focus::FocusChange) -> Outcome {
        for f in self.fields.iter_mut() {
            match &mut f.kind {
                FormFieldKind::Input(i) if i.id == ch.from => {
                    return match i.on_blur() {
                        Some(InputEvent::Committed) => {
                            self.dirty = true;
                            self.events.push(FormEvent::Changed(f.name.clone()));
                            Outcome::Changed
                        }
                        Some(_) => Outcome::Changed,
                        None => Outcome::Ignored,
                    };
                }
                FormFieldKind::Select(s) if s.id == ch.from => return s.on_blur(),
                _ => {}
            }
        }
        Outcome::Ignored
    }

    pub fn on_paste(&mut self, text: &str) -> Outcome {
        for f in self.fields.iter_mut() {
            if let FormFieldKind::Input(i) = &mut f.kind
                && i.editing
            {
                return i.on_paste(text);
            }
        }
        Outcome::Consumed
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        self.scroll.scroll_by(delta as isize);
        Outcome::Changed
    }

    fn content_height(&self) -> u16 {
        let mut h = 0u16;
        for f in self.fields.iter().filter(|f| f.visible) {
            h += f.height() + 1;
        }
        h
    }

    pub fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let content = self.content_height();
        let err = u16::from(self.error.is_some());
        let h = 2 + 2 + content + err + 2;
        let (area, inner) = crate::ui::modal::Frame::new(self.width, h)
            .title(&self.title)
            .meta(self.meta.as_deref())
            .render(screen, buf, ctx);
        self.area = area;
        let t = ctx.theme;
        let bg = t.surface_elevated;
        let body_h = inner.height.saturating_sub(3 + err);
        self.scroll.set_content(content as usize);
        self.scroll.set_viewport(body_h as usize);
        // keep the focused field visible
        let cur = ctx.interaction.focus;
        let mut y_off = 0u16;
        for f in self.fields.iter().filter(|f| f.visible) {
            if f.focus_id().is_some() && f.focus_id() == cur {
                let top = y_off as usize;
                let bottom = (y_off + f.height()) as usize;
                if top < self.scroll.offset {
                    self.scroll.scroll_to(top);
                } else if bottom > self.scroll.offset + body_h as usize {
                    self.scroll
                        .scroll_to(bottom.saturating_sub(body_h as usize));
                }
            }
            y_off += f.height() + 1;
        }
        let body = Rect::new(inner.x, inner.y, inner.width, body_h);
        let mut y: i32 = body.y as i32 - self.scroll.offset as i32;
        let mut open_select: Option<usize> = None;
        for (fi, f) in self.fields.iter_mut().enumerate() {
            if !f.visible {
                continue;
            }
            let fh = f.height() as i32;
            if y + fh > body.y as i32 && y < body.bottom() as i32 {
                let yy = y.max(body.y as i32) as u16;
                let clip_top = (body.y as i32 - y).max(0) as u16;
                let avail = (body.bottom() as i32 - y.max(body.y as i32)).max(0) as u16;
                let r = Rect::new(
                    inner.x - 1,
                    yy,
                    inner.width + 1,
                    avail.min(fh as u16 - clip_top),
                );
                if clip_top == 0 && !r.is_empty() {
                    match &mut f.kind {
                        FormFieldKind::Input(i) => i.render(r, buf, ctx, bg),
                        FormFieldKind::Select(s) => {
                            if s.open {
                                open_select = Some(fi);
                            } else {
                                s.render(r, buf, ctx, bg);
                            }
                        }
                        FormFieldKind::Check(c) => c.render(r, buf, ctx, bg),
                        FormFieldKind::Radio(rg) => rg.render(r, buf, ctx, bg),
                        FormFieldKind::Chooser {
                            label,
                            value,
                            detail,
                            button,
                        } => {
                            let focused = ctx.interaction.focused(button.id);
                            buf.set_string(r.x + 2, r.y, label.as_str(), t.label(focused).bg(bg));
                            if r.height >= 2 {
                                let bw = button.width();
                                let vw = r.width.saturating_sub(bw + 4) as usize;
                                buf.set_string(
                                    r.x + 2,
                                    r.y + 1,
                                    truncate(value, vw),
                                    t.primary().bg(bg),
                                );
                                button.render(
                                    Rect::new(r.right().saturating_sub(bw + 1), r.y + 1, bw, 1),
                                    buf,
                                    ctx,
                                    bg,
                                );
                            }
                            if r.height >= 3
                                && let Some(d) = detail
                            {
                                buf.set_string(
                                    r.x + 2,
                                    r.y + 2,
                                    truncate(d, r.width.saturating_sub(3) as usize),
                                    t.muted().bg(bg),
                                );
                            }
                        }
                        FormFieldKind::Note(lines) => {
                            for (i, (l, tone)) in lines.iter().enumerate() {
                                if (i as u16) < r.height {
                                    buf.set_string(
                                        r.x + 2,
                                        r.y + i as u16,
                                        truncate(l, r.width.saturating_sub(3) as usize),
                                        Style::new().fg(t.tone(*tone)).bg(bg),
                                    );
                                }
                            }
                        }
                    }
                }
            }
            y += fh + 1;
        }
        if self.scroll.overflows() {
            crate::ui::fade::scroll_edges(
                buf,
                ctx,
                Rect::new(
                    area.x,
                    body.y,
                    (area.right() - 2).saturating_sub(area.x),
                    body.height,
                ),
                &self.scroll,
            );
            scrollbar::render_vertical(
                Rect::new(area.right() - 2, body.y, 1, body.height),
                buf,
                ctx,
                self.id,
                &self.scroll,
                true,
            );
        }
        if let Some(e) = &self.error {
            let ey = inner.bottom().saturating_sub(2);
            buf.set_string(
                inner.x,
                ey,
                truncate(&format!("! {e}"), inner.width as usize),
                t.error_fg().bg(bg),
            );
            buf.set_string(
                inner.x,
                ey,
                "!",
                t.error_fg().bg(bg).add_modifier(Modifier::BOLD),
            );
        }
        let ay = inner.bottom().saturating_sub(1);
        let mut widths: Vec<u16> = self.actions.iter().map(|(_, b)| b.width()).collect();
        widths.push(self.cancel.width());
        widths.push(self.save.width());
        let rects = row_layout_right(Rect::new(inner.x, ay, inner.width, 1), &widths, 1);
        let n = self.actions.len();
        for (i, (_, b)) in self.actions.iter_mut().enumerate() {
            b.render(rects[i], buf, ctx, bg);
        }
        self.cancel.render(rects[n], buf, ctx, bg);
        self.save.render(rects[n + 1], buf, ctx, bg);
        // hits on top of the surface
        for f in &self.fields {
            if !f.visible {
                continue;
            }
            match &f.kind {
                FormFieldKind::Input(i) => ctx.hits.register(i.id, i.area),
                FormFieldKind::Select(s) => ctx.hits.register(s.id, s.area),
                FormFieldKind::Check(c) => ctx.hits.register(c.id, c.area),
                FormFieldKind::Radio(r) => {
                    for (i, a) in r.areas.iter().enumerate() {
                        ctx.hits.register(r.option_id(i), *a);
                    }
                }
                FormFieldKind::Chooser { button, .. } => ctx.hits.register(button.id, button.area),
                FormFieldKind::Note(_) => {}
            }
        }
        for (_, b) in &self.actions {
            ctx.hits.register(b.id, b.area);
        }
        ctx.hits.register(self.cancel.id, self.cancel.area);
        ctx.hits.register(self.save.id, self.save.area);
        // an open select popup draws last
        if let Some(fi) = open_select {
            let mut y: i32 = body.y as i32 - self.scroll.offset as i32;
            for (i, f) in self.fields.iter_mut().enumerate() {
                if !f.visible {
                    continue;
                }
                if i == fi
                    && let FormFieldKind::Select(s) = &mut f.kind
                {
                    let r = Rect::new(
                        inner.x - 1,
                        y.max(body.y as i32) as u16,
                        inner.width + 1,
                        Select::HEIGHT,
                    );
                    s.render(r, buf, ctx, bg);
                    ctx.hits.register(s.id, s.area);
                    for k in 0..s.options.len() {
                        if let Some(a) = ctx.hits.area_of(s.option_id(k)) {
                            ctx.hits.register(s.option_id(k), a);
                        }
                    }
                }
                y += f.height() as i32 + 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;

    fn screen_text(buf: &Buffer) -> String {
        buf.content().iter().map(|c| c.symbol()).collect()
    }

    #[test]
    fn a_library_form_dialog_reads_its_values_back_by_name() {
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut d = FormDialog::new(
            WidgetId::of("form"),
            "New mount",
            vec![
                FormField::input("path", TextInput::new(WidgetId::of("form.path"), "Path")),
                FormField::check(
                    "ro",
                    Checkbox::new(WidgetId::of("form.ro"), "Read-only", true),
                ),
            ],
        );
        d.set_text("path", "/srv/data");
        {
            let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 90, 30));
            d.render(Rect::new(0, 0, 90, 30), &mut buf, &mut ctx);
            assert!(screen_text(&buf).contains("New mount"));
        }
        assert_eq!(d.text("path"), "/srv/data");
        let values = d.values();
        assert!(
            values
                .iter()
                .any(|(n, v)| n == "ro" && *v == FieldValue::Bool(true))
        );
    }
}
