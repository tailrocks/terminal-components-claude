//! Colour as meaning in the showcase: each page shows the library roles
//! (status kinds, environments, tones, links, syntax literals, meter
//! levels and staleness) through the library API, driven through the real
//! app (colour follow-ups, `docs/audits/design-review-2026-10`).

use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use junie_tui::theme::{Theme, contrast_ratio};
use ratatui::{
    Terminal,
    backend::TestBackend,
    buffer::Cell,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Position,
    style::Modifier,
};

use crate::app::{App, PageId};

struct Harness {
    app: App,
    term: Terminal<TestBackend>,
}

impl Harness {
    fn new(width: u16, height: u16, page: PageId) -> Self {
        let mut app = App::new(Theme::junie());
        app.goto(page);
        let mut term = Terminal::new(TestBackend::new(width, height)).unwrap();
        term.draw(|frame| app.render(frame)).unwrap();
        Self { app, term }
    }

    fn send(&mut self, input: Input) {
        self.app.handle(input);
        self.term.draw(|frame| self.app.render(frame)).unwrap();
    }

    fn key(&mut self, code: KeyCode) {
        self.send(Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        }));
    }

    fn tick(&mut self) {
        self.send(Input::Tick);
    }

    fn click(&mut self, x: u16, y: u16) {
        self.send(Input::Mouse(Mouse::new(
            MouseKind::Down,
            Position::new(x, y),
        )));
        self.send(Input::Mouse(Mouse::new(MouseKind::Up, Position::new(x, y))));
    }

    fn cell(&self, x: u16, y: u16) -> &Cell {
        &self.term.backend().buffer()[(x, y)]
    }

    fn row(&self, y: u16) -> String {
        let buffer = self.term.backend().buffer();
        (0..buffer.area.width)
            .map(|x| buffer[(x, y)].symbol().to_owned())
            .collect()
    }

    fn rows(&self) -> u16 {
        self.term.backend().buffer().area.height
    }

    /// Every cell position where `needle` starts, in columns.
    fn find_all(&self, needle: &str) -> Vec<(u16, u16)> {
        let mut out = Vec::new();
        for y in 0..self.rows() {
            let row = self.row(y);
            for (x, _) in row.match_indices(needle) {
                out.push((junie_tui::ui::text::width(&row[..x]) as u16, y));
            }
        }
        out
    }

    fn find(&self, needle: &str) -> (u16, u16) {
        *self
            .find_all(needle)
            .first()
            .unwrap_or_else(|| panic!("missing {needle:?}"))
    }

    fn click_label(&mut self, needle: &str) {
        let (x, y) = self.find(needle);
        self.click(x, y);
    }
}

fn reads(cell: &Cell) -> f64 {
    contrast_ratio(cell.fg, cell.bg).expect("measurable colours")
}

/// Task runner rows lead with their `Theme::status` glyph: queued `○`
/// muted, running the spinner with its label in info, done `✓` in success,
/// failed `✗` in error, cancelled `–` as skipped; the state word reads at
/// 4.5:1 on the card (skipped is faint, stepped up).
#[test]
fn taskrunner_states_take_the_status_roles() {
    let t = Theme::junie();
    let mut h = Harness::new(140, 45, PageId::TaskRunner);
    let (qx, qy) = h.find("queued");
    let glyph = h.cell(qx - 14, qy);
    assert_eq!(glyph.symbol(), t.glyphs.dot_off, "queued leads with ○");
    assert_eq!(glyph.fg, t.text_muted);
    assert!(reads(h.cell(qx, qy)) >= 4.5, "queued word reads");

    h.click_label("Run pipeline");
    h.tick();
    // the running row: the spinner glyph in the accent, then the name
    let (sx, sy) = (0..h.rows())
        .flat_map(|y| (0..140u16).map(move |x| (x, y)))
        .find(|&(x, y)| {
            t.glyphs.spinner.contains(&h.cell(x, y).symbol())
                && h.cell(x, y).fg == t.accent
                && h.cell(x + 2, y).symbol() != " "
        })
        .expect("a running task row");
    assert_eq!(h.cell(sx + 2, sy).fg, t.info, "running label in info");

    h.click_label("Cancel");
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    let (cx, cy) = h.find("cancelled");
    let dash = h.cell(cx - 14, cy);
    assert_eq!(dash.symbol(), t.glyphs.dash, "cancelled is skipped: –");
    assert!(reads(h.cell(cx, cy)) >= 4.5, "cancelled word reads");

    let mut h = Harness::new(140, 45, PageId::TaskRunner);
    h.click_label("Run pipeline");
    for _ in 0..300 {
        h.tick();
    }
    let done = h
        .find_all(&format!("{} compile", t.glyphs.done))
        .into_iter()
        .find(|&(_, y)| h.row(y).contains('%'))
        .expect("compile done");
    assert_eq!(h.cell(done.0, done.1).fg, t.success, "done is success ✓");
    let failed = h
        .find_all(&format!("{} integrat", t.glyphs.fail))
        .into_iter()
        .find(|&(_, y)| h.row(y).contains('%'))
        .expect("integration failed with ✗");
    let cell = h.cell(failed.0, failed.1);
    assert_eq!(cell.fg, t.error);
    assert!(cell.modifier.contains(Modifier::BOLD));
}

