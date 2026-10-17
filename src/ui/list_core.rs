//! The model every row widget shares: cursor rules, scrolling, selection,
//! click policy, type-ahead, reveal and what shows instead of rows.
//!
//! [`ListBox`](crate::widgets::list::ListBox),
//! [`TreeView`](crate::widgets::tree::TreeView),
//! [`PropsList`](crate::widgets::props::PropsList) and
//! [`StepRail`](crate::widgets::steps::StepRail) embed one `ListCore` as
//! `core` and dereference to it, so `list.cursor`, `tree.scroll` or
//! `props.content` read and write the shared fields. The widget keeps only
//! what is its own: its rows, its key map, how a row is painted.
//!
//! One rule per concern, here:
//! - the [`Cursor`] a key moves ([`Self::cursor_model`], [`Self::key_step`]);
//! - the content shown instead of rows ([`ContentState`],
//!   [`Self::draw_content`]);
//! - the focus-stop policy ([`Self::is_focus_stop`]): a list with nothing
//!   to act on is skipped by Tab (K37), unless
//!   [`Self::skip_when_empty`] is off.

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

/// The shared state of a row widget (see the module docs).
#[derive(Debug, Clone)]
pub struct ListCore {
    /// The keyboard row.
    pub cursor: usize,
    /// ↑ on the first row lands on the last and ↓ on the last on the
    /// first. Default off: it stops at the end.
    pub wrap: bool,
    /// Arrow, page and Home/End movement steps over disabled rows (they
    /// cannot be used, so the cursor never parks on them). Default `true`.
    pub skip_disabled: bool,
    /// Which keys move the cursor (default [`NavKeys::default`]).
    pub nav: NavKeys,
    /// The scrolled rows and the bar.
    pub scroll: ScrollPane,
    /// The rows selected (multi selection; marks).
    pub selection: RowSelection,
    /// What a click does beyond moving the cursor. Default
    /// [`ClickPolicy::DATA`].
    pub click: ClickPolicy,
    /// Revealing cut labels; `reveal.config` overrides the global default.
    pub reveal: RevealState,
    /// Typing a prefix jumps to the matching row. Default from the
    /// [`NavProfile`]: on under `Desktop`, off under `Vim`.
    pub type_ahead: bool,
    /// Whether single-character shortcuts win over type-ahead. Default
    /// `None`: the app-wide [`TypeAheadPrecedence::current`].
    pub type_ahead_precedence: Option<TypeAheadPrecedence>,
    /// The typed prefix and its timeout.
    pub typeahead: TypeAhead,
    /// Loading, a load error, or the rows. Default [`ContentState::Ready`].
    pub content: ContentState,
    /// What a ready widget with no rows shows. `None`: the widget's own
    /// default (the theme's empty-list text for a list, nothing for a
    /// tree, facts or steps).
    pub empty: Option<EmptyState>,
    /// A widget with no rows to act on (empty, loading or failed) is no
    /// focus stop: Tab passes it (K37; APG "a composite with no items is
    /// not in the tab sequence"). Its error actions stay focusable.
    /// Default `true`.
    pub skip_when_empty: bool,
}

impl ListCore {
    /// A core over `len` rows with the defaults above.
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
        }
    }

    /// The cursor with this core's rules.
    pub fn cursor_model(&self) -> Cursor {
        Cursor::new(self.cursor)
            .wrap(self.wrap)
            .skip_disabled(self.skip_disabled)
    }

    /// A key as cursor movement over `src`: a navigation key or a
    /// type-ahead letter (per [`Self::nav`], [`Self::type_ahead`] and its
    /// precedence). `None` when the key is neither; the widget then
    /// handles it. The cursor is not written: the widget applies the
    /// [`Moved`] (it may extend a selection on the way).
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

    /// Whether `len` rows show (ready, and at least one).
    pub fn has_rows(&self, len: usize) -> bool {
        self.content.is_ready() && len > 0
    }

    /// Whether the widget registers as a focus stop with `len` rows
    /// ([`Self::skip_when_empty`]).
    pub fn is_focus_stop(&self, len: usize) -> bool {
        !self.skip_when_empty || self.has_rows(len)
    }

    /// Draw what shows instead of `len` rows in `area`, if anything:
    /// loading, the error, or (ready with no rows) [`Self::empty`], else
    /// `default_empty` as one muted line. Returns `true` when the rows are
    /// not drawn.
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
        assert!(!drew && s.trim().is_empty(), "ready with rows: the rows draw");
        let (drew, s, _) = draw(&core, 0);
        assert!(drew && s.contains("Nothing"), "ready, no rows: {s}");
        core.empty = Some(EmptyState::new("No tasks").hint("Add one"));
        assert!(draw(&core, 0).1.contains("Add one"));
        core.content = ContentState::loading("Fetching rows");
        let (drew, s, _) = draw(&core, 3);
        assert!(drew && s.contains("Fetching rows"), "loading hides rows: {s}");
        let retry = WidgetId::of("retry");
        core.content = ContentState::error(
            EmptyState::new("Could not load")
                .actions(vec![EmptyAction::primary(retry, "Retry")]),
        );
        let (drew, s, hits) = draw(&core, 3);
        assert!(drew && s.contains("! Could not load") && s.contains("Retry"), "{s}");
        assert!(
            hits.regions().iter().any(|r| r.id == retry),
            "the Retry button takes the pointer"
        );
        assert_eq!(core.content.actions().len(), 1);
    }

    /// K37: an empty, loading or failed list is no focus stop by default.
    #[test]
    fn a_widget_with_nothing_to_act_on_is_no_focus_stop() {
        let mut core = ListCore::new(0);
        assert!(!core.is_focus_stop(0));
        assert!(core.is_focus_stop(2));
        core.content = ContentState::loading("");
        assert!(!core.is_focus_stop(2));
        core.skip_when_empty = false;
        assert!(core.is_focus_stop(0), "knob: stays a stop");
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
        assert_eq!(moved(&mut core, KeyCode::Down).and_then(Moved::index), Some(2));
        core.wrap = true;
        assert_eq!(moved(&mut core, KeyCode::Down), Some(Moved::To(0)));
        assert_eq!(moved(&mut core, KeyCode::Enter), None);
    }
}
