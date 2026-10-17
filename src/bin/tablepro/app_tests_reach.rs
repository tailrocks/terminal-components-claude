//! Reachability gate (owner rule 2026-10-17): every screen and overlay of
//! TablePro keeps all of its content reachable at every size of the
//! matrix: shown, scrolled to, or cut with its `…` reveal
//! ([`junie_tui::shell::reach`]). Known gaps are the `tablepro` lines of
//! `tests/reach_allow.txt`, each with its finding id
//! (docs/audits/design-review-2026-10/responsive.md); the gate fails on a
//! new gap and on a fixed one whose line is still there.
//!
//! `REACH_DUMP=<case>:<w>:<h>` prints one case's frame at a size
//! (`cargo nextest run --bin tablepro reach_dump --no-capture`).

use junie_tui::core::event::{Input, Key};
use junie_tui::runtime::Application;
use junie_tui::shell::reach::{Case, Harness, Reveal};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::app::App;
use crate::app_tests::H;
use crate::db::SafeMode;
use crate::workbench::WorkTab;

impl Harness for H {
    type App = App;

    fn fresh() -> Self {
        H::new(120, 40)
    }

    fn into_parts(self) -> (App, Terminal<TestBackend>) {
        (self.app, self.term)
    }

    fn from_parts(app: App, term: Terminal<TestBackend>) -> Self {
        H { app, term }
    }
}

const fn page(name: &'static str, base: fn(&mut H)) -> Case<H> {
    Case::page(name, base)
}

const fn layer(name: &'static str, base: fn(&mut H), open: fn(&mut H)) -> Case<H> {
    Case::layer(name, base, open)
}

fn connected(h: &mut H) {
    *h = H::connected(120, 40);
}

fn orders(h: &mut H) {
    connected(h);
    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    h.ticks(3);
}

fn shift_f10(h: &mut H) {
    h.app.handle(Input::Key(Key {
        code: KeyCode::F(10),
        mods: KeyModifiers::SHIFT,
    }));
    h.draw();
}

fn query(h: &mut H, sql: &str) {
    h.app.workbench.as_mut().unwrap().connection.safe_mode = SafeMode::Silent;
    h.ctrl('t');
    match h.app.workbench.as_mut().unwrap().active_tab_mut() {
        Some(WorkTab::Query(q)) => q.editor.set_text(sql),
        _ => panic!("expected a query tab"),
    }
    h.draw();
}

fn pending_edit(h: &mut H) {
    orders(h);
    for _ in 0..6 {
        h.key(KeyCode::Right);
    }
    h.key(KeyCode::Enter);
    h.cmd('a');
    h.type_str("EUR");
    h.key(KeyCode::Enter);
}

/// Below 100 columns the explorer is a drawer that follows focus: it
/// covers the tab while focused and steps aside when focus leaves.
const DRAWER: &[Reveal<H>] = &[
    Reveal {
        step: |h| {
            let id = h.wb().explorer.id;
            h.app.shell.focus.focus(id);
            h.draw();
        },
        times: 1,
    },
    Reveal {
        step: |h| {
            if let Some(id) = h.wb().primary_focus() {
                h.app.shell.focus.focus(id);
                h.draw();
            }
        },
        times: 1,
    },
];

/// A history row cut with `…` shows in full in the detail beside the
/// list as the cursor steps onto it.
const HISTORY: &[Reveal<H>] = &[
    DRAWER[0],
    DRAWER[1],
    Reveal {
        step: |h| {
            h.key(KeyCode::Down);
        },
        times: 15,
    },
];

