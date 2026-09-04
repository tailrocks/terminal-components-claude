//! `Panel` — the surface container (`COMPONENT_ARCHITECTURE.md` §14.1,
//! §18.2 `panel`, Appendix A 4E).

use core::fmt;

use ratatui_core::layout::Rect;

use super::{Overrides, SlotFn, cell_at, first_row};
use crate::id::{Id, Part, PartRef};
use crate::layout::{Insets, inset};
use crate::measure::{Constraints, Size};
use crate::response::StateFlags;
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Surface, Variant};
use crate::ui::{FrameRead, Ui};

/// How a panel marks its edge.
///
/// This is a geometry decision, not a look: a card pads its content and
/// raises the surface, a frame draws a border and keeps the surface. A
/// [`Variant`] could not express it, because a recipe binds styles and
/// cannot move the inner rect.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PanelKind {
    /// A filled rectangle one surface level above its parent, no border.
    #[default]
    Card,
    /// A bordered pane on the parent's own surface.
    Framed,
}

/// A titled container that fills a rectangle, marks its edge and hands its
/// content the inner rect on its own surface.
///
/// ## Construction
/// `Panel::new(id)` — a [`PanelKind::Card`]. `.kind(PanelKind::Framed)`
/// selects the bordered pane.
///
/// ## Ownership
/// The caller owns the title and the meta text (`&'a str`) and the
/// `focused` predicate. `Panel` is stateless (§3: no `PanelState`) and the
/// runtime owns nothing on its behalf beyond the decorative hit regions
/// `draw` registers.
///
/// ## Configuration
/// `.kind(PanelKind)` (`Card`), `.title(&str)` (none), `.meta(&str)`
/// (none), `.focused(bool)` (`false`), `.patch`, `.patch_part`, `.slot`,
/// `.state_override`.
///
/// ## Variants
/// `Family::PANEL`, `Variant::DEFAULT` only; `Recipe.default_variant` is
/// `DEFAULT`. Card versus framed is [`PanelKind`], not a `Variant`.
///
/// ## States
/// `FOCUSED` only, and it is **props-derived**: `.focused(true)` is the
/// caller saying "the region I contain holds focus". `Panel` registers no
/// focus stop, so the runtime half of its state is always empty.
///
/// ## Actions
/// None. `Panel` has no `update` phase and emits no action: it owns no
/// interaction state, consumes no key and claims no pointer. Every region
/// it registers is `Decorative`, so an intent resolving to it is discarded
/// silently (§6.1).
///
/// ## Focus
/// Never a focus stop; `Focusability` is not used. It opens no scope and
/// traps nothing. The focused *control* inside the panel keeps its own
/// accent gutter; the panel's own gutter marks the container.
///
/// ## Keyboard
/// None. `Panel` declares no [`crate::keymap::Bindings`] table.
///
/// ## Mouse
/// None. `Part::CONTAINER` and, when framed, `Part::BORDER` are registered
/// with [`Ui::register_decor`] so `area_of_part` answers and a click lands
/// on the panel's content rather than on a stale sibling; neither delivers
/// a `Pointer` intent.
///
/// ## Layout
/// The head row is `area`'s first row: the focus gutter at `area.x + 1`,
/// the title from `area.x + 2`, the meta right-aligned. A card insets by
/// `design.space.card_inset` horizontally and one row vertically, plus one
/// more row when it has a title; a frame insets by
/// `design.space.frame_inset` horizontally and one row vertically. The
/// inner rect is computed with [`crate::layout::inset`], which clamps, so
/// it can never escape `area` — the `width ≤ 4` escape of the legacy
/// framed panel is structurally impossible. `draw` returns the body
/// closure's value, or `None` when `area` is empty (R5).
///
/// ## Parts
/// `CONTAINER` (the fill), `GUTTER` (the container focus bar), `TITLE`,
/// `DETAIL` (the right-aligned meta), `BORDER` (framed only).
///
/// ## Overrides
/// `.patch` and `.patch_part` reach every part. `.slot` is honoured for
/// `Part::GUTTER`, `Part::TITLE`, `Part::DETAIL` and `Part::BORDER`.
/// `Part::CONTAINER` is **not** slot-addressable: it is the plane the
/// panel pushes and the body inherits, so replacing it would leave the
/// content painted against a surface nothing filled.
///
/// ## Identity
/// One `Id`; no items and no `ItemKey`.
///
/// ## Testing
/// `PanelCase` with `Caps::empty()`; `render::components::panel::*`.
///
/// ## Invariants
/// The inner rect is always a subrect of `area`. The body closure runs
/// with the panel's own [`Surface`] pushed, so a child resolving
/// `Role::CurrentSurface` gets the plane the panel filled. A forced state
/// (A11) registers nothing.
pub struct Panel<'a> {
    id: Id,
    kind: PanelKind,
    title: Option<&'a str>,
    meta: Option<&'a str>,
    focused: bool,
    ov: Overrides<'a>,
}

