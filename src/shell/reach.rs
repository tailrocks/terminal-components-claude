//! Reachability audit: is everything an app shows at a roomy size still
//! reachable at a small one?
//!
//! The owner rule (2026-10-17): every screen keeps all of its content
//! reachable at any terminal size. When content cannot be shown in full,
//! a scrollbar appears and the keyboard, the wheel and a drag reach the
//! rest; text cut to fit ends in `…` and reveals its full value. A layout
//! that squeezes content away with no bar breaks the rule.
//!
//! A [`Case`] (a page, or an overlay over a page) of a real app on a test
//! backend ([`Harness`]) is drawn at a reference size (300×100) and its
//! words read; then it is drawn at the size under test, every scroll
//! container is paged to its ends on both axes and the case's
//! [`Reveal`]s are applied, collecting the words passed
//! ([`Case::run`]). A reference word never reached is
//! [`Reach::missing`]; one only reached as a prefix cut with `…` is
//! [`Reach::cut`] (shown, with its reveal). Apps' audit tests
//! (`app_tests_reach.rs` in each app, [`reach_tests!`](crate::reach_tests))
//! run every screen and overlay through it at the size matrix [`SIZES`]
//! against `tests/reach_allow.txt` ([`check`]);
//! docs/audits/design-review-2026-10/responsive.md records the findings.
//!
//! Words are runs of letters and digits of at least [`MIN_WORD`] cells,
//! read row by row; the footer row is left out on both sides (its hints
//! drop by priority at narrow widths and stay in the help overlay).

use std::collections::BTreeSet;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Position;

use super::app::App;
use crate::core::event::{Input, Mouse, MouseKind};
use crate::core::hit::{Axis, Bar, HitRegistry};
use crate::core::id::WidgetId;
use crate::runtime::Application;

/// The size matrix every app screen is audited at.
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

/// The reference size: big enough that no screen of the apps needs to
/// scroll to show its content.
const REFERENCE: (u16, u16) = (300, 100);

/// The shortest word compared (shorter runs are glyph noise: keys, units).
const MIN_WORD: usize = 3;

/// Pages tried per axis of a container before giving up on its end.
const MAX_PAGES: usize = 200;

/// Pages in a row that reach no new word before an axis counts as
/// explored (vertical, horizontal): rows past the reference's own
/// viewport hold no word it shows.
const STALL: (usize, usize) = (3, 12);

/// What [`Probe::audit`] found at one size.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Reach {
    /// The size under test.
    pub size: (u16, u16),
    /// The app drew its too-small screen at this size.
    pub too_small: bool,
    /// Reference words never reached at this size.
    pub missing: BTreeSet<String>,
    /// Reference words reached only cut with `…` (shown, with a reveal).
    pub cut: BTreeSet<String>,
    /// Scrollbars explored.
    pub bars: usize,
}

impl Reach {
    /// The finding at `size`: `want` against `reached`. A too-small
    /// screen reports nothing missing (it names the size it needs).
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

/// The words of `buf`, rows `0..rows`: whole words and the prefixes of
/// words cut with `…`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Words {
    pub whole: BTreeSet<String>,
    pub cut: BTreeSet<String>,
}

impl Words {
    /// Read the words of `buf` above the footer row.
    pub(crate) fn read(buf: &Buffer) -> Self {
        let mut out = Self::default();
        out.add(buf);
        out
    }

