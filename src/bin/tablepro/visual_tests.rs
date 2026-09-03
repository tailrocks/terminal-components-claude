//! Pre-refactor cell-exact visual digests for TablePro (WP-0).
//!
//! Every surface is reached by a deterministic key sequence through the real
//! `App` on a `TestBackend`; the whole buffer (symbol, fg, bg, modifier of
//! every cell) is folded into an FNV-1a digest, one line per surface, and
//! compared against `tests/baselines/tablepro.txt`. Regenerate only with
//! `UPDATE_BASELINE=1` after inspecting the change.

use ratatui::crossterm::event::KeyCode;

use crate::app::{Modal, Screen};
use crate::app_tests::H;
use crate::connections::ConnState;
use crate::workbench::WorkTab;

/// FNV-1a over every cell of the current frame. No rect is excluded.
fn digest(h: &H) -> u64 {
    let buf = h.term.backend().buffer();
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for cell in buf.content.iter() {
        let s = format!(
            "{}|{:?}|{:?}|{:?};",
            cell.symbol(),
            cell.fg,
            cell.bg,
            cell.modifier
        );
        for b in s.bytes() {
            hash ^= b as u64;
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
    }
    hash
}

fn down(h: &mut H, n: usize) {
    for _ in 0..n {
        h.key(KeyCode::Down);
    }
}

fn right(h: &mut H, n: usize) {
    for _ in 0..n {
        h.key(KeyCode::Right);
    }
}

/// Connected workbench with the `orders` table tab open and the grid focused.
fn table_tab(w: u16, hgt: u16) -> H {
    let mut h = H::connected(w, hgt);
    down(&mut h, 5);
    h.key(KeyCode::Enter);
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::Table(t)) if t.name == "orders"));
    h
}

/// Connected workbench with `sql` typed into the query editor (still in
/// insert mode).
fn query_typed(w: u16, hgt: u16, sql: &str) -> H {
    let mut h = H::connected(w, hgt);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Char('i'));
    h.type_str(sql);
    assert!(h.wb_query().editor.editing);
    h
}

/// Same as [`query_typed`] but back in navigation mode, ready to run.
fn query_nav(w: u16, hgt: u16, sql: &str) -> H {
    let mut h = query_typed(w, hgt, sql);
    // the first Esc only closes an open completion popup
    h.key(KeyCode::Esc);
    if h.wb_query().editor.editing {
        h.key(KeyCode::Esc);
    }
    assert!(!h.wb_query().editor.editing);
    h
}

type Builder = fn(u16, u16) -> H;