impl fmt::Debug for Panel<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Panel")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .field("title", &self.title)
            .field("meta", &self.meta)
            .field("focused", &self.focused)
            .field("overrides", &self.ov)
            .finish()
    }
}

impl<'a> Panel<'a> {
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::GUTTER,
        Part::TITLE,
        Part::DETAIL,
        Part::BORDER,
    ];

    /// A card panel.
    pub const fn new(id: Id) -> Self {
        Panel {
            id,
            kind: PanelKind::Card,
            title: None,
            meta: None,
            focused: false,
            ov: Overrides::new(),
        }
    }

    /// The id.
    pub const fn id(&self) -> Id {
        self.id
    }

    /// Card or framed.
    #[must_use]
    pub const fn kind(mut self, k: PanelKind) -> Self {
        self.kind = k;
        self
    }

    /// The title, painted in the head row.
    #[must_use]
    pub const fn title(mut self, t: &'a str) -> Self {
        self.title = Some(t);
        self
    }

    /// Right-aligned secondary text in the head row.
    #[must_use]
    pub const fn meta(mut self, m: &'a str) -> Self {
        self.meta = Some(m);
        self
    }

    /// Whether the region this panel contains holds focus.
    #[must_use]
    pub const fn focused(mut self, yes: bool) -> Self {
        self.focused = yes;
        self
    }

    /// An instance patch over every part.
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.patch(p);
        self
    }

    /// Per-part instance patches.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.patch_part(ps);
        self
    }

    /// Replace one part's painting.
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    /// Showcase / fixture use only (A11).
    #[must_use]
    pub const fn state_override(mut self, s: StateFlags) -> Self {
        self.ov = self.ov.state_override(s);
        self
    }

    /// The state the panel's own props imply — §39's derived half, which a
    /// forced state may add to and may never erase.
    const fn derived(&self) -> StateFlags {
        if self.focused {
            StateFlags::FOCUSED
        } else {
            StateFlags::empty()
        }
    }

    /// The surface the panel fills and its body inherits: a card raises the
    /// current plane by one ladder step, a frame keeps it and marks the edge
    /// with a border instead (§10).
    fn plane(&self, ui: &Ui<'_>) -> Surface {
        match self.kind {
            PanelKind::Card => ui.theme().raise(ui.surface()),
            PanelKind::Framed => ui.surface(),
        }
    }

    /// One row of vertical padding, plus a second on a card that has a
    /// title, plus the frame's own row.
    const fn top_inset(&self) -> u16 {
        match self.kind {
            PanelKind::Card if self.title.is_some() => 2,
            PanelKind::Card | PanelKind::Framed => 1,
        }
    }

    /// The content rect for `area`, without painting anything.
    ///
    /// Pure geometry: the same arithmetic [`Panel::draw`] hands the body
    /// closure, so a caller that needs the inner rect in `update` — to size
    /// a child, or to decide whether the content fits — reads it here
    /// instead of guessing.
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
            // `Rect::inner` — the symmetric branch of `layout::inset` — answers
            // `Rect::ZERO` when the margin does not fit, and `Rect::ZERO`'s
            // origin is `(0, 0)`, not the panel's. An empty rect is the right
            // answer; an empty rect *somewhere else on the screen* is the
            // escape §18.2 asks to fix, because a caller that reads `.x`
            // before checking `.is_empty()` lands outside the panel.
            return Rect {
                x: area.x,
                y: area.y,
                width: 0,
                height: 0,
            };
        }
        r
    }

    /// The natural size: the chrome plus one content cell.
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
        Size {
            min: (chrome_w.saturating_add(1), chrome_h.saturating_add(1)),
            preferred: (
                chrome_w.saturating_add(1).max(head),
                chrome_h.saturating_add(1),
            ),
        }
        .fit(c)
    }

    /// Paint the chrome, then run `body` on the inner rect with the panel's
    /// own surface pushed.
    ///
    /// `None` when `area` is empty: there is no rect to hand the body and
    /// nothing was painted or registered (R5).
    pub fn draw<R>(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        body: impl FnOnce(&mut Ui<'_>, Rect) -> R,
    ) -> Option<R> {
        if area.is_empty() {
            return None;
        }
        let plane = self.plane(ui);
        Some(ui.with_surface(plane, |ui| {
            let inner = self.inner(ui, area);
            self.chrome(ui, area);
            body(ui, inner)
        }))
    }

    /// The fill, the border and the head row.
    fn chrome(&self, ui: &mut Ui<'_>, area: Rect) {
        let ov = self.ov;
        let id = self.id;
        let live = ov.flags(StateFlags::empty(), self.derived());
        let forced = ov.is_forced();
        let container = ov.style(
            ui,
            id,
            Family::PANEL,
            Variant::DEFAULT,
            Part::CONTAINER,
            live,
        );
        ui.fill(area, container.style);
        if !forced {
            ui.register_decor(id, PartRef::of(Part::CONTAINER), area);
        }
        if self.kind == PanelKind::Framed {
            let border = ov.style(ui, id, Family::PANEL, Variant::DEFAULT, Part::BORDER, live);
            if let Some(f) = ov.slot_for(Part::BORDER) {
                f(ui, area);
            } else {
                ui.frame(area, border.style);
            }
            if !forced {
                ui.register_decor(id, PartRef::of(Part::BORDER), area);
            }
        }
        self.head(ui, area, live, container.style);
    }

    /// The head row: focus gutter, title, right-aligned meta.
    fn head(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        live: StateFlags,
        fill: ratatui_core::style::Style,
    ) {
        let head = first_row(area);
        if head.is_empty() || area.width < 3 {
            return;
        }
        let ov = self.ov;
        let id = self.id;
        // one blank cell either side of a run, so a framed panel's head reads
        // as a label cut into the rule rather than text jammed against it
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
        // the head span never touches the corner columns
        let span_w = area.width.saturating_sub(3);
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
            if pad == 1 {
                ui.fill(cell_at(head, x.saturating_sub(1)), fill);
            }
            let used = if let Some(f) = ov.slot_for(Part::DETAIL) {
                f(ui, rect);
                rect.width
            } else {
                let s = ov.style(ui, id, Family::PANEL, Variant::DEFAULT, Part::DETAIL, live);
                ui.paint_str(rect, m, s.style)
            };
            if pad == 1 && used > 0 {
                ui.fill(cell_at(head, x.saturating_add(used)), fill);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Position;

    use super::*;
    use crate::runtime::Runtime;
    use crate::runtime::stub::{SCREEN, Stub};
    use crate::theme::{Role, Theme};

    const ID: Id = Id::root("panel.tests");

    fn symbol_at(buf: &Buffer, x: u16, y: u16) -> String {
        buf.cell(Position::new(x, y))
            .map_or_else(String::new, |c| c.symbol().to_owned())
    }

    /// §18.2's `panel` row: *"framed inner rect escaping the panel for
    /// `width ≤ 4` **fixed**"*. The legacy arithmetic was
    /// `area.inner(Margin(1, 1))` followed by `x + 2` and
    /// `width.saturating_sub(3)`, which put the inner rect's origin outside
    /// `area` for every narrow panel — a caller that painted into it wrote
    /// over its neighbour. The fix is structural: the inset goes through
    /// [`crate::layout::inset`], which clamps, so there is no arithmetic left
    /// that can escape.
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
        });
    }

    /// The head row is confined to `area.x + 1 ..= area.right() - 2`, so a
    /// framed panel's four corner glyphs survive a title and a meta that are
    /// both far too long for the row. Without the confinement the title runs
    /// over the top-right corner and the frame stops reading as a frame.
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
        });
        let b = Theme::junie().design.borders;
        assert_eq!(symbol_at(&buf, 0, 0), b.top_left);
        assert_eq!(symbol_at(&buf, 19, 0), b.top_right);
        assert_eq!(symbol_at(&buf, 0, 4), b.bottom_left);
        assert_eq!(symbol_at(&buf, 19, 4), b.bottom_right);
    }

    /// `.focused(true)` is the **props-derived** half of §39's Invariant Q:
    /// the panel registers no focus stop, so the runtime can never supply
    /// `FOCUSED`, and the only way the container focus bar can appear is the
    /// prop. It must appear, and it must not appear otherwise — that one
    /// glyph is the whole of the panel's focus affordance at
    /// `ColorLevel::Mono`.
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
            });
            symbol_at(&buf, 1, 0)
        };
        assert_eq!(render(true), bar, "a focused panel paints no focus bar");
        assert_ne!(
            render(false),
            bar,
            "an unfocused panel paints the focus bar"
        );
    }

    /// A `Panel` is a container: every region it registers is `Decorative`,
    /// so it never enters the focus ring and an intent that resolves to it is
    /// discarded silently (§6.1, §21's decorative-owner rule).
    #[test]
    fn a_panel_registers_no_focus_stop() {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_scene(SCREEN, &mut buf, |ui, a| {
            Panel::new(ID)
                .kind(PanelKind::Framed)
                .title("Files")
                .draw(ui, a, |_, inner| inner);
        });
        assert!(!rt.ring().is_registered(ID));
        assert_eq!(rt.ring().reachable().count(), 0);
        assert!(rt.area_of(ID).is_some(), "the container is not addressable");
    }

    /// A forced rendering (A11) registers nothing at all, so a reference
    /// panel on a showcase page cannot become a live hit target.
    #[test]
    fn a_forced_panel_registers_nothing() {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_scene(SCREEN, &mut buf, |ui, a| {
            Panel::new(ID)
                .kind(PanelKind::Framed)
                .title("Files")
                .state_override(StateFlags::FOCUSED)
                .draw(ui, a, |_, inner| inner);
        });
        assert!(rt.area_of(ID).is_none());
    }

    /// §33's Invariant P, in the direction the conformance registry cannot
    /// check on its own: **every** declared part must be one this component
    /// actually resolves, proven by the property — a per-part patch on it
    /// changes the painted cells — and not by reading the const back. A part
    /// declared and never painted is `PARTS` lying about the styling surface.
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
                    .focused(true);
                if patched.is_some() {
                    p = p.patch_part(&ps);
                }
                p.draw(ui, area, |_, inner| inner);
            });
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

    /// §45's Invariant R: the `## Overrides` section is a contract, so the
    /// parts it names as slot-addressable are **exactly** the parts for which
    /// installing a slot changes the painted cells. Both directions are
    /// asserted, because §45.1 found six components over-claiming and one
    /// under-claiming.
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
                    .focused(true);
                if let Some(part) = slot {
                    p = p.slot(part, &marker);
                }
                p.draw(ui, area, |_, inner| inner);
            });
            buf
        };
        let plain = render(None);
        // documented as slot-addressable
        for part in [Part::GUTTER, Part::TITLE, Part::DETAIL, Part::BORDER] {
            assert_ne!(
                render(Some(part)),
                plain,
                "`## Overrides` grants a slot on {part:?} and it is dropped"
            );
        }
        // documented as NOT slot-addressable
        assert_eq!(
            render(Some(Part::CONTAINER)),
            plain,
            "a slot on Part::CONTAINER changes cells, and `## Overrides` says it does not"
        );
    }
}
