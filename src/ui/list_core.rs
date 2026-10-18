use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::core::event::Key;
use crate::core::keymap::NavKeys;
use crate::core::list::{Cursor, KeyStep, Moved, NavProfile, RowSource, TypeAheadPrecedence};
use crate::core::scroll::ScrollPane;
use crate::core::selection::{ClickPolicy, RowSelection};
use crate::core::typeahead::TypeAhead;
use crate::ui::content::{self, ContentState};
use crate::ui::ctx::RenderCtx;
use crate::ui::reveal::RevealState;
use crate::widgets::empty::EmptyState;

#[derive(Debug, Clone)]
pub struct ListCore {
    pub cursor: usize,

    pub wrap: bool,

    pub skip_disabled: bool,

    pub nav: NavKeys,

    pub scroll: ScrollPane,

    pub selection: RowSelection,

    pub click: ClickPolicy,

    pub reveal: RevealState,

    pub type_ahead: bool,

    pub type_ahead_precedence: Option<TypeAheadPrecedence>,

    pub typeahead: TypeAhead,

    pub content: ContentState,

    pub empty: Option<EmptyState>,

    pub skip_when_empty: bool,

    pub inactive_selection: Option<crate::theme::InactiveSelection>,
}

impl ListCore {
    pub fn new(len: usize) -> Self {
        Self {
            cursor: 0,
            wrap: false,
            skip_disabled: true,
            nav: NavKeys::default(),
            scroll: ScrollPane::new(len),
            selection: RowSelection::new(),
            click: ClickPolicy::DATA,
            reveal: RevealState::default(),
            type_ahead: NavProfile::current().type_ahead(),
            type_ahead_precedence: None,
            typeahead: TypeAhead::default(),
            content: ContentState::Ready,
            empty: None,
            skip_when_empty: true,
            inactive_selection: None,
        }
    }

    pub fn cursor_model(&self) -> Cursor {
        Cursor::new(self.cursor)
            .wrap(self.wrap)
            .skip_disabled(self.skip_disabled)
    }

    pub fn key_step<S: RowSource + ?Sized>(
        &mut self,
        key: &Key,
        src: &S,
    ) -> Option<(KeyStep, Moved)> {
        let precedence = self
            .type_ahead_precedence
            .unwrap_or_else(TypeAheadPrecedence::current);
        let step = KeyStep::ordered(key, &self.nav, self.type_ahead, precedence)?;
        let moved = match step {
            KeyStep::Type(c) => self.cursor_model().type_ahead(c, &mut self.typeahead, src),
            KeyStep::Nav(nav) => self
                .cursor_model()
                .apply(nav, src, self.scroll.viewport_len),
        };
        Some((step, moved))
    }

    pub fn has_rows(&self, len: usize) -> bool {
        self.content.is_ready() && len > 0
    }

    pub fn is_focus_stop(&self, len: usize, focused: bool) -> bool {
        focused || !self.skip_when_empty || self.has_rows(len)
    }

    pub fn draw_content(
        &self,
        len: usize,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
        default_empty: Option<&str>,
    ) -> bool {
        match &self.content {
            ContentState::Loading(label) => {
                content::draw_loading(area, buf, ctx, label, bg);
                true
            }
            ContentState::Error(e) => {
                crate::widgets::empty::render_ctx(area, buf, ctx, e, bg);
                true
            }
            ContentState::Ready if len == 0 => {
                if let Some(e) = &self.empty {
                    content::draw_empty(area, buf, ctx, e, bg);
                } else if let Some(text) = default_empty {
                    content::draw_line(area, buf, ctx.theme, text, bg);
                }
                true
            }
            ContentState::Ready => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::core::id::WidgetId;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;
    use crate::widgets::empty::EmptyAction;
    use ratatui::crossterm::event::{KeyCode, KeyModifiers};

    fn text(buf: &Buffer) -> String {
        let a = buf.area;
        (a.y..a.bottom())
            .map(|y| {
                (a.x..a.right())
                    .map(|x| buf[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn draw(core: &ListCore, len: usize) -> (bool, String, HitRegistry) {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let area = Rect::new(0, 0, 30, 6);
        let mut buf = Buffer::empty(area);
        let drew = {
            let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
            core.draw_content(len, area, &mut buf, &mut ctx, theme.canvas, Some("Nothing"))
        };
        (drew, text(&buf), hits)
    }

    #[test]
    fn content_replaces_the_rows_and_names_its_state() {
        let mut core = ListCore::new(0);
        let (drew, s, _) = draw(&core, 3);
        assert!(
            !drew && s.trim().is_empty(),
            "ready with rows: the rows draw"
        );
        let (drew, s, _) = draw(&core, 0);
        assert!(drew && s.contains("Nothing"), "ready, no rows: {s}");
        core.empty = Some(EmptyState::new("No tasks").hint("Add one"));
        assert!(draw(&core, 0).1.contains("Add one"));
        core.content = ContentState::loading("Fetching rows");
        let (drew, s, _) = draw(&core, 3);
        assert!(
            drew && s.contains("Fetching rows"),
            "loading hides rows: {s}"
        );
        let retry = WidgetId::of("retry");
        core.content = ContentState::error(
            EmptyState::new("Could not load").actions(vec![EmptyAction::primary(retry, "Retry")]),
        );
        let (drew, s, hits) = draw(&core, 3);
        assert!(
            drew && s.contains("! Could not load") && s.contains("Retry"),
            "{s}"
        );
        assert!(
            hits.regions().iter().any(|r| r.id == retry),
            "the Retry button takes the pointer"
        );
        assert_eq!(core.content.actions().len(), 1);
    }

    #[test]
    fn a_widget_with_nothing_to_act_on_is_no_focus_stop() {
        let mut core = ListCore::new(0);
        assert!(!core.is_focus_stop(0, false));
        assert!(core.is_focus_stop(2, false));
        assert!(core.is_focus_stop(0, true), "focus stays while it empties");
        core.content = ContentState::loading("");
        assert!(!core.is_focus_stop(2, false));
        core.skip_when_empty = false;
        assert!(core.is_focus_stop(0, false), "knob: stays a stop");
    }

    #[test]
    fn key_step_moves_through_the_shared_cursor_rules() {
        let rows: &[&str] = &["Alpha", "Beta", "Gamma"];
        let mut core = ListCore::new(3);
        core.type_ahead = true;
        let key = |code| Key {
            code,
            mods: KeyModifiers::NONE,
        };
        let moved = |core: &mut ListCore, code| core.key_step(&key(code), rows).map(|(_, m)| m);
        assert_eq!(moved(&mut core, KeyCode::Down), Some(Moved::To(1)));
        assert_eq!(core.cursor, 0, "the widget applies the move");
        assert_eq!(moved(&mut core, KeyCode::Char('g')), Some(Moved::To(2)));
        core.cursor = 2;
        assert_eq!(
            moved(&mut core, KeyCode::Down).and_then(Moved::index),
            Some(2)
        );
        core.wrap = true;
        assert_eq!(moved(&mut core, KeyCode::Down), Some(Moved::To(0)));
        assert_eq!(moved(&mut core, KeyCode::Enter), None);
    }
}
