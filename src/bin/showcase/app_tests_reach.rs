//! The whole content is always reachable (owner priority 2026-10-17):
//! every templated page, every section, at 80x24, 100x25, 106x29 and
//! 120x40, shows at some scroll position of its section body everything
//! it shows on a large screen. The body sits in the template's
//! `ScrollView`, so a section laid out at its natural size scrolls to
//! every row and column; a section whose own layout squeezes instead of
//! overflowing (its Playground has no `Page::measure` yet) loses words,
//! and is listed in `tests/showcase_reach_allow.txt` with its
//! `docs/remaining-work.md` row until its page is fixed. A listed entry
//! that passes fails as stale, so the list only shrinks.

use std::collections::BTreeSet;

use junie_tui::runtime::Application;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::app::{App, NAV_ENTRIES};
use crate::pages::template::Section;
use junie_tui::theme::Theme;

/// The sizes the owner checks pages at.
const SIZES: [(u16, u16); 4] = [(80, 24), (100, 25), (106, 29), (120, 40)];

/// A screen large enough to show every page whole.
const REFERENCE: (u16, u16) = (220, 70);

/// Words of three or more letters or digits in `area`: what a reader
/// would miss when the section cuts it off.
fn words(buf: &Buffer, area: Rect, out: &mut BTreeSet<String>) {
    for y in area.top()..area.bottom() {
        let row: String = (area.left()..area.right())
            .map(|x| buf[(x, y)].symbol())
            .collect();
        for w in row.split(|c: char| !c.is_alphanumeric()) {
            if w.chars().count() >= 3 {
                out.insert(w.to_owned());
            }
        }
    }
}

struct Shot {
    app: App,
    term: Terminal<TestBackend>,
}

impl Shot {
    fn new(w: u16, h: u16, page: crate::app::PageId, section: Section) -> Option<Self> {
        let mut app = App::new(Theme::junie());
        app.goto(page);
        let t = app.pages[page.index()].template_mut()?;
        t.set_section(section);
        let term = Terminal::new(TestBackend::new(w, h)).unwrap();
        let mut s = Self { app, term };
        s.draw();
        Some(s)
    }

    fn draw(&mut self) {
        self.term.draw(|f| self.app.render(f)).unwrap();
    }

    fn view(&mut self) -> &mut junie_tui::ui::scroll_view::ScrollView {
        let i = self.app.page.index();
        &mut self.app.pages[i].template_mut().unwrap().view
    }

    /// Words in the section body in view.
    fn words(&mut self, out: &mut BTreeSet<String>) {
        let area = self.view().last_layout().viewport;
        words(self.term.backend().buffer(), area, out);
    }

    /// Words in the section body at every scroll position: a window per
    /// view on each axis, and the end of each.
    fn reachable(&mut self) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        self.words(&mut out);
        let l = self.view().last_layout();
        let steps = |content: u16, view: u16| -> Vec<usize> {
            // windows overlap by half, so a word cut by one edge is
            // whole in the next
            let max = content.saturating_sub(view) as usize;
            let step = (view / 2).max(1) as usize;
            let mut v: Vec<usize> = (0..=max).step_by(step).collect();
            v.push(max);
            v
        };
        for y in steps(l.content.height, l.viewport.height) {
            for x in steps(l.content.width, l.viewport.width) {
                self.view().scroll_to(x, y);
                self.draw();
                self.words(&mut out);
            }
        }
        out
    }
}

/// `slug | section | WxH` entries of the allowlist.
fn allowed() -> BTreeSet<String> {
    let text = include_str!("../../../tests/showcase_reach_allow.txt");
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            let parts: Vec<&str> = l.split('|').map(str::trim).collect();
            assert!(
                parts.len() == 4 && parts[3].starts_with('R'),
                "showcase_reach_allow.txt: `page | section | WxH | Rn reason`: {l}"
            );
            format!("{} | {} | {}", parts[0], parts[1], parts[2])
        })
        .collect()
}

