use core::fmt;

use ratatui_core::layout::Rect;

use super::{PartStyle, SlotFn, cell_at, first_row};
use crate::id::{Id, Part, PartRef};
use crate::layout::{Insets, inset};
use crate::measure::{Constraints, Size};
use crate::response::StateFlags;
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Surface, Variant};
use crate::ui::{FrameRead, Ui};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PanelKind {
    #[default]
    Card,
    Framed,
}

pub struct Panel<'a> {
    id: Id,
    kind: PanelKind,
    title: Option<&'a str>,
    meta: Option<&'a str>,
    badge: Option<&'a str>,
    focused: bool,
    ov: PartStyle<'a>,
}

impl fmt::Debug for Panel<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Panel")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .field("title", &self.title)
            .field("meta", &self.meta)
            .field("badge", &self.badge)
            .field("focused", &self.focused)
            .field("overrides", &self.ov)
            .finish()
    }
}

impl<'a> Panel<'a> {
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::GUTTER,
        Part::TITLE,
        Part::DETAIL,
        Part::BADGE,
        Part::BORDER,
    ];

    pub const fn new(id: Id) -> Self {
        Panel {
            id,
            kind: PanelKind::Card,
            title: None,
            meta: None,
            badge: None,
            focused: false,
            ov: PartStyle::new(),
        }
    }

    pub const fn id(&self) -> Id {
        self.id
    }

    #[must_use]
    pub const fn kind(mut self, k: PanelKind) -> Self {
        self.kind = k;
        self
    }

    #[must_use]
    pub const fn title(mut self, t: &'a str) -> Self {
        self.title = Some(t);
        self
    }

    #[must_use]
    pub const fn meta(mut self, m: &'a str) -> Self {
        self.meta = Some(m);
        self
    }

    #[must_use]
    pub const fn badge(mut self, text: &'a str) -> Self {
        self.badge = if text.is_empty() { None } else { Some(text) };
        self
    }

    #[must_use]
    pub const fn focused(mut self, yes: bool) -> Self {
        self.focused = yes;
        self
    }

    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    const fn derived(&self) -> StateFlags {
        if self.focused {
            StateFlags::FOCUSED
        } else {
            StateFlags::empty()
        }
    }

    fn plane(&self, ui: &Ui<'_>) -> Surface {
        match self.kind {
            PanelKind::Card => ui.theme().raise(ui.surface()),
            PanelKind::Framed => ui.surface(),
        }
    }

    const fn top_inset(&self) -> u16 {
        match self.kind {
            PanelKind::Card if self.title.is_some() => 2,
            PanelKind::Card | PanelKind::Framed => 1,
        }
    }

    fn insets(&self, ui: &Ui<'_>) -> Insets {
        let side = match self.kind {
            PanelKind::Card => ui.design().space.card_inset,
            PanelKind::Framed => ui.design().space.frame_inset,
        };
        Insets {
            l: side,
            t: self.top_inset(),
            r: side,
            b: 1,
        }
    }

    pub fn inner(&self, ui: &Ui<'_>, area: Rect) -> Rect {
        let r = inset(area, self.insets(ui));
        if r.is_empty() {
            return Rect {
                x: area.x,
                y: area.y,
                width: 0,
                height: 0,
            };
        }
        r
    }

    fn badge_lane_width(&self) -> Option<u16> {
        let title_w = self.title.map_or(0, crate::text::width);
        let meta_w = self.meta.map_or(0, crate::text::width);
        let badge_w = self.badge.map(crate::text::width)?;

        Some(
            title_w
                .saturating_add(meta_w)
                .saturating_add(badge_w)
                .saturating_add(7)
                .saturating_add(if self.kind == PanelKind::Framed {
                    1u16.saturating_add(2u16.saturating_mul(u16::from(self.meta.is_some())))
                } else {
                    0
                }),
        )
    }

    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        let insets = self.insets(ui);
        let chrome_w = insets.l.saturating_add(insets.r);
        let chrome_h = insets.t.saturating_add(insets.b);
        let title_w = self.title.map_or(0, crate::text::width);
        let meta_w = self.meta.map_or(0, crate::text::width);
        let head = title_w
            .saturating_add(meta_w)
            .saturating_add(u16::from(meta_w != 0))
            .saturating_add(2);
        let head = self.badge_lane_width().map_or(head, |lane| lane.max(head));
        Size {
            min: (chrome_w.saturating_add(1), chrome_h.saturating_add(1)),
            preferred: (
                chrome_w.saturating_add(1).max(head),
                chrome_h.saturating_add(1),
            ),
        }
        .fit(c)
    }

    /// Measure this panel around its actual body content.
    ///
    /// The body receives constraints reduced by the same insets used by
    /// [`Self::inner`]. Its measured size is then combined with panel chrome,
    /// title, metadata, and badge requirements before fitting the outer
    /// constraints. The callback must measure only; it must not draw, update
    /// input state, or perform application effects.
    pub fn measure_with_body(
        &self,
        ui: &Ui<'_>,
        constraints: Constraints,
        measure_body: impl FnOnce(Constraints) -> Size,
    ) -> Size {
        let insets = self.insets(ui);
        let chrome_w = insets.l.saturating_add(insets.r);
        let chrome_h = insets.t.saturating_add(insets.b);
        let body_constraints = Constraints {
            max: (
                constraints.max.0.saturating_sub(chrome_w),
                constraints.max.1.saturating_sub(chrome_h),
            ),
            tight_w: constraints.tight_w,
            tight_h: constraints.tight_h,
        };
        let body = measure_body(body_constraints);
        let title_w = self.title.map_or(0, crate::text::width);
        let meta_w = self.meta.map_or(0, crate::text::width);
        let head = title_w
            .saturating_add(meta_w)
            .saturating_add(u16::from(meta_w != 0))
            .saturating_add(2);
        let head = self.badge_lane_width().map_or(head, |lane| lane.max(head));
        let one_body_cell = 1;
        Size {
            min: (
                chrome_w
                    .saturating_add(body.min.0)
                    .max(chrome_w.saturating_add(one_body_cell)),
                chrome_h
                    .saturating_add(body.min.1)
                    .max(chrome_h.saturating_add(one_body_cell)),
            ),
            preferred: (
                chrome_w
                    .saturating_add(body.preferred.0)
                    .max(chrome_w.saturating_add(one_body_cell))
                    .max(head),
                chrome_h
                    .saturating_add(body.preferred.1)
                    .max(chrome_h.saturating_add(one_body_cell)),
            ),
        }
        .fit(constraints)
    }

    pub fn draw<R>(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        body: impl FnOnce(&mut Ui<'_>, Rect) -> R,
    ) -> R {
        let plane = self.plane(ui);
        ui.with_surface(plane, |ui| {
            let inner = self.inner(ui, area);
            self.chrome(ui, area);
            ui.with_area(inner, |ui| body(ui, inner))
        })
    }

    fn chrome(&self, ui: &mut Ui<'_>, area: Rect) {
        let ov = self.ov;
        let id = self.id;
        let live = PartStyle::flags(StateFlags::empty(), self.derived());
        let container = ov.style(
            ui,
            id,
            Family::PANEL,
            Variant::DEFAULT,
            Part::CONTAINER,
            live,
        );
        ui.fill(area, container.style);
        ui.register_decor(id, PartRef::of(Part::CONTAINER), area);
        if self.kind == PanelKind::Framed {
            let border = ov.style(ui, id, Family::PANEL, Variant::DEFAULT, Part::BORDER, live);
            if let Some(f) = ov.slot_for(Part::BORDER) {
                f(ui, area);
            } else {
                ui.frame(area, border.style);
            }
            ui.register_decor(id, PartRef::of(Part::BORDER), area);
        }
        self.head(ui, area, live, container.style);
    }

    fn draw_badge(&self, ui: &mut Ui<'_>, rect: Rect, live: StateFlags) {
        let Some(text) = self.badge.filter(|_| !rect.is_empty()) else {
            return;
        };
        let style = self.ov.style(
            ui,
            self.id,
            Family::PANEL,
            Variant::DEFAULT,
            Part::BADGE,
            live,
        );
        ui.with_area(rect, |ui| {
            if let Some(paint) = self.ov.slot_for(Part::BADGE) {
                paint(ui, rect);
            } else {
                ui.fill(rect, style.over(ui.surface_style()));
                ui.paint_str(
                    Rect {
                        x: rect.x.saturating_add(1),
                        y: rect.y,
                        width: rect.width.saturating_sub(2),
                        height: 1,
                    },
                    text,
                    style.style,
                );
            }
        });
    }

    fn head(&self, ui: &mut Ui<'_>, area: Rect, live: StateFlags, fill: crate::theme::PaintStyle) {
        let head = first_row(area);
        if head.is_empty() || area.width < 3 {
            return;
        }
        let ov = self.ov;
        let id = self.id;

        let pad = u16::from(self.kind == PanelKind::Framed);
        let gutter_x = area.x.saturating_add(1);
        let gutter = cell_at(head, gutter_x);
        if let Some(f) = ov.slot_for(Part::GUTTER) {
            f(ui, gutter);
        } else {
            let g = ov.style(ui, id, Family::PANEL, Variant::DEFAULT, Part::GUTTER, live);
            match g.glyph {
                Slot::Set(glyph) => {
                    ui.glyph(gutter, glyph, g.style);
                }
                Slot::Inherit if live.contains(StateFlags::FOCUSED) => {
                    ui.glyph(gutter, GlyphRole::FocusBar, g.style);
                }
                Slot::Inherit | Slot::Clear => ui.fill(gutter, g.style),
            }
        }
        let text_x = area.x.saturating_add(2);

        let span_w = area.width.saturating_sub(3);

        if self
            .badge_lane_width()
            .is_none_or(|needed| area.width < needed)
        {
            self.head_without_badge(ui, area, live, fill);
            return;
        }

        let span_right = text_x.saturating_add(span_w);
        let mut right = span_right;

        let meta_block = self.meta.and_then(|m| {
            let width = crate::text::width(m).saturating_add(pad.saturating_mul(2));
            (width < span_w).then_some(width)
        });
        let meta_rect = meta_block.map(|width| {
            right = right.saturating_sub(width);
            Rect {
                x: right,
                y: head.y,
                width,
                height: 1,
            }
        });

        let badge_block = self.badge.and_then(|b| {
            let width = crate::text::width(b).saturating_add(2);
            let gap = u16::from(meta_rect.is_some());
            (width.saturating_add(gap) <= right.saturating_sub(text_x)).then_some((width, gap))
        });
        let badge_rect = badge_block.map(|(width, gap)| {
            right = right.saturating_sub(gap).saturating_sub(width);
            Rect {
                x: right,
                y: head.y,
                width,
                height: 1,
            }
        });

        if let Some(t) = self.title {
            let rect = Rect {
                x: text_x,
                y: head.y,
                width: right.saturating_sub(text_x),
                height: 1,
            };
            if let Some(f) = ov.slot_for(Part::TITLE) {
                f(ui, rect);
            } else {
                let s = ov.style(ui, id, Family::PANEL, Variant::DEFAULT, Part::TITLE, live);
                paint_label(ui, rect, t, pad, s.style);
            }
        }

        if let Some(rect) = badge_rect {
            self.draw_badge(ui, rect, live);
        }

        if let (Some(m), Some(rect)) = (self.meta, meta_rect) {
            if let Some(f) = ov.slot_for(Part::DETAIL) {
                f(ui, rect);
            } else {
                let s = ov.style(ui, id, Family::PANEL, Variant::DEFAULT, Part::DETAIL, live);
                paint_label(ui, rect, m, pad, s.style);
            }
        }
    }

    fn head_without_badge(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        live: StateFlags,
        fill: crate::theme::PaintStyle,
    ) {
        let head = first_row(area);
        let text_x = area.x.saturating_add(2);
        let span_w = area.width.saturating_sub(3);
        let pad = u16::from(self.kind == PanelKind::Framed);
        let ov = self.ov;
        let id = self.id;
        let meta_block = self.meta.map_or(0, |m| {
            let want = crate::text::width(m).saturating_add(pad.saturating_mul(2));
            if want < span_w { want } else { 0 }
        });
        let title_room = span_w.saturating_sub(meta_block);
        if let Some(t) = self.title {
            let avail = title_room.saturating_sub(pad);
            let rect = Rect {
                x: text_x,
                y: head.y,
                width: avail,
                height: 1,
            };
            let used = if let Some(f) = ov.slot_for(Part::TITLE) {
                f(ui, rect);
                avail
            } else {
                let s = ov.style(ui, id, Family::PANEL, Variant::DEFAULT, Part::TITLE, live);
                ui.paint_str(rect, t, s.style)
            };
            if pad == 1 && used > 0 {
                ui.fill(cell_at(head, text_x.saturating_add(used)), fill);
            }
        }
        if let (Some(m), true) = (self.meta, meta_block > 0) {
            let x = text_x.saturating_add(title_room).saturating_add(pad);
            let rect = Rect {
                x,
                y: head.y,
                width: meta_block.saturating_sub(pad.saturating_mul(2)),
                height: 1,
            };
            let used = if let Some(f) = ov.slot_for(Part::DETAIL) {
                f(ui, rect);
                rect.width
            } else {
                let s = ov.style(ui, id, Family::PANEL, Variant::DEFAULT, Part::DETAIL, live);

                if pad == 1 {
                    ui.fill(cell_at(head, x.saturating_sub(1)), s.style);
                }
                ui.paint_str(rect, m, s.style)
            };
            if pad == 1 && used > 0 {
                let style = ov
                    .style(ui, id, Family::PANEL, Variant::DEFAULT, Part::DETAIL, live)
                    .style;
                ui.fill(cell_at(head, x.saturating_add(used)), style);
            }
        }
    }
}

