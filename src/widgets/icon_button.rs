//! A padded glyph button: the close `×` of modal frames, tabs and chips,
//! and a tab strip's `+` (docs/design/close-button.md).
//!
//! The plate is the glyph with [`IconButton::pad`] cells either side
//! (default one: ` × `, three cells, the terminal analogue of WCAG 2.2
//! SC 2.5.8's 24 px target). At rest only the glyph shows, in secondary
//! ink on the host plane; hovered or pressed the whole plate takes the
//! next step of the plane it sits on (as a macOS or browser tab close
//! button shows its hover plate), and the hit region is exactly the plate.
//! A close button never carries the focus `▎`: keyboard focus stays on its
//! owner, which closes by its own key (Esc, `x`, Delete).

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::core::id::WidgetId;
use crate::theme::Theme;
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::text::width;
use crate::ui::tooltip::Tip;

/// Which ladder an [`IconButton`] walks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconLook {
    /// A close or remove button: [`Theme::close_plate`] (`Kind::Close`).
    #[default]
    Close,
    /// A text-like action such as `+`: the title ladder (`Kind::Title`).
    Title,
}

/// A glyph on a padded plate. See the [module docs](self).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconButton {
    pub id: WidgetId,
    /// The glyph. `None`: the theme's close glyph ([`Glyphs::close`],
    /// `×`, ASCII `x`).
    ///
    /// [`Glyphs::close`]: crate::theme::Glyphs::close
    pub glyph: Option<String>,
    pub look: IconLook,
    /// Cells of plate either side of the glyph. Default
    /// [`Self::DEFAULT_PAD`] (a three-cell plate).
    pub pad: u16,
    /// Hover takes the danger ink and press the danger fill (the Windows
    /// window-close idiom). Default off: closing a tab or a dialog is not
    /// destructive (macOS).
    pub danger_hover: bool,
    /// The tooltip. `None`: the owner's default (`Close · Esc` on a modal
    /// frame, `Close tab · x` on a tab, ...); an empty [`Tip`] shows none.
    pub tip: Option<Tip>,
}

impl IconButton {
    /// One cell of plate either side of the glyph.
    pub const DEFAULT_PAD: u16 = 1;

    /// A close button with the theme's close glyph.
    pub fn close(id: WidgetId) -> Self {
        Self {
            id,
            glyph: None,
            look: IconLook::Close,
            pad: Self::DEFAULT_PAD,
            danger_hover: false,
            tip: None,
        }
    }

    /// A text-like glyph button (`+`) on the title ladder.
    pub fn new(id: WidgetId, glyph: &str) -> Self {
        Self {
            glyph: Some(glyph.to_owned()),
            look: IconLook::Title,
            ..Self::close(id)
        }
    }

    /// The same button under another id (a template applied per tab).
    pub fn with_id(&self, id: WidgetId) -> Self {
        Self { id, ..self.clone() }
    }

    /// See [`Self::pad`].
    pub fn pad(mut self, cells: u16) -> Self {
        self.pad = cells;
        self
    }

    /// The plate `cells` wide (odd widths centre the glyph): sets
    /// [`Self::pad`] to `(cells - 1) / 2`.
    pub fn plate(self, cells: u16) -> Self {
        self.pad(cells.saturating_sub(1) / 2)
    }

    /// See [`Self::danger_hover`].
    pub fn danger_hover(mut self, on: bool) -> Self {
        self.danger_hover = on;
        self
    }

    /// See [`Self::tip`].
    pub fn tip(mut self, tip: Tip) -> Self {
        self.tip = Some(tip);
        self
    }

