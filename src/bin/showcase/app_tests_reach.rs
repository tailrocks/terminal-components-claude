//! The whole content is always reachable (owner priority 2026-10-17):
//! every templated page, every section, at 80x24, 100x25, 106x29 and
//! 120x40, shows at some scroll position of its section body everything
//! it shows on a large screen. The body sits in the template's
//! `ScrollView`, so a section laid out at its natural size scrolls to
//! every row and column; a section whose own layout squeezes instead of
//! overflowing (its Playground has no `Page::measure` yet) loses words,
//! and is listed in `tests/showcase_reach_allow.txt` with its
//! `docs/remaining-work.md` row until its page is fixed. An inner
//! scroller that shows a scrollbar (a list, a table, a log) holds more
//! than its window by design: what wheeling it to its end shows counts
//! as reachable. A listed entry
//! that passes fails as stale, so the list only shrinks.

use std::collections::BTreeSet;

use junie_tui::core::event::{Input, Mouse, MouseKind};
use junie_tui::core::hit::{Axis, Bar};
use junie_tui::core::id::WidgetId;
use junie_tui::runtime::Application;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};

use crate::app::{App, NAV_ENTRIES};
use crate::pages::template::Section;
use junie_tui::theme::Theme;

/// The sizes the owner checks pages at.
const SIZES: [(u16, u16); 4] = [(80, 24), (100, 25), (106, 29), (120, 40)];

/// A screen large enough to show every page whole.
const REFERENCE: (u16, u16) = (220, 70);

/// Wheel notches an inner scroller gets to reach its end.
const INNER_NOTCHES: usize = 400;

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
    /// Words to look for: scrolling stops once all are seen.
    want: Option<BTreeSet<String>>,
}

impl Shot {
    fn new(w: u16, h: u16, page: crate::app::PageId, section: Section) -> Option<Self> {
        let mut app = App::new(Theme::junie());
        app.open_page(page);
        let t = app.pages[page.index()].template_mut()?;
        t.set_section(section);
        let term = Terminal::new(TestBackend::new(w, h)).unwrap();
        let mut s = Self {
            app,
            term,
            want: None,
        };
        s.draw();
        Some(s)
    }

    fn draw(&mut self) {
        self.term.draw(|f| self.app.render(f)).unwrap();
    }

    fn view(&mut self) -> &mut junie_tui::ui::scroll_view::ScrollView {
        let i = self.app.page().index();
        &mut self.app.pages[i].template_mut().unwrap().view
    }

    /// Words in the section body in view.
    fn words(&mut self, out: &mut BTreeSet<String>) {
        let area = self.view().last_layout().viewport;
        words(self.term.backend().buffer(), area, out);
    }

    /// Words in the section body at every scroll position: a window per
    /// view on each axis, and the end of each.
    /// Whether every wanted word was seen.
    fn done(&self, out: &BTreeSet<String>) -> bool {
        self.want.as_ref().is_some_and(|w| w.is_subset(out))
    }

