//! Application shell: layout, event routing, focus/hover bookkeeping,
//! navigation, modal stack, footer hints.

use std::time::{Duration, Instant};

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};

use crate::pages::{Page, PageCtx, PageEvent, Request};
use junie_tui::core::event::{Input, Key, Mouse, MouseKind, Outcome};
use junie_tui::core::focus::{Focus, FocusRing};
use junie_tui::core::hit::HitRegistry;
use junie_tui::core::id::WidgetId;
use junie_tui::theme::{BadgeKind, Theme};
use junie_tui::ui::ctx::{Interaction, RenderCtx, fill};
use junie_tui::widgets::dialog::{Dialog, DialogBody};

pub const MIN_WIDTH: u16 = 72;
pub const MIN_HEIGHT: u16 = 20;

const NAV: WidgetId = WidgetId::of("app.nav");
const HEADER_HELP: WidgetId = WidgetId::of("app.header.help");
const HEADER_INSPECT: WidgetId = WidgetId::of("app.header.inspect");
const HELP_DIALOG: WidgetId = WidgetId::of("app.help");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageId {
    Overview,
    Buttons,
    Forms,
    Inputs,
    TextAreas,
    Panels,
    Sidebars,
    Dialogs,
    Tables,
    Editable,
    Lists,
    Trees,
    Progress,
    Scrolling,
    Editor,
    Grid,
    Chips,
    Pickers,
    Settings,
    TaskRunner,
}

pub struct NavEntry {
    pub id: PageId,
    pub label: &'static str,
    pub section: &'static str,
}

pub const NAV_ENTRIES: &[NavEntry] = &[
    NavEntry {
        id: PageId::Overview,
        label: "Overview",
        section: "Foundations",
    },
    NavEntry {
        id: PageId::Buttons,
        label: "Buttons",
        section: "Components",
    },
    NavEntry {
        id: PageId::Inputs,
        label: "Inputs",
        section: "Components",
    },
    NavEntry {
        id: PageId::TextAreas,
        label: "Text areas",
        section: "Components",
    },
    NavEntry {
        id: PageId::Forms,
        label: "Forms",
        section: "Components",
    },
    NavEntry {
        id: PageId::Lists,
        label: "Lists",
        section: "Components",
    },
    NavEntry {
        id: PageId::Trees,
        label: "Trees",
        section: "Components",
    },
    NavEntry {
        id: PageId::Tables,
        label: "Tables",
        section: "Components",
    },
    NavEntry {
        id: PageId::Editable,
        label: "Editable tables",
        section: "Components",
    },
    NavEntry {
        id: PageId::Panels,
        label: "Panels",
        section: "Components",
    },
    NavEntry {
        id: PageId::Sidebars,
        label: "Sidebars",
        section: "Components",
    },
    NavEntry {
        id: PageId::Dialogs,
        label: "Dialogs",
        section: "Components",
    },
    NavEntry {
        id: PageId::Progress,
        label: "Progress",
        section: "Components",
    },
    NavEntry {
        id: PageId::Scrolling,
        label: "Scrolling",
        section: "Components",
    },
    NavEntry {
        id: PageId::Editor,
        label: "Code editor",
        section: "Components",
    },
    NavEntry {
        id: PageId::Grid,
        label: "Data grid",
        section: "Components",
    },
    NavEntry {
        id: PageId::Chips,
        label: "Chips & selects",
        section: "Components",
    },
    NavEntry {
        id: PageId::Pickers,
        label: "Pickers",
        section: "Components",
    },
    NavEntry {
        id: PageId::Settings,
        label: "Settings",
        section: "Screens",
    },
    NavEntry {
        id: PageId::TaskRunner,
        label: "Task runner",
        section: "Screens",
    },
];

impl PageId {
    pub fn index(self) -> usize {
        NAV_ENTRIES.iter().position(|e| e.id == self).unwrap_or(0)
    }
    pub fn from_name(name: &str) -> Option<Self> {
        // compare on letters and digits only, so "chips & selects",
        // "chips-selects" and "chipsselects" all resolve
        let norm = |s: &str| -> String {
            s.chars()
                .filter(|c| c.is_ascii_alphanumeric())
                .map(|c| c.to_ascii_lowercase())
                .collect()
        };
        let n = norm(name);
        NAV_ENTRIES
            .iter()
            .find(|e| norm(e.label) == n)
            .map(|e| e.id)
    }
}

