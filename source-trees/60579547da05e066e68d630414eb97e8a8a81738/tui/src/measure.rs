//! Measurement (`COMPONENT_ARCHITECTURE.md` §10).

use crate::ui::Ui;

/// A measured size: the minimum a component can use and what it prefers.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Size {
    /// `(width, height)` below which the component degrades.
    pub min: (u16, u16),
    /// `(width, height)` the component would like.
    pub preferred: (u16, u16),
}

/// The space offered to `measure`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Constraints {
    /// The largest `(width, height)` available.
    pub max: (u16, u16),
    /// The width is fixed at `max.0`.
    pub tight_w: bool,
    /// The height is fixed at `max.1`.
    pub tight_h: bool,
}

impl Constraints {
    /// Loose constraints up to `w × h`.
    pub const fn loose(w: u16, h: u16) -> Self {
        Constraints {
            max: (w, h),
            tight_w: false,
            tight_h: false,
        }
    }

    /// Both axes fixed.
    pub const fn tight(w: u16, h: u16) -> Self {
        Constraints {
            max: (w, h),
            tight_w: true,
            tight_h: true,
        }
    }
}

impl Size {
    /// A size whose minimum and preferred are equal.
    pub const fn exact(w: u16, h: u16) -> Self {
        Size {
            min: (w, h),
            preferred: (w, h),
        }
    }

    /// Clip both pairs to `c.max`; tight axes report exactly `c.max`.
    #[must_use]
    pub fn fit(self, c: Constraints) -> Size {
        let clip = |(w, h): (u16, u16)| {
            (
                if c.tight_w { c.max.0 } else { w.min(c.max.0) },
                if c.tight_h { c.max.1 } else { h.min(c.max.1) },
            )
        };
        Size {
            min: clip(self.min),
            preferred: clip(self.preferred),
        }
    }
}

/// Optional measurement for components that can size themselves.
pub trait Measure {
    /// Measure against the design tokens and the offered constraints.
    fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Rect;

    use super::*;
    use crate::id::Part;
    use crate::response::StateFlags;
    use crate::theme::PartMetrics;
    use crate::theme::{Family, GlyphRole, Role, Slot, StylePatch, Surface, Theme, Variant};
    use crate::ui::cx::{FrameRead, LastFrame};
    use crate::ui::{FrameState, UiCore};

    const SCREEN: Rect = Rect {
        x: 0,
        y: 0,
        width: 40,
        height: 6,
    };

    /// Run `f` with a real `Ui` over a scratch page.
    fn with_ui<R>(theme: &Theme, f: impl FnOnce(&mut Ui<'_>) -> R) -> (R, FrameState) {
        let mut frame = FrameState::default();
        frame.reset(1, SCREEN);
        let mut page = Buffer::empty(SCREEN);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let out = {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, theme, &last);
            f(&mut ui)
        };
        (out, frame)
    }

    const STATES: [StateFlags; 6] = [
        StateFlags::empty(),
        StateFlags::FOCUSED,
        StateFlags::HOVERED,
        StateFlags::PRESSED,
        StateFlags::DISABLED,
        StateFlags::SELECTED,
    ];

    const PARTS: [Part; 8] = [
        Part::CONTAINER,
        Part::GUTTER,
        Part::MARKER,
        Part::LABEL,
        Part::META,
        Part::TRACK,
        Part::THUMB,
        Part::BORDER,
    ];

