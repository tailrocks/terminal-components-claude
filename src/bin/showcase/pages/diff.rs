//! Existing DiffView in unified, review and empty states. Copy requests are
//! retained in this demo; no system clipboard or repository access occurs.
//! Fixtures are synchronous, so loading and error states do not apply.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::button::{Button, row_layout};
use junie_tui::widgets::diff::{DiffFile, DiffHunk, DiffLine, DiffMode, DiffStatus, DiffView};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::viewport::ViewportEvent;

const ID: WidgetId = WidgetId::of("diff");

pub struct DiffPage {
    view: DiffView,
    review: Button,
    empty: Button,
    copied: Option<String>,
}

fn sample() -> DiffFile {
    DiffFile {
        path: "config/service.toml".into(),
        status: DiffStatus::Modified,
        hunks: (0..5)
            .map(|i| DiffHunk {
                old_start: i * 8 + 1,
                new_start: i * 9 + 1,
                lines: vec![
                    DiffLine::context(format!("[service.worker_{i}]")),
                    DiffLine::remove("attempts = 3"),
                    DiffLine::add("attempts = 5"),
                    DiffLine::add("backoff = \"exponential\""),
                    DiffLine::context("region = \"東京\""),
                    DiffLine::context("label = \"cafe\u{301} ☕\""),
                    DiffLine::context("endpoint = \"https://api.example.test/workers/health\""),
                    DiffLine::context("enabled = true"),
                ],
            })
            .collect(),
    }
}

impl DiffPage {
    pub fn new() -> Self {
        let mut view = DiffView::new(ID.sub("view"));
        view.set_file(Some(sample()));
        Self {
            view,
            review: Button::toggle(ID.sub("review"), "Review", false),
            empty: Button::toggle(ID.sub("empty"), "Empty", false),
            copied: None,
        }
    }

    fn update_controls(&mut self) {
        self.view.set_mode(if self.review.on == Some(true) {
            DiffMode::Review
        } else {
            DiffMode::Unified
        });
        self.view.set_file(if self.empty.on == Some(true) {
            None
        } else {
            Some(sample())
        });
        self.view.term.clear_selection();
        self.copied = None;
    }
}

