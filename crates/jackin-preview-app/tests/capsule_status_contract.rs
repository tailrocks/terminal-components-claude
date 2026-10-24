#![allow(
    dead_code,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    clippy::arithmetic_side_effects,
    clippy::doc_markdown,
    clippy::indexing_slicing,
    clippy::many_single_char_names,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::too_many_lines,
    clippy::unwrap_used,
    clippy::expect_used
)]

use jackin_preview_app::domain::usage::{Freshness, FreshnessInfo, QuotaStatus};
use jackin_preview_app::{Motion, Route, Scenario};
use support::H;
use termrock::{Color, KeyCode, Modifier, Position, Role, Surface, Theme};

mod support;

const FOOTER_Y: u16 = 28;
const METER_X: u16 = 82;

fn resolved(role: Role, surface: Surface) -> Color {
    termrock::theme::resolve::bind_role(&Theme::junie(), role, surface)
        .expect("Junie theme resolves the footer role")
}

fn set_session_percent(h: &mut H, account_id: &str, pct: u8) {
    let account = h
        .app_mut()
        .world
        .accounts
        .get_mut(account_id)
        .expect("fixture account exists");
    let session = account
        .usage
        .windows
        .iter_mut()
        .find(|window| window.id == "session")
        .expect("fixture session window exists");
    session.used_pct = Some(pct);
    session.status = QuotaStatus::from_pct(pct);
}

fn assert_footer_meter(
    h: &H,
    pct: u8,
    stale: bool,
    track_width: u16,
    filled_width: u16,
    marker: &str,
) {
    let text = h.text();
    let row = text.lines().nth(usize::from(FOOTER_Y)).unwrap_or("");
    assert!(row.contains("Session"), "footer row: {row:?}");
    assert!(row.contains(&format!("{pct}%")), "footer row: {row:?}");

    let elevated = resolved(Role::Surface(Surface::Elevated), Surface::Canvas);
    let warning = resolved(Role::Warning, Surface::Elevated);
    let muted = resolved(Role::Fg(termrock::FgStep::Muted), Surface::Elevated);

    for (x, ch) in (74..81).zip("Session".chars()) {
        let cell = h.harness.cell(x, FOOTER_Y);
        assert_eq!(cell.symbol(), ch.to_string(), "label cell x={x}");
        assert_eq!(cell.fg, muted, "label fg x={x}");
        assert_eq!(cell.bg, elevated, "label bg x={x}");
        assert!(!cell.modifier.contains(Modifier::BOLD), "label bold x={x}");
    }
    assert_eq!(h.harness.cell(81, FOOTER_Y).symbol(), " ");

    for x in METER_X..METER_X.saturating_add(track_width) {
        let offset = x.saturating_sub(METER_X);
        let filled = offset < filled_width;
        let cell = h.harness.cell(x, FOOTER_Y);
        assert_eq!(
            cell.symbol(),
            if filled && stale {
                "╍"
            } else if filled {
                "━"
            } else {
                "─"
            },
            "track glyph x={x}"
        );
        assert_eq!(
            cell.fg,
            if filled {
                warning
            } else {
                Color::Rgb(0x3a, 0x3a, 0x3a)
            }
        );
        assert_eq!(cell.bg, elevated, "track bg x={x}");
        assert!(
            !cell.modifier.contains(Modifier::ITALIC),
            "track italic x={x}"
        );
    }

    let value = format!("{pct}%");
    let value_x = METER_X.saturating_add(track_width).saturating_add(1);
    let text_color = if stale { muted } else { warning };
    let mut x = value_x;
    for ch in value.chars() {
        let cell = h.harness.cell(x, FOOTER_Y);
        assert_eq!(cell.symbol(), ch.to_string(), "value cell x={x}");
        assert_eq!(cell.fg, text_color, "value fg x={x}");
        assert_eq!(cell.bg, elevated, "value bg x={x}");
        assert_eq!(
            cell.modifier.contains(Modifier::ITALIC),
            stale,
            "value italic x={x}"
        );
        assert!(!cell.modifier.contains(Modifier::BOLD), "value bold x={x}");
        x = x.saturating_add(1);
    }
    assert_eq!(
        x, 97,
        "meter value end remains at the reviewed footer column"
    );

    let blank = h.harness.cell(97, FOOTER_Y);
    assert_eq!(blank.symbol(), " ");
    assert_eq!(blank.fg, warning);
    assert_eq!(blank.bg, elevated);
    let suffix = h.harness.cell(98, FOOTER_Y);
    assert_eq!(suffix.symbol(), marker);
    assert_eq!(suffix.fg, warning);
    assert_eq!(suffix.bg, elevated);
    assert!(!suffix.modifier.contains(Modifier::ITALIC));
    assert!(!suffix.modifier.contains(Modifier::BOLD));
}

fn assert_focused_account(h: &H, expected: &str) {
    let daemon = h
        .app()
        .world
        .daemons
        .get("jk-7f3a")
        .expect("reviewed running instance has a daemon");
    let tab = daemon
        .active_tab()
        .expect("reviewed daemon has an active tab");
    let pane = daemon
        .pane(tab.focused)
        .expect("focused pane is present in the daemon");
    assert_eq!(pane.proc.account.as_deref(), Some(expected));
}