    static OV: [crate::theme::OverlayRule; 1] = [(
        Family::LIST,
        Variant::DEFAULT,
        Part::LABEL,
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Warning),
    )];

    /// The one test that keeps a second resolution path from drifting: the
    /// `&self` `Ui::resolve` must equal the `&mut self` `Ui::style` field for
    /// field, over the whole built-in recipe set (Adjudication N2).
    #[test]
    fn ui_resolve_equals_ui_style_for_every_family_variant_part() {
        for theme in [Theme::junie(), Theme::paper()] {
            with_ui(&theme, |ui| {
                let ov = crate::theme::Overlay::new(&OV);
                for pushed in [false, true] {
                    let body = |ui: &mut Ui<'_>| {
                        for &f in Family::ALL {
                            for &v in Variant::ALL {
                                for &p in &PARTS {
                                    for &st in &STATES {
                                        let a = ui.resolve(f, v, p, st);
                                        let b = ui.style(f, v, p, st);
                                        assert_eq!(a, b, "{f:?}/{v:?}/{p:?}/{st:?}");
                                    }
                                }
                            }
                        }
                        // a custom family goes through the neutral recipe on
                        // both paths
                        let cf = Family::custom("drift");
                        assert_eq!(
                            ui.resolve(cf, Variant::DEFAULT, Part::LABEL, StateFlags::FOCUSED),
                            ui.style(cf, Variant::DEFAULT, Part::LABEL, StateFlags::FOCUSED)
                        );
                    };
                    if pushed {
                        ui.with_overlay(&ov, body);
                    } else {
                        body(ui);
                    }
                }
            });
        }
    }

    /// Invariant M1: `Ui::style` is the *painting* query and alone writes the
    /// roles and the `testing` record; measurement records nothing, or
    /// `declared_parts_are_the_parts_actually_styled` would pass on a part a
    /// component only measured.
    #[test]
    fn measure_records_no_roles_and_no_styled_parts() {
        let theme = Theme::junie();
        let ((), frame) = with_ui(&theme, |ui| {
            for i in 0..1000u32 {
                let st = STATES[(i as usize) % STATES.len()];
                let _ = ui.resolve(Family::LIST, Variant::DEFAULT, Part::LABEL, st);
                let _ = ui.glyph_str(GlyphRole::FocusBar);
            }
            for pos in SCREEN.positions() {
                assert_eq!(ui.roles_at(pos), crate::ui::CellRoles::default());
            }
        });
        #[cfg(feature = "testing")]
        assert!(frame.styled_parts.is_empty() && frame.styled_queries.is_empty());
        let _ = frame;
    }

    /// Measurement must not evict a painting entry from the 256-slot memo
    /// (§11.1 A3, §20.9-2), so it performs no cache read and no cache write.
    #[cfg(feature = "testing")]
    #[test]
    fn measure_does_not_touch_the_style_cache() {
        let theme = Theme::junie();
        with_ui(&theme, |ui| {
            let _ = ui.style(
                Family::LIST,
                Variant::DEFAULT,
                Part::LABEL,
                StateFlags::empty(),
            );
            let before = ui.style_cache_stats();
            for i in 0..1000u32 {
                let st = STATES[(i as usize) % STATES.len()];
                let _ = ui.resolve(Family::LIST, Variant::DEFAULT, Part::LABEL, st);
            }
            assert_eq!(ui.style_cache_stats(), before);
        });
    }

    /// The case that forced Adjudication N2: a natural width that depends on
    /// a themed glyph, computed from `&Ui` (the `Measure::measure` receiver).
    #[test]
    fn natural_width_follows_the_themed_glyph() {
        struct Gutterful<'a> {
            label: &'a str,
        }

        impl Measure for Gutterful<'_> {
            fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
                let flags = ui.state(crate::id::Id::root("m.button"));
                let g = match ui
                    .resolve(Family::BUTTON, Variant::DEFAULT, Part::GUTTER, flags)
                    .glyph
                {
                    Slot::Set(r) => crate::text::width(ui.glyph_str(r)),
                    Slot::Inherit | Slot::Clear => 0,
                };
                let pad = 1u16;
                let w = g
                    .saturating_add(pad)
                    .saturating_add(crate::text::width(self.label))
                    .saturating_add(pad);
                Size::exact(w, 1).fit(c)
            }
        }

        let b = Gutterful { label: "Save" };
        let c = Constraints::loose(40, 3);
        // Junie binds `FocusBar` to a one-cell glyph, and the gutter carries
        // it only while focused
        let junie = Theme::junie();
        let (rest, _) = with_ui(&junie, |ui| b.measure(ui, c));
        assert_eq!(rest.preferred.0, 6);

        // a theme that rebinds `FocusBar` to a two-cell glyph widens the
        // measured width by exactly one column, without the component
        // knowing which glyph the theme chose
        let mut wide = Theme::junie();
        wide.design.glyphs.set(GlyphRole::FocusBar, "██");
        let mut narrow = Theme::junie();
        narrow.design.glyphs.set(GlyphRole::FocusBar, "█");
        let focused = StateFlags::FOCUSED;
        let measure_focused = |t: &Theme| {
            let mut frame = FrameState::default();
            frame.reset(1, SCREEN);
            let mut page = Buffer::empty(SCREEN);
            let mut core = UiCore::default();
            let mut last = LastFrame::default();
            last.snapshot.focus = Some(crate::id::Id::root("m.button"));
            let ui = Ui::new(&mut frame, &mut page, &mut core, t, &last);
            assert_eq!(ui.state(crate::id::Id::root("m.button")), focused);
            b.measure(&ui, c).preferred.0
        };
        assert_eq!(measure_focused(&wide), measure_focused(&narrow) + 1);
    }

    /// `Theme::metrics` is `Theme::resolve` minus the colour binding: the two
    /// share one `accumulate`, so the glyph, size and alignment a component
    /// sizes against in `update` are the ones `draw` paints with.
    #[test]
    fn metrics_are_surface_independent() {
        for theme in [Theme::junie(), Theme::paper()] {
            for &f in Family::ALL {
                for &p in &PARTS {
                    for &st in &STATES {
                        let m = theme.metrics(f, Variant::DEFAULT, p, st);
                        for s in [
                            Surface::Canvas,
                            Surface::Surface,
                            Surface::Elevated,
                            Surface::Overlay,
                            Surface::Popover,
                            Surface::Field,
                        ] {
                            let r = theme.resolve(f, Variant::DEFAULT, p, st, s);
                            assert_eq!(m, PartMetrics::from(r), "{f:?}/{p:?}/{st:?}/{s:?}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn measure_reports_min_and_preferred() {
        let s = Size {
            min: (10, 1),
            preferred: (40, 3),
        };
        let f = s.fit(Constraints::loose(20, 2));
        assert_eq!(
            f,
            Size {
                min: (10, 1),
                preferred: (20, 2)
            }
        );
        let t = s.fit(Constraints::tight(30, 5));
        assert_eq!(t, Size::exact(30, 5));
        let c = Constraints {
            max: (12, 9),
            tight_w: true,
            tight_h: false,
        };
        assert_eq!(
            s.fit(c),
            Size {
                min: (12, 1),
                preferred: (12, 3)
            }
        );
    }
}