fn paint_label(
    ui: &mut Ui<'_>,
    block: Rect,
    text: &str,
    pad: u16,
    style: crate::theme::PaintStyle,
) {
    let inner = Rect {
        x: block.x.saturating_add(pad),
        width: block.width.saturating_sub(pad.saturating_mul(2)),
        ..block
    };
    let text_width = crate::text::width(text).min(inner.width);
    let label = Rect {
        width: text_width.saturating_add(pad.saturating_mul(2)),
        ..block
    };
    if pad > 0 {
        ui.fill(label, style);
    }
    ui.paint_str(inner, text, style);
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Position;

    use super::*;
    use crate::runtime::Runtime;
    use crate::runtime::stub::{SCREEN, Stub};
    use crate::theme::{Role, Theme};

    const ID: Id = Id::root("panel.tests");

    #[test]
    fn body_runs_once_for_an_empty_area_and_returns_its_value() {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        let calls = Cell::new(0);
        let seen = Cell::new(Rect::ZERO);
        let area = Rect::new(7, 5, 0, 0);
        let mut answer = 0;
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            answer = Panel::new(ID).draw(ui, area, |_, inner| {
                calls.set(calls.get() + 1);
                seen.set(inner);
                42
            });
        })
        .commit_presented();
        assert_eq!(answer, 42);
        assert_eq!(calls.get(), 1);
        assert_eq!(seen.get(), area);
    }

    #[test]
    fn body_paint_is_clipped_to_the_inner_rect() {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        let area = Rect::new(4, 3, 12, 5);
        let mut inner = Rect::ZERO;
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            Panel::new(ID).draw(ui, area, |ui, body| {
                inner = body;
                let style = ui.surface_style();
                for row in SCREEN.rows() {
                    ui.paint_str(row, "ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ", style);
                }
            });
        })
        .commit_presented();
        for pos in SCREEN.positions() {
            let is_z = buf.cell(pos).is_some_and(|cell| cell.symbol() == "Z");
            assert_eq!(is_z, inner.contains(pos), "body clip mismatch at {pos:?}");
        }
    }

    fn symbol_at(buf: &Buffer, x: u16, y: u16) -> String {
        buf.cell(Position::new(x, y))
            .map_or_else(String::new, |c| c.symbol().to_owned())
    }

    #[test]
    fn the_inner_rect_never_escapes_the_panel() {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            for kind in [PanelKind::Card, PanelKind::Framed] {
                for title in [None, Some("t")] {
                    for w in 0u16..=9 {
                        for h in 0u16..=5 {
                            let area = Rect {
                                x: 3,
                                y: 2,
                                width: w,
                                height: h,
                            };
                            let mut p = Panel::new(ID).kind(kind);
                            if let Some(t) = title {
                                p = p.title(t);
                            }
                            let inner = p.inner(ui, area);
                            assert!(
                                inner.x >= area.x
                                    && inner.y >= area.y
                                    && inner.right() <= area.right()
                                    && inner.bottom() <= area.bottom(),
                                "{kind:?} {title:?} {w}x{h}: {inner:?} escapes {area:?}"
                            );
                        }
                    }
                }
            }
        })
        .commit_presented();
    }

    #[test]
    fn a_framed_head_row_keeps_the_corner_columns() {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        let area = Rect {
            x: 0,
            y: 0,
            width: 20,
            height: 5,
        };
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            Panel::new(ID)
                .kind(PanelKind::Framed)
                .title("a title far too long for this panel")
                .meta("and a meta as well")
                .draw(ui, area, |_, inner| inner);
        })
        .commit_presented();
        let b = Theme::junie().design.borders;
        assert_eq!(symbol_at(&buf, 0, 0), b.top_left);
        assert_eq!(symbol_at(&buf, 19, 0), b.top_right);
        assert_eq!(symbol_at(&buf, 0, 4), b.bottom_left);
        assert_eq!(symbol_at(&buf, 19, 4), b.bottom_right);
    }

    #[test]
    fn the_focus_bar_is_painted_from_the_props_and_only_then() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 14,
            height: 4,
        };
        let bar = Theme::junie().design.glyphs.get(GlyphRole::FocusBar);
        let render = |focused: bool| {
            let mut rt = Runtime::new(Stub::default(), Theme::junie());
            let mut buf = Buffer::empty(SCREEN);
            rt.draw_scene(SCREEN, &mut buf, |ui, _| {
                Panel::new(ID)
                    .title("Files")
                    .focused(focused)
                    .draw(ui, area, |_, inner| inner);
            })
            .commit_presented();
            symbol_at(&buf, 1, 0)
        };
        assert_eq!(render(true), bar, "a focused panel paints no focus bar");
        assert_ne!(
            render(false),
            bar,
            "an unfocused panel paints the focus bar"
        );
    }

    #[test]
    fn a_panel_registers_no_focus_stop() {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_scene(SCREEN, &mut buf, |ui, a| {
            Panel::new(ID)
                .kind(PanelKind::Framed)
                .title("Files")
                .draw(ui, a, |_, inner| inner);
        })
        .commit_presented();
        assert!(!rt.ring().is_registered(ID));
        assert_eq!(rt.ring().reachable().count(), 0);
        assert!(rt.area_of(ID).is_some(), "the container is not addressable");
    }

    #[test]
    fn a_reference_panel_registers_nothing() {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_scene(SCREEN, &mut buf, |ui, a| {
            ui.reference(None, |ui| {
                Panel::new(ID)
                    .kind(PanelKind::Framed)
                    .title("Files")
                    .draw(ui, a, |_, inner| inner);
            });
        })
        .commit_presented();
        assert!(rt.area_of(ID).is_none());
    }

    #[test]
    fn every_declared_part_is_one_a_drawn_panel_styles() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 24,
            height: 5,
        };
        let render = |patched: Option<Part>| {
            let ps: [(Part, StylePatch); 1] = [(
                patched.unwrap_or(Part::CONTAINER),
                StylePatch::new().set_fg(Role::Warning).set_bg(Role::Danger),
            )];
            let mut rt = Runtime::new(Stub::default(), Theme::junie());
            let mut buf = Buffer::empty(SCREEN);
            rt.draw_scene(SCREEN, &mut buf, |ui, _| {
                let mut p = Panel::new(ID)
                    .kind(PanelKind::Framed)
                    .title("Files")
                    .meta("12")
                    .badge("EDIT")
                    .focused(true);
                if patched.is_some() {
                    p = p.patch_part(&ps);
                }
                p.draw(ui, area, |_, inner| inner);
            })
            .commit_presented();
            buf
        };
        let plain = render(None);
        for part in Panel::PARTS {
            assert_ne!(
                render(Some(*part)),
                plain,
                "Panel declares {part:?} and paints nothing with it"
            );
        }
    }

    #[test]
    fn the_slot_addressable_parts_are_exactly_the_documented_ones() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 24,
            height: 5,
        };
        let marker = |ui: &mut Ui<'_>, r: Rect| {
            let s = ui.surface_style();
            ui.paint_str(r, "ZZZZ", s);
        };
        let render = |slot: Option<Part>| {
            let mut rt = Runtime::new(Stub::default(), Theme::junie());
            let mut buf = Buffer::empty(SCREEN);
            rt.draw_scene(SCREEN, &mut buf, |ui, _| {
                let mut p = Panel::new(ID)
                    .kind(PanelKind::Framed)
                    .title("Files")
                    .meta("12")
                    .badge("EDIT")
                    .focused(true);
                if let Some(part) = slot {
                    p = p.slot(part, &marker);
                }
                p.draw(ui, area, |_, inner| inner);
            })
            .commit_presented();
            buf
        };
        let plain = render(None);

        for part in [
            Part::GUTTER,
            Part::TITLE,
            Part::DETAIL,
            Part::BADGE,
            Part::BORDER,
        ] {
            assert_ne!(
                render(Some(part)),
                plain,
                "`## Overrides` grants a slot on {part:?} and it is dropped"
            );
        }

        assert_eq!(
            render(Some(Part::CONTAINER)),
            plain,
            "a slot on Part::CONTAINER changes cells, and `## Overrides` says it does not"
        );
    }
}
