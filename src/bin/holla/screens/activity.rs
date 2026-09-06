//! Activity: a named piece of long-running work with retained output. The
//! page is the multiplexer body — scope, state and output survive
//! navigation; digits switch fast, `0` returns home. Multi-service log
//! streams keep service identity per line.

use std::collections::HashMap;

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Tone;
use junie_tui::ui::ctx::{RenderCtx, fill};
use junie_tui::ui::text::{truncate, width};
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::props::{self, Prop};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};

use crate::domain::activity::{Activity, ActivityState};
use crate::screens::{Cx, Go, Screen};
use crate::sim::world::World;

/// Strip hit id base: one child per activity (`STRIP.child(activity id)`).
pub const STRIP: WidgetId = WidgetId::of("strip.act");

/// The activity a strip hit belongs to, if any.
pub fn hit_activity(id: WidgetId, w: &World) -> Option<u32> {
    ordered(w)
        .iter()
        .find(|a| STRIP.child(a.id as usize) == id)
        .map(|a| a.id)
}

/// One tone per state, shared by the strip and the page (§consistency).
pub fn state_tone(s: ActivityState) -> Tone {
    match s {
        ActivityState::Running => Tone::Success,
        ActivityState::Waiting => Tone::Warning,
        ActivityState::Succeeded => Tone::Muted,
        ActivityState::Failed => Tone::Error,
        ActivityState::Detached => Tone::Faint,
    }
}

/// Strip/page glyph per state (`●` lives, `…` waits, `✗` failed).
pub fn state_glyph(s: ActivityState) -> &'static str {
    match s {
        ActivityState::Running => "●",
        ActivityState::Waiting => "…",
        ActivityState::Succeeded => "✓",
        ActivityState::Failed => "✗",
        ActivityState::Detached => "−",
    }
}

/// Activities in strip order: id ascending, digit `n` jumps to the n-th.
pub fn ordered(w: &World) -> Vec<&Activity> {
    let mut v: Vec<&Activity> = w.activities.iter().collect();
    v.sort_by_key(|a| a.id);
    v
}

pub fn elapsed_label(a: &Activity, now_ms: i64) -> String {
    let ms = now_ms.saturating_sub(a.started_ms);
    let mins = ms / 60_000;
    if mins < 1 {
        "just now".to_owned()
    } else if mins < 60 {
        format!("{mins}m ago")
    } else if mins < 60 * 24 {
        format!("{}h ago", mins / 60)
    } else {
        format!("{}d ago", mins / (60 * 24))
    }
}

#[derive(Default)]
pub struct ActivityScreen {
    /// The activity this page shows (set by the shell on Go::Activity).
    pub current: Option<u32>,
    /// Retained scroll per activity: leaving and returning loses nothing.
    scrolls: HashMap<u32, usize>,
}

impl ActivityScreen {
    pub fn show(&mut self, id: u32) {
        self.current = Some(id);
    }

    fn scroll_by(&mut self, w: &World, delta: i32) {
        let Some(id) = self.current else { return };
        let max = self
            .current(w)
            .map(|a| a.lines.len())
            .unwrap_or(0)
            .saturating_sub(1);
        let e = self.scrolls.entry(id).or_insert(max);
        *e = (*e as i32 + delta).clamp(0, max as i32) as usize;
    }

    fn current<'a>(&self, w: &'a World) -> Option<&'a Activity> {
        w.activities.iter().find(|a| Some(a.id) == self.current)
    }

    /// Service tone for a merged log prefix (`api | …`): deterministic per
    /// name, so a service keeps its color across the stream.
    fn service_tone(name: &str) -> Tone {
        const PALETTE: [Tone; 4] = [Tone::Success, Tone::Warning, Tone::Normal, Tone::Secondary];
        let h: usize = name.bytes().map(|b| b as usize).sum();
        PALETTE[h % PALETTE.len()]
    }
}

