//! The reachability harness: does every word a screen shows at a generous
//! size stay reachable at every shipping size?
//!
//! Ported from PR #26 `src/shell/reach.rs` (commit `f4867e9b5`). The parity
//! adaptation drives the [`termrock_runtime::Runtime`] directly over a test
//! buffer instead of the source's `Application` trait, and wheels through
//! [`termrock_core::event::MouseKind::Wheel`] (axis plus notch delta) instead
//! of four wheel kinds. The source's `Words::add_overflowed` and the
//! reveal-driven `Reveal` steps are not ported: they read the source's
//! `ui::reveal` overflow log, which the parity feedback wave owns; a case
//! that hides words only behind truncation reveals stays an allow-file row
//! until that wave lands.

use std::collections::{BTreeMap, BTreeSet};

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Position;
use ratatui_core::layout::Rect;
use termrock_core::event::{Input, MouseKind};
use termrock_core::platform::ClientPlatform;
use termrock_runtime::event::Axis;
use termrock_runtime::hit::{Bar, RegionKind, Registry};
use termrock_runtime::{App, Id, Runtime};
use termrock_theme::Theme as ThemeType;

/// The sizes a screen must survive.
pub const SIZES: [(u16, u16); 8] = [
    (40, 12),
    (60, 18),
    (72, 20),
    (80, 24),
    (100, 25),
    (106, 29),
    (120, 40),
    (160, 50),
];

const REFERENCE: (u16, u16) = (300, 100);

const MIN_WORD: usize = 3;

const MAX_PAGES: usize = 200;

/// Pages before an axis gives up; stalls before the walk backs off.
const STALL: (usize, usize) = (3, 12);

/// What one scenario at one size reached.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Reach {
    /// The size probed.
    pub size: (u16, u16),
    /// The screen showed its too-small panel instead.
    pub too_small: bool,
    /// Reference words no scroll reached.
    pub missing: BTreeSet<String>,
    /// Reference words reached only as a cut prefix.
    pub cut: BTreeSet<String>,
    /// Wheel pages the exploration turned.
    pub bars: usize,
}

impl Reach {
    pub(crate) fn of(
        size: (u16, u16),
        too_small: bool,
        want: &Words,
        reached: &Words,
        bars: usize,
    ) -> Self {
        if too_small {
            return Self {
                size,
                too_small,
                ..Self::default()
            };
        }
        let (missing, cut) = want.against(reached);
        Self {
            size,
            too_small,
            missing,
            cut,
            bars,
        }
    }
}

/// The words one frame shows: whole words, and prefixes cut by truncation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Words {
    /// Words shown in full.
    pub whole: BTreeSet<String>,
    /// Prefixes a `…` cell cut short.
    pub cut: BTreeSet<String>,
}

impl Words {
    /// Read a whole buffer minus its footer row.
    pub fn read(buf: &Buffer) -> Self {
        let mut out = Self::default();
        out.add(buf);
        out
    }

    /// Add a buffer's words, skipping its footer row.
    pub fn add(&mut self, buf: &Buffer) {
        let area = buf.area();
        self.add_area(
            buf,
            Rect::new(area.x, area.y, area.width, area.height.saturating_sub(1)),
        );
    }

    /// Add the words of one region of a buffer.
    pub fn add_area(&mut self, buf: &Buffer, area: Rect) {
        let area = area.intersection(*buf.area());
        let rows = area.height;
        for y in 0..rows {
            let mut word = String::new();
            for x in 0..area.width {
                let sym = buf[(area.x + x, area.y + y)].symbol();
                let mut chars = sym.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) if c.is_alphanumeric() => word.push(c),
                    (Some('…'), None) => {
                        if !word.is_empty() {
                            self.cut.insert(std::mem::take(&mut word));
                        }
                    }
                    (None, _) => {}
                    _ => self.push(&mut word),
                }
            }
            self.push(&mut word);
        }
    }

    fn push(&mut self, word: &mut String) {
        if word.chars().count() >= MIN_WORD {
            self.whole.insert(word.clone());
        }
        word.clear();
    }

    /// Merge another frame's words in.
    pub fn extend(&mut self, other: Self) {
        self.whole.extend(other.whole);
        self.cut.extend(other.cut);
    }

    /// Sort the reference words `reached` never showed whole into missing
    /// and cut.
    pub fn against(&self, reached: &Self) -> (BTreeSet<String>, BTreeSet<String>) {
        let mut missing = BTreeSet::new();
        let mut cut = BTreeSet::new();
        for w in &self.whole {
            if reached.whole.contains(w) {
                continue;
            }
            if reached.whole.iter().any(|r| r.contains(w.as_str())) {
                continue;
            }
            if reached.cut.iter().any(|p| w.starts_with(p.as_str())) {
                cut.insert(w.clone());
            } else {
                missing.insert(w.clone());
            }
        }
        (missing, cut)
    }
}

