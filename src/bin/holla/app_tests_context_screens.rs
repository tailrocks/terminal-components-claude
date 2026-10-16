//! Row context menus on the plan, disk and activity screens, through the
//! real App (hit regions, focus and rendered cells).

use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;

use crate::app_tests::H;
use crate::app_tests_context::{
    assert_at_pointer, assert_below_row, choose, menu_area, right_click, shift_f10,
};
use crate::domain::plan::{PlanPhase, StepState};
use crate::scenario::{Motion, Scenario};
use crate::screens::plan;

fn until<F: Fn(&H) -> bool>(h: &mut H, max_ticks: usize, f: F) -> bool {
    for _ in 0..max_ticks {
        if f(h) {
            return true;
        }
        h.ticks(1);
    }
    f(h)
}

fn row(h: &H, id: junie_tui::core::id::WidgetId) -> Rect {
    h.app
        .hits
        .area_of(id)
        .unwrap_or_else(|| panic!("row {id:?} not drawn\n{}", h.text()))
}

// ------------------------------------------------------------------ plan

fn review() -> H {
    H::new(Scenario::UpgradePlan, Motion::Reduced, 0, 120, 40)
}

#[test]
fn a_plan_step_right_click_opens_its_menu_at_the_pointer_and_toggles_it() {
    let mut h = review();
    let r = row(&h, plan::OUTLINE.child(5));
    let (x, y) = (r.x + 10, r.y);
    right_click(&mut h, x, y);
    assert_eq!(h.app.focus.current(), Some(plan::STEP_MENU));
    assert_at_pointer(&h, plan::STEP_MENU, x, y);
    let t = h.text();
    assert!(t.contains("06 "), "titled by the step\n{t}");
    assert!(
        t.contains("Exclude step") && t.contains("Discard plan"),
        "{t}"
    );
    assert!(h.app.world.plans[0].steps[5].included);
    choose(&mut h, "Exclude step");
    assert!(h.app.modals.is_empty());
    assert!(!h.app.world.plans[0].steps[5].included, "excluded");
    assert_eq!(h.app.focus.current(), Some(plan::OUTLINE));

    // a required step offers the toggle disabled, with its reason
    let r0 = row(&h, plan::OUTLINE.child(0));
    right_click(&mut h, r0.x + 10, r0.y);
    assert!(h.text().contains("Required by the plan"), "{}", h.text());
    choose(&mut h, "Exclude step");
    assert!(!h.app.modals.is_empty(), "a disabled row does not choose");
    assert!(h.app.world.plans[0].steps[0].included);
    h.key(KeyCode::Esc);
    assert!(h.app.modals.is_empty());
}

#[test]
fn shift_f10_opens_the_step_menu_below_the_cursor_row() {
    let mut h = review();
    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Char(' '));
    assert!(!h.app.world.plans[0].steps[5].included);
    let r = row(&h, plan::OUTLINE.child(5));
    shift_f10(&mut h);
    assert_eq!(h.app.focus.current(), Some(plan::STEP_MENU), "{}", h.text());
    assert_below_row(&h, plan::STEP_MENU, r.y);
    choose(&mut h, "Undo last change");
    assert!(h.app.world.plans[0].steps[5].included, "undone");
}

#[test]
fn a_right_click_outside_the_step_menu_reopens_it_on_the_other_step() {
    let mut h = review();
    let r5 = row(&h, plan::OUTLINE.child(5));
    let r1 = row(&h, plan::OUTLINE.child(1));
    right_click(&mut h, r5.x + 10, r5.y);
    let first = menu_area(&h, plan::STEP_MENU);
    assert!(
        r1.y < first.y || r1.y >= first.bottom(),
        "step 02 is clear of the menu"
    );
    right_click(&mut h, r1.x + 10, r1.y);
    assert_eq!(h.app.modals.len(), 1);
    assert_at_pointer(&h, plan::STEP_MENU, r1.x + 10, r1.y);
    assert!(h.text().contains("02 "), "{}", h.text());
    h.key(KeyCode::Esc);
    assert!(h.app.modals.is_empty());
}

#[test]
fn a_running_plan_step_menu_retries_a_failed_step() {
    let mut h = review();
    h.key(KeyCode::Char('c'));
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert!(until(&mut h, 800, |h| h.app.world.plans[0].phase
        == PlanPhase::Done));
    assert_eq!(h.app.world.plans[0].steps[5].state, StepState::Failed);
    // a succeeded step: retry and skip are offered disabled with reasons
    let r3 = row(&h, plan::OUTLINE.child(3));
    right_click(&mut h, r3.x + 10, r3.y);
    assert_at_pointer(&h, plan::STEP_MENU, r3.x + 10, r3.y);
    let t = h.text();
    assert!(t.contains("Only a failed step can be retried"), "{t}");
    assert!(t.contains("The plan is not running"), "{t}");
    h.key(KeyCode::Esc);
    let r5 = row(&h, plan::OUTLINE.child(5));
    right_click(&mut h, r5.x + 10, r5.y);
    choose(&mut h, "Retry step");
    assert!(h.app.modals.is_empty());
    assert_eq!(h.app.world.plans[0].steps[5].state, StepState::Ready);
    assert_eq!(h.app.world.plans[0].steps[3].state, StepState::Succeeded);
}
