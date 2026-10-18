//! An activity tab: one long-lived piece of work with its retained output,
//! state, scope and actions (CONCEPT §8.14, HP14, HP15). Output reaches the
//! viewport by appending, so selection, marks and scrollback identity
//! survive new lines; a retention drop is shown, never hidden. Merged logs
//! keep each service's identity and can be shown or hidden per stream.
//! Monitors can be attached (keys go to the program) and detached with
//! Ctrl+]. Tasks with a live stdin take typed input in input mode (`i`).

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
use junie_tui::widgets::viewport::{Line, Mark, Span, TextViewport, ViewportEvent, line_text};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};

use crate::domain::activity::{
    ActivityKind, ActivityState, InputError, KILL_AFTER_TICKS, LineTone, RETAIN_LINES, key_bytes,
};
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
    /// Typed keys go to the program's stdin.
    pub input: bool,
    /// Output lines mirrored into the viewport so far.
    synced: usize,
    synced_dropped: usize,
    synced_hidden: Vec<String>,
    /// Text of the last mirrored line: a prompt row changes in place.
    synced_last: Option<String>,
    synced_services: Vec<String>,
    follow_ups: Vec<Button>,
    find_query: Option<String>,
    find_matches: Vec<(usize, std::ops::Range<usize>)>,
    find_at: usize,
}

impl ActivityTab {
    pub fn new(id: &str) -> Self {
        let mut chips = ChipBar::new(CHIPS);
        chips.add_label = None;
        Self {
            id: id.into(),
            view: TextViewport::new(VIEW).max_lines(RETAIN_LINES).wrap(true),
            chips,
            attached: false,
            input: false,
            synced: 0,
            synced_dropped: 0,
            synced_hidden: vec![],
            synced_last: None,
            synced_services: vec![],
            follow_ups: vec![],
            find_query: None,
            find_matches: vec![],
            find_at: 0,
        }
    }

