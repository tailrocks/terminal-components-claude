//! `Brand` — the accent-filled identity lockup (`COMPONENT_ARCHITECTURE.md`
//! §11.6, §18.2, Appendix A 4G).

use core::fmt;

use ratatui_core::layout::Rect;

use super::{PartStyle, SlotFn, first_row, shift};
use crate::focus::Focusability;
use crate::id::{Id, Part};
use crate::intent::{Intent, Phase};
use crate::measure::{Constraints, Size};
use crate::response::{Activated, Response, StateFlags};
use crate::text::width;
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};

/// The product mark: the one lockup that fills with the accent.
///
/// ## Construction
/// `Brand::new(id, text)`. The mark text is always the application's; the
/// library bakes in no wordmark.
///
/// ## Ownership
/// Stateless. The caller owns the mark text and the optional tagline; the
/// runtime owns hover and press when the lockup is `.clickable(true)`.
///
/// ## Configuration
/// `.variant(Variant)` (default `Recipe.default_variant`), `.compact(bool)`
/// (`false` — drops the one-cell fill padding for tight strips),
/// `.tagline(&str)` (none), `.clickable(bool)` (`false`), `.patch`,
/// `.patch_part`, `.slot`.
///
/// ## Variants
/// `Family::BRAND`; `DEFAULT` only. The accent fill, the on-accent
/// foreground and the bold weight are `Theme::junie()` recipe defaults, not
/// component code (§11.6) — a theme restyles the lockup without touching
/// this file.
///
/// ## States
/// Wears `HOVERED` and `PRESSED` from the runtime when `.clickable(true)`;
/// derives nothing. A non-clickable lockup registers nothing and wears no
/// state at all.
///
/// ## Actions
/// `Response<Activated>` — `Activated` on a click, and only when
/// `.clickable(true)`.
///
/// ## Focus
/// `Focusability::ClickOnly` when clickable, so the lockup takes hover and
/// press but never enters the tab ring: an identity mark is not a control an
/// operator tabs to. `swallows_typing` is false; `.autofocus()` does not
/// exist.
///
/// ## Keyboard
/// None. The lockup has no chord and publishes no `Bindings` table; a
/// product that wants a keyboard path binds one through its own `KeyMap`.
///
/// ## Mouse
/// `PartRef::of(Part::LABEL)` when clickable: press and release paint the
/// pressed plane, a click emits `Activated`.
///
/// ## Layout
/// `measure` returns `(pad + mark + pad [+ gap + tagline], 1)` where `pad` is
/// `0` under `.compact(true)` and `1` otherwise. `draw` uses the first row of
/// `area`, clipped to that width, and returns the rect it painted; a
/// degenerate rect registers nothing and paints nothing (R5).
///
/// ## Parts
/// `LABEL` (the filled lockup), `META` (the tagline beside it).
///
/// ## Overrides
/// `.patch` and `.patch_part` reach `Part::LABEL` and `Part::META`.
/// `.slot(p, …)` changes painted cells for exactly `Part::LABEL` and
/// `Part::META` — every part `Brand` declares, so nothing is excluded. In
/// particular there is no `CONTAINER` to exclude: the accent fill is
/// `LABEL`'s own painting rather than a surface behind it, so a `LABEL`
/// slot replaces fill and mark together, and the lockup keeps its rect, its
/// `ClickOnly` control and its `PartRef::of(Part::LABEL)` hit region across
/// the substitution (§45.4). `META` is painted, and therefore
/// slot-addressable, only when `.tagline(s)` carries a non-empty string and
/// the row still has columns after the lockup and the gap; with no tagline
/// the component paints no `META` cell for a slot to replace.
///
/// ## Identity
/// One `Id` per instance; no items.
///
/// ## Testing
/// `BrandCase` with no capabilities;
/// `render::components::brand::{default, pressed, empty}`;
/// `brand::a_slot_changes_painted_cells_for_exactly_label_and_meta`, which
/// asserts Invariant R (§45.3) in both directions — a named part that stops
/// changing cells and an unnamed part that starts changing them each fail
/// it.
///
/// `BrandCase` deliberately declares `Caps::empty()`: the conformance fixture
/// is the non-interactive lockup, while the opt-in `ClickOnly` path is covered
/// by the runtime-driven unit tests in this module:
/// `only_a_clickable_lockup_registers_a_click_only_control_and_a_label_part`,
/// `only_a_clickable_lockup_emits_activated_on_a_click` and
/// `only_a_clickable_lockup_lifts_to_accent_hover_under_the_pointer`. Each
/// asserts the plain lockup does **none** of it, so both the mouse-only path
/// and the non-interactive default remain executable and pinned.
///
/// ## Invariants
/// The mono `PRESSED` bracket rule (§11.4) is honoured only when the lockup
/// has room for both brackets — a `.compact(true)` lockup reserves no padding
/// and a mono fallback must never change geometry, so it keeps the plain
/// mark. Never allocates; never writes outside `area`.
pub struct Brand<'a> {
    id: Id,
    text: &'a str,
    tagline: Option<&'a str>,
    variant: Variant,
    compact: bool,
    clickable: bool,
    ov: PartStyle<'a>,
}

