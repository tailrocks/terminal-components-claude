//! Colour as meaning through the real app: environments, statuses, the
//! write gate, the primary action and SQL literals take the library's
//! roles (`Theme::env`, `Theme::status`, `Tone`, `SyntaxTone::Literal`),
//! never app-local colours (docs/audits/design-review-2026-10/
//! colour-follow-ups.md, tablepro).

use ratatui::buffer::Cell;
use ratatui::crossterm::event::KeyCode;
use ratatui::style::{Color, Modifier};

use junie_tui::theme::{ButtonKind, EnvKind, StatusKind, SyntaxTone, Theme, Tone};

use crate::app::Modal;
use crate::app_tests::H;
use crate::connections::ConnState;

fn theme() -> Theme {
    Theme::junie()
}

fn cell(h: &H, (x, y): (u16, u16)) -> Cell {
    h.term.backend().buffer()[(x, y)].clone()
}

/// The cell at the first occurrence of `needle`, offset by `dx` cells.
fn at(h: &H, needle: &str, dx: u16) -> Cell {
    let (x, y) = h
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} not on screen\n{}", h.text()));
    cell(h, (x + dx, y))
}

/// `tone` as drawn on the cell's own background.
fn ink(t: &Theme, tone: Tone, c: &Cell) -> Color {
    t.tone_on(tone, c.bg)
}

fn set_query(h: &mut H, sql: &str) {
    h.key(KeyCode::Tab);
    h.type_str(sql);
    h.key(KeyCode::Esc);
}

#[test]
fn strip_brand_is_the_accent_and_the_environment_badge_comes_from_theme_env() {
    let t = theme();
    let h = H::connected(120, 40);
    // the brand mark is live state: the accent as a foreground
    let brand = at(&h, "▪", 0);
    assert_eq!(brand.fg, ink(&t, Tone::Active, &brand), "brand ▪");
    // production: the danger label, bold, with its glyph
    let (glyph, style) = t.env(EnvKind::Production);
    let badge = format!("{glyph} production");
    let g = at(&h, &badge, 0);
    assert_eq!(g.fg, ink(&t, EnvKind::Production.tone(), &g));
    assert_eq!(Some(g.fg), style.fg.map(|c| t.legible(c, g.bg)));
    assert!(g.modifier.contains(Modifier::BOLD), "production is bold");
    let word = at(&h, &badge, 2);
    assert_eq!(word.fg, g.fg, "the word shares the badge tone");
}

#[test]
fn strip_environment_badges_take_each_environment_tone() {
    let t = theme();
    for (env, kind) in [
        (crate::db::Environment::Staging, EnvKind::Staging),
        (crate::db::Environment::Development, EnvKind::Development),
        (crate::db::Environment::Local, EnvKind::Local),
    ] {
        let mut h = H::connected(120, 40);
        h.app.workbench.as_mut().unwrap().connection.environment = env;
        h.draw();
        let c = at(&h, env.label(), 0);
        assert_eq!(c.fg, ink(&t, kind.tone(), &c), "{env:?}");
        let glyph = t.glyphs.env(kind);
        if !glyph.is_empty() {
            let g = at(&h, &format!("{glyph} {}", env.label()), 0);
            assert_eq!(g.fg, c.fg, "{env:?}: the glyph shares the tone");
        }
    }
}

#[test]
fn strip_safe_badge_is_a_passed_check_and_silent_production_a_warning() {
    let t = theme();
    let mut h = H::connected(120, 40);
    h.app.workbench.as_mut().unwrap().connection.safe_mode = crate::db::SafeMode::Safe;
    h.draw();
    let (ok, _) = t.status(StatusKind::Ok);
    let c = at(&h, &format!("{ok} safe"), 0);
    assert_eq!(
        c.fg,
        ink(&t, StatusKind::Ok.tone(), &c),
        "✓ safe in success"
    );
    h.app.workbench.as_mut().unwrap().connection.safe_mode = crate::db::SafeMode::Silent;
    h.draw();
    let (warn, _) = t.status(StatusKind::Warn);
    let c = at(&h, &format!("{warn} silent"), 0);
    assert_eq!(c.fg, ink(&t, Tone::Warning, &c), "silent on production");
    assert!(c.modifier.contains(Modifier::BOLD));
}

