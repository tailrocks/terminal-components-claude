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

pub struct HintBar;

impl HintBar {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::keyhint::hint;

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