impl fmt::Debug for Brand<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Brand")
            .field("id", &self.id)
            .field("text", &self.text)
            .field("tagline", &self.tagline)
            .field("compact", &self.compact)
            .field("clickable", &self.clickable)
            .field("overrides", &self.ov)
            .finish_non_exhaustive()
    }
}

impl<'a> Brand<'a> {
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[Part::LABEL, Part::META];

    /// A lockup showing `text`.
    pub const fn new(id: Id, text: &'a str) -> Self {
        Brand {
            id,
            text,
            tagline: None,
            variant: Variant::DEFAULT,
            compact: false,
            clickable: false,
            ov: PartStyle::new(),
        }
    }

    /// The id.
    pub const fn id(&self) -> Id {
        self.id
    }

    /// Set the variant.
    #[must_use]
    pub const fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    /// Drop the fill padding for a tight strip; the treatment is unchanged.
    #[must_use]
    pub const fn compact(mut self, yes: bool) -> Self {
        self.compact = yes;
        self
    }

    /// A muted line beside the lockup.
    #[must_use]
    pub const fn tagline(mut self, s: &'a str) -> Self {
        self.tagline = Some(s);
        self
    }

    /// Take hover, press and clicks.
    #[must_use]
    pub const fn clickable(mut self, yes: bool) -> Self {
        self.clickable = yes;
        self
    }

    /// An instance patch over every part (precedence 6).
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    /// Per-part patches.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    /// Replace one part's painting; layout and hit regions stay.
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    /// The fill padding either side of the mark.
    const fn pad(&self) -> u16 {
        if self.compact { 0 } else { 1 }
    }

    /// Columns the filled lockup occupies.
    fn lockup_width(&self) -> u16 {
        width(self.text)
            .saturating_add(self.pad())
            .saturating_add(self.pad())
    }

    /// Columns the whole component occupies, tagline included.
    fn natural_width(&self, ui: &Ui<'_>) -> u16 {
        match self.tagline {
            Some(t) if !t.is_empty() => self
                .lockup_width()
                .saturating_add(ui.design().space.gap.max(1))
                .saturating_add(width(t)),
            _ => self.lockup_width(),
        }
    }

    /// The update phase; only a `.clickable(true)` lockup can act.
    pub fn update(&self, cx: &mut Cx<'_>) -> Response<Activated> {
        if !self.clickable {
            return Response::ignored();
        }
        let mut r: Response<Activated> = Response::ignored();
        for it in cx.intents(self.id) {
            match it {
                Intent::Pointer {
                    phase: Phase::Click | Phase::DoubleClick,
                    ..
                } => r = Response::action(Activated),
                Intent::Pointer { .. } if !r.activated() => r = Response::changed(),
                _ => {}
            }
        }
        r.for_id(self.id)
    }

