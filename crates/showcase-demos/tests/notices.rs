use showcase_demos::pages::{Page, notices::NoticesPage};
use termrock::{
    App, Axis, Color, Cx, FgStep, Id, ItemKey, KeyCode, Modifier, Part, PartRef, Rect, Response,
    Theme, Ui,
};
use termrock_test_support::Harness;

const PAGE_TABS: Id = Id::root("showcase_ui::pages::notices.page_tabs");
const PLAY_SCROLL: Id = Id::root("showcase_ui::pages::notices.play_scroll");
const INLINE_PANEL: Id = Id::root("showcase_ui::pages::notices.inline_panel");
const CONNECTION_PANEL: Id = Id::root("showcase_ui::pages::notices.connection_panel");
const QUERY_PANEL: Id = Id::root("showcase_ui::pages::notices.query_panel");
const STATES_DETAIL_PANEL: Id = Id::root("showcase_ui::pages::notices.states_detail_panel");
const STATES_HINT_PANEL: Id = Id::root("showcase_ui::pages::notices.states_hint_panel");
const TEST_CONNECTION: Id = Id::root("showcase_ui::pages::notices.test_connection");
const OPTIONS_SCROLL: Id = Id::root("showcase_ui::pages::notices.options_scroll");
const STATES_SCROLL: Id = Id::root("showcase_ui::pages::notices.states_scroll");
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

struct PageAreaHost {
    page: NoticesPage,
    area: Rect,
}

impl App for PageHost {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.page.update(cx).response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.page.draw(ui, ui.full());
    }
}

impl App for PageAreaHost {
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
            page: NoticesPage::new(),
        },
        Theme::junie(),
        width,
        height,
    )
}

fn area_host(area: Rect) -> Harness<PageAreaHost> {
    Harness::new(
        PageAreaHost {
            page: NoticesPage::new(),
            area,
        },
        Theme::junie(),
        64,
        16,
    )
}

fn section_area_host(section: usize, area: Rect) -> Harness<PageAreaHost> {
    let mut h = area_host(Rect::new(0, 0, 64, 16));
    assert!(h.tab_to(PAGE_TABS));
    for _ in 0..section {
        let _ = h.key(KeyCode::Right);
    }
    h.app_mut().area = area;
    h.draw();
    h
}

fn page_debug(h: &Harness<PageHost>) -> String {
    format!("{:?}", h.app().page)
}

fn stored_scroll_offset(h: &Harness<PageHost>, field: &str) -> usize {
    let marker = format!("{field}: ScrollState {{ offset: ");
    page_debug(h)
        .split(&marker)
        .nth(1)
        .and_then(|state| state.split(',').next())
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| panic!("missing {field} offset in {:?}", h.app().page))
}

fn scroll_to_bottom(h: &mut Harness<PageHost>, id: Id) {
    for _ in 0..20 {
        let area = h.area_of(id).expect("the active section has a scroll area");
        let _ = h.wheel(Axis::V, 3, area.x.saturating_add(1), area.y);
    }
}

