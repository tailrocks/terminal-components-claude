use core::cell::RefCell;
use core::time::Duration;

use ratatui_core::layout::Rect;
use termrock_collections::ScrollState;
use termrock_controls::Button;
use termrock_core::id::{Id, ItemKey, Part, PartRef};
use termrock_core::intent::{Intent, Phase};
use termrock_core::response::Response;
use termrock_feedback::{Notice, NoticeSource};
use termrock_fields::{TextInput, TextInputState};
use termrock_overlays::{HelpAction, HelpOverlay, HelpOverlayState, HelpSection, NoticeReveal};
use termrock_runtime::author::{Axes, VerticalWindow};
use termrock_runtime::layer::{LayerSize, LayerSpec};
use termrock_runtime::{App, Cx, Focusability, ReferenceState, ReferenceTarget, Ui};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const OWNER: Id = Id::root("notice-reveal.owner");
const BUTTON: Id = Id::root("notice-reveal.button");
const INPUT: Id = Id::root("notice-reveal.input");
const PAGE_SCROLL: Id = Id::root("notice-reveal.page-scroll");
const MODAL: Id = Id::root("notice-reveal.modal");
const CAPTURE: Id = Id::root("notice-reveal.capture");
const HELP: Id = Id::root("notice-reveal.help");
const CUT_PART: Part = Part::custom("notice-reveal-cut");
const NOTICE_AREA: Rect = Rect::new(10, 3, 30, 5);
const BUTTON_AREA: Rect = Rect::new(1, 21, 10, 1);
const INPUT_AREA: Rect = Rect::new(15, 21, 20, 1);
const TALL_MESSAGE: &str = concat!(
    "MESSAGE_SOURCE_START\nROW01\nROW02\nROW03\nROW04\nROW05\nROW06\nROW07\nROW08\nROW09\nROW10\n",
    "ROW11\nROW12\nROW13\nROW14\nROW15\nROW16\nROW17\nROW18\nROW19\nROW20\n",
    "ROW21\nROW22\nROW23\nROW24\nROW25\nROW26\nROW27\nROW28\nROW29\nROW30\n",
    "ROW31\nROW32\nROW33\nROW34\nROW35\nROW36\nROW37\nROW38\nROW39\nROW40\n",
    "MESSAGE_SOURCE_TAIL"
);
const HELP_ROWS: &[(&str, &str)] = &[("Esc", "Close")];
const HELP_SECTIONS: &[HelpSection<'static>] = &[HelpSection::plain("Notice", HELP_ROWS)];

fn help() -> HelpOverlay<'static> {
    HelpOverlay::new(HELP, "reveal-test", HELP_SECTIONS)
}

fn cut(source: NoticeSource, row: u16) -> PartRef {
    let tag = match source {
        NoticeSource::Message => 0_u64,
        NoticeSource::Detail => 1_u64,
        NoticeSource::Hint => 2_u64,
        _ => unreachable!("known Notice source"),
    };
    PartRef::item(CUT_PART, ItemKey::num((tag << 16) | u64::from(row)))
}

struct Page {
    notice: Notice<'static>,
    notice_area: Rect,
    window: VerticalWindow,
    button_area: Rect,
    revision: u64,
    visible: bool,
    reveal: RefCell<NoticeReveal>,
    activations: usize,
    input: TextInputState,
    input_value: String,
    page_scroll: ScrollState,
    page_scroll_area: Rect,
    register_page_scroll: bool,
    open_modal: bool,
    open_help: bool,
    help_state: HelpOverlayState,
    reference: bool,
    capture_area: Option<Rect>,
}

