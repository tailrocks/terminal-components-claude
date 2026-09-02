//! Hint bar: the one key-hint surface an application shell owns, pinned to
//! the bottom row. Every interaction context contributes a layer; the
//! topmost layer that exists wins (topmost modal or menu › temporary mode ›
//! active screen › global fallback), so a modal never renders its own hint
//! row and the footer never moves.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::theme::{BadgeKind, Theme, Tone};
use crate::widgets::keyhint::{self, Hint};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HintLayer {
    pub hints: Vec<Hint>,
    pub badge: Option<(&'static str, BadgeKind)>,
    pub status: Option<(String, Tone)>,
}

impl HintLayer {
    pub fn new(hints: Vec<Hint>) -> Self {
        Self {
            hints,
            badge: None,
            status: None,
        }
    }
    pub fn badge(mut self, text: &'static str, kind: BadgeKind) -> Self {
        self.badge = Some((text, kind));
        self
    }
    pub fn status(mut self, s: impl Into<String>, tone: Tone) -> Self {
        self.status = Some((s.into(), tone));
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
        keyhint::render_toned(
            area,
            buf,
            t,
            &layer.hints,
            layer.badge,
            layer.status.as_ref().map(|(s, tone)| (s.as_str(), *tone)),
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
        assert!(!row.contains("Cancel"));
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