#[test]
fn every_section_reaches_its_whole_content_by_scrolling() {
    let allowed = allowed();
    let mut problems = Vec::new();
    let mut failing = BTreeSet::new();
    for entry in NAV_ENTRIES {
        for section in Section::ALL {
            let Some(mut big) = Shot::new(REFERENCE.0, REFERENCE.1, entry.id, section) else {
                break;
            };
            let want = big.reachable();
            for (w, h) in SIZES {
                let mut small = Shot::new(w, h, entry.id, section).unwrap();
                let seen = small.reachable();
                let missing: Vec<&String> = want.difference(&seen).collect();
                let key = format!("{:?} | {} | {w}x{h}", entry.id, section.label());
                if missing.is_empty() {
                    if allowed.contains(&key) {
                        problems.push(format!(
                            "stale: {key} reaches everything now; delete its line"
                        ));
                    }
                    continue;
                }
                failing.insert(key.clone());
                if !allowed.contains(&key) {
                    let shown: Vec<&str> = missing.iter().take(12).map(|s| s.as_str()).collect();
                    problems.push(format!("{key}: never in view: {}", shown.join(" ")));
                }
            }
        }
    }
    for key in allowed.difference(&failing) {
        if !problems.iter().any(|p| p.contains(key.as_str())) {
            problems.push(format!(
                "stale: {key} is listed but not failing; delete its line"
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "content cut off for good (tests/showcase_reach_allow.txt):\n{}",
        problems.join("\n")
    );
}

/// Toggles enough to fill any page twice over.
static LONG: &[crate::options::Opt] = &{
    use crate::options::{Effect, Opt};
    const fn opt(key: &'static str, label: &'static str) -> Opt {
        Opt {
            key,
            label,
            values: &["off", "on"],
            default: 0,
            effect: Effect::Frame,
            needs: &[],
        }
    }
    [
        opt("long.a01", "Option one"),
        opt("long.a02", "Option two"),
        opt("long.a03", "Option three"),
        opt("long.a04", "Option four"),
        opt("long.a05", "Option five"),
        opt("long.a06", "Option six"),
        opt("long.a07", "Option seven"),
        opt("long.a08", "Option eight"),
        opt("long.a09", "Option nine"),
        opt("long.a10", "Option ten"),
        opt("long.a11", "Option eleven"),
        opt("long.a12", "Option twelve"),
        opt("long.a13", "Option thirteen"),
        opt("long.a14", "Option fourteen"),
        opt("long.a15", "Option fifteen"),
        opt("long.a16", "Option sixteen"),
        opt("long.a17", "Option seventeen"),
        opt("long.a18", "Option eighteen"),
        opt("long.a19", "Option nineteen"),
        opt("long.a20", "Option twenty"),
        opt("long.a21", "Option twenty-one"),
        opt("long.a22", "Option twenty-two"),
        opt("long.a23", "Option twenty-three"),
        opt("long.a24", "The last option"),
    ]
};

/// One frame of a template's Options section over [`LONG`] in a 60x16
/// page, with `focus` focused: the screen and the hit regions.
fn options_frame(
    t: &mut crate::pages::template::Template,
    focus: Option<junie_tui::core::id::WidgetId>,
) -> (Buffer, junie_tui::core::hit::HitRegistry) {
    use junie_tui::core::focus::FocusRing;
    use junie_tui::core::hit::HitRegistry;
    use junie_tui::ui::ctx::{Interaction, RenderCtx};
    let theme = Theme::junie();
    let mut hits = HitRegistry::default();
    let mut ring = FocusRing::default();
    let area = Rect::new(0, 0, 60, 16);
    let mut buf = Buffer::empty(area);
    {
        let interaction = Interaction {
            focus,
            ..Interaction::default()
        };
        let mut ctx = RenderCtx::new(&theme, interaction, &mut hits, &mut ring);
        t.sync_options(LONG, |_| 0);
        let mut frame = t.begin(area, area.as_size(), &buf, &mut ctx);
        let body = t.render(area, frame.buffer(&mut buf), &mut ctx);
        t.render_options(body, frame.buffer(&mut buf), &mut ctx);
        t.end(frame, &mut buf, &mut ctx);
        ctx.flush_overlays(&mut buf);
    }
    (buf, hits)
}

fn text(buf: &Buffer) -> String {
    (0..buf.area.height)
        .map(|y| {
            (0..buf.area.width)
                .map(|x| buf[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// S17 data point (coordinator, 2026-10-17): a long option table pushed
/// options out of view. The Options section scrolls: its bar shows, every
/// option is reachable by scrolling, and Tab onto one out of view brings it
/// into view.
#[test]
fn a_long_option_table_scrolls_and_follows_focus() {
    use crate::pages::template::{Section, Template};
    use junie_tui::core::id::WidgetId;
    use junie_tui::ui::specimen::StateGrid;
    let owner = WidgetId::of("long");
    let mut t = Template::new(owner, StateGrid::new(owner.sub("grid")));
    t.set_section(Section::Options);
    let (buf, _) = options_frame(&mut t, None);
    let l = t.view.last_layout();
    assert!(l.v_track.is_some(), "the section shows its bar");
    assert!(!text(&buf).contains("The last option"));
    // scrolled to the end, the last option is in view
    t.view.scroll_to(0, usize::MAX);
    let (buf, _) = options_frame(&mut t, None);
    assert!(text(&buf).contains("The last option"), "{}", text(&buf));
    // focus on the last option from the top: it comes into view
    t.view.reset();
    let last = t.option_control("long.a24").unwrap();
    let (buf, hits) = options_frame(&mut t, Some(last));
    let at = hits.area_of(last).expect("the focused option is on screen");
    assert!(t.view.last_layout().viewport.contains(at.as_position()));
    assert!(text(&buf).contains("The last option"));
}

/// The Inspector's Options pane holds the page's options and every shell
/// option: on a short screen it scrolls, by wheel, to the last of them.
#[test]
fn the_inspector_options_pane_scrolls_to_its_last_option() {
    use junie_tui::core::event::{Input, Mouse, MouseKind};
    use ratatui::layout::Position;
    let last = crate::options::SHELL.last().unwrap().label;
    let mut app = App::new(Theme::junie());
    app.goto(crate::app::PageId::Buttons);
    let mut term = Terminal::new(TestBackend::new(120, 24)).unwrap();
    term.draw(|f| app.render(f)).unwrap();
    app.toggle_inspector();
    app.insp.set_tab(crate::inspector::Tab::Options);
    term.draw(|f| app.render(f)).unwrap();
    let l = app.insp.view.last_layout();
    assert!(l.overflows() && l.v_track.is_some(), "{l:?}");
    let shown = |term: &Terminal<TestBackend>| {
        let b = term.backend().buffer();
        let mut s = BTreeSet::new();
        words(b, b.area, &mut s);
        s
    };
    let word = last.split(' ').next_back().unwrap().to_owned();
    assert!(!shown(&term).contains(&word));
    let at = Position::new(l.viewport.x + 2, l.viewport.y + 2);
    for _ in 0..40 {
        app.handle(Input::Mouse(Mouse::new(MouseKind::WheelDown, at)));
        term.draw(|f| app.render(f)).unwrap();
    }
    assert!(app.insp.view.offset().1 > 0);
    assert!(shown(&term).contains(&word), "{last}");
}
