//! Termrock HelpOverlay modal shortcut catalog with multi-column responsive layout and scrolling.

use ratatui::crossterm::event::KeyCode;
use ratatui::style::{Modifier, Style};

use crate::termrock::identity::Id;
use crate::termrock::layout::{Constraints, Position, Rect, Size};
use crate::termrock::response::{Flow, Input, Invalidate, MouseKind, Response, UpdateCause};
use crate::termrock::runtime::{Cx, MeasureCx, Ui};
use crate::termrock::scroll::ScrollState;
use crate::termrock::text::{truncate, width};
use crate::termrock::theme::StylePatch;

/// A single key shortcut and description entry in a help section.
#[derive(Debug, Clone)]
pub struct HelpItem<'a> {
    pub chord: &'a str,
    pub description: &'a str,
}

impl<'a> HelpItem<'a> {
    pub const fn new(chord: &'a str, description: &'a str) -> Self {
        Self { chord, description }
    }
}

/// A logical section of help items under a common heading.
#[derive(Debug, Clone)]
pub struct HelpSection<'a> {
    pub title: &'a str,
    pub items: &'a [HelpItem<'a>],
}

impl<'a> HelpSection<'a> {
    pub const fn new(title: &'a str, items: &'a [HelpItem<'a>]) -> Self {
        Self { title, items }
    }
}

/// Action outcome emitted by HelpOverlay interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpAction {
    Dismissed,
}

/// Durable view state for HelpOverlay.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HelpOverlayState {
    pub scroll: ScrollState,
}

impl HelpOverlayState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// A scrollable, modal keyboard-shortcut help overlay component.
#[derive(Debug, Clone)]
pub struct HelpOverlay<'a> {
    pub id: Id,
    pub sections: &'a [HelpSection<'a>],
    pub title: &'a str,
    pub scope: Option<&'a str>,
    pub patch: Option<StylePatch>,
}

