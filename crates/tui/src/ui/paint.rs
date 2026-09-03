//! Painting (`COMPONENT_ARCHITECTURE.md` §5 R3, §17.0 A2, §22.2 items 1–2, 16, 18).
//!
//! Every method clips to the current area and marks the layer's
//! written-cell bitset. `paint_str` *is* `Buffer::set_stringn`; `paint_spans`
//! walks the spans and paints each through `Buffer::set_span`, allocating
//! nothing;
//! `paint_cell` resets the cells a wide grapheme shadows; `fill` and
//! `dim_layer` are deliberate re-implementations of `ratatui_widgets::{Fill,
//! Dimmed}` because foreign widgets cannot mark the bitset or walk roles.

use ratatui_core::buffer::{Buffer, CellWidth};
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Modifier, Style};
use ratatui_core::text::Span as RawSpan;

use super::{CellRoles, Ui};
use crate::text::Span;
use crate::theme::{FgStep, GlyphRole, Role, Surface};

impl Ui<'_> {
    /// Paint one grapheme at `pos`. The cells shadowed by a wide grapheme
    /// are reset, as `set_stringn` does, so the diff stays correct (R‑6).
    pub fn paint_cell(&mut self, pos: Position, symbol: &str, s: Style) {
        if !self.clip.contains(pos) {
            return;
        }
        let w = usize::from(if symbol.contains(char::is_control) {
            0
        } else {
            symbol.cell_width()
        });
        if w == 0 {
            return;
        }
        let right = self.clip.right();
        let buf = self.buffer();
        if let Some(c) = buf.cell_mut(pos) {
            c.set_symbol(symbol).set_style(s);
        }
        let mut x = pos.x.saturating_add(1);
        let end = pos.x.saturating_add(w as u16);
        while x < end && x < right {
            if let Some(c) = buf.cell_mut(Position::new(x, pos.y)) {
                c.reset();
            }
            x = x.saturating_add(1);
        }
        let mut px = pos.x;
        while px < end && px < right {
            self.mark(Position::new(px, pos.y));
            px = px.saturating_add(1);
        }
    }

    /// Paint `text` from `area`'s origin, clipped to `area.width` and the
    /// clip rect. Returns the columns written. Never pre-truncates (R‑2).
    pub fn paint_str(&mut self, area: Rect, text: &str, s: Style) -> u16 {
        let area = area.intersection(self.clip);
        if area.is_empty() {
            return 0;
        }
        let (end, _) = self
            .buffer()
            .set_stringn(area.x, area.y, text, usize::from(area.width), s);
        let written = end.saturating_sub(area.x);
        self.mark_area(Rect {
            x: area.x,
            y: area.y,
            width: written,
            height: 1,
        });
        written
    }

    /// Multi-style single-line paint: each `Span`'s role is resolved against
    /// the live theme and surface and written through `Buffer::set_span`
    /// (R‑3 — the same `set_stringn` width accounting as `paint_str`), one
    /// span at a time with **no intermediate allocation** (§20.9-6, R5).
    /// `base` is the part style the spans inherit. Returns the columns
    /// written.
    pub fn paint_spans(&mut self, area: Rect, spans: &[Span<'_>], base: Style) -> u16 {
        let area = area.intersection(self.clip);
        if area.is_empty() || spans.is_empty() {
            return 0;
        }
        let base_roles = self.roles;
        let right = area.right();
        let mut x = area.x;
        for sp in spans {
            if x >= right {
                break;
            }
            let mut st = base.add_modifier(sp.add);
            if let Some(r) = sp.role
                && let Some(c) =
                    crate::theme::resolve::bind_role(self.theme_ref(), r, self.surface())
            {
                st = st.fg(c);
            }
            self.set_roles(CellRoles {
                fg: sp.role.or(base_roles.fg),
                bg: base_roles.bg,
            });
            let width = right.saturating_sub(x);
            let (end, _) = self
                .buffer()
                .set_span(x, area.y, &RawSpan::styled(sp.text, st), width);
            self.mark_area(Rect {
                x,
                y: area.y,
                width: end.saturating_sub(x),
                height: 1,
            });
            x = end;
        }
        self.set_roles(base_roles);
        x.saturating_sub(area.x)
    }

    /// Restyle `area` without touching symbols (`Buffer::set_style`).
    pub fn paint_style(&mut self, area: Rect, s: Style) {
        let area = area.intersection(self.clip);
        if area.is_empty() {
            return;
        }
        self.buffer().set_style(area, s);
        self.mark_area(area);
    }

    /// Fill `area` with spaces in `s` (per-position `set_symbol(" ")`).
    pub fn fill(&mut self, area: Rect, s: Style) {
        let area = area.intersection(self.clip);
        if area.is_empty() {
            return;
        }
        {
            let buf = self.buffer();
            for pos in area.positions() {
                if let Some(c) = buf.cell_mut(pos) {
                    c.set_symbol(" ").set_style(s);
                }
            }
        }
        self.mark_area(area);
    }

    /// A quiet rule across `area`'s first row (`GlyphRole::RuleQuiet`).
    pub fn rule(&mut self, area: Rect) {
        let g = self.theme_ref().design.glyphs.get(GlyphRole::RuleQuiet);
        let fg =
            crate::theme::resolve::bind_role(self.theme_ref(), Role::BorderSubtle, self.surface);
        let mut s = Style::new();
        s.fg = fg;
        self.set_roles(CellRoles {
            fg: Some(Role::BorderSubtle),
            bg: None,
        });
        let row = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        }
        .intersection(self.clip);
        for x in row.columns() {
            self.paint_cell(Position::new(x.x, row.y), g, s);
        }
    }

    /// Draw the theme border set around `area` in `s`; returns the inner rect.
    pub fn frame(&mut self, area: Rect, s: Style) -> Rect {
        let area = area.intersection(self.clip);
        if area.width < 2 || area.height < 2 {
            return Rect::ZERO;
        }
        let b = self.theme_ref().design.borders;
        let left = area.left();
        let right = area.right().saturating_sub(1);
        let top = area.top();
        let bottom = area.bottom().saturating_sub(1);
        for col in area.columns().map(|c| c.x) {
            self.paint_cell(Position::new(col, top), b.horizontal_top, s);
            self.paint_cell(Position::new(col, bottom), b.horizontal_bottom, s);
        }
        for row in area.rows().map(|r| r.y) {
            self.paint_cell(Position::new(left, row), b.vertical_left, s);
            self.paint_cell(Position::new(right, row), b.vertical_right, s);
        }
        self.paint_cell(Position::new(left, top), b.top_left, s);
        self.paint_cell(Position::new(right, top), b.top_right, s);
        self.paint_cell(Position::new(left, bottom), b.bottom_left, s);
        self.paint_cell(Position::new(right, bottom), b.bottom_right, s);
        Rect {
            x: left.saturating_add(1),
            y: top.saturating_add(1),
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        }
    }

    /// Paint a glyph role at `area`'s origin; returns the columns written.
    pub fn glyph(&mut self, area: Rect, g: GlyphRole, s: Style) -> u16 {
        let sym = self.theme_ref().design.glyphs.get(g);
        self.paint_str(area, sym, s)
    }

    /// The buffer and the current clip rect. The documented escape hatch:
    /// marks the whole clip rect written.
    pub fn raw(&mut self) -> (&mut Buffer, Rect) {
        let clip = self.clip;
        self.mark_area(clip);
        (self.buffer(), clip)
    }

    /// Dim the page under a layer by walking the role recorded per painted
    /// cell and stepping it down the foreground ladder by `steps`; fills
    /// with the backdrop role become the backdrop colour. Modifiers are
    /// cleared. Walks only `area`.
    pub fn dim_layer(&mut self, area: Rect, steps: u8) {
        let area = area.intersection(self.frame.screen);
        let theme = self.theme_ref();
        let surface = self.surface;
        let backdrop_text = crate::theme::resolve::bind_role(theme, Role::BackdropFg, surface);
        let backdrop_fill = crate::theme::resolve::bind_role(theme, Role::BackdropBg, surface);
        for pos in area.positions() {
            let roles = self.roles_at(pos);
            let fg = match roles.fg {
                Some(Role::Fg(step)) => {
                    let i = step.index().saturating_add(usize::from(steps)).min(4);
                    crate::theme::resolve::bind_role(theme, Role::Fg(index_to_step(i)), surface)
                }
                Some(Role::CurrentSurface | Role::RaisedSurface | Role::Surface(_)) => None,
                Some(_) => {
                    crate::theme::resolve::bind_role(theme, Role::Fg(FgStep::Muted), surface)
                        .or(backdrop_text)
                }
                None => backdrop_text,
            };
            let bg = match roles.bg {
                Some(Role::Surface(s)) => {
                    crate::theme::resolve::bind_role(theme, Role::Surface(s), surface)
                }
                Some(Role::CurrentSurface) => Some(theme.bg(Surface::Canvas)),
                Some(Role::RaisedSurface) => Some(theme.bg(Surface::Surface)),
                _ => backdrop_fill,
            };
            let page = self.page_mut();
            if let Some(c) = page.cell_mut(pos) {
                let mut st = Style::new();
                st.fg = fg;
                st.bg = bg;
                c.set_style(st);
                c.modifier = Modifier::empty();
            }
        }
    }
}

const fn index_to_step(i: usize) -> FgStep {
    match i {
        0 => FgStep::Primary,
        1 => FgStep::Secondary,
        2 => FgStep::Muted,
        3 => FgStep::Faint,
        _ => FgStep::Ghost,
    }
}
