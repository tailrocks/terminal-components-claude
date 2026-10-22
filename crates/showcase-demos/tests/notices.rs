use showcase_demos::pages::{Page, notices::NoticesPage};
use termrock::{
    App, Axis, Cx, FgStep, Id, ItemKey, KeyCode, Modifier, Part, PartRef, Response, Theme, Ui,
};
use termrock_test_support::Harness;

const PAGE_TABS: Id = Id::root("showcase_ui::pages::notices.page_tabs");
const PLAY_SCROLL: Id = Id::root("showcase_ui::pages::notices.play_scroll");
const INLINE_PANEL: Id = Id::root("showcase_ui::pages::notices.inline_panel");
const TEST_CONNECTION: Id = Id::root("showcase_ui::pages::notices.test_connection");
const KIND: Id = Id::root("showcase_ui::pages::notices.option.kind");
const DETAIL_LINES: Id = Id::root("showcase_ui::pages::notices.option.detail_lines");
const MESSAGE_LINES: Id = Id::root("showcase_ui::pages::notices.option.message_lines");
const DETAIL_TONE: Id = Id::root("showcase_ui::pages::notices.option.detail_tone");
const WIDTH: Id = Id::root("showcase_ui::pages::notices.option.width");
const DETAIL: Id = Id::root("showcase_ui::pages::notices.option.detail");
const HINT: Id = Id::root("showcase_ui::pages::notices.option.hint");
const MARK_CUT: Id = Id::root("showcase_ui::pages::notices.option.mark_cut");

struct PageHost {
    page: NoticesPage,
}

impl App for PageHost {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.page.update(cx).response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.page.draw(ui, ui.full());
    }
}

fn host(width: u16, height: u16) -> Harness<PageHost> {
    Harness::new(
        PageHost {
            page: NoticesPage::new(),
        },
        Theme::junie(),
        width,
        height,
    )
}

fn page_debug(h: &Harness<PageHost>) -> String {
    format!("{:?}", h.app().page)
}

fn select_index(h: &mut Harness<PageHost>, id: Id, index: usize) {
    assert!(h.tab_to(id), "select {id:?} participates in keyboard focus");
    let _ = h.key(KeyCode::Enter);
    for _ in 0..index {
        let _ = h.key(KeyCode::Down);
    }
    let _ = h.key(KeyCode::Enter);
}

fn move_select(h: &mut Harness<PageHost>, id: Id, direction: KeyCode, steps: usize) {
    assert!(h.tab_to(id), "select {id:?} participates in keyboard focus");
    let _ = h.key(KeyCode::Enter);
    for _ in 0..steps {
        let _ = h.key(direction);
    }
    let _ = h.key(KeyCode::Enter);
}

#[test]
fn real_connection_query_and_four_inline_kinds_render_at_both_shell_sizes() {
    for (width, height) in [(120, 40), (80, 24)] {
        let mut h = host(width, height);
        assert!(
            h.text().contains("prod-replica"),
            "{width}x{height}\n{}",
            h.text()
        );
        assert!(h.text().contains("Authentication failed"));
        assert!(h.area_of(TEST_CONNECTION).is_some());
        assert!(h.area_of(PLAY_SCROLL).is_some());

        if width == 120 {
            assert!(h.text().contains("Result 1"), "{}", h.text());
            assert!(h.text().contains("New connection"), "{}", h.text());
            assert!(h.text().contains("Port must be a number"));
            assert!(h.text().contains("3 rows changed outside this session"));
            assert!(h.text().contains("Read-only replica: edits are disabled"));
            assert!(h.text().contains("Saved 12 rows in 84 ms"));

            for marker in ["!", "▲", "i", "✓"] {
                assert!(
                    h.text().contains(marker),
                    "missing marker {marker}\n{}",
                    h.text()
                );
            }
            let (message_x, message_y) = h
                .find("Authentication failed")
                .expect("the live connection notice is drawn");
            let marker = h.cell(message_x.saturating_sub(2), message_y);
            assert_eq!(marker.symbol(), "!");
            assert!(marker.modifier.contains(Modifier::BOLD));
            let detail_row = h.find_row("FATAL: password authentication").unwrap();
            let (message_start, _) = h.find("Authentication failed").unwrap();
            let (detail_start, _) = h.find("FATAL: password authentication").unwrap();
            assert_eq!(
                detail_start, message_start,
                "supporting rows keep the two-column inset"
            );
            assert!(detail_row > message_y);
        } else {
            for _ in 0..12 {
                if h.text().contains("New connection") {
                    break;
                }
                let area = h.area_of(PLAY_SCROLL).unwrap();
                let _ = h.wheel(Axis::V, 3, area.x.saturating_add(2), area.y + 1);
            }
            assert!(h.text().contains("Result 1"), "{}", h.text());
            assert!(h.text().contains("New connection"), "{}", h.text());
            let scroll = h.area_of(PLAY_SCROLL).unwrap();
            let inline = h
                .area_of(INLINE_PANEL)
                .expect("the scrolled inline card remains registered");
            assert!(inline.x >= scroll.x && inline.right() <= scroll.right());
            assert!(inline.y >= scroll.y && inline.bottom() <= scroll.bottom());
        }
        assert!(
            h.diagnostics().is_empty(),
            "{width}x{height}: {:?}",
            h.diagnostics()
        );
    }
}