fn cases() -> Vec<Case<H>> {
    let mut cases = vec![
        // the connections screen
        page("connections", |_| {}),
        page("connections-empty", |h| {
            h.app.connections.connections.clear();
            h.draw();
        }),
        page("conn-failed", |h| {
            let (x, y) = h.find("Staging").unwrap();
            h.click(x, y);
            h.key(KeyCode::Enter);
            h.ticks(12);
        }),
        page("conn-form-new", |h| {
            h.ctrl('n');
        }),
        page("conn-form-edit", |h| {
            let (x, y) = h.find("Local PostgreSQL").unwrap();
            h.click(x, y);
            h.key(KeyCode::Char('e'));
        }),
        page("conn-form-advanced", |h| {
            h.ctrl('n');
            h.key(KeyCode::BackTab);
            h.key(KeyCode::Right);
        }),
        page("conn-form-errors", |h| {
            h.ctrl('n');
            h.ctrl('s');
        }),
        layer(
            "conn-delete",
            |_| {},
            |h| {
                h.key(KeyCode::Char('d'));
            },
        ),
        layer(
            "conn-menu",
            |_| {},
            |h| {
                let id = h.app.connections.tree().id;
                h.app.shell.focus.focus(id);
                h.draw();
                shift_f10(h);
            },
        ),
        // the workbench
        page("workbench", connected),
        page("query-editing", |h| {
            connected(h);
            h.key(KeyCode::Tab);
        }),
        page("table-data", orders),
        page("table-structure", |h| {
            orders(h);
            h.ctrl('d');
        }),
        page("table-ddl", |h| {
            orders(h);
            h.ctrl('d');
            let id = match h.wb().active_tab() {
                Some(WorkTab::Table(t)) => t.structure_tabs.id,
                _ => panic!("expected a table tab"),
            };
            h.app.shell.focus.focus(id);
            h.draw();
            h.key(KeyCode::Char('6'));
        }),
        page("table-filtered", |h| {
            orders(h);
            for _ in 0..4 {
                h.key(KeyCode::Right);
            }
            h.key(KeyCode::Char('f'));
            h.key(KeyCode::Enter);
        }),
        page("table-pending", pending_edit),
        page("history", |h| {
            connected(h);
            h.ctrl('y');
        })
        .reveals(HISTORY),
        page("result-rows", |h| {
            connected(h);
            query(h, "SELECT * FROM orders");
            h.ctrl('r');
            h.ticks(10);
        }),
        page("result-error", |h| {
            connected(h);
            query(h, "SELECT nope FROM orders");
            h.ctrl('r');
            h.ticks(10);
        }),
        page("result-affected", |h| {
            connected(h);
            query(h, "DELETE FROM orders WHERE id = 1");
            h.ctrl('r');
            h.ticks(10);
        }),
        page("result-plan", |h| {
            connected(h);
            query(h, "SELECT * FROM orders WHERE status = 'paid' LIMIT 10");
            h.alt('x');
            h.ticks(10);
        }),
        page("result-multi", |h| {
            connected(h);
            query(
                h,
                "SELECT * FROM orders LIMIT 1; SELECT * FROM customers LIMIT 1",
            );
            h.alt('r');
            h.ticks(16);
        }),
        page("maximized", |h| {
            connected(h);
            h.key(KeyCode::Char('z'));
        }),
        page("explorer-hidden", |h| {
            connected(h);
            h.ctrl('b');
        }),
        // workbench overlays
        layer("help", connected, |h| {
            h.key(KeyCode::Char('?'));
        }),
        layer("open-quickly", connected, |h| {
            h.ctrl('o');
        }),
        layer("tab-list", connected, |h| {
            h.ctrl('g');
        }),
        layer("safe-mode", connected, |h| {
            h.ctrl('l');
        }),
        layer("filter-editor", orders, |h| {
            h.key(KeyCode::Char('f'));
        }),
        layer(
            "completion",
            |h| {
                connected(h);
                h.key(KeyCode::Tab);
            },
            |h| h.type_str("SELECT * FROM ord"),
        ),
        layer(
            "gate-typed",
            |h| {
                connected(h);
                h.key(KeyCode::Tab);
                h.type_str("DELETE FROM orders");
                h.key(KeyCode::Esc);
            },
            |h| {
                h.ctrl('r');
            },
        ),
        layer("commit", pending_edit, |h| {
            h.ctrl('s');
        }),
        layer("sql-preview", pending_edit, |h| {
            h.key(KeyCode::Char('p'));
        }),
        layer("discard", pending_edit, |h| {
            h.key(KeyCode::Char('U'));
        }),
        layer(
            "cell-viewer",
            |h| {
                orders(h);
                h.key(KeyCode::Home);
                for _ in 0..7 {
                    h.key(KeyCode::Right);
                }
            },
            |h| {
                h.key(KeyCode::Enter);
            },
        ),
        layer(
            "quit-confirm",
            |h| {
                connected(h);
                h.ctrl('t');
                h.type_str("SELECT 2");
                h.key(KeyCode::Esc);
            },
            |h| {
                h.ctrl('c');
            },
        ),
        layer(
            "grid-menu",
            |h| {
                orders(h);
                h.key(KeyCode::Down);
                h.key(KeyCode::Right);
            },
            shift_f10,
        ),
        layer(
            "explorer-menu",
            |h| {
                connected(h);
                for _ in 0..5 {
                    h.key(KeyCode::Down);
                }
            },
            shift_f10,
        ),
    ];
    // every workbench case reaches the explorer drawer and the tab under it
    for case in &mut cases {
        if !case.name.starts_with("conn") && case.reveals.is_empty() {
            case.reveals = DRAWER;
        }
    }
    cases
}

junie_tui::reach_tests! {
    "tablepro", H, cases();
    connections = "connections",
    connections_empty = "connections-empty",
    conn_failed = "conn-failed",
    conn_form_new = "conn-form-new",
    conn_form_edit = "conn-form-edit",
    conn_form_advanced = "conn-form-advanced",
    conn_form_errors = "conn-form-errors",
    conn_delete = "conn-delete",
    conn_menu = "conn-menu",
    workbench = "workbench",
    query_editing = "query-editing",
    table_data = "table-data",
    table_structure = "table-structure",
    table_ddl = "table-ddl",
    table_filtered = "table-filtered",
    table_pending = "table-pending",
    history = "history",
    result_rows = "result-rows",
    result_error = "result-error",
    result_affected = "result-affected",
    result_plan = "result-plan",
    result_multi = "result-multi",
    maximized = "maximized",
    explorer_hidden = "explorer-hidden",
    help = "help",
    open_quickly = "open-quickly",
    tab_list = "tab-list",
    safe_mode = "safe-mode",
    filter_editor = "filter-editor",
    completion = "completion",
    gate_typed = "gate-typed",
    commit = "commit",
    sql_preview = "sql-preview",
    discard = "discard",
    cell_viewer = "cell-viewer",
    quit_confirm = "quit-confirm",
    grid_menu = "grid-menu",
    explorer_menu = "explorer-menu",
}
