#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    missing_docs,
    unreachable_pub
)]

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

/// The row of the tree that carries `needle`, as a screen row.
fn tree_row(h: &Harness<App>, needle: &str) -> u16 {
    let (x, y) = h.find(needle).unwrap_or_else(|| {
        panic!("`{needle}` is not on the page:\n{}", h.text());
    });
    assert!(x > 0, "`{needle}` must sit in the tree");
    y
}

#[test]
fn a_click_on_a_painted_tree_row_selects_that_row() {
    let mut h = Harness::new(accounts_app(), Theme::junie(), 120, 40);
    // The group row "▾ Claude" is the second row of the tree. The hit region
    // must be that row, not the row below it.
    let row = tree_row(&h, "▾ Claude");
    let _ = h.click(8, row);
    assert_eq!(
        h.app().accounts.list.cursor(),
        Some(termrock::ItemKey::index(1)),
        "the click must land on the painted group row:\n{}",
        h.text()
    );
    // The account below the group keeps its own row.
    let row = tree_row(&h, "★ Personal");
    let _ = h.click(9, row);
    assert_eq!(
        h.app().accounts.list.cursor(),
        Some(termrock::ItemKey::index(2)),
        "the click must land on the painted account row:\n{}",
        h.text()
    );
}

#[test]
fn the_narrow_tree_scrolls_with_the_cursor() {
    let mut h = Harness::new(accounts_app(), Theme::junie(), 80, 24);
    // The narrow list shows eleven rows. The cursor walks past them.
    for _ in 0..20 {
        let _ = h.key(KeyCode::Down);
    }
    h.draw();
    let text = h.text();
    assert!(
        !text.contains("★ Personal"),
        "the list must scroll past the first rows:\n{text}"
    );
    assert!(
        text.contains("▾ Grok"),
        "the list must show the rows under the cursor:\n{text}"
    );
    // The range counts the rows the list shows, and never past the tree.
    assert!(text.contains("of 23"), "the range: {text}");
    let end = range_end(&text).unwrap_or_else(|| panic!("no range:\n{text}"));
    assert!(end <= 23, "the range must stop at the tree: {text}");
}

#[test]
fn the_range_indicator_is_clamped_to_the_tree() {
    let mut h = Harness::new(accounts_app(), Theme::junie(), 80, 24);
    for _ in 0..20 {
        let _ = h.key(KeyCode::Down);
    }
    h.draw();
    let text = h.text();
    // The tree holds twenty-three rows: the range ends inside the tree.
    let end = range_end(&text).unwrap_or_else(|| panic!("no range:\n{text}"));
    assert!(end <= 23, "the range must stop at the tree: {text}");
    assert!(
        end >= 11,
        "the range must start at the first visible row: {text}"
    );
}