#[test]
fn connection_tree_and_detail_show_environments_in_their_tones() {
    let t = theme();
    let mut h = H::new(120, 40);
    // the tree's environment glyph: production's ◆ in the danger tone
    let (x, y) = h.find("Production").unwrap();
    let g = cell(&h, (x - 2, y));
    assert_eq!(g.symbol(), t.glyphs.env(EnvKind::Production));
    assert_eq!(g.fg, ink(&t, Tone::Danger, &g), "tree glyph");
    // select it: the detail card's Environment value follows
    h.click(x, y);
    let (ex, ey) = h.find("Environment").unwrap();
    let row: String = (ex..120)
        .map(|x| cell(&h, (x, ey)).symbol().to_owned())
        .collect();
    let vx = ex + row.find("production").expect("environment value") as u16;
    let v = cell(&h, (vx, ey));
    assert_eq!(v.fg, t.tone(Tone::Danger), "detail Environment value");
}

#[test]
fn ssl_on_is_a_passed_check_in_the_detail_card() {
    let t = theme();
    let mut h = H::new(120, 40);
    let i = h
        .app
        .connections
        .connections
        .iter()
        .position(|c| c.ssl)
        .expect("an SSL connection");
    let name = h.app.connections.connections[i].name.clone();
    let (x, y) = h.find(&name).unwrap();
    h.click(x, y);
    let (ok, style) = t.status(StatusKind::Ok);
    let c = at(&h, &format!("{ok} on"), 0);
    assert_eq!(c.fg, style.fg.unwrap(), "SSL on in success");
}

#[test]
fn test_connection_result_takes_the_status_glyph_and_tone() {
    let t = theme();
    for (name, kind) in [
        ("Local PostgreSQL", StatusKind::Ok),
        ("Analytics", StatusKind::Fail),
    ] {
        let mut h = H::new(120, 40);
        let (x, y) = h.find(name).unwrap();
        h.click(x, y);
        h.key(KeyCode::Char('e'));
        let (x, y) = h.find("Test connection").unwrap();
        h.click(x, y);
        h.ticks(10);
        assert!(matches!(h.app.connections.state, ConnState::Tested(_)));
        let (glyph, style) = t.status(kind);
        let (bx, by) = h.find("Save & connect").expect("footer");
        let row: Vec<Cell> = (bx..120).map(|x| cell(&h, (x, by))).collect();
        let g = row
            .iter()
            .find(|c| c.symbol() == glyph)
            .unwrap_or_else(|| panic!("{name}: {glyph} after the buttons\n{}", h.text()));
        assert_eq!(g.fg, style.fg.unwrap(), "{name}: glyph");
        assert_eq!(
            g.modifier.contains(Modifier::BOLD),
            style.add_modifier.contains(Modifier::BOLD)
        );
    }
}

#[test]
fn write_gate_on_production_warns_names_the_target_and_arms_a_danger_button() {
    let t = theme();
    let mut h = H::connected(120, 40);
    set_query(&mut h, "UPDATE orders SET status = 'paid' WHERE id = 'x'");
    h.ctrl('r');
    let Some(Modal::Dialog(d)) = h.app.modal() else {
        panic!("the write gate opens\n{}", h.text());
    };
    assert_eq!(
        d.actions[1].kind,
        ButtonKind::Danger,
        "Execute against production is a danger button"
    );
    // ▲ in warning before the title
    let (warn, ws) = t.status(StatusKind::Warn);
    let title = at(&h, &format!("{warn} Execute write query?"), 0);
    assert_eq!(title.fg, t.legible(ws.fg.unwrap(), title.bg), "title glyph");
    // the target line in production's tone
    let target = at(&h, "Production · production", 0);
    assert_eq!(target.fg, t.tone(EnvKind::Production.tone()), "Target");
}

