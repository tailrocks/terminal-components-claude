use std::time::Duration;

use showcase_ui::{App, PageId};
use termrock::{
    Axis, FeedbackClock, Id, ItemKey, KeyCode, Part, PartRef, Rect, SimulationMoment, Theme,
};
use termrock_test_support::Harness;

const NAV: Id = Id::root("showcase_ui::app::navigation");
const SUMMARY: Id = Id::root("showcase_ui::pages::forms::forms.summary");
const DETAILS: Id = Id::root("showcase_ui::pages::forms::forms.details");
const REVIEWER: Id = Id::root("showcase_ui::pages::forms::forms.reviewer");
const MODE: Id = Id::root("showcase_ui::pages::forms::forms.mode");
const RUN_TESTS: Id = Id::root("showcase_ui::pages::forms::forms.run_tests");
const OPEN_PR: Id = Id::root("showcase_ui::pages::forms::forms.open_pr");
const CONFIRM: Id = Id::root("showcase_ui::pages::forms::forms.confirm");
const AUTO_APPROVE: Id = Id::root("showcase_ui::pages::forms::forms.auto_approve");
const NOTIFY: Id = Id::root("showcase_ui::pages::forms::forms.notify");
const SAVE: Id = Id::root("showcase_ui::pages::forms::forms.save");
const RESET: Id = Id::root("showcase_ui::pages::forms::forms.reset");
const TASK_CARD: Id = Id::root("showcase_ui::pages::forms::forms.new_task");
const TASK_SCROLL: Id = Id::root("showcase_ui::pages::forms::forms.new_task.scroll");
const FLOW: Id = Id::root("showcase_ui::pages::forms::forms.flow");
const DEPLOY_STAGE: Id = Id::root("showcase_ui::pages::forms::forms.deploy.stage");
const DEFAULT_DETAILS: &str = "Describe the change and its rollback plan.";
const TASK_OWNERS: [Id; 11] = [
    SUMMARY,
    DETAILS,
    REVIEWER,
    MODE,
    RUN_TESTS,
    OPEN_PR,
    CONFIRM,
    AUTO_APPROVE,
    NOTIFY,
    SAVE,
    RESET,
];

fn harness(width: u16, height: u16) -> Harness<App> {
    Harness::new_with_feedback_clock(
        App::with_page(PageId::Forms),
        Theme::junie(),
        width,
        height,
        FeedbackClock::Simulation {
            initial: SimulationMoment::ZERO,
        },
    )
}

fn enter_forms(h: &mut Harness<App>) {
    assert_eq!(
        h.focus(),
        Some(NAV),
        "the app starts at real navigation focus"
    );
    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("New task"), "{}", h.text());
}

fn reachable(h: &Harness<App>, owner: Id) -> bool {
    h.ring().reachable().any(|entry| entry.id == owner)
}

fn assert_task_owner_enrolled_once(h: &Harness<App>, owner: Id) {
    let count = h
        .ring()
        .entries()
        .iter()
        .filter(|entry| entry.id == owner)
        .count();
    assert_eq!(
        count, 1,
        "enabled Task-card owner {owner:?} has one native or focus-only entry"
    );
}

fn assert_unique_reachable_owners(h: &Harness<App>) {
    let mut seen = Vec::new();
    for entry in h.ring().reachable() {
        assert!(
            !seen.contains(&entry.id),
            "duplicate reachable owner {:?}",
            entry.id
        );
        seen.push(entry.id);
    }
}

fn inside(outer: Rect, inner: Rect) -> bool {
    !inner.is_empty()
        && inner.x >= outer.x
        && inner.y >= outer.y
        && inner.right() <= outer.right()
        && inner.bottom() <= outer.bottom()
}

fn task_scroll_viewport(h: &Harness<App>) -> Rect {
    h.area_of_part(TASK_SCROLL, PartRef::of(Part::CONTAINER))
        .or_else(|| h.area_of(TASK_SCROLL))
        .expect("short Task card registers its native scroll viewport")
}

fn task_action_footer(h: &Harness<App>) -> Rect {
    let viewport = task_scroll_viewport(h);
    Rect {
        y: viewport.bottom(),
        height: 1,
        ..viewport
    }
}

