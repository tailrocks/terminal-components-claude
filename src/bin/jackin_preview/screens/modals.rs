//! Host modal families composed from the design-system primitives: the
//! file browser, choice dialog (radio + buttons), form dialog (selects,
//! inputs, masked secrets, choose-buttons), the 1Password reference chain,
//! the read-only info dialog with copyable rows, and the help overlay.

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::focus::{Focus, FocusRing};
use junie_tui::core::id::WidgetId;
use junie_tui::core::scroll::ScrollState;
use junie_tui::theme::{ButtonKind, Theme, Tone};
use junie_tui::ui::ctx::{RenderCtx, fill};
use junie_tui::ui::popup::{Placement, place};
use junie_tui::ui::text::{truncate, width};
use junie_tui::widgets::button::{Button, row_layout_right};
use junie_tui::widgets::choice::{Checkbox, RadioGroup};
use junie_tui::widgets::input::{InputEvent, TextInput};
use junie_tui::widgets::list::{ListBox, ListItem, SelectMode};
use junie_tui::widgets::picker::{Picker, PickerEvent, PickerItem, PickerStatus};
use junie_tui::widgets::progress::render_spinner;
use junie_tui::widgets::props::{Prop, PropsEvent, PropsList};
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::select::Select;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};

use crate::domain::onepassword::OpReference;
use crate::sim::onepassword::{FieldKind, OpError, SimOnePassword};
use crate::sim::world::{FsEntry, World};