#[test]
fn write_gate_off_production_keeps_a_plain_title_and_primary_execute() {
    let t = theme();
    let mut h = H::connected(120, 40);
    {
        let c = &mut h.app.workbench.as_mut().unwrap().connection;
        c.environment = crate::db::Environment::Staging;
        c.safe_mode = crate::db::SafeMode::Alert;
    }
    set_query(&mut h, "UPDATE orders SET status = 'paid' WHERE id = 'x'");
    h.ctrl('r');
    let Some(Modal::Dialog(d)) = h.app.modal() else {
        panic!("the write gate opens\n{}", h.text());
    };
    assert_eq!(d.actions[1].kind, ButtonKind::Primary);
    assert!(d.title_status.is_none());
    let (warn, _) = t.status(StatusKind::Warn);
    assert!(!h.text().contains(&format!("{warn} Execute")));
}

#[test]
fn footer_status_carries_its_kind() {
    let t = theme();
    let mut h = H::connected(120, 40);
    // connected: ✓ in success
    let (ok, os) = t.status(StatusKind::Ok);
    let c = at(&h, &format!("{ok} Connected to Production"), 0);
    assert_eq!(c.fg, os.fg.unwrap(), "connected");
    let label = at(&h, &format!("{ok} Connected to Production"), 2);
    assert_eq!(label.fg, t.status_label(StatusKind::Ok).fg.unwrap());
    // a cancelled query is information
    set_query(&mut h, "SELECT * FROM orders");
    h.ctrl('r');
    h.key(KeyCode::Esc);
    let (info, is) = t.status(StatusKind::Info);
    let c = at(&h, &format!("{info} Query cancelled"), 0);
    assert_eq!(c.fg, is.fg.unwrap(), "cancelled");
    // the Safe Mode block needs attention
    h.app.workbench.as_mut().unwrap().connection.safe_mode = crate::db::SafeMode::ReadOnly;
    h.key(KeyCode::Char('i'));
    h.cmd('a');
    h.type_str("DELETE FROM orders WHERE id = 'x'");
    h.key(KeyCode::Esc);
    h.ctrl('r');
    let (warn, ws) = t.status(StatusKind::Warn);
    let c = at(&h, &format!("{warn} Cannot execute write queries"), 0);
    assert_eq!(c.fg, ws.fg.unwrap(), "blocked by Safe Mode");
}

#[test]
fn statement_results_lead_with_their_status() {
    let t = theme();
    let mut h = H::connected(120, 40);
    set_query(&mut h, "SELECT * FROM orders LIMIT 3");
    h.ctrl('r');
    h.ticks(10);
    let (ok, os) = t.status(StatusKind::Ok);
    let c = at(&h, &format!("{ok} 3 rows"), 0);
    assert_eq!(c.fg, os.fg.unwrap(), "ok result");
    h.key(KeyCode::Char('i'));
    h.cmd('a');
    h.type_str("SELECT * FROM nope");
    h.key(KeyCode::Esc);
    h.ctrl('r');
    h.ticks(10);
    let (fail, fs) = t.status(StatusKind::Fail);
    let c = at(&h, &format!("{fail} failed"), 0);
    assert_eq!(c.fg, fs.fg.unwrap(), "failed result");
    assert!(c.modifier.contains(Modifier::BOLD));
    let label = at(&h, &format!("{fail} failed"), 2);
    assert_eq!(label.fg, t.status_label(StatusKind::Fail).fg.unwrap());
}

#[test]
fn toolbar_run_is_the_primary_action() {
    let mut h = H::connected(120, 40);
    h.key(KeyCode::Tab);
    let wb = h.wb();
    let Some(crate::workbench::WorkTab::Query(_)) = wb.active_tab() else {
        panic!("a query tab");
    };
    let t = theme();
    let run = at(&h, "Run ", 0);
    // the primary button paints the accent plate
    let plain_run_all = at(&h, "Run all", 0);
    assert_ne!(run.bg, plain_run_all.bg, "Run stands apart from Run all");
    assert_eq!(run.fg, t.text_on_accent, "Run is the primary button");
}