fn assert_owner_visible(h: &Harness<App>, owner: Id) -> Rect {
    let area = h
        .area_of(owner)
        .unwrap_or_else(|| panic!("focused Task-card owner {owner:?} has no native area"));
    assert!(!area.is_empty(), "focused owner {owner:?} has empty area");
    assert!(
        inside(*h.buffer().area(), area),
        "owner {owner:?} is outside the frame: {area:?}"
    );
    if let Some(viewport) = h
        .area_of_part(TASK_SCROLL, PartRef::of(Part::CONTAINER))
        .or_else(|| h.area_of(TASK_SCROLL))
    {
        let owner_viewport = if [SAVE, RESET].contains(&owner) {
            task_action_footer(h)
        } else {
            viewport
        };
        assert!(
            inside(owner_viewport, area),
            "owner {owner:?} is outside its Task-card viewport: {area:?}"
        );
    } else {
        let nav = h.area_of(NAV).expect("the shell navigation has an area");
        assert!(
            area.x >= nav.right(),
            "page owner {owner:?} escaped into the sidebar: {area:?}"
        );
    }
    area
}

fn focus_visible(h: &mut Harness<App>, owner: Id) -> Rect {
    assert!(
        h.tab_to(owner),
        "ordinary Tab traversal cannot reach {owner:?}"
    );
    assert_eq!(h.focus(), Some(owner));
    assert!(h.state_of(owner).contains(termrock::StateFlags::FOCUSED));
    let area = assert_owner_visible(h, owner);
    assert_unique_reachable_owners(h);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    area
}

fn rect_cells(h: &Harness<App>, rect: Rect) -> Vec<String> {
    (rect.y..rect.bottom())
        .flat_map(|y| (rect.x..rect.right()).map(move |x| format!("{x},{y}:{:?}", h.cell(x, y))))
        .collect()
}