pub struct App {
    pub theme: Theme,
    pub pages: Vec<Box<dyn Page>>,
    pub page: PageId,
    pub nav_cursor: usize,
    pub focus: Focus,
    pub ring: FocusRing,
    pub hits: HitRegistry,
    pub hover: Option<WidgetId>,
    pub pressed: Option<WidgetId>,
    pub mouse: Option<Position>,
    pub hover_suppressed: bool,
    pub dialog: Option<Dialog>,
    pub inspector: bool,
    pub size: (u16, u16),
    pub tick: u64,
    pub last_key: Option<String>,
    pub status: Option<(String, Instant)>,
    pub flash: Option<(WidgetId, Instant)>,
    pub quit: bool,
    nav_areas: Vec<Rect>,
    layout: ShellLayout,
    /// Focus to restore when the help dialog closes.
    saved_focus: Option<WidgetId>,
}

#[derive(Debug, Default, Clone, Copy)]
struct ShellLayout {
    header: Rect,
    sidebar: Rect,
    main: Rect,
    inspector: Rect,
    footer: Rect,
    too_small: bool,
}

impl App {
    /// Where the navigation sidebar was drawn in the last frame (the visual
    /// baseline test hashes everything but it).
    #[allow(dead_code)]
    pub fn sidebar_area(&self) -> Rect {
        self.layout.sidebar
    }

    pub fn new(theme: Theme) -> Self {
        use crate::pages::*;
        let pages: Vec<Box<dyn Page>> = NAV_ENTRIES
            .iter()
            .map(|e| -> Box<dyn Page> {
                match e.id {
                    PageId::Overview => Box::new(overview::OverviewPage::new()),
                    PageId::Buttons => Box::new(buttons::ButtonsPage::new()),
                    PageId::Forms => Box::new(forms::FormsPage::new()),
                    PageId::Inputs => Box::new(inputs::InputsPage::new()),
                    PageId::TextAreas => Box::new(textareas::TextAreasPage::new()),
                    PageId::Panels => Box::new(panels::PanelsPage::new()),
                    PageId::Sidebars => Box::new(sidebars::SidebarsPage::new()),
                    PageId::Dialogs => Box::new(dialogs::DialogsPage::new()),
                    PageId::Tables => Box::new(tables::TablesPage::new()),
                    PageId::Editable => Box::new(editable::EditablePage::new()),
                    PageId::Lists => Box::new(lists::ListsPage::new()),
                    PageId::Trees => Box::new(trees::TreesPage::new()),
                    PageId::Progress => Box::new(progress::ProgressPage::new()),
                    PageId::Scrolling => Box::new(scrolling::ScrollingPage::new()),
                    PageId::Editor => Box::new(editor::EditorPage::new()),
                    PageId::Grid => Box::new(grid::GridPage::new()),
                    PageId::Chips => Box::new(chips::ChipsPage::new()),
                    PageId::Pickers => Box::new(pickers::PickersPage::new()),
                    PageId::Settings => Box::new(settings::SettingsPage::new()),
                    PageId::TaskRunner => Box::new(taskrunner::TaskRunnerPage::new()),
                }
            })
            .collect();
        let mut focus = Focus::default();
        focus.focus(NAV);
        Self {
            theme,
            pages,
            page: PageId::Overview,
            nav_cursor: 0,
            focus,
            ring: FocusRing::default(),
            hits: HitRegistry::default(),
            hover: None,
            pressed: None,
            mouse: None,
            hover_suppressed: false,
            dialog: None,
            inspector: false,
            size: (0, 0),
            tick: 0,
            last_key: None,
            status: None,
            flash: None,
            quit: false,
            nav_areas: vec![],
            layout: ShellLayout::default(),
            saved_focus: None,
        }
    }

    pub fn goto(&mut self, page: PageId) {
        if self.page != page {
            self.page = page;
            self.nav_cursor = page.index();
            // focus stays on nav if it was there; otherwise first widget on page
            if !self.focus.is(NAV) {
                self.focus.set(None);
            }
        }
    }

