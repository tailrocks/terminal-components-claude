use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;

use crate::core::event::Outcome;
use crate::core::id::WidgetId;
use crate::theme::Kind;
use crate::ui::ctx::{RenderCtx, VisualState};
use crate::ui::layout::{Split, SplitDir};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SeamDoubleClick {
    #[default]
    Reset,

    Ignore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Splitter {
    pub id: WidgetId,
    pub dir: SplitDir,

    pub area: Rect,

    pub hit_pad: u16,

    pub double_click: SeamDoubleClick,

    grab: Option<i32>,
}

impl Splitter {
    pub const fn new(id: WidgetId, dir: SplitDir) -> Self {
        Self {
            id,
            dir,
            area: Rect::ZERO,
            hit_pad: 1,
            double_click: SeamDoubleClick::Reset,
            grab: None,
        }
    }

    pub const fn hit_pad(mut self, cells: u16) -> Self {
        self.hit_pad = cells;
        self
    }

    pub const fn double_click(mut self, policy: SeamDoubleClick) -> Self {
        self.double_click = policy;
        self
    }

    pub fn hit_area(&self, clip: Rect) -> Rect {
        if self.area.is_empty() {
            return Rect::ZERO;
        }
        let p = self.hit_pad;
        let r = match self.dir {
            SplitDir::Horizontal => {
                let x = self.area.x.saturating_sub(p);
                Rect::new(x, self.area.y, self.area.right() + p - x, self.area.height)
            }
            SplitDir::Vertical => {
                let y = self.area.y.saturating_sub(p);
                Rect::new(self.area.x, y, self.area.width, self.area.bottom() + p - y)
            }
        };
        r.intersection(clip)
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        self.render_held(area, buf, ctx, bg, false);
    }

    pub fn render_held(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
        held: bool,
    ) {
        let area = area.intersection(*buf.area());
        self.area = area;

        let pressed = held || ctx.interaction.pressed == Some(self.id);
        if !pressed {
            self.grab = None;
        }
        if area.is_empty() {
            return;
        }
        let t = ctx.theme;
        let hovered = ctx.interaction.hovered(self.id);
        let s = VisualState {
            hovered,
            pressed,
            ..Default::default()
        };
        let style = t.interactive(Kind::Seam, s, t.plane_of(bg)).style.bg(bg);
        let glyph = match (self.dir, hovered || pressed) {
            (SplitDir::Horizontal, false) => t.glyphs.divider,
            (SplitDir::Horizontal, true) => t.glyphs.scroll_thumb,
            (SplitDir::Vertical, false) => t.glyphs.h_divider,
            (SplitDir::Vertical, true) => t.glyphs.scroll_h_thumb,
        };
        for pos in area.positions() {
            buf.set_string(pos.x, pos.y, glyph, style);
        }

        let hit = self.hit_area(*buf.area());
        ctx.clickable_under(self.id, hit);
        ctx.clickable(self.id, area);
        ctx.pointer_shape(
            self.id,
            match self.dir {
                SplitDir::Horizontal => crate::core::shape::PointerShape::ColResize,
                SplitDir::Vertical => crate::core::shape::PointerShape::RowResize,
            },
        );
    }

    fn grab_at(&self, split: &Split, container: Rect, gap: u16, pos: Position) -> i32 {
        let handle = split.handle(self.dir, container, gap);
        let (at, span) = match self.dir {
            SplitDir::Horizontal => (pos.x as i32 - handle.x as i32, handle.width),
            SplitDir::Vertical => (pos.y as i32 - handle.y as i32, handle.height),
        };
        let pad = self.hit_pad as i32;
        at.clamp(-pad, (span as i32 - 1).max(0) + pad)
    }

    pub fn on_press(&mut self, split: &Split, container: Rect, gap: u16, pos: Position) {
        self.grab = Some(self.grab_at(split, container, gap, pos));
    }

    pub fn on_drag(
        &mut self,
        split: &mut Split,
        container: Rect,
        gap: u16,
        pos: Position,
    ) -> Outcome {
        let grab = match self.grab {
            Some(g) => g,
            None => {
                let g = self.grab_at(split, container, gap, pos);
                self.grab = Some(g);
                g
            }
        };
        let shift = |v: u16| (v as i32 - grab).max(0) as u16;
        let target = match self.dir {
            SplitDir::Horizontal => Position::new(shift(pos.x), pos.y),
            SplitDir::Vertical => Position::new(pos.x, shift(pos.y)),
        };
        if split.drag_to(self.dir, container, gap, target) {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    pub fn on_click(&mut self, split: &mut Split, count: u8) -> Outcome {
        self.grab = None;
        if count != 2 {
            return Outcome::Consumed;
        }
        match self.double_click {
            SeamDoubleClick::Reset => {
                let before = *split;
                split.reset();
                if *split == before {
                    Outcome::Consumed
                } else {
                    Outcome::Changed
                }
            }
            SeamDoubleClick::Ignore => Outcome::Consumed,
        }
    }
}

impl crate::ui::specimen::Specimen for Splitter {
    fn specimen_id(&self) -> WidgetId {
        self.id
    }

    fn supports(&self, state: crate::ui::specimen::SpecimenState) -> bool {
        use crate::ui::specimen::SpecimenState::*;
        matches!(state, Rest | Hover | Pressed)
    }

    fn specimen_height(&self, _state: crate::ui::specimen::SpecimenState) -> u16 {
        match self.dir {
            SplitDir::Horizontal => 3,
            SplitDir::Vertical => 1,
        }
    }

    fn specimen(
        &self,
        _state: crate::ui::specimen::SpecimenState,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
    ) {
        let strip = match self.dir {
            SplitDir::Horizontal => Rect::new(area.x + 1, area.y, 1, area.height),
            SplitDir::Vertical => Rect::new(area.x, area.y, area.width.min(12), 1),
        };
        let mut s = *self;
        s.render(strip, buf, ctx, bg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::theme::{ColorLevel, Theme};
    use crate::ui::ctx::Interaction;

    #[test]
    fn seam_rest_hover_and_drag_differ_at_every_level() {
        let id = WidgetId::of("seam");
        for level in ColorLevel::ALL {
            let t = Theme::for_level(level);
            let draw = |interaction: Interaction| {
                let mut hits = HitRegistry::default();
                let mut ring = FocusRing::default();
                let mut ctx = RenderCtx::new(&t, interaction, &mut hits, &mut ring);
                let mut buf = Buffer::empty(Rect::new(0, 0, 1, 3));
                Splitter::new(id, SplitDir::Horizontal).render(
                    Rect::new(0, 0, 1, 3),
                    &mut buf,
                    &mut ctx,
                    t.canvas,
                );
                let c = buf[(0, 1)].clone();
                let ink = if level == ColorLevel::Mono {
                    None
                } else {
                    Some(c.fg)
                };
                (c.symbol().to_owned(), ink, c.modifier, c.fg != c.bg)
            };
            let rest = draw(Interaction::default());
            let hover = draw(Interaction {
                hover: Some(id),
                ..Default::default()
            });
            let drag = draw(Interaction {
                hover: Some(id),
                pressed: Some(id),
                ..Default::default()
            });
            assert!(rest.3, "{level:?}: the rest seam is visible");
            assert_ne!(rest, hover, "{level:?}");
            assert_ne!(hover, drag, "{level:?}");
            assert_ne!(rest, drag, "{level:?}");
        }
    }

    #[test]
    fn seam_states_are_thin_grey_heavy_grey_and_heavy_accent() {
        let id = WidgetId::of("seam");
        for level in ColorLevel::ALL {
            let t = Theme::for_level(level);
            for dir in [SplitDir::Horizontal, SplitDir::Vertical] {
                let draw = |interaction: Interaction| {
                    let mut hits = HitRegistry::default();
                    let mut ring = FocusRing::default();
                    let mut ctx = RenderCtx::new(&t, interaction, &mut hits, &mut ring);
                    let area = Rect::new(0, 0, 3, 3);
                    let mut buf = Buffer::empty(area);
                    let strip = match dir {
                        SplitDir::Horizontal => Rect::new(1, 0, 1, 3),
                        SplitDir::Vertical => Rect::new(0, 1, 3, 1),
                    };
                    Splitter::new(id, dir).render(strip, &mut buf, &mut ctx, t.canvas);
                    buf[(1, 1)].clone()
                };
                let rest = draw(Interaction::default());
                let hover = draw(Interaction {
                    hover: Some(id),
                    ..Default::default()
                });
                let drag = draw(Interaction {
                    hover: Some(id),
                    pressed: Some(id),
                    ..Default::default()
                });
                let (thin, heavy) = match dir {
                    SplitDir::Horizontal => (t.glyphs.divider, t.glyphs.scroll_thumb),
                    SplitDir::Vertical => (t.glyphs.h_divider, t.glyphs.scroll_h_thumb),
                };
                let at = format!("{level:?} {dir:?}");
                assert_eq!(rest.symbol(), thin, "{at}");
                assert_eq!(hover.symbol(), heavy, "{at}");
                assert_eq!(drag.symbol(), heavy, "{at}");
                assert_eq!(rest.fg, t.ladders.seam.rest, "{at}");
                assert_eq!(hover.fg, t.ladders.seam.hover, "{at}");
                if level == ColorLevel::Mono {
                    assert!(hover.modifier.contains(t.mods.thumb_hover), "{at}");
                    assert!(drag.modifier.contains(t.mods.thumb_pressed), "{at}");
                    assert_ne!(hover.modifier, drag.modifier, "{at}");
                } else {
                    assert_ne!(rest.fg, hover.fg, "{at}: hover is a brighter grey");
                    assert_eq!(
                        drag.fg, t.ladders.handle.pressed,
                        "{at}: drag is the accent"
                    );
                    assert_ne!(drag.fg, hover.fg, "{at}");
                }
            }
        }
    }

    fn render_seam(seam: &mut Splitter, split: &Split, container: Rect) -> HitRegistry {
        let t = Theme::for_level(ColorLevel::TrueColor);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(container);
        let handle = split.handle(seam.dir, container, 1);
        seam.render(handle, &mut buf, &mut ctx, t.canvas);
        hits
    }

    #[test]
    fn the_seam_takes_presses_one_cell_either_side() {
        let id = WidgetId::of("seam");
        let split = Split::new(50, 4, 4);
        let container = Rect::new(0, 0, 41, 5);
        let mut seam = Splitter::new(id, SplitDir::Horizontal);
        let hits = render_seam(&mut seam, &split, container);
        let x = split.handle(SplitDir::Horizontal, container, 1).x;
        for dx in [x - 1, x, x + 1] {
            assert_eq!(hits.hit(Position::new(dx, 2)), Some(id), "x={dx}");
        }
        assert_eq!(hits.hit(Position::new(x + 2, 2)), None);

        let mut hits = render_seam(&mut seam, &split, container);
        let bar = WidgetId::of("bar");
        hits.register(bar, Rect::new(x - 1, 0, 1, 5));
        assert_eq!(hits.hit(Position::new(x - 1, 2)), Some(bar));
        assert_eq!(hits.hit(Position::new(x + 1, 2)), Some(id));
        let mut tight = Splitter::new(id, SplitDir::Horizontal).hit_pad(0);
        let hits = render_seam(&mut tight, &split, container);
        assert_eq!(hits.hit(Position::new(x - 1, 2)), None, "hit_pad(0)");
    }

    #[test]
    fn a_drag_keeps_the_grab_offset_and_moves_cell_by_cell() {
        let id = WidgetId::of("seam");
        let mut split = Split::new(50, 4, 4);

        let container = Rect::new(0, 0, 62, 5);
        let mut seam = Splitter::new(id, SplitDir::Horizontal);
        render_seam(&mut seam, &split, container);
        let x0 = split.handle(SplitDir::Horizontal, container, 1).x;

        seam.on_press(&split, container, 1, Position::new(x0 + 1, 2));
        for step in 1..=7u16 {
            seam.on_drag(&mut split, container, 1, Position::new(x0 + 1 + step, 2));
            let x = split.handle(SplitDir::Horizontal, container, 1).x;
            assert_eq!(x, x0 + step, "step {step}");
        }

        let mut split = Split::new(50, 4, 4);
        let mut seam = Splitter::new(id, SplitDir::Horizontal);
        render_seam(&mut seam, &split, container);
        seam.on_drag(&mut split, container, 1, Position::new(x0 - 1, 2));
        assert_eq!(split.handle(SplitDir::Horizontal, container, 1).x, x0);
        seam.on_drag(&mut split, container, 1, Position::new(x0 - 4, 2));
        assert_eq!(split.handle(SplitDir::Horizontal, container, 1).x, x0 - 3);
    }

    #[test]
    fn a_double_click_resets_the_split() {
        let id = WidgetId::of("seam");
        let container = Rect::new(0, 0, 62, 5);
        let mut split = Split::new(50, 4, 4);
        let mut seam = Splitter::new(id, SplitDir::Horizontal);
        render_seam(&mut seam, &split, container);
        seam.on_drag(&mut split, container, 1, Position::new(50, 2));
        let moved = split;
        assert_eq!(seam.on_click(&mut split, 1), Outcome::Consumed);
        assert_eq!(split, moved, "a single click changes nothing");
        assert_eq!(seam.on_click(&mut split, 2), Outcome::Changed);
        assert_eq!(split, Split::new(50, 4, 4));
        let mut split = moved;
        let mut off = Splitter::new(id, SplitDir::Horizontal).double_click(SeamDoubleClick::Ignore);
        off.on_click(&mut split, 2);
        assert_eq!(split, moved);
    }
}