    /// Add the words of `buf` above the footer row.
    pub(crate) fn add(&mut self, buf: &Buffer) {
        let area = buf.area;
        let rows = area.height.saturating_sub(1);
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
                    // a wide glyph's trailing cell is an empty symbol
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

    /// Merge `other` in.
    pub(crate) fn extend(&mut self, other: Self) {
        self.whole.extend(other.whole);
        self.cut.extend(other.cut);
    }

    /// Compare a reference against what was reached.
    pub(crate) fn against(&self, reached: &Self) -> (BTreeSet<String>, BTreeSet<String>) {
        let mut missing = BTreeSet::new();
        let mut cut = BTreeSet::new();
        for w in &self.whole {
            if reached.whole.contains(w) {
                continue;
            }
            // reached whole inside a longer run (a word cut by a border at
            // the reference, or glued to a neighbour) counts as reached
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

/// A real app on a test backend, driven by input like a terminal would.
pub(crate) struct Probe<A> {
    pub(crate) app: A,
    pub(crate) term: Terminal<TestBackend>,
}

impl<A: App + Application> Probe<A> {
    /// Take over an app and its terminal (an app test harness's parts).
    pub(crate) fn new(app: A, term: Terminal<TestBackend>) -> Self {
        let mut p = Self { app, term };
        p.draw();
        p
    }

    pub(crate) fn draw(&mut self) {
        self.term
            .draw(|f| crate::shell::render(&mut self.app, f))
            .expect("test backend draws");
    }

    /// Route `input`, then draw a frame.
    pub(crate) fn send(&mut self, input: Input) {
        self.app.handle(input);
        self.draw();
    }

    pub(crate) fn resize(&mut self, w: u16, h: u16) {
        self.term.backend_mut().resize(w, h);
        self.send(Input::Resize(w, h));
    }

    pub(crate) fn buffer(&self) -> &Buffer {
        self.term.backend().buffer()
    }

    pub(crate) fn hits(&self) -> &HitRegistry {
        &self.app.shell().hits
    }

    /// The words of the current state at `size`.
    pub(crate) fn words_at(&mut self, size: (u16, u16)) -> Words {
        self.resize(size.0, size.1);
        Words::read(self.buffer())
    }

    /// The reference words of the current state: its words at `reference`
    /// less those in `except` (the page under an overlay, read before the
    /// overlay opened, and the pieces of them the overlay cuts) and the
    /// frame's own size (a status readout).
    pub(crate) fn reference(&mut self, reference: (u16, u16), except: &Words) -> Words {
        let mut want = self.words_at(reference);
        let dims = [reference.0.to_string(), reference.1.to_string()];
        want.whole
            .retain(|w| !dims.contains(w) && !except.whole.iter().any(|e| e.contains(w.as_str())));
        want.cut.clear();
        want
    }

    /// Add every word reachable from the current frame to `reached`: the
    /// frame's own, then each scroll container paged to its end on each
    /// of its axes (both axes together for a container with two bars),
    /// and the bars that appear at a container's end, one level down.
    /// Stops early once every word of `want` is reached. Returns the
    /// containers explored.
    pub(crate) fn explore(&mut self, want: &Words, reached: &mut Words) -> usize {
        // the items a strip collapsed into its `…` overflow item are
        // reached through its reveal, like text cut with `…`
        for hidden in crate::ui::reveal::overflowed() {
            for word in hidden.split(|c: char| !c.is_alphanumeric()) {
                if word.chars().count() >= MIN_WORD {
                    reached.cut.insert(word.to_owned());
                }
            }
        }
        let left = want.against(reached).0;
        let mut acc = Acc {
            reached: std::mem::take(reached),
            left,
            want: want.whole.clone(),
            last: BTreeSet::new(),
        };
        acc.add(self.buffer());
        let mut done: Vec<WidgetId> = Vec::new();
        self.explore_owners(&mut acc, &mut done, 0);
        *reached = acc.reached;
        done.len()
    }

    fn bars(&self) -> Vec<Bar> {
        let hits = self.hits();
        let mut seen: Vec<Bar> = Vec::new();
        for r in hits.regions() {
            if let Some(b) = hits.bar(r.id)
                && !seen.iter().any(|s| s.id == b.id)
            {
                seen.push(b);
            }
        }
        seen
    }

    /// Where a wheel over `bar`'s content lands: the middle of the region
    /// its owner registered when a wheel there reaches the owner, else the
    /// cell beside the middle of its track.
    fn wheel_spot(&self, bar: Bar) -> Position {
        let hits = self.hits();
        let reaches = |p: Position| hits.scroll_stack(p).contains(&bar.owner);
        let middle = hits
            .regions()
            .iter()
            .rev()
            .find(|r| r.owner == bar.owner && r.id == bar.owner && !r.hover_only)
            .map(|r| Position::new(r.area.x + r.area.width / 2, r.area.y + r.area.height / 2));
        if let Some(p) = middle.filter(|&p| reaches(p)) {
            return p;
        }
        let t = bar.track;
        match bar.axis {
            Axis::Vertical => Position::new(t.x.saturating_sub(1), t.y + t.height / 2),
            Axis::Horizontal => Position::new(t.x + t.width / 2, t.y.saturating_sub(1)),
        }
    }

    /// The bar of `owner` on `axis` in the current frame.
    fn bar_of(&self, owner: WidgetId, axis: Axis) -> Option<Bar> {
        self.bars()
            .into_iter()
            .find(|b| b.owner == owner && b.axis == axis)
    }

    fn explore_owners(&mut self, acc: &mut Acc, done: &mut Vec<WidgetId>, depth: u8) {
        for bar in self.bars() {
            if done.contains(&bar.owner) || acc.done() {
                continue;
            }
            let owner = bar.owner;
            done.push(owner);
            let v = self.bar_of(owner, Axis::Vertical).is_some();
            let h = self.bar_of(owner, Axis::Horizontal).is_some();
            // rows first; each row page swept across when there are two
            let mut v_pages = 0;
            let mut stalled = 0;
            loop {
                if h {
                    let pages = self.page_to_end(owner, Axis::Horizontal, true, acc);
                    self.page_back(owner, Axis::Horizontal, true, pages);
                }
                if !v || acc.done() || v_pages >= MAX_PAGES || stalled >= STALL.0 {
                    break;
                }
                if !self.page(owner, Axis::Vertical, true) {
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
            self.page_back(owner, Axis::Vertical, true, v_pages);
            // content that starts at its end (a log or terminal following
            // its tail) is reached backwards
            for axis in [Axis::Vertical, Axis::Horizontal] {
                if self.bar_of(owner, axis).is_some() && !acc.done() {
                    let pages = self.page_to_end(owner, axis, false, acc);
                    self.page_back(owner, axis, false, pages);
                }
            }
        }
    }

    /// Page `owner` along `axis` until the frame stops changing, every
    /// word is reached, or [`STALL`] pages in a row reach none; returns
    /// the pages that changed the frame.
    fn page_to_end(&mut self, owner: WidgetId, axis: Axis, forward: bool, acc: &mut Acc) -> usize {
        let limit = match axis {
            Axis::Vertical => STALL.0,
            Axis::Horizontal => STALL.1,
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

    /// Undo `pages` pages of `owner` along `axis` in one frame.
    fn page_back(&mut self, owner: WidgetId, axis: Axis, forward: bool, pages: usize) {
        if pages == 0 {
            return;
        }
        let Some(bar) = self.bar_of(owner, axis) else {
            return;
        };
        let kind = match (axis, forward) {
            (Axis::Vertical, true) => MouseKind::WheelUp,
            (Axis::Vertical, false) => MouseKind::WheelDown,
            (Axis::Horizontal, true) => MouseKind::WheelLeft,
            (Axis::Horizontal, false) => MouseKind::WheelRight,
        };
        let at = self.wheel_spot(bar);
        for _ in 0..pages * usize::from(notches(bar)) {
            self.app.handle(Input::Mouse(Mouse::new(kind, at)));
        }
        self.draw();
    }

    /// One page of wheel notches over `owner`'s content along `axis`
    /// (forward or back), drawn as one frame. Whether the frame changed.
    fn page(&mut self, owner: WidgetId, axis: Axis, forward: bool) -> bool {
        let Some(bar) = self.bar_of(owner, axis) else {
            return false;
        };
        let kind = match (axis, forward) {
            (Axis::Vertical, true) => MouseKind::WheelDown,
            (Axis::Vertical, false) => MouseKind::WheelUp,
            (Axis::Horizontal, true) => MouseKind::WheelRight,
            (Axis::Horizontal, false) => MouseKind::WheelLeft,
        };
        let before = self.buffer().clone();
        let at = self.wheel_spot(bar);
        for _ in 0..notches(bar) {
            self.app.handle(Input::Mouse(Mouse::new(kind, at)));
        }
        self.draw();
        *self.buffer() != before
    }
}

/// Wheel notches in one page along `bar`'s axis. A vertical notch scrolls
/// up to three rows, so a third of the track never skips past a viewport.
/// A horizontal notch may scroll a whole column (a grid), so a page is one
/// notch.
fn notches(bar: Bar) -> u16 {
    match bar.axis {
        // a short track pages notch by notch: a page never skips a row
        // the viewport could have shown (a stack scrolls by whole child)
        Axis::Vertical if bar.track.height < 9 => 1,
        Axis::Vertical => (bar.track.height / 3 - 1).max(1),
        Axis::Horizontal => 1,
    }
}

/// What an exploration has reached so far, and the words it still looks
/// for (checked against each new frame only).
struct Acc {
    reached: Words,
    left: BTreeSet<String>,
    /// Every reference word, reached or not.
    want: BTreeSet<String>,
    /// The words of the last frame.
    last: BTreeSet<String>,
}

impl Acc {
    /// Add a frame's words; whether it brought a reference word into view
    /// that the last frame did not show (rows past the reference's own
    /// viewport hold none, so paging through them stalls; rows already
    /// reached by an earlier exploration still count as progress).
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

/// The audit gate: every row of `found` (`scenario`, its [`Reach`]) of
/// `app` must miss exactly the words `allow` (the text of
/// `tests/reach_allow.txt`) records for it, no more (a regression) and no
/// fewer (a fixed finding whose line must go). Lines read
/// `<app> <scenario> <W>x<H> <word>… # <finding id> <reason>`.
///
/// Returns the lines that should replace `app`'s lines when they differ,
/// so a failing gate prints the exact update.
fn gate(app: &str, found: &[(String, Reach)], allow: &str) -> Result<(), String> {
    use std::collections::BTreeMap;
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
        // `<case>@macos` / `<case>@other`: a line for one client platform
        // (its hints, and so what fits, differ), over a plain line
        let (scenario, only) = match scenario.split_once('@') {
            Some((name, plat)) => (name, Some(plat)),
            None => (scenario, None),
        };
        let here = match crate::core::event::ClientPlatform::current() {
            crate::core::event::ClientPlatform::MacOs => "macos",
            crate::core::event::ClientPlatform::Other => "other",
        };
        if only.is_some_and(|p| p != here) {
            continue;
        }
        if only.is_none() && allowed.contains_key(&(scenario.to_string(), size.to_string())) {
            continue;
        }
        // a gate over some rows checks only their lines
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

/// An app's test harness, as the audit drives it: a fresh app at 120×40
/// and the app and its terminal it holds.
pub trait Harness: Sized {
    type App: App + Application;

    /// The app's usual start, at 120×40.
    fn fresh() -> Self;
    fn into_parts(self) -> (Self::App, Terminal<TestBackend>);
    fn from_parts(app: Self::App, term: Terminal<TestBackend>) -> Self;
}

/// One step of an audit case.
pub type Setup<H> = fn(&mut H);

/// A reveal: another way content is reached at a small size, applied to
/// the case at that size `times` times, exploring after each (a drawer
/// opened once; a cursor stepped down a list whose detail shows the
/// row in full).
pub struct Reveal<H> {
    pub step: Setup<H>,
    pub times: usize,
}

impl<H> Clone for Reveal<H> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<H> Copy for Reveal<H> {}

/// One audited screen or overlay: its page (`base`), the overlay opened
/// over it (if any) and its [`Reveal`]s.
pub struct Case<H: 'static> {
    pub name: &'static str,
    pub base: Setup<H>,
    pub overlay: Option<Setup<H>>,
    pub reveals: &'static [Reveal<H>],
}

impl<H> Case<H> {
    /// A page: the state `base` leaves the fresh app in.
    pub const fn page(name: &'static str, base: Setup<H>) -> Self {
        Self {
            name,
            base,
            overlay: None,
            reveals: &[],
        }
    }

    /// An overlay `open` opens over the page `base` leaves. The page's own
    /// words are left out of the overlay's reference.
    pub const fn layer(name: &'static str, base: Setup<H>, open: Setup<H>) -> Self {
        Self {
            name,
            base,
            overlay: Some(open),
            reveals: &[],
        }
    }

    /// The case with `reveals`.
    pub const fn reveals(mut self, reveals: &'static [Reveal<H>]) -> Self {
        self.reveals = reveals;
        self
    }
}

impl<H: Harness> Case<H> {
    /// The case's state at 120×40, and the page's reference words to leave
    /// out of an overlay's reference.
    fn build(&self) -> (H, Words) {
        let mut h = H::fresh();
        (self.base)(&mut h);
        let mut except = Words::default();
        if let Some(open) = self.overlay {
            let (app, term) = h.into_parts();
            let mut p = Probe::new(app, term);
            except = p.reference(REFERENCE, &Words::default());
            p.resize(120, 40);
            h = H::from_parts(p.app, p.term);
            open(&mut h);
        }
        (h, except)
    }

    /// The case at `size`: its reference words against those reached by
    /// scrolling, and by each of its reveals.
    pub fn run(&self, size: (u16, u16)) -> Reach {
        let (h, except) = self.build();
        let (app, term) = h.into_parts();
        let mut p = Probe::new(app, term);
        let want = p.reference(REFERENCE, &except);
        p.resize(size.0, size.1);
        let small = p.app.shell().too_small;
        let mut reached = Words::default();
        let mut bars = p.explore(&want, &mut reached);
        for reveal in self.reveals {
            if small || want.against(&reached).0.is_empty() {
                break;
            }
            let (h, _) = self.build();
            let (app, term) = h.into_parts();
            let mut p = Probe::new(app, term);
            p.resize(size.0, size.1);
            for _ in 0..reveal.times {
                let (app, term) = (p.app, p.term);
                let mut h = H::from_parts(app, term);
                (reveal.step)(&mut h);
                let (app, term) = h.into_parts();
                p = Probe::new(app, term);
                bars += p.explore(&want, &mut reached);
            }
        }
        Reach::of(size, small, &want, &reached, bars)
    }
}

/// Gate the case `name` of `cases` at `size` against the `app` lines of
/// `allow` ([`gate`]); panics with the update on a difference.
pub fn check<H: Harness>(app: &str, cases: &[Case<H>], name: &str, size: (u16, u16), allow: &str) {
    let case = cases
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("no case {name}"));
    let found = [(name.to_string(), case.run(size))];
    if let Err(e) = gate(app, &found, allow) {
        panic!("{e}");
    }
}

/// `REACH_DUMP=<case>:<w>:<h>`: print the case's frame at that size and
/// what the audit finds there (debugging aid; does nothing without it).
pub fn dump<H: Harness>(cases: &[Case<H>]) {
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
    let (h, _) = case.build();
    let (app, term) = h.into_parts();
    let mut p = Probe::new(app, term);
    p.resize(size.0, size.1);
    let buf = p.buffer();
    let mut text = String::new();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            text.push_str(buf[(x, y)].symbol());
        }
        text.push('\n');
    }
    let r = case.run(size);
    eprintln!("{text}\nmissing {:?}\ncut {:?}", r.missing, r.cut);
}

/// One `#[test]` per audit case and size (a module per case, a test per
/// size of [`SIZES`], so the matrix runs in parallel), a test that every
/// case has its module, and the `reach_dump` aid. In an app's
/// `app_tests_reach.rs`: `reach_tests!("app", H, cases(); mod_name =
/// "case-name", …)`, with `H:` [`Harness`] and `cases()` the
/// `Vec<Case<H>>`.
#[macro_export]
macro_rules! reach_tests {
    ($app:literal, $h:ty, $cases:expr; $($test:ident = $name:literal,)*) => {
        const REACH_ALLOW: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/reach_allow.txt"
        ));

        $(
            mod $test {
                use super::*;

                fn at(size: (u16, u16)) {
                    $crate::shell::reach::check::<$h>($app, &$cases, $name, size, REACH_ALLOW);
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
            // the per-size tests above are the matrix
            assert_eq!(
                $crate::shell::reach::SIZES,
                [(40, 12), (60, 18), (72, 20), (80, 24), (100, 25), (106, 29), (120, 40), (160, 50)]
            );
            let tested = [$($name),*];
            for case in $cases {
                assert!(tested.contains(&case.name), "{} has no reach test", case.name);
            }
        }

        #[test]
        fn reach_dump() {
            $crate::shell::reach::dump::<$h>(&$cases);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    fn buf(lines: &[&str]) -> Buffer {
        let w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;
        let mut b = Buffer::empty(Rect::new(0, 0, w, lines.len() as u16 + 1));
        for (y, l) in lines.iter().enumerate() {
            b.set_string(0, y as u16, l, ratatui::style::Style::default());
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
