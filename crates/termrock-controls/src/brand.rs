#![doc = r" `Brand` — the accent-filled identity lockup (`COMPONENT_ARCHITECTURE.md`"]
#![doc = r" §11.6, §18.2, Appendix A 4G)."]

use core::fmt;

use ratatui_core::layout::Rect;

use super::{PartStyle, SlotFn, cell_at, first_row, paint_pressed_bracket, shift};
use crate::focus::Focusability;
use crate::id::{Id, Part};
use crate::intent::{Intent, Phase};
use crate::measure::{Constraints, Size};
use crate::response::{Activated, Response, StateFlags};
use crate::text::width;
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};

#[doc = r" The product mark: the one lockup that fills with the accent."]
#[doc = r""]
#[doc = r" ## Construction"]
#[doc = r" `Brand::new(id, text)`. The mark text is always the application's; the"]
#[doc = r" library bakes in no wordmark."]
#[doc = r""]
#[doc = r" ## Ownership"]
#[doc = r" Stateless. The caller owns the mark text and the optional tagline; the"]
#[doc = r" runtime owns hover and press when the lockup is `.clickable(true)`."]
#[doc = r""]
#[doc = r" ## Configuration"]
#[doc = r" `.variant(Variant)` (default `Recipe.default_variant`), `.compact(bool)`"]
#[doc = r" (`false` — drops the one-cell fill padding for tight strips),"]
#[doc = r" `.tagline(&str)` (none), `.clickable(bool)` (`false`), `.patch`,"]
#[doc = r" `.patch_part`, `.slot`."]
#[doc = r""]
#[doc = r" ## Variants"]
#[doc = r" `Family::BRAND`; `DEFAULT` only. The accent fill, the on-accent"]
#[doc = r" foreground and the bold weight are `Theme::junie()` recipe defaults, not"]
#[doc = r" component code (§11.6) — a theme restyles the lockup without touching"]
#[doc = r" this file."]
#[doc = r""]
#[doc = r" ## States"]
#[doc = r" Wears `HOVERED` and `PRESSED` from the runtime when `.clickable(true)`;"]
#[doc = r" derives nothing. A non-clickable lockup registers nothing and wears no"]
#[doc = r" state at all."]
#[doc = r""]
#[doc = r" ## Actions"]
#[doc = r" `Response<Activated>` — `Activated` on a click, and only when"]
#[doc = r" `.clickable(true)`."]
#[doc = r""]
#[doc = r" ## Focus"]
#[doc = r" `Focusability::ClickOnly` when clickable, so the lockup takes hover and"]
#[doc = r" press but never enters the tab ring: an identity mark is not a control an"]
#[doc = r" operator tabs to. `swallows_typing` is false; `.autofocus()` does not"]
#[doc = r" exist."]
#[doc = r""]
#[doc = r" ## Keyboard"]
#[doc = r" None. The lockup has no chord and publishes no `Bindings` table; a"]
#[doc = r" product that wants a keyboard path binds one through its own `KeyMap`."]
#[doc = r""]
#[doc = r" ## Mouse"]
#[doc = r" `PartRef::of(Part::LABEL)` when clickable: press and release paint the"]
#[doc = r" pressed plane, a click emits `Activated`."]
#[doc = r""]
#[doc = r" ## Layout"]
#[doc = r" `measure` returns `(pad + mark + pad [+ gap + tagline], 1)` where `pad` is"]
#[doc = r" `0` under `.compact(true)` and `1` otherwise. `draw` uses the first row of"]
#[doc = r" `area`, clipped to that width, and returns the rect it painted; a"]
#[doc = r" degenerate rect registers nothing and paints nothing (R5)."]
#[doc = r""]
#[doc = r" ## Parts"]
#[doc = r" `LABEL` (the filled lockup), `META` (the tagline beside it)."]
#[doc = r""]
#[doc = r" ## Overrides"]
#[doc = r" `.patch` and `.patch_part` reach `Part::LABEL` and `Part::META`."]
#[doc = r" `.slot(p, …)` changes painted cells for exactly `Part::LABEL` and"]
#[doc = r" `Part::META` — every part `Brand` declares, so nothing is excluded. In"]
#[doc = r" particular there is no `CONTAINER` to exclude: the accent fill is"]
#[doc = r" `LABEL`'s own painting rather than a surface behind it, so a `LABEL`"]
#[doc = r" slot replaces fill and mark together, and the lockup keeps its rect, its"]
#[doc = r" `ClickOnly` control and its `PartRef::of(Part::LABEL)` hit region across"]
#[doc = r" the substitution (§45.4). `META` is painted, and therefore"]
#[doc = r" slot-addressable, only when `.tagline(s)` carries a non-empty string and"]
#[doc = r" the row still has columns after the lockup and the gap; with no tagline"]
#[doc = r" the component paints no `META` cell for a slot to replace."]
#[doc = r""]
#[doc = r" ## Identity"]
#[doc = r" One `Id` per instance; no items."]
#[doc = r""]
#[doc = r" ## Testing"]
#[doc = r" `BrandCase` with no capabilities;"]
#[doc = r" `render::components::brand::{default, pressed, empty}`;"]
#[doc = r" `brand::a_slot_changes_painted_cells_for_exactly_label_and_meta`, which"]
#[doc = r" asserts Invariant R (§45.3) in both directions — a named part that stops"]
#[doc = r" changing cells and an unnamed part that starts changing them each fail"]
#[doc = r" it."]
#[doc = r""]
#[doc = r" `BrandCase` deliberately declares `Caps::empty()`: the conformance fixture"]
#[doc = r" is the non-interactive lockup, while the opt-in `ClickOnly` path is covered"]
#[doc = r" by the runtime-driven unit tests in this module:"]
#[doc = r" `only_a_clickable_lockup_registers_a_click_only_control_and_a_label_part`,"]
#[doc = r" `only_a_clickable_lockup_emits_activated_on_a_click` and"]
#[doc = r" `only_a_clickable_lockup_lifts_to_accent_hover_under_the_pointer`. Each"]
#[doc = r" asserts the plain lockup does **none** of it, so both the mouse-only path"]
#[doc = r" and the non-interactive default remain executable and pinned."]
#[doc = r""]
#[doc = r" ## Invariants"]
#[doc = r" The mono `PRESSED` bracket rule (§11.4) is honoured only when the lockup"]
#[doc = r" has room for both brackets — a `.compact(true)` lockup reserves no padding"]
#[doc = r" and a mono fallback must never change geometry, so it keeps the plain"]
#[doc = r" mark. Never allocates; never writes outside `area`."]
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
    #[doc = r" The parts this component styles."]
    pub const PARTS: &'static [Part] = &[Part::LABEL, Part::META];

    #[doc = r" A lockup showing `text`."]
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

    #[doc = r" The id."]
    pub const fn id(&self) -> Id {
        self.id
    }

    #[doc = r" Set the variant."]
    #[must_use]
    pub const fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    #[doc = r" Drop the fill padding for a tight strip; the treatment is unchanged."]
    #[must_use]
    pub const fn compact(mut self, yes: bool) -> Self {
        self.compact = yes;
        self
    }

    #[doc = r" A muted line beside the lockup."]
    #[must_use]
    pub const fn tagline(mut self, s: &'a str) -> Self {
        self.tagline = Some(s);
        self
    }

    #[doc = r" Take hover, press and clicks."]
    #[must_use]
    pub const fn clickable(mut self, yes: bool) -> Self {
        self.clickable = yes;
        self
    }

    #[doc = r" An instance patch over every part (precedence 6)."]
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[doc = r" Per-part patches."]
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[doc = r" Replace one part's painting; layout and hit regions stay."]
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    #[doc = r" The fill padding either side of the mark."]
    const fn pad(&self) -> u16 {
        if self.compact { 0 } else { 1 }
    }

    #[doc = r" Columns the filled lockup occupies."]
    fn lockup_width(&self) -> u16 {
        width(self.text)
            .saturating_add(self.pad())
            .saturating_add(self.pad())
    }

    #[doc = r" Columns the whole component occupies, tagline included."]
    fn natural_width(&self, ui: &Ui<'_>) -> u16 {
        match self.tagline {
            Some(t) if !t.is_empty() => self
                .lockup_width()
                .saturating_add(ui.design().space.gap.max(1))
                .saturating_add(width(t)),
            _ => self.lockup_width(),
        }
    }

    #[doc = r" The update phase; only a `.clickable(true)` lockup can act."]
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

    #[doc = r" The draw phase; returns the rect painted."]
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
                ui.paint_str(shift(lockup, 1), self.text, s.style);
                paint_pressed_bracket(
                    ui,
                    cell_at(lockup, lockup.x),
                    cell_at(lockup, lockup.right().saturating_sub(1)),
                    s.style,
                );
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

    #[doc = r" The natural size: one row, the lockup plus any tagline."]
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        Size::exact(self.natural_width(ui), 1).fit(c)
    }
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::{Position, Rect};
    use ratatui_core::style::Color;

    use super::*;
    use crate::event::MouseKind;
    use crate::id::PartRef;
    use crate::runtime::stub::{SCREEN, Stub, mouse};
    use crate::runtime::{App, Runtime};
    use crate::theme::resolve::bind_role;
    use crate::theme::{ColorLevel, Role, Surface, Theme};

    const MARK: Id = Id::root("brand.tests");
    const OTHER: Id = Id::root("brand.tests.stop");
    const TEXT: &str = "Junie";
    #[doc = r#" `pad + "Junie" + pad`: what `draw` returns and what it registers."#]
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

    #[doc = r#" One lockup plus one ordinary focus stop, so "the lockup is not in the"#]
    #[doc = r#" ring" is asserted against a ring that is non-empty for another reason"#]
    #[doc = r" rather than against an empty one."]
    #[derive(Default)]
    struct BrandPage {
        clickable: bool,
        activations: usize,
        #[doc = r" Frames on which `Brand::update` consumed a pointer intent."]
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

    #[doc = r" A page that has drawn twice: the first draw settles focus, the second"]
    #[doc = r" paints it, exactly as the harness does."]
    fn page(clickable: bool) -> (Runtime<BrandPage>, Buffer) {
        let app = BrandPage {
            clickable,
            ..BrandPage::default()
        };
        let mut rt = Runtime::new(app, Theme::junie());
        let _ = rt.initialize();
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        (rt, buf)
    }

    fn bg_at(buf: &Buffer, x: u16, y: u16) -> Option<Color> {
        buf.cell(Position::new(x, y)).map(|c| c.bg)
    }

    #[test]
    fn mono_pressed_brand_keeps_the_lockup_padding() {
        const ID: Id = Id::root("brand.tests.mono");
        let area = Rect::new(0, 0, 12, 1);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie().downgrade(ColorLevel::Mono));
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, area| {
                ui.reference(
                    Some(crate::ReferenceTarget::new(
                        ID,
                        crate::ReferenceState::PRESSED,
                    )),
                    |ui| Brand::new(ID, TEXT).clickable(true).draw(ui, area),
                );
            })
            .commit_presented();

        assert_eq!(
            buffer
                .cell(Position::new(0, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some("["),
            "the left bracket occupies the lockup's leading pad"
        );
        assert_eq!(
            buffer
                .cell(Position::new(1, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some("J"),
            "the mark stays one cell inside the lockup"
        );
        assert_eq!(
            buffer
                .cell(Position::new(6, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some("]"),
            "the right bracket occupies the lockup's trailing pad"
        );
    }

    #[doc = r" The centre of the lockup, in both the clickable and the plain case."]
    const fn pointer() -> (u16, u16) {
        (LOCKUP.x + LOCKUP.width / 2, LOCKUP.y)
    }

    #[doc = r" GAP-1, half one: what `.clickable(true)` puts in the registry, and"]
    #[doc = r" that `.clickable(false)` puts nothing there. Successor to the legacy"]
    #[doc = r" `widgets::brand::tests::clickable_lockup_registers_and_lifts_on_hover`."]
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

    #[doc = r" GAP-1, half two: the action. A plain lockup never even sees a pointer"]
    #[doc = r" intent, because it registered nothing to be addressed through."]
    #[test]
    fn only_a_clickable_lockup_emits_activated_on_a_click() {
        let (x, y) = pointer();

        let (mut rt, mut buf) = page(true);
        let _ = crate::runtime::stub::deliver(&mut rt, mouse(MouseKind::Down, x, y));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
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
        let _ = crate::runtime::stub::deliver(&mut rt, mouse(MouseKind::Up, x, y));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        assert_eq!(
            rt.app().activations,
            1,
            "the release completes the click and emits `Activated`"
        );

        let (mut rt, mut buf) = page(false);
        let _ = crate::runtime::stub::deliver(&mut rt, mouse(MouseKind::Down, x, y));
        let _ = crate::runtime::stub::deliver(&mut rt, mouse(MouseKind::Up, x, y));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
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

    #[doc = r" The tagline the slot sweep needs, so `META` is a part that paints."]
    const TAGLINE: &str = "build 7";

    #[doc = r" Every part the slot sweep installs on, and whether `Brand` is"]
    #[doc = r" expected to honour it: the two parts it declares, then four it does"]
    #[doc = r" not, so the sweep can fail in either direction."]
    const SWEPT: &[(Part, bool)] = &[
        (Part::LABEL, true),
        (Part::META, true),
        (Part::CONTAINER, false),
        (Part::ICON, false),
        (Part::MARKER, false),
        (Part::GUTTER, false),
    ];

    #[doc = r" A lockup with a tagline, optionally with one part replaced."]
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

    #[doc = r" The cells one draw paints, with `slot` installed or with none."]
    fn painted(slot: Option<Part>) -> Buffer {
        let mut rt = Runtime::new(SlotPage(slot), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        buf
    }

    #[doc = r" Invariant R (§45.3): the parts the `## Overrides` section names as"]
    #[doc = r" slot-addressable are **exactly** the parts for which installing"]
    #[doc = r" `.slot(p, …)` changes the painted cells. Asserted in both"]
    #[doc = r" directions, so a dropped `slot_for` consult and a slot honoured"]
    #[doc = r" without being named each fail."]
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

    #[doc = r" GAP-1, half three: the painted hover affordance. The legacy test"]
    #[doc = r" asserted `accent_hover` under the pointer; this asserts it from the"]
    #[doc = r" buffer, and asserts the plain lockup stays on the accent plane."]
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
        let _ = crate::runtime::stub::deliver(&mut rt, mouse(MouseKind::Move, x, y));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
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
        let _ = crate::runtime::stub::deliver(&mut rt, mouse(MouseKind::Move, x, y));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        assert_eq!(rt.hover(), None, "a plain lockup is not a hit target");
        assert_eq!(
            bg_at(&buf, x, y),
            accent,
            "and never leaves the accent plane"
        );
    }
}