#[test]
fn every_notice_option_is_a_live_control_and_changes_the_rendered_notices() {
    let mut h = host(120, 40);
    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Right);

    for id in [KIND, DETAIL_LINES, MESSAGE_LINES, DETAIL_TONE, WIDTH] {
        assert!(h.area_of(id).is_some(), "option {id:?} is drawn");
    }
    for id in [DETAIL, HINT, MARK_CUT] {
        assert!(h.area_of(id).is_some(), "option {id:?} is drawn");
    }

    select_index(&mut h, KIND, 3);
    select_index(&mut h, DETAIL_LINES, 1);
    select_index(&mut h, MESSAGE_LINES, 1);
    select_index(&mut h, DETAIL_TONE, 1);
    select_index(&mut h, WIDTH, 1);
    for id in [DETAIL, HINT, MARK_CUT] {
        assert!(
            h.tab_to(id),
            "checkbox {id:?} participates in keyboard focus"
        );
        let _ = h.key(KeyCode::Char(' '));
    }

    let state = page_debug(&h);
    assert!(state.contains("kind: 3"), "{state}");
    assert!(state.contains("detail_lines: 1"), "{state}");
    assert!(state.contains("message_lines: 1"), "{state}");
    assert!(state.contains("detail_tone: 1"), "{state}");
    assert!(state.contains("width: 1"), "{state}");
    assert!(state.contains("show_detail: false"), "{state}");
    assert!(state.contains("show_hint: false"), "{state}");
    assert!(state.contains("mark_cut: false"), "{state}");

    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Left);
    assert!(
        h.text().contains("✓"),
        "the query Notice uses the selected kind\n{}",
        h.text()
    );
    assert!(
        !h.text().contains("LINE 1:"),
        "hidden detail stays suppressed\n{}",
        h.text()
    );
    assert!(!h.text().contains("Inspect the statement detail"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn detail_options_change_real_notice_rows_and_colors() {
    let mut h = host(120, 60);
    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Right);
    select_index(&mut h, DETAIL_LINES, 2);
    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Left);
    let wide_line = h.find("LINE 1:").unwrap().1;
    let wide_order = h.find("ORDER BY").unwrap().1;
    let wide_detail_rows = wide_order - wide_line;

    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Right);
    select_index(&mut h, WIDTH, 1);
    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Left);
    let narrow_line = h.find("LINE 1:").unwrap().1;
    let narrow_order = h.find("ORDER BY").unwrap().1;
    assert!(
        narrow_order - narrow_line > wide_detail_rows,
        "Narrow uses more real Notice rows than Wide: wide delta={wide_detail_rows}, narrow delta={}\n{}",
        narrow_order - narrow_line,
        h.text()
    );

    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Right);
    move_select(&mut h, DETAIL_LINES, KeyCode::Up, 1);
    select_index(&mut h, DETAIL_TONE, 1);
    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Left);

    assert!(h.find("LINE 1:").is_some(), "{}", h.text());
    assert!(
        h.find("ORDER BY").is_none(),
        "one detail row must not expose later SQL lines\n{}",
        h.text()
    );
    let (detail_x, detail_y) = h.find("LINE 1:").unwrap();
    assert_eq!(
        h.cell(detail_x, detail_y).fg,
        Theme::junie().color.fg[FgStep::Muted.index()],
        "Muted is applied to the real SQL detail row"
    );
    assert_ne!(
        h.cell(detail_x, detail_y).fg,
        Theme::junie().color.fg[FgStep::Secondary.index()],
        "the selected Muted tone differs from the default Secondary tone"
    );
    assert!(
        h.row(detail_y).contains("…"),
        "the one-row detail cap marks discarded SQL\n{}",
        h.row(detail_y)
    );

    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Right);
    move_select(&mut h, DETAIL_LINES, KeyCode::Down, 1);
    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Left);
    assert!(
        h.find("ORDER BY").is_some(),
        "four detail rows expose the later SQL clause\n{}",
        h.text()
    );

    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Right);
    move_select(&mut h, DETAIL_LINES, KeyCode::Up, 1);
    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Left);
    let detail_y = h.find("LINE 1:").unwrap().1;
    assert!(h.row(detail_y).contains("…"), "{}", h.row(detail_y));

    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Right);
    assert!(h.tab_to(MARK_CUT));
    let _ = h.key(KeyCode::Char(' '));
    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Left);
    let (detail_x, detail_y) = h.find("LINE 1:").unwrap();
    assert!(
        !h.row(detail_y).contains("…"),
        "turning off the cut marker preserves the detail text without an ellipsis\n{}",
        h.row(detail_y)
    );
    assert_eq!(
        h.cell(detail_x, detail_y).fg,
        Theme::junie().color.fg[FgStep::Muted.index()]
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn message_line_cap_changes_wrapped_inline_notice_height() {
    fn next_notice_row(message_lines: usize) -> u16 {
        let mut h = host(40, 100);
        assert!(h.tab_to(PAGE_TABS));
        let _ = h.key(KeyCode::Right);
        if message_lines == 1 {
            select_index(&mut h, MESSAGE_LINES, 1);
        }
        assert!(h.tab_to(PAGE_TABS));
        let _ = h.key(KeyCode::Left);
        h.find_row("3 rows changed")
            .expect("the warning follows the wrapped port-error message")
    }

    let two_line_cap = next_notice_row(2);
    let one_line_cap = next_notice_row(1);
    assert_eq!(
        two_line_cap,
        one_line_cap + 1,
        "the real port-error message wraps to a second row only when allowed"
    );
}

#[test]
fn select_rows_and_checkboxes_accept_pointer_changes() {
    let mut h = host(120, 40);
    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Right);

    let _ = h.click_id(KIND);
    let _ = h.click_part(KIND, PartRef::item(Part::ROW, ItemKey::index(3)));
    let _ = h.click_id(HINT);
    let state = page_debug(&h);
    assert!(state.contains("kind: 3"), "{state}");
    assert!(state.contains("show_hint: false"), "{state}");

    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Left);
    assert!(h.text().contains("✓"), "{}", h.text());
    assert!(
        !h.text().contains("Inspect the statement detail below"),
        "the pointer-disabled hint disappears from the live Notice"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn inert_state_specimens_do_not_register_live_page_controls() {
    let mut h = host(120, 40);
    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Right);
    assert!(h.text().contains("With detail"));
    assert!(h.text().contains("With hint"));
    assert!(h.text().contains("Overflow"));
    assert!(h.text().contains("Narrow"));
    assert!(h.area_of(TEST_CONNECTION).is_none());
    for id in [
        KIND,
        DETAIL_LINES,
        MESSAGE_LINES,
        DETAIL_TONE,
        WIDTH,
        DETAIL,
        HINT,
        MARK_CUT,
    ] {
        assert!(
            h.area_of(id).is_none(),
            "reference specimens stay inert: {id:?}"
        );
    }
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn notice_controls_remain_inside_the_requested_area_after_resize() {
    let mut h = host(120, 40);
    let _ = h.resize(80, 24);
    let scroll = h
        .area_of(PLAY_SCROLL)
        .expect("the live content remains scrollable");
    let button = h
        .area_of(TEST_CONNECTION)
        .expect("Test connection remains reachable");
    assert!(button.x >= scroll.x && button.right() <= scroll.right());
    assert!(button.y >= scroll.y && button.bottom() <= scroll.bottom());

    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Right);
    assert!(h.area_of(KIND).is_some());
    assert!(h.area_of(DETAIL).is_some());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