/// The progress page: running bars in the active tone, red only as a run
/// or a glyph (no red block fills), and the meter options: owner
/// thresholds, a fixed level, a stale domain warning and both stale
/// styles.
#[test]
fn progress_page_shows_meter_levels_and_staleness_without_red_fills() {
    let t = Theme::junie();
    let h = Harness::new(160, 50, PageId::Progress);
    let buffer = h.term.backend().buffer();
    let red_fills = buffer.content().iter().filter(|c| c.bg == t.error).count();
    assert_eq!(red_fills, 0, "no red block fills");

    let (x, y) = h.find("Halfway");
    let run = (x..x + 30)
        .find(|&cx| h.cell(cx, y).symbol() == t.glyphs.bar_fill)
        .expect("halfway run");
    assert_eq!(h.cell(run, y).fg, t.accent, "a running bar is active");

    // owner thresholds: 88% of a disk is attention, not danger
    let (x, y) = h.find("88% disk");
    let line = h.row(y);
    assert!(line.contains(&format!("critical at 95 {}", t.glyphs.warning)));
    let run = (0..x)
        .rev()
        .find(|&cx| h.cell(cx, y).symbol() == t.glyphs.bar_fill)
        .expect("disk run");
    assert_eq!(h.cell(run, y).fg, t.warning);

    // a stale domain warning keeps its hue and ▲, drawn dashed
    let (x, y) = h.find("82% · stale warning");
    let run = (0..x)
        .rev()
        .find(|&cx| h.cell(cx, y).symbol() == t.glyphs.bar_stale)
        .expect("dashed run");
    assert_eq!(h.cell(run, y).fg, t.warning);
    assert!(h.cell(x, y).modifier.contains(Modifier::ITALIC));

    // the default stale meter keeps its level; the faint style hides it
    let (_, y) = h.find("Stale     ");
    assert!(h.row(y).contains(t.glyphs.bar_stale));
    assert!(h.row(y).contains(&format!("76% used {}", t.glyphs.warning)));
    let (x, y) = h.find("faint stale style");
    let run = (0..x)
        .rev()
        .find(|&cx| h.cell(cx, y).symbol() == t.glyphs.bar_fill)
        .expect("faint run");
    assert_eq!(h.cell(run, y).fg, t.text_faint);
}

/// `true` and `false` in the code editor take the literal tone (lavender
/// italic, as numbers), not the keyword tone.
#[test]
fn editor_booleans_are_literals() {
    let t = Theme::junie();
    let h = Harness::new(140, 45, PageId::Editor);
    let (x, y) = h.find("gzip(true)");
    let cell = h.cell(x + 5, y);
    assert_eq!(cell.symbol(), "t");
    assert_eq!(cell.fg, t.syntax_literal);
    assert!(cell.modifier.contains(Modifier::ITALIC));
    assert!(!cell.modifier.contains(Modifier::BOLD), "not a keyword");
}

/// The Overview's Roles card draws every status kind, environment, tone,
/// syntax class and a link through the library, each readable on the
/// card.
#[test]
fn overview_roles_draw_statuses_environments_tones_code_and_link() {
    let t = Theme::junie();
    let h = Harness::new(160, 50, PageId::Overview);
    // the Roles card is in the left column; the Principles text on the
    // right also names some glyphs
    let (roles_x, roles_y) = h.find("Roles");
    let at = |needle: &str| {
        let (x, y) = h
            .find_all(needle)
            .into_iter()
            .find(|&(x, y)| x < roles_x + 46 && y > roles_y)
            .unwrap_or_else(|| panic!("{needle:?} in Roles"));
        h.cell(x, y).clone()
    };
    assert_eq!(at("✓ ok").fg, t.success);
    let fail = at("✗ failed");
    assert_eq!(fail.fg, t.error);
    assert!(fail.modifier.contains(Modifier::BOLD));
    assert_eq!(at("▲ warn").fg, t.warning);
    assert_eq!(at("⠋ running").fg, t.accent, "the spinner is active");
    let (x, y) = h.find("⠋ running");
    assert_eq!(h.cell(x + 2, y).fg, t.info, "running label is info");
    let (x, y) = h.find("– skipped");
    assert!(reads(h.cell(x + 2, y)) >= 4.5, "skipped label steps up");
    let prod = at("◆ production");
    assert_eq!(prod.fg, t.danger_label);
    assert!(prod.modifier.contains(Modifier::BOLD));
    assert_eq!(at("◇ staging").fg, t.warning);
    assert_eq!(at("· development").fg, t.info);
    let (x, y) = h.find("faint active");
    assert!(reads(h.cell(x, y)) >= 4.5, "faint steps up on the card");
    assert_eq!(h.cell(x + 6, y).fg, t.accent, "active is the accent");
    assert_eq!(at("danger").fg, t.danger_label);
    let link = at("docs/colour.md");
    assert_eq!(link.fg, t.link);
    let (x, y) = h.find("let ok = true");
    assert_eq!(h.cell(x, y).fg, t.syntax_keyword);
    let lit = h.cell(x + 9, y);
    assert_eq!(lit.symbol(), "t");
    assert_eq!(lit.fg, t.syntax_literal);
    assert!(lit.modifier.contains(Modifier::ITALIC));
}

