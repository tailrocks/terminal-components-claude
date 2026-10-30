#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used, missing_docs)]

//! The accounts page is live: the tree, the inspector, and the drawer come
//! from the registry, and no screen size picks a different screen.

use jackin_preview_app::{App, Motion, Route, Scenario};
use termrock::{KeyCode, Theme};
use termrock_test_support::Harness;

fn accounts_app() -> App {
    App::for_scenario(Scenario::AccountsMixed, Motion::Paused)
}

#[test]
fn the_accounts_page_renders_the_live_registry_tree() {
    let app = accounts_app();
    let h = Harness::new(app, Theme::junie(), 120, 40);
    let text = h.text();
    // The tree is the registry: a provider group per provider, and one row
    // per account of that provider.
    for row in [
        "Overview",
        "▾ Claude",
        "★ Personal",
        "Archived contractor laptop",
        "Work",
        "▾ Codex",
        "★ Primary",
        "Experiments",
        "▾ Amp",
        "▾ Grok",
        "★ Team",
        "▾ Z.AI",
        "▾ Kimi",
        "▾ MiniMax",
        "▾ OpenCode",
        "★ Go subscription",
        "ci-bot",
        "Unsupported",
        "+ Add account…",
    ] {
        assert!(text.contains(row), "missing tree row `{row}`:\n{text}");
    }
    // The counts come from the registry summary.
    assert!(text.contains("12 · 4 ▲ · 4 !"), "{text}");
    assert!(text.contains("12 accounts · 8 providers"), "{text}");
    assert!(
        text.contains("12 accounts · 11 enabled · 1 disabled · 8 providers"),
        "{text}"
    );
    // The inspector is the registry summary, not a frozen string.
    assert!(
        text.contains("degraded · 4 warnings · 1 exhausted"),
        "{text}"
    );
    assert!(text.contains("7 registered · 5 discovered"), "{text}");
    // The health of the registry, read from the live accounts.
    assert!(text.contains("Claude · Work"), "{text}");
}

#[test]
fn the_accounts_page_is_the_same_page_at_every_size() {
    let app = accounts_app();
    let wide = Harness::new(app, Theme::junie(), 120, 40);
    let narrow = Harness::new(accounts_app(), Theme::junie(), 80, 24);
    for text in [wide.text(), narrow.text()] {
        for row in ["Overview", "▾ Claude", "★ Personal", "Work"] {
            assert!(
                text.contains(row),
                "missing `{row}` at another size:\n{text}"
            );
        }
        assert!(text.contains("12 · 4 ▲ · 4 !"), "{text}");
    }
    // The narrow page keeps the counts of the registry.
    assert!(
        narrow.text().contains("12 · 4 ▲ · 4 !"),
        "{}",
        narrow.text()
    );
}

#[test]
fn the_accounts_page_follows_the_live_selection() {
    let mut h = Harness::new(accounts_app(), Theme::junie(), 120, 40);
    assert_eq!(h.app().route(), Route::Accounts);
    // Four rows down land on the work account, and the page follows.
    for _ in 0..4 {
        let _ = h.key(KeyCode::Down);
    }
    h.draw();
    let text = h.text();
    assert!(
        text.contains("Accounts › Claude › Work"),
        "the crumb must follow the cursor:\n{text}"
    );
    assert!(text.contains("alexey@chainargos.com"), "{text}");
    assert!(text.contains("op://v_eng01/it_ant01/credential"), "{text}");
    // The tree marks the cursor row.
    assert!(text.contains("▎     Work"), "{text}");
}

#[test]
fn the_accounts_page_has_no_size_keyed_screen() {
    // The page must not choose a different screen by exact size, by frozen
    // theme identity, by paused motion, or by a fixture label.
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/app/accounts_page.rs"
    ))
    .unwrap_or_else(|error| panic!("accounts page source: {error}"));
    for needle in [
        "historical",
        "screen.width ==",
        "screen.height ==",
        "full.width ==",
        "full.height ==",
        "Motion::Paused",
        "Color::Rgb(0x48, 0xe0, 0x54)",
    ] {
        assert!(
            !source.contains(needle),
            "the accounts page must not key on `{needle}`"
        );
    }
    // The app must not keep a screen-selection guard for the accounts route.
    let app = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/app.rs"))
        .unwrap_or_else(|error| panic!("app source: {error}"));
    for needle in [
        "historical_accounts_filter",
        "historical_accounts_detail",
        "historical_accounts_drawer",
        "historical_audit_accounts",
        "draw_accounts_boot_needle",
    ] {
        assert!(
            !app.contains(needle),
            "the accounts route must not keep the `{needle}` bypass"
        );
    }
}