    fn line_of(svc_w: usize, entry: &(Option<String>, String, LineTone)) -> Line {
        let (svc, text, tone) = entry;
        let mut line: Line = vec![];
        if let Some(s) = svc {
            line.push(Span::new(format!("{:<w$}  ", s, w = svc_w), Tone::Secondary).bold());
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
    }

    /// Mirror the activity's output into the viewport. Appends push; a
    /// changed last line (a prompt taking its answer) is replaced in place;
    /// only a stream visibility change or a retention drop rebuilds.
    fn sync(&mut self, w: &World) {
        let Some(a) = w.activity(&self.id) else {
            return;
        };
        if self.input && !a.accepts_input() {
            // the program stopped reading: the keyboard is holla's again
            self.input = false;
        }
        let services = a.services();
        if self.chips.chips.len() != services.len() || self.synced_services != services {
            self.chips.chips = services
                .iter()
                .map(|s| {
                    let mut c = Chip::new(s);
                    c.removable = false;
                    c.enabled = !a.hidden_services.contains(s);
                    c
                })
                .collect();
            self.synced_services = services.clone();
        } else {
            for (c, s) in self.chips.chips.iter_mut().zip(&services) {
                c.enabled = !a.hidden_services.contains(s);
            }
        }
        let svc_w = services.iter().map(|s| width(s)).max().unwrap_or(0);
        let visible = |e: &(Option<String>, String, LineTone)| {
            e.0.as_ref().is_none_or(|s| !a.hidden_services.contains(s))
        };
        let mut changed = false;
        // retention: the activity dropped `delta` leading lines since the
        // last sync; the viewport's own cap evicts the same lines as new
        // ones are pushed, so the mirror index moves back by `delta` and
        // nothing retained is rebuilt
        let delta = a.dropped.saturating_sub(self.synced_dropped);
        let rebuild = self.synced_hidden != a.hidden_services
            || a.output.len() + delta < self.synced
            || a.dropped < self.synced_dropped;
        if !rebuild && delta > 0 {
            self.synced = self.synced.saturating_sub(delta);
            self.synced_dropped = a.dropped;
        }
        if rebuild {
            let follow = self.view.follow;
            self.view.set_lines(
                a.output
                    .iter()
                    .filter(|e| visible(e))
                    .map(|e| Self::line_of(svc_w, e))
                    .collect(),
            );
            self.view.follow = follow;
            self.synced = a.output.len();
            self.synced_hidden = a.hidden_services.clone();
            self.synced_dropped = a.dropped;
            changed = true;
        } else {
            if self.synced > 0
                && let Some(last) = a.output.get(self.synced - 1)
                && self.synced_last.as_deref() != Some(last.1.as_str())
                && visible(last)
            {
                self.view.replace_last(Self::line_of(svc_w, last));
                changed = true;
            }
            for e in &a.output[self.synced..] {
                if visible(e) {
                    self.view.push(Self::line_of(svc_w, e));
                    changed = true;
                }
            }
            self.synced = a.output.len();
        }
        self.synced_last = a.output.last().map(|e| e.1.clone());
        if changed && self.find_query.is_some() {
            self.run_find();
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

    fn run_find(&mut self) {
        let q = self.find_query.clone().unwrap_or_default();
        let before = self.find_matches.get(self.find_at).cloned();
        self.find_matches.clear();
        if q.is_empty() {
            self.view.clear_marks();
            return;
        }
        let texts: Vec<String> = self.view.lines().map(|l| line_text(l)).collect();
        for (i, l) in texts.iter().enumerate() {
            for r in junie_tui::ui::text::find_ranges(l, &q, false) {
                self.find_matches.push((i, r));
            }
        }
        // keep the current match when it survives, else start over
        self.find_at = before
            .and_then(|b| self.find_matches.iter().position(|m| *m == b))
            .unwrap_or(0);
        self.apply_marks();
    }

    fn apply_marks(&mut self) {
        let marks: Vec<Mark> = self
            .find_matches
            .iter()
            .enumerate()
            .map(|(k, (line, r))| Mark {
                line: *line,
                range: r.clone(),
                current: k == self.find_at,
            })
            .collect();
        self.view.set_marks(marks);
        if let Some((line, _)) = self.find_matches.get(self.find_at) {
            self.view.set_follow(false);
            self.view.reveal_line(*line);
        }
    }

    fn send(&mut self, bytes: &[u8], w: &mut World, cx: &mut Cx) -> Outcome {
        let tick = w.tick;
        let Some(a) = w.activity_mut(&self.id) else {
            return Outcome::Ignored;
        };
        match a.send_input(bytes, tick) {
            Ok(()) => {
                if bytes == [0x03] {
                    cx.status("Sent ^C · stopping · SIGKILL follows if it lingers");
                    self.input = false;
                } else if bytes == [0x04] {
                    cx.status("Sent EOF");
                }
                Outcome::Changed
            }
            Err(e) => {
                self.input = false;
                cx.status(match e {
                    InputError::Finished => "The program has finished · nothing reads input",
                    InputError::NotStarted => "Queued · it has no stdin until it starts",
                    InputError::NoStdin => "No stdin · a detached monitor takes no input",
                });
                Outcome::Changed
            }
        }
    }

    fn toggle_stream(&mut self, name: String, w: &mut World) {
        if let Some(a) = w.activity_mut(&self.id) {
            if a.hidden_services.contains(&name) {
                a.hidden_services.retain(|s| s != &name);
            } else {
                a.hidden_services.push(name);
            }
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
        } else if l.starts_with("upgrade mise") || l.starts_with("mise upgrade") {
            cx.go(Go::Run {
                item: "upgrade.mise".into(),
                args: vec![],
            });
        } else if let Some(rest) = l.strip_prefix("run ") {
            let id = w
                .items()
                .into_iter()
                .find(|i| i.label.to_lowercase() == rest || i.id == rest)
                .map(|i| i.id);
            match id {
                Some(id) => cx.go(Go::Run {
                    item: id,
                    args: vec![],
                }),
                None => cx.status(format!("{label}: no such action here")),
            }
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
        if self.input && !w.activity(&self.id).is_some_and(|a| a.accepts_input()) {
            // the program stopped reading: the keyboard is holla's again
            self.input = false;
        }
        if self.input {
            // every key is bytes for the program; Esc alone returns the
            // keyboard; Alt chords belong to the shell (tabs, alternatives)
            if key.code == KeyCode::Esc && key.plain() {
                self.input = false;
                cx.status("Keyboard returned to holla · the program keeps running");
                return Outcome::Changed;
            }
            if key.alt() {
                return Outcome::Ignored;
            }
            return match key_bytes(key) {
                Some(bytes) => self.send(&bytes, w, cx),
                None => Outcome::Consumed,
            };
        }
        if let Some(q) = self.find_query.clone() {
            match key.code {
                KeyCode::Esc => {
                    self.find_query = None;
                    self.find_matches.clear();
                    self.view.clear_marks();
                    return Outcome::Changed;
                }
                KeyCode::Enter | KeyCode::Down if !self.find_matches.is_empty() => {
                    self.find_at = (self.find_at + 1) % self.find_matches.len();
                    self.apply_marks();
                    return Outcome::Changed;
                }
                KeyCode::Up if !self.find_matches.is_empty() => {
                    self.find_at =
                        (self.find_at + self.find_matches.len() - 1) % self.find_matches.len();
                    self.apply_marks();
                    return Outcome::Changed;
                }
                KeyCode::Enter | KeyCode::Down | KeyCode::Up => return Outcome::Consumed,
                KeyCode::Backspace => {
                    let mut q = q;
                    q.pop();
                    self.find_query = Some(q);
                    self.run_find();
                    return Outcome::Changed;
                }
                KeyCode::Char(c) if !key.ctrl() && !key.alt() => {
                    let mut q = q;
                    q.push(c);
                    self.find_query = Some(q);
                    self.run_find();
                    return Outcome::Changed;
                }
                _ => {}
            }
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
                    self.toggle_stream(name, w);
                    return Outcome::Changed;
                }
                _ => {}
            }
            if o.consumed() {
                return o;
            }
        }
        let (attachable, accepts_input, state) = w
            .activity(&self.id)
            .map(|a| (a.attachable && a.state.live(), a.accepts_input(), a.state))
            .unwrap_or((false, false, ActivityState::Failed));
        match key.code {
            KeyCode::Enter if attachable && key.plain() => {
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
            KeyCode::Char('i') if key.plain() => {
                if attachable {
                    self.attached = true;
                    cx.status("Attached");
                } else if accepts_input {
                    self.input = true;
                    self.view.set_follow(true);
                    cx.status("Typing goes to the program · Enter sends a line · Ctrl+D EOF · Ctrl+C stops · Esc returns");
                } else {
                    cx.status(match state {
                        ActivityState::Queued => "Queued · it has no stdin until it starts",
                        s if s.live() => "This program takes no input",
                        _ => "The program has finished · nothing reads input",
                    });
                }
                return Outcome::Changed;
            }
            KeyCode::Char('/') if key.plain() => {
                self.find_query = Some(String::new());
                cx.focus.focus(VIEW);
                return Outcome::Changed;
            }
            KeyCode::Char('s') if key.plain() => {
                if let Some(a) = w.activity_mut(&self.id) {
                    match a.state {
                        ActivityState::Cancelling => {
                            let since = a
                                .cancel
                                .as_ref()
                                .map(|c| tick.saturating_sub(c.requested))
                                .unwrap_or(0);
                            cx.status(format!(
                                "Already stopping · SIGTERM sent · SIGKILL in {}",
                                ticks_label(KILL_AFTER_TICKS.saturating_sub(since))
                            ));
                        }
                        s if s.live() => {
                            let name = a.name.clone();
                            a.stop(tick);
                            cx.status(format!("Stopping {name} · SIGTERM sent · output kept"));
                        }
                        _ => cx.status("Already finished"),
                    }
                }
                self.input = false;
                return Outcome::Changed;
            }
            KeyCode::Char('r') if key.plain() => {
                if let Some(a) = w.activity_mut(&self.id) {
                    if a.state.live() {
                        cx.status("Still running · s stops it first");
                        return Outcome::Changed;
                    }
                    a.restart(tick);
                    a.advance(tick);
                    cx.status(format!("Restarted {}", a.name));
                }
                self.synced = 0;
                self.synced_last = None;
                self.view.clear();
                self.find_matches.clear();
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
                if let Some(name) = name {
                    self.toggle_stream(name, w);
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

    fn on_paste(&mut self, text: &str, w: &mut World, _cx: &mut Cx) -> Outcome {
        if self.input {
            let tick = w.tick;
            if let Some(a) = w.activity_mut(&self.id) {
                // pasted text is bytes for the program, never keys for holla
                let _ = a.send_input(text.as_bytes(), tick);
            }
            return Outcome::Changed;
        }
        if let Some(q) = self.find_query.as_mut() {
            q.push_str(text.trim());
            self.run_find();
            return Outcome::Changed;
        }
        Outcome::Consumed
    }

    fn on_click(&mut self, id: WidgetId, pos: Position, w: &mut World, cx: &mut Cx) -> Outcome {
        if self.chips.owns(id) {
            let (o, ev) = self.chips.on_click(id);
            if let Some(ChipEvent::Activate(i)) | Some(ChipEvent::Toggle(i)) = ev {
                let name = self.chips.chips[i].label.clone();
                self.toggle_stream(name, w);
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
        if id == junie_tui::widgets::scrollbar::id_for(VIEW) {
            // a press on the track jumps there; the drag that follows tracks
            return self.view.on_scrollbar(pos);
        }
        Outcome::Ignored
    }

    fn on_drag(&mut self, pressed: WidgetId, pos: Position, _w: &mut World) -> Outcome {
        if pressed == VIEW {
            return self.view.on_drag(pos);
        }
        if pressed == junie_tui::widgets::scrollbar::id_for(VIEW) {
            return self.view.on_scrollbar_drag(pos);
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
        let before = (self.synced, self.view.revision());
        self.sync(w);
        if before != (self.synced, self.view.revision()) {
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
            ActivityState::Cancelling => Tone::Warning,
            ActivityState::Stopped | ActivityState::Detached | ActivityState::Queued => Tone::Muted,
            _ => Tone::Secondary,
        };
        let elapsed = ticks_label(a.duration_ticks(w.tick));
        let mut head = format!(" · {} · {elapsed}", a.state.label());
        if a.state == ActivityState::Queued
            && let Some(after) = &a.after
        {
            let name = w
                .activity(after)
                .map(|p| p.name.clone())
                .unwrap_or(after.clone());
            head.push_str(&format!(" · after {name}"));
        }
        if let Some(e) = a.exit {
            head.push_str(&format!(" · exit {e}"));
        }
        if a.waiting.is_some() {
            head.push_str(" · waiting for input");
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
            ratatui::style::Style::new().fg(t.tone(if a.waiting.is_some() {
                Tone::Warning
            } else {
                state_tone
            })),
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
        // the exact command, then insights or origin
        let mut insight = if a.insights.is_empty() {
            format!("started by {}", a.origin)
        } else {
            a.insights
                .iter()
                .map(|(k, v)| format!("{k} {v}"))
                .collect::<Vec<_>>()
                .join(" · ")
        };
        if let Some(b) = &a.batch {
            insight = format!("{insight} · batch {b}");
        }
        if !a.argv.is_empty() {
            insight = format!(
                "$ {} · {insight}",
                a.argv
                    .iter()
                    .map(|c| c.display())
                    .collect::<Vec<_>>()
                    .join(" && ")
            );
        }
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
        let mut meta = if self.attached {
            "keys go to the program".to_owned()
        } else if self.input {
            "typing goes to stdin · Esc returns".to_owned()
        } else if let Some(q) = &self.find_query {
            format!(
                "find “{q}” · {} of {}",
                if self.find_matches.is_empty() {
                    0
                } else {
                    self.find_at + 1
                },
                self.find_matches.len()
            )
        } else if self.view.is_at_tail() || self.view.follow {
            "following".to_owned()
        } else {
            format!("▲ {} · f follows", self.view.scrollback_depth())
        };
        if a.dropped > 0 {
            meta = format!(
                "{meta} · {} earlier lines dropped · last {} kept",
                a.dropped, RETAIN_LINES
            );
        }
        let title = match &a.kind {
            ActivityKind::Logs { .. } => "merged logs".to_owned(),
            ActivityKind::Monitor { tool } => format!("{tool} screen"),
            ActivityKind::Ssh { alias } => format!("ssh {alias}"),
            _ => "output".to_owned(),
        };
        let panel = Panel::framed(Some(&title))
            .focused(focused || self.input)
            .meta(&meta);
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
        if self.input {
            return vec![
                hint("Type", "To stdin"),
                hint("Enter", "Send line"),
                hint("Ctrl+D", "EOF"),
                hint("Ctrl+C", "Stop"),
                hint("Esc", "Keyboard back"),
            ];
        }
        if self.find_query.is_some() {
            return vec![
                hint("Type", "Find"),
                hint("Enter / ↓", "Next"),
                hint("↑", "Previous"),
                hint("Esc", "Close find"),
            ];
        }
        let a = w.activity(&self.id);
        let live = a.is_some_and(|a| a.state.live());
        let attachable = a.is_some_and(|a| a.attachable && a.state.live());
        let accepts = a.is_some_and(|a| a.accepts_input());
        let waiting = a.is_some_and(|a| a.waiting.is_some());
        let mut v = vec![];
        if focus == Some(CHIPS) {
            v.push(hint("← →", "Stream"));
            v.push(hint("Space", "Show / hide"));
        } else {
            v.push(hint("↑↓", "Scroll"));
            v.push(hint("f", "Follow"));
            v.push(hint("/", "Find"));
            v.push(hint("y", "Copy"));
        }
        if attachable {
            v.push(hint("Enter", "Attach"));
        } else if accepts {
            v.push(hint("i", if waiting { "Answer" } else { "Type input" }));
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
            bits.center = Some(if a.waiting.is_some() {
                StatusItem::new(format!("? input wanted · {elapsed}"), Tone::Warning).priority(7)
            } else {
                match a.state {
                    ActivityState::Running | ActivityState::Detached => {
                        StatusItem::new(elapsed, Tone::Secondary).busy().priority(6)
                    }
                    ActivityState::Cancelling => {
                        StatusItem::new(format!("stopping · {elapsed}"), Tone::Warning)
                            .busy()
                            .priority(6)
                    }
                    ActivityState::Failed => StatusItem::new(
                        format!("! exit {} · {elapsed}", a.exit.unwrap_or(1)),
                        Tone::Error,
                    )
                    .priority(6),
                    _ => StatusItem::new(format!("{} · {elapsed}", a.state.label()), Tone::Muted)
                        .priority(6),
                }
            });
        }
        bits
    }

    fn is_editing(&self) -> bool {
        self.attached || self.input || self.find_query.is_some()
    }

    fn animating(&self, w: &World) -> bool {
        w.activity(&self.id).is_some_and(|a| a.state.live())
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Some(VIEW)
    }

    #[cfg(test)]
    fn as_activity(&mut self) -> Option<&mut ActivityTab> {
        Some(self)
    }

    fn on_esc_top(&mut self, _w: &mut World, cx: &mut Cx) -> Outcome {
        cx.go(Go::Here);
        Outcome::Changed
    }
}

#[cfg(test)]
impl ActivityTab {
    /// The retained output viewport (work counters, marks, selection).
    pub fn view(&self) -> &TextViewport {
        &self.view
    }
}