#[test]
fn capsule_footer_uses_the_focused_session_model_and_stale_style() {
    let h = H::new(Scenario::OutroLast, Motion::Paused, 0, 100, 30);
    assert_eq!(h.app().route(), Route::Capsule);
    assert_eq!(
        h.app()
            .world
            .instance("jk-7f3a")
            .map(|instance| instance.status),
        Some(jackin_preview_app::domain::InstanceStatus::Running)
    );
    assert_focused_account(&h, "acct-claude-work");

    let account = h
        .app()
        .world
        .accounts
        .get("acct-claude-work")
        .expect("focused Claude account exists");
    let session = account
        .usage
        .windows
        .iter()
        .find(|window| window.id == "session")
        .expect("focused account has a Session window");
    assert_eq!(session.label, "Session · 5-hour");
    assert_eq!(
        session.category,
        jackin_preview_app::domain::WindowCategory::Session
    );
    assert_eq!(
        session.unit,
        jackin_preview_app::domain::WindowUnit::Percent
    );
    assert_eq!(session.used_pct, Some(76));
    assert_eq!(session.status, QuotaStatus::Warning);
    assert_eq!(account.usage.freshness.phase, Freshness::Stale);
    assert_eq!(
        h.app().world.now_secs()
            - account
                .usage
                .freshness
                .last_good_secs
                .expect("last-good timestamp"),
        47 * 60
    );

    assert_footer_meter(&h, 76, true, 11, 8, "▲");
    assert_eq!(h.harness.cursor(), Some(Position::new(3, 26)));
}

#[test]
fn capsule_footer_tracks_the_quota_thresholds() {
    let mut h = H::new(Scenario::OutroLast, Motion::Paused, 0, 100, 30);

    set_session_percent(&mut h, "acct-claude-work", 74);
    h.draw();
    assert_footer_meter(&h, 74, true, 11, 8, " ");

    set_session_percent(&mut h, "acct-claude-work", 75);
    h.draw();
    assert_footer_meter(&h, 75, true, 11, 8, "▲");

    set_session_percent(&mut h, "acct-claude-work", 100);
    h.draw();
    assert_footer_meter(&h, 100, true, 10, 10, "!");
}

#[test]
fn capsule_footer_ignores_nonfocused_account_until_real_pane_focus_changes() {
    let mut h = H::new(Scenario::OutroLast, Motion::Paused, 0, 100, 30);
    assert_focused_account(&h, "acct-claude-work");

    set_session_percent(&mut h, "acct-codex-primary", 99);
    h.draw();
    assert_focused_account(&h, "acct-claude-work");
    assert_footer_meter(&h, 76, true, 11, 8, "▲");

    h.click(70, 8);
    h.draw();
    assert_focused_account(&h, "acct-codex-primary");
    assert_footer_meter(&h, 99, false, 11, 11, "▲");
}

#[test]
fn capsule_footer_freshness_changes_presentation_without_changing_quota() {
    let mut h = H::new(Scenario::OutroLast, Motion::Paused, 0, 100, 30);
    let now = h.app().world.now_secs();

    h.app_mut()
        .world
        .accounts
        .get_mut("acct-claude-work")
        .expect("focused Claude account exists")
        .usage
        .freshness = FreshnessInfo::current(now);
    h.draw();
    assert_footer_meter(&h, 76, false, 11, 8, "▲");

    h.app_mut()
        .world
        .accounts
        .get_mut("acct-claude-work")
        .expect("focused Claude account exists")
        .usage
        .freshness = FreshnessInfo::stale(now - 47 * 60, now + 13 * 60);
    h.draw();
    assert_footer_meter(&h, 76, true, 11, 8, "▲");

    h.app_mut()
        .world
        .accounts
        .get_mut("acct-claude-work")
        .expect("focused Claude account exists")
        .usage
        .freshness = FreshnessInfo::failed(Some(now - 47 * 60), Some(now + 13 * 60));
    h.draw();
    assert_footer_meter(&h, 76, true, 11, 8, "▲");
}

#[test]
fn usage_detail_shows_the_same_session_window_and_stale_last_good_value() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Paused, 40, 100, 30);

    // The live detail host shows the first sorted account. Make Work the
    // Anthropic default in this isolated model so the ordinary Usage action
    // selects it, then exercise the real tab and Enter inputs below.
    h.app_mut()
        .world
        .accounts
        .get_mut("acct-claude-personal")
        .expect("personal account exists")
        .default_for_provider = false;
    h.app_mut()
        .world
        .accounts
        .get_mut("acct-claude-work")
        .expect("work account exists")
        .default_for_provider = true;

    h.key(KeyCode::Char('u'));
    assert_eq!(h.app().route(), Route::Usage);
    assert_eq!(h.app().usage.selected(), Some("acct-claude-work"));

    let account = h
        .app()
        .world
        .accounts
        .get("acct-claude-work")
        .expect("work account exists in the usage fixture");
    let session = account
        .usage
        .windows
        .iter()
        .find(|window| window.id == "session")
        .expect("work account has a Session window");
    assert_eq!(session.label, "Session · 5-hour");
    assert_eq!(session.used_pct, Some(76));
    assert_eq!(account.usage.freshness.phase, Freshness::Stale);
    assert_eq!(
        h.app().world.now_secs()
            - account
                .usage
                .freshness
                .last_good_secs
                .expect("last-good timestamp"),
        47 * 60
    );

    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);

    assert!(h.app().usage.detail_open());
    let detail = h.text();
    assert!(detail.contains("Session · 5-hour"), "{detail}");
    assert!(detail.contains("76%"), "{detail}");
    assert!(detail.contains("stale · last good 47 min ago"), "{detail}");
}
