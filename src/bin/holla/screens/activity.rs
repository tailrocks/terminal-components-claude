//! An activity tab: one long-lived piece of work with its retained output,
//! state, scope and actions (CONCEPT §8.14). Merged logs keep each service's
//! identity and can be shown or hidden per stream. Monitors can be attached
//! (keys go to the program) and detached with Ctrl+].

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Tone;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::text::{truncate, width};
use junie_tui::widgets::button::Button;
use junie_tui::widgets::chips::{Chip, ChipBar, ChipEvent};
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::statusbar::StatusItem;
use junie_tui::widgets::viewport::{Line, Span, TextViewport, ViewportEvent};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};

use crate::domain::activity::{ActivityKind, ActivityState, LineTone};
use crate::screens::{Cx, Go, Screen, StatusBits, ticks_label};
use crate::sim::world::World;

pub const VIEW: WidgetId = WidgetId::of("activity.view");
pub const CHIPS: WidgetId = WidgetId::of("activity.chips");
const FOLLOW_UP: WidgetId = WidgetId::of("activity.follow");

pub struct ActivityTab {
    pub id: String,
    view: TextViewport,
    chips: ChipBar,
    pub attached: bool,
    rendered_lines: usize,
    rendered_hidden: Vec<String>,
    follow_ups: Vec<Button>,
}

impl ActivityTab {
    pub fn new(id: &str) -> Self {
        let mut chips = ChipBar::new(CHIPS);
        chips.add_label = None;
        Self {
            id: id.into(),
            view: TextViewport::new(VIEW).max_lines(4000).wrap(true),
            chips,
            attached: false,
            rendered_lines: 0,
            rendered_hidden: vec![],
            follow_ups: vec![],
        }
    }

    fn sync(&mut self, w: &World) {
        let Some(a) = w.activity(&self.id) else {
            return;
        };
        let services = a.services();
        if self.chips.chips.len() != services.len() {
            self.chips.chips = services
                .iter()
                .map(|s| {
                    let mut c = Chip::new(s);
                    c.removable = false;
                    c.enabled = !a.hidden_services.contains(s);
                    c
                })
                .collect();
        } else {
            for (c, s) in self.chips.chips.iter_mut().zip(&services) {
                c.enabled = !a.hidden_services.contains(s);
            }
        }
        if self.rendered_lines != a.output.len() || self.rendered_hidden != a.hidden_services {
            let svc_w = services.iter().map(|s| width(s)).max().unwrap_or(0);
            let lines: Vec<Line> = a
                .output
                .iter()
                .filter(|(svc, _, _)| svc.as_ref().is_none_or(|s| !a.hidden_services.contains(s)))
                .map(|(svc, text, tone)| {
                    let mut line: Line = vec![];
                    if let Some(s) = svc {
                        line.push(
                            Span::new(format!("{:<w$}  ", s, w = svc_w), Tone::Secondary).bold(),
                        );
                    }
                    let raw = *tone;
                    let tone = match tone {
                        LineTone::Normal => Tone::Normal,
                        LineTone::Muted => Tone::Muted,
                        LineTone::Warning => Tone::Warning,
                        LineTone::Error => Tone::Error,
                        LineTone::Success => Tone::Secondary,
                    };
                    let mut span = Span::new(text.clone(), tone);
                    if raw == LineTone::Error {
                        span = span.bold();
                    }
                    line.push(span);
                    line
                })
                .collect();
            let follow = self.view.follow;
            self.view.set_lines(lines);
            self.view.follow = follow;
            self.rendered_lines = a.output.len();
            self.rendered_hidden = a.hidden_services.clone();
        }
        if self.follow_ups.len() != a.follow_ups.len() {
            self.follow_ups = a
                .follow_ups
                .iter()
                .enumerate()
                .map(|(i, f)| Button::subtle(FOLLOW_UP.child(i), f))
                .collect();
        }
    }

    fn follow_up(&self, label: &str, w: &World, cx: &mut Cx) {
        let l = label.to_lowercase();
        if l.starts_with("find the process on port") {
            let port = l.rsplit(' ').next().unwrap_or("5173").to_owned();
            cx.go(Go::Run {
                item: "system.port".into(),
                args: vec![("port".into(), port)],
            });
        } else if l.starts_with("retry") {
            cx.go(Go::Restart(self.id.clone()));
        } else if l.starts_with("open the mise") {
            cx.go(Go::Run {
                item: "file.mise".into(),
                args: vec![],
            });
        } else if l.starts_with("run tests again") {
            let id = w
                .items()
                .into_iter()
                .find(|i| i.id.ends_with(":test") || i.id == "mise.task.test")
                .map(|i| i.id);
            match id {
                Some(id) => cx.go(Go::Run {
                    item: id,
                    args: vec![],
                }),
                None => cx.status("No test task here"),
            }
        } else if l.starts_with("review") {
            cx.go(Go::Run {
                item: "git.review".into(),
                args: vec![],
            });
        } else {
            cx.status(format!("{label}: not available in the preview"));
        }
    }
}