/// The destructive dialog leads its title with the warning status glyph
/// (`Dialog::title_status`).
#[test]
fn destructive_dialog_title_leads_with_a_warning_status() {
    let t = Theme::junie();
    let mut h = Harness::new(120, 40, PageId::Dialogs);
    h.key(KeyCode::Char('d'));
    let (x, y) = h.find("Delete branch?");
    let glyph = h.cell(x - 2, y);
    assert_eq!(glyph.symbol(), t.glyphs.warning);
    assert_eq!(glyph.fg, t.legible(t.warning, glyph.bg));
}

/// List metadata takes its tone (`ListItem::meta_tone`): a new file in
/// success, a modified one in warning, both readable on the card.
#[test]
fn list_metadata_takes_its_change_tone() {
    let t = Theme::junie();
    let h = Harness::new(160, 50, PageId::Lists);
    let (x, y) = h.find("tests/checkout.rs");
    let row = h.row(y);
    let at = row[..]
        .find(" new")
        .map(|b| junie_tui::ui::text::width(&row[..b]) as u16 + 1);
    let cell = h.cell(at.expect("new meta"), y);
    assert_eq!(cell.fg, t.tone_on(junie_tui::theme::Tone::Success, cell.bg));
    let _ = x;
    let (_, y) = h.find("src/config.rs");
    let row = h.row(y);
    let b = row.find("modified").expect("modified meta");
    let cell = h.cell(junie_tui::ui::text::width(&row[..b]) as u16, y);
    assert_eq!(cell.fg, t.tone_on(junie_tui::theme::Tone::Warning, cell.bg));
    assert!(reads(cell) >= 4.5);
}

/// The task runner's deploy targets carry their environment: glyph and
/// tone from the library (`TreeNode::glyph_tone`), production's label in
/// the danger tone (`TreeNode::label_tone`).
#[test]
fn taskrunner_deploy_targets_carry_their_environment() {
    let t = Theme::junie();
    let h = Harness::new(140, 45, PageId::TaskRunner);
    let (x, y) = h.find("◆ production");
    let glyph = h.cell(x, y);
    assert_eq!(
        glyph.fg,
        t.tone_on(junie_tui::theme::Tone::Danger, glyph.bg)
    );
    let label = h.cell(x + 2, y);
    assert_eq!(
        label.fg,
        t.tone_on(junie_tui::theme::Tone::Danger, label.bg)
    );
    let (x, y) = h.find("◇ staging");
    assert_eq!(h.cell(x, y).fg, t.warning);
}

/// Toned lines on the Chrome page: hint bar status parts, a span line and
/// a footer status led by its `StatusKind` glyph.
#[test]
fn chrome_shows_toned_status_parts_spans_and_a_footer_status() {
    let t = Theme::junie();
    let h = Harness::new(120, 40, PageId::Chrome);
    let (x, y) = h.find("last: nothing yet");
    assert_ne!(
        h.cell(x, y).fg,
        h.cell(x + 6, y).fg,
        "the parts keep their own tones"
    );
    let (x, y) = h.find("1 failed");
    let cell = h.cell(x, y);
    assert_eq!(cell.fg, t.tone_on(junie_tui::theme::Tone::Error, cell.bg));
    assert!(cell.modifier.contains(Modifier::BOLD));
    let (x, y) = h.find("2 ok");
    assert_eq!(
        h.cell(x, y).fg,
        t.tone_on(junie_tui::theme::Tone::Success, h.cell(x, y).bg)
    );
    let (x, y) = h.find("✓ Saved");
    assert_eq!(h.cell(x, y).fg, t.success);
}