impl Page {
    fn new(notice: Notice<'static>) -> Self {
        Self {
            notice,
            notice_area: NOTICE_AREA,
            window: VerticalWindow::full(NOTICE_AREA),
            button_area: BUTTON_AREA,
            revision: 0,
            visible: true,
            reveal: RefCell::new(NoticeReveal::new(OWNER)),
            activations: 0,
            input: TextInputState::default(),
            input_value: String::new(),
            page_scroll: {
                let mut state = ScrollState::new(100);
                state.apply_layout(24, 100);
                state
            },
            page_scroll_area: Rect::new(0, 0, 80, 24),
            register_page_scroll: false,
            open_modal: false,
            open_help: false,
            help_state: HelpOverlayState::default(),
            reference: false,
            capture_area: None,
        }
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.open_modal && !cx.is_open(MODAL) {
            cx.open_layer(MODAL, LayerSpec::modal(MODAL).size(LayerSize::Fixed(12, 4)));
        } else if !self.open_modal && cx.is_open(MODAL) {
            cx.close_layer(MODAL, None);
        }
        if self.open_help && !cx.is_open(HELP) {
            cx.open_layer(HELP, help().layer(cx));
        }

        let mut response = Button::new(BUTTON, "Action")
            .update(cx)
            .on_activated(|| self.activations = self.activations.saturating_add(1));
        if cx.is_open(MODAL) {
            response |= Button::new(MODAL, "Modal action").update(cx).erase();
        }
        response |= help()
            .update(cx, &mut self.help_state)
            .on_action(|action| match action {
                HelpAction::Closed(_) => {}
            });
        let capture_press = cx.intents(CAPTURE).find_map(|intent| match intent {
            Intent::Pointer {
                phase: Phase::Press,
                part,
                ..
            } => Some(part),
            _ => None,
        });
        if let Some(part) = capture_press {
            let _ = cx.capture(CAPTURE, part);
        }
        response |= TextInput::new(INPUT)
            .update(cx, &mut self.input, &mut self.input_value)
            .erase();
        if self.register_page_scroll {
            response |= termrock_runtime::ScrollRegion::new(PAGE_SCROLL)
                .scrollbar_visible(false)
                .update(cx, &mut self.page_scroll, 100);
        }
        response |= self.reveal.get_mut().update(cx, self.revision);
        response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Button::new(BUTTON, "Action").draw(ui, self.button_area);
        TextInput::new(INPUT).draw(ui, INPUT_AREA, &self.input);
        if let Some(area) = self.capture_area {
            ui.register_control(CAPTURE, area, Focusability::Focusable);
        }
        if self.register_page_scroll {
            ui.register_scroll(
                PAGE_SCROLL,
                self.page_scroll_area,
                Axes::V,
                self.page_scroll.headroom_v(),
            );
        }
        if self.visible {
            let paint = |ui: &mut Ui<'_>| {
                self.notice.draw_window(ui, self.window);
                let _ = self.reveal.borrow_mut().draw(
                    ui,
                    &self.notice,
                    self.notice_area,
                    self.window,
                    self.revision,
                );
            };
            if self.reference {
                let target = ReferenceTarget::new(OWNER, ReferenceState::HOVERED)
                    .part(cut(NoticeSource::Message, 0));
                ui.reference(Some(target), paint);
            } else {
                paint(ui);
            }
        }
        ui.layer(MODAL, |ui, area| {
            Button::new(MODAL, "Modal action").draw(ui, area)
        });
        ui.layer(HELP, |ui, area| help().draw(ui, area, &self.help_state));
    }
}

fn harness(notice: Notice<'static>) -> Harness<Page> {
    Harness::new(Page::new(notice), Theme::junie(), 80, 24)
}

fn hover_message(h: &mut Harness<Page>) {
    let area = h
        .area_of_part(OWNER, cut(NoticeSource::Message, 0))
        .expect("the clipped message row is a passive hover target");
    let x = area.x.saturating_add(area.width / 2);
    let _ = h.mouse(termrock_runtime::MouseKind::Move, x, area.y);
}

fn long_notice() -> Notice<'static> {
    Notice::warning(
        "MESSAGE_SOURCE_START one two three four five six seven eight nine ten eleven twelve MESSAGE_SOURCE_TAIL",
    )
    .message_lines(1)
}

