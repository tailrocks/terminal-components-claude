//! Diff (plan §7 SC25): a changes list of files in every `DiffStatus`
//! beside a `DiffView` that follows the list's cursor (the two stack when
//! narrow), unified or old / new review, the no-file state, and the
//! view's States grid. Copy requests are retained in this demo; no system
//! clipboard or repository access occurs. Options: `crate::options::DIFF`.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::app::PageId;
use crate::options::{PageValues, page_options};
use crate::pages::template::{Section, Template};
use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::core::scroll::Scrollable;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::specimen::{SpecimenState, StateGrid};
use junie_tui::widgets::button::{Button, row_layout};
use junie_tui::widgets::diff::{DiffFile, DiffHunk, DiffLine, DiffMode, DiffStatus, DiffView};
use junie_tui::widgets::list::{ListBox, ListItem, SelectMode};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::viewport::ViewportAction;

const ID: WidgetId = WidgetId::of("diff");
/// Width of the changes list beside the diff.
const LIST_W: u16 = 38;

pub struct DiffPage {
    template: Template,
    opts: PageValues,
    view: DiffView,
    files: ListBox,
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

/// The change set: one file per status.
fn changes() -> Vec<DiffFile> {
    vec![
        sample(),
        DiffFile {
            path: "docs/retries.md".into(),
            status: DiffStatus::Added,
            hunks: vec![DiffHunk {
                old_start: 0,
                new_start: 1,
                lines: vec![
                    DiffLine::add("# Retries"),
                    DiffLine::add(""),
                    DiffLine::add("Workers retry five times with exponential backoff."),
                ],
            }],
        },
        DiffFile {
            path: "src/legacy_backoff.rs".into(),
            status: DiffStatus::Deleted,
            hunks: vec![DiffHunk {
                old_start: 1,
                new_start: 0,
                lines: vec![
                    DiffLine::remove("pub fn linear(attempt: u32) -> u64 {"),
                    DiffLine::remove("    200 * attempt as u64"),
                    DiffLine::remove("}"),
                ],
            }],
        },
        DiffFile {
            path: "src/backoff/retry.rs".into(),
            status: DiffStatus::Renamed {
                from: "src/retry.rs".into(),
            },
            hunks: vec![DiffHunk {
                old_start: 1,
                new_start: 1,
                lines: vec![
                    DiffLine::remove("use crate::legacy_backoff::linear;"),
                    DiffLine::add("use super::exponential;"),
                    DiffLine::context("pub async fn fetch(url: &str) -> Result<Body, Error> {"),
                ],
            }],
        },
    ]
}

impl DiffPage {
    pub fn new() -> Self {
        let items = changes()
            .iter()
            .map(|f| ListItem::new(&f.header()).meta(&f.summary()))
            .collect();
        let mut page = Self {
            template: Template::new(ID, states_grid()),
            opts: PageValues::new(page_options(PageId::Diff)),
            view: DiffView::new(ID.sub("view")),
            files: ListBox::new(ID.sub("files"), items, SelectMode::Single),
            review: Button::toggle(ID.sub("review"), "Review", false),
            empty: Button::toggle(ID.sub("empty"), "Empty", false),
            copied: None,
        };
        page.update_controls();
        page
    }

    /// The view from the controls and the options: the file under the
    /// list's cursor (or none), in the mode the Review toggle picks.
    fn update_controls(&mut self) {
        let review = self.review.on == Some(true);
        if review != (self.view.mode == DiffMode::Review) {
            self.view.set_mode(self.view.mode.toggled());
        }
        let none = self.empty.on == Some(true) || self.opts.value("diff.file") == "none";
        self.view.set_file(if none {
            None
        } else {
            changes().into_iter().nth(self.files.cursor)
        });
        self.view.term.clear_selection();
        self.copied = None;
    }

