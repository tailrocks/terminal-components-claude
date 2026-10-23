use showcase_demos::pages::{Page, diff::DiffPage};
use termrock::{App, Axis, ColorLevel, Cx, Id, KeyCode, Rect, Response, Theme, Ui};
use termrock_test_support::Harness;

const PAGE_TABS: Id = Id::root("showcase_ui::pages::diff.page_tabs");
const FILE_LIST: Id = Id::root("showcase_ui::pages::diff.files");
const LIVE_VIEW: Id = Id::root("showcase_ui::pages::diff.view");
const REVIEW: Id = Id::root("showcase_ui::pages::diff.review");
const EMPTY: Id = Id::root("showcase_ui::pages::diff.empty");
const MODE: Id = Id::root("showcase_ui::pages::diff.option.mode");
const FILES: Id = Id::root("showcase_ui::pages::diff.option.files");
const WIDTH: Id = Id::root("showcase_ui::pages::diff.option.width");
const STATES_SCROLL: Id = Id::root("showcase_ui::pages::diff.states_scroll");
const STATE_ROOT: Id = Id::root("showcase_ui::pages::diff.states");

struct PageHost {
    page: DiffPage,
    status: Option<String>,
}

impl App for PageHost {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let update = self.page.update(cx);
        self.status = update.status.map(|status| status.0);
        update.response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.page.draw(ui, ui.full());
    }
}

struct AreaHost {
    page: DiffPage,
    area: Rect,
}

impl App for AreaHost {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.page.update(cx).response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.page.draw(ui, self.area);
    }
}

fn host(width: u16, height: u16) -> Harness<PageHost> {
    Harness::new(
        PageHost {
            page: DiffPage::new(),
            status: None,
        },
        Theme::junie(),
        width,
        height,
    )
}

fn area_host(width: u16, height: u16, area: Rect) -> Harness<AreaHost> {
    Harness::new(
        AreaHost {
            page: DiffPage::new(),
            area,
        },
        Theme::junie(),
        width,
        height,
    )
}

fn section(h: &mut Harness<PageHost>, index: usize) {
    assert!(h.tab_to(PAGE_TABS));
    let key = char::from(b'1'.saturating_add(index as u8));
    let _ = h.key(KeyCode::Char(key));
}

fn choose_select(h: &mut Harness<PageHost>, id: Id, down: bool) {
    assert!(h.tab_to(id), "Select {id:?} is a live option");
    let _ = h.key(KeyCode::Enter);
    if down {
        let _ = h.key(KeyCode::Down);
    } else {
        let _ = h.key(KeyCode::Up);
    }
    let _ = h.key(KeyCode::Enter);
}

fn specimen_id(state: usize, mode: usize) -> Id {
    STATE_ROOT.index(state.saturating_mul(2).saturating_add(mode))
}