#[test]
fn explain_names_the_hot_node_in_warning() {
    let t = theme();
    let mut h = H::connected(120, 40);
    set_query(
        &mut h,
        "SELECT * FROM orders WHERE notes LIKE '%gift%' ORDER BY created_at LIMIT 10",
    );
    h.alt('x');
    h.ticks(10);
    let text = h.text();
    let (y, line) = text
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains(t.glyphs.warning) && l.contains(".."))
        .unwrap_or_else(|| panic!("a hot node\n{text}"));
    let first = line
        .char_indices()
        .find(|(_, c)| c.is_ascii_alphabetic())
        .map(|(i, _)| line[..i].chars().count() as u16)
        .unwrap();
    let c = cell(&h, (first, y as u16));
    assert_eq!(c.fg, ink(&t, Tone::Warning, &c), "hot node name: {line:?}");
}

#[test]
fn history_marks_slow_durations_in_warning() {
    let t = theme();
    let mut h = H::connected(120, 40);
    h.ctrl('y');
    // every connection's history: the seed's slow run is on another one
    h.key(KeyCode::Char('c'));
    let slow = "1.82 s";
    let c = at(&h, slow, 0);
    assert_eq!(c.fg, ink(&t, Tone::Warning, &c), "slow run in the list");
    let fast = at(&h, "38 ms", 0);
    assert_eq!(fast.fg, t.text_muted, "a quick run stays muted");
}

#[test]
fn sql_booleans_and_null_take_the_literal_hue() {
    let t = theme();
    let mut h = H::connected(120, 40);
    set_query(&mut h, "SELECT NULL, TRUE, FALSE FROM orders");
    let lit = t.syntax(SyntaxTone::Literal);
    for word in ["NULL", "TRUE", "FALSE"] {
        let c = at(&h, word, 0);
        assert_eq!(Some(c.fg), lit.fg, "{word}");
        assert!(c.modifier.contains(Modifier::ITALIC), "{word}");
    }
    let kw = at(&h, "SELECT NULL", 0);
    assert_eq!(Some(kw.fg), t.syntax(SyntaxTone::Keyword).fg, "SELECT");
}

/// Every text cell of `needle` (first occurrence) reads at 3:1 or more.
fn assert_reads(h: &H, needle: &str, why: &str) {
    let (x, y) = h
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} not on screen\n{}", h.text()));
    for dx in 0..needle.chars().count() as u16 {
        let c = cell(h, (x + dx, y));
        if c.symbol().trim().is_empty() {
            continue;
        }
        let r = junie_tui::theme::contrast_ratio(c.fg, c.bg).unwrap_or(21.0);
        assert!(
            r >= 3.0,
            "{why}: {:?} of {needle:?} is {:?} on {:?} ({r:.2}:1)",
            c.symbol(),
            c.fg,
            c.bg
        );
    }
}

#[test]
fn page_text_under_a_modal_dims_but_never_disappears_at_16_colours() {
    // 16 colours draw the selected tab Black on DarkGray; the backdrop
    // took that canvas ink for a blank and left it Black on Black
    let mut h = H::connected(120, 40);
    h.app.theme = Theme::for_level(junie_tui::theme::ColorLevel::Ansi16);
    h.draw();
    h.ctrl('o');
    assert!(matches!(h.app.modal(), Some(Modal::Picker(..))));
    assert_reads(&h, "Query 1", "selected tab under the picker");
}

#[test]
fn page_text_under_a_modal_dims_but_never_disappears() {
    // a button label on the raised plane dimmed to the ghost tier, the
    // colour of the plane itself (1.0:1)
    let mut h = H::new(160, 50);
    for _ in 0..7 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Char('d'));
    assert!(h.find("Delete connection?").is_some(), "{}", h.text());
    for label in ["Connect", "Edit", "Duplicate", "Delete…"] {
        assert_reads(&h, label, "detail button under the delete dialog");
    }
}