/// One scenario the reach harness drives: an app plus the setup that puts
/// its screen on show.
pub trait Scenario: Sized {
    /// The app under test.
    type App: App;

    /// The theme the app runs in.
    fn theme() -> ThemeType {
        ThemeType::junie()
    }

    /// A fresh app in its initial screen state.
    fn fresh() -> Self::App;

    /// Is the app showing its too-small panel at this size? Too-small
    /// scenarios are recorded, not failed.
    fn too_small(app: &Self::App) -> bool {
        let _ = app;
        false
    }
}

/// One named screen with its setup.
pub struct Case<S: Scenario + 'static> {
    /// The scenario's name in the allow file.
    pub name: &'static str,
    /// The setup that shows the screen, run before the runtime starts.
    pub base: fn(&mut S::App),
}

impl<S: Scenario + 'static> Case<S> {
    /// A page case.
    pub const fn page(name: &'static str, base: fn(&mut S::App)) -> Self {
        Self { name, base }
    }

    fn build(&self) -> Probe<S> {
        let mut app = S::fresh();
        (self.base)(&mut app);
        Probe::new(app, S::theme())
    }

    /// Run the case at `size` and report what it reached.
    pub fn run(&self, size: (u16, u16)) -> Reach {
        let mut probe = self.build();
        let want = probe.reference(REFERENCE);
        probe.resize(size.0, size.1);
        let small = S::too_small(probe.rt.app());
        let mut reached = Words::default();
        let bars = probe.explore(&want, &mut reached);
        Reach::of(size, small, &want, &reached, bars)
    }
}

/// A scenario probe driving the runtime over a plain test buffer.
pub(crate) struct Probe<S: Scenario> {
    rt: Runtime<S::App>,
    buf: Buffer,
    area: Rect,
}

impl<S: Scenario> Probe<S> {
    fn new(app: S::App, theme: ThemeType) -> Self {
        let area = Rect::new(0, 0, 80, 24);
        let mut rt = Runtime::new(app, theme);
        let buf = Buffer::empty(area);
        let _ = rt.initialize();
        let mut probe = Self { rt, buf, area };
        probe.draw();
        probe
    }

    /// Settle pending work, then draw one committed frame.
    fn draw(&mut self) {
        for _ in 0..16 {
            if self.rt.needs_settle() {
                let _ = self.rt.settle();
            }
            if !self.rt.needs_present() && !self.rt.needs_settle() {
                return;
            }
            self.rt
                .draw_buffer(self.area, &mut self.buf)
                .commit_presented();
        }
    }

    fn handle(&mut self, input: Input) {
        // The runtime takes an input only once the app is ready. A pending
        // input is settled and retried, so a probe that arrives too early
        // loses the step instead of panicking the harness.
        let mut input = input;
        for _ in 0..16 {
            match self.rt.handle(input) {
                Ok(_) => break,
                Err(pending) => {
                    input = pending.into_input();
                    let _ = self.rt.settle();
                }
            }
        }
        self.draw();
    }

    fn resize(&mut self, w: u16, h: u16) {
        self.area = Rect::new(0, 0, w, h);
        self.buf = Buffer::empty(self.area);
        self.handle(Input::Resize(w, h));
    }

    fn buffer(&self) -> &Buffer {
        &self.buf
    }
}

impl<S: Scenario> Probe<S> {
    fn registry(&self) -> Option<&Registry> {
        self.rt.projection_registry()
    }

    fn bars(&self) -> Vec<Bar> {
        self.registry()
            .map(|hits| hits.bars().to_vec())
            .unwrap_or_default()
    }

    fn bar_of(&self, owner: Id, axis: Axis) -> Option<Bar> {
        self.bars()
            .into_iter()
            .find(|b| b.owner == owner && b.axis == axis)
    }