#[test]
fn four_real_changes_keep_canonical_status_counts_and_unicode_rows() {
    let mut h = host(120, 40);
    let mut text = h.text();
    assert!(text.contains("config/service.toml"), "{text}");
    assert!(text.contains("+10 −5"), "{text}");
    assert!(text.contains("[service.worker_0]"), "{text}");
    assert!(text.contains("attempts = 3"), "{text}");
    assert!(text.contains("backoff = \"exponential\""), "{text}");
    assert!(text.contains("東京"), "{text}");
    assert!(text.contains("cafe\u{301} ☕"), "{text}");
    assert!(
        text.contains("https://api.example.test/workers/health"),
        "{text}"
    );
    assert!(text.contains("+3 −0"), "{text}");
    assert!(text.contains("+0 −3"), "{text}");
    assert!(text.contains("+1 −1"), "{text}");

    assert!(h.tab_to(FILE_LIST));
    let _ = h.key(KeyCode::Down);
    text = h.text();
    assert!(text.contains("docs/retries.md"), "{text}");
    assert!(
        text.contains("Workers retry five times with exponential backoff."),
        "{text}"
    );

    let _ = h.key(KeyCode::Down);
    text = h.text();
    assert!(text.contains("src/legacy_backoff.rs"), "{text}");
    assert!(
        text.contains("pub fn linear(attempt: u32) -> u64 {"),
        "{text}"
    );

    let _ = h.key(KeyCode::Down);
    text = h.text();
    assert!(text.contains("src/backoff/retry.rs"), "{text}");
    assert!(
        text.contains("src/retry.rs → src/backoff/retry.rs"),
        "{text}"
    );
    assert!(text.contains("use super::exponential;"), "{text}");
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn review_empty_and_option_selects_stay_synchronized() {
    let mut h = host(120, 40);
    assert!(!h.text().contains(" · review"));
    assert!(h.tab_to(REVIEW));
    let _ = h.key(KeyCode::Char(' '));
    assert!(h.text().contains(" · review"), "{}", h.text());

    section(&mut h, 1);
    choose_select(&mut h, MODE, false);
    section(&mut h, 0);
    assert!(!h.text().contains(" · review"), "{}", h.text());

    assert!(h.tab_to(REVIEW));
    let _ = h.click_id(REVIEW);
    assert!(h.text().contains(" · review"), "{}", h.text());

    assert!(h.tab_to(EMPTY));
    let _ = h.key(KeyCode::Char(' '));
    assert!(h.text().contains("No file selected"), "{}", h.text());

    section(&mut h, 1);
    choose_select(&mut h, FILES, true);
    section(&mut h, 0);
    assert!(h.text().contains("No file selected"), "{}", h.text());
    assert!(h.tab_to(EMPTY));
    let _ = h.key(KeyCode::Char(' '));
    assert!(h.text().contains("No file selected"), "{}", h.text());

    section(&mut h, 1);
    choose_select(&mut h, FILES, false);
    section(&mut h, 0);
    assert!(h.text().contains("config/service.toml"), "{}", h.text());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn list_and_diff_follow_sixty_column_and_narrow_width_rules() {
    let wide = host(60, 24);
    let list = wide.area_of(FILE_LIST).expect("wide list");
    let diff = wide.area_of(LIVE_VIEW).expect("wide diff");
    assert_eq!(list.y, diff.y);
    assert!(diff.x >= list.right().saturating_add(2));

    let stacked = host(59, 24);
    let list = stacked.area_of(FILE_LIST).expect("stacked list");
    let diff = stacked.area_of(LIVE_VIEW).expect("stacked diff");
    assert_eq!(list.x, diff.x);
    assert!(diff.y >= list.bottom());

    let mut narrow = host(120, 40);
    section(&mut narrow, 1);
    choose_select(&mut narrow, WIDTH, true);
    section(&mut narrow, 0);
    let list = narrow.area_of(FILE_LIST).expect("narrow list");
    let diff = narrow.area_of(LIVE_VIEW).expect("narrow diff");
    assert_eq!(list.x, diff.x);
    assert!(diff.y >= list.bottom());
    assert!(list.right() <= 48);
    assert!(diff.right() <= 48);
    assert!(
        narrow.diagnostics().is_empty(),
        "{:?}",
        narrow.diagnostics()
    );
}

#[test]
fn shared_diff_view_scrolls_pans_and_captures_unicode_selection() {
    let mut h = host(120, 40);
    let expected = "cafe\u{301} ☕";
    let (x, y) = h.find("cafe").expect("Unicode config row is visible");
    let (cup_x, cup_y) = h.find("☕").expect("cup row is visible");
    assert_eq!(cup_y, y);
    let cup_width = termrock::width("☕").max(1);
    let end_x = cup_x.saturating_add(cup_width);
    let view = h.area_of(LIVE_VIEW).expect("live diff viewport");
    assert!(
        end_x < view.right(),
        "selection endpoint must stay inside the live view"
    );
    let _ = h.drag((x, y), (end_x, y));
    assert_eq!(h.focus(), Some(LIVE_VIEW));
    let _ = h.key(KeyCode::Char('y'));
    let status = format!("Captured selection: {expected}");
    assert_eq!(h.app().status.as_deref(), Some(status.as_str()));
    assert!(h.text().contains("Selection captured"), "{}", h.text());

    let mut pan = host(80, 24);
    assert!(pan.tab_to(LIVE_VIEW));
    let area = pan.area_of(LIVE_VIEW).expect("live diff viewport");
    let before = pan.text();
    let _ = pan.key(KeyCode::Right);
    let after_key = pan.text();
    assert_ne!(before, after_key, "right pans the overflowing logical row");
    let _ = pan.wheel(
        Axis::H,
        2,
        area.x.saturating_add(2),
        area.y.saturating_add(2),
    );
    assert_ne!(after_key, pan.text(), "horizontal wheel pans independently");
    let after_horizontal = pan.text();
    let _ = pan.wheel(
        Axis::V,
        2,
        area.x.saturating_add(2),
        area.y.saturating_add(3),
    );
    assert_ne!(
        after_horizontal,
        pan.text(),
        "vertical wheel still scrolls rows"
    );
    assert!(pan.diagnostics().is_empty(), "{:?}", pan.diagnostics());
}

#[test]
fn captured_selection_survives_idle_and_clears_only_for_changed_controls() {
    let mut h = host(120, 40);
    let (x, y) = h.find("attempts = 3").expect("removed row is visible");
    let _ = h.drag((x, y), (x.saturating_add(7), y));
    let _ = h.key(KeyCode::Char('y'));
    assert!(
        h.app()
            .status
            .as_deref()
            .is_some_and(|s| s.starts_with("Captured selection:"))
    );
    h.ticks(2);
    assert!(h.text().contains("Selection captured"), "{}", h.text());

    assert!(h.tab_to(FILE_LIST));
    let list = h.area_of(FILE_LIST).unwrap();
    let _ = h.click(list.x.saturating_add(2), list.y);
    assert!(
        h.text().contains("Selection captured"),
        "same-file click is a no-op\n{}",
        h.text()
    );
    let _ = h.key(KeyCode::Up);
    assert!(
        h.text().contains("Selection captured"),
        "boundary move is a no-op\n{}",
        h.text()
    );

    assert!(h.tab_to(REVIEW));
    let _ = h.click_id(REVIEW);
    assert!(
        !h.text().contains("Selection captured"),
        "mode change clears copy feedback\n{}",
        h.text()
    );
    let _ = h.key(KeyCode::Char('y'));
    assert!(
        h.app().status.is_none(),
        "cleared selection cannot be captured again"
    );

    let _ = h.tab_to(PAGE_TABS);
    let _ = h.key(KeyCode::Right);
    assert!(h.tab_to(WIDTH));
    let _ = h.key(KeyCode::Enter);
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Enter);
    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Left);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn gallery_uses_real_inert_diff_views_and_keeps_rows_reachable() {
    for color in [ColorLevel::TrueColor, ColorLevel::Mono] {
        let mut h = host(120, 40).with_color(color);
        section(&mut h, 2);
        let text = h.text();
        for label in ["Unified", "Review", "Rest", "Focused", "Empty", "Narrow"] {
            assert!(text.contains(label), "missing {label} at {color:?}\n{text}");
        }
        assert!(text.contains("No file selected"), "{color:?}\n{text}");
        assert!(text.contains("retry.toml"), "{color:?}\n{text}");
        assert!(
            h.diagnostics().is_empty(),
            "{color:?}: {:?}",
            h.diagnostics()
        );

        assert!(h.tab_to(PAGE_TABS));
        for _ in 0..16 {
            let _ = h.key(KeyCode::Tab);
            let focus = h.focus();
            for state in 0..4 {
                for mode in 0..2 {
                    assert_ne!(focus, Some(specimen_id(state, mode)), "gallery is inert");
                }
            }
        }
    }

    let mut short = host(80, 12);
    section(&mut short, 2);
    let scroll = short
        .area_of(STATES_SCROLL)
        .expect("state gallery scroll region");
    for _ in 0..16 {
        let _ = short.wheel(Axis::V, 3, scroll.x.saturating_add(1), scroll.y);
    }
    assert!(
        short.text().contains("Narrow"),
        "last state row is reachable\n{}",
        short.text()
    );
    assert!(short.diagnostics().is_empty(), "{:?}", short.diagnostics());
}

#[test]
fn page_clips_offset_and_tiny_areas_without_ghost_targets() {
    let page_area = Rect::new(7, 5, 54, 16);
    let mut h = area_host(80, 24, page_area);
    for id in [PAGE_TABS, REVIEW, EMPTY, FILE_LIST, LIVE_VIEW] {
        if let Some(area) = h.area_of(id) {
            assert!(
                area.x >= page_area.x && area.y >= page_area.y,
                "{id:?}: {area:?}"
            );
            assert!(area.right() <= page_area.right(), "{id:?}: {area:?}");
            assert!(area.bottom() <= page_area.bottom(), "{id:?}: {area:?}");
        }
    }
    let focus = h.focus();
    let _ = h.click(2, 2);
    assert_eq!(h.focus(), focus, "outside-page click has no owner");
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

    let tiny = host(1, 1);
    assert!(tiny.area_of(PAGE_TABS).is_none());
    assert!(tiny.area_of(FILE_LIST).is_none());
    assert!(tiny.area_of(LIVE_VIEW).is_none());
    assert!(tiny.diagnostics().is_empty(), "{:?}", tiny.diagnostics());

    let zero = area_host(40, 20, Rect::new(12, 8, 0, 0));
    assert!(zero.area_of(PAGE_TABS).is_none());
    assert!(zero.area_of(FILE_LIST).is_none());
    assert!(zero.diagnostics().is_empty(), "{:?}", zero.diagnostics());
}

#[test]
fn review_mode_remains_requested_while_narrow_content_falls_back_and_restores() {
    let mut h = host(120, 40);
    assert!(h.tab_to(REVIEW));
    let _ = h.key(KeyCode::Char(' '));
    assert!(h.text().contains(" · review"));

    section(&mut h, 1);
    choose_select(&mut h, WIDTH, true);
    section(&mut h, 0);
    assert!(
        !h.text().contains(" · review"),
        "narrow view uses the shared Unified fallback\n{}",
        h.text()
    );
    let diff = h.area_of(LIVE_VIEW).expect("narrow stacked diff");
    assert!(diff.width <= 48);

    section(&mut h, 1);
    choose_select(&mut h, WIDTH, false);
    section(&mut h, 0);
    assert!(
        h.text().contains(" · review"),
        "wider layout restores Review"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
