#![doc = r" `Panel` — the surface container (`COMPONENT_ARCHITECTURE.md` §14.1,"]
#![doc = r" §18.2 `panel`, Appendix A 4E)."]

use core::fmt;

use ratatui_core::layout::Rect;

use super::{PartStyle, SlotFn, cell_at, first_row};
use crate::id::{Id, Part, PartRef};
use crate::layout::{Insets, inset};
use crate::measure::{Constraints, Size};
use crate::response::StateFlags;
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Surface, Variant};
use crate::ui::{FrameRead, Ui};

#[doc = r" How a panel marks its edge."]
#[doc = r""]
#[doc = r" This is a geometry decision, not a look: a card pads its content and"]
#[doc = r" raises the surface, a frame draws a border and keeps the surface. A"]
#[doc = r" [`Variant`] could not express it, because a recipe binds styles and"]
#[doc = r" cannot move the inner rect."]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PanelKind {
    #[doc = r" A filled rectangle one surface level above its parent, no border."]
    #[default]
    Card,
    #[doc = r" A bordered pane on the parent's own surface."]
    Framed,
}

#[doc = r" A titled container that fills a rectangle, marks its edge and hands its"]
#[doc = r" content the inner rect on its own surface."]
#[doc = r""]
#[doc = r" ## Construction"]
#[doc = r" `Panel::new(id)` — a [`PanelKind::Card`]. `.kind(PanelKind::Framed)`"]
#[doc = r" selects the bordered pane."]
#[doc = r""]
#[doc = r" ## Ownership"]
#[doc = r" The caller owns the title, meta and badge text (`&'a str`) and the"]
#[doc = r" `focused` predicate. `Panel` is stateless (§3: no `PanelState`) and the"]
#[doc = r" runtime owns nothing on its behalf beyond the decorative hit regions"]
#[doc = r" `draw` registers."]
#[doc = r""]
#[doc = r" ## Configuration"]
#[doc = r" `.kind(PanelKind)` (`Card`), `.title(&str)` (none), `.meta(&str)`"]
#[doc = r" (none), `.badge(&str)` (none), `.focused(bool)` (`false`), `.patch`,"]
#[doc = r" `.patch_part`, `.slot`, reference fixtures use"]
#[doc = r" [`Ui::reference`](crate::Ui::reference)."]
#[doc = r""]
#[doc = r" ## Variants"]
#[doc = r" `Family::PANEL`, `Variant::DEFAULT` only; `Recipe.default_variant` is"]
#[doc = r" `DEFAULT`. Card versus framed is [`PanelKind`], not a `Variant`."]
#[doc = r""]
#[doc = r" ## States"]
#[doc = r" `FOCUSED` only, and it is **props-derived**: `.focused(true)` is the"]
#[doc = r#" caller saying "the region I contain holds focus". `Panel` registers no"#]
#[doc = r" focus stop, so the runtime half of its state is always empty."]
#[doc = r""]
#[doc = r" ## Actions"]
#[doc = r" None. `Panel` has no `update` phase and emits no action: it owns no"]
#[doc = r" interaction state, consumes no key and claims no pointer. Every region"]
#[doc = r" it registers is `Decorative`, so an intent resolving to it is discarded"]
#[doc = r" silently (§6.1)."]
#[doc = r""]
#[doc = r" ## Focus"]
#[doc = r" Never a focus stop; `Focusability` is not used. It opens no scope and"]
#[doc = r" traps nothing. The focused *control* inside the panel keeps its own"]
#[doc = r" accent gutter; the panel's own gutter marks the container."]
#[doc = r""]
#[doc = r" ## Keyboard"]
#[doc = r" None. `Panel` declares no [`crate::keymap::Bindings`] table."]
#[doc = r""]
#[doc = r" ## Mouse"]
#[doc = r" None. `Part::CONTAINER` and, when framed, `Part::BORDER` are registered"]
#[doc = r" with [`Ui::register_decor`] so `area_of_part` answers and a click lands"]
#[doc = r" on the panel's content rather than on a stale sibling; neither delivers"]
#[doc = r" a `Pointer` intent."]
#[doc = r""]
#[doc = r" ## Layout"]
#[doc = r" The head row is `area`'s first row: the focus gutter at `area.x + 1`,"]
#[doc = r" the title from `area.x + 2`, the meta right-aligned. A card insets by"]
#[doc = r" `design.space.card_inset` horizontally and one row vertically, plus one"]
#[doc = r" more row when it has a title; a frame insets by"]
#[doc = r" `design.space.frame_inset` horizontally and one row vertically. The"]
#[doc = r" inner rect is computed with [`crate::layout::inset`], which clamps, so"]
#[doc = r" it can never escape `area` — the `width ≤ 4` escape of the legacy"]
#[doc = r" framed panel is structurally impossible. `draw` returns the body"]
#[doc = r" closure's value. The body runs exactly once even for an empty area; its"]
#[doc = r" painting is clipped to the anchored inner rect."]
#[doc = r""]
#[doc = r" ## Parts"]
#[doc = r" `CONTAINER` (the fill), `GUTTER` (the container focus bar), `TITLE`,"]
#[doc = r" `DETAIL` (the right-aligned meta), `BADGE` (padded, before meta), `BORDER`"]
#[doc = r" (framed only)."]
#[doc = r""]
#[doc = r" ## Overrides"]
#[doc = r" `.patch` and `.patch_part` reach every part. `.slot` is honoured for"]
#[doc = r" `Part::GUTTER`, `Part::TITLE`, `Part::DETAIL`, `Part::BADGE` and"]
#[doc = r" `Part::BORDER`."]
#[doc = r" `Part::CONTAINER` is **not** slot-addressable: it is the plane the"]
#[doc = r" panel pushes and the body inherits, so replacing it would leave the"]
#[doc = r" content painted against a surface nothing filled."]
#[doc = r""]
#[doc = r" ## Identity"]
#[doc = r" One `Id`; no items and no `ItemKey`."]
#[doc = r""]
#[doc = r" ## Testing"]
#[doc = r" `PanelCase` with `Caps::empty()`; `render::components::panel::*`."]
#[doc = r""]
#[doc = r" ## Invariants"]
#[doc = r" The inner rect is always a subrect of `area`. The body closure runs"]
#[doc = r" with the panel's own [`Surface`] pushed, so a child resolving"]
#[doc = r" `Role::CurrentSurface` gets the plane the panel filled. An outer"]
#[doc = r" [`Ui::reference`](crate::Ui::reference) scope suppresses registrations."]
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
    #[doc = r" The parts this component styles."]
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::GUTTER,
        Part::TITLE,
        Part::DETAIL,
        Part::BADGE,
        Part::BORDER,
    ];

    #[doc = r" A card panel."]
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

    #[doc = r" The id."]
    pub const fn id(&self) -> Id {
        self.id
    }

    #[doc = r" Card or framed."]
    #[must_use]
    pub const fn kind(mut self, k: PanelKind) -> Self {
        self.kind = k;
        self
    }

    #[doc = r" The title, painted in the head row."]
    #[must_use]
    pub const fn title(mut self, t: &'a str) -> Self {
        self.title = Some(t);
        self
    }

    #[doc = r" Right-aligned secondary text in the head row."]
    #[must_use]
    pub const fn meta(mut self, m: &'a str) -> Self {
        self.meta = Some(m);
        self
    }

    #[doc = r" A padded badge before the right-aligned metadata, styled by `Part::BADGE`."]
    #[doc = r""]
    #[doc = r" The `PANEL/BADGE` recipe owns its colors and emphasis; callers supply"]
    #[doc = r" only the label, so capability downgrade and theme mapping stay shared."]
    #[doc = r" Empty badges are absent. A badge is hidden when its complete text,"]
    #[doc = r" padding and gaps cannot fit alongside the title and metadata."]
    #[must_use]
    pub const fn badge(mut self, text: &'a str) -> Self {
        self.badge = if text.is_empty() { None } else { Some(text) };
        self
    }

    #[doc = r" Whether the region this panel contains holds focus."]
    #[must_use]
    pub const fn focused(mut self, yes: bool) -> Self {
        self.focused = yes;
        self
    }

    #[doc = r" An instance patch over every part."]
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[doc = r" Per-part instance patches."]
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[doc = r" Replace one part's painting."]
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    #[doc = r" The state the panel's own props imply."]
    const fn derived(&self) -> StateFlags {
        if self.focused {
            StateFlags::FOCUSED
        } else {
            StateFlags::empty()
        }
    }

    #[doc = r" The surface the panel fills and its body inherits: a card raises the"]
    #[doc = r" current plane by one ladder step, a frame keeps it and marks the edge"]
    #[doc = r" with a border instead (§10)."]
    fn plane(&self, ui: &Ui<'_>) -> Surface {
        match self.kind {
            PanelKind::Card => ui.theme().raise(ui.surface()),
            PanelKind::Framed => ui.surface(),
        }
    }

    #[doc = r" One row of vertical padding, plus a second on a card that has a"]
    #[doc = r" title, plus the frame's own row."]
    const fn top_inset(&self) -> u16 {
        match self.kind {
            PanelKind::Card if self.title.is_some() => 2,
            PanelKind::Card | PanelKind::Framed => 1,
        }
    }

    #[doc = r" The content rect for `area`, without painting anything."]
    #[doc = r""]
    #[doc = r" Pure geometry: the same arithmetic [`Panel::draw`] hands the body"]
    #[doc = r" closure, so a caller that needs the inner rect in `update` — to size"]
    #[doc = r" a child, or to decide whether the content fits — reads it here"]
    #[doc = r" instead of guessing."]
    pub fn inner(&self, ui: &Ui<'_>, area: Rect) -> Rect {
        let side = match self.kind {
            PanelKind::Card => ui.design().space.card_inset,
            PanelKind::Framed => ui.design().space.frame_inset,
        };
        let r = inset(
            area,
            Insets {
                l: side,
                t: self.top_inset(),
                r: side,
                b: 1,
            },
        );
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

    #[doc = r" The head-row width the badge lane needs to sit beside the title and"]
    #[doc = r" the metadata, or `None` without a badge. The single source of truth"]
    #[doc = r" shared by `measure` (which reports it as the preferred width) and"]
    #[doc = r" `head` (which falls back to the no-badge layout below it), so the two"]
    #[doc = r" can never disagree about when the badge fits."]
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

    #[doc = r" The natural size: the chrome plus one content cell."]
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        let side = match self.kind {
            PanelKind::Card => ui.design().space.card_inset,
            PanelKind::Framed => ui.design().space.frame_inset,
        };
        let chrome_w = side.saturating_mul(2);
        let chrome_h = self.top_inset().saturating_add(1);
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

    #[doc = r" Paint the chrome, then run `body` on the inner rect with the panel's"]
    #[doc = r" own surface pushed."]
    #[doc = r""]
    #[doc = r" The body runs exactly once for every area. On an empty or tiny area it"]
    #[doc = r" receives an empty rect anchored inside `area`, and all effects remain"]
    #[doc = r" clipped to that rect (R4/R5)."]
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

    #[doc = r" The fill, the border and the head row."]
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

    #[doc = r" Paint the badge into `rect`: the slot replacement when one is"]
    #[doc = r" configured, otherwise the padded label composited over the panel's"]
    #[doc = r" surface (§22 R‑9) so the badge never resets the plane it sits on."]
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

    #[doc = r" The head row: focus gutter, title, badge and right-aligned meta."]
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

#[doc = r" Paint a title/meta label, adding the legacy frame padding without an"]
#[doc = r" allocation. Only the label's cells are touched; unused head-row space"]
#[doc = r" keeps the frame rule underneath it."]
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

    #[doc = r#" §18.2's `panel` row: *"framed inner rect escaping the panel for"#]
    #[doc = r#" `width ≤ 4` **fixed**"*. The legacy arithmetic was"#]
    #[doc = r" `area.inner(Margin(1, 1))` followed by `x + 2` and"]
    #[doc = r" `width.saturating_sub(3)`, which put the inner rect's origin outside"]
    #[doc = r" `area` for every narrow panel — a caller that painted into it wrote"]
    #[doc = r" over its neighbour. The fix is structural: the inset goes through"]
    #[doc = r" [`crate::layout::inset`], which clamps, so there is no arithmetic left"]
    #[doc = r" that can escape."]
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

    #[doc = r" The head row is confined to `area.x + 1 ..= area.right() - 2`, so a"]
    #[doc = r" framed panel's four corner glyphs survive a title and a meta that are"]
    #[doc = r" both far too long for the row. Without the confinement the title runs"]
    #[doc = r" over the top-right corner and the frame stops reading as a frame."]
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

    #[doc = r" `.focused(true)` is the **props-derived** half of §39's Invariant Q:"]
    #[doc = r" the panel registers no focus stop, so the runtime can never supply"]
    #[doc = r" `FOCUSED`, and the only way the container focus bar can appear is the"]
    #[doc = r" prop. It must appear, and it must not appear otherwise — that one"]
    #[doc = r" glyph is the whole of the panel's focus affordance at"]
    #[doc = r" `ColorLevel::Mono`."]
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

    #[doc = r" A `Panel` is a container: every region it registers is `Decorative`,"]
    #[doc = r" so it never enters the focus ring and an intent that resolves to it is"]
    #[doc = r" discarded silently (§6.1, §21's decorative-owner rule)."]
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

    #[doc = r" A reference panel cannot become a live hit target."]
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

    #[doc = r" §33's Invariant P, in the direction the conformance registry cannot"]
    #[doc = r" check on its own: **every** declared part must be one this component"]
    #[doc = r" actually resolves, proven by the property — a per-part patch on it"]
    #[doc = r" changes the painted cells — and not by reading the const back. A part"]
    #[doc = r" declared and never painted is `PARTS` lying about the styling surface."]
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

    #[doc = r" §45's Invariant R: the `## Overrides` section is a contract, so the"]
    #[doc = r" parts it names as slot-addressable are **exactly** the parts for which"]
    #[doc = r" installing a slot changes the painted cells. Both directions are"]
    #[doc = r" asserted, because §45.1 found six components over-claiming and one"]
    #[doc = r" under-claiming."]
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