impl Screen for ActivityTab {
    fn on_key(&mut self, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome {
        let tick = w.tick;
        if self.attached {
            // crossterm reports Ctrl+] as Ctrl+5
            if key.ctrl() && matches!(key.code, KeyCode::Char(']') | KeyCode::Char('5')) {
                self.attached = false;
                cx.status("Detached · the program keeps running in its tab");
                return Outcome::Changed;
            }
            if key.is_char('q') {
                // the program exits on its own quit key
                if let Some(a) = w.activity_mut(&self.id) {
                    a.state = ActivityState::Stopped;
                    a.ended_tick = Some(tick);
                    a.exit = Some(0);
                    a.output
                        .push((None, "program exited".into(), LineTone::Muted));
                }
                self.attached = false;
                cx.status("Program exited · tab kept with its last screen");
                return Outcome::Changed;
            }
            return Outcome::Consumed;
        }
        // follow-up buttons
        for i in 0..self.follow_ups.len() {
            if cx.focus.is(self.follow_ups[i].id) {
                let (o, fired) = self.follow_ups[i].on_key(key);
                if fired {
                    let label = self.follow_ups[i].label.clone();
                    self.follow_up(&label, w, cx);
                    return Outcome::Changed;
                }
                if o.consumed() {
                    return o;
                }
            }
        }
        if cx.focus.is(CHIPS) {
            let (o, ev) = self.chips.on_key(key);
            match ev {
                Some(ChipEvent::Toggle(i)) | Some(ChipEvent::Activate(i)) => {
                    let name = self.chips.chips[i].label.clone();
                    if let Some(a) = w.activity_mut(&self.id) {
                        if a.hidden_services.contains(&name) {
                            a.hidden_services.retain(|s| s != &name);
                        } else {
                            a.hidden_services.push(name);
                        }
                    }
                    return Outcome::Changed;
                }
                _ => {}
            }
            if o.consumed() {
                return o;
            }
        }
        let attachable = w
            .activity(&self.id)
            .is_some_and(|a| a.attachable && a.state.live());
        match key.code {
            KeyCode::Enter | KeyCode::Char('i') if attachable && key.plain() => {
                self.attached = true;
                let tool = w
                    .activity(&self.id)
                    .map(|a| a.name.clone())
                    .unwrap_or_default();
                if let Some(a) = w.activity_mut(&self.id)
                    && a.state == ActivityState::Detached
                {
                    a.state = ActivityState::Running;
                }
                cx.status(format!("Attached to {tool}"));
                return Outcome::Changed;
            }
            KeyCode::Char('s') if key.plain() => {
                if let Some(a) = w.activity_mut(&self.id) {
                    if a.state.live() {
                        a.stop(tick);
                        cx.status(format!("Stopped {} · output kept", a.name));
                    } else {
                        cx.status("Already finished");
                    }
                }
                return Outcome::Changed;
            }
            KeyCode::Char('r') if key.plain() => {
                if let Some(a) = w.activity_mut(&self.id) {
                    a.restart(tick);
                    a.advance(tick);
                    cx.status(format!("Restarted {}", a.name));
                }
                self.rendered_lines = usize::MAX;
                return Outcome::Changed;
            }
            KeyCode::Char('x') if key.plain() => {
                cx.go(Go::CloseTab);
                return Outcome::Changed;
            }
            KeyCode::Char(c) if key.plain() && c.is_ascii_digit() && c != '0' => {
                let i = c as usize - '1' as usize;
                let name = w
                    .activity(&self.id)
                    .map(|a| a.services())
                    .unwrap_or_default()
                    .get(i)
                    .cloned();
                if let Some(name) = name
                    && let Some(a) = w.activity_mut(&self.id)
                {
                    if a.hidden_services.contains(&name) {
                        a.hidden_services.retain(|s| s != &name);
                    } else {
                        a.hidden_services.push(name);
                    }
                    return Outcome::Changed;
                }
                return Outcome::Ignored;
            }
            _ => {}
        }
        if cx.focus.is(VIEW) {
            let (o, ev) = self.view.on_key(key);
            match ev {
                Some(ViewportEvent::Copy(text)) => cx.copy(text),
                Some(ViewportEvent::FollowChanged(f)) => cx.status(if f {
                    "Following the tail"
                } else {
                    "Follow paused · End resumes"
                }),
                _ => {}
            }
            return o;
        }
        Outcome::Ignored
    }

    fn on_click(&mut self, id: WidgetId, pos: Position, w: &mut World, cx: &mut Cx) -> Outcome {
        if self.chips.owns(id) {
            let (o, ev) = self.chips.on_click(id);
            if let Some(ChipEvent::Activate(i)) | Some(ChipEvent::Toggle(i)) = ev {
                let name = self.chips.chips[i].label.clone();
                if let Some(a) = w.activity_mut(&self.id) {
                    if a.hidden_services.contains(&name) {
                        a.hidden_services.retain(|s| s != &name);
                    } else {
                        a.hidden_services.push(name);
                    }
                }
            }
            cx.focus.focus(CHIPS);
            return o.or(Outcome::Changed);
        }
        for i in 0..self.follow_ups.len() {
            if self.follow_ups[i].id == id {
                cx.focus.focus(id);
                if self.follow_ups[i].on_click() {
                    let label = self.follow_ups[i].label.clone();
                    self.follow_up(&label, w, cx);
                }
                return Outcome::Changed;
            }
        }
        if id == junie_tui::widgets::scrollbar::id_for(VIEW) {
            return self.view.on_scrollbar(pos);
        }
        if id == VIEW {
            cx.focus.focus(VIEW);
            return self.view.on_click(pos).or(Outcome::Changed);
        }
        Outcome::Ignored
    }

    fn on_double_click(
        &mut self,
        id: WidgetId,
        pos: Position,
        _w: &mut World,
        _cx: &mut Cx,
    ) -> Outcome {
        if id == VIEW {
            return self.view.select_word_at(pos);
        }
        Outcome::Ignored
    }

    fn on_press(&mut self, id: WidgetId, pos: Position, _w: &mut World) -> Outcome {
        if id == VIEW {
            return self.view.on_click(pos);
        }
        Outcome::Ignored
    }

    fn on_drag(&mut self, pressed: WidgetId, pos: Position, _w: &mut World) -> Outcome {
        if pressed == VIEW {
            return self.view.on_drag(pos);
        }
        if pressed == junie_tui::widgets::scrollbar::id_for(VIEW) {
            return self.view.on_scrollbar(pos);
        }
        Outcome::Ignored
    }

    fn on_wheel(&mut self, id: WidgetId, delta: i32, _pos: Position, _w: &mut World) -> Outcome {
        if self.view.owns(id) {
            return self.view.on_wheel(delta);
        }
        Outcome::Ignored
    }

    fn on_tick(&mut self, w: &mut World, _cx: &mut Cx) -> Outcome {
        let before = self.rendered_lines;
        self.sync(w);
        if before != self.rendered_lines {
            Outcome::Changed
        } else {
            Outcome::Ignored
        }
    }

    fn enter(&mut self, w: &mut World, _cx: &mut Cx) {
        self.sync(w);
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        self.sync(w);
        let t = ctx.theme;
        let Some(a) = w.activity(&self.id) else {
            junie_tui::widgets::empty::render(
                area,
                buf,
                t,
                &junie_tui::widgets::empty::EmptyState::new("This activity is gone")
                    .hint("Esc returns to Here"),
                t.canvas,
            );
            return;
        };
        // header: name · state · elapsed · exit, then scope on the right
        let state_tone = match a.state {
            ActivityState::Failed => Tone::Error,
            ActivityState::Succeeded => Tone::Secondary,
            ActivityState::Stopped | ActivityState::Detached => Tone::Muted,
            _ => Tone::Secondary,
        };
        let elapsed = ticks_label(a.duration_ticks(w.tick));
        let mut head = format!(" · {} · {elapsed}", a.state.label());
        if let Some(e) = a.exit {
            head.push_str(&format!(" · exit {e}"));
        }
        if self.attached {
            head.push_str(" · attached");
        }
        // an attached monitor's frame says so once; the hint bar owns the key
        buf.set_string(area.x + 1, area.y, &a.name, t.title());
        buf.set_string(
            area.x + 1 + width(&a.name) as u16,
            area.y,
            &head,
            ratatui::style::Style::new().fg(t.tone(state_tone)),
        );
        let right = if a.scope.direction == crate::domain::context::Scope::System {
            a.host.clone()
        } else {
            format!(
                "{} · {} · {}",
                a.scope.word,
                w.location.short(&a.scope.runs_in),
                a.host
            )
        };
        let rw = width(&right) as u16;
        if rw + 2 + width(&a.name) as u16 + width(&head) as u16 + 4 < area.width {
            buf.set_string(
                area.right().saturating_sub(rw + 1),
                area.y,
                &right,
                t.muted(),
            );
        }
        // insights or origin
        let insight = if a.insights.is_empty() {
            format!("started by {}", a.origin)
        } else {
            a.insights
                .iter()
                .map(|(k, v)| format!("{k} {v}"))
                .collect::<Vec<_>>()
                .join(" · ")
        };
        buf.set_string(
            area.x + 1,
            area.y + 1,
            truncate(&insight, area.width.saturating_sub(2) as usize),
            t.muted(),
        );
        let mut y = area.y + 3;
        if !self.chips.chips.is_empty() {
            self.chips.render(
                Rect::new(area.x + 1, y, area.width.saturating_sub(2), 1),
                buf,
                ctx,
                t.canvas,
            );
            y += 2;
        }
        let follow_h: u16 = if !self.follow_ups.is_empty() && !a.state.live() {
            2
        } else {
            0
        };
        let pane = Rect::new(
            area.x,
            y,
            area.width,
            area.bottom().saturating_sub(y + follow_h),
        );
        let focused = ctx.interaction.focused(VIEW);
        let meta = if self.attached {
            "keys go to the program".to_owned()
        } else if self.view.is_at_tail() || self.view.follow {
            "following".to_owned()
        } else {
            format!("▲ {} · f follows", self.view.scrollback_depth())
        };
        let title = match &a.kind {
            ActivityKind::Logs { .. } => "merged logs".to_owned(),
            ActivityKind::Monitor { tool } => format!("{tool} screen"),
            ActivityKind::Ssh { alias } => format!("ssh {alias}"),
            _ => "output".to_owned(),
        };
        let panel = Panel::framed(Some(&title)).focused(focused).meta(&meta);
        let inner = panel.render(pane, buf, t);
        self.view.render(inner, buf, ctx, t.canvas);
        if follow_h > 0 {
            let y = area.bottom().saturating_sub(1);
            buf.set_string(area.x + 1, y, "Next", t.faint());
            let mut x = area.x + 7;
            for b in &mut self.follow_ups {
                let w = b.width();
                if x + w > area.right() {
                    break;
                }
                b.render(Rect::new(x, y, w, 1), buf, ctx, t.canvas);
                x += w + 2;
            }
        }
        if self.attached {
            // the program owns the keyboard: no cursor of ours
            ctx.cursor = None;
        }
    }

    fn hints(&self, focus: Option<WidgetId>, w: &World) -> Vec<Hint> {
        if self.attached {
            return vec![hint("Ctrl+]", "Detach"), hint("q", "Exit the program")];
        }
        let a = w.activity(&self.id);
        let live = a.is_some_and(|a| a.state.live());
        let attachable = a.is_some_and(|a| a.attachable && a.state.live());
        let mut v = vec![];
        if focus == Some(CHIPS) {
            v.push(hint("← →", "Stream"));
            v.push(hint("Space", "Show / hide"));
        } else {
            v.push(hint("↑↓", "Scroll"));
            v.push(hint("f", "Follow"));
            v.push(hint("y", "Copy"));
        }
        if attachable {
            v.push(hint("Enter", "Attach"));
        }
        if live {
            v.push(hint("s", "Stop"));
        } else {
            v.push(hint("r", "Restart"));
        }
        v.push(hint("x", "Close"));
        v.push(hint("Esc", "Here"));
        v
    }

    fn crumb(&self, w: &World) -> String {
        w.activity(&self.id)
            .map(|a| a.name.clone())
            .unwrap_or_default()
    }

    fn status(&self, w: &World) -> StatusBits {
        let mut bits = StatusBits::default();
        // the header names the activity and its state; the status bar keeps
        // only the live fact
        if let Some(a) = w.activity(&self.id) {
            let elapsed = ticks_label(a.duration_ticks(w.tick));
            bits.center = Some(match a.state {
                ActivityState::Running | ActivityState::Detached => {
                    StatusItem::new(elapsed, Tone::Secondary).busy().priority(6)
                }
                ActivityState::Failed => StatusItem::new(
                    format!("! exit {} · {elapsed}", a.exit.unwrap_or(1)),
                    Tone::Error,
                )
                .priority(6),
                _ => StatusItem::new(format!("{} · {elapsed}", a.state.label()), Tone::Muted)
                    .priority(6),
            });
        }
        bits
    }

    fn is_editing(&self) -> bool {
        self.attached
    }

    fn animating(&self, w: &World) -> bool {
        w.activity(&self.id).is_some_and(|a| a.state.live())
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Some(VIEW)
    }

    fn on_esc_top(&mut self, _w: &mut World, cx: &mut Cx) -> Outcome {
        cx.go(Go::Here);
        Outcome::Changed
    }
}