impl Page for DiffPage {
    fn title(&self) -> &'static str {
        "Diff viewer"
    }

    fn blurb(&self) -> &'static str {
        "Unified or old / new review · narrow panes use unified · select and copy text"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let controls = Rect::new(area.x, area.y, area.width, area.height.min(1));
        let rects = row_layout(controls, &[self.review.width(), self.empty.width()], 2);
        self.review.render(rects[0], buf, ctx, ctx.theme.canvas);
        self.empty.render(rects[1], buf, ctx, ctx.theme.canvas);
        let body = Rect::new(
            area.x,
            area.y.saturating_add(2).min(area.bottom()),
            area.width,
            area.height.saturating_sub(2),
        );
        let panel = Panel::framed(Some("Diff")).focused(ctx.interaction.focused(self.view.id()));
        let inner = panel.render(body, buf, ctx.theme);
        self.view.render(inner, buf, ctx, ctx.theme.canvas);
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        match ev {
            PageEvent::Key(key) => {
                if cx.focus.is(self.review.id) || cx.focus.is(self.empty.id) {
                    let button = if cx.focus.is(self.review.id) {
                        &mut self.review
                    } else {
                        &mut self.empty
                    };
                    let (outcome, activated) = button.on_key(key);
                    if activated {
                        self.update_controls();
                    }
                    return outcome;
                }
                if cx.focus.is(self.view.id()) {
                    let (outcome, event) = self.view.on_key(key);
                    if let Some(ViewportEvent::Copy(text)) = event {
                        self.copied = Some(text);
                        cx.status("Selection copied in demo");
                    }
                    return outcome;
                }
                Outcome::Ignored
            }
            PageEvent::Press { id, pos } if *id == self.view.id() => {
                cx.focus.focus(self.view.id());
                self.view.on_click(*pos)
            }
            PageEvent::Click { id, pos } => {
                if *id == self.review.id || *id == self.empty.id {
                    let button = if *id == self.review.id {
                        &mut self.review
                    } else {
                        &mut self.empty
                    };
                    cx.focus.focus(button.id);
                    if button.on_click() {
                        self.update_controls();
                        return Outcome::Changed;
                    }
                }
                if *id == self.view.id() {
                    cx.focus.focus(self.view.id());
                    return Outcome::Consumed;
                }
                if *id == scrollbar::id_for(self.view.id()) {
                    return self.view.on_scrollbar(*pos);
                }
                Outcome::Ignored
            }
            PageEvent::Drag { pressed, pos } => {
                if *pressed == scrollbar::id_for(self.view.id()) {
                    return self.view.on_scrollbar_drag(*pos);
                }
                if *pressed == self.view.id() {
                    cx.focus.focus(self.view.id());
                    return self.view.on_drag(*pos);
                }
                Outcome::Ignored
            }
            PageEvent::Wheel { id, delta } if self.view.owns(*id) => self.view.on_wheel(*delta),
            _ => Outcome::Ignored,
        }
    }

    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        if focus == Some(self.view.id()) {
            vec![
                ("↑ ↓", "Scroll"),
                ("← →", "Pan"),
                ("drag", "Select"),
                ("y", "Copy"),
                ("Esc", "Clear"),
            ]
        } else {
            vec![("Enter", "Toggle"), ("Tab", "Next control")]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use junie_tui::core::focus::{Focus, FocusRing};
    use junie_tui::core::hit::HitRegistry;
    use junie_tui::theme::{ColorLevel, Theme};
    use junie_tui::ui::ctx::Interaction;
    use ratatui::crossterm::event::{KeyCode, KeyModifiers};
    use ratatui::layout::Position;

    fn render(page: &mut DiffPage, width: u16, level: ColorLevel) -> (Buffer, FocusRing) {
        let area = Rect::new(0, 0, width, 16);
        let mut buf = Buffer::empty(area);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let theme = Theme::for_level(level);
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        page.render(area, &mut buf, &mut ctx);
        (buf, ring)
    }

    fn text(buf: &Buffer) -> String {
        buf.content().iter().map(|cell| cell.symbol()).collect()
    }

    #[test]
    fn default_review_narrow_and_empty_render_with_focus_stops_in_monochrome() {
        for level in [ColorLevel::TrueColor, ColorLevel::Mono] {
            let mut page = DiffPage::new();
            let (buf, ring) = render(&mut page, 100, level);
            assert!(text(&buf).contains("- attempts = 3"));
            assert!(text(&buf).contains("+ attempts = 5"));
            assert_eq!(
                ring.reachable(),
                &[page.review.id, page.empty.id, page.view.id()]
            );
            page.review.on = Some(true);
            page.update_controls();
            let (buf, _) = render(&mut page, 100, level);
            assert!(text(&buf).contains("Old") && text(&buf).contains("New"));
            let (buf, _) = render(&mut page, 32, level);
            assert!(text(&buf).contains("+ attempts = 5"));
            page.empty.on = Some(true);
            page.update_controls();
            let (buf, _) = render(&mut page, 60, level);
            assert!(text(&buf).contains("No file selected"));
        }
    }

    #[test]
    fn keyboard_and_mouse_controls_scroll_and_copy_selection() {
        let mut page = DiffPage::new();
        let (_, ring) = render(&mut page, 100, ColorLevel::Mono);
        let mut focus = Focus::default();
        focus.focus(page.review.id);
        let mut cx = PageCtx {
            focus: &mut focus,
            ring: &ring,
            requests: vec![],
        };
        let enter = PageEvent::Key(junie_tui::core::event::Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        });
        page.handle(&enter, &mut cx);
        assert_eq!(page.view.mode, DiffMode::Review);
        page.handle(
            &PageEvent::Click {
                id: page.review.id,
                pos: Position::new(1, 0),
            },
            &mut cx,
        );
        assert_eq!(page.view.mode, DiffMode::Unified);
        render(&mut page, 100, ColorLevel::Mono);
        let pos = page.view.term.area.as_position();
        page.handle(
            &PageEvent::Press {
                id: page.view.id(),
                pos,
            },
            &mut cx,
        );
        page.handle(
            &PageEvent::Drag {
                pressed: page.view.id(),
                pos: Position::new(pos.x + 12, pos.y),
            },
            &mut cx,
        );
        page.handle(
            &PageEvent::Click {
                id: page.view.id(),
                pos: Position::new(pos.x + 12, pos.y),
            },
            &mut cx,
        );
        page.handle(
            &PageEvent::Key(junie_tui::core::event::Key {
                code: KeyCode::Char('y'),
                mods: KeyModifiers::NONE,
            }),
            &mut cx,
        );
        assert!(page.copied.as_ref().is_some_and(|text| !text.is_empty()));
        page.handle(
            &PageEvent::Wheel {
                id: page.view.id(),
                delta: 3,
            },
            &mut cx,
        );
        assert_eq!(page.view.term.scroll.offset, 3);
    }

    #[test]
    fn shell_drag_preserves_original_press_through_release_in_both_viewports() {
        use crate::app::{App, PageId};
        use junie_tui::core::event::{Input, Mouse, MouseKind};
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        for (page, id) in [
            (PageId::Diff, ID.sub("view")),
            (PageId::Terminal, WidgetId::of("terminal").sub("term")),
        ] {
            let mut app = App::new(Theme::junie());
            app.goto(page);
            let mut term = Terminal::new(TestBackend::new(140, 40)).unwrap();
            term.draw(|frame| app.render(frame)).unwrap();
            let area = app.hits.area_of(id).unwrap();
            let start = Position::new(area.x + 2, area.y);
            let end = Position::new(area.x + 10, area.y);
            for (kind, pos) in [
                (MouseKind::Down, start),
                (MouseKind::Drag, end),
                (MouseKind::Up, end),
            ] {
                app.handle(Input::Mouse(Mouse { kind, pos }));
                term.draw(|frame| app.render(frame)).unwrap();
            }
            assert_eq!(app.focus.current(), Some(id));
            assert_eq!(
                term.backend().buffer()[(start.x, start.y)].bg,
                app.theme.selection().bg.unwrap(),
                "selection must start at pointer down, even when the first motion is far away"
            );
        }
    }
}
