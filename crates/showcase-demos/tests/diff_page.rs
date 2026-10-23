use std::cell::RefCell;

use showcase_demos::pages::{Page, diff::DiffPage};
use termrock::{
    App, Axis, ColorLevel, Cx, Id, KeyCode, KeyModifiers, Rect, Response, Role, Surface, Theme, Ui,
    theme::resolve::bind_role,
};
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
const GALLERY_STATES: [&str; 4] = ["Rest", "Focused", "Empty", "Narrow"];

struct PageHost {
    page: DiffPage,
    status: Option<String>,
    hints: RefCell<Vec<(&'static str, &'static str)>>,
}

impl App for PageHost {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let update = self.page.update(cx);
        self.status = update.status.map(|status| status.0);
        update.response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        *self.hints.borrow_mut() = self.page.hints(ui).to_vec();
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
            hints: RefCell::new(Vec::new()),
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

fn find_in(h: &Harness<PageHost>, area: Rect, needle: &str) -> Option<(u16, u16)> {
    (area.y..area.bottom()).find_map(|y| {
        let row = h.row(y);
        let byte = row.find(needle)?;
        let x = termrock::width(&row[..byte]);
        (x >= area.x && x < area.right()).then_some((x, y))
    })
}

fn text_in(h: &Harness<PageHost>, area: Rect) -> String {
    (area.y..area.bottom())
        .map(|y| {
            (area.x..area.right())
                .map(|x| h.cell(x, y).symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn state_view_area(h: &Harness<PageHost>, two_columns: bool) -> Rect {
    let area = h
        .area_of(STATES_SCROLL)
        .expect("the real state gallery has a scroll viewport");
    let content_rows = if two_columns {
        1usize.saturating_add(4usize.saturating_mul(6))
    } else {
        50
    };
    assert!(
        area.height as usize >= content_rows,
        "the width control fits without a vertical scrollbar, so its component width is the actual post-gutter width: area={area:?}, content_rows={content_rows}"
    );
    area
}

fn review_specimen_area(h: &Harness<PageHost>, two_columns: bool, state: usize) -> Rect {
    let body = state_view_area(h, two_columns);
    let label = GALLERY_STATES[state];
    let mut in_review_group = false;
    let label_y = (body.y..body.bottom()).find(|y| {
        let line = h.row(*y);
        let row = line.trim_start();
        if two_columns {
            row.starts_with(label)
        } else if row.starts_with("review") {
            in_review_group = true;
            false
        } else {
            in_review_group && row.starts_with(label)
        }
    });
    let label_y = label_y.unwrap_or_else(|| {
        panic!(
            "the {label} label is visible in the {} gallery\n{}",
            if two_columns { "matrix" } else { "stacked" },
            h.text()
        )
    });
    if two_columns {
        let content_x = body.x.saturating_add(10);
        let content_width = body.right().saturating_sub(content_x);
        let left_width = content_width.saturating_sub(2) / 2;
        let review_x = content_x.saturating_add(left_width).saturating_add(2);
        let review_width = body.right().saturating_sub(review_x);
        Rect::new(review_x, label_y.saturating_add(1), review_width, 5)
    } else {
        Rect::new(body.x, label_y.saturating_add(1), body.width, 5)
    }
}

fn assert_unified_review_fallback(h: &Harness<PageHost>, state: usize, two_columns: bool) {
    let area = review_specimen_area(h, two_columns, state);
    assert!(
        area.width < 45,
        "control exercises actual post-scrollbar-gutter DiffView width below the shared 45-column Review threshold: {area:?}"
    );
    let text = text_in(h, area);
    for expected in [
        "M retry.toml  +1 −1 · 1 hunk",
        "@@ -1,2 +1,2 @@",
        "  1     - attempts = 3",
        "      1 + attempts = 5",
        "  2   2   enabled = true",
    ] {
        assert!(
            text.contains(expected),
            "requested Review state {state} uses a complete actual Unified fallback; missing {expected:?} in area {area:?}:\n{text}\nfull frame:\n{}",
            h.text()
        );
    }
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
fn chooser_and_diff_header_keep_each_status_role_and_rename_origin() {
    let theme = Theme::junie();
    let expected_roles = [
        ("M", Role::Warning),
        ("A", Role::Success),
        ("D", Role::Danger),
        ("R", Role::Fg(termrock::FgStep::Secondary)),
    ];
    let mut h = Harness::new(
        PageHost {
            page: DiffPage::new(),
            status: None,
            hints: RefCell::new(Vec::new()),
        },
        theme.clone(),
        120,
        40,
    );

    for (index, (marker, role)) in expected_roles.into_iter().enumerate() {
        assert!(h.tab_to(FILE_LIST));
        for _ in 0..4 {
            let _ = h.key(KeyCode::Up);
        }
        for _ in 0..index {
            let _ = h.key(KeyCode::Down);
        }
        let chooser_text = h.text();
        let path = [
            "config/service.toml",
            "docs/retries.md",
            "src/legacy_backoff.rs",
            "src/backoff/retry.rs",
        ][index];
        let list_area = h.area_of(FILE_LIST).expect("change chooser rows");
        let chooser_needle = [
            "M config/service.toml",
            "A docs/retries.md",
            "D src/legacy_backoff.rs",
            "R src/retry.rs",
        ][index];
        let (chooser_x, chooser_y) = find_in(&h, list_area, chooser_needle)
            .unwrap_or_else(|| panic!("chooser lost {marker} status for {path}\n{chooser_text}"));
        let chooser_role = bind_role(&theme, role, Surface::Canvas).expect("semantic status role");
        assert_eq!(
            h.cell(chooser_x, chooser_y).fg,
            chooser_role,
            "{marker} chooser marker has its semantic role"
        );
        if index == 3 {
            assert!(
                h.row(chooser_y).contains("R src/retry.rs → src/back +1 −1"),
                "chooser lost the rename origin, destination prefix, or count\n{}",
                h.row(chooser_y)
            );
        }

        let header_needle = if index == 3 {
            "R src/retry.rs → src/backoff/retry.rs".to_owned()
        } else {
            format!("{marker} {path}")
        };
        let count = [
            "+10 −5 · 5 hunks",
            "+3 −0 · 1 hunk",
            "+0 −3 · 1 hunk",
            "+1 −1 · 1 hunk",
        ][index];
        let (header_x, header_y) = (0..h.buffer().area().height)
            .find_map(|y| {
                let row = h.row(y);
                if !row.contains(&header_needle) || !row.contains(count) {
                    return None;
                }
                let byte = row.find(&header_needle)?;
                Some((termrock::width(&row[..byte]), y))
            })
            .unwrap_or_else(|| panic!("live header lost {header_needle}\n{chooser_text}"));
        let header_row = h.row(header_y);
        if index == 3 {
            assert!(
                header_row.contains("src/retry.rs → src/backoff/retry.rs"),
                "live header lost the full rename\n{header_row}"
            );
        }
        let expected = bind_role(&theme, role, Surface::Canvas).expect("semantic status role");
        assert_eq!(
            h.cell(header_x, header_y).fg,
            expected,
            "{marker} live Diff status marker has its semantic role"
        );
        assert!(header_row.contains(count), "{header_row}");
    }
    assert!(h.text().contains("+1 −1 · 1 hunk"));
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
    let _ = h.key_mod(KeyCode::Right, KeyModifiers::SHIFT);
    let _ = h.key(KeyCode::Char('y'));
    let status = format!("Captured selection: {expected}\"");
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
fn keyboard_selection_can_start_at_the_visible_diff_tail() {
    let mut h = host(120, 40);
    assert!(h.tab_to(FILE_LIST));
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Down);
    assert!(h.text().contains("src/legacy_backoff.rs"));

    assert!(h.tab_to(LIVE_VIEW));
    let _ = h.key_mod(KeyCode::Left, KeyModifiers::SHIFT);
    let _ = h.key(KeyCode::Char('y'));
    assert_eq!(
        h.app().status.as_deref(),
        Some("Captured selection: }"),
        "Shift+Left starts from the last visible logical line"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn selection_hints_describe_start_and_then_capture_controls() {
    let mut h = host(120, 40);
    assert!(h.tab_to(LIVE_VIEW));
    let hints = h.app().hints.borrow().clone();
    assert!(hints.contains(&("Drag", "Select")), "{hints:?}");
    assert!(
        hints.contains(&("Shift+← →", "Start / extend (default)")),
        "{hints:?}"
    );
    assert!(
        !hints.iter().any(|(_, action)| *action == "Capture"),
        "{hints:?}"
    );
    assert!(
        !hints.iter().any(|(_, action)| *action == "Clear"),
        "{hints:?}"
    );

    let (x, y) = h.find("attempts = 3").expect("removed row is visible");
    let _ = h.drag((x, y), (x.saturating_add(7), y));
    let hints = h.app().hints.borrow().clone();
    assert!(hints.contains(&("Shift+← →", "Extend")), "{hints:?}");
    assert!(hints.contains(&("y", "Capture")), "{hints:?}");
    assert!(hints.contains(&("Esc", "Clear")), "{hints:?}");

    assert!(h.tab_to(FILE_LIST));
    let hints = h.app().hints.borrow().clone();
    assert!(hints.contains(&("↑ ↓", "View file")), "{hints:?}");
    assert!(
        !hints.iter().any(|(_, action)| *action == "Capture"),
        "{hints:?}"
    );
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
            text.contains("+1 −1 · 1 hunk"),
            "Rest and Focused specimens retain one removal, one addition, and one hunk\n{text}"
        );
        assert!(
            text.contains("@@ -1,2 +1,2 @@"),
            "the canonical gallery hunk header is rendered\n{text}"
        );
        assert!(
            text.contains("  1     - attempts = 3"),
            "the removal uses the canonical old line number\n{text}"
        );
        assert!(
            text.contains("      1 + attempts = 5"),
            "the addition uses the canonical new line number\n{text}"
        );
        assert!(
            text.contains("  2   2   enabled = true"),
            "the context tail remains visible after the hunk\n{text}"
        );
        assert!(
            h.diagnostics().is_empty(),
            "{color:?}: {:?}",
            h.diagnostics()
        );
        assert_eq!(text.matches("M retry.toml").count(), 6, "{color:?}: {text}");
        assert_eq!(
            text.matches("No file selected").count(),
            2,
            "{color:?}: {text}"
        );
        assert_eq!(
            text.matches("@@ -1,2 +1,2 @@").count(),
            6,
            "all Rest, Focused, and Narrow views retain their real hunk in both columns\n{text}"
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

    let mut short = host(48, 12);
    section(&mut short, 2);
    let scroll = short
        .area_of(STATES_SCROLL)
        .expect("state gallery scroll region");
    let tabs = short.area_of(PAGE_TABS).expect("page section tabs");
    assert!(
        scroll.y >= tabs.bottom(),
        "gallery follows tabs: {scroll:?} {tabs:?}"
    );
    assert!(
        scroll.height >= 8,
        "short stacked cards need a viewport tall enough for one group heading and a complete six-row specimen: {scroll:?}"
    );
    let initial_short_text = short.text();
    let mut active_mode = Some(0);
    let mut reached = [[false; GALLERY_STATES.len()]; 2];
    for _ in 0..64 {
        for y in 0..short.buffer().area().height {
            let row = short.row(y);
            let trimmed = row.trim_start();
            if trimmed.starts_with("unified") {
                active_mode = Some(0);
            } else if trimmed.starts_with("review") {
                active_mode = Some(1);
            }
            let Some(state_index) = GALLERY_STATES
                .iter()
                .position(|label| trimmed.starts_with(label))
            else {
                continue;
            };
            // The one-row group heading can scroll out before the next card is
            // fully visible. The duplicated Rest label then marks the start
            // of the second mode group even if that heading was skipped.
            if active_mode == Some(0)
                && state_index == 0
                && reached[0].iter().all(|visible| *visible)
            {
                active_mode = Some(1);
            }
            let Some(mode_index) = active_mode else {
                continue;
            };

            let frame = short.text();
            if state_index == 2 {
                assert!(
                    short.row(y.saturating_add(1)).contains("No file selected"),
                    "the complete Empty specimen is reachable in mode {mode_index}\n{frame}"
                );
            } else {
                let diff_rows = 5;
                let last_y = y.saturating_add(diff_rows as u16);
                assert!(
                    last_y < short.buffer().area().height,
                    "a gallery label cannot be shown without its complete diff body (label_y={y}, body_rows={diff_rows}, scroll={scroll:?}, terminal={:?})\n{frame}",
                    short.buffer().area()
                );
                let body = (y.saturating_add(1)..=last_y)
                    .map(|row| short.row(row))
                    .collect::<Vec<_>>()
                    .join("\n");
                assert!(
                    body.contains("M retry.toml") && body.contains("+1 −1 · 1 hunk"),
                    "specimen status and actual hunk counts remain visible\n{frame}"
                );
                assert!(
                    body.contains("@@ -1,2 +1,2 @@"),
                    "specimen keeps its real hunk header\n{frame}"
                );
                if mode_index == 1 && state_index != 3 {
                    assert!(
                        body.contains("· review")
                            && body.contains("attempts = 3")
                            && body.contains("attempts = 5")
                            && body.contains("enabled = true"),
                        "the requested Review specimen retains both sides and its context tail\n{frame}"
                    );
                } else {
                    assert!(
                        !body.contains("· review")
                            && body.contains("  1     - attempts = 3")
                            && body.contains("      1 + attempts = 5")
                            && body.contains("  2   2   enabled = true"),
                        "the complete Unified projection retains numbering and context tail\n{frame}"
                    );
                }
            }
            reached[mode_index][state_index] = true;
        }
        if reached.iter().flatten().all(|visible| *visible) {
            break;
        }
        let _ = short.wheel(Axis::V, 1, scroll.x.saturating_add(1), scroll.y);
    }
    assert_eq!(
        reached,
        [[true; GALLERY_STATES.len()]; 2],
        "all eight complete specimens are reachable in the short stacked gallery; initial frame:\n{initial_short_text}\nfinal frame:\n{}",
        short.text()
    );
    assert!(short.diagnostics().is_empty(), "{:?}", short.diagnostics());
}

#[test]
fn matrix_review_fallback_keeps_complete_unified_rows_at_eighty_columns() {
    let mut h = host(80, 40);
    section(&mut h, 2);

    for state in [0, 1] {
        assert_unified_review_fallback(&h, state, true);
    }
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn stacked_review_fallback_keeps_complete_unified_rows_at_forty_four_columns() {
    let mut h = host(44, 64);
    section(&mut h, 2);

    for state in [0, 1] {
        assert_unified_review_fallback(&h, state, false);
    }
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn sufficiently_wide_gallery_review_keeps_paired_rows() {
    let mut h = host(120, 40);
    section(&mut h, 2);

    for state in [0, 1] {
        let area = review_specimen_area(&h, true, state);
        assert!(area.width >= 45, "wide Review control: {area:?}");
        let text = text_in(&h, Rect { height: 4, ..area });
        for expected in [
            "M retry.toml  +1 −1 · 1 hunk · review",
            "@@ -1,2 +1,2 @@",
            "attempts = 3",
            "attempts = 5",
            "enabled = true",
        ] {
            assert!(
                text.contains(expected),
                "wide requested Review state {state} retains paired rows; missing {expected:?} in area {area:?}:\n{text}"
            );
        }
    }
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
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
