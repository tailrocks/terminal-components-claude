//! `Brand` — the accent-filled identity lockup (`COMPONENT_ARCHITECTURE.md`
//! §11.6, §18.2, Appendix A 4G).

use core::fmt;

use ratatui_core::layout::Rect;

use super::{Overrides, SlotFn, first_row, shift};
use crate::focus::Focusability;
use crate::id::{Id, Part};
use crate::intent::{Intent, Phase};
use crate::measure::{Constraints, Size};
use crate::response::{Activated, Response, StateFlags};
use crate::text::width;
use crate::theme::{Family, GlyphRole, StylePatch, Variant};
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
/// `.patch_part`, `.slot`, `.state_override`.
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
/// `.patch`, `.patch_part` and `.slot` on both parts.
///
/// ## Identity
/// One `Id` per instance; no items.
///
/// ## Testing
/// `BrandCase` with no capabilities;
/// `render::components::brand::{default, pressed, empty}`.
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
    ov: Overrides<'a>,
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
            ov: Overrides::new(),
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
        self.ov = self.ov.patch(p);
        self
    }

    /// Per-part patches.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.patch_part(ps);
        self
    }

    /// Replace one part's painting; layout and hit regions stay.
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    /// Showcase / fixture use only (A11): render a forced state. Such a
    /// lockup registers nothing.
    #[must_use]
    pub const fn state_override(mut self, s: StateFlags) -> Self {
        self.ov = self.ov.state_override(s);
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
        let forced = self.ov.is_forced();
        let lockup = Rect {
            width: self.lockup_width().min(area.width),
            ..area
        };
        if self.clickable && !forced {
            ui.register_control(self.id, lockup, Focusability::ClickOnly);
        }
        let live = self.ov.flags(if self.clickable {
            ui.state(self.id)
        } else {
            StateFlags::empty()
        });
        let ov = self.ov;
        if let Some(f) = ov.slot_for(Part::LABEL) {
            f(ui, lockup);
        } else {
            let s = ov.style(ui, self.id, Family::BRAND, self.variant, Part::LABEL, live);
            ui.fill(lockup, s.style);
            let pad = self.pad();
            let bracketed = s.glyph == Some(GlyphRole::PressLeft)
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
        if self.clickable && !forced {
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