const SURFACES: &[(&str, Builder)] = &[
    ("connections", |w, h| {
        let hh = H::new(w, h);
        assert_eq!(hh.app.screen, Screen::Connections);
        hh
    }),
    ("connections-failed", |w, h| {
        let mut hh = H::new(w, h);
        let i = hh
            .app
            .connections
            .connections
            .iter()
            .position(|c| c.name == "Analytics")
            .unwrap();
        hh.app.connections.start_connect(i);
        hh.ticks(14);
        assert_eq!(hh.app.screen, Screen::Connections);
        assert!(
            matches!(hh.app.connections.state, ConnState::Failed { .. }),
            "{}",
            hh.text()
        );
        hh
    }),
    ("workbench-default", |w, h| {
        let hh = H::connected(w, h);
        assert_eq!(hh.app.screen, Screen::Workbench);
        hh
    }),
    ("explorer-focused", |w, h| {
        let mut hh = H::connected(w, h);
        assert_eq!(hh.focus(), Some(hh.wb().explorer.id));
        down(&mut hh, 2);
        hh
    }),
    ("table-grid", table_tab),
    ("grid-cell-editing", |w, h| {
        let mut hh = table_tab(w, h);
        right(&mut hh, 6);
        hh.key(KeyCode::Enter);
        hh.ctrl('l');
        hh.type_str("EUR");
        hh
    }),
    ("pending-change-bar", |w, h| {
        let mut hh = table_tab(w, h);
        right(&mut hh, 6);
        hh.key(KeyCode::Enter);
        hh.ctrl('l');
        hh.type_str("EUR");
        hh.key(KeyCode::Enter);
        assert!(hh.text().contains("• 1 pending"), "{}", hh.text());
        hh
    }),
    ("structure-view", |w, h| {
        let mut hh = table_tab(w, h);
        hh.ctrl('d');
        assert!(hh.text().contains("Columns"));
        hh
    }),
    ("query-editing", |w, h| {
        query_typed(w, h, "SELECT * FROM orders")
    }),
    ("completion-popup", |w, h| {
        let hh = query_typed(w, h, "SELECT * FROM ord");
        assert!(hh.wb_query().completion.is_open());
        hh
    }),
    ("results-grid", |w, h| {
        let mut hh = query_nav(w, h, "SELECT * FROM orders LIMIT 25");
        hh.ctrl('r');
        hh.ticks(10);
        assert!(!hh.wb_query().is_running());
        assert_eq!(hh.wb_query().results.len(), 1);
        hh
    }),
    ("error-result", |w, h| {
        let mut hh = query_nav(w, h, "SELECT nope FROM orders");
        hh.ctrl('r');
        hh.ticks(10);
        assert!(hh.text().contains("column \"nope\" does not exist"));
        hh
    }),
    ("explain-plan", |w, h| {
        let mut hh = query_nav(
            w,
            h,
            "SELECT * FROM orders WHERE notes LIKE '%gift%' ORDER BY created_at LIMIT 10",
        );
        hh.alt('x');
        hh.ticks(10);
        assert!(hh.text().contains("EXPLAIN ANALYZE"));
        hh
    }),
    ("history-tab", |w, h| {
        let mut hh = H::connected(w, h);
        hh.ctrl('y');
        assert!(matches!(hh.wb().active_tab(), Some(WorkTab::History(_))));
        hh
    }),
    ("quick-switcher", |w, h| {
        let mut hh = H::connected(w, h);
        hh.ctrl('o');
        hh.type_str("cust");
        assert!(matches!(hh.app.modal, Some(Modal::Picker(..))));
        hh
    }),
    ("tab-list-picker", |w, h| {
        let mut hh = table_tab(w, h);
        hh.ctrl('t');
        hh.ctrl('g');
        assert!(matches!(hh.app.modal, Some(Modal::Picker(..))));
        assert!(hh.text().contains("Open tabs"));
        hh
    }),
    ("safe-mode-picker", |w, h| {
        let mut hh = H::connected(w, h);
        hh.ctrl('l');
        assert!(hh.text().contains("Safe Mode · this connection"));
        hh
    }),
    ("filter-editor", |w, h| {
        let mut hh = table_tab(w, h);
        right(&mut hh, 4);
        hh.key(KeyCode::Char('f'));
        assert!(matches!(hh.app.modal, Some(Modal::Filter(_))));
        hh
    }),
    ("safety-dialog-typed-ack", |w, h| {
        let mut hh = query_nav(w, h, "DELETE FROM orders");
        hh.ctrl('r');
        assert!(matches!(hh.app.modal, Some(Modal::Dialog(_))));
        assert!(hh.text().contains("Type orders to confirm"));
        hh.key(KeyCode::Enter);
        hh.type_str("orders");
        hh
    }),
    ("help-dialog", |w, h| {
        let mut hh = H::connected(w, h);
        hh.key(KeyCode::Char('?'));
        assert!(
            matches!(hh.app.modal, Some(Modal::Dialog(_))),
            "{}",
            hh.text()
        );
        hh
    }),
    ("maximised-tab", |w, h| {
        let mut hh = table_tab(w, h);
        hh.key(KeyCode::Char('z'));
        assert!(hh.wb().maximized, "{}", hh.text());
        hh
    }),
];

#[test]
fn tablepro_visual_baseline() {
    use std::fmt::Write as _;
    let mut out = String::new();
    for (w, hgt) in [(120u16, 40u16), (80, 24)] {
        for (label, build) in SURFACES {
            let a = digest(&build(w, hgt));
            let b = digest(&build(w, hgt));
            assert_eq!(
                a, b,
                "{w}x{hgt} {label}: two builds of the same surface differ"
            );
            writeln!(out, "{w}x{hgt} {label} {a:016x}").unwrap();
        }
    }
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/baselines/tablepro.txt");
    if std::env::var_os("UPDATE_BASELINE").is_some() {
        std::fs::write(path, &out).unwrap();
        return;
    }
    let expected =
        std::fs::read_to_string(path).expect("baseline file; run with UPDATE_BASELINE=1");
    assert_eq!(
        out, expected,
        "tablepro rendering changed; inspect before updating the baseline"
    );
}