    /// The draw phase; returns the rect painted.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        let area = Rect {
            width: self.natural_width(ui).min(area.width),
            ..first_row(area)
        };
        if area.is_empty() {
            return area;
        }
        let lockup = Rect {
            width: self.lockup_width().min(area.width),
            ..area
        };
        if self.clickable && !ui.is_inert() {
            ui.register_control(self.id, lockup, Focusability::ClickOnly);
        }
        // runtime only, and only while clickable — an unclickable lockup
        // registers nothing, so the snapshot has nothing to say about it
        let live = PartStyle::flags(
            if self.clickable {
                ui.state(self.id)
            } else {
                StateFlags::empty()
            },
            StateFlags::empty(),
        );
        let ov = self.ov;
        if let Some(f) = ov.slot_for(Part::LABEL) {
            f(ui, lockup);
        } else {
            let s = ov.style(ui, self.id, Family::BRAND, self.variant, Part::LABEL, live);
            ui.fill(lockup, s.style);
            let pad = self.pad();
            let bracketed = matches!(s.glyph, Slot::Set(GlyphRole::PressLeft))
                && lockup.width >= width(self.text).saturating_add(2);
            if bracketed {
                // §11.4's mono `PRESSED` rule: `[mark]`, painted into the
                // padding columns so the lockup keeps its width
                let mut t = lockup;
                let used = ui.glyph(t, GlyphRole::PressLeft, s.style);
                t = shift(t, used);
                let used = ui.paint_str(t, self.text, s.style);
                t = shift(t, used);
                ui.glyph(t, GlyphRole::PressRight, s.style);
            } else {
                ui.paint_str(shift(lockup, pad), self.text, s.style);
            }
        }
        if self.clickable && !ui.is_inert() {
            ui.register_part(self.id, crate::id::PartRef::of(Part::LABEL), lockup);
        }
        if let Some(t) = self.tagline.filter(|t| !t.is_empty()) {
            let gap = ui.design().space.gap.max(1);
            let rest = shift(area, lockup.width.saturating_add(gap));
            if !rest.is_empty() {
                if let Some(f) = ov.slot_for(Part::META) {
                    f(ui, rest);
                } else {
                    let s = ov.style(ui, self.id, Family::BRAND, self.variant, Part::META, live);
                    ui.paint_str(rest, t, s.style);
                }
            }
        }
        area
    }

    /// The natural size: one row, the lockup plus any tagline.
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        Size::exact(self.natural_width(ui), 1).fit(c)
    }
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Position;
    use ratatui_core::style::Color;

    use super::*;
    use crate::event::MouseKind;
    use crate::id::PartRef;
    use crate::runtime::stub::{SCREEN, mouse};
    use crate::runtime::{App, Runtime};
    use crate::theme::resolve::bind_role;
    use crate::theme::{Role, Surface, Theme};

    const MARK: Id = Id::root("brand.tests");
    const OTHER: Id = Id::root("brand.tests.stop");
    const TEXT: &str = "Junie";
    /// `pad + "Junie" + pad`: what `draw` returns and what it registers.
    const LOCKUP: Rect = Rect {
        x: 0,
        y: 0,
        width: 7,
        height: 1,
    };
    const STOP: Rect = Rect {
        x: 0,
        y: 3,
        width: 10,
        height: 1,
    };

    /// One lockup plus one ordinary focus stop, so "the lockup is not in the
    /// ring" is asserted against a ring that is non-empty for another reason
    /// rather than against an empty one.
    #[derive(Default)]
    struct BrandPage {
        clickable: bool,
        activations: usize,
        /// Frames on which `Brand::update` consumed a pointer intent.
        pointer_frames: usize,
    }

    impl BrandPage {
        fn brand(&self) -> Brand<'static> {
            Brand::new(MARK, TEXT).clickable(self.clickable)
        }
    }

    impl App for BrandPage {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let mut r = self.brand().update(cx);
            if r.is_consumed() {
                self.pointer_frames = self.pointer_frames.saturating_add(1);
            }
            if r.take_action() == Some(Activated) {
                self.activations = self.activations.saturating_add(1);
            }
            for _ in cx.intents(OTHER) {}
            r.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            self.brand().draw(ui, SCREEN);
            ui.register_control(OTHER, STOP, Focusability::Focusable);
        }
    }

    /// A page that has drawn twice: the first draw settles focus, the second
    /// paints it, exactly as the harness does.
    fn page(clickable: bool) -> (Runtime<BrandPage>, Buffer) {
        let app = BrandPage {
            clickable,
            ..BrandPage::default()
        };
        let mut rt = Runtime::new(app, Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_buffer(SCREEN, &mut buf);
        rt.draw_buffer(SCREEN, &mut buf);
        (rt, buf)
    }

    fn bg_at(buf: &Buffer, x: u16, y: u16) -> Option<Color> {
        buf.cell(Position::new(x, y)).map(|c| c.bg)
    }

    /// The centre of the lockup, in both the clickable and the plain case.
    const fn pointer() -> (u16, u16) {
        (LOCKUP.x + LOCKUP.width / 2, LOCKUP.y)
    }

    /// GAP-1, half one: what `.clickable(true)` puts in the registry, and
    /// that `.clickable(false)` puts nothing there. Successor to the legacy
    /// `widgets::brand::tests::clickable_lockup_registers_and_lifts_on_hover`.
    #[test]
    fn only_a_clickable_lockup_registers_a_click_only_control_and_a_label_part() {
        let (rt, _buf) = page(true);
        assert_eq!(
            rt.area_of(MARK),
            Some(LOCKUP),
            "a clickable lockup is a hit target over its own rect"
        );
        assert_eq!(
            rt.area_of_part(MARK, PartRef::of(Part::LABEL)),
            Some(LOCKUP),
            "and registers its LABEL part over the same rect"
        );
        assert!(
            !rt.ring().is_registered(MARK),
            "ClickOnly: an identity mark is never a tab stop"
        );
        assert!(
            rt.ring().is_registered(OTHER),
            "the ring is non-empty, so the assertion above is not vacuous"
        );

        let (rt, _buf) = page(false);
        assert_eq!(
            rt.area_of(MARK),
            None,
            "a plain lockup registers no control at all"
        );
        assert_eq!(
            rt.area_of_part(MARK, PartRef::of(Part::LABEL)),
            None,
            "and no LABEL part"
        );
        assert!(!rt.ring().is_registered(MARK));
    }

    /// GAP-1, half two: the action. A plain lockup never even sees a pointer
    /// intent, because it registered nothing to be addressed through.
    #[test]
    fn only_a_clickable_lockup_emits_activated_on_a_click() {
        let (x, y) = pointer();

        let (mut rt, mut buf) = page(true);
        let _ = rt.handle(mouse(MouseKind::Down, x, y));
        rt.draw_buffer(SCREEN, &mut buf);
        assert_eq!(
            rt.app().activations,
            0,
            "the press alone is not an activation"
        );
        assert_eq!(
            rt.app().pointer_frames,
            1,
            "but the press did reach `Brand::update`"
        );
        let _ = rt.handle(mouse(MouseKind::Up, x, y));
        rt.draw_buffer(SCREEN, &mut buf);
        assert_eq!(
            rt.app().activations,
            1,
            "the release completes the click and emits `Activated`"
        );

        let (mut rt, mut buf) = page(false);
        let _ = rt.handle(mouse(MouseKind::Down, x, y));
        let _ = rt.handle(mouse(MouseKind::Up, x, y));
        rt.draw_buffer(SCREEN, &mut buf);
        assert_eq!(
            rt.app().activations,
            0,
            "a plain lockup emits nothing on a click"
        );
        assert_eq!(
            rt.app().pointer_frames,
            0,
            "and consumes no pointer intent at all"
        );
    }

    /// The tagline the slot sweep needs, so `META` is a part that paints.
    const TAGLINE: &str = "build 7";

    /// Every part the slot sweep installs on, and whether `Brand` is
    /// expected to honour it: the two parts it declares, then four it does
    /// not, so the sweep can fail in either direction.
    const SWEPT: &[(Part, bool)] = &[
        (Part::LABEL, true),
        (Part::META, true),
        (Part::CONTAINER, false),
        (Part::ICON, false),
        (Part::MARKER, false),
        (Part::GUTTER, false),
    ];

    /// A lockup with a tagline, optionally with one part replaced.
    struct SlotPage(Option<Part>);

    impl App for SlotPage {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let replaced = |ui: &mut Ui<'_>, r: Rect| {
                let s = ui.surface_style();
                ui.paint_str(r, "########", s);
            };
            let mut b = Brand::new(MARK, TEXT).tagline(TAGLINE);
            if let Some(p) = self.0 {
                b = b.slot(p, &replaced);
            }
            b.draw(ui, SCREEN);
        }
    }

    /// The cells one draw paints, with `slot` installed or with none.
    fn painted(slot: Option<Part>) -> Buffer {
        let mut rt = Runtime::new(SlotPage(slot), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_buffer(SCREEN, &mut buf);
        buf
    }

    /// Invariant R (§45.3): the parts the `## Overrides` section names as
    /// slot-addressable are **exactly** the parts for which installing
    /// `.slot(p, …)` changes the painted cells. Asserted in both
    /// directions, so a dropped `slot_for` consult and a slot honoured
    /// without being named each fail.
    #[test]
    fn a_slot_changes_painted_cells_for_exactly_label_and_meta() {
        let plain = painted(None);
        for (p, honoured) in SWEPT {
            let with = painted(Some(*p));
            assert_eq!(
                with != plain,
                *honoured,
                "`.slot({p:?}, …)` is documented as {}, but the painted cells {} the plain lockup's",
                if *honoured { "honoured" } else { "inert" },
                if with == plain {
                    "match"
                } else {
                    "differ from"
                }
            );
        }
        assert_eq!(
            Brand::PARTS,
            &[Part::LABEL, Part::META],
            "the honoured set above is the whole of `PARTS`, so no declared part is excluded"
        );
    }

    /// GAP-1, half three: the painted hover affordance. The legacy test
    /// asserted `accent_hover` under the pointer; this asserts it from the
    /// buffer, and asserts the plain lockup stays on the accent plane.
    #[test]
    fn only_a_clickable_lockup_lifts_to_accent_hover_under_the_pointer() {
        let theme = Theme::junie();
        let accent = bind_role(&theme, Role::Accent, Surface::Canvas);
        let accent_hover = bind_role(&theme, Role::AccentHover, Surface::Canvas);
        assert!(
            accent.is_some() && accent != accent_hover,
            "the two accent planes must differ, or this test cannot fail"
        );
        let (x, y) = pointer();

        let (mut rt, mut buf) = page(true);
        assert_eq!(bg_at(&buf, x, y), accent, "unhovered: the accent plane");
        let _ = rt.handle(mouse(MouseKind::Move, x, y));
        rt.draw_buffer(SCREEN, &mut buf);
        assert_eq!(rt.hover(), Some(MARK), "the pointer is over the lockup");
        assert_eq!(
            bg_at(&buf, x, y),
            accent_hover,
            "a hovered clickable lockup lifts to the hover plane"
        );
        assert_eq!(
            bg_at(&buf, LOCKUP.x, y),
            accent_hover,
            "the whole lockup lifts, padding included"
        );

        let (mut rt, mut buf) = page(false);
        let _ = rt.handle(mouse(MouseKind::Move, x, y));
        rt.draw_buffer(SCREEN, &mut buf);
        assert_eq!(rt.hover(), None, "a plain lockup is not a hit target");
        assert_eq!(
            bg_at(&buf, x, y),
            accent,
            "and never leaves the accent plane"
        );
    }
}