    /// The position a wheel event at `bar` lands on: the owner's centre when
    /// it is under the scroll stack, else beside the track.
    fn wheel_spot(&self, bar: Bar) -> Position {
        let Some(hits) = self.registry() else {
            return Position::ORIGIN;
        };
        let reaches = |p: Position| hits.hit_scroll(p, bar.axis).is_some();
        let middle = hits
            .regions()
            .iter()
            .rev()
            .find(|r| r.owner == bar.owner && r.kind == RegionKind::Control)
            .map(|r| Position::new(r.area.x + r.area.width / 2, r.area.y + r.area.height / 2));
        if let Some(p) = middle.filter(|&p| reaches(p)) {
            return p;
        }
        let t = bar.track;
        match bar.axis {
            Axis::V => Position::new(t.x.saturating_sub(1), t.y + t.height / 2),
            Axis::H => Position::new(t.x + t.width / 2, t.y.saturating_sub(1)),
        }
    }

    fn wheel(&mut self, bar: Bar, forward: bool) {
        let delta = (notches(bar) as i16) * if forward { 1 } else { -1 };
        let kind = MouseKind::Wheel(bar.axis, delta);
        let at = self.wheel_spot(bar);
        self.handle(Input::Mouse(termrock_core::event::Mouse {
            kind,
            pos: at,
            mods: termrock_core::keys::KeyModifiers::NONE,
        }));
    }

    /// One wheel page. `true` when the frame changed.
    fn page(&mut self, owner: Id, axis: Axis, forward: bool) -> bool {
        let Some(bar) = self.bar_of(owner, axis) else {
            return false;
        };
        let before = self.buffer().clone();
        self.wheel(bar, forward);
        *self.buffer() != before
    }

    fn page_to_end(&mut self, owner: Id, axis: Axis, forward: bool, acc: &mut Acc) -> usize {
        let limit = match axis {
            Axis::V => STALL.0,
            Axis::H => STALL.1,
        };
        let mut pages = 0;
        let mut stalled = 0;
        while pages < MAX_PAGES && stalled < limit && !acc.done() && self.page(owner, axis, forward)
        {
            pages += 1;
            stalled = if acc.add(self.buffer()) {
                0
            } else {
                stalled + 1
            };
        }
        pages
    }

    fn page_back(&mut self, owner: Id, axis: Axis, forward: bool, pages: usize) {
        if pages == 0 {
            return;
        }
        let Some(bar) = self.bar_of(owner, axis) else {
            return;
        };
        for _ in 0..pages * usize::from(notches(bar)) {
            self.wheel(bar, !forward);
        }
    }

    /// The reference words at [`REFERENCE`], minus the reference size's own
    /// dimensions so the size itself never counts as missing.
    fn reference(&mut self, reference: (u16, u16)) -> Words {
        self.resize(reference.0, reference.1);
        let mut want = Words::read(self.buffer());
        let dims = [reference.0.to_string(), reference.1.to_string()];
        want.whole.retain(|w| !dims.contains(w));
        want.cut.clear();
        want
    }

    /// Wheel every scrollable to both ends, collecting what shows, until
    /// every reference word is reached or nothing new turns up.
    fn explore(&mut self, want: &Words, reached: &mut Words) -> usize {
        let left = want.against(reached).0;
        let mut acc = Acc {
            reached: std::mem::take(reached),
            left,
            want: want.whole.clone(),
            last: BTreeSet::new(),
        };
        acc.add(self.buffer());
        let mut done: Vec<Id> = Vec::new();
        self.explore_owners(&mut acc, &mut done, 0);
        *reached = acc.reached;
        done.len()
    }

    fn explore_owners(&mut self, acc: &mut Acc, done: &mut Vec<Id>, depth: u8) {
        for bar in self.bars() {
            if done.contains(&bar.owner) || acc.done() {
                continue;
            }
            let owner = bar.owner;
            done.push(owner);
            let v = self.bar_of(owner, Axis::V).is_some();
            let h = self.bar_of(owner, Axis::H).is_some();

            let mut v_pages = 0;
            let mut stalled = 0;
            loop {
                if h {
                    let pages = self.page_to_end(owner, Axis::H, true, acc);
                    self.page_back(owner, Axis::H, true, pages);
                }
                if !v || acc.done() || v_pages >= MAX_PAGES || stalled >= STALL.0 {
                    break;
                }
                if !self.page(owner, Axis::V, true) {
                    break;
                }
                v_pages += 1;
                stalled = if acc.add(self.buffer()) {
                    0
                } else {
                    stalled + 1
                };
            }
            if depth == 0 {
                self.explore_owners(acc, done, depth + 1);
            }
            self.page_back(owner, Axis::V, true, v_pages);

            for axis in [Axis::V, Axis::H] {
                if self.bar_of(owner, axis).is_some() && !acc.done() {
                    let pages = self.page_to_end(owner, axis, false, acc);
                    self.page_back(owner, axis, false, pages);
                }
            }
        }
    }
}