    fn reachable(&mut self, inner: bool) -> BTreeSet<String> {
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
                if self.done(&out) {
                    return out;
                }
                if inner {
                    self.inner(&mut out);
                }
            }
        }
        out
    }

    /// The shared rule for inner scrollers: a scroller in the section
    /// (a list, a table, a log, a stack of panels, a text area that
    /// scrolls its lines sideways) holds more than its window by design,
    /// so what wheeling it brings into view is reachable. Wheel each
    /// scroller in view (from its bars, or over its body when it shows
    /// none) to its start and then to its end (until it stops moving, or
    /// the notch chains on to the section's view), sideways at each window
    /// of rows. Then put the section back where it was.
    fn inner(&mut self, out: &mut BTreeSet<String>) {
        let view = self.view().id;
        let viewport = self.view().last_layout().viewport;
        let bars: Vec<Bar> = self
            .app
            .shell
            .hits
            .bars()
            .iter()
            .filter(|b| b.owner != view && b.track.intersects(viewport))
            .copied()
            .collect();
        // each scroller once: its bars' cells (the wheel over a bar
        // scrolls its owner), and the rows a window holds
        let mut targets: Vec<(WidgetId, Option<Position>, Option<Position>, usize)> = Vec::new();
        for b in &bars {
            if targets.iter().any(|t| t.0 == b.owner) {
                continue;
            }
            let at = |axis: Axis| {
                bars.iter()
                    .find(|x| x.owner == b.owner && x.axis == axis)
                    .map(|x| {
                        let r = x.track.intersection(viewport);
                        Position::new(r.x, r.y)
                    })
            };
            let rows = bars
                .iter()
                .find(|x| x.owner == b.owner && x.axis == Axis::Vertical)
                .map_or(1, |x| (x.track.height / 3).max(1) as usize);
            targets.push((b.owner, at(Axis::Vertical), at(Axis::Horizontal), rows));
        }
        // a wheel-scrollable region without a bar (a text area's sideways
        // scroll) is wheeled over its body both ways
        let regions: Vec<(WidgetId, Rect)> = self
            .app
            .shell
            .hits
            .regions()
            .iter()
            .filter(|r| r.scroll_only && r.id != view && r.area.intersects(viewport))
            .map(|r| (r.id, r.area.intersection(viewport)))
            .collect();
        for (id, r) in regions {
            let pos = Position::new(r.x + r.width / 2, r.y + r.height / 2);
            match targets.iter_mut().find(|t| t.0 == id) {
                // a scroller with one bar is wheeled over its body on the
                // other axis (a code editor's sideways scroll shows none)
                Some(t) => {
                    t.1 = t.1.or(Some(pos));
                    t.2 = t.2.or(Some(pos));
                }
                None => targets.push((id, Some(pos), Some(pos), (r.height / 3).max(1) as usize)),
            }
        }
        for (_, v, h, rows) in targets {
            if self.done(out) {
                return;
            }
            let start = self.view().offset();
            let mut notch = 0usize;
            let mut across = |s: &mut Self, out: &mut BTreeSet<String>| {
                notch += 1;
                if !notch.is_multiple_of(rows) {
                    return;
                }
                if let Some(h) = h {
                    // to its end sideways and back
                    s.sweep(MouseKind::WheelRight, h, start, out, &mut |_, _| {});
                    s.sweep(MouseKind::WheelLeft, h, start, out, &mut |_, _| {});
                }
            };
            if let Some(h) = h {
                self.sweep(MouseKind::WheelRight, h, start, out, &mut |_, _| {});
                self.sweep(MouseKind::WheelLeft, h, start, out, &mut |_, _| {});
            }
            if let Some(v) = v {
                // to its start (a log that follows sits at its end), then
                // to its end
                self.sweep(MouseKind::WheelUp, v, start, out, &mut across);
                self.sweep(MouseKind::WheelDown, v, start, out, &mut across);
            }
        }
    }

    /// Wheel `kind` at `pos` until the screen stops changing or the notch
    /// chains on to the section's view (it left `start`), calling `each`
    /// after every notch that moved something; then the section's view
    /// back at `start`.
    fn sweep(
        &mut self,
        kind: MouseKind,
        pos: Position,
        start: (usize, usize),
        out: &mut BTreeSet<String>,
        each: &mut dyn FnMut(&mut Self, &mut BTreeSet<String>),
    ) {
        let mut last = text(self.term.backend().buffer());
        for _ in 0..INNER_NOTCHES {
            self.app.handle(Input::Mouse(Mouse::new(kind, pos)));
            self.draw();
            self.words(out);
            let now = text(self.term.backend().buffer());
            if self.view().offset() != start || now == last || self.done(out) {
                break;
            }
            last = now;
            each(self, out);
        }
        self.view().scroll_to(start.0, start.1);
        self.draw();
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

/// The gate over every `PARTS`-th page from `part` (the pages split so
/// nextest runs the parts side by side): each section at each size, and
/// the allowlist's entries for those pages.
fn reach_part(part: usize) {
    let allowed = allowed();
    let mut problems = Vec::new();
    let mut failing = BTreeSet::new();
    let mut pages = BTreeSet::new();
    for entry in NAV_ENTRIES.iter().skip(part).step_by(PARTS) {
        pages.insert(format!("{:?}", entry.id));
        let t0 = std::time::Instant::now();
        struct T(std::time::Instant, String);
        impl Drop for T { fn drop(&mut self) { eprintln!("TIME {} {:?}", self.1, self.0.elapsed()); } }
        let _t = T(t0, format!("{:?}", entry.id));
        for section in Section::ALL {
            let Some(mut big) = Shot::new(REFERENCE.0, REFERENCE.1, entry.id, section) else {
                break;
            };
            let want = big.reachable(false);
            for (w, h) in SIZES {
                let mut small = Shot::new(w, h, entry.id, section).unwrap();
                small.want = Some(want.clone());
                let seen = small.reachable(true);
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
                eprintln!("DUMP {key}: {}", missing.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" "));
                if !allowed.contains(&key) {
                    let shown: Vec<&str> = missing.iter().take(12).map(|s| s.as_str()).collect();
                    problems.push(format!("{key}: never in view: {}", shown.join(" ")));
                }
            }
        }
    }
    let mine = allowed
        .iter()
        .filter(|k| pages.contains(k.split(" | ").next().unwrap_or_default()));
    for key in mine {
        if !failing.contains(key) && !problems.iter().any(|p| p.contains(key.as_str())) {
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

/// The parts the gate runs in.
const PARTS: usize = 8;

#[test]
fn every_section_reaches_its_whole_content_by_scrolling_0() {
    reach_part(0);
}
#[test]
fn every_section_reaches_its_whole_content_by_scrolling_1() {
    reach_part(1);
}
#[test]
fn every_section_reaches_its_whole_content_by_scrolling_2() {
    reach_part(2);
}
#[test]
fn every_section_reaches_its_whole_content_by_scrolling_3() {
    reach_part(3);
}
#[test]
fn every_section_reaches_its_whole_content_by_scrolling_4() {
    reach_part(4);
}
#[test]
fn every_section_reaches_its_whole_content_by_scrolling_5() {
    reach_part(5);
}
#[test]
fn every_section_reaches_its_whole_content_by_scrolling_6() {
    reach_part(6);
}
#[test]
fn every_section_reaches_its_whole_content_by_scrolling_7() {
    reach_part(7);
}

/// Every allowlisted page is a page the gate walks (a renamed page's
/// lines would otherwise never be checked).
#[test]
fn every_reach_allowlist_entry_names_a_page() {
    let pages: BTreeSet<String> = NAV_ENTRIES.iter().map(|e| format!("{:?}", e.id)).collect();
    for key in allowed() {
        let page = key.split(" | ").next().unwrap_or_default();
        assert!(
            pages.contains(page),
            "showcase_reach_allow.txt: no page {page}"
        );
    }
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
    app.open_page(crate::app::PageId::Buttons);
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