    pub fn animating(&self) -> bool {
        let page = &self.pages[self.page.index()];
        page.animating()
            || self.flash.is_some()
            || self
                .dialog
                .as_ref()
                .is_some_and(|d| d.actions.iter().any(|b| b.busy))
    }

    pub fn tick_interval(&self) -> Duration {
        if self.animating() {
            Duration::from_millis(80)
        } else {
            Duration::from_millis(400)
        }
    }

    fn interaction(&self) -> Interaction {
        let flash = match self.flash {
            Some((id, at)) if at.elapsed() < Duration::from_millis(140) => Some(id),
            _ => None,
        };
        Interaction {
            focus: self.focus.current(),
            hover: self.hover,
            pressed: self.pressed,
            flash,
            focus_hidden: false,
            hover_suppressed: self.hover_suppressed,
            tick: self.tick,
        }
    }

    fn set_status(&mut self, s: String) {
        self.status = Some((s, Instant::now()));
    }

    // ------------------------------------------------------------------ input

    pub fn handle(&mut self, input: Input) -> Outcome {
        match input {
            Input::Resize(w, h) => {
                self.size = (w, h);
                Outcome::Changed
            }
            Input::Tick => self.on_tick(),
            Input::Paste(text) => {
                if let Some(d) = self.dialog.as_mut() {
                    return d.on_paste(&text);
                }
                self.dispatch(PageEvent::Paste(text))
            }
            Input::Key(key) => {
                if key.ctrl_char('c') {
                    self.quit = true;
                    return Outcome::Consumed;
                }
                self.last_key = Some(describe_key(&key));
                self.hover_suppressed = true;
                self.on_key(key)
            }
            Input::Mouse(m) => self.on_mouse(m),
        }
    }

    fn on_tick(&mut self) -> Outcome {
        let mut out = Outcome::Ignored;
        if self.animating() {
            self.tick = self.tick.wrapping_add(1);
            out = Outcome::Changed;
        }
        if let Some((_, at)) = self.flash
            && at.elapsed() >= Duration::from_millis(140)
        {
            self.flash = None;
            out = Outcome::Changed;
        }
        if let Some((_, at)) = &self.status
            && at.elapsed() > Duration::from_secs(4)
        {
            self.status = None;
            out = Outcome::Changed;
        }
        if self.dialog.is_none() {
            let page_out = self.dispatch(PageEvent::Tick);
            out = out.or(page_out);
        }
        out
    }

    fn dispatch(&mut self, ev: PageEvent) -> Outcome {
        let mut cx = PageCtx {
            focus: &mut self.focus,
            ring: &self.ring,
            requests: Vec::new(),
        };
        let i = self.page.index();
        let out = self.pages[i].handle(&ev, &mut cx);
        let requests = std::mem::take(&mut cx.requests);
        for r in requests {
            match r {
                Request::OpenDialog(d) => self.open_dialog(*d),
                Request::Status(s) => self.set_status(s),
            }
        }
        out
    }

    fn open_dialog(&mut self, d: Dialog) {
        self.saved_focus = self.focus.current();
        self.focus.set(Some(d.initial_focus));
        self.dialog = Some(d);
        self.hover = None;
        self.pressed = None;
    }

    fn close_dialog(&mut self) -> Outcome {
        let Some(d) = self.dialog.take() else {
            return Outcome::Ignored;
        };
        self.focus.set(self.saved_focus.take());
        if d.id != HELP_DIALOG
            && let Some(result) = d.result
        {
            let id = d.id;
            let value = match &d.body {
                DialogBody::Input(i) => Some(i.text().to_owned()),
                _ => None,
            };
            return self
                .dispatch(PageEvent::DialogClosed { id, result, value })
                .or(Outcome::Changed);
        }
        Outcome::Changed
    }