fn area_text(h: &Harness<App>, rect: Rect) -> String {
    (rect.y..rect.bottom())
        .map(|y| {
            (rect.x..rect.right())
                .map(|x| h.cell(x, y).symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn replace_input(h: &mut Harness<App>, owner: Id, value: &str) {
    assert!(
        h.tab_to(owner),
        "ordinary Tab traversal cannot reach {owner:?}"
    );
    let _ = h.key(KeyCode::Enter);
    let _ = h.ctrl('l');
    let _ = h.type_str(value);
    let _ = h.key(KeyCode::Enter);
}

fn replace_short_input(h: &mut Harness<App>, owner: Id, value: &str) {
    assert!(
        h.tab_to(owner),
        "ordinary Tab traversal cannot reach {owner:?}"
    );
    let _ = h.ctrl('l');
    let _ = h.type_str(value);
    let _ = h.key(KeyCode::Enter);
    assert!(
        h.text().contains(value),
        "the focused field accepts {value:?}"
    );
}

fn rects_overlap(a: Rect, b: Rect) -> bool {
    a.x < b.right() && b.x < a.right() && a.y < b.bottom() && b.y < a.bottom()
}

fn assert_task_geometry(h: &Harness<App>) {
    let viewport = task_scroll_viewport(h);
    let footer = task_action_footer(h);
    let card = h.area_of(TASK_CARD).expect("the Task card is visible");
    assert!(inside(card, footer), "the action footer stays in the card");
    for region in h.runtime().registry().regions() {
        if TASK_OWNERS.contains(&region.owner) {
            let owner_viewport = if [SAVE, RESET].contains(&region.owner) {
                footer
            } else {
                viewport
            };
            assert!(
                inside(owner_viewport, region.area),
                "Task-card region escaped its assigned viewport: {region:?}"
            );
        }
    }
    for action in [SAVE, RESET] {
        let Some(action_area) = h.area_of(action) else {
            continue;
        };
        assert_eq!(
            action_area.y, footer.y,
            "action {action:?} uses the footer row"
        );
        assert_eq!(action_area.height, 1, "action {action:?} uses one row");
        assert!(inside(footer, action_area));
        for field in [
            SUMMARY,
            DETAILS,
            REVIEWER,
            MODE,
            RUN_TESTS,
            OPEN_PR,
            CONFIRM,
            AUTO_APPROVE,
            NOTIFY,
        ] {
            if let Some(field_area) = h.area_of(field) {
                assert!(
                    !rects_overlap(action_area, field_area),
                    "action {action:?} overlaps field {field:?}"
                );
            }
        }
    }
}

#[test]
fn minimum_route_reaches_confirmed_create_and_done_through_normal_focus() {
    let mut h = harness(72, 20);
    enter_forms(&mut h);
    assert!(
        task_scroll_viewport(&h).height >= 6,
        "the supported minimum fits the largest six-row Task-card section"
    );

    for owner in [
        SUMMARY,
        DETAILS,
        REVIEWER,
        MODE,
        RUN_TESTS,
        OPEN_PR,
        CONFIRM,
        AUTO_APPROVE,
        RESET,
    ] {
        assert_task_owner_enrolled_once(&h, owner);
    }

    assert!(!reachable(&h, NOTIFY), "disabled Notify is not tabbable");
    assert!(
        !reachable(&h, SAVE),
        "unchecked confirmation disables Create"
    );
    assert!(
        reachable(&h, RESET),
        "Reset remains a real reachable action"
    );

    focus_visible(&mut h, SUMMARY);
    let _ = h.type_str("Fix");
    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("Fix"), "short valid Summary is accepted");

    for owner in [DETAILS, REVIEWER, MODE, RUN_TESTS, OPEN_PR, CONFIRM] {
        focus_visible(&mut h, owner);
    }

    let confirm_area = assert_owner_visible(&h, CONFIRM);
    let before_marker = h
        .cell(confirm_area.x.saturating_add(2), confirm_area.y)
        .symbol()
        .to_owned();
    let _ = h.key(KeyCode::Char(' '));
    assert!(reachable(&h, SAVE), "checked confirmation enables Create");
    assert_task_owner_enrolled_once(&h, SAVE);
    let after_marker = h
        .cell(confirm_area.x.saturating_add(2), confirm_area.y)
        .symbol()
        .to_owned();
    assert_ne!(
        before_marker, after_marker,
        "native checkbox marker changes"
    );

    let save_area = focus_visible(&mut h, SAVE);
    assert!(inside(task_action_footer(&h), save_area));
    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("Creating task…"), "{}", h.text());
    let busy_start = h.now();
    for _ in 0..23 {
        let _ = h.advance(Duration::from_millis(80));
    }
    assert_eq!(
        h.now(),
        busy_start.saturating_add(Duration::from_millis(23 * 80)),
        "the admitted task deadline advances in 23 exact 80 ms steps"
    );
    assert!(h.text().contains("Task created ✓"), "{}", h.text());
    assert_task_geometry(&h);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn minimum_backtab_mouse_activation_and_reset_cancel_busy_work() {
    let mut h = harness(72, 20);
    enter_forms(&mut h);
    replace_short_input(&mut h, SUMMARY, "Fix");

    let confirm_area = focus_visible(&mut h, CONFIRM);
    let before_marker = h
        .cell(confirm_area.x.saturating_add(2), confirm_area.y)
        .symbol()
        .to_owned();
    let _ = h.click_id(CONFIRM);
    let after_marker = h
        .cell(confirm_area.x.saturating_add(2), confirm_area.y)
        .symbol()
        .to_owned();
    assert_ne!(
        before_marker, after_marker,
        "mouse toggles the native checkbox"
    );
    assert!(reachable(&h, SAVE));

    assert!(focus_visible(&mut h, SAVE).width > 0);
    let _ = h.key(KeyCode::BackTab);
    assert_eq!(h.focus(), Some(AUTO_APPROVE));
    assert_owner_visible(&h, AUTO_APPROVE);
    let _ = h.key(KeyCode::Tab);
    assert_eq!(h.focus(), Some(SAVE), "disabled Notify stays skipped");
    assert_owner_visible(&h, SAVE);

    let _ = h.click_id(SAVE);
    assert!(h.text().contains("Creating task…"), "{}", h.text());
    assert!(focus_visible(&mut h, RESET).width > 0);
    let _ = h.click_id(RESET);
    assert_eq!(h.focus(), Some(SUMMARY));
    assert!(h.text().contains("Form reset"), "{}", h.text());
    for _ in 0..23 {
        let _ = h.advance(Duration::from_millis(80));
    }
    assert!(!h.text().contains("Creating task…"));
    assert!(!h.text().contains("Task created ✓"));
    assert!(h.text().contains("Short imperative summary"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn minimum_validation_reveals_summary_details_and_reviewer_in_order() {
    let mut h = harness(72, 20);
    enter_forms(&mut h);

    let _ = h.ctrl('s');
    assert_eq!(h.focus(), Some(SUMMARY));
    assert_owner_visible(&h, SUMMARY);
    assert!(h.text().contains("Required: summary"), "{}", h.text());

    replace_short_input(&mut h, SUMMARY, "Fix");
    let details = focus_visible(&mut h, DETAILS);
    let details_text = h
        .area_of_part(DETAILS, PartRef::of(Part::TEXT))
        .unwrap_or(details);
    let _ = h.click(details_text.x, details_text.y);
    let _ = h.ctrl('l');
    let _ = h.key(KeyCode::Backspace);
    let _ = h.key(KeyCode::Esc);
    let _ = h.ctrl('s');
    assert_eq!(h.focus(), Some(DETAILS));
    assert_owner_visible(&h, DETAILS);

    let details_text = h
        .area_of_part(DETAILS, PartRef::of(Part::TEXT))
        .expect("the focused native Details text area remains visible");
    let _ = h.click(details_text.x, details_text.y);
    let _ = h.paste(DEFAULT_DETAILS);
    let _ = h.key(KeyCode::Esc);

    let reviewer = focus_visible(&mut h, REVIEWER);
    let reviewer_text = h
        .area_of_part(REVIEWER, PartRef::of(Part::TEXT))
        .unwrap_or(reviewer);
    let _ = h.click(reviewer_text.x, reviewer_text.y);
    let _ = h.ctrl('l');
    let _ = h.type_str("not-an-address");
    let _ = h.key(KeyCode::Enter);
    let _ = h.ctrl('s');
    assert_eq!(h.focus(), Some(REVIEWER));
    assert_owner_visible(&h, REVIEWER);
    assert!(h.text().contains("valid email"), "{}", h.text());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn native_wheel_preserves_task_scroll_and_repaint_does_not_snap_back() {
    let mut h = harness(72, 20);
    enter_forms(&mut h);
    focus_visible(&mut h, CONFIRM);

    let viewport = task_scroll_viewport(&h);
    let thumb_before = h
        .area_of_part(TASK_SCROLL, PartRef::of(Part::THUMB))
        .expect("overflow shows the native Task-card scrollbar thumb");
    let _ = h.wheel(
        Axis::V,
        1,
        viewport.x.saturating_add(1),
        viewport.y.saturating_add(1),
    );
    let thumb_after_wheel = h
        .area_of_part(TASK_SCROLL, PartRef::of(Part::THUMB))
        .expect("Task-card thumb remains registered after wheel input");
    assert_ne!(thumb_before, thumb_after_wheel, "wheel moves the Task card");
    h.draw();
    assert_eq!(
        h.area_of_part(TASK_SCROLL, PartRef::of(Part::THUMB)),
        Some(thumb_after_wheel),
        "a repaint keeps deliberate wheel movement"
    );

    focus_visible(&mut h, SUMMARY);
    assert!(h.text().contains("Task"), "focus movement reveals Summary");
    assert_task_geometry(&h);

    let details = focus_visible(&mut h, DETAILS);
    let details_text = h
        .area_of_part(DETAILS, PartRef::of(Part::TEXT))
        .unwrap_or(details);
    let _ = h.click(details_text.x, details_text.y);
    let _ = h.ctrl('l');
    let long_details = (1..=12)
        .map(|line| format!("change line {line:02}"))
        .collect::<Vec<_>>()
        .join("\n");
    let _ = h.paste(&long_details);
    let _ = h.key(KeyCode::Esc);

    let text_area = h
        .area_of_part(DETAILS, PartRef::of(Part::TEXT))
        .expect("the real Details text clip is registered");
    let outer_thumb_before = h.area_of_part(TASK_SCROLL, PartRef::of(Part::THUMB));
    let inner_thumb_before = h.area_of_part(DETAILS, PartRef::of(Part::THUMB));
    let visible_before = area_text(&h, text_area);
    let _ = h.wheel(
        Axis::V,
        -2,
        text_area.x.saturating_add(1),
        text_area.y.saturating_add(1),
    );
    let text_area_after = h
        .area_of_part(DETAILS, PartRef::of(Part::TEXT))
        .expect("Details text clip remains registered after wheel input");
    assert_ne!(area_text(&h, text_area_after), visible_before);
    assert_ne!(
        h.area_of_part(DETAILS, PartRef::of(Part::THUMB)),
        inner_thumb_before,
        "the nested TextArea owns wheel scrolling"
    );
    assert_eq!(
        h.area_of_part(TASK_SCROLL, PartRef::of(Part::THUMB)),
        outer_thumb_before,
        "wheel over Details does not move the Task-card ScrollRegion"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn resize_reset_and_too_small_keep_task_card_isolated() {
    let mut h = harness(120, 40);
    enter_forms(&mut h);

    replace_short_input(&mut h, SUMMARY, "Fix");
    assert!(h.tab_to(MODE));
    let _ = h.key(KeyCode::Down);
    assert!(h.tab_to(RUN_TESTS));
    let _ = h.key(KeyCode::Char(' '));
    assert!(h.tab_to(OPEN_PR));
    let _ = h.key(KeyCode::Char(' '));
    assert!(h.tab_to(AUTO_APPROVE));
    let _ = h.key(KeyCode::Char(' '));
    assert!(h.tab_to(CONFIRM));
    let _ = h.key(KeyCode::Char(' '));

    let _ = focus_visible(&mut h, DEPLOY_STAGE);
    let _ = h.click_part(DEPLOY_STAGE, PartRef::of(Part::FIELD));
    assert!(h.is_open(DEPLOY_STAGE), "native Deploy select opens");
    let _ = h.click_part(DEPLOY_STAGE, PartRef::item(Part::ROW, ItemKey::index(1)));
    assert!(h.text().contains("Production"), "{}", h.text());
    let confirm_step = ItemKey::index(2);
    let _ = h.click_part(FLOW, PartRef::item(Part::LABEL, confirm_step));
    assert!(focus_visible(&mut h, CONFIRM).width > 0);
    let flow_label = h
        .area_of_part(FLOW, PartRef::item(Part::LABEL, confirm_step))
        .expect("the native Wizard Confirm step remains addressable");
    let flow_before_reset = rect_cells(&h, flow_label);
    let deploy_before_reset = rect_cells(
        &h,
        h.area_of(DEPLOY_STAGE)
            .expect("the selected Deploy stage has a native owner area"),
    );

    for (width, height) in [(80, 24), (100, 30), (72, 20)] {
        let _ = h.resize(width, height);
        assert_eq!(h.focus(), Some(CONFIRM));
        assert_owner_visible(&h, CONFIRM);
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }

    let viewport = task_scroll_viewport(&h);
    let thumb_before_resize_wheel = h
        .area_of_part(TASK_SCROLL, PartRef::of(Part::THUMB))
        .expect("resized short card retains its native scrollbar");
    let _ = h.wheel(
        Axis::V,
        1,
        viewport.x.saturating_add(1),
        viewport.y.saturating_add(1),
    );
    let thumb_after_resize_wheel = h
        .area_of_part(TASK_SCROLL, PartRef::of(Part::THUMB))
        .expect("Task-card thumb remains registered after resize and wheel");
    assert_ne!(thumb_before_resize_wheel, thumb_after_resize_wheel);
    h.draw();
    assert_eq!(
        h.area_of_part(TASK_SCROLL, PartRef::of(Part::THUMB)),
        Some(thumb_after_resize_wheel),
        "a deliberate wheel after resize is not replaced by a queued focus projection"
    );

    assert!(focus_visible(&mut h, RESET).width > 0);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.focus(), Some(SUMMARY));
    assert!(h.text().contains("Short imperative summary"));
    assert!(!reachable(&h, NOTIFY));
    assert!(!reachable(&h, SAVE), "Reset clears confirmation");

    let mut defaults = harness(72, 20);
    enter_forms(&mut defaults);
    for owner in [
        SUMMARY,
        DETAILS,
        MODE,
        RUN_TESTS,
        OPEN_PR,
        CONFIRM,
        AUTO_APPROVE,
    ] {
        focus_visible(&mut h, owner);
        focus_visible(&mut defaults, owner);
        assert_eq!(
            rect_cells(&h, assert_owner_visible(&h, owner)),
            rect_cells(&defaults, assert_owner_visible(&defaults, owner)),
            "Reset restores default native cells for {owner:?}"
        );
    }

    let _ = h.resize(120, 40);
    assert_eq!(h.focus(), Some(AUTO_APPROVE));
    let flow_after_reset = h
        .area_of_part(FLOW, PartRef::item(Part::LABEL, confirm_step))
        .expect("Wizard step remains present after Task reset");
    assert_eq!(rect_cells(&h, flow_after_reset), flow_before_reset);
    let deploy_after_reset = h
        .area_of(DEPLOY_STAGE)
        .expect("Deploy stage remains present after Task reset");
    assert_eq!(rect_cells(&h, deploy_after_reset), deploy_before_reset);
    assert!(
        h.text().contains("Production"),
        "Deploy data survives Reset"
    );

    for (width, height) in [(71, 20), (72, 19)] {
        let _ = h.resize(width, height);
        for owner in TASK_OWNERS {
            assert_eq!(
                h.area_of(owner),
                None,
                "unsupported {width}x{height} view registers no Task control {owner:?}"
            );
            assert!(
                !reachable(&h, owner),
                "unsupported {width}x{height} view exposes no Task focus stop {owner:?}"
            );
        }
    }
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn full_fit_route_keeps_native_owners_and_submission() {
    let mut h = harness(120, 40);
    enter_forms(&mut h);

    let _ = focus_visible(&mut h, SUMMARY);
    replace_input(&mut h, SUMMARY, "Fix task card");

    let _ = focus_visible(&mut h, DETAILS);
    let _ = focus_visible(&mut h, REVIEWER);
    let reviewer = h
        .area_of_part(REVIEWER, PartRef::of(Part::TEXT))
        .expect("full-fit Reviewer text area is visible");
    let _ = h.click(reviewer.x, reviewer.y);
    let _ = h.ctrl('l');
    let _ = h.type_str("not-an-address");
    let _ = h.key(KeyCode::Enter);

    let _ = focus_visible(&mut h, MODE);
    let _ = h.key(KeyCode::Up);
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Down);
    let _ = focus_visible(&mut h, RUN_TESTS);
    let _ = h.key(KeyCode::Char(' '));
    let _ = focus_visible(&mut h, OPEN_PR);
    let _ = h.key(KeyCode::Char(' '));
    let _ = focus_visible(&mut h, AUTO_APPROVE);
    let _ = h.key(KeyCode::Char(' '));
    let _ = focus_visible(&mut h, CONFIRM);
    let _ = h.key(KeyCode::Char(' '));

    let _ = focus_visible(&mut h, RESET);
    let _ = h.key(KeyCode::Enter);
    let _ = h.ctrl('s');
    assert_eq!(h.focus(), Some(SUMMARY));
    assert_owner_visible(&h, SUMMARY);
    replace_input(&mut h, SUMMARY, "Fix");
    let _ = focus_visible(&mut h, CONFIRM);
    let _ = h.key(KeyCode::Char(' '));
    let _ = focus_visible(&mut h, SAVE);
    let _ = h.key(KeyCode::Enter);
    for _ in 0..23 {
        let _ = h.advance(Duration::from_millis(80));
    }
    assert!(h.text().contains("Task created ✓"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