impl<'a> HelpOverlay<'a> {
    pub fn new(id: Id, sections: &'a [HelpSection<'a>]) -> Self {
        Self {
            id,
            sections,
            title: "Keyboard shortcuts",
            scope: None,
            patch: None,
        }
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    pub fn scope(mut self, scope: &'a str) -> Self {
        self.scope = Some(scope);
        self
    }

    pub fn patch(mut self, patch: StylePatch) -> Self {
        self.patch = Some(patch);
        self
    }

    pub fn frame_rect(&self, area: Rect) -> Rect {
        let w = (area.width * 4 / 5).clamp(40, 114).min(area.width);
        let h = (area.height * 4 / 5).clamp(12, 34).min(area.height);
        let x = area.x + (area.width.saturating_sub(w)) / 2;
        let y = area.y + (area.height.saturating_sub(h)) / 2;
        Rect::new(x, y, w, h)
    }

    pub fn measure(&self, _cx: &MeasureCx<'_>, constraints: Constraints) -> Size {
        let desired_w = 76;
        let desired_h = 20;
        constraints.clamp(Size::new(desired_w, desired_h))
    }

    pub fn update(&self, cx: &mut Cx<'_>, state: &mut HelpOverlayState) -> Response<HelpAction> {
        let area = Rect::new(0, 0, 80, 24);
        let frame = self.frame_rect(area);

        let total_lines: usize = self.sections.iter().map(|s| s.items.len() + 2).sum();
        state.scroll.total = total_lines.max(30);
        if state.scroll.viewport == 0 {
            state.scroll.viewport = 10;
        }

        match cx.cause() {
            UpdateCause::Input(Input::Key(k), _) => match k.code {
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Enter => {
                    cx.request_invalidate(Invalidate::Paint);
                    return Response::action(self.id.clone(), HelpAction::Dismissed)
                        .with_flow(Flow::Consumed)
                        .with_invalidate(Invalidate::Paint);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    state.scroll.scroll_up(1);
                    cx.request_invalidate(Invalidate::Paint);
                    return Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    state.scroll.scroll_down(1);
                    cx.request_invalidate(Invalidate::Paint);
                    return Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint);
                }
                KeyCode::PageUp => {
                    state.scroll.scroll_up(8);
                    cx.request_invalidate(Invalidate::Paint);
                    return Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint);
                }
                KeyCode::PageDown => {
                    state.scroll.scroll_down(8);
                    cx.request_invalidate(Invalidate::Paint);
                    return Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint);
                }
                _ => {}
            },
            UpdateCause::Input(Input::Mouse(m), _) => {
                let pos = Position::new(m.pos.x, m.pos.y);
                match m.kind {
                    MouseKind::Up if !frame.contains(pos) => {
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::action(self.id.clone(), HelpAction::Dismissed)
                            .with_flow(Flow::Consumed)
                            .with_invalidate(Invalidate::Paint);
                    }
                    MouseKind::WheelUp => {
                        state.scroll.scroll_up(2);
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone())
                            .with_invalidate(Invalidate::Paint);
                    }
                    MouseKind::WheelDown => {
                        state.scroll.scroll_down(2);
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone())
                            .with_invalidate(Invalidate::Paint);
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        Response::bubble(self.id.clone())
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, state: &HelpOverlayState) -> Rect {
        if area.is_empty() {
            return Rect::zero();
        }

        let theme = ui.theme;
        let frame = self.frame_rect(area);

        // Dim backdrop outside frame
        if frame.width < area.width || frame.height < area.height {
            let dim_style = Style::new().fg(theme.tokens.text_muted);
            for y in area.y..area.y + area.height {
                for x in area.x..area.x + area.width {
                    let pos = Position::new(x, y);
                    if !frame.contains(pos) {
                        ui.set_string(x, y, " ", dim_style);
                    }
                }
            }
        }

        ui.register_focus(self.id.clone(), true);
        ui.register_hit(self.id.clone(), frame);

        let bg = theme.tokens.surface_elevated;
        let border_fg = theme.tokens.border_strong;
        let border_style = Style::new().bg(bg).fg(border_fg);

        // Fill surface
        ui.fill_rect(frame, Style::new().bg(bg));

        let right_x = frame.x + frame.width.saturating_sub(1);
        let bottom_y = frame.y + frame.height.saturating_sub(1);

        // Rounded borders
        ui.set_string(frame.x, frame.y, "┌", border_style);
        ui.set_string(right_x, frame.y, "┐", border_style);
        ui.set_string(frame.x, bottom_y, "└", border_style);
        ui.set_string(right_x, bottom_y, "┘", border_style);

        for x in (frame.x + 1)..right_x {
            ui.set_string(x, frame.y, "─", border_style);
            ui.set_string(x, bottom_y, "─", border_style);
        }
        for y in (frame.y + 1)..bottom_y {
            ui.set_string(frame.x, y, "│", border_style);
            ui.set_string(right_x, y, "│", border_style);
        }

        // Title and optional scope
        let title_text = if let Some(scope) = self.scope {
            format!(" {} · {} ", self.title, scope)
        } else {
            format!(" {} ", self.title)
        };
        let max_title_w = (frame.width.saturating_sub(4)) as usize;
        let tr_title = truncate(&title_text, max_title_w);
        let title_style = Style::new()
            .bg(bg)
            .fg(theme.tokens.text_primary)
            .add_modifier(Modifier::BOLD);
        ui.set_string(frame.x + 2, frame.y, &tr_title, title_style);

        // Footer hint row
        let footer_y = bottom_y.saturating_sub(1);
        let hint_text = " ↑↓ Scroll · Esc Close ";
        let hint_style = Style::new().bg(bg).fg(theme.tokens.text_muted);
        let hint_x = frame.x + 2;
        ui.set_string(hint_x, footer_y, hint_text, hint_style);

        // Body area calculation
        let body_top = frame.y + 1;
        let body_bottom = footer_y;
        let body_h = body_bottom.saturating_sub(body_top);
        let inner_w = frame.width.saturating_sub(4);

        if body_h == 0 || inner_w == 0 {
            return frame;
        }

        // Determine column count: 1, 2, or 3
        let col_count = if frame.width >= 106 {
            3
        } else if frame.width >= 70 {
            2
        } else {
            1
        };

        let col_w = (inner_w.saturating_sub((col_count - 1) * 2)) / col_count;

        // Build flat lines list per section
        struct RenderLine<'a> {
            is_header: bool,
            chord: &'a str,
            desc: &'a str,
        }

        let mut lines = Vec::new();
        for sec in self.sections {
            lines.push(RenderLine {
                is_header: true,
                chord: sec.title,
                desc: "",
            });
            for item in sec.items {
                lines.push(RenderLine {
                    is_header: false,
                    chord: item.chord,
                    desc: item.description,
                });
            }
            // Blank spacer
            lines.push(RenderLine {
                is_header: false,
                chord: "",
                desc: "",
            });
        }

        // Apply scroll offset
        let scroll_offset = state.scroll.offset();
        let visible_lines = if scroll_offset < lines.len() {
            &lines[scroll_offset..]
        } else {
            &[]
        };

        // Render lines across columns
        let mut curr_line_idx = 0;
        for c in 0..col_count {
            let col_x = frame.x + 2 + c * (col_w + 2);

            for row_idx in 0..body_h {
                if curr_line_idx >= visible_lines.len() {
                    break;
                }
                let line = &visible_lines[curr_line_idx];
                curr_line_idx += 1;

                let row_y = body_top + row_idx;

                if line.is_header {
                    let header_style = Style::new()
                        .bg(bg)
                        .fg(theme.tokens.accent)
                        .add_modifier(Modifier::BOLD);
                    let tr_header = truncate(line.chord, col_w as usize);
                    ui.set_string(col_x, row_y, &tr_header, header_style);
                } else if !line.chord.is_empty() {
                    let chord_style = Style::new()
                        .bg(bg)
                        .fg(theme.tokens.text_primary)
                        .add_modifier(Modifier::BOLD);
                    let chord_w = (width(line.chord) + 1).min(col_w as usize);
                    let tr_chord = truncate(line.chord, chord_w);
                    ui.set_string(col_x, row_y, &tr_chord, chord_style);

                    let desc_x = col_x + chord_w as u16 + 1;
                    if desc_x < col_x + col_w {
                        let desc_avail = (col_x + col_w - desc_x) as usize;
                        let tr_desc = truncate(line.desc, desc_avail);
                        let desc_style = Style::new().bg(bg).fg(theme.tokens.text_secondary);
                        ui.set_string(desc_x, row_y, &tr_desc, desc_style);
                    }
                }
            }
        }

        frame
    }
}
