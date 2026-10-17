//! Hint bar: the one key-hint surface an application shell owns, pinned to
//! the bottom row. Every interaction context contributes a layer; the
//! topmost layer that exists wins (topmost modal or menu › temporary mode ›
//! active screen › global fallback), so a modal never renders its own hint
//! row and the footer never moves.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::theme::{BadgeKind, HintAlign, Theme, Tone};
use crate::widgets::keyhint::{self, Hint};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HintLayer {
    pub hints: Vec<Hint>,
    pub badge: Option<(&'static str, BadgeKind)>,
    pub status: Option<(String, Tone)>,
    /// The status as parts in their own tones ([`Self::status_parts`]);
    /// empty for a status in one tone.
    pub status_parts: Vec<(String, Tone)>,
    /// Alignment for this layer only. `None` (the default) follows the
    /// theme's [`HintBarStyle::align`](crate::theme::HintBarStyle::align),
    /// which is where an app sets it once for every screen.
    pub align: Option<HintAlign>,
}

impl HintLayer {
    pub fn new(hints: Vec<Hint>) -> Self {
        Self {
            hints,
            badge: None,
            status: None,
            status_parts: vec![],
            align: None,
        }
    }
    /// Override the theme's alignment for this layer.
    pub fn align(mut self, align: HintAlign) -> Self {
        self.align = Some(align);
        self
    }
    pub fn badge(mut self, text: &'static str, kind: BadgeKind) -> Self {
        self.badge = Some((text, kind));
        self
    }
    pub fn status(mut self, s: impl Into<String>, tone: Tone) -> Self {
        self.status = Some((s.into(), tone));
        self.status_parts.clear();
        self
    }
    /// A status made of parts in their own tones (`2 ok` in success,
    /// `1 failed` in error). `tone` is the message's severity: it picks the
    /// `!` or `▲` mark, as for [`Self::status`].
    pub fn status_parts(mut self, parts: Vec<(String, Tone)>, tone: Tone) -> Self {
        let text: String = parts.iter().map(|(p, _)| p.as_str()).collect();
        self.status = Some((text, tone));
        self.status_parts = parts;
        self
    }
}

/// The layer's bar with every hint in full, its badge and its whole
/// status (its `!` or `▲` mark included): [`keyhint::width`].
impl crate::ui::measure::Measure for HintLayer {
    fn measure(
        &self,
        _avail: ratatui::layout::Size,
        ctx: &crate::ui::ctx::RenderCtx,
    ) -> ratatui::layout::Size {
        let status = self.status.as_ref().map(|(text, tone)| match tone {
            Tone::Error => format!("! {text}"),
            Tone::Warning => format!("{} {text}", ctx.theme.glyphs.warning),
            _ => text.clone(),
        });
        let w = keyhint::width(&self.hints, self.badge.map(|(b, _)| b), status.as_deref());
        ratatui::layout::Size::new(w, 1)
    }
}

pub struct HintBar;

/// The keys of the hints a footer drew this frame
/// ([`HintBar::render_ctx`]): a click on a hint replays its key (M1).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HintKeys(Vec<Option<crate::core::event::Key>>);

impl HintKeys {
    /// Make the drawn hints click targets: `spots` are where each of
    /// `hints` landed ([`keyhint::render_parts_at`],
    /// [`keyhint::render_status_at`]). Every hint that names one key
    /// ([`keyhint::key_of`]) registers as a [`Part::KEY`](crate::core::id::Part::KEY)
    /// of [`HintBar::ID`]; hover underlines it and a press reverses it.
    pub fn register(
        ctx: &mut crate::ui::ctx::RenderCtx,
        buf: &mut Buffer,
        hints: &[keyhint::Hint],
        spots: Vec<(usize, Rect)>,
    ) -> Self {
        use crate::core::id::{ItemKey, Part, PartRef};
        use ratatui::style::{Modifier, Style};
        let mut keys = vec![None; hints.len()];
        for (i, cell) in spots {
            let Some(key) = hints.get(i).and_then(|h| keyhint::key_of(h.key)) else {
                continue;
            };
            keys[i] = Some(key);
            let part = PartRef::item(Part::KEY, ItemKey::index(i));
            let id = ctx.register_part(HintBar::ID, part, cell);
            if ctx.interaction.pressed(id) {
                buf.set_style(cell, Style::new().add_modifier(Modifier::REVERSED));
            } else if ctx.interaction.hovered(id) {
                buf.set_style(cell, Style::new().add_modifier(Modifier::UNDERLINED));
            }
        }
        HintKeys(keys)
    }

    /// The key a click on the footer replays: the hint under a completed
    /// click on [`HintBar::ID`].
    pub fn clicked(
        &self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> Option<crate::core::event::Key> {
        if !ev.on(HintBar::ID) || !ev.is(crate::core::intent::Phase::Click) {
            return None;
        }
        let i = ev.index_of(crate::core::id::Part::KEY)?;
        self.0.get(i).copied().flatten()
    }
}

impl HintBar {
    /// The footer's hit owner: each clickable hint is its
    /// `Part::KEY` item.
    pub const ID: crate::core::id::WidgetId = crate::core::id::WidgetId::of("hintbar");

    /// [`Self::render`] with every hint that names one key
    /// ([`keyhint::key_of`]) a click target: hover underlines it, a press
    /// reverses its key, and a click replays the key through the app
    /// ([`HintKeys::clicked`]). Hints naming several keys (`↑↓`) stay text.
    pub fn render_ctx(
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut crate::ui::ctx::RenderCtx,
        layer: &HintLayer,
    ) -> HintKeys {
        let spots = Self::render_at(area, buf, ctx.theme, layer);
        HintKeys::register(ctx, buf, &layer.hints, spots)
    }

    /// [`Self::render`] reporting where each hint landed, for a footer
    /// whose hits register later ([`HintKeys::register`]): an app that
    /// draws its footer under popups still painted as overlays registers
    /// the hints after them, so the footer stays on the topmost layer and
    /// the layers in between keep only their own regions.
    pub fn render_at(
        area: Rect,
        buf: &mut Buffer,
        t: &Theme,
        layer: &HintLayer,
    ) -> Vec<(usize, Rect)> {
        let style = crate::theme::HintBarStyle {
            align: layer.align.unwrap_or(t.hintbar.align),
            ..t.hintbar
        };
        let (_, spots) = keyhint::render_parts_at(
            area,
            buf,
            t,
            &layer.hints,
            layer.badge,
            layer.status.as_ref().map(|(s, tone)| (s.as_str(), *tone)),
            &layer.status_parts,
            &style,
        );
        spots
    }

    /// The first present layer wins; `layers` is ordered from the topmost
    /// context down to the global fallback.
    pub fn resolve(layers: &[Option<HintLayer>]) -> HintLayer {
        layers.iter().flatten().next().cloned().unwrap_or_default()
    }

    /// Draws the layer on `area` (one row). Returns how many hints fit;
    /// dropped hints leave a `…` marker so the operator knows there is more.
    pub fn render(area: Rect, buf: &mut Buffer, t: &Theme, layer: &HintLayer) -> usize {
        let style = crate::theme::HintBarStyle {
            align: layer.align.unwrap_or(t.hintbar.align),
            ..t.hintbar
        };
        keyhint::render_parts(
            area,
            buf,
            t,
            &layer.hints,
            layer.badge,
            layer.status.as_ref().map(|(s, tone)| (s.as_str(), *tone)),
            &layer.status_parts,
            &style,
        )
    }
}

/// A hint layer drawn in a forced state for a States grid
/// ([`crate::ui::specimen`]): the bar at rest, with its status cut
/// (overflow), with an error status, with the EDIT badge (editing) and in
/// the grid's narrow slot, where hints drop by priority.
impl crate::ui::specimen::Specimen for HintLayer {
    fn specimen_id(&self) -> crate::core::id::WidgetId {
        crate::core::id::WidgetId::of("hintbar.specimen")
    }

    fn supports(&self, state: crate::ui::specimen::SpecimenState) -> bool {
        use crate::ui::specimen::SpecimenState::*;
        matches!(state, Rest | Overflow | Error | Editing | Narrow)
    }

    fn specimen(
        &self,
        state: crate::ui::specimen::SpecimenState,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut crate::ui::ctx::RenderCtx,
        bg: ratatui::style::Color,
    ) {
        use crate::ui::specimen::SpecimenState::*;
        let mut layer = self.clone();
        match state {
            Overflow => {
                layer.status = Some((
                    "Saved 3 rows to payments.settlements · 2 warnings".into(),
                    Tone::Secondary,
                ));
            }
            Error => layer.status = Some(("Connection lost".into(), Tone::Error)),
            Editing => layer.badge = Some(("EDIT", BadgeKind::Edit)),
            _ => {}
        }
        crate::ui::ctx::fill(buf, area, ratatui::style::Style::new().bg(bg));
        HintBar::render(area, buf, ctx.theme, &layer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A layer drawn at its measured width shows every hint and its whole
    /// status.
    #[test]
    fn measure_shows_the_whole_layer() {
        use crate::core::focus::FocusRing;
        use crate::core::hit::HitRegistry;
        use crate::ui::measure::Measure;
        let t = Theme::junie();
        let (mut hits, mut ring) = (HitRegistry::default(), FocusRing::default());
        let ctx = crate::ui::ctx::RenderCtx::new(
            &t,
            crate::ui::ctx::Interaction::default(),
            &mut hits,
            &mut ring,
        );
        let layer = HintLayer::new(vec![
            keyhint::hint("Enter", "Open"),
            keyhint::hint("Esc", "Back"),
        ])
        .status("2 unsaved changes", Tone::Warning)
        .badge("EDIT", BadgeKind::Edit);
        let w = layer.measure(ratatui::layout::Size::new(1, 1), &ctx).width;
        let mut buf = Buffer::empty(Rect::new(0, 0, w, 1));
        HintBar::render(Rect::new(0, 0, w, 1), &mut buf, &t, &layer);
        let row: String = (0..w).map(|x| buf[(x, 0)].symbol()).collect();
        for part in ["EDIT", "Enter Open", "Esc Back", "2 unsaved changes"] {
            assert!(row.contains(part), "{row:?} lacks {part}");
        }
    }
    use crate::widgets::keyhint::hint;

    /// M1: every hint that names one key is a click target whose click
    /// replays the key; a hint naming several keys stays text; the cells
    /// at rest are those `render` draws.
    #[test]
    fn a_click_on_a_footer_hint_replays_its_key() {
        use crate::core::event::Key;
        use crate::core::focus::FocusRing;
        use crate::core::hit::HitRegistry;
        use crate::core::id::{ItemKey, Part, PartRef};
        use crate::core::intent::Phase;
        use crate::core::pointer::PointerEvent;
        use crate::ui::ctx::{Interaction, RenderCtx};
        use ratatui::crossterm::event::{KeyCode, KeyModifiers};
        use ratatui::layout::Position;
        let t = Theme::junie();
        let layer = HintLayer::new(vec![
            hint("↑↓", "Move"),
            hint("Ctrl+S", "Save"),
            hint("Esc", "Back"),
        ]);
        let area = Rect::new(0, 0, 60, 1);
        let mut plain = Buffer::empty(area);
        HintBar::render(area, &mut plain, &t, &layer);
        let mut buf = Buffer::empty(area);
        let (mut hits, mut ring) = (HitRegistry::default(), FocusRing::default());
        let keys = {
            let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
            HintBar::render_ctx(area, &mut buf, &mut ctx, &layer)
        };
        assert_eq!(buf, plain, "rest looks as before");
        let save = HintBar::ID.part(PartRef::item(Part::KEY, ItemKey::index(1)));
        let at = hits.area_of(save).expect("Ctrl+S is a target");
        assert!(
            hits.area_of(HintBar::ID.part(PartRef::item(Part::KEY, ItemKey::index(0))))
                .is_none()
        );
        let pos = Position::new(at.x, 0);
        let click = PointerEvent::new(&hits, Phase::Click, save, pos, 1, KeyModifiers::NONE);
        assert_eq!(
            keys.clicked(&click),
            Some(Key {
                code: KeyCode::Char('s'),
                mods: KeyModifiers::CONTROL
            })
        );
        let press = PointerEvent::new(&hits, Phase::Press, save, pos, 1, KeyModifiers::NONE);
        assert_eq!(keys.clicked(&press), None, "only a completed click");
    }

    #[test]
    fn topmost_layer_wins_and_fallback_is_empty() {
        let screen = HintLayer::new(vec![hint("Enter", "Launch")]);
        let modal = HintLayer::new(vec![hint("Esc", "Close")]);
        let r = HintBar::resolve(&[Some(modal.clone()), None, Some(screen.clone())]);
        assert_eq!(r, modal);
        let r = HintBar::resolve(&[None, None, Some(screen.clone())]);
        assert_eq!(r, screen);
        assert_eq!(HintBar::resolve(&[None, None]), HintLayer::default());
    }

    #[test]
    fn a_layer_follows_the_theme_alignment_unless_it_overrides_it() {
        let mut t = Theme::junie();
        t.hintbar.align = HintAlign::Center;
        let layer = HintLayer::new(vec![hint("Esc", "Back")]);
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 1));
        HintBar::render(Rect::new(0, 0, 40, 1), &mut buf, &t, &layer);
        // "Esc Back" is 8 cells: centred in 40 it starts at 16
        assert_eq!(buf[(16, 0)].symbol(), "E");
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 1));
        let start = layer.align(HintAlign::Start);
        HintBar::render(Rect::new(0, 0, 40, 1), &mut buf, &t, &start);
        assert_eq!(buf[(1, 0)].symbol(), "E");
    }

    #[test]
    fn narrow_rows_drop_from_the_right_and_mark_it() {
        let t = Theme::junie();
        let layer = HintLayer::new(vec![
            hint("Enter", "Open"),
            hint("Space", "Choose"),
            hint("g", "Git URL"),
            hint("Tab", "Next"),
            hint("Esc", "Cancel"),
        ]);
        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 1));
        let n = HintBar::render(Rect::new(0, 0, 30, 1), &mut buf, &t, &layer);
        assert!((2..5).contains(&n), "{n}");
        let row: String = (0..30u16)
            .map(|x| buf[(x, 0)].symbol().to_owned())
            .collect();
        assert!(row.contains("Enter Open"));
        assert!(row.contains('…'), "{row:?}");
        // normal hints go from the right; Esc is pinned and stays
        assert!(!row.contains("Next"), "{row:?}");
        assert!(row.contains("Esc Cancel"), "{row:?}");
        let mut wide = Buffer::empty(Rect::new(0, 0, 120, 1));
        assert_eq!(
            HintBar::render(Rect::new(0, 0, 120, 1), &mut wide, &t, &layer),
            5
        );
        let row: String = (0..120u16)
            .map(|x| wide[(x, 0)].symbol().to_owned())
            .collect();
        assert!(!row.contains('…'));
    }
}
