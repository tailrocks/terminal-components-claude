use std::collections::BTreeSet;

use junie_tui::core::event::{Input, Mouse, MouseKind};
use junie_tui::core::hit::{Axis, Bar};
use junie_tui::core::id::WidgetId;
use junie_tui::runtime::Application;
use junie_tui::shell::reach::Words;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};

use crate::app::{App, NAV_ENTRIES};
use crate::pages::template::Section;
use junie_tui::theme::Theme;

const SIZES: [(u16, u16); 4] = [(80, 24), (100, 25), (106, 29), (120, 40)];

const REFERENCE: (u16, u16) = (220, 70);

const INNER_NOTCHES: usize = 400;

fn words(buf: &Buffer, area: Rect, out: &mut Words) {
    out.add_area(buf, area);
}

struct Shot {
    app: App,
    term: Terminal<TestBackend>,

    want: Option<Words>,
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

    fn words(&mut self, out: &mut Words) {
        let area = self.view().last_layout().viewport;
        words(self.term.backend().buffer(), area, out);
        out.add_overflowed();
    }

    fn done(&self, out: &Words) -> bool {
        self.want
            .as_ref()
            .is_some_and(|w| w.against(out).0.is_empty())
    }

    fn reachable(&mut self, inner: bool) -> Words {
        let mut out = Words::default();
        self.words(&mut out);
        let l = self.view().last_layout();
        let steps = |content: u16, view: u16| -> Vec<usize> {
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

    fn inner(&mut self, out: &mut Words) {
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
        let mut owners = Vec::new();
        for b in &bars {
            if !owners.contains(&b.owner) {
                owners.push(b.owner);
            }
        }

        let scrollers: Vec<(WidgetId, Rect)> = self
            .app
            .shell
            .hits
            .scrollers()
            .into_iter()
            .filter(|(id, a)| *id != view && a.intersects(viewport))
            .collect();
        for (id, _) in &scrollers {
            if !owners.contains(id) {
                owners.push(*id);
            }
        }
        for owner in owners {
            if self.done(out) {
                return;
            }

            let body = scrollers
                .iter()
                .find(|(id, _)| *id == owner)
                .map(|(_, a)| a.intersection(viewport))
                .filter(|a| !a.is_empty());
            let at = |axis: Axis| {
                bars.iter()
                    .find(|b| b.owner == owner && b.axis == axis)
                    .map(|b| {
                        let r = b.track.intersection(viewport);
                        Position::new(r.x, r.y)
                    })
                    .or_else(|| body.map(|a| Position::new(a.x + a.width / 2, a.y + a.height / 2)))
            };
            let (v, h) = (at(Axis::Vertical), at(Axis::Horizontal));

            let rows = bars
                .iter()
                .find(|b| b.owner == owner && b.axis == Axis::Vertical)
                .map(|b| b.track.height)
                .or(body.map(|a| a.height))
                .map_or(1, |h| (h / 3).max(1) as usize);
            let start = self.view().offset();
            let mut notch = 0usize;
            let mut across = |s: &mut Self, out: &mut Words| {
                notch += 1;
                if !notch.is_multiple_of(rows) {
                    return;
                }
                if let Some(h) = h {
                    s.sweep(MouseKind::WheelRight, h, start, out, &mut |_, _| {});
                    s.sweep(MouseKind::WheelLeft, h, start, out, &mut |_, _| {});
                }
            };
            if let Some(h) = h {
                self.sweep(MouseKind::WheelRight, h, start, out, &mut |_, _| {});
                self.sweep(MouseKind::WheelLeft, h, start, out, &mut |_, _| {});
            }
            if let Some(v) = v {
                self.sweep(MouseKind::WheelUp, v, start, out, &mut across);
                self.sweep(MouseKind::WheelDown, v, start, out, &mut across);
            }
        }
    }

    fn sweep(
        &mut self,
        kind: MouseKind,
        pos: Position,
        start: (usize, usize),
        out: &mut Words,
        each: &mut dyn FnMut(&mut Self, &mut Words),
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

fn reach_part(part: usize) {
    let allowed = allowed();
    let mut problems = Vec::new();
    let mut failing = BTreeSet::new();
    let mut pages = BTreeSet::new();
    for entry in NAV_ENTRIES.iter().skip(part).step_by(PARTS) {
        pages.insert(format!("{:?}", entry.id));
        for section in Section::ALL {
            let Some(mut big) = Shot::new(REFERENCE.0, REFERENCE.1, entry.id, section) else {
                break;
            };
            let want = big.reachable(false);
            let want = Words {
                whole: want.whole,
                ..Words::default()
            };
            for (w, h) in SIZES {
                let mut small = Shot::new(w, h, entry.id, section).unwrap();
                small.want = Some(want.clone());
                let seen = small.reachable(true);
                let (missing, _cut) = want.against(&seen);
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

    t.view.scroll_to(0, usize::MAX);
    let (buf, _) = options_frame(&mut t, None);
    assert!(text(&buf).contains("The last option"), "{}", text(&buf));

    t.view.reset();
    let last = t.option_control("long.a24").unwrap();
    let (buf, hits) = options_frame(&mut t, Some(last));
    let at = hits.area_of(last).expect("the focused option is on screen");
    assert!(t.view.last_layout().viewport.contains(at.as_position()));
    assert!(text(&buf).contains("The last option"));
}

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
        let mut s = Words::default();
        words(b, b.area, &mut s);
        s.whole
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