    fn on_key(&mut self, key: Key) -> Outcome {
        if self.layout.too_small {
            if key.is_char('q') {
                self.quit = true;
            }
            return Outcome::Consumed;
        }
        if let Some(d) = self.dialog.as_mut() {
            let out = d.on_key(&key, &mut self.focus, &self.ring);
            if d.result.is_some() {
                return self.close_dialog().or(out);
            }
            return out.or(Outcome::Consumed);
        }
        // sidebar navigation owns keys while focused
        if self.focus.is(NAV) {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') if key.plain() => {
                    self.nav_cursor = self.nav_cursor.saturating_sub(1);
                    return Outcome::Changed;
                }
                KeyCode::Down | KeyCode::Char('j') if key.plain() => {
                    self.nav_cursor = (self.nav_cursor + 1).min(NAV_ENTRIES.len() - 1);
                    return Outcome::Changed;
                }
                KeyCode::Home | KeyCode::Char('g') if key.plain() => {
                    self.nav_cursor = 0;
                    return Outcome::Changed;
                }
                KeyCode::End | KeyCode::Char('G') => {
                    self.nav_cursor = NAV_ENTRIES.len() - 1;
                    return Outcome::Changed;
                }
                KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Right | KeyCode::Char('l') => {
                    let target = NAV_ENTRIES[self.nav_cursor].id;
                    self.goto(target);
                    if matches!(key.code, KeyCode::Right | KeyCode::Char('l')) {
                        self.focus.set(None);
                        self.focus.next(&self.ring);
                        if self.focus.is(NAV) {
                            self.focus.next(&self.ring);
                        }
                    }
                    return Outcome::Changed;
                }
                _ => {}
            }
        }
        // page first (editing widgets capture everything)
        let activating = matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')) && key.plain();
        let was_editing = self.pages[self.page.index()].editing();
        let out = self.dispatch(PageEvent::Key(key));
        if out.consumed() {
            if activating
                && !was_editing
                && out == Outcome::Changed
                && let Some(f) = self.focus.current()
            {
                self.flash(f);
            }
            return out;
        }
        // global keys
        match key.code {
            KeyCode::Tab => {
                self.focus.next(&self.ring);
                Outcome::Changed
            }
            KeyCode::BackTab => {
                self.focus.prev(&self.ring);
                Outcome::Changed
            }
            KeyCode::Char('q') if key.plain() => {
                self.quit = true;
                Outcome::Consumed
            }
            KeyCode::Char('?') => {
                self.open_help();
                Outcome::Changed
            }
            KeyCode::Char('i') if key.plain() => {
                self.inspector = !self.inspector;
                Outcome::Changed
            }
            KeyCode::Char(']') => {
                let next = (self.page.index() + 1) % NAV_ENTRIES.len();
                self.goto(NAV_ENTRIES[next].id);
                Outcome::Changed
            }
            KeyCode::Char('[') => {
                let n = NAV_ENTRIES.len();
                let prev = (self.page.index() + n - 1) % n;
                self.goto(NAV_ENTRIES[prev].id);
                Outcome::Changed
            }
            KeyCode::Char('0') if key.plain() => {
                self.focus.focus(NAV);
                Outcome::Changed
            }
            KeyCode::Esc => {
                // Esc at top level: return focus to navigation
                if !self.focus.is(NAV) {
                    self.focus.focus(NAV);
                    Outcome::Changed
                } else {
                    Outcome::Consumed
                }
            }
            _ => Outcome::Ignored,
        }
    }

    fn open_help(&mut self) {
        let text = "Tab / Shift+Tab   move keyboard focus\n\
                    ↑ ↓ ← →           move inside the focused control\n\
                    Enter / Space     activate · start editing\n\
                    Esc               cancel editing · back to navigation\n\
                    [ ]               previous / next page\n\
                    0                 jump to navigation\n\
                    i                 toggle state inspector\n\
                    q                 quit\n\n\
                    Mouse: hover to preview, click to focus and activate, wheel to scroll, drag the scrollbar thumb.";
        let mut d = Dialog::confirm(HELP_DIALOG, "Keyboard & mouse", text, "Close");
        d.actions.remove(0);
        d.actions[0].kind = junie_tui::theme::ButtonKind::Secondary;
        d.cancel_index = Some(0);
        d.initial_focus = d.actions[0].id;
        d.width = 70;
        self.open_dialog(d);
    }

    fn on_mouse(&mut self, m: Mouse) -> Outcome {
        self.mouse = Some(m.pos);
        match m.kind {
            MouseKind::Move => {
                let was = self.hover;
                let suppressed = self.hover_suppressed;
                self.hover_suppressed = false;
                self.hover = self.hits.hit(m.pos);
                if self.hover != was || suppressed {
                    Outcome::Changed
                } else {
                    Outcome::Ignored
                }
            }
            MouseKind::Drag => {
                self.hover = self.hits.hit(m.pos);
                if let Some(pressed) = self.pressed {
                    if self.dialog.is_some() {
                        return Outcome::Consumed;
                    }
                    return self.dispatch(PageEvent::Drag {
                        pressed,
                        pos: m.pos,
                    });
                }
                Outcome::Ignored
            }
            MouseKind::Down => {
                let hit = self.hits.hit(m.pos);
                self.pressed = hit;
                self.hover = hit;
                let Some(id) = hit else {
                    if self.dialog.is_some() {
                        return Outcome::Consumed;
                    }
                    return Outcome::Ignored;
                };
                if self.dialog.is_some() {
                    return Outcome::Changed;
                }
                if let Some(i) = self.nav_index_at(id) {
                    self.focus.focus(NAV);
                    self.nav_cursor = i;
                    return Outcome::Changed;
                }
                if self.ring.contains(id) {
                    self.focus.focus(id);
                }
                Outcome::Changed
            }
            MouseKind::Up => {
                let hit = self.hits.hit(m.pos);
                let pressed = self.pressed.take();
                let Some(id) = hit else {
                    if let Some(d) = self.dialog.as_mut()
                        && pressed.is_none()
                    {
                        let out = d.on_click_outside();
                        if d.result.is_some() {
                            return self.close_dialog().or(out);
                        }
                    }
                    return Outcome::Changed;
                };
                if pressed != Some(id) {
                    return Outcome::Changed;
                }
                if let Some(d) = self.dialog.as_mut() {
                    let out = d.on_click(id, m.pos, &mut self.focus);
                    if d.result.is_some() {
                        return self.close_dialog().or(out);
                    }
                    return out.or(Outcome::Changed);
                }
                if id == HEADER_HELP {
                    self.open_help();
                    return Outcome::Changed;
                }
                if id == HEADER_INSPECT {
                    self.inspector = !self.inspector;
                    return Outcome::Changed;
                }
                if let Some(i) = self.nav_index_at(id) {
                    self.nav_cursor = i;
                    self.goto(NAV_ENTRIES[i].id);
                    self.focus.focus(NAV);
                    return Outcome::Changed;
                }
                self.flash(id);
                self.dispatch(PageEvent::Click { id, pos: m.pos })
                    .or(Outcome::Changed)
            }
            MouseKind::WheelLeft | MouseKind::WheelRight => Outcome::Ignored,
            MouseKind::WheelUp | MouseKind::WheelDown => {
                let delta = if m.kind == MouseKind::WheelUp { -3 } else { 3 };
                if self.dialog.is_some() {
                    return Outcome::Consumed;
                }
                let Some(id) = self.hits.hit_scroll(m.pos) else {
                    return Outcome::Ignored;
                };
                if id == NAV || self.nav_index_at(id).is_some() {
                    return Outcome::Consumed;
                }
                self.dispatch(PageEvent::Wheel { id, delta })
            }
        }
    }

    fn nav_index_at(&self, id: WidgetId) -> Option<usize> {
        (0..NAV_ENTRIES.len()).find(|&i| NAV.child(i) == id)
    }

    /// Flash a widget as pressed (keyboard activation feedback).
    pub fn flash(&mut self, id: WidgetId) {
        self.flash = Some((id, Instant::now()));
    }

    // ----------------------------------------------------------------- render

    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        self.size = (area.width, area.height);
        let theme = self.theme;
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let interaction = self.interaction();
        let mut cursor = None;
        {
            let buf = frame.buffer_mut();
            let mut ctx = RenderCtx::new(&theme, interaction, &mut hits, &mut ring);
            self.draw(area, buf, &mut ctx);
            cursor = cursor.or(ctx.cursor);
        }
        self.hits = hits;
        self.ring = ring;
        // keep focus valid against the freshly built ring
        if self.dialog.is_none() {
            if !self.layout.too_small
                && !self.focus.current().is_some_and(|c| self.ring.contains(c))
            {
                self.focus.set(self.ring.first());
            }
        } else {
            self.focus.ensure_valid(&self.ring);
        }
        if let Some(pos) = cursor {
            frame.set_cursor_position(pos);
        }
    }

    fn compute_layout(&self, area: Rect) -> ShellLayout {
        if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
            return ShellLayout {
                too_small: true,
                ..Default::default()
            };
        }
        let header = Rect::new(area.x, area.y, area.width, 1);
        let footer = Rect::new(area.x, area.bottom() - 1, area.width, 1);
        let body = Rect::new(
            area.x,
            area.y + 2,
            area.width,
            area.height.saturating_sub(4),
        );
        let wide = area.width >= 110;
        let sidebar_w = if wide { 24 } else { 19 };
        let inspector_w = if self.inspector && area.width >= 100 {
            30
        } else {
            0
        };
        let sidebar = Rect::new(body.x, body.y, sidebar_w, body.height);
        let main_x = body.x + sidebar_w + 2;
        let main_w = body
            .width
            .saturating_sub(sidebar_w + 2 + inspector_w + if inspector_w > 0 { 2 } else { 0 });
        let main = Rect::new(main_x, body.y, main_w, body.height);
        let inspector = if inspector_w > 0 {
            Rect::new(main.right() + 2, body.y, inspector_w, body.height)
        } else {
            Rect::ZERO
        };
        ShellLayout {
            header,
            sidebar,
            main,
            inspector,
            footer,
            too_small: false,
        }
    }

    fn draw(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = self.theme;
        fill(buf, area, t.base());
        let layout = self.compute_layout(area);
        self.layout = layout;
        if layout.too_small {
            self.draw_too_small(area, buf);
            return;
        }
        self.draw_header(layout.header, buf, ctx);
        self.draw_sidebar(layout.sidebar, buf, ctx);
        self.draw_main(layout.main, buf, ctx);
        if !layout.inspector.is_empty() {
            self.draw_inspector(layout.inspector, buf, ctx);
        }
        self.draw_footer(layout.footer, buf, ctx);
        if let Some(d) = self.dialog.as_mut() {
            d.render(area, buf, ctx);
        }
    }

    fn draw_too_small(&self, area: Rect, buf: &mut Buffer) {
        let t = self.theme;
        let lines = [
            ("Junie Design system", t.title()),
            ("Terminal too small", t.secondary()),
            (
                &*format!(
                    "Need {MIN_WIDTH}×{MIN_HEIGHT}, have {}×{}",
                    area.width, area.height
                ),
                t.muted(),
            ),
            ("q Quit", t.faint()),
        ];
        let y0 = area.y + area.height.saturating_sub(5) / 2;
        for (i, (text, style)) in lines.iter().enumerate() {
            let w = junie_tui::ui::text::width(text) as u16;
            let x = area.x + area.width.saturating_sub(w) / 2;
            let y = y0 + i as u16 + if i == 3 { 1 } else { 0 };
            if y < area.bottom() {
                buf.set_string(x, y, text, style.bg(t.canvas));
            }
        }
    }

    fn draw_header(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = self.theme;
        let mut x = area.x + 1;
        buf.set_string(x, area.y, "▪", t.accent_fg());
        x += 2;
        buf.set_string(x, area.y, "Junie", t.title());
        x += 6;
        buf.set_string(x, area.y, "Design system", t.secondary());
        x += 14;
        let entry = &NAV_ENTRIES[self.page.index()];
        let crumb = format!("/ {} / {}", entry.section, entry.label);
        buf.set_string(x, area.y, &crumb, t.muted());
        let crumb_w = junie_tui::ui::text::width(&crumb) as u16;
        // right side: capability + actions
        let cap = format!("{} · {}×{}", t.level.label(), self.size.0, self.size.1);
        let mut rx = area.right().saturating_sub(1);
        let help = " ? Help ";
        let insp = if self.inspector {
            " i Inspector · on "
        } else {
            " i Inspector "
        };
        for (label, id) in [(help, HEADER_HELP), (insp, HEADER_INSPECT)] {
            let w = junie_tui::ui::text::width(label) as u16;
            rx = rx.saturating_sub(w);
            let hovered = ctx.interaction.hovered(id);
            let style = if hovered {
                t.primary().bg(t.surface)
            } else {
                t.muted()
            };
            buf.set_string(rx, area.y, label, style);
            ctx.clickable(id, Rect::new(rx, area.y, w, 1));
            rx = rx.saturating_sub(1);
        }
        let cw = junie_tui::ui::text::width(&cap) as u16;
        if rx > x + crumb_w + cw + 2 {
            buf.set_string(rx.saturating_sub(cw + 1), area.y, &cap, t.faint());
        }
    }

    fn draw_sidebar(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = self.theme;
        let focused = ctx.interaction.focused(NAV);
        let mut y = area.y;
        let mut section = "";
        self.nav_areas.clear();
        let sections = 3u16;
        let compact = area.height < NAV_ENTRIES.len() as u16 + sections * 2 - 1;
        for (i, e) in NAV_ENTRIES.iter().enumerate() {
            if e.section != section && !compact {
                if y > area.y {
                    y += 1;
                }
                if y >= area.bottom() {
                    break;
                }
                buf.set_string(area.x + 3, y, e.section, t.faint());
                section = e.section;
                y += 1;
            } else if e.section != section && compact && y > area.y {
                // compact: a one-cell gap between groups, no labels
                section = e.section;
            }
            if y >= area.bottom() {
                break;
            }
            let row = Rect::new(area.x, y, area.width, 1);
            let rid = NAV.child(i);
            let mut s = ctx.state(rid);
            s.focused = focused && i == self.nav_cursor;
            let current = e.id == self.page;
            let st = t.row(s, t.canvas);
            fill(buf, row, st);
            buf.set_string(row.x, y, "▎", t.gutter(s, st.bg.unwrap_or(t.canvas), false));
            if current {
                let ms = st.fg(t.accent);
                buf.set_string(row.x + 1, y, "›", ms);
            }
            let label_style = if current || s.focused || s.hovered {
                st.fg(t.text_primary)
            } else {
                st.fg(t.text_secondary)
            };
            buf.set_string(
                row.x + 3,
                y,
                junie_tui::ui::text::fit(e.label, area.width.saturating_sub(4) as usize),
                label_style,
            );
            ctx.clickable(rid, row);
            self.nav_areas.push(row);
            y += 1;
        }
        // the sidebar as a whole is one focus stop
        if !ctx.inert {
            ctx.ring.register(NAV);
            ctx.hits.register_scroll(NAV, area);
        }
    }

    fn draw_main(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = self.theme;
        let i = self.page.index();
        let page = self.pages[i].as_mut();
        // page title row
        buf.set_string(area.x, area.y, page.title(), t.title());
        let tw = junie_tui::ui::text::width(page.title()) as u16;
        if area.width > tw + 4 {
            let blurb = junie_tui::ui::text::truncate(page.blurb(), (area.width - tw - 3) as usize);
            buf.set_string(area.x + tw + 2, area.y, &blurb, t.muted());
        }
        let body = Rect::new(
            area.x,
            area.y + 2,
            area.width,
            area.height.saturating_sub(2),
        );
        page.render(body, buf, ctx);
    }

    fn draw_inspector(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = self.theme;
        let panel = junie_tui::widgets::panel::Panel::card(Some("State"));
        let inner = panel.render(area, buf, &t);
        let bg = t.surface;
        let page = &self.pages[self.page.index()];
        let mode = if self.dialog.is_some() {
            "MODAL"
        } else if page.editing() {
            "EDIT"
        } else {
            "NAV"
        };
        let fmt_rect = |r: Option<Rect>| match r {
            Some(r) => format!("{}·{} {}×{}", r.x, r.y, r.width, r.height),
            None => "—".to_owned(),
        };
        let focus_area = self.focus.current().and_then(|f| self.hits.area_of(f));
        let hover_area = self.hover.and_then(|h| self.hits.area_of(h));
        let rows: Vec<(&str, String)> = vec![
            ("mode", mode.to_owned()),
            ("focus", fmt_rect(focus_area)),
            (
                "hover",
                if self.hover_suppressed {
                    "suppressed".into()
                } else {
                    fmt_rect(hover_area)
                },
            ),
            (
                "pressed",
                fmt_rect(self.pressed.and_then(|p| self.hits.area_of(p))),
            ),
            (
                "mouse",
                self.mouse
                    .map(|p| format!("{}·{}", p.x, p.y))
                    .unwrap_or("—".into()),
            ),
            ("last key", self.last_key.clone().unwrap_or("—".into())),
            (
                "focus ring",
                format!("{} stops", self.ring.reachable().len()),
            ),
            ("hit regions", format!("{}", self.hits_len())),
            ("tick", format!("{}", self.tick)),
            ("colors", t.level.label().to_owned()),
        ];
        for (i, (k, v)) in rows.iter().enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.bottom() {
                break;
            }
            buf.set_string(inner.x, y, format!("{k:<11}"), t.muted().bg(bg));
            let vs = t.primary().bg(bg);
            buf.set_string(
                inner.x + 12,
                y,
                junie_tui::ui::text::truncate(v, inner.width.saturating_sub(12) as usize),
                vs,
            );
        }
        let _ = ctx;
    }

    fn hits_len(&self) -> usize {
        self.hits.len()
    }

    fn draw_footer(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = self.theme;
        let page = &self.pages[self.page.index()];
        let mut x = area.x + 1;
        let mut hints: Vec<(String, String)> = Vec::new();
        if let Some(d) = self.dialog.as_ref() {
            if d.is_editing() {
                hints.push(("Enter".into(), "Confirm".into()));
                hints.push(("Esc".into(), "Cancel".into()));
            } else {
                hints.push(("← →".into(), "Choose".into()));
                hints.push(("Enter".into(), "Confirm".into()));
                hints.push(("Esc".into(), "Cancel".into()));
                if matches!(d.body, DialogBody::Text(_)) && d.id != HELP_DIALOG {
                    hints.push(("y / n".into(), "Quick answer".into()));
                }
            }
        } else if self.focus.is(NAV) {
            hints.push(("↑ ↓".into(), "Move".into()));
            hints.push(("Enter".into(), "Open".into()));
            hints.push(("Tab".into(), "Into page".into()));
            hints.push(("q".into(), "Quit".into()));
        } else {
            for (k, v) in page.hints(self.focus.current()) {
                hints.push((k.into(), v.into()));
            }
            if !page.editing() {
                hints.push(("Tab".into(), "Next".into()));
            }
        }
        if page.editing() && self.dialog.is_none() {
            let badge = " EDIT ";
            buf.set_string(x, area.y, badge, t.badge(BadgeKind::Edit));
            x += badge.len() as u16 + 2;
        }
        let right_reserved = 14u16;
        for (k, v) in &hints {
            let kw = junie_tui::ui::text::width(k) as u16;
            let w = kw + 1 + junie_tui::ui::text::width(v) as u16 + 2;
            if x + w + right_reserved > area.right() {
                break;
            }
            buf.set_string(x, area.y, k, t.key_hint_key());
            buf.set_string(x + kw + 1, area.y, v, t.key_hint_action());
            x += w;
        }
        // right: status message or help hint
        if let Some((s, _)) = &self.status {
            let w = junie_tui::ui::text::width(s) as u16;
            if area.right() > w + 1 {
                buf.set_string(area.right() - w - 1, area.y, s, t.secondary());
            }
        }
        let _ = ctx;
    }
}

fn describe_key(k: &Key) -> String {
    let mut s = String::new();
    if k.ctrl() {
        s.push_str("Ctrl+");
    }
    if k.alt() {
        s.push_str("Alt+");
    }
    if k.shift() && !matches!(k.code, KeyCode::Char(_)) {
        s.push_str("Shift+");
    }
    match k.code {
        KeyCode::Char(' ') => s.push_str("Space"),
        KeyCode::Char(c) => s.push(c),
        KeyCode::BackTab => s.push_str("Shift+Tab"),
        other => s.push_str(&format!("{other:?}")),
    }
    s
}
