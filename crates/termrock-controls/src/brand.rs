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
    pub const PARTS: &'static [Part] = &[Part::LABEL, Part::META];

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

    pub const fn id(&self) -> Id {
        self.id
    }

    #[must_use]
    pub const fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    #[must_use]
    pub const fn compact(mut self, yes: bool) -> Self {
        self.compact = yes;
        self
    }

    #[must_use]
    pub const fn tagline(mut self, s: &'a str) -> Self {
        self.tagline = Some(s);
        self
    }

    #[must_use]
    pub const fn clickable(mut self, yes: bool) -> Self {
        self.clickable = yes;
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

    const fn pad(&self) -> u16 {
        if self.compact { 0 } else { 1 }
    }

    fn lockup_width(&self) -> u16 {
        width(self.text)
            .saturating_add(self.pad())
            .saturating_add(self.pad())
    }

    fn natural_width(&self, ui: &Ui<'_>) -> u16 {
        match self.tagline {
            Some(t) if !t.is_empty() => self
                .lockup_width()
                .saturating_add(ui.design().space.gap.max(1))
                .saturating_add(width(t)),
            _ => self.lockup_width(),
        }
    }

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

    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        Size::exact(self.natural_width(ui), 1).fit(c)
    }
}