/// The last row of the range the summary carries.
fn range_end(text: &str) -> Option<usize> {
    let at = text.find('–')?;
    let digits: String = text[at + '–'.len_utf8()..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

#[test]
fn a_committed_filter_keeps_the_query_and_gives_the_keys_back() {
    let mut h = Harness::new(accounts_app(), Theme::junie(), 120, 40);
    let _ = h.key(KeyCode::Char('/'));
    h.type_str("work");
    let _ = h.key(KeyCode::Enter);
    // The query stays and the tree follows it.
    assert_eq!(h.app().accounts.filter_query, "work");
    assert!(!h.app().accounts.filter_input.is_editing());
    h.draw();
    let text = h.text();
    assert!(text.contains("Accounts · filt"), "{text}");
    assert!(text.contains("▾ Claude"), "{text}");
    assert!(
        !text.contains("★ Primary"),
        "the filter must narrow the tree:\n{text}"
    );
    // The list takes the keys again: Down moves the cursor.
    let before = h.app().accounts.list.cursor();
    let _ = h.key(KeyCode::Down);
    assert_ne!(
        h.app().accounts.list.cursor(),
        before,
        "the list must be live after the commit"
    );
    assert_eq!(h.app().accounts.filter_query, "work");
}

#[test]
fn a_filtered_tree_drops_the_sentinel_and_the_add_row() {
    let mut h = Harness::new(accounts_app(), Theme::junie(), 120, 40);
    let _ = h.key(KeyCode::Char('/'));
    h.type_str("work");
    let _ = h.key(KeyCode::Enter);
    h.draw();
    let text = h.text();
    assert!(
        !text.contains("Unsupported"),
        "the sentinel row is not part of a query:\n{text}"
    );
    assert!(
        !text.contains("+ Add account…"),
        "the add row is not part of a query:\n{text}"
    );
}

#[test]
fn the_filtered_title_is_cut_to_the_painted_width() {
    let mut h = Harness::new(accounts_app(), Theme::junie(), 120, 40);
    let _ = h.key(KeyCode::Char('/'));
    h.type_str("work");
    let _ = h.key(KeyCode::Enter);
    h.draw();
    let text = h.text();
    // The title stops one column before the summary and marks the cut.
    assert!(text.contains("Accounts · filt…"), "{text}");
    assert!(
        !text.contains("Accounts · filter"),
        "the title must be cut to the box:\n{text}"
    );
    let cell = h.buffer().cell((19, 2)).expect("filter title cut");
    assert_eq!(cell.symbol(), "…", "{text}");
}

#[test]
fn the_account_keys_do_what_the_hint_bar_names() {
    let mut h = Harness::new(accounts_app(), Theme::junie(), 120, 40);
    for _ in 0..4 {
        let _ = h.key(KeyCode::Down);
    }
    h.draw();
    assert!(
        h.text().contains("t Disable"),
        "the hint bar names the toggle:\n{}",
        h.text()
    );
    // `t` toggles the account the cursor stands on.
    let _ = h.key(KeyCode::Char('t'));
    let account = h
        .app()
        .world
        .accounts
        .get("acct-claude-work")
        .expect("the work account");
    assert!(!account.enabled, "the toggle must disable the account");
    let _ = h.key(KeyCode::Char('t'));
    assert!(
        h.app()
            .world
            .accounts
            .get("acct-claude-work")
            .expect("the work account")
            .enabled,
        "the toggle must enable the account again"
    );
    // `d` asks for the removal the hint bar names.
    let _ = h.key(KeyCode::Char('d'));
    assert_eq!(
        h.app().accounts.remove_confirmation.as_deref(),
        Some("acct-claude-work"),
        "the remove key must ask for the removal"
    );
    let _ = h.key(KeyCode::Esc);
    // `m` goes to the usage of the accounts.
    let _ = h.key(KeyCode::Char('m'));
    assert_eq!(
        h.app().route(),
        Route::Usage,
        "the usage key must open usage"
    );
    let _ = h.key(KeyCode::Char('a'));
    // `e` opens the form on the account the cursor stands on.
    let mut h = Harness::new(accounts_app(), Theme::junie(), 120, 40);
    for _ in 0..4 {
        let _ = h.key(KeyCode::Down);
    }
    let _ = h.key(KeyCode::Char('e'));
    assert!(
        h.app().accounts.form_open,
        "the edit key must open the form"
    );
    assert_eq!(
        h.app().accounts.editing.as_deref(),
        Some("acct-claude-work"),
        "the form must edit the account under the cursor"
    );
    assert_eq!(h.app().accounts.draft_name, "Work");
}

#[test]
fn the_add_row_control_sits_on_the_painted_row() {
    let mut h = Harness::new(accounts_app(), Theme::junie(), 80, 24);
    // The tree scrolls to the end, so the add row is painted near the foot.
    for _ in 0..22 {
        let _ = h.key(KeyCode::Down);
    }
    h.draw();
    let text = h.text();
    assert!(text.contains("+ Add account…"), "{text}");
    let (x, y) = h.find("+ Add account…").expect("the add row");
    let area = h
        .area_of(jackin_preview_app::ACCOUNT_ADD)
        .expect("the add control");
    assert_eq!(
        area.y, y,
        "the add control must sit on the painted row: {area:?} at {x},{y}"
    );
    // A click on the add row opens the picker.
    let _ = h.click_id(jackin_preview_app::ACCOUNT_ADD);
    assert!(
        h.is_open(jackin_preview_app::ACCOUNT_PICKER),
        "the add row must open the account picker:\n{text}"
    );
}