#[test]
fn hover_reveal_waits_299ms_then_shows_the_full_current_source_without_focus_or_cursor() {
    let mut h = harness(long_notice());
    let before_focus = h.focus();
    let before_cursor = h.cursor();
    hover_message(&mut h);

    let _ = h.advance(Duration::from_millis(299));
    assert!(!h.text().contains("MESSAGE_SOURCE_TAIL"));
    assert_eq!(h.focus(), before_focus);
    assert_eq!(h.cursor(), before_cursor);

    let _ = h.advance(Duration::from_millis(1));
    assert!(h.text().contains("MESSAGE_SOURCE_TAIL"));
    assert_eq!(h.focus(), before_focus);
    assert_eq!(h.cursor(), before_cursor);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn source_row_identity_streams_more_than_three_cut_rows_and_resets_only_on_a_new_row() {
    let notice = Notice::error("界界界界界界界界").message_lines(8);
    let area = Rect::new(10, 3, 3, 8);
    let mut app = Page::new(notice);
    app.notice_area = area;
    app.window = VerticalWindow::full(area);
    let mut h = Harness::new(app, Theme::junie(), 40, 16);

    for row in 0..8 {
        assert!(
            h.area_of_part(OWNER, cut(NoticeSource::Message, row))
                .is_some(),
            "missing streamed cut row {row}"
        );
    }
    let row_four = h
        .area_of_part(OWNER, cut(NoticeSource::Message, 4))
        .expect("fourth cut target is registered");
    let _ = h.mouse(termrock_runtime::MouseKind::Move, row_four.x, row_four.y);
    let first_deadline = h.next_deadline().expect("rest deadline scheduled");
    let _ = h.mouse(
        termrock_runtime::MouseKind::Move,
        row_four.right().saturating_sub(1),
        row_four.y,
    );
    assert_eq!(h.next_deadline(), Some(first_deadline));
    let before = h.count("…");
    let _ = h.advance(Duration::from_millis(299));
    assert_eq!(h.count("…"), before);
    let _ = h.advance(Duration::from_millis(300));
    assert!(h.count("…") > before, "the fourth source cut is revealed");

    let row_five = h
        .area_of_part(OWNER, cut(NoticeSource::Message, 5))
        .expect("fifth cut target is registered");
    let _ = h.mouse(termrock_runtime::MouseKind::Move, row_five.x, row_five.y);
    assert_eq!(h.count("…"), before, "new row starts a fresh rest delay");
    let _ = h.advance(Duration::from_millis(300));
    assert!(h.count("…") > before, "the fifth source cut is revealed");
}

#[test]
fn horizontal_motion_within_one_wide_cut_row_keeps_the_original_deadline() {
    let mut h = harness(long_notice());
    let target = h
        .area_of_part(OWNER, cut(NoticeSource::Message, 0))
        .expect("visible cut source row");
    assert!(target.width > 1);
    let _ = h.mouse(termrock_runtime::MouseKind::Move, target.x, target.y);
    let deadline = h.next_deadline().expect("hover rest deadline");
    let _ = h.advance(Duration::from_millis(100));
    let _ = h.mouse(
        termrock_runtime::MouseKind::Move,
        target.right().saturating_sub(1),
        target.y,
    );
    assert_eq!(h.next_deadline(), Some(deadline));
    let _ = h.advance(Duration::from_millis(200));
    assert!(h.text().contains("MESSAGE_SOURCE_TAIL"));
}

#[test]
fn message_detail_and_hint_sources_have_distinct_live_targets() {
    let notice = Notice::warning("message header MESSAGE_TAIL_SENTINEL")
        .message_lines(1)
        .detail("detail header DETAIL_TAIL_SENTINEL")
        .max_lines(1)
        .hint("hint header HINT_TAIL_SENTINEL");
    let mut h = harness(notice);
    let message = h
        .area_of_part(OWNER, cut(NoticeSource::Message, 0))
        .expect("message cut");
    let detail = h
        .area_of_part(OWNER, cut(NoticeSource::Detail, 1))
        .expect("detail cut");
    let hint = h
        .area_of_part(OWNER, cut(NoticeSource::Hint, 2))
        .expect("hint cut");
    assert_ne!(message.y, detail.y);
    assert_ne!(detail.y, hint.y);

    for (source, area, expected, others) in [
        (
            NoticeSource::Message,
            message,
            "MESSAGE_TAIL_SENTINEL",
            ["DETAIL_TAIL_SENTINEL", "HINT_TAIL_SENTINEL"],
        ),
        (
            NoticeSource::Detail,
            detail,
            "DETAIL_TAIL_SENTINEL",
            ["MESSAGE_TAIL_SENTINEL", "HINT_TAIL_SENTINEL"],
        ),
        (
            NoticeSource::Hint,
            hint,
            "HINT_TAIL_SENTINEL",
            ["MESSAGE_TAIL_SENTINEL", "DETAIL_TAIL_SENTINEL"],
        ),
    ] {
        let _ = h.mouse(termrock_runtime::MouseKind::Move, area.x, area.y);
        let _ = h.advance(Duration::from_millis(300));
        assert!(
            h.text().contains(expected),
            "missing {source:?}: {}",
            h.text()
        );
        for other in others {
            assert!(
                !h.text().contains(other),
                "stale source {other}: {}",
                h.text()
            );
        }
    }
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn hidden_marker_does_not_disable_reveal_and_source_removal_clears_it_immediately() {
    let mut h = harness(long_notice().mark_cut(false));
    hover_message(&mut h);
    let _ = h.advance(Duration::from_millis(300));
    assert!(h.text().contains("MESSAGE_SOURCE_TAIL"));

    h.app_mut().notice = Notice::warning("fitting source").message_lines(2);
    let revision = h.app().revision.saturating_add(1);
    h.app_mut().revision = revision;
    h.draw();
    assert!(!h.text().contains("MESSAGE_SOURCE_TAIL"));
    assert!(
        h.area_of_part(OWNER, cut(NoticeSource::Message, 0))
            .is_none()
    );
    assert!(
        h.area_of_part(
            OWNER.sub("notice-reveal-scroll"),
            PartRef::of(Part::CONTAINER)
        )
        .is_none()
    );
}

#[test]
fn source_revision_and_fitting_layout_do_not_reuse_old_text_or_scroll_targets() {
    let mut h = harness(long_notice());
    hover_message(&mut h);
    let _ = h.advance(Duration::from_millis(300));
    assert!(h.text().contains("MESSAGE_SOURCE_TAIL"));

    h.app_mut().notice = Notice::warning("NEW_SOURCE_FITS").message_lines(2);
    let revision = h.app().revision.saturating_add(1);
    h.app_mut().revision = revision;
    h.draw();
    assert!(!h.text().contains("MESSAGE_SOURCE_TAIL"));
    assert!(
        h.area_of_part(OWNER, cut(NoticeSource::Message, 0))
            .is_none()
    );

    h.app_mut().notice = Notice::warning(
        "NEW_SOURCE_START one two three four five six seven eight nine ten eleven twelve NEW_SOURCE_TAIL",
    )
    .message_lines(1);
    let revision = h.app().revision.saturating_add(1);
    h.app_mut().revision = revision;
    h.draw();
    assert!(!h.text().contains("NEW_SOURCE_TAIL"));
    assert!(
        h.area_of_part(OWNER, cut(NoticeSource::Message, 0))
            .is_some()
    );
    let _ = h.tick();
    let _ = h.advance(Duration::from_millis(299));
    assert!(!h.text().contains("NEW_SOURCE_TAIL"));
    let _ = h.advance(Duration::from_millis(1));
    assert!(h.text().contains("NEW_SOURCE_TAIL"));
}

#[test]
fn owner_reset_cannot_be_revived_by_an_old_repaint_deadline() {
    let mut h = harness(long_notice());
    hover_message(&mut h);
    let _ = h.advance(Duration::from_millis(100));
    h.app_mut().reveal.borrow_mut().reset();
    h.draw();
    assert!(!h.text().contains("MESSAGE_SOURCE_TAIL"));
    assert!(
        h.area_of_part(
            OWNER.sub("notice-reveal-scroll"),
            PartRef::of(Part::CONTAINER)
        )
        .is_none()
    );

    // The old request wakes at the original 300 ms mark. A stationary,
    // currently hovered row starts a new 300 ms interval from that update.
    let _ = h.advance(Duration::from_millis(200));
    assert!(!h.text().contains("MESSAGE_SOURCE_TAIL"));
    let _ = h.advance(Duration::from_millis(299));
    assert!(!h.text().contains("MESSAGE_SOURCE_TAIL"));
    let _ = h.advance(Duration::from_millis(1));
    assert!(h.text().contains("MESSAGE_SOURCE_TAIL"));
}

#[test]
fn reveal_scroll_reaches_full_source_tail_and_forwards_at_both_ends() {
    let mut h = harness(Notice::warning(TALL_MESSAGE).message_lines(1));
    h.app_mut().register_page_scroll = true;
    h.draw();
    hover_message(&mut h);
    let _ = h.advance(Duration::from_millis(300));
    assert!(!h.text().contains("MESSAGE_SOURCE_TAIL"));
    let scroll_area = h
        .area_of_part(
            OWNER.sub("notice-reveal-scroll"),
            PartRef::of(Part::CONTAINER),
        )
        .expect("overflowing reveal registers its passive scroll target");

    let _ = h.wheel(termrock_runtime::Axis::V, 1, scroll_area.x, scroll_area.y);
    let _ = h.tick();
    assert!(
        h.row(scroll_area.y).contains("ROW03"),
        "one passive wheel detent advances exactly three source rows: {}",
        h.row(scroll_area.y)
    );

    for _ in 0..30 {
        let _ = h.wheel(termrock_runtime::Axis::V, 1, scroll_area.x, scroll_area.y);
        let _ = h.tick();
    }
    assert!(h.text().contains("MESSAGE_SOURCE_TAIL"));
    let after_tail_forward = h.app().page_scroll.offset();
    assert!(after_tail_forward > 0, "wheel at reveal tail forwards");

    for _ in 0..30 {
        let _ = h.wheel(termrock_runtime::Axis::V, -1, scroll_area.x, scroll_area.y);
        let _ = h.tick();
    }
    assert!(
        h.app().page_scroll.offset() < after_tail_forward,
        "wheel at reveal head forwards"
    );
}

#[test]
fn first_click_and_focused_text_input_keep_their_normal_ownership() {
    let mut h = harness(long_notice());
    let source = h
        .area_of_part(OWNER, cut(NoticeSource::Message, 0))
        .expect("source row");
    h.app_mut().button_area = source;
    h.draw();
    hover_message(&mut h);
    let _ = h.advance(Duration::from_millis(300));
    assert!(h.text().contains("MESSAGE_SOURCE_TAIL"));

    let _ = h.click(source.x.saturating_add(1), source.y);
    assert_eq!(h.app().activations, 1);

    let mut h = harness(long_notice());
    h.app_mut().input.begin("");
    h.runtime_mut().set_focus(Some(INPUT));
    h.draw();
    hover_message(&mut h);
    let _ = h.advance(Duration::from_millis(300));
    assert!(h.text().contains("MESSAGE_SOURCE_TAIL"));
    assert_eq!(h.focus(), Some(INPUT));
    let _ = h.type_str("typed");
    assert_eq!(h.app().input.draft_text(), Some("typed"));
    assert!(!h.text().contains("MESSAGE_SOURCE_TAIL"));
    assert_eq!(h.focus(), Some(INPUT));
    let _ = h.key(termrock_runtime::KeyCode::Tab);
    assert_eq!(h.focus(), Some(BUTTON));
    let _ = h.key(termrock_runtime::KeyCode::Esc);
    assert_eq!(h.focus(), Some(BUTTON));
    assert!(!h.runtime().is_open(OWNER));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn keyboard_and_modal_ownership_suppress_the_passive_reveal_without_capturing_it() {
    let mut h = harness(long_notice());
    h.app_mut().input.begin("");
    h.runtime_mut().set_focus(Some(INPUT));
    h.draw();
    hover_message(&mut h);
    let _ = h.advance(Duration::from_millis(300));
    assert!(h.text().contains("MESSAGE_SOURCE_TAIL"));

    let _ = h.key(termrock_runtime::KeyCode::Tab);
    assert_eq!(h.focus(), Some(BUTTON));
    assert!(!h.text().contains("MESSAGE_SOURCE_TAIL"));
    assert!(!h.runtime().is_open(MODAL));

    let mut h = harness(long_notice());
    h.app_mut().input.begin("");
    h.runtime_mut().set_focus(Some(INPUT));
    h.draw();
    hover_message(&mut h);
    let _ = h.advance(Duration::from_millis(300));
    assert!(h.text().contains("MESSAGE_SOURCE_TAIL"));
    let _ = h.key(termrock_runtime::KeyCode::Esc);
    assert_eq!(h.focus(), Some(INPUT));
    assert!(!h.text().contains("MESSAGE_SOURCE_TAIL"));
    assert!(!h.runtime().is_open(MODAL));

    let mut h = harness(long_notice());
    hover_message(&mut h);
    let _ = h.advance(Duration::from_millis(300));
    assert!(h.text().contains("MESSAGE_SOURCE_TAIL"));
    h.app_mut().open_modal = true;
    let _ = h.tick();
    assert!(h.runtime().is_open(MODAL));
    assert!(!h.text().contains("MESSAGE_SOURCE_TAIL"));
    assert!(
        h.area_of_part(
            OWNER.sub("notice-reveal-scroll"),
            PartRef::of(Part::CONTAINER)
        )
        .is_none()
    );
    h.app_mut().open_modal = false;
    let _ = h.tick();
    let _ = h.tick();
    assert!(!h.runtime().is_open(MODAL));
    assert!(!h.text().contains("MESSAGE_SOURCE_TAIL"));
    let _ = h.advance(Duration::from_millis(299));
    assert!(!h.text().contains("MESSAGE_SOURCE_TAIL"));
    let _ = h.advance(Duration::from_millis(1));
    assert!(h.text().contains("MESSAGE_SOURCE_TAIL"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

    let mut h = harness(long_notice());
    hover_message(&mut h);
    let _ = h.advance(Duration::from_millis(300));
    assert!(h.text().contains("MESSAGE_SOURCE_TAIL"));
    h.app_mut().open_help = true;
    let _ = h.tick();
    assert!(h.runtime().is_open(HELP));
    assert!(!h.text().contains("MESSAGE_SOURCE_TAIL"));
    assert!(
        h.area_of_part(
            OWNER.sub("notice-reveal-scroll"),
            PartRef::of(Part::CONTAINER)
        )
        .is_none()
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn real_pointer_capture_and_reference_rendering_clear_reveal_state_and_registration() {
    let mut h = harness(long_notice());
    let target = h
        .area_of_part(OWNER, cut(NoticeSource::Message, 0))
        .expect("live cut row");
    h.app_mut().capture_area = Some(target);
    h.draw();
    let _ = h.mouse(
        termrock_runtime::MouseKind::Move,
        target.x.saturating_add(1),
        target.y,
    );
    let _ = h.advance(Duration::from_millis(300));
    assert!(h.text().contains("MESSAGE_SOURCE_TAIL"));

    let _ = h.mouse(
        termrock_runtime::MouseKind::Down,
        target.x.saturating_add(1),
        target.y,
    );
    assert_eq!(h.runtime().capture_owner(), Some(CAPTURE));
    assert!(!h.text().contains("MESSAGE_SOURCE_TAIL"));
    assert!(
        h.area_of_part(
            OWNER.sub("notice-reveal-scroll"),
            PartRef::of(Part::CONTAINER)
        )
        .is_none()
    );
    let _ = h.mouse(
        termrock_runtime::MouseKind::Up,
        target.x.saturating_add(1),
        target.y,
    );

    let mut h = harness(long_notice());
    hover_message(&mut h);
    let _ = h.advance(Duration::from_millis(300));
    assert!(h.text().contains("MESSAGE_SOURCE_TAIL"));
    h.app_mut().reference = true;
    h.draw();
    assert!(!h.text().contains("MESSAGE_SOURCE_TAIL"));
    assert!(
        h.area_of_part(OWNER, cut(NoticeSource::Message, 0))
            .is_none()
    );
    assert!(
        h.area_of_part(
            OWNER.sub("notice-reveal-scroll"),
            PartRef::of(Part::CONTAINER)
        )
        .is_none()
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn off_window_and_clipped_cut_rows_never_receive_hover_registration() {
    let notice = Notice::new("first second third fourth fifth").message_lines(1);
    let logical = Rect::new(10, 3, 30, 5);
    let window =
        VerticalWindow::new(Rect::new(10, 8, 30, 2), 5, 3).expect("valid lower two-row crop");
    let mut app = Page::new(notice);
    app.notice_area = logical;
    app.window = window;
    let mut h = Harness::new(app, Theme::junie(), 80, 24);
    assert!(
        h.area_of_part(OWNER, cut(NoticeSource::Message, 0))
            .is_none()
    );

    h.app_mut().window =
        VerticalWindow::new(Rect::new(10, 3, 30, 1), 5, 0).expect("valid clipped first source row");
    h.draw();
    let target = h
        .area_of_part(OWNER, cut(NoticeSource::Message, 0))
        .expect("visible cut row remains addressable");
    assert_eq!(target, Rect::new(12, 3, 28, 1));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

    let fitting = Notice::new("fits in the original logical body");
    let mut app = Page::new(fitting);
    app.notice_area = logical;
    app.window = VerticalWindow::new(Rect::new(10, 3, 30, 1), 5, 0)
        .expect("physical crop of a fitting source");
    let h = Harness::new(app, Theme::junie(), 80, 24);
    assert!(
        h.area_of_part(OWNER, cut(NoticeSource::Message, 0))
            .is_none()
    );
}

#[test]
fn resized_narrow_screen_clips_source_targets_and_keeps_pane_inside_the_viewport() {
    let mut app = Page::new(Notice::warning(TALL_MESSAGE).message_lines(1));
    let logical = Rect::new(5, 5, 30, 5);
    app.notice_area = logical;
    app.window = VerticalWindow::full(logical);
    let mut h = Harness::new(app, Theme::junie(), 40, 20);
    let _ = h.resize(14, 10);
    let target = h
        .area_of_part(OWNER, cut(NoticeSource::Message, 0))
        .expect("the visible part of the source row remains a target");
    assert_eq!(target, Rect::new(7, 5, 7, 1));

    let _ = h.mouse(
        termrock_runtime::MouseKind::Move,
        target.right().saturating_sub(1),
        target.y,
    );
    let _ = h.advance(Duration::from_millis(300));
    let pane = h
        .area_of_part(
            OWNER.sub("notice-reveal-scroll"),
            PartRef::of(Part::CONTAINER),
        )
        .expect("tall source has a clipped passive scroll area");
    assert_eq!(pane.intersection(Rect::new(0, 0, 14, 10)), pane);
    assert!(pane.contains(ratatui_core::layout::Position::new(target.x, target.y)));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