    /// The glyph drawn under `t`.
    pub fn glyph<'a>(&'a self, t: &Theme) -> &'a str {
        self.glyph.as_deref().unwrap_or(t.glyphs.close)
    }

    /// The plate's width in cells.
    pub fn width(&self, t: &Theme) -> u16 {
        width(self.glyph(t)) as u16 + 2 * self.pad
    }

    /// The plate's style inside a host styled `host`, for this frame's
    /// pointer state.
    pub fn style(&self, ctx: &RenderCtx, host: Style) -> Style {
        let t = ctx.theme;
        let mut s = ctx.state(self.id);
        s.focused = false;
        match self.look {
            IconLook::Close => t.close_plate(s, host, self.danger_hover),
            IconLook::Title => {
                let bg = host.bg.unwrap_or(t.canvas);
                t.danger_plate(s, ctx.title_style(self.id, bg), self.danger_hover)
            }
        }
    }

    /// Draw the plate with its left edge at `(x, y)` over whatever was
    /// there (a frame border), register it and show its tooltip. Returns
    /// the plate (clipped to the buffer).
    pub fn render(
        &self,
        x: u16,
        y: u16,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        host: Style,
        default_tip: Option<Tip>,
    ) -> Rect {
        let plate = Rect::new(x, y, self.width(ctx.theme), 1).intersection(*buf.area());
        fill(buf, plate, host);
        self.render_in(plate, x + self.pad, buf, ctx, host, default_tip)
    }

    /// Draw the glyph at `glyph_x` on `plate`, cells an owner laid out
    /// itself (inside a tab or a chip, styled `host`): at rest only the
    /// glyph changes; hovered or pressed every cell of the plate takes the
    /// plate's style, keeping its symbol. Registers `plate` as the button
    /// and shows its tooltip.
    pub fn render_in(
        &self,
        plate: Rect,
        glyph_x: u16,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        host: Style,
        default_tip: Option<Tip>,
    ) -> Rect {
        let plate = plate.intersection(*buf.area());
        if plate.is_empty() {
            return Rect::ZERO;
        }
        let st = self.style(ctx, host);
        let s = ctx.state(self.id);
        if s.hovered || s.pressed {
            for x in plate.left()..plate.right() {
                if let Some(cell) = buf.cell_mut((x, plate.y)) {
                    cell.set_style(Style::reset().patch(st));
                }
            }
        }
        buf.set_string(glyph_x, plate.y, self.glyph(ctx.theme), st);
        ctx.clickable(self.id, plate);
        if let Some(tip) = self.tip.clone().or(default_tip) {
            crate::ui::tooltip::show(ctx, self.id, plate, &tip, None);
        }
        plate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::{ColorLevel, Glyphs};
    use crate::ui::ctx::Interaction;
    use ratatui::layout::Position;
    use ratatui::style::Modifier;

    const ID: WidgetId = WidgetId::of("icon");

    /// Draw `b` at (2, 0) on a modal frame's border; returns the buffer,
    /// the hits and the plate.
    fn draw(t: &Theme, b: &IconButton, inter: Interaction) -> (Buffer, HitRegistry, Rect) {
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 16, 1));
        let host = t.border(true).bg(t.surface_elevated);
        let all = buf.area;
        fill(&mut buf, all, host);
        let r = {
            let mut ctx = RenderCtx::new(t, inter, &mut hits, &mut ring);
            b.render(2, 0, &mut buf, &mut ctx, host, None)
        };
        (buf, hits, r)
    }

    fn hovered() -> Interaction {
        Interaction {
            hover: Some(ID),
            ..Default::default()
        }
    }

    fn pressed() -> Interaction {
        Interaction {
            hover: Some(ID),
            pressed: Some(ID),
            ..Default::default()
        }
    }

    /// What a terminal at `level` shows of a cell: no colours in mono.
    fn look(t: &Theme, b: &Buffer, x: u16) -> (Option<ratatui::style::Color>, Modifier) {
        let c = &b[(x, 0)];
        let bg = (t.level != ColorLevel::Mono).then_some(c.bg);
        (bg, c.modifier)
    }

    #[test]
    fn the_hit_area_is_the_drawn_plate_at_every_size() {
        let t = Theme::junie();
        for pad in 0..=3 {
            let b = IconButton::close(ID).pad(pad);
            let (_, hits, r) = draw(&t, &b, Interaction::default());
            assert_eq!(r, Rect::new(2, 0, 1 + 2 * pad, 1), "pad {pad}");
            assert_eq!(b.width(&t), r.width);
            for x in 0..16 {
                let want = r.contains(Position::new(x, 0)).then_some(ID);
                assert_eq!(hits.hit(Position::new(x, 0)), want, "pad {pad} x {x}");
            }
        }
        // `plate(cells)` is the plate width; three cells by default
        assert_eq!(IconButton::close(ID).width(&t), 3);
        assert_eq!(IconButton::close(ID).plate(5).width(&t), 5);
        assert_eq!(IconButton::close(ID).plate(1).width(&t), 1);
    }

    #[test]
    fn hover_and_press_light_the_whole_plate_at_every_level() {
        for level in ColorLevel::ALL {
            let t = Theme::for_level(level);
            for danger in [false, true] {
                let b = IconButton::close(ID).danger_hover(danger);
                let (rest, _, r) = draw(&t, &b, Interaction::default());
                let (hover, _, _) = draw(&t, &b, hovered());
                let (press, _, _) = draw(&t, &b, pressed());
                let ctx = format!("{level:?} danger {danger}");
                for x in r.left()..r.right() {
                    let (a, h, p) = (look(&t, &rest, x), look(&t, &hover, x), look(&t, &press, x));
                    assert_ne!(a, h, "rest = hover at {x}: {ctx}");
                    assert_ne!(h, p, "hover = pressed at {x}: {ctx}");
                    assert_ne!(a, p, "rest = pressed at {x}: {ctx}");
                    // one plate: every cell of it looks the same
                    assert_eq!(h, look(&t, &hover, r.x), "hover plate at {x}: {ctx}");
                    assert_eq!(p, look(&t, &press, r.x), "pressed plate at {x}: {ctx}");
                }
                // cells beside the plate never change
                for x in [r.x - 1, r.right()] {
                    assert_eq!(look(&t, &rest, x), look(&t, &hover, x), "{ctx}");
                    assert_eq!(look(&t, &rest, x), look(&t, &press, x), "{ctx}");
                }
                // the glyph stays legible on its plate
                for buf in [&rest, &hover, &press] {
                    let c = &buf[(r.x + 1, 0)];
                    assert_eq!(c.symbol(), "×", "{ctx}");
                    assert_ne!(c.fg, c.bg, "{ctx}");
                    if level == ColorLevel::TrueColor {
                        let k = crate::theme::contrast_ratio(c.fg, c.bg).unwrap();
                        assert!(k >= 4.5, "{k:.2}: {ctx}");
                    }
                }
            }
        }
    }

    #[test]
    fn the_danger_tint_only_changes_hover_and_press() {
        let t = Theme::junie();
        let plain = IconButton::close(ID);
        let danger = plain.clone().danger_hover(true);
        let at = |b: &IconButton, i| draw(&t, b, i).0[(3, 0)].clone();
        assert_eq!(
            at(&plain, Interaction::default()),
            at(&danger, Interaction::default())
        );
        assert_ne!(at(&plain, hovered()).fg, at(&danger, hovered()).fg);
        assert_eq!(at(&danger, pressed()).bg, t.ladders.danger.pressed);
    }

    #[test]
    fn ascii_glyphs_draw_an_x_on_the_same_plate() {
        let t = Theme::for_level(ColorLevel::Mono).with_glyphs(Glyphs::ASCII);
        let b = IconButton::close(ID);
        let (buf, _, r) = draw(&t, &b, hovered());
        assert_eq!(r.width, 3);
        assert_eq!(buf[(r.x + 1, 0)].symbol(), "x");
        for x in r.left()..r.right() {
            assert!(buf[(x, 0)].modifier.contains(Modifier::UNDERLINED), "{x}");
        }
    }

    #[test]
    fn a_glyph_button_walks_the_title_ladder() {
        let t = Theme::junie();
        let b = IconButton::new(ID, "+");
        let (rest, _, r) = draw(&t, &b, Interaction::default());
        let (hover, _, _) = draw(&t, &b, hovered());
        assert_eq!(r.width, 3);
        assert_eq!(rest[(r.x + 1, 0)].symbol(), "+");
        for x in r.left()..r.right() {
            assert_ne!(rest[(x, 0)].bg, hover[(x, 0)].bg, "{x}");
        }
    }
}
