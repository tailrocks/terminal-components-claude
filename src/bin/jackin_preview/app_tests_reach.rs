//! Reachability gate (owner rule 2026-10-17): every screen and overlay of
//! jackin-preview keeps all of its content reachable at every size of the
//! matrix: shown, scrolled to, or cut with its `…` reveal
//! ([`junie_tui::shell::reach`]). Known gaps are the `jackin` lines of
//! `tests/reach_allow.txt`, each with its finding id
//! (docs/audits/design-review-2026-10/responsive.md); the gate fails on a
//! new gap and on a fixed one whose line is still there.
//!
//! `REACH_DUMP=<case>:<w>:<h>` prints one case's frame at a size
//! (`cargo nextest run --bin jackin-preview reach_dump --no-capture`).

use junie_tui::core::event::{Input, Key};
use junie_tui::runtime::Application;
use junie_tui::shell::reach::{Case, Harness, Reveal};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::app::{App, Route};
use crate::app_tests::H;
use crate::scenario::{Motion, Scenario};
use crate::screens::Go;

impl Harness for H {
    type App = App;

    fn fresh() -> Self {
        H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40)
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

fn at(h: &mut H, scenario: Scenario) {
    *h = H::new(scenario, Motion::Reduced, 0, 120, 40);
}

fn shift_f10(h: &mut H) {
    h.app.handle(Input::Key(Key {
        code: KeyCode::F(10),
        mods: KeyModifiers::SHIFT,
    }));
    h.draw();
}

fn hard_manager(h: &mut H) {
    at(h, Scenario::HardCases);
    for _ in 0..8 {
        if h.app.route == Route::Manager {
            break;
        }
        h.ticks(3);
        h.key(KeyCode::Enter);
    }
}

fn editor(h: &mut H) {
    h.app.go(Go::Editor {
        workspace: Some(1),
        pending: None,
    });
    h.draw();
}

fn settings(h: &mut H) {
    h.key(KeyCode::Char('s'));
}

fn capsule(h: &mut H) {
    at(h, Scenario::CapsuleMulti);
}

fn cockpit(h: &mut H) {
    *h = H::new(Scenario::LaunchRunning, Motion::Paused, 110, 120, 40);
}

fn accounts(h: &mut H) {
    at(h, Scenario::AccountsMixed);
}

/// A list row cut with `…` shows in full in the detail beside the list as
/// the cursor steps onto it.
const ROWS: &[Reveal<H>] = &[Reveal {
    step: |h| {
        h.key(KeyCode::Down);
    },
    times: 16,
}];

fn cases() -> Vec<Case<H>> {
    vec![
        // routes and their panes
        page("intro", |h| {
            *h = H::new(Scenario::FirstUse, Motion::Reduced, 0, 120, 40);
        }),
        page("manager", |_| {}).reveals(ROWS),
        page("manager-expanded", |h| {
            h.key(KeyCode::Down);
            h.key(KeyCode::Right);
        }),
        page("manager-detail", |h| {
            h.key(KeyCode::Down);
            h.key(KeyCode::Tab);
        }),
        page("manager-hard", hard_manager).reveals(ROWS),
        page("prelude", |h| {
            h.app.go(Go::Prelude);
            h.draw();
        }),
        page("editor", editor),
        page("editor-mounts", |h| {
            editor(h);
            h.key(KeyCode::Char('2'));
        }),
        page("editor-roles", |h| {
            editor(h);
            h.key(KeyCode::Char('3'));
        }),
        page("editor-env", |h| {
            editor(h);
            h.key(KeyCode::Char('4'));
        }),
        page("editor-accounts", |h| {
            editor(h);
            h.key(KeyCode::Char('5'));
        }),
        page("settings", settings),
        page("settings-mounts", |h| {
            settings(h);
            h.key(KeyCode::Char('2'));
        }),
        page("settings-env", |h| {
            settings(h);
            h.key(KeyCode::Char('3'));
        }),
        page("settings-agents", |h| {
            settings(h);
            h.key(KeyCode::Char('4'));
        }),
        page("settings-trust", |h| {
            settings(h);
            h.key(KeyCode::Char('5'));
        }),
        page("accounts", accounts).reveals(ROWS),
        page("usage", |h| {
            h.key(KeyCode::Char('u'));
        })
        .reveals(ROWS),
        page("usage-detail", |h| {
            h.key(KeyCode::Char('u'));
            h.key(KeyCode::Down);
            h.key(KeyCode::Enter);
        }),
        page("cockpit", cockpit),
        page("cockpit-log", |h| {
            cockpit(h);
            h.key(KeyCode::Char('b'));
        }),
        page("cockpit-failure", |h| {
            at(h, Scenario::LaunchFailure);
            for _ in 0..20 {
                if h.text().contains("Launch failed") {
                    break;
                }
                h.ticks(3);
            }
        }),
        page("capsule", capsule),
        page("capsule-zoom", |h| {
            capsule(h);
            h.ctrl('b');
            h.key(KeyCode::Char('z'));
        }),
        // menus and dialogs
        layer(
            "help",
            |_| {},
            |h| {
                h.key(KeyCode::Char('?'));
            },
        ),
        layer(
            "menu-file",
            |_| {},
            |h| {
                h.key(KeyCode::F(10));
            },
        ),
        layer(
            "about",
            |_| {},
            |h| {
                let (x, y) = h.find("Help").expect("the Help menu");
                h.click(x, y);
                let (x, y) = h.find("About jackin-preview").expect("About");
                h.click(x, y);
            },
        ),
        layer(
            "launch-picker",
            |h| {
                h.key(KeyCode::Down);
            },
            |h| {
                h.key(KeyCode::Enter);
            },
        ),
        layer(
            "row-menu",
            |h| {
                for _ in 0..3 {
                    h.key(KeyCode::Down);
                }
            },
            shift_f10,
        ),
        layer(
            "delete",
            |h| {
                h.key(KeyCode::Down);
            },
            |h| {
                h.key(KeyCode::Char('d'));
            },
        ),
        layer(
            "prelude-source",
            |_| {},
            |h| {
                h.key(KeyCode::Char('n'));
            },
        ),
        layer("account-form", accounts, |h| {
            h.key(KeyCode::Char('a'));
        }),
        layer(
            "account-menu",
            |h| {
                accounts(h);
                h.key(KeyCode::Down);
            },
            shift_f10,
        ),
        layer(
            "env-form",
            |h| {
                h.key(KeyCode::Down);
                h.key(KeyCode::Char('e'));
                h.key(KeyCode::Char('4'));
                h.key(KeyCode::Enter);
            },
            |h| {
                h.key(KeyCode::Char('a'));
            },
        ),
        layer(
            "mount-form",
            |h| {
                editor(h);
                h.key(KeyCode::Char('2'));
                h.key(KeyCode::Enter);
            },
            |h| {
                h.key(KeyCode::Char('a'));
            },
        ),
        layer(
            "editor-preview",
            |h| {
                h.key(KeyCode::Down);
                h.key(KeyCode::Char('e'));
                h.key(KeyCode::Char(']'));
                h.key(KeyCode::Enter);
                h.key(KeyCode::Char('r'));
            },
            |h| {
                h.ctrl('s');
            },
        ),
        layer(
            "settings-preview",
            |h| {
                settings(h);
                h.key(KeyCode::Char('5'));
                h.key(KeyCode::Enter);
                h.key(KeyCode::Char(' '));
            },
            |h| {
                h.ctrl('s');
            },
        ),
        layer("cockpit-cancel", cockpit, |h| {
            h.key(KeyCode::Char('x'));
        }),
        layer("cockpit-info", cockpit, |h| {
            h.key(KeyCode::Char('i'));
        }),
        layer("capsule-palette", capsule, |h| {
            h.ctrl('\\');
        }),
        layer("capsule-menu", capsule, |h| {
            h.key(KeyCode::F(10));
        }),
        layer("capsule-agent", capsule, |h| {
            h.ctrl('b');
            h.key(KeyCode::Char('c'));
        }),
        layer("capsule-usage", capsule, |h| {
            h.ctrl('b');
            h.key(KeyCode::Char('u'));
        }),
        layer("capsule-info", capsule, |h| {
            h.ctrl('b');
            h.key(KeyCode::Char('i'));
        }),
        layer("capsule-exit", capsule, |h| {
            h.ctrl('q');
        }),
        layer("capsule-inspect", capsule, |h| {
            h.key(KeyCode::F(10));
            h.key(KeyCode::Right);
            h.key(KeyCode::Right);
            h.key(KeyCode::End);
            h.key(KeyCode::Enter);
        }),
    ]
}

junie_tui::reach_tests! {
    "jackin", H, cases();
    intro = "intro",
    manager = "manager",
    manager_expanded = "manager-expanded",
    manager_detail = "manager-detail",
    manager_hard = "manager-hard",
    prelude = "prelude",
    editor_general = "editor",
    editor_mounts = "editor-mounts",
    editor_roles = "editor-roles",
    editor_env = "editor-env",
    editor_accounts = "editor-accounts",
    settings_general = "settings",
    settings_mounts = "settings-mounts",
    settings_env = "settings-env",
    settings_agents = "settings-agents",
    settings_trust = "settings-trust",
    accounts_page = "accounts",
    usage = "usage",
    usage_detail = "usage-detail",
    cockpit_page = "cockpit",
    cockpit_log = "cockpit-log",
    cockpit_failure = "cockpit-failure",
    capsule_page = "capsule",
    capsule_zoom = "capsule-zoom",
    help = "help",
    menu_file = "menu-file",
    about = "about",
    launch_picker = "launch-picker",
    row_menu = "row-menu",
    delete = "delete",
    prelude_source = "prelude-source",
    account_form = "account-form",
    account_menu = "account-menu",
    env_form = "env-form",
    mount_form = "mount-form",
    editor_preview = "editor-preview",
    settings_preview = "settings-preview",
    cockpit_cancel = "cockpit-cancel",
    cockpit_info = "cockpit-info",
    capsule_palette = "capsule-palette",
    capsule_menu = "capsule-menu",
    capsule_agent = "capsule-agent",
    capsule_usage = "capsule-usage",
    capsule_info = "capsule-info",
    capsule_exit = "capsule-exit",
    capsule_inspect = "capsule-inspect",
}