fn notches(bar: Bar) -> u16 {
    match bar.axis {
        Axis::V if bar.track.height < 9 => 1,
        Axis::V => (bar.track.height / 3 - 1).max(1),
        Axis::H => 1,
    }
}

struct Acc {
    reached: Words,
    left: BTreeSet<String>,
    want: BTreeSet<String>,
    last: BTreeSet<String>,
}

impl Acc {
    fn add(&mut self, buf: &Buffer) -> bool {
        let frame = Words::read(buf);
        let fresh = frame
            .whole
            .iter()
            .any(|w| self.want.contains(w) && !self.last.contains(w));
        self.last.clone_from(&frame.whole);
        if !self.left.is_empty() {
            let probe = Words {
                whole: std::mem::take(&mut self.left),
                cut: BTreeSet::new(),
            };
            self.left = probe.against(&frame).0;
        }
        self.reached.extend(frame);
        fresh
    }

    fn done(&self) -> bool {
        self.left.is_empty()
    }
}

/// Check the found reach against the allow file. Every scenario line reads
/// `<app> <scenario> <WxH> <word...> # <reason>`; a fixed row must drop its
/// line and a new row must carry a reason.
pub fn gate(app: &str, found: &[(String, Reach)], allow: &str) -> Result<(), String> {
    let mut allowed: BTreeMap<(String, String), (BTreeSet<String>, String)> = BTreeMap::new();
    for line in allow.lines() {
        let (body, note) = line.split_once('#').unwrap_or((line, ""));
        let mut it = body.split_whitespace();
        if it.next() != Some(app) {
            continue;
        }
        let (Some(scenario), Some(size)) = (it.next(), it.next()) else {
            continue;
        };
        let (scenario, only) = match scenario.split_once('@') {
            Some((name, plat)) => (name, Some(plat)),
            None => (scenario, None),
        };
        let here = match ClientPlatform::current() {
            ClientPlatform::MacOs => "macos",
            ClientPlatform::Other => "other",
        };
        if only.is_some_and(|p| p != here) {
            continue;
        }
        if only.is_none() && allowed.contains_key(&(scenario.to_string(), size.to_string())) {
            continue;
        }
        let shown = |r: &Reach| format!("{}x{}", r.size.0, r.size.1) == size;
        if !found.iter().any(|(s, r)| s == scenario && shown(r)) {
            continue;
        }
        allowed.insert(
            (scenario.to_string(), size.to_string()),
            (it.map(str::to_string).collect(), note.trim().to_string()),
        );
    }
    let mut want = String::new();
    let mut bad = Vec::new();
    for (scenario, r) in found {
        let size = format!("{}x{}", r.size.0, r.size.1);
        let key = (scenario.clone(), size.clone());
        let (ok, note) = allowed.remove(&key).unwrap_or_default();
        if r.missing.is_empty() {
            if !ok.is_empty() {
                bad.push(format!("{scenario} {size}: fixed, drop its line"));
            }
            continue;
        }
        let words: Vec<&str> = r.missing.iter().map(String::as_str).collect();
        let note = if note.is_empty() {
            "R-apps-? <reason>"
        } else {
            &note
        };
        want.push_str(&format!(
            "{app} {scenario} {size} {} # {note}\n",
            words.join(" ")
        ));
        if ok != r.missing {
            let new: Vec<_> = r.missing.difference(&ok).collect();
            let gone: Vec<_> = ok.difference(&r.missing).collect();
            bad.push(format!(
                "{scenario} {size}: newly hidden {new:?}, now reached {gone:?}"
            ));
        }
    }
    for (scenario, size) in allowed.keys() {
        bad.push(format!("{scenario} {size}: no such row, drop its line"));
    }
    if bad.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{app}: reachability differs from tests/reach_allow.txt\n  {}\n\
             {app}'s lines should read:\n{want}",
            bad.join("\n  ")
        ))
    }
}

