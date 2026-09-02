//! Stage rail: an ordered list of steps with a lifecycle state each. The
//! rail knows nothing about what the steps mean; it draws the frontier,
//! the completed count and one row per step, optionally as a focus stop
//! with a cursor for detail selection.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::scroll::ScrollState;
use crate::ui::ctx::{RenderCtx, fill};
use crate::widgets::scrollbar;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum StepState {
    #[default]
    Queued,
    Running,
    Done,
    Skipped,
    Failed,
    Blocked,
}

impl StepState {
    pub fn label(self) -> &'static str {
        match self {
            StepState::Queued => "queued",
            StepState::Running => "running",
            StepState::Done => "done",
            StepState::Skipped => "skipped",
            StepState::Failed => "failed",
            StepState::Blocked => "blocked",
        }
    }

    pub fn terminal(self) -> bool {
        matches!(self, StepState::Done | StepState::Skipped | StepState::Failed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub label: String,
    pub state: StepState,
    /// Right-aligned meta: activity text or duration.
    pub meta: Option<String>,
}

impl Step {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_owned(),
            state: StepState::Queued,
            meta: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StepRail {
    pub id: WidgetId,
    pub steps: Vec<Step>,
    pub selectable: bool,
    pub cursor: usize,
    pub scroll: ScrollState,
    pub area: Rect,
    /// Show the ordinal (`03`) before the label.
    pub numbered: bool,
}

impl StepRail {
    pub fn new(id: WidgetId, steps: Vec<Step>) -> Self {
        let n = steps.len();
        Self {
            id,
            steps,
            selectable: false,
            cursor: 0,
            scroll: ScrollState::new(n),
            area: Rect::ZERO,
            numbered: true,
        }
    }

    pub fn selectable(mut self, on: bool) -> Self {
        self.selectable = on;
        self
    }

    pub fn set_state(&mut self, i: usize, state: StepState) {
        if let Some(s) = self.steps.get_mut(i) {
            s.state = state;
        }
    }

    pub fn set_meta(&mut self, i: usize, meta: Option<String>) {
        if let Some(s) = self.steps.get_mut(i) {
            s.meta = meta;
        }
    }

    /// First step that is not finished: the frontier.
    pub fn frontier(&self) -> Option<usize> {
        self.steps.iter().position(|s| !s.state.terminal())
    }

    /// (done, skipped, failed)
    pub fn counts(&self) -> (usize, usize, usize) {
        let c = |st| self.steps.iter().filter(|s| s.state == st).count();
        (c(StepState::Done), c(StepState::Skipped), c(StepState::Failed))
    }

    pub fn failed(&self) -> Option<usize> {
        self.steps.iter().position(|s| s.state == StepState::Failed)
    }

    pub fn row_id(&self, i: usize) -> WidgetId {
        self.id.child(i)
    }

    pub fn locate(&self, id: WidgetId) -> Option<usize> {
        self.scroll.visible_range().find(|&i| self.row_id(i) == id)
    }

    pub fn owns(&self, id: WidgetId) -> bool {
        id == self.id || id == scrollbar::id_for(self.id) || self.locate(id).is_some()
    }

    fn set_cursor(&mut self, i: usize) {
        self.cursor = i.min(self.steps.len().saturating_sub(1));
        self.scroll.ensure_visible(self.cursor);
    }

    pub fn on_key(&mut self, key: &Key) -> Outcome {
        if !self.selectable || self.steps.is_empty() {
            return Outcome::Ignored;
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if key.plain() => {
                self.set_cursor(self.cursor.saturating_sub(1))
            }
            KeyCode::Down | KeyCode::Char('j') if key.plain() => self.set_cursor(self.cursor + 1),
            KeyCode::Home | KeyCode::Char('g') if key.plain() => self.set_cursor(0),
            KeyCode::End | KeyCode::Char('G') => self.set_cursor(usize::MAX),
            _ => return Outcome::Ignored,
        }
        Outcome::Changed
    }

    pub fn on_click(&mut self, row: usize) -> Outcome {
        if !self.selectable || row >= self.steps.len() {
            return Outcome::Consumed;
        }
        self.set_cursor(row);
        Outcome::Changed
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        self.scroll.scroll_by(delta as isize);
        Outcome::Changed
    }

    pub fn on_scrollbar(&mut self, pos: Position) -> Outcome {
        let track = Rect::new(
            self.area.right().saturating_sub(1),
            self.area.y,
            1,
            self.area.height,
        );
        self.scroll
            .scroll_to(scrollbar::offset_for_click(track, pos, &self.scroll));
        Outcome::Changed
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        self.area = area;
        let t = ctx.theme;
        let focused = self.selectable && ctx.interaction.focused(self.id);
        self.scroll.set_content(self.steps.len());
        self.scroll.set_viewport(area.height as usize);
        if self.selectable {
            ctx.control(self.id, area, false);
        }
        ctx.scrollable(self.id, area);
        let has_sb = self.scroll.overflows();
        let row_w = area.width.saturating_sub(u16::from(has_sb));
        let frontier = self.frontier();
        for (k, i) in self.scroll.visible_range().enumerate() {
            let y = area.y + k as u16;
            let step = &self.steps[i];
            let rid = self.row_id(i);
            let mut s = ctx.state(rid);
            s.focused = focused && i == self.cursor;
            if !self.selectable {
                s.hovered = false;
            }
            let row = Rect::new(area.x, y, row_w, 1);
            let st = t.row(s, bg);
            fill(buf, row, st);
            buf.set_string(row.x, y, "▎", t.gutter(s, st.bg.unwrap_or(bg), false));
            let (glyph, gstyle) = match step.state {
                StepState::Queued | StepState::Skipped | StepState::Blocked => (" ", st),
                StepState::Running => (
                    crate::widgets::progress::spinner_frame(ctx.interaction.tick),
                    st.fg(t.accent),
                ),
                StepState::Done => ("✓", st.fg(t.success)),
                StepState::Failed => ("!", st.fg(t.error).add_modifier(Modifier::BOLD)),
            };
            buf.set_string(row.x + 1, y, glyph, gstyle);
            let mut x = row.x + 3;
            if self.numbered {
                let num = format!("{:02}", i + 1);
                buf.set_string(
                    x,
                    y,
                    &num,
                    st.fg(if step.state == StepState::Running {
                        t.text_secondary
                    } else {
                        t.text_faint
                    })
                    .remove_modifier(Modifier::BOLD),
                );
                x += 3;
            }
            let label_style = match step.state {
                StepState::Queued => st.fg(t.text_muted),
                StepState::Running => st.fg(t.text_primary).add_modifier(Modifier::BOLD),
                StepState::Done => st.fg(t.text_secondary),
                StepState::Skipped => st.fg(t.text_faint),
                StepState::Failed => st.fg(t.error).add_modifier(Modifier::BOLD),
                StepState::Blocked => st.fg(t.text_secondary),
            };
            let label_style = if s.focused && !s.pressed {
                label_style.add_modifier(Modifier::BOLD)
            } else {
                label_style
            };
            let meta: Option<String> = match (&step.meta, step.state) {
                (Some(m), _) => Some(m.clone()),
                (None, StepState::Queued) => Some("queued".into()),
                (None, StepState::Skipped) => Some("skipped".into()),
                (None, StepState::Blocked) => Some("blocked".into()),
                _ => None,
            };
            let meta_style = match step.state {
                StepState::Running => st.fg(t.text_secondary),
                StepState::Failed => st.fg(t.error),
                StepState::Blocked => st.fg(t.warning),
                _ => st.fg(t.text_faint),
            }
            .remove_modifier(Modifier::BOLD);
            let avail = row.right().saturating_sub(x + 1) as usize;
            let mw = meta.as_ref().map(|m| crate::ui::text::width(m)).unwrap_or(0);
            let show_meta = mw > 0 && avail >= mw + 12;
            let lw = if show_meta { avail - mw - 2 } else { avail };
            buf.set_string(
                x,
                y,
                crate::ui::text::truncate(&step.label, lw),
                label_style,
            );
            if show_meta && let Some(m) = &meta {
                buf.set_string(row.right().saturating_sub(mw as u16 + 1), y, m, meta_style);
            }
            if self.selectable {
                ctx.clickable(rid, row);
            }
            let _ = frontier;
        }
        if has_sb {
            let sb = Rect::new(area.right() - 1, area.y, 1, area.height);
            scrollbar::render_vertical(sb, buf, ctx, self.id, &self.scroll, focused);
        }
        let _ = Style::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontier_and_counts() {
        let mut r = StepRail::new(
            WidgetId::of("r"),
            ["a", "b", "c", "d"].iter().map(|s| Step::new(s)).collect(),
        );
        r.set_state(0, StepState::Done);
        r.set_state(1, StepState::Skipped);
        r.set_state(2, StepState::Running);
        assert_eq!(r.frontier(), Some(2));
        assert_eq!(r.counts(), (1, 1, 0));
        r.set_state(2, StepState::Failed);
        assert_eq!(r.failed(), Some(2));
        assert_eq!(r.frontier(), Some(3));
    }
}