impl Screen for ActivityScreen {
    fn on_key(&mut self, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome {
        match key.code {
            KeyCode::Char('0') if key.plain() => {
                cx.go(Go::Home);
                Outcome::Changed
            }
            KeyCode::Char(c) if key.plain() && c.is_ascii_digit() => {
                let n = (c as usize) - ('0' as usize);
                match ordered(w).get(n - 1) {
                    Some(a) => {
                        let id = a.id;
                        cx.go(Go::Activity(id));
                    }
                    None => cx.status(format!("No activity {n}")),
                }
                Outcome::Changed
            }
            KeyCode::Up | KeyCode::Down => {
                let delta: i32 = if key.code == KeyCode::Up { -1 } else { 1 };
                self.scroll_by(w, delta);
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    fn on_wheel(&mut self, _id: WidgetId, delta: i32, _pos: Position, w: &mut World) -> Outcome {
        self.scroll_by(w, delta);
        Outcome::Changed
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        fill(buf, area, t.base());
        let Some(id) = self.current else { return };
        let Some(a) = self.current(w) else { return };
        // title: glyph + name + state
        buf.set_string(
            area.x,
            area.y,
            state_glyph(a.state),
            Style::new().fg(t.tone(state_tone(a.state))),
        );
        buf.set_string(
            area.x + 2,
            area.y,
            truncate(&a.name, area.width.saturating_sub(4) as usize),
            t.primary().add_modifier(Modifier::BOLD),
        );
        let facts = vec![
            Prop::new("Scope", &a.scope),
            Prop::new(
                "State",
                format!("{} · {}", a.state.label(), elapsed_label(a, w.now_ms())),
            ),
            Prop::new("Output", format!("{} retained lines", a.lines.len())),
        ];
        props::render(
            Rect::new(area.x, area.y + 2, area.width, 3),
            buf,
            t,
            &facts,
            t.canvas,
        );
        // retained output; scroll offset sticks per activity
        let top = area.y + 6;
        let view_h = area.bottom().saturating_sub(top) as usize;
        let max = a
            .lines
            .len()
            .saturating_sub(view_h.min(a.lines.len().max(1)));
        let scroll = *self.scrolls.get(&id).unwrap_or(&max).min(&max);
        self.scrolls.insert(id, scroll);
        let merged = a.lines.iter().any(|l| l.contains(" | "));
        for (i, line) in a.lines.iter().skip(scroll).take(view_h).enumerate() {
            let y = top + i as u16;
            if merged && let Some((svc, rest)) = line.split_once(" | ") {
                buf.set_string(
                    area.x,
                    y,
                    truncate(svc, 12),
                    Style::new().fg(t.tone(Self::service_tone(svc.trim()))),
                );
                let sw = (width(svc) as u16).min(12);
                buf.set_string(area.x + sw, y, " | ", t.faint());
                buf.set_string(
                    area.x + sw + 3,
                    y,
                    truncate(rest, area.width.saturating_sub(sw + 3) as usize),
                    t.secondary(),
                );
            } else {
                buf.set_string(
                    area.x,
                    y,
                    truncate(line, area.width as usize),
                    t.secondary(),
                );
            }
        }
        if a.lines.len() > view_h {
            let label = format!(
                "{}–{} of {}",
                scroll + 1,
                (scroll + view_h).min(a.lines.len()),
                a.lines.len()
            );
            let lw = width(&label) as u16;
            if lw + 2 < area.width {
                buf.set_string(area.right().saturating_sub(lw), area.y, &label, t.faint());
            }
        }
    }

    fn hints(&self, _focus: Option<WidgetId>, w: &World) -> Vec<Hint> {
        let mut h = vec![hint("0", "Home"), hint("↑ ↓", "Scroll output")];
        if w.activities.len() > 1 {
            h.push(hint("1–9", "Switch activity"));
            h.push(hint("Ctrl+A", "Next"));
        }
        h.push(hint("Esc", "Home"));
        h
    }

    fn crumb(&self, w: &World) -> String {
        match self.current(w) {
            Some(a) => format!("Activity · {}", a.name),
            None => "Activity".to_owned(),
        }
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        None
    }
}