/// Check one case at one size against the allow file.
pub fn check<S: Scenario>(app: &str, cases: &[Case<S>], name: &str, size: (u16, u16), allow: &str) {
    let case = cases
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("no case {name}"));
    let found = [(name.to_string(), case.run(size))];
    if let Err(e) = gate(app, &found, allow) {
        panic!("{e}");
    }
}

/// Dump one case's frame and reach report under `REACH_DUMP=name[:w[:h]]`.
pub fn dump<S: Scenario>(cases: &[Case<S>]) {
    let Ok(spec) = std::env::var("REACH_DUMP") else {
        return;
    };
    let mut it = spec.split(':');
    let name = it.next().unwrap_or_default();
    let mut num = || it.next().and_then(|n| n.parse().ok()).unwrap_or(80);
    let size = (num(), num());
    let case = cases
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("no case {name}"));
    let mut probe = case.build();
    probe.resize(size.0, size.1);
    let buf = probe.buffer();
    let mut text = String::new();
    for y in 0..buf.area().height {
        for x in 0..buf.area().width {
            text.push_str(buf[(x, y)].symbol());
        }
        text.push('\n');
    }
    let want = probe.reference(size);
    let mut reached = Words::default();
    probe.explore(&want, &mut reached);
    let r = Reach::of(size, false, &want, &reached, 0);
    eprintln!("{text}\nmissing {:?}\ncut {:?}", r.missing, r.cut);
}

/// Declare the per-size reach tests for a scenario's cases.
#[macro_export]
macro_rules! reach_tests {
    ($app:literal, $s:ty, $cases:expr; $($test:ident = $name:literal,)*) => {
        const REACH_ALLOW: &str = "";

        $(
            mod $test {
                use super::*;

                fn at(size: (u16, u16)) {
                    $crate::reach::check::<$s>($app, &$cases, $name, size, REACH_ALLOW);
                }

                #[test]
                fn s40x12() {
                    at((40, 12));
                }

                #[test]
                fn s60x18() {
                    at((60, 18));
                }

                #[test]
                fn s72x20() {
                    at((72, 20));
                }

                #[test]
                fn s80x24() {
                    at((80, 24));
                }

                #[test]
                fn s100x25() {
                    at((100, 25));
                }

                #[test]
                fn s106x29() {
                    at((106, 29));
                }

                #[test]
                fn s120x40() {
                    at((120, 40));
                }

                #[test]
                fn s160x50() {
                    at((160, 50));
                }
            }
        )*

        #[test]
        fn every_case_has_its_test() {
            assert_eq!(
                $crate::reach::SIZES,
                [(40, 12), (60, 18), (72, 20), (80, 24), (100, 25), (106, 29), (120, 40), (160, 50)]
            );
            let tested = [$($name),*];
            for case in $cases {
                assert!(tested.contains(&case.name), "{} has no reach test", case.name);
            }
        }

        #[test]
        fn reach_dump() {
            $crate::reach::dump::<$s>(&$cases);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buf(lines: &[&str]) -> Buffer {
        let w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;
        let mut b = Buffer::empty(Rect::new(0, 0, w, lines.len() as u16 + 1));
        for (y, l) in lines.iter().enumerate() {
            b.set_string(0, y as u16, l, ratatui_core::style::Style::default());
        }
        b
    }

    #[test]
    fn words_split_cut_and_skip_the_footer() {
        let w = Words::read(&buf(&["alpha be gamma│delt…", "x 1234"]));
        assert!(w.whole.contains("alpha"));
        assert!(w.whole.contains("gamma"));
        assert!(!w.whole.contains("be"));
        assert!(w.cut.contains("delt"));
        assert!(w.whole.contains("1234"));
        // add_area honours its region, so a caller can skip the footer row.
        let b = buf(&["alpha delta", "footer row"]);
        let mut w = Words::default();
        w.add_area(&b, Rect::new(0, 0, 11, 1));
        assert!(
            w.whole.contains("alpha") && !w.whole.contains("footer"),
            "{w:?}"
        );
    }

    #[test]
    fn against_sorts_missing_from_cut() {
        let want = Words::read(&buf(&["alpha delta omega"]));
        let got = Words::read(&buf(&["alpha delt…"]));
        let (missing, cut) = want.against(&got);
        assert_eq!(missing.into_iter().collect::<Vec<_>>(), ["omega"]);
        assert_eq!(cut.into_iter().collect::<Vec<_>>(), ["delta"]);
    }
}