    fn render_playground(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
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
        // a narrow page stacks the list over the diff
        let body = match self.opts.value("diff.width") {
            "narrow" => Rect::new(body.x, body.y, body.width.min(48), body.height),
            _ => body,
        };
        let (list, diff) = crate::pages::layout::columns(body, LIST_W, 2);
        let t = ctx.theme;
        let panel = Panel::framed(Some("Changes"))
            .meta("4 files")
            .focused(ctx.interaction.focused(self.files.id));
        let bg = panel.bg(t);
        let inner = panel.render(list, buf, t);
        self.files.render(inner, buf, ctx, bg);
        let panel = Panel::framed(Some("Diff")).focused(ctx.interaction.focused(self.view.id()));
        let inner = panel.render(diff, buf, ctx.theme);
        self.view.render(inner, buf, ctx, ctx.theme.canvas);
    }
}

/// The States grid: the view unified and in review, at rest, focused, with
/// no file and in a narrow slot (where review falls back to unified).
fn states_grid() -> StateGrid {
    use SpecimenState::*;
    let id = ID.sub("states");
    let file = DiffFile {
        path: "retry.toml".into(),
        status: DiffStatus::Modified,
        hunks: vec![DiffHunk {
            old_start: 1,
            new_start: 1,
            lines: vec![
                DiffLine::remove("attempts = 3"),
                DiffLine::add("attempts = 5"),
                DiffLine::context("enabled = true"),
            ],
        }],
    };
    let mut unified = DiffView::new(id.child(0));
    unified.set_file(Some(file.clone()));
    let mut review = DiffView::new(id.child(1));
    review.set_file(Some(file));
    review.set_mode(DiffMode::Review);
    StateGrid::new(id)
        .label_width(10)
        .column_width(42)
        .rows(&[Rest, Focused, Empty, Narrow])
        .column("Unified", unified)
        .column("Review", review)
}

impl Page for DiffPage {
    fn title(&self) -> &'static str {
        "Diff viewer"
    }

    fn blurb(&self) -> &'static str {
        "Unified or old / new review · narrow panes use unified · select and copy text"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let body = self.template.render(area, buf, ctx);
        match self.template.section() {
            Section::Playground => self.render_playground(body, buf, ctx),
            Section::Options => self.template.render_options(body, buf, ctx),
            Section::States => self.template.render_states(body, buf, ctx),
        }
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        if let Some(o) = self.template.handle(ev, cx) {
            return o;
        }
        if self.template.section() != Section::Playground {
            return Outcome::Ignored;
        }
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
                if cx.focus.is(self.files.id) {
                    let before = self.files.cursor;
                    let o = self.files.on_key(key);
                    if self.files.cursor != before {
                        self.update_controls();
                    }
                    return o;
                }
                if cx.focus.is(self.view.id()) {
                    let (outcome, event) = self.view.on_key(key);
                    if let Some(ViewportAction::Copy(text)) = event {
                        self.copied = Some(text);
                        cx.status("Selection copied");
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
                if let Some(row) = self.files.locate(*id) {
                    cx.focus.focus(self.files.id);
                    let before = self.files.cursor;
                    let o = self.files.on_click(row);
                    if self.files.cursor != before {
                        self.update_controls();
                    }
                    return o;
                }
                if *id == self.view.id() {
                    cx.focus.focus(self.view.id());
                    return Outcome::Consumed;
                }
                if scrollbar::is_bar_of(*id, self.view.id()) {
                    return self.view.on_scrollbar(*pos);
                }
                Outcome::Ignored
            }
            PageEvent::Drag { pressed, pos } => {
                if scrollbar::is_bar_of(*pressed, self.view.id()) {
                    return self.view.on_bar_drag(*pressed, *pos);
                }
                if *pressed == self.view.id() {
                    cx.focus.focus(self.view.id());
                    return self.view.on_drag(*pos);
                }
                Outcome::Ignored
            }
            PageEvent::Wheel { id, delta } if self.view.owns(*id) => self.view.on_wheel(*delta),
            PageEvent::WheelH { id, delta } if self.view.owns(*id) => self.view.on_wheel_h(*delta),
            _ => Outcome::Ignored,
        }
    }

    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        if let Some(h) = self.template.hints(focus) {
            return h;
        }
        if focus == Some(self.view.id()) {
            vec![
                ("↑↓", "Scroll"),
                ("←→", "Pan"),
                ("drag", "Select"),
                ("y", "Copy"),
                ("Esc", "Clear"),
            ]
        } else if focus == Some(self.files.id) {
            vec![("↑↓", "File"), ("Tab", "Next control")]
        } else {
            vec![("Enter", "Toggle"), ("Tab", "Next control")]
        }
    }

    fn option_value(&self, key: &str) -> Option<usize> {
        // the Review toggle and the mode option are one setting
        if key == "diff.mode" {
            return Some(usize::from(self.review.on == Some(true)));
        }
        self.opts.get(key)
    }

    fn set_option(&mut self, key: &str, value: usize) {
        if self.opts.set(key, value) {
            if key == "diff.mode" {
                self.review.on = Some(self.opts.value("diff.mode") == "review");
            }
            self.update_controls();
        }
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn template(&self) -> Option<&Template> {
        Some(&self.template)
    }

    fn template_mut(&mut self) -> Option<&mut Template> {
        Some(&mut self.template)
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
        let area = Rect::new(0, 0, width, 20);
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
                &[
                    page.template.tabs.id,
                    page.review.id,
                    page.empty.id,
                    page.files.id,
                    page.view.id()
                ]
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
            mods: KeyModifiers::NONE,
            clicks: 0,
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
                app.handle(Input::Mouse(Mouse::new(kind, pos)));
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