/// Dim the page (footer excluded), start the modal barrier and draw an
/// elevated rounded frame. Returns the inner content area.
#[allow(clippy::too_many_arguments)]
pub fn modal_frame(
    screen: Rect,
    buf: &mut Buffer,
    ctx: &mut RenderCtx,
    width_: u16,
    height: u16,
    title: &str,
    meta: Option<&str>,
    center: bool,
) -> (Rect, Rect) {
    let t = ctx.theme;
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
    let w = width_.min(screen.width.saturating_sub(4)).max(20);
    let h = height.min(screen.height.saturating_sub(2)).max(5);
    let area = if center {
        Rect::new(
            screen.x + (screen.width - w) / 2,
            screen.y + (screen.height.saturating_sub(h)) / 2,
            w,
            h,
        )
    } else {
        place(screen, Rect::ZERO, w, h, Placement::Center)
    };
    let bg = t.surface_elevated;
    fill(buf, area, Style::new().bg(bg));
    let block = ratatui::widgets::Block::new()
        .borders(ratatui::widgets::Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(t.border(true).bg(bg));
    ratatui::widgets::Widget::render(block, area, buf);
    if area.width > 6 {
        let tt = format!(
            " {} ",
            truncate(title, area.width.saturating_sub(6) as usize)
        );
        buf.set_string(area.x + 2, area.y, &tt, t.title().bg(bg));
        if let Some(m) = meta {
            let mt = format!(" {m} ");
            let mw = width(&mt) as u16;
            if area.width > width(&tt) as u16 + mw + 4 {
                buf.set_string(area.right() - 2 - mw, area.y, &mt, t.faint().bg(bg));
            }
        }
    }
    ctx.hits.register(WidgetId::of("modal.surface"), area);
    (area, area.inner(ratatui::layout::Margin::new(3, 1)))
}

fn hint_row(buf: &mut Buffer, inner: Rect, t: &Theme, text: &str) {
    let y = inner.bottom().saturating_sub(1);
    buf.set_string(
        inner.x,
        y,
        truncate(text, inner.width as usize),
        t.faint().bg(t.surface_elevated),
    );
}

// ------------------------------------------------------------ file browser

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserResult {
    Chosen { path: String, readonly: bool },
    GitUrl(String),
    Cancelled,
}

pub struct FileBrowser {
    pub title: String,
    pub cwd: String,
    pub path: TextInput,
    pub list: ListBox,
    pub entries: Vec<FsEntry>,
    pub readonly: Option<Checkbox>,
    pub git_url: Button,
    pub cancel: Button,
    pub choose: Button,
    /// Git URL mode: the field holds a URL and `Choose` resolves it.
    pub url_mode: bool,
    pub resolving: Option<(String, i64)>,
    pub error: Option<String>,
    pub dirs_only: bool,
    pub result: Option<BrowserResult>,
    pub area: Rect,
}

impl FileBrowser {
    pub fn new(
        id: WidgetId,
        title: &str,
        cwd: &str,
        with_readonly: bool,
        dirs_only: bool,
        w: &World,
    ) -> Self {
        let mut b = Self {
            title: title.to_owned(),
            cwd: cwd.to_owned(),
            path: TextInput::new(id.sub("path"), "Path").plain_label(),
            list: ListBox::new(id.sub("list"), vec![], SelectMode::Single)
                .empty_text("Empty folder"),
            entries: vec![],
            readonly: with_readonly.then(|| Checkbox::new(id.sub("ro"), "Mount read-only", false)),
            git_url: Button::secondary(id.sub("git"), "Git URL…"),
            cancel: Button::subtle(id.sub("cancel"), "Cancel"),
            choose: Button::primary(id.sub("choose"), "Choose"),
            url_mode: false,
            resolving: None,
            error: None,
            dirs_only,
            result: None,
            area: Rect::ZERO,
        };
        b.load(w);
        b
    }

    pub fn load(&mut self, w: &World) {
        let cwd = self.cwd.trim_end_matches('/').to_owned();
        let mut items: Vec<FsEntry> =
            w.fs.iter()
                .filter(|e| {
                    e.path
                        .rsplit_once('/')
                        .map(|(parent, _)| parent == cwd)
                        .unwrap_or(false)
                })
                .cloned()
                .collect();
        items.sort_by(|a, b| b.dir.cmp(&a.dir).then(a.path.cmp(&b.path)));
        let mut rows = vec![ListItem::new("..").meta("parent")];
        for e in &items {
            let name = e.path.rsplit('/').next().unwrap_or(&e.path).to_owned();
            let label = if e.dir { format!("{name}/") } else { name };
            let meta = match (&e.git, e.meta.as_str()) {
                (Some(b), _) => format!("git · {b}"),
                (None, "") => String::new(),
                (None, m) => m.to_owned(),
            };
            let mut li = ListItem::new(&label);
            if !meta.is_empty() {
                li = li.meta(&meta);
            }
            li.disabled = self.dirs_only && !e.dir;
            rows.push(li);
        }
        self.entries = items;
        let cur = self.list.cursor.min(rows.len().saturating_sub(1));
        self.list = ListBox::new(self.list.id, rows, SelectMode::Single).empty_text("Empty folder");
        self.list.cursor = cur;
        self.path = TextInput::new(self.path.id, "Path")
            .plain_label()
            .value(&w.tilde(&self.cwd));
        self.error = None;
    }

    fn open_cursor(&mut self, w: &World) -> Outcome {
        let i = self.list.cursor;
        if i == 0 {
            if let Some((parent, _)) = self.cwd.rsplit_once('/')
                && !parent.is_empty()
            {
                self.cwd = parent.to_owned();
                self.list.cursor = 0;
                self.load(w);
            }
            return Outcome::Changed;
        }
        let Some(e) = self.entries.get(i - 1) else {
            return Outcome::Consumed;
        };
        if e.dir {
            self.cwd = e.path.clone();
            self.list.cursor = 0;
            self.load(w);
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    fn choose(&mut self, w: &World) -> Outcome {
        if self.url_mode {
            let url = self.path.text().trim().to_owned();
            if url.is_empty() {
                self.error = Some("Enter a repository URL".into());
                return Outcome::Changed;
            }
            if !w.github.iter().any(|r| url.contains(&r.full_name)) {
                self.error = Some(format!("Could not resolve {url}: repository not found"));
                return Outcome::Changed;
            }
            self.result = Some(BrowserResult::GitUrl(url));
            return Outcome::Changed;
        }
        let i = self.list.cursor;
        let path = if i == 0 {
            self.cwd.clone()
        } else {
            match self.entries.get(i - 1) {
                Some(e) if e.dir => e.path.clone(),
                Some(_) if self.dirs_only => {
                    self.error = Some("Choose a folder, not a file".into());
                    return Outcome::Changed;
                }
                Some(e) => e.path.clone(),
                None => self.cwd.clone(),
            }
        };
        self.result = Some(BrowserResult::Chosen {
            path,
            readonly: self.readonly.as_ref().is_some_and(|c| c.checked),
        });
        Outcome::Changed
    }

    fn toggle_url_mode(&mut self, w: &World) {
        self.url_mode = !self.url_mode;
        if self.url_mode {
            self.path = TextInput::new(self.path.id, "Git URL")
                .plain_label()
                .placeholder("github.com/org/repo");
            self.git_url.label = "Browse folders".into();
        } else {
            self.git_url.label = "Git URL…".into();
            self.load(w);
        }
        self.error = None;
    }

    pub fn on_key(&mut self, key: &Key, focus: &mut Focus, ring: &FocusRing, w: &World) -> Outcome {
        let cur = focus.current();
        if cur == Some(self.path.id) {
            let editing = self.path.editing;
            let (o, ev) = self.path.on_key(key);
            match ev {
                Some(InputEvent::Committed) => {
                    if self.url_mode {
                        return self.choose(w);
                    }
                    let typed = crate::sim::world::expand(&w.home, self.path.text().trim());
                    if w.fs.iter().any(|e| e.path == typed && e.dir) {
                        self.cwd = typed;
                        self.list.cursor = 0;
                        self.load(w);
                        focus.focus(self.list.id);
                    } else {
                        self.error = Some(format!("Folder not found: {}", self.path.text()));
                    }
                    return Outcome::Changed;
                }
                Some(InputEvent::CommittedTab { backward }) => {
                    if backward {
                        focus.prev(ring)
                    } else {
                        focus.next(ring)
                    }
                    return Outcome::Changed;
                }
                _ => {}
            }
            if o.consumed() || editing {
                return o.or(Outcome::Consumed);
            }
        }
        if cur == Some(self.list.id) {
            match key.code {
                KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') if key.plain() => {
                    return self.open_cursor(w);
                }
                KeyCode::Left | KeyCode::Char('h') | KeyCode::Backspace if key.plain() => {
                    self.list.cursor = 0;
                    return self.open_cursor(w);
                }
                KeyCode::Char(' ') | KeyCode::Char('s') if key.plain() => return self.choose(w),
                KeyCode::Char('g') if key.plain() => {
                    self.toggle_url_mode(w);
                    focus.focus(self.path.id);
                    self.path.begin_edit();
                    return Outcome::Changed;
                }
                _ => {}
            }
            let o = self.list.on_key(key);
            if o.consumed() {
                return o;
            }
        }
        if let Some(c) = self.readonly.as_mut()
            && cur == Some(c.id)
        {
            let o = c.on_key(key);
            if o.consumed() {
                return o;
            }
        }
        if cur == Some(self.git_url.id) {
            let (o, fired) = self.git_url.on_key(key);
            if fired {
                self.toggle_url_mode(w);
                focus.focus(self.path.id);
                self.path.begin_edit();
                return Outcome::Changed;
            }
            if o.consumed() {
                return o;
            }
        }
        if cur == Some(self.cancel.id) {
            let (o, fired) = self.cancel.on_key(key);
            if fired {
                self.result = Some(BrowserResult::Cancelled);
                return Outcome::Changed;
            }
            if o.consumed() {
                return o;
            }
        }
        if cur == Some(self.choose.id) {
            let (o, fired) = self.choose.on_key(key);
            if fired {
                return self.choose(w);
            }
            if o.consumed() {
                return o;
            }
        }
        match key.code {
            KeyCode::Esc => {
                self.result = Some(BrowserResult::Cancelled);
                Outcome::Changed
            }
            KeyCode::Tab => {
                focus.next(ring);
                Outcome::Changed
            }
            KeyCode::BackTab => {
                focus.prev(ring);
                Outcome::Changed
            }
            KeyCode::Enter if cur == Some(self.list.id) => self.open_cursor(w),
            _ => Outcome::Consumed,
        }
    }

    pub fn on_click(
        &mut self,
        id: WidgetId,
        pos: Position,
        focus: &mut Focus,
        w: &World,
    ) -> Outcome {
        if id == self.path.id {
            let was = focus.is(id);
            focus.focus(id);
            return self.path.on_click(pos, was);
        }
        if let Some(i) = self.list.locate(id) {
            focus.focus(self.list.id);
            let was = self.list.cursor == i;
            self.list.cursor = i;
            self.list.chosen = Some(i);
            if was {
                return self.open_cursor(w);
            }
            return Outcome::Changed;
        }
        if id == scrollbar::id_for(self.list.id) {
            return self.list.on_scrollbar(pos);
        }
        if let Some(c) = self.readonly.as_mut()
            && c.id == id
        {
            focus.focus(id);
            return c.on_click();
        }
        if id == self.git_url.id {
            focus.focus(id);
            if self.git_url.on_click() {
                self.toggle_url_mode(w);
                focus.focus(self.path.id);
                self.path.begin_edit();
            }
            return Outcome::Changed;
        }
        if id == self.cancel.id && self.cancel.on_click() {
            self.result = Some(BrowserResult::Cancelled);
            return Outcome::Changed;
        }
        if id == self.choose.id && self.choose.on_click() {
            return self.choose(w);
        }
        Outcome::Consumed
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        self.list.on_wheel(delta)
    }

    pub fn on_paste(&mut self, text: &str) -> Outcome {
        self.path.on_paste(text)
    }

    pub fn is_editing(&self) -> bool {
        self.path.editing
    }

    pub fn tick(&mut self, now_ms: i64) -> Outcome {
        if let Some((url, at)) = &self.resolving
            && now_ms >= *at
        {
            let u = url.clone();
            self.resolving = None;
            self.result = Some(BrowserResult::GitUrl(u));
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    pub fn render(
        &mut self,
        screen: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        stepper: Option<&str>,
    ) {
        let (w, h) = if screen.width < 100 {
            (
                screen.width.saturating_sub(4).min(76),
                screen.height.saturating_sub(4).min(20),
            )
        } else {
            (84, 22)
        };
        let (area, inner) = modal_frame(screen, buf, ctx, w, h, &self.title, None, true);
        self.area = area;
        let t = ctx.theme;
        let bg = t.surface_elevated;
        let mut y = inner.y;
        if let Some(s) = stepper {
            buf.set_string(
                inner.x,
                y,
                truncate(s, inner.width as usize),
                t.muted().bg(bg),
            );
            y += 2;
        }
        self.path
            .render(Rect::new(inner.x - 1, y, inner.width + 1, 2), buf, ctx, bg);
        y += 3;
        let extra: u16 = 2 + u16::from(self.readonly.is_some()) + 1;
        let list_h = inner.bottom().saturating_sub(y + extra + 1);
        if self.url_mode {
            let lines = [
                "Enter a GitHub repository URL. The repository is cloned into the",
                "Construct at launch; the branch is chosen in the mount row.",
            ];
            for (i, l) in lines.iter().enumerate() {
                buf.set_string(inner.x + 1, y + i as u16, l, t.muted().bg(bg));
            }
            if let Some((url, _)) = &self.resolving {
                render_spinner(
                    Rect::new(inner.x + 1, y + 3, inner.width, 1),
                    buf,
                    ctx,
                    &format!("Resolving {url}…"),
                    bg,
                );
            }
        } else {
            self.list.render(
                Rect::new(inner.x - 1, y, inner.width + 1, list_h),
                buf,
                ctx,
                bg,
            );
        }
        y += list_h + 1;
        if let Some(c) = self.readonly.as_mut() {
            c.render(Rect::new(inner.x - 1, y, inner.width + 1, 1), buf, ctx, bg);
            y += 1;
        }
        if let Some(e) = &self.error {
            buf.set_string(
                inner.x,
                y,
                truncate(&format!("! {e}"), inner.width as usize),
                t.error_fg().bg(bg),
            );
        }
        let ay = inner.bottom().saturating_sub(1);
        let widths = [
            self.git_url.width(),
            self.cancel.width(),
            self.choose.width(),
        ];
        let rects = row_layout_right(Rect::new(inner.x, ay, inner.width, 1), &widths, 1);
        self.git_url.render(rects[0], buf, ctx, bg);
        self.cancel.render(rects[1], buf, ctx, bg);
        self.choose.render(rects[2], buf, ctx, bg);
        ctx.hits.register(self.path.id, self.path.area);
        for b in [&self.git_url, &self.cancel, &self.choose] {
            ctx.hits.register(b.id, b.area);
        }
        if let Some(c) = &self.readonly {
            ctx.hits.register(c.id, c.area);
        }
    }

    pub fn initial_focus(&self) -> WidgetId {
        self.list.id
    }
}

// ---------------------------------------------------------------- choice

/// A question with radio options and buttons (mount destination choice,
/// dirty-exit branches, split direction…).
pub struct ChoiceDialog {
    pub title: String,
    pub lines: Vec<(String, Tone)>,
    pub radio: RadioGroup,
    pub buttons: Vec<Button>,
    /// Button index Esc maps to.
    pub cancel_index: usize,
    /// Result: Some(option) when the primary fired, None when cancelled.
    pub result: Option<Option<usize>>,
    /// Which button fired (for owners with several non-cancel buttons).
    pub fired: Option<usize>,
    pub width: u16,
    pub stepper: Option<String>,
    pub area: Rect,
    pub option_tones: Vec<Tone>,
}

impl ChoiceDialog {
    pub fn new(id: WidgetId, title: &str, label: &str, options: &[&str], selected: usize) -> Self {
        Self {
            title: title.to_owned(),
            lines: vec![],
            radio: RadioGroup::new(id.sub("radio"), label, options, selected),
            buttons: vec![
                Button::subtle(id.sub("cancel"), "Cancel"),
                Button::primary(id.sub("ok"), "Next"),
            ],
            cancel_index: 0,
            result: None,
            fired: None,
            width: 54,
            stepper: None,
            area: Rect::ZERO,
            option_tones: vec![],
        }
    }

    pub fn buttons(mut self, buttons: Vec<Button>, cancel_index: usize) -> Self {
        self.buttons = buttons;
        self.cancel_index = cancel_index;
        self
    }

    pub fn line(mut self, text: impl Into<String>, tone: Tone) -> Self {
        self.lines.push((text.into(), tone));
        self
    }

    pub fn stepper(mut self, s: &str) -> Self {
        self.stepper = Some(s.to_owned());
        self
    }

    pub fn width(mut self, w: u16) -> Self {
        self.width = w;
        self
    }

    pub fn option_tones(mut self, tones: Vec<Tone>) -> Self {
        self.option_tones = tones;
        self
    }

    pub fn initial_focus(&self) -> WidgetId {
        self.radio.id
    }

    fn fire(&mut self, i: usize) -> Outcome {
        self.fired = Some(i);
        self.result = Some(if i == self.cancel_index {
            None
        } else {
            Some(self.radio.selected)
        });
        Outcome::Changed
    }

    pub fn on_key(&mut self, key: &Key, focus: &mut Focus, ring: &FocusRing) -> Outcome {
        let cur = focus.current();
        if cur == Some(self.radio.id) {
            if key.is(KeyCode::Enter) {
                let primary = self
                    .buttons
                    .iter()
                    .position(|b| b.kind == ButtonKind::Primary || b.kind == ButtonKind::Danger)
                    .unwrap_or(self.buttons.len() - 1);
                return self.fire(primary);
            }
            let o = self.radio.on_key(key);
            if o.consumed() {
                return o;
            }
        }
        for i in 0..self.buttons.len() {
            if cur == Some(self.buttons[i].id) {
                let (o, fired) = self.buttons[i].on_key(key);
                if fired {
                    return self.fire(i);
                }
                if o.consumed() {
                    return o;
                }
            }
        }
        match key.code {
            KeyCode::Esc => self.fire(self.cancel_index),
            KeyCode::Tab => {
                focus.next(ring);
                Outcome::Changed
            }
            KeyCode::BackTab => {
                focus.prev(ring);
                Outcome::Changed
            }
            KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') => {
                if let Some(i) = self.buttons.iter().position(|b| Some(b.id) == cur) {
                    let n = self.buttons.len();
                    let j = if matches!(key.code, KeyCode::Left | KeyCode::Char('h')) {
                        (i + n - 1) % n
                    } else {
                        (i + 1) % n
                    };
                    focus.focus(self.buttons[j].id);
                }
                Outcome::Changed
            }
            _ => Outcome::Consumed,
        }
    }

    pub fn on_click(&mut self, id: WidgetId, focus: &mut Focus) -> Outcome {
        for i in 0..self.radio.options.len() {
            if self.radio.option_id(i) == id {
                focus.focus(self.radio.id);
                return self.radio.on_click(i);
            }
        }
        for i in 0..self.buttons.len() {
            if self.buttons[i].id == id {
                focus.focus(id);
                if self.buttons[i].on_click() {
                    return self.fire(i);
                }
                return Outcome::Changed;
            }
        }
        Outcome::Consumed
    }

    pub fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let body = self.lines.len() as u16 + if self.lines.is_empty() { 0 } else { 1 };
        let stepper = if self.stepper.is_some() { 2 } else { 0 };
        let h = 2 + 2 + stepper + body + self.radio.height() + 2 + 1;
        let (area, inner) = modal_frame(screen, buf, ctx, self.width, h, &self.title, None, true);
        self.area = area;
        let t = ctx.theme;
        let bg = t.surface_elevated;
        let mut y = inner.y;
        if let Some(s) = &self.stepper {
            buf.set_string(
                inner.x,
                y,
                truncate(s, inner.width as usize),
                t.muted().bg(bg),
            );
            y += 2;
        }
        for (l, tone) in &self.lines {
            buf.set_string(
                inner.x,
                y,
                truncate(l, inner.width as usize),
                Style::new().fg(t.tone(*tone)).bg(bg),
            );
            y += 1;
        }
        if !self.lines.is_empty() {
            y += 1;
        }
        self.radio.render(
            Rect::new(inner.x - 1, y, inner.width + 1, self.radio.height()),
            buf,
            ctx,
            bg,
        );
        // re-tone options (e.g. a destructive last row)
        for (i, tone) in self.option_tones.iter().enumerate() {
            if let Some(r) = self.radio.areas.get(i)
                && *tone != Tone::Normal
                && i != self.radio.cursor
            {
                buf.set_string(
                    r.x + 5,
                    r.y,
                    &self.radio.options[i],
                    Style::new().fg(t.tone(*tone)).bg(bg),
                );
            }
        }
        let ay = inner.bottom().saturating_sub(1);
        let widths: Vec<u16> = self.buttons.iter().map(|b| b.width()).collect();
        let rects = row_layout_right(Rect::new(inner.x, ay, inner.width, 1), &widths, 1);
        for (b, r) in self.buttons.iter_mut().zip(rects) {
            b.render(r, buf, ctx, bg);
        }
        for i in 0..self.radio.options.len() {
            if let Some(r) = self.radio.areas.get(i) {
                ctx.hits.register(self.radio.option_id(i), *r);
            }
        }
        for b in &self.buttons {
            ctx.hits.register(b.id, b.area);
        }
    }
}

// ------------------------------------------------------------------ form

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldValue {
    Text(String),
    Choice(usize),
    Bool(bool),
}

pub type FormValues = Vec<(String, FieldValue)>;

pub enum FieldKindW {
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
    pub kind: FieldKindW,
    pub visible: bool,
}

impl FormField {
    pub fn input(name: &str, input: TextInput) -> Self {
        Self {
            name: name.into(),
            kind: FieldKindW::Input(input),
            visible: true,
        }
    }
    pub fn select(name: &str, select: Select) -> Self {
        Self {
            name: name.into(),
            kind: FieldKindW::Select(select),
            visible: true,
        }
    }
    pub fn check(name: &str, c: Checkbox) -> Self {
        Self {
            name: name.into(),
            kind: FieldKindW::Check(c),
            visible: true,
        }
    }
    pub fn radio(name: &str, r: RadioGroup) -> Self {
        Self {
            name: name.into(),
            kind: FieldKindW::Radio(r),
            visible: true,
        }
    }
    pub fn chooser(name: &str, id: WidgetId, label: &str, value: &str, button: &str) -> Self {
        Self {
            name: name.into(),
            kind: FieldKindW::Chooser {
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
            kind: FieldKindW::Note(lines),
            visible: true,
        }
    }
    pub fn hidden(mut self) -> Self {
        self.visible = false;
        self
    }

    fn height(&self) -> u16 {
        match &self.kind {
            FieldKindW::Input(_) => TextInput::HEIGHT,
            FieldKindW::Select(_) => Select::HEIGHT,
            FieldKindW::Check(_) => 1,
            FieldKindW::Radio(r) => r.height(),
            FieldKindW::Chooser { detail, .. } => 2 + u16::from(detail.is_some()),
            FieldKindW::Note(l) => l.len() as u16,
        }
    }

    pub fn value(&self) -> FieldValue {
        match &self.kind {
            FieldKindW::Input(i) => FieldValue::Text(i.text().to_owned()),
            FieldKindW::Select(s) => FieldValue::Choice(s.selected),
            FieldKindW::Check(c) => FieldValue::Bool(c.checked),
            FieldKindW::Radio(r) => FieldValue::Choice(r.selected),
            FieldKindW::Chooser { value, .. } => FieldValue::Text(value.clone()),
            FieldKindW::Note(_) => FieldValue::Text(String::new()),
        }
    }

    fn focus_id(&self) -> Option<WidgetId> {
        match &self.kind {
            FieldKindW::Input(i) => Some(i.id),
            FieldKindW::Select(s) => Some(s.id),
            FieldKindW::Check(c) => Some(c.id),
            FieldKindW::Radio(r) => Some(r.id),
            FieldKindW::Chooser { button, .. } => Some(button.id),
            FieldKindW::Note(_) => None,
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
                FieldKindW::Input(i) => {
                    *i = TextInput::new(i.id, &i.label.clone())
                        .value(v)
                        .placeholder(&i.placeholder.clone())
                }
                FieldKindW::Chooser { value, .. } => *value = v.to_owned(),
                _ => {}
            }
        }
    }

    pub fn set_chooser(&mut self, name: &str, value: &str, detail: Option<&str>) {
        if let Some(f) = self.field_mut(name)
            && let FieldKindW::Chooser {
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
            && let FieldKindW::Note(l) = &mut f.kind
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
            .any(|f| matches!(&f.kind, FieldKindW::Input(i) if i.editing))
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
            .any(|f| matches!(&f.kind, FieldKindW::Select(s) if s.open))
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
                FieldKindW::Input(i) if cur == Some(i.id) => {
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
                FieldKindW::Select(s) if cur == Some(s.id) => {
                    let (o, ev) = s.on_key(key);
                    if ev.is_some() {
                        self.dirty = true;
                        self.events.push(FormEvent::Changed(name));
                    }
                    if o.consumed() {
                        return o;
                    }
                }
                FieldKindW::Check(c) if cur == Some(c.id) => {
                    let o = c.on_key(key);
                    if o == Outcome::Changed {
                        self.dirty = true;
                        self.events.push(FormEvent::Changed(name));
                    }
                    if o.consumed() {
                        return o;
                    }
                }
                FieldKindW::Radio(r) if cur == Some(r.id) => {
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
                FieldKindW::Chooser { button, .. } if cur == Some(button.id) => {
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

    pub fn on_click(&mut self, id: WidgetId, pos: Position, focus: &mut Focus) -> Outcome {
        for f in self.fields.iter_mut() {
            let name = f.name.clone();
            match &mut f.kind {
                FieldKindW::Input(i) if i.id == id => {
                    let was = focus.is(id);
                    focus.focus(id);
                    return i.on_click(pos, was);
                }
                FieldKindW::Select(s) if s.owns(id) => {
                    focus.focus(s.id);
                    let (o, ev) = s.on_click(id);
                    if ev.is_some() {
                        self.dirty = true;
                        self.events.push(FormEvent::Changed(name));
                    }
                    return o.or(Outcome::Changed);
                }
                FieldKindW::Check(c) if c.id == id => {
                    focus.focus(id);
                    self.dirty = true;
                    self.events.push(FormEvent::Changed(name));
                    return c.on_click();
                }
                FieldKindW::Radio(r) => {
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
                FieldKindW::Chooser { button, .. } if button.id == id => {
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
            if let FieldKindW::Select(s) = &mut f.kind {
                s.dismiss();
            }
        }
        Outcome::Consumed
    }

    pub fn on_paste(&mut self, text: &str) -> Outcome {
        for f in self.fields.iter_mut() {
            if let FieldKindW::Input(i) = &mut f.kind
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
        let (area, inner) = modal_frame(
            screen,
            buf,
            ctx,
            self.width,
            h,
            &self.title,
            self.meta.as_deref(),
            true,
        );
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
                        FieldKindW::Input(i) => i.render(r, buf, ctx, bg),
                        FieldKindW::Select(s) => {
                            if s.open {
                                open_select = Some(fi);
                            } else {
                                s.render(r, buf, ctx, bg);
                            }
                        }
                        FieldKindW::Check(c) => c.render(r, buf, ctx, bg),
                        FieldKindW::Radio(rg) => rg.render(r, buf, ctx, bg),
                        FieldKindW::Chooser {
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
                        FieldKindW::Note(lines) => {
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
                FieldKindW::Input(i) => ctx.hits.register(i.id, i.area),
                FieldKindW::Select(s) => ctx.hits.register(s.id, s.area),
                FieldKindW::Check(c) => ctx.hits.register(c.id, c.area),
                FieldKindW::Radio(r) => {
                    for (i, a) in r.areas.iter().enumerate() {
                        ctx.hits.register(r.option_id(i), *a);
                    }
                }
                FieldKindW::Chooser { button, .. } => ctx.hits.register(button.id, button.area),
                FieldKindW::Note(_) => {}
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
                    && let FieldKindW::Select(s) = &mut f.kind
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

// ------------------------------------------------------- 1Password chain

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpStep {
    Account,
    Vault,
    Item,
    Field,
}

/// Account → vault → item → field, each stage a searchable picker fed by
/// the simulated service; only reference metadata leaves the flow.
pub struct OpFlow {
    pub step: OpStep,
    pub account: Option<String>,
    pub vault: Option<(String, String)>,
    pub item: Option<(String, String)>,
    pub picker: Picker,
    pub result: Option<Option<OpReference>>,
    pub loading_until: Option<i64>,
    pub last_error: Option<OpError>,
    /// Restricts the field stage to concealed fields.
    pub concealed_only: bool,
}

impl OpFlow {
    pub fn new(id: WidgetId, op: &SimOnePassword, now_ms: i64) -> Self {
        let mut f = Self {
            step: OpStep::Account,
            account: None,
            vault: None,
            item: None,
            picker: Picker::new(id.sub("picker"), "1Password"),
            result: None,
            loading_until: None,
            last_error: None,
            concealed_only: true,
        };
        f.picker.width = 64;
        // a single available account is chosen automatically
        let avail: Vec<&crate::sim::onepassword::OpAccount> = op
            .accounts
            .iter()
            .filter(|a| a.state == crate::sim::onepassword::OpAccountState::Available)
            .collect();
        if op.accounts.len() == 1 || (avail.len() == 1 && op.accounts.len() == 1) {
            f.account = Some(op.accounts[0].id.clone());
            f.step = OpStep::Vault;
        }
        f.begin_load(now_ms, op);
        f
    }

    fn crumb(&self) -> String {
        let mut parts = vec!["1Password".to_owned()];
        if let Some(a) = &self.account {
            parts.push(a.split('.').next().unwrap_or(a).to_owned());
        }
        if let Some((_, v)) = &self.vault {
            parts.push(v.clone());
        }
        if let Some((_, i)) = &self.item {
            parts.push(i.clone());
        }
        parts.join(" › ")
    }

    fn begin_load(&mut self, now_ms: i64, op: &SimOnePassword) {
        self.picker.query.clear();
        self.picker.title = match self.step {
            OpStep::Account => "Choose 1Password account".into(),
            OpStep::Vault => "Choose vault".into(),
            OpStep::Item => "Choose item".into(),
            OpStep::Field => "Choose field".into(),
        };
        self.picker.scope = Some(self.crumb());
        self.picker.placeholder = match self.step {
            OpStep::Account => "Search accounts…".into(),
            OpStep::Vault => "Search vaults…".into(),
            OpStep::Item => "Search items…".into(),
            OpStep::Field => "Search fields…".into(),
        };
        self.picker.searchable = true;
        self.picker.status = PickerStatus::Loading(match self.step {
            OpStep::Account => "loading accounts…".into(),
            OpStep::Vault => "loading vaults…".into(),
            OpStep::Item => format!(
                "loading items from {}…",
                self.vault.as_ref().map(|v| v.1.as_str()).unwrap_or("vault")
            ),
            OpStep::Field => format!(
                "loading {}…",
                self.item.as_ref().map(|i| i.1.as_str()).unwrap_or("item")
            ),
        });
        self.picker.set_items(vec![]);
        self.loading_until = Some(now_ms + op.latency_ms);
    }

    /// Called by the app on ticks: finish loading when due.
    pub fn tick(&mut self, now_ms: i64, op: &SimOnePassword) -> Outcome {
        if let Some(t) = self.loading_until
            && now_ms >= t
        {
            self.loading_until = None;
            self.refresh(op);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn set_error(&mut self, e: OpError) {
        let (msg, detail) = match &e {
            OpError::Locked => (
                "1Password is locked".to_owned(),
                Some("Unlock it in the 1Password app, then press r to retry".to_owned()),
            ),
            OpError::AuthorizationRequired { account } => (
                "Jackin is not authorized for 1Password CLI".to_owned(),
                Some(format!(
                    "Approve the request for {account} in 1Password, then press r to retry"
                )),
            ),
            OpError::PermissionDenied { vault } => (
                format!("No access to vault {vault}"),
                Some("Choose another vault".into()),
            ),
            other => (other.message(), None),
        };
        self.picker.status = PickerStatus::Error {
            message: msg,
            detail,
        };
        self.last_error = Some(e);
    }

    /// Rebuild rows for the current step and query.
    pub fn refresh(&mut self, op: &SimOnePassword) {
        self.picker.status = PickerStatus::Ready;
        self.last_error = None;
        let q = self.picker.query.to_lowercase();
        let matches = |s: &str| q.is_empty() || s.to_lowercase().contains(&q);
        let items: Vec<PickerItem> = match self.step {
            OpStep::Account => match op.list_accounts() {
                Ok(accts) => accts
                    .iter()
                    .filter(|a| matches(&a.id) || matches(&a.email))
                    .map(|a| PickerItem {
                        label: a.id.clone(),
                        detail: a.email.clone(),
                        glyph: "▪",
                        group: "",
                        tag: Some(match a.state {
                            crate::sim::onepassword::OpAccountState::Available => "signed in",
                            crate::sim::onepassword::OpAccountState::Locked => "locked",
                            crate::sim::onepassword::OpAccountState::AuthorizationRequired => {
                                "authorize"
                            }
                        }),
                        matched: vec![],
                        disabled: false,
                    })
                    .collect(),
                Err(e) => {
                    self.set_error(e);
                    return;
                }
            },
            OpStep::Vault => match op.list_vaults(self.account.as_deref().unwrap_or("")) {
                Ok(vaults) => vaults
                    .iter()
                    .filter(|v| matches(&v.name))
                    .map(|v| PickerItem {
                        label: v.name.clone(),
                        detail: format!("{} items", v.items.len()),
                        glyph: "▪",
                        group: "",
                        tag: Some(match v.access {
                            crate::sim::onepassword::VaultAccess::ReadWrite => "",
                            crate::sim::onepassword::VaultAccess::ReadOnly => "read-only",
                            crate::sim::onepassword::VaultAccess::Denied => "no access",
                        }),
                        matched: vec![],
                        disabled: false,
                    })
                    .collect(),
                Err(e) => {
                    self.set_error(e);
                    return;
                }
            },
            OpStep::Item => {
                let (vid, _) = self.vault.clone().unwrap_or_default();
                match op.list_items(self.account.as_deref().unwrap_or(""), &vid) {
                    Ok(items) => items
                        .iter()
                        .filter(|i| matches(&i.title))
                        .map(|i| PickerItem {
                            label: i.title.clone(),
                            detail: format!("{} · {} fields", i.category, i.fields.len()),
                            glyph: " ",
                            group: "",
                            tag: None,
                            matched: vec![],
                            disabled: false,
                        })
                        .collect(),
                    Err(e) => {
                        self.set_error(e);
                        return;
                    }
                }
            }
            OpStep::Field => {
                let (vid, _) = self.vault.clone().unwrap_or_default();
                let (iid, _) = self.item.clone().unwrap_or_default();
                match op.list_fields(self.account.as_deref().unwrap_or(""), &vid, &iid) {
                    Ok(fields) => fields
                        .iter()
                        .filter(|f| matches(&f.label))
                        .map(|f| PickerItem {
                            label: f.label.clone(),
                            detail: f.kind.label().to_owned(),
                            glyph: if f.kind == FieldKind::Concealed {
                                "•"
                            } else {
                                " "
                            },
                            group: "",
                            tag: None,
                            matched: vec![],
                            disabled: self.concealed_only && f.kind != FieldKind::Concealed,
                        })
                        .collect(),
                    Err(e) => {
                        self.set_error(e);
                        return;
                    }
                }
            }
        };
        if items.is_empty() {
            self.picker.empty_text = "No matches".into();
        }
        self.picker.set_items(items);
    }

    fn back(&mut self, now_ms: i64, op: &SimOnePassword) -> Outcome {
        match self.step {
            OpStep::Account => {
                self.result = Some(None);
            }
            OpStep::Vault => {
                if op.accounts.len() > 1 {
                    self.step = OpStep::Account;
                    self.account = None;
                    self.begin_load(now_ms, op);
                } else {
                    self.result = Some(None);
                }
            }
            OpStep::Item => {
                self.step = OpStep::Vault;
                self.vault = None;
                self.begin_load(now_ms, op);
            }
            OpStep::Field => {
                self.step = OpStep::Item;
                self.item = None;
                self.begin_load(now_ms, op);
            }
        }
        Outcome::Changed
    }

    fn choose(&mut self, i: usize, now_ms: i64, op: &SimOnePassword) -> Outcome {
        let Some(row) = self.picker.items.get(i).cloned() else {
            return Outcome::Consumed;
        };
        match self.step {
            OpStep::Account => {
                self.account = Some(row.label);
                self.step = OpStep::Vault;
            }
            OpStep::Vault => {
                let vid = op
                    .list_vaults(self.account.as_deref().unwrap_or(""))
                    .ok()
                    .and_then(|vs| {
                        vs.iter()
                            .find(|v| v.name == row.label)
                            .map(|v| v.id.clone())
                    })
                    .unwrap_or_default();
                self.vault = Some((vid, row.label));
                self.step = OpStep::Item;
            }
            OpStep::Item => {
                let (vid, _) = self.vault.clone().unwrap_or_default();
                let iid = op
                    .list_items(self.account.as_deref().unwrap_or(""), &vid)
                    .ok()
                    .and_then(|is| {
                        is.iter()
                            .find(|i| i.title == row.label)
                            .map(|i| i.id.clone())
                    })
                    .unwrap_or_default();
                self.item = Some((iid, row.label));
                self.step = OpStep::Field;
            }
            OpStep::Field => {
                let (vid, _) = self.vault.clone().unwrap_or_default();
                let (iid, _) = self.item.clone().unwrap_or_default();
                match op.reference(
                    self.account.as_deref().unwrap_or(""),
                    &vid,
                    &iid,
                    &row.label,
                ) {
                    Ok(r) => match op.describe(&r) {
                        Ok(_) => self.result = Some(Some(r)),
                        Err(e) => self.set_error(e),
                    },
                    Err(e) => self.set_error(e),
                }
                return Outcome::Changed;
            }
        }
        self.begin_load(now_ms, op);
        Outcome::Changed
    }

    pub fn on_key(&mut self, key: &Key, now_ms: i64, op: &SimOnePassword) -> Outcome {
        if key.is_char('r')
            && self.picker.status != PickerStatus::Ready
            && self.picker.query.is_empty()
        {
            self.begin_load(now_ms, op);
            return Outcome::Changed;
        }
        if self.loading_until.is_some() && !key.is(KeyCode::Esc) {
            return Outcome::Consumed;
        }
        let (o, ev) = self.picker.on_key(key);
        match ev {
            Some(PickerEvent::QueryChanged) => {
                if (self.picker.status == PickerStatus::Ready
                    || matches!(self.picker.status, PickerStatus::Error { .. }))
                    && self.last_error.is_none()
                {
                    self.refresh(op);
                }
                Outcome::Changed
            }
            Some(PickerEvent::Chosen(i)) | Some(PickerEvent::ChosenAlt(i)) => {
                self.choose(i, now_ms, op)
            }
            Some(PickerEvent::Back) | Some(PickerEvent::Cancelled) => self.back(now_ms, op),
            _ => o.or(Outcome::Consumed),
        }
    }

    pub fn on_click(&mut self, id: WidgetId, now_ms: i64, op: &SimOnePassword) -> Outcome {
        if let Some(PickerEvent::Chosen(i)) = self.picker.on_click(id) {
            return self.choose(i, now_ms, op);
        }
        Outcome::Consumed
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        self.picker.on_wheel(delta)
    }

    pub fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let hints = match (self.step, &self.picker.status) {
            (_, PickerStatus::Error { .. }) => "r Retry · Esc Back",
            (OpStep::Field, _) => {
                "↑↓ Move · Enter Save reference · Esc Back to items · only the reference is saved"
            }
            (OpStep::Account, _) => "↑↓ Move · Enter Choose · Esc Cancel",
            _ => "↑↓ Move · Enter Choose · Backspace Back · Esc Back",
        };
        self.picker.render(screen, buf, ctx, hints);
    }
}

// ------------------------------------------------------------------ info

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InfoResult {
    Closed,
    Copy(String),
    /// Extra action button index fired.
    Action(usize),
}

/// Read-only facts with copyable rows and explicit actions.
pub struct InfoDialog {
    pub id: WidgetId,
    pub title: String,
    pub meta: Option<String>,
    pub intro: Vec<(String, Tone)>,
    pub props: PropsList,
    pub detail: Vec<String>,
    pub detail_scroll: ScrollState,
    pub actions: Vec<Button>,
    pub close: Button,
    pub width: u16,
    pub result: Option<InfoResult>,
    pub area: Rect,
    /// Copies of copyable values by row index.
    pub copy_values: Vec<Option<String>>,
    pub error_title: bool,
}

impl InfoDialog {
    pub fn new(id: WidgetId, title: &str, props: Vec<Prop>) -> Self {
        let copy_values = props
            .iter()
            .map(|p| p.copyable.then(|| p.value.clone()))
            .collect();
        Self {
            id,
            title: title.to_owned(),
            meta: None,
            intro: vec![],
            props: PropsList::new(id.sub("props"), props),
            detail: vec![],
            detail_scroll: ScrollState::default(),
            actions: vec![],
            close: Button::secondary(id.sub("close"), "Close"),
            width: 66,
            result: None,
            area: Rect::ZERO,
            copy_values,
            error_title: false,
        }
    }

    pub fn intro(mut self, lines: Vec<(String, Tone)>) -> Self {
        self.intro = lines;
        self
    }

    pub fn detail(mut self, lines: Vec<String>) -> Self {
        self.detail = lines;
        self
    }

    pub fn action(mut self, b: Button) -> Self {
        self.actions.push(b);
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

    pub fn error(mut self) -> Self {
        self.error_title = true;
        self
    }

    pub fn initial_focus(&self) -> WidgetId {
        if self.props.props.is_empty() {
            self.close.id
        } else {
            self.props.id
        }
    }

    pub fn on_key(&mut self, key: &Key, focus: &mut Focus, ring: &FocusRing) -> Outcome {
        let cur = focus.current();
        if cur == Some(self.props.id) {
            let (o, ev) = self.props.on_key(key);
            match ev {
                Some(PropsEvent::Copy(i)) => {
                    if let Some(Some(v)) = self.copy_values.get(i) {
                        self.result = Some(InfoResult::Copy(v.clone()));
                    }
                    return Outcome::Changed;
                }
                Some(PropsEvent::Activate(i)) => {
                    if let Some(Some(v)) = self.copy_values.get(i) {
                        self.result = Some(InfoResult::Copy(v.clone()));
                        return Outcome::Changed;
                    }
                }
                None => {}
            }
            if o.consumed() {
                return o;
            }
            // detail scrolling from the list
            match key.code {
                KeyCode::PageDown => {
                    self.detail_scroll.page_down();
                    return Outcome::Changed;
                }
                KeyCode::PageUp => {
                    self.detail_scroll.page_up();
                    return Outcome::Changed;
                }
                _ => {}
            }
        }
        for i in 0..self.actions.len() {
            if cur == Some(self.actions[i].id) {
                let (o, fired) = self.actions[i].on_key(key);
                if fired {
                    self.result = Some(InfoResult::Action(i));
                    return Outcome::Changed;
                }
                if o.consumed() {
                    return o;
                }
            }
        }
        if cur == Some(self.close.id) {
            let (o, fired) = self.close.on_key(key);
            if fired {
                self.result = Some(InfoResult::Closed);
                return Outcome::Changed;
            }
            if o.consumed() {
                return o;
            }
        }
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.result = Some(InfoResult::Closed);
                Outcome::Changed
            }
            KeyCode::Enter => {
                self.result = Some(InfoResult::Closed);
                Outcome::Changed
            }
            KeyCode::Tab => {
                focus.next(ring);
                Outcome::Changed
            }
            KeyCode::BackTab => {
                focus.prev(ring);
                Outcome::Changed
            }
            KeyCode::Char('y') => {
                // copy the first copyable value
                if let Some(Some(v)) = self.copy_values.iter().find(|v| v.is_some()) {
                    self.result = Some(InfoResult::Copy(v.clone()));
                }
                Outcome::Changed
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.detail_scroll.scroll_by(1);
                Outcome::Changed
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.detail_scroll.scroll_by(-1);
                Outcome::Changed
            }
            _ => Outcome::Consumed,
        }
    }

    pub fn on_click(&mut self, id: WidgetId, pos: Position, focus: &mut Focus) -> Outcome {
        if let Some(i) = self.props.locate(id) {
            focus.focus(self.props.id);
            let (o, _) = self.props.on_click(i);
            if let Some(Some(v)) = self.copy_values.get(i) {
                self.result = Some(InfoResult::Copy(v.clone()));
            }
            return o;
        }
        if id == scrollbar::id_for(self.props.id) {
            return self.props.on_scrollbar(pos);
        }
        if id == scrollbar::id_for(self.id) {
            let track = Rect::new(
                self.area.right() - 2,
                self.area.y + 1,
                1,
                self.area.height.saturating_sub(2),
            );
            self.detail_scroll.scroll_to(scrollbar::offset_for_click(
                track,
                pos,
                &self.detail_scroll,
            ));
            return Outcome::Changed;
        }
        for i in 0..self.actions.len() {
            if self.actions[i].id == id {
                focus.focus(id);
                if self.actions[i].on_click() {
                    self.result = Some(InfoResult::Action(i));
                }
                return Outcome::Changed;
            }
        }
        if id == self.close.id {
            focus.focus(id);
            if self.close.on_click() {
                self.result = Some(InfoResult::Closed);
            }
            return Outcome::Changed;
        }
        Outcome::Consumed
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        if self.detail.is_empty() {
            self.props.on_wheel(delta)
        } else {
            self.detail_scroll.scroll_by(delta as isize);
            Outcome::Changed
        }
    }

    pub fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let intro_h = self.intro.len() as u16 + u16::from(!self.intro.is_empty());
        let props_h = self.props.props.len() as u16;
        let detail_h = if self.detail.is_empty() {
            0
        } else {
            (self.detail.len() as u16).min(8) + 1
        };
        let h = 2 + 2 + intro_h + props_h + detail_h + 2;
        let title = if self.error_title {
            format!("! {}", self.title)
        } else {
            self.title.clone()
        };
        let (area, inner) = modal_frame(
            screen,
            buf,
            ctx,
            self.width,
            h,
            &title,
            self.meta.as_deref(),
            true,
        );
        self.area = area;
        let t = ctx.theme;
        let bg = t.surface_elevated;
        if self.error_title {
            buf.set_string(
                area.x + 3,
                area.y,
                "!",
                t.error_fg().bg(bg).add_modifier(Modifier::BOLD),
            );
        }
        let mut y = inner.y;
        for (l, tone) in &self.intro {
            buf.set_string(
                inner.x,
                y,
                truncate(l, inner.width as usize),
                Style::new().fg(t.tone(*tone)).bg(bg),
            );
            y += 1;
        }
        if !self.intro.is_empty() {
            y += 1;
        }
        let avail = inner.bottom().saturating_sub(y + 2);
        let ph = props_h.min(avail);
        if ph > 0 {
            self.props
                .render(Rect::new(inner.x - 1, y, inner.width + 1, ph), buf, ctx, bg);
            y += ph;
        }
        if !self.detail.is_empty() {
            y += 1;
            let dh = inner.bottom().saturating_sub(y + 2);
            let wrapped: Vec<String> = self
                .detail
                .iter()
                .flat_map(|l| junie_tui::ui::text::wrap(l, inner.width.saturating_sub(2) as usize))
                .collect();
            self.detail_scroll.set_content(wrapped.len());
            self.detail_scroll.set_viewport(dh as usize);
            for (k, i) in self.detail_scroll.visible_range().enumerate() {
                buf.set_string(inner.x, y + k as u16, &wrapped[i], t.secondary().bg(bg));
            }
            if self.detail_scroll.overflows() {
                scrollbar::render_vertical(
                    Rect::new(area.right() - 2, y, 1, dh),
                    buf,
                    ctx,
                    self.id,
                    &self.detail_scroll,
                    true,
                );
            }
        }
        let ay = inner.bottom().saturating_sub(1);
        let mut widths: Vec<u16> = self.actions.iter().map(|b| b.width()).collect();
        widths.push(self.close.width());
        let rects = row_layout_right(Rect::new(inner.x, ay, inner.width, 1), &widths, 1);
        for (i, b) in self.actions.iter_mut().enumerate() {
            b.render(rects[i], buf, ctx, bg);
        }
        self.close.render(rects[self.actions.len()], buf, ctx, bg);
        for b in &self.actions {
            ctx.hits.register(b.id, b.area);
        }
        ctx.hits.register(self.close.id, self.close.area);
        for i in self.props.scroll.visible_range() {
            if let Some(a) = ctx.hits.area_of(self.props.row_id(i)) {
                ctx.hits.register(self.props.row_id(i), a);
            }
        }
    }
}

// ------------------------------------------------------------------ help

pub struct HelpOverlay {
    pub id: WidgetId,
    pub scope: String,
    pub sections: Vec<(String, Vec<(String, String)>)>,
    pub scroll: ScrollState,
    pub area: Rect,
    pub closed: bool,
}

impl HelpOverlay {
    pub fn new(id: WidgetId, scope: &str, sections: Vec<(&str, Vec<(&str, &str)>)>) -> Self {
        Self {
            id,
            scope: scope.to_owned(),
            sections: sections
                .into_iter()
                .map(|(t, rows)| {
                    (
                        t.to_owned(),
                        rows.into_iter()
                            .map(|(k, a)| (k.to_owned(), a.to_owned()))
                            .collect(),
                    )
                })
                .collect(),
            scroll: ScrollState::default(),
            area: Rect::ZERO,
            closed: false,
        }
    }

    pub fn on_key(&mut self, key: &Key) -> Outcome {
        match key.code {
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Enter => {
                self.closed = true;
                Outcome::Changed
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll.scroll_by(1);
                Outcome::Changed
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll.scroll_by(-1);
                Outcome::Changed
            }
            KeyCode::PageDown => {
                self.scroll.page_down();
                Outcome::Changed
            }
            KeyCode::PageUp => {
                self.scroll.page_up();
                Outcome::Changed
            }
            _ => Outcome::Consumed,
        }
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        self.scroll.scroll_by(delta as isize);
        Outcome::Changed
    }

    pub fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let w = screen.width.saturating_sub(4);
        let h = screen.height.saturating_sub(3);
        let (area, inner) = modal_frame(
            screen,
            buf,
            ctx,
            w,
            h,
            "Keyboard shortcuts",
            Some(&self.scope),
            true,
        );
        self.area = area;
        let t = ctx.theme;
        let bg = t.surface_elevated;
        // columns: as many as fit at 36 cells each
        let col_w = 36u16;
        let cols = (inner.width / col_w).clamp(1, 3) as usize;
        // flatten sections into column blocks
        let mut blocks: Vec<Vec<(String, String, bool)>> = vec![];
        for (title, rows) in &self.sections {
            let mut b = vec![(title.clone(), String::new(), true)];
            for (k, a) in rows {
                b.push((k.clone(), a.clone(), false));
            }
            b.push((String::new(), String::new(), false));
            blocks.push(b);
        }
        // distribute blocks round-robin into columns
        let mut columns: Vec<Vec<(String, String, bool)>> = vec![vec![]; cols];
        for (i, b) in blocks.into_iter().enumerate() {
            columns[i % cols].extend(b);
        }
        let total = columns.iter().map(Vec::len).max().unwrap_or(0);
        let body_h = inner.height.saturating_sub(1);
        self.scroll.set_content(total);
        self.scroll.set_viewport(body_h as usize);
        ctx.scrollable(self.id, inner);
        for (ci, col) in columns.iter().enumerate() {
            let x = inner.x + ci as u16 * col_w;
            for (k, i) in self.scroll.visible_range().enumerate() {
                let Some((key, action, heading)) = col.get(i) else {
                    continue;
                };
                let y = inner.y + k as u16;
                if *heading {
                    buf.set_string(x, y, key, t.secondary().bg(bg).add_modifier(Modifier::BOLD));
                } else {
                    buf.set_string(x, y, truncate(key, 12), t.key_hint_key().bg(bg));
                    buf.set_string(
                        x + 13,
                        y,
                        truncate(action, (col_w - 15) as usize),
                        t.key_hint_action().bg(bg),
                    );
                }
            }
        }
        if self.scroll.overflows() {
            scrollbar::render_vertical(
                Rect::new(area.right() - 2, inner.y, 1, body_h),
                buf,
                ctx,
                self.id,
                &self.scroll,
                true,
            );
            let pos = scrollbar::position_label(&self.scroll);
            let pw = width(&pos) as u16;
            buf.set_string(
                area.right().saturating_sub(pw + 4),
                area.y,
                format!(" {pos} "),
                t.faint().bg(bg),
            );
        }
        hint_row(buf, inner, t, "↑↓ Scroll · Esc Close");
    }
}