fn choose_compact_notice_options(h: &mut Harness<PageHost>) {
    select_index(h, DETAIL_LINES, 1);
    select_index(h, MESSAGE_LINES, 1);
    for id in [DETAIL, HINT] {
        assert!(h.tab_to(id), "option {id:?} is reachable");
        let _ = h.key(KeyCode::Char(' '));
    }
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
fn short_playground_scroll_keeps_source_rows_and_drops_offscreen_button() {
    let mut h = host(80, 12);
    let scroll = h
        .area_of(PLAY_SCROLL)
        .expect("the compact playground has a scroll viewport");

    for _ in 0..16 {
        let _ = h.wheel(Axis::V, 3, scroll.x.saturating_add(1), scroll.y);
    }

    let text = h.text();
    assert!(
        text.contains("Read-only replica"),
        "{text}\npage={:?}",
        h.app().page
    );
    assert!(
        text.contains("Saved 12 rows in 84 ms"),
        "{text}\npage={:?}",
        h.app().page
    );
    assert!(
        !text.contains("prod-replica"),
        "the first card is above the short scrolled viewport\n{text}"
    );
    assert!(
        h.area_of(TEST_CONNECTION).is_none(),
        "the fully scrolled-away button must not register a ghost hit target"
    );
    let visible = h
        .area_of(INLINE_PANEL)
        .expect("the last logical card remains visible at the bottom");
    let scroll = h.area_of(PLAY_SCROLL).unwrap();
    assert!(visible.y >= scroll.y && visible.bottom() <= scroll.bottom());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn cropped_notice_rows_match_the_same_logical_rows_in_a_full_render() {
    let full = host(80, 24);
    let mut cropped = host(80, 12);
    let full_view = full
        .area_of(PLAY_SCROLL)
        .expect("the full playground has a scroll viewport");
    let cropped_view = cropped
        .area_of(PLAY_SCROLL)
        .expect("the compact playground has a scroll viewport");
    let _ = cropped.wheel(Axis::V, 2, cropped_view.x.saturating_add(1), cropped_view.y);

    let debug = page_debug(&cropped);
    let offset = debug
        .split("play_scroll: ScrollState { offset: ")
        .nth(1)
        .and_then(|state| state.split(',').next())
        .and_then(|value| value.parse::<u16>().ok())
        .expect("the compact page records its logical scroll offset");
    assert_eq!(offset, 6, "{debug}");

    let content_width = cropped_view.width.saturating_sub(1);
    for row in 0..cropped_view.height {
        let cropped_y = cropped_view.y.saturating_add(row);
        let full_y = full_view.y.saturating_add(offset).saturating_add(row);
        for x in cropped_view.x..cropped_view.x.saturating_add(content_width) {
            let actual = cropped.cell(x, cropped_y);
            let source = full.cell(x, full_y);
            assert_eq!(actual.symbol(), source.symbol(), "at ({x},{cropped_y})");
            assert_eq!(actual.fg, source.fg, "at ({x},{cropped_y})");
            assert_eq!(actual.bg, source.bg, "at ({x},{cropped_y})");
            assert_eq!(actual.modifier, source.modifier, "at ({x},{cropped_y})");
        }
    }
    assert!(
        cropped.diagnostics().is_empty(),
        "{:?}",
        cropped.diagnostics()
    );
}

#[test]
fn short_options_scroll_omits_offscreen_selects_and_keeps_visible_checkbox_live() {
    let mut h = host(80, 12);
    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Right);
    let scroll = h
        .area_of(OPTIONS_SCROLL)
        .expect("Options has a scroll viewport");

    for _ in 0..4 {
        let _ = h.wheel(Axis::V, 3, scroll.x.saturating_add(1), scroll.y);
    }

    assert!(h.area_of(KIND).is_none(), "the first Select scrolled away");
    let cut = h
        .area_of(MARK_CUT)
        .expect("the last checkbox remains visible");
    assert!(cut.y >= scroll.y && cut.bottom() <= scroll.bottom());
    assert!(h.tab_to(MARK_CUT));
    assert_eq!(h.focus(), Some(MARK_CUT));
    let _ = h.key(KeyCode::Char(' '));
    assert!(page_debug(&h).contains("mark_cut: false"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn options_resize_projects_the_new_viewport_on_the_first_returned_frame() {
    let mut h = host(80, 12);
    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Right);
    scroll_to_bottom(&mut h, OPTIONS_SCROLL);
    assert!(stored_scroll_offset(&h, "options_scroll") > 0);

    let _ = h.resize(80, 40);

    let mut fresh = host(80, 40);
    assert!(fresh.tab_to(PAGE_TABS));
    let _ = fresh.key(KeyCode::Right);
    assert_eq!(
        h.text(),
        fresh.text(),
        "the first resized frame uses offset zero"
    );
    assert_eq!(h.focus(), Some(PAGE_TABS));
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
        assert_eq!(h.area_of(id), fresh.area_of(id), "control {id:?}");
        assert!(h.area_of(id).is_some(), "control {id:?} remains reachable");
    }
    assert_eq!(
        h.area_of_part(OPTIONS_SCROLL, PartRef::of(Part::THUMB)),
        None,
        "fitting Options has no stale scrollbar thumb"
    );
    assert!(h.tab_to(KIND));
    assert_eq!(h.focus(), Some(KIND));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn options_resize_preserves_a_still_valid_scroll_offset() {
    let mut h = host(80, 12);
    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Right);
    let area = h.area_of(OPTIONS_SCROLL).unwrap();
    let _ = h.wheel(Axis::V, 1, area.x.saturating_add(1), area.y);
    let offset = stored_scroll_offset(&h, "options_scroll");
    assert!(offset > 0);

    let _ = h.resize(80, 13);

    let mut fresh = host(80, 13);
    assert!(fresh.tab_to(PAGE_TABS));
    let _ = fresh.key(KeyCode::Right);
    let area = fresh.area_of(OPTIONS_SCROLL).unwrap();
    let _ = fresh.wheel(Axis::V, 1, area.x.saturating_add(1), area.y);
    assert_eq!(stored_scroll_offset(&fresh, "options_scroll"), offset);
    assert_eq!(h.text(), fresh.text(), "the legal offset is not reset");
    assert_eq!(h.area_of(KIND), fresh.area_of(KIND));
    assert_eq!(
        h.area_of_part(OPTIONS_SCROLL, PartRef::of(Part::THUMB)),
        fresh.area_of_part(OPTIONS_SCROLL, PartRef::of(Part::THUMB))
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn shrinking_playground_options_to_fit_projects_the_first_source_row() {
    let mut h = host(80, 34);
    assert!(
        h.area_of_part(PLAY_SCROLL, PartRef::of(Part::THUMB))
            .is_some(),
        "the default Notices playground overflows at this size"
    );
    scroll_to_bottom(&mut h, PLAY_SCROLL);
    assert!(stored_scroll_offset(&h, "play_scroll") > 0);

    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Right);
    choose_compact_notice_options(&mut h);
    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Left);

    let text = h.text();
    assert!(
        text.contains("prod-replica"),
        "the fitting source begins at row zero\n{text}\npage={:?}",
        h.app().page
    );
    assert!(text.contains("Authentication failed"), "{text}");
    assert!(text.contains("Result 1"), "{text}");
    assert!(text.contains("Saved 12 rows in 84 ms"), "{text}");
    let state = page_debug(&h);
    for option in [
        "detail_lines: 1",
        "message_lines: 1",
        "show_detail: false",
        "show_hint: false",
    ] {
        assert!(
            state.contains(option),
            "the option interaction changed {option}: {state}"
        );
    }
    assert!(h.area_of(TEST_CONNECTION).is_some());
    assert!(h.area_of(INLINE_PANEL).is_some());
    assert_eq!(
        h.area_of_part(PLAY_SCROLL, PartRef::of(Part::THUMB)),
        None,
        "all content now fits, so no thumb is drawn"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn shrinking_states_options_keeps_current_body_and_scrollbar_in_sync() {
    let mut h = host(80, 12);
    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Right);
    scroll_to_bottom(&mut h, STATES_SCROLL);
    assert!(stored_scroll_offset(&h, "states_scroll") > 0);

    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Left);
    choose_compact_notice_options(&mut h);
    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Right);

    let text = h.text();
    assert!(
        text.contains("With detail"),
        "the first source panel remains visible\n{text}"
    );
    assert!(text.contains("Authentication failed"), "{text}");
    assert!(
        !text.contains("Connection test"),
        "hints are hidden\n{text}"
    );
    assert!(
        !text.contains("Password rejected for acme_app"),
        "details are hidden\n{text}"
    );
    let view = h.area_of(STATES_SCROLL).unwrap();
    let panel = h
        .area_of(STATES_DETAIL_PANEL)
        .expect("the first card is projected into view");
    assert!(panel.y >= view.y && panel.bottom() <= view.bottom());
    let track = h
        .area_of_part(STATES_SCROLL, PartRef::of(Part::TRACK))
        .expect("the reduced States content still overflows");
    let thumb = h
        .area_of_part(STATES_SCROLL, PartRef::of(Part::THUMB))
        .expect("the overflow thumb remains visible");
    assert_eq!(
        thumb.y, track.y,
        "the body and chrome both start at source row zero"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn playground_resize_projects_the_new_viewport_before_painting_body_rows() {
    let mut h = host(80, 12);
    scroll_to_bottom(&mut h, PLAY_SCROLL);
    assert!(stored_scroll_offset(&h, "play_scroll") > 0);

    let _ = h.resize(80, 40);

    let text = h.text();
    assert!(
        text.contains("prod-replica"),
        "the first card fits after resize\n{text}"
    );
    assert!(text.contains("Result 1"), "{text}");
    assert!(text.contains("Saved 12 rows in 84 ms"), "{text}");
    let view = h.area_of(PLAY_SCROLL).unwrap();
    let connection = h.area_of(CONNECTION_PANEL).unwrap();
    let button = h.area_of(TEST_CONNECTION).unwrap();
    assert!(connection.y >= view.y && connection.bottom() <= view.bottom());
    assert!(button.y >= view.y && button.bottom() <= view.bottom());
    assert_eq!(
        h.area_of_part(PLAY_SCROLL, PartRef::of(Part::THUMB)),
        None,
        "fitting Playground has no stale thumb"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn states_resize_projects_the_new_viewport_before_painting_gallery_rows() {
    let mut h = host(80, 12);
    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Right);
    scroll_to_bottom(&mut h, STATES_SCROLL);
    assert!(stored_scroll_offset(&h, "states_scroll") > 0);

    let _ = h.resize(80, 40);

    let text = h.text();
    assert!(
        text.contains("With detail"),
        "the first gallery card is visible\n{text}"
    );
    assert!(
        text.contains("Connection test"),
        "the last gallery specimen is visible\n{text}"
    );
    assert!(text.contains("Hint narrow tail"), "{text}");
    let view = h.area_of(STATES_SCROLL).unwrap();
    for id in [STATES_DETAIL_PANEL, STATES_HINT_PANEL] {
        let panel = h.area_of(id).expect("both panels fit after resize");
        assert!(panel.y >= view.y && panel.bottom() <= view.bottom());
    }
    assert_eq!(
        h.area_of_part(STATES_SCROLL, PartRef::of(Part::THUMB)),
        None,
        "fitting States has no stale thumb"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn short_states_scroll_reaches_the_later_reference_panel_without_ghosting_the_first() {
    let mut h = host(80, 12);
    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Right);
    let scroll = h
        .area_of(STATES_SCROLL)
        .expect("States has a scroll viewport");

    for _ in 0..16 {
        let _ = h.wheel(Axis::V, 3, scroll.x.saturating_add(1), scroll.y);
    }

    assert!(h.area_of(STATES_DETAIL_PANEL).is_none());
    let hint_panel = h
        .area_of(STATES_HINT_PANEL)
        .expect("the later logical panel remains visible at the bottom");
    assert!(hint_panel.y >= scroll.y && hint_panel.bottom() <= scroll.bottom());
    assert!(h.text().contains("Retry is available"), "{}", h.text());
    assert!(h.text().contains("Connection test"), "{}", h.text());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
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
fn error_and_overflow_specimens_show_their_named_notice_states() {
    let mut h = host(120, 60);
    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Right);

    let error_message = "Authentication failed while checking";
    let detail_error = h
        .find(error_message)
        .expect("the detail column has a literal Error specimen");
    assert_eq!(
        h.cell(detail_error.0.saturating_sub(2), detail_error.1)
            .symbol(),
        "!",
        "the detail Error specimen uses the Error marker"
    );
    assert_eq!(
        h.cell(detail_error.0.saturating_sub(2), detail_error.1).fg,
        Theme::junie().color.danger
    );

    let hint_error = "Replica connection failed while checking";
    let hint_position = h
        .find(hint_error)
        .expect("the hint column has its own literal Error specimen");
    assert_eq!(
        h.cell(hint_position.0.saturating_sub(2), hint_position.1)
            .symbol(),
        "!"
    );
    assert_eq!(
        h.cell(hint_position.0.saturating_sub(2), hint_position.1)
            .fg,
        Theme::junie().color.danger
    );

    let overflow_labels = (0..h.buffer().area().height)
        .filter(|row| h.row(*row).contains("Overflow"))
        .collect::<Vec<_>>();
    assert_eq!(overflow_labels.len(), 2, "each column has one Overflow row");
    for label_row in overflow_labels {
        assert!(
            (label_row..label_row.saturating_add(3).min(h.buffer().area().height))
                .any(|row| h.row(row).contains('…')),
            "the Overflow specimen visibly truncates near row {label_row}\n{}",
            h.text()
        );
    }

    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Left);
    assert!(h.tab_to(MARK_CUT));
    let _ = h.key(KeyCode::Char(' '));
    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Right);
    assert!(
        (0..h.buffer().area().height).all(|row| !h.row(row).contains('…')),
        "hiding the cut marker does not leave an ellipsis in the actual specimens\n{}",
        h.text()
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn states_specimens_fit_their_exact_narrow_render_width() {
    let mut h = host(80, 48);
    assert!(h.tab_to(PAGE_TABS));
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Right);

    let text = h.text();
    for specimen in [
        "Rest",
        "Authentication failed while checking",
        "Overflow",
        "Detail narrow",
        "tail remains",
        "Narrow specimen",
        "Replica connection failed while checking the configured",
        "Hint narrow tail",
        "Connection test",
    ] {
        assert!(
            text.contains(specimen),
            "the narrow rendered sample is not fully painted; missing {specimen}\n{text}"
        );
    }
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn page_area_bounds_paint_and_control_registration_at_offset_and_tiny_sizes() {
    let requested = [
        Rect::new(7, 3, 0, 4),
        Rect::new(7, 3, 4, 0),
        Rect::new(7, 3, 1, 1),
        Rect::new(7, 3, 2, 2),
        Rect::new(7, 3, 40, 1),
        Rect::new(7, 3, 40, 2),
        Rect::new(7, 3, 40, 8),
    ];

    for section in 0..3 {
        for area in requested {
            let h = section_area_host(section, area);
            for y in 0..h.buffer().area().height {
                for x in 0..h.buffer().area().width {
                    if x >= area.x && x < area.right() && y >= area.y && y < area.bottom() {
                        continue;
                    }
                    let cell = h.cell(x, y);
                    assert_eq!(cell.symbol(), " ", "outside {area:?} at ({x},{y})");
                    assert_eq!(cell.fg, Color::Reset, "outside {area:?} at ({x},{y})");
                    assert_eq!(cell.bg, Color::Reset, "outside {area:?} at ({x},{y})");
                    assert!(cell.modifier.is_empty(), "outside {area:?} at ({x},{y})");
                }
            }
            assert_eq!(h.cursor(), None, "small page rect {area:?} has no cursor");

            for id in [
                PAGE_TABS,
                PLAY_SCROLL,
                OPTIONS_SCROLL,
                STATES_SCROLL,
                CONNECTION_PANEL,
                QUERY_PANEL,
                INLINE_PANEL,
                STATES_DETAIL_PANEL,
                STATES_HINT_PANEL,
                TEST_CONNECTION,
                KIND,
                DETAIL_LINES,
                MESSAGE_LINES,
                DETAIL_TONE,
                WIDTH,
                DETAIL,
                HINT,
                MARK_CUT,
            ] {
                if let Some(control) = h.area_of(id) {
                    assert!(
                        control.x >= area.x
                            && control.y >= area.y
                            && control.right() <= area.right()
                            && control.bottom() <= area.bottom(),
                        "section {section}: {id:?} at {control:?} escaped {area:?}"
                    );
                }
            }
            assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
        }
    }

    let mut h = section_area_host(1, Rect::new(7, 3, 40, 12));
    let kind = h
        .area_of(KIND)
        .expect("the offset Options page draws its Select");
    assert!(kind.x >= 7 && kind.y >= 3 && kind.right() <= 47 && kind.bottom() <= 15);
    assert!(
        h.tab_to(KIND),
        "the offset Select remains keyboard focusable"
    );
    let _ = h.click_id(KIND);
    assert!(
        h.area_of(KIND).is_some(),
        "pointer activation stays in the page area"
    );
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
