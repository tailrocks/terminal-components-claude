//! Painting (`COMPONENT_ARCHITECTURE.md` §5 R3, §17.0 A2, §22.2 items 1–2, 16, 18).
//!
//! Every method clips to the current area and marks the layer's
//! written-cell bitset. Cell, string, and span painters share Ratatui
//! grapheme/width semantics in one allocation-free writer that also records
//! provenance;
//! `paint_cell` resets the cells a wide grapheme shadows; `fill` and
//! `dim_layer` are deliberate re-implementations of `ratatui_widgets::{Fill,
//! Dimmed}` because foreign widgets cannot mark the bitset or walk roles.

use ratatui_core::buffer::{Buffer, CellWidth};
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Modifier, Style};
use unicode_segmentation::UnicodeSegmentation;

use super::Ui;
use crate::text::Span;
use crate::theme::{FgStep, GlyphRole, PaintStyle, Role, Surface, Theme};

impl Ui<'_> {
    /// Paint graphemes at `pos`, refusing any grapheme wider than the clip.
    pub fn paint_cell(&mut self, pos: Position, symbol: &str, s: impl Into<PaintStyle>) {
        if self.clip.contains(pos) {
            self.paint_str(
                Rect::new(pos.x, pos.y, self.clip.right().saturating_sub(pos.x), 1),
                symbol,
                s,
            );
        }
    }

    /// Paint text with Ratatui's grapheme and width semantics. Lead cells
    /// retain the supplied origin; shadow cells reset both bytes and origin.
    /// This single walk is shared by cell, string, span, and glyph painters.
    pub fn paint_str(&mut self, area: Rect, text: &str, s: impl Into<PaintStyle>) -> u16 {
        let area = area.intersection(self.clip);
        if area.is_empty() {
            return 0;
        }
        let s = s.into();
        let mut x = area.x;
        let mut remaining = area.width;
        for symbol in text
            .graphemes(true)
            .filter(|symbol| !symbol.contains(char::is_control))
        {
            let width = symbol.cell_width();
            if width == 0 {
                continue;
            }
            let Some(rest) = remaining.checked_sub(width) else {
                break;
            };
            remaining = rest;
            let pos = Position::new(x, area.y);
            if let Some(cell) = self.buffer().cell_mut(pos) {
                cell.set_symbol(symbol).set_style(s.into_style());
            }
            self.mark(pos, Some(s));
            let end = x.saturating_add(width);
            x = x.saturating_add(1);
            while x < end {
                let pos = Position::new(x, area.y);
                if let Some(cell) = self.buffer().cell_mut(pos) {
                    cell.reset();
                }
                self.mark(pos, None);
                x = x.saturating_add(1);
            }
        }
        x.saturating_sub(area.x)
    }

    /// Paint semantic spans with no allocation, inheriting `base` independently
    /// for each span. Width and continuation handling share the string writer.
    pub fn paint_spans(
        &mut self,
        area: Rect,
        spans: &[Span<'_>],
        base: impl Into<PaintStyle>,
    ) -> u16 {
        let area = area.intersection(self.clip);
        if area.is_empty() {
            return 0;
        }
        let base = base.into();
        let mut x = area.x;
        for sp in spans {
            if x >= area.right() {
                break;
            }
            let mut st = base.add_modifier(sp.add);
            if let Some(role) = sp.role {
                st = st.patch(self.paint_patch(&crate::theme::StylePatch::new().set_fg(role)));
            }
            x = x.saturating_add(self.paint_str(
                Rect::new(x, area.y, area.right().saturating_sub(x), 1),
                sp.text,
                st,
            ));
        }
        x.saturating_sub(area.x)
    }

    /// Restyle `area` without touching symbols (`Buffer::set_style`).
    pub fn paint_style(&mut self, area: Rect, s: impl Into<PaintStyle>) {
        let s = s.into();
        let area = area.intersection(self.clip);
        if area.is_empty() {
            return;
        }
        self.buffer().set_style(area, s.into_style());
        self.mark_area(area, Some(s));
    }

    /// Fill `area` with spaces in `s` (per-position `set_symbol(" ")`).
    pub fn fill(&mut self, area: Rect, s: impl Into<PaintStyle>) {
        let s = s.into();
        let area = area.intersection(self.clip);
        if area.is_empty() {
            return;
        }
        {
            let buf = self.buffer();
            for pos in area.positions() {
                if let Some(c) = buf.cell_mut(pos) {
                    c.set_symbol(" ").set_style(s.into_style());
                }
            }
        }
        self.mark_area(area, Some(s));
    }

    /// A quiet rule across `area`'s first row (`GlyphRole::RuleQuiet`).
    pub fn rule(&mut self, area: Rect) {
        let g = self.theme_ref().design.glyphs.get(GlyphRole::RuleQuiet);
        let s = self.paint_patch(&crate::theme::StylePatch::new().set_fg(Role::BorderSubtle));
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
    pub fn frame(&mut self, area: Rect, s: impl Into<PaintStyle>) -> Rect {
        let s = s.into();
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
    pub fn glyph(&mut self, area: Rect, g: GlyphRole, s: impl Into<PaintStyle>) -> u16 {
        let s = s.into();
        let sym = self.theme_ref().design.glyphs.get(g);
        self.paint_str(area, sym, s)
    }

    /// The buffer and the current clip rect. The documented escape hatch:
    /// marks the whole clip rect written.
    pub fn raw(&mut self) -> (&mut Buffer, Rect) {
        let clip = self.clip;
        self.mark_area(clip, None);
        (self.buffer(), clip)
    }

    /// Dim the page under a layer by walking the role recorded per painted
    /// cell and stepping it down the foreground ladder semantically
    /// (§54, `docs/reviews/laneC-app-tick.md` Q4). `steps == 0` is identity:
    /// not a restyle to the same colours, but no write at all, so the frame
    /// is byte-identical. One step is `Fg(Muted)`, two `Fg(Faint)`, three
    /// `Fg(Ghost)` and four or more erases the glyph into the resolved
    /// backdrop background; ladder roles start from their own rung and erase
    /// once they step past `Ghost`; `Accent`/`AccentHover`/`AccentPressed`
    /// walk the accent chain and erase past its end. Backgrounds resolve
    /// from the recorded background role — never by colour identity, which
    /// is exactly the reverse-lookup defect this replaces. Only `BOLD`
    /// survives; every other modifier is cleared. Walks only `area`.
    pub fn dim_layer(&mut self, area: Rect, steps: u8) {
        if steps == 0 {
            return;
        }
        let area = area.intersection(self.frame.screen);
        let theme = self.theme_ref();
        let surface = self.surface;
        let backdrop_text = crate::theme::resolve::bind_role(theme, Role::BackdropFg, surface);
        let backdrop_fill = crate::theme::resolve::bind_role(theme, Role::BackdropBg, surface);
        for pos in area.positions() {
            let roles = self.roles_at(pos);
            let fg = match roles.fg {
                // a ladder role steps from its own rung and erases past Ghost
                Some(Role::Fg(step)) => ladder(theme, surface, step.index(), steps),
                // the accent chain degrades through hover and pressed
                Some(Role::Accent) => accent(theme, surface, 0, steps),
                Some(Role::AccentHover) => accent(theme, surface, 1, steps),
                Some(Role::AccentPressed) => accent(theme, surface, 2, steps),
                // a background role recorded as a foreground carries no text
                Some(Role::CurrentSurface | Role::RaisedSurface | Role::Surface(_)) => {
                    FadeResult::Fg(None)
                }
                // every other semantic foreground: Muted, Faint, Ghost, erase
                Some(_) => ladder(theme, surface, 1, steps),
                None => FadeResult::Fg(backdrop_text),
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
                let bold = c.modifier.intersection(Modifier::BOLD);
                let mut st = Style::new();
                st.fg = match fg {
                    FadeResult::Fg(f) => f,
                    // erased: the glyph goes, and what is left is the
                    // resolved backdrop background
                    FadeResult::Erase => {
                        c.set_symbol(" ");
                        backdrop_fill
                    }
                };
                st.bg = bg;
                c.set_style(st);
                c.modifier = bold;
            }
        }
    }
}

/// The outcome of stepping one recorded foreground role down.
enum FadeResult {
    /// The dimmed foreground (`None` leaves the cell's foreground alone).
    Fg(Option<Color>),
    /// The glyph is erased into the backdrop.
    Erase,
}

/// Step `base` (an `FgStep` index) down by `steps`, erasing past `Ghost`.
fn ladder(theme: &Theme, surface: Surface, base: usize, steps: u8) -> FadeResult {
    match base.saturating_add(usize::from(steps)) {
        i if i <= 4 => FadeResult::Fg(crate::theme::resolve::bind_role(
            theme,
            Role::Fg(index_to_step(i)),
            surface,
        )),
        _ => FadeResult::Erase,
    }
}

/// Step the accent chain (`Accent`, `AccentHover`, `AccentPressed`) down by
/// `steps` from `base`, erasing past its end.
fn accent(theme: &Theme, surface: Surface, base: usize, steps: u8) -> FadeResult {
    let role = match base.saturating_add(usize::from(steps)) {
        0 => Role::Accent,
        1 => Role::AccentHover,
        2 => Role::AccentPressed,
        _ => return FadeResult::Erase,
    };
    FadeResult::Fg(crate::theme::resolve::bind_role(theme, role, surface))
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

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::{Position, Rect};
    use ratatui_core::style::{Color, Modifier, Style};

    use super::super::cx::LastFrame;
    use super::super::{FrameState, Ui, UiCore};
    use crate::theme::{FgStep, Role, Surface, Theme};

    const SCREEN: Rect = Rect {
        x: 0,
        y: 0,
        width: 8,
        height: 2,
    };

    fn with_ui<R>(theme: &Theme, f: impl FnOnce(&mut Ui<'_>) -> R) -> (R, Buffer) {
        let mut frame = FrameState::default();
        frame.reset(1, SCREEN);
        let mut page = Buffer::empty(SCREEN);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let out = {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, theme, &last);
            f(&mut ui)
        };
        (out, page)
    }

    /// Paint `symbol` at `(0, 0)` carrying `fg` as its recorded foreground
    /// role over the canvas, then dim the screen by `steps`.
    fn dimmed_cell(
        theme: &Theme,
        fg: Role,
        symbol: &str,
        modifier: Modifier,
        steps: u8,
    ) -> ratatui_core::buffer::Cell {
        let ((), page) = with_ui(theme, |ui| {
            let style = ui.paint_patch(
                &crate::theme::StylePatch::new()
                    .set_fg(fg)
                    .set_bg(Role::CurrentSurface)
                    .add(modifier),
            );
            ui.paint_cell(Position::ORIGIN, symbol, style);
            ui.dim_layer(SCREEN, steps);
        });
        page.cell(Position::ORIGIN).expect("cell").clone()
    }

    fn fg_of(theme: &Theme, step: FgStep) -> Color {
        crate::theme::resolve::bind_role(theme, Role::Fg(step), Surface::Canvas).expect("fg")
    }

    /// §54: `dim_layer(area, 0)` is identity. It is not a restyle to the
    /// colours the roles already resolve to — it writes nothing at all, so
    /// every cell, including symbols, modifiers and never-painted cells,
    /// is byte-for-byte what it was.
    #[test]
    fn dim_layer_zero_steps_is_byte_identical() {
        for theme in [Theme::junie(), Theme::paper()] {
            let (before, after) = with_ui(&theme, |ui| {
                ui.paint_str(
                    SCREEN,
                    "ok",
                    Style::new()
                        .fg(fg_of(&theme, FgStep::Primary))
                        .add_modifier(Modifier::ITALIC | Modifier::BOLD),
                );
                ui.paint_cell(
                    Position::new(4, 1),
                    "x",
                    Style::new().fg(Color::Green).add_modifier(Modifier::DIM),
                );
                let before = ui.page_mut().clone();
                ui.dim_layer(SCREEN, 0);
                (before, ui.page_mut().clone())
            })
            .0;
            assert_eq!(before, after, "dim_layer(area, 0) must write nothing");
        }
    }

    /// Q4: the four non-ladder tone roles walk Muted, Faint, Ghost and then
    /// erase into the resolved backdrop background; ladder roles step from
    /// their own rung and erase past `Ghost`; the accent chain degrades
    /// through hover and pressed and then erases. Only `BOLD` survives, and
    /// nothing is decided by colour identity.
    #[test]
    fn dim_layer_semantic_roles_step_monotonically_and_erase() {
        for theme in [Theme::junie(), Theme::paper()] {
            let backdrop_bg =
                crate::theme::resolve::bind_role(&theme, Role::BackdropBg, Surface::Canvas);
            // the four non-ladder tones: Muted, Faint, Ghost, erase
            for role in [Role::Success, Role::Warning, Role::Danger, Role::Info] {
                for (steps, step) in [(1u8, FgStep::Muted), (2, FgStep::Faint), (3, FgStep::Ghost)]
                {
                    let c = dimmed_cell(&theme, role, "x", Modifier::empty(), steps);
                    assert_eq!(c.fg, fg_of(&theme, step), "{role:?} at {steps}");
                    assert_eq!(c.symbol(), "x", "{role:?} at {steps} keeps its glyph");
                }
                for steps in [4u8, 5, 9] {
                    let c = dimmed_cell(&theme, role, "x", Modifier::empty(), steps);
                    assert_eq!(c.symbol(), " ", "{role:?} erases at {steps}");
                    assert_eq!(c.fg, backdrop_bg.expect("backdrop"), "{role:?} at {steps}");
                }
            }
            // every other non-ladder foreground role uses the same rule —
            // `BorderSubtle` and `DisabledFg` are exactly the two the legacy
            // colour-identity lookup misclassified
            for role in [Role::BorderSubtle, Role::DisabledFg, Role::Focus] {
                assert_eq!(
                    dimmed_cell(&theme, role, "x", Modifier::empty(), 1).fg,
                    fg_of(&theme, FgStep::Muted),
                    "{role:?}"
                );
                assert_eq!(
                    dimmed_cell(&theme, role, "x", Modifier::empty(), 4).symbol(),
                    " ",
                    "{role:?}"
                );
            }
            // ladder roles step from their own rung, saturating at Ghost and
            // erasing only past it
            for (start, steps, want) in [
                (FgStep::Primary, 1u8, FgStep::Secondary),
                (FgStep::Primary, 4, FgStep::Ghost),
                (FgStep::Secondary, 2, FgStep::Faint),
                (FgStep::Muted, 2, FgStep::Ghost),
                (FgStep::Ghost, 0, FgStep::Ghost),
            ] {
                let c = dimmed_cell(&theme, Role::Fg(start), "x", Modifier::empty(), steps);
                assert_eq!(c.fg, fg_of(&theme, want), "{start:?} + {steps}");
                assert_eq!(c.symbol(), "x");
            }
            for (start, steps) in [
                (FgStep::Primary, 5u8),
                (FgStep::Muted, 3),
                (FgStep::Ghost, 1),
            ] {
                let c = dimmed_cell(&theme, Role::Fg(start), "x", Modifier::empty(), steps);
                assert_eq!(c.symbol(), " ", "{start:?} + {steps} erases past Ghost");
                assert_eq!(c.fg, backdrop_bg.expect("backdrop"));
            }
            // the accent chain
            let accent_of = |r: Role| {
                crate::theme::resolve::bind_role(&theme, r, Surface::Canvas).expect("accent")
            };
            for (start, steps, want) in [
                (Role::Accent, 1u8, Role::AccentHover),
                (Role::Accent, 2, Role::AccentPressed),
                (Role::AccentHover, 1, Role::AccentPressed),
            ] {
                let c = dimmed_cell(&theme, start, "x", Modifier::empty(), steps);
                assert_eq!(c.fg, accent_of(want), "{start:?} + {steps}");
                assert_eq!(c.symbol(), "x");
            }
            for (start, steps) in [
                (Role::Accent, 3u8),
                (Role::AccentHover, 2),
                (Role::AccentPressed, 1),
            ] {
                let c = dimmed_cell(&theme, start, "x", Modifier::empty(), steps);
                assert_eq!(c.symbol(), " ", "{start:?} + {steps} erases past the chain");
                assert_eq!(c.fg, backdrop_bg.expect("backdrop"));
            }
            // only BOLD survives
            let c = dimmed_cell(
                &theme,
                Role::Fg(FgStep::Primary),
                "x",
                Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED,
                1,
            );
            assert_eq!(c.modifier, Modifier::BOLD);
            let c = dimmed_cell(
                &theme,
                Role::Fg(FgStep::Primary),
                "x",
                Modifier::ITALIC | Modifier::REVERSED,
                1,
            );
            assert_eq!(c.modifier, Modifier::empty());
            // the background is resolved from the recorded background role,
            // never from the cell's colour
            let c = dimmed_cell(&theme, Role::Fg(FgStep::Primary), "x", Modifier::empty(), 1);
            assert_eq!(c.bg, theme.bg(Surface::Canvas));
        }
    }
}
